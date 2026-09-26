use soroban_sdk::String;

use crate::Error;

const MAX_STRING_LEN: u32 = 256;
const MIN_STRING_LEN: u32 = 1;
const MAX_LOCATION_LEN: u32 = 512;
const MAX_MODEL_LEN: u32 = 128;
const MAX_STALE_SWEEP_BATCH: u32 = 20;

#[must_use]
pub fn validate_string(s: &String, min: u32, max: u32) -> Result<(), Error> {
    let len = s.len();
    if len < min {
        return Err(Error::InputTooShort);
    }
    if len > max {
        return Err(Error::InputTooLong);
    }
    Ok(())
}

#[must_use]
pub fn validate_name(s: &String) -> Result<(), Error> {
    validate_string(s, MIN_STRING_LEN, MAX_STRING_LEN)
}

#[must_use]
pub fn validate_model(s: &String) -> Result<(), Error> {
    validate_string(s, MIN_STRING_LEN, MAX_MODEL_LEN)
}

#[must_use]
pub fn validate_serial(s: &String) -> Result<(), Error> {
    validate_string(s, MIN_STRING_LEN, MAX_STRING_LEN)
}

#[must_use]
pub fn validate_location(s: &String) -> Result<(), Error> {
    validate_string(s, 0, MAX_LOCATION_LEN)
}

#[must_use]
pub fn validate_metric_value(value: u32, max: u32) -> Result<(), Error> {
    if value > max {
        return Err(Error::InvalidMetricValue);
    }
    Ok(())
}

/// Stale-device sweeps are caller-bounded so a single invocation can never
/// walk an unbounded number of devices in one ledger transaction.
#[must_use]
pub fn validate_sweep_batch(len: u32) -> Result<(), Error> {
    if len > MAX_STALE_SWEEP_BATCH {
        return Err(Error::StaleSweepTooLarge);
    }
    Ok(())
}
