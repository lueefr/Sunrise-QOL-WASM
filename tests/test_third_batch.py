"""Regression checks for only the next README trio."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ThirdBatchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.enchant = (ROOT / "src/enchant_command.rs").read_text()
        cls.meal = (ROOT / "src/extended_bonemeal.rs").read_text()
        cls.followers = (ROOT / "src/followers.rs").read_text()
        cls.hooks = (ROOT / "src/lib.rs").read_text()
        cls.gui = (ROOT / "src/gui.rs").read_text()

    def test_readme_exact_three(self):
        text = (ROOT / "README.md").read_text()
        block = text.split("### Bundle-inspired quality-of-life features")[1].split(
            "### Additional quality-of-life features")[0]
        checks = re.findall(r"^- \[([ x])\] (.*)$", block, re.M)
        index = next(i for i, (_, line) in enumerate(checks) if line.startswith("Commands can apply enchantments"))
        self.assertEqual([v for v, _ in checks[index:index + 3]], ["x", "x", "x"])
        self.assertEqual(checks[index + 3][0], "x")

    def test_enchant_permission_and_limits(self):
        for term in ('"enchant_command"', 'permission_level(host).unwrap_or(0) < 2',
                     'fn parse(', 'MAX_LEVEL: u32 = 255', 'MAX_QUEUE: usize = 64',
                     'ComponentId::new("minecraft:enchantments")', 'enchantments_compatible(',
                     'player.set_inventory_slot(host, slot, Some(next))'):
            self.assertIn(term, self.enchant)
        self.assertIn('self.enchant_command.command(host, event)', self.hooks)
        self.assertIn('self.enchant_command.tick(host, event)', self.hooks)

    def test_bonemeal_authoritative_consumption(self):
        for term in ['HostSide::Server(host)', 'event.player != Some(player.uuid())',
                     'player.is_creative(host)', 'event.successful = true',
                     'if left == 0 { *event.stack = None; }',
                     'minecraft:nether_wart', 'minecraft:sugar_cane', 'minecraft:cactus',
                     'minecraft:vine', 'next_age(']:
            self.assertIn(term, self.meal)
        self.assertIn('extended_bonemeal::apply(side, event)', self.hooks)

    def test_followers_enforce_owner_and_success(self):
        for term in ['MAX_PETS: usize = 32', 'owner(&nbt, uuid)',
                     'sitting(&nbt)', 'LeashHolder::Unleashed',
                     'pending.target != event.dimension',
                     'pet.teleport_to_level(host, &pending.target, to, None)']:
            self.assertIn(term, self.followers)
        self.assertIn('self.followers.before(host, event)', self.hooks)
        self.assertIn('self.followers.after(host, event)', self.hooks)

    def test_modules_off_by_default(self):
        for key in ("enchant_command", "extended_bonemeal", "followers_teleport"):
            self.assertIn(f'ModuleSpec {{ key: "{key}",', self.gui)
        self.assertIn('range("followers_radius", "Follower Search Radius", 50., 1., 256., 1.)', self.gui)


if __name__ == "__main__":
    unittest.main()
