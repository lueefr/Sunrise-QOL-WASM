from pathlib import Path
import json
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {'inventory_totem': 'Totem Protection', 'inventory_mending': 'Item Repair', 'double_doors': 'Linked Doors', 'infinite_trading': 'Trading', 'shulker_drops_two': 'Shulker Loot', 'respawning_shulkers': 'Shulker Respawns', 'enchantment_level_indicator': 'Enchantment Caps', 'what_durability': 'Durability', 'always_wither_skull': 'Wither Skull Drops', 'friendly_griefing': 'Mob Griefing', 'gui_clock': 'Clock', 'gui_compass': 'Compass', 'vanilla_zoom': 'Zoom', 'dragon_drops_elytra': 'Dragon Loot', 'hide_hands': 'Hand Visibility', 'first_join_message': 'First Join', 'welcome_message': 'Join Messages', 'vein_miner': 'Vein Mining', 'tree_harvester': 'Tree Cutting', 'leaf_decay': 'Leaves', 'ench_level_langpatch': 'Level Formatting', 'superflat_world_no_slimes': 'Superflat Slimes', 'tax_free_levels': 'XP Costs', 'unlimited_enchantments': 'Enchantments', 'affordable_anvil': 'Anvils'}


class ModuleLabelTests(unittest.TestCase):
    def test_module_ids_and_labels(self):
        gui = (ROOT / "src/gui.rs").read_text()
        actual = dict(re.findall(r'ModuleSpec \{ key: "([^"]+)", label: "([^"]+)"', gui))
        self.assertEqual(actual, EXPECTED)
        self.assertEqual(len(set(actual.values())), len(actual))

    def test_locales(self):
        for lang in ("en_us", "pt_br"):
            values = json.loads((ROOT / f"assets/horizon_qol_suite/lang/{lang}.json").read_text())
            for key, label in EXPECTED.items():
                i18n = f"qol.module.{key}"
                if i18n in values:
                    self.assertEqual(values[i18n], label)
                legacy = f"horizon_qol_suite.option.{key}"
                if legacy in values:
                    self.assertEqual(values[legacy], label + ": ON/OFF")

    def test_legacy_ids_not_renamed(self):
        source = (ROOT / "src/gui.rs").read_text()
        self.assertIn('const SCREEN_ID: &str = "horizon_qol_suite"', source)
        for key in EXPECTED:
            self.assertIn(f'ModuleSpec {{ key: "{key}",', source)
        self.assertIn('"unlimited_max_level"', source)
        self.assertIn('"unlimited_max_anvil_cost"', source)
        self.assertNotIn('Accelerated Decay (Leaves Be Gone)', source)


if __name__ == "__main__":
    unittest.main()
