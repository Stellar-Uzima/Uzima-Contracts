use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env, String, Vec,
};

fn create_test_env() -> (Env, Address, PharmaSupplyChainClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(PharmaSupplyChain, ());
    let client = PharmaSupplyChainClient::new(&env, &contract_id);

    client.initialize(&admin);

    (env, admin, client)
}

#[test]
fn test_initialize() {
    let (_env, _admin, client) = create_test_env();

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_batches, 0);
    assert_eq!(analytics.active_shipments, 0);
    assert_eq!(analytics.total_recalls, 0);
}

#[test]
fn test_register_manufacturer() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(
        &env,
        [
            String::from_str(&env, "GMP"),
            String::from_str(&env, "ISO9001"),
        ],
    );

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );
}

#[test]
fn test_register_medication() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    // Register manufacturer first
    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Acetaminophen")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "500mg"),
        active_ingredients,
    };
    // Register medication
    client.register_medication(
        &String::from_str(&env, "MED001"),
        &String::from_str(&env, "Paracetamol 500mg"),
        &String::from_str(&env, "Acetaminophen"),
        &String::from_str(&env, "0000-0000-00"),
        &MedicationType::OverTheCounter,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    let medication = client.get_medication(&String::from_str(&env, "MED001"));
    assert!(medication.is_some());
    assert_eq!(
        medication.unwrap().name,
        String::from_str(&env, "Paracetamol 500mg")
    );
}

#[test]
fn test_create_batch_with_authentication() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    // Register manufacturer
    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Acetaminophen")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "500mg"),
        active_ingredients,
    };
    // Register medication
    client.register_medication(
        &String::from_str(&env, "MED001"),
        &String::from_str(&env, "Paracetamol 500mg"),
        &String::from_str(&env, "Acetaminophen"),
        &String::from_str(&env, "0000-0000-00"),
        &MedicationType::OverTheCounter,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    // Create batch
    let auth_hash = client.create_batch(
        &String::from_str(&env, "BATCH001"),
        &String::from_str(&env, "MED001"),
        &10000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-001"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-001"),
    );

    assert!(!auth_hash.is_empty());

    let batch = client.get_batch(&String::from_str(&env, "BATCH001"));
    assert!(batch.is_some());

    let batch_data = batch.unwrap();
    assert_eq!(batch_data.quantity, 10000);
    assert_eq!(batch_data.current_stage, SupplyChainStage::Manufacturing);

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_batches, 1);
}

#[test]
fn test_verify_batch_authenticity() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Acetaminophen")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "500mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED001"),
        &String::from_str(&env, "Paracetamol 500mg"),
        &String::from_str(&env, "Acetaminophen"),
        &String::from_str(&env, "0000-0000-00"),
        &MedicationType::OverTheCounter,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    let auth_hash = client.create_batch(
        &String::from_str(&env, "BATCH001"),
        &String::from_str(&env, "MED001"),
        &10000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-001"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-001"),
    );

    // Verify with correct hash
    let is_valid =
        client.verify_batch_authenticity(&String::from_str(&env, "BATCH001"), &auth_hash);
    assert!(is_valid);

    // Verify with incorrect hash
    let fake_hash = Bytes::from_array(&env, &[1, 2, 3, 4]);
    let is_valid_fake =
        client.verify_batch_authenticity(&String::from_str(&env, "BATCH001"), &fake_hash);
    assert!(!is_valid_fake);

    let analytics = client.get_analytics();
    assert_eq!(analytics.counterfeit_attempts, 1);
}

