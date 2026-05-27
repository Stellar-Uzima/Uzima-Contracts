/// Shared error type for contract utilities used in tests and examples
use alloc::string::String;
use core::fmt;

/// A small, test-oriented contract error type.
#[derive(Clone, PartialEq, Eq)]
pub enum ContractError {
    Code(i32),
    Message(String),
}

impl From<i32> for ContractError {
    fn from(code: i32) -> Self {
        ContractError::Code(code)
    }
}

impl From<String> for ContractError {
    fn from(s: String) -> Self {
        ContractError::Message(s)
    }
}

impl From<&str> for ContractError {
    fn from(s: &str) -> Self {
        ContractError::Message(s.to_string())
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::Code(c) => write!(f, "ContractError::Code({})", c),
            ContractError::Message(s) => write!(f, "ContractError::Message({})", s),
        }
    }
}

impl fmt::Debug for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for ContractError {}

impl ContractError {
    /// If the error was numeric, return the code.
    pub fn code(&self) -> Option<i32> {
        match self {
            ContractError::Code(c) => Some(*c),
            _ => None,
        }
    }
}
