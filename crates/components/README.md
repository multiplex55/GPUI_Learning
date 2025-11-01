# Components Crate

Opinionated wrappers built on top of [`gpui-component`] primitives. Each
component is documented with small usage snippets and is designed to pair with
`designsystem`'s tokens and theme registry.

## Example

```no_run
use components::{CommandPaletteTrigger, DashboardCard, DockLayoutPanel, KpiGrid, KpiMetric, ThemeSwitch};
use designsystem::{IconLoader, IconName, ThemeRegistry};
use gpui::Application;

let app = Application::headless()
    .with_assets(IconLoader::asset_source());
app.run(|cx| {
    let registry = ThemeRegistry::new();
    registry.install(cx);

    let layout = DockLayoutPanel::default()
        .sidebar(Text::new("Navigation"))
        .toolbar(CommandPaletteTrigger::new("cmd", "Command Palette", Some("ctrl-k"), IconName::Search))
        .child(
            DashboardCard::new("Revenue")
                .icon(IconName::Activity)
                .description("MTD vs last month")
                .child(Text::new("$82.4k"))
        )
        .child(
            KpiGrid::default()
                .push(KpiMetric::new("124", "New Users").icon(IconName::Users))
                .push(KpiMetric::new("36%", "Conversion").trend("+4.3%"))
        )
        .child(ThemeSwitch::new("theme", registry.clone()));

    cx.new(|_, _| layout);
});
```

[`gpui-component`]: https://crates.io/crates/gpui-component
