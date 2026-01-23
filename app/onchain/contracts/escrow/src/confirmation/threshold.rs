use soroban_sdk::Env;
use crate::confirmation::types::ConfirmationThreshold;

/// Threshold calculation logic
pub struct ThresholdLogic;

impl ThresholdLogic {
    /// Check if confirmation threshold has been met
    pub fn is_threshold_met(
        threshold: ConfirmationThreshold,
        confirmations: u32,
        total_parties: u32,
    ) -> bool {
        match threshold {
            ConfirmationThreshold::All => confirmations >= total_parties,
            ConfirmationThreshold::Majority => {
                let required = (total_parties as f64 / 2.0).ceil() as u32;
                confirmations >= required
            }
            ConfirmationThreshold::Custom(required) => confirmations >= required,
        }
    }

    /// Get required confirmations for a threshold
    pub fn get_required_confirmations(
        threshold: ConfirmationThreshold,
        total_parties: u32,
    ) -> u32 {
        match threshold {
            ConfirmationThreshold::All => total_parties,
            ConfirmationThreshold::Majority => {
                ((total_parties as f64 / 2.0).ceil() as u32).max(1)
            }
            ConfirmationThreshold::Custom(required) => required.min(total_parties),
        }
    }

    /// Calculate remaining confirmations needed
    pub fn get_remaining_confirmations(
        threshold: ConfirmationThreshold,
        confirmations: u32,
        total_parties: u32,
    ) -> u32 {
        let required = Self::get_required_confirmations(threshold, total_parties);
        if confirmations >= required {
            0
        } else {
            required - confirmations
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_threshold() {
        assert!(!ThresholdLogic::is_threshold_met(
            ConfirmationThreshold::All,
            2,
            3
        ));
        assert!(ThresholdLogic::is_threshold_met(
            ConfirmationThreshold::All,
            3,
            3
        ));
    }

    #[test]
    fn test_majority_threshold() {
        assert!(!ThresholdLogic::is_threshold_met(
            ConfirmationThreshold::Majority,
            1,
            3
        ));
        assert!(ThresholdLogic::is_threshold_met(
            ConfirmationThreshold::Majority,
            2,
            3
        ));
    }

    #[test]
    fn test_custom_threshold() {
        assert!(!ThresholdLogic::is_threshold_met(
            ConfirmationThreshold::Custom(2),
            1,
            3
        ));
        assert!(ThresholdLogic::is_threshold_met(
            ConfirmationThreshold::Custom(2),
            2,
            3
        ));
    }

    #[test]
    fn test_remaining_confirmations() {
        let remaining =
            ThresholdLogic::get_remaining_confirmations(ConfirmationThreshold::All, 1, 3);
        assert_eq!(remaining, 2);

        let remaining =
            ThresholdLogic::get_remaining_confirmations(ConfirmationThreshold::All, 3, 3);
        assert_eq!(remaining, 0);
    }
}
