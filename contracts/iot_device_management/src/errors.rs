use soroban_sdk::{contracterror, symbol_short, Symbol};

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::NotAdmin => "Not Admin",
            Error::NotDeviceOperator => "Not Device Operator",
            Error::NotManufacturer => "Not Manufacturer",
            Error::InputTooLong => "Input Too Long",
            Error::InputTooShort => "Input Too Short",
            Error::InvalidDeviceType => "Invalid Device Type",
            Error::InvalidFirmwareHash => "Invalid Firmware Hash",
            Error::InvalidMetricValue => "Invalid Metric Value",
            Error::InvalidTimestamp => "Invalid Timestamp",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::ContractPaused => "Contract Paused",
            Error::NotPaused => "Not Paused",
            Error::DeviceNotFound => "Device Not Found",
            Error::DeviceAlreadyRegistered => "Device Already Registered",
            Error::ManufacturerNotRegistered => "Manufacturer Not Registered",
            Error::ManufacturerAlreadyRegistered => "Manufacturer Already Registered",
            Error::FirmwareVersionNotFound => "Firmware Version Not Found",
            Error::FirmwareAlreadyExists => "Firmware Already Exists",
            Error::ChannelNotFound => "Channel Not Found",
            Error::InvalidEncryptionKey => "Invalid Encryption Key",
            Error::KeyRotationTooFrequent => "Key Rotation Too Frequent",
            Error::DeviceDecommissioned => "Device Decommissioned",
            Error::FirmwareNotApproved => "Firmware Not Approved",
            Error::HeartbeatTooFrequent => "Heartbeat Too Frequent",
            Error::DeviceNotActive => "Device Not Active",
            Error::DeviceSuspended => "Device Suspended",
            Error::DowngradeNotAllowed => "Downgrade Not Allowed",
            Error::DeviceOffline => "Device Offline",
        };
        f.write_str(message)
    }
}

#[allow(dead_code)]
pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized
        | Error::NotAdmin
        | Error::NotDeviceOperator
        | Error::NotManufacturer => {
            symbol_short!("CHK_AUTH")
        },
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized
        | Error::DeviceAlreadyRegistered
        | Error::ManufacturerAlreadyRegistered
        | Error::FirmwareAlreadyExists => {
            symbol_short!("ALREADY")
        },
        Error::ContractPaused | Error::HeartbeatTooFrequent | Error::KeyRotationTooFrequent => {
            symbol_short!("RE_TRY_L")
        },
        Error::InputTooLong | Error::InputTooShort => symbol_short!("CHK_LEN"),
        Error::DeviceNotFound
        | Error::ManufacturerNotRegistered
        | Error::FirmwareVersionNotFound
        | Error::ChannelNotFound => {
            symbol_short!("CHK_ID")
        },
        _ => symbol_short!("CONTACT"),
    }
}
