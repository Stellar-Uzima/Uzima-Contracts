#[contracttype]
pub enum DrugDiscoveryEvent {
    MilestoneReached { research_id: BytesN<32>, milestone: u32 },
    TrialPhaseUpdated { research_id: BytesN<32>, phase: u32 },
    ResultPublished { research_id: BytesN<32>, timestamp: u64 },
}

/// Register a new healthcare provider
/// 
/// # Arguments
/// * `provider` - Address of the provider
/// * `credentials` - Verified credentials
/// 
/// # Errors
/// Returns `Error::AlreadyRegistered` if provider exists
/// 
/// # Example
/// ```
/// // Add example
/// ```

warning: unused import: `String`
 --> contracts/secure_enclave/src/lib.rs:5:5