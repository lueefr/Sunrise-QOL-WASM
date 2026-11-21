#!/usr/bin/env python3
"""Manual real-game probe. Run only against a disposable developer world."""
import argparse, json, subprocess, time, hashlib
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--ctl', required=True)
parser.add_argument('--socket', required=True)
parser.add_argument('--game-dir', type=Path, required=True)
parser.add_argument('--package', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args=parser.parse_args()
base=[args.ctl, '--socket', args.socket]
def call(args):
 p=subprocess.run(base+args,capture_output=True,text=True)
 d=json.loads(p.stdout)
 if p.returncode:raise RuntimeError(d)
 return d
def cmd(s):return call(['command',s])
def step(n):
 before=call(['status'])['day_time'];cmd('tick step '+str(n));deadline=time.monotonic()+n*.05+10
 while time.monotonic()<deadline:
  if (call(['status'])['day_time']-before)%24000>=n:
   time.sleep(.08);return
  time.sleep(.05)
 raise RuntimeError('world did not complete requested steps')
config=args.game_dir/'config/sunrise_qol'
def setting(key,value):
 config.mkdir(exist_ok=True);(config/key).write_text(value)
def door(kind='oak_door'):
 cmd('fill 0 91 0 0 92 0 minecraft:air')
 for y,half in [(91,'lower'),(92,'upper')]:cmd(f'setblock 0 {y} 0 minecraft:{kind}[half={half},facing=north,hinge=left,open=false,powered=false]')
observations=[]
def expect(key,open):
 states=[call(['block','0',str(y),'0'])['state'] for y in [91,92]]
 ok=all(('open='+str(open).lower()) in s for s in states)
 observations.append({'case':key,'passed':ok,'expected_open':open,'states':states})
 print(key,ok)
 if not ok:raise AssertionError(observations[-1])
state=call(['ui'])
if state.get('screen') != 'playing':
 raise RuntimeError('Close every menu and resume the disposable world before running this probe')
cmd('gamemode creative')
cmd('fill -5 90 -5 12 90 12 minecraft:stone')
cmd('fill -4 91 -4 4 94 4 minecraft:air')
setting('automatic_doors','false');step(1);door();cmd('tp @s 0.5 91 1.5');step(2);expect('disabled',False)
setting('automatic_doors','true');step(2);expect('approach_opens_both_halves',True)
cmd('tp @s 10.5 91 10.5');step(10);expect('whole_second_delay_not_expired',True)
step(35);expect('departure_closes_after_delay',False)
cmd('tp @s 0.5 91 1.5');step(2);expect('return_reopens',True)
step(45);expect('nearby_player_prevents_closure',True)
cmd('tp @s 10.5 91 10.5');step(2);expect('departure_after_expiry_closes',False)
setting('automatic_doors','false');step(1);door('iron_door');setting('auto_iron_doors','false');setting('automatic_doors','true');cmd('tp @s 0.5 91 1.5');step(2);expect('iron_excluded',False)
setting('auto_iron_doors','true');step(2);expect('iron_included',True)
setting('automatic_doors','false');step(1)
result={'sdk':'94256510f74e6843754e0a93fd9e4da7e45b1056','runtime':'nightly-2026-11-20-9425651','kind':'actual-game-control-observations','package_sha256':hashlib.sha256(args.package.read_bytes()).hexdigest(),'module':'automatic_doors','observations':observations,'fully_certified_one_to_one':False,'limitations':['Single integrated-server player; no multiplayer differential reference execution','Does not verify redstone, custom doors, all entity query boundaries or all modules']}
args.output.write_text(json.dumps(result,indent=2)+'\n')
