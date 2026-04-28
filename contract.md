# Smart Contract Implementation Guide

This file contains implementations and patterns for resolving key smart contract issues with proper separation of concerns.

---

## 1. Event Replay System

// ========================================
// EVENT REPLAY IMPLEMENTATION
// ========================================

/// Configuration structure for event replay operations
/// Allows filtering and range specification for event reconstruction
pub struct EventReplay {
    pub from_ledger: u32,
    pub to_ledger: Option<u32>,
    pub event_types: Vec<Symbol>,
    pub addresses: Vec<Address>,
}

/// Event structure for replay results
pub struct Event {
    pub ledger: u32,
    pub timestamp: u64,
    pub event_type: Symbol,
    pub address: Address,
    pub data: Vec<u8>,
}

/// Error types for event replay operations
#[derive(Debug)]
pub enum ReplayError {
    InvalidLedgerRange,
    EventNotFound,
    StorageError,
    FilterError,
}

/// Main event replay function
/// Retrieves and filters events based on configuration
pub fn replay_events(env: Env, config: EventReplay) -> Result<Vec<Event>, ReplayError> {
    // Validate ledger range
    if config.from_ledger < 1 {
        return Err(ReplayError::InvalidLedgerRange);
    }
    
    if let Some(to_ledger) = config.to_ledger {
        if to_ledger < config.from_ledger {
            return Err(ReplayError::InvalidLedgerRange);
        }
    }
    
    // Initialize events collection
    let mut events = Vec::new();
    
    // Iterate through ledger range
    let current_ledger = env.ledger().sequence;
    let end_ledger = config.to_ledger.unwrap_or(current_ledger);
    
    for ledger_seq in config.from_ledger..=end_ledger {
        // Get events for this ledger
        let ledger_events = get_ledger_events(&env, ledger_seq)?;
        
        // Filter events based on criteria
        for event in ledger_events {
            if should_include_event(&event, &config) {
                events.push(event);
            }
        }
    }
    
    Ok(events)
}

/// Helper function to retrieve events from a specific ledger
fn get_ledger_events(env: &Env, ledger_seq: u32) -> Result<Vec<Event>, ReplayError> {
    // Implementation would access ledger storage
    // This is a placeholder for actual ledger event retrieval
    Ok(Vec::new())
}

/// Helper function to filter events based on configuration
fn should_include_event(event: &Event, config: &EventReplay) -> bool {
    // Filter by event type
    if !config.event_types.is_empty() && !config.event_types.contains(&event.event_type) {
        return false;
    }
    
    // Filter by address
    if !config.addresses.is_empty() && !config.addresses.contains(&event.address) {
        return false;
    }
    
    true
}

/// State reconstruction from events
pub fn reconstruct_state(env: Env, events: Vec<Event>) -> Result<(), ReplayError> {
    for event in events {
        apply_event_to_state(&env, event)?;
    }
    Ok(())
}

/// Apply individual event to contract state
fn apply_event_to_state(env: &Env, event: Event) -> Result<(), ReplayError> {
    // Implementation would update contract state based on event
    // This varies by contract type and event structure
    Ok(())
}

---

## 2. Standardized Contract Metadata

// ========================================
// METADATA SCHEMA IMPLEMENTATION
// ========================================

/// Standardized metadata structure for contract discovery
#[derive(Serialize, Deserialize, Clone)]
pub struct ContractMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: AuthorInfo,
    pub license: String,
    pub repository: String,
    pub categories: Vec<String>,
    pub interfaces: Vec<String>,
    pub networks: Vec<NetworkInfo>,
}

/// Author information structure
#[derive(Serialize, Deserialize, Clone)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub website: Option<String>,
}

/// Network deployment information
#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub contract_address: Option<Address>,
    pub deployment_block: Option<u32>,
    pub is_active: bool,
}

