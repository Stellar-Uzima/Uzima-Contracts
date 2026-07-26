use soroban_sdk::{contracttype, Address, Env};

#[derive(Clone)]
#[contracttype]
pub enum NonceKey {
    Nonce(Address),
}

pub fn get_nonce(env: &Env, user: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&NonceKey::Nonce(user.clone()))
        .unwrap_or(0)
}

pub fn consume_nonce(env: &Env, user: &Address, expected: u64) -> Result<(), &'static str> {
    let current = get_nonce(env, user);
    if current != expected {
        return Err("invalid nonce");
    }
    env.storage()
        .persistent()
        .set(&NonceKey::Nonce(user.clone()), &(current + 1));
    Ok(())
}

pub fn set_nonce(env: &Env, user: &Address, nonce: u64) {
    env.storage()
        .persistent()
        .set(&NonceKey::Nonce(user.clone()), &nonce);
}