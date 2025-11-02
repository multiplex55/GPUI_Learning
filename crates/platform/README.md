# Platform Crate

Cross-application services such as configuration persistence, feature flags,
localization helpers, and a simple command bus.

## Asset bundling

Static assets such as fonts and marketing imagery live under
`crates/platform/assets`. The crate's build script automatically walks the
`fonts/` and `images/` directories, generating an embedded manifest that exposes
each file through [`EMBEDDED_ASSETS`]. Registering those bytes with GPUI is as
simple as:

```rust
use gpui::Application;
use platform::EMBEDDED_ASSETS;

let mut app = Application::headless();

EMBEDDED_ASSETS.register_with(
    |name, bytes| {
        // Register fonts using the gpui API that your application prefers.
        // For example: app.fonts_mut().load_font_data(bytes.to_vec());
        let _ = (name, bytes);
    },
    |path, bytes| {
        // Expose images through a custom AssetSource or inline loader.
        let _ = (path, bytes);
    },
);
```

The helper keeps paths portable so packaging for macOS, Windows, and Linux only
requires copying the generated artifacts. During CI the build script flags any
new assets via Cargo's `rerun-if-changed` metadata, ensuring release archives
are always in sync.

To keep the assets directory lightweight the build script also invokes the
[`example_plot`] crate, which renders a small accessibility progress chart using
the [`plotters`] library. The generated PNG is written to
`crates/platform/assets/images/accessibility-checklist.png` during the build and
automatically included in the manifest.

## Example

```no_run
use platform::{bootstrap, CommandBus, ConfigStore, FeatureFlags, LocalizationRegistry};
use unic_langid::langid;
use gpui::Application;

let app = Application::headless();
let store = ConfigStore::default();
app.run(|cx| {
    let config = bootstrap(cx, &store).expect("config");
    let flags = FeatureFlags::from_env();

    let registry = LocalizationRegistry::new(langid!("en-US"));
    registry.register_messages(langid!("en-US"), [("welcome", "Welcome back!")]);

    let bus = CommandBus::<String>::new();
    let receiver = bus.subscribe();
    bus.publish("refresh".to_string());
    assert_eq!(receiver.recv().ok(), Some("refresh".to_string()));
});
```

[`EMBEDDED_ASSETS`]: crate::EMBEDDED_ASSETS
[`example_plot`]: ../example_plot
[`plotters`]: https://github.com/plotters-rs/plotters
