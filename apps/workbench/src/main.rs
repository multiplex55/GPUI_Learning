use std::fmt::Write as _;
use std::str::FromStr;

use chrono::Utc;
use clap::Parser;
use components::{
    docs::render_snippet, DashboardCard, DockLayoutPanel, KpiGrid, KpiMetric, ThemeSwitch,
};
use data::VirtualListBenchmark;
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
    group_box::GroupBox,
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
    bootstrap, BenchmarkRunRecord, CommandBus, ConfigStore, EditorBenchmarkSummary, LayoutState,
    LocalizationRegistry, VirtualizationBenchmarkSummary, WorkspaceConfig,
};
use unic_langid::{langid, LanguageIdentifier};

#[derive(Debug, Clone, Parser)]
#[command(name = "workbench", about = "Launch the GPUI workbench shell", version, long_about = None)]
struct WorkbenchCli {
    /// Automatically open windows, demos, or auxiliary launchers.
    #[arg(long = "open", value_name = "TARGET", value_parser = LaunchTarget::from_str)]
    open: Vec<LaunchTarget>,
}

#[derive(Debug, Clone)]
enum LaunchTarget {
    Demo(DemoSlug),
    GalleryWindow,
    DemoLauncher,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DemoSlug {
    DataExplorer,
    MarkdownNotes,
    CodePlayground,
    OperationsDashboard,
    WebviewDocs,
}

impl DemoSlug {
    fn label(self) -> &'static str {
        match self {
            Self::DataExplorer => "Virtualized Data Explorer",
            Self::MarkdownNotes => "Markdown Notes",
            Self::CodePlayground => "Code Playground",
            Self::OperationsDashboard => "Operations Dashboard",
            Self::WebviewDocs => "Embedded Docs",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::DataExplorer => "data-explorer",
            Self::MarkdownNotes => "markdown-notes",
            Self::CodePlayground => "code-playground",
            Self::OperationsDashboard => "operations-dashboard",
            Self::WebviewDocs => "webview",
        }
    }
}

impl FromStr for DemoSlug {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "data-explorer" | "data" => Ok(Self::DataExplorer),
            "markdown-notes" | "notes" => Ok(Self::MarkdownNotes),
            "code-playground" | "playground" => Ok(Self::CodePlayground),
            "operations-dashboard" | "dashboard" => Ok(Self::OperationsDashboard),
            "webview" | "docs" | "embedded-docs" => Ok(Self::WebviewDocs),
            other => Err(format!("unknown demo target: {other}")),
        }
    }
}

impl FromStr for LaunchTarget {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(demo) = value.strip_prefix("demo=") {
            return DemoSlug::from_str(demo).map(LaunchTarget::Demo);
        }
        match value {
            "gallery" | "gallery-window" => Ok(LaunchTarget::GalleryWindow),
            "demos" | "launcher" | "demo-launcher" => Ok(LaunchTarget::DemoLauncher),
            other => Err(format!("unknown open target: {other}")),
        }
    }
}

