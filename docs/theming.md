# Theming tokens

The design system exposes a [`ThemeRegistry`](../apps/gallery/src/main.rs) that carries the current palette, typography scale, and radius presets. The workbench and gallery both clone the registry so that every auxiliary window stays colour-synchronised. The new gallery CLI accepts `--open theme=<light|dark|high-contrast>` which maps to `ThemeSelector::variant()` and drives `GalleryApp::apply_theme`.

## Token sources
- `crates/designsystem/src/tokens.rs` defines semantic colours (`primary`, `accent`, `muted`, etc.) and high-contrast alternates. The `GalleryApp::render_palette_overlay` view visualises these tokens side-by-side for each variant.
- Iconography is grouped into runtime sets (`ICON_SETS`) so overlays only load the required assets. The quick-launcher button in the gallery toggles between the core and product sets via `GalleryLaunchTarget::IconSet`.

## Best practices
- Install the registry once per GPUI `Application` and clone handles for additional windows; this keeps theme transitions cheap and atomic.
- Drive theme selection through the command bus or launch targets so documentation and automation stay aligned.
- When adding new tokens, update the palette inspector strings to keep in-app docs authoritative.
- Run `cargo test -p designsystem contrast` after changing colours; the test suite enforces a minimum 4.5:1 contrast for light/dark
  and 7:1 for the high-contrast variant.

## Gotchas
- Forgetting to call `gpui_component::init(cx)` before cloning the registry results in unstyled components for newly spawned windows.
- Mixing manual colours with token-driven styles breaks high-contrast themes. Prefer semantic colours exported from the design system.
- Layout snapshots should be reset when theme variants change drastically; the workbench does this by publishing `ResetLayout` commands.
