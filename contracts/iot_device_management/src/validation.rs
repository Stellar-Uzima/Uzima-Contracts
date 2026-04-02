use soroban_sdk::String;

use crate::IoTError;

const MAX_STRING_LEN: u32 = 256;
const MIN_STRING_LEN: u32 = 1;
const MAX_LOCATION_LEN: u32 = 512;
const MAX_MODEL_LEN: u32 = 128;

pub fn validate_string(s: &String, min: u32, max: u32) -> Result<(), IoTError> {
    let len = s.len();
    if len < min {
        return Err(IoTError::StringTooShort);
    }
    if len > max {
        return Err(IoTError::StringTooLong);
    }
    Ok(())
}

pub fn validate_name(s: &String) -> Result<(), IoTError> {
    validate_string(s, MIN_STRING_LEN, MAX_STRING_LEN)
}

pub fn validate_model(s: &String) -> Result<(), IoTError> {
    validate_string(s, MIN_STRING_LEN, MAX_MODEL_LEN)
}

pub fn validate_serial(s: &String) -> Result<(), IoTError> {
    validate_string(s, MIN_STRING_LEN, MAX_STRING_LEN)
}

pub fn validate_location(s: &String) -> Result<(), IoTError> {
    validate_string(s, 0, MAX_LOCATION_LEN)
}

pub fn validate_metric_value(value: u32, max: u32) -> Result<(), IoTError> {
    if value > max {
        return Err(IoTError::InvalidMetricValue);
    }
    Ok(())
}
