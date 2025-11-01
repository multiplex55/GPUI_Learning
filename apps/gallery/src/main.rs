use std::time::Duration;

use components::{docs::render_snippet, DockLayoutPanel, ThemeSwitch};
use designsystem::{install_defaults, IconLoader, IconName, ThemeRegistry, ThemeVariant};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, SharedString, Timer, Window,
    WindowBounds, WindowOptions,
};
use gpui_component::{
    alert::{Alert, AlertVariant},
    button::{Button, ButtonVariants as _},
    group_box::GroupBox,
    icon::Icon,
    resizable::{h_resizable, resizable_panel},
    styled::{h_flex, v_flex, StyledExt as _},
    switch::Switch,
    tab::{Tab, TabBar, TabVariant},
    text::Text,
};
use platform::{bootstrap, CommandBus, ConfigStore, LocalizationRegistry};
use unic_langid::{langid, LanguageIdentifier};

fn main() {
    let app = install_defaults(Application::new());
    app.run(|cx| {
        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        bootstrap(cx, &store).expect("workspace configuration");

        let localization = seed_localization();
        let command_bus = CommandBus::new();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1440.0), px(900.0)),
                    cx,
                ))),
                titlebar: Some("GPUI Gallery".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("GPUI Component Gallery");
                cx.new(|_| {
                    GalleryApp::new(registry.clone(), localization.clone(), command_bus.clone())
                })
            },
        )
        .expect("gallery window");
        cx.activate(true);
    });
}

fn seed_localization() -> LocalizationRegistry {
    let registry = LocalizationRegistry::new(langid!("en-US"));
    registry.register_messages(
        langid!("en-US"),
        [
            ("category.inputs", "Inputs"),
            ("category.navigation", "Navigation"),
            ("category.feedback", "Feedback"),
            ("category.data", "Data Display"),
            ("category.layout", "Layout"),
            ("category.overlays", "Overlays"),
            ("category.editors", "Editors"),
            ("theme.palette", "Palette Inspector"),
            (
                "theme.instructions",
                "Switch between light, dark, and high-contrast workspaces.",
            ),
            ("knobs.size", "Cycle size"),
            ("knobs.variant", "Cycle variant"),
            ("knobs.icon", "Toggle icon"),
            ("knobs.disabled", "Disable component"),
            ("layout.reset", "Reset Layout"),
            ("icons.heading", "Runtime Icon Sets"),
            (
                "icons.docs",
                "Add new icons by dropping SVGs into the design system and rebuilding.",
            ),
            ("docs.keyboard", "Keyboard navigation"),
            (
                "docs.keyboard.body",
                "Arrow keys move between tabs, while Alt+Shift+F toggles focus traps.",
            ),
        ],
    );
    registry.register_messages(
        langid!("es-ES"),
        [
            ("category.inputs", "Entradas"),
            ("category.navigation", "Navegación"),
            ("category.feedback", "Retroalimentación"),
            ("category.data", "Visualización de datos"),
            ("category.layout", "Disposición"),
            ("category.overlays", "Superposiciones"),
            ("category.editors", "Editores"),
            ("theme.palette", "Inspector de paleta"),
            (
                "theme.instructions",
                "Alterna entre temas claros, oscuros y de alto contraste.",
            ),
            ("knobs.size", "Cambiar tamaño"),
            ("knobs.variant", "Cambiar variante"),
            ("knobs.icon", "Icono"),
            ("knobs.disabled", "Deshabilitar"),
            ("layout.reset", "Restablecer diseño"),
            ("icons.heading", "Conjuntos de iconos"),
            (
                "icons.docs",
                "Añade iconos SVG al sistema de diseño y recompila.",
            ),
            ("docs.keyboard", "Navegación con teclado"),
            (
                "docs.keyboard.body",
                "Las flechas cambian entre pestañas y Alt+Shift+F alterna el foco.",
            ),
        ],
    );
    registry
}

#[derive(Clone, Copy)]
enum GalleryCommand {
    ResetLayout,
}

#[derive(Clone, Copy)]
struct DemoKnobs {
    size: gpui_component::styled::Size,
    variant_index: usize,
    show_icon: bool,
    disabled: bool,
}

impl Default for DemoKnobs {
    fn default() -> Self {
        Self {
            size: gpui_component::styled::Size::Medium,
            variant_index: 0,
            show_icon: true,
            disabled: false,
        }
    }
}

struct IconSetDescriptor {
    name: &'static str,
    description: &'static str,
    icons: &'static [IconName],
}

