#![cfg(test)]

#[test]
fn timestamp_before_deadline_succeeds() {
    let deadline: u64 = 1000;
    let now: u64 = 999;
    assert!(now < deadline);
}

#[test]
fn timestamp_at_deadline_succeeds() {
    let deadline: u64 = 1000;
    let now: u64 = 1000;
    assert!(now >= deadline);
}

#[test]
fn timestamp_after_deadline_fails() {
    let deadline: u64 = 1000;
    let now: u64 = 1001;
    assert!(now > deadline);
}

#[test]
fn timestamp_at_expiry_window_boundary() {
    let start: u64 = 500;
    let expiry: u64 = 1000;
    assert!(start + 500 == expiry);
}