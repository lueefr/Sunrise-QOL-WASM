#!/usr/bin/env python3
"""Check the built guest against the shared standard library before Wasmtime loads it."""
from __future__ import annotations

import json
import sys
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from wasm_link_check import contents, symbols


def verify(package: Path, standard: Path) -> dict:
    imports, guest_exports = symbols(contents(package))
    _, std_exports = symbols(contents(standard))
    known = {(entry["symbol"], entry["kind"]) for entry in guest_exports + std_exports}
    rust_imports = [
        entry for entry in imports
        if entry["module"] == "env" and entry["symbol"].startswith("_R")
    ]
    missing = sorted({
        entry["symbol"] for entry in rust_imports
        if (entry["symbol"], entry["kind"]) not in known
    })
    return {
        "package": str(package),
        "standard": str(standard),
        "rust_imports": len(rust_imports),
        "standard_exports": len(std_exports),
        "missing_rust_symbols": missing,
        "passed": len(missing) == 0,
    }


HARVEST_TAGS = (
    "block/blacklist.json",
    "block/hoe_never_required.json",
    "block/radius_harvest_blacklist.json",
    "item/low_tier_hoes.json",
    "item/mid_tier_hoes.json",
    "item/high_tier_hoes.json",
)


def verify_harvest_tags(package: Path) -> dict:
    """Verify the original tag datasets are actually shipped in the .smod."""
    prefix = "data/rightclickharvest/tags/"
    missing = []
    different = []
    with ZipFile(package) as archive:
        files = set(archive.namelist())
        for name in HARVEST_TAGS:
            member = prefix + name
            source = ROOT / member
            if member not in files:
                missing.append(member)
            elif archive.read(member) != source.read_bytes():
                different.append(member)
    return {
        "harvest_tag_files": len(HARVEST_TAGS),
        "missing_harvest_tags": missing,
        "modified_harvest_tags": different,
        "harvest_tags_passed": not missing and not different,
    }


def verify_identity(package: Path) -> dict:
    """The renamed mod must bundle no obsolete resource or recipe namespace."""
    with ZipFile(package) as archive:
        files = set(archive.namelist())
        required = {
            "assets/sunrise_qol/lang/en_us.json",
            "assets/sunrise_qol/lang/pt_br.json",
            "data/sunrise_qol/recipe/wool_tweaks/black_wool.json",
        }
        missing = sorted(required - files)
        legacy = sorted(name for name in files
                        if name.startswith(("assets/horizon_qol_suite/",
                                            "data/horizon_qol_suite/")))
        recipes = [name for name in files
                   if name.startswith("data/sunrise_qol/recipe/") and name.endswith(".json")]
        wrong = [name for name in recipes
                 if b'"horizon_qol_suite:' in archive.read(name)
                 or b'"sunrise_qol:' not in archive.read(name)]
    return {
        "missing_namespaced_files": missing,
        "legacy_namespaced_files": legacy,
        "wrong_recipe_conditions": sorted(wrong),
        "namespaced_recipe_count": len(recipes),
        "identity_passed": not missing and not legacy and not wrong and len(recipes) == 76,
    }


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("Usage: python tools/sandbox/check.py package.smod sunrise-wasm-std.wasm")
    result = verify(Path(sys.argv[1]), Path(sys.argv[2]))
    result.update(verify_harvest_tags(Path(sys.argv[1])))
    result.update(verify_identity(Path(sys.argv[1])))
    print(json.dumps(result, indent=2))
    if not result["passed"]:
        sys.exit("Unresolved Rust symbols: the .smod is incompatible with this std member")
    if not result["harvest_tags_passed"]:
        sys.exit("The .smod does not contain all original Right Click Harvest tags")
    if not result["identity_passed"]:
        sys.exit("The .smod still has legacy namespaces or missing Sunrise QOL assets")


if __name__ == "__main__":
    main()