const ICON_SETS: &[IconSetDescriptor] = &[
    IconSetDescriptor {
        name: "Core",
        description: "Essential UI chrome shared across applications.",
        icons: &[
            IconName::Menu,
            IconName::Search,
            IconName::Settings,
            IconName::Palette,
        ],
    },
    IconSetDescriptor {
        name: "Product",
        description: "Feature-specific glyphs loaded at runtime.",
        icons: &[
            IconName::ChartPie,
            IconName::Calendar,
            IconName::Globe,
            IconName::LayoutDashboard,
        ],
    },
];

const THEME_VARIANTS: &[(ThemeVariant, IconName, &str)] = &[
    (ThemeVariant::Light, IconName::Sun, "Light"),
    (ThemeVariant::Dark, IconName::Moon, "Dark"),
    (
        ThemeVariant::HighContrast,
        IconName::Palette,
        "High Contrast",
    ),
];

const GALLERY_CATEGORIES: &[(&str, IconName)] = &[
    ("category.inputs", IconName::SquareTerminal),
    ("category.navigation", IconName::PanelLeftOpen),
    ("category.feedback", IconName::Bell),
    ("category.data", IconName::ChartPie),
    ("category.layout", IconName::LayoutDashboard),
    ("category.overlays", IconName::PanelRightOpen),
    ("category.editors", IconName::BookOpen),
];

struct GalleryApp {
    theme_registry: ThemeRegistry,
    localization: LocalizationRegistry,
    locale: LanguageIdentifier,
    command_bus: CommandBus<GalleryCommand>,
    receiver: crossbeam_channel::Receiver<GalleryCommand>,
    knobs: DemoKnobs,
    active_category: usize,
    palette_overlay: bool,
    icon_set: usize,
    layout_epoch: u64,
    theme_preview: ThemeVariant,
}

impl GalleryApp {
    fn new(
        theme_registry: ThemeRegistry,
        localization: LocalizationRegistry,
        command_bus: CommandBus<GalleryCommand>,
    ) -> Self {
        let receiver = command_bus.subscribe();
        Self {
            theme_preview: theme_registry.active(),
            theme_registry,
            localization,
            locale: langid!("en-US"),
            command_bus,
            receiver,
            knobs: DemoKnobs::default(),
            active_category: 0,
            palette_overlay: false,
            icon_set: 0,
            layout_epoch: 0,
        }
    }

    fn t(&self, key: &str) -> SharedString {
        self.localization
            .translate(&self.locale, key)
            .unwrap_or_else(|| key.to_owned())
            .into()
    }

