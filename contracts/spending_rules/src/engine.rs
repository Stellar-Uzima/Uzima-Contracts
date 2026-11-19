use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol};

use crate::errors::Error;
use crate::types::Rule;
use crate::DataKey;

pub fn evaluate_transaction(
    env: &Env,
    user: &Address,
    category: &Symbol,
    amount: i128,
    zk_proof: &Option<Bytes>,
) -> Result<(), Error> {
    let rule: Rule = env
        .storage()
        .persistent()
        .get(&DataKey::Rule((user.clone(), category.clone())))
        .ok_or(Error::RuleNotFound)?;

    if amount > rule.zk_required_above {
        match zk_proof {
            None => return Err(Error::ZkProofRequired),
            Some(proof) => {
                let zk_verifier_addr: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::ZkVerifierAddr)
                    .ok_or(Error::NotInitialized)?;

                let public_inputs_hash: BytesN<32> = env.crypto().sha256(proof).into();

                let vk_version: u32 = 1;
                let verified: bool = env.invoke_contract(
                    &zk_verifier_addr,
                    &Symbol::new(env, "verify_proof"),
                    soroban_sdk::vec![
                        env,
                        vk_version.into_val(env),
                        public_inputs_hash.into_val(env),
                        proof.clone().into_val(env),
                    ],
                );
                if !verified {
                    return Err(Error::ZkVerificationFailed);
                }
            }
        }
    }

    let spending_limits_addr: Address = env
        .storage()
        .instance()
        .get(&DataKey::SpendingLimitsAddr)
        .ok_or(Error::NotInitialized)?;

    let within_limit: bool = env.invoke_contract(
        &spending_limits_addr,
        &Symbol::new(env, "check_limit"),
        soroban_sdk::vec![
            env,
            user.clone().into_val(env),
            amount.into_val(env),
        ],
    );
    if !within_limit {
        return Err(Error::OverallLimitExceeded);
    }

    let spending_categories_addr: Address = env
        .storage()
        .instance()
        .get(&DataKey::SpendingCategoriesAddr)
        .ok_or(Error::NotInitialized)?;

    let current_category_spent: i128 = env.invoke_contract(
        &spending_categories_addr,
        &Symbol::new(env, "get_category_spent"),
        soroban_sdk::vec![
            env,
            user.clone().into_val(env),
            category.clone().into_val(env),
        ],
    );

    if current_category_spent + amount > rule.weekly_limit {
        return Err(Error::CategoryLimitExceeded);
    }

    env.invoke_contract(
        &spending_categories_addr,
        &Symbol::new(env, "record_category_spent"),
        soroban_sdk::vec![
            env,
            user.clone().into_val(env),
            category.clone().into_val(env),
            amount.into_val(env),
        ],
    );

    env.invoke_contract(
        &spending_limits_addr,
        &Symbol::new(env, "record_spent"),
        soroban_sdk::vec![
            env,
            user.clone().into_val(env),
            amount.into_val(env),
        ],
    );

    Ok(())
}
