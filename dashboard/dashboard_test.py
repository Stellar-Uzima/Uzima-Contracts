import json
import os
import unittest

class TestMaintainerDashboard(unittest.TestCase):
    def test_dashboard_files_exist(self):
        """Verifies that the dashboard outputs have been correctly compiled."""
        self.assertTrue(os.path.exists("dashboard/metrics.json"), "metrics.json missing")
        self.assertTrue(os.path.exists("docs/PORTFOLIO_HEALTH.md"), "PORTFOLIO_HEALTH.md missing")

    def test_metrics_json_structure(self):
        """Validates JSON schema structure for pipeline consumption."""
        with open("dashboard/metrics.json") as f:
            data = json.load(f)
            self.assertIn("audit", data)
            self.assertIn("contracts", data)
            self.assertIsInstance(data["contracts"], dict)

if __name__ == "__main__":
    unittest.main()