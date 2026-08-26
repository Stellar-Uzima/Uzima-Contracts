use soroban_sdk::{contracttype, Symbol};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Rule {
    pub category: Symbol,
    pub weekly_limit: i128,
    pub zk_required_above: i128,
}
