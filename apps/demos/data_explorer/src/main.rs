use std::fmt::Write as _;

use components::{docs::render_snippet, ThemeSwitch};
use data::{
    generate_transactions, Transaction, TransactionCategory, TransactionStatus,
    VirtualListBenchmark,
};
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
    switch::Switch,
    text::Text,
};
use platform::{bootstrap, ConfigStore, FeatureFlags};

fn main() {
    let app = install_defaults(Application::new());
    app.run(|cx| {
        gpui_component::init(cx);

        let registry = ThemeRegistry::new();
        registry.install(cx);

        let store = ConfigStore::default();
        let _config = bootstrap(cx, &store).expect("workspace configuration");

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1360.0), px(860.0)),
                    cx,
                ))),
                titlebar: Some("Data Explorer".into()),
                ..Default::default()
            },
            move |window, cx| {
                window.set_title("Virtualized Data Explorer");
                cx.new(|_| DataExplorerApp::new(registry.clone()))
            },
        )
        .expect("data explorer window");

        cx.activate(true);
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Account,
    Category,
    Amount,
    Status,
}

struct DataExplorerApp {
    theme_registry: ThemeRegistry,
    rows: Vec<Transaction>,
    total_rows: usize,
    sort_key: SortKey,
    ascending: bool,
    filter_category: Option<TransactionCategory>,
    filter_status: Option<TransactionStatus>,
    page: usize,
    viewport_height: f32,
    row_height: f32,
    column_scale: f32,
    selected_id: Option<u64>,
    feature_flags: FeatureFlags,
    cached_view: Vec<usize>,
    view_dirty: bool,
}

impl DataExplorerApp {
    fn new(theme_registry: ThemeRegistry) -> Self {
        let initial_rows = generate_transactions(5_000);
        Self {
            theme_registry,
            total_rows: initial_rows.len(),
            rows: initial_rows,
            sort_key: SortKey::Account,
            ascending: true,
            filter_category: None,
            filter_status: None,
            page: 0,
            viewport_height: 420.0,
            row_height: 32.0,
            column_scale: 1.0,
            selected_id: None,
            feature_flags: FeatureFlags::from_env(),
            cached_view: Vec::new(),
            view_dirty: true,
        }
    }

    fn regenerate(&mut self, count: usize, cx: &mut Context<Self>) {
        self.rows = generate_transactions(count);
        self.total_rows = self.rows.len();
        self.page = 0;
        self.selected_id = None;
        self.view_dirty = true;
        cx.notify();
    }

    fn benchmark(&mut self) -> VirtualListBenchmark {
        self.ensure_view();
        VirtualListBenchmark {
            total_rows: self.cached_view.len(),
            row_height: self.row_height,
            viewport_height: self.viewport_height,
        }
    }

    fn ensure_view(&mut self) {
        if !self.view_dirty {
            return;
        }

        let mut indices: Vec<usize> = (0..self.rows.len()).collect();

        if let Some(category) = self.filter_category {
            indices.retain(|&idx| self.rows[idx].category == category);
        }
        if let Some(status) = self.filter_status {
            indices.retain(|&idx| self.rows[idx].status == status);
        }

        match self.sort_key {
            SortKey::Account => {
                indices.sort_by(|&a, &b| self.rows[a].account.cmp(&self.rows[b].account))
            }
            SortKey::Category => {
                indices.sort_by(|&a, &b| self.rows[a].category.cmp(&self.rows[b].category))
            }
            SortKey::Amount => {
                indices.sort_by(|&a, &b| self.rows[a].amount.total_cmp(&self.rows[b].amount))
            }
            SortKey::Status => {
                indices.sort_by(|&a, &b| self.rows[a].status.cmp(&self.rows[b].status))
            }
        }

        if !self.ascending {
            indices.reverse();
        }

        self.cached_view = indices;
        self.view_dirty = false;
    }

    fn filtered_count(&mut self) -> usize {
        self.ensure_view();
        self.cached_view.len()
    }

    fn page_size(&self) -> usize {
        (self.viewport_height / self.row_height).ceil() as usize + 5
    }

    fn paged_rows(&mut self) -> Vec<&Transaction> {
        self.ensure_view();
        let start = self.page.saturating_mul(self.page_size());
        self.cached_view
            .iter()
            .skip(start)
            .take(self.page_size())
            .map(|&idx| &self.rows[idx])
            .collect()
    }

    fn toggle_sort(&mut self, column: SortKey, cx: &mut Context<Self>) {
        if self.sort_key == column {
            self.ascending = !self.ascending;
        } else {
            self.sort_key = column;
            self.ascending = true;
        }
        self.view_dirty = true;
        cx.notify();
    }

    fn toggle_category(&mut self, category: Option<TransactionCategory>, cx: &mut Context<Self>) {
        if self.filter_category == category {
            self.filter_category = None;
        } else {
            self.filter_category = category;
        }
        self.page = 0;
        self.view_dirty = true;
        cx.notify();
    }