/// Metadata management functions
impl ContractMetadata {
    /// Create new metadata instance
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: String::new(),
            author: AuthorInfo {
                name: String::new(),
                email: None,
                website: None,
            },
            license: "MIT".to_string(),
            repository: String::new(),
            categories: Vec::new(),
            interfaces: Vec::new(),
            networks: Vec::new(),
        }
    }
    
    /// Validate metadata completeness
    pub fn validate(&self) -> Result<(), MetadataError> {
        if self.name.is_empty() {
            return Err(MetadataError::MissingName);
        }
        
        if self.version.is_empty() {
            return Err(MetadataError::MissingVersion);
        }
        
        if self.author.name.is_empty() {
            return Err(MetadataError::MissingAuthor);
        }
        
        Ok(())
    }
    
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, MetadataError> {
        serde_json::to_string(self).map_err(|_| MetadataError::SerializationError)
    }
    
    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self, MetadataError> {
        serde_json::from_str(json).map_err(|_| MetadataError::DeserializationError)
    }
}

/// Metadata error types
#[derive(Debug)]
pub enum MetadataError {
    MissingName,
    MissingVersion,
    MissingAuthor,
    SerializationError,
    DeserializationError,
}

/// Metadata storage and retrieval
pub struct MetadataRegistry;

impl MetadataRegistry {
    /// Store contract metadata
    pub fn store_metadata(env: Env, address: Address, metadata: ContractMetadata) -> Result<(), MetadataError> {
        let key = DataKey::Metadata(address);
        let json = metadata.to_json()?;
        env.storage().persistent().set(&key, &json);
        Ok(())
    }
    
    /// Retrieve contract metadata
    pub fn get_metadata(env: Env, address: Address) -> Result<Option<ContractMetadata>, MetadataError> {
        let key = DataKey::Metadata(address);
        
        if let Some(json) = env.storage().persistent().get::<String>(&key) {
            let metadata = ContractMetadata::from_json(&json)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }
    
    /// List all registered contracts
    pub fn list_contracts(env: Env) -> Result<Vec<Address>, MetadataError> {
        // Implementation would scan storage for metadata entries
        Ok(Vec::new())
    }
}

/// Data keys for storage
#[derive(ContractType)]
pub enum DataKey {
    Metadata(Address),
    ContractRegistry,
}

---

## 3. Contract-to-Contract Interaction Patterns

// ========================================
// INTERACTION PATTERNS IMPLEMENTATION
// ========================================

/// Factory Pattern for creating contract instances
pub struct ContractFactory;

impl ContractFactory {
    /// Create new contract instance with configuration
    pub fn create_instance(env: Env, config: ContractConfig) -> Result<Address, FactoryError> {
        // Validate configuration
        config.validate()?;
        
        // Deploy new contract instance
        let contract_address = env.deployer().deploy_contract(
            &config.wasm_hash,
            &config.salt,
            &config.init_args
        )?;
        
        // Register instance if needed
        if config.auto_register {
            Self::register_instance(&env, contract_address, &config.metadata)?;
        }
        
        Ok(contract_address)
    }
    
    /// Register contract instance in factory registry
    fn register_instance(env: &Env, address: Address, metadata: &ContractMetadata) -> Result<(), FactoryError> {
        let key = DataKey::FactoryRegistry;
        let mut registry: Vec<Address> = env.storage().persistent().get(&key).unwrap_or_default();
        registry.push(address);
        env.storage().persistent().set(&key, &registry);
        
        // Store metadata
        MetadataRegistry::store_metadata(env.clone(), address, metadata.clone())?;
        
        Ok(())
    }
    
    /// Get instances created by this factory
    pub fn get_instances(env: Env) -> Result<Vec<Address>, FactoryError> {
        let key = DataKey::FactoryRegistry;
        Ok(env.storage().persistent().get(&key).unwrap_or_default())
    }
}

/// Configuration for contract deployment
#[derive(Clone)]
pub struct ContractConfig {
    pub wasm_hash: BytesN<32>,
    pub salt: BytesN<32>,
    pub init_args: Vec<Val>,
    pub metadata: ContractMetadata,
    pub auto_register: bool,
}

impl ContractConfig {
    pub fn validate(&self) -> Result<(), FactoryError> {
        if self.wasm_hash.is_zero() {
            return Err(FactoryError::InvalidWasmHash);
        }
        Ok(())
    }
}

/// Factory error types
#[derive(Debug)]
pub enum FactoryError {
    InvalidWasmHash,
    DeploymentFailed,
    RegistrationFailed,
    MetadataError(MetadataError),
}

/// Registry Pattern for contract discovery and management
pub struct ContractRegistry;

impl ContractRegistry {
    /// Register contract in global registry
    pub fn register_contract(env: Env, address: Address, metadata: ContractMetadata) -> Result<(), RegistryError> {
        // Validate metadata
        metadata.validate()?;
        
        // Check if already registered
        if Self::is_registered(&env, address)? {
            return Err(RegistryError::AlreadyRegistered);
        }
        
        // Store in registry
        let key = DataKey::ContractRegistry;
        let mut registry: RegistryEntry = env.storage().persistent().get(&key).unwrap_or_default();
        
        registry.contracts.insert(address, metadata);
        env.storage().persistent().set(&key, &registry);
        
        Ok(())
    }
    
