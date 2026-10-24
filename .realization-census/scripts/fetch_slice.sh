#!/usr/bin/env bash
# fetch_slice.sh <slice> : pull a reader's sheets from its data branch and print grain checks
s=$1; W=/home/briansrls/.worktrees/quiet-seal-78; D=$W/.realization-census/slices/$s
cd $W && git fetch -q origin census-data/quiet-seal-78-$s || exit 1
git log -1 --format='%h %s' FETCH_HEAD; mkdir -p $D
for f in modules findings; do git show FETCH_HEAD:.realization-census/slices/$s/$f.tsv > $D/$f.tsv || exit 1; done
python3 - $D $W/.realization-census/slices/$s.tsv <<'PY'
import csv,sys,collections
D,sl=sys.argv[1],sys.argv[2]
m=list(csv.DictReader(open(D+'/modules.tsv'),delimiter='\t')); f=list(csv.DictReader(open(D+'/findings.tsv'),delimiter='\t'))
want={x['module'] for x in csv.DictReader(open(sl),delimiter='\t')}; got={x['module'] for x in m}
print('modules',len(m),'findings',len(f),'missing',sorted(want-got)[:5],'extra',sorted(got-want)[:5])
bad=[x for x in m if len(x)!=11 or None in x.values()]; print('malformed module rows',len(bad))
spec=collections.Counter(x['specimen'] for x in f); print('repeated specimens (>2):',[(k[:60],v) for k,v in spec.items() if v>2][:5])
noquote=sum(1 for x in f if '"' not in x['specimen'] and '(' not in x['specimen']); print('specimens without quote or call:',noquote)
print(collections.Counter((x['dimension'],x['level']) for x in f).most_common())
PY
