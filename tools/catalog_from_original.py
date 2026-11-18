#!/usr/bin/env python3
"""Extract the original suite's 25 modules and option defaults for parity tracking."""
from pathlib import Path
import argparse, re, json
parser = argparse.ArgumentParser(description='Extract menu metadata from Horizon QoL 0.4.13')
parser.add_argument('original_menu', type=Path, help='Path to the original mixins/horizon_qol_suite/menu.rs')
args = parser.parse_args()
old = args.original_menu.read_text()
base = Path(__file__).resolve().parents[1]

def split_calls(s):
    out=[]; i=0
    while i<len(s):
        m=re.search(r'\b(flag|num|choice|text)\s*\(',s[i:])
        if not m: break
        a=i+m.start(); b=i+m.end(); depth=1; in_string=False; escaped=False; j=b
        while j<len(s) and depth:
            c=s[j]
            if in_string:
                if escaped: escaped=False
                elif c=='\\': escaped=True
                elif c=='"': in_string=False
            else:
                if c=='"': in_string=True
                elif c=='(': depth+=1
                elif c==')': depth-=1
            j+=1
        out.append((m.group(1),s[b:j-1]));i=j
    return out

def fields(s):
    out=[];parts=[];i=0;in_string=False;depth=0;last=0
    for i,c in enumerate(s):
        if c=='"' and (i==0 or s[i-1]!='\\'):in_string=not in_string
        if not in_string:
            if c in '[({':depth+=1
            elif c in '])}':depth-=1
            elif c==',' and depth==0:parts.append(s[last:i].strip());last=i+1
    parts.append(s[last:].strip())
    return parts

arrays={}
for m in re.finditer(r'const\s+(\w+):&\[Opt\]=&\[(.*?)\];',old,re.S):
    arr=[]
    for kind, args in split_calls(m.group(2)):
        f=fields(args)
        key=json.loads(f[0]);
        if kind=='flag': arr.append({'key':key,'type':'bool','default':f[1].strip()=='true'})
        elif kind=='num':arr.append({'key':key,'type':'number','default':int(f[1]),'min':int(f[2]),'max':int(f[3]),'step':int(f[4])})
        elif kind=='text':arr.append({'key':key,'type':'text','default':json.loads(f[1])})
        else:arr.append({'key':key,'type':'choice','default':json.loads(f[1]),'options':re.findall(r'"([^"]*)"', f[2])})
    arrays[m.group(1)]=arr
modules=[]
for m in re.finditer(r'Module\{name:"([^"]+)",switch:(\d+),options:([^}]+)\}',old):
    name,order,options=m.groups()
    options=options.strip()
    modules.append({'name':name,'order':int(order),'key':re.sub(r'[^a-z0-9]+','_',name.lower()).strip('_'),'options':arrays.get(options,[])})
assert len(modules)==25,len(modules)
(base/'docs'/'original-catalog.json').write_text(json.dumps({'origin':'Horizon QoL 0.4.13-alpha','modules':modules},indent=2,ensure_ascii=False)+'\n')
print('Catalog saved:',len(modules),'modules',sum(len(x['options']) for x in modules),'options')
