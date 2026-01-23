use soroban_sdk::{Address, Env, Symbol, Vec, Map};
use crate::confirmation::types::{PartyConfirmation, ConfirmationState, EscrowConfirmationStatus};

/// Storage keys for confirmation data
pub struct ConfirmationStorageKeys;

impl ConfirmationStorageKeys {
    /// Key for party confirmation records: (escrow_id) -> Map<Address, PartyConfirmation>
    pub fn party_confirmations(escrow_id: u64) -> Vec<u8> {
        format!("party_conf_{}", escrow_id).into_bytes()
    }

    /// Key for escrow confirmation status: (escrow_id) -> EscrowConfirmationStatus
    pub fn escrow_status(escrow_id: u64) -> Vec<u8> {
        format!("escrow_status_{}", escrow_id).into_bytes()
    }

    /// Key for threshold configuration: (escrow_id) -> ConfirmationThreshold
    pub fn threshold_config(escrow_id: u64) -> Vec<u8> {
        format!("threshold_{}", escrow_id).into_bytes()
    }

    /// Key for parties list: (escrow_id) -> Vec<Address>
    pub fn parties_list(escrow_id: u64) -> Vec<u8> {
        format!("parties_{}", escrow_id).into_bytes()
    }

    /// Key for confirmation count: (escrow_id) -> u32
    pub fn confirmation_count(escrow_id: u64) -> Vec<u8> {
        format!("conf_count_{}", escrow_id).into_bytes()
    }
}

/// Confirmation storage operations
pub struct ConfirmationStorage;

impl ConfirmationStorage {
    /// Get confirmation state for a specific party
    pub fn get_party_confirmation(
        env: &Env,
        escrow_id: u64,
        party: &Address,
    ) -> Option<PartyConfirmation> {
        let key = ConfirmationStorageKeys::party_confirmations(escrow_id);
        env.storage()
            .persistent()
            .get::<Vec<u8>, Map<Address, PartyConfirmation>>(&key)
            .and_then(|map| map.get(party.clone()).ok())
    }

    /// Set confirmation state for a party
    pub fn set_party_confirmation(
        env: &Env,
        escrow_id: u64,
        party: &Address,
        confirmation: PartyConfirmation,
    ) {
        let key = ConfirmationStorageKeys::party_confirmations(escrow_id);
        let mut map = env
            .storage()
            .persistent()
            .get::<Vec<u8>, Map<Address, PartyConfirmation>>(&key)
            .unwrap_or_else(|| Map::new(env));
        map.set(party.clone(), confirmation);
        env.storage()
            .persistent()
            .set::<Vec<u8>, Map<Address, PartyConfirmation>>(&key, &map);
    }

    /// Get current escrow confirmation status
    pub fn get_status(env: &Env, escrow_id: u64) -> EscrowConfirmationStatus {
        let key = ConfirmationStorageKeys::escrow_status(escrow_id);
        env.storage()
            .persistent()
            .get::<Vec<u8>, u32>(&key)
            .map(|status| match status {
                0 => EscrowConfirmationStatus::Pending,
                1 => EscrowConfirmationStatus::Confirmed,
                2 => EscrowConfirmationStatus::Failed,
                3 => EscrowConfirmationStatus::Locked,
                _ => EscrowConfirmationStatus::Pending,
            })
            .unwrap_or(EscrowConfirmationStatus::Pending)
    }

    /// Set escrow confirmation status
    pub fn set_status(env: &Env, escrow_id: u64, status: EscrowConfirmationStatus) {
        let key = ConfirmationStorageKeys::escrow_status(escrow_id);
        let status_code = match status {
            EscrowConfirmationStatus::Pending => 0,
            EscrowConfirmationStatus::Confirmed => 1,
            EscrowConfirmationStatus::Failed => 2,
            EscrowConfirmationStatus::Locked => 3,
        };
        env.storage().persistent().set::<Vec<u8>, u32>(&key, &status_code);
    }

    /// Get confirmation count for an escrow
    pub fn get_confirmation_count(env: &Env, escrow_id: u64) -> u32 {
        let key = ConfirmationStorageKeys::confirmation_count(escrow_id);
        env.storage()
            .persistent()
            .get::<Vec<u8>, u32>(&key)
            .unwrap_or(0)
    }

    /// Increment confirmation count
    pub fn increment_confirmation_count(env: &Env, escrow_id: u64) {
        let key = ConfirmationStorageKeys::confirmation_count(escrow_id);
        let count = Self::get_confirmation_count(env, escrow_id);
        env.storage()
            .persistent()
            .set::<Vec<u8>, u32>(&key, &(count + 1));
    }
}
