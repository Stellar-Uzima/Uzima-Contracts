#![no_std]

use soroban_sdk::{symbol_short, Address, Env, Symbol};

use super::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DashboardCapability {
    RecordMetric = 1,
    RecordSnapshot = 2,
    CreateTemplate = 3,
    ScheduleReport = 4,
    RunReport = 5,
    ManageDataLake = 6,
    SyncAiRound = 7,
    ViewDashboard = 8,
    ManageCollectors = 9,
    ConfigurePrivacy = 10,
}

impl DashboardCapability {
    pub fn to_symbol(self) -> Symbol {
        match self {
            Self::RecordMetric => symbol_short!("rec_met"),
            Self::RecordSnapshot => symbol_short!("rec_snap"),
            Self::CreateTemplate => symbol_short!("crt_tpl"),
            Self::ScheduleReport => symbol_short!("sch_rpt"),
            Self::RunReport => symbol_short!("run_rpt"),
            Self::ManageDataLake => symbol_short!("mng_lake"),
            Self::SyncAiRound => symbol_short!("syn_ai"),
            Self::ViewDashboard => symbol_short!("view_dash"),
            Self::ManageCollectors => symbol_short!("mng_col"),
            Self::ConfigurePrivacy => symbol_short!("cfg_priv"),
        }
    }
}

#[derive(Clone)]
#[contracttype]
pub struct CapabilityGrant {
    pub granted_at: u64,
    pub expires_at: u64,
    pub granted_by: Address,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum CapabilityDataKey {
    Capability(Address, u32),
}

pub fn grant_capability(
    env: &Env,
    admin: &Address,
    target: &Address,
    capability: DashboardCapability,
    ttl_ledgers: u32,
) -> Result<(), Error> {
    let now = env.ledger().sequence();
    let expires = now.saturating_add(ttl_ledgers);

    let grant = CapabilityGrant {
        granted_at: now,
        expires_at: expires,
        granted_by: admin.clone(),
    };

    env.storage().persistent().set(
        &CapabilityDataKey::Capability(target.clone(), capability as u32),
        &grant,
    );

    env.events().publish(
        (symbol_short!("cap_grant"),),
        (target.clone(), capability.to_symbol(), expires),
    );

    Ok(())
}

pub fn revoke_capability(
    env: &Env,
    _admin: &Address,
    target: &Address,
    capability_id: u32,
) -> Result<(), Error> {
    env.storage().persistent().remove(&CapabilityDataKey::Capability(
        target.clone(),
        capability_id,
    ));

    env.events().publish(
        (symbol_short!("cap_revoke"),),
        (target.clone(), capability_id),
    );

    Ok(())
}

pub fn has_capability(env: &Env, address: &Address, capability_id: u32) -> bool {
    let key = CapabilityDataKey::Capability(address.clone(), capability_id);
    let grant: Option<CapabilityGrant> = env.storage().persistent().get(&key);

    match grant {
        None => false,
        Some(g) => {
            let current = env.ledger().sequence();
            current <= g.expires_at
        }
    }
}

pub fn require_capability(
    env: &Env,
    address: &Address,
    capability_id: u32,
) -> Result<(), Error> {
    if !has_capability(env, address, capability_id) {
        return Err(Error::NotAuthorized);
    }
    Ok(())
}

pub fn get_capability_grant(
    env: &Env,
    address: &Address,
    capability_id: u32,
) -> Option<CapabilityGrant> {
    let key = CapabilityDataKey::Capability(address.clone(), capability_id);
    env.storage().persistent().get(&key)
}
