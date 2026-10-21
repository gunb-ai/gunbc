#!/usr/bin/env python3
# Schedule-rows static auditor for the wet terminal (or any record-list .dag).
# usage: sched_audit.py <head-ref> <base-ref> <file> [expected_added_count] [claim-root ...]
# Reports: added/removed rows base->head (brace-depth aware), field-anomaly audit
# (exactly one identity/entry/function/expectation; no empty record), and for ADDED rows
# whether entry file exists at head and function exists as 'test fn' there.
import subprocess, sys, re, os

head, base, f = sys.argv[1], sys.argv[2], sys.argv[3]
exp = int(sys.argv[4]) if len(sys.argv) > 4 else None

def blob(ref, path):
    return subprocess.run(['git','show',f'{ref}:{path}'], capture_output=True, text=True,
                          cwd='/tmp/c4v/pr14575').stdout

def rows(t, rec='WetScheduledClaim {'):
    out=[]; idx=0
    while True:
        i=t.find(rec, idx)
        if i<0: break
        j=i+len(rec); depth=1
        while depth>0 and j<len(t):
            if t[j]=='{': depth+=1
            elif t[j]=='}': depth-=1
            j+=1
        out.append(t[i+len(rec):j-1]); idx=j
    return out

def topfields(r):
    d=0; fields=[]; cur=''
    for ch in r:
        if ch in '{[': d+=1
        elif ch in '}]': d-=1
        if ch==':' and d==0:
            fields.append(cur.strip().split('\n')[-1].strip()); cur=''
        else: cur+=ch
    return fields

def ident(r):
    m=re.search(r'identity: WitnessIdentity \{\s*value: "([^"]+)"', r)
    e=re.search(r'entry: "([^"]+)"', r)
    fn=re.search(r'\n\s*function: "([^"]+)"', r)
    ex=re.search(r'expectation: (\w[\w ]*)', r)
    return (m.group(1) if m else '?', e.group(1) if e else '?', fn.group(1) if fn else '?',
            re.sub(r'\s+',' ',ex.group(1)).strip() if ex else '?')

tb=blob(base,f); th=blob(head,f)
if not th: print(f"FATAL: no blob {head}:{f}"); sys.exit(2)
rb, rh = rows(tb), rows(th)
ib = [ident(r) for r in rb]; ih=[ident(r) for r in rh]
sb, sh = set(ib), set(ih)
added = [r for r in rh if ident(r) not in sb]
removed = [r for r in rb if ident(r) not in sh]
dups = len(ih)-len(sh)
print(f"rows: base={len(rb)} head={len(rh)} added={len(added)} removed={len(removed)} duplicate_rows={dups}")
if exp is not None:
    print("COUNT_MATCH" if len(added)==exp else f"COUNT_MISMATCH expected={exp} added={len(added)}")
if removed:
    print("REMOVED (data loss!):")
    for r in removed: print("  -", ident(r))
for i in sorted(set(ih) - sb):
    print("  +", i)
prob=[]
for r in rh:
    tf=topfields(r)
    if sorted(tf)!=sorted(['identity','entry','function','expectation']): prob.append((tf, re.sub(r'\s+',' ',r)[:90]))
    if not r.strip(): prob.append(('EMPTY',''))
print("FIELD_ANOMALIES:", "NONE" if not prob else prob[:8])
# added rows resolve in claim files at head
bad=0
for r in added:
    idy,entry,fn,ex = ident(r)
    txt=blob(head, entry)
    if not txt: print(f"  ENTRY_MISSING_AT_HEAD {entry} ({idy})"); bad+=1; continue
    if re.search(r'test fn '+re.escape(fn)+r'\s*\(', txt) is None:
        print(f"  FUNCTION_MISSING_IN_ENTRY {fn} in {entry} ({idy})"); bad+=1
print("ADDED_ROWS_RESOLUTION:", "ALL_OK" if bad==0 else f"{bad} BAD")
print("AUDIT_DONE")
