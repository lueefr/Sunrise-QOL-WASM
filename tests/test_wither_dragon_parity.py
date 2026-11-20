"""Source-parity regressions for Wither Skull and Dragon Drops Elytra.

These checks verify the presence of engine hooks and independently tested
selection helpers, not actual Minecraft gameplay certification.
"""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class WitherDragonParityTests(unittest.TestCase):
    def test_wither_tasks_are_not_lost_during_busy_ticks(self):
        src = (ROOT / "src/wither_skull.rs").read_text()
        self.assertIn("pending.front().is_some_and", src)
        self.assertIn("pending.pop_front()", src)
        self.assertNotIn("PER_TICK", src)
        self.assertNotIn("if pending.len() >= CAP", src)
        self.assertIn('kind == "minecraft:wither_skeleton_skull" && age < 10', src)
        self.assertIn('event.kind != "minecraft:wither_skeleton"', src)

    def test_dragon_killer_and_world_query_order(self):
        src = (ROOT / "src/dragon_drops.rs").read_text()
        lib = (ROOT / "src/lib.rs").read_text()
        self.assertIn("LivingDamagePostEvent", src)
        self.assertIn("dragon.cause = Some(event.source.causing)", src)
        self.assertIn("event.container.health_damage() <= 0.", src)
        self.assertIn("level.entities_in(host, min, max)", src)
        self.assertIn("fn recipient(", src)
        self.assertIn("fn living_damage_post(", lib)
        self.assertIn("self.dragon_drops.damage(event)", lib)
        self.assertLess(src.index("let broadcast = translate("),
                        src.index('Identifier::parse("minecraft:elytra")'))


if __name__ == "__main__":
    unittest.main()
