import csv,collections,re
S='/tmp/claude-1000/-home-briansrls--worktrees-gunbc-jolly-hawk-60/f5e0c546-79ec-40e4-b407-11d43a4e07eb/scratchpad'
V=list(csv.DictReader(open(S+'/vivid.tsv'),delimiter='\t'))
M=list(csv.DictReader(open(S+'/inventory_merged.tsv'),delimiter='\t'))
mm=collections.defaultdict(list)
for r in M: mm[r['module']].append(r)
DONE={'gunbc.compute.attempt_lifecycle','gunbc.compute.compute_seat_ensure','gunbc.roadmap.roadmap_attempt_worktree_claim','gunbc.roadmap.roadmap_session_activation_ensure','gunbc.compute.compute_attempt_slots_root','gunbc.dispatch_credential_acquisition','gunbc.roadmap.roadmap_attempt_worktree_claim_root','gunbc.roadmap.roadmap_session_attempt_slots_root'}
SHARED_APPROVAL={'gunbc.auth.approval_decision_store','gunbc.auth.approval_store_receipt','gunbc.auth.approval_broker_serve','gunbc.auth.approval_keyring_converge'}
def sub(m):
    p=m.split('.')
    if p[0]=='v2': return 'v2_workflow'
    if p[0] in('product','extdeps'): return p[0]+'.'+p[1]
    if p[1]=='instruments': return 'instruments'
    if len(p)>2: return p[1]
    for k in('workspace','approval','dispatch','fleet','roadmap','deploy','durable','host','harness','runner','fabric','compute'):
        if p[1].startswith(k): return k
    return 'flat'
SH={'APPEND_LOG':'Blocked: append-only','SECRET_PATH':'ReHomedAccess','DEPLOY_TREE':'ReHomedPlacement','RECEIPT':'ReHomedObservability',
  'SCRATCH':'NotAStore: scratch within one run (M1)','KERNEL_ABI':'NotAStore: kernel ABI path (d)','TOOL_BINARY':'NotAStore: tool binary path (d)',
  'WITNESS_FIXTURE':'NotAStore: witness fixture','LIBRARY':'NotAStore: store library (owns no root)','NONE':'NotAStore: no store (hand read)',
  'SLOT':'Migrate: seal (slot store)','RECORD_DIR':'Migrate: seal in place (+FormatConsolidation row)','SINGLE_FILE':'Migrate: seal in place (+FormatConsolidation row)',
  'HAND_CAS':'Migrate: hand lock/CAS -> hold library'}
def disp(module,shape,merged):
    if module in DONE: return 'Done: sealed by #14944/#14972'
    if module in SHARED_APPROVAL and shape in('SLOT','RECORD_DIR','HAND_CAS','SINGLE_FILE'): return 'Blocked: shared (gunbc.rung_drop approval_store_single_writer_by_agreement)'
    if merged.startswith('Blocked') and 'seed' not in merged and shape not in('RECEIPT','SECRET_PATH','DEPLOY_TREE','SCRATCH','KERNEL_ABI','TOOL_BINARY','LIBRARY','NONE','WITNESS_FIXTURE'): return merged
    return SH.get(shape, merged or 'Migrate?: unread')
out=[]; vm=set()
for v in V:
    m=v['module']; vm.add(m)
    merged=(mm.get(m) or [{}])[0].get('disposition','')
    d=disp(m,v['shape'],merged)
    if 'srv3_bmcweb_login_body' in v['evidence']+v['store'] or ('bmc_onboard' in m and v['shape']=='SECRET_PATH'): d='OutOfLane: BMC /tmp credential file (fix dispatched by stern-ram-763)'
    out.append({'module':m,'store':v['store'],'shape':v['shape'],'disposition':d,'cohort':sub(m) if d.startswith('Migrate') else '','root':v['root'],'business_key':v['business_key'],'readers':v['readers'],'evidence':v['evidence'],'note':v['note'],'jolly':' | '.join(sorted({r['jolly'] for r in mm.get(m,[])}))[:120],'bright':' | '.join(sorted({r['bright'] for r in mm.get(m,[])}))[:120],'source':'vivid'})
for m,rs in mm.items():
    if m in vm: continue
    for r in rs:
        d='Done: sealed by #14944/#14972' if m in DONE else r['disposition']
        out.append({'module':m,'store':r['symbol'],'shape':'','disposition':d,'cohort':sub(m) if d.startswith('Migrate') else '','root':'','business_key':'','readers':'','evidence':r['evidence'],'note':r['bright_blocked_cause'],'jolly':r['jolly'],'bright':r['bright'],'source':'jolly+bright'})
with open(S+'/pub/.store-inventory/inventory.tsv','w') as f:
    w=csv.DictWriter(f,fieldnames=list(out[0].keys()),delimiter='\t'); w.writeheader(); w.writerows(out)
print('rows',len(out),'modules',len({o['module'] for o in out}))
print(collections.Counter(re.split(r'[:(]',o['disposition'])[0].strip() for o in out))
co=collections.defaultdict(set)
for o in out:
    if o['cohort']: co[o['cohort']].add(o['module'])
print(sorted(((k,len(v)) for k,v in co.items()),key=lambda kv:-kv[1]))
for o in out:
    if o['disposition'].startswith('Blocked') and 'seed' not in o['disposition']: print('  B',o['module'],o['store'][:40],o['disposition'][:60],sep=' | ')
