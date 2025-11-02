# Accessibility checklist

The component gallery now ships with a dedicated accessibility flow that covers
color contrast, focus management, and keyboard-only navigation. Use the
following checklist during release reviews:

## Automated checks

1. **Token contrast validation** – run the design system tests to verify that
   light, dark, and high-contrast palettes meet WCAG expectations:
   
   ```bash
   cargo test -p designsystem contrast
   ```
2. **Icon pipeline linting** – normalize new SVG packs with
   `cargo xtask icons --pack <core|product> <path>` so the build script can
   regenerate the runtime enum without manual adjustments.

## Manual verifications

1. **Focus order** – launch the gallery (`cargo run --package gallery`) and use
   `Tab` / `Shift+Tab` to move through the header, quick launcher, category
   tabs, and demo controls in order.
2. **Keyboard-only navigation** – open the “Keyboard navigation” card at the top
   of the gallery. Confirm that the documented shortcuts (`Ctrl+K` for the
   command palette, arrow keys for tabs, etc.) match actual behaviour.
3. **Command palette coverage** – press `Ctrl+K` or the command palette button,
   then trigger each action via keyboard to ensure the palette drives category
   selection, theme switching, and layout resets.
4. **High-contrast theme** – toggle the high-contrast option either through the
   theme switcher or command palette action. Visually inspect that copy on
   primary, accent, muted, and background surfaces meets the expected contrast.
5. **Icon packs** – in the “Runtime Icon Sets” panel select both the Core and
   Product packs to confirm the new assets display correctly and include
   accessible labels.

Document results for each release in your QA notes so regressions can be traced
quickly.
