import re, collections, json, sys
S=sys.argv[1]
pat=re.compile(r'(?:SITE )?error\[([^:]+):(\d+):(\d+)\]: JOINCENSUS (\S+) elem=(\S+) sep=(\S+) binder=(\S+)')
rows={}
def add(path,pop):
    for l in open(path):
        m=pat.search(l.strip())
        if not m: continue
        f,ln,col,route,elem,sep,binder=m.groups(); loc=f'{f}:{ln}:{col}'
        if loc in rows: continue
        rows[loc]=dict(loc=loc,route=route,elem=elem,sep=sep,binder=binder,pop=pop)
V2='src/v2 per-file entry compile (outside the dag-primary population)'
add(f'{S}/join/join-corpus-lines.txt','corpus compile (dag primary)')
add(f'{S}/join/join-v1-lines.txt','src/v1 self-compile')
add(f'{S}/join-entry2a.out',V2)
add(f'{S}/join/join-v2-entry-rest.txt',V2)
print('rows',len(rows), dict(collections.Counter(r['pop'] for r in rows.values())))
def module_file(mod):
    p=mod.split('.')
    return ('src/v2/'+'/'.join(p[1:]) if p[0]=='v2' else 'dag/'+'/'.join(p))+'.dag'
shape={}
def decl_line(rd):
    if rd in shape: return shape[rd]
    q=rd.split(':',1)[1].rsplit('/',1)[0]; mod,name=q.rsplit('.',1)
    try:
        t=open(module_file(mod)).read(); m=re.search(r'^\s*(?:pub\s+)?type\s+'+re.escape(name)+r'\b([^\n]*)',t,re.M)
        shape[rd]=('type '+name+m.group(1)).strip()[:100] if m else '(no type line)'
    except Exception: shape[rd]='(declaration file not found)'
    return shape[rd]
def cls(rd):
    if rd in ('no-separator','no-element','unknown','unknown-receiver'): return 'U unknown ('+rd+')'
    base=rd.rsplit('/',1)[0]
    if base=='kernel:String:mint': return 'S kernel String'
    if base=='kernel:String:named': return 'S kernel String (read by name only)'
    if base.startswith('kernel:'): return 'N kernel non-text ('+base.split(':')[1]+')'
    if base.startswith('tyvar:') or base=='unidentified': return 'U unknown ('+base.split(':')[0]+')'
    sh=decl_line(rd)
    if base.startswith(('decl:v2.','decl-env:v2.')): return 'V v2-native declared type ('+base.split(':',1)[1]+')'
    if re.search(r'=\s*(String|NonEmptyStr)\b',sh) or base.endswith('std.types.NonEmptyStr'): return 'T text refinement or brand'
    if re.search(r'=\s*(Int|Nat)\b',sh): return 'N declared numeric'
    return 'R declared record/coproduct/other'
for r in rows.values():
    r['ec']=cls(r['elem']); r['sc']=cls(r['sep']); r['card']=r['elem'].rsplit('/',1)[1] if '/' in r['elem'] else '?'
out=['location\troute\telement_class\telement_reading\tseparator_class\tseparator_reading\tbinder\tpopulation']
for loc in sorted(rows):
    r=rows[loc]; out.append('\t'.join([loc,r['route'],r['ec'],r['elem'],r['sc'],r['sep'],r['binder'],r['pop']]))
open(f'{S}/join/join-sites.tsv','w').write('\n'.join(out)+'\n')
summ=[]
ec=collections.Counter(r['ec'] for r in rows.values()); sc=collections.Counter(r['sc'] for r in rows.values())
summ.append('ELEMENT CLASSES'); summ+= [f'{v:6d}  {k}' for k,v in ec.most_common()]
summ.append('SEPARATOR CLASSES'); summ+= [f'{v:6d}  {k}' for k,v in sc.most_common()]
summ.append('ELEMENT READINGS OUTSIDE kernel String (with declaration line)')
summ+= [f"{v:6d}  {k}  {shape.get(k,'')}" for k,v in collections.Counter(r['elem'] for r in rows.values() if not r['ec'].startswith('S kernel String')).most_common()]
summ.append('SEPARATOR READINGS OUTSIDE kernel String')
summ+= [f"{v:6d}  {k}  {shape.get(k,'')}" for k,v in collections.Counter(r['sep'] for r in rows.values() if not r['sc'].startswith('S kernel String')).most_common()]
summ.append('ROUTES: '+', '.join(f'{k} {v}' for k,v in collections.Counter(r['route'] for r in rows.values()).most_common()))
summ.append('BY POPULATION: '+', '.join(f'{k} {v}' for k,v in collections.Counter(r['pop'] for r in rows.values()).most_common()))
summ.append(f"TOTAL typed join sites: {len(rows)}; optional elements: {sum(1 for r in rows.values() if r['card']=='opt')}")
open(f'{S}/join/join-summary.txt','w').write('\n'.join(summ)+'\n')
print('\n'.join(summ))
