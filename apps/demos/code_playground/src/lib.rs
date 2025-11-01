use components::{docs::render_snippet, ThemeSwitch};
use designsystem::{install_defaults, IconName, ThemeRegistry};
use gpui::{
    prelude::*, px, size, App, Application, Bounds, Context, Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    accordion::Accordion,
    alert::{Alert, AlertVariant},
    button::{Button, ButtonVariants as _},
    group_box::GroupBox,
    icon::Icon,
    resizable::{h_resizable, resizable_panel},
    styled::{h_flex, v_flex, StyledExt as _},
    text::Text,
};
use platform::{bootstrap, ConfigStore};

pub fn run() {
    let app = install_defaults(Application::new());
    app.run(|cx| {
        gpui_component::init(cx);

        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        bootstrap(cx, &store).expect("workspace configuration");

        launch(cx, registry.clone());

        cx.activate(true);
    });
}

pub fn launch(cx: &mut App, registry: ThemeRegistry) {
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1280.0), px(820.0)),
                cx,
            ))),
            titlebar: Some("Code Playground".into()),
            ..Default::default()
        },
        move |window, cx| {
            window.set_title("Code Playground with LSP");
            cx.new(|_| CodePlaygroundApp::new(registry.clone()))
        },
    )
    .expect("code playground window");
}

struct Snippet {
    name: &'static str,
    code: &'static str,
    description: &'static str,
}

pub struct CodePlaygroundApp {
    theme_registry: ThemeRegistry,
    snippets: Vec<Snippet>,
    active: usize,
    diagnostics: Vec<String>,
    lsp_online: bool,
}

impl CodePlaygroundApp {
    pub fn new(theme_registry: ThemeRegistry) -> Self {
        let snippets = vec![
            Snippet {
                name: "hello.rs",
                code: "fn main() {\n    println!(\"hello playground\");\n}\n",
                description: "Minimal Rust entry point to verify the LSP handshake and formatting integration.",
            },
            Snippet {
                name: "state.rs",
                code: "pub struct Counter {\n    value: i32,\n}\n\nimpl Counter {\n    pub fn increment(&mut self) {\n        self.value += 1;\n    }\n}\n",
                description: "Example of tracking mutable state inside a gpui widget using derived listeners.",
            },
            Snippet {
                name: "async.rs",
                code: "use tokio::time::{sleep, Duration};\n\npub async fn fetch() -> anyhow::Result<()> {\n    sleep(Duration::from_millis(32)).await;\n    Ok(())\n}\n",
                description: "Demonstrates the async executor story and how diagnostics surface warnings for unused futures.",
            },
        ];

        Self {
            theme_registry,
            snippets,
            active: 0,
            diagnostics: vec![
                "warning: unused import: `tokio::time::sleep`".into(),
                "note: rust-analyzer suggests adding `#[allow(dead_code)]`".into(),
            ],
            lsp_online: true,
        }
    }

    fn active_snippet(&self) -> &Snippet {
        &self.snippets[self.active]
    }

    fn select_snippet(&mut self, index: usize, cx: &mut Context<Self>) {
        self.active = index;
        self.diagnostics = match self.snippets[index].name {
            "hello.rs" => vec![],
            "state.rs" => vec!["info: consider deriving Default for Counter".into()],
            "async.rs" => vec![
                "warning: unused import: `tokio::time::sleep`".into(),
                "note: futures must be awaited".into(),
            ],
            _ => vec![],
        };
        cx.notify();
    }

    fn toggle_lsp(&mut self, cx: &mut Context<Self>) {
        self.lsp_online = !self.lsp_online;
        if !self.lsp_online {
            self.diagnostics
                .push("error: language server connection lost".into());
        } else {
            self.diagnostics
                .retain(|msg| !msg.contains("connection lost"));
        }
        cx.notify();
    }

