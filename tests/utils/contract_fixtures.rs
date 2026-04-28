/// Library of complex contract fixtures for integration testing
use soroban_sdk::{Address, Env, String, Vec, vec};
use crate::utils::{
    IntegrationTestEnv, UserRole, MedicalRecordGenerator, 
    MedicalEntry, RecordMetadata, UserFixture
};
use medical_records::{MedicalRecordsContractClient, Role};
use sut_token::SutTokenClient;

/// Fixture for a fully configured MedicalRecords contract
pub struct MedicalRecordsFixture {
    pub contract_id: Address,
    pub client: MedicalRecordsContractClient<'static>,
    pub admin: Address,
}

impl MedicalRecordsFixture {
    /// Create a new MedicalRecords fixture with default initialization
    pub fn new(test_env: &IntegrationTestEnv) -> Self {
        let (contract_id, client) = test_env.register_medical_records();
        let admin = &test_env.team.admin.address;
        
        client.initialize(admin);
        
        // Configure standard roles for the team
        for doctor in &test_env.team.doctors {
            client.manage_user(admin, &doctor.address, &Role::Doctor).expect("Failed to manage user");
        }
        for patient in &test_env.team.patients {
            client.manage_user(admin, &patient.address, &Role::Patient).expect("Failed to manage user");
        }
        
        Self {
            contract_id,
            client,
            admin: admin.clone(),
        }
    }

    /// Add a set of sample records to the contract
    pub fn with_sample_data(self, test_env: &IntegrationTestEnv) -> Self {
        let env = &test_env.env;
        let mut gen = MedicalRecordGenerator::new();
        
        let doctor = &test_env.team.doctors[0].address;
        let patient = &test_env.team.patients[0].address;
        
        for i in 0..5 {
            let diagnosis = String::from_str(env, &format!("Sample Diagnosis {}", i));
            let treatment = String::from_str(env, &format!("Sample Treatment {}", i));
            
            self.client.add_record(
                doctor,
                patient,
                &diagnosis,
                &treatment,
                &false,
                &vec![env, String::from_str(env, "sample")],
                &String::from_str(env, "General"),
                &String::from_str(env, "Routine"),
                &String::from_str(env, &format!("QmHash{}", i)),
            ).expect("Failed to add record");
        }
        
        self
    }
}

/// Fixture for a fully configured SUT Token
pub struct SutTokenFixture {
    pub contract_id: Address,
    pub client: SutTokenClient<'static>,
    pub admin: Address,
}

impl SutTokenFixture {
    /// Create a new SUT Token fixture
    pub fn new(test_env: &IntegrationTestEnv) -> Self {
        let (contract_id, client) = test_env.register_token(&test_env.admin);
        
        Self {
            contract_id,
            client,
            admin: test_env.admin.clone(),
        }
    }

    /// Distribute tokens to all team members
    pub fn distribute_tokens(self, test_env: &IntegrationTestEnv, amount: i128) -> Self {
        for user in test_env.team.all_users() {
            self.client.mint(&self.admin, &user.address, &amount).expect("Failed to mint tokens");
        }
        self
    }
}

/// A "Full System" fixture containing all integrated contracts
pub struct SystemFixture {
    pub medical_records: MedicalRecordsFixture,
    pub token: SutTokenFixture,
}

impl SystemFixture {
    /// Create a complete system fixture
    pub fn new(test_env: &IntegrationTestEnv) -> Self {
        let medical_records = MedicalRecordsFixture::new(test_env).with_sample_data(test_env);
        let token = SutTokenFixture::new(test_env).distribute_tokens(test_env, 10_000_000);
        
        Self {
            medical_records,
            token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::IntegrationTestEnv;

    #[test]
    fn test_medical_records_fixture() {
        let test_env = IntegrationTestEnv::new();
        let fixture = MedicalRecordsFixture::new(&test_env);
        assert_eq!(fixture.admin, test_env.admin);
        
        let doctor = &test_env.team.doctors[0].address;
        assert_eq!(fixture.client.get_user_role(doctor), Role::Doctor);
    }

    #[test]
    fn test_token_fixture() {
        let test_env = IntegrationTestEnv::new();
        let fixture = SutTokenFixture::new(&test_env).distribute_tokens(&test_env, 1000);
        
        let patient = &test_env.team.patients[0].address;
        assert_eq!(fixture.client.balance_of(patient), 1000);
    }

    #[test]
    fn test_system_fixture() {
        let test_env = IntegrationTestEnv::new();
        let system = SystemFixture::new(&test_env);
        
        assert!(system.medical_records.client.get_patient_record_count(&test_env.team.patients[0].address) > 0);
        assert!(system.token.client.balance_of(&test_env.team.patients[0].address) > 0);
    }
}
