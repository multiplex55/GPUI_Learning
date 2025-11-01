#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

/// Lists the placeholder datasets bundled with the workspace demos.
#[must_use]
pub fn available_datasets() -> Vec<&'static str> {
    vec!["sales_metrics", "customer_feedback", "system_health"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_expected_datasets() {
        assert!(available_datasets().contains(&"sales_metrics"));
    }
}
