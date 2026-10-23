#!/bin/bash
# CONFIG B: PR #14256's arm at 835cbd38e1e regenerated into the seed, then suite + self-host + census.
# HEAD CHOICE: an earlier dispatch of this file targeted a268c0272d7 and was killed at 8 minutes,
# because quiet-eagle-395 moved the head underneath it -- 274c274154b replaces the quadratic
# applied_children_agree with one linear lockstep walker (arguments_agree_in_lockstep), which IS the
# arm, so a census of a268c would measure a relation that no longer exists. 835cbd38e1e adds only
# quiet-eagle-395's own copy of my Box pair as a test; the source change is 274c274154b.
# Carrier keep/peer_arm_04_types.dag was regenerated from 835cbd38e1e the same way (peer file + my 3
# helpers; the checker's "missing TextJudgment arms" is a false alarm -- TextNotAsked and friends are
# VARIANTS inside `type TextJudgment` at :953, which a line-anchored `^type X` scan cannot see).
#
# CARRIER NOTE (measured, and CORRECTED after I first read it wrong). keep/peer_arm_04_types.dag is
# a UNION SCAFFOLD: quiet-eagle-395's 04_types.dag at 835cbd38e1e + the three fns my branch added
# (type_head_carrier, type_head_is_declared_carrier_alias, type_heads_agree), which exist ONLY on my
# branch -- origin/main defines none of them, and neither does the peer's file. I first wrote this up
# as a "landing conflict" claiming the peer's post-merge file had LOST TextJudgment and its arms.
# That was false and I checked it: their file has TextJudgment 6x and TextNotAskedForCallableComponent
# Residue, same counts as mine. And `git diff --stat $(merge-base HEAD MERGE_HEAD) MERGE_HEAD --
# src/v1/04_types.dag` is EMPTY -- main never touched 04_types in this merge, which is why my index
# matches HEAD byte for byte. So this is an ordinary merge of two files that add different things,
# not a conflict; the scaffold exists only because I need BOTH sides' changes to run one measurement.
# It is throwaway and never committed.
#
# BEHAVIOURAL PRESENCE PROOF, because a source-level marker is not a seed-level marker: an earlier
# dispatch of this same run aborted at its own check, which grepped the .dag spelling and found
# nothing even though the arm was installed -- the emitter does not reproduce that text. The
# reliable proof is the peer's own row: same_arm_bare_head_and_its_application_is_accepted is RED
# without the arm and GREEN with it, and nothing else in this tree can flip that row.
#
# ALREADY MEASURED, same head, same test file:
#   CONFIG A (no arm): 25 passed / 3 failed -- same_arm_alias..., same_arm_bare_head..., and
#     fold_accepts_disj_and_applied_..._same_arguments. fold_refuses_disj_and_applied_..._different_
#     arguments PASSED (the baseline fold refuses Box<Int> vs a Disj Box<Bool>).
# EXPECTED HERE if the arm reached the seed: bare_head GREEN, accepts_disj_same_arguments GREEN, and
# the DIFFERENT-arguments refusal control RED -- that last one is the hole quiet-eagle-395 has agreed
# to narrow, so report whichever way it goes rather than assuming.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
REF=keep/converged3
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
cp keep/my_04_infer.dag src/v1/04_infer.dag

echo "########## install baseline (my converged seed) ##########"
for f in $MIRRORS; do cp "$REF/$f" "$SEED/$f"; done
echo "baseline seed: bindAs=$(grep -ac 'bound as' $SEED/v1_compiler_infer.rs) U=$(grep -ac UnifyGenericsAcc $SEED/v1_compiler_infer.rs) (want bindAs>0 and U>0: my fold present) sha=$(sha256sum $SEED/v1_compiler_infer.rs | cut -c1-16)"
[ "$(grep -ac UnifyGenericsAcc $SEED/v1_compiler_infer.rs)" -gt 0 ] || { echo "WRONG BASELINE: fold absent from seed; aborting"; exit 6; }

echo "########## GATE ##########"
cargo check -p v1-compiler-tests --tests > /tmp/g2.log 2>&1; g=$?; tail -12 /tmp/g2.log; echo "gate rc=$g"
[ "$g" != "0" ] && { echo "test crate does not compile"; exit 4; }

echo "########## apply PR #14256's arm to the SOURCE, then regenerate the seed ##########"
cp keep/peer_arm_04_types.dag src/v1/04_types.dag
echo "source dag: lockstepWalker=$(grep -ac arguments_agree_in_lockstep src/v1/04_types.dag) (want >=2) appliedSpelling=$(grep -ac applied_spelling_arguments_agree src/v1/04_types.dag) (want >=1) declIdentity=$(grep -ac type_declaration_identity_key src/v1/04_types.dag) (want >=1) myHelpers=$(grep -ac 'fn type_head_carrier' src/v1/04_types.dag) (want 1)"

# GATE 1 -- PARSE, before spending an hour building. My first scaffold was malformed and the runner
# said so in five lines: 'module index refused: 1 unparseable .dag source(s) /
# src/v1/04_types.dag:93247-93247: heads-only parse: unterminated function body'. The cause was my
# extractor, which ended each appended fn at the FIRST line whose strip is '}' -- a nested match arm
# closer, not the function's own. So an unbalanced file reaches the runner, the regenerator refuses
# at corpus load, and there is no candidate tree. Fail here instead of at minute 60.
python3 - <<'PY'
import sys
s=open('src/v1/04_types.dag').read()
d=s.count('{')-s.count('}')
print("gate1 brace depth =",d,"(want 0)  bytes =",len(s))
if d!=0:
    print("ABORT: unbalanced .dag would be refused at corpus load"); sys.exit(9)
