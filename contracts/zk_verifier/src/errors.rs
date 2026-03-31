use soroban_sdk::contracterror;

#[contracterror]
pub enum ZkError {
    InvalidProof = 1,
    VerificationFailed = 2,
}
