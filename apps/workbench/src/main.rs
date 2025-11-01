use components::{
    docs::render_snippet, DashboardCard, DockLayoutPanel, KpiGrid, KpiMetric, ThemeSwitch,
};
use designsystem::{install_defaults, IconName, ThemeRegistry, ThemeVariant};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Keystroke, SharedString, Window,
    WindowBounds, WindowOptions,
};
use gpui_component::{
    accordion::Accordion,
    alert::{Alert, AlertVariant},
    button::{Button, ButtonVariants as _},
    chart::{AreaChart, BarChart, LineChart, PieChart},
    icon::Icon,
    kbd::Kbd,
    notification::{Notification, NotificationType},
    resizable::{h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem, SidebarToggleButton},
    styled::{h_flex, v_flex, StyledExt as _},
    switch::Switch,
    tab::{Tab, TabBar, TabVariant},
    text::Text,
    ContextModal,
};
use platform::{
    bootstrap, CommandBus, ConfigStore, LayoutState, LocalizationRegistry, WorkspaceConfig,
};
use unic_langid::{langid, LanguageIdentifier};

fn main() {
    let app = install_defaults(Application::new());
    app.run(|cx| {
        gpui_component::init(cx);

        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        let config = bootstrap(cx, &store).expect("workspace configuration");
        let localization = seed_localization();
        let command_bus = CommandBus::new();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1440.0), px(900.0)),
                    cx,
                ))),
                titlebar: Some("GPUI Workbench".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("GPUI Workbench Shell");
                let app = WorkbenchApp::new(
                    registry.clone(),
                    localization.clone(),
                    command_bus.clone(),
                    config.clone(),
                    store.clone(),
                );
                cx.new(|_| app)
            },
        )
        .expect("workbench window");

        cx.activate(true);
    });
}

fn seed_localization() -> LocalizationRegistry {
    let registry = LocalizationRegistry::new(langid!("en-US"));
    registry.register_messages(
        langid!("en-US"),
        [
            ("nav.dashboard", "Dashboard"),
            ("nav.gallery", "Gallery"),
            ("nav.demos", "Demos"),
            (
                "dashboard.subtitle",
                "Cross-application workspace shell with dockable analytics",
            ),
            ("dashboard.sidebar.primary", "Workspace"),
            ("dashboard.sidebar.activity", "Activity"),
            ("dashboard.sidebar.shortcuts", "Shortcuts"),
            ("dashboard.sidebar.health", "Health"),
            ("dashboard.sidebar.actions", "Actions"),
            ("dashboard.kpis", "Key metrics"),
            ("dashboard.activity", "Live activity"),
            ("dashboard.notifications", "Notifications"),
            ("dashboard.forms", "Forms"),
            ("dashboard.forms.wizard", "Provision environment"),
            ("dashboard.forms.filters", "Data filters"),
            ("dashboard.forms.validation", "Validation"),
            ("dashboard.docs", "How it works"),
            ("dashboard.palette", "Command palette"),
            ("dashboard.reset", "Reset layout"),
            ("gallery.launch", "Open component gallery"),
            ("demos.launch", "Open demo workspaces"),
            ("docs.shortcuts", "Keyboard shortcuts"),
            (
                "docs.shortcuts.body",
                "Command palette opens with Ctrl+P / Cmd+P",
            ),
            ("locale.toggle", "Switch locale"),
            ("toast.title", "Saved"),
            ("toast.body", "Workspace preferences synced"),
        ],
    );
    registry.register_messages(
        langid!("es-ES"),
        [
            ("nav.dashboard", "Panel"),
            ("nav.gallery", "Galería"),
            ("nav.demos", "Demostraciones"),
            (
                "dashboard.subtitle",
                "Shell analítico con paneles acoplables",
            ),
            ("dashboard.sidebar.primary", "Espacio"),
            ("dashboard.sidebar.activity", "Actividad"),
            ("dashboard.sidebar.shortcuts", "Atajos"),
            ("dashboard.sidebar.health", "Estado"),
            ("dashboard.sidebar.actions", "Acciones"),
            ("dashboard.kpis", "Métricas"),
            ("dashboard.activity", "Actividad"),
            ("dashboard.notifications", "Notificaciones"),
            ("dashboard.forms", "Formularios"),
            ("dashboard.forms.wizard", "Aprovisionar entorno"),
            ("dashboard.forms.filters", "Filtros de datos"),
            ("dashboard.forms.validation", "Validación"),
            ("dashboard.docs", "Cómo funciona"),
            ("dashboard.palette", "Paleta de comandos"),
            ("dashboard.reset", "Restablecer diseño"),
            ("gallery.launch", "Abrir galería"),
            ("demos.launch", "Abrir demostraciones"),
            ("docs.shortcuts", "Atajos de teclado"),
            (
                "docs.shortcuts.body",
                "La paleta se abre con Ctrl+P o Cmd+P",
            ),
            ("locale.toggle", "Cambiar idioma"),
            ("toast.title", "Guardado"),
            ("toast.body", "Preferencias sincronizadas"),
        ],
    );
    registry
}

