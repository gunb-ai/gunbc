import re, json, collections
CAPS=re.compile(r'\s*\|\|CHILD-CAP|\s*\|\|DEPTH-CAP')
NODE=re.compile(r'^(?P<path>[LR][0-9A-Za-z]*) n=(?P<name>[^|]*)\|conn=(?P<conn>[^|]*)\|card=(?P<card>[^|]*)\|decl=(?P<decl>[^|]*)\|infer=(?P<infer>.*?)\|shape=(?P<shape>.*?)\|nc=(?P<nc>\d+)\|idsp=(?P<idsp>.*)$')
v2={}
for line in open('/tmp/probeB8_unpack/sh.log',errors='replace'):
    if not line.startswith('REBINDARM|'): continue
    b=line[len('REBINDARM|'):].rstrip('\n'); i=b.find(';;TREE')
    d=dict(kv.split('=',1) for kv in b[:i].rstrip('|').split('|') if '=' in kv)
    nodes={}
    for seg in b[i+6:].split(' ;; '):
        for side in seg.split(' >< '):
            m=NODE.match(side.strip())
            if m:
                g=m.groupdict(); g['idsp']=CAPS.sub('',g['idsp']); nodes[g['path']]=g
    v2.setdefault((d.get('module'),d.get('bind'),d.get('unfoldedL'),d.get('unfoldedR')),[]).append(nodes)
rows=json.load(open('/tmp/rebindv3_rows.json'))
def P(s): return set(x for x in (s or '').split(',') if x)
res=collections.Counter(); ctrl=[]; loose=[]; hits=[]
for r in rows:
    nodes=v2[(r['module'],r['bind'],r['lshape'],r['rshape'])][0]
    Lv=sorted(g['name'] for g in nodes.values() if g['path'].startswith('L') and g['nc']=='0')
    Lp,Rp=P(r.get('lparams')),P(r.get('rparams'))
    conn = r.get('lconn','').startswith('Disj') and not r.get('rconn','').startswith('Disj') and r.get('lname')==r.get('rname')
    decl_form = (len(Lp)>0 and any(x in Lp for x in Lv))
    res[('conn' if conn else 'other','L-declform' if decl_form else 'L-notown','R-noparams' if not Rp else 'R-hasparams')]+=1
    if conn and decl_form and len(hits)<5:
        hits.append((r['module'],r['bind'],Lv[:7],sorted(Lp)[:6],r.get('lleafn'),len(Lv)))
    same_applied = (r.get('lconn')==r.get('rconn') and r.get('lname')==r.get('rname') and r.get('lshape')!=r.get('rshape'))
    if same_applied:
        ctrl.append((r['module'],r['bind'],r['lshape'][:40],r['rshape'][:40],decl_form,len(Lp),len(Rp),r.get('core')))
        if decl_form: loose.append((r['module'],r['bind'],sorted(Lp)[:5],Lv[:5]))
print("PREDICATE under test: bound node declares params AND >=1 bound leaf name is one of them")
for k,v in res.most_common(8): print("  %4d  %s"%(v,k))
print("\nconn-class rows where it FIRES (first 5)  [module, bind, bound leaves, bound params]:")
for h in hits: print("   %s bind=%s\n      leaves=%s\n      params=%s  (leafn printed=%s actual=%d)"%(h[0],h[1],h[2],h[3],h[4],h[5]))
print("\nCONTROL rows (same head, both applied, DIFFERENT shapes) -- predicate must be FALSE:")
for c in ctrl[:8]: print("   %-32s bind=%s L=%s R=%s -> declform=%s Lp=%d Rp=%d core=%s"%(c[0][:32],c[1],c[2],c[3],c[4],c[5],c[6],c[7]))
print("\nLOOSE (control rows where predicate wrongly fires): %d"%len(loose))
for l in loose[:6]: print("   ",l)
