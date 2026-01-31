#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Address, Env, Map, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    NotArbiter = 2,
    DisputeNotFound = 3,
}

#[contract]
pub struct DisputeResolution;

#[contractimpl]
impl DisputeResolution {
    pub fn initialize(env: Env, arbiters: Vec<Address>) {
        env.storage()
            .instance()
            .set(&symbol_short!("arbiters"), &arbiters);
    }

    // Flags a proposal as disputed.
    // In a real system, you might require a token bond (transfer) here.
    pub fn dispute(env: Env, proposal_id: u64, challenger: Address) {
        challenger.require_auth();
        // Here you would normally call token.transfer_from(challenger, contract, BOND_AMOUNT)

        let mut disputes: Map<u64, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("disputes"))
            .unwrap_or(Map::new(&env));
        disputes.set(proposal_id, true); // True means active dispute
        env.storage()
            .persistent()
            .set(&symbol_short!("disputes"), &disputes);
    }

    // Arbiters can resolve the dispute.
    // valid_proposal = true (clear dispute), false (kill proposal)
    pub fn resolve(
        env: Env,
        proposal_id: u64,
        arbiter: Address,
        valid_proposal: bool,
    ) -> Result<(), Error> {
        arbiter.require_auth();
        let arbiters: Vec<Address> = env
            .storage()
            .instance()
            .get(&symbol_short!("arbiters"))
            .ok_or(Error::NotInitialized)?;
        if !arbiters.contains(&arbiter) {
            return Err(Error::NotArbiter);
        }

        let mut disputes: Map<u64, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("disputes"))
            .ok_or(Error::DisputeNotFound)?;

        if valid_proposal {
            disputes.remove(proposal_id); // Remove dispute, allow execution
        } else {
            // Leave dispute active effectively blocking it forever (or add specific status)
            disputes.set(proposal_id, true);
        }
        env.storage()
            .persistent()
            .set(&symbol_short!("disputes"), &disputes);
        Ok(())
    }

    pub fn is_disputed(env: Env, proposal_id: u64) -> bool {
        let disputes: Map<u64, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("disputes"))
            .unwrap_or(Map::new(&env));
        disputes.get(proposal_id).unwrap_or(false)
    }
}
