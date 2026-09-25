//! Card payment processing module
//!
//! This module is gated behind the `CARD_ENABLED` feature flag.
//! When the flag is disabled, all card-related routes return 404.

pub mod fixtures;
pub mod screening;
pub mod webhooks;

use std::sync::Arc;

/// Check if card functionality is enabled via environment variable
pub fn is_card_enabled() -> bool {
    std::env::var("CARD_ENABLED")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Card-specific application state extension
#[derive(Clone)]
pub struct CardState {
    /// Screening service for country validation
    pub screening: Arc<dyn screening::Screening>,
}

impl CardState {
    /// Create new card state with default screening (AllowAll)
    pub fn new() -> Self {
        Self {
            screening: Arc::new(screening::AllowAllScreening::new(
                std::env::var("CARD_COUNTRY_DENYLIST")
                    .ok()
                    .map(|v| {
                        v.split(',')
                            .map(|s| s.trim().to_uppercase())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default(),
            )),
        }
    }
}

impl Default for CardState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_card_enabled_defaults_false() {
        std::env::remove_var("CARD_ENABLED");
        assert!(!is_card_enabled());
    }

    #[test]
    fn test_is_card_enabled_true_values() {
        for val in ["1", "true", "yes", "on", "TRUE", "Yes"] {
            std::env::set_var("CARD_ENABLED", val);
            assert!(is_card_enabled(), "failed for value: {}", val);
        }
        std::env::remove_var("CARD_ENABLED");
    }

    #[test]
    fn test_is_card_enabled_false_values() {
        for val in ["0", "false", "no", "off", "FALSE", "No"] {
            std::env::set_var("CARD_ENABLED", val);
            assert!(!is_card_enabled(), "failed for value: {}", val);
        }
        std::env::remove_var("CARD_ENABLED");
    }
}
