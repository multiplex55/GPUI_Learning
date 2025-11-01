# Component usage guidelines

The gallery app demonstrates the canonical composition of `gpui-component` widgets. Each documentation panel embeds a snippet plus best practices and gotchas—those strings live alongside the component examples so the UI stays self-documenting.

## Buttons & inputs
- `GalleryApp::render_inputs` drives a `Button` through the `DemoKnobs` state. The doc callout encourages using the `ButtonVariants` trait so all styling flows through the design system.
- Workbench wizard steps follow the same pattern; validation errors live in `WizardState` and are surfaced through an accordion with explicit warning text.

## Navigation
- `render_navigation` showcases segmented `TabBar` controls. The on-click listener updates local state and calls `cx.notify()`; the matching doc warns that omitting the notification keeps the UI stale. Keyboard guidance lives in the adjacent `GroupBox` so screen readers get context.

## Feedback & overlays
- Alerts, notifications, and icon overlays wrap `GroupBox` to inherit consistent padding. Docs emphasise picking intent-specific variants and spacing stacked alerts.
- Layout helpers (`DockLayoutPanel`, `resizable_panel`) ship with reset commands. The workbench demo launcher publishes `ResetLayout` through the command bus when users request a factory reset.

## Quick launchers
Both the workbench and gallery now include quick-launch toolbars:
- The workbench `DemoLauncher` publishes `WorkbenchCommand::OpenDemo(..)` so the host shell opens the requested demo window. CLI arguments reuse the same command bus for parity.
- The gallery `render_launcher` triggers category switches, theme changes, and palette overlays, mirroring `--open` options. A doc snippet demonstrates the equivalent `cargo run` invocation.
