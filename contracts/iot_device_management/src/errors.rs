use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Authorization (100–199) ---
    Unauthorized = 100,
    NotAdmin = 102,
    NotDeviceOperator = 115,
    NotManufacturer = 116,

    // --- Input Validation (200–299) ---
    InputTooLong = 201,
    InputTooShort = 202,
    InvalidDeviceType = 240,
    InvalidFirmwareHash = 250,
    InvalidMetricValue = 260,
    InvalidTimestamp = 270,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    NotPaused = 303,

    // --- Entity Existence (400–499) ---
    DeviceNotFound = 405,
    DeviceAlreadyRegistered = 420,
    ManufacturerNotRegistered = 425,
    ManufacturerAlreadyRegistered = 426,
    FirmwareVersionNotFound = 430,
    FirmwareAlreadyExists = 431,
    ChannelNotFound = 440,

    // --- Cryptography (600–699) ---
    InvalidEncryptionKey = 602,
    KeyRotationTooFrequent = 603,

    // --- Domain-Specific: IoT (800–899) ---
    DeviceDecommissioned = 820,
    FirmwareNotApproved = 821,
    HeartbeatTooFrequent = 822,
    DeviceNotActive = 823,
    DeviceSuspended = 824,
    DowngradeNotAllowed = 825,
    DeviceOffline = 826,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotAdmin => write!(f, "not admin"),
            Error::NotDeviceOperator => write!(f, "not device operator"),
            Error::NotManufacturer => write!(f, "not manufacturer"),
            Error::InputTooLong => write!(f, "input too long"),
            Error::InputTooShort => write!(f, "input too short"),
            Error::InvalidDeviceType => write!(f, "invalid device type"),
            Error::InvalidFirmwareHash => write!(f, "invalid firmware hash"),
            Error::InvalidMetricValue => write!(f, "invalid metric value"),
            Error::InvalidTimestamp => write!(f, "invalid timestamp"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::NotPaused => write!(f, "not paused"),
            Error::DeviceNotFound => write!(f, "device not found"),
            Error::DeviceAlreadyRegistered => write!(f, "device already registered"),
            Error::ManufacturerNotRegistered => write!(f, "manufacturer not registered"),
            Error::ManufacturerAlreadyRegistered => write!(f, "manufacturer already registered"),
            Error::FirmwareVersionNotFound => write!(f, "firmware version not found"),
            Error::FirmwareAlreadyExists => write!(f, "firmware already exists"),
            Error::ChannelNotFound => write!(f, "channel not found"),
            Error::InvalidEncryptionKey => write!(f, "invalid encryption key"),
            Error::KeyRotationTooFrequent => write!(f, "key rotation too frequent"),
            Error::DeviceDecommissioned => write!(f, "device decommissioned"),
            Error::FirmwareNotApproved => write!(f, "firmware not approved"),
            Error::HeartbeatTooFrequent => write!(f, "heartbeat too frequent"),
            Error::DeviceNotActive => write!(f, "device not active"),
            Error::DeviceSuspended => write!(f, "device suspended"),
            Error::DowngradeNotAllowed => write!(f, "downgrade not allowed"),
            Error::DeviceOffline => write!(f, "device offline"),
        }
    }
}


#[cfg(test)]
pub fn get_suggestion(error: Error) -> soroban_sdk::soroban_sdk::Symbol {
    match error {
        Error::Unauthorized
        | Error::NotAdmin
        | Error::NotDeviceOperator
        | Error::NotManufacturer => {
            soroban_sdk::soroban_sdk::symbol_short!("CHK_AUTH")
        },
        Error::NotInitialized => soroban_sdk::symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized
        | Error::DeviceAlreadyRegistered
        | Error::ManufacturerAlreadyRegistered
        | Error::FirmwareAlreadyExists => {
            soroban_sdk::soroban_sdk::symbol_short!("ALREADY")
        },
        Error::ContractPaused | Error::HeartbeatTooFrequent | Error::KeyRotationTooFrequent => {
            soroban_sdk::symbol_short!("RE_TRY_L")
        },
        Error::InputTooLong | Error::InputTooShort => soroban_sdk::symbol_short!("CHK_LEN"),
        Error::DeviceNotFound
        | Error::ManufacturerNotRegistered
        | Error::FirmwareVersionNotFound
        | Error::ChannelNotFound => {
            soroban_sdk::soroban_sdk::symbol_short!("CHK_ID")
        },
        _ => soroban_sdk::soroban_sdk::symbol_short!("CONTACT"),
    }
}