#[derive(Clone, Copy, Debug)]
enum WorkbenchCommand {
    ResetLayout,
    OpenGallery,
    OpenDemos,
    ToggleTheme,
    ToggleLocale,
    ShowPalette,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchTab {
    Dashboard,
    Gallery,
    Demos,
}

impl WorkbenchTab {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "nav.dashboard",
            Self::Gallery => "nav.gallery",
            Self::Demos => "nav.demos",
        }
    }
}

#[derive(Default)]
struct WizardState {
    step: usize,
    segment: Option<&'static str>,
    region: Option<&'static str>,
    error: Option<SharedString>,
}

#[derive(Default)]
struct FilterState {
    status_ix: Option<usize>,
    owner_ix: Option<usize>,
    include_archived: bool,
    error: Option<SharedString>,
}

struct WorkbenchApp {
    theme_registry: ThemeRegistry,
    localization: LocalizationRegistry,
    locale: LanguageIdentifier,
    command_bus: CommandBus<WorkbenchCommand>,
    receiver: crossbeam_channel::Receiver<WorkbenchCommand>,
    workspace_config: WorkspaceConfig,
    config_store: ConfigStore,
    selected_tab: WorkbenchTab,
    sidebar_collapsed: bool,
    layout_epoch: u64,
    toast_counter: usize,
    wizard: WizardState,
    filter: FilterState,
    chart_tick: usize,
    theme_variant: ThemeVariant,
}

impl WorkbenchApp {
    fn new(
        theme_registry: ThemeRegistry,
        localization: LocalizationRegistry,
        command_bus: CommandBus<WorkbenchCommand>,
        workspace_config: WorkspaceConfig,
        config_store: ConfigStore,
    ) -> Self {
        let receiver = command_bus.subscribe();
        let mut app = Self {
            theme_variant: theme_registry.active(),
            theme_registry,
            localization,
            locale: langid!("en-US"),
            command_bus,
            receiver,
            workspace_config,
            config_store,
            selected_tab: WorkbenchTab::Dashboard,
            sidebar_collapsed: false,
            layout_epoch: 0,
            toast_counter: 0,
            wizard: WizardState::default(),
            filter: FilterState::default(),
            chart_tick: 0,
        };
        if let Some(state) = app.workspace_config.layout_state.clone() {
            app.apply_persisted_state(&state);
        }
        app
    }

    fn translate(&self, key: &str) -> SharedString {
        self.localization
            .translate(&self.locale, key)
            .unwrap_or_else(|| key.to_owned())
            .into()
    }

    fn apply_persisted_state(&mut self, state: &str) {
        for token in state.split(';') {
            let (key, value) = token.split_once(':').unwrap_or((token, ""));
            match key {
                "tab" => {
                    self.selected_tab = match value {
                        "Gallery" => WorkbenchTab::Gallery,
                        "Demos" => WorkbenchTab::Demos,
                        _ => WorkbenchTab::Dashboard,
                    };
                }
                "sidebar" => {
                    self.sidebar_collapsed = value == "1";
                }
                "epoch" => {
                    if let Ok(epoch) = value.parse::<u64>() {
                        self.layout_epoch = epoch;
                    }
                }
                "locale" => {
                    if let Ok(locale) = value.parse::<LanguageIdentifier>() {
                        self.locale = locale;
                    }
                }
                "theme" => {
                    self.theme_variant = match value {
                        "Dark" => ThemeVariant::Dark,
                        "HighContrast" => ThemeVariant::HighContrast,
                        _ => ThemeVariant::Light,
                    };
                }
                _ => {}
            }
        }
    }

    fn persist_state(&mut self, cx: &mut Context<Self>) {
        let snapshot = format!(
            "tab:{:?};sidebar:{};epoch:{};locale:{};theme:{:?}",
            self.selected_tab,
            if self.sidebar_collapsed { 1 } else { 0 },
            self.layout_epoch,
            self.locale,
            self.theme_variant,
        );
        self.workspace_config.layout_state = Some(snapshot.clone());
        cx.set_global(LayoutState(snapshot));
        if let Err(err) = self.config_store.save(&self.workspace_config) {
            eprintln!("failed to persist layout: {err}");
        }
    }

