import sys,subprocess,glob
S=sys.argv[1]
done={'main':set(),'head':set()}
for f in glob.glob(S+'/wave*/all.txt'):
    for l in open(f):
        p=l.split()
        if len(p)>2 and p[1]=='ROW': done[p[0]].add(p[2])
allf=open(S+'/sample.txt').read().split()
rem=[f for f in allf if not (f in done['main'] and f in done['head'])]
open(S+'/remaining.txt','w').write('\n'.join(rem)+('\n' if rem else ''))
print(len(rem))
