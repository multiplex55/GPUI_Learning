/// Lists the placeholder datasets bundled with the workspace demos.
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
