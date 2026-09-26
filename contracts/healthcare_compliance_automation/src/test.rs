use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};

const FRAMEWORK: &str = "HIPAA";
const ADVISORY_FRAMEWORK: &str = "ISO27001";

/// Builds a short distinct name without `format!`, since the crate is
/// `forbid(alloc)`. `n` must be < 100.
fn name(env: &Env, prefix: u8, n: u32) -> String {
    let tens = b'0' + ((n / 10) % 10) as u8;
    let ones = b'0' + (n % 10) as u8;
    String::from_bytes(env, &[prefix, tens, ones])
}

fn evidence(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

fn control(env: &Env, id: u32, mandatory: bool) -> PolicyControl {
    PolicyControl {
        control_id: name(env, b'C', id),
        title: name(env, b'T', id),
        mandatory,
    }
}

fn setup(env: &Env) -> (HealthcareComplianceAutomationClient<'_>, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register_contract(None, HealthcareComplianceAutomation);
    let client = HealthcareComplianceAutomationClient::new(env, &contract_id);

    let mut frameworks: Vec<String> = Vec::new(env);
    frameworks.push_back(String::from_str(env, FRAMEWORK));
    frameworks.push_back(String::from_str(env, ADVISORY_FRAMEWORK));
    client.initialize(&admin, &frameworks);

    (client, admin)
}

fn framework(env: &Env) -> String {
    String::from_str(env, FRAMEWORK)
}

// ─── Initialization ──────────────────────────────────────────────────────────

#[test]
fn test_initialize_stores_frameworks() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let list = client.get_supported_frameworks();
    assert_eq!(list.frameworks.len(), 2);
    assert_eq!(list.frameworks.get(0).unwrap(), String::from_str(&env, FRAMEWORK));
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_panics() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let mut frameworks: Vec<String> = Vec::new(&env);
    frameworks.push_back(String::from_str(&env, "GDPR"));
    client.initialize(&admin, &frameworks);
}

#[test]
#[should_panic(expected = "duplicate framework")]
fn test_initialize_rejects_duplicate_framework() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, HealthcareComplianceAutomation);
    let client = HealthcareComplianceAutomationClient::new(&env, &contract_id);

    let mut frameworks: Vec<String> = Vec::new(&env);
    frameworks.push_back(String::from_str(&env, "HIPAA"));
    frameworks.push_back(String::from_str(&env, "HIPAA"));
    client.initialize(&admin, &frameworks);
}

#[test]
#[should_panic(expected = "empty framework name")]
fn test_initialize_rejects_empty_framework_name() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, HealthcareComplianceAutomation);
    let client = HealthcareComplianceAutomationClient::new(&env, &contract_id);

    let mut frameworks: Vec<String> = Vec::new(&env);
    frameworks.push_back(String::from_str(&env, ""));
    client.initialize(&admin, &frameworks);
}

#[test]
#[should_panic(expected = "too many frameworks")]
fn test_initialize_rejects_too_many_frameworks() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, HealthcareComplianceAutomation);
    let client = HealthcareComplianceAutomationClient::new(&env, &contract_id);

    let mut frameworks: Vec<String> = Vec::new(&env);
    let mut i: u32 = 0;
    while i <= MAX_FRAMEWORKS {
        frameworks.push_back(name(&env, b'F', i));
        i += 1;
    }
    client.initialize(&admin, &frameworks);
}

// ─── add_framework ───────────────────────────────────────────────────────────

#[test]
fn test_add_framework_by_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.add_framework(&admin, &String::from_str(&env, "GDPR"));
    assert_eq!(client.get_supported_frameworks().frameworks.len(), 3);
}

/// The scaffold accepted any authenticated caller: `admin` was authenticated but
/// never compared against the stored admin, so anyone could extend the list.
#[test]
#[should_panic(expected = "caller is not the admin")]
fn test_add_framework_by_non_admin_panics() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let impostor = Address::generate(&env);

    client.add_framework(&impostor, &String::from_str(&env, "GDPR"));
}

#[test]
#[should_panic(expected = "framework already supported")]
fn test_add_duplicate_framework_panics() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.add_framework(&admin, &String::from_str(&env, FRAMEWORK));
}

// ─── add_control ─────────────────────────────────────────────────────────────

