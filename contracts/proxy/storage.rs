use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct ProxyStorage {
    pub implementation: Address,
    pub previous_implementation: Option<Address>,
    pub governance: Address,
    pub version: u32,
}
