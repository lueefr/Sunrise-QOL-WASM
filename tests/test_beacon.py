"""Beacon placement implementation checks: do not consume a stack without placement."""
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class BeaconTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.src = (ROOT / "src/beacon.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()

    def test_beacon_placement_is_behind_opt_in(self):
        self.assertIn('if !on(host, "beacon_placement")', self.src)
        self.assertIn('player.may_build(host) != Some(true)', self.src)
        self.assertIn('player.is_spectator(host) != Some(false)', self.src)
        self.assertIn('event.click.stack.cloned()', self.src)
        self.assertIn('player.may_use_item_at(host', self.src)
        self.assertIn('player.place_block(host, level, pos, state, true)', self.src)
        self.assertIn('player.set_inventory_slot(host, slot, next)', self.src)
        self.assertIn('if !empty(&block.id(host)) { continue; }', self.src)
        self.assertIn('let _ = level.set_block(host, pos, previous,', self.src)

    def test_base_recovery_is_optional_and_preserves_drops(self):
        self.assertIn('flag(host, "beacon_break_base", true)', self.src)
        self.assertIn('event.state.block().id(host) != "minecraft:beacon"', self.src)
        self.assertIn('player.destroy_block(host, pos, false)', self.src)
        self.assertIn('self.beacon.broken(host, event)', self.hooks)
        self.assertIn('self.beacon.click(host, event)', self.hooks)
        self.assertIn('toggle("beacon_break_base", "Recover Base When Beacon Breaks", true)', self.gui)

    def test_bounded_and_ordered(self):
        self.assertIn('MAX_BASE: usize = 164', self.src)
        self.assertIn('for layer in 1_i32..=4', self.src)
        self.assertIn('if placed >= limit { break; }', self.src)
        self.assertIn('let limit = if many { MAX_BASE } else { 1 }', self.src)
        readme = (ROOT / "README.md").read_text()
        self.assertLess(readme.index('Nearby doors open automatically'), readme.index('Beacon bases can be placed'))
        self.assertIn('- [x] Beacon bases can be placed', readme)
        self.assertIn('- [x] Conduits and their surrounding blocks', readme)


if __name__ == "__main__":
    unittest.main()
