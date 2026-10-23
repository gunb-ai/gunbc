#!/bin/bash
# Parent msg_3c0234f5: on main, is v1_compiler_infer.rs's drift DECLARED? Checked locally:
#   HAND_MAINTAINED_STAGE0_FILES (bootstrap_stage0_crate_layout_generated.rs:5) does NOT contain it;
#   EMITTER_PRODUCED_DIVERGENT_STAGE0_FILES (:54) contains ONLY "main.rs".
# So the v1_compiler_infer.rs divergence is UNDECLARED -> a main-level 'cannot regenerate itself'
# defect, which the parent is dispatching separately and which is NOT mine to fix in #14152. What the
# parent asked for is the FIRST DIFFERING DECLARATION. That is what this prints: clean main, one
# regen round, then the candidate vs committed diff grouped by the enclosing top-level declaration.
# NO git checkout / git clean; restores by cp.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
# clean main: main's version of every tracked file my branch touches (staged locally, 27 files)
find keep/clean -type f | while read -r f; do rel=${f#keep/clean/}; case "$rel" in
  *.removed) rm -f "${rel%.removed}";; *) mkdir -p "$(dirname $rel)"; cp "$f" "$rel";; esac; done
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
echo "clean-main: dagrelation=$(grep -c generic_types_unify src/v1/04_infer.dag) (want 0) claimtext=$(grep -c 'text:' src/v1/tests/claim/generic_identity_census.dag) (want 0) seedfold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs) (want 0)"
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/drift.log 2>&1
echo "rc=$?  $(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/drift.log | head -1)"
grep -a "^required-regen:" /tmp/drift.log | grep -aE "first_generation_equal|FAIL|drift|declared" | head -5 | cut -c1-220
echo "===== candidate files present:"; ls "$CAND" 2>/dev/null | head -8
if [ -f "$CAND/v1_compiler_infer.rs" ]; then
  python3 - <<'PY'
import re, difflib
cand=open('target/stage0-regen-candidate/src/v1_compiler_infer.rs',encoding='utf8').read().splitlines()
comm=open('src/v1/stage0/src/v1_compiler_infer.rs',encoding='utf8').read().splitlines()
print(f"committed={len(comm)} lines, candidate={len(cand)} lines")
DECL=re.compile(r'^\s*(pub )?(async )?(fn|struct|enum|impl|trait|type|const|static|mod)\b')
def decls(lines):
    out=[]
    for i,l in enumerate(lines):
        if DECL.match(l):
            m=re.match(r'^\s*(?:pub )?(?:async )?(?:fn|struct|enum|impl|trait|type|const|static|mod)\s+([A-Za-z0-9_]+)',l)
            out.append((i+1, l.strip()[:90], m.group(1) if m else '?'))
    return out
dc, dd = decls(comm), decls(cand)
print(f"top-level declarations: committed={len(dc)} candidate={len(dd)}")
names_c=[d[2] for d in dc]; names_d=[d[2] for d in dd]
only_c=sorted(set(names_c)-set(names_d)); only_d=sorted(set(names_d)-set(names_c))
print("declarations ONLY in committed (emitter dropped):", len(only_c), only_c[:15])
print("declarations ONLY in candidate (emitter added):  ", len(only_d), only_d[:15])
sm=difflib.SequenceMatcher(None, names_c, names_d, autojunk=False)
ops=[o for o in sm.get_opcodes() if o[0]!='equal']
print(f"sequence ops: {len(ops)} (equal blocks {len([o for o in sm.get_opcodes() if o[0]=='equal'])})")
for tag,i1,i2,j1,j2 in ops[:6]:
    print(f"  {tag} committed[{i1}:{i2}] candidate[{j1}:{j2}]")
    for k in range(i1,i2): print("     - ", dc[k])
    for k in range(j1,j2): print("     + ", dd[k])
# first differing line, with its enclosing declaration
sm2=difflib.SequenceMatcher(None, comm, cand, autojunk=False)
first=None
for tag,i1,i2,j1,j2 in sm2.get_opcodes():
    if tag!='equal': first=(tag,i1,i2,j1,j2); break
if first is None:
    print("IDENTICAL after all -> no drift")
else:
    tag,i1,i2,j1,j2=first
    enc=next((d for d in reversed(dc) if d[0]<=i1+1), None)
    enced=next((d for d in reversed(dd) if d[0]<=j1+1), None)
    print(f"FIRST DIFFERING LINE: {tag} committed line {i1+1}-{i2}, candidate line {j1+1}-{j2}")
    print(f"  enclosing declaration (committed): {enc}")
    print(f"  enclosing declaration (candidate): {enced}")
    for k in range(i1,min(i2,i1+6)): print(f"   committed:{k+1}: {comm[k][:160]}")
    for k in range(j1,min(j2,j1+6)): print(f"   candidate:{k+1}: {cand[k][:160]}")
PY
else
  echo "NO CANDIDATE v1_compiler_infer.rs -- drift is elsewhere; listing what changed:"
  diff -rq "$SEED" "$CAND" 2>/dev/null | head -10
fi
cp /tmp/drift.log drift_cleanmain.log
for f in $MIRRORS; do cp "seedgreen/$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: dag=$(sha256sum src/v1/04_infer.dag|cut -c1-12) seed=$(sha256sum $SEED/v1_compiler_infer.rs|cut -c1-12)"
echo DRIFT_DONE
