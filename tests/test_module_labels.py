from pathlib import Path
import json
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = {'unlimited_enchantments': 'Enchantment Levels', 'affordable_anvil': 'Anvil Costs', 'tax_free_levels': 'XP Cost Rules', 'enchantment_level_indicator': 'Max-Level Indicators', 'ench_level_langpatch': 'Level Formatting', 'inventory_totem': 'Inventory Totems', 'inventory_mending': 'XP Item Repair', 'bottled_air': 'Bottled Air', 'what_durability': 'Durability Control', 'gui_clock': 'World Clock', 'gui_compass': 'Navigation Compass', 'vanilla_zoom': 'Camera Zoom', 'hide_hands': 'Hand Visibility', 'vein_miner': 'Vein Mining', 'tree_harvester': 'Tree Harvesting', 'leaf_decay': 'Leaf Decay', 'double_doors': 'Paired Doors', 'automatic_doors': 'Automatic Doors', 'beacon_placement': 'Beacon Placement', 'conduit_placement': 'Conduit Placement', 'spawner_control': 'Spawner Control', 'friendly_griefing': 'Mob Griefing', 'superflat_world_no_slimes': 'Superflat Slime Spawns', 'shulker_drops_two': 'Shulker Shell Drops', 'respawning_shulkers': 'Shulker Respawning', 'always_wither_skull': 'Wither Skull Drops', 'dragon_drops_elytra': 'Dragon Drops', 'infinite_trading': 'Trade Restocking', 'first_join_message': 'First Join Message', 'welcome_message': 'Welcome Messages', 'compact_help': 'Command Help'}
EXPECTED_GROUPS = {'Enchanting': ['unlimited_enchantments', 'affordable_anvil', 'tax_free_levels', 'enchantment_level_indicator', 'ench_level_langpatch'], 'Inventory': ['inventory_totem', 'inventory_mending', 'bottled_air', 'what_durability'], 'Hud': ['gui_clock', 'gui_compass', 'vanilla_zoom', 'hide_hands'], 'Gathering': ['vein_miner', 'tree_harvester', 'leaf_decay'], 'World': ['double_doors', 'automatic_doors', 'beacon_placement', 'conduit_placement', 'spawner_control', 'friendly_griefing', 'superflat_world_no_slimes'], 'Creatures': ['shulker_drops_two', 'respawning_shulkers', 'always_wither_skull', 'dragon_drops_elytra'], 'Trading': ['infinite_trading'], 'Messages': ['first_join_message', 'welcome_message', 'compact_help']}



class ModuleLabelTests(unittest.TestCase):
    def test_module_ids_and_labels(self):
        gui = (ROOT / "src/gui.rs").read_text()
        actual = dict(re.findall(r'ModuleSpec \{ key: "([^"]+)", label: "([^"]+)"', gui))
        self.assertEqual(actual, EXPECTED)
        self.assertEqual(len(set(actual.values())), len(actual))

    def test_groups_are_contiguous_and_complete(self):
        gui = (ROOT / "src/gui.rs").read_text()
        entries = re.findall(
            r'ModuleSpec \{ key: "([^"]+)", label: "[^"]+", options: .*?, category: Category::(\w+) \}',
            gui,
        )
        self.assertEqual(len(entries), len(EXPECTED))
        actual = {}
        for key, group in entries:
            actual.setdefault(group, []).append(key)
        self.assertEqual(actual, EXPECTED_GROUPS)
        self.assertIn('CATEGORY_H', gui)
        self.assertIn('category.label()', gui)
        self.assertIn('if index == 0 || MODULES[index - 1].category != category', gui)
        self.assertNotIn('Grind Enchantments', gui)

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