#[test]
fn test_add_control_and_read_back() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    client.add_control(&admin, &framework(&env), &control(&env, 2, false));

    let list = client.get_controls(&framework(&env));
    assert_eq!(list.controls.len(), 2);
    assert!(list.controls.get(0).unwrap().mandatory);
    assert!(!list.controls.get(1).unwrap().mandatory);
}

#[test]
fn test_add_control_unknown_framework_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    assert_eq!(
        client.try_add_control(&admin, &String::from_str(&env, "NOPE"), &control(&env, 1, true)),
        Err(Ok(3))
    );
}

#[test]
fn test_add_duplicate_control_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    assert_eq!(
        client.try_add_control(&admin, &framework(&env), &control(&env, 1, true)),
        Err(Ok(4))
    );
}

#[test]
fn test_add_control_empty_id_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let mut c = control(&env, 1, true);
    c.control_id = String::from_str(&env, "");
    assert_eq!(
        client.try_add_control(&admin, &framework(&env), &c),
        Err(Ok(8))
    );
}

#[test]
fn test_add_control_by_non_admin_rejected() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let impostor = Address::generate(&env);

    assert_eq!(
        client.try_add_control(&impostor, &framework(&env), &control(&env, 1, true)),
        Err(Ok(2))
    );
}

#[test]
fn test_controls_are_per_framework() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    let other = client.get_controls(&String::from_str(&env, ADVISORY_FRAMEWORK));

    assert_eq!(other.controls.len(), 0);
}

// ─── record_control_status ───────────────────────────────────────────────────

#[test]
fn test_record_status_rejects_unknown_control() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));

    assert_eq!(
        client.try_record_control_status(
            &admin,
            &framework(&env),
            &name(&env, b'C', 9),
            &true,
            &evidence(&env, 1),
        ),
        Err(Ok(5))
    );
}

#[test]
fn test_record_status_rejects_unknown_framework() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    assert_eq!(
        client.try_record_control_status(
            &admin,
            &String::from_str(&env, "NOPE"),
            &name(&env, b'C', 1),
            &true,
            &evidence(&env, 1),
        ),
        Err(Ok(3))
    );
}

#[test]
fn test_record_status_by_non_admin_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    let impostor = Address::generate(&env);

    assert_eq!(
        client.try_record_control_status(
            &impostor,
            &framework(&env),
            &name(&env, b'C', 1),
            &true,
            &evidence(&env, 1),
        ),
        Err(Ok(2))
    );
}

#[test]
fn test_record_status_overwrites_previous_assessment() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));

    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &false,
        &evidence(&env, 1),
    );
    env.ledger().with_mut(|li| li.timestamp = 2_000);
    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &true,
        &evidence(&env, 2),
    );

    let report = client.evaluate_framework(&admin, &framework(&env));
    assert_eq!(report.satisfied, 1);
    assert!(report.results.get(0).unwrap().assessed);
    assert_eq!(report.results.get(0).unwrap().reported_at, 2_000);
}

// ─── evaluate_framework ──────────────────────────────────────────────────────

#[test]
fn test_evaluate_without_controls_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    assert_eq!(
        client.try_evaluate_framework(&admin, &framework(&env)),
        Err(Ok(6))
    );
}

#[test]
fn test_evaluate_by_non_admin_rejected() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    let impostor = Address::generate(&env);

    assert_eq!(
        client.try_evaluate_framework(&impostor, &framework(&env)),
        Err(Ok(2))
    );
}

/// With nothing assessed, mandatory controls are violations, so an untouched
/// checklist cannot pass.
#[test]
fn test_evaluate_unassessed_mandatory_controls_fail() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    client.add_control(&admin, &framework(&env), &control(&env, 2, true));

    let report = client.evaluate_framework(&admin, &framework(&env));

    assert_eq!(report.total, 2);
    assert_eq!(report.unassessed, 2);
    assert_eq!(report.satisfied, 0);
    assert_eq!(report.mandatory_violations, 2);
    assert!(!report.compliant);
    assert!(!report.results.get(0).unwrap().assessed);
}

#[test]
fn test_evaluate_all_mandatory_satisfied_passes() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 5_000);
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    client.add_control(&admin, &framework(&env), &control(&env, 2, true));

    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &true,
        &evidence(&env, 1),
    );
    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 2),
        &true,
        &evidence(&env, 2),
    );

    let report = client.evaluate_framework(&admin, &framework(&env));

    assert_eq!(report.total, 2);
    assert_eq!(report.satisfied, 2);
    assert_eq!(report.violated, 0);
    assert_eq!(report.unassessed, 0);
    assert_eq!(report.mandatory_violations, 0);
    assert!(report.compliant);
    assert_eq!(report.evaluated_at, 5_000);
}

