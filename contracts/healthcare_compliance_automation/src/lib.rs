#![no_std]
#![forbid(alloc)]
//! healthcare_compliance_automation - Healthcare smart contract on Stellar blockchain.
//!
//! Stores the regulatory frameworks an operator supports, the policy controls
//! each framework requires, the assessed status of each control, and the
//! resulting compliance report.
//!
//! Signatures of `initialize`, `add_framework` and `get_supported_frameworks`
//! are unchanged from the original scaffold, because `npm run abi:check` fails
//! on breaking changes. The two functions that could previously only panic still
//! do, rather than returning a new `Result`; the administrative entrypoints
//! added here do return `Result`.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

#[cfg(test)]
mod test;

const ADMIN: Symbol = symbol_short!("ADMIN");
const SUPPORTED: Symbol = symbol_short!("SUPPORTED");

/// Bound on `SUPPORTED`, which lives in instance storage and is rewritten whole
/// on every `add_framework`.
const MAX_FRAMEWORKS: u32 = 32;
/// Bound on the per-framework control list in persistent storage.
const MAX_CONTROLS: u32 = 64;

#[derive(Clone)]
#[contracterror]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    NotAdmin = 2,
    FrameworkUnknown = 3,
    ControlExists = 4,
    ControlUnknown = 5,
    NoControls = 6,
    TooManyControls = 7,
    EmptyName = 8,
    ReportMissing = 9,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // No `FrameworkExists` or `TooManyFrameworks` variant: `initialize` and
    // `add_framework` predate the error type and return `()`, so a duplicate or
    // over-long framework list panics rather than returning a code.
    /// Declared controls for a framework.
    Controls(String),
    /// Latest assessed status for one control.
    Status(String, String),
    /// Latest generated report for a framework.
    Report(String),
}

#[derive(Clone)]
#[contracttype]
pub struct FrameworkList {
    pub frameworks: Vec<String>,
}

/// One required control in a framework's checklist.
#[derive(Clone)]
#[contracttype]
pub struct PolicyControl {
    pub control_id: String,
    pub title: String,
    /// Mandatory controls decide `ComplianceReport::compliant`; advisory ones
    /// are reported but do not.
    pub mandatory: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ControlList {
    pub controls: Vec<PolicyControl>,
}

/// An assessment recorded against a control.
#[derive(Clone)]
#[contracttype]
pub struct ControlStatus {
    pub satisfied: bool,
    /// Hash of the supporting evidence document, so an assessment can be
    /// referenced without putting document contents on chain.
    pub evidence: BytesN<32>,
    pub reported_at: u64,
}

/// One row of a checklist evaluation.
#[derive(Clone)]
#[contracttype]
pub struct ControlResult {
    pub control_id: String,
    pub title: String,
    pub mandatory: bool,
    pub satisfied: bool,
    /// False when no status has been recorded, which is distinct from a
    /// recorded failure.
    pub assessed: bool,
    pub reported_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ComplianceReport {
    pub framework: String,
    pub evaluated_at: u64,
    pub total: u32,
    pub satisfied: u32,
    pub violated: u32,
    /// Controls with no recorded status.
    pub unassessed: u32,
    pub mandatory_violations: u32,
    /// True when no mandatory control is violated. Unassessed mandatory
    /// controls count as violations, so an incomplete checklist cannot pass.
    pub compliant: bool,
    pub results: Vec<ControlResult>,
}

#[contract]
pub struct HealthcareComplianceAutomation;

#[contractimpl]
impl HealthcareComplianceAutomation {
    /// Store the admin and the initially supported frameworks.
    ///
    /// Panics on a repeated or empty framework name, or when `frameworks`
    /// exceeds `MAX_FRAMEWORKS`.
    pub fn initialize(env: Env, admin: Address, frameworks: Vec<String>) {
        governance_commons::init_guard(&env);
        admin.require_auth();

        if frameworks.len() > MAX_FRAMEWORKS {
            panic!("too many frameworks");
        }

        let mut seen: Vec<String> = Vec::new(&env);
        let mut i: u32 = 0;
        while i < frameworks.len() {
            let framework = frameworks.get(i).unwrap();
            if framework.is_empty() {
                panic!("empty framework name");
            }
            if contains(&seen, &framework) {
                panic!("duplicate framework");
            }
            seen.push_back(framework);
            i += 1;
        }

        env.storage().instance().set(&ADMIN, &admin);
        let list = FrameworkList { frameworks };
        env.storage().instance().set(&SUPPORTED, &list);

        // Keep the publish call on one line: scripts/events/scan.mjs matches the
        // emitter as a literal and does not strip comments, so breaking it across
        // lines drops the event from docs/EVENTS.md and the schema registry, and
        // a prose mention of the emitter is parsed as a phantom event. Both make
        // `npm run events:validate` fail on drift.
        env.events().publish(
            (symbol_short!("hca"), symbol_short!("init")),
            admin,
        );
    }

