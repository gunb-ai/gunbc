#!/bin/bash
# THE MEASUREMENT, FINALLY ATTRIBUTABLE. Everything before this ran either in the SHARED runner
# workspace (sharp-boar-42: it carries other sessions' uncommitted .dag edits) or against seeds
# CONVERGED THERE, including carriers/converged3 -- which is why it is dropped entirely here. What the
# previous dispatch (refusable_14152.sh) established, at a pinned clean clone of main with my source
# applied, is that the ONLY hard diagnostics were six, ALL in src/v1/04_infer.dag, and ALL MINE and
# PRE-FIX: variant 'Present'/'Absent' not found in type 'Node' (:859620,:859751 -- my match over
# children, which is List<Node>), no field 'identifier_span' on type 'Node' (:860326), function 'trim'
# not found in scope + its effect-summary consequence (:861022). No 02_parse pair, no rebind refusal:
# the tree never got that far. The cause is a bookkeeping error of mine -- the fixes for exactly those
# six live only in the merge INDEX, not in the pushed branch head d4410d980a9, which predates them.
#
# SO: seed = the clone's OWN stage0 mirrors (main's), source = the FIXED .dag from the scratch carrier
# (carriers/my_04_infer.dag, verified identical to my index), tests = my branch. Regen must converge
# main's seed against my source; if it refuses, the receipt is the answer and NO suite runs, because a
# suite on an unconverged seed is what produced my last two void results. Two configs, one clone:
#   A = my fold, no peer arm.  B = same + quiet-eagle-395's relation at 835cbd38e1e (peer arm carrier).
# The fold's own relation stays MYSELF in both, so the only variable is 04_types.dag, and the answer is
# the B-minus-A census difference BY IDENTITY -- the split the parent asked for.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
BASE_SHA=59606870e8ba3001a845627696509e8aa16bb0f0
CARRIER_SHA=bd68035135c62c17dd5be2777ff9770955b58f50
SRC_SHA=d4410d980a91b619fdd45a628f454b4d2ee0c670
URL=https://github.com/gunb-ai/gunbc.git
TESTS="src/v1/tests/claim/generic_identity_census.dag src/v1/tests/src/applied_constructor_type_equals_test.rs src/v1/tests/src/cross_coproduct_generic_locate_test.rs src/v1/tests/src/cross_coproduct_probe_discharge_test.rs src/v1/tests/src/lib.rs"
D=/tmp/cleanab-14152; rm -rf $D; mkdir -p $D; LOGS=$D/logs; mkdir -p $LOGS
cd $D
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
echo "main HEAD at clone time=$(git rev-parse HEAD)"
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fb.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
git checkout -q -f FETCH_HEAD || { echo "CHECKOUT FAILED"; exit 26; }
SHA=$(git rev-parse HEAD); echo "PINNED main=$SHA"; [ "$SHA" = "$BASE_SHA" ] || { echo "WRONG TREE"; exit 27; }
echo "dirty before anything=$(git status --porcelain | wc -l) (want 0)  SEED sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
# carriers: fixed source from the scratch branch, tests from the WIP branch. Both by SHA, both printed.
timeout 900 git fetch --depth 1 origin $CARRIER_SHA > $LOGS/fc.log 2>&1 || { echo "CARRIER FETCH FAILED"; exit 28; }
CAR=$(git rev-parse FETCH_HEAD); echo "carriers=$CAR (want prefix $CARRIER_SHA)"
git show $CAR:carriers/my_04_infer.dag > $LOGS/my_04_infer.dag        || { echo "no carrier my_04_infer"; exit 29; }
git show $CAR:carriers/peer_arm_04_types.dag > $LOGS/peer_arm_04_types.dag || { echo "no carrier peer_arm"; exit 29; }
git show $CAR:carriers/my_04_types_backup.dag > $LOGS/my_04_types.dag || { echo "no carrier my_04_types"; exit 29; }
git show $CAR:tools/rebind_census.sh > $LOGS/rebind_census.sh         || { echo "no census"; exit 29; }
timeout 900 git fetch --depth 1 origin $SRC_SHA > $LOGS/fs.log 2>&1 || { echo "SRC FETCH FAILED"; exit 30; }
echo "tests from=$SRC_SHA"
for f in $TESTS; do git show $SRC_SHA:$f > $f || { echo "MISSING $f"; exit 31; }; done
echo "carrier markers: fixedPeel=$(grep -ac 'resolved.value' $LOGS/my_04_infer.dag) rawChildren=$(grep -ac 'any(c => binding_has_type_variable_leaf(n: c))' $LOGS/my_04_infer.dag) badTrim=$(grep -ac 'trim(value:' $LOGS/my_04_infer.dag) badIdentSpan=$(grep -ac 'n.identifier_span' $LOGS/my_04_infer.dag) (want 1,1,0,0)"
[ "$(grep -ac 'trim(value:' $LOGS/my_04_infer.dag)" = "0" ] || { echo "CARRIER IS THE PRE-FIX FILE -- abort"; exit 32; }
[ "$(grep -ac 'n.identifier_span' $LOGS/my_04_infer.dag)" = "0" ] || { echo "CARRIER STILL HAS identifier_span -- abort"; exit 32; }

