"""Conduit frame and conservation checks for the third README feature."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class ConduitTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src = (ROOT / "src/conduit.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()

    def test_frame_has_bounded_unique_positions(self):
        self.assertIn("FRAME_SIZE: usize = 42", self.src)
        self.assertIn("if !result.contains(&pos) { result.push(pos); }", self.src)
        self.assertIn("if a.abs() != 2 && b.abs() != 2 { continue; }", self.src)
        self.assertIn("fn frame(center: [i32; 3])", self.src)

    def test_secure_inventory_decrement_and_rollback(self):
        self.assertIn('if !on(host, "conduit_placement")', self.src)
        self.assertIn("player.may_build(host) != Some(true)", self.src)
        self.assertIn("player.may_use_item_at(host", self.src)
        self.assertIn("player.place_block(host, level, pos, state, true)", self.src)
        self.assertIn("player.set_inventory_slot(host, slot, next)", self.src)
        self.assertIn("level.set_block(host, pos, previous", self.src)
        self.assertIn("if !replaceable(&block.id(host)) { continue; }", self.src)
        self.assertIn("if placed >= limit { break; }", self.src)

    def test_frame_recovery_only_when_module_enabled(self):
        self.assertIn('flag(host, "conduit_break_frame", true)', self.src)
        self.assertIn('event.state.block().id(host) != "minecraft:conduit"', self.src)
        self.assertIn("player.destroy_block(host, pos, false)", self.src)
        self.assertIn("self.conduit.click(host, event)", self.hooks)
        self.assertIn("self.conduit.broken(host, event)", self.hooks)
        self.assertIn('toggle("conduit_break_frame", "Recover Frame When Conduit Breaks", true)', self.gui)

    def test_order_in_readme(self):
        readme = (ROOT / "README.md").read_text()
        self.assertLess(readme.index("Beacon bases can be placed"), readme.index("Conduits and their surrounding blocks"))
        self.assertIn("- [x] Conduits and their surrounding blocks", readme)
        self.assertIn("- [ ] Mob spawners can be disabled", readme)


if __name__ == "__main__":
    unittest.main()