    fn render_file_explorer(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        GroupBox::new()
            .title(Text::new("Files"))
            .child(
                v_flex()
                    .gap_2()
                    .children(self.snippets.iter().enumerate().map(|(index, snippet)| {
                        let is_active = index == self.active;
                        Button::new(format!("file-{}", snippet.name))
                            .label(snippet.name)
                            .when(is_active, |btn| btn.primary())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select_snippet(index, cx);
                            }))
                            .into_any_element()
                    })),
            )
    }

    fn render_editor(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let snippet = self.active_snippet();
        let code = snippet.code;
        let snippet_id = format!("playground-{}", snippet.name);
        let view = render_snippet(snippet_id, snippet.name, code, window, cx);
        GroupBox::new()
            .title(Text::new("Editor"))
            .child(Text::new(snippet.description))
            .child(view)
    }

    fn render_diagnostics(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        GroupBox::new()
            .title(Text::new("Diagnostics"))
            .child(if self.diagnostics.is_empty() {
                Text::new("No issues – formatter and LSP are happy.")
                    .text_color(theme.muted_foreground)
                    .into_any_element()
            } else {
                v_flex()
                    .gap_1()
                    .children(self.diagnostics.iter().cloned().map(|msg| {
                        let color = if msg.starts_with("warning") {
                            theme.warning
                        } else if msg.starts_with("error") {
                            theme.destructive
                        } else {
                            theme.muted_foreground
                        };
                        Text::new(msg).text_color(color).into_any_element()
                    }))
                    .into_any_element()
            })
    }

    fn render_documentation(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let usage = r#"let mut editor = gpui_component::editor::CodeEditor::new(cx);
editor.set_language("rust");
editor.set_contents(snippet.code);"#;

        let integration = r#"let lsp = rust_analyzer::Client::spawn()?;
editor.connect_language_server(lsp);"#;

        let gotchas = r#"// Diagnostics stream is asynchronous.
if !lsp_online {
    return Err("lost connection".into());
}"#;

        Accordion::new("playground-docs")
            .bordered(false)
            .item(|item| {
                item.title(Text::new("Component usage"))
                    .content(render_snippet("playground-usage", "Embed editor", usage, window, cx))
            })
            .item(|item| {
                item.title(Text::new("Why it's useful"))
                    .content(Text::new(
                        "Bundle curated snippets, file navigation, and diagnostics to tell the story of how the GPUI editor layers on rust-analyzer.",
                    ))
            })
            .item(|item| {
                item.title(Text::new("Integration steps"))
                    .content(render_snippet(
                        "playground-integration",
                        "Hook up rust-analyzer",
                        integration,
                        window,
                        cx,
                    ))
            })
            .item(|item| {
                item.title(Text::new("Gotchas"))
                    .content(render_snippet(
                        "playground-gotchas",
                        "Async diagnostics",
                        gotchas,
                        window,
                        cx,
                    ))
            })
    }
}

impl gpui::Render for CodePlaygroundApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child(Text::new("Code Playground").size(22.0).font_weight_bold())
                            .child(Text::new(
                                "Experiment with gpui's code editor, mocked file explorer, and diagnostics hooked to rust-analyzer.",
                            )),
                    )
                    .child(ThemeSwitch::new("playground-theme", self.theme_registry.clone()).label("Theme")),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        Button::new("toggle-lsp")
                            .label(if self.lsp_online { "Disconnect LSP" } else { "Reconnect LSP" })
                            .icon(Icon::new(IconName::Workflow))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_lsp(cx);
                            })),
                    )
                    .child(
                        Alert::new("playground-tip")
                            .variant(AlertVariant::Info)
                            .title(Text::new("Limitations"))
                            .description(Text::new(
                                "The demo focuses on wiring patterns; actual editing is simulated with rendered snippets but the layout mirrors the production shell.",
                            )),
                    ),
            )
            .child(
                h_resizable("playground-split")
                    .panel(resizable_panel("files", 0.24).child(self.render_file_explorer(cx)))
                    .panel(resizable_panel("editor", 0.52).child(self.render_editor(window, cx)))
                    .panel(resizable_panel("diagnostics", 0.24).child(self.render_diagnostics(cx)))
                    .min_panel_width(px(220.0)),
            )
            .child(self.render_documentation(window, cx))
    }
}
