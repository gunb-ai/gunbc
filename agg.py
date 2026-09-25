import json,glob,collections,re
sites=json.load(open('/tmp/tidy-otter-out/sites.json'))
sites={int(k):v for k,v in sites.items()}
sites[999999]=['src/v2/std/node.dag',0,'node_synthetic (caller outside tagged files)']
S=collections.Counter(); site=collections.Counter(); prod=collections.defaultdict(collections.Counter)
kinds=collections.Counter(); D=collections.Counter(); modules=0; nonM=collections.Counter()
for f in sorted(glob.glob('/tmp/tidy-otter-out/i_batch*.out')):
    for l in open(f):
        l=l.rstrip('\n')
        if l.startswith('error: function') : l=l.split('returned `',1)[1]
        p=l.split('\t')
        if p[0]=='S' and p[1].startswith('module '):
            modules+=1
            for k,v in re.findall(r'(\w+)=(\d+)',p[1]): S[k]+=int(v)
        elif p[0] in 'RPUG' and len(p)==2: nonM[p[0]]+=1
        elif p[0]=='E':
            how=p[3]
            if how=='label': key='LABEL (Named edge)'
            else:
                t=int(how.split(':')[1]); 
                if t==-1: key='UNTAGGED OccurrenceSynthetic'
                elif t==-3: key='ATOM carrying an enclosing non-atom occurrence (wrong source)'
                elif t==-4: key='ATOM carrying ANOTHER authored atom occurrence (wrong source)'
                else: key='%s :: %s'%(sites[t][0].split('/')[-1],sites[t][2])
            kinds[how.split(':')[0]]+=1
            site[key]+=1; prod[key][p[1]]+=1
        elif p[0]=='D': D[p[1]]+=1
print('modules',modules,dict(nonM)); print(dict(S)); print('E total',sum(site.values()),dict(kinds))
for k,v in site.most_common():
    print(f'{v:6d} {100*v/sum(site.values()):5.1f}%  {k}   top prods: '+', '.join(f'{a.replace("dag_surface_","")}={b}' for a,b in prod[k].most_common(4)))
print('D (probe-unmatched, mostly module header):',sum(D.values()),D.most_common(5))
