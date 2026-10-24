import json,csv,re,collections,subprocess
S='/tmp/claude-1000/-home-briansrls--worktrees-gunbc-jolly-hawk-60/f5e0c546-79ec-40e4-b407-11d43a4e07eb/scratchpad'
idx=json.load(open(S+'/idx_pr_14972.json'))
cen=collections.defaultdict(list)
for r in csv.DictReader(open(S+'/census/findings.tsv'),delimiter='\t'):
    if r['dimension']=='storage' and r['site'] in('Forked','Literal','Rederived'): cen[r['module']].append(r)
MOVED={'gunbc.fleet.printer_batch':'gunbc.fleet.printer_actuation'}
def rooty(lib):
    src=subprocess.run(['git','show','pr/14972:'+idx[lib]['path']],capture_output=True,text=True).stdout
    return {n for n,a in re.findall(r'^fn ([a-z_0-9]+)\(([^)]*)\)',src,re.M) if re.search(r'\b(root|dir|directory|path|base|fs)\w*\s*:',a)}
LIBS={'gunbc.durable_cas_file_store':'CAS','gunbc.durable_exclusive_hold_file_store':'HOLD','gunbc.fabric.fabric_storage_file_store':'FABRIC',
      'extdeps.realization.materialization_store_local':'MATSTORE','gunbc.serving.caller_backlog':'BACKLOG','gunbc.auth.approval_store_receipt':'RECEIPTLIB',
      'gunbc.fabric.fabric_required_build_cell':'BUILDCELL'}
RF={l:rooty(l) for l in LIBS}
EFFW=re.compile(r'^Filesystem\.(Write\w*|Delete|LinkCreateNew)$'); EFFD=re.compile(r'^shell\.(Mkdir|Chmod)'); EFFS=re.compile(r'^shell\.Remove'); EFFM=re.compile(r'^shell\.(Copy|Move)')
pop={}
for m,d in idx.items():
    if d['class'] not in('production','extdeps','instrument'): continue
    tags=collections.Counter(); scratch_only=True; any_eff=False
    for dn,dd in d['decls'].items():
        calls=dd['calls']; scr=any(c['name'].startswith('shell.Mktemp') for c in calls)
        for c in calls:
            if c['to_mod'] in LIBS and c['name'] in RF[c['to_mod']] and m not in LIBS: tags[LIBS[c['to_mod']]]+=1
            if c['name']=='file_slot_store_over': tags['MINT']+=1
            t=None
            if EFFW.match(c['name']): t='RAW'
            elif EFFD.match(c['name']): t='DIR'
            elif EFFS.match(c['name']): t='SWEEP'
            elif EFFM.match(c['name']): t='MOVE'
            if t:
                any_eff=True
                if not scr: scratch_only=False; tags[t]+=1
                else: tags['scratch_'+t]+=1
    c=cen.get(m,[])+[r for k,v in MOVED.items() if v==m for r in cen.get(k,[])]
    if tags or c:
        pop[m]={'path':d['path'],'class':d['class'],'tags':dict(tags),'census':[{k:r[k] for k in('class_id','site','symbol','specimen')} for r in c]}
for k in cen:
    if k not in idx and k not in MOVED: pop[k]={'path':'','class':'seed' if k.startswith('src/v1') else '?','tags':{},'census':[{kk:r[kk] for kk in('class_id','site','symbol','specimen')} for r in cen[k]]}
json.dump(pop,open(S+'/population.json','w'),indent=0)
cnt=collections.Counter()
for m,p in pop.items():
    t=p['tags']; prim=[x for x in('MINT','CAS','HOLD','FABRIC','MATSTORE','BACKLOG','RECEIPTLIB','BUILDCELL','RAW','DIR','MOVE','SWEEP') if t.get(x)]
    p['prim']=prim[0] if prim else ('SCRATCH_ONLY' if any(k.startswith('scratch_') for k in t) else 'NO_EFFECT')
    cnt[(p['class'],p['prim'],bool(p['census']))]+=1
json.dump(pop,open(S+'/population.json','w'),indent=0)
for k,v in sorted(cnt.items()): print(v,*k,sep='\t')
print('total',len(pop))
