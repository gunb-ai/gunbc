# witness-import-scan (episodic scratch; NOT a repo instrument, carries no claim)

From the #16359 witness repair (session tidy-swift-586). Python source is
embedded below (repo policy bans `*.py`); copy it to any local path and run
`python3 genimp.py dag/test/claim/<witness>.dag`. It prints candidate
`import <home-module> { <names> }` lines for symbols used but neither declared
locally nor already imported, resolving each name to its declaring module over
dag/ + src/v2 with a builtin allowlist.

Offered so the remaining import-less witnesses (vivid-koi-387: 200 modules
without an import line at 49bca34f, 182 under dag/test/claim) can be dispatched
as one lane instead of file-by-file.

Verified limitations from #16359 (each caused a wrong import or a miss):
1. Names used as bare VALUES (`target_host: srv1_host`) are not matched by the
   `name(` heuristic - scan struct fields too, or the effect checker later
   refuses 'identity established but names no registry row'.
2. Symbols with NO declaration in the tree must be dropped, not guessed.
3. Unresolvable rows (`import Sym { UNKNOWN }`) are garbage - every name needs
   a real declaring module.
4. Cross-brand casts (`x as String as DnsLabel`) are not generated; witnesses
   passing data into branded parameters need them by hand.
5. Only EXECUTION certifies: run each file under claim_batch (one entry per
   process, GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES=450). Effect-callee and
   stale-pin classes surface under execution, not under imports alone.

```python
import re, subprocess, sys
p = sys.argv[1]
s = open(p).read()
nows = re.sub(r"//.*", "", s)
local = set(re.findall(r"^(?:data|fn|type)\s+([A-Za-z_][A-Za-z0-9_]*)", s, re.M))
imported = set()
for m in re.findall(r"import [a-z0-9_.]+ \{([^}]*)\}", s):
    imported |= set(x.strip() for x in m.split(","))
cap = set(re.findall(r"\b([A-Z][A-Za-z0-9_]{2,})\b", nows))
used_fn = set(re.findall(r"\b([a-z_][a-z0-9_]{3,})\(", nows))
builtin = {"Bool","String","Int","Nat","Second","Millisecond","Present","Absent","Ok","Err","True","False","Some","None"}
syms = sorted((cap | used_fn) - local - imported - builtin)

def home(sym):
    pats = [rf"^(data|fn|pub fn|type) {sym}\b", rf"^\s*\|\s*{sym}\b", rf"^\s+{sym}: "]
    for pat in pats:
        out = subprocess.run(["grep","-rln","--include=*.dag","-E",pat,"dag","src/v2"],
                             capture_output=True, text=True).stdout.split()
        hits = sorted(set(f for f in out if f != p))
        if hits:
            return hits[0][4:-4].replace("/", ".")
    return None

groups = {}
missing = []
for s_ in syms:
    h = home(s_)
    if h:
        groups.setdefault(h, set()).add(s_)
    else:
        missing.append(s_)
for m, ss in sorted(groups.items()):
    print(f"import {m} {{ {', '.join(sorted(ss))} }}")
if missing:
    print("UNKNOWN:", missing)
```
