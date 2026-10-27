import json,sys,collections
d=collections.defaultdict(list)
for l in open(sys.argv[1]): r=json.loads(l); d[r["n"]].append(r["agg"])
q=lambda a,p: sorted(a)[min(len(a)-1,int(p*(len(a)-1)+.5))]
for n in sorted(d):
    a=d[n]; print(f"N={n:>2}: rounds={len(a):>3}  median {q(a,.5):6.0f}  p90 {q(a,.9):6.0f}  p99 {q(a,.99):6.0f}  max {max(a):6.0f} tok/s")
