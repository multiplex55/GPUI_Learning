#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

pub mod docs;

use designsystem::{IconLoader, IconName, ThemeRegistry, ThemeVariant};
use gpui::{
    platform::keystroke::Keystroke, prelude::FluentBuilder as _, px, AnyElement, App, IntoElement,
    ParentElement, RenderOnce, SharedString, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    group_box::GroupBox,
    kbd::Kbd,
    styled::{h_flex, v_flex, StyledExt as _},
    switch::Switch,
    text::Text,
    Icon,
};
use smallvec::SmallVec;

/// Card component tailored for dashboard summaries.
#[derive(Default, IntoElement)]
pub struct DashboardCard {
    title: SharedString,
    description: Option<SharedString>,
    icon: Option<IconName>,
    actions: SmallVec<[AnyElement; 2]>,
    body: SmallVec<[AnyElement; 4]>,
}

impl DashboardCard {
    /// Creates a new card with the provided title.
    #[must_use]
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Adds supporting copy below the title.
    #[must_use]
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Displays the given icon in the card header.
    #[must_use]
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Appends a leading action (for example a link or menu trigger).
    #[must_use]
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl ParentElement for DashboardCard {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(children);
    }
}

impl RenderOnce for DashboardCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> AnyElement {
        let title_block = v_flex()
            .gap_1()
            .child(Text::new(self.title.clone()).size(18.0))
            .when_some(self.description.clone(), |col, description| {
                col.child(
                    Text::new(description)
                        .text_color(cx.theme().muted_foreground)
                        .size(13.0),
                )
            });

        let header = h_flex()
            .gap_3()
            .items_center()
            .when_some(self.icon, |row, icon| {
                row.child(
                    Icon::default()
                        .path(IconLoader::asset_path(icon))
                        .size(px(20.0)),
                )
            })
            .child(title_block)
            .when(!self.actions.is_empty(), |row| {
                row.child(h_flex().gap_2().ml_auto().children(self.actions))
            });

        GroupBox::new()
            .fill()
            .title(header)
            .child(v_flex().gap_3().children(self.body))
            .into_any_element()
    }
}

/// Metric descriptor rendered inside a [`KpiGrid`].
#[derive(Debug, Clone)]
pub struct KpiMetric {
    /// Human readable label.
    pub label: SharedString,
    /// Highlighted value string.
    pub value: SharedString,
    /// Optional supplemental text (e.g. delta).
    pub trend: Option<SharedString>,
    /// Optional icon displayed above the value.
    pub icon: Option<IconName>,
}

impl KpiMetric {
    /// Constructs a metric with value and label.
    #[must_use]
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            trend: None,
            icon: None,
        }
    }

    /// Annotates the metric with a delta or time range string.
    #[must_use]
    pub fn trend(mut self, trend: impl Into<SharedString>) -> Self {
        self.trend = Some(trend.into());
        self
    }

    /// Adds an icon to the KPI block.
    #[must_use]
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// Responsive grid for arranging KPI blocks.
#[derive(Default, IntoElement)]
pub struct KpiGrid {
    metrics: SmallVec<[KpiMetric; 4]>,
}

impl KpiGrid {
    /// Pushes a new metric into the grid.
    #[must_use]
    pub fn push(mut self, metric: KpiMetric) -> Self {
        self.metrics.push(metric);
        self
    }
}

impl RenderOnce for KpiGrid {
    fn render(self, _window: &mut Window, cx: &mut App) -> AnyElement {
        let cells = self
            .metrics
            .into_iter()
            .map(|metric| {
                let mut block = v_flex().gap_1();
                if let Some(icon) = metric.icon {
                    block = block.child(
                        Icon::default()
                            .path(IconLoader::asset_path(icon))
                            .size_5()
                            .text_color(cx.theme().accent),
                    );
                }
                block
                    .child(Text::new(metric.value).size(22.0).font_weight_bold())
                    .child(Text::new(metric.label).text_color(cx.theme().muted_foreground))
                    .when_some(metric.trend, |col, trend| {
                        col.child(Text::new(trend).text_color(cx.theme().accent))
                    })
            })
            .map(IntoElement::into_any_element)
            .collect();

        h_flex()
            .flex_wrap()
            .gap_6()
            .child(v_flex().gap_4().children(cells))
            .into_any_element()
    }
}

