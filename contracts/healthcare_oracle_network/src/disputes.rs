use soroban_sdk::{symbol_short, Address, Env, String};

use crate::types::{
    Config, ConsensusRecord, DataKey, Dispute, DisputeStatus, Error, FeedKey, FeedKind,
};
use crate::utils;

pub fn raise_dispute(
    env: Env,
    challenger: Address,
    kind: FeedKind,
    feed_id: String,
    reason: String,
) -> Result<u64, Error> {
    challenger.require_auth();
    utils::require_initialized(&env)?;

    if reason.len() == 0 || feed_id.len() == 0 {
        return Err(Error::InvalidData);
    }

    let key = FeedKey { kind, feed_id };
    let consensus: ConsensusRecord = env
        .storage()
        .persistent()
        .get(&DataKey::Consensus(key.clone()))
        .ok_or(Error::ConsensusNotFound)?;

    if consensus.disputed {
        return Err(Error::InvalidDisputeState);
    }

    let dispute_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::DisputeCount)
        .unwrap_or(0u64)
        .saturating_add(1);

    let dispute = Dispute {
        id: dispute_id,
        key,
        round_id: consensus.round_id,
        challenger,
        reason,
        status: DisputeStatus::Open,
        opened_at: env.ledger().timestamp(),
        resolved_at: None,
        resolver: None,
        ruling: None,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Dispute(dispute_id), &dispute);
    env.storage()
        .instance()
        .set(&DataKey::DisputeCount, &dispute_id);

    env.events()
        .publish((symbol_short!("dispute"),), dispute_id);
    Ok(dispute_id)
}

pub fn resolve_dispute(
    env: Env,
    resolver: Address,
    dispute_id: u64,
    valid_dispute: bool,
    ruling: String,
    penalized_oracle: Option<Address>,
) -> Result<(), Error> {
    resolver.require_auth();
    let config: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)?;

    if resolver != config.admin && !config.arbiters.contains(&resolver) {
        return Err(Error::Unauthorized);
    }

    let mut dispute: Dispute = env
        .storage()
        .persistent()
        .get(&DataKey::Dispute(dispute_id))
        .ok_or(Error::DisputeNotFound)?;

    if dispute.status != DisputeStatus::Open {
        return Err(Error::DisputeAlreadyResolved);
    }

    let mut consensus: ConsensusRecord = env
        .storage()
        .persistent()
        .get(&DataKey::Consensus(dispute.key.clone()))
        .ok_or(Error::ConsensusNotFound)?;

    if valid_dispute {
        consensus.disputed = true;
        dispute.status = DisputeStatus::ResolvedValid;

        if let Some(oracle) = penalized_oracle {
            utils::adjust_reputation(&env, oracle, -15, true)?;
        }
    } else {
        dispute.status = DisputeStatus::ResolvedInvalid;
        let mut i = 0;
        while i < consensus.submitters.len() {
            let submitter = consensus.submitters.get(i).unwrap();
            utils::adjust_reputation(&env, submitter, 2, false)?;
            i += 1;
        }
    }

    dispute.resolved_at = Some(env.ledger().timestamp());
    dispute.resolver = Some(resolver);
    dispute.ruling = Some(ruling);

    env.storage()
        .persistent()
        .set(&DataKey::Consensus(dispute.key.clone()), &consensus);
    env.storage()
        .persistent()
        .set(&DataKey::Dispute(dispute_id), &dispute);

    env.events()
        .publish((symbol_short!("resolve"), dispute_id), valid_dispute);

    Ok(())
}

pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
    env.storage()
        .persistent()
        .get(&DataKey::Dispute(dispute_id))
}
