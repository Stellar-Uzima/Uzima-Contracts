import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from risk_scoring_engine import ContractRiskAnalyzer


class TestRiskScoringEngine(unittest.TestCase):
    def test_analyzer_metrics_calculation(self):
        with TemporaryDirectory() as tmpdir:
            contract_dir = Path(tmpdir) / "test_contract"
            contract_dir.mkdir()
            src_dir = contract_dir / "src"
            src_dir.mkdir()

            code_content = """
            pub fn test_fn(env: Env, admin: Address) {
                admin.require_auth();
                if true {
                    env.storage().instance().set(&Symbol::new(&env, "KEY"), &1);
                }
            }
            """
            with open(src_dir / "lib.rs", "w") as f:
                f.write(code_content)

            analyzer = ContractRiskAnalyzer(contract_dir)
            metrics = analyzer.evaluate_composite_risk()

            self.assertEqual(metrics["contract"], "test_contract")
            self.assertIn("composite_score", metrics)
            self.assertGreater(metrics["metrics"]["blast_radius"], 0)


if __name__ == "__main__":
    unittest.main()