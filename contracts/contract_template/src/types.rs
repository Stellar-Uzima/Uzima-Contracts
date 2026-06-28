use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone)]
pub struct ContractData {
    pub owner: Address,
    pub value: String,
}
