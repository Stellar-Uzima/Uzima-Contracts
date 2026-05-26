use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{
    Expr, ExprBinary, ExprCall, ExprIf, ExprMatch, ExprMethodCall, ExprWhile, ImplItem, Item,
    ItemEnum, ItemFn, ItemImpl, ItemStruct,
};
use walkdir::WalkDir;

/// Individual complexity dimensions from issue #481.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplexityComponents {
    pub cyclomatic_complexity: u32,
    pub data_structure_complexity: u32,
    pub external_interaction_count: u32,
    pub state_transition_count: u32,
    pub permission_model_complexity: u32,
}

/// Normalized component scores (0–100 each).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityComponentScores {
    pub cyclomatic: u32,
    pub data_structure: u32,
    pub external_interaction: u32,
    pub state_transition: u32,
    pub permission_model: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityGrade {
    Low,
    Medium,
    High,
}

/// Aggregated complexity result for one contract crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractComplexityScore {
    pub contract_name: String,
    pub total_score: u32,
    pub grade: ComplexityGrade,
    pub components: ComplexityComponents,
    pub component_scores: ComplexityComponentScores,
    pub function_count: u32,
    pub analyzed_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub generated_at: String,
    pub workspace_average: u32,
    pub contracts: Vec<ContractComplexityScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplexityTrendStore {
    pub snapshots: Vec<ComplexityTrendSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityTrendSnapshot {
    pub recorded_at: String,
    pub workspace_average: u32,
    pub contracts: HashMap<String, u32>,
}

const WEIGHT_CYCLOMATIC: f64 = 0.25;
const WEIGHT_DATA: f64 = 0.20;
const WEIGHT_EXTERNAL: f64 = 0.20;
const WEIGHT_STATE: f64 = 0.20;
const WEIGHT_PERMISSION: f64 = 0.15;

const CAP_CYCLOMATIC: f64 = 80.0;
const CAP_DATA: f64 = 120.0;
const CAP_EXTERNAL: f64 = 40.0;
const CAP_STATE: f64 = 30.0;
const CAP_PERMISSION: f64 = 35.0;

pub fn analyze_contract_complexity(contracts_path: &Path) -> Result<ComplexityReport, Box<dyn std::error::Error>> {
    let mut by_contract: HashMap<String, ComplexityComponents> = HashMap::new();
    let mut function_counts: HashMap<String, u32> = HashMap::new();
    let mut file_counts: HashMap<String, u32> = HashMap::new();

    for entry in WalkDir::new(contracts_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let contract_name = contract_name_from_path(contracts_path, path);
        let Some(name) = contract_name else {
            continue;
        };

        let content = fs::read_to_string(path)?;
        let file_metrics = analyze_rust_source(&content);

        let entry = by_contract.entry(name.clone()).or_default();
        entry.cyclomatic_complexity += file_metrics.cyclomatic_complexity;
        entry.data_structure_complexity += file_metrics.data_structure_complexity;
        entry.external_interaction_count += file_metrics.external_interaction_count;
        entry.state_transition_count += file_metrics.state_transition_count;
        entry.permission_model_complexity += file_metrics.permission_model_complexity;

        *function_counts.entry(name.clone()).or_insert(0) += file_metrics.function_count;
        *file_counts.entry(name).or_insert(0) += 1;
    }

    let mut contracts: Vec<ContractComplexityScore> = by_contract
        .into_iter()
        .map(|(contract_name, components)| {
            let component_scores = score_components(&components);
            let total_score = weighted_total(&component_scores);
            ContractComplexityScore {
                grade: grade_from_score(total_score),
                function_count: function_counts.get(&contract_name).copied().unwrap_or(0),
                analyzed_files: file_counts.get(&contract_name).copied().unwrap_or(0),
                contract_name,
                total_score,
                components,
                component_scores,
            }
        })
        .collect();

    contracts.sort_by(|a, b| b.total_score.cmp(&a.total_score));

    let workspace_average = if contracts.is_empty() {
        0
    } else {
        contracts.iter().map(|c| c.total_score).sum::<u32>() / contracts.len() as u32
    };

    Ok(ComplexityReport {
        generated_at: chrono_like_timestamp(),
        workspace_average,
        contracts,
    })
}

pub fn save_report(report: &ComplexityReport, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

pub fn record_trend(report: &ComplexityReport, trends_path: &Path) -> Result<ComplexityTrendStore, Box<dyn std::error::Error>> {
    let mut store = load_trends(trends_path)?;
    let mut contracts = HashMap::new();
    for c in &report.contracts {
        contracts.insert(c.contract_name.clone(), c.total_score);
    }

    store.snapshots.push(ComplexityTrendSnapshot {
        recorded_at: report.generated_at.clone(),
        workspace_average: report.workspace_average,
        contracts,
    });

    if store.snapshots.len() > 90 {
        let drain = store.snapshots.len() - 90;
        store.snapshots.drain(0..drain);
    }

    if let Some(parent) = trends_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(trends_path, serde_json::to_string_pretty(&store)?)?;
    Ok(store)
}

pub fn load_trends(path: &Path) -> Result<ComplexityTrendStore, Box<dyn std::error::Error>> {
    if path.exists() {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    } else {
        Ok(ComplexityTrendStore::default())
    }
}

fn contract_name_from_path(contracts_root: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(contracts_root).ok()?;
    let mut parts = rel.components();
    let first = parts.next()?;
    first.as_os_str().to_str().map(|s| s.to_string())
}

#[derive(Default)]
struct FileMetrics {
    cyclomatic_complexity: u32,
    data_structure_complexity: u32,
    external_interaction_count: u32,
    state_transition_count: u32,
    permission_model_complexity: u32,
    function_count: u32,
}

fn analyze_rust_source(content: &str) -> FileMetrics {
    let mut metrics = FileMetrics::default();

    if let Ok(ast) = syn::parse_file(content) {
        let mut visitor = ComplexityVisitor::default();
        visitor.visit_file(&ast);
        metrics.cyclomatic_complexity = visitor.cyclomatic;
        metrics.data_structure_complexity = visitor.data_structure;
        metrics.external_interaction_count = visitor.external_interactions;
        metrics.state_transition_count = visitor.state_transitions;
        metrics.permission_model_complexity = visitor.permission_complexity;
        metrics.function_count = visitor.function_count;
    }

    metrics
}

#[derive(Default)]
struct ComplexityVisitor {
    cyclomatic: u32,
    data_structure: u32,
    external_interactions: u32,
    state_transitions: u32,
    permission_complexity: u32,
    function_count: u32,
    enum_variant_count: u32,
    in_function_depth: u32,
}

impl ComplexityVisitor {
    fn bump_cyclomatic(&mut self, amount: u32) {
        self.cyclomatic += amount;
    }

    fn enter_function(&mut self, name: &str) {
        self.function_count += 1;
        self.cyclomatic += 1;
        let name = name.to_lowercase();
        if name.contains("require_auth") || name.contains("only_admin") || name.contains("authorize") {
            self.permission_complexity += 1;
        }
        if name.contains("transition") || name.contains("set_status") || name.contains("update_status") {
            self.state_transitions += 1;
        }
        self.in_function_depth += 1;
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Struct(ItemStruct { fields, .. }) => {
                self.data_structure += 2;
                self.data_structure += fields.len() as u32;
            }
            Item::Enum(ItemEnum { variants, .. }) => {
                self.data_structure += 3;
                let variant_count = variants.len() as u32;
                self.enum_variant_count += variant_count;
                self.state_transitions += variant_count;
                for v in variants {
                    self.data_structure += v.fields.len() as u32;
                }
            }
            _ => {}
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.enter_function(&node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.in_function_depth -= 1;
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            if let ImplItem::Fn(method) = item {
                self.enter_function(&method.sig.ident.to_string());
            }
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.bump_cyclomatic(1);
        syn::visit::visit_expr_if(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        let arms = node.arms.len().max(1) as u32;
        self.bump_cyclomatic(arms);
        for arm in &node.arms {
            if contains_status_pattern(&arm.pat) {
                self.state_transitions += 1;
            }
        }
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_while(&mut self, node: &'ast ExprWhile) {
        self.bump_cyclomatic(1);
        syn::visit::visit_expr_while(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        if matches!(node.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) {
            self.bump_cyclomatic(1);
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path) = &*node.func {
            let path_str = quote::quote!(#path).to_string();
            if path_str.contains("invoke_contract") || path_str.contains("call_contract") {
                self.external_interactions += 1;
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();
        match method.as_str() {
            "require_auth" | "require_auth_for_current_contract" => {
                self.permission_complexity += 2;
            }
            "storage" => {
                if method == "storage" {
                    // env.storage().get/set — counted via method chain below
                }
            }
            "get" | "set" | "update" | "extend_ttl" | "delete" => {
                self.data_structure += 1;
            }
            "invoke_contract" | "invoke_contract_v2" => {
                self.external_interactions += 2;
            }
            _ => {
                if method.contains("auth") || method.contains("admin") || method.contains("role") {
                    self.permission_complexity += 1;
                }
                if method.contains("status") || method.contains("transition") {
                    self.state_transitions += 1;
                }
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn contains_status_pattern(pat: &syn::Pat) -> bool {
    let text = quote::quote!(#pat).to_string().to_lowercase();
    text.contains("status") || text.contains("pending") || text.contains("active")
}

fn normalize(value: u32, cap: f64) -> u32 {
    let ratio = (value as f64 / cap).min(1.0);
    (ratio * 100.0).round() as u32
}

fn score_components(components: &ComplexityComponents) -> ComplexityComponentScores {
    ComplexityComponentScores {
        cyclomatic: normalize(components.cyclomatic_complexity, CAP_CYCLOMATIC),
        data_structure: normalize(components.data_structure_complexity, CAP_DATA),
        external_interaction: normalize(components.external_interaction_count, CAP_EXTERNAL),
        state_transition: normalize(components.state_transition_count, CAP_STATE),
        permission_model: normalize(components.permission_model_complexity, CAP_PERMISSION),
    }
}

fn weighted_total(scores: &ComplexityComponentScores) -> u32 {
    let total = scores.cyclomatic as f64 * WEIGHT_CYCLOMATIC
        + scores.data_structure as f64 * WEIGHT_DATA
        + scores.external_interaction as f64 * WEIGHT_EXTERNAL
        + scores.state_transition as f64 * WEIGHT_STATE
        + scores.permission_model as f64 * WEIGHT_PERMISSION;
    total.round() as u32
}

fn grade_from_score(score: u32) -> ComplexityGrade {
    if score < 40 {
        ComplexityGrade::Low
    } else if score < 70 {
        ComplexityGrade::Medium
    } else {
        ComplexityGrade::High
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        enum Error { Invalid }

        struct Example;

        impl Example {
            pub fn init(env: Env, admin: Address) {
                admin.require_auth();
                if env.storage().instance().has(&admin) {
                    return;
                }
                env.storage().instance().set(&admin, &true);
            }

            pub fn transition(env: Env, _id: u32, status: u32) -> Result<(), Error> {
                env.current_contract_address().require_auth();
                match status {
                    0 => {}
                    1 => {}
                    _ => return Err(Error::Invalid),
                }
                Ok(())
            }
        }

        enum State { A, B, C }

        struct Record { id: u32 }
    "#;

    #[test]
    fn analyzes_sample_contract_metrics() {
        let m = analyze_rust_source(SAMPLE);
        assert!(m.cyclomatic_complexity >= 3);
        assert!(m.data_structure_complexity >= 5);
        assert!(m.external_interaction_count >= 0);
        assert!(m.permission_model_complexity >= 2);
        assert!(m.state_transition_count >= 3);
        assert_eq!(m.function_count, 2);
    }

    #[test]
    fn weighted_score_is_bounded() {
        let components = ComplexityComponents {
            cyclomatic_complexity: 200,
            data_structure_complexity: 500,
            external_interaction_count: 100,
            state_transition_count: 100,
            permission_model_complexity: 100,
        };
        let scores = score_components(&components);
        let total = weighted_total(&scores);
        assert_eq!(total, 100);
        assert_eq!(grade_from_score(total), ComplexityGrade::High);
    }
}
