"""Ordered README implementation tests for the Automatic Doors feature."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class AutomaticDoorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src = (ROOT / "src/automatic_doors.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()

    def test_authoritative_server_and_bounded_work(self):
        self.assertIn("fn tick(&self, host: &mut dyn ServerHost", self.src)
        self.assertIn("ServerTickPhase::ServerTickEnd { world_running: true", self.src)
        self.assertIn("host.players().server_players()", self.src)
        self.assertIn("MAX_TRACKED: usize = 256", self.src)
        self.assertIn("MAX_PLAYERS: usize = 24", self.src)
        self.assertIn("MAX_CLOSES: usize = 32", self.src)
        self.assertIn("SCAN_EVERY", self.src)
        self.assertIn("self.automatic_doors.reset()", self.hooks)

    def test_only_closes_doors_opened_by_module(self):
        self.assertIn("struct Opened", self.src)
        self.assertIn("list.push(Opened {", self.src)
        self.assertIn("self.close_due(host, &players, now, iron)", self.src)
        self.assertIn("close_enough(player.pos, entry.pos)", self.src)
        self.assertIn('property(&text, "powered") != Some("true")', self.src)
        self.assertIn('property(&lower_text, "open") != Some("false")', self.src)
        self.assertIn('level.set_block(host, pos, next, BlockUpdateFlags::NEIGHBORS', self.src)
        self.assertIn('if !on(host, "automatic_doors") || delay == 0', self.src)

    def test_all_original_options(self):
        self.assertIn('range("auto_door_open_time", "Open Duration (ms)", 2500., 0., 10000., 50.)', self.gui)
        self.assertIn('toggle("auto_iron_doors", "Include Iron Doors", true)', self.gui)
        self.assertIn('toggle("auto_prevent_sneak", "Don\'t Open While Sneaking", true)', self.gui)
        self.assertIn('ModuleSpec { key: "automatic_doors", label: "Automatic Doors"', self.gui)
        self.assertIn("fn delay_ticks(", self.src)
        self.assertIn('name.ends_with("_trapdoor")', self.src)

    def test_readme_sequence(self):
        text = (ROOT / "README.md").read_text()
        features = text.split("### Bundle-inspired quality-of-life features")[1].split("### Additional quality-of-life features")[0]
        entries = re.findall(r"^- \[[ x]\] (.*)$", features, flags=re.MULTILINE)
        self.assertTrue(entries[0].startswith("Guaranteed skull drops"))
        self.assertTrue(entries[1].startswith("Nearby doors open automatically"))
        self.assertIn("- [x] Nearby doors open automatically", features)
        self.assertIn("- [ ] Beacon bases can be placed", features)


if __name__ == "__main__":
    unittest.main()
