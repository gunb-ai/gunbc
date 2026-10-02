#!/usr/bin/env python3
"""Copy a DAG closure with hashes. Emission requires --preserve-source-paths:
byte equality alone does not preserve path-dependent declaration realization.
"""
import argparse, hashlib, json, re, shutil
from pathlib import Path
parser=argparse.ArgumentParser()
parser.add_argument('entry',nargs='+',help='module names')
parser.add_argument('--output',required=True)
parser.add_argument('--include-sealed-callers',action='store_true',help='Include caller-only authority modules when their reference resolution is required')
parser.add_argument('--preserve-source-paths', action='store_true', help='Retain declaration path identity for emitter qualification')
args=parser.parse_args()
modules={}
for root in ('dag','src/v2'):
    for path in Path(root).rglob('*.dag'):
        source=path.read_text()
        match=re.search(r'^module\s+([\w.]+)',source,re.M)
        if match:
            name=match.group(1)
            if name in modules: raise SystemExit('duplicate module '+name)
            modules[name]=(path,re.findall(r'^import\s+([\w.]+)',source,re.M),source)
seen=set(); pending=list(args.entry)
while pending:
    name=pending.pop()
    if name in seen: continue
    seen.add(name)
    if name not in modules: raise SystemExit('missing module '+name)
    pending.extend(modules[name][1])
    uncommented=re.sub(r'"(?:\\.|[^"\\])*"|//[^\n]*', lambda m: '' if m[0].startswith('//') else m[0], modules[name][2])
    for seal in re.findall(r'admit_callers\s*:\s*\[(.*?)\]',uncommented,re.S):
        for referenced in re.findall(r'module_path\s*:\s*"([\w.]+)"',seal):
            if args.include_sealed_callers and referenced in modules: pending.append(referenced)
    # Qualified references can name a module without an import in this language.
    code=re.sub(r'"(?:\\.|[^"\\])*"|//[^\n]*', '', modules[name][2])
    for qualified in re.findall(r'\b[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)+',code):
        parts=qualified.split('.')
        for end in range(len(parts),0,-1):
            prefix='.'.join(parts[:end])
            if prefix in modules:
                pending.append(prefix)
                break
out=Path(args.output)
if not out.resolve().is_relative_to((Path.cwd()/'target').resolve()):
    raise SystemExit('output must be a scratch subdirectory of this checkout target/')
out.mkdir(parents=True,exist_ok=True)
for stale in out.rglob('*.dag'): stale.unlink()
manifest=[]
for name in sorted(seen):
    original=modules[name][0]; target=out/original if args.preserve_source_paths else out/(name+'.dag')
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(original,target)
    manifest.append({'module':name,'source':str(original),'copy':str(target),'sha256':hashlib.sha256(original.read_bytes()).hexdigest()})
(out/'sources.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(str(out),len(manifest),'unchanged modules')