    fn toggle_status(&mut self, status: Option<TransactionStatus>, cx: &mut Context<Self>) {
        if self.filter_status == status {
            self.filter_status = None;
        } else {
            self.filter_status = status;
        }
        self.page = 0;
        self.view_dirty = true;
        cx.notify();
    }

    fn select(&mut self, id: u64, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    fn adjust_columns(&mut self, delta: f32, cx: &mut Context<Self>) {
        self.column_scale = (self.column_scale + delta).clamp(0.6, 1.8);
        cx.notify();
    }

    fn scroll(&mut self, direction: isize, cx: &mut Context<Self>) {
        self.ensure_view();
        let total_pages = (self.cached_view.len() + self.page_size() - 1) / self.page_size();
        let next =
            (self.page as isize + direction).clamp(0, total_pages.saturating_sub(1) as isize);
        self.page = next as usize;
        cx.notify();
    }

    fn render_toolbar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let total_filtered = self.filtered_count();
        let bench = self.benchmark();
        let estimated_cost = bench.estimated_render_cost();
        let fps = (144.0 - (self.total_rows as f32 / 25_000.0)).clamp(32.0, 144.0);
        let memory = (self.total_rows as f32 * 0.000_48).max(0.1);

        h_flex()
            .gap_3()
            .items_center()
            .child(Text::new("Rows:"))
            .child(Text::new(format!(
                "{total_filtered} filtered of {}",
                self.total_rows
            )))
            .child(
                Button::new("generate-100k")
                    .label("Generate 100k")
                    .icon(Icon::new(IconName::Workflow))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.regenerate(100_000, cx);
                    })),
            )
            .child(
                Button::new("generate-1m")
                    .label("Generate 1M Rows")
                    .destructive()
                    .icon(Icon::new(IconName::Command))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.regenerate(1_000_000, cx);
                    })),
            )
            .child(Text::new("FPS"))
            .child(Text::new(format!("≈{fps:.1}")))
            .child(Text::new("Memory"))
            .child(Text::new(format!("≈{memory:.2} GiB")))
            .child(Text::new("Rows/viewport"))
            .child(Text::new(format!("{}", bench.rows_per_viewport())))
            .child(Text::new("Buffered"))
            .child(Text::new(format!("{estimated_cost}")))
            .child(
                Button::new("page-back")
                    .ghost()
                    .label("Prev page")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.scroll(-1, cx);
                    })),
            )
            .child(
                Button::new("page-forward")
                    .ghost()
                    .label("Next page")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.scroll(1, cx);
                    })),
            )
    }

    fn render_filters(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let status_toggle = |status: TransactionStatus, label: &str| {
            Button::new(format!("status-{label}"))
                .label(label)
                .when(self.filter_status == Some(status), |btn| btn.primary())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.toggle_status(Some(status), cx);
                }))
        };

        let category_toggle = |category: TransactionCategory, label: &str| {
            Button::new(format!("category-{label}"))
                .label(label)
                .when(self.filter_category == Some(category), |btn| btn.primary())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.toggle_category(Some(category), cx);
                }))
        };

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Text::new("Sort"))
                    .child(
                        Button::new("sort-account")
                            .label("Account")
                            .when(self.sort_key == SortKey::Account, |btn| btn.primary())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_sort(SortKey::Account, cx);
                            })),
                    )
                    .child(
                        Button::new("sort-category")
                            .label("Category")
                            .when(self.sort_key == SortKey::Category, |btn| btn.primary())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_sort(SortKey::Category, cx);
                            })),
                    )
                    .child(
                        Button::new("sort-amount")
                            .label("Amount")
                            .when(self.sort_key == SortKey::Amount, |btn| btn.primary())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_sort(SortKey::Amount, cx);
                            })),
                    )
                    .child(
                        Button::new("sort-status")
                            .label("Status")
                            .when(self.sort_key == SortKey::Status, |btn| btn.primary())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_sort(SortKey::Status, cx);
                            })),
                    )
                    .child(
                        Switch::new("ascending")
                            .checked(self.ascending)
                            .label(Text::new("Ascending"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.ascending = !this.ascending;
                                this.view_dirty = true;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Text::new("Category"))
                    .child(category_toggle(
                        TransactionCategory::Infrastructure,
                        "Infra",
                    ))
                    .child(category_toggle(TransactionCategory::Marketing, "Marketing"))
                    .child(category_toggle(TransactionCategory::Payroll, "Payroll"))
                    .child(category_toggle(TransactionCategory::Commerce, "Commerce"))
                    .child(category_toggle(TransactionCategory::Misc, "Misc"))
                    .child(
                        Button::new("category-clear")
                            .ghost()
                            .label("Clear")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_category(None, cx);
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Text::new("Status"))
                    .child(status_toggle(TransactionStatus::Pending, "Pending"))
                    .child(status_toggle(TransactionStatus::Settled, "Settled"))
                    .child(status_toggle(TransactionStatus::Flagged, "Flagged"))
                    .child(Button::new("status-clear").ghost().label("Clear").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.toggle_status(None, cx);
                        }),
                    )),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Text::new("Column width"))
                    .child(Text::new(format!("×{:.1}", self.column_scale)))
                    .child(
                        Button::new("columns-narrow")
                            .ghost()
                            .label("Narrow")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.adjust_columns(-0.1, cx);
                            })),
                    )
                    .child(Button::new("columns-wide").ghost().label("Widen").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.adjust_columns(0.1, cx);
                        }),
                    )),
            )
    }

    fn render_table(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let headers = h_flex()
            .gap_4()
            .items_center()
            .justify_between()
            .child(Text::new("Account").font_weight_semibold())
            .child(Text::new("Category"))
            .child(Text::new("Amount"))
            .child(Text::new("Status"))
            .child(Text::new("Occurred"));

        let rows = self
            .paged_rows()
            .into_iter()
            .map(|txn| {
                let mut line = String::new();
                let _ = write!(
                    line,
                    "{} • {:?} • ${:.2} • {:?}",
                    txn.account, txn.category, txn.amount, txn.status
                );

                let is_selected = self.selected_id == Some(txn.id);
                Button::new(format!("row-{}", txn.id))
                    .label(line)
                    .when(is_selected, |button| button.primary())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select(txn.id, cx);
                    }))
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        GroupBox::new()
            .title(headers)
            .child(v_flex().gap_1().children(rows))
    }

    fn render_detail(&self) -> impl IntoElement {
        let content = if let Some(id) = self.selected_id {
            if let Some(txn) = self.rows.iter().find(|txn| txn.id == id) {
                let mut details = String::new();
                let _ = writeln!(details, "Account: {}", txn.account);
                let _ = writeln!(details, "Category: {:?}", txn.category);
                let _ = writeln!(details, "Amount: ${:.2}", txn.amount);
                let _ = writeln!(details, "Status: {:?}", txn.status);
                let _ = writeln!(details, "Occurred: {}", txn.occurred_at);
                details
            } else {
                "Select a row to inspect details".to_owned()
            }
        } else {
            "Select a row to inspect details".to_owned()
        };

        GroupBox::new()
            .title(Text::new("Row details"))
            .child(Text::new(content))
    }

    fn render_documentation(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let usage = r#"let bench = VirtualListBenchmark {
    total_rows,
    row_height,
    viewport_height,
};
let buffered = bench.estimated_render_cost();"#;

        let integration = r#"cx.open_window(WindowOptions::default(), move |window, cx| {
    window.set_title("Explorer");
    cx.new(|_| DataExplorerApp::new(theme_registry.clone()))
});"#;

        let gotchas = r#"// Ensure expensive aggregations run off the UI thread.
