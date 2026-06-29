use interoperability_suite::InteroperabilitySuite;

const CONTRACTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../contracts");

fn build_suite() -> InteroperabilitySuite {
    InteroperabilitySuite::discover_from_contract_dir(CONTRACTS_DIR)
        .expect("should discover contracts and create pair matrix")
}

#[test]
fn cross_contract_calls_cover_all_contract_pairs() {
    let mut suite = build_suite();
    suite
        .run_cross_contract_calls()
        .expect("cross-contract call checks should pass");

    suite
        .assert_expected_pair_count()
        .expect("pair count should match n*(n-1)/2");
    suite
        .assert_cross_contract_calls_covered()
        .expect("all pairs should be covered for cross-contract calls");
}

#[test]
fn data_format_compatibility_covers_all_contract_pairs() {
    let mut suite = build_suite();
    suite
        .run_data_format_compatibility()
        .expect("data format compatibility checks should pass");

    suite
        .assert_expected_pair_count()
        .expect("pair count should match n*(n-1)/2");
    suite
        .assert_data_format_compatibility_covered()
        .expect("all pairs should be covered for data format compatibility");
}

#[test]
fn event_subscription_handling_covers_all_contract_pairs() {
    let mut suite = build_suite();
    suite
        .run_event_subscription_handling()
        .expect("event subscription handling checks should pass");

    suite
        .assert_expected_pair_count()
        .expect("pair count should match n*(n-1)/2");
    suite
        .assert_event_subscription_handling_covered()
        .expect("all pairs should be covered for event subscriptions");
}

#[test]
fn state_consistency_checks_cover_all_contract_pairs() {
    let mut suite = build_suite();
    suite
        .run_state_consistency_checks()
        .expect("state consistency checks should pass");

    suite
        .assert_expected_pair_count()
        .expect("pair count should match n*(n-1)/2");
    suite
        .assert_state_consistency_checks_covered()
        .expect("all pairs should be covered for state consistency");
}

#[test]
fn upgrade_compatibility_covers_all_contract_pairs() {
    let mut suite = build_suite();
    suite
        .run_upgrade_compatibility_checks()
        .expect("upgrade compatibility checks should pass");

    suite
        .assert_expected_pair_count()
        .expect("pair count should match n*(n-1)/2");
    suite
        .assert_upgrade_compatibility_covered()
        .expect("all pairs should be covered for upgrade compatibility");
}

#[test]
fn interoperability_suite_is_operational_end_to_end() {
    let mut suite = build_suite();
    suite
        .run_all_scenarios()
        .expect("all interoperability scenarios should execute successfully");

    suite
        .assert_expected_pair_count()
        .expect("pair count should match n*(n-1)/2");
    suite
        .assert_full_coverage()
        .expect("all pairs should be fully covered across all scenarios");
}
