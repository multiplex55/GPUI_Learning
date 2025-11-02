use std::str::FromStr;
use std::time::Duration;

use clap::Parser;
use components::{docs::render_snippet, DockLayoutPanel, ThemeSwitch};
use designsystem::{install_defaults, IconLoader, IconName, ThemeRegistry, ThemeVariant};
use gpui::{
    div, prelude::*, px, size, AnyElement, App, Application, Bounds, Context, SharedString, Timer,
    Window, WindowBounds, WindowOptions,
};
use gpui::platform::keystroke::Keystroke;
use gpui_component::{
    alert::{Alert, AlertVariant},
    button::{Button, ButtonVariants as _},
    group_box::GroupBox,
    icon::Icon,
    kbd::Kbd,
    resizable::{h_resizable, resizable_panel},
    styled::{h_flex, v_flex, StyledExt as _},
    switch::Switch,
    tab::{Tab, TabBar, TabVariant},
    text::Text,
};
use platform::{bootstrap, CommandBus, ConfigStore, LocalizationRegistry};
use unic_langid::{langid, LanguageIdentifier};

#[derive(Debug, Clone, Parser)]
#[command(name = "gallery", about = "Launch the GPUI component gallery", version, long_about = None)]
struct GalleryCli {
    /// Apply initial state before the gallery window appears.
    #[arg(long = "open", value_name = "TARGET", value_parser = GalleryLaunchTarget::from_str)]
    open: Vec<GalleryLaunchTarget>,
}

#[derive(Debug, Clone)]
enum GalleryLaunchTarget {
    Category(GalleryCategorySlug),
    IconSet(IconSetSlug),
    Theme(ThemeSelector),
    PaletteOverlay,
    DocsKeyboard,
    Locale(LanguageIdentifier),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GalleryCategorySlug {
    Inputs,
    Navigation,
    Feedback,
    Data,
    Layout,
    Overlays,
    Editors,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IconSetSlug {
    Core,
    Product,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeSelector {
    Light,
    Dark,
    HighContrast,
}

impl FromStr for GalleryLaunchTarget {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(category) = value.strip_prefix("category=") {
            return GalleryCategorySlug::from_str(category).map(Self::Category);
        }
        if let Some(icon) = value.strip_prefix("icons=") {
            return IconSetSlug::from_str(icon).map(Self::IconSet);
        }
        if let Some(theme) = value.strip_prefix("theme=") {
            return ThemeSelector::from_str(theme).map(Self::Theme);
        }
        if let Some(locale) = value.strip_prefix("locale=") {
            return LanguageIdentifier::from_str(locale)
                .map(Self::Locale)
                .map_err(|err| format!("invalid locale: {err}"));
        }
        match value {
            "palette" | "palette-overlay" => Ok(Self::PaletteOverlay),
            "docs=keyboard" | "docs" => Ok(Self::DocsKeyboard),
            other => Err(format!("unknown open target: {other}")),
        }
    }
}

impl FromStr for GalleryCategorySlug {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "inputs" => Ok(Self::Inputs),
            "navigation" => Ok(Self::Navigation),
            "feedback" => Ok(Self::Feedback),
            "data" => Ok(Self::Data),
            "layout" => Ok(Self::Layout),
            "overlays" => Ok(Self::Overlays),
            "editors" => Ok(Self::Editors),
            other => Err(format!("unknown category: {other}")),
        }
    }
}

impl GalleryCategorySlug {
    fn index(self) -> usize {
        match self {
            Self::Inputs => 0,
            Self::Navigation => 1,
            Self::Feedback => 2,
            Self::Data => 3,
            Self::Layout => 4,
            Self::Overlays => 5,
            Self::Editors => 6,
        }
    }
}

impl FromStr for IconSetSlug {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "core" => Ok(Self::Core),
            "product" => Ok(Self::Product),
            other => Err(format!("unknown icon set: {other}")),
        }
    }
}