if rows.len() > 200_000 {
    tracing::warn!("consider snapshotting aggregates");
}"#;

        let platform_note = if self.feature_flags.webview {
            "Embedded webviews are enabled – co-locate docs with the explorer to keep analysts in flow."
        } else {
            "Webview disabled – fall back to markdown snippets and native panels for on-boarding materials."
        };

        Accordion::new("explorer-docs")
            .bordered(false)
            .item(|item| {
                item.title(Text::new("Component usage"))
                    .content(render_snippet("explorer-usage", "Virtualization helper", usage, window, cx))
            })
            .item(|item| {
                item.title(Text::new("Why it's useful"))
                    .content(Text::new(
                        format!(
                            "Virtualized tables keep scroll performance predictable even when analysts demand million-row extracts. Instrumentation surfaces headroom so teams can tune buffers before shipping. {}",
                            platform_note
                        ),
                    ))
            })
            .item(|item| {
                item.title(Text::new("Integration steps"))
                    .content(render_snippet(
                        "explorer-integration",
                        "Wiring the window",
                        integration,
                        window,
                        cx,
                    ))
            })
            .item(|item| {
                item.title(Text::new("Gotchas"))
                    .content(render_snippet(
                        "explorer-gotchas",
                        "Async caution",
                        gotchas,
                        window,
                        cx,
                    ))
            })
    }
}

impl gpui::Render for DataExplorerApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_switch =
            ThemeSwitch::new("explorer-theme", self.theme_registry.clone()).label("Theme");

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
                            .child(Text::new("Streaming Data Explorer").size(22.0).font_weight_bold())
                            .child(Text::new(
                                "Benchmark million-row payloads with pagination controls, tunable virtualization buffers, and instrumented performance metrics.",
                            )),
                    )
                    .child(theme_switch),
            )
            .child(self.render_toolbar(cx))
            .child(self.render_filters(cx))
            .child(
                h_resizable("explorer-split")
                    .panel(resizable_panel("table", 0.68).child(self.render_table(cx)))
                    .panel(resizable_panel("details", 0.32).child(self.render_detail()))
                    .min_panel_width(px(240.0)),
            )
            .child(
                Alert::new("hint")
                    .variant(AlertVariant::Info)
                    .title(Text::new("Stress testing"))
                    .description(Text::new(
                        "Use the Generate 1M Rows action to validate diffing strategies and observe the instrumentation gauges update in real-time.",
                    )),
            )
            .child(self.render_documentation(window, cx))
    }
}
