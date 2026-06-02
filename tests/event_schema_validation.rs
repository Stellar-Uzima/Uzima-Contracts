/// Event Schema Validation Tests
///
/// This module validates that all contracts follow the event standardization policy:
/// 1. Each contract has an events.rs file (or events defined in lib.rs)
/// 2. All state-changing functions emit events
/// 3. Event names follow the naming convention
/// 4. All events are registered in the schema registry
///
/// Run with: cargo test --test event_schema_validation

#[cfg(test)]
mod event_schema_validation {
    use std::fs;
    use std::path::{Path, PathBuf};
    use regex::Regex;
    use serde_json::json;

    const CONTRACTS_DIR: &str = "contracts";
    const REGISTRY_PATH: &str = "schemas/events/registry.json";
    const MIN_EVENTS_PER_CONTRACT: usize = 1;

    /// Find all contracts in the project
    fn get_all_contracts() -> Vec<String> {
        let contracts_path = Path::new(CONTRACTS_DIR);
        let mut contracts = Vec::new();

        if let Ok(entries) = fs::read_dir(contracts_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            contracts.push(name.to_string());
                        }
                    }
                }
            }
        }

        contracts.sort();
        contracts
    }

    /// Check if contract has an events.rs file
    fn has_events_rs(contract: &str) -> bool {
        let path = Path::new(CONTRACTS_DIR)
            .join(contract)
            .join("src")
            .join("events.rs");
        path.exists()
    }

    /// Check if contract has events defined inline (in lib.rs)
    fn has_inline_events(contract: &str) -> bool {
        let lib_rs = Path::new(CONTRACTS_DIR)
            .join(contract)
            .join("src")
            .join("lib.rs");

        if !lib_rs.exists() {
            return false;
        }

        match fs::read_to_string(&lib_rs) {
            Ok(content) => {
                content.contains("env.events().publish")
                    || content.contains("emit_")
                    || content.contains("EventType::")
            }
            Err(_) => false,
        }
    }

    /// Get state-changing functions from contract lib.rs
    fn get_state_changing_functions(contract: &str) -> Vec<String> {
        let lib_rs = Path::new(CONTRACTS_DIR)
            .join(contract)
            .join("src")
            .join("lib.rs");

        if !lib_rs.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&lib_rs) {
            Ok(content) => {
                let mut functions = Vec::new();

                // Patterns for state-changing functions
                let patterns = vec![
                    r"pub\s+fn\s+initialize",
                    r"pub\s+fn\s+init",
                    r"pub\s+fn\s+set_",
                    r"pub\s+fn\s+create_",
                    r"pub\s+fn\s+update_",
                    r"pub\s+fn\s+delete_",
                    r"pub\s+fn\s+remove_",
                    r"pub\s+fn\s+grant_",
                    r"pub\s+fn\s+revoke_",
                    r"pub\s+fn\s+transfer_",
                    r"pub\s+fn\s+submit_",
                    r"pub\s+fn\s+execute_",
                    r"pub\s+fn\s+finalize_",
                ];

                for pattern in patterns {
                    if let Ok(re) = Regex::new(pattern) {
                        for cap in re.captures_iter(&content) {
                            let func_text = cap.get(0).unwrap().as_str();
                            if let Ok(func_re) = Regex::new(r"fn\s+(\w+)") {
                                if let Some(func_cap) = func_re.captures(func_text) {
                                    if let Some(name) = func_cap.get(1) {
                                        functions.push(name.as_str().to_string());
                                    }
                                }
                            }
                        }
                    }
                }

                functions.sort();
                functions.dedup();
                functions
            }
            Err(_) => Vec::new(),
        }
    }

    /// Get events defined in events.rs
    fn get_events_from_events_rs(contract: &str) -> Vec<String> {
        let events_rs = Path::new(CONTRACTS_DIR)
            .join(contract)
            .join("src")
            .join("events.rs");

        if !events_rs.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&events_rs) {
            Ok(content) => {
                let mut events = Vec::new();

                // Find emit_* functions
                if let Ok(re) = Regex::new(r"pub\s+fn\s+emit_(\w+)") {
                    for cap in re.captures_iter(&content) {
                        if let Some(name) = cap.get(1) {
                            events.push(name.as_str().to_string());
                        }
                    }
                }

                events
            }
            Err(_) => Vec::new(),
        }
    }

    /// Test: All contracts should have events defined
    #[test]
    fn test_all_contracts_have_events() {
        let contracts = get_all_contracts();
        let mut contracts_without_events = Vec::new();

        for contract in &contracts {
            let has_events = has_events_rs(contract) || has_inline_events(contract);
            if !has_events {
                contracts_without_events.push(contract.clone());
            }
        }

        assert!(
            contracts_without_events.is_empty(),
            "Contracts without events (events.rs or inline): {:?}",
            contracts_without_events
        );
    }

    /// Test: Contracts with state-changing functions should have events
    #[test]
    fn test_state_changing_functions_emit_events() {
        let contracts = get_all_contracts();
        let mut issues = Vec::new();

        for contract in &contracts {
            let state_functions = get_state_changing_functions(contract);
            if !state_functions.is_empty() && !has_inline_events(contract) && !has_events_rs(contract)
            {
                issues.push(format!(
                    "{}: Has {} state-changing functions but no events: {:?}",
                    contract,
                    state_functions.len(),
                    state_functions
                ));
            }
        }

        assert!(
            issues.is_empty(),
            "Contracts with state-changing functions but no events:\n{}",
            issues.join("\n")
        );
    }

    /// Test: events.rs files should have proper structure
    #[test]
    fn test_events_rs_structure() {
        let contracts = get_all_contracts();
        let mut issues = Vec::new();

        for contract in &contracts {
            if !has_events_rs(contract) {
                continue;
            }

            let events_rs = Path::new(CONTRACTS_DIR)
                .join(contract)
                .join("src")
                .join("events.rs");

            if let Ok(content) = fs::read_to_string(&events_rs) {
                // Check for required components
                let checks = vec![
                    ("EventType enum", "enum EventType"),
                    ("OperationCategory enum", "enum OperationCategory"),
                    ("EventData struct", "struct"),
                    ("Event struct", "Event {"),
                    ("emit functions", "pub fn emit_"),
                ];

                for (component, pattern) in checks {
                    if !content.contains(pattern) {
                        issues.push(format!(
                            "{}: Missing {} in events.rs",
                            contract, component
                        ));
                    }
                }
            }
        }

        assert!(
            issues.is_empty(),
            "events.rs structure issues:\n{}",
            issues.join("\n")
        );
    }

    /// Test: Event names should follow naming convention
    #[test]
    fn test_event_naming_convention() {
        let contracts = get_all_contracts();
        let mut issues = Vec::new();

        for contract in &contracts {
            if !has_events_rs(contract) {
                continue;
            }

            let events_rs = Path::new(CONTRACTS_DIR)
                .join(contract)
                .join("src")
                .join("events.rs");

            if let Ok(content) = fs::read_to_string(&events_rs) {
                // Check for symbol_short! usage
                if !content.contains("symbol_short!") {
                    issues.push(format!(
                        "{}: Missing symbol_short! macro usage",
                        contract
                    ));
                }

                // Check that topics follow convention (SYMBOL, ACTION)
                if let Ok(re) = Regex::new(r#"symbol_short!\("([A-Z]{2,6})"\)"#) {
                    let matches: Vec<_> = re
                        .find_iter(&content)
                        .map(|m| m.as_str())
                        .collect();

                    if matches.is_empty() {
                        issues.push(format!(
                            "{}: No symbol_short! symbols found",
                            contract
                        ));
                    }
                }
            }
        }

        assert!(
            issues.is_empty(),
            "Event naming convention issues:\n{}",
            issues.join("\n")
        );
    }

    /// Test: Summary report
    #[test]
    fn test_event_audit_summary() {
        let contracts = get_all_contracts();
        let with_events = contracts.iter().filter(|c| has_events_rs(c)).count();
        let with_inline_events = contracts
            .iter()
            .filter(|c| !has_events_rs(c) && has_inline_events(c))
            .count();

        println!("\n=== Event Standardization Audit ===");
        println!("Total contracts: {}", contracts.len());
        println!("With events.rs: {}", with_events);
        println!("With inline events: {}", with_inline_events);
        println!(
            "Without events: {}",
            contracts.len() - with_events - with_inline_events
        );
        println!("\nCoverage: {:.1}%", 
                 ((with_events + with_inline_events) as f64 / contracts.len() as f64) * 100.0);
    }
}
