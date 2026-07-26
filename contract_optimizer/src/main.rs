use clap::{Parser, Subcommand};
use contract_optimizer::complexity::{
    analyze_contract_complexity, check_report_thresholds, load_trends, record_trend, save_report,
    ContractThresholdResult, ThresholdLevel,
};
use contract_optimizer::metrics::AccuracyMetrics;
use contract_optimizer::{analyze_contracts, generate_report, integrate_pr_review};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "contract_optimizer")]
#[command(about = "Contract Optimization Recommendations Engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze contracts for optimization opportunities
    Analyze {
        /// Path to the contracts directory
        #[arg(short, long, default_value = "../contracts")]
        contracts_path: PathBuf,
        /// Output format: json or text
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Generate optimization report
    Report {
        /// Path to analysis results
        #[arg(short, long)]
        input: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show accuracy metrics
    Metrics {
        /// Path to metrics file
        #[arg(short, long, default_value = "optimization_metrics.json")]
        metrics_file: PathBuf,
    },
    /// Score contract complexity (cyclomatic, data, external calls, state, permissions)
    Complexity {
        /// Path to the contracts directory
        #[arg(short, long, default_value = "contracts")]
        contracts_path: PathBuf,
        /// JSON report output path
        #[arg(short, long, default_value = "dashboard/data/complexity_report.json")]
        output: PathBuf,
        /// Trend history file
        #[arg(long, default_value = "dashboard/data/complexity_trends.json")]
        trends: PathBuf,
        /// Skip writing trend snapshot
        #[arg(long)]
        no_trend: bool,
    },
    /// Run complexity scoring and enforce CI thresholds (warn/fail).
    /// Exits with code 0 (pass), 0+warnings (warn-only), or 1 (fail).
    /// Writes a PR-comment-ready text file to --comment-output.
    CheckComplexity {
        /// Path to the contracts directory
        #[arg(short, long, default_value = "contracts")]
        contracts_path: PathBuf,
        /// JSON report output path
        #[arg(short, long, default_value = "dashboard/data/complexity_report.json")]
        output: PathBuf,
        /// PR comment output path
        #[arg(long, default_value = "reports/complexity_pr_comment.txt")]
        comment_output: PathBuf,
    },
    /// Integrate analysis into a GitHub PR review
    PrReview {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,
        /// Pull request number
        #[arg(short, long)]
        pr_number: u64,
        /// GitHub token
        #[arg(short, long)]
        token: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            contracts_path,
            format,
        } => {
            let recommendations = analyze_contracts(&contracts_path)?;
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&recommendations)?),
                "text" => {
                    for rec in recommendations {
                        println!("Contract: {}", rec.contract_name);
                        for opt in rec.optimizations {
                            println!("  - {}: {}", opt.category, opt.description);
                        }
                        println!();
                    }
                },
                _ => eprintln!("Invalid format. Use 'json' or 'text'"),
            }
        },
        Commands::Report { input, output } => {
            let report = generate_report(&input)?;
            if let Some(out) = output {
                std::fs::write(&out, report)?;
                println!("Report written to {:?}", out);
            } else {
                println!("{}", report);
            }
        },
        Commands::Complexity {
            contracts_path,
            output,
            trends,
            no_trend,
        } => {
            let report = analyze_contract_complexity(&contracts_path)?;
            save_report(&report, &output)?;
            if !no_trend {
                record_trend(&report, &trends)?;
            }
            println!(
                "Complexity report: {} contracts, workspace average {}",
                report.contracts.len(),
                report.workspace_average
            );
            println!("Written to {}", output.display());
            if !no_trend {
                let store = load_trends(&trends)?;
                println!("Trend snapshots: {}", store.snapshots.len());
            }
        },
        Commands::CheckComplexity {
            contracts_path,
            output,
            comment_output,
        } => {
            let report = analyze_contract_complexity(&contracts_path)?;
            save_report(&report, &output)?;

            let summary = check_report_thresholds(&report);

            // Build PR comment body
            let priority_targets = ["medical_records", "cross_chain_bridge"];
            let mut comment = String::new();

            match summary.overall_level {
                ThresholdLevel::Pass => {
                    comment.push_str("### :white_check_mark: Contract Complexity\n\n");
                    comment.push_str("All contracts pass complexity thresholds.\n");
                },
                ThresholdLevel::Warn => {
                    comment.push_str("### :warning: Contract Complexity\n\n");
                    comment.push_str("Some contracts exceed **warn** thresholds. Review recommended.\n");
                },
                ThresholdLevel::Fail => {
                    comment.push_str("### :x: Contract Complexity\n\n");
                    comment.push_str(
                        "One or more contracts exceed **fail** thresholds. Pipeline blocked.\n",
                    );
                },
            }

            comment.push_str("\n| Contract | Total Score | Grade | Level |\n");
            comment.push_str("|----------|------------:|-------|-------|\n");

            for c in &summary.contracts {
                let emoji = match c.level {
                    ThresholdLevel::Pass => ":white_check_mark:",
                    ThresholdLevel::Warn => ":warning:",
                    ThresholdLevel::Fail => ":x:",
                };
                let grade_str = format!("{:?}", c.grade);
                let level_str = format!("{:?}", c.level);
                comment.push_str(&format!(
                    "| {} {} | {} | {} | {} |\n",
                    emoji, c.contract_name, c.total_score, grade_str, level_str,
                ));
            }

            // Violations detail
            let has_violations: Vec<&ContractThresholdResult> = summary
                .contracts
                .iter()
                .filter(|c| !c.violations.is_empty())
                .collect();
            if !has_violations.is_empty() {
                comment.push_str("\n**Threshold Breaches:**\n\n");
                for c in &has_violations {
                    comment.push_str(&format!("- **{}**\n", c.contract_name));
                    for v in &c.violations {
                        let lvl = format!("{:?}", v.level);
                        comment.push_str(&format!(
                            "  - `{}` = **{}** ({} threshold: {})\n",
                            v.metric, v.actual, lvl, v.threshold,
                        ));
                    }
                }
            }

            // Priority refactoring targets
            let priority_hits: Vec<&str> = priority_targets
                .iter()
                .copied()
                .filter(|t| report.contracts.iter().any(|c| &c.contract_name == t))
                .collect();
            if !priority_hits.is_empty() {
                comment.push_str("\n** :rotating_light: Priority Refactoring Targets**\n\n");
                comment.push_str(
                    "The following contracts are flagged for priority refactoring due to their \
                     size and complexity. Consider breaking them into smaller modules:\n\n",
                );
                for name in priority_hits {
                    comment.push_str(&format!("- `contracts/{}`\n", name));
                }
            }

            comment.push_str("\n> Run `./scripts/check_complexity.sh` locally to inspect.\n");
            comment.push_str(
                "> Download the `complexity-report` artifact for the full JSON breakdown.\n",
            );

            // Write PR comment file
            if let Some(parent) = comment_output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&comment_output, &comment)?;
            println!("PR comment written to {}", comment_output.display());

            // Print summary to stdout
            println!(
                "Overall: {:?} — {} contracts checked",
                summary.overall_level,
                summary.contracts.len()
            );
            for c in &summary.contracts {
                println!("  {:?}: {} (score {})", c.level, c.contract_name, c.total_score);
            }

            // Exit code: 0 for pass/warn, 1 for fail
            if summary.overall_level == ThresholdLevel::Fail {
                eprintln!("FAIL: One or more contracts exceed the fail threshold.");
                std::process::exit(1);
            }
        },
        Commands::PrReview {
            repo,
            pr_number,
            token,
        } => {
            integrate_pr_review(&repo, pr_number, &token).await?;
            println!("PR review integration completed");
        },
        Commands::Metrics { metrics_file } => {
            let metrics = AccuracyMetrics::load(&metrics_file)?;
            println!("Optimization Engine Accuracy Metrics");
            println!("====================================");
            println!("Total Recommendations: {}", metrics.total_recommendations);
            println!(
                "Applied Recommendations: {}",
                metrics.applied_recommendations
            );
            println!("Accuracy Rate: {:.2}%", metrics.accuracy_rate());
            println!("\nBy Category:");
            for (category, cat_metrics) in &metrics.categories {
                let rate = if cat_metrics.total > 0 {
                    (cat_metrics.applied as f64 / cat_metrics.total as f64) * 100.0
                } else {
                    0.0
                };
                println!(
                    "  {}: {}/{} ({:.2}%)",
                    category, cat_metrics.applied, cat_metrics.total, rate
                );
            }
        },
    }

    Ok(())
}
