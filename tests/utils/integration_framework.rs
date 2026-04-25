/// Integration testing framework for Uzima Contracts
use soroban_sdk::{
    Address, BytesN, Env, IntoVal, String as SorobanString, Val, Vec,
    testutils::{Address as _, Events, Ledger},
    vec,
};
use crate::utils::{UserFixture, UserFixtureFactory, HealthcareTeam};

// Import contract types for registration helpers
// Note: In a real multi-crate workspace, these might need to be imported via crate features or separate dev-dependencies

/// A unified environment for integration testing multiple contracts
pub struct IntegrationTestEnv {
    pub env: Env,
    pub admin: Address,
    pub team: HealthcareTeam,
}

impl IntegrationTestEnv {
    /// Initialize a new integration test environment with a full healthcare team
    pub fn new() -> Self {
        let env = Env::default();
        // Enable mock auths by default for integration tests to focus on logic
        env.mock_all_auths();
        
        let team = UserFixtureFactory::create_healthcare_team(&env);
        let admin = team.admin.address.clone();

        Self {
            env,
            admin,
            team,
        }
    }

    /// Advance the ledger time by a specific number of seconds
    pub fn jump_time(&self, seconds: u64) {
        let current_time = self.env.ledger().timestamp();
        self.env.ledger().with_mut(|l| {
            l.timestamp = current_time + seconds;
        });
    }

    /// Set the ledger time to a specific timestamp
    pub fn set_time(&self, timestamp: u64) {
        self.env.ledger().with_mut(|l| {
            l.timestamp = timestamp;
        });
    }

    /// Advance the ledger time by a specific number of days
    pub fn jump_days(&self, days: u64) {
        self.jump_time(days * 86400);
    }

    /// Get all events emitted during the test so far
    pub fn get_events(&self) -> Vec<(Address, Vec<Val>, Val)> {
        self.env.events().all()
    }

    /// Assert that a specific event was emitted
    pub fn assert_event_emitted(&self, contract_id: &Address, topics: Vec<Val>, data: Val) {
        let events = self.env.events().all();
        let found = events.iter().any(|(id, t, d)| {
            id == *contract_id && t == topics && d == data
        });
        assert!(found, "Expected event not found for contract {:?}", contract_id);
    }

    /// Assert that an event with specific topics was emitted (ignoring data)
    pub fn assert_event_topics(&self, contract_id: &Address, topics: Vec<Val>) {
        let events = self.env.events().all();
        let found = events.iter().any(|(id, t, _)| {
            id == *contract_id && t == topics
        });
        assert!(found, "Expected event topics not found for contract {:?}", contract_id);
    }

    /// Generate a new random address in the test environment
    pub fn generate_address(&self) -> Address {
        Address::generate(&self.env)
    }

    /// Utility to convert a value into a Soroban Val
    pub fn to_val<T: IntoVal<Env, Val>>(&self, val: T) -> Val {
        val.into_val(&self.env)
    }

    /// Utility to create a Soroban Vec of topics
    pub fn topics<T: IntoVal<Env, Val>>(&self, topics: &[T]) -> Vec<Val> {
        let mut v = Vec::new(&self.env);
        for t in topics {
            v.push(t.into_val(&self.env));
        }
        v
    }

    // --- Contract Registration Helpers ---

    /// Create a standard SAC (Stellar Asset Contract) token for testing
    pub fn register_sac_token(&self, admin: &Address) -> Address {
        self.env.register_stellar_asset_contract(admin.clone())
    }
}

/// Helper to mock a contract call with a specific result
pub trait MockService {
    fn setup_mock_response<T: IntoVal<Env, Val>>(&self, contract_id: &Address, fn_name: &str, response: T);
}

impl MockService for IntegrationTestEnv {
    fn setup_mock_response<T: IntoVal<Env, Val>>(&self, _contract_id: &Address, _fn_name: &str, _response: T) {
        // Implementation for mocking external contract calls
        // In Soroban, this is usually done by registering a mock contract implementation.
        // This is a placeholder for more complex logic.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_initialization() {
        let test_env = IntegrationTestEnv::new();
        assert!(test_env.team.doctors.len() > 0);
        assert_eq!(test_env.admin, test_env.team.admin.address);
    }

    #[test]
    fn test_time_control() {
        let test_env = IntegrationTestEnv::new();
        let start_time = test_env.env.ledger().timestamp();
        test_env.jump_time(3600);
        assert_eq!(test_env.env.ledger().timestamp(), start_time + 3600);
    }

    #[test]
    fn test_contract_registration() {
        let test_env = IntegrationTestEnv::new();
        let token_id = test_env.register_sac_token(&test_env.admin);
        assert!(token_id.to_string().len() > 0);
    }
}

