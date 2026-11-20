#!/usr/bin/env python3
"""Carry legacy native Config values into the suite's private config directory."""
from pathlib import Path
import argparse
import re
import shutil

ID = 'horizon_qol_suite'


def keys(root: Path) -> set[str]:
    gui = (root / 'src/gui.rs').read_text()
    settings = (root / 'src/settings.rs').read_text()
    alias = dict(re.findall(r'"([A-Za-z0-9]+)"\s*=>\s*"([a-z0-9]+)"', settings))
    modules = re.findall(r'ModuleSpec \{ key: "([^"]+)"', gui)
    options = re.findall(r'(?:toggle|range|choice)\("([^"]+)",', gui)
    assert len(modules) == 44 and len(options) == 108, 'Unexpected settings catalog: migration stopped'
    all_keys = {alias.get(name, name) for name in modules + options}
    assert len(all_keys) == 152 and all(re.fullmatch(r'[a-z0-9_.-]+', name) for name in all_keys)
    return all_keys


def migrate(game: Path, root: Path) -> int:
    previous = game / 'sunrise-mods' / '.sunrise-mod-data' / ID
    if not previous.is_dir():
        return 0
    target = game / 'config' / ID
    moved = 0
    for name in sorted(keys(root)):
        src = previous / name
        dest = target / name
        if not src.is_file() or src.is_symlink() or dest.exists() or dest.is_symlink():
            continue
        target.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
        moved += 1
    # Move the old Tree Harvester fast-decay switch into the new Leaf Decay module.
    # The user's explicit new setting always wins; never overwrite an existing value.
    new_key = target / 'accelerateleafdecay'
    old_keys = [target / 'enablefastleafdecay', previous / 'enablefastleafdecay']
    if not new_key.exists() and not new_key.is_symlink():
        for candidate in old_keys:
            if candidate.is_file() and not candidate.is_symlink():
                target.mkdir(parents=True, exist_ok=True)
                shutil.copy2(candidate, new_key)
                moved += 1
                break
    return moved


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--game', type=Path, required=True)
    args = parser.parse_args()
    print(f'Migrated {migrate(args.game, Path(__file__).resolve().parents[1])} legacy settings (existing files untouched).')