    fn process_commands(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                WorkbenchCommand::ResetLayout => {
                    self.layout_epoch = self.layout_epoch.wrapping_add(1);
                    self.sidebar_collapsed = false;
                    self.selected_tab = WorkbenchTab::Dashboard;
                    self.persist_state(cx);
                    window.push_notification(
                        Notification::new(self.translate("toast.title"))
                            .title(self.translate("toast.title"))
                            .content(|_, _| Text::new("Layout restored").into_any_element())
                            .with_type(NotificationType::Info),
                        cx,
                    );
                }
                WorkbenchCommand::OpenGallery => self.open_gallery_window(cx),
                WorkbenchCommand::OpenDemos => self.open_demo_window(cx),
                WorkbenchCommand::ToggleTheme => {
                    self.cycle_theme(cx);
                }
                WorkbenchCommand::ToggleLocale => self.toggle_locale(cx),
                WorkbenchCommand::ShowPalette => self.open_palette(window, cx),
            }
        }
    }

    fn open_palette(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.open_drawer(cx, |drawer, window, cx| {
            drawer.title("Command palette").content(|window, cx| {
                v_flex()
                    .gap_3()
                    .child(Text::new("Palette commands").font_weight_bold().size(16.0))
                    .child(Text::new("• Reset layout"))
                    .child(Text::new("• Toggle theme"))
                    .child(Text::new("• Switch locale"))
                    .child(render_snippet(
                        "palette-snippet",
                        "Palette integration",
                        r#"command_bus.publish(WorkbenchCommand::ShowPalette);"#,
                        window,
                        cx,
                    ))
            })
        });
    }

    fn toggle_locale(&mut self, cx: &mut Context<Self>) {
        self.locale = if self.locale == langid!("en-US") {
            langid!("es-ES")
        } else {
            langid!("en-US")
        };
        self.persist_state(cx);
        cx.notify();
    }

    fn cycle_theme(&mut self, cx: &mut Context<Self>) {
        self.theme_variant = match self.theme_variant {
            ThemeVariant::Light => ThemeVariant::Dark,
            ThemeVariant::Dark => ThemeVariant::HighContrast,
            ThemeVariant::HighContrast => ThemeVariant::Light,
        };
        self.theme_registry.apply(self.theme_variant, cx);
        self.persist_state(cx);
        cx.notify();
    }

    fn open_gallery_window(&self, cx: &mut Context<Self>) {
        let registry = self.theme_registry.clone();
        let localization = self.localization.clone();
        cx.open_window(
            WindowOptions {
                titlebar: Some("Component Gallery Preview".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("Gallery Preview");
                let view = GalleryPreview::new(registry.clone(), localization.clone());
                cx.new(|_| view)
            },
        )
        .expect("gallery preview window");
    }

    fn open_demo_window(&self, cx: &mut Context<Self>) {
        let registry = self.theme_registry.clone();
        cx.open_window(
            WindowOptions {
                titlebar: Some("Demo Launchpad".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("Demo Launchpad");
                let view = DemoLauncher::new(registry.clone());
                cx.new(|_| view)
            },
        )
        .expect("demo window");
    }

    fn render_view(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.process_commands(window, cx);
        self.chart_tick = self.chart_tick.wrapping_add(1);

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .p_6()
            .gap_6()
            .child(self.render_top_bar(window, cx))
            .child(match self.selected_tab {
                WorkbenchTab::Dashboard => self.render_dashboard(window, cx).into_any_element(),
                WorkbenchTab::Gallery => self.render_gallery_tab(window, cx).into_any_element(),
                WorkbenchTab::Demos => self.render_demos_tab(window, cx).into_any_element(),
            })
    }

    fn render_top_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shortcuts = self.translate("docs.shortcuts");
        let shortcuts_body = self.translate("docs.shortcuts.body");

        let palette_shortcut = if cfg!(target_os = "macos") {
            Keystroke::parse("meta+p").ok()
        } else {
            Keystroke::parse("ctrl+p").ok()
        };

        h_flex()
            .items_center()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(Text::new("GPUI Workbench").size(22.0).font_weight_bold())
                    .child(
                        Text::new(self.translate("dashboard.subtitle"))
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
            .child(self.render_nav_tabs(cx))
            .child(
                Button::new("palette")
                    .ghost()
                    .icon(Icon::new(IconName::Command))
                    .label(self.translate("dashboard.palette"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.command_bus.publish(WorkbenchCommand::ShowPalette);
                        this.open_palette(window, cx);
                    })),
            )
            .child(ThemeSwitch::new("workbench-theme", self.theme_registry.clone()).label("Theme"))
            .child(
                Button::new("toggle-locale")
                    .label(self.translate("locale.toggle"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.command_bus.publish(WorkbenchCommand::ToggleLocale);
                    })),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Text::new(shortcuts).font_weight_medium())
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Text::new(shortcuts_body.clone())
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                palette_shortcut
                                    .map(|key| Kbd::new(key).appearance(true))
                                    .unwrap_or_else(|| {
                                        Kbd::new(Keystroke::parse("ctrl+p").unwrap())
                                    }),
                            ),
                    ),
            )
            .child(
                Button::new("reset-layout")
                    .ghost()
                    .icon(Icon::new(IconName::Replace))
                    .label(self.translate("dashboard.reset"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.command_bus.publish(WorkbenchCommand::ResetLayout);
                    })),
            )
            .child(
                Button::new("notify")
                    .ghost()
                    .icon(Icon::new(IconName::Bell))
                    .label("Toast")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toast_counter += 1;
                        let caption =
                            format!("{} #{}", this.translate("toast.body"), this.toast_counter);
                        window.push_notification(
                            Notification::new(caption.clone())
                                .title(this.translate("toast.title"))
                                .content(move |_, _| Text::new(caption.clone()).into_any_element())
                                .with_type(NotificationType::Success),
                            cx,
                        );
                    })),
            )
            .child(
                Button::new("open-gallery")
                    .ghost()
                    .icon(Icon::new(IconName::LayoutDashboard))
                    .label(self.translate("gallery.launch"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.command_bus.publish(WorkbenchCommand::OpenGallery);
                    })),
            )
            .child(
                Button::new("open-demos")
                    .ghost()
                    .icon(Icon::new(IconName::Workflow))
                    .label(self.translate("demos.launch"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.command_bus.publish(WorkbenchCommand::OpenDemos);
                    })),
            )
    }

    fn render_nav_tabs(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        TabBar::new("workbench-tabs")
            .with_variant(TabVariant::Underline)
            .selected_index(match self.selected_tab {
                WorkbenchTab::Dashboard => 0,
                WorkbenchTab::Gallery => 1,
                WorkbenchTab::Demos => 2,
            })
            .on_click(cx.listener(|this, index, _, cx| {
                this.selected_tab = match *index {
                    0 => WorkbenchTab::Dashboard,
                    1 => WorkbenchTab::Gallery,
                    _ => WorkbenchTab::Demos,
                };
                this.persist_state(cx);
                cx.notify();
            }))
            .children([
                Tab::new(self.translate(WorkbenchTab::Dashboard.label())).id("tab-dashboard"),
                Tab::new(self.translate(WorkbenchTab::Gallery.label())).id("tab-gallery"),
                Tab::new(self.translate(WorkbenchTab::Demos.label())).id("tab-demos"),
            ])
    }

    fn render_dashboard(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let resizable = h_resizable(format!("dashboard-panels-{}", self.layout_epoch))
            .child(resizable_panel().child(self.render_sidebar(cx)))
            .child(
                resizable_panel()
                    .flex_1()
                    .child(self.render_main_panels(window, cx)),
            );

        v_flex()
            .gap_6()
            .child(resizable)
            .child(self.render_docs(window, cx))
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let primary_label = self.translate("dashboard.sidebar.primary");
        let activity_label = self.translate("dashboard.sidebar.activity");
        let shortcut_label = self.translate("dashboard.sidebar.shortcuts");
        let health_label = self.translate("dashboard.sidebar.health");
        let actions_label = self.translate("dashboard.sidebar.actions");

        Sidebar::left()
            .collapsed(self.sidebar_collapsed)
            .header(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(Text::new("Navigation").font_weight_semibold())
                    .child(
                        SidebarToggleButton::left()
                            .collapsed(self.sidebar_collapsed)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sidebar_collapsed = !this.sidebar_collapsed;
                                this.persist_state(cx);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                SidebarGroup::new(primary_label).child(
                    SidebarMenu::new().children([
                        SidebarMenuItem::new("Overview")
                            .icon(Icon::new(IconName::LayoutDashboard))
                            .active(true)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.selected_tab = WorkbenchTab::Dashboard;
                                this.persist_state(cx);
                                cx.notify();
                            })),
                        SidebarMenuItem::new("Roadmap").icon(Icon::new(IconName::CalendarClock)),
                        SidebarMenuItem::new("Automation")
                            .icon(Icon::new(IconName::Cpu))
                            .suffix(Text::new("Beta").text_xs()),
                    ]),
                ),
            )
            .child(
                SidebarGroup::new(activity_label).child(
                    SidebarMenu::new().children([
                        SidebarMenuItem::new("Incidents")
                            .icon(Icon::new(IconName::Flame))
                            .on_click(cx.listener(|this, _, window, cx| {
                                window.push_notification(
                                    Notification::new("Incident escalated")
                                        .with_type(NotificationType::Warning),
                                    cx,
                                );
                            })),
                        SidebarMenuItem::new("Deployments").icon(Icon::new(IconName::Ship)),
                    ]),
                ),
            )
            .child(
                SidebarGroup::new(shortcut_label)
                    .collapsed(self.sidebar_collapsed)
                    .child(
                        SidebarMenu::new().children([
                            SidebarMenuItem::new("Command palette")
                                .icon(Icon::new(IconName::Command))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.command_bus.publish(WorkbenchCommand::ShowPalette);
                                    this.open_palette(window, cx);
                                })),
                            SidebarMenuItem::new("Reset layout")
                                .icon(Icon::new(IconName::Replace))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.command_bus.publish(WorkbenchCommand::ResetLayout);
                                    cx.notify();
                                })),
                        ]),
                    ),
            )
            .child(
                SidebarGroup::new(health_label).child(
                    DashboardCard::new("Service SLO")
                        .description("Rolling 30 day uptime")
                        .action(Text::new("99.9%"))
                        .child(Text::new("Compute"))
                        .child(Text::new("Edge network"))
                        .child(
                            Alert::new("Service health is nominal").variant(AlertVariant::Success),
                        ),
                ),
            )
            .footer(
                v_flex()
                    .gap_2()
                    .child(Text::new(actions_label).text_sm().font_weight_semibold())
                    .child(Button::new("sidebar-theme").label("Cycle theme").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.command_bus.publish(WorkbenchCommand::ToggleTheme);
                            cx.notify();
                        }),
                    )),
            )
    }

    fn render_main_panels(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        DockLayoutPanel::default()
            .sidebar(
                v_flex()
                    .gap_3()
                    .child(Text::new(self.translate("dashboard.kpis")).font_weight_semibold())
                    .child(self.render_kpis(cx)),
            )
            .toolbar(
                h_flex()
                    .gap_2()
                    .child(Text::new(self.translate("dashboard.activity")))
                    .child(
                        Switch::new("autoplay-stream")
                            .checked(self.chart_tick % 2 == 0)
                            .label(Text::new("Autoplay")),
                    )
                    .child(
                        Button::new("ping")
                            .ghost()
                            .label("Ping services")
                            .icon(Icon::new(IconName::Satellite))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.command_bus.publish(WorkbenchCommand::OpenDemos);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                v_flex()
                    .gap_5()
                    .child(self.render_charts(window, cx))
                    .child(self.render_forms(window, cx))
                    .child(self.render_notifications(window, cx)),
            )
    }

    fn render_kpis(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let grid = KpiGrid::default()
            .push(
                KpiMetric::new("1,024", "Active users")
                    .trend("+12% WoW")
                    .icon(IconName::Users),
            )
            .push(
                KpiMetric::new("287 ms", "P99 latency")
                    .trend("−48 ms")
                    .icon(IconName::Timer),
            )
            .push(
                KpiMetric::new("4.6", "Support CSAT")
                    .trend("200 responses")
                    .icon(IconName::Stars),
            )
            .push(KpiMetric::new("18", "Open incidents").trend("3 blocking"));

        div()
            .bg(cx.theme().popover)
            .p_4()
            .rounded(cx.theme().radius)
            .child(grid)
    }

    fn render_charts(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let revenue = [
            ("Jan", 32.0f32),
            ("Feb", 38.0),
            ("Mar", 42.0),
            ("Apr", 48.0),
            ("May", 55.0),
            ("Jun", 62.0),
        ];
        let incidents = [
            ("Auth", 6.0f32),
            ("Billing", 3.0),
            ("Compute", 8.0),
            ("Edge", 2.0),
        ];
        let active = [("North", 45.0), ("EMEA", 32.0), ("APAC", 23.0)];
        let budget = [
            ("Ops", 28.0f32),
            ("R&D", 44.0),
            ("Growth", 18.0),
            ("People", 10.0),
        ];

        h_flex()
            .gap_4()
            .child(
                DashboardCard::new("Monthly revenue")
                    .description("Line chart with moving average")
                    .child(
                        div().h(px(220.0)).child(
                            LineChart::new(revenue)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value)
                                .natural()
                                .dot(),
                        ),
                    )
                    .child(expandable_docs(
                        "line-chart-docs",
                        "Line chart",
                        r#"LineChart::new(points)
    .x(|(label, _)| *label)
    .y(|(_, value)| *value)
    .natural();"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new("Incident volume")
                    .description("Bar chart by subsystem")
                    .child(
                        div().h(px(220.0)).child(
                            BarChart::new(incidents)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value),
                        ),
                    )
                    .child(expandable_docs(
                        "bar-chart-docs",
                        "Bar chart",
                        r#"BarChart::new(dataset)
    .x(|entry| entry.category)
    .y(|entry| entry.count);"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new("Active regions")
                    .description("Area chart with stacked signal")
                    .child(
                        div().h(px(220.0)).child(
                            AreaChart::new(active)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value),
                        ),
                    )
                    .child(expandable_docs(
                        "area-chart-docs",
                        "Area chart",
                        r#"AreaChart::new(points)
    .x(|(label, _)| *label)
    .y(|(_, value)| *value);"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new("Budget mix")
                    .description("Pie chart with department allocation")
                    .child(
                        div().h(px(220.0)).child(
                            PieChart::new(budget)
                                .value(|(_, value)| *value)
                                .label(|(label, _)| (*label).into()),
                        ),
                    )
                    .child(expandable_docs(
                        "pie-chart-docs",
                        "Pie chart",
                        r#"PieChart::new(slices)
    .label(|slice| slice.name.clone())
    .value(|slice| slice.percent);"#,
                        window,
                        cx,
                    )),
            )
    }

    fn render_forms(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_5()
            .child(
                DashboardCard::new(self.translate("dashboard.forms.wizard"))
                    .description("Three-step provisioning wizard with validation")
                    .child(self.render_wizard(cx))
                    .child(expandable_docs(
                        "wizard-docs",
                        "Wizard state",
                        r#"if let Some(error) = wizard.error.clone() {
    Text::new(error).text_color(cx.theme().danger);
}"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new(self.translate("dashboard.forms.filters"))
                    .description("Fast filters with validation feedback")
                    .child(self.render_filters(cx))
                    .child(expandable_docs(
                        "filters-docs",
                        "Filter actions",
                        r#"Button::new("apply")
    .label("Apply filters")
    .on_click(cx.listener(|this, _, _, cx| {
        this.validate_filters();
        cx.notify();
    }));"#,
                        window,
                        cx,
                    )),
            )
    }

    fn render_wizard(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let steps = ["Choose segment", "Select region", "Review & launch"];
        let current = self.wizard.step.min(steps.len() - 1);
        let step_label = steps[current];

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .children(steps.iter().enumerate().map(|(ix, label)| {
                        let mut badge = Text::new(format!("{}", label)).text_sm();
                        if ix == current {
                            badge = badge.font_weight_semibold();
                        }
                        badge.into_any_element()
                    })),
            )
            .child(match current {
                0 => h_flex().gap_3().child(
                    Button::new("segment")
                        .label(self.wizard.segment.unwrap_or("Select customer segment"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            const SEGMENTS: [&str; 3] = ["Enterprise", "Scale-up", "Startup"];
                            let next = this
                                .wizard
                                .segment
                                .and_then(|current| SEGMENTS.iter().position(|v| v == &current))
                                .map(|ix| (ix + 1) % SEGMENTS.len())
                                .unwrap_or(0);
                            this.wizard.segment = Some(SEGMENTS[next]);
                            this.wizard.error = None;
                            cx.notify();
                        })),
                ),
                1 => h_flex().gap_3().child(
                    Button::new("region")
                        .label(self.wizard.region.unwrap_or("Choose deployment region"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            const REGIONS: [&str; 3] = ["us-east", "eu-west", "ap-southeast"];
                            let next = this
                                .wizard
                                .region
                                .and_then(|current| REGIONS.iter().position(|v| v == &current))
                                .map(|ix| (ix + 1) % REGIONS.len())
                                .unwrap_or(0);
                            this.wizard.region = Some(REGIONS[next]);
                            this.wizard.error = None;
                            cx.notify();
                        })),
                ),
                _ => v_flex()
                    .gap_2()
                    .child(Text::new("Summary").font_weight_semibold())
                    .child(Text::new(format!(
                        "Segment: {}",
                        self.wizard.segment.unwrap_or("Not selected"),
                    )))
                    .child(Text::new(format!(
                        "Region: {}",
                        self.wizard.region.unwrap_or("Not selected"),
                    )))
                    .child(
                        Text::new("Launch uses multi-tenant cluster with autoscaling.")
                            .text_color(cx.theme().muted_foreground),
                    ),
            })
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("back")
                            .disabled(current == 0)
                            .label("Back")
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.wizard.step > 0 {
                                    this.wizard.step -= 1;
                                    this.wizard.error = None;
                                    cx.notify();
                                }
                            })),
                    )
                    .child(
                        Button::new("next")
                            .label(if current == steps.len() - 1 {
                                "Launch"
                            } else {
                                "Next"
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                match this.wizard.step {
                                    0 => {
                                        if this.wizard.segment.is_none() {
                                            this.wizard.error = Some(
                                                "Select a customer segment to continue".into(),
                                            );
                                        } else {
                                            this.wizard.step = 1;
                                            this.wizard.error = None;
                                        }
                                    }
                                    1 => {
                                        if this.wizard.region.is_none() {
                                            this.wizard.error =
                                                Some("Choose a deployment region".into());
                                        } else {
                                            this.wizard.step = 2;
                                            this.wizard.error = None;
                                        }
                                    }
                                    _ => {
                                        window.push_notification(
                                            Notification::new("Environment provisioning")
                                                .with_type(NotificationType::Info),
                                            cx,
                                        );
                                        this.wizard = WizardState::default();
                                    }
                                }
                                cx.notify();
                            })),
                    ),
            )
            .when_some(self.wizard.error.clone(), |col, error| {
                col.child(Text::new(error).text_color(cx.theme().danger))
            })
            .child(Text::new(step_label).text_color(cx.theme().muted_foreground))
    }

    fn render_filters(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let statuses = ["Active", "Paused", "Archived"];
        let owners = ["Core", "Data", "Edge"];

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(Text::new("Status"))
                    .child(
                        Button::new("status")
                            .label(
                                self.filter
                                    .status_ix
                                    .and_then(|ix| statuses.get(ix).copied())
                                    .unwrap_or("Select status"),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                let next = this
                                    .filter
                                    .status_ix
                                    .map(|ix| (ix + 1) % statuses.len())
                                    .unwrap_or(0);
                                this.filter.status_ix = Some(next);
                                this.filter.error = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        Switch::new("archived")
                            .checked(self.filter.include_archived)
                            .label(Text::new("Include archived"))
                            .on_click(cx.listener(|this, state, _, cx| {
                                this.filter.include_archived = *state;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                h_flex().gap_2().child(Text::new("Owner")).child(
                    Button::new("owner")
                        .label(
                            self.filter
                                .owner_ix
                                .and_then(|ix| owners.get(ix).copied())
                                .unwrap_or("Select team"),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            let next = this
                                .filter
                                .owner_ix
                                .map(|ix| (ix + 1) % owners.len())
                                .unwrap_or(0);
                            this.filter.owner_ix = Some(next);
                            this.filter.error = None;
                            cx.notify();
                        })),
                ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("apply-filters")
                            .label("Apply")
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.filter.status_ix.is_none() {
                                    this.filter.error = Some("Select a status filter".into());
                                } else {
                                    this.filter.error = None;
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("reset-filters")
                            .label("Clear")
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = FilterState::default();
                                cx.notify();
                            })),
                    ),
            )
            .when_some(self.filter.error.clone(), |col, error| {
                col.child(Text::new(error).text_color(cx.theme().danger))
            })
    }

    fn render_notifications(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        DashboardCard::new(self.translate("dashboard.notifications"))
            .description("Event stream rendered as toasts")
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("emit-info")
                            .label("Info toast")
                            .on_click(cx.listener(|_, _, window, cx| {
                                window.push_notification(
                                    Notification::new("Sync in progress")
                                        .with_type(NotificationType::Info),
                                    cx,
                                );
                            })),
                    )
                    .child(Button::new("emit-warning").label("Warning toast").on_click(
                        cx.listener(|_, _, window, cx| {
                            window.push_notification(
                                Notification::new("Deployment queue paused")
                                    .with_type(NotificationType::Warning),
                                cx,
                            );
                        }),
                    )),
            )
            .child(
                Alert::new("Notifications use gpui-component::ContextModal")
                    .variant(AlertVariant::Info),
            )
            .child(expandable_docs(
                "toast-docs",
                "Notifications",
                r#"window.push_notification(
    Notification::new("Deploy started")
        .with_type(NotificationType::Info),
    cx,
);"#,
                window,
                cx,
            ))
    }

    fn render_docs(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        DashboardCard::new(self.translate("dashboard.docs"))
            .description("Expandable sections showcase inline documentation")
            .child(expandable_docs(
                "layout-docs",
                "Persisted layout",
                r#"self.persist_state(cx);
let snapshot = cx.global::<LayoutState>().0.clone();"#,
                window,
                cx,
            ))
            .child(expandable_docs(
                "commands-docs",
                "Command bus",
                r#"command_bus.publish(WorkbenchCommand::ResetLayout);"#,
                window,
                cx,
            ))
    }

    fn render_gallery_tab(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(
                Text::new("Component gallery preview")
                    .size(18.0)
                    .font_weight_semibold(),
            )
            .child(Text::new(
                "Launch the gallery within this shell to explore every component with live knobs.",
            ))
            .child(expandable_docs(
                "gallery-launch-docs",
                "Open gallery window",
                r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Gallery Preview");
    cx.new(|_| GalleryPreview::new(registry.clone(), localization.clone()))
});"#,
                window,
                cx,
            ))
    }

    fn render_demos_tab(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(Text::new("Demo launchpad").size(18.0).font_weight_semibold())
            .child(Text::new(
                "Spin up purpose-built demo shells (markdown notes, dashboards, explorers) without leaving the workbench.",
            ))
            .child(expandable_docs(
                "demos-launch-docs",
                "Open demo window",
                r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Demo Launchpad");
    cx.new(|_| DemoLauncher::new(theme_registry.clone()))
});"#,
                window,
                cx,
            ))
    }
}

