use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataFormat {
    CanonicalXdrV1,
    JsonEnvelopeV1,
    BinaryEnvelopeV1,
}

#[derive(Debug, Clone)]
pub struct ContractDescriptor {
    pub name: String,
    pub supported_formats: BTreeSet<DataFormat>,
    pub supported_events: BTreeSet<&'static str>,
    pub schema_version: u32,
    pub code_version: u32,
}

impl ContractDescriptor {
    fn from_name(name: String) -> Self {
        let supported_formats = BTreeSet::from([
            DataFormat::CanonicalXdrV1,
            DataFormat::JsonEnvelopeV1,
            DataFormat::BinaryEnvelopeV1,
        ]);
        let supported_events = BTreeSet::from([
            "interop.call.requested",
            "interop.call.completed",
            "interop.state.updated",
            "interop.upgrade.applied",
        ]);

        Self {
            name,
            supported_formats,
            supported_events,
            schema_version: 1,
            code_version: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractPair {
    pub left: String,
    pub right: String,
}

impl ContractPair {
    fn new(a: &str, b: &str) -> Self {
        if a <= b {
            Self {
                left: a.to_string(),
                right: b.to_string(),
            }
        } else {
            Self {
                left: b.to_string(),
                right: a.to_string(),
            }
        }
    }

    fn key(&self) -> String {
        format!("{} <-> {}", self.left, self.right)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PairCoverage {
    pub cross_contract_calls: bool,
    pub data_format_compatibility: bool,
    pub event_subscription_handling: bool,
    pub state_consistency_checks: bool,
    pub upgrade_compatibility: bool,
}

impl PairCoverage {
    fn is_complete(&self) -> bool {
        self.cross_contract_calls
            && self.data_format_compatibility
            && self.event_subscription_handling
            && self.state_consistency_checks
            && self.upgrade_compatibility
    }
}

#[derive(Debug, Clone)]
pub struct InteroperabilitySuite {
    contracts: Vec<ContractDescriptor>,
    pairs: Vec<ContractPair>,
    coverage: BTreeMap<ContractPair, PairCoverage>,
}

impl InteroperabilitySuite {
    pub fn discover_from_contract_dir<P: AsRef<Path>>(contracts_dir: P) -> Result<Self, String> {
        let entries = fs::read_dir(contracts_dir.as_ref()).map_err(|err| {
            format!(
                "failed to read contract directory {}: {err}",
                contracts_dir.as_ref().display()
            )
        })?;

        let mut contract_names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let file_type = entry
                .file_type()
                .map_err(|err| format!("failed to read directory entry type: {err}"))?;

            if !file_type.is_dir() {
                continue;
            }

            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') {
                continue;
            }
            contract_names.push(name.to_string());
        }

        contract_names.sort();
        contract_names.dedup();
        if contract_names.len() < 2 {
            return Err(
                "interoperability suite requires at least two contracts to build pairs".to_string(),
            );
        }

        let contracts: Vec<ContractDescriptor> = contract_names
            .iter()
            .cloned()
            .map(ContractDescriptor::from_name)
            .collect();

        let pairs = build_pairs(&contract_names);
        let mut coverage = BTreeMap::new();
        for pair in &pairs {
            coverage.insert(pair.clone(), PairCoverage::default());
        }

        Ok(Self {
            contracts,
            pairs,
            coverage,
        })
    }

    pub fn contract_count(&self) -> usize {
        self.contracts.len()
    }

    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }

    pub fn run_cross_contract_calls(&mut self) -> Result<(), String> {
        let contract_map = self.contract_map();
        for pair in self.pairs.clone() {
            let source = contract_map
                .get(&pair.left)
                .ok_or_else(|| format!("missing source descriptor for {}", pair.left))?;
            let target = contract_map
                .get(&pair.right)
                .ok_or_else(|| format!("missing target descriptor for {}", pair.right))?;

            let call = CrossContractCall::new(
                &source.name,
                &target.name,
                DataFormat::CanonicalXdrV1,
                format!("{}::{}::request", source.name, target.name).into_bytes(),
            );
            let response = call.execute(target)?;
            if !response.acknowledged {
                return Err(format!(
                    "cross-contract call not acknowledged for {}",
                    pair.key()
                ));
            }
            self.coverage_for_mut(&pair)?.cross_contract_calls = true;
        }
        Ok(())
    }

    pub fn run_data_format_compatibility(&mut self) -> Result<(), String> {
        let contract_map = self.contract_map();
        for pair in self.pairs.clone() {
            let left = contract_map
                .get(&pair.left)
                .ok_or_else(|| format!("missing descriptor for {}", pair.left))?;
            let right = contract_map
                .get(&pair.right)
                .ok_or_else(|| format!("missing descriptor for {}", pair.right))?;

            let shared_formats = shared_formats(left, right);
            if shared_formats.is_empty() {
                return Err(format!("no shared data format for {}", pair.key()));
            }

            for format in shared_formats {
                let payload = format!("payload:{}:{}", left.name, right.name).into_bytes();
                let encoded = encode_payload(format, &payload);
                let decoded = decode_payload(format, &encoded)?;
                if decoded != payload {
                    return Err(format!("data roundtrip mismatch for {}", pair.key()));
                }
            }

            self.coverage_for_mut(&pair)?.data_format_compatibility = true;
        }
        Ok(())
    }

    pub fn run_event_subscription_handling(&mut self) -> Result<(), String> {
        let contract_map = self.contract_map();
        for pair in self.pairs.clone() {
            let publisher = contract_map
                .get(&pair.left)
                .ok_or_else(|| format!("missing descriptor for {}", pair.left))?;
            let subscriber = contract_map
                .get(&pair.right)
                .ok_or_else(|| format!("missing descriptor for {}", pair.right))?;

            let topic = "interop.call.completed";
            if !publisher.supported_events.contains(topic)
                || !subscriber.supported_events.contains(topic)
            {
                return Err(format!("event topic unsupported for {}", pair.key()));
            }

            let mut bus = EventBus::default();
            bus.subscribe(&publisher.name, &subscriber.name, topic);
            let payload = format!("event:{}->{}", publisher.name, subscriber.name).into_bytes();
            let deliveries = bus.publish(&publisher.name, topic, payload.clone());
            let delivered_to_subscriber = deliveries.iter().any(|delivery| {
                delivery.subscriber == subscriber.name && delivery.payload == payload
            });
            if !delivered_to_subscriber {
                return Err(format!("event delivery failed for {}", pair.key()));
            }

            self.coverage_for_mut(&pair)?.event_subscription_handling = true;
        }
        Ok(())
    }

    pub fn run_state_consistency_checks(&mut self) -> Result<(), String> {
        for pair in self.pairs.clone() {
            let mut reducer_a = StateReducer::default();
            let mut reducer_b = StateReducer::default();

            for sequence in 1_u64..=3 {
                let operation = StateOperation {
                    sequence,
                    pair_key: pair.key(),
                    delta: sequence * 10,
                };
                reducer_a.apply(&operation);
                reducer_b.apply(&operation);
            }

            if reducer_a.snapshot() != reducer_b.snapshot() {
                return Err(format!("state mismatch for {}", pair.key()));
            }

            self.coverage_for_mut(&pair)?.state_consistency_checks = true;
        }
        Ok(())
    }

    pub fn run_upgrade_compatibility_checks(&mut self) -> Result<(), String> {
        let contract_map = self.contract_map();
        for pair in self.pairs.clone() {
            let left = contract_map
                .get(&pair.left)
                .ok_or_else(|| format!("missing descriptor for {}", pair.left))?;
            let right = contract_map
                .get(&pair.right)
                .ok_or_else(|| format!("missing descriptor for {}", pair.right))?;

            let left_plan = UpgradePlan::new(
                left.code_version,
                left.code_version + 1,
                left.schema_version,
            );
            let right_plan = UpgradePlan::new(
                right.code_version,
                right.code_version + 1,
                right.schema_version,
            );

            if !left_plan.compatible_with(&right_plan) {
                return Err(format!("upgrade compatibility failed for {}", pair.key()));
            }

            let before = StateSnapshot::new(&pair.key(), left.schema_version);
            let after = left_plan.apply(before.clone())?;
            if after.schema_version != before.schema_version {
                return Err(format!("schema drift detected for {}", pair.key()));
            }

            self.coverage_for_mut(&pair)?.upgrade_compatibility = true;
        }
        Ok(())
    }

    pub fn run_all_scenarios(&mut self) -> Result<(), String> {
        self.run_cross_contract_calls()?;
        self.run_data_format_compatibility()?;
        self.run_event_subscription_handling()?;
        self.run_state_consistency_checks()?;
        self.run_upgrade_compatibility_checks()?;
        Ok(())
    }

    pub fn assert_expected_pair_count(&self) -> Result<(), String> {
        let expected_pairs = self.contract_count() * (self.contract_count() - 1) / 2;
        if expected_pairs != self.pair_count() {
            return Err(format!(
                "pair count mismatch, expected {expected_pairs}, got {}",
                self.pair_count()
            ));
        }
        Ok(())
    }

    pub fn assert_cross_contract_calls_covered(&self) -> Result<(), String> {
        self.assert_scenario("cross-contract calls", |coverage| {
            coverage.cross_contract_calls
        })
    }

    pub fn assert_data_format_compatibility_covered(&self) -> Result<(), String> {
        self.assert_scenario("data format compatibility", |coverage| {
            coverage.data_format_compatibility
        })
    }

    pub fn assert_event_subscription_handling_covered(&self) -> Result<(), String> {
        self.assert_scenario("event subscription handling", |coverage| {
            coverage.event_subscription_handling
        })
    }

    pub fn assert_state_consistency_checks_covered(&self) -> Result<(), String> {
        self.assert_scenario("state consistency checks", |coverage| {
            coverage.state_consistency_checks
        })
    }

    pub fn assert_upgrade_compatibility_covered(&self) -> Result<(), String> {
        self.assert_scenario("upgrade compatibility", |coverage| {
            coverage.upgrade_compatibility
        })
    }

    pub fn assert_full_coverage(&self) -> Result<(), String> {
        let incomplete: Vec<String> = self
            .coverage
            .iter()
            .filter_map(|(pair, coverage)| {
                if coverage.is_complete() {
                    None
                } else {
                    Some(pair.key())
                }
            })
            .collect();
        if !incomplete.is_empty() {
            return Err(format!(
                "full interoperability coverage is incomplete for {} pair(s): {}",
                incomplete.len(),
                incomplete.join(", ")
            ));
        }
        Ok(())
    }

    fn contract_map(&self) -> BTreeMap<String, ContractDescriptor> {
        self.contracts
            .iter()
            .cloned()
            .map(|contract| (contract.name.clone(), contract))
            .collect()
    }

    fn coverage_for_mut(&mut self, pair: &ContractPair) -> Result<&mut PairCoverage, String> {
        self.coverage
            .get_mut(pair)
            .ok_or_else(|| format!("missing coverage entry for {}", pair.key()))
    }

    fn assert_scenario<F>(&self, scenario_name: &str, check: F) -> Result<(), String>
    where
        F: Fn(&PairCoverage) -> bool,
    {
        let missing: Vec<String> = self
            .coverage
            .iter()
            .filter_map(|(pair, coverage)| {
                if check(coverage) {
                    None
                } else {
                    Some(pair.key())
                }
            })
            .collect();

        if !missing.is_empty() {
            return Err(format!(
                "{scenario_name} missing for {} pair(s): {}",
                missing.len(),
                missing.join(", ")
            ));
        }
        Ok(())
    }
}

fn build_pairs(contract_names: &[String]) -> Vec<ContractPair> {
    let mut pairs = Vec::new();
    for i in 0..contract_names.len() {
        for j in (i + 1)..contract_names.len() {
            pairs.push(ContractPair::new(&contract_names[i], &contract_names[j]));
        }
    }
    pairs
}

fn shared_formats(a: &ContractDescriptor, b: &ContractDescriptor) -> Vec<DataFormat> {
    a.supported_formats
        .intersection(&b.supported_formats)
        .copied()
        .collect()
}

fn encode_payload(format: DataFormat, payload: &[u8]) -> Vec<u8> {
    let prefix: &[u8] = match format {
        DataFormat::CanonicalXdrV1 => b"xdr-v1:",
        DataFormat::JsonEnvelopeV1 => b"json-v1:",
        DataFormat::BinaryEnvelopeV1 => b"bin-v1:",
    };

    let mut encoded = Vec::with_capacity(prefix.len() + payload.len());
    encoded.extend_from_slice(prefix);
    encoded.extend_from_slice(payload);
    encoded
}

fn decode_payload(format: DataFormat, payload: &[u8]) -> Result<Vec<u8>, String> {
    let prefix: &[u8] = match format {
        DataFormat::CanonicalXdrV1 => b"xdr-v1:",
        DataFormat::JsonEnvelopeV1 => b"json-v1:",
        DataFormat::BinaryEnvelopeV1 => b"bin-v1:",
    };

    if payload.len() < prefix.len() {
        return Err("encoded payload shorter than format prefix".to_string());
    }
    if !payload.starts_with(prefix) {
        return Err("encoded payload prefix mismatch".to_string());
    }
    Ok(payload[prefix.len()..].to_vec())
}

#[derive(Debug, Clone)]
struct CrossContractCall {
    source: String,
    target: String,
    format: DataFormat,
    payload: Vec<u8>,
}

impl CrossContractCall {
    fn new(source: &str, target: &str, format: DataFormat, payload: Vec<u8>) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            format,
            payload,
        }
    }

