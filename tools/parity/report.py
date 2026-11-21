#!/usr/bin/env python3
"""Machine-readable status for every configured QOL module, without false certification."""
import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NAMES = {
    "underwater_enchanting": "Underwater Enchanting",
    "quick_paths": "Quick Paths",
    "move_boats": "Move Boats",
    "move_minecarts": "Move Minecarts",

    "stack_refill": "Stack Refill",
    "sleep_sooner": "Sleep Sooner",
    "surface_mushrooms": "Surface Mushrooms",

    "furnace_recycle": "Furnace Recycle",
    "kelp_fertilizer": "Kelp Fertilizer",
    "no_feather_trample": "No Feather Trample",
    "thorny_bush_protection": "Thorny Bush Protection",
    "village_bell_recipe": "Village Bell Recipe",

    "wool_tweaks": "Wool Tweaks",
    "unlimited_enchantments": "Unlimited Enchantments",
    "enchant_command": "Extended Enchant Command",
    "affordable_anvil": "Affordable Anvil",
    "tax_free_levels": "Tax Free Levels",
    "enchantment_level_indicator": "Enchantment Level Indicator",
    "ench_level_langpatch": "Ench. Level LangPatch",
    "inventory_totem": "Inventory Totem",
    "inventory_mending": "Inventory Mending",
    "bottled_air": "Bottled Air",
    "nutritious_milk": "Nutritious Milk",
    "what_durability": "What Durability",
    "gui_clock": "GUI Clock",
    "gui_compass": "GUI Compass",
    "vanilla_zoom": "Vanilla Zoom",
    "hide_hands": "Hide Hands",
    "vein_miner": "Vein Miner",
    "tree_harvester": "Tree Harvester",
    "leaf_decay": "Leaf Decay",
    "extended_bonemeal": "Extended Bone Meal",
    "hoe_tweaks": "Hoe Tweaks",
    "right_click_harvest": "Right Click Harvest",
    "double_doors": "Double Doors",
    "automatic_doors": "Automatic Doors",
    "zombie_proof_doors": "Zombie-Proof Doors",
    "beacon_placement": "Beacon Placement",
    "conduit_placement": "Conduit Placement",
    "keep_soil_tilled": "Keep My Soil Tilled",
    "softer_hay_bales": "Softer Hay Bales",
    "spawner_control": "Spawner Control",
    "cycle_paintings": "Painting Cycling",
    "dismount_entity": "Passenger Removal",
    "ground_elytra_fireworks": "Ground Elytra Fireworks",
    "followers_teleport": "Followers Teleport Too",
    "friendly_griefing": "Friendly Griefing",
    "superflat_world_no_slimes": "Superflat World No Slimes",
    "drowned_prevention": "Drowned Prevention",
    "shulker_drops_two": "Shulker Drops Two",
    "respawning_shulkers": "Respawning Shulkers",
    "always_wither_skull": "Always a Wither Skull",
    "dragon_drops_elytra": "Dragon Drops Elytra",
    "infinite_trading": "Infinite Trading",
    "first_join_message": "First Join Message",
    "welcome_message": "Welcome Message",
    "compact_help": "Command Help",
}
MODULE = re.compile(r'ModuleSpec \{ key: "([^"]+)", label: "([^"]+)"')
TABLE = re.compile(r'^\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*(.*?)\s*\|\s*$', re.M)


def inventory(root=ROOT):
    gui = (root / "src/gui.rs").read_text()
    parity = (root / "docs/PARITY.md").read_text()
    audit = (root / "docs/UPSTREAM-AUDIT.md").read_text()
    listed = dict(MODULE.findall(gui))
    matrix = [(name, status, details) for name, status, details
              in TABLE.findall(parity) if name not in ("Module", "---")]
    rows = {name: (status, details) for name, status, details in matrix}
    reviewed = {line.split("|")[1].strip() for line in audit.splitlines()
                if line.startswith("| ") and "github.com/" in line}
    missing = set(listed) - set(NAMES)
    extra = set(NAMES) - set(listed)
    if missing or extra:
        raise ValueError(f"Unmapped GUI modules {sorted(missing)}; obsolete mappings {sorted(extra)}")
    if len(rows) != len(matrix):
        raise ValueError("Repeated rows in parity matrix")
    absent = sorted((key, NAMES[key]) for key in listed if NAMES[key] not in rows)
    if absent:
        raise ValueError(f"Undocumented QOL module: {absent}")
    summary = []
    for key, label in listed.items():
        source_status, differences = rows[NAMES[key]]
        if not differences.strip():
            raise ValueError(f"Module {key} has no recorded limitations")
        summary.append({
            "key": key,
            "ui_label": label,
            "parity_row": NAMES[key],
            "source_status": source_status,
            "source_file_audited": NAMES[key] in reviewed,
            "known_differences": differences,
            "certified_one_to_one": False,
            "gameplay_matrix_complete": False,
        })
    return {
        "schema": 1,
        "module_count": len(summary),
        "source_audit_rows": sum(entry["source_file_audited"] for entry in summary),
        "fully_certified": 0,
        "other_requested_features_not_configured": [
            "Full Brightness Toggle",
        ],
        "modules": summary,
        "warning": "Source matrix and partial gameplay scenarios do not establish 1:1 equivalence.",
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    info = inventory()
    body = json.dumps(info, indent=2, ensure_ascii=False) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(body)
    else:
        print(body, end="")


if __name__ == "__main__":
    main()
