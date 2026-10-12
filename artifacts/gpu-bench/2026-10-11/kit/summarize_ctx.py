import json,sys,collections
d=collections.defaultdict(list)
for l in open(sys.argv[1]): d[json.loads(l)["n"]].append(json.loads(l))
q=lambda a,p: sorted(a)[min(len(a)-1,int(p*(len(a)-1)+.5))]
for n in sorted(d):
    R=d[n]; agg=[r["agg"] for r in R]; per=[x for r in R for x in r["per"] if x]; tt=[x for r in R for x in r["ttft"]]
    print(f"N={n} ctx~{R[0]['ctx']}: rounds={len(R)} | aggregate med {q(agg,.5):.0f} p90 {q(agg,.9):.0f} p99 {q(agg,.99):.0f} | per-stream med {q(per,.5):.0f} p99 {q(per,.99):.0f} | TTFT med {q(tt,.5):.1f}s p90 {q(tt,.9):.1f}s")
