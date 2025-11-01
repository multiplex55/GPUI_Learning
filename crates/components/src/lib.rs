/// Returns the catalog of core components exposed by the shared library.
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
