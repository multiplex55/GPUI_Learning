# Platform Crate

Cross-application services such as configuration persistence, feature flags,
localization helpers, and a simple command bus.

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
