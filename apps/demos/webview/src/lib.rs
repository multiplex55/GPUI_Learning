#![allow(clippy::module_name_repetitions)]

#[cfg(not(feature = "webview"))]
use designsystem::ThemeRegistry;
#[cfg(not(feature = "webview"))]
use gpui::App;
#[cfg(not(feature = "webview"))]
pub fn run() {
    println!("Rebuild with --features webview to launch the embedded documentation view.");
}

#[cfg(feature = "webview")]
pub mod app {
    use components::{docs::render_snippet, ThemeSwitch};
    use designsystem::{install_defaults, ThemeRegistry};
    use gpui::{
        prelude::*, px, size, App, Application, Bounds, Context, Window, WindowBounds,
        WindowOptions,
    };
    use gpui_component::{
        accordion::Accordion,
        alert::{Alert, AlertVariant},
        button::{Button, ButtonVariants as _},
        group_box::GroupBox,
        resizable::{h_resizable, resizable_panel},
        styled::{h_flex, v_flex, StyledExt as _},
        text::Text,
    };
    use platform::{bootstrap, ConfigStore, FeatureFlags};

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
                    size(px(1100.0), px(720.0)),
                    cx,
                ))),
                titlebar: Some("Embedded Docs".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("Webview Documentation");
                cx.new(|_| WebviewDemoApp::new(registry.clone()))
            },
        )
        .expect("webview demo window");
    }

    pub struct WebviewDemoApp {
        theme_registry: ThemeRegistry,
        feature_flags: FeatureFlags,
        active_page: usize,
        local_pages: Vec<(&'static str, &'static str)>,
    }

    impl WebviewDemoApp {
        pub fn new(theme_registry: ThemeRegistry) -> Self {
            Self {
                theme_registry,
                feature_flags: FeatureFlags::from_env(),
                active_page: 0,
                local_pages: vec![
                    (
                        "getting-started.html",
                        "<h1>Getting started</h1><p>Drop internal docs in <code>docs/</code> and GPUI will surface them.</p>",
                    ),
                    (
                        "hotkeys.html",
                        "<h1>Hotkeys</h1><p>⌘+P opens the command palette, ⌥+⇧+F toggles focus traps.</p>",
                    ),
                ],
            }
        }

        pub fn cycle_page(&mut self, cx: &mut Context<Self>) {
            self.active_page = (self.active_page + 1) % self.local_pages.len();
            cx.notify();
        }

        pub fn render_content(&self) -> impl IntoElement {
            let (name, markup) = self.local_pages[self.active_page];
            GroupBox::new()
                .title(Text::new(format!("Local documentation – {name}")))
                .child(Text::new(markup))
        }

        pub fn render_constraints(&self) -> impl IntoElement {
            let message = if self.feature_flags.webview {
                "Feature flag enabled – interactive web content allowed."
            } else {
                "Feature flag disabled – rendering static HTML snippets instead of a live webview."
            };
            Alert::new("webview-constraints")
                .variant(if self.feature_flags.webview {
                    AlertVariant::Info
                } else {
                    AlertVariant::Warning
                })
                .title(Text::new("Platform constraints"))
                .description(Text::new(message))
        }

        pub fn render_documentation(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let usage = r#"let feature_flags = FeatureFlags::from_env();
if feature_flags.webview {
    // mount gpui-component webview
} else {
    // fallback to markdown or HTML snippets
}"#;

            let integration = r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Docs");
    cx.new(|_| WebviewDemoApp::new(theme_registry.clone()))
});"#;

            let gotchas = r#"// Loading remote content is often restricted.
if url.scheme() != "file" {
    bail!("Only local files are supported");
}"#;

            Accordion::new("webview-docs")
                .bordered(false)
                .item(|item| {
                    item.title(Text::new("Component usage"))
                        .content(render_snippet("webview-usage", "Feature gate", usage, window, cx))
                })
                .item(|item| {
                    item.title(Text::new("Why it's useful"))
                        .content(Text::new(
                            "Blend native analytics with curated docs so new teammates can debug incidents without leaving the workspace.",
                        ))
                })
                .item(|item| {
                    item.title(Text::new("Integration steps"))
                        .content(render_snippet(
                            "webview-integration",
                            "Open window",
                            integration,
                            window,
                            cx,
                        ))
                })
                .item(|item| {
                    item.title(Text::new("Gotchas"))
                        .content(render_snippet(
                            "webview-gotchas",
                            "Security",
                            gotchas,
                            window,
                            cx,
                        ))
                })
        }

        fn render_toolbar(&self) -> impl IntoElement {
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(Text::new("Embedded documentation").size(22.0).font_weight_bold())
                        .child(Text::new(
                            "Load local HTML alongside GPUI surfaces to document deployment runbooks and onboarding flows.",
                        )),
                )
                .child(ThemeSwitch::new("webview-theme", self.theme_registry.clone()).label("Theme"))
        }
    }

    impl gpui::Render for WebviewDemoApp {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            v_flex()
                .size_full()
                .bg(cx.theme().background)
                .p_6()
                .gap_4()
                .child(self.render_toolbar())
                .child(
                    h_resizable("webview-split")
                        .panel(resizable_panel("content", 0.65).child(self.render_content()))
                        .panel(
                            resizable_panel("meta", 0.35).child(
                                v_flex()
                                    .gap_3()
                                    .child(self.render_constraints())
                                    .child(
                                        Button::new("cycle-page")
                                            .ghost()
                                            .label("Next document")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.cycle_page(cx);
                                            })),
                                    )
                                    .child(render_snippet(
                                        "webview-local",
                                        "Local HTML",
                                        r#"let html = std::fs::read_to_string(path)?;
webview.load_html(&html);"#,
                                        window,
                                        cx,
                                    )),
                            ),
                        )
                        .min_panel_width(px(220.0)),
                )
                .child(self.render_documentation(window, cx))
        }
    }
}

#[cfg(feature = "webview")]
pub use app::{launch, run, WebviewDemoApp};

#[cfg(not(feature = "webview"))]
pub fn launch(_: &mut App, _: ThemeRegistry) {
    println!("Rebuild with --features webview to launch the embedded documentation view.");
}
