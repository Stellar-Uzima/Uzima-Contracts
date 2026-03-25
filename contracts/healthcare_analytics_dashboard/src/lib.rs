#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, symbol_short, Address,
    BytesN, Env, String, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct DashboardConfig {
    pub admin: Address,
    pub min_cohort_size: u32,
    pub noise_bps: u32,
    pub realtime_enabled: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct MetricAggregate {
    pub metric_name: String,
    pub period_id: u64,
    pub total_value_bps: u64,
    pub count: u32,
    pub min_value_bps: u32,
    pub max_value_bps: u32,
    pub avg_value_bps: u32,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DashboardSnapshot {
    pub active_users: u32,
    pub tx_count: u32,
    pub error_count: u32,
    pub latency_p95_ms: u32,
    pub uptime_bps: u32,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PerformanceKpi {
    pub total_snapshots: u32,
    pub avg_latency_p95_ms: u32,
    pub avg_uptime_bps: u32,
    pub avg_error_rate_bps: u32,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ReportTemplate {
    pub id: u64,
    pub name: String,
    pub metric_filters: Vec<String>,
    pub include_compliance: bool,
    pub include_performance: bool,
    pub output_format: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ReportSchedule {
    pub id: u64,
    pub template_id: u64,
    pub cadence_seconds: u64,
    pub next_run_at: u64,
    pub last_run_at: u64,
    pub enabled: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ComplianceSummary {
    pub period_id: u64,
    pub total_checks: u32,
    pub passed_checks: u32,
    pub total_violations: u32,
    pub total_audit_events: u32,
    pub severity_bps: u32,
    pub generated_at: u64,
    pub latest_report_ref: String,
}

#[derive(Clone)]
#[contracttype]
pub struct VisualizationPoint {
    pub period_id: u64,
    pub avg_value_bps: u32,
    pub sample_count: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ExportRecord {
    pub export_id: u64,
    pub template_id: u64,
    pub output_format: String,
    pub data_ref: String,
    pub checksum: BytesN<32>,
    pub generated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AiRoundInsight {
    pub round_id: u64,
    pub min_participants: u32,
    pub total_updates: u32,
    pub dp_epsilon: u32,
    pub is_finalized: bool,
    pub started_at: u64,
    pub finalized_at: u64,
    pub participation_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct AiFederatedRound {
    pub id: u64,
    pub base_model_id: BytesN<32>,
    pub min_participants: u32,
    pub dp_epsilon: u32,
    pub started_at: u64,
    pub finalized_at: u64,
    pub total_updates: u32,
    pub is_finalized: bool,
}

#[contractclient(name = "AiAnalyticsClient")]
pub trait AiAnalyticsTrait {
    fn get_round(env: Env, round_id: u64) -> Option<AiFederatedRound>;
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Collector(Address),
    Metric(String, u64),
    MetricPeriods(String),
    LatestSnapshot,
    PerformanceKpi,
    TemplateCounter,
    Template(u64),
    ScheduleCounter,
    Schedule(u64),
    Compliance(u64),
    ExportCounter,
    Export(u64),
    AiContract,
    AiInsight(u64),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    InvalidInput = 4,
    PrivacyThresholdNotMet = 5,
    MetricNotFound = 6,
    TemplateNotFound = 7,
    ScheduleNotFound = 8,
    ComplianceNotFound = 9,
    AiAnalyticsNotConfigured = 10,
    AiRoundNotFound = 11,
}

#[contract]
pub struct HealthcareAnalyticsDashboardContract;

#[contractimpl]
impl HealthcareAnalyticsDashboardContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        min_cohort_size: u32,
        noise_bps: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }
        if min_cohort_size == 0 || noise_bps > 10_000 {
            return Err(Error::InvalidInput);
        }

        let config = DashboardConfig {
            admin,
            min_cohort_size,
            noise_bps,
            realtime_enabled: true,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::TemplateCounter, &0u64);
        env.storage().instance().set(&DataKey::ScheduleCounter, &0u64);
        env.storage().instance().set(&DataKey::ExportCounter, &0u64);

        env.events().publish((symbol_short!("DashInit"),), true);
        Ok(true)
    }

    fn load_config(env: &Env) -> Result<DashboardConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<DashboardConfig, Error> {
        let config = Self::load_config(env)?;
        if config.admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(config)
    }

    fn ensure_collector_or_admin(env: &Env, caller: &Address) -> Result<DashboardConfig, Error> {
        let config = Self::load_config(env)?;
        if config.admin == *caller {
            return Ok(config);
        }
        let authorized: bool = env
            .storage()
            .instance()
            .get(&DataKey::Collector(caller.clone()))
            .unwrap_or(false);
        if !authorized {
            return Err(Error::NotAuthorized);
        }
        Ok(config)
    }

    fn next_counter(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }

    fn add_noise(value_bps: u32, noise_bps: u32, period_id: u64, sample_index: u32) -> u32 {
        if noise_bps == 0 {
            return value_bps;
        }

        let max_noise = (value_bps / 100).saturating_mul(noise_bps / 100);
        if max_noise == 0 {
            return value_bps;
        }

        let seed = (period_id as u32)
            .wrapping_mul(31)
            .wrapping_add(sample_index.wrapping_mul(17));
        let noise = seed % (max_noise.saturating_add(1));
        if seed % 2 == 0 {
            value_bps.saturating_add(noise).min(10_000)
        } else {
            value_bps.saturating_sub(noise)
        }
    }

    fn track_metric_period(env: &Env, metric_name: &String, period_id: u64) {
        let mut periods: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::MetricPeriods(metric_name.clone()))
            .unwrap_or(Vec::new(env));

        let mut exists = false;
        for existing in periods.iter() {
            if existing == period_id {
                exists = true;
                break;
            }
        }

        if !exists {
            periods.push_back(period_id);
            env.storage()
                .instance()
                .set(&DataKey::MetricPeriods(metric_name.clone()), &periods);
        }
    }

    pub fn set_collector(
        env: Env,
        caller: Address,
        collector: Address,
        enabled: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        env.storage()
            .instance()
            .set(&DataKey::Collector(collector.clone()), &enabled);
        env.events()
            .publish((symbol_short!("Collector"),), (collector, enabled));
        Ok(true)
    }

    pub fn configure_ai_analytics(
        env: Env,
        caller: Address,
        ai_analytics_contract: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        env.storage()
            .instance()
            .set(&DataKey::AiContract, &ai_analytics_contract.clone());
        env.events()
            .publish((symbol_short!("AiConfig"),), ai_analytics_contract);
        Ok(true)
    }

    pub fn record_medical_metric(
        env: Env,
        caller: Address,
        metric_name: String,
        period_id: u64,
        metric_value_bps: u32,
        cohort_size: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let config = Self::ensure_collector_or_admin(&env, &caller)?;

        if metric_name.is_empty() || metric_value_bps > 10_000 {
            return Err(Error::InvalidInput);
        }
        if cohort_size < config.min_cohort_size {
            return Err(Error::PrivacyThresholdNotMet);
        }

        let key = DataKey::Metric(metric_name.clone(), period_id);
        let existing: Option<MetricAggregate> = env.storage().instance().get(&key);
        let sample_index = existing.as_ref().map(|aggregate| aggregate.count).unwrap_or(0);
        let noisy_value = Self::add_noise(metric_value_bps, config.noise_bps, period_id, sample_index);
        let timestamp = env.ledger().timestamp();

        let next = match existing {
            Some(mut aggregate) => {
                aggregate.count = aggregate.count.saturating_add(1);
                aggregate.total_value_bps = aggregate
                    .total_value_bps
                    .saturating_add(u64::from(noisy_value));
                aggregate.min_value_bps = aggregate.min_value_bps.min(noisy_value);
                aggregate.max_value_bps = aggregate.max_value_bps.max(noisy_value);
                aggregate.avg_value_bps =
                    (aggregate.total_value_bps / u64::from(aggregate.count)) as u32;
                aggregate.last_updated = timestamp;
                aggregate
            }
            None => MetricAggregate {
                metric_name: metric_name.clone(),
                period_id,
                total_value_bps: u64::from(noisy_value),
                count: 1,
                min_value_bps: noisy_value,
                max_value_bps: noisy_value,
                avg_value_bps: noisy_value,
                last_updated: timestamp,
            },
        };

        env.storage().instance().set(&key, &next);
        Self::track_metric_period(&env, &metric_name, period_id);

        env.events().publish(
            (symbol_short!("PrivAgg"),),
            (metric_name, period_id, next.avg_value_bps, next.count),
        );
        Ok(true)
    }

    pub fn record_system_snapshot(
        env: Env,
        caller: Address,
        active_users: u32,
        tx_count: u32,
        error_count: u32,
        latency_p95_ms: u32,
        uptime_bps: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let config = Self::ensure_collector_or_admin(&env, &caller)?;
        if !config.realtime_enabled || uptime_bps > 10_000 {
            return Err(Error::InvalidInput);
        }

        let timestamp = env.ledger().timestamp();
        let snapshot = DashboardSnapshot {
            active_users,
            tx_count,
            error_count,
            latency_p95_ms,
            uptime_bps,
            timestamp,
        };
        env.storage().instance().set(&DataKey::LatestSnapshot, &snapshot);

        let mut kpi: PerformanceKpi = env
            .storage()
            .instance()
            .get(&DataKey::PerformanceKpi)
            .unwrap_or(PerformanceKpi {
                total_snapshots: 0,
                avg_latency_p95_ms: 0,
                avg_uptime_bps: 0,
                avg_error_rate_bps: 0,
                last_updated: timestamp,
            });

        let new_count = kpi.total_snapshots.saturating_add(1);
        let error_rate_bps = if tx_count == 0 {
            0
        } else {
            ((u64::from(error_count) * 10_000) / u64::from(tx_count)) as u32
        };

        kpi.avg_latency_p95_ms = ((u64::from(kpi.avg_latency_p95_ms) * u64::from(kpi.total_snapshots)
            + u64::from(latency_p95_ms))
            / u64::from(new_count)) as u32;
        kpi.avg_uptime_bps = ((u64::from(kpi.avg_uptime_bps) * u64::from(kpi.total_snapshots)
            + u64::from(uptime_bps))
            / u64::from(new_count)) as u32;
        kpi.avg_error_rate_bps =
            ((u64::from(kpi.avg_error_rate_bps) * u64::from(kpi.total_snapshots)
                + u64::from(error_rate_bps))
                / u64::from(new_count)) as u32;
        kpi.total_snapshots = new_count;
        kpi.last_updated = timestamp;

        env.storage().instance().set(&DataKey::PerformanceKpi, &kpi);
        env.events().publish(
            (symbol_short!("DashSnap"),),
            (active_users, tx_count, error_count, latency_p95_ms),
        );
        Ok(true)
    }

    pub fn create_report_template(
        env: Env,
        caller: Address,
        name: String,
        metric_filters: Vec<String>,
        include_compliance: bool,
        include_performance: bool,
        output_format: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;
        if name.is_empty() || output_format.is_empty() {
            return Err(Error::InvalidInput);
        }

        let id = Self::next_counter(&env, &DataKey::TemplateCounter);
        let template = ReportTemplate {
            id,
            name,
            metric_filters,
            include_compliance,
            include_performance,
            output_format,
            created_at: env.ledger().timestamp(),
        };

        env.storage().instance().set(&DataKey::Template(id), &template);
        env.events().publish((symbol_short!("TplCreate"),), id);
        Ok(id)
    }

    pub fn schedule_report(
        env: Env,
        caller: Address,
        template_id: u64,
        cadence_seconds: u64,
        next_run_at: u64,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;
        if cadence_seconds == 0 {
            return Err(Error::InvalidInput);
        }
        let _template: ReportTemplate = env
            .storage()
            .instance()
            .get(&DataKey::Template(template_id))
            .ok_or(Error::TemplateNotFound)?;

        let id = Self::next_counter(&env, &DataKey::ScheduleCounter);
        let schedule = ReportSchedule {
            id,
            template_id,
            cadence_seconds,
            next_run_at,
            last_run_at: 0,
            enabled: true,
        };
        env.storage().instance().set(&DataKey::Schedule(id), &schedule);
        env.events()
            .publish((symbol_short!("RptSched"),), (id, template_id));
        Ok(id)
    }

    pub fn run_scheduled_report(
        env: Env,
        caller: Address,
        schedule_id: u64,
        data_ref: String,
        checksum: BytesN<32>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller)?;

        let mut schedule: ReportSchedule = env
            .storage()
            .instance()
            .get(&DataKey::Schedule(schedule_id))
            .ok_or(Error::ScheduleNotFound)?;
        if !schedule.enabled || data_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let template: ReportTemplate = env
            .storage()
            .instance()
            .get(&DataKey::Template(schedule.template_id))
            .ok_or(Error::TemplateNotFound)?;

        let export_id = Self::next_counter(&env, &DataKey::ExportCounter);
        let now = env.ledger().timestamp();
        let export = ExportRecord {
            export_id,
            template_id: template.id,
            output_format: template.output_format,
            data_ref,
            checksum,
            generated_at: now,
        };

        schedule.last_run_at = now;
        schedule.next_run_at = now.saturating_add(schedule.cadence_seconds);

        env.storage().instance().set(&DataKey::Export(export_id), &export);
        env.storage()
            .instance()
            .set(&DataKey::Schedule(schedule_id), &schedule);
        env.events()
            .publish((symbol_short!("RptRun"),), (schedule_id, export_id));
        Ok(export_id)
    }

    pub fn upsert_compliance_summary(
        env: Env,
        caller: Address,
        period_id: u64,
        passed: bool,
        violation_count: u32,
        audit_event_count: u32,
        severity_bps: u32,
        report_ref: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_collector_or_admin(&env, &caller)?;
        if severity_bps > 10_000 || report_ref.is_empty() {
            return Err(Error::InvalidInput);
        }

        let mut summary: ComplianceSummary = env
            .storage()
            .instance()
            .get(&DataKey::Compliance(period_id))
            .unwrap_or(ComplianceSummary {
                period_id,
                total_checks: 0,
                passed_checks: 0,
                total_violations: 0,
                total_audit_events: 0,
                severity_bps: 0,
                generated_at: 0,
                latest_report_ref: report_ref.clone(),
            });

        summary.total_checks = summary.total_checks.saturating_add(1);
        if passed {
            summary.passed_checks = summary.passed_checks.saturating_add(1);
        }
        summary.total_violations = summary.total_violations.saturating_add(violation_count);
        summary.total_audit_events = summary.total_audit_events.saturating_add(audit_event_count);
        summary.severity_bps = severity_bps;
        summary.generated_at = env.ledger().timestamp();
        summary.latest_report_ref = report_ref;

        env.storage()
            .instance()
            .set(&DataKey::Compliance(period_id), &summary);
        env.events().publish(
            (symbol_short!("CompAuto"),),
            (period_id, summary.total_checks, summary.total_violations),
        );
        Ok(true)
    }

    pub fn sync_ai_round(env: Env, caller: Address, round_id: u64) -> Result<AiRoundInsight, Error> {
        caller.require_auth();
        Self::ensure_collector_or_admin(&env, &caller)?;

        let ai_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AiContract)
            .ok_or(Error::AiAnalyticsNotConfigured)?;

        let client = AiAnalyticsClient::new(&env, &ai_addr);
        let round = client.get_round(&round_id).ok_or(Error::AiRoundNotFound)?;

        let participation_bps = if round.min_participants == 0 {
            0
        } else {
            ((u64::from(round.total_updates) * 10_000) / u64::from(round.min_participants)).min(10_000)
                as u32
        };

        let insight = AiRoundInsight {
            round_id: round.id,
            min_participants: round.min_participants,
            total_updates: round.total_updates,
            dp_epsilon: round.dp_epsilon,
            is_finalized: round.is_finalized,
            started_at: round.started_at,
            finalized_at: round.finalized_at,
            participation_bps,
        };

        env.storage()
            .instance()
            .set(&DataKey::AiInsight(round_id), &insight);
        env.events().publish(
            (symbol_short!("AiSync"),),
            (round_id, insight.total_updates, insight.participation_bps),
        );

        Ok(insight)
    }

    pub fn get_config(env: Env) -> Result<DashboardConfig, Error> {
        Self::load_config(&env)
    }

    pub fn get_metric_aggregate(
        env: Env,
        metric_name: String,
        period_id: u64,
    ) -> Result<MetricAggregate, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Metric(metric_name, period_id))
            .ok_or(Error::MetricNotFound)
    }

    pub fn get_latest_snapshot(env: Env) -> Option<DashboardSnapshot> {
        env.storage().instance().get(&DataKey::LatestSnapshot)
    }

    pub fn get_performance_kpi(env: Env) -> Option<PerformanceKpi> {
        env.storage().instance().get(&DataKey::PerformanceKpi)
    }

    pub fn get_report_template(env: Env, template_id: u64) -> Option<ReportTemplate> {
        env.storage().instance().get(&DataKey::Template(template_id))
    }

    pub fn get_report_schedule(env: Env, schedule_id: u64) -> Option<ReportSchedule> {
        env.storage().instance().get(&DataKey::Schedule(schedule_id))
    }

    pub fn get_compliance_summary(env: Env, period_id: u64) -> Option<ComplianceSummary> {
        env.storage().instance().get(&DataKey::Compliance(period_id))
    }

    pub fn get_export_record(env: Env, export_id: u64) -> Option<ExportRecord> {
        env.storage().instance().get(&DataKey::Export(export_id))
    }

    pub fn get_ai_round_insight(env: Env, round_id: u64) -> Option<AiRoundInsight> {
        env.storage().instance().get(&DataKey::AiInsight(round_id))
    }

    pub fn get_visualization_series(
        env: Env,
        metric_name: String,
        max_points: u32,
    ) -> Vec<VisualizationPoint> {
        let periods: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::MetricPeriods(metric_name.clone()))
            .unwrap_or(Vec::new(&env));
        let mut points = Vec::new(&env);
        let mut emitted = 0u32;

        for period_id in periods.iter() {
            if emitted >= max_points {
                break;
            }
            if let Some(aggregate) = env
                .storage()
                .instance()
                .get::<DataKey, MetricAggregate>(&DataKey::Metric(metric_name.clone(), period_id))
            {
                points.push_back(VisualizationPoint {
                    period_id,
                    avg_value_bps: aggregate.avg_value_bps,
                    sample_count: aggregate.count,
                });
                emitted = emitted.saturating_add(1);
            }
        }

        points
    }
}

