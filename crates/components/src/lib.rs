#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

/// Returns the catalog of core components exposed by the shared library.
#[must_use]
pub fn component_catalog() -> Vec<&'static str> {
    vec!["button", "card", "nav"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_not_empty() {
        assert!(!component_catalog().is_empty());
    }
}
