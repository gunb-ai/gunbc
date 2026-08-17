"""Namespace-cut binding RISK CENSUS.

Usage:  binding_identity_oracle.py <import-era-ref> <branch-ref> <out-receipt.json>

Materializes both trees from git refs itself, so a run is reproducible from the
two SHAs alone. See docs/plans/namespace-cut-binding-risk-census.md for what
this does and does NOT establish -- in particular it classifies GLOBAL DECLARER
UNIQUENESS, which is an upper bound on risk, not occurrence-level pre/post
binding parity.
"""
import re,os,json,collections,sys,subprocess,tempfile

def materialize(ref, dest):
    os.makedirs(dest, exist_ok=True)
    sha=subprocess.check_output(["git","rev-parse",ref],text=True).strip()
    tar=subprocess.check_output(["git","archive",sha])
    import tarfile,io
    tarfile.open(fileobj=io.BytesIO(tar)).extractall(dest)
    return sha

if len(sys.argv)!=4:
    sys.exit(__doc__)
IMPORT_ERA_REF, BRANCH_REF, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
_tmp=tempfile.mkdtemp(prefix="bindcensus-")
MAIN=os.path.join(_tmp,"import_era"); BRANCH=os.path.join(_tmp,"branch")
IMPORT_ERA_SHA=materialize(IMPORT_ERA_REF, MAIN)
BRANCH_SHA=materialize(BRANCH_REF, BRANCH)
KERNEL={"String","Int","Bool","Float","Secret","Json","Unit","Bytes"}

def module_of(src):
    m=re.match(r'\s*module\s+([\w.]+)', src)
    return m.group(1) if m else None

# --- declaration index over MAIN: name -> {module}
decl=collections.defaultdict(set)
files=[]
for root,_,fs in os.walk(MAIN):
    for fn in fs:
        if fn.endswith(".dag"): files.append(os.path.join(root,fn))
for p in files:
    s=open(p,errors="replace").read(); mod=module_of(s)
    if not mod: continue
    for nm in re.findall(r'^(?:type|fn|func|data|test\s+fn)\s+([A-Za-z_]\w*)', s, re.M): decl[nm].add(mod)
    for nm in re.findall(r'^\s*[=|]\s*([A-Z]\w*)', s, re.M): decl[nm].add(mod)          # coproduct variants
    for nm in re.findall(r'^\s*\|\s*([A-Z]\w*)\s*(?:\{|$)', s, re.M): decl[nm].add(mod)

# --- for each branch file, classify each name its pre-cut import bound
stats=collections.Counter(); risky=[]
for p in files:
    rel=os.path.relpath(p,MAIN)
    rel_abs=os.path.join(BRANCH,rel)
    if not os.path.exists(rel_abs): continue
    msrc=open(p,errors="replace").read()
    binds={}
    for blk in re.finditer(r'^import\s+([\w.]+)\s*\{([^}]*)\}', msrc, re.M):
        for nm in re.findall(r'[A-Za-z_]\w*', blk.group(2)):
            if nm not in KERNEL: binds[nm]=blk.group(1)
    if not binds: continue
    bsrc=open(rel_abs,errors="replace").read()
    body=re.sub(r'^\s*//.*$','',bsrc,flags=re.M)
    own=module_of(bsrc)
    for nm,imp_mod in binds.items():
        # is the name referenced BARE in the branch file?
        if not re.search(r'(?<![.\w])'+re.escape(nm)+r'(?![\w])', body): continue
        d=decl.get(nm,set())
        if not d: stats["UNKNOWN (no declarer found)"]+=1; continue
        if len(d)==1:
            stats["SAFE (globally unique declarer)"]+=1
        else:
            # multiple declarers: bare resolution depends on which is in the pool
            stats["AMBIGUOUS (multi-declared, pool decides)"]+=1
            risky.append((rel,nm,imp_mod,sorted(d)))
print("=== BINDING-IDENTITY ORACLE over bare cross-module references")
for k,v in stats.most_common(): print(f"{v:7d}  {k}")
print(f"\ndistinct risky (file,name) pairs: {len(risky)}")
print(f"distinct ambiguous names: {len(set(r[1] for r in risky))}")
json.dump({
  "provenance": {
    "import_era_ref": IMPORT_ERA_REF, "import_era_sha": IMPORT_ERA_SHA,
    "branch_ref": BRANCH_REF, "branch_sha": BRANCH_SHA,
    "file_universe": "*.dag under the whole tree of each ref",
    "denominator_note": "rows are (file, name) pairs where an import-era import bound `name` and the branch still references it BARE",
  },
  "totals": dict(stats),
  "rows": [{"file":f,"name":n,"import_said":m,"declarers":d} for f,n,m,d in risky],
}, open(OUT,"w"), indent=1)
for f,n,m,d in risky[:8]:
    print(f"  {n}: import said {m}; declared in {len(d)}: {d[:3]}")
