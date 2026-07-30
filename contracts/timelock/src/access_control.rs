use soroban_sdk::Env;

pub fn require_role(env: &Env, role: u32, caller: &soroban_sdk::Address) -> Result<(), crate::errors::Error> {
    if caller != &env.current_contract_address() {
        return Err(crate::errors::Error::Unauthorized);
    }
    Ok(())
}