    /// Add a framework to the supported list.
    ///
    /// Panics if the caller is not the admin, or on a repeated or empty name.
    /// Previously the `admin` argument was only authenticated, never checked
    /// against the stored admin, so any address could extend the list.
    pub fn add_framework(env: Env, admin: Address, framework: String) {
        admin.require_auth();
        // This entrypoint returns `()` for ABI compatibility, so the check has
        // to panic rather than propagate. Discarding the Result here would let
        // any authenticated caller through, which is the bug being fixed.
        match Self::require_admin(&env, &admin) {
            Ok(()) => {}
            Err(Error::NotInitialized) => panic!("contract not initialized"),
            Err(_) => panic!("caller is not the admin"),
        }

        if framework.is_empty() {
            panic!("empty framework name");
        }

        let mut list: FrameworkList = env
            .storage()
            .instance()
            .get(&SUPPORTED)
            .unwrap_or(FrameworkList {
                frameworks: Vec::new(&env),
            });

        if contains(&list.frameworks, &framework) {
            panic!("framework already supported");
        }
        if list.frameworks.len() >= MAX_FRAMEWORKS {
            panic!("too many frameworks");
        }

        list.frameworks.push_back(framework.clone());
        env.storage().instance().set(&SUPPORTED, &list);

        env.events().publish(
            (symbol_short!("hca"), symbol_short!("fw_add")),
            framework,
        );
    }

    pub fn get_supported_frameworks(env: Env) -> FrameworkList {
        env.storage()
            .instance()
            .get(&SUPPORTED)
            .unwrap_or(FrameworkList {
                frameworks: Vec::new(&env),
            })
    }

    /// Declare a control that `framework` requires.
    pub fn add_control(
        env: Env,
        admin: Address,
        framework: String,
        control: PolicyControl,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_framework(&env, &framework)?;

        if control.control_id.is_empty() || control.title.is_empty() {
            return Err(Error::EmptyName);
        }

        let mut list = Self::load_controls(&env, &framework);
        if list.controls.len() >= MAX_CONTROLS {
            return Err(Error::TooManyControls);
        }

        let mut i: u32 = 0;
        while i < list.controls.len() {
            if list.controls.get(i).unwrap().control_id == control.control_id {
                return Err(Error::ControlExists);
            }
            i += 1;
        }

        let control_id = control.control_id.clone();
        list.controls.push_back(control);
        env.storage()
            .persistent()
            .set(&DataKey::Controls(framework.clone()), &list);

        env.events().publish(
            (symbol_short!("hca"), symbol_short!("ctl_add")),
            (framework, control_id),
        );
        Ok(())
    }

    /// The declared checklist for a framework, in declaration order.
    pub fn get_controls(env: Env, framework: String) -> Result<ControlList, Error> {
        Self::require_initialized(&env)?;
        Self::require_framework(&env, &framework)?;
        Ok(Self::load_controls(&env, &framework))
    }

    /// Record an assessment for one control. Re-recording overwrites.
    pub fn record_control_status(
        env: Env,
        admin: Address,
        framework: String,
        control_id: String,
        satisfied: bool,
        evidence: BytesN<32>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_framework(&env, &framework)?;

        let list = Self::load_controls(&env, &framework);
        if !has_control(&list, &control_id) {
            return Err(Error::ControlUnknown);
        }

        let status = ControlStatus {
            satisfied,
            evidence,
            reported_at: env.ledger().timestamp(),
        };
        env.storage().persistent().set(
            &DataKey::Status(framework.clone(), control_id.clone()),
            &status,
        );

        env.events().publish(
            (symbol_short!("hca"), symbol_short!("ctl_rec")),
            (framework, control_id, satisfied),
        );
        Ok(())
    }

