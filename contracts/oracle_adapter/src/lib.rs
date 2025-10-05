#![no_std]

pub mod contract;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use contract::OracleAdapterContract;
