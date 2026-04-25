use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    InvalidPatient = 4,
    InvalidProvider = 5,
    InvalidAmount = 6,
    AppointmentNotFound = 7,
    AppointmentAlreadyConfirmed = 8,
    AppointmentAlreadyRefunded = 9,
    InsufficientFunds = 10,
    TokenTransferFailed = 11,
    InvalidState = 12,
    DoubleWithdrawal = 13,
    OnlyPatientCanRefund = 14,
    OnlyProviderCanConfirm = 15,
}
