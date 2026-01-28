use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub struct UpgradeProposal {
    pub new_implementation: Address,
    pub votes_for: u32,
    pub votes_against: u32,
    pub deadline: u64,
    pub executed: bool,
}

#[contract]
pub struct UpgradeGovernance;

#[contractimpl]
impl UpgradeGovernance {
    pub fn create_proposal(
        env: Env,
        new_impl: Address,
        deadline: u64,
    ) -> UpgradeProposal {
        UpgradeProposal {
            new_implementation: new_impl,
            votes_for: 0,
            votes_against: 0,
            deadline,
            executed: false,
        }
    }

    pub fn vote_for(env: Env, mut proposal: UpgradeProposal) -> UpgradeProposal {
        env.invoker().require_auth();
        proposal.votes_for += 1;
        proposal
    }

    pub fn vote_against(env: Env, mut proposal: UpgradeProposal) -> UpgradeProposal {
        env.invoker().require_auth();
        proposal.votes_against += 1;
        proposal
    }

    pub fn can_execute(env: Env, proposal: &UpgradeProposal) -> bool {
        env.ledger().timestamp() > proposal.deadline
            && proposal.votes_for > proposal.votes_against
            && !proposal.executed
    }
}
