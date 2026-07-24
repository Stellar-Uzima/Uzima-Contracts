#![no_std]
#![forbid(alloc)]
//! token_sale - Healthcare smart contract on Stellar blockchain.

mod contract;
mod errors;
mod storage;
mod types;
mod vesting;

#[cfg(test)]
mod test;

#[cfg(test)]
mod invariant_tests;

pub use contract::{TokenSaleContract, TokenSaleContractClient};
pub use errors::Error;
pub use vesting::{VestingContract, VestingContractClient};
