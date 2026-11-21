"""The release must fail if real-engine cases fail, disappear or are repeated."""
import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("gameplay_report", ROOT / "tools/gameplay/report.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class GameplayTests(unittest.TestCase):
    def test_evidence_must_be_complete(self):
        required = json.loads((ROOT / "tools/gameplay/cases.json").read_text())["required"]
        success = "\n".join("QOL_GAMEPLAY|" + key + "|PASS" for key in required)
        complete = success + "\ntest result: ok. 1 passed;"
        self.assertTrue(module.validate(complete, required, 0)[0])
        self.assertFalse(module.validate(complete, required, 1)[0])
        self.assertFalse(module.validate(success, required, 0)[0])
        self.assertFalse(module.validate(complete.replace("PASS", "FAIL", 1), required, 0)[0])
        self.assertFalse(module.validate(complete + "\nQOL_GAMEPLAY|soil.disabled|PASS", required, 0)[0])
        self.assertFalse(module.validate(complete.replace("QOL_GAMEPLAY|soil.no_farmland|PASS", ""), required, 0)[0])
        self.assertFalse(module.validate(complete + "\nQOL_GAMEPLAY|unknown|PASS", required, 0)[0])

    def test_release_follows_real_game_gate(self):
        for path in [".github/workflows/sandbox.yml", ".github/workflows/release.yml"]:
            flow = (ROOT / path).read_text()
            self.assertIn("bash tools/gameplay/run.sh", flow)
            self.assertIn("SUNRISE_TEST_RESOURCES", flow)
            self.assertIn("gameplay-evidence.json", flow)
            self.assertLess(flow.index("bash tools/sandbox/run.sh"), flow.index("bash tools/gameplay/run.sh"))
        release = (ROOT / ".github/workflows/release.yml").read_text()
        self.assertLess(release.index("bash tools/gameplay/run.sh"),
                        release.index("name: Commit version and tag only after a successful build"))

    def test_no_fallback_to_fake_game_resources(self):
        runner = (ROOT / "tools/gameplay/run.sh").read_text()
        for filename in ("facts.json", "gameplay-facts.json", "block-behaviours.json"):
            self.assertIn(filename, runner)
        self.assertIn("Provision a licensed Sunrise prepared-data folder", runner)
        self.assertIn("exit 1", runner)
        self.assertIn("cargo check --manifest-path", runner)
        self.assertIn("--lib --tests --no-default-features --features testing", runner)
        self.assertIn("cargo test --manifest-path", runner)

    def test_real_game_not_just_mock_dispatch(self):
        src = (ROOT / "tools/gameplay/sunrise_qol_gameplay.rs.txt").read_text()
        self.assertIn("sunrise_mod_wasm::load(", src)
        self.assertIn("super::native_entity_refs::tests::with_game(", src)
        self.assertIn("game.player_destroy_block(actor, fruit, false)", src)
        self.assertIn("block(game, soil)", src)
        self.assertNotIn("dispatch_player_destroyed_block(", src)
        self.assertIn("mod_platform::__private::initialize()", src)
        self.assertIn("#![cfg(test)]", src)
        runner = (ROOT / "tools/gameplay/run.sh").read_text()
        self.assertNotIn('game_rs="$source/crates/sunrise-client/src/game.rs"', runner)
        for name in json.loads((ROOT / "tools/gameplay/cases.json").read_text())["required"]:
            self.assertIn("QOL_GAMEPLAY|" + name + "|PASS", src)


if __name__ == "__main__":
    unittest.main()
