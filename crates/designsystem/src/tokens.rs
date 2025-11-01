//! Core design tokens used to build Workspace themes.

use serde::{Deserialize, Serialize};

/// Shared color palette for a theme variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary: &'static str,
    pub on_primary: &'static str,
    pub accent: &'static str,
    pub on_accent: &'static str,
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_border: &'static str,
    pub muted: &'static str,
    pub on_muted: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub danger: &'static str,
}

/// Canonical spacing scale that maps semantic names to pixel values.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpacingScale {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl SpacingScale {
    /// Returns the spacing token as raw pixels.
    #[must_use]
    pub const fn as_px(&self, size: SpacingToken) -> f32 {
        match size {
            SpacingToken::XXS => self.xxs,
            SpacingToken::XS => self.xs,
            SpacingToken::SM => self.sm,
            SpacingToken::MD => self.md,
            SpacingToken::LG => self.lg,
            SpacingToken::XL => self.xl,
            SpacingToken::XXL => self.xxl,
        }
    }
}

/// Named spacing tokens that are used throughout layout helpers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpacingToken {
    XXS,
    XS,
    SM,
    MD,
    LG,
    XL,
    XXL,
}

/// Typographic scale storing base sizes for text styles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyScale {
    pub font_family: &'static str,
    pub display: f32,
    pub headline: f32,
    pub title: f32,
    pub body: f32,
    pub label: f32,
}

/// Declarative elevation tokens that can be transformed into GPUI shadows.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElevationLevel {
    pub y_offset: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub opacity: f32,
}

/// Collection of elevation definitions for common surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationScale {
    pub flat: ElevationLevel,
    pub raised: ElevationLevel,
    pub floating: ElevationLevel,
}

/// Bundles the complete token set for a theme variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTokens {
    pub colors: ColorPalette,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub elevations: ElevationScale,
}

const BASE_SPACING: SpacingScale = SpacingScale {
    xxs: 2.0,
    xs: 4.0,
    sm: 8.0,
    md: 12.0,
    lg: 16.0,
    xl: 24.0,
    xxl: 32.0,
};

const BASE_ELEVATION: ElevationScale = ElevationScale {
    flat: ElevationLevel {
        y_offset: 0.0,
        blur_radius: 0.0,
        spread_radius: 0.0,
        opacity: 0.0,
    },
    raised: ElevationLevel {
        y_offset: 8.0,
        blur_radius: 16.0,
        spread_radius: 0.0,
        opacity: 0.12,
    },
    floating: ElevationLevel {
        y_offset: 16.0,
        blur_radius: 28.0,
        spread_radius: 0.0,
        opacity: 0.18,
    },
};

/// Default design tokens for the light workspace theme.
pub fn light_tokens() -> DesignTokens {
    DesignTokens {
        colors: ColorPalette {
            primary: "#2563eb",
            on_primary: "#f8fafc",
            accent: "#0ea5e9",
            on_accent: "#ecfeff",
            background: "#f5f7fa",
            surface: "#ffffff",
            surface_border: "#cbd5f5",
            muted: "#e2e8f0",
            on_muted: "#475569",
            success: "#10b981",
            warning: "#f59e0b",
            danger: "#ef4444",
        },
        typography: TypographyScale {
            font_family: "Inter",
            display: 36.0,
            headline: 28.0,
            title: 20.0,
            body: 16.0,
            label: 12.0,
        },
        spacing: BASE_SPACING,
        elevations: BASE_ELEVATION,
    }
}

/// Default design tokens for the dark workspace theme.
pub fn dark_tokens() -> DesignTokens {
    DesignTokens {
        colors: ColorPalette {
            primary: "#60a5fa",
            on_primary: "#0b1120",
            accent: "#38bdf8",
            on_accent: "#082f49",
            background: "#0f172a",
            surface: "#111c33",
            surface_border: "#1f2a44",
            muted: "#1e293b",
            on_muted: "#cbd5f5",
            success: "#34d399",
            warning: "#fbbf24",
            danger: "#f87171",
        },
        typography: TypographyScale {
            font_family: "Inter",
            display: 36.0,
            headline: 28.0,
            title: 20.0,
            body: 16.0,
            label: 12.0,
        },
        spacing: BASE_SPACING,
        elevations: BASE_ELEVATION,
    }
}

/// High contrast token set tuned for accessibility needs.
pub fn high_contrast_tokens() -> DesignTokens {
    DesignTokens {
        colors: ColorPalette {
            primary: "#1d4ed8",
            on_primary: "#ffffff",
            accent: "#0ea5e9",
            on_accent: "#062f43",
            background: "#020617",
            surface: "#050b1e",
            surface_border: "#f8fafc",
            muted: "#111827",
            on_muted: "#e2e8f0",
            success: "#22c55e",
            warning: "#facc15",
            danger: "#f87171",
        },
        typography: TypographyScale {
            font_family: "Atkinson Hyperlegible",
            display: 40.0,
            headline: 30.0,
            title: 22.0,
            body: 18.0,
            label: 14.0,
        },
        spacing: SpacingScale {
            xl: BASE_SPACING.xl,
            xxl: BASE_SPACING.xxl,
            ..BASE_SPACING
        },
        elevations: ElevationScale {
            floating: ElevationLevel {
                opacity: 0.22,
                ..BASE_ELEVATION.floating
            },
            ..BASE_ELEVATION
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_returns_expected_px() {
        let spacing = BASE_SPACING;
        assert_eq!(spacing.as_px(SpacingToken::MD), 12.0);
    }
}
