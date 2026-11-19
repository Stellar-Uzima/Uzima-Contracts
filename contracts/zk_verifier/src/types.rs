use crate::{errors::Error, types::ZkProof};
use soroban_sdk::{BytesN, Env};

pub fn verify_groth16_proof(
    _env: Env,
    proof: ZkProof,
    _public_inputs: BytesN<32>,
) -> Result<bool, Error> {
    if proof.a.len() == 0 || proof.b.len() == 0 || proof.c.len() == 0 {
        return Err(Error::InvalidProof);
    }

    Err(Error::VerificationFailed)
}
