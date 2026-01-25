use soroban_sdk::contracterror;

#[contracterror]
pub enum MedicalError {
    Unauthorized = 1,
    ZkVerificationFailed = 2,
}
