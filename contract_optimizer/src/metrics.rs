use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    pub total_recommendations: usize,
    pub applied_recommendations: usize,
    pub categories: HashMap<String, CategoryMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub total: usize,
    pub applied: usize,
}

impl AccuracyMetrics {
    pub fn new() -> Self {
        Self {
            total_recommendations: 0,
            applied_recommendations: 0,
            categories: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn record_recommendations(&mut self, recommendations: &[ContractAnalysis]) {
        for analysis in recommendations {
            for opt in &analysis.optimizations {
                self.total_recommendations += 1;
                self.categories.entry(opt.category.clone())
                    .or_insert(CategoryMetrics { total: 0, applied: 0 })
                    .total += 1;
            }
        }
    }

    pub fn record_applied(&mut self, category: &str) {
        self.applied_recommendations += 1;
        if let Some(cat) = self.categories.get_mut(category) {
            cat.applied += 1;
        }
    }

    pub fn accuracy_rate(&self) -> f64 {
        if self.total_recommendations == 0 {
            0.0
        } else {
            (self.applied_recommendations as f64 / self.total_recommendations as f64) * 100.0
        }
    }
}