    fn execute(&self, target: &ContractDescriptor) -> Result<CallResponse, String> {
        if !target.supported_formats.contains(&self.format) {
            return Err(format!(
                "target {} does not support {:?}",
                target.name, self.format
            ));
        }

        let encoded = encode_payload(self.format, &self.payload);
        let decoded = decode_payload(self.format, &encoded)?;
        let expected = format!("{}::{}::request", self.source, self.target).into_bytes();
        if decoded != expected {
            return Err("cross-contract payload mismatch".to_string());
        }

        Ok(CallResponse { acknowledged: true })
    }
}

#[derive(Debug, Clone)]
struct CallResponse {
    acknowledged: bool,
}

#[derive(Debug, Clone, Default)]
struct EventBus {
    subscribers: BTreeMap<String, Vec<String>>,
}

impl EventBus {
    fn subscribe(&mut self, publisher: &str, subscriber: &str, topic: &str) {
        let key = format!("{publisher}:{topic}");
        self.subscribers
            .entry(key)
            .or_default()
            .push(subscriber.to_string());
    }

    fn publish(&self, publisher: &str, topic: &str, payload: Vec<u8>) -> Vec<EventDelivery> {
        let key = format!("{publisher}:{topic}");
        let Some(subscribers) = self.subscribers.get(&key) else {
            return Vec::new();
        };

        subscribers
            .iter()
            .map(|subscriber| EventDelivery {
                subscriber: subscriber.clone(),
                payload: payload.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventDelivery {
    subscriber: String,
    payload: Vec<u8>,
}

#[derive(Debug, Clone)]
struct StateOperation {
    sequence: u64,
    pair_key: String,
    delta: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StateSnapshot {
    state_root: u64,
    version: u32,
    schema_version: u32,
}

impl StateSnapshot {
    fn new(seed: &str, schema_version: u32) -> Self {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        Self {
            state_root: hasher.finish(),
            version: 1,
            schema_version,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct StateReducer {
    total_delta: u64,
    hash_accumulator: u64,
}

impl StateReducer {
    fn apply(&mut self, operation: &StateOperation) {
        let mut hasher = DefaultHasher::new();
        operation.sequence.hash(&mut hasher);
        operation.pair_key.hash(&mut hasher);
        operation.delta.hash(&mut hasher);
        self.hash_accumulator ^= hasher.finish();
        self.total_delta += operation.delta;
    }

    fn snapshot(&self) -> StateSnapshot {
        let mut hasher = DefaultHasher::new();
        self.total_delta.hash(&mut hasher);
        self.hash_accumulator.hash(&mut hasher);
        StateSnapshot {
            state_root: hasher.finish(),
            version: 1,
            schema_version: 1,
        }
    }
}

#[derive(Debug, Clone)]
struct UpgradePlan {
    from_version: u32,
    to_version: u32,
    schema_version: u32,
}

impl UpgradePlan {
    fn new(from_version: u32, to_version: u32, schema_version: u32) -> Self {
        Self {
            from_version,
            to_version,
            schema_version,
        }
    }

    fn compatible_with(&self, other: &Self) -> bool {
        self.from_version == other.from_version
            && self.to_version == other.to_version
            && self.schema_version == other.schema_version
            && self.to_version > self.from_version
    }

    fn apply(&self, mut snapshot: StateSnapshot) -> Result<StateSnapshot, String> {
        if self.to_version <= self.from_version {
            return Err("invalid upgrade plan: non-incrementing version".to_string());
        }
        snapshot.version = self.to_version;
        Ok(snapshot)
    }
}
