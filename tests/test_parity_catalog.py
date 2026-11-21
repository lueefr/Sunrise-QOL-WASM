"""Reject forgotten parity rows or claims of 1:1 without gameplay evidence."""
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
file = ROOT / "tools/parity/report.py"
spec = importlib.util.spec_from_file_location("source_parity_report", file)
parity = importlib.util.module_from_spec(spec)
spec.loader.exec_module(parity)


class ParityCatalogTests(unittest.TestCase):
    def test_every_registered_module_has_limitations(self):
        report = parity.inventory()
        self.assertEqual(report["module_count"], 56)
        self.assertEqual(len(report["modules"]), 56)
        self.assertEqual(len({row["key"] for row in report["modules"]}), 56)
        self.assertTrue(all(row["known_differences"] for row in report["modules"]))
        self.assertEqual(report["fully_certified"], 0)
        self.assertTrue(all(not row["certified_one_to_one"] for row in report["modules"]))

    def test_parity_matrix_is_collected_by_both_workflows(self):
        for path in [".github/workflows/sandbox.yml", ".github/workflows/release.yml"]:
            workflow = (ROOT / path).read_text()
            self.assertIn("python3 tools/parity/report.py --output target/parity-coverage.json", workflow)
            self.assertIn("sunrise-qol-parity-matrix", workflow)

    def test_requested_unavailable_feature_is_not_in_module_roster(self):
        report = parity.inventory()
        self.assertIn("Full Brightness Toggle", report["other_requested_features_not_configured"])
        self.assertNotIn("full_brightness", [row["key"] for row in report["modules"]])

    def test_leaf_decay_is_shared_and_not_invented_as_tree_harvest_extension(self):
        report = parity.inventory()
        leaf = next(row for row in report["modules"] if row["key"] == "leaf_decay")
        self.assertIn("Tree Harvester", leaf["known_differences"])
        self.assertFalse(leaf["certified_one_to_one"])


if __name__ == "__main__":
    unittest.main()
