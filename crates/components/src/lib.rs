#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

use designsystem::{IconLoader, IconName, ThemeRegistry, ThemeVariant};
use gpui::{
    platform::keystroke::Keystroke, prelude::FluentBuilder as _, px, AnyElement, App, IntoElement,
    ParentElement, SharedString, Window,
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
#[derive(Default)]
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

impl IntoElement for DashboardCard {
    type Element = DashboardCardElement;

    fn into_element(self) -> Self::Element {
        DashboardCardElement(self)
    }
}

/// Adapter so that [`DashboardCard`] can implement [`RenderOnce`].
pub struct DashboardCardElement(DashboardCard);

gpui::impl_render_once!(DashboardCardElement);

gpui::impl_widget_render!(
    DashboardCardElement,
    |this: DashboardCardElement, _window: &mut Window, cx: &mut App| {
        let card = this.0;
        let title_block = v_flex()
            .gap_1()
            .child(Text::new(card.title.clone()).size(18.0))
            .when_some(card.description.clone(), |col, description| {
                col.child(
                    Text::new(description)
                        .text_color(cx.theme().muted_foreground)
                        .size(13.0),
                )
            });

        let header = h_flex()
            .gap_3()
            .items_center()
            .when_some(card.icon, |row, icon| {
                row.child(
                    Icon::default()
                        .path(IconLoader::asset_path(icon))
                        .size(px(20.0)),
                )
            })
            .child(title_block)
            .when(!card.actions.is_empty(), |row| {
                row.child(h_flex().gap_2().ml_auto().children(card.actions))
            });

        GroupBox::new()
            .fill()
            .title(header)
            .child(v_flex().gap_3().children(card.body))
    }
);

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
#[derive(Default)]
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

gpui::impl_render_once!(KpiGrid);

gpui::impl_widget_render!(KpiGrid, |grid: KpiGrid,
                                    _window: &mut Window,
                                    cx: &mut App| {
    let cells = grid
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
});

/// High level dock-like layout with a sidebar and primary panel.
#[derive(Default)]
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

gpui::impl_render_once!(DockLayoutPanel);

gpui::impl_widget_render!(
    DockLayoutPanel,
    |panel: DockLayoutPanel, _window: &mut Window, cx: &mut App| {
        h_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_3()
                    .w(px(220.0))
                    .bg(cx.theme().muted)
                    .p_4()
                    .rounded(cx.theme().radius)
                    .children(panel.sidebar),
            )
            .child(
                v_flex()
                    .gap_4()
                    .flex_1()
                    .child(h_flex().gap_2().when(!panel.toolbar.is_empty(), |row| {
                        row.children(panel.toolbar.clone())
                    }))
                    .child(
                        v_flex()
                            .gap_4()
                            .bg(cx.theme().popover)
                            .p_6()
                            .rounded(cx.theme().radius_lg)
                            .children(panel.content),
                    ),
            )
    }
);

/// A themed switch that flips between the light and dark modes.
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

gpui::impl_render_once!(ThemeSwitch);

gpui::impl_widget_render!(
    ThemeSwitch,
    |switch: ThemeSwitch, window: &mut Window, cx: &mut App| {
        let registry = switch.registry.clone();
        let checked = !matches!(registry.active(), ThemeVariant::Light);

        Switch::new(switch.id.clone())
            .checked(checked)
            .when_some(switch.label.clone(), |s, label| s.label(Text::new(label)))
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
);

/// Button styled trigger that opens a command palette.
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

gpui::impl_render_once!(CommandPaletteTrigger);

gpui::impl_widget_render!(
    CommandPaletteTrigger,
    |trigger: CommandPaletteTrigger, window: &mut Window, cx: &mut App| {
        let mut button = Button::new(trigger.id.clone())
            .ghost()
            .icon(
                Icon::default()
                    .path(IconLoader::asset_path(trigger.icon))
                    .text_color(cx.theme().foreground),
            )
            .label(trigger.label.clone());

        if let Some(shortcut) = trigger.shortcut.clone() {
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
);

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