#[cfg(all(test, feature = "testutils"))]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;
    use ai_analytics::{AiAnalyticsContract, AiAnalyticsContractClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec;

    #[test]
    fn test_privacy_preserving_aggregation_and_kpi_flow() {
        let env = Env::default();
        let id = env.register_contract(None, HealthcareAnalyticsDashboardContract);
        let client = HealthcareAnalyticsDashboardContractClient::new(&env, &id);

        let admin = Address::generate(&env);
        let collector = Address::generate(&env);

        client.mock_all_auths().initialize(&admin, &5u32, &200u32);
        client
            .mock_all_auths()
            .set_collector(&admin, &collector, &true);

        let metric = String::from_str(&env, "record_access_rate");

        let low_privacy = client.mock_all_auths().try_record_medical_metric(
            &collector,
            &metric,
            &20260325u64,
            &6400u32,
            &3u32,
        );
        assert!(low_privacy.is_err());

        assert!(client.mock_all_auths().record_medical_metric(
            &collector,
            &metric,
            &20260325u64,
            &6400u32,
            &7u32,
        ));

        let aggregate = client.get_metric_aggregate(&metric, &20260325u64);
        assert_eq!(aggregate.count, 1u32);

        assert!(client.mock_all_auths().record_system_snapshot(
            &collector,
            &120u32,
            &1000u32,
            &10u32,
            &180u32,
            &9900u32,
        ));

        let kpi = client.get_performance_kpi().unwrap();
        assert_eq!(kpi.total_snapshots, 1u32);
        assert_eq!(kpi.avg_error_rate_bps, 100u32);

        let points = client.get_visualization_series(&metric, &10u32);
        assert_eq!(points.len(), 1);
        assert_eq!(points.get(0).unwrap().sample_count, 1u32);
    }

    #[test]
    fn test_templates_schedules_compliance_and_export() {
        let env = Env::default();
        let id = env.register_contract(None, HealthcareAnalyticsDashboardContract);
        let client = HealthcareAnalyticsDashboardContractClient::new(&env, &id);

        let admin = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &3u32, &100u32);

        let filters = vec![
            &env,
            String::from_str(&env, "record_access_rate"),
            String::from_str(&env, "system_latency"),
        ];

        let template_id = client.mock_all_auths().create_report_template(
            &admin,
            &String::from_str(&env, "Weekly Ops"),
            &filters,
            &true,
            &true,
            &String::from_str(&env, "csv"),
        );
        let template = client.get_report_template(&template_id).unwrap();
        assert_eq!(template.output_format, String::from_str(&env, "csv"));

        let schedule_id = client
            .mock_all_auths()
            .schedule_report(&admin, &template_id, &86400u64, &100u64);
        let schedule = client.get_report_schedule(&schedule_id).unwrap();
        assert!(schedule.enabled);

        assert!(client.mock_all_auths().upsert_compliance_summary(
            &admin,
            &202603u64,
            &false,
            &2u32,
            &10u32,
            &7800u32,
            &String::from_str(&env, "ipfs://compliance-monthly"),
        ));
        let compliance = client.get_compliance_summary(&202603u64).unwrap();
        assert_eq!(compliance.total_checks, 1u32);
        assert_eq!(compliance.total_violations, 2u32);

        let checksum = BytesN::from_array(&env, &[8u8; 32]);
        let export_id = client.mock_all_auths().run_scheduled_report(
            &admin,
            &schedule_id,
            &String::from_str(&env, "ipfs://exports/weekly-ops.csv"),
            &checksum,
        );
        let export = client.get_export_record(&export_id).unwrap();
        assert_eq!(export.template_id, template_id);
    }

    #[test]
    fn test_ai_analytics_integration_sync_round() {
        let env = Env::default();

        let ai_id = env.register_contract(None, AiAnalyticsContract);
        let ai_client = AiAnalyticsContractClient::new(&env, &ai_id);

        let dash_id = env.register_contract(None, HealthcareAnalyticsDashboardContract);
        let dash_client = HealthcareAnalyticsDashboardContractClient::new(&env, &dash_id);

        let admin = Address::generate(&env);
        let participant_1 = Address::generate(&env);
        let participant_2 = Address::generate(&env);

        ai_client.mock_all_auths().initialize(&admin);

        let base_model = BytesN::from_array(&env, &[1u8; 32]);
        let round_id =
            ai_client
                .mock_all_auths()
                .start_round(&admin, &base_model, &2u32, &50u32);

        ai_client.mock_all_auths().submit_update(
            &participant_1,
            &round_id,
            &BytesN::from_array(&env, &[2u8; 32]),
            &100u32,
        );
        ai_client.mock_all_auths().submit_update(
            &participant_2,
            &round_id,
            &BytesN::from_array(&env, &[3u8; 32]),
            &120u32,
        );

        ai_client.mock_all_auths().finalize_round(
            &admin,
            &round_id,
            &BytesN::from_array(&env, &[4u8; 32]),
            &String::from_str(&env, "model v2"),
            &String::from_str(&env, "ipfs://metrics"),
            &String::from_str(&env, "ipfs://fairness"),
        );

        dash_client.mock_all_auths().initialize(&admin, &2u32, &50u32);
        dash_client
            .mock_all_auths()
            .configure_ai_analytics(&admin, &ai_id);

        let insight = dash_client.mock_all_auths().sync_ai_round(&admin, &round_id);
        assert_eq!(insight.round_id, round_id);
        assert_eq!(insight.total_updates, 2u32);
        assert_eq!(insight.participation_bps, 10_000u32);

        let stored = dash_client.get_ai_round_insight(&round_id).unwrap();
        assert!(stored.is_finalized);
    }
}