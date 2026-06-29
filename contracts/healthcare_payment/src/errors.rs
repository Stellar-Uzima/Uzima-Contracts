#![no_std]
use soroban_sdk::{contracterror, contracttype};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    ClaimNotFound = 5,
    PreAuthNotFound = 6,
    PaymentPlanNotFound = 7,
    InsuranceProviderNotFound = 8,
    CoveragePolicyNotFound = 9,
    InvalidCoverage = 10,
    UnsupportedTransaction = 11,
    Reentrancy = 12,
    CircuitOpen = 13,
    Arithmetic = 14,
}