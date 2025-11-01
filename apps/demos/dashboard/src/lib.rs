use components::{docs::render_snippet, DashboardCard, KpiGrid, KpiMetric, ThemeSwitch};
use designsystem::{install_defaults, IconName, ThemeRegistry};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Window, WindowBounds,
    WindowOptions,
};
use gpui_component::{
    accordion::Accordion,
    alert::{Alert, AlertVariant},
    button::{Button, ButtonVariants as _},
    chart::{AreaChart, BarChart, LineChart, PieChart},
    group_box::GroupBox,
    icon::Icon,
    resizable::{h_resizable, resizable_panel},
    styled::{h_flex, v_flex, StyledExt as _},
    switch::Switch,
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
                size(px(1320.0), px(860.0)),
                cx,
            ))),
            titlebar: Some("Operations Dashboard".into()),
            ..Default::default()
        },
        move |window, cx| {
            window.set_title("Operations Control Center");
            cx.new(|_| DashboardDemoApp::new(registry.clone()))
        },
    )
    .expect("dashboard demo window");
}

pub struct DashboardDemoApp {
    theme_registry: ThemeRegistry,
    alerts_enabled: bool,
    triage_filter: usize,
}

impl DashboardDemoApp {
    pub fn new(theme_registry: ThemeRegistry) -> Self {
        Self {
            theme_registry,
            alerts_enabled: true,
            triage_filter: 0,
        }
    }

    fn toggle_alerts(&mut self, cx: &mut Context<Self>) {
        self.alerts_enabled = !self.alerts_enabled;
        cx.notify();
    }

    fn cycle_triage(&mut self, cx: &mut Context<Self>) {
        self.triage_filter = (self.triage_filter + 1) % 3;
        cx.notify();
    }

    fn render_kpis(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let grid = KpiGrid::default()
            .push(
                KpiMetric::new("12", "Active incidents")
                    .trend("4 customer facing")
                    .icon(IconName::ShieldAlert),
            )
            .push(
                KpiMetric::new("42m", "Mean time to resolve")
                    .trend("−8% vs last week")
                    .icon(IconName::Timer),
            )
            .push(
                KpiMetric::new("99.2%", "Uptime")
                    .trend("SLO: 99.0%")
                    .icon(IconName::Cpu),
            )
            .push(
                KpiMetric::new("1.8k", "Alerts processed")
                    .trend("+15% load")
                    .icon(IconName::Satellite),
            );

        div()
            .bg(cx.theme().popover)
            .p_5()
            .rounded(cx.theme().radius_lg)
            .child(grid)
    }

    fn render_charts(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let response_time = [
            ("00:00", 120.0f32),
            ("06:00", 98.0),
            ("12:00", 140.0),
            ("18:00", 110.0),
            ("24:00", 90.0),
        ];
        let incidents = [("P0", 2.0f32), ("P1", 6.0), ("P2", 18.0), ("P3", 44.0)];
        let capacity = [
            ("Compute", 72.0f32),
            ("Storage", 55.0),
            ("Network", 30.0),
            ("Queues", 18.0),
        ];
        let geography = [("NA", 58.0f32), ("EMEA", 24.0), ("APAC", 18.0)];

        h_flex()
            .gap_4()
            .child(
                DashboardCard::new("Response time")
                    .description("Line chart overlaying the past 24 hours")
                    .icon(IconName::Timer)
                    .child(
                        div().h(px(220.0)).child(
                            LineChart::new(response_time)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value)
                                .dot()
                                .natural(),
                        ),
                    )
                    .child(render_snippet(
                        "dashboard-line",
                        "Line chart binding",
                        r#"LineChart::new(points)
    .x(|(label, _)| *label)
    .y(|(_, value)| *value)
    .natural();"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new("Incident mix")
                    .description("Bar chart grouped by priority")
                    .icon(IconName::ShieldAlert)
                    .child(
                        div().h(px(220.0)).child(
                            BarChart::new(incidents)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value),
                        ),
                    )
                    .child(render_snippet(
                        "dashboard-bars",
                        "Bar chart",
                        r#"BarChart::new(data)
    .x(|(label, _)| *label)
    .y(|(_, value)| *value);"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new("Capacity forecast")
                    .description("Area chart predicting next 30 days")
                    .icon(IconName::Workflow)
                    .child(
                        div().h(px(220.0)).child(
                            AreaChart::new(capacity)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value),
                        ),
                    )
                    .child(render_snippet(
                        "dashboard-area",
                        "Area chart",
                        r#"AreaChart::new(series)
    .x(|(label, _)| *label)
    .y(|(_, value)| *value);"#,
                        window,
                        cx,
                    )),
            )
            .child(
                DashboardCard::new("Regional load")
                    .description("Pie chart by data center")
                    .icon(IconName::Satellite)
                    .child(
                        div().h(px(220.0)).child(
                            PieChart::new(geography)
                                .x(|(label, _)| *label)
                                .y(|(_, value)| *value),
                        ),
                    )
                    .child(render_snippet(
                        "dashboard-pie",
                        "Pie chart",
                        r#"PieChart::new(slices)
    .x(|(label, _)| *label)
    .y(|(_, value)| *value);"#,
                        window,
                        cx,
                    )),
            )
    }

    fn render_triage(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let filter_label = match self.triage_filter {
            0 => "All severities",
            1 => "Critical incidents",
            _ => "Customer impacting",
        };

        GroupBox::new().title(Text::new("Triage queue")).child(
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Text::new(format!("Filter: {filter_label}")))
                        .child(
                            Button::new("cycle-triage")
                                .ghost()
                                .label("Cycle filter")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cycle_triage(cx);
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(Text::new("• P0 – CDN saturation (15m to breach)"))
                        .child(Text::new("• P1 – Kafka consumer lag"))
                        .child(Text::new("• P2 – Scheduled maintenance")),
                )
                .child(
                    Switch::new("alerts-toggle")
                        .checked(self.alerts_enabled)
                        .label(Text::new("Escalate via PagerDuty"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_alerts(cx);
                        })),
                ),
        )
    }

    fn render_documentation(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let usage = r#"let cards = KpiGrid::default()
    .push(KpiMetric::new("99.9%", "Uptime"));"#;

        let integration = r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Operations Dashboard");
    cx.new(|_| DashboardDemoApp::new(theme_registry.clone()))
});"#;

        let gotchas = r#"// Keep the dashboard fast by moving expensive analytics off-thread.
