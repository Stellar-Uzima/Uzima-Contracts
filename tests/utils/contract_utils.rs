/// Contract utilities for common testing operations
use soroban_sdk::{Address, Env, String as SorobanString};

/// Result type for contract operations
pub type ContractResult<T> = Result<T, String>;

/// Setup helper for contract initialization
pub struct ContractSetup {
    pub env: Env,
    pub admin: Address,
    pub users: Vec<Address>,
}

impl ContractSetup {
    /// Create a new contract setup with test environment
    pub fn new() -> Self {
        let env = Env::default();
        let admin = Address::generate(&env);
        
        Self {
            env,
            admin,
            users: Vec::new(),
        }
    }

    /// Add mock authentication for all calls
    pub fn with_mock_auth(mut self) -> Self {
        self.env.mock_all_auths();
        self
    }

    /// Generate a random test address
    pub fn generate_address(&self) -> Address {
        Address::generate(&self.env)
    }

    /// Create N test users
    pub fn create_users(&mut self, count: usize) -> Vec<Address> {
        let mut users = Vec::new();
        for _ in 0..count {
            users.push(self.generate_address());
        }
        self.users = users.clone();
        users
    }
}

/// Error assertion helpers
pub fn assert_contract_error(result: Result<(), i32>, expected_code: i32) {
    match result {
        Err(code) => assert_eq!(code, expected_code, "Contract error code mismatch"),
        Ok(_) => panic!("Expected contract error but succeeded"),
    }
}

/// Success assertion helper
pub fn assert_contract_success<T>(result: Result<T, i32>) -> T {
    result.expect("Contract operation failed")
}

/// Helper to convert string literals to Soroban strings
pub fn to_soroban_string(env: &Env, s: &str) -> SorobanString {
    SorobanString::from_str(env, s)
}

/// Test timing utilities
pub struct TestTimer {
    start: std::time::Instant,
}

impl TestTimer {
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_setup_creation() {
        let setup = ContractSetup::new();
        assert_eq!(setup.users.len(), 0);
    }

    #[test]
    fn test_generate_address() {
        let setup = ContractSetup::new();
        let addr1 = setup.generate_address();
        let addr2 = setup.generate_address();
        // Addresses should be different
        assert_ne!(addr1, addr2);
    }

    #[test]
    fn test_create_users() {
        let mut setup = ContractSetup::new();
        let users = setup.create_users(5);
        assert_eq!(users.len(), 5);
        assert_eq!(setup.users.len(), 5);
    }

    #[test]
    fn test_timer() {
        let timer = TestTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
