import json
import os
import unittest

class TestBenchmarkRunner(unittest.TestCase):
    def test_resource_budget_schema(self):
        """Validates that contract resource budget rules exist and are formatted correctly."""
        budget_path = "resource-budgets/contracts_budget.json"
        self.assertTrue(os.path.exists(budget_path), "Resource budget config is missing")
        
        with open(budget_path, "r") as f:
            data = json.load(f)
            self.assertIn("limits", data)
            self.assertIn("sync_manager", data["limits"])
            self.assertIn("cpu_instructions_max", data["limits"]["sync_manager"])

    def test_benchmark_output_reporting(self):
        """Ensures benchmark runner outputs valid telemetry JSON."""
        output_path = "tests/benchmark_results.json"
        if os.path.exists(output_path):
            with open(output_path, "r") as f:
                data = json.load(f)
                self.assertIn("timestamp", data)
                self.assertIn("results", data)

if __name__ == "__main__":
    unittest.main()