impl gpui::Render for WorkbenchApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_view(window, cx)
    }
}

struct GalleryPreview {
    theme_registry: ThemeRegistry,
    localization: LocalizationRegistry,
    locale: LanguageIdentifier,
}

impl GalleryPreview {
    fn new(theme_registry: ThemeRegistry, localization: LocalizationRegistry) -> Self {
        Self {
            theme_registry,
            localization,
            locale: langid!("en-US"),
        }
    }

    fn translate(&self, key: &str) -> SharedString {
        self.localization
            .translate(&self.locale, key)
            .unwrap_or_else(|| key.to_owned())
            .into()
    }
}

impl gpui::Render for GalleryPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_4()
            .p_6()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(Text::new("Gallery preview").size(20.0).font_weight_bold())
                    .child(
                        ThemeSwitch::new("gallery-preview-theme", self.theme_registry.clone())
                            .label("Theme"),
                    ),
            )
            .child(
                Text::new(
                    "This lightweight preview shares the same theme registry and localization as the gallery binary.",
                )
                .text_color(cx.theme().muted_foreground),
            )
            .child(expandable_docs(
                "gallery-preview-docs",
                "Reuse registries",
                r#"let registry = ThemeRegistry::new();
registry.install(cx);

let preview = GalleryPreview::new(registry.clone(), localization.clone());"#,
                window,
                cx,
            ))
    }
}

