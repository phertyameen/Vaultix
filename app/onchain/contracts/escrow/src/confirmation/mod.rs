// Multi-Party Confirmation Logic Module
//
// This module provides a complete confirmation system for escrow contracts,
// enabling multiple parties to confirm that escrow conditions have been met
// before funds can be released.
//
// # Architecture
//
// The confirmation system is composed of four main components:
//
// 1. **types.rs** - Type definitions and enums
//    - ConfirmationThreshold: Configurable threshold (All, Majority, Custom)
//    - ConfirmationState: Tracks each party's confirmation status
//    - EscrowConfirmationStatus: Overall escrow confirmation state
//    - ConfirmationEvent: Event data emitted on confirmation
//
// 2. **storage.rs** - Persistent storage operations
//    - Stores party confirmations with timestamps
//    - Tracks overall escrow confirmation status
//    - Maintains confirmation counts
//    - Manages threshold configuration
//
// 3. **threshold.rs** - Threshold calculation logic
//    - Determines if confirmation threshold has been met
//    - Calculates remaining confirmations needed
//    - Supports configurable threshold types
//
// 4. **confirmation.rs** - Core confirmation logic
//    - Main `confirm()` function for party confirmations
//    - Authorization validation
//    - Duplicate prevention
//    - Status updates on threshold achievement
//    - Error handling

pub mod types;
pub mod storage;
pub mod threshold;
pub mod confirmation;

pub use confirmation::{ConfirmationLogic, ConfirmationError};
pub use types::{ConfirmationThreshold, ConfirmationState, EscrowConfirmationStatus};
