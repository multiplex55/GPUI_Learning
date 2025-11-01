#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

/// Returns the default primary color used across the workspace design system.
#[must_use]
pub const fn primary_color() -> &'static str {
    "#0f172a"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_color_matches_brand_palette() {
        assert_eq!(primary_color(), "#0f172a");
    }
}
