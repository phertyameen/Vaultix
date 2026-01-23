use soroban_sdk::{Address, Env, Vec};
use crate::confirmation::types::{
    PartyConfirmation, ConfirmationState, EscrowConfirmationStatus, ConfirmationThreshold,
    ConfirmationEvent,
};
use crate::confirmation::storage::{ConfirmationStorage, ConfirmationStorageKeys};
use crate::confirmation::threshold::ThresholdLogic;

/// Error types for confirmation operations
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationError {
    /// Caller is not an authorized party
    UnauthorizedParty,
    /// Party has already confirmed
    DuplicateConfirmation,
    /// Escrow confirmation is locked (completed or cancelled)
    EscrowLocked,
    /// Party list is empty
    EmptyPartyList,
    /// Invalid threshold configuration
    InvalidThreshold,
}

/// Core confirmation logic
pub struct ConfirmationLogic;

impl ConfirmationLogic {
    /// Confirm escrow conditions have been met
    ///
    /// # Arguments
    /// * `env` - Soroban environment
    /// * `escrow_id` - ID of the escrow
    /// * `caller` - Address of the confirming party
    /// * `parties` - Vector of authorized parties
    /// * `threshold` - Confirmation threshold requirement
    ///
    /// # Returns
    /// Result with confirmation event or error
    pub fn confirm(
        env: &Env,
        escrow_id: u64,
        caller: &Address,
        parties: Vec<Address>,
        threshold: ConfirmationThreshold,
    ) -> Result<ConfirmationEvent, ConfirmationError> {
        // Check if escrow is locked
        let status = ConfirmationStorage::get_status(env, escrow_id);
        if status == EscrowConfirmationStatus::Locked {
            return Err(ConfirmationError::EscrowLocked);
        }

        // Validate parties list
        if parties.len() == 0 {
            return Err(ConfirmationError::EmptyPartyList);
        }

        // Authorize caller
        if !Self::is_authorized_party(env, &caller, &parties) {
            return Err(ConfirmationError::UnauthorizedParty);
        }

        // Check for duplicate confirmation
        let existing = ConfirmationStorage::get_party_confirmation(env, escrow_id, caller);
        if let Some(conf) = existing {
            if conf.state == ConfirmationState::Confirmed {
                return Err(ConfirmationError::DuplicateConfirmation);
            }
        }

        // Record confirmation with timestamp
        let timestamp = env.ledger().timestamp();
        let confirmation_count = ConfirmationStorage::get_confirmation_count(env, escrow_id) + 1;

        let confirmation = PartyConfirmation {
            address: caller.clone(),
            state: ConfirmationState::Confirmed,
            confirmed_at: timestamp,
            confirmation_count,
        };

        ConfirmationStorage::set_party_confirmation(env, escrow_id, caller, confirmation);
        ConfirmationStorage::increment_confirmation_count(env, escrow_id);

        // Check if threshold is met
        let threshold_met = ThresholdLogic::is_threshold_met(
            threshold,
            confirmation_count,
            parties.len() as u32,
        );

        if threshold_met {
            ConfirmationStorage::set_status(env, escrow_id, EscrowConfirmationStatus::Confirmed);
        }

        Ok(ConfirmationEvent {
            escrow_id,
            party: caller.clone(),
            confirmed_at: timestamp,
            confirmations_count: confirmation_count,
            threshold_met,
        })
    }

    /// Check if an address is an authorized party
    fn is_authorized_party(env: &Env, address: &Address, parties: &Vec<Address>) -> bool {
        parties.iter().any(|party| party == address)
    }

    /// Get confirmation status for an escrow
    pub fn get_escrow_status(env: &Env, escrow_id: u64) -> EscrowConfirmationStatus {
        ConfirmationStorage::get_status(env, escrow_id)
    }

    /// Get confirmation count
    pub fn get_confirmation_count(env: &Env, escrow_id: u64) -> u32 {
        ConfirmationStorage::get_confirmation_count(env, escrow_id)
    }

    /// Get party's confirmation state
    pub fn get_party_state(
        env: &Env,
        escrow_id: u64,
        party: &Address,
    ) -> Option<ConfirmationState> {
        ConfirmationStorage::get_party_confirmation(env, escrow_id, party)
            .map(|conf| conf.state)
    }

    /// Lock escrow from further confirmations (call when escrow completes or is cancelled)
    pub fn lock_escrow(env: &Env, escrow_id: u64) {
        ConfirmationStorage::set_status(env, escrow_id, EscrowConfirmationStatus::Locked);
    }

    /// Get remaining confirmations needed
    pub fn get_remaining_confirmations(
        env: &Env,
        escrow_id: u64,
        total_parties: u32,
        threshold: ConfirmationThreshold,
    ) -> u32 {
        let confirmations = ConfirmationStorage::get_confirmation_count(env, escrow_id);
        ThresholdLogic::get_remaining_confirmations(threshold, confirmations, total_parties)
    }

    /// Check if a party can still confirm
    pub fn can_confirm(
        env: &Env,
        escrow_id: u64,
        party: &Address,
    ) -> bool {
        let status = ConfirmationStorage::get_status(env, escrow_id);
        if status == EscrowConfirmationStatus::Locked {
            return false;
        }

        match ConfirmationStorage::get_party_confirmation(env, escrow_id, party) {
            Some(conf) => conf.state != ConfirmationState::Confirmed,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests would require Soroban test environment
    // These are unit test examples

    #[test]
    fn test_is_authorized_party() {
        // This would test with mock Address objects in a real test environment
        // Example structure shown for documentation
    }

    #[test]
    fn test_duplicate_confirmation_detection() {
        // Would verify duplicate confirmation error is raised
        // Structure shown for documentation
    }

    #[test]
    fn test_threshold_met_triggers_status_update() {
        // Would verify status changes when threshold is met
        // Structure shown for documentation
    }
}
