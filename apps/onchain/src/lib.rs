#![no_std]
use soroban_sdk::{
    token, Address, Env, Symbol, Vec, contract, contracterror, contractimpl, contracttype,
    symbol_short,
};

// Milestone status tracking
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MilestoneStatus {
    Pending,
    Released,
    Disputed,
}

// Individual milestone in an escrow
#[contracttype]
#[derive(Clone, Debug)]
pub struct Milestone {
    pub amount: i128,
    pub status: MilestoneStatus,
    pub description: Symbol,
}

// Overall escrow status
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EscrowStatus {
    Created,   // Escrow created but funds not yet deposited
    Active,    // Funds deposited and locked in contract
    Completed, // All milestones released
    Cancelled, // Escrow cancelled, funds refunded
}

// Main escrow structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Escrow {
    pub depositor: Address,
    pub recipient: Address,
    pub token_address: Address, // NEW: Token contract address
    pub total_amount: i128,
    pub total_released: i128,
    pub milestones: Vec<Milestone>,
    pub status: EscrowStatus,
    pub deadline: u64, // NEW: Deadline for escrow completion
}

// Contract error types
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    EscrowNotFound = 1,
    EscrowAlreadyExists = 2,
    MilestoneNotFound = 3,
    MilestoneAlreadyReleased = 4,
    UnauthorizedAccess = 5,
    InvalidMilestoneAmount = 6,
    TotalAmountMismatch = 7,
    InsufficientBalance = 8,
    EscrowNotActive = 9,
    VectorTooLarge = 10,
    ZeroAmount = 11,
    InvalidDeadline = 12,
    SelfDealing = 13,
    EscrowAlreadyFunded = 14,  // NEW: Prevent double funding
    TokenTransferFailed = 15,  // NEW: Token transfer error
}

#[contract]
pub struct VaultixEscrow;

#[contractimpl]
impl VaultixEscrow {
    /// Creates a new escrow with milestone-based payment releases.
    /// NOTE: This only creates the escrow structure. Funds must be deposited separately via deposit_funds().
    ///
    /// # Arguments
    /// * `escrow_id` - Unique identifier for the escrow
    /// * `depositor` - Address funding the escrow
    /// * `recipient` - Address receiving milestone payments
    /// * `token_address` - Address of the token contract (e.g., XLM, USDC)
    /// * `milestones` - Vector of milestones defining payment schedule
    /// * `deadline` - Unix timestamp deadline for escrow completion
    ///
    /// # Errors
    /// * `EscrowAlreadyExists` - If escrow_id is already in use
    /// * `VectorTooLarge` - If more than 20 milestones provided
    /// * `InvalidMilestoneAmount` - If any milestone amount is zero or negative
    /// * `SelfDealing` - If depositor and recipient are the same
    pub fn create_escrow(
        env: Env,
        escrow_id: u64,
        depositor: Address,
        recipient: Address,
        token_address: Address,
        milestones: Vec<Milestone>,
        deadline: u64,
    ) -> Result<(), Error> {
        // Authenticate the depositor
        depositor.require_auth();

        // Validate no self-dealing (depositor cannot be recipient)
        if depositor == recipient {
            return Err(Error::SelfDealing);
        }

        // Check if escrow already exists
        let storage_key = get_storage_key(escrow_id);
        if env.storage().persistent().has(&storage_key) {
            return Err(Error::EscrowAlreadyExists);
        }

        // Validate milestones and calculate total
        let total_amount = validate_milestones(&milestones)?;

        // Initialize all milestones to Pending status
        let mut initialized_milestones = Vec::new(&env);
        for milestone in milestones.iter() {
            let mut m = milestone.clone();
            m.status = MilestoneStatus::Pending;
            initialized_milestones.push_back(m);
        }

        // Create the escrow in Created state (not yet funded)
        let escrow = Escrow {
            depositor: depositor.clone(),
            recipient,
            token_address,
            total_amount,
            total_released: 0,
            milestones: initialized_milestones,
            status: EscrowStatus::Created, // Initially Created, becomes Active after deposit
            deadline,
        };

        // Save to persistent storage
        env.storage().persistent().set(&storage_key, &escrow);
        
        // Extend TTL for long-term storage
        env.storage().persistent().extend_ttl(
            &storage_key,
            100,
            2_000_000,
        );

        Ok(())
    }