converge () { # $1 tag, $2 = 04_types to install
  local T=$1 TY=$2 CAND=target/stage0-regen-candidate/src SEED=src/v1/stage0/src
  echo "########## CONFIG $T ##########"
  git checkout -q -f -- src/v1/stage0 2>/dev/null || true    # reset ONLY the generated mirrors to main's
  cp $LOGS/my_04_infer.dag src/v1/04_infer.dag; cp $TY src/v1/04_types.dag
  echo "$T src: fold=$(grep -ac UnifyGenericsAcc src/v1/04_infer.dag) arm=$(grep -ac arguments_agree_in_lockstep src/v1/04_types.dag) helpers=$(grep -ac 'fn type_head_carrier' src/v1/04_types.dag)"
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  rm -rf target/stage0-regen-candidate 2>/dev/null
  local r=1 inst=1
  while [ $r -le 3 ] && [ "$inst" != "0" ]; do
    target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_${T}_${r}.log 2>&1
    echo "=== $T round $r rc=$?"; grep -a "^required-regen" $LOGS/rr_${T}_${r}.log | head -2 | cut -c1-300
    if [ ! -d "$CAND" ]; then
      echo "$T NO CANDIDATE: the receipt above IS the answer. Hard diagnostics verbatim:"
      grep -aE "^type mismatch|^call shape|arm decides|not found in|no field|effect summary" $LOGS/rr_${T}_${r}.log | head -12 | cut -c1-300
      return 7
    fi
    inst=0
    for f in $(ls "$CAND" 2>/dev/null); do
      if [ -f "$SEED/$f" ]; then cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "   installed $f"; };
      else echo "   CANDIDATE NOT IN SEED (reported, never committed): $f"; fi
    done
    echo "   $T round $r installed=$inst"; r=$((r+1))
  done
  echo "$T SEED PROOF: fold bindAs=$(grep -ac 'bound as' $SEED/v1_compiler_infer.rs) U=$(grep -ac UnifyGenericsAcc $SEED/v1_compiler_infer.rs) | arm lockstep=$(grep -ac arguments_agree_in_lockstep $SEED/v1_compiler_infer_types.rs) appliedSpelling=$(grep -ac applied_spelling_arguments_agree $SEED/v1_compiler_infer_types.rs) | sha=$(sha256sum $SEED/v1_compiler_infer.rs | cut -c1-16)"
  if [ "$(grep -ac 'bound as' $SEED/v1_compiler_infer.rs)" -eq 0 ]; then echo "$T ABORT: fold not in seed"; return 7; fi
  if [ "$T" = "B" ] && [ "$(grep -ac arguments_agree_in_lockstep $SEED/v1_compiler_infer_types.rs)" -eq 0 ]; then
    echo "$T ABORT: arm not in seed; refusing to run a suite that would be config A's numbers"; return 7; fi
  echo "--- $T SUITE (compiled against THIS config's regenerated mirror, so it tests this seed) ---"
  cargo build --release -p v1-compiler-tests --tests > $LOGS/tb_${T}.log 2>&1
  echo "$T test build rc=$?"; grep -E "^error|error\[" $LOGS/tb_${T}.log | head -6 | cut -c1-200
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error\[|panicked|assertion" | sed -E 's/^test cross_coproduct_generic_locate_test:://' > $LOGS/suite_${T}.txt
  if [ ! -s $LOGS/suite_${T}.txt ]; then echo "$T SUITE PRODUCED NO OUTPUT: FAILURE, not a pass"; tail -15 $LOGS/tb_${T}.log; return 8; fi
  echo "$T suite passed=$(grep -c 'ok$' $LOGS/suite_${T}.txt) failed=$(grep -c FAILED $LOGS/suite_${T}.txt)"
  grep FAILED $LOGS/suite_${T}.txt | cut -c1-100 | sed 's/^/   FAIL /'
  grep -E "disj_and_applied|two_applied_box|bare_head|alias_and_its_applied" $LOGS/suite_${T}.txt | cut -c1-100 | sed 's/^/   PAIR /'
  echo "--- $T SELF-HOST ---"
  cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh_${T}.log 2>&1
  echo "$T self-host rc=$? rebind=$(grep -ac 'bound as' $LOGS/sh_${T}.log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' $LOGS/sh_${T}.log | sort -u | wc -l)"
  bash $LOGS/rebind_census.sh $LOGS/sh_${T}.log --json $LOGS/census_${T}.json > $LOGS/census_${T}.txt 2>&1
  echo "$T census rc=$?"; grep -a "total rows=\|classes:" $LOGS/census_${T}.txt | head -2 | cut -c1-300
  return 0
}

