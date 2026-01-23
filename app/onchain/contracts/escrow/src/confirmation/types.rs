use soroban_sdk::{Address, Env};

/// Threshold configuration for confirmation requirements
#[derive(Clone, Copy)]
pub enum ConfirmationThreshold {
    /// All parties must confirm
    All,
    /// Majority of parties must confirm (>= 50%)
    Majority,
    /// Custom number of parties required
    Custom(u32),
}

/// State of a party's confirmation
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationState {
    /// Party has not confirmed
    Pending,
    /// Party has confirmed
    Confirmed,
    /// Confirmation was rejected (cannot re-confirm)
    Rejected,
}

/// Confirmation record for a single party
#[derive(Clone)]
pub struct PartyConfirmation {
    pub address: Address,
    pub state: ConfirmationState,
    pub confirmed_at: u64,
    pub confirmation_count: u32,
}

/// Overall confirmation status for an escrow
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EscrowConfirmationStatus {
    /// Awaiting confirmations
    Pending,
    /// Threshold met, ready for release
    Confirmed,
    /// Confirmation failed or rejected
    Failed,
    /// Escrow completed or cancelled, no more confirmations allowed
    Locked,
}

/// Confirmation event data
#[derive(Clone)]
pub struct ConfirmationEvent {
    pub escrow_id: u64,
    pub party: Address,
    pub confirmed_at: u64,
    pub confirmations_count: u32,
    pub threshold_met: bool,
}