#[test]
fn test_create_shipment_with_iot() {
    let (env, _admin, client) = create_test_env();

    // Setup manufacturer and medication
    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Insulin")]);

    let med_config = MedicationConfig {
        requires_cold_chain: true,
        min_temp_celsius: 2,
        max_temp_celsius: 8,
        max_humidity_percent: 70,
        shelf_life_days: 365,
        dosage_form: String::from_str(&env, "Injectable"),
        strength: String::from_str(&env, "100IU/ml"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED002"),
        &String::from_str(&env, "Insulin Injection"),
        &String::from_str(&env, "Human Insulin"),
        &String::from_str(&env, "0000-0000-01"),
        &MedicationType::Prescription,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH002"),
        &String::from_str(&env, "MED002"),
        &5000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-002"),
        &String::from_str(&env, "Facility B"),
        &String::from_str(&env, "QC-CERT-002"),
    );

    let distributor = Address::generate(&env);

    // Create shipment with IoT device
    client.create_shipment(
        &String::from_str(&env, "SHIP001"),
        &String::from_str(&env, "BATCH002"),
        &1000,
        &distributor,
        &SupplyChainStage::Distribution,
        &(env.ledger().timestamp() + 86400),
        &Some(String::from_str(&env, "IOT-DEVICE-001")),
    );

    let shipment = client.get_shipment(&String::from_str(&env, "SHIP001"));
    assert!(shipment.is_some());

    let shipment_data = shipment.unwrap();
    assert_eq!(shipment_data.status, ShipmentStatus::InTransit);

    let analytics = client.get_analytics();
    assert_eq!(analytics.active_shipments, 1);
}

#[test]
fn test_condition_monitoring_with_violations() {
    let (env, _admin, client) = create_test_env();

    // Setup
    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Vaccine")]);

    let med_config = MedicationConfig {
        requires_cold_chain: true,
        min_temp_celsius: -70,
        max_temp_celsius: -60,
        max_humidity_percent: 50,
        shelf_life_days: 180,
        dosage_form: String::from_str(&env, "Injectable"),
        strength: String::from_str(&env, "30mcg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED003"),
        &String::from_str(&env, "COVID-19 Vaccine"),
        &String::from_str(&env, "mRNA Vaccine"),
        &String::from_str(&env, "0000-0000-02"),
        &MedicationType::Vaccine,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH003"),
        &String::from_str(&env, "MED003"),
        &2000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-003"),
        &String::from_str(&env, "Facility C"),
        &String::from_str(&env, "QC-CERT-003"),
    );

    let distributor = Address::generate(&env);

    client.create_shipment(
        &String::from_str(&env, "SHIP002"),
        &String::from_str(&env, "BATCH003"),
        &500,
        &distributor,
        &SupplyChainStage::Distribution,
        &(env.ledger().timestamp() + 86400),
        &Some(String::from_str(&env, "IOT-DEVICE-002")),
    );

    // Log normal condition
    client.log_condition_data(
        &String::from_str(&env, "LOG001"),
        &String::from_str(&env, "SHIP002"),
        &-65, // Within range
        &45,
        &Some(40750000),  // Latitude
        &Some(-73980000), // Longitude
        &String::from_str(&env, "IOT-DEVICE-002"),
    );

    let analytics = client.get_analytics();
    assert_eq!(analytics.condition_violations, 0);

    // Log temperature violation
    client.log_condition_data(
        &String::from_str(&env, "LOG002"),
        &String::from_str(&env, "SHIP002"),
        &-50, // Above max temp
        &45,
        &Some(40750000),
        &Some(-73980000),
        &String::from_str(&env, "IOT-DEVICE-002"),
    );

    let analytics = client.get_analytics();
    assert_eq!(analytics.condition_violations, 1);

    let shipment = client.get_shipment(&String::from_str(&env, "SHIP002"));
    assert_eq!(shipment.unwrap().status, ShipmentStatus::ConditionViolation);
}

#[test]
fn test_complete_shipment() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Acetaminophen")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "500mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED001"),
        &String::from_str(&env, "Paracetamol 500mg"),
        &String::from_str(&env, "Acetaminophen"),
        &String::from_str(&env, "0000-0000-00"),
        &MedicationType::OverTheCounter,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH001"),
        &String::from_str(&env, "MED001"),
        &10000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-001"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-001"),
    );

    let distributor = Address::generate(&env);

    client.create_shipment(
        &String::from_str(&env, "SHIP003"),
        &String::from_str(&env, "BATCH001"),
        &5000,
        &distributor,
        &SupplyChainStage::Distribution,
        &(env.ledger().timestamp() + 86400),
        &None,
    );

    // Complete shipment
    client.complete_shipment(&String::from_str(&env, "SHIP003"), &true);

    let shipment = client.get_shipment(&String::from_str(&env, "SHIP003"));
    assert_eq!(shipment.clone().unwrap().status, ShipmentStatus::Delivered);
    assert!(shipment.unwrap().delivered_at.is_some());

    // Verify batch location updated
    let batch = client.get_batch(&String::from_str(&env, "BATCH001"));
    assert_eq!(batch.clone().unwrap().current_holder, distributor);
    assert_eq!(batch.unwrap().current_stage, SupplyChainStage::Distribution);
}

