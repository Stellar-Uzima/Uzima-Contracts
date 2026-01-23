#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct ComplianceRule {
    pub region: String,          
    pub category: String,        
    pub requires_explicit_consent: bool,
    pub min_retention_days: u64,
    pub max_retention_days: u64,
    pub data_residency_required: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    NotAuthorized = 1,
    RuleNotFound = 2,
    ValidationFailed = 3,
}

#[contract]
pub struct ComplianceRulesContract;

#[contractimpl]
impl ComplianceRulesContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&symbol_short!("ADMIN")) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&symbol_short!("ADMIN"), &admin);
    }


    pub fn set_rule(
        env: Env, 
        region: String, 
        category: String, 
        rule: ComplianceRule
    ) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&symbol_short!("ADMIN")).unwrap();
        admin.require_auth();

        let key = (region, category);
        env.storage().persistent().set(&key, &rule);
        Ok(())
    }



    pub fn check_compliance(
        env: Env,
        actor_region: String,
        data_region: String,
        category: String,
        consent_valid: bool,
    ) -> bool {
    
    
        let key = (data_region.clone(), category.clone());
        let rule: ComplianceRule = match env.storage().persistent().get(&key) {
            Some(r) => r,
            None => {
            
                match env.storage().persistent().get(&(String::from_str(&env, "GLOBAL"), category)) {
                    Some(r) => r,
                    None => return true,
                }
            }
        };

    
        if rule.data_residency_required && actor_region != data_region {
            return false;
        }

    
        if rule.requires_explicit_consent && !consent_valid {
            return false;
        }

        true
    }
    

    pub fn get_retention_policy(env: Env, region: String, category: String) -> (u64, u64) {
         let key = (region.clone(), category.clone());
         if let Some(rule) = env.storage().persistent().get::<_, ComplianceRule>(&key) {
             (rule.min_retention_days, rule.max_retention_days)
         } else {
             (0, 0)
         }
    }
}