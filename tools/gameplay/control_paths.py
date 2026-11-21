import json,subprocess,time,sys,hashlib
from pathlib import Path
import argparse
parser=argparse.ArgumentParser(description='Destructive tests: disposable developer world only')
for key in ('ctl','socket','game-dir','package','output'):parser.add_argument('--'+key,required=True)
args=parser.parse_args()
base=[args.ctl,'--socket',args.socket]
root=Path(args.game_dir) / 'config/horizon_qol_suite'
rows=[]
def call(*args):
 p=subprocess.run(base+list(args),capture_output=True,text=True);r=json.loads(p.stdout)
 if p.returncode:raise RuntimeError(r)
 return r
def cmd(s):
 r=call('command',s)
 if s.startswith(('setblock ','fill ','tp ','item replace ','summon ')):time.sleep(.25)
 return r
def step(n):
 before=call('status')['day_time'];cmd(f'tick step {n}');deadline=time.monotonic()+n/20+8
 while (call('status')['day_time']-before)%24000<n:
  if time.monotonic()>deadline:raise TimeoutError('ticks')
  time.sleep(.05)
def switch(key,value):root.joinpath(key).write_text(str(value).lower());step(2)
def record(name,ok,**e):
 rows.append(dict(case=name,passed=bool(ok),**e));print(name,bool(ok),e,flush=True)
 if not ok:raise AssertionError(name)
def data(target,path):return cmd(f'data get {target} {path}')['chat']
try:
 cmd('difficulty peaceful');cmd('gamemode survival');cmd('clear @s');cmd('fill 38 90 -2 47 90 4 stone');cmd('fill 40 90 0 44 90 0 grass_block');cmd('fill 39 91 -1 45 94 2 air');cmd('tp @s 42.5 91 3.5');cmd('item replace entity @s hotbar.0 with diamond_shovel');call('select','1')
 def states():return [call('block',str(x),'90','0')['block'] for x in range(40,44)]
 switch('quick_paths',False);call('key','shift','down');time.sleep(.2)
 try:call('interact','40','90','0','up')
 finally:call('key','shift','up')
 step(2);r=states();record('disabled_only_vanilla_clicked_block',r==['minecraft:dirt_path']+['minecraft:grass_block']*3,blocks=r)
 cmd('setblock 40 90 0 grass_block');cmd('item replace entity @s hotbar.0 with diamond_shovel');switch('quick_paths',True);call('key','shift','down');time.sleep(.2)
 try:call('interact','40','90','0','up')
 finally:call('key','shift','up')
 step(2);r=states();record('start_selection_keeps_ground',r==['minecraft:grass_block']*4,blocks=r)
 call('interact','43','90','0','up');step(2);r=states();record('route_creates_four_paths',r==['minecraft:dirt_path']*4,blocks=r)
 damage=data('entity @s','SelectedItem');record('route_costs_four_durability',any('damage: 4' in x or '"minecraft:damage": 4' in x for x in damage),item=damage)
 call('interact','43','90','0','up');step(2);r=states();record('endpoint_undo_restores_grass',r==['minecraft:grass_block']*4,blocks=r)

finally:
 for key in ['quick_paths']:root.joinpath(key).write_text('false')
 Path(args.output).write_text(json.dumps({'scenarios':rows,'package_sha256':hashlib.sha256(Path(args.package).read_bytes()).hexdigest()},indent=2)+'\n')