#[test]
fn test_prescription_and_dispensation() {
    let (env, _admin, client) = create_test_env();

    // Setup
    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Amoxicillin")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Capsule"),
        strength: String::from_str(&env, "500mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED004"),
        &String::from_str(&env, "Amoxicillin 500mg"),
        &String::from_str(&env, "Amoxicillin"),
        &String::from_str(&env, "0000-0000-03"),
        &MedicationType::Prescription,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH004"),
        &String::from_str(&env, "MED004"),
        &10000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-004"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-004"),
    );

    let patient = Address::generate(&env);
    let prescriber = Address::generate(&env);

    // Create prescription
    client.create_prescription(
        &String::from_str(&env, "RX001"),
        &prescriber,
        &patient,
        &String::from_str(&env, "MED004"),
        &30,
        &String::from_str(&env, "Take 1 capsule every 8 hours"),
        &2,
        &(env.ledger().timestamp() + 7776000), // 90 days
        &Some(String::from_str(&env, "EMR-12345")),
    );

    // Dispense medication
    let pharmacist = Address::generate(&env);
    client.dispense_medication(
        &String::from_str(&env, "DISP001"),
        &pharmacist,
        &String::from_str(&env, "RX001"),
        &String::from_str(&env, "BATCH004"),
        &30,
        &String::from_str(&env, "VERIFY-123456"),
    );
}

#[test]
fn test_controlled_substance_tracking() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Oxycodone")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "10mg"),
        active_ingredients,
    };
    // Register controlled substance (Schedule 2)
    client.register_medication(
        &String::from_str(&env, "MED005"),
        &String::from_str(&env, "Oxycodone 10mg"),
        &String::from_str(&env, "Oxycodone"),
        &String::from_str(&env, "0000-0000-04"),
        &MedicationType::ControlledSubstance,
        &ControlledSubstanceSchedule::Schedule2,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH005"),
        &String::from_str(&env, "MED005"),
        &1000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-005"),
        &String::from_str(&env, "Facility D"),
        &String::from_str(&env, "QC-CERT-005"),
    );

    let patient = Address::generate(&env);
    let prescriber = Address::generate(&env);

    client.create_prescription(
        &String::from_str(&env, "RX002"),
        &prescriber,
        &patient,
        &String::from_str(&env, "MED005"),
        &20,
        &String::from_str(&env, "Take 1 tablet every 12 hours for pain"),
        &0,                                    // No refills for Schedule 2
        &(env.ledger().timestamp() + 2592000), // 30 days
        &Some(String::from_str(&env, "EMR-67890")),
    );

    let pharmacist = Address::generate(&env);
    client.dispense_medication(
        &String::from_str(&env, "DISP002"),
        &pharmacist,
        &String::from_str(&env, "RX002"),
        &String::from_str(&env, "BATCH005"),
        &20,
        &String::from_str(&env, "VERIFY-789012"),
    );

    let analytics = client.get_analytics();
    assert_eq!(analytics.cs_dispensations, 1);
}

#[test]
fn test_recall_management() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Aspirin")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 1095,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "325mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED006"),
        &String::from_str(&env, "Aspirin 325mg"),
        &String::from_str(&env, "Acetylsalicylic Acid"),
        &String::from_str(&env, "0000-0000-05"),
        &MedicationType::OverTheCounter,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH006"),
        &String::from_str(&env, "MED006"),
        &50000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-006"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-006"),
    );

    // Initiate recall
    let batch_ids = Vec::from_array(&env, [String::from_str(&env, "BATCH006")]);

    client.initiate_recall(
        &String::from_str(&env, "RECALL001"),
        &batch_ids,
        &String::from_str(&env, "MED006"),
        &RecallLevel::Class2,
        &String::from_str(
            &env,
            "Potential contamination detected in manufacturing facility",
        ),
    );

    let recall = client.get_recall(&String::from_str(&env, "RECALL001"));
    assert!(recall.is_some());
    assert_eq!(recall.unwrap().level, RecallLevel::Class2);

    let batch = client.get_batch(&String::from_str(&env, "BATCH006"));
    assert!(batch.unwrap().is_recalled);

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_recalls, 1);
}

