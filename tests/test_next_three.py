"""Focused checks for the next three README features, in original order."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class NextThreeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.painting = (ROOT / "src/painting.rs").read_text()
        cls.dismount = (ROOT / "src/dismount.rs").read_text()
        cls.elytra = (ROOT / "src/elytra.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()

    def test_readme_only_next_three(self):
        readme = (ROOT / "README.md").read_text()
        lines = readme.split("### Bundle-inspired quality-of-life features")[1].split(
            "### Additional quality-of-life features")[0]
        items = re.findall(r"^- \[([ x])\] (.*)$", lines, re.MULTILINE)
        names = [label for _, label in items]
        a = next(i for i, s in enumerate(names) if s.startswith("Placed paintings"))
        self.assertTrue(all(items[i][0] == "x" for i in range(a, a + 3)))
        self.assertTrue(names[a + 1].startswith("Passengers can be removed"))
        self.assertTrue(names[a + 2].startswith("Fireworks can launch"))
        self.assertEqual(items[a + 3][0], " ")

    def test_painting_same_size_and_retains_content(self):
        self.assertIn("variant_size(host, name) != Some(size)", self.painting)
        self.assertIn("PackType::ServerData", self.painting)
        self.assertIn("painting.nbt(host)", self.painting)
        self.assertIn('nbt.insert("variant".into()', self.painting)
        self.assertIn('host.summon(&dimension, "minecraft:painting", pos, nbt)', self.painting)
        self.assertIn("painting.discard(host)", self.painting)
        self.assertIn("MAX_VARIANTS: usize = 128", self.painting)
        self.assertIn('name.starts_with(\'#\')', self.painting)

    def test_dismount_requires_sneak_and_protects_players(self):
        self.assertIn('if !on(host, "dismount_entity")', self.dismount)
        self.assertIn("player.is_secondary_use_active(host) != Some(true)", self.dismount)
        self.assertIn('flag(host, "dismount_players", false)', self.dismount)
        self.assertIn('passenger.kind().id(host) == "minecraft:player"', self.dismount)
        self.assertIn("passenger.stop_riding(host)", self.dismount)
        self.assertIn("passengers.into_iter().take(8)", self.dismount)

    def test_elytra_uses_native_rocket(self):
        self.assertIn('if !host.playing() || !on(host, "ground_elytra_fireworks")', self.elytra)
        self.assertIn('event.click.stack.item().id(host) != "minecraft:firework_rocket"', self.elytra)
        self.assertIn('stack.item().id(host) == "minecraft:elytra"', self.elytra)
        self.assertIn('host.push_player([0., lift, 0.])', self.elytra)
        self.assertIn('Identifier::parse("minecraft:player_command")', self.elytra)
        self.assertIn('let _ = host.click_mapping("key.use")', self.elytra)
        self.assertIn("MAX_TRIES: u8 = 6", self.elytra)
        self.assertIn("TIMEOUT_MS: u64 = 2400", self.elytra)
        self.assertIn("self.elytra.reset()", self.hooks)

    def test_ui_is_still_single_scroll_all_off(self):
        self.assertIn('ModuleSpec { key: "cycle_paintings", label: "Painting Cycling"', self.gui)
        self.assertIn('ModuleSpec { key: "dismount_entity", label: "Passenger Removal"', self.gui)
        self.assertIn('ModuleSpec { key: "ground_elytra_fireworks", label: "Ground Elytra Fireworks"', self.gui)
        self.assertIn('toggle("dismount_players", "Allow Removing Player Passengers", false)', self.gui)
        self.assertIn('range("elytra_launch_strength", "Ground Lift Strength (%)", 35., 20., 80., 5.)', self.gui)


if __name__ == "__main__":
    unittest.main()
