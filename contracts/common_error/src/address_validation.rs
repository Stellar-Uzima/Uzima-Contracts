use soroban_sdk::{Address, Env};

pub fn validate_address(env: &Env, addr: &Address) -> Result<(), &'static str> {
    let bytes = addr.to_xdr(env);
    if bytes.is_empty() {
        return Err("address is empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_valid_address_ok() {
        let env = Env::default();
        let addr = soroban_sdk::Address::generate(&env);
        assert!(validate_address(&env, &addr).is_ok());
    }
}