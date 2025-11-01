use components::{docs::render_snippet, ThemeSwitch};
use designsystem::{install_defaults, IconName, ThemeRegistry};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Window, WindowBounds,
    WindowOptions,
};
use gpui_component::{
    accordion::Accordion,
    alert::{Alert, AlertVariant},
    button::{Button, ButtonVariants as _},
    group_box::GroupBox,
    icon::Icon,
    resizable::{h_resizable, resizable_panel},
    styled::{h_flex, v_flex, StyledExt as _},
    switch::Switch,
    text::{Text, TextView, TextViewStyle},
};
use platform::{bootstrap, ConfigStore, WorkspaceConfig};

fn main() {
    let app = install_defaults(Application::new());
    app.run(|cx| {
        gpui_component::init(cx);

        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        let config = bootstrap(cx, &store).expect("workspace configuration");

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1240.0), px(820.0)),
                    cx,
                ))),
                titlebar: Some("Markdown Notes".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("Markdown & Notes Workspace");
                cx.new(|_| MarkdownNotesApp::new(registry.clone(), store.clone(), config.clone()))
            },
        )
        .expect("markdown notes window");

        cx.activate(true);
    });
}

struct MarkdownNotesApp {
    theme_registry: ThemeRegistry,
    config_store: ConfigStore,
    workspace_config: WorkspaceConfig,
    active_document: Option<String>,
    buffer: String,
    recent_documents: Vec<String>,
    synthetic_counter: usize,
    status: Option<String>,
}

impl MarkdownNotesApp {
    fn new(theme_registry: ThemeRegistry, store: ConfigStore, mut config: WorkspaceConfig) -> Self {
        let recent = config.recent_workspaces.clone();
        if config.recent_workspaces.is_empty() {
            config.push_recent("welcome.md");
        }
        Self {
            theme_registry,
            config_store: store,
            workspace_config: config,
            active_document: None,
            buffer: "# Welcome to Markdown Notes\n\n- Use the buttons to insert headings and code fences.\n- Drop a `.md` file to load its contents.\n".into(),
            recent_documents: recent,
            synthetic_counter: 1,
            status: None,
        }
    }

    fn load_document(&mut self, name: String, contents: String, cx: &mut Context<Self>) {
        self.active_document = Some(name.clone());
        self.buffer = contents;
        self.workspace_config.push_recent(name.clone());
        self.recent_documents = self.workspace_config.recent_workspaces.clone();
        if let Err(err) = self.config_store.save(&self.workspace_config) {
            eprintln!("failed to persist notes config: {err}");
        }
        self.status = Some(format!("Loaded {name}"));
        cx.notify();
    }

    fn simulate_drop(&mut self, cx: &mut Context<Self>) {
        let name = format!("meeting-notes-{}.md", self.synthetic_counter);
        self.synthetic_counter += 1;
        let contents = format!(
            "# Sprint review\n\n## Decisions\n- Promote the virtualized explorer to beta.\n- Embed docs with `{name}` as the canonical slug.\n\n```rust\nfn summarize() {{ println!(\"ship it\"); }}\n```\n"
        );
        self.load_document(name, contents, cx);
    }

    fn insert_heading(&mut self, cx: &mut Context<Self>) {
        self.buffer.push_str("\n## New section\n\n");
        self.status = Some("Inserted heading".into());
        cx.notify();
    }

    fn insert_code_fence(&mut self, cx: &mut Context<Self>) {
        self.buffer.push_str("```rust\n// code goes here\n```\n");
        self.status = Some("Added code fence".into());
        cx.notify();
    }

    fn reset_document(&mut self, cx: &mut Context<Self>) {
        self.buffer.clear();
        self.status = Some("Cleared note".into());
        cx.notify();
    }

