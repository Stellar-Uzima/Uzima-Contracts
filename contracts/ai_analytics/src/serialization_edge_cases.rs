// Module preserved for compatibility. The previous contents did not
// compile against the current soroban-sdk (calls to non-existent
// `try_to_val`, `String + &String`, and `to_string` in no_std). They
// were never exercised because the crate's test profile was not being
// built. Regression coverage now lives in `test_serialization.rs`.
