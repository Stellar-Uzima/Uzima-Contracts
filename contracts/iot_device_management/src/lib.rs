#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::used_underscore_binding)]

mod errors;
pub use errors::IoTError;

use soroban_sdk::{contract, Env};

#[contract]
pub struct IoTDeviceManagement;
