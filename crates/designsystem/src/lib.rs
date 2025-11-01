/// Returns the default primary color used across the workspace design system.
pub fn primary_color() -> &'static str {
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