    /// Unregister contract from registry
    pub fn unregister_contract(env: Env, address: Address) -> Result<(), RegistryError> {
        let key = DataKey::ContractRegistry;
        let mut registry: RegistryEntry = env.storage().persistent().get(&key).unwrap_or_default();
        
        if !registry.contracts.contains_key(&address) {
            return Err(RegistryError::NotRegistered);
        }
        
        registry.contracts.remove(&address);
        env.storage().persistent().set(&key, &registry);
        
        Ok(())
    }
    
    /// Get contract metadata from registry
    pub fn get_contract(env: Env, address: Address) -> Result<Option<ContractMetadata>, RegistryError> {
        let key = DataKey::ContractRegistry;
        let registry: RegistryEntry = env.storage().persistent().get(&key).unwrap_or_default();
        Ok(registry.contracts.get(&address).cloned())
    }
    
    /// Find contracts by category
    pub fn find_by_category(env: Env, category: &str) -> Result<Vec<(Address, ContractMetadata)>, RegistryError> {
        let key = DataKey::ContractRegistry;
        let registry: RegistryEntry = env.storage().persistent().get(&key).unwrap_or_default();
        
        let mut results = Vec::new();
        for (address, metadata) in registry.contracts {
            if metadata.categories.contains(&category.to_string()) {
                results.push((address, metadata));
            }
        }
        
        Ok(results)
    }
    
    /// Check if contract is registered
    fn is_registered(env: &Env, address: Address) -> Result<bool, RegistryError> {
        let key = DataKey::ContractRegistry;
        let registry: RegistryEntry = env.storage().persistent().get(&key).unwrap_or_default();
        Ok(registry.contracts.contains_key(&address))
    }
}

/// Registry storage structure
#[derive(ContractType, Default)]
pub struct RegistryEntry {
    pub contracts: Map<Address, ContractMetadata>,
}

/// Registry error types
#[derive(Debug)]
pub enum RegistryError {
    AlreadyRegistered,
    NotRegistered,
    MetadataError(MetadataError),
    StorageError,
}

/// Adapter Pattern for interface conversion
pub struct ContractAdapter<ExternalData, InternalData> {
    pub converter: fn(ExternalData) -> Result<InternalData, AdapterError>,
}

impl<ExternalData, InternalData> ContractAdapter<ExternalData, InternalData> {
    /// Create new adapter with conversion function
    pub fn new(converter: fn(ExternalData) -> Result<InternalData, AdapterError>) -> Self {
        Self { converter }
    }
    
    /// Convert external data to internal format
    pub fn adapt_interface(&self, data: ExternalData) -> Result<InternalData, AdapterError> {
        (self.converter)(data)
    }
    
    /// Batch conversion for multiple data items
    pub fn adapt_batch(&self, data_list: Vec<ExternalData>) -> Result<Vec<InternalData>, AdapterError> {
        let mut results = Vec::new();
        for data in data_list {
            results.push(self.adapt_interface(data)?);
        }
        Ok(results)
    }
}

/// Adapter error types
#[derive(Debug)]
pub enum AdapterError {
    ConversionFailed,
    InvalidFormat,
    MissingField,
}

/// Example adapters for common use cases
pub mod adapters {
    use super::*;
    
