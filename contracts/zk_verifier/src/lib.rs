#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct VerifyingKeyConfig {
    pub version: u32,
    pub vk_hash: BytesN<32>,
    pub circuit_id: BytesN<32>,
    pub attestor: Address,
    pub metadata_hash: BytesN<32>,
    pub created_at: u64,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProofAttestation {
    pub vk_version: u32,
    pub public_inputs_hash: BytesN<32>,
    pub proof_hash: BytesN<32>,
    pub verified: bool,
    pub attestor: Address,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct NullifierRecord {
    pub nullifier: BytesN<32>,
    pub consumed_at: u64,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    CurrentVersion,
    DefaultTtl,
    VerifyingKey(u32),
    Attestation(BytesN<32>),
    Nullifier(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidInput = 4,
    VersionNotFound = 5,
}

const MAX_DEFAULT_TTL: u64 = 86_400;
const MIN_DEFAULT_TTL: u64 = 1;

#[contract]
pub struct ZkVerifierContract;

#[contractimpl]
impl ZkVerifierContract {
    pub fn initialize(env: Env, admin: Address, default_ttl: u64) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        if !(MIN_DEFAULT_TTL..=MAX_DEFAULT_TTL).contains(&default_ttl) {
            return Err(Error::InvalidInput);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::CurrentVersion, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::DefaultTtl, &default_ttl);

        env.events()
            .publish((symbol_short!("ZKVER"), symbol_short!("INIT")), admin);
        Ok(())
    }

    pub fn set_default_ttl(env: Env, caller: Address, ttl: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &caller)?;

        if !(MIN_DEFAULT_TTL..=MAX_DEFAULT_TTL).contains(&ttl) {
            return Err(Error::InvalidInput);
        }
        env.storage().instance().set(&DataKey::DefaultTtl, &ttl);
        env.events()
            .publish((symbol_short!("ZKVER"), symbol_short!("TTL")), ttl);
        Ok(true)
    }

    pub fn get_default_ttl(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::DefaultTtl)
            .unwrap_or(300)
    }

    pub fn register_verifying_key(
        env: Env,
        caller: Address,
        vk_hash: BytesN<32>,
        circuit_id: BytesN<32>,
        attestor: Address,
        metadata_hash: BytesN<32>,
    ) -> Result<u32, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &caller)?;

        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentVersion)
            .unwrap_or(0);
        let next = current.saturating_add(1);

        let key = VerifyingKeyConfig {
            version: next,
            vk_hash,
            circuit_id,
            attestor: attestor.clone(),
            metadata_hash,
            created_at: env.ledger().timestamp(),
            active: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::VerifyingKey(next), &key);
        env.storage()
            .instance()
            .set(&DataKey::CurrentVersion, &next);

        env.events().publish(
            (symbol_short!("ZKVER"), symbol_short!("VKREG")),
            (next, attestor),
        );
        Ok(next)
    }

    pub fn deactivate_verifying_key(
        env: Env,
        caller: Address,
        version: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &caller)?;

        let mut key: VerifyingKeyConfig = env
            .storage()
            .persistent()
            .get(&DataKey::VerifyingKey(version))
            .ok_or(Error::VersionNotFound)?;
        if !key.active {
            return Ok(false);
        }
        key.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::VerifyingKey(version), &key);

        env.events()
            .publish((symbol_short!("ZKVER"), symbol_short!("VKOFF")), version);
        Ok(true)
    }

    pub fn get_verifying_key(env: Env, version: u32) -> Option<VerifyingKeyConfig> {
        env.storage()
            .persistent()
            .get(&DataKey::VerifyingKey(version))
    }

    pub fn get_current_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CurrentVersion)
            .unwrap_or(0)
    }

    pub fn submit_attestation(
        env: Env,
        attestor: Address,
        vk_version: u32,
        public_inputs_hash: BytesN<32>,
        proof_hash: BytesN<32>,
        verified: bool,
        ttl: u64,
    ) -> Result<bool, Error> {
        attestor.require_auth();
        Self::require_initialized(&env)?;

        let key = Self::read_vk(&env, vk_version)?;
        if !key.active || key.attestor != attestor {
            return Err(Error::NotAuthorized);
        }

        let effective_ttl = if ttl == 0 {
            Self::get_default_ttl(env.clone())
        } else {
            let cap = Self::get_default_ttl(env.clone());
            if ttl > cap {
                cap
            } else {
                ttl
            }
        };

        let attestation = ProofAttestation {
            vk_version,
            public_inputs_hash: public_inputs_hash.clone(),
            proof_hash: proof_hash.clone(),
            verified,
            attestor,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp().saturating_add(effective_ttl),
        };
        let attestation_key =
            Self::compute_attestation_key(&env, vk_version, &public_inputs_hash, &proof_hash);
        env.storage()
            .persistent()
            .set(&DataKey::Attestation(attestation_key), &attestation);

        env.events().publish(
            (symbol_short!("ZKVER"), symbol_short!("ATTEST")),
            (vk_version, verified),
        );
        Ok(true)
    }

    pub fn verify_proof(
        env: Env,
        vk_version: u32,
        public_inputs_hash: BytesN<32>,
        proof: Bytes,
    ) -> bool {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return false;
        }

        let key: VerifyingKeyConfig = match env
            .storage()
            .persistent()
            .get(&DataKey::VerifyingKey(vk_version))
        {
            Some(v) => v,
            None => return false,
        };
        if !key.active {
            return false;
        }

        let proof_hash: BytesN<32> = env.crypto().sha256(&proof).into();
        let attestation_key =
            Self::compute_attestation_key(&env, vk_version, &public_inputs_hash, &proof_hash);
        let attestation: ProofAttestation = match env
            .storage()
            .persistent()
            .get(&DataKey::Attestation(attestation_key))
        {
            Some(v) => v,
            None => return false,
        };

        if !attestation.verified {
            return false;
        }
        if attestation.expires_at <= env.ledger().timestamp() {
            return false;
        }
        true
    }

    pub fn get_attestation(
        env: Env,
        vk_version: u32,
        public_inputs_hash: BytesN<32>,
        proof_hash: BytesN<32>,
    ) -> Option<ProofAttestation> {
        let key = Self::compute_attestation_key(&env, vk_version, &public_inputs_hash, &proof_hash);
        env.storage().persistent().get(&DataKey::Attestation(key))
    }

    pub fn compute_proof_hash(env: Env, proof: Bytes) -> BytesN<32> {
        env.crypto().sha256(&proof).into()
    }

    pub fn mark_nullifier_used(env: Env, nullifier: BytesN<32>) -> bool {
        let key = DataKey::Nullifier(nullifier.clone());
        if env.storage().persistent().has(&key) {
            return false;
        }

        let value = NullifierRecord {
            nullifier,
            consumed_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &value);
        true
    }

    pub fn is_nullifier_used(env: Env, nullifier: BytesN<32>) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Nullifier(nullifier))
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin == *caller {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn read_vk(env: &Env, version: u32) -> Result<VerifyingKeyConfig, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::VerifyingKey(version))
            .ok_or(Error::VersionNotFound)
    }

    fn compute_attestation_key(
        env: &Env,
        vk_version: u32,
        public_inputs_hash: &BytesN<32>,
        proof_hash: &BytesN<32>,
    ) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, &vk_version.to_be_bytes()));
        Self::append_bytes32(env, &mut payload, public_inputs_hash);
        Self::append_bytes32(env, &mut payload, proof_hash);
        env.crypto().sha256(&payload).into()
    }

    fn append_bytes32(env: &Env, payload: &mut Bytes, value: &BytesN<32>) {
        payload.append(&Bytes::from_slice(env, &value.to_array()));
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_attested_verification_flow() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().with_mut(|li| {
            li.timestamp = 1_000;
            li.sequence_number = 11;
        });

        let admin = Address::generate(&env);
        let attestor = Address::generate(&env);
        let contract_id = env.register_contract(None, ZkVerifierContract);
        let client = ZkVerifierContractClient::new(&env, &contract_id);
        client.initialize(&admin, &600);

        let vk_hash = BytesN::from_array(&env, &[1u8; 32]);
        let circuit_id = BytesN::from_array(&env, &[2u8; 32]);
        let metadata_hash = BytesN::from_array(&env, &[3u8; 32]);
        let version =
            client.register_verifying_key(&admin, &vk_hash, &circuit_id, &attestor, &metadata_hash);
        assert_eq!(version, 1);

        let public_inputs_hash = BytesN::from_array(&env, &[4u8; 32]);
        let proof = Bytes::from_slice(&env, b"proof-v1");
        let proof_hash: BytesN<32> = env.crypto().sha256(&proof).into();

        client.submit_attestation(&attestor, &1, &public_inputs_hash, &proof_hash, &true, &300);

        assert!(client.verify_proof(&1, &public_inputs_hash, &proof));

        env.ledger().set_timestamp(2_000);
        let public_inputs_hash_2 = BytesN::from_array(&env, &[5u8; 32]);
        let proof_2 = Bytes::from_slice(&env, b"proof-v2");
        assert!(!client.verify_proof(&1, &public_inputs_hash_2, &proof_2));
    }
}
