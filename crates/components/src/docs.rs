//! Documentation helpers for generating live code snippets inside the gallery.

use gpui::{App, IntoElement, Window};
use gpui_component::{
    group_box::GroupBox,
    styled::{v_flex, StyledExt as _},
    text::{Text, TextView, TextViewStyle},
};

/// Renders a syntax highlighted snippet inside a group box.
#[must_use]
pub fn render_snippet(
    id: impl Into<gpui::ElementId>,
    title: impl Into<gpui::SharedString>,
    code: impl AsRef<str>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let markdown = format!("```rust\n{}\n```", code.as_ref());
    let text_view = TextView::markdown(id, markdown, window, cx).style(TextViewStyle::default());

    GroupBox::new()
        .title(Text::new(title))
        .child(v_flex().gap_3().child(text_view))
}
