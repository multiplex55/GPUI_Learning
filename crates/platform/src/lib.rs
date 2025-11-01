#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, missing_docs, unreachable_pub)]

/// Initializes core platform services required by the applications.
#[must_use]
pub const fn initialize() -> bool {
    // Placeholder for GPUI runtime setup hooks.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_returns_success() {
        assert!(initialize());
    }
}