    /// Token amount adapter
    pub fn adapt_token_amount(external: TokenAmount) -> Result<InternalAmount, AdapterError> {
        if external.decimals > 18 {
            return Err(AdapterError::InvalidFormat);
        }
        
        Ok(InternalAmount {
            value: external.value,
            decimals: external.decimals,
            symbol: external.symbol,
        })
    }
    
    /// Address format adapter
    pub fn adapt_address(external: ExternalAddress) -> Result<Address, AdapterError> {
        Address::from_string(&external.value).map_err(|_| AdapterError::InvalidFormat)
    }
}

/// Example data structures for adapters
#[derive(Clone)]
pub struct TokenAmount {
    pub value: u128,
    pub decimals: u8,
    pub symbol: String,
}

#[derive(Clone)]
pub struct InternalAmount {
    pub value: u128,
    pub decimals: u8,
    pub symbol: String,
}

#[derive(Clone)]
pub struct ExternalAddress {
    pub value: String,
}

---

## 4. Unified Code Style Guide

// ========================================
// CODE STYLE GUIDE AND STANDARDS
// ========================================

/*
========================================
STANDARDIZED CODING CONVENTIONS
========================================
*/

// 1. NAMING CONVENTIONS
// =====================

// Functions: snake_case with descriptive verbs
pub fn calculate_token_balance() { }
pub fn validate_user_permissions() { }
pub fn emit_transfer_event() { }

// Variables: snake_case, meaningful names
let user_balance = 1000;
let contract_metadata = Metadata::new();
let is_authorized = true;

// Constants: SCREAMING_SNAKE_CASE
pub const MAX_SUPPLY: u128 = 1_000_000_000;
pub const DEFAULT_DECIMALS: u8 = 18;
pub const CONTRACT_VERSION: &str = "1.0.0";

// Structs/Enums: PascalCase
pub struct TokenContract { }
pub enum ContractState { }
pub trait EventInterface { }

// Type aliases: PascalCase with descriptive suffix
pub type ContractAddress = Address;
pub type TokenAmount = u128;
pub type Timestamp = u64;

// 2. CODE FORMATTING STANDARDS
// ===========================

// Indentation: 4 spaces (no tabs)
// Line length: Maximum 100 characters
// Blank lines: 1 between logical sections, 2 between major sections

// Function organization
pub fn example_function(
    param1: String,
    param2: u64,
) -> Result<ReturnType, ErrorType> {
    // Input validation first
    if param1.is_empty() {
        return Err(ErrorType::InvalidInput);
    }
    
    // Main logic
    let result = process_data(param1, param2)?;
    
    // Return result
    Ok(result)
}

// Struct organization
pub struct WellOrganizedStruct {
    // Public fields first
    pub public_field: u64,
    
    // Private fields after
    private_field: String,
    
    // Complex types last
    complex_data: Vec<CustomType>,
}

impl WellOrganizedStruct {
    // Constructor first
    pub fn new() -> Self {
        Self {
            public_field: 0,
            private_field: String::new(),
            complex_data: Vec::new(),
        }
    }
    
    // Public methods
    pub fn public_method(&self) -> Result<u64, Error> {
        // Implementation
        Ok(self.public_field)
    }
    
    // Private methods last
    fn private_method(&mut self) -> Result<(), Error> {
        // Implementation
        Ok(())
    }
}

// 3. MODULE ORGANIZATION GUIDELINES
// ================================

// File structure:
// - Imports and use statements
// - Constants and static variables
// - Type definitions (structs, enums)
// - Trait definitions
// - Implementation blocks
// - Public functions
// - Private functions
// - Unit tests (if applicable)

// Example module organization:
mod contract_module {
    // 1. Imports
    use soroban_sdk::{contractimpl, Address, Env};
    use super::types::*;
    
    // 2. Constants
    pub const MODULE_VERSION: &str = "1.0.0";
    
    // 3. Type definitions
    pub struct ModuleData {
        pub value: u64,
    }
    
    // 4. Trait definitions
    pub trait ModuleInterface {
        fn process(&self) -> Result<(), Error>;
    }
    
    // 5. Implementations
    impl ModuleData {
        pub fn new(value: u64) -> Self {
            Self { value }
        }
    }
    
