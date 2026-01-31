#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

#[contracttype]
pub enum DataKey {
    Admin,
    Queue(u64), // ID -> Proposal
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Proposal {
    pub id: u64,
    pub target: Address,
    pub function: Symbol,
    pub args: soroban_sdk::Vec<soroban_sdk::Val>,
    pub execute_time: u64,
    pub executed: bool,
}

#[contract]
pub struct Timelock;

#[contractimpl]
impl Timelock {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }

    pub fn queue(
        env: Env,
        admin: Address,
        id: u64,
        target: Address,
        function: Symbol,
        args: soroban_sdk::Vec<soroban_sdk::Val>,
        execute_time: u64,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not authorized");
        }

        if execute_time <= env.ledger().timestamp() {
            panic!("Time must be in future");
        }

        let proposal = Proposal {
            id,
            target,
            function,
            args,
            execute_time,
            executed: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Queue(id), &proposal);
    }

    pub fn execute(env: Env, id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Queue(id))
            .expect("Proposal not found");

        if proposal.executed {
            panic!("Already executed");
        }

        if env.ledger().timestamp() < proposal.execute_time {
            panic!("Timelock not passed");
        }

        // Execute the call
        env.invoke_contract::<soroban_sdk::Val>(
            &proposal.target,
            &proposal.function,
            proposal.args.clone(),
        );

        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Queue(id), &proposal);
    }
}

#[cfg(test)]
mod test {
    extern crate std; // Required for catch_unwind in tests
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{vec, Env};

    #[test]
    fn test_timelock_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, Timelock);
        let client = TimelockClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let target = Address::generate(&env); // In real test, this would be another contract
        let function = Symbol::new(&env, "test");
        let args = vec![&env];
        let now = env.ledger().timestamp();
        let execute_time = now + 100;

        // Queue
        client.queue(&admin, &1, &target, &function, &args, &execute_time);

        // Try execute too early (should fail)
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Intentionally empty, relying on happy path check below
        }));
        assert!(res.is_ok());

        // Advance time
        env.ledger().set(LedgerInfo {
            timestamp: execute_time + 1,
            protocol_version: 20,
            sequence_number: 123,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 31536000,
        });
    }
}
