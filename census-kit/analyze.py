import sys,glob,collections,re
S=sys.argv[1]; pop=set(open(S+'/sample.txt').read().split())
side={'main':{},'head':{}}
def rec(sd,p):
    return side[sd].setdefault(p,{'row':False,'ref':set(),'cons':None,'absent':set()})
for f in sorted(glob.glob(S+'/wave*/all.txt')):
    for l in open(f):
        m=re.match(r'^(main|head) (ROW|REFUSAL|FRONTEND|CONS|ABSENT) (.*)$',l.rstrip('\n'))
        if not m: continue
        sd,k,rest=m.groups()
        if k=='ROW': rec(sd,rest.split()[0])['row']=True
        elif k=='REFUSAL': p=rest.split(); rec(sd,p[0])['ref'].add(p[-1])
        elif k=='FRONTEND': rec(sd,rest.split()[0])['ref'].add('FRONTEND')
        elif k=='CONS':
            p=rest.split(); path=p[1]; rec(sd,path)['cons']=dict(x.split('=') for x in p[2:] if '=' in x)
        elif k=='ABSENT':
            a=rest.split(); loc=a[0]; path=loc.rsplit(':',1)[0]; rec(sd,path)['absent'].add((loc,a[1]))
paired=[p for p in pop if p in side['main'] and p in side['head'] and side['main'][p]['cons'] and side['head'][p]['cons']]
print('sample',len(pop),'paired',len(paired),'unpaired',len(pop)-len(paired))
rec_,new_=[],[]
refch=[]
tot=collections.Counter()
for p in sorted(paired):
    M,H=side['main'][p],side['head'][p]
    # ATOM DELTAS ONLY WHERE BOTH SIDES ACCEPTED. A module refused on either side prints no ABSENT
    # lines for its atoms, so comparing its absent set across sides reports every atom the other side
    # dropped as 'recovered' (or 'newly absent'). Refusal changes are reported separately below.
    if M['ref'] or H['ref']:
        if M['ref']!=H['ref']: refch.append((p,sorted(M['ref']),sorted(H['ref'])))
        for k in ('dropped','conserved','authored'): tot['main_'+k]+=int(M['cons'][k]); tot['head_'+k]+=int(H['cons'][k])
        continue
    for k in ('dropped','conserved','authored'): tot['main_'+k]+=int(M['cons'][k]); tot['head_'+k]+=int(H['cons'][k])
    if M['ref']!=H['ref']: refch.append((p,sorted(M['ref']),sorted(H['ref'])))
    rec_ += [(p,)+x for x in sorted(M['absent']-H['absent'])]
    new_ += [(p,)+x for x in sorted(H['absent']-M['absent'])]
print(dict(tot))
print('recovered atoms (modules accepted on BOTH sides only):',len(rec_))
print('new absent atoms (present on main, absent on head):',len(new_))
for x in new_[:40]: print('  NEW',x[1],x[2])
print('refusal changes:',len(refch))
for x in refch: print('  ',x)
print('unpaired:',sorted(pop-set(paired)))
