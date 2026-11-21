#!/usr/bin/env python3
"""Run destructive Wool Tweaks probes only in a disposable developer world."""
import argparse
import hashlib
import json
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
for name in ('ctl', 'socket', 'game-dir', 'package', 'output'):
    parser.add_argument('--' + name, required=True)
args = parser.parse_args()
base = [args.ctl, '--socket', args.socket]

def call(*command):
    result = subprocess.run(base + list(command), capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError({'command': command, 'stdout': result.stdout, 'stderr': result.stderr})
    return json.loads(result.stdout)

def cmd(command):
    return call('command', command)

setting = Path(args.game_dir) / 'config/horizon_qol_suite/wool_tweaks'
if call('ui')['screen'] != 'playing':
    raise RuntimeError('Resume the disposable world and close every menu first')
cmd('fill -5 90 -5 12 90 12 minecraft:stone')
cmd('tp @s 2.5 91 2.5')
cmd('difficulty peaceful')
call('select', '1')
rows = []

def probe(name, block, expected, count, *, enabled=True, creative=False, offhand=False):
    setting.write_text(str(enabled).lower())
    cmd('gamemode ' + ('creative' if creative else 'survival'))
    cmd('clear @s')
    cmd('item replace entity @s ' + ('weapon.offhand' if offhand else 'hotbar.0')
        + ' with minecraft:red_dye 8')
    cmd('setblock 2 91 0 minecraft:' + block)
    call('select', '1')
    time.sleep(.15)
    reply = call('interact', '2', '91', '0')
    state = call('block', '2', '91', '0')['block']
    remaining = call('inventory')['counts'].get('minecraft:red_dye', 0)
    row = {'case': name, 'block': state, 'dye_count': remaining,
           'passed': state == 'minecraft:' + expected and remaining == count}
    rows.append(row)
    print(name, row['passed'], flush=True)
    if not row['passed']:
        raise AssertionError({'observation': row, 'interaction': reply})

try:
    probe('disabled', 'white_wool', 'white_wool', 8, enabled=False)
    probe('survival_wool_spends_one', 'white_wool', 'red_wool', 7)
    probe('same_color_spends_nothing', 'red_wool', 'red_wool', 8)
    probe('survival_carpet_spends_one', 'white_carpet', 'red_carpet', 7)
    probe('creative_spends_nothing', 'white_wool', 'red_wool', 8, creative=True)
    probe('unrelated_block', 'stone', 'stone', 8)
    probe('moss_carpet_is_not_colored_wool_carpet', 'moss_carpet', 'moss_carpet', 8)
    probe('offhand_survival_spends_one', 'white_wool', 'red_wool', 7, offhand=True)
finally:
    setting.write_text('false')

Path(args.output).write_text(json.dumps({
    'sdk': '94256510f74e6843754e0a93fd9e4da7e45b1056',
    'kind': 'actual-game-control-observations',
    'module': 'wool_tweaks',
    'package_sha256': hashlib.sha256(Path(args.package).read_bytes()).hexdigest(),
    'observations': rows,
    'fully_certified_one_to_one': False,
    'limitations': ['No original-mod differential run, multiplayer or custom species',
                    'Bed dyeing and recipe changes are not implemented'],
}, indent=2) + '\n')
