use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IoTError {
    // System (1-9)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractPaused = 3,
    NotPaused = 4,

    // Auth (10-19)
    NotAdmin = 10,
    NotAuthorized = 11,
    NotDeviceOperator = 12,
    NotManufacturer = 13,

    // Device (20-39)
    DeviceAlreadyRegistered = 20,
    DeviceNotFound = 21,
    DeviceNotActive = 22,
    DeviceDecommissioned = 23,
    DeviceSuspended = 24,
    InvalidDeviceType = 25,
    InvalidDeviceId = 26,
    ManufacturerNotRegistered = 27,
    ManufacturerAlreadyRegistered = 28,

    // Firmware (40-49)
    FirmwareVersionNotFound = 40,
    FirmwareAlreadyExists = 41,
    FirmwareNotApproved = 42,
    InvalidFirmwareHash = 43,
    DowngradeNotAllowed = 44,

    // Health (50-59)
    HeartbeatTooFrequent = 50,
    InvalidMetricValue = 51,
    DeviceOffline = 52,

    // Communication (60-69)
    InvalidEncryptionKey = 60,
    KeyRotationTooFrequent = 61,
    ChannelNotFound = 62,

    // Validation (70-79)
    StringTooLong = 70,
    StringTooShort = 71,
    InvalidTimestamp = 72,
}
