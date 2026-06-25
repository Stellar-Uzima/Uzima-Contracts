use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Error {
    /// Contract has not been initialized yet.
    NotInitialized = 1,
    /// Contract has already been initialized.
    AlreadyInitialized = 2,
    /// Caller is not authorized to perform this action.
    Unauthorized = 3,
    /// A string or bytes input exceeded the maximum allowed length.
    InputTooLong = 4,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::Unauthorized => "Unauthorized",
            Error::InputTooLong => "Input Too Long",
        };
        f.write_str(message)
    }
}