    /// Deposits funds into an escrow, transitioning it from Created to Active.
    /// The depositor must have approved this contract to spend the required amount.
    ///
    /// # Arguments
    /// * `escrow_id` - Identifier of the escrow to fund
    ///
    /// # Errors
    /// * `EscrowNotFound` - If escrow doesn't exist
    /// * `UnauthorizedAccess` - If caller is not the depositor
    /// * `EscrowAlreadyFunded` - If escrow is already in Active state
    /// * `TokenTransferFailed` - If token transfer fails
    pub fn deposit_funds(env: Env, escrow_id: u64) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        // Load escrow from storage
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        // Verify authorization - only depositor can fund
        escrow.depositor.require_auth();

        // Check escrow hasn't already been funded
        if escrow.status != EscrowStatus::Created {
            return Err(Error::EscrowAlreadyFunded);
        }

        // Initialize token client for the specified token
        let token_client = token::Client::new(&env, &escrow.token_address);

        // Transfer tokens from depositor to contract
        // NOTE: Depositor must have approved this contract to spend their tokens
        token_client.transfer_from(
            &env.current_contract_address(), // spender (this contract)
            &escrow.depositor,                // from (depositor's address)
            &env.current_contract_address(), // to (contract's address - holds in escrow)
            &escrow.total_amount,            // amount to transfer
        );

        // Update escrow status to Active
        escrow.status = EscrowStatus::Active;

        // Save updated escrow
        env.storage().persistent().set(&storage_key, &escrow);
        
        // Extend TTL
        env.storage().persistent().extend_ttl(
            &storage_key,
            100,
            2_000_000,
        );

