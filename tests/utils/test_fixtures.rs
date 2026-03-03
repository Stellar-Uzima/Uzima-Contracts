/// Test fixtures for different user roles and scenarios
use soroban_sdk::{Address, Env, String as SorobanString};

/// User role fixture
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserRole {
    Admin,
    Doctor,
    Patient,
    Nurse,
    Pharmacist,
    DataAnalyst,
    AuditorRole,
    GovernanceRole,
}

/// Fixture for a test user with a specific role
#[derive(Clone)]
pub struct UserFixture {
    pub address: Address,
    pub role: UserRole,
    pub name: String,
    pub email: String,
    pub verified: bool,
}

impl UserFixture {
    /// Create a new user fixture
    pub fn new(env: &Env, address: Address, role: UserRole, name: &str, email: &str) -> Self {
        Self {
            address,
            role,
            name: name.to_string(),
            email: email.to_string(),
            verified: false,
        }
    }

    /// Mark user as verified
    pub fn verified(mut self) -> Self {
        self.verified = true;
        self
    }
}

/// Test fixture factory for common user scenarios
pub struct UserFixtureFactory;

impl UserFixtureFactory {
    /// Create admin fixture
    pub fn create_admin(env: &Env) -> UserFixture {
        let address = Address::generate(env);
        UserFixture::new(env, address, UserRole::Admin, "Admin User", "admin@hospital.com").verified()
    }

    /// Create doctor fixture
    pub fn create_doctor(env: &Env) -> UserFixture {
        let address = Address::generate(env);
        UserFixture::new(env, address, UserRole::Doctor, "Dr. Smith", "smith@hospital.com").verified()
    }

    /// Create multiple doctors
    pub fn create_doctors(env: &Env, count: usize) -> Vec<UserFixture> {
        (0..count)
            .map(|i| {
                let address = Address::generate(env);
                UserFixture::new(
                    env,
                    address,
                    UserRole::Doctor,
                    &format!("Dr. Doctor{}", i),
                    &format!("doctor{}@hospital.com", i),
                )
                .verified()
            })
            .collect()
    }

    /// Create patient fixture
    pub fn create_patient(env: &Env) -> UserFixture {
        let address = Address::generate(env);
        UserFixture::new(env, address, UserRole::Patient, "John Patient", "patient@example.com")
            .verified()
    }

    /// Create multiple patients
    pub fn create_patients(env: &Env, count: usize) -> Vec<UserFixture> {
        (0..count)
            .map(|i| {
                let address = Address::generate(env);
                UserFixture::new(
                    env,
                    address,
                    UserRole::Patient,
                    &format!("Patient{}", i),
                    &format!("patient{}@example.com", i),
                )
                .verified()
            })
            .collect()
    }

    /// Create nurse fixture
    pub fn create_nurse(env: &Env) -> UserFixture {
        let address = Address::generate(env);
        UserFixture::new(env, address, UserRole::Nurse, "Nurse Jane", "nurse@hospital.com")
            .verified()
    }

    /// Create pharmacist fixture
    pub fn create_pharmacist(env: &Env) -> UserFixture {
        let address = Address::generate(env);
        UserFixture::new(
            env,
            address,
            UserRole::Pharmacist,
            "Pharmacist Bob",
            "pharmacist@pharmacy.com",
        )
        .verified()
    }

    /// Create complete healthcare team
    pub fn create_healthcare_team(env: &Env) -> HealthcareTeam {
        HealthcareTeam {
            admin: Self::create_admin(env),
            doctors: Self::create_doctors(env, 3),
            patients: Self::create_patients(env, 5),
            nurses: (0..2).map(|_| Self::create_nurse(env)).collect(),
            pharmacists: (0..1).map(|_| Self::create_pharmacist(env)).collect(),
        }
    }
}

/// Complete healthcare team fixture
pub struct HealthcareTeam {
    pub admin: UserFixture,
    pub doctors: Vec<UserFixture>,
    pub patients: Vec<UserFixture>,
    pub nurses: Vec<UserFixture>,
    pub pharmacists: Vec<UserFixture>,
}

impl HealthcareTeam {
    /// Get all users from the team
    pub fn all_users(&self) -> Vec<&UserFixture> {
        let mut users = vec![&self.admin];
        users.extend(self.doctors.iter());
        users.extend(self.patients.iter());
        users.extend(self.nurses.iter());
        users.extend(self.pharmacists.iter());
        users
    }

