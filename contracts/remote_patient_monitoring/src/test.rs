#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env, String, vec};
    use crate::{RemotePatientMonitoringContract, RemotePatientMonitoringContractClient};

    fn setup() -> (Env, RemotePatientMonitoringContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RemotePatientMonitoringContract);
        let client = RemotePatientMonitoringContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_leap_year_vital_sign() {
        let (env, client, admin) = setup();
        let patient = Address::generate(&env);
        let device_id = 1;

        client.register_device(&admin, &device_id, &0, &patient, &vec![&env, String::from_str(&env, "WiFi")]);

        // Mock timestamp to Feb 29, 2028
        let leap_year_ts = 1835352000;
        env.ledger().with_mut(|li| li.timestamp = leap_year_ts);

        client.submit_vital_sign(&patient, &patient, &device_id, &String::from_str(&env, "heart_rate"), &75, &String::from_str(&env, "bpm"), &100);

        let vitals = client.get_vitals(&patient, &1);
        assert_eq!(vitals.get(0).unwrap().timestamp, leap_year_ts);
    }

    #[test]
    fn test_year_2038_vital_sign() {
        let (env, client, admin) = setup();
        let patient = Address::generate(&env);
        let device_id = 1;

        client.register_device(&admin, &device_id, &0, &patient, &vec![&env, String::from_str(&env, "WiFi")]);

        // Mock timestamp to Year 2040
        let future_ts = 2209017600;
        env.ledger().with_mut(|li| li.timestamp = future_ts);

        client.submit_vital_sign(&patient, &patient, &device_id, &String::from_str(&env, "heart_rate"), &75, &String::from_str(&env, "bpm"), &100);

        let vitals = client.get_vitals(&patient, &1);
        assert_eq!(vitals.get(0).unwrap().timestamp, future_ts);
    }
}
