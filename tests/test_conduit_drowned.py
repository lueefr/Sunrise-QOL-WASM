"""Conduit Power must prevent drowned spawns without touching other mobs."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class DrownedTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src=(ROOT/"src/conduit_drowned.rs").read_text()
        cls.gui=(ROOT/"src/gui.rs").read_text()
        cls.hooks=(ROOT/"src/lib.rs").read_text()

    def test_read_only_conduit_power_check(self):
        self.assertIn('if !on(host, "drowned_prevention")',self.src)
        self.assertIn('event.kind != "minecraft:drowned"',self.src)
        self.assertIn("event.level.online_players(host)",self.src)
        self.assertIn("host.living(player.entity())",self.src)
        self.assertIn("living.effects(host)",self.src)
        self.assertIn('effect.id.id(host) == "minecraft:conduit_power"',self.src)
        self.assertIn("*event.result = SpawnCheck::Fail",self.src)
        self.assertIn("conduit_drowned::check(host, event)",self.hooks)

    def test_original_range(self):
        self.assertIn("let dx = first[0] - other[0]",self.src)
        self.assertIn("let dz = first[2] - other[2]",self.src)
        self.assertIn('range("drowned_radius", "Protection Radius", 400., 0., 400., 20.)',self.gui)
        readme=(ROOT/"README.md").read_text()
        self.assertLess(readme.index("The help command gains clearer"),readme.index("Active conduit effects suppress nearby"))
        self.assertIn("- [x] Active conduit effects suppress nearby",readme)
        self.assertIn("- [ ] Placed paintings can be cycled",readme)


if __name__=="__main__":
    unittest.main()
