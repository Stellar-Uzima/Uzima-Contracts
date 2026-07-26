use soroban_sdk::Symbol;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StandardError {
    Unauthorized = 100,
    InvalidInput = 200,
    NotFound = 300,
    AlreadyExists = 301,
    InsufficientBalance = 500,
    InternalError = 900,
}

impl From<StandardError> for Symbol {
    fn from(e: StandardError) -> Self {
        match e {
            StandardError::Unauthorized => Symbol::new(&[&b"UNAUTHORIZED"[..]]),
            StandardError::InvalidInput => Symbol::new(&[&b"INVALID_INPUT"[..]]),
            StandardError::NotFound => Symbol::new(&[&b"NOT_FOUND"[..]]),
            StandardError::AlreadyExists => Symbol::new(&[&b"ALREADY_EXISTS"[..]]),
            StandardError::InsufficientBalance => Symbol::new(&[&b"INSUFFICIENT_BALANCE"[..]]),
            StandardError::InternalError => Symbol::new(&[&b"INTERNAL_ERROR"[..]]),
        }
    }
}