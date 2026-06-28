Healthcare Compliance Automation Contract

This small helper contract stores the list of supported regulatory frameworks and provides an on-chain reference point for automation tooling.

Functions:
- initialize(admin: Address, frameworks: Vec<String>)
- add_framework(admin: Address, framework: String)
- get_supported_frameworks() -> FrameworkList

Testing:
- Build the contract: cargo build --target wasm32-unknown-unknown
- Deploy to local Soroban network with the Soroban CLI

This contract is intentionally minimal: the heavy lifting (real-time monitoring, automated evidence collection, report generation) is implemented off-chain via scripts that call into the main `healthcare_compliance` contract.