    impl ModuleInterface for ModuleData {
        fn process(&self) -> Result<(), Error> {
            // Implementation
            Ok(())
        }
    }
    
    // 6. Public functions
    pub fn create_module(value: u64) -> ModuleData {
        ModuleData::new(value)
    }
    
    // 7. Private functions
    fn internal_helper(data: &ModuleData) -> u64 {
        data.value * 2
    }
}

// 4. COMMENT AND DOCUMENTATION STANDARDS
// ======================================

/// Documentation comments for public items
/// 
/// # Arguments
/// 
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
/// 
/// # Returns
/// 
/// Returns `Result` with success type or error
/// 
/// # Examples
/// 
/// ```
/// let result = public_function("input", 42);
/// assert!(result.is_ok());
/// ```
pub fn well_documented_function(param1: &str, param2: u64) -> Result<String, Error> {
    // Implementation comments explain complex logic
    let processed = format!("{}-{}", param1, param2); // Combine inputs
    
    Ok(processed)
}

// Inline comments for non-obvious code
let result = complex_calculation(
    input_data,
    MAX_ITERATIONS, // Use constant to avoid magic numbers
    &config,        // Pass config by reference for efficiency
);

// TODO comments for future improvements
// TODO: Add input validation for edge cases
// FIXME: This is a temporary workaround for performance issues

// 5. ERROR HANDLING PATTERNS
// ==========================

// Custom error types with descriptive variants
#[derive(Debug, PartialEq)]
pub enum ContractError {
    // Input validation errors
    InvalidInput(String),
    InsufficientBalance,
    UnauthorizedAccess,
    
    // State errors
    AlreadyInitialized,
    NotInitialized,
    
    // External errors
    ExternalCallFailed,
    NetworkError,
}

impl ContractError {
    /// Convert error to user-friendly message
    pub fn to_message(&self) -> &'static str {
        match self {
            ContractError::InvalidInput(_) => "Invalid input provided",
            ContractError::InsufficientBalance => "Insufficient balance",
            ContractError::UnauthorizedAccess => "Access denied",
            ContractError::AlreadyInitialized => "Contract already initialized",
            ContractError::NotInitialized => "Contract not initialized",
            ContractError::ExternalCallFailed => "External call failed",
            ContractError::NetworkError => "Network error occurred",
        }
    }
}

// Result type alias for consistency
pub type ContractResult<T> = Result<T, ContractError>;

// Error handling pattern
pub fn safe_operation(input: &str) -> ContractResult<u64> {
    // Validate input first
    if input.is_empty() {
        return Err(ContractError::InvalidInput("Empty input".to_string()));
    }
    
    // Parse with error handling
    let value = input.parse::<u64>()
        .map_err(|_| ContractError::InvalidInput("Invalid number".to_string()))?;
    
    // Business logic validation
    if value > MAX_SUPPLY {
        return Err(ContractError::InsufficientBalance);
    }
    
    Ok(value)
}

// 6. TEST STRUCTURE CONVENTIONS
// ==============================

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test organization: setup, execution, verification
    
    #[test]
    fn test_function_success_case() {
        // Setup
        let input = "valid_input";
        let expected = 42;
        
        // Execution
        let result = function_under_test(input);
        
        // Verification
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
    
    #[test]
    fn test_function_error_case() {
        // Setup
        let invalid_input = "";
        
        // Execution
        let result = function_under_test(invalid_input);
        
        // Verification
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ContractError::InvalidInput);
    }
    
    // Helper functions for tests
    fn setup_test_env() -> Env {
        // Common test setup
        Env::default()
    }
}

// 7. CI/CD ENFORCEMENT CHECKS
// ===========================

/*
CI/CD Pipeline Requirements:

1. Code Formatting:
   - rustfmt for consistent formatting
   - Maximum line length: 100 characters
   - 4-space indentation

2. Linting:
   - clippy for code quality
   - deny warnings in CI
   - Custom rules for contract-specific patterns

3. Testing:
   - Minimum 80% code coverage
   - All public functions must have tests
   - Integration tests for cross-contract interactions

4. Documentation:
   - All public items must have documentation
   - Examples for complex functions
   - README with usage instructions

5. Security:
   - Security audit for all changes
   - Dependency vulnerability scanning
   - Static analysis for common issues

Example CI/CD configuration:
```yaml
# .github/workflows/contract-checks.yml
name: Contract Quality Checks
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Check formatting
        run: cargo fmt -- --check
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Run tests
        run: cargo test --coverage
      - name: Check documentation
        run: cargo doc --no-deps
```
*/

