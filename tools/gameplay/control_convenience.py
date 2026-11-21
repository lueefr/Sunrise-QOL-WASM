import json,subprocess,time,sys,hashlib
from pathlib import Path
import argparse
parser=argparse.ArgumentParser(description='Destructive tests: disposable developer world only')
for key in ('ctl','socket','game-dir','package','output'):parser.add_argument('--'+key,required=True)
args=parser.parse_args()
base=[args.ctl,'--socket',args.socket]
root=Path(args.game_dir) / 'config/sunrise_qol'
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
 cmd('difficulty peaceful');cmd('gamemode survival');cmd('clear @s');cmd('effect clear @s');cmd('kill @e[type=item]');cmd('fill 15 90 -3 32 90 4 stone');cmd('fill 18 91 -2 32 93 2 air');cmd('tp @s 20.5 91 3.5')
 switch('stack_refill',True);cmd('item replace entity @s hotbar.0 with stone 1');cmd('item replace entity @s inventory.0 with stone 10');call('select','1');time.sleep(.2);call('interact','20','90','1','up');step(3)
 r=call('inventory');record('last_placed_block_refills',r['counts'].get('minecraft:stone')==10 and (r.get('held') or {}).get('item')=='minecraft:stone',inventory=r)
 switch('stack_refill',False);cmd('item replace entity @s hotbar.0 with stone 1');cmd('setblock 20 91 1 air');call('interact','20','90','1','up');step(3);r=call('inventory');record('disabled_refill_leaves_hand_empty',(r.get('held') or {}).get('item') is None,inventory=r)
 switch('surface_mushrooms',True);cmd('time set noon');cmd('item replace entity @s hotbar.0 with red_mushroom 2');cmd('setblock 20 91 1 air');time.sleep(.2);call('interact','20','90','1','up');r=call('block','20','91','1');record('bright_surface_mushroom',r['block']=='minecraft:red_mushroom' and call('inventory')['counts'].get('minecraft:red_mushroom')==1,state=r)
 switch('surface_mushrooms',False);cmd('setblock 20 91 1 air');call('interact','20','90','1','up');r=call('block','20','91','1');record('disabled_bright_mushroom_rejected',r['block']=='minecraft:air' and call('inventory')['counts'].get('minecraft:red_mushroom')==1,state=r)
 cmd('clear @s');cmd('setblock 22 91 -1 white_bed[facing=north,part=head]');cmd('setblock 22 91 0 white_bed[facing=north,part=foot]');cmd('tp @s 22.5 91 2.5');switch('sleep_sooner',True);cmd('time set 11000');time.sleep(.2);call('interact','22','91','0');r=call('status');record('too_early_no_clock_advance',11000<=r['day_time']%24000<12000,time=r['day_time'])
 cmd('time set 12000');call('interact','22','91','0');r=call('status');record('early_bed_advances_to_dusk',12540<=r['day_time']%24000<13000,time=r['day_time']);call('key','escape','tap');time.sleep(.2);cmd('tp @s 25.5 91 2.5')
 import re
 def coordinates(lines):
  return [float(x) for x in re.findall(r'(-?\d+(?:\.\d+)?)d', ' '.join(lines))]
 for key,kind,arg in [('move_boats','oak_boat','boat'),('move_minecarts','minecart','minecart')]:
  switch(key,True);cmd('tp @s 25.5 91 2.5');cmd('kill @e[tag=qol_carry]');cmd('summon '+kind+' 25.5 91 0.5 {Tags:["qol_carry"]}');step(3)
  original_uuid=data('entity @e[tag=qol_carry,limit=1]','UUID');cmd('qolcarry '+arg);step(3);cmd('tp @s 28.5 91 2.5');step(3)
  r=data('entity @e[tag=qol_carry,limit=1]','Pos');pos=coordinates(r);uuid=data('entity @e[tag=qol_carry,limit=1]','UUID')
  record(key+'_carried_same_entity',len(pos)==3 and pos[0]>26 and uuid==original_uuid,position=r,uuid=uuid)
  cmd('qolcarry drop');cmd('tp @s 25.5 91 2.5');step(3);r2=data('entity @e[tag=qol_carry,limit=1]','Pos');pos2=coordinates(r2);record(key+'_released',len(pos2)==3 and abs(pos2[0]-pos[0])<1,position=r2)
  cmd('kill @e[tag=qol_carry]');cmd('kill @e[type=item]')

finally:
 for key in ['stack_refill','surface_mushrooms','sleep_sooner','move_boats','move_minecarts']:root.joinpath(key).write_text('false')
 Path(args.output).write_text(json.dumps({'scenarios':rows,'package_sha256':hashlib.sha256(Path(args.package).read_bytes()).hexdigest()},indent=2)+'\n')