struct DemoLauncher {
    theme_registry: ThemeRegistry,
}

impl DemoLauncher {
    fn new(theme_registry: ThemeRegistry) -> Self {
        Self { theme_registry }
    }
}

impl gpui::Render for DemoLauncher {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .p_6()
            .bg(cx.theme().background)
            .child(
                h_flex()
                    .justify_between()
                    .child(Text::new("Demo launchpad").size(20.0).font_weight_bold())
                    .child(
                        ThemeSwitch::new("demo-launcher-theme", self.theme_registry.clone())
                            .label("Theme"),
                    ),
            )
            .child(Text::new(
                "Launch markdown notes, analytics dashboards, and exploratory tooling side-by-side.",
            ))
            .child(
                h_flex()
                    .gap_3()
                    .child(Button::new("notes").label("Markdown notes"))
                    .child(Button::new("analytics").label("Analytics dashboard"))
                    .child(Button::new("explorer").label("Data explorer")),
            )
            .child(expandable_docs(
                "demo-launcher-docs",
                "Launch pattern",
                r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Markdown Notes");
    // mount demo view here
});"#,
                window,
                cx,
            ))
    }
}

fn expandable_docs(
    id: &str,
    title: impl Into<SharedString>,
    code: &str,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let snippet_id = format!("{id}-snippet");

    Accordion::new(id).bordered(false).item(|item| {
        item.title(
            h_flex()
                .justify_between()
                .items_center()
                .child(Text::new(title))
                .child(Text::new("View code").text_color(cx.theme().muted_foreground)),
        )
        .content(render_snippet(snippet_id, "Snippet", code, window, cx))
    })
}
