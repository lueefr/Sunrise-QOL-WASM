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
def cmd(s):return call('command',s)
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
 cmd('difficulty peaceful');cmd('gamemode survival');cmd('clear @s');cmd('effect clear @s');cmd('tp @s 20.5 91 0.5');cmd('fill 15 90 -3 30 90 3 stone')
 for key,recipe in [('furnace_recycle','furnace_recycle/smelt_bucket'),('village_bell_recipe','village_bell_recipe/bell')]:
  switch(key,False);cmd('reload');time.sleep(.5)
  r=cmd('recipe give @s horizon_qol_suite:'+recipe);record(key+'_off',not r['success'],response=r)
  switch(key,True);cmd('reload');time.sleep(.5);cmd('recipe take @s *')
  r=cmd('recipe give @s horizon_qol_suite:'+recipe);record(key+'_on',r['success'],response=r)
 cmd('setblock 21 91 0 furnace');cmd('item replace block 21 91 0 container.0 with bucket 1');cmd('item replace block 21 91 0 container.1 with coal 1');step(205)
 r=data('block 21 91 0','Items');record('bucket_smelted',any('iron_ingot' in x for x in r) and not any('bucket' in x for x in r),items=r)
 switch('kelp_fertilizer',True);cmd('item replace entity @s hotbar.0 with kelp 2');call('select','1');cmd('setblock 20 90 1 farmland');cmd('setblock 20 91 1 wheat[age=0]');cmd('tp @s 20.5 91 3.5');time.sleep(.3)
 call('interact','20','91','1');time.sleep(.3)
 r=call('block','20','91','1');counts=call('inventory')['counts'];record('kelp_grows_wheat',r['state']!='minecraft:wheat[age=0]' and counts.get('minecraft:kelp')==1,state=r,counts=counts)
 cmd('setblock 22 91 1 stone');time.sleep(.2);call('interact','22','91','1');time.sleep(.2);record('kelp_invalid_no_consumption',call('inventory')['counts'].get('minecraft:kelp')==1)
 switch('thorny_bush_protection',True);cmd('item replace entity @s armor.legs with iron_leggings');step(2)
 r=cmd('damage @s 1 minecraft:sweet_berry_bush');record('leggings_block_berry',not r['success'],response=r)
 cmd('item replace entity @s armor.legs with air');step(21);r=cmd('damage @s 1 minecraft:sweet_berry_bush');record('bare_legs_take_berry',r['success'],response=r)
 cmd('item replace entity @s armor.legs with iron_leggings');step(21);r=cmd('damage @s 1 minecraft:generic');record('leggings_do_not_block_other_damage',r['success'],response=r)
 switch('thorny_bush_protection',False);step(21);r=cmd('damage @s 1 minecraft:sweet_berry_bush');record('disabled_protection_vanilla_damage',r['success'],response=r)
 switch('no_feather_trample',True);cmd('item replace entity @s armor.feet with iron_boots[minecraft:enchantments={"minecraft:feather_falling":1}]');step(2)
 cmd('setblock 25 90 0 farmland[moisture=7]');cmd('tp @s 25.5 97 0.5');step(40);r=call('block','25','90','0');record('feather_falling_preserves_farmland','farmland' in r['state'],state=r)
 cmd('item replace entity @s armor.feet with air');cmd('effect give @s slow_falling 60 0 true');step(2);cmd('setblock 26 90 0 farmland[moisture=7]');cmd('tp @s 26.5 94 0.5');step(90);r=call('block','26','90','0');record('slow_falling_preserves_farmland','farmland' in r['state'],state=r)
 cmd('effect clear @s');cmd('item replace entity @s armor.feet with air');step(2);cmd('setblock 27 90 0 farmland[moisture=7]');cmd('tp @s 27.5 97 0.5');step(40);r=call('block','27','90','0');record('unprotected_farmland_tramples',r['block']=='minecraft:dirt',state=r)

finally:
 for key in ['kelp_fertilizer','furnace_recycle','village_bell_recipe','thorny_bush_protection','no_feather_trample']:root.joinpath(key).write_text('false')
 Path(args.output).write_text(json.dumps({'scenarios':rows,'package_sha256':hashlib.sha256(Path(args.package).read_bytes()).hexdigest()},indent=2)+'\n')
