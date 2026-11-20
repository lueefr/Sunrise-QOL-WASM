"""Source-behavior regression checks for four corrected modules, not gameplay certification."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class BatchParityTests(unittest.TestCase):
    def test_bottled_air_consumption_and_fire_rules(self):
        src = (ROOT / "src/bottled_air.rs").read_text()
        for part in ('"minecraft:potion_contents"', 'entity.is_under_water(host)',
                     '"minecraft:water"', 'InteractionResult::Fail',
                     '(1..=35).contains(slot)', 'random.next_double() > chance',
                     'player.give_stack(host, stack)'):
            self.assertIn(part, src)

    def test_milk_is_player_only_with_decimal_saturation(self):
        src = (ROOT / "src/milk.rs").read_text()
        gui = (ROOT / "src/gui.rs").read_text()
        self.assertIn("event.entity.is_some()", src)
        self.assertIn("food.saturation + saturation", src)
        self.assertIn('range("saturationLevelIncrease", "Saturation Restored", 10., 0., 20., 0.1)', gui)

    def test_zombies_do_not_lose_door_opening(self):
        src = (ROOT / "src/zombie.rs").read_text()
        self.assertIn('mob.remove_goal(host, "BreakDoor")', src)
        self.assertNotIn('mob.set_can_open_doors(host, false)', src)
        self.assertIn('assert!(!zombie("minecraft:zombified_piglin"))', src)

    def test_extended_bone_meal_wart_and_modded_crops(self):
        src = (ROOT / "src/extended_bonemeal.rs").read_text()
        self.assertIn('id != "minecraft:nether_wart"', src)
        self.assertIn("event.valid_bonemeal_target", src)
        self.assertIn("fn modded_age(", src)


if __name__ == "__main__":
    unittest.main()
