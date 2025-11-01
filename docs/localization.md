# Localization strategy

Localization flows through the `platform::LocalizationRegistry`. The workbench seeds English and Spanish strings for navigation labels, docs tooltips, and notifications. The gallery reuses the same registry and exposes `--open locale=<lang>` so you can script locale switches alongside theme or category presets.

## Registry design
- `LocalizationRegistry::register_messages` stores key/value pairs per `LanguageIdentifier`. The workbench stores locale toggles in the command bus so the UI and docs stay consistent.
- `GalleryApp::apply_launch_target` reacts to `GalleryLaunchTarget::Locale` by updating `self.locale` and calling `cx.notify()`.

## Best practices
- Keep locale keys stable (`nav.dashboard`, `docs.keyboard`) so docs and UI reference the same translations.
- Provide fallbacks for new keys; the registry falls back to the default locale when a translation is missing.
- Bundle CLI-driven locale tests in CI using `cargo xtask gallery --target locale=es-ES` to ensure hot paths remain translated.

## Gotchas
- Forgetting to clone the localization registry before opening auxiliary windows leads to untranslated strings; both the workbench and gallery clone the registry when spawning previews.
- Locale changes should call `cx.notify()` and persist state if you want the setting to survive restarts.
