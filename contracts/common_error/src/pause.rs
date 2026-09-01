use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

#[derive(Clone)]
#[contracttype]
pub enum PauseKey {
    Paused,
    Pauser,
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get::<_, bool>(&PauseKey::Paused).unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool, caller: Address) -> Result<(), &'static str> {
    let pauser: Address = env.storage().instance().get(&PauseKey::Pauser)
        .ok_or("no pauser set")?;
    if caller != pauser {
        return Err("unauthorized");
    }
    env.storage().instance().set(&PauseKey::Paused, &paused);
    if paused {
        env.events().publish((symbol_short!("pause"),), (caller,));
    } else {
        env.events().publish((symbol_short!("unpause"),), (caller,));
    }
    Ok(())
}

pub fn initialize_pauser(env: &Env, pauser: Address) {
    env.storage().instance().set(&PauseKey::Pauser, &pauser);
    env.storage().instance().set(&PauseKey::Paused, &false);
}