converge A $LOGS/my_04_types.dag; RA=$?
converge B $LOGS/peer_arm_04_types.dag; RB=$?
echo "########## RESULT AVAILABILITY: A rc=$RA B rc=$RB (a comparison with a failed side is not a comparison) ##########"
if [ "$RA" = "0" ] && [ "$RB" = "0" ]; then
python3 - <<'PY'
import json,collections,re
def load(t):
    try: return json.load(open('/tmp/cleanab-14152/logs/census_%s.json'%t))
    except Exception as e: print("load %s failed: %s"%(t,e)); return None
A,B=load('A'),load('B')
if A is None or B is None: raise SystemExit
ident=lambda r:(r.get('formal',''),r.get('actual',''),r.get('span',''))
ia={ident(r) for r in A}; ib={ident(r) for r in B}
print("rows: A=%d B=%d | admitted by the shared relation=%d | newly refused=%d"%(len(A),len(B),len(ia-ib),len(ib-ia)))
c=collections.Counter()
for r in A:
    if ident(r) not in ib: c[r.get('class','?')]+=1
print("admitted, by class:",dict(c))
print("still refused, by class:",dict(collections.Counter(r.get('class','?') for r in A if ident(r) in ib)))
cop=[r for r in A if 'Coproduct(' in ''.join(ident(r)) and 'Node(' in ''.join(ident(r))]
kept=[r for r in cop if ident(r) in ib]; adm=[r for r in cop if ident(r) not in ib]
print("COPRODUCT-vs-NODE total=%d admitted=%d still-refused=%d"%(len(cop),len(adm),len(kept)))
print("  STILL REFUSED, by identity (first 15):")
for r in kept[:15]:
    f,a,s=ident(r); print("    %-36s | %-36s | %s"%(f[:36],a[:36],(s or '')[:30]))
PY
fi
echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c sh_A.log sh_B.log census_A.json census_B.json suite_A.txt suite_B.txt rr_A_1.log rr_B_1.log 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
echo CLEANAB_DONE