    fn render_editor(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let document_label = self
            .active_document
            .clone()
            .unwrap_or_else(|| "untitled.md".into());

        GroupBox::new()
            .title(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(Text::new(format!("Editing {document_label}")))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("insert-heading")
                                    .ghost()
                                    .label("Heading")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.insert_heading(cx);
                                    })),
                            )
                            .child(
                                Button::new("insert-fence")
                                    .ghost()
                                    .label("Code fence")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.insert_code_fence(cx);
                                    })),
                            )
                            .child(
                                Button::new("clear-note")
                                    .ghost()
                                    .label("Clear")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.reset_document(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(Text::new(
                        "The live editor stores markdown in memory; changes are persisted when you pick a different document.",
                    ))
                    .child(
                        div()
                            .bg(cx.theme().muted())
                            .p_3()
                            .rounded(cx.theme().radius)
                            .child(Text::new(self.buffer.clone())),
                    ),
            )
    }

    fn render_preview(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let markdown = self.buffer.clone();
        let view = TextView::markdown("markdown-preview", markdown, window, cx)
            .style(TextViewStyle::default());
        GroupBox::new().title(Text::new("Live preview")).child(view)
    }

    fn render_recent(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        GroupBox::new()
            .title(Text::new("Recent documents"))
            .child(
                v_flex()
                    .gap_2()
                    .children(self.recent_documents.iter().take(6).cloned().map(|name| {
                        Button::new(format!("recent-{name}"))
                            .label(name.clone())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let payload = format!("# {name}\n\nImported via MRU.");
                                this.load_document(name.clone(), payload, cx);
                            }))
                            .into_any_element()
                    })),
            )
    }

    fn render_documentation(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let usage = r#"let mut notes = MarkdownNotesApp::new(theme_registry.clone(), store.clone(), config.clone());
notes.load_document("retro.md".into(), contents, cx);"#;

        let integration = r#"platform::bootstrap(cx, &store)?;
cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Notes");
    cx.new(|_| MarkdownNotesApp::new(theme.clone(), store.clone(), config.clone()))
});"#;

        let gotchas = r#"// Persist frequently when wiring to real editors.
if note.bytes().len() > 500_000 {
    log::warn!("consider streaming to disk");
}"#;

        Accordion::new("notes-docs")
            .bordered(false)
            .item(|item| {
                item.title(Text::new("Component usage"))
                    .content(render_snippet("notes-usage", "Editor bootstrap", usage, window, cx))
            })
            .item(|item| {
                item.title(Text::new("Why it's useful"))
                    .content(Text::new(
                        "Pairing a split markdown view with a persistent MRU makes it trivial to jot postmortems while keeping tribal knowledge in sync.",
                    ))
            })
            .item(|item| {
                item.title(Text::new("Integration steps"))
                    .content(render_snippet(
                        "notes-integration",
                        "Window wiring",
                        integration,
                        window,
                        cx,
                    ))
            })
            .item(|item| {
                item.title(Text::new("Gotchas"))
                    .content(render_snippet(
                        "notes-gotchas",
                        "Large files",
                        gotchas,
                        window,
                        cx,
                    ))
            })
    }
}

impl gpui::Render for MarkdownNotesApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status_banner = self.status.clone().map(|message| {
            Alert::new("notes-status")
                .variant(AlertVariant::Success)
                .title(Text::new(message))
        });

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .p_6()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Text::new("Markdown workspace").size(22.0).font_weight_bold())
                            .child(Text::new(
                                "Capture meeting notes, architecture decisions, and runbooks side-by-side with live previews.",
                            )),
                    )
                    .child(ThemeSwitch::new("notes-theme", self.theme_registry.clone()).label("Theme")),
            )
            .when_some(status_banner, |col, banner| col.child(banner))
            .child(
                h_resizable("notes-split")
                    .panel(resizable_panel("editor", 0.48).child(self.render_editor(cx)))
                    .panel(resizable_panel("preview", 0.52).child(self.render_preview(window, cx)))
                    .min_panel_width(px(240.0)),
            )
            .child(
                h_flex()
                    .gap_4()
                    .child(
                        Button::new("simulate-drop")
                            .label("Simulate .md drop")
                            .icon(Icon::new(IconName::Workflow))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.simulate_drop(cx);
                            })),
                    )
                    .child(
                        Switch::new("auto-save")
                            .checked(true)
                            .label(Text::new("Persist recents"))
                            .tooltip("MRU persistence provided by crates/platform"),
                    ),
            )
            .child(self.render_recent(cx))
            .child(self.render_documentation(window, cx))
    }
}
