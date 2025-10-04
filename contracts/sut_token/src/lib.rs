#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, Address, Env, String,
    Symbol, Vec,
};

contractmeta!(
    key = "Description",
    val = "SUT Token - Stellar Utility Token for payments, staking, and access control"
);

// Error types
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InsufficientBalance = 4,
    InsufficientAllowance = 5,
    ExceedsSupplyCap = 6,
    InvalidAmount = 7,
    InvalidAddress = 8,
    SnapshotNotFound = 9,
}

// Data structures
#[derive(Clone)]
#[contracttype]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct TokenInfo {
    pub total_supply: i128,
    pub supply_cap: i128,
    pub admin: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct Snapshot {
    pub block_number: u32,
    pub total_supply: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct Checkpoint {
    pub snapshot_id: u32,
    pub balance: i128,
}

// Storage keys
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Metadata,
    TokenInfo,
    Balance(Address),
    Allowance(Address, Address), // owner, spender
    Minter(Address),
    Snapshot(u32), // snapshot_id
    SnapshotCount,
    UserCheckpoints(Address),     // Vec<Checkpoint> for user
    UserCheckpointCount(Address), // number of checkpoints for user
}

// Events
#[derive(Clone)]
#[contracttype]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct ApprovalEvent {
    pub owner: Address,
    pub spender: Address,
    pub amount: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct MintEvent {
    pub to: Address,
    pub amount: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct BurnEvent {
    pub from: Address,
    pub amount: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct SnapshotEvent {
    pub id: u32,
    pub block_number: u32,
}

#[contract]
pub struct SutToken;

#[contractimpl]
impl SutToken {
    /// Initialize the token contract
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        supply_cap: i128,
    ) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Metadata) {
            return Err(Error::AlreadyInitialized);
        }

        // Validate inputs
        if supply_cap <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Set metadata
        let metadata = TokenMetadata {
            name,
            symbol,
            decimals,
        };
        env.storage().instance().set(&DataKey::Metadata, &metadata);

        // Set token info
        let token_info = TokenInfo {
            total_supply: 0,
            supply_cap,
            admin: admin.clone(),
        };
        env.storage()
            .instance()
            .set(&DataKey::TokenInfo, &token_info);

        // Set admin as initial minter
        env.storage()
            .instance()
            .set(&DataKey::Minter(admin.clone()), &true);

        // Initialize snapshot counter
        env.storage().instance().set(&DataKey::SnapshotCount, &0u32);

        Ok(())
    }

    /// Get token name
    pub fn name(env: Env) -> Result<String, Error> {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&DataKey::Metadata)
            .ok_or(Error::NotInitialized)?;
        Ok(metadata.name)
    }

    /// Get token symbol
    pub fn symbol(env: Env) -> Result<String, Error> {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&DataKey::Metadata)
            .ok_or(Error::NotInitialized)?;
        Ok(metadata.symbol)
    }

    /// Get token decimals
    pub fn decimals(env: Env) -> Result<u32, Error> {
        let metadata: TokenMetadata = env
            .storage()
            .instance()
            .get(&DataKey::Metadata)
            .ok_or(Error::NotInitialized)?;
        Ok(metadata.decimals)
    }

    /// Get total supply
    pub fn total_supply(env: Env) -> Result<i128, Error> {
        let token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;
        Ok(token_info.total_supply)
    }

    /// Get supply cap
    pub fn supply_cap(env: Env) -> Result<i128, Error> {
        let token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;
        Ok(token_info.supply_cap)
    }

    /// Get balance of an address
    pub fn balance_of(env: Env, account: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(account))
            .unwrap_or(0)
    }

    /// Get allowance between owner and spender
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Allowance(owner, spender))
            .unwrap_or(0)
    }

    /// Transfer tokens
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        // Validate inputs
        if amount < 0 {
            return Err(Error::InvalidAmount);
        }
        if amount == 0 {
            return Ok(());
        }

        // Require authorization from the 'from' address
        from.require_auth();

        Self::transfer_internal(&env, &from, &to, amount)?;

        // Emit transfer event
        let event = TransferEvent {
            from: from.clone(),
            to: to.clone(),
            amount,
        };
        env.events()
            .publish((Symbol::new(&env, "transfer"),), event);

        Ok(())
    }

    /// Transfer tokens from one address to another (requires allowance)
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        // Validate inputs
        if amount < 0 {
            return Err(Error::InvalidAmount);
        }
        if amount == 0 {
            return Ok(());
        }

        // Require authorization from the spender
        spender.require_auth();

        // Check allowance
        let allowance = Self::allowance(env.clone(), from.clone(), spender.clone());
        if allowance < amount {
            return Err(Error::InsufficientAllowance);
        }

        // Update allowance
        let new_allowance = allowance - amount;
        if new_allowance == 0 {
            env.storage()
                .persistent()
                .remove(&DataKey::Allowance(from.clone(), spender.clone()));
        } else {
            env.storage().persistent().set(
                &DataKey::Allowance(from.clone(), spender.clone()),
                &new_allowance,
            );
        }

        Self::transfer_internal(&env, &from, &to, amount)?;

        // Emit transfer event
        let event = TransferEvent {
            from: from.clone(),
            to: to.clone(),
            amount,
        };
        env.events()
            .publish((Symbol::new(&env, "transfer"),), event);

        Ok(())
    }

    /// Approve spender to spend tokens
    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) -> Result<(), Error> {
        if amount < 0 {
            return Err(Error::InvalidAmount);
        }

        // Require authorization from the owner
        owner.require_auth();

        // Set allowance
        if amount == 0 {
            env.storage()
                .persistent()
                .remove(&DataKey::Allowance(owner.clone(), spender.clone()));
        } else {
            env.storage()
                .persistent()
                .set(&DataKey::Allowance(owner.clone(), spender.clone()), &amount);
        }

        // Emit approval event
        let event = ApprovalEvent {
            owner: owner.clone(),
            spender: spender.clone(),
            amount,
        };
        env.events()
            .publish((Symbol::new(&env, "approval"),), event);

        Ok(())
    }

    /// Mint new tokens (only by minter)
    pub fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Require authorization from the minter
        minter.require_auth();

        // Check if caller is a minter
        if !env
            .storage()
            .instance()
            .get(&DataKey::Minter(minter.clone()))
            .unwrap_or(false)
        {
            return Err(Error::Unauthorized);
        }

        // Get current token info
        let mut token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;

        // Check supply cap
        if token_info.total_supply + amount > token_info.supply_cap {
            return Err(Error::ExceedsSupplyCap);
        }

        // Update total supply
        token_info.total_supply += amount;
        env.storage()
            .instance()
            .set(&DataKey::TokenInfo, &token_info);

        // Update recipient balance
        let current_balance = Self::balance_of(env.clone(), to.clone());
        let new_balance = current_balance + amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);

        // Update checkpoint for recipient
        Self::update_checkpoint(&env, &to, new_balance);

        // Emit mint event
        let event = MintEvent {
            to: to.clone(),
            amount,
        };
        env.events().publish((Symbol::new(&env, "mint"),), event);

        Ok(())
    }

    /// Burn tokens (only by minter)
    pub fn burn(env: Env, minter: Address, from: Address, amount: i128) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        // Require authorization from the minter
        minter.require_auth();

        // Check if caller is a minter
        if !env
            .storage()
            .instance()
            .get(&DataKey::Minter(minter.clone()))
            .unwrap_or(false)
        {
            return Err(Error::Unauthorized);
        }

        // Check balance
        let current_balance = Self::balance_of(env.clone(), from.clone());
        if current_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Get current token info
        let mut token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;

        // Update total supply
        token_info.total_supply -= amount;
        env.storage()
            .instance()
            .set(&DataKey::TokenInfo, &token_info);

        // Update sender balance
        let new_balance = current_balance - amount;
        if new_balance == 0 {
            env.storage()
                .persistent()
                .remove(&DataKey::Balance(from.clone()));
        } else {
            env.storage()
                .persistent()
                .set(&DataKey::Balance(from.clone()), &new_balance);
        }

        // Update checkpoint for sender
        Self::update_checkpoint(&env, &from, new_balance);

        // Emit burn event
        let event = BurnEvent {
            from: from.clone(),
            amount,
        };
        env.events().publish((Symbol::new(&env, "burn"),), event);

        Ok(())
    }

    /// Add a new minter (only by admin)
    pub fn add_minter(env: Env, minter: Address) -> Result<(), Error> {
        let token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;

        // Require authorization from admin
        token_info.admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Minter(minter), &true);
        Ok(())
    }

    /// Remove a minter (only by admin)
    pub fn remove_minter(env: Env, minter: Address) -> Result<(), Error> {
        let token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;

        // Require authorization from admin
        token_info.admin.require_auth();

        env.storage().instance().remove(&DataKey::Minter(minter));
        Ok(())
    }

    /// Check if address is a minter
    pub fn is_minter(env: Env, address: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Minter(address))
            .unwrap_or(false)
    }

    /// Create a snapshot for voting/rewards
    pub fn snapshot(env: Env) -> Result<u32, Error> {
        let token_info: TokenInfo = env
            .storage()
            .instance()
            .get(&DataKey::TokenInfo)
            .ok_or(Error::NotInitialized)?;

        // Require authorization from admin
        token_info.admin.require_auth();

        // Get current snapshot count
        let snapshot_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SnapshotCount)
            .unwrap_or(0);

        let snapshot_id = snapshot_count + 1;
        let block_number = env.ledger().sequence();

        // Create snapshot
        let snapshot = Snapshot {
            block_number,
            total_supply: token_info.total_supply,
        };

        env.storage()
            .instance()
            .set(&DataKey::Snapshot(snapshot_id), &snapshot);
        env.storage()
            .instance()
            .set(&DataKey::SnapshotCount, &snapshot_id);

        // Emit snapshot event
        let event = SnapshotEvent {
            id: snapshot_id,
            block_number,
        };
        env.events()
            .publish((Symbol::new(&env, "snapshot"),), event);

        Ok(snapshot_id)
    }

    /// Get balance at snapshot
    pub fn balance_of_at(env: Env, account: Address, snapshot_id: u32) -> Result<i128, Error> {
        let _snapshot: Snapshot = env
            .storage()
            .instance()
            .get(&DataKey::Snapshot(snapshot_id))
            .ok_or(Error::SnapshotNotFound)?;

        Ok(Self::get_balance_at_snapshot(&env, &account, snapshot_id))
    }

    /// Get total supply at snapshot
    pub fn total_supply_at(env: Env, snapshot_id: u32) -> Result<i128, Error> {
        let snapshot: Snapshot = env
            .storage()
            .instance()
            .get(&DataKey::Snapshot(snapshot_id))
            .ok_or(Error::SnapshotNotFound)?;

        Ok(snapshot.total_supply)
    }

    // Internal helper functions
    fn transfer_internal(
        env: &Env,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), Error> {
        // Check sender balance
        let from_balance = Self::balance_of(env.clone(), from.clone());
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        // Update balances
        let new_from_balance = from_balance - amount;
        if new_from_balance == 0 {
            env.storage()
                .persistent()
                .remove(&DataKey::Balance(from.clone()));
        } else {
            env.storage()
                .persistent()
                .set(&DataKey::Balance(from.clone()), &new_from_balance);
        }

        let to_balance = Self::balance_of(env.clone(), to.clone());
        let new_to_balance = to_balance + amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_to_balance);

        // Update checkpoints for both accounts
        Self::update_checkpoint(env, from, new_from_balance);
        Self::update_checkpoint(env, to, new_to_balance);

        Ok(())
    }

    fn update_checkpoint(env: &Env, user: &Address, new_balance: i128) {
        let current_snapshot_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SnapshotCount)
            .unwrap_or(0);

        if current_snapshot_count == 0 {
            return;
        }

        let checkpoint_count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::UserCheckpointCount(user.clone()))
            .unwrap_or(0);

        let mut checkpoints: Vec<Checkpoint> = env
            .storage()
            .persistent()
            .get(&DataKey::UserCheckpoints(user.clone()))
            .unwrap_or(Vec::new(env));

        if checkpoint_count == 0
            || checkpoints.get(checkpoint_count - 1).unwrap().snapshot_id < current_snapshot_count
        {
            let new_checkpoint = Checkpoint {
                snapshot_id: current_snapshot_count,
                balance: new_balance,
            };
            checkpoints.push_back(new_checkpoint);

            env.storage()
                .persistent()
                .set(&DataKey::UserCheckpoints(user.clone()), &checkpoints);
            env.storage().persistent().set(
                &DataKey::UserCheckpointCount(user.clone()),
                &(checkpoint_count + 1),
            );
        } else if checkpoint_count > 0 {
            let last_checkpoint = checkpoints.get(checkpoint_count - 1).unwrap();
            if last_checkpoint.snapshot_id == current_snapshot_count {
                let updated_checkpoint = Checkpoint {
                    snapshot_id: current_snapshot_count,
                    balance: new_balance,
                };
                checkpoints.set(checkpoint_count - 1, updated_checkpoint);
                env.storage()
                    .persistent()
                    .set(&DataKey::UserCheckpoints(user.clone()), &checkpoints);
            }
        }
    }

    fn get_balance_at_snapshot(env: &Env, user: &Address, snapshot_id: u32) -> i128 {
        let checkpoints: Vec<Checkpoint> = env
            .storage()
            .persistent()
            .get(&DataKey::UserCheckpoints(user.clone()))
            .unwrap_or(Vec::new(env));

        let checkpoint_count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::UserCheckpointCount(user.clone()))
            .unwrap_or(0);

        if checkpoint_count == 0 {
            return 0;
        }

        let mut low = 0u32;
        let mut high = checkpoint_count;

        while low < high {
            let mid = (low + high) / 2;
            let checkpoint = checkpoints.get(mid).unwrap();

            if checkpoint.snapshot_id <= snapshot_id {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if low == 0 {
            0
        } else {
            checkpoints.get(low - 1).unwrap().balance
        }
    }
}

// Tests temporarily disabled due to stellar-xdr dependency conflicts in Soroban SDK v20
// mod test;
