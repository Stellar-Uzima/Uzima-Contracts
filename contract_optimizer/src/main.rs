use clap::{Parser, Subcommand};
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