PY
[ "$?" != "0" ] && { echo "SOURCE DOES NOT PARSE-BALANCE; aborting"; exit 9; }
i=1
while [ $i -le 3 ]; do
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/ob_$i.log 2>&1
  echo "=== round $i rc=$? diag=$(grep -acE '^type mismatch|^call shape|arm decides' /tmp/ob_$i.log)"
  grep -aE "^type mismatch|^call shape|arm decides" /tmp/ob_$i.log | head -4 | cut -c1-200
  [ -d "$CAND" ] || break
  inst=0
  for f in $(ls "$CAND" 2>/dev/null); do
    [ -f "$SEED/$f" ] || continue
    cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "  installed $f"; }
  done
  echo "round $i installed=$inst"; [ "$inst" -eq 0 ] && break; i=$((i+1))
done
echo "-- SEED-level proof of the arm (grep the emitted form, never the .dag text):"
for pat in arguments_agree_in_lockstep applied_spelling_arguments_agree generic_declaration_application type_declaration_identity_key; do
  echo "   seed $pat = $(grep -ac "$pat" $SEED/v1_compiler_infer_types.rs)"
done
echo "   seed myHelpers = $(grep -ac 'fn type_head_carrier' $SEED/v1_compiler_infer_types.rs) (scaffold, want >=1)"
# GATE 2 -- PROVE THE ARM IS IN THE SEED BEFORE RUNNING THE SUITE. My previous config-B dispatch ran
# a full suite after installing nothing and printed 25 passed / 3 failed, byte-identical to config A
# (same three rows, same :242/:257). That number is meaningless and I refused to report it. A suite
# that runs on an unregenerated seed looks exactly like a suite that ran on a regenerated one, so the
# colour of the rows is not evidence -- the seed's own text is.
if [ "$(grep -ac arguments_agree_in_lockstep $SEED/v1_compiler_infer_types.rs)" -eq 0 ]; then
  echo "ABORT: the arm is NOT in the seed. Printing the whole regen log as the answer, and NOT running"
  echo "the suite, because a suite on an arm-free seed is config A wearing config B's label."
  echo "--- full regen log:"; cat /tmp/ob_1.log
  cp keep/my_04_types_backup.dag src/v1/04_types.dag
  for f in $MIRRORS; do cp "$REF/$f" "$SEED/$f"; done
  cp keep/my_04_infer.dag src/v1/04_infer.dag
  exit 7
fi

echo "########## SUITE at config B ##########"
cargo build --release -p v1-compiler-tests --tests 2>&1 | grep -E "^error|error\[|Finished" | tail -3
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error|error\[|panicked|assertion|left:|right:" | head -48

echo "########## SELF-HOST at config B, with denominator ##########"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
timeout 2100 target/release/gunbc test //gunbc/instruments:self-host > /tmp/ob_sh.log 2>&1
echo "self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/ob_sh.log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' /tmp/ob_sh.log | sort -u | wc -l)"
echo "--- identical-printing rebind rows (expected==got after 'bound as'; the class my peel may cause):"
grep -a "bound as" /tmp/ob_sh.log | sed -E "s/.*bound as '([^']*)'.*got '([^']*)'.*/\1|\2/" | awk -F'|' '$1==$2' | sort | uniq -c | sort -rn | head -6 | cut -c1-170
cp /tmp/ob_sh.log selfhost_B3.log

echo "########## CENSUS: per-row split by identity, against the peer arm ##########"
bash rebind_census_14152.sh selfhost_B3.log --json /tmp/census_B.json > /tmp/census_B.txt 2>&1
echo "census rc=$?"; tail -45 /tmp/census_B.txt
echo "--- COPRODUCT-VS-NODE split, what the parent asked for:"
python3 - > /tmp/split.txt 2>&1 <<'PY2'
import json,collections,re
try: rows=json.load(open('/tmp/census_B.json'))
except Exception as e: print("no json:",e); rows=[]
def txt(r): return (r.get('formal','')+' '+r.get('actual',''))
cop=[r for r in rows if 'Coproduct(' in txt(r) and 'Node(' in txt(r)]
print("total rows=%d  coproduct-vs-node=%d" % (len(rows),len(cop)))
print("classes:",dict(collections.Counter(r.get('class','?') for r in rows)))
un=collections.Counter()
for r in cop:
    f=r.get('formal',''); a=r.get('actual','')
    key=(r.get('class','?'),
         re.sub(r'<.*','',f.split('(')[-1]) if '(' in f else f,
         re.sub(r'<.*','',a.split('(')[-1]) if '(' in a else a)
    un[key]+=1
for k,v in un.most_common(14): print("   ",v,k)
PY2
cat /tmp/split.txt

echo "=== BEGIN-TAR-LOG ==="
( cd . && tar -cz selfhost_B3.log /tmp/census_B.json /tmp/split.txt 2>/dev/null | base64 -w0 ); echo
echo "=== END-TAR-LOG ==="

echo "########## restore working tree ##########"
cp keep/my_04_types_backup.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "$REF/$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: peerMarkers=$(grep -ac applied_spelling_arguments_agree src/v1/04_types.dag) (want 0)"
echo CONFB_DONE
