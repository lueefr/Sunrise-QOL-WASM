#!/usr/bin/env python3
"""Destructive Wool Tweaks recipe observations in a disposable developer world."""
import argparse
import hashlib
import json
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
for key in ('ctl', 'socket', 'game-dir', 'package', 'output'):
    parser.add_argument('--' + key, required=True)
args = parser.parse_args()
base = [args.ctl, '--socket', args.socket]
setting = Path(args.game_dir) / 'config/sunrise_qol/wool_tweaks'
rows = []

def call(*command):
    result = subprocess.run(base + list(command), capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return json.loads(result.stdout)

def cmd(command):
    return call('command', command)

def until(check, description, timeout=8):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = check()
        if result:
            return result
        time.sleep(.05)
    raise RuntimeError('Timed out waiting for ' + description)

def record(name, passed, **evidence):
    rows.append({'case': name, 'passed': bool(passed), **evidence})
    print(name, bool(passed), flush=True)
    if not passed:
        raise AssertionError(rows[-1])

def reload(enabled):
    setting.write_text(str(enabled).lower())
    before = call('status')['day_time']
    cmd('tick step 2')
    until(lambda: (call('status')['day_time'] - before) % 24000 >= 2, 'two server ticks')
    cmd('reload')
    time.sleep(.5)

def slot(number):
    return next((value for value in call('screen').get('slots', []) if value['slot'] == number), {})

def carry(expected):
    return (call('screen').get('carried') or {}).get('item') == expected

def move(source, target, item):
    call('click-slot', str(source))
    until(lambda: carry(item), 'cursor ' + item)
    call('click-slot', str(target))
    until(lambda: slot(target).get('item') == item and carry(None), 'grid input ' + item)

def two_by_two(name, inputs, output, count):
    cmd('clear @s')
    for index, item in enumerate(inputs):
        cmd(f'item replace entity @s hotbar.{index} with {item} 1')
    until(lambda: all(call('inventory')['counts'].get(item, 0) == 1 for item in inputs), 'fresh inventory')
    call('open-inventory')
    until(lambda: call('screen')['screen'] == 'inventory', 'inventory')
    for index, item in enumerate(inputs):
        move(index, 100 + index, item)
    result = until(lambda: (value if (value := slot(200)).get('item') == output else None), 'crafting output')
    record(name + '_preview', result.get('count') == count, output=result)
    call('click-slot', '200', 'shift')
    counts = until(lambda: (counts if (counts := call('inventory')['counts']).get(output) == count
                            and all(counts.get(item, 0) == 0 for item in inputs) else None), 'craft consumption')
    record(name + '_take', True, counts=counts)
    call('close')
    until(lambda: call('ui')['screen'] == 'playing', 'closed inventory')

if call('ui')['screen'] != 'playing':
    raise RuntimeError('Resume the disposable world and close every menu')
try:
    cmd('gamemode survival')
    cmd('difficulty peaceful')
    reload(False)
    answer = cmd('recipe give @s sunrise_qol:wool_tweaks/wool_to_string')
    record('disabled_recipe_absent', not answer['success'] and any('Unknown recipe' in line for line in answer['chat']), response=answer)
    reload(True)
    cmd('recipe take @s *')
    recipes = sorted((Path(__file__).resolve().parents[2] / 'data/sunrise_qol/recipe/wool_tweaks').glob('*.json'))
    admitted = []
    for path in recipes:
        answer = cmd('recipe give @s sunrise_qol:wool_tweaks/' + path.stem)
        if answer.get('success'):
            admitted.append(path.stem)
    record('all_49_recipes_admitted', len(admitted) == len(recipes) == 49, recipes=admitted)
    two_by_two('wool_to_four_strings', ['minecraft:white_wool'], 'minecraft:string', 4)
    two_by_two('white_dye_on_colored_wool', ['minecraft:blue_wool', 'minecraft:white_dye'], 'minecraft:white_wool', 1)
    two_by_two('bed_recolor', ['minecraft:red_bed', 'minecraft:blue_dye'], 'minecraft:blue_bed', 1)
    cmd('clear @s')
    cmd('item replace entity @s hotbar.0 with minecraft:red_carpet 8')
    cmd('item replace entity @s hotbar.1 with minecraft:blue_dye 1')
    cmd('fill 1 91 -1 4 91 1 minecraft:air')
    cmd('tp @s 2.5 91 2.5')
    cmd('setblock 2 91 0 minecraft:crafting_table')
    time.sleep(.2)
    call('interact', '2', '91', '0')
    until(lambda: call('screen').get('craft_size') == 3, 'crafting table')
    call('click-slot', '0')
    until(lambda: carry('minecraft:red_carpet'), 'eight carpets')
    for cell in (100, 101, 102, 103, 105, 106, 107, 108):
        call('click-slot', str(cell), 'right')
        until(lambda: slot(cell).get('item') == 'minecraft:red_carpet', 'carpet ring cell')
    until(lambda: carry(None), 'empty cursor')
    move(1, 104, 'minecraft:blue_dye')
    result = until(lambda: (value if (value := slot(200)).get('item') == 'minecraft:blue_carpet' else None), 'eight dyed carpets')
    record('carpet_ring_preview', result.get('count') == 8, output=result)
    call('click-slot', '200', 'shift')
    counts = until(lambda: (value if (value := call('inventory')['counts']).get('minecraft:blue_carpet') == 8
                           and not value.get('minecraft:red_carpet') and not value.get('minecraft:blue_dye') else None), 'carpet consumption')
    record('carpet_ring_take', True, counts=counts)
    call('close')
    until(lambda: call('ui')['screen'] == 'playing', 'closed table')
    reload(False)
    answer = cmd('recipe give @s sunrise_qol:wool_tweaks/wool_to_string')
    record('disabled_after_reload', not answer['success'] and any('Unknown recipe' in line for line in answer['chat']), response=answer)
finally:
    setting.write_text('false')

Path(args.output).write_text(json.dumps({
    'sdk': '94256510f74e6843754e0a93fd9e4da7e45b1056',
    'kind': 'actual-game-control-observations',
    'package_sha256': hashlib.sha256(Path(args.package).read_bytes()).hexdigest(),
    'module': 'wool_tweaks_recipes', 'observations': rows,
    'fully_certified_one_to_one': False,
    'limitations': ['Requires /reload after toggling and after starting a world',
                    'No original Java mod differential run or multiplayer verification'],
}, indent=2) + '\n')
