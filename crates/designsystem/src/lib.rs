#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

mod icons;
mod theme;
mod tokens;

pub use icons::{IconAssetSource, IconLoader, IconName};
pub use theme::{ThemeDefinition, ThemeError, ThemeRegistry, ThemeVariant};
pub use tokens::{
    dark_tokens, high_contrast_tokens, light_tokens, ColorPalette, DesignTokens, ElevationLevel,
    ElevationScale, SpacingScale, SpacingToken, TypographyScale,
};

/// Installs the design system defaults into the supplied application context.
///
/// ```no_run
/// use designsystem::{IconLoader, ThemeRegistry};
/// use gpui::Application;
///
/// let app = Application::headless()
///     .with_assets(IconLoader::asset_source());
/// app.run(|cx| {
///     ThemeRegistry::new().install(cx);
/// });
/// ```
#[allow(clippy::module_name_repetitions)]
pub fn install_defaults(app: gpui::Application) -> gpui::Application {
    app.with_assets(IconLoader::asset_source())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_table_contains_entries() {
        assert!(!IconLoader::all().is_empty());
    }

    #[test]
    fn theme_registry_has_three_variants() {
        let registry = ThemeRegistry::new();
        assert_eq!(registry.definitions().count(), 3);
    }
}