        Ok(())
    }

    /// Retrieves escrow details (read-only)
    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, Error> {
        let storage_key = get_storage_key(escrow_id);
        env.storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)
    }

    /// Read-only helper to fetch escrow status
    pub fn get_state(env: Env, escrow_id: u64) -> Result<EscrowStatus, Error> {
        let escrow = Self::get_escrow(env, escrow_id)?;
        Ok(escrow.status)
    }

    /// Releases a specific milestone payment to the recipient.
    /// This transfers the milestone amount from the contract to the recipient.
    ///
    /// # Arguments
    /// * `escrow_id` - Identifier of the escrow
    /// * `milestone_index` - Index of the milestone to release
    ///
    /// # Errors
    /// * `EscrowNotFound` - If escrow doesn't exist
    /// * `UnauthorizedAccess` - If caller is not the depositor
    /// * `EscrowNotActive` - If escrow is not in Active state
    /// * `MilestoneNotFound` - If index is out of bounds
    /// * `MilestoneAlreadyReleased` - If milestone was already released
    pub fn release_milestone(env: Env, escrow_id: u64, milestone_index: u32) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        // Load escrow from storage
        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        // Verify authorization - only depositor can release funds
        escrow.depositor.require_auth();

        // Check escrow is active (funds deposited)
        if escrow.status != EscrowStatus::Active {
            return Err(Error::EscrowNotActive);
        }

        // Verify milestone index is valid
        if milestone_index >= escrow.milestones.len() {
            return Err(Error::MilestoneNotFound);
        }

        // Get the milestone
        let mut milestone = escrow
            .milestones
            .get(milestone_index)
            .ok_or(Error::MilestoneNotFound)?;

        // Check if already released
        if milestone.status == MilestoneStatus::Released {
            return Err(Error::MilestoneAlreadyReleased);
        }

        // Initialize token client
        let token_client = token::Client::new(&env, &escrow.token_address);

        // Transfer milestone amount from contract to recipient
        token_client.transfer(
            &env.current_contract_address(), // from (contract address)
            &escrow.recipient,                // to (recipient address)
            &milestone.amount,                // amount to release
        );

        // Update milestone status
        milestone.status = MilestoneStatus::Released;
        escrow.milestones.set(milestone_index, milestone.clone());

        // Update total released with overflow protection
        escrow.total_released = escrow
            .total_released
            .checked_add(milestone.amount)
            .ok_or(Error::InvalidMilestoneAmount)?;

        // Save updated escrow
        env.storage().persistent().set(&storage_key, &escrow);
        
        // Extend TTL
        env.storage().persistent().extend_ttl(
            &storage_key,
            100,
            2_000_000,
        );

        Ok(())
    }

    /// Cancels an escrow before any milestones are released.
    /// Returns all funds to the depositor.
    ///
    /// # Arguments
    /// * `escrow_id` - Identifier of the escrow
    ///
    /// # Errors
    /// * `EscrowNotFound` - If escrow doesn't exist
    /// * `UnauthorizedAccess` - If caller is not the depositor
    /// * `MilestoneAlreadyReleased` - If any milestone has been released
    pub fn cancel_escrow(env: Env, escrow_id: u64) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        // Verify authorization
        escrow.depositor.require_auth();

        // Verify no milestones have been released
        if escrow.total_released > 0 {
            return Err(Error::MilestoneAlreadyReleased);
        }

        // If escrow was funded (Active status), refund the depositor
        if escrow.status == EscrowStatus::Active {
            let token_client = token::Client::new(&env, &escrow.token_address);
            
            // Transfer all funds back to depositor
            token_client.transfer(
                &env.current_contract_address(), // from (contract)
                &escrow.depositor,                // to (depositor)
                &escrow.total_amount,            // full amount
            );
        }

        // Update status
        escrow.status = EscrowStatus::Cancelled;
        env.storage().persistent().set(&storage_key, &escrow);
        
        // Extend TTL
        env.storage().persistent().extend_ttl(
            &storage_key,
            100,
            2_000_000,
        );

        Ok(())
    }

    /// Marks an escrow as completed after all milestones are released.
    ///
    /// # Arguments
    /// * `escrow_id` - Identifier of the escrow
    ///
    /// # Errors
    /// * `EscrowNotFound` - If escrow doesn't exist
    /// * `UnauthorizedAccess` - If caller is not the depositor
    /// * `EscrowNotActive` - If not all milestones are released
    pub fn complete_escrow(env: Env, escrow_id: u64) -> Result<(), Error> {
        let storage_key = get_storage_key(escrow_id);

        let mut escrow: Escrow = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(Error::EscrowNotFound)?;

        // Verify authorization
        escrow.depositor.require_auth();

        // Verify all milestones are released
        if !verify_all_released(&escrow.milestones) {
            return Err(Error::EscrowNotActive);
        }

        // Update status
        escrow.status = EscrowStatus::Completed;
        env.storage().persistent().set(&storage_key, &escrow);
        
        // Extend TTL
        env.storage().persistent().extend_ttl(
            &storage_key,
            100,
            2_000_000,
        );

        Ok(())
    }
}

// Helper function to generate storage key
fn get_storage_key(escrow_id: u64) -> (Symbol, u64) {
    (symbol_short!("escrow"), escrow_id)
}

// Validates milestone vector and returns total amount
fn validate_milestones(milestones: &Vec<Milestone>) -> Result<i128, Error> {
    // Check vector size to prevent gas issues
    if milestones.len() > 20 {
        return Err(Error::VectorTooLarge);
    }

    let mut total: i128 = 0;

    // Validate each milestone and calculate total
    for milestone in milestones.iter() {
        if milestone.amount <= 0 {
            return Err(Error::ZeroAmount);
        }

        total = total
            .checked_add(milestone.amount)
            .ok_or(Error::InvalidMilestoneAmount)?;
    }

    Ok(total)
}

// Checks if all milestones have been released
fn verify_all_released(milestones: &Vec<Milestone>) -> bool {
    for milestone in milestones.iter() {
        if milestone.status != MilestoneStatus::Released {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test;