    /// Evaluate a framework's checklist and persist the report.
    ///
    /// A mandatory control that has no recorded status counts as a violation, so
    /// a partially assessed checklist cannot report as compliant.
    pub fn evaluate_framework(
        env: Env,
        admin: Address,
        framework: String,
    ) -> Result<ComplianceReport, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_framework(&env, &framework)?;

        let list = Self::load_controls(&env, &framework);
        if list.controls.is_empty() {
            return Err(Error::NoControls);
        }

        let mut results: Vec<ControlResult> = Vec::new(&env);
        let mut satisfied: u32 = 0;
        let mut violated: u32 = 0;
        let mut unassessed: u32 = 0;
        let mut mandatory_violations: u32 = 0;

        let mut i: u32 = 0;
        while i < list.controls.len() {
            let control = list.controls.get(i).unwrap();
            let key = DataKey::Status(framework.clone(), control.control_id.clone());
            let status: Option<ControlStatus> = env.storage().persistent().get(&key);

            let (is_satisfied, assessed, reported_at) = match status {
                Some(s) => {
                    if s.satisfied {
                        satisfied += 1;
                    } else {
                        violated += 1;
                    }
                    (s.satisfied, true, s.reported_at)
                }
                None => {
                    unassessed += 1;
                    (false, false, 0)
                }
            };

            if control.mandatory && !is_satisfied {
                mandatory_violations += 1;
            }

            results.push_back(ControlResult {
                control_id: control.control_id,
                title: control.title,
                mandatory: control.mandatory,
                satisfied: is_satisfied,
                assessed,
                reported_at,
            });
            i += 1;
        }

        let report = ComplianceReport {
            framework: framework.clone(),
            evaluated_at: env.ledger().timestamp(),
            total: list.controls.len(),
            satisfied,
            violated,
            unassessed,
            mandatory_violations,
            compliant: mandatory_violations == 0,
            results,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Report(framework.clone()), &report);

        env.events().publish(
            (symbol_short!("hca"), symbol_short!("eval")),
            (framework, report.compliant, mandatory_violations),
        );
        Ok(report)
    }

    /// The most recent report for a framework, or `ReportMissing`.
    pub fn get_last_report(env: Env, framework: String) -> Result<ComplianceReport, Error> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .get(&DataKey::Report(framework))
            .ok_or(Error::ReportMissing)
    }

    // ─── Internals ──────────────────────────────────────────────────────────

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&ADMIN) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        if *caller == admin {
            Ok(())
        } else {
            Err(Error::NotAdmin)
        }
    }

    fn require_framework(env: &Env, framework: &String) -> Result<(), Error> {
        let list: FrameworkList = env
            .storage()
            .instance()
            .get(&SUPPORTED)
            .unwrap_or(FrameworkList {
                frameworks: Vec::new(env),
            });
        if contains(&list.frameworks, framework) {
            Ok(())
        } else {
            Err(Error::FrameworkUnknown)
        }
    }

    fn load_controls(env: &Env, framework: &String) -> ControlList {
        env.storage()
            .persistent()
            .get(&DataKey::Controls(framework.clone()))
            .unwrap_or(ControlList {
                controls: Vec::new(env),
            })
    }
}

/// Membership test over a soroban `Vec<String>`, which has no `contains`.
fn contains(list: &Vec<String>, needle: &String) -> bool {
    let mut i: u32 = 0;
    while i < list.len() {
        if list.get(i).unwrap() == *needle {
            return true;
        }
        i += 1;
    }
    false
}

fn has_control(list: &ControlList, control_id: &String) -> bool {
    let mut i: u32 = 0;
    while i < list.controls.len() {
        if list.controls.get(i).unwrap().control_id == *control_id {
            return true;
        }
        i += 1;
    }
    false
}