    fn process_commands(&mut self, cx: &mut Context<Self>) {
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                GalleryCommand::ResetLayout => {
                    self.layout_epoch = self.layout_epoch.wrapping_add(1);
                    cx.notify();
                }
            }
        }
    }

    fn cycle_size(&mut self) {
        self.knobs.size = match self.knobs.size {
            gpui_component::styled::Size::XSmall => gpui_component::styled::Size::Small,
            gpui_component::styled::Size::Small => gpui_component::styled::Size::Medium,
            gpui_component::styled::Size::Medium => gpui_component::styled::Size::Large,
            gpui_component::styled::Size::Large => gpui_component::styled::Size::XSmall,
            gpui_component::styled::Size::Size(_) => gpui_component::styled::Size::Medium,
        };
    }

    fn cycle_variant(&mut self) {
        self.knobs.variant_index = (self.knobs.variant_index + 1) % 4;
    }

    fn variant_label(&self) -> &'static str {
        match self.knobs.variant_index {
            0 => "Secondary",
            1 => "Primary",
            2 => "Ghost",
            _ => "Danger",
        }
    }

    fn apply_theme(&mut self, variant: ThemeVariant, cx: &mut Context<Self>) {
        self.theme_registry.apply(variant, cx);
        self.theme_preview = variant;
        cx.notify();
    }

    fn render_knobs(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let size_key = self.t("knobs.size");
        let variant_key = self.t("knobs.variant");
        let icon_key = self.t("knobs.icon");
        let disabled_key = self.t("knobs.disabled");
        let variant_label = SharedString::from(self.variant_label());

        h_flex()
            .gap_3()
            .child(
                Button::new("knob-size")
                    .label(size_key)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.cycle_size();
                        cx.notify();
                    })),
            )
            .child(
                Button::new("knob-variant")
                    .label(format!("{variant_key}: {variant_label}"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.cycle_variant();
                        cx.notify();
                    })),
            )
            .child(
                Switch::new("knob-icon")
                    .checked(self.knobs.show_icon)
                    .label(Text::new(icon_key))
                    .on_click(cx.listener(|this, state, _, cx| {
                        this.knobs.show_icon = *state;
                        cx.notify();
                    })),
            )
            .child(
                Switch::new("knob-disabled")
                    .checked(self.knobs.disabled)
                    .label(Text::new(disabled_key))
                    .on_click(cx.listener(|this, state, _, cx| {
                        this.knobs.disabled = *state;
                        cx.notify();
                    })),
            )
    }

    fn render_header(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_controls = self.render_theme_controls(window, cx);

        h_flex()
            .justify_between()
            .items_center()
            .child(
                v_flex()
                    .gap_1()
                    .child(Text::new("Component Gallery").size(20.0))
                    .child(
                        Text::new("Explore workspace primitives with live knobs and docs.")
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
            .child(theme_controls)
    }

    fn render_theme_controls(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let palette_label = self.t("theme.palette");
        let instructions = self.t("theme.instructions");
        let active_ix = THEME_VARIANTS
            .iter()
            .position(|(variant, _, _)| *variant == self.theme_preview)
            .unwrap_or(0);

        v_flex()
            .gap_3()
            .child(Text::new(instructions).text_color(cx.theme().muted_foreground))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        ThemeSwitch::new("theme-toggle", self.theme_registry.clone())
                            .label("Light/Dark"),
                    )
                    .child(
                        TabBar::new("theme-variants")
                            .with_variant(TabVariant::Pill)
                            .selected_index(active_ix)
                            .on_click(cx.listener(|this, index, window, cx| {
                                if let Some((variant, _, _)) = THEME_VARIANTS.get(*index) {
                                    this.apply_theme(*variant, cx);
                                    window.refresh();
                                }
                            }))
                            .children(THEME_VARIANTS.iter().enumerate().map(
                                |(ix, (variant, icon, label))| {
                                    Tab::new(*label)
                                        .prefix(Icon::new(*icon))
                                        .selected(*variant == self.theme_preview)
                                        .with_size(gpui_component::styled::Size::Small)
                                        .id(format!("theme-tab-{ix}"))
                                },
                            )),
                    )
                    .child(
                        Button::new("palette-inspector")
                            .ghost()
                            .icon(Icon::new(IconName::Inspector))
                            .label(palette_label.clone())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.palette_overlay = !this.palette_overlay;
                                cx.notify();
                                window.refresh();
                            })),
                    ),
            )
            .when(self.palette_overlay, |content| {
                content.child(self.render_palette_overlay(window, cx))
            })
    }

    fn render_palette_overlay(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let definition = self.theme_registry.definition(self.theme_preview);
        let palette = &definition.tokens.colors;
        let tokens = [
            ("Primary", cx.theme().primary, palette.primary),
            (
                "On Primary",
                cx.theme().primary_foreground,
                palette.on_primary,
            ),
            ("Accent", cx.theme().accent, palette.accent),
            ("On Accent", cx.theme().accent_foreground, palette.on_accent),
            ("Background", cx.theme().background, palette.background),
            ("Surface", cx.theme().popover, palette.surface),
            ("Muted", cx.theme().muted, palette.muted),
            ("Success", cx.theme().success, palette.success),
            ("Warning", cx.theme().warning, palette.warning),
            ("Danger", cx.theme().danger, palette.danger),
        ];

        GroupBox::new()
            .title(Text::new(format!(
                "{} palette",
                self.theme_preview.as_str()
            )))
            .child(
                h_flex()
                    .gap_3()
                    .flex_wrap()
                    .children(tokens.into_iter().map(|(label, color, hex)| {
                        v_flex()
                            .gap_1()
                            .w(px(120.0))
                            .child(
                                div()
                                    .rounded(cx.theme().radius)
                                    .h(px(48.0))
                                    .bg(color)
                                    .shadow_md(),
                            )
                            .child(Text::new(label))
                            .child(
                                Text::new(hex)
                                    .text_color(cx.theme().muted_foreground)
                                    .size(12.0),
                            )
                    })),
            )
    }

    fn render_category_tabs(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self
            .active_category
            .min(GALLERY_CATEGORIES.len().saturating_sub(1));

        TabBar::new("gallery-categories")
            .underline()
            .selected_index(selected)
            .on_click(cx.listener(|this, index, _, cx| {
                this.active_category = *index;
                cx.notify();
            }))
            .children(
                GALLERY_CATEGORIES
                    .iter()
                    .enumerate()
                    .map(|(index, (key, icon))| {
                        Tab::new(self.t(key))
                            .prefix(Icon::new(*icon))
                            .selected(index == self.active_category)
                            .id(format!("gallery-tab-{index}"))
                    }),
            )
    }

    fn render_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut demo_button = Button::new("demo-button")
            .label("Call to Action")
            .with_size(self.knobs.size)
            .tooltip("Press enter or space to activate")
            .on_click(cx.listener(|this, _, window, cx| {
                window
                    .spawn(cx, async move |cx| {
                        Timer::after(Duration::from_millis(250)).await;
                        cx.update(|_, _| {});
                    })
                    .detach();
                cx.notify();
            }));

        demo_button = match self.knobs.variant_index {
            0 => demo_button,
            1 => demo_button.primary(),
            2 => demo_button.ghost(),
            _ => demo_button.danger(),
        };

        if self.knobs.disabled {
            demo_button = demo_button.disabled(true);
        }
        if self.knobs.show_icon {
            demo_button = demo_button.icon(Icon::new(IconName::ChevronRight));
        }

        v_flex()
            .gap_4()
            .child(self.render_knobs(cx))
            .child(
                GroupBox::new()
                    .title(Text::new("Buttons"))
                    .child(h_flex().gap_3().items_center().child(demo_button)),
            )
            .child(render_snippet(
                "inputs-snippet",
                "Button with knobs",
                r#"let mut button = Button::new("primary")
    .label("Call to Action")
    .with_size(Size::Medium)
    .primary();
if disabled {
    button = button.disabled(true);
}
button.build(window, cx);"#,
                window,
                cx,
            ))
    }

    fn render_navigation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let tabs = TabBar::new("nav-demo")
            .with_variant(TabVariant::Segmented)
            .selected_index(1)
            .children([
                Tab::new("Inbox").prefix(Icon::new(IconName::Inbox)),
                Tab::new("Boards").prefix(Icon::new(IconName::LayoutDashboard)),
                Tab::new("Timeline").prefix(Icon::new(IconName::Calendar)),
            ]);

        v_flex()
            .gap_4()
            .child(
                GroupBox::new()
                    .title(Text::new("Segmented tabs"))
                    .child(tabs),
            )
            .child(render_snippet(
                "nav-snippet",
                "Keyboard navigation",
                r#"TabBar::new("nav")
    .with_variant(TabVariant::Segmented)
    .on_click(cx.listener(|this, index, _, cx| {
        this.active = *index;
        cx.notify();
    }))
    .children([
        Tab::new("Inbox"),
        Tab::new("Boards"),
        Tab::new("Timeline"),
    ]);"#,
                window,
                cx,
            ))
            .child(
                GroupBox::new()
                    .title(Text::new(self.t("docs.keyboard")))
                    .child(
                        Text::new(self.t("docs.keyboard.body"))
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
    }

    fn render_feedback(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(GroupBox::new().title(Text::new("Alert variants")).child(
                v_flex().gap_3().children([
                    Alert::info(
                        "alert-info",
                        Text::new("Inform users about neutral updates."),
                    ),
                    Alert::success("alert-success", Text::new("Celebrate successful outcomes.")),
                    Alert::warning("alert-warning", Text::new("Warn about recoverable issues.")),
                    Alert::error("alert-error", Text::new("Escalate critical failures.")),
                ]),
            ))
            .child(render_snippet(
                "feedback-snippet",
                "Alert usage",
                r#"Alert::success("upload", Text::new("Upload complete"))
    .with_variant(AlertVariant::Success)
    .icon(IconName::CircleCheck);"#,
                window,
                cx,
            ))
    }

    fn render_data(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(GroupBox::new().title(Text::new("Metric summary")).child(
                h_flex().gap_4().children([
                    self.metric_card("Active Users", "12,481", IconName::CircleUser, cx),
                    self.metric_card("Conversion", "36%", IconName::ChartPie, cx),
                    self.metric_card("NPS", "68", IconName::Heart, cx),
                ]),
            ))
            .child(render_snippet(
                "data-snippet",
                "GroupBox",
                r#"GroupBox::new()
    .title(Text::new("KPIs"))
    .child(h_flex().gap_4().children(metrics));"#,
                window,
                cx,
            ))
    }

    fn metric_card(
        &self,
        label: &str,
        value: &str,
        icon: IconName,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        GroupBox::new().title(Text::new(label)).child(
            h_flex()
                .gap_3()
                .items_center()
                .child(Icon::new(icon).with_size(gpui_component::styled::Size::Large))
                .child(Text::new(value).size(24.0))
                .child(Text::new("vs last week").text_color(cx.theme().muted_foreground)),
        )
    }

    fn render_layout(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let resizable_id = format!("layout-panels-{}", self.layout_epoch);
        let dock = DockLayoutPanel::default()
            .sidebar(
                v_flex().gap_2().child(Text::new("Docked sidebar")).child(
                    Text::new("Place persistent navigation here")
                        .text_color(cx.theme().muted_foreground),
                ),
            )
            .toolbar(
                h_flex().gap_2().child(
                    Button::new("dock-refresh")
                        .ghost()
                        .icon(Icon::new(IconName::Replace))
                        .label("Refresh"),
                ),
            )
            .child(Text::new("Dock layouts keep navigation anchored."));

        let resizable = h_resizable(resizable_id)
            .child(
                resizable_panel().child(
                    v_flex()
                        .gap_2()
                        .child(Text::new("Primary panel"))
                        .child(Text::new("Drag handles to resize")),
                ),
            )
            .child(
                resizable_panel().child(
                    v_flex()
                        .gap_2()
                        .child(Text::new("Secondary panel"))
                        .child(Text::new("Useful for logs or previews")),
                ),
            );

        v_flex()
            .gap_4()
            .child(dock)
            .child(resizable)
            .child(
                Button::new("reset-layout")
                    .label(self.t("layout.reset"))
                    .icon(Icon::new(IconName::Replace))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.command_bus.publish(GalleryCommand::ResetLayout);
                        cx.notify();
                    })),
            )
            .child(render_snippet(
                "layout-snippet",
                "Reset layout command",
                r#"command_bus.publish(GalleryCommand::ResetLayout);
// Subscribers reset their layout epoch when they
// receive the broadcast."#,
                window,
                cx,
            ))
    }

    fn render_overlays(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let heading = self.t("icons.heading");
        let docs = self.t("icons.docs");
        let active = ICON_SETS.get(self.icon_set).unwrap_or(&ICON_SETS[0]);

        v_flex()
            .gap_4()
            .child(
                GroupBox::new().title(Text::new(heading)).child(
                    v_flex()
                        .gap_3()
                        .child(
                            Text::new(active.description).text_color(cx.theme().muted_foreground),
                        )
                        .child(
                            h_flex()
                                .gap_3()
                                .flex_wrap()
                                .children(active.icons.iter().map(|icon| {
                                    v_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .rounded(cx.theme().radius)
                                                .p_3()
                                                .bg(cx.theme().muted)
                                                .child(Icon::new(*icon).with_size(
                                                    gpui_component::styled::Size::Large,
                                                )),
                                        )
                                        .child(
                                            Text::new(IconLoader::asset_path(*icon))
                                                .size(12.0)
                                                .text_color(cx.theme().muted_foreground),
                                        )
                                })),
                        )
                        .child(
                            TabBar::new("icon-sets")
                                .with_variant(TabVariant::Pill)
                                .selected_index(self.icon_set)
                                .on_click(cx.listener(|this, index, _, cx| {
                                    this.icon_set = *index;
                                    cx.notify();
                                }))
                                .children(ICON_SETS.iter().enumerate().map(|(ix, set)| {
                                    Tab::new(set.name)
                                        .selected(ix == self.icon_set)
                                        .id(format!("icon-tab-{ix}"))
                                })),
                        ),
                ),
            )
            .child(Text::new(docs).text_color(cx.theme().muted_foreground))
            .child(render_snippet(
                "icons-snippet",
                "Registering icons",
                r#"for (stem, name) in IconLoader::all() {
    println!("{} -> {}", stem, name.asset_path());
}"#,
                window,
                cx,
            ))
    }

    fn render_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let markdown = "```rust\nfn greet(name: &str) {\n    println!(\"Hello {name}!\");\n}\n```";
        let editor = gpui_component::text::TextView::markdown("code-demo", markdown, window, cx);

        v_flex()
            .gap_4()
            .child(
                GroupBox::new()
                    .title(Text::new("Live markdown"))
                    .child(editor),
            )
            .child(render_snippet(
                "editors-snippet",
                "render_snippet helper",
                r#"let snippet = render_snippet(
    "demo",
    "Widget code",
    source,
    window,
    cx,
);"#,
                window,
                cx,
            ))
    }
}

impl gpui::Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.process_commands(cx);

        v_flex()
            .size_full()
            .gap_6()
            .p_6()
            .bg(cx.theme().background)
            .child(self.render_header(window, cx))
            .child(self.render_category_tabs(cx))
            .child(match self.active_category {
                0 => self.render_inputs(window, cx).into_any_element(),
                1 => self.render_navigation(window, cx).into_any_element(),
                2 => self.render_feedback(window, cx).into_any_element(),
                3 => self.render_data(window, cx).into_any_element(),
                4 => self.render_layout(window, cx).into_any_element(),
                5 => self.render_overlays(window, cx).into_any_element(),
                _ => self.render_editors(window, cx).into_any_element(),
            })
    }
}
