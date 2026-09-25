import glob,collections,json
sites={int(k):v for k,v in json.load(open('/tmp/tidy-otter-out/sites.json')).items()}
c=collections.Counter(); ex=collections.defaultdict(list)
for f in sorted(glob.glob('/tmp/tidy-otter-out/i_batch*.out')):
    for l in open(f):
        l=l.rstrip('\n')
        if 'returned `' in l: l=l.split('returned `',1)[1]
        p=l.split('\t')
        if p[0]!='E': continue
        how=p[3]
        b='label' if how=='label' else {-3:'wrongsrc-shell',-4:'wrongsrc-other-atom',-1:'untagged'}.get(int(how.split(':')[1]), sites.get(int(how.split(':')[1]),['','','?'])[2])
        ctx='/'.join(x.replace('dag_surface_','') for x in p[4].split('/')[-3:])
        c[(b,ctx)]+=1
        if len(ex[(b,ctx)])<5: ex[(b,ctx)].append(p[2])
for k,v in c.most_common(40): print(v,k[0],'|',k[1],ex[k])