impl IconSetSlug {
    fn index(self) -> usize {
        match self {
            Self::Core => 0,
            Self::Product => 1,
        }
    }
}

impl FromStr for ThemeSelector {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "high-contrast" | "contrast" => Ok(Self::HighContrast),
            other => Err(format!("unknown theme: {other}")),
        }
    }
}

impl ThemeSelector {
    fn variant(self) -> ThemeVariant {
        match self {
            Self::Light => ThemeVariant::Light,
            Self::Dark => ThemeVariant::Dark,
            Self::HighContrast => ThemeVariant::HighContrast,
        }
    }
}

fn main() {
    let cli = GalleryCli::parse();
    let launch_targets = cli.open;

    let app = install_defaults(Application::new());
    app.run(move |cx| {
        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        bootstrap(cx, &store).expect("workspace configuration");

        let localization = seed_localization();
        let command_bus = CommandBus::new();
        let pending_launches = launch_targets.clone();

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
                let pending_launches = pending_launches.clone();
                cx.new(move |_| {
                    GalleryApp::new(
                        registry.clone(),
                        localization.clone(),
                        command_bus.clone(),
                        pending_launches.clone(),
                    )
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
            IconName::ProductAccessibility,
            IconName::ProductPalette,
            IconName::ProductLocalization,
            IconName::ProductShortcuts,
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
    pending_launches: Vec<GalleryLaunchTarget>,
    command_palette_open: bool,
}

#[derive(Clone, Copy)]
struct PaletteAction {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    shortcut: &'static str,
    handler: fn(&mut GalleryApp, &mut Window, &mut Context<GalleryApp>),
}

impl PaletteAction {
    const fn new(
        id: &'static str,
        label: &'static str,
        description: &'static str,
        shortcut: &'static str,
        handler: fn(&mut GalleryApp, &mut Window, &mut Context<GalleryApp>),
    ) -> Self {
        Self {
            id,
            label,
            description,
            shortcut,
            handler,
        }
    }
}

impl GalleryApp {
    fn new(
        theme_registry: ThemeRegistry,
        localization: LocalizationRegistry,
        command_bus: CommandBus<GalleryCommand>,
        pending_launches: Vec<GalleryLaunchTarget>,
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
            pending_launches,
            command_palette_open: false,
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

    fn process_pending(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_launches.is_empty() {
            return;
        }
        let launches = std::mem::take(&mut self.pending_launches);
        for launch in launches {
            self.apply_launch_target(launch, window, cx);
        }
    }

    fn apply_launch_target(
        &mut self,
        target: GalleryLaunchTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            GalleryLaunchTarget::Category(slug) => {
                self.active_category = slug.index();
                cx.notify();
            }
            GalleryLaunchTarget::IconSet(slug) => {
                self.icon_set = slug.index();
                cx.notify();
            }
            GalleryLaunchTarget::Theme(selector) => {
                self.apply_theme(selector.variant(), cx);
                window.refresh();
            }
            GalleryLaunchTarget::PaletteOverlay => {
                self.palette_overlay = true;
                cx.notify();
            }
            GalleryLaunchTarget::DocsKeyboard => {
                self.active_category = GalleryCategorySlug::Navigation.index();
                self.palette_overlay = true;
                self.command_palette_open = true;
                cx.notify();
                window.refresh();
            }
            GalleryLaunchTarget::Locale(locale) => {
                self.locale = locale;
                cx.notify();
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

    fn render_shortcut_keys(&self, keys: &[&str]) -> AnyElement {
        let mut row = h_flex().gap_1().items_center();
        for key in keys {
            if key.contains('+') {
                for token in key.split('+') {
                    row = row.child(self.render_single_key(token));
                }
            } else {
                row = row.child(self.render_single_key(key));
            }
        }
        row.into_any_element()
    }

    fn render_single_key(&self, token: &str) -> AnyElement {
        match token {
            "arrowleft" => return Text::new("←").into_any_element(),
            "arrowright" => return Text::new("→").into_any_element(),
            "arrowup" => return Text::new("↑").into_any_element(),
            "arrowdown" => return Text::new("↓").into_any_element(),
            _ => {}
        }
        if let Ok(parsed) = Keystroke::parse(token) {
            Kbd::new(parsed).appearance(true).into_any_element()
        } else {
            Text::new(token.to_uppercase()).into_any_element()
        }
    }

    fn render_keyboard_shortcuts(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: &[(&[&str], &str)] = &[
            (&["tab"], "Move focus forward"),
            (&["shift+tab"], "Move focus backward"),
            (&["arrowleft"], "Previous category tab"),
            (&["arrowright"], "Next category tab"),
            (&["ctrl+k"], "Open the command palette"),
        ];

        GroupBox::new()
            .title(Text::new("Keyboard navigation").font_weight_semibold())
            .child(
                v_flex().gap_2().children(rows.iter().map(|(keys, description)| {
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(self.render_shortcut_keys(keys))
                        .child(
                            Text::new(*description)
                                .text_color(cx.theme().muted_foreground)
                                .size(13.0),
                        )
                        .into_any_element()
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
            .child(
                v_flex()
                    .gap_3()
                    .child(theme_controls)
                    .child(
                        Button::new("open-command-palette")
                            .ghost()
                            .icon(Icon::new(IconName::Search))
                            .label("Command palette")
                            .child(
                                h_flex()
                                    .gap_1()
                                    .ml_auto()
                                    .child(self.render_shortcut_keys(&["ctrl+k"])),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.command_palette_open = true;
                                cx.notify();
                                window.refresh();
                            })),
                    ),
            )
    }

    fn palette_actions(&self) -> &'static [PaletteAction] {
        PALETTE_ACTIONS
    }

    fn render_command_palette(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let actions = self.palette_actions();

        GroupBox::new()
            .title(Text::new("Command palette").font_weight_semibold())
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        Text::new("Run keyboard-first actions without leaving the keyboard.")
                            .text_color(cx.theme().muted_foreground)
                            .size(13.0),
                    )
                    .children(actions.iter().enumerate().map(|(ix, action)| {
                        let handler = action.handler;
                        let shortcut = action.shortcut;
                        let description = action.description;
                        let mut button = Button::new(format!("palette-action-{ix}"))
                            .label(action.label)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.command_palette_open = false;
                                handler(this, window, cx);
                                cx.notify();
                            }));
                        if let Ok(parsed) = Keystroke::parse(shortcut) {
                            button = button.child(
                                h_flex()
                                    .gap_1()
                                    .ml_auto()
                                    .child(Kbd::new(parsed).appearance(true)),
                            );
                        }

                        v_flex()
                            .gap_1()
                            .child(button)
                            .child(
                                Text::new(description)
                                    .text_color(cx.theme().muted_foreground)
                                    .size(12.0),
                            )
                            .into_any_element()
                    }))
                    .child(
                        h_flex()
                            .justify_end()
                            .child(
                                Button::new("palette-close")
                                    .ghost()
                                    .label("Close")
                                    .child(self.render_shortcut_keys(&["escape"]))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.command_palette_open = false;
                                        cx.notify();
                                        window.refresh();
                                    })),
                            ),
                    ),
            )
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

    fn render_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        GroupBox::new()
            .title(Text::new("Quick launcher").font_weight_semibold())
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("jump-inputs")
                            .label("Inputs")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_category = GalleryCategorySlug::Inputs.index();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("jump-navigation")
                            .label("Navigation")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_category = GalleryCategorySlug::Navigation.index();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("jump-feedback")
                            .label("Feedback")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_category = GalleryCategorySlug::Feedback.index();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("jump-data")
                            .label("Data display")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_category = GalleryCategorySlug::Data.index();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("quick-palette")
                            .label("Show palette")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.palette_overlay = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("quick-dark")
                            .label("Dark theme")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.apply_theme(ThemeVariant::Dark, cx);
                                window.refresh();
                            })),
                    )
                    .child(
                        Button::new("quick-contrast")
                            .label("High contrast")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.apply_theme(ThemeVariant::HighContrast, cx);
                                window.refresh();
                            })),
                    ),
            )
            .child(doc_section(
                "launcher-snippet",
                "CLI shortcut",
                r#"cargo run --package gallery -- --open category=navigation"#,
                &[
                    "Chain multiple --open flags (for example `--open theme=dark --open category=overlays`) to reproduce complex layouts.",
                    "Use `cargo xtask gallery --target navigation` for an even shorter alias during smoke tests.",
                ],
                &[
                    "Remember to quote arguments that contain equals signs when scripting in fish or PowerShell.",
                ],
                window,
                cx,
            ))
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
            .child(doc_section(
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
                &[
                    "Compose buttons through the `ButtonVariants` trait so styling stays consistent with the design system.",
                    "Drive preview knobs from a dedicated state struct to keep live docs and demos in sync.",
                ],
                &[
                    "Avoid mutating the returned button after calling `.build`—constructors consume builder state.",
                ],
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
            .child(doc_section(
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
                &[
                    "Keep the selected index in state so hotkeys and UI always agree.",
                    "Use `.with_variant(TabVariant::Segmented)` for keyboard-friendly, high-contrast tabs.",
                ],
                &[
                    "Neglecting to call `cx.notify()` after updating `active` will leave the tab bar visually stale.",
                ],
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
            .child(doc_section(
                "feedback-snippet",
                "Alert usage",
                r#"Alert::success("upload", Text::new("Upload complete"))
    .with_variant(AlertVariant::Success)
    .icon(IconName::CircleCheck);"#,
                &[
                    "Choose intent-specific variants so colors communicate status without additional copy.",
                    "Pair iconography with concise descriptions to improve accessibility.",
                ],
                &[
                    "Avoid stacking alerts without spacing—vertical rhythm keeps toast queues readable.",
                ],
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
            .child(doc_section(
                "data-snippet",
                "GroupBox",
                r#"GroupBox::new()
    .title(Text::new("KPIs"))
    .child(h_flex().gap_4().children(metrics));"#,
                &[
                    "Use `GroupBox` to wrap related metrics so they inherit consistent spacing and titles.",
                    "Combine iconography and typography to surface hierarchy inside metric cards.",
                ],
                &[
                    "Avoid hard-coding widths—let flex layouts adapt to translations and longer labels.",
                ],
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
            .child(doc_section(
                "layout-snippet",
                "Reset layout command",
                r#"command_bus.publish(GalleryCommand::ResetLayout);
// Subscribers reset their layout epoch when they
// receive the broadcast."#,
                &[
                    "Keep layout snapshots in sync by broadcasting a reset whenever dock geometry changes dramatically.",
                    "Store the layout epoch in config so reloads pick up the latest state.",
                ],
                &[
                    "Resetting without incrementing the epoch leaves resizable panels stuck with stale IDs.",
                ],
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
            .child(doc_section(
                "icons-snippet",
                "Registering icons",
                r#"for (stem, name) in IconLoader::all() {
    println!("{} -> {}", stem, name.asset_path());
}"#,
                &[ 
                    "Bundle icons into small runtime sets so overlays only load what they need.",
                    "Normalize new packs with `cargo xtask icons --pack product <dir>` so SVG attributes match gpui expectations.",
                ],
                &[
                    "After adding SVGs, rebuild the workspace—cached icons will not update until the loader regenerates.",
                    "Use `--clean` when importing if you need to replace an entire pack; otherwise files accumulate.",
                ],
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
            .child(doc_section(
                "editors-snippet",
                "render_snippet helper",
                r#"let snippet = render_snippet(
    "demo",
    "Widget code",
    source,
    window,
    cx,
);"#,
                &[
                    "Wrap code examples with `render_snippet` so they inherit theme-aware styling.",
                    "Keep snippet IDs stable—hot reload compares identifiers to preserve scroll position.",
                ],
                &[
                    "Do not reuse snippet IDs across tabs or the editor will recycle the wrong buffer.",
                ],
                window,
                cx,
            ))
    }
}

impl gpui::Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.process_commands(cx);
        self.process_pending(window, cx);

        v_flex()
            .size_full()
            .gap_6()
            .p_6()
            .bg(cx.theme().background)
            .child(self.render_header(window, cx))
            .child(self.render_keyboard_shortcuts(cx))
            .child(self.render_launcher(window, cx))
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
            .when(self.command_palette_open, |content| {
                content.child(self.render_command_palette(window, cx))
            })
    }
}

fn action_focus_inputs(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) {
    app.active_category = GalleryCategorySlug::Inputs.index();
    cx.notify();
}

fn action_focus_overlays(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) {
    app.active_category = GalleryCategorySlug::Overlays.index();
    cx.notify();
}

fn action_switch_high_contrast(
    app: &mut GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) {
    app.apply_theme(ThemeVariant::HighContrast, cx);
    window.refresh();
}

fn action_toggle_palette_overlay(
    app: &mut GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) {
    app.palette_overlay = !app.palette_overlay;
    cx.notify();
    window.refresh();
}

fn action_reset_layout(
    app: &mut GalleryApp,
    _window: &mut Window,
    cx: &mut Context<GalleryApp>,
) {
    app.command_bus.publish(GalleryCommand::ResetLayout);
    cx.notify();
}

static PALETTE_ACTIONS: &[PaletteAction] = &[
    PaletteAction::new(
        "palette-inputs",
        "Focus inputs category",
        "Moves the gallery focus to the Inputs showcase tab for quick inspection.",
        "ctrl+1",
        action_focus_inputs,
    ),
    PaletteAction::new(
        "palette-overlays",
        "Jump to overlay demos",
        "Opens the overlay examples so keyboard navigation can be verified.",
        "ctrl+2",
        action_focus_overlays,
    ),
    PaletteAction::new(
        "palette-theme-contrast",
        "Switch to high-contrast theme",
        "Applies the accessibility-tuned palette for contrast audits.",
        "ctrl+shift+h",
        action_switch_high_contrast,
    ),
    PaletteAction::new(
        "palette-overlay-toggle",
        "Toggle palette inspector",
        "Shows or hides the live token inspector for color reviews.",
        "ctrl+.",
        action_toggle_palette_overlay,
    ),
    PaletteAction::new(
        "palette-reset-layout",
        "Reset demo layout",
        "Broadcasts a layout reset so panels return to their starting sizes.",
        "ctrl+shift+r",
        action_reset_layout,
    ),
];

fn doc_section(
    id: &str,
    title: &str,
    code: &str,
    best_practices: &[&str],
    gotchas: &[&str],
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let snippet_id = format!("{id}-snippet");

    v_flex()
        .gap_3()
        .child(render_snippet(snippet_id, title, code, window, cx))
        .when(!best_practices.is_empty(), |col| {
            col.child(GroupBox::new().title(Text::new("Best practices")).child(
                v_flex().gap_2().children(best_practices.iter().map(|tip| {
                    Text::new(*tip)
                        .text_color(cx.theme().muted_foreground)
                        .into_any_element()
                })),
            ))
        })
        .when(!gotchas.is_empty(), |col| {
            col.child(
                GroupBox::new()
                    .title(Text::new("Gotchas"))
                    .child(v_flex().gap_2().children(gotchas.iter().map(|tip| {
                        Text::new(*tip)
                            .text_color(cx.theme().danger)
                            .into_any_element()
                    }))),
            )
        })
}
