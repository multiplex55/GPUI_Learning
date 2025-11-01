//! Theme registry that bridges Workspace tokens with `gpui-component`.

use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gpui::{px, App, Global};
use gpui_component::theme::{Theme, ThemeConfig, ThemeConfigColors, ThemeMode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::tokens::{
    dark_tokens, high_contrast_tokens, light_tokens, ColorPalette, DesignTokens, TypographyScale,
};

/// Available theme variants shipped with the design system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeVariant {
    Light,
    Dark,
    HighContrast,
}

impl ThemeVariant {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ThemeVariant::Light => "light",
            ThemeVariant::Dark => "dark",
            ThemeVariant::HighContrast => "high-contrast",
        }
    }

    fn mode(self) -> ThemeMode {
        match self {
            ThemeVariant::Light => ThemeMode::Light,
            ThemeVariant::Dark | ThemeVariant::HighContrast => ThemeMode::Dark,
        }
    }
}

/// Error returned when a theme lookup fails.
#[derive(Debug, Error)]
pub enum ThemeError {
    /// Raised when the variant name is unknown.
    #[error("unknown theme variant '{0}'")]
    UnknownVariant(String),
}

/// Complete definition for a workspace theme variant.
#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub variant: ThemeVariant,
    pub tokens: DesignTokens,
    pub config: ThemeConfig,
}

impl ThemeDefinition {
    fn new(variant: ThemeVariant, tokens: DesignTokens) -> Self {
        let mut colors = ThemeConfigColors::default();
        let palette = &tokens.colors;
        colors.primary = Some(palette.primary.into());
        colors.primary_foreground = Some(palette.on_primary.into());
        colors.accent = Some(palette.accent.into());
        colors.accent_foreground = Some(palette.on_accent.into());
        colors.background = Some(palette.background.into());
        colors.popover = Some(palette.surface.into());
        colors.popover_foreground = Some(palette.on_muted.into());
        colors.border = Some(palette.surface_border.into());
        colors.list = Some(palette.surface.into());
        colors.list_hover = Some(palette.muted.into());
        colors.muted = Some(palette.muted.into());
        colors.muted_foreground = Some(palette.on_muted.into());
        colors.success = Some(palette.success.into());
        colors.info = Some(palette.accent.into());
        colors.danger = Some(palette.danger.into());
        colors.warning = Some(palette.warning.into());
        colors.list_active = Some(palette.accent.into());
        colors.list_active_border = Some(palette.on_accent.into());

        let typography = &tokens.typography;
        let mut config = ThemeConfig::default();
        config.is_default = matches!(variant, ThemeVariant::Light);
        config.mode = variant.mode();
        config.name = variant.as_str().into();
        config.font_size = Some(typography.body);
        config.font_family = Some(typography.font_family.into());
        config.radius = Some(8);
        config.radius_lg = Some(16);
        config.shadow = Some(true);
        config.colors = colors;

        Self {
            variant,
            tokens,
            config,
        }
    }
}

struct ThemeRegistryInner {
    definitions: HashMap<ThemeVariant, ThemeDefinition>,
    active: Mutex<ThemeVariant>,
    order: Vec<ThemeVariant>,
}

impl ThemeRegistryInner {
    fn definition(&self, variant: ThemeVariant) -> &ThemeDefinition {
        self.definitions
            .get(&variant)
            .expect("theme definition registered")
    }
}

/// State manager that tracks the currently active theme and provides helpers
/// for mutating GPUI's global theme instance.
#[derive(Clone)]
pub struct ThemeRegistry {
    inner: Arc<ThemeRegistryInner>,
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        static BUILT_INS: Lazy<HashMap<ThemeVariant, ThemeDefinition>> = Lazy::new(|| {
            let mut map = HashMap::new();
            map.insert(
                ThemeVariant::Light,
                ThemeDefinition::new(ThemeVariant::Light, light_tokens()),
            );
            map.insert(
                ThemeVariant::Dark,
                ThemeDefinition::new(ThemeVariant::Dark, dark_tokens()),
            );
            map.insert(
                ThemeVariant::HighContrast,
                ThemeDefinition::new(ThemeVariant::HighContrast, high_contrast_tokens()),
            );
            map
        });

        Self {
            inner: Arc::new(ThemeRegistryInner {
                definitions: BUILT_INS.clone(),
                active: Mutex::new(ThemeVariant::Light),
                order: vec![
                    ThemeVariant::Light,
                    ThemeVariant::Dark,
                    ThemeVariant::HighContrast,
                ],
            }),
        }
    }
}

impl ThemeRegistry {
    /// Creates a new registry with the built-in tokens.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently active theme variant.
    #[must_use]
    pub fn active(&self) -> ThemeVariant {
        *self.inner.active.lock().expect("theme mutex poisoned")
    }

    /// Iterates over the theme definitions in a stable order.
    pub fn definitions(&self) -> impl Iterator<Item = &ThemeDefinition> {
        self.inner
            .order
            .iter()
            .map(|variant| self.definition(*variant))
    }

    /// Returns the definition for the requested variant.
    #[must_use]
    pub fn definition(&self, variant: ThemeVariant) -> &ThemeDefinition {
        self.inner.definition(variant)
    }

    /// Looks up a variant by its slug.
    pub fn variant_from_str(&self, name: &str) -> Result<ThemeVariant, ThemeError> {
        self.inner
            .order
            .iter()
            .copied()
            .find(|variant| variant.as_str() == name)
            .ok_or_else(|| ThemeError::UnknownVariant(name.to_owned()))
    }

    /// Applies the variant and updates the GPUI theme globals.
    pub fn apply(&self, variant: ThemeVariant, cx: &mut App) {
        if !cx.has_global::<Theme>() {
            gpui_component::theme::init(cx);
        }

        let light = Rc::new(self.definition(ThemeVariant::Light).config.clone());
        let dark = Rc::new(self.definition(ThemeVariant::Dark).config.clone());
        let high_contrast = Rc::new(self.definition(ThemeVariant::HighContrast).config.clone());

        let theme = Theme::global_mut(cx);
        theme.light_theme = light.clone();
        theme.dark_theme = dark.clone();

        let selected = match variant {
            ThemeVariant::Light => light,
            ThemeVariant::Dark => dark,
            ThemeVariant::HighContrast => high_contrast,
        };

        theme.apply_config(&selected);
        theme.mode = variant.mode();
        theme.shadow = true;
        theme.font_size = px(self.definition(variant).tokens.typography.body);

        *self.inner.active.lock().expect("theme mutex poisoned") = variant;
    }

    /// Convenience helper to cycle between the built-in variants.
    pub fn cycle(&self, cx: &mut App) {
        let next = match self.active() {
            ThemeVariant::Light => ThemeVariant::Dark,
            ThemeVariant::Dark => ThemeVariant::HighContrast,
            ThemeVariant::HighContrast => ThemeVariant::Light,
        };
        self.apply(next, cx);
    }

    /// Applies the default (light) theme.
    pub fn install(&self, cx: &mut App) {
        self.apply(ThemeVariant::Light, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_lookup_by_name() {
        let registry = ThemeRegistry::new();
        assert_eq!(
            registry.variant_from_str("dark").unwrap(),
            ThemeVariant::Dark
        );
        assert!(matches!(
            registry.variant_from_str("missing"),
            Err(ThemeError::UnknownVariant(name)) if name == "missing"
        ));
    }

    #[test]
    fn typography_sizes_propagate() {
        let definition = ThemeRegistry::new().definition(ThemeVariant::HighContrast);
        assert!(definition.tokens.typography.body > 16.0);
    }
}