tokio::spawn(async move { refresh_kpis().await });"#;

        Accordion::new("dashboard-docs")
            .bordered(false)
            .item(|item| {
                item.title(Text::new("Component usage"))
                    .content(render_snippet("dashboard-usage", "KPIs", usage, window, cx))
            })
            .item(|item| {
                item.title(Text::new("Why it's useful"))
                    .content(Text::new(
                        "Focuses on incident response workflows rather than marketing metrics, showcasing how shared components compose into a real operations hub.",
                    ))
            })
            .item(|item| {
                item.title(Text::new("Integration steps"))
                    .content(render_snippet(
                        "dashboard-integration",
                        "Window wiring",
                        integration,
                        window,
                        cx,
                    ))
            })
            .item(|item| {
                item.title(Text::new("Gotchas"))
                    .content(render_snippet(
                        "dashboard-gotchas",
                        "Async analytics",
                        gotchas,
                        window,
                        cx,
                    ))
            })
    }
}

impl gpui::Render for DashboardDemoApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .p_6()
            .gap_5()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Text::new("Operations command center").size(22.0).font_weight_bold())
                            .child(Text::new(
                                "Purpose-built dashboard for SRE incident response – KPIs, charts, alerts, and triage tools share the same design tokens as the workbench.",
                            )),
                    )
                    .child(ThemeSwitch::new("dashboard-theme", self.theme_registry.clone()).label("Theme")),
            )
            .child(self.render_kpis(cx))
            .child(self.render_charts(window, cx))
            .child(
                h_resizable("dashboard-split")
                    .panel(resizable_panel("triage", 0.4).child(self.render_triage(cx)))
                    .panel(
                        resizable_panel("forms", 0.6).child(
                            GroupBox::new()
                                .title(Text::new("Mitigation playbook"))
                                .child(
                                    v_flex()
                                        .gap_3()
                                        .child(Text::new("• Run safety checks"))
                                        .child(Text::new("• Coordinate messaging"))
                                        .child(Text::new("• File follow-up tasks"))
                                        .child(
                                            Button::new("acknowledge")
                                                .primary()
                                                .label("Acknowledge incident")
                                                .icon(Icon::new(IconName::Command)),
                                        ),
                                ),
                        ),
                    )
                    .min_panel_width(px(240.0)),
            )
            .child(
                Alert::new("dashboard-alert")
                    .variant(if self.alerts_enabled {
                        AlertVariant::Info
                    } else {
                        AlertVariant::Warning
                    })
                    .title(Text::new(if self.alerts_enabled {
                        "Auto-escalation enabled"
                    } else {
                        "Auto-escalation paused"
                    }))
                    .description(Text::new(
                        "Flip the toggle above to control alert routing – useful when running incident drills vs real incidents.",
                    )),
            )
            .child(self.render_documentation(window, cx))
    }
}
