# Design System Crate

This crate captures the visual primitives used across the workspace demos. It
exposes theme tokens, an integration layer with [`gpui-component`], and a
compile-time icon pipeline that turns SVG assets into a type-safe API.

## Quick start

```no_run
use designsystem::{IconLoader, ThemeRegistry};
use gpui::Application;

let app = Application::headless()
    .with_assets(IconLoader::asset_source());
app.run(|cx| {
    let registry = ThemeRegistry::new();
    registry.install(cx);
});
```

## Modules

- **`tokens`** – typed color, typography, spacing, and elevation scales with
  helpers for generating theme definitions.
- **`theme`** – a [`ThemeRegistry`] that wires the tokens into
  `gpui-component`'s [`Theme`] globals and offers variant management utilities.
- **`icons`** – a `build.rs` driven pipeline that reads SVG files and emits an
  [`IconName`] enum plus an [`IconAssetSource`] that can be attached to a GPUI
  application.

[`IconAssetSource`]: crate::IconAssetSource
[`IconName`]: crate::IconName
[`Theme`]: gpui_component::theme::Theme
[`ThemeRegistry`]: crate::ThemeRegistry
[`gpui-component`]: https://crates.io/crates/gpui-component
