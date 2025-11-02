//! Core design tokens used to build Workspace themes.

use serde::Serialize;

/// Shared color palette for a theme variant.
#[derive(Debug, Clone, Serialize)]
pub struct ColorPalette {
    /// Brand primary surface color.
    pub primary: &'static str,
    /// Foreground color rendered atop [`primary`].
    pub on_primary: &'static str,
    /// Accent surface color used for callouts.
    pub accent: &'static str,
    /// Foreground color rendered atop [`accent`].
    pub on_accent: &'static str,
    /// Application background color.
    pub background: &'static str,
    /// Default surface color for panels and cards.
    pub surface: &'static str,
    /// Border color separating surfaces.
    pub surface_border: &'static str,
    /// Muted fill color for secondary controls.
    pub muted: &'static str,
    /// Foreground color for muted surfaces.
    pub on_muted: &'static str,
    /// Color used to communicate success states.
    pub success: &'static str,
    /// Color used to communicate warning states.
    pub warning: &'static str,
    /// Color used to communicate error states.
    pub danger: &'static str,
}

/// Canonical spacing scale that maps semantic names to pixel values.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SpacingScale {
    /// Extra-extra-small spacing value.
    pub xxs: f32,
    /// Extra-small spacing value.
    pub xs: f32,
    /// Small spacing value.
    pub sm: f32,
    /// Medium spacing value.
    pub md: f32,
    /// Large spacing value.
    pub lg: f32,
    /// Extra-large spacing value.
    pub xl: f32,
    /// Extra-extra-large spacing value.
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
#[derive(Debug, Clone, Copy, Serialize)]
pub enum SpacingToken {
    /// Extra-extra-small spacing token.
    XXS,
    /// Extra-small spacing token.
    XS,
    /// Small spacing token.
    SM,
    /// Medium spacing token.
    MD,
    /// Large spacing token.
    LG,
    /// Extra-large spacing token.
    XL,
    /// Extra-extra-large spacing token.
    XXL,
}

/// Typographic scale storing base sizes for text styles.
#[derive(Debug, Clone, Serialize)]
pub struct TypographyScale {
    /// Default font family for text rendering.
    pub font_family: &'static str,
    /// Display heading size in pixels.
    pub display: f32,
    /// Headline size in pixels.
    pub headline: f32,
    /// Title text size in pixels.
    pub title: f32,
    /// Body text size in pixels.
    pub body: f32,
    /// Label text size in pixels.
    pub label: f32,
}

/// Declarative elevation tokens that can be transformed into GPUI shadows.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ElevationLevel {
    /// Vertical offset in pixels for the shadow.
    pub y_offset: f32,
    /// Blur radius in pixels for the shadow.
    pub blur_radius: f32,
    /// Spread radius in pixels for the shadow.
    pub spread_radius: f32,
    /// Opacity value for the shadow.
    pub opacity: f32,
}

/// Collection of elevation definitions for common surfaces.
#[derive(Debug, Clone, Serialize)]
pub struct ElevationScale {
    /// Flat surfaces without any elevation.
    pub flat: ElevationLevel,
    /// Raised surfaces such as cards.
    pub raised: ElevationLevel,
    /// Floating surfaces like dialogs and overlays.
    pub floating: ElevationLevel,
}

/// Bundles the complete token set for a theme variant.
#[derive(Debug, Clone, Serialize)]
pub struct DesignTokens {
    /// Color palette applied to the variant.
    pub colors: ColorPalette,
    /// Typographic scale associated with the variant.
    pub typography: TypographyScale,
    /// Spacing scale used for layout primitives.
    pub spacing: SpacingScale,
    /// Elevation scale used to generate shadows.
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
            on_accent: "#000000",
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
            font_family: "Inter",
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

    #[test]
    fn theme_contrast_ratios() {
        let themes = [light_tokens(), dark_tokens(), high_contrast_tokens()];
        let thresholds = [4.5, 4.5, 7.0];

        for (tokens, threshold) in themes.into_iter().zip(thresholds) {
            assert!(
                contrast_ratio(tokens.colors.primary, tokens.colors.on_primary) >= threshold,
                "primary contrast below threshold"
            );
            assert!(
                contrast_ratio(tokens.colors.accent, tokens.colors.on_accent) >= threshold,
                "accent contrast below threshold"
            );
            assert!(
                contrast_ratio(tokens.colors.muted, tokens.colors.on_muted) >= threshold,
                "muted contrast below threshold"
            );
            assert!(
                contrast_ratio(tokens.colors.background, tokens.colors.on_muted) >= threshold,
                "background contrast below threshold"
            );
        }
    }

    fn contrast_ratio(foreground: &str, background: &str) -> f32 {
        let fg = linear_rgb(foreground);
        let bg = linear_rgb(background);
        let l1 = relative_luminance(fg);
        let l2 = relative_luminance(bg);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    fn linear_rgb(hex: &str) -> (f32, f32, f32) {
        let bytes = hex_to_bytes(hex);
        (
            srgb_to_linear(bytes[0]),
            srgb_to_linear(bytes[1]),
            srgb_to_linear(bytes[2]),
        )
    }

    fn hex_to_bytes(hex: &str) -> [f32; 3] {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).expect("red");
        let g = u8::from_str_radix(&hex[2..4], 16).expect("green");
        let b = u8::from_str_radix(&hex[4..6], 16).expect("blue");
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
    }

    fn srgb_to_linear(value: f32) -> f32 {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    fn relative_luminance((r, g, b): (f32, f32, f32)) -> f32 {
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}
