#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct RouterFeeConfig {
    pub platform_fee_bps: u32,
    pub fee_receiver: Address,
}

const FEE_CONF: Symbol = symbol_short!("feeconf");

#[contract]
pub struct PaymentRouter;

#[contractimpl]
impl PaymentRouter {
    pub fn set_fee_config(env: Env, fee_receiver: Address, platform_fee_bps: u32) {
        if platform_fee_bps > 10_000 {
            panic!("Invalid fee bps");
        }
        let conf = RouterFeeConfig { fee_receiver, platform_fee_bps };
        env.storage().persistent().set(&FEE_CONF, &conf);
    }

    pub fn get_fee_config(env: Env) -> Option<RouterFeeConfig> {
        env.storage().persistent().get(&FEE_CONF)
    }

    pub fn compute_split(env: Env, amount: i128) -> (i128, i128) {
        let conf: RouterFeeConfig = env
            .storage()
            .persistent()
            .get(&FEE_CONF)
            .unwrap_or_else(|| panic!("Fee not set"));
        let fee = amount * (conf.platform_fee_bps as i128) / 10_000;
        let provider = amount - fee;
        env.events().publish((symbol_short!("FeeSplit"),), (provider, fee));
        (provider, fee)
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_fee_split() {
        let env = Env::default();
        let cid = env.register_contract(None, PaymentRouter);
        let client = PaymentRouterClient::new(&env, &cid);
        client.set_fee_config(&Address::generate(&env), &1000u32); // 10%
        let (provider, fee) = client.compute_split(&1000i128);
        assert_eq!(provider, 900);
        assert_eq!(fee, 100);
    }
}