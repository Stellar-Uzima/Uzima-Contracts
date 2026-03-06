use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    UserNotRegistered = 4,
    InvalidInput = 5,
    InsufficientPermissions = 6,
    EmergencyAccessRequired = 7,
    PrivacyViolation = 8,
    CrisisDetected = 9,
    SessionNotFound = 10,
    AssessmentNotFound = 11,
    MedicationPlanNotFound = 12,
    GroupNotFound = 13,
    ProfessionalNotFound = 14,
    ProgramNotFound = 15,
    DatasetNotFound = 16,
    QueryNotApproved = 17,
    RiskThresholdExceeded = 18,
    ConsentRequired = 19,
    DataAnonymizationFailed = 20,
}