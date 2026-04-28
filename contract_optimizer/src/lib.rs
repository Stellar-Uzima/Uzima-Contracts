use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use syn::{visit::Visit, File, ItemFn, ExprCall, ExprMethodCall, Expr};
use walkdir::WalkDir;

pub mod metrics;

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub description: String,
    pub severity: String, // "low", "medium", "high"
    pub location: Option<String>, // file:line
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractAnalysis {
    pub contract_name: String,
    pub optimizations: Vec<OptimizationRecommendation>,
}

pub fn analyze_contracts(contracts_path: &Path) -> Result<Vec<ContractAnalysis>, Box<dyn std::error::Error>> {
    let mut analyses = Vec::new();

    for entry in WalkDir::new(contracts_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "rs" {
            if let Some(contract_name) = entry.path().parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let optimizations = analyze_file(&content, entry.path());
                    if !optimizations.is_empty() {
                        analyses.push(ContractAnalysis {
                            contract_name: contract_name.to_string(),
                            optimizations,
                        });
                    }
                }
            }
        }
    }

    // Record metrics
    let mut metrics = metrics::AccuracyMetrics::load(Path::new("optimization_metrics.json")).unwrap_or_default();
    metrics.record_recommendations(&analyses);
    metrics.save(Path::new("optimization_metrics.json"))?;

    Ok(analyses)
}

fn analyze_file(content: &str, path: &Path) -> Vec<OptimizationRecommendation> {
    let mut recommendations = Vec::new();

    // Parse the file
    if let Ok(ast) = syn::parse_file(content) {
        let mut visitor = OptimizationVisitor::new(path);
        visitor.visit_file(&ast);
        recommendations.extend(visitor.recommendations);
    }

    // Additional text-based analysis
    recommendations.extend(analyze_text_patterns(content, path));

    recommendations
}

struct OptimizationVisitor<'a> {
    path: &'a Path,
    recommendations: Vec<OptimizationRecommendation>,
}

impl<'a> OptimizationVisitor<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            recommendations: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for OptimizationVisitor<'a> {
    fn visit_item_fn(&mut self, node: &'a ItemFn) {
        // Check for gas-intensive operations
        if let Some(block) = &node.block {
            self.analyze_function_body(block, &node.sig.ident.to_string());
        }
    }

    fn visit_expr_call(&mut self, node: &'a ExprCall) {
        // Check for expensive function calls
        if let Expr::Path(path) = &*node.func {
            if let Some(ident) = path.path.get_ident() {
                match ident.to_string().as_str() {
                    "env" => {
                        if let Some(arg) = node.args.first() {
                            if let Expr::MethodCall(method_call) = arg {
                                if let Some(method_ident) = &method_call.method.get_ident() {
                                    match method_ident.to_string().as_str() {
                                        "storage" => {
                                            self.recommendations.push(OptimizationRecommendation {
                                                category: "Storage Efficiency".to_string(),
                                                description: "Frequent storage operations detected".to_string(),
                                                severity: "medium".to_string(),
                                                location: Some(format!("{}:{}", self.path.display(), line_number(node))),
                                                suggestion: "Consider batching storage operations or using temporary variables".to_string(),
                                            });
                                        }
                                        "events" => {
                                            self.recommendations.push(OptimizationRecommendation {
                                                category: "Gas Optimization".to_string(),
                                                description: "Event emission in loop or frequent call".to_string(),
                                                severity: "low".to_string(),
                                                location: Some(format!("{}:{}", self.path.display(), line_number(node))),
                                                suggestion: "Emit events outside of loops when possible".to_string(),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl<'a> OptimizationVisitor<'a> {
    fn analyze_function_body(&mut self, block: &'a syn::Block, fn_name: &str) {
        // Check for loops that might be gas-intensive
        for stmt in &block.stmts {
            if let syn::Stmt::Expr(Expr::ForLoop(for_loop), _) = stmt {
                self.recommendations.push(OptimizationRecommendation {
                    category: "Algorithm Optimization".to_string(),
                    description: format!("Loop detected in function '{}'", fn_name),
                    severity: "medium".to_string(),
                    location: Some(format!("{}:{}", self.path.display(), line_number(for_loop))),
                    suggestion: "Consider if loop iterations can be reduced or operations batched".to_string(),
                });
            }
        }
    }
}

fn analyze_text_patterns(content: &str, path: &Path) -> Vec<OptimizationRecommendation> {
    let mut recommendations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        // Check for large data structures
        if line.contains("Vec::new()") && line.contains("push") {
            recommendations.push(OptimizationRecommendation {
                category: "Storage Efficiency".to_string(),
                description: "Dynamic vector allocation detected".to_string(),
                severity: "low".to_string(),
                location: Some(format!("{}:{}", path.display(), line_num)),
                suggestion: "Consider using fixed-size arrays or pre-allocating capacity".to_string(),
            });
        }

        // Check for string operations
        if line.contains("String::from") || line.contains(".to_string()") {
            recommendations.push(OptimizationRecommendation {
                category: "Gas Optimization".to_string(),
                description: "String allocation detected".to_string(),
                severity: "low".to_string(),
                location: Some(format!("{}:{}", path.display(), line_num)),
                suggestion: "Use &str where possible to avoid allocations".to_string(),
            });
        }

        // Check for potential batching opportunities
        if line.contains("for") && (line.contains("storage") || line.contains("write")) {
            recommendations.push(OptimizationRecommendation {
                category: "Batching Opportunities".to_string(),
                description: "Storage write in loop detected".to_string(),
                severity: "high".to_string(),
                location: Some(format!("{}:{}", path.display(), line_num)),
                suggestion: "Consider batching multiple storage writes into a single operation".to_string(),
            });
        }

        // Check for parallelization possibilities (though limited in Soroban)
        if line.contains("cross_chain") || line.contains("multi_region") {
            recommendations.push(OptimizationRecommendation {
                category: "Parallelization Possibilities".to_string(),
                description: "Cross-chain or multi-region operation detected".to_string(),
                severity: "medium".to_string(),
                location: Some(format!("{}:{}", path.display(), line_num)),
                suggestion: "Consider asynchronous processing or parallel execution where applicable".to_string(),
            });
        }
    }

    recommendations
}

fn line_number<T>(node: &T) -> usize {
    // Simplified: in practice, use node.span().start().line
    // But for this demo, return 0
    0
}

pub fn generate_report(input_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(input_path)?;
    let analyses: Vec<ContractAnalysis> = serde_json::from_str(&content)?;

    let mut report = String::new();
    report.push_str("# Contract Optimization Report\n\n");

    for analysis in analyses {
        report.push_str(&format!("## {}\n\n", analysis.contract_name));
        for opt in analysis.optimizations {
            report.push_str(&format!("### {} ({})\n", opt.category, opt.severity.to_uppercase()));
            report.push_str(&format!("**Description:** {}\n\n", opt.description));
            report.push_str(&format!("**Suggestion:** {}\n\n", opt.suggestion));
            if let Some(loc) = &opt.location {
                report.push_str(&format!("**Location:** {}\n\n", loc));
            }
            report.push_str("---\n\n");
        }
    }

    Ok(report)
}

pub async fn integrate_pr_review(repo: &str, pr_number: u64, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This would integrate with GitHub API to post comments on PR
    // For now, just a placeholder
    println!("Integrating analysis into PR review for {}/{}", repo, pr_number);
    // Use octocrab to create PR comments with recommendations
    Ok(())
}