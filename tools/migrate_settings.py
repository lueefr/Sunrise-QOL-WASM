#!/usr/bin/env python3
"""Migrate old Sunrise QOL identities without overwriting user settings."""
from pathlib import Path
import argparse
import os
import re
import shutil
import tempfile

ID = "sunrise_qol"
LEGACY_ID = "horizon_qol_suite"


def keys(root: Path) -> set[str]:
    gui = (root / "src/gui.rs").read_text()
    settings = (root / "src/settings.rs").read_text()
    alias = dict(re.findall(r'"([A-Za-z0-9]+)"\s*=>\s*"([a-z0-9]+)"', settings))
    modules = re.findall(r'ModuleSpec \{ key: "([^"]+)"', gui)
    options = re.findall(r'(?:toggle|range|choice|text)\("([^"]+)",', gui)
    assert len(modules) == 56 and len(options) == 124, "Unexpected settings catalog: migration stopped"
    all_keys = {alias.get(name, name) for name in modules + options}
    assert len(all_keys) == 180 and all(re.fullmatch(r"[a-z0-9_.-]+", name) for name in all_keys)
    return all_keys


def copy_value(src: Path, dest: Path) -> bool:
    if src.is_symlink() or not src.is_file() or dest.exists() or dest.is_symlink():
        return False
    if dest.parent.is_symlink() or (dest.parent.exists() and not dest.parent.is_dir()):
        raise ValueError("Unsafe Sunrise QOL destination: " + str(dest.parent))
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dest)
    return True


def migrate(game: Path, root: Path) -> int:
    """Copy private old settings first, then older native values, never replace new ones."""
    names = keys(root)
    target = game / "config" / ID
    old_private = game / "config" / LEGACY_ID
    old_native = game / "sunrise-mods" / ".sunrise-mod-data" / LEGACY_ID
    sources = (old_private, old_native)
    moved = 0
    for source in sources:
        if source.is_symlink() or not source.is_dir():
            continue
        for name in sorted(names):
            moved += copy_value(source / name, target / name)
    # Preserve the former Tree Harvester leaf-decay setting if it has not been set.
    new_key = target / "accelerateleafdecay"
    if not new_key.exists() and not new_key.is_symlink():
        for candidate in (target / "enablefastleafdecay",
                          old_private / "enablefastleafdecay",
                          old_native / "enablefastleafdecay"):
            if copy_value(candidate, new_key):
                moved += 1
                break
    return moved


def activate(game: Path) -> bool:
    """Replace the mod identity and retire the old package only after the new one exists."""
    mods = game / "sunrise-mods"
    current = mods / (ID + ".smod")
    legacy = mods / (LEGACY_ID + ".smod")
    if current.is_symlink() or not current.is_file() or current.stat().st_size == 0:
        raise ValueError("Verified sunrise_qol.smod must be installed before activation")
    if legacy.is_symlink():
        raise ValueError("Refusing to rename a symlinked legacy mod")
    config = game / "sunrise-mods.txt"
    if config.is_symlink():
        raise ValueError("Refusing to modify a symlinked sunrise-mods.txt")
    original = config.read_text() if config.exists() else ""
    lines = [line for line in original.splitlines()
             if not re.match(r"^mod\.(?:horizon_qol_suite|sunrise_qol)\s*:", line.strip())]
    updated = "\n".join(lines) + ("\n" if lines else "") + "mod.sunrise_qol:true\n"
    # A retired .smod must not remain loadable, but keep the old package for rollback.
    backup = None
    if legacy.is_file():
        backup = mods / (LEGACY_ID + ".smod.bak")
        index = 1
        while backup.exists() or backup.is_symlink():
            backup = mods / (LEGACY_ID + ".smod.bak." + str(index))
            index += 1
        legacy.rename(backup)
    try:
        if updated != original:
            if config.exists():
                saved = config.with_name(config.name + ".bak")
                if not saved.exists() and not saved.is_symlink():
                    shutil.copy2(config, saved)
            game.mkdir(parents=True, exist_ok=True)
            fd, temp = tempfile.mkstemp(prefix=".sunrise-mods-", dir=game)
            try:
                with os.fdopen(fd, "w", encoding="utf-8") as stream:
                    stream.write(updated)
                os.replace(temp, config)
            finally:
                if os.path.exists(temp):
                    os.unlink(temp)
    except Exception:
        if backup is not None:
            backup.rename(legacy)
        raise
    return updated != original or backup is not None


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--game", type=Path, required=True)
    parser.add_argument("--activate", action="store_true",
                        help="Enable the already installed sunrise_qol.smod and retire its old ID")
    args = parser.parse_args()
    if args.activate:
        print("Sunrise QOL identity activated." if activate(args.game)
              else "Sunrise QOL identity already active.")
    else:
        copied = migrate(args.game, Path(__file__).resolve().parents[1])
        print(f"Migrated {copied} settings into {ID} (existing files untouched).")


if __name__ == "__main__":
    main()