fn main() {
    let cli = WorkbenchCli::parse();
    let launch_targets = cli.open;

    let app = install_defaults(Application::new());
    app.run(move |cx| {
        gpui_component::init(cx);

        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        let config = bootstrap(cx, &store).expect("workspace configuration");
        let localization = seed_localization();
        let command_bus = CommandBus::new();
        let open_requests = launch_targets.clone();

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
                let open_requests = open_requests.clone();
                let registry_for_app = registry.clone();
                let localization_for_app = localization.clone();
                let store_for_app = store.clone();
                let config_for_app = config.clone();
                let bus_for_app = command_bus.clone();
                let bus_for_launch = command_bus.clone();
                cx.new(move |_| {
                    let mut app = WorkbenchApp::new(
                        registry_for_app.clone(),
                        localization_for_app.clone(),
                        bus_for_app.clone(),
                        config_for_app.clone(),
                        store_for_app.clone(),
                    );
                    for target in &open_requests {
                        match target {
                            LaunchTarget::GalleryWindow => {
                                bus_for_launch.publish(WorkbenchCommand::OpenGallery);
                            }
                            LaunchTarget::DemoLauncher => {
                                bus_for_launch.publish(WorkbenchCommand::OpenDemos);
                            }
                            LaunchTarget::Demo(demo) => {
                                bus_for_launch.publish(WorkbenchCommand::OpenDemo(*demo));
                            }
                        }
                    }
                    app
                })
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
            ("nav.performance", "Performance"),
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
            ("nav.performance", "Rendimiento"),
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
    OpenDemo(DemoSlug),
    ToggleTheme,
    ToggleLocale,
    ShowPalette,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchTab {
    Dashboard,
    Gallery,
    Demos,
    Performance,
}

impl WorkbenchTab {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "nav.dashboard",
            Self::Gallery => "nav.gallery",
            Self::Demos => "nav.demos",
            Self::Performance => "nav.performance",
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

#[derive(Debug, Clone)]
struct VirtualizationSample {
    frame: usize,
    scroll_fps: f32,
    render_latency_ms: f32,
    memory_mib: f32,
}

#[derive(Debug, Clone)]
struct EditorSample {
    tick: usize,
    typing_latency_ms: f32,
    lsp_latency_ms: f32,
    memory_mib: f32,
}

struct PerformanceState {
    virtualization_rows: usize,
    virtualization_row_height: f32,
    virtualization_viewport: f32,
    virtualization_overscan: usize,
    virtualization_dense_layout: bool,
    virtualization_samples: Vec<VirtualizationSample>,
    editor_lines: usize,
    editor_highlighting: bool,
    editor_lsp: bool,
    editor_samples: Vec<EditorSample>,
    history: Vec<BenchmarkRunRecord>,
    run_counter: u64,
}

impl PerformanceState {
    fn new(history: Vec<BenchmarkRunRecord>) -> Self {
        let run_counter = history.last().map(|run| run.id).unwrap_or(0);
        Self {
            virtualization_rows: 200_000,
            virtualization_row_height: 28.0,
            virtualization_viewport: 540.0,
            virtualization_overscan: 256,
            virtualization_dense_layout: false,
            virtualization_samples: Vec::new(),
            editor_lines: 200_000,
            editor_highlighting: true,
            editor_lsp: true,
            editor_samples: Vec::new(),
            history,
            run_counter,
        }
    }

    fn virtualization_benchmark(&self) -> VirtualListBenchmark {
        VirtualListBenchmark {
            total_rows: self.virtualization_rows,
            row_height: self.virtualization_row_height,
            viewport_height: self.virtualization_viewport,
        }
    }

    fn virtualization_summary(&self) -> VirtualizationBenchmarkSummary {
        let total = self.virtualization_samples.len() as f32;
        let (avg_fps, avg_latency, peak_memory) = if total.abs() < f32::EPSILON {
            (0.0, 0.0, 0.0)
        } else {
            let sum_fps: f32 = self
                .virtualization_samples
                .iter()
                .map(|sample| sample.scroll_fps)
                .sum();
            let sum_latency: f32 = self
                .virtualization_samples
                .iter()
                .map(|sample| sample.render_latency_ms)
                .sum();
            let peak_memory = self
                .virtualization_samples
                .iter()
                .fold(0.0, |acc, sample| acc.max(sample.memory_mib));
            (sum_fps / total, sum_latency / total, peak_memory)
        };

        VirtualizationBenchmarkSummary {
            rows: self.virtualization_rows,
            overscan: self.virtualization_overscan,
            avg_scroll_fps: (avg_fps * 10.0).round() / 10.0,
            avg_render_latency_ms: (avg_latency * 10.0).round() / 10.0,
            peak_memory_mib: (peak_memory * 10.0).round() / 10.0,
        }
    }

    fn editor_summary(&self) -> EditorBenchmarkSummary {
        let total = self.editor_samples.len() as f32;
        let (avg_typing, avg_lsp, peak_memory) = if total.abs() < f32::EPSILON {
            (0.0, 0.0, 0.0)
        } else {
            let sum_typing: f32 = self
                .editor_samples
                .iter()
                .map(|sample| sample.typing_latency_ms)
                .sum();
            let sum_lsp: f32 = self
                .editor_samples
                .iter()
                .map(|sample| sample.lsp_latency_ms)
                .sum();
            let peak_memory = self
                .editor_samples
                .iter()
                .fold(0.0, |acc, sample| acc.max(sample.memory_mib));
            (sum_typing / total, sum_lsp / total, peak_memory)
        };

        EditorBenchmarkSummary {
            lines: self.editor_lines,
            syntax_highlighting: self.editor_highlighting,
            lsp_enabled: self.editor_lsp,
            avg_typing_latency_ms: (avg_typing * 10.0).round() / 10.0,
            avg_lsp_latency_ms: (avg_lsp * 10.0).round() / 10.0,
            peak_memory_mib: (peak_memory * 10.0).round() / 10.0,
        }
    }

    fn synthesize_virtualization_sample(&mut self) -> VirtualizationSample {
        let frame = self.virtualization_samples.len();
        let load = self.virtualization_rows as f32 / 200_000.0;
        let overscan_factor = (self.virtualization_overscan.max(32) as f32) / 256.0;
        let density_factor = if self.virtualization_dense_layout {
            1.18
        } else {
            1.0
        };
        let base_fps = 160.0 / (1.0 + load * 1.65 + overscan_factor * 0.55 * density_factor);
        let jitter = ((frame % 11) as f32 * 0.35) - 1.1;
        let scroll_fps = (base_fps + jitter).clamp(18.0, 240.0);
        let render_latency_ms = (4.0 + load * 20.0 + overscan_factor * 3.8 + density_factor * 2.6)
            + (frame % 7) as f32 * 0.08;
        let memory_base = self.virtualization_rows as f32 * 0.000_32;
        let memory_mib =
            (memory_base * (1.0 + overscan_factor * 0.48) * density_factor).clamp(64.0, 4096.0);

        VirtualizationSample {
            frame,
            scroll_fps,
            render_latency_ms,
            memory_mib,
        }
    }

    fn synthesize_editor_sample(&mut self) -> EditorSample {
        let tick = self.editor_samples.len();
        let load = self.editor_lines as f32 / 200_000.0;
        let highlight_factor = if self.editor_highlighting { 1.24 } else { 0.82 };
        let lsp_factor = if self.editor_lsp { 1.35 } else { 0.68 };
        let typing_latency_ms = (6.0 + load * 21.0) * highlight_factor + (tick % 13) as f32 * 0.06;
        let lsp_latency_ms = (22.0 + load * 68.0) * lsp_factor + (tick % 9) as f32 * 0.09;
        let memory_mib = ((self.editor_lines as f32 * 0.000_45) * highlight_factor)
            + if self.editor_lsp { 220.0 } else { 80.0 };

        EditorSample {
            tick,
            typing_latency_ms,
            lsp_latency_ms,
            memory_mib: memory_mib.min(8_192.0),
        }
    }

    fn capture_virtualization_sample(&mut self) -> VirtualizationSample {
        let sample = self.synthesize_virtualization_sample();
        self.virtualization_samples.push(sample.clone());
        sample
    }

    fn capture_editor_sample(&mut self) -> EditorSample {
        let sample = self.synthesize_editor_sample();
        self.editor_samples.push(sample.clone());
        sample
    }

    fn run_full_suite(&mut self) -> BenchmarkRunRecord {
        self.virtualization_samples.clear();
        for _ in 0..120 {
            self.capture_virtualization_sample();
        }

        self.editor_samples.clear();
        for _ in 0..120 {
            self.capture_editor_sample();
        }

        self.run_counter = self.run_counter.saturating_add(1);
        let virtualization = self.virtualization_summary();
        let editor = self.editor_summary();
        let run = BenchmarkRunRecord {
            id: self.run_counter,
            recorded_at: Utc::now(),
            virtualization,
            editor,
        };
        self.history.push(run.clone());
        run
    }

    fn clear_history(&mut self) {
        self.history.clear();
        self.run_counter = 0;
    }

    fn set_history(&mut self, history: Vec<BenchmarkRunRecord>) {
        self.history = history;
        self.run_counter = self.history.last().map(|run| run.id).unwrap_or(0);
    }

    fn adjust_virtualization_rows(&mut self, rows: usize) {
        self.virtualization_rows = rows;
        self.virtualization_samples.clear();
    }

    fn adjust_virtualization_overscan(&mut self, overscan: usize) {
        self.virtualization_overscan = overscan;
        self.virtualization_samples.clear();
    }

    fn toggle_virtualization_density(&mut self, dense: bool) {
        self.virtualization_dense_layout = dense;
        self.virtualization_samples.clear();
    }

    fn adjust_editor_lines(&mut self, lines: usize) {
        self.editor_lines = lines.max(10_000).min(500_000);
        self.editor_samples.clear();
    }

    fn toggle_highlighting(&mut self, enabled: bool) {
        self.editor_highlighting = enabled;
        self.editor_samples.clear();
    }

    fn toggle_lsp(&mut self, enabled: bool) {
        self.editor_lsp = enabled;
        self.editor_samples.clear();
    }

    fn virtualization_series(&self) -> (Vec<(f32, f32)>, Vec<(f32, f32)>, Vec<(f32, f32)>) {
        let fps = self
            .virtualization_samples
            .iter()
            .map(|sample| (sample.frame as f32, sample.scroll_fps))
            .collect();
        let latency = self
            .virtualization_samples
            .iter()
            .map(|sample| (sample.frame as f32, sample.render_latency_ms))
            .collect();
        let memory = self
            .virtualization_samples
            .iter()
            .map(|sample| (sample.frame as f32, sample.memory_mib))
            .collect();
        (fps, latency, memory)
    }

    fn editor_series(&self) -> (Vec<(f32, f32)>, Vec<(f32, f32)>, Vec<(f32, f32)>) {
        let typing = self
            .editor_samples
            .iter()
            .map(|sample| (sample.tick as f32, sample.typing_latency_ms))
            .collect();
        let lsp = self
            .editor_samples
            .iter()
            .map(|sample| (sample.tick as f32, sample.lsp_latency_ms))
            .collect();
        let memory = self
            .editor_samples
            .iter()
            .map(|sample| (sample.tick as f32, sample.memory_mib))
            .collect();
        (typing, lsp, memory)
    }

    fn editor_preview(&self) -> String {
        let mut preview = String::new();
        let preview_lines = self.editor_lines.min(240);
        for line in 0..preview_lines {
            let _ = writeln!(
                preview,
                "// stress test line {:06} -- synthetic workload",
                line + 1
            );
            let _ = writeln!(
                preview,
                "fn hot_path_{line}() {{ let mut acc = {line}; acc += {} as i32; }}",
                self.editor_lines
            );
        }
        if self.editor_lines > preview_lines {
            preview.push_str("// … output truncated for preview …\n");
        }
        preview
    }
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
    performance: PerformanceState,
}

impl WorkbenchApp {
    fn new(
        theme_registry: ThemeRegistry,
        localization: LocalizationRegistry,
        command_bus: CommandBus<WorkbenchCommand>,
        workspace_config: WorkspaceConfig,
        config_store: ConfigStore,
    ) -> Self {
        let history = workspace_config.benchmark_runs.clone();
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
            performance: PerformanceState::new(history),
        };
        if let Some(state) = app.workspace_config.layout_state.clone() {
            app.apply_persisted_state(&state);
        }
        app.performance
            .set_history(app.workspace_config.benchmark_runs.clone());
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
                        "Performance" => WorkbenchTab::Performance,
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
                WorkbenchCommand::OpenDemos => {
                    self.selected_tab = WorkbenchTab::Demos;
                    self.open_demo_window(cx);
                    cx.notify();
                }
                WorkbenchCommand::OpenDemo(demo) => self.open_demo(demo, window, cx),
                WorkbenchCommand::ToggleTheme => {
                    self.cycle_theme(cx);
                }
                WorkbenchCommand::ToggleLocale => self.toggle_locale(cx),
                WorkbenchCommand::ShowPalette => self.open_palette(window, cx),
            }
        }
    }

    fn run_benchmark_suite(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let run = self.performance.run_full_suite();
        self.workspace_config.record_benchmark(run.clone());
        if let Err(err) = self.config_store.save(&self.workspace_config) {
            eprintln!("failed to persist benchmark run: {err}");
        }

        let message = format!(
            "Scroll {:.1} FPS • Typing {:.1} ms • LSP {:.1} ms",
            run.virtualization.avg_scroll_fps,
            run.editor.avg_typing_latency_ms,
            run.editor.avg_lsp_latency_ms
        );

        window.push_notification(
            Notification::new("Benchmark suite complete")
                .title("Benchmark suite complete")
                .content(move |_, _| Text::new(message.clone()).into_any_element())
                .with_type(NotificationType::Info),
            cx,
        );
        cx.notify();
    }

    fn clear_benchmark_history(&mut self, cx: &mut Context<Self>) {
        self.performance.clear_history();
        self.workspace_config.benchmark_runs.clear();
        if let Err(err) = self.config_store.save(&self.workspace_config) {
            eprintln!("failed to clear benchmark history: {err}");
        }
        cx.notify();
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

    fn open_demo(&self, demo: DemoSlug, window: &mut Window, cx: &mut Context<Self>) {
        match demo {
            DemoSlug::DataExplorer => {
                let registry = self.theme_registry.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: Some("Data Explorer".into()),
                        ..Default::default()
                    },
                    move |window, cx| {
                        window.set_title("Virtualized Data Explorer");
                        cx.new(|_| data_explorer::DataExplorerApp::new(registry.clone()))
                    },
                )
                .expect("data explorer demo window");
            }
            DemoSlug::MarkdownNotes => {
                let registry = self.theme_registry.clone();
                let store = self.config_store.clone();
                let config = self.workspace_config.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: Some("Markdown Notes".into()),
                        ..Default::default()
                    },
                    move |window, cx| {
                        window.set_title("Markdown & Notes Workspace");
                        cx.new(|_| {
                            markdown_notes::MarkdownNotesApp::new(
                                registry.clone(),
                                store.clone(),
                                config.clone(),
                            )
                        })
                    },
                )
                .expect("markdown notes demo window");
            }
            DemoSlug::CodePlayground => {
                let registry = self.theme_registry.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: Some("Code Playground".into()),
                        ..Default::default()
                    },
                    move |window, cx| {
                        window.set_title("Code Playground with LSP");
                        cx.new(|_| code_playground::CodePlaygroundApp::new(registry.clone()))
                    },
                )
                .expect("code playground demo window");
            }
            DemoSlug::OperationsDashboard => {
                let registry = self.theme_registry.clone();
                cx.open_window(
                    WindowOptions {
                        titlebar: Some("Operations Dashboard".into()),
                        ..Default::default()
                    },
                    move |window, cx| {
                        window.set_title("Operations Control Center");
                        cx.new(|_| dashboard::DashboardDemoApp::new(registry.clone()))
                    },
                )
                .expect("dashboard demo window");
            }
            DemoSlug::WebviewDocs => {
                if cfg!(feature = "webview") {
                    #[cfg(feature = "webview")]
                    {
                        let registry = self.theme_registry.clone();
                        cx.open_window(
                            WindowOptions {
                                titlebar: Some("Embedded Docs".into()),
                                ..Default::default()
                            },
                            move |window, cx| {
                                window.set_title("Webview Documentation");
                                cx.new(|_| webview_demo::app::WebviewDemoApp::new(registry.clone()))
                            },
                        )
                        .expect("webview demo window");
                    }
                } else {
                    window.push_notification(
                        Notification::new("Webview feature disabled")
                            .title("Webview demo unavailable")
                            .content(|_, _| {
                                Text::new(
                                    "Enable the `webview` feature flag and set FEATURE_WEBVIEW=1 to launch this demo.",
                                )
                                .into_any_element()
                            })
                            .with_type(NotificationType::Warning),
                        cx,
                    );
                }
            }
        }
    }

    fn open_demo_window(&self, cx: &mut Context<Self>) {
        let registry = self.theme_registry.clone();
        let bus = self.command_bus.clone();
        cx.open_window(
            WindowOptions {
                titlebar: Some("Demo Launchpad".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("Demo Launchpad");
                let view = DemoLauncher::new(registry.clone(), bus.clone());
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
                WorkbenchTab::Performance => {
                    self.render_performance_tab(window, cx).into_any_element()
                }
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
                WorkbenchTab::Performance => 3,
            })
            .on_click(cx.listener(|this, index, _, cx| {
                this.selected_tab = match *index {
                    0 => WorkbenchTab::Dashboard,
                    1 => WorkbenchTab::Gallery,
                    2 => WorkbenchTab::Demos,
                    _ => WorkbenchTab::Performance,
                };
                this.persist_state(cx);
                cx.notify();
            }))
            .children([
                Tab::new(self.translate(WorkbenchTab::Dashboard.label())).id("tab-dashboard"),
                Tab::new(self.translate(WorkbenchTab::Gallery.label())).id("tab-gallery"),
                Tab::new(self.translate(WorkbenchTab::Demos.label())).id("tab-demos"),
                Tab::new(self.translate(WorkbenchTab::Performance.label())).id("tab-performance"),
            ])
    }

    fn render_performance_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let history_count = self.performance.history.len();

        v_flex()
            .gap_5()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        Button::new("perf-run-suite")
                            .label("Run benchmark suite")
                            .icon(Icon::new(IconName::Timer))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.run_benchmark_suite(window, cx);
                            })),
                    )
                    .child(
                        Button::new("perf-clear-history")
                            .ghost()
                            .label("Clear history")
                            .icon(Icon::new(IconName::Trash))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_benchmark_history(cx);
                            })),
                    )
                    .child(
                        Text::new(format!("{} recorded runs", history_count))
                            .text_color(cx.theme().muted_foreground),
                    ),
            )
            .child(self.render_virtualization_panel(cx))
            .child(self.render_editor_panel(window, cx))
            .child(self.render_benchmark_history(cx))
            .child(self.render_performance_docs(cx))
    }

    fn render_virtualization_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = self.performance.virtualization_summary();
        let benchmark = self.performance.virtualization_benchmark();
        let (fps_series, latency_series, memory_series) = self.performance.virtualization_series();
        let latest = self.performance.virtualization_samples.last().cloned();

        let fps_chart = if fps_series.is_empty() {
            Text::new("Capture scroll samples to see FPS trends.")
                .text_color(cx.theme().muted_foreground)
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h(px(200.0))
                .child(
                    LineChart::new(fps_series.clone())
                        .x(|(frame, _)| *frame)
                        .y(|(_, value)| *value),
                )
                .into_any_element()
        };

        let latency_chart = if latency_series.is_empty() {
            Text::new("Latency samples appear once the benchmark runs.")
                .text_color(cx.theme().muted_foreground)
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h(px(200.0))
                .child(
                    LineChart::new(latency_series.clone())
                        .x(|(frame, _)| *frame)
                        .y(|(_, value)| *value),
                )
                .into_any_element()
        };

        let memory_chart = if memory_series.is_empty() {
            Text::new("Memory tracking is populated during a run.")
                .text_color(cx.theme().muted_foreground)
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h(px(200.0))
                .child(
                    LineChart::new(memory_series.clone())
                        .x(|(frame, _)| *frame)
                        .y(|(_, value)| *value),
                )
                .into_any_element()
        };

        let viewport_rows = benchmark.rows_per_viewport();
        let buffered = benchmark.suggested_buffer();
        let render_cost = benchmark.estimated_render_cost();

        DashboardCard::new("Virtualized list benchmark")
            .description("Simulates scroll performance against large GPUI lists and updates charts in real time.")
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_4()
                            .child(Text::new(format!("Avg FPS {:.1}", summary.avg_scroll_fps)).font_weight_semibold())
                            .child(Text::new(format!("Avg latency {:.1} ms", summary.avg_render_latency_ms)))
                            .child(Text::new(format!("Peak memory {:.1} MiB", summary.peak_memory_mib)))
                            .child(Text::new(format!("Rows {}", summary.rows)))
                            .child(Text::new(format!("Overscan {}", summary.overscan))),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("perf-rows-100k")
                                    .label("100k rows")
                                    .when(self.performance.virtualization_rows == 100_000, |btn| btn.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_virtualization_rows(100_000);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-rows-200k")
                                    .label("200k rows")
                                    .when(self.performance.virtualization_rows == 200_000, |btn| btn.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_virtualization_rows(200_000);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-rows-400k")
                                    .label("400k rows")
                                    .when(self.performance.virtualization_rows == 400_000, |btn| btn.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_virtualization_rows(400_000);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-overscan-128")
                                    .label("Overscan 128")
                                    .when(self.performance.virtualization_overscan == 128, |btn| btn.ghost())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_virtualization_overscan(128);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-overscan-256")
                                    .label("Overscan 256")
                                    .when(self.performance.virtualization_overscan == 256, |btn| btn.ghost())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_virtualization_overscan(256);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-capture-scroll")
                                    .ghost()
                                    .label("Capture scroll sample")
                                    .icon(Icon::new(IconName::Workflow))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.capture_virtualization_sample();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Switch::new("perf-dense-layout")
                                    .checked(self.performance.virtualization_dense_layout)
                                    .label(Text::new("Dense rows"))
                                    .on_click(cx.listener(|this, state, _, cx| {
                                        this.performance.toggle_virtualization_density(*state);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .children([fps_chart, latency_chart, memory_chart]),
                    )
                    .child(
                        Text::new(format!(
                            "Rows per viewport: {viewport_rows} • Suggested buffer: {buffered} • Estimated render cost: {render_cost}",
                        ))
                        .text_color(cx.theme().muted_foreground),
                    )
                    .when_some(latest, |col, sample| {
                        col.child(
                            Text::new(format!(
                                "Latest sample → {:.1} FPS / {:.1} ms / {:.1} MiB",
                                sample.scroll_fps, sample.render_latency_ms, sample.memory_mib
                            ))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                        )
                    }),
            )
    }

    fn render_editor_panel(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let summary = self.performance.editor_summary();
        let (typing_series, lsp_series, memory_series) = self.performance.editor_series();
        let latest = self.performance.editor_samples.last().cloned();
        let preview = self.performance.editor_preview();

        let typing_chart = if typing_series.is_empty() {
            Text::new("Typing latency samples appear during a run.")
                .text_color(cx.theme().muted_foreground)
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h(px(200.0))
                .child(
                    LineChart::new(typing_series.clone())
                        .x(|(frame, _)| *frame)
                        .y(|(_, value)| *value),
                )
                .into_any_element()
        };

        let lsp_chart = if lsp_series.is_empty() {
            Text::new("Language server latency populates after capturing samples.")
                .text_color(cx.theme().muted_foreground)
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h(px(200.0))
                .child(
                    LineChart::new(lsp_series.clone())
                        .x(|(frame, _)| *frame)
                        .y(|(_, value)| *value),
                )
                .into_any_element()
        };

        let memory_chart = if memory_series.is_empty() {
            Text::new("Memory impact becomes visible once the stress test runs.")
                .text_color(cx.theme().muted_foreground)
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h(px(200.0))
                .child(
                    LineChart::new(memory_series.clone())
                        .x(|(frame, _)| *frame)
                        .y(|(_, value)| *value),
                )
                .into_any_element()
        };

        DashboardCard::new("Editor stress test")
            .description("Loads a synthetic ~200k line buffer, tracks typing latency, LSP responsiveness, and memory consumption.")
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_4()
                            .child(Text::new(format!("Lines {}", summary.lines)).font_weight_semibold())
                            .child(Text::new(format!("Typing {:.1} ms", summary.avg_typing_latency_ms)))
                            .child(Text::new(format!("LSP {:.1} ms", summary.avg_lsp_latency_ms)))
                            .child(Text::new(format!("Peak memory {:.1} MiB", summary.peak_memory_mib)))
                            .child(
                                Text::new(format!(
                                    "Highlighting {} • LSP {}",
                                    if summary.syntax_highlighting { "on" } else { "off" },
                                    if summary.lsp_enabled { "on" } else { "off" }
                                ))
                                .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("perf-lines-100k")
                                    .label("100k lines")
                                    .when(self.performance.editor_lines == 100_000, |btn| btn.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_editor_lines(100_000);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-lines-200k")
                                    .label("200k lines")
                                    .when(self.performance.editor_lines == 200_000, |btn| btn.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_editor_lines(200_000);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-lines-400k")
                                    .label("400k lines")
                                    .when(self.performance.editor_lines == 400_000, |btn| btn.primary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.adjust_editor_lines(400_000);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("perf-capture-editor")
                                    .ghost()
                                    .label("Capture typing sample")
                                    .icon(Icon::new(IconName::Pen))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.performance.capture_editor_sample();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Switch::new("perf-highlight")
                                    .checked(self.performance.editor_highlighting)
                                    .label(Text::new("Syntax highlighting"))
                                    .on_click(cx.listener(|this, state, _, cx| {
                                        this.performance.toggle_highlighting(*state);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Switch::new("perf-lsp")
                                    .checked(self.performance.editor_lsp)
                                    .label(Text::new("LSP updates"))
                                    .on_click(cx.listener(|this, state, _, cx| {
                                        this.performance.toggle_lsp(*state);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .children([typing_chart, lsp_chart, memory_chart]),
                    )
                    .child(
                        render_snippet(
                            "perf-editor-preview",
                            format!(
                                "Synthetic buffer preview ({} lines, highlighting {})",
                                self.performance.editor_lines,
                                if self.performance.editor_highlighting { "on" } else { "off" }
                            ),
                            preview,
                            window,
                            cx,
                        ),
                    )
                    .when_some(latest, |col, sample| {
                        col.child(
                            Text::new(format!(
                                "Latest sample → typing {:.1} ms / LSP {:.1} ms / {:.1} MiB",
                                sample.typing_latency_ms,
                                sample.lsp_latency_ms,
                                sample.memory_mib
                            ))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground),
                        )
                    }),
            )
    }

    fn render_benchmark_history(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.performance.history.is_empty() {
            return DashboardCard::new("Benchmark history")
                .description("Run the suite to populate virtualization and editor comparisons.")
                .child(
                    Text::new("No historical runs have been recorded yet.")
                        .text_color(cx.theme().muted_foreground),
                );
        }

        let mut recent: Vec<_> = self
            .performance
            .history
            .iter()
            .cloned()
            .rev()
            .take(12)
            .collect();
        recent.reverse();

        let fps_history: Vec<(f32, f32)> = recent
            .iter()
            .enumerate()
            .map(|(idx, run)| (idx as f32 + 1.0, run.virtualization.avg_scroll_fps))
            .collect();
        let typing_history: Vec<(f32, f32)> = recent
            .iter()
            .enumerate()
            .map(|(idx, run)| (idx as f32 + 1.0, run.editor.avg_typing_latency_ms))
            .collect();
        let memory_history: Vec<(String, f32)> = recent
            .iter()
            .map(|run| (format!("Run {}", run.id), run.editor.peak_memory_mib))
            .collect();

        let fps_chart = div().flex_1().h(px(200.0)).child(
            LineChart::new(fps_history.clone())
                .x(|(sample, _)| *sample)
                .y(|(_, value)| *value),
        );

        let typing_chart = div().flex_1().h(px(200.0)).child(
            LineChart::new(typing_history.clone())
                .x(|(sample, _)| *sample)
                .y(|(_, value)| *value),
        );

        let memory_chart = div().flex_1().h(px(200.0)).child(
            BarChart::new(memory_history.clone())
                .x(|(label, _)| label.as_str())
                .y(|(_, value)| *value),
        );

        let latest = recent.last().cloned();

        let card = DashboardCard::new("Benchmark history")
            .description("Compare virtualization FPS, typing latency, and memory across runs.")
            .child(h_flex().gap_3().children([
                fps_chart.into_any_element(),
                typing_chart.into_any_element(),
                memory_chart.into_any_element(),
            ]));

        if let Some(run) = latest {
            card.child(
                Text::new(format!(
                    "Latest run {} → {:.1} FPS, {:.1} ms typing, {:.1} MiB peak",
                    run.id,
                    run.virtualization.avg_scroll_fps,
                    run.editor.avg_typing_latency_ms,
                    run.editor.peak_memory_mib
                ))
                .text_sm()
                .text_color(cx.theme().muted_foreground),
            )
        } else {
            card
        }
    }

    fn render_performance_docs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let methodology = vec![
            "Virtualization samples simulate scroll workloads using overscan, row height, and dataset size to stress the renderer.",
            "Rows/viewport and suggested buffer are derived from data::VirtualListBenchmark to keep heuristics aligned with component guidance.",
            "Editor metrics synthesize keystroke and LSP timings while highlighting how feature toggles influence resource usage.",
        ];
        let practices = vec![
            "Prefer fixed row heights when possible; it keeps virtualization math predictable and minimizes render churn.",
            "Defer expensive formatting until rows become visible—gpui-component list cells can lazily hydrate detail views.",
            "Throttle LSP updates for large files by batching edits; the stress test shows how latency spikes with eager syncs.",
        ];
        let limitations = vec![
            "Metrics are synthesized inside the demo environment and should be calibrated against production telemetry.",
            "Synthetic editor previews truncate after a few hundred lines to avoid overwhelming the UI renderer.",
            "GPU timing data is approximated; attach tracy or wgpu capture tooling for hardware-level investigations.",
        ];

        Accordion::new("performance-docs")
            .bordered(false)
            .item(|item| {
                item.title(Text::new("Methodology")).content(
                    v_flex().gap_2().children(
                        methodology
                            .into_iter()
                            .map(|entry| Text::new(entry).text_sm().into_any_element()),
                    ),
                )
            })
            .item(|item| {
                item.title(Text::new("Optimization practices")).content(
                    v_flex().gap_2().children(
                        practices
                            .into_iter()
                            .map(|entry| Text::new(entry).text_sm().into_any_element()),
                    ),
                )
            })
            .item(|item| {
                item.title(Text::new("Known limitations")).content(
                    v_flex().gap_2().children(
                        limitations
                            .into_iter()
                            .map(|entry| Text::new(entry).text_sm().into_any_element()),
                    ),
                )
            })
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
                        SidebarMenuItem::new("Performance lab")
                            .icon(Icon::new(IconName::Activity))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.selected_tab = WorkbenchTab::Performance;
                                this.persist_state(cx);
                                cx.notify();
                            })),
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
                        &[
                            "Keep categorical labels short and rotate them when rendering dozens of buckets.",
                            "Group related series by stacking bars instead of overlaying multiple charts.",
                        ],
                        &[
                            "Do not mix absolute and percentage counts in the same view—normalize first.",
                        ],
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
                        &[
                            "Layer cumulative metrics in declaration order so the legend matches the stack.",
                            "Use semi-transparent fills to keep gridlines readable behind the area.",
                        ],
                        &[
                            "Baseline drift hides negative trends—explicitly set `.baseline(0.0)` for balanced datasets.",
                        ],
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
                        &[
                            "Limit the slice count to highlight the top contributors and aggregate the rest into an 'Other'.",
                            "Pair pies with numeric totals so screen readers can describe the distribution.",
                        ],
                        &[
                            "Avoid pie charts for negative values—prefer a stacked bar or waterfall view.",
                        ],
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
                        &[
                            "Persist wizard progress in `WizardState` so the UI survives window refreshes.",
                            "Guard every transition with validation and return early when the user corrects inputs.",
                        ],
                        &[
                            "Forgetting to reset `wizard.error` leaves stale messages on the next step.",
                        ],
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
                        &[
                            "Keep filter state in a dedicated struct so resets can reuse `Default::default()`.",
                            "Disable expensive apply buttons until all required options are selected.",
                        ],
                        &[
                            "Remember to debounce network-bound filters—spamming apply floods telemetry.",
                        ],
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
                &[
                    "Group notifications by domain key to avoid spamming the user with duplicates.",
                    "Render actionable links inside the toast content so users can resolve issues inline.",
                ],
                &[
                    "Always invoke `window.push_notification` from the UI thread—background threads must use the command bus.",
                ],
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
                &[
                    "Call `persist_state` after every user-driven mutation so the config store stays fresh.",
                    "Keep the serialized snapshot human readable to simplify support diagnostics.",
                ],
                &[
                    "Large layouts can exceed default config size limits—compress or rotate files when they grow beyond a few hundred KB.",
                ],
                window,
                cx,
            ))
            .child(expandable_docs(
                "commands-docs",
                "Command bus",
                r#"command_bus.publish(WorkbenchCommand::ResetLayout);"#,
                &[
                    "Create a dedicated enum for each subsystem so command payloads stay type safe.",
                    "Publish commands instead of calling UI methods directly to keep business logic testable.",
                ],
                &[
                    "Dropping the last receiver breaks broadcasts—store handles in your root app and refresh them on hot reload.",
                ],
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
                &[
                    "Clone the existing theme registry so gallery and workbench stay color-synchronized.",
                    "Pipe localization handles through to keep category labels translated.",
                ],
                &[
                    "Avoid opening multiple preview windows without tracking handles—each consumes GPU resources.",
                ],
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
    cx.new(|_| DemoLauncher::new(theme_registry.clone(), command_bus.clone()))
});"#,
                &[
                    "Reuse the global command bus so launcher buttons trigger the host workbench.",
                    "Pass cloned theme registries to keep demo windows aligned with the active palette.",
                ],
                &[
                    "Forgetting to clone the command bus will move it into the closure and break other subscribers.",
                ],
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
                &[
                    "Install the registry once at startup and clone handles for additional windows.",
                    "Share localization state so component labels match the active language.",
                ],
                &[
                    "Forgetting to call `gpui_component::init` before cloning registries leaves previews without component styles.",
                ],
                window,
                cx,
            ))
    }
}

struct DemoLauncher {
    theme_registry: ThemeRegistry,
    command_bus: CommandBus<WorkbenchCommand>,
}

impl DemoLauncher {
    fn new(theme_registry: ThemeRegistry, command_bus: CommandBus<WorkbenchCommand>) -> Self {
        Self {
            theme_registry,
            command_bus,
        }
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
                    .child(
                        Button::new("notes")
                            .label("Markdown notes")
                            .on_click(cx.listener(|this, _, _, _| {
                                this
                                    .command_bus
                                    .publish(WorkbenchCommand::OpenDemo(DemoSlug::MarkdownNotes));
                            })),
                    )
                    .child(
                        Button::new("analytics")
                            .label("Operations dashboard")
                            .on_click(cx.listener(|this, _, _, _| {
                                this
                                    .command_bus
                                    .publish(WorkbenchCommand::OpenDemo(DemoSlug::OperationsDashboard));
                            })),
                    )
                    .child(
                        Button::new("explorer")
                            .label("Data explorer")
                            .on_click(cx.listener(|this, _, _, _| {
                                this
                                    .command_bus
                                    .publish(WorkbenchCommand::OpenDemo(DemoSlug::DataExplorer));
                            })),
                    )
                    .child(
                        Button::new("playground")
                            .label("Code playground")
                            .on_click(cx.listener(|this, _, _, _| {
                                this
                                    .command_bus
                                    .publish(WorkbenchCommand::OpenDemo(DemoSlug::CodePlayground));
                            })),
                    )
                    .child(
                        Button::new("docs")
                            .label("Embedded docs")
                            .on_click(cx.listener(|this, _, _, _| {
                                this
                                    .command_bus
                                    .publish(WorkbenchCommand::OpenDemo(DemoSlug::WebviewDocs));
                            })),
                    ),
            )
            .child(expandable_docs(
                "demo-launcher-docs",
                "Launch pattern",
                r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Markdown Notes");
    cx.new(|_| DemoLauncher::new(theme_registry.clone(), command_bus.clone()))
});"#,
                &[
                    "Reuse the existing workbench command bus so launcher buttons can trigger shared actions.",
                    "Clone registries and stores before entering the closure to avoid moving them.",
                ],
                &[
                    "Opening launchers without cloning the bus will panic because the bus does not implement Copy.",
                ],
                window,
                cx,
            ))
    }
}

fn expandable_docs(
    id: &str,
    title: impl Into<SharedString>,
    code: &str,
    best_practices: &[&str],
    gotchas: &[&str],
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
        .content(
            v_flex()
                .gap_3()
                .child(render_snippet(snippet_id, "Snippet", code, window, cx))
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
                    col.child(GroupBox::new().title(Text::new("Gotchas")).child(
                        v_flex().gap_2().children(gotchas.iter().map(|tip| {
                            Text::new(*tip)
                                .text_color(cx.theme().danger)
                                .into_any_element()
                        })),
                    ))
                }),
        )
    })
}
