#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Map, String, Vec, Bytes,
};

#[derive(Clone)]
#[contracttype]
pub struct KPI {
    pub name: String,
    pub value: u64,
    pub count: u64,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Report {
    pub report_id: String,
    pub template_id: String,
    pub generated_at: u64,
    pub content_ref: String, // IPFS or external storage reference
}

#[contracttype]
pub enum DataKey {
    Admin,
    Ingester(Address),
    MetricSum(String),
    MetricCount(String),
    MetricLastUpdated(String),
    Templates,
    Schedules,
    Reports,
    AiAnalyticsAddr,
    MinKAnon,
}

#[contract]
pub struct HealthcareAnalyticsContract;

#[contractimpl]
impl HealthcareAnalyticsContract {
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return false;
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        // set default min k-anonymity
        env.storage()
            .instance()
            .set(&DataKey::MinKAnon, &5u32);
        true
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<(), ()> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(())?;
        if admin != *caller {
            return Err(());
        }
        Ok(())
    }

    pub fn register_ingester(env: Env, admin: Address, ingester: Address) -> Result<bool, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Ingester(ingester.clone()), &true);
        Ok(true)
    }

    pub fn unregister_ingester(env: Env, admin: Address, ingester: Address) -> Result<bool, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage().instance().remove(&DataKey::Ingester(ingester.clone()));
        Ok(true)
    }

    pub fn set_min_k_anon(env: Env, admin: Address, k: u32) -> Result<bool, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::MinKAnon, &k);
        Ok(true)
    }

    pub fn link_ai_analytics(env: Env, admin: Address, addr: Address) -> Result<bool, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::AiAnalyticsAddr, &addr);
        Ok(true)
    }

    // Submit an aggregated metric. No raw patient identifiers are accepted.
    // Only registered ingesters or admin can submit.
    pub fn submit_metric(
        env: Env,
        caller: Address,
        metric_name: String,
        value: u64,
    ) -> Result<bool, ()> {
        caller.require_auth();
        let is_admin = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .map(|a| a == caller)
            .unwrap_or(false);

        let is_ingester = env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Ingester(caller.clone()))
            .unwrap_or(false);

        if !is_admin && !is_ingester {
            return Err(());
        }

        // update sum
        let sum_key = DataKey::MetricSum(metric_name.clone());
        let count_key = DataKey::MetricCount(metric_name.clone());
        let last_key = DataKey::MetricLastUpdated(metric_name.clone());

        let mut sum: u64 = env.storage().instance().get(&sum_key).unwrap_or(0u64);
        let mut count: u64 = env.storage().instance().get(&count_key).unwrap_or(0u64);

        sum = sum.saturating_add(value);
        count = count.saturating_add(1u64);

        env.storage().instance().set(&sum_key, &sum);
        env.storage().instance().set(&count_key, &count);
        env.storage()
            .instance()
            .set(&last_key, &env.ledger().timestamp());

        // emit lightweight event for off-chain subscribers
        env.events().publish((symbol_short!("ANLT"),), (metric_name, sum, count));

        Ok(true)
    }

    pub fn get_metric(env: Env, caller: Address, metric_name: String) -> Result<Option<KPI>, ()> {
        caller.require_auth();

        let sum_key = DataKey::MetricSum(metric_name.clone());
        let count_key = DataKey::MetricCount(metric_name.clone());
        let last_key = DataKey::MetricLastUpdated(metric_name.clone());

        let sum: u64 = env.storage().instance().get(&sum_key).unwrap_or(0u64);
        let count: u64 = env.storage().instance().get(&count_key).unwrap_or(0u64);
        let last: u64 = env.storage().instance().get(&last_key).unwrap_or(0u64);

        let min_k: u32 = env.storage().instance().get(&DataKey::MinKAnon).unwrap_or(5u32);

        // privacy-preserving: enforce k-anonymity threshold - if count < k, do not return raw metric
        if count < (min_k as u64) {
            return Ok(None);
        }

        let avg = if count == 0 { 0 } else { sum / count };

        let kpi = KPI {
            name: metric_name,
            value: avg,
            count,
            last_updated: last,
        };
        Ok(Some(kpi))
    }

    pub fn create_template(env: Env, admin: Address, template_id: String, template: String) -> Result<bool, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        let mut templates: Map<String, String> = env
            .storage()
            .instance()
            .get(&DataKey::Templates)
            .unwrap_or(Map::new(&env));
        templates.set(template_id.clone(), template.clone());
        env.storage().instance().set(&DataKey::Templates, &templates);
        Ok(true)
    }

    pub fn schedule_report(
        env: Env,
        admin: Address,
        schedule_id: String,
        template_id: String,
        interval_secs: u64,
        next_run: u64,
    ) -> Result<bool, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        let mut schedules: Map<String, (String, u64, u64)> = env
            .storage()
            .instance()
            .get(&DataKey::Schedules)
            .unwrap_or(Map::new(&env));
        schedules.set(schedule_id.clone(), (template_id, interval_secs, next_run));
        env.storage().instance().set(&DataKey::Schedules, &schedules);
        Ok(true)
    }

    // Run scheduled reports that are due. Admin-only to avoid automated loops.
    pub fn run_scheduled_reports(env: Env, admin: Address) -> Result<Vec<String>, ()> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;

        let mut schedules: Map<String, (String, u64, u64)> = env
            .storage()
            .instance()
            .get(&DataKey::Schedules)
            .unwrap_or(Map::new(&env));

        let mut reports: Map<String, Report> = env
            .storage()
            .instance()
            .get(&DataKey::Reports)
            .unwrap_or(Map::new(&env));

        let mut out_ids: Vec<String> = Vec::new(&env);
        let now = env.ledger().timestamp();

        for (schedule_id, tuple) in schedules.iter() {
            let (template_id, interval, next_run) = tuple;
            if now >= next_run {
                // Build a trivial report using template reference and current KPI snapshot
                let report_id = String::from_str(&env, &format!("rep-{}-{}", schedule_id, now));
                let content_ref = String::from_str(
                    &env,
                    &format!(
                        "generated://{}/{}",
                        template_id.clone().to_string(),
                        report_id.clone().to_string()
                    ),
                );

                let report = Report {
                    report_id: report_id.clone(),
                    template_id: template_id.clone(),
                    generated_at: now,
                    content_ref: content_ref.clone(),
                };

                reports.set(report_id.clone(), report);
                out_ids.push_back(report_id.clone());

                // update schedule next_run
                let new_next = next_run.saturating_add(interval);
                schedules.set(schedule_id.clone(), (template_id.clone(), interval, new_next));

                // emit event
                env.events().publish((symbol_short!("RPT"),), (report_id.clone(), content_ref));
            }
        }

        env.storage().instance().set(&DataKey::Schedules, &schedules);
        env.storage().instance().set(&DataKey::Reports, &reports);

        Ok(out_ids)
    }

    pub fn get_report(env: Env, caller: Address, report_id: String) -> Result<Option<Report>, ()> {
        caller.require_auth();
        let reports: Map<String, Report> = env
            .storage()
            .instance()
            .get(&DataKey::Reports)
            .unwrap_or(Map::new(&env));
        Ok(reports.get(report_id))
    }

    pub fn export_report_csv(env: Env, caller: Address, report_id: String) -> Result<Option<Bytes>, ()> {
        caller.require_auth();
        // For privacy, only return CSV metadata (not raw patient data)
        if let Some(r) = Self::get_report(env.clone(), caller.clone(), report_id.clone())? {
            let csv = String::from_str(
                &env,
                &format!("report_id,template_id,generated_at,content_ref\n{},{},{},{}",
                    r.report_id.to_string(), r.template_id.to_string(), r.generated_at, r.content_ref.to_string()
                )
            );
            Ok(Some(csv.into()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_basic_flow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HealthcareAnalyticsContract);
        let client = HealthcareAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.mock_all_auths().initialize(&admin);

        let ingester = Address::generate(&env);
        assert!(client.mock_all_auths().register_ingester(&admin, &ingester));

        // submit metrics
        assert!(client.mock_all_auths().submit_metric(&ingester, &String::from_str(&env, "records_created"), &10u64));
        assert!(client.mock_all_auths().submit_metric(&ingester, &String::from_str(&env, "records_created"), &5u64));

        // get metric: count=2 so below default k=5 => should be None
        let none_kpi = client.mock_all_auths().get_metric(&admin, &String::from_str(&env, "records_created"));
        assert!(none_kpi.is_none());

        // increase to reach k
        for _ in 0..3 {
            assert!(client.mock_all_auths().submit_metric(&ingester, &String::from_str(&env, "records_created"), &1u64));
        }
        let kpi = client.mock_all_auths().get_metric(&admin, &String::from_str(&env, "records_created")).unwrap();
        assert_eq!(kpi.count, 5u64);

        // create template and schedule
        assert!(client.mock_all_auths().create_template(&admin, &String::from_str(&env, "t1"), &String::from_str(&env, "Monthly Summary")));
        let now = env.ledger().timestamp();
        assert!(client.mock_all_auths().schedule_report(&admin, &String::from_str(&env, "s1"), &String::from_str(&env, "t1"), &10u64, &now));

        let ids = client.mock_all_auths().run_scheduled_reports(&admin);
        assert_eq!(ids.len(), 1);
        let rep = client.mock_all_auths().get_report(&admin, &ids.get(0).unwrap());
        assert!(rep.is_some());
    }
}