#[test]
fn test_evaluate_failed_mandatory_control_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));

    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &false,
        &evidence(&env, 1),
    );

    let report = client.evaluate_framework(&admin, &framework(&env));

    assert_eq!(report.violated, 1);
    assert_eq!(report.mandatory_violations, 1);
    assert!(!report.compliant);
    // A recorded failure is distinct from no assessment.
    assert!(report.results.get(0).unwrap().assessed);
    assert!(!report.results.get(0).unwrap().satisfied);
}

/// An advisory control failing is reported but does not decide compliance.
#[test]
fn test_advisory_violation_does_not_break_compliance() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    client.add_control(&admin, &framework(&env), &control(&env, 2, false));

    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &true,
        &evidence(&env, 1),
    );
    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 2),
        &false,
        &evidence(&env, 2),
    );

    let report = client.evaluate_framework(&admin, &framework(&env));

    assert_eq!(report.satisfied, 1);
    assert_eq!(report.violated, 1);
    assert_eq!(report.mandatory_violations, 0);
    assert!(report.compliant);
}

/// An unassessed advisory control is a violation in the tally but not a
/// mandatory one, so it still cannot silently pass as compliant if a mandatory
/// control is also missing.
#[test]
fn test_mixed_assessed_and_unassessed_report() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));
    client.add_control(&admin, &framework(&env), &control(&env, 2, false));
    client.add_control(&admin, &framework(&env), &control(&env, 3, true));

    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &true,
        &evidence(&env, 1),
    );

    let report = client.evaluate_framework(&admin, &framework(&env));

    assert_eq!(report.total, 3);
    assert_eq!(report.satisfied, 1);
    assert_eq!(report.unassessed, 2);
    // C1 satisfied, C2 advisory-unassessed, C3 mandatory-unassessed.
    assert_eq!(report.mandatory_violations, 1);
    assert!(!report.compliant);
}

// ─── get_last_report ─────────────────────────────────────────────────────────

#[test]
fn test_get_last_report_before_evaluation_reports_missing() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    assert_eq!(
        client.try_get_last_report(&framework(&env)),
        Err(Ok(9))
    );
}

#[test]
fn test_get_last_report_returns_latest_evaluation() {
    let env = Env::default();
    env.ledger().with_mut(|li| li.timestamp = 1_000);
    let (client, admin) = setup(&env);
    client.add_control(&admin, &framework(&env), &control(&env, 1, true));

    client.evaluate_framework(&admin, &framework(&env));
    env.ledger().with_mut(|li| li.timestamp = 9_000);
    client.record_control_status(
        &admin,
        &framework(&env),
        &name(&env, b'C', 1),
        &true,
        &evidence(&env, 1),
    );
    client.evaluate_framework(&admin, &framework(&env));

    let stored = client.get_last_report(&framework(&env));
    assert_eq!(stored.evaluated_at, 9_000);
    assert!(stored.compliant);
}

#[test]
fn test_getters_reject_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, HealthcareComplianceAutomation);
    let client = HealthcareComplianceAutomationClient::new(&env, &contract_id);

    // Both call require_initialized before anything else, so the code is
    // NotInitialized rather than NotAdmin.
    assert_eq!(client.try_get_controls(&framework(&env)), Err(Ok(1)));
    assert_eq!(client.try_get_last_report(&framework(&env)), Err(Ok(1)));
}

#[test]
fn test_add_control_rejects_too_many_controls() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let mut i: u32 = 0;
    while i < MAX_CONTROLS {
        client.add_control(&admin, &framework(&env), &control(&env, i, true));
        i += 1;
    }
    assert_eq!(client.get_controls(&framework(&env)).controls.len(), MAX_CONTROLS);

    assert_eq!(
        client.try_add_control(&admin, &framework(&env), &control(&env, 99, true)),
        Err(Ok(7))
    );
}

#[test]
#[should_panic(expected = "contract not initialized")]
fn test_add_framework_before_initialization_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, HealthcareComplianceAutomation);
    let client = HealthcareComplianceAutomationClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.add_framework(&admin, &String::from_str(&env, "GDPR"));
}
