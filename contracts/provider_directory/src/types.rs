use soroban_sdk::{contracterror, contracttype, Address, String, Symbol, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    ProfileNotFound = 4,
    ProfileAlreadyExists = 5,
    InvalidSpecialty = 6,
    InvalidAvailability = 7,
    NotVerified = 8,
    ContractPaused = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderProfile {
    pub address: Address,
    pub name: String,
    pub specialties: Vec<Symbol>,
    pub bio: String,
    pub location: String,
    pub contact_info: String,
    pub is_verified: bool,
    pub reputation_score: u32,
    pub joining_timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Availability {
    pub day_of_week: u32, // 1-7 for Mon-Sun
    pub start_hour: u32,
    pub end_hour: u32,
    pub timezone: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Referral {
    pub from_provider: Address,
    pub to_provider: Address,
    pub patient: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PrivacySettings {
    pub show_contact_info: bool,
    pub show_location: bool,
    pub allow_referrals: bool,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Paused,
    IdentityRegistry,
    Profile(Address),
    Availability(Address),
    Privacy(Address),
    ProviderList, // Vector of addresses for discovery
}
