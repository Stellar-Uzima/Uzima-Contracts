/// Performance testing utilities for contracts
use std::time::Instant;
use std::collections::HashMap;

/// Performance test result
#[derive(Clone, Debug)]
pub struct PerformanceResult {
    pub name: String,
    pub duration_ms: u128,
    pub iterations: u32,
    pub avg_ms: f64,
    pub min_ms: u128,
    pub max_ms: u128,
}

impl PerformanceResult {
    /// Check if performance is acceptable
    pub fn is_acceptable(&self, max_ms: u128) -> bool {
        self.avg_ms as u128 <= max_ms
    }

    /// Get performance summary
    pub fn summary(&self) -> String {
        format!(
            "Test: {} | Iterations: {} | Avg: {:.2}ms | Min: {}ms | Max: {}ms",
            self.name, self.iterations, self.avg_ms, self.min_ms, self.max_ms
        )
    }
}

/// Performance benchmark runner
pub struct BenchmarkRunner {
    name: String,
    iterations: u32,
    results: Vec<u128>,
}

impl BenchmarkRunner {
    /// Create new benchmark runner
    pub fn new(name: &str, iterations: u32) -> Self {
        Self {
            name: name.to_string(),
            iterations,
            results: Vec::new(),
        }
    }

    /// Run a benchmark function
    pub fn run<F: FnMut()>(&mut self, mut f: F) -> PerformanceResult {
        for _ in 0..self.iterations {
            let start = Instant::now();
            f();
            self.results.push(start.elapsed().as_millis());
        }

        let total: u128 = self.results.iter().sum();
        let min = *self.results.iter().min().unwrap_or(&0);
        let max = *self.results.iter().max().unwrap_or(&0);
        let avg = total as f64 / self.iterations as f64;

        PerformanceResult {
            name: self.name.clone(),
            duration_ms: total,
            iterations: self.iterations,
            avg_ms: avg,
            min_ms: min,
            max_ms: max,
        }
    }
}

/// Performance test suite
pub struct PerformanceSuite {
    tests: HashMap<String, PerformanceResult>,
}

impl PerformanceSuite {
    /// Create new suite
    pub fn new() -> Self {
        Self {
            tests: HashMap::new(),
        }
    }

    /// Add performance test result
    pub fn add(&mut self, result: PerformanceResult) {
        self.tests.insert(result.name.clone(), result);
    }

    /// Get test result
    pub fn get(&self, name: &str) -> Option<&PerformanceResult> {
        self.tests.get(name)
    }

    /// Get all results
    pub fn all_results(&self) -> Vec<&PerformanceResult> {
        self.tests.values().collect()
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = "=== Performance Test Report ===\n".to_string();
        for result in self.all_results() {
            report.push_str(&format!("{}\n", result.summary()));
        }
        report.push_str("================================\n");
        report
    }

    /// Check all tests pass performance targets
    pub fn validate_targets(&self, targets: &HashMap<String, u128>) -> bool {
        for (name, max_ms) in targets {
            if let Some(result) = self.get(name) {
                if !result.is_acceptable(*max_ms) {
                    println!(
                        "PERF FAIL: {} - {}ms > {}ms",
                        name, result.avg_ms as u128, max_ms
                    );
                    return false;
                }
            }
        }
        true
    }
}

/// Load test utilities
pub struct LoadTest {
    name: String,
    concurrent_operations: usize,
}

impl LoadTest {
    /// Create new load test
    pub fn new(name: &str, concurrent_ops: usize) -> Self {
        Self {
            name: name.to_string(),
            concurrent_operations: concurrent_ops,
        }
    }

    /// Calculate throughput (operations per second)
    pub fn calculate_throughput(operations: usize, duration_secs: f64) -> f64 {
        operations as f64 / duration_secs
    }

    /// Calculate latency percentiles
    pub fn calculate_percentiles(mut durations: Vec<u128>) -> Percentiles {
        durations.sort();
        let len = durations.len();

        Percentiles {
            p50: durations[len / 2],
            p95: durations[(len as f64 * 0.95) as usize],
            p99: durations[(len as f64 * 0.99) as usize],
        }
    }
}

/// Latency percentiles
#[derive(Debug, Clone)]
pub struct Percentiles {
    pub p50: u128,
    pub p95: u128,
    pub p99: u128,
}

/// Stress test utilities
pub struct StressTest {
    pub name: String,
    pub duration_secs: u64,
    pub concurrent_threads: usize,
}

impl StressTest {
    /// Create new stress test
    pub fn new(name: &str, duration_secs: u64, threads: usize) -> Self {
        Self {
            name: name.to_string(),
            duration_secs,
            concurrent_threads: threads,
        }
    }
}

/// Memory usage tracker
pub struct MemoryTracker {
    initial: Option<usize>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self { initial: None }
    }

    /// Start tracking memory
    pub fn start(&mut self) {
        // Note: This is a simplified tracker. Real implementation would use system APIs
        self.initial = Some(0);
    }

    /// Calculate memory delta
    pub fn delta(&self) -> Option<usize> {
        self.initial.map(|initial| {
            // Simplified: return a placeholder value
            0
        })
    }
}

/// Common performance targets (in milliseconds)
pub mod performance_targets {
    use std::collections::HashMap;

    pub fn default_targets() -> HashMap<String, u128> {
        let mut targets = HashMap::new();
        targets.insert("record_creation".to_string(), 100);
        targets.insert("record_access".to_string(), 50);
        targets.insert("consent_grant".to_string(), 75);
        targets.insert("record_sharing".to_string(), 80);
        targets.insert("bulk_read".to_string(), 500);
        targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new("test", 5);
        let result = runner.run(|| {
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
        assert!(result.avg_ms >= 1.0);
    }

    #[test]
    fn test_performance_suite() {
        let mut suite = PerformanceSuite::new();
        let result = PerformanceResult {
            name: "test_operation".to_string(),
            duration_ms: 500,
            iterations: 10,
            avg_ms: 50.0,
            min_ms: 40,
            max_ms: 60,
        };
        suite.add(result);
        assert!(suite.get("test_operation").is_some());
    }

    #[test]
    fn test_calculate_throughput() {
        let throughput = LoadTest::calculate_throughput(1000, 10.0);
        assert_eq!(throughput, 100.0);
    }

    #[test]
    fn test_calculate_percentiles() {
        let durations = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let percentiles = LoadTest::calculate_percentiles(durations);
        assert!(percentiles.p50 > 0);
        assert!(percentiles.p95 > percentiles.p50);
        assert!(percentiles.p99 >= percentiles.p95);
    }

    #[test]
    fn test_performance_result_summary() {
        let result = PerformanceResult {
            name: "test".to_string(),
            duration_ms: 500,
            iterations: 10,
            avg_ms: 50.0,
            min_ms: 40,
            max_ms: 60,
        };
        let summary = result.summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("50.00ms"));
    }
}