// 8. CONTRACT-SPECIFIC PATTERNS
// =============================

// Event emission pattern
pub fn emit_standard_event(env: &Env, event_type: &str, data: &str) {
    let topics = (event_type,);
    env.events().publish(topics, data);
}

// Storage access pattern
pub fn safe_storage_get<T: ContractType>(env: &Env, key: &DataKey) -> Option<T> {
    env.storage().persistent().get(key)
}

pub fn safe_storage_set<T: ContractType>(env: &Env, key: &DataKey, value: &T) {
    env.storage().persistent().set(key, value);
}

// Authorization pattern
pub fn require_authorization(env: &Env, required_address: &Address) -> Result<(), ContractError> {
    let caller = env.current_contract_address();
    if &caller != required_address {
        return Err(ContractError::UnauthorizedAccess);
    }
    Ok(())
}

// Input validation pattern
pub fn validate_address(address: &Address) -> Result<(), ContractError> {
    if address.is_zero() {
        return Err(ContractError::InvalidInput("Zero address".to_string()));
    }
    Ok(())
}

pub fn validate_amount(amount: u128) -> Result<(), ContractError> {
    if amount == 0 {
        return Err(ContractError::InvalidInput("Zero amount".to_string()));
    }
    if amount > MAX_SUPPLY {
        return Err(ContractError::InsufficientBalance);
    }
    Ok(())
}

// 9. PERFORMANCE GUIDELINES
// =========================

// Use references for large parameters
pub fn process_large_data(data: &Vec<u8>) -> Result<(), Error> {
    // Process data efficiently
    Ok(())
}

// Minimize storage operations
pub fn batch_storage_update(env: &Env, updates: Vec<(DataKey, Val)>) -> Result<(), Error> {
    for (key, value) in updates {
        env.storage().persistent().set(&key, &value);
    }
    Ok(())
}

// Use efficient data structures
pub struct EfficientContract {
    // Use Map for key-value storage
    balances: Map<Address, u128>,
    // Use Vec for ordered data
    history: Vec<TransactionRecord>,
}

// 10. SECURITY PATTERNS
// ====================

// Reentrancy protection
pub struct ReentrancyGuard {
    entered: bool,
}

impl ReentrancyGuard {
    pub fn new() -> Self {
        Self { entered: false }
    }
    
    pub fn enter(&mut self) -> Result<(), ContractError> {
        if self.entered {
            return Err(ContractError::ReentrancyDetected);
        }
        self.entered = true;
        Ok(())
    }
    
    pub fn exit(&mut self) {
        self.entered = false;
    }
}

// Access control pattern
pub struct AccessControl {
    owner: Address,
    authorized: Map<Address, bool>,
}

impl AccessControl {
    pub fn require_owner(&self, caller: &Address) -> Result<(), ContractError> {
        if &self.owner != caller {
            return Err(ContractError::UnauthorizedAccess);
        }
        Ok(())
    }
    
    pub fn require_authorized(&self, caller: &Address) -> Result<(), ContractError> {
        if !self.authorized.get(caller).unwrap_or(false) {
            return Err(ContractError::UnauthorizedAccess);
        }
        Ok(())
    }
}

// Safe math operations
pub fn safe_add(a: u128, b: u128) -> Result<u128, ContractError> {
    a.checked_add(b)
        .ok_or(ContractError::Overflow)
}

pub fn safe_sub(a: u128, b: u128) -> Result<u128, ContractError> {
    a.checked_sub(b)
        .ok_or(ContractError::Underflow)
}

pub fn safe_mul(a: u128, b: u128) -> Result<u128, ContractError> {
    a.checked_mul(b)
        .ok_or(ContractError::Overflow)
}

/*
========================================
END OF STYLE GUIDE
========================================
*/
