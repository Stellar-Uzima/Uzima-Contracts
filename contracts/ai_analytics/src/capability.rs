#![no_std]

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use crate::types::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AnalyticsCapability {
    StartRound = 1,
    SubmitUpdate = 2,
    FinalizeRound = 3,
    ReadRound = 4,
    ReadModel = 5,
    ManageParticipants = 6,
    ExportResults = 7,
}

impl AnalyticsCapability {
    pub fn to_symbol(self) -> Symbol {
        match self {
            Self::StartRound => symbol_short!("start_rnd"),
            Self::SubmitUpdate => symbol_short!("submit_up"),
            Self::FinalizeRound => symbol_short!("final_rnd"),
            Self::ReadRound => symbol_short!("read_rnd"),
            Self::ReadModel => symbol_short!("read_mod"),
            Self::ManageParticipants => symbol_short!("mng_part"),
            Self::ExportResults => symbol_short!("exp_res"),
        }
    }
}

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
    DefaultCapability,
}

pub fn grant_analytics_capability(
    env: &Env,
    admin: &Address,
    target: &Address,
    capability: AnalyticsCapability,
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

pub fn grant_dashboard_capability(
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

pub fn check_admin_or_capability(
    env: &Env,
    caller: &Address,
    admin: &Address,
    capability_id: u32,
) -> Result<(), Error> {
    if *caller == *admin {
        return Ok(());
    }
    require_capability(env, caller, capability_id)
}

#[cfg(all(test, feature = "testutils"))]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_grant_and_check_capability() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        grant_analytics_capability(&env, &admin, &user, AnalyticsCapability::ReadRound, 100)
            .unwrap();

        assert!(has_capability(&env, &user, AnalyticsCapability::ReadRound as u32));
        assert!(!has_capability(&env, &user, AnalyticsCapability::StartRound as u32));
    }

    #[test]
    fn test_revoke_capability() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        grant_dashboard_capability(
            &env,
            &admin,
            &user,
            DashboardCapability::RecordMetric,
            100,
        )
        .unwrap();
        assert!(has_capability(&env, &user, DashboardCapability::RecordMetric as u32));

        revoke_capability(
            &env,
            &admin,
            &user,
            DashboardCapability::RecordMetric as u32,
        )
        .unwrap();
        assert!(!has_capability(&env, &user, DashboardCapability::RecordMetric as u32));
    }

    #[test]
    fn test_require_capability_fails_without_grant() {
        let env = Env::default();
        let user = Address::generate(&env);

        let result = require_capability(&env, &user, AnalyticsCapability::StartRound as u32);
        assert!(result.is_err());
    }
}
