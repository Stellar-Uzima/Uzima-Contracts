#![no_std]

use soroban_sdk::{contract, contractimpl, Env, BytesN};

mod verifier;
mod types;
mod errors;

use verifier::verify_groth16_proof;
use types::ZkProof;
use errors::ZkError;

#[contract]
pub struct ZkVerifierContract;

#[contractimpl]
impl ZkVerifierContract {
    pub fn verify(
        env: Env,
        proof: ZkProof,
        public_inputs: BytesN<32>,
    ) -> Result<bool, ZkError> {
        verify_groth16_proof(env, proof, public_inputs)
    }
}