    /// Get all user addresses
    pub fn all_addresses(&self) -> Vec<Address> {
        self.all_users().iter().map(|u| u.address.clone()).collect()
    }
}

/// Test scenario fixtures
pub struct ScenarioFixture {
    pub name: String,
    pub description: String,
    pub preconditions: Vec<String>,
    pub expected_outcomes: Vec<String>,
}

impl ScenarioFixture {
    /// Create new scenario
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            preconditions: Vec::new(),
            expected_outcomes: Vec::new(),
        }
    }

    /// Add precondition
    pub fn with_precondition(mut self, precondition: &str) -> Self {
        self.preconditions.push(precondition.to_string());
        self
    }

    /// Add expected outcome
    pub fn with_expected_outcome(mut self, outcome: &str) -> Self {
        self.expected_outcomes.push(outcome.to_string());
        self
    }
}

/// Common test scenarios
pub mod scenarios {
    use super::*;

    /// Scenario: Patient creates medical record
    pub fn patient_record_creation() -> ScenarioFixture {
        ScenarioFixture::new(
            "Patient Creates Medical Record",
            "A patient initializes a new medical record",
        )
        .with_precondition("Patient has valid account")
        .with_precondition("Patient has required permissions")
        .with_expected_outcome("Record is created successfully")
        .with_expected_outcome("Record ID is generated")
        .with_expected_outcome("Patient is listed as owner")
    }

    /// Scenario: Doctor shares record with another doctor
    pub fn record_sharing() -> ScenarioFixture {
        ScenarioFixture::new(
            "Doctor Shares Record",
            "A doctor shares a patient record with a colleague",
        )
        .with_precondition("Doctor has access to record")
        .with_precondition("Recipient is verified doctor")
        .with_expected_outcome("Record is shared")
        .with_expected_outcome("Share event is logged")
    }

    /// Scenario: Patient consents to record access
    pub fn consent_management() -> ScenarioFixture {
        ScenarioFixture::new(
            "Patient Consent Management",
            "Patient grants and revokes consent for record access",
        )
        .with_precondition("Patient owns the record")
        .with_precondition("Record exists and is valid")
        .with_expected_outcome("Consent is granted/revoked")
        .with_expected_outcome("Consent history is maintained")
    }

    /// Scenario: Multi-hospital data access
    pub fn cross_hospital_access() -> ScenarioFixture {
        ScenarioFixture::new(
            "Cross-Hospital Record Access",
            "Healthcare providers from different hospitals access patient records",
        )
        .with_precondition("Patient has consented to cross-hospital access")
        .with_precondition("Both hospitals are verified")
        .with_expected_outcome("Record is accessible across hospitals")
        .with_expected_outcome("Access is logged and auditable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_fixture_creation() {
        let env = soroban_sdk::Env::default();
        let addr = Address::generate(&env);
        let user = UserFixture::new(&env, addr.clone(), UserRole::Doctor, "Test", "test@test.com");
        assert_eq!(user.role, UserRole::Doctor);
        assert!(!user.verified);
    }

    #[test]
    fn test_user_fixture_verified() {
        let env = soroban_sdk::Env::default();
        let addr = Address::generate(&env);
        let user = UserFixture::new(&env, addr, UserRole::Patient, "Test", "test@test.com")
            .verified();
        assert!(user.verified);
    }

    #[test]
    fn test_fixture_factory_admin() {
        let env = soroban_sdk::Env::default();
        let admin = UserFixtureFactory::create_admin(&env);
        assert_eq!(admin.role, UserRole::Admin);
        assert!(admin.verified);
    }

    #[test]
    fn test_fixture_factory_doctors() {
        let env = soroban_sdk::Env::default();
        let doctors = UserFixtureFactory::create_doctors(&env, 3);
        assert_eq!(doctors.len(), 3);
        for doctor in doctors {
            assert_eq!(doctor.role, UserRole::Doctor);
        }
    }

    #[test]
    fn test_healthcare_team() {
        let env = soroban_sdk::Env::default();
        let team = UserFixtureFactory::create_healthcare_team(&env);
        assert_eq!(team.doctors.len(), 3);
        assert_eq!(team.patients.len(), 5);
        assert!(team.all_users().len() > 0);
    }

    #[test]
    fn test_scenario_fixture() {
        let scenario = scenarios::patient_record_creation();
        assert_eq!(scenario.name, "Patient Creates Medical Record");
        assert!(scenario.preconditions.len() > 0);
        assert!(scenario.expected_outcomes.len() > 0);
    }
}