/// High level dock-like layout with a sidebar and primary panel.
#[derive(Default, IntoElement)]
pub struct DockLayoutPanel {
    sidebar: SmallVec<[AnyElement; 2]>,
    toolbar: SmallVec<[AnyElement; 2]>,
    content: SmallVec<[AnyElement; 4]>,
}

impl DockLayoutPanel {
    /// Adds an element to the vertical sidebar strip.
    #[must_use]
    pub fn sidebar(mut self, element: impl IntoElement) -> Self {
        self.sidebar.push(element.into_any_element());
        self
    }

    /// Adds a toolbar element above the main content.
    #[must_use]
    pub fn toolbar(mut self, element: impl IntoElement) -> Self {
        self.toolbar.push(element.into_any_element());
        self
    }
}

impl ParentElement for DockLayoutPanel {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(children);
    }
}

impl RenderOnce for DockLayoutPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> AnyElement {
        h_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_3()
                    .w(px(220.0))
                    .bg(cx.theme().muted)
                    .p_4()
                    .rounded(cx.theme().radius)
                    .children(self.sidebar),
            )
            .child(
                v_flex()
                    .gap_4()
                    .flex_1()
                    .child(h_flex().gap_2().when(!self.toolbar.is_empty(), |row| {
                        row.children(self.toolbar.clone())
                    }))
                    .child(
                        v_flex()
                            .gap_4()
                            .bg(cx.theme().popover)
                            .p_6()
                            .rounded(cx.theme().radius_lg)
                            .children(self.content),
                    ),
            )
            .into_any_element()
    }
}

/// A themed switch that flips between the light and dark modes.
#[derive(IntoElement)]
pub struct ThemeSwitch {
    id: SharedString,
    registry: ThemeRegistry,
    label: Option<SharedString>,
}

impl ThemeSwitch {
    /// Creates a new switch bound to the provided registry handle.
    #[must_use]
    pub fn new(id: impl Into<SharedString>, registry: ThemeRegistry) -> Self {
        Self {
            id: id.into(),
            registry,
            label: None,
        }
    }

    /// Renders the label next to the switch.
    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for ThemeSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let registry = self.registry.clone();
        let checked = !matches!(registry.active(), ThemeVariant::Light);

        Switch::new(self.id.clone())
            .checked(checked)
            .when_some(self.label.clone(), |s, label| s.label(Text::new(label)))
            .on_click(move |state, window, cx| {
                let variant = if *state {
                    ThemeVariant::Dark
                } else {
                    ThemeVariant::Light
                };
                registry.apply(variant, cx);
                window.refresh();
            })
            .tooltip("Toggle workspace theme")
            .build(window, cx)
    }
}

/// Button styled trigger that opens a command palette.
#[derive(IntoElement)]
pub struct CommandPaletteTrigger {
    id: SharedString,
    label: SharedString,
    shortcut: Option<SharedString>,
    icon: IconName,
}

impl CommandPaletteTrigger {
    /// Creates a new trigger with label, shortcut hint and icon.
    #[must_use]
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        shortcut: Option<impl Into<SharedString>>,
        icon: IconName,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: shortcut.map(Into::into),
            icon,
        }
    }
}

impl RenderOnce for CommandPaletteTrigger {
    fn render(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let mut button = Button::new(self.id.clone())
            .ghost()
            .icon(
                Icon::default()
                    .path(IconLoader::asset_path(self.icon))
                    .text_color(cx.theme().foreground),
            )
            .label(self.label.clone());

        if let Some(shortcut) = self.shortcut.clone() {
            if let Ok(key) = Keystroke::parse(&shortcut) {
                button = button.child(
                    h_flex()
                        .gap_1()
                        .ml_3()
                        .child(Text::new("/"))
                        .child(Kbd::new(key).appearance(true)),
                );
            }
        }

        button.build(window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_card_accepts_children() {
        let mut card = DashboardCard::new("Title");
        card.extend([Text::new("Body").into_any_element()]);
        assert_eq!(card.body.len(), 1);
    }

    #[test]
    fn kpi_metric_builder() {
        let metric = KpiMetric::new("123", "Orders").trend("+8%");
        assert_eq!(metric.trend.unwrap(), "+8%".into());
    }
}
