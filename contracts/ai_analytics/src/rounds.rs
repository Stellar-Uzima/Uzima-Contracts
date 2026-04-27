use soroban_sdk::{symbol_short, Address, BytesN, Env, String};

use crate::{
    serialization_utils::{SafeSerialize, SerializationError},
    types::{DataKey, Error, FederatedRound, ModelMetadata, ParticipantUpdateMeta},
    utils,
};

pub fn start_round(
    env: Env,
    caller: Address,
    base_model_id: BytesN<32>,
    min_participants: u32,
    dp_epsilon: u32,
) -> Result<u64, Error> {
    caller.require_auth();
    utils::ensure_admin(&env, &caller)?;

    if min_participants == 0 {
        return Err(Error::NotEnoughParticipants);
    }

    let id = utils::next_round_id(&env);
    let round = FederatedRound {
        id,
        base_model_id,
        min_participants,
        dp_epsilon,
        started_at: env.ledger().timestamp(),
        finalized_at: 0,
        total_updates: 0,
        is_finalized: false,
    };

    // Validate serialization before storing
    round
        .safe_serialize(&env)
        .map_err(|_| Error::SerializationError)?;

    env.storage().instance().set(&DataKey::Round(id), &round);
    env.events().publish((symbol_short!("RndStart"),), id);
    Ok(id)
}

pub fn submit_update(
    env: Env,
    participant: Address,
    round_id: u64,
    update_hash: BytesN<32>,
    num_samples: u32,
) -> Result<bool, Error> {
    participant.require_auth();

    let mut round: FederatedRound = env
        .storage()
        .instance()
        .get(&DataKey::Round(round_id))
        .ok_or(Error::RoundNotFound)?;

    if round.is_finalized {
        return Err(Error::RoundFinalized);
    }

    let key = DataKey::ParticipantUpdate(round_id, participant.clone());
    if env.storage().instance().has(&key) {
        return Err(Error::DuplicateUpdate);
    }

    let update = ParticipantUpdateMeta {
        round_id,
        participant: participant.clone(),
        update_hash,
        num_samples,
    };

    // Validate serialization before storing
    update
        .safe_serialize(&env)
        .map_err(|_| Error::SerializationError)?;

    env.storage().instance().set(&key, &update);

    round.total_updates = round.total_updates.saturating_add(1);
    env.storage()
        .instance()
        .set(&DataKey::Round(round_id), &round);

    env.events()
        .publish((symbol_short!("UpdSubmit"),), (round_id, participant));

    Ok(true)
}

pub fn finalize_round(
    env: Env,
    caller: Address,
    round_id: u64,
    new_model_id: BytesN<32>,
    description: String,
    metrics_ref: String,
    fairness_report_ref: String,
) -> Result<bool, Error> {
    caller.require_auth();
    utils::ensure_admin(&env, &caller)?;

    let mut round: FederatedRound = env
        .storage()
        .instance()
        .get(&DataKey::Round(round_id))
        .ok_or(Error::RoundNotFound)?;

    if round.is_finalized {
        return Err(Error::RoundFinalized);
    }

    if round.total_updates < round.min_participants {
        return Err(Error::NotEnoughParticipants);
    }

    round.is_finalized = true;
    round.finalized_at = env.ledger().timestamp();
    env.storage()
        .instance()
        .set(&DataKey::Round(round_id), &round);

    let metadata = ModelMetadata {
        model_id: new_model_id.clone(),
        round_id,
        description,
        metrics_ref,
        fairness_report_ref,
        created_at: round.finalized_at,
    };

    // Validate serialization before storing
    metadata
        .safe_serialize(&env)
        .map_err(|_| Error::SerializationError)?;

    env.storage()
        .instance()
        .set(&DataKey::Model(new_model_id.clone()), &metadata);

    env.events()
        .publish((symbol_short!("RndFinal"),), (round_id, new_model_id));

    Ok(true)
}

pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
    env.storage().instance().get(&DataKey::Round(round_id))
}

pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
    env.storage().instance().get(&DataKey::Model(model_id))
}