#[test]
fn test_adverse_event_reporting() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Penicillin")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "500mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED007"),
        &String::from_str(&env, "Penicillin V 500mg"),
        &String::from_str(&env, "Penicillin"),
        &String::from_str(&env, "0000-0000-06"),
        &MedicationType::Prescription,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    client.create_batch(
        &String::from_str(&env, "BATCH007"),
        &String::from_str(&env, "MED007"),
        &10000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-007"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-007"),
    );

    let patient = Address::generate(&env);
    let reporter = Address::generate(&env);

    // Report adverse event
    client.report_adverse_event(
        &String::from_str(&env, "AE001"),
        &reporter,
        &patient,
        &String::from_str(&env, "MED007"),
        &String::from_str(&env, "BATCH007"),
        &String::from_str(&env, "DISP003"),
        &3, // Moderate severity
        &String::from_str(&env, "Allergic reaction - rash and itching"),
    );

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_adverse_events, 1);
}

#[test]
fn test_expiry_checking() {
    let (env, _admin, client) = create_test_env();

    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Ibuprofen")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 2, // Very short shelf life for testing (2 days)
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "200mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED008"),
        &String::from_str(&env, "Ibuprofen 200mg"),
        &String::from_str(&env, "Ibuprofen"),
        &String::from_str(&env, "0000-0000-07"),
        &MedicationType::OverTheCounter,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    env.ledger().with_mut(|li| {
        li.timestamp = 86400 * 30;
    });
    let old_date = env.ledger().timestamp() - (86400 * 10); // 10 days ago

    client.create_batch(
        &String::from_str(&env, "BATCH008"),
        &String::from_str(&env, "MED008"),
        &5000,
        &old_date,
        &String::from_str(&env, "LOT-2024-008"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-008"),
    );

    let is_expired = client.is_batch_expired(&String::from_str(&env, "BATCH008"));
    assert!(is_expired);
}

#[test]
fn test_supply_chain_transparency() {
    let (env, _admin, client) = create_test_env();

    // Full supply chain test
    let manufacturer_address = Address::generate(&env);
    let certifications = Vec::from_array(&env, [String::from_str(&env, "GMP")]);

    client.register_manufacturer(
        &String::from_str(&env, "MFG001"),
        &manufacturer_address,
        &String::from_str(&env, "PharmaCorp Inc"),
        &String::from_str(&env, "LIC-12345"),
        &certifications,
        &String::from_str(&env, "USA"),
    );

    let active_ingredients = Vec::from_array(&env, [String::from_str(&env, "Metformin")]);

    let med_config = MedicationConfig {
        requires_cold_chain: false,
        min_temp_celsius: 15,
        max_temp_celsius: 30,
        max_humidity_percent: 60,
        shelf_life_days: 730,
        dosage_form: String::from_str(&env, "Tablet"),
        strength: String::from_str(&env, "1000mg"),
        active_ingredients,
    };
    client.register_medication(
        &String::from_str(&env, "MED009"),
        &String::from_str(&env, "Metformin 1000mg"),
        &String::from_str(&env, "Metformin"),
        &String::from_str(&env, "0000-0000-08"),
        &MedicationType::Prescription,
        &ControlledSubstanceSchedule::NotControlled,
        &String::from_str(&env, "MFG001"),
        &med_config,
    );

    // Create batch
    let _auth_hash = client.create_batch(
        &String::from_str(&env, "BATCH009"),
        &String::from_str(&env, "MED009"),
        &20000,
        &env.ledger().timestamp(),
        &String::from_str(&env, "LOT-2024-009"),
        &String::from_str(&env, "Facility A"),
        &String::from_str(&env, "QC-CERT-009"),
    );

    // Ship to distributor
    let distributor = Address::generate(&env);
    client.create_shipment(
        &String::from_str(&env, "SHIP004"),
        &String::from_str(&env, "BATCH009"),
        &10000,
        &distributor,
        &SupplyChainStage::Distribution,
        &(env.ledger().timestamp() + 86400),
        &None,
    );

    client.complete_shipment(&String::from_str(&env, "SHIP004"), &true);

    // Ship to pharmacy
    let pharmacy = Address::generate(&env);
    client.create_shipment(
        &String::from_str(&env, "SHIP005"),
        &String::from_str(&env, "BATCH009"),
        &1000,
        &pharmacy,
        &SupplyChainStage::Pharmacy,
        &(env.ledger().timestamp() + 172800),
        &None,
    );

    client.complete_shipment(&String::from_str(&env, "SHIP005"), &true);

    // Verify final location
    let batch = client.get_batch(&String::from_str(&env, "BATCH009"));
    assert_eq!(
        batch.clone().unwrap().current_stage,
        SupplyChainStage::Pharmacy
    );
    assert_eq!(batch.unwrap().current_holder, pharmacy);
}
