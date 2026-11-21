"""Bottled air should add bounded oxygen only for a submerged glass bottle user."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class BottledAirTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src = (ROOT / "src/bottled_air.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()

    def test_server_air_checks(self):
        self.assertIn('if !on(host, "bottled_air")', self.src)
        self.assertIn('held.item().id(host) != "minecraft:glass_bottle"', self.src)
        self.assertIn("entity.is_in_water(host) != Some(true)", self.src)
        self.assertIn("entity.air_supply(host)", self.src)
        self.assertIn("entity.max_air_supply(host)", self.src)
        self.assertIn("entity.set_air_supply(host, current.saturating_add(restored).min(max))", self.src)
        self.assertIn("current.saturating_add(restored).min(max)", self.src)
        self.assertIn("bottled_air::use_bottle(HostSide::Server(host), event)", self.hooks)
        self.assertIn("Flow::Continue", self.src)

    def test_settings_and_readme_order(self):
        self.assertIn('range("bottled_air_amount", "Oxygen Restored", 150., 0., 1000., 10.)', self.gui)
        self.assertIn('ModuleSpec { key: "bottled_air", label: "Bottled Air"', self.gui)
        readme = (ROOT / "README.md").read_text()
        self.assertLess(readme.index("Mob spawners can be disabled"), readme.index("Glass bottles can temporarily"))
        self.assertIn("- [x] Glass bottles can temporarily", readme)
        self.assertIn("- [x] The help command gains clearer", readme)


if __name__ == "__main__":
    unittest.main()
