#!/bin/bash
# THE ATTRIBUTABLE MEASUREMENT. sharp-boar-42 (via stern-bat-401): main does NOT drift -- my
# 'FAIL generated surface drift: v1_compiler_infer.rs' and the config-B numbers just returned (366
# rows, SAME_TEXT 7->187, suite 23/5) were taken in the SHARED runner workspace
# /root/workspace/repo-root, which carries other sessions' uncommitted .dag edits and moves between
# dispatches. So every number in this lane is re-taken here, in a FRESH CLONE printed at its SHA, and
# a run that cannot print that is not a measurement. Verified possible: clone rc=0, `status --porcelain`
# empty, and an arbitrary SHA is fetchable (probe_clone.sh).
#
# ONE clone, TWO configs, so the comparison is between two seeds of the same tree:
#   A = my fold, no peer arm.               B = same, plus quiet-eagle-395's relation at 835cbd38e1e.
# Each config CONVERGES its own seed to a fixed point against this clone (installing candidate
# mirrors, never git checkout/clean -- that wiped carriers mid-run once), then runs the suite and the
# self-host census. Baseline seed is carriers/converged3, NOT main's mirrors: the test crate calls
# unify_generics with the TextJudgment argument my branch adds, so main's mirrors do not compile it.
#
# CARRIERS come from origin at a printed SHA (scratch/rebind-census @ bd68035135c), never from a
# shared directory. Fold relation stays MYSELF in both configs; only 04_types.dag changes -- the
# question is what the shared relation does to the rebind census, not whose relation the fold uses.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
CARRIER_SHA=bd68035135c62c17dd5be2777ff9770955b58f50
# PINNED: every number already reported in this lane was taken against origin/main @
# 59606870e8ba3001a845627696509e8aa16bb0f0, which is what I merged. An unpinned clone would measure
# whatever main is at dispatch time and could not be compared with them.
BASE_SHA=59606870e8ba3001a845627696509e8aa16bb0f0
URL=https://github.com/gunb-ai/gunbc.git
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
D=/tmp/fresh-14152; rm -rf $D; mkdir -p $D/logs; cd $D
LOGS=$D/logs
echo "########## FRESH CLONE ##########"
timeout 600 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
echo "main HEAD at clone time=$(git rev-parse HEAD)"
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fetchbase.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
# A checkout INSIDE a fresh clone under /tmp is the point of the instrument; the rule this lane
# follows is never to checkout/clean in the SHARED workspace, where it destroyed carriers mid-run.
git checkout -q -f FETCH_HEAD || { echo "CHECKOUT FAILED"; exit 26; }
CLONE_SHA=$(git rev-parse HEAD); echo "PINNED clone SHA=$CLONE_SHA (want $BASE_SHA)"
[ "$CLONE_SHA" = "$BASE_SHA" ] || { echo "WRONG TREE -- abort"; exit 27; }
DIRTY=$(git status --porcelain | wc -l); echo "dirty lines in fresh clone=$DIRTY (want 0)"
[ "$DIRTY" != "0" ] && { echo "CLONE NOT CLEAN -- abort, nothing measured here is attributable"; exit 21; }
timeout 600 git fetch --depth 1 origin $CARRIER_SHA > $LOGS/fetch.log 2>&1 || { echo "FETCH FAILED"; exit 22; }
mkdir -p carriers/converged3
for f in my_04_infer.dag peer_arm_04_types.dag my_04_types_backup.dag; do
  git show FETCH_HEAD:carriers/$f > carriers/$f || { echo "missing carrier $f"; exit 23; }
done
for f in $MIRRORS; do git show FETCH_HEAD:carriers/converged3/$f > carriers/converged3/$f || { echo "missing seed $f"; exit 24; }; done
git show FETCH_HEAD:tools/rebind_census.sh > /tmp/rebind_census.sh
git show FETCH_HEAD:carriers/README.md > carriers/README.md
echo "carriers: $(wc -c < carriers/my_04_infer.dag) $(wc -c < carriers/peer_arm_04_types.dag) bytes; census=$(wc -c < /tmp/rebind_census.sh) bytes"
cp src/v1/04_types.dag carriers/clone_head_04_types.dag
echo "clone HEAD 04_types: mine-only helpers absent? type_head_carrier=$(grep -ac 'fn type_head_carrier' src/v1/04_types.dag) (0 expected) peerArm=$(grep -ac arguments_agree_in_lockstep src/v1/04_types.dag)"

run_config () { # $1 = tag, $2 = 04_types file to use, $3 = fold source
  local T=$1 TY=$2 INFER=$3
  echo "########## CONFIG $T ##########"
  SEED=src/v1/stage0/src; CAND=target/stage0-regen-candidate/src
  for f in $MIRRORS; do cp carriers/converged3/$f $SEED/$f; done
  cp $INFER src/v1/04_infer.dag; cp $TY src/v1/04_types.dag
  echo "$T source: lockstep=$(grep -ac arguments_agree_in_lockstep src/v1/04_types.dag) appliedSpelling=$(grep -ac applied_spelling_arguments_agree src/v1/04_types.dag) myHelpers=$(grep -ac 'fn type_head_carrier' src/v1/04_types.dag) foldIn04infer=$(grep -ac UnifyGenericsAcc src/v1/04_infer.dag)"
  # NO LOCAL BRACE/PARSE GATE, deliberately. My first version here counted '{' minus '}' and declared
  # carriers/my_04_infer.dag unbalanced at depth 1, aborting BOTH configs on a file that compiles,
  # regenerates and ran a full suite yesterday -- the extra brace is inside a string literal. Any
  # local heuristic either false-positives like that or is too weak to matter. The AUTHORITATIVE parse
  # is the runner's own, and it is fast and precise: a malformed .dag is refused at corpus load in
  # about five minutes with 'module index refused: N unparseable .dag source(s)' plus the exact
  # file:start-end span and reason, which is how I found my malformed appended block
  # (04_types.dag:93247, 'heads-only parse: unterminated function body'). So parse failures are caught
  # downstream by round 1's receipt, and the gates kept here are the ones local checks do PROVE:
  # the arm's presence in the EMITTED seed, and the candidate tree's existence.
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  rm -rf target/stage0-regen-candidate 2>/dev/null
  r=1; inst=1
  while [ $r -le 3 ] && [ "$inst" != "0" ]; do
    target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr_"$T"_"$r".log 2>&1
    echo "=== $T round $r rc=$? receipt:"; grep -a "^required-regen:" /tmp/rr_"$T"_"$r".log | head -3 | cut -c1-210
    inst=0
    if [ -d "$CAND" ]; then
      for f in $(ls "$CAND" 2>/dev/null); do
        if [ -f "$SEED/$f" ]; then cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "   installed $f"; };
        else echo "   CANDIDATE NOT IN SEED (reported, not committed): $f"; fi
      done
    else echo "   no candidate tree this round"; fi
    echo "   $T round $r installed=$inst"; r=$((r+1))
  done
  echo "$T SEED PROOF: fold bindAs=$(grep -ac 'bound as' $SEED/v1_compiler_infer.rs) U=$(grep -ac UnifyGenericsAcc $SEED/v1_compiler_infer.rs) | arm lockstep=$(grep -ac arguments_agree_in_lockstep $SEED/v1_compiler_infer_types.rs) appliedSpelling=$(grep -ac applied_spelling_arguments_agree $SEED/v1_compiler_infer_types.rs) | sha=$(sha256sum $SEED/v1_compiler_infer.rs | cut -c1-16)"
  if [ "$T" = "B" ] && [ "$(grep -ac arguments_agree_in_lockstep $SEED/v1_compiler_infer_types.rs)" -eq 0 ]; then
    echo "CONFIG B ABORT: the arm never reached the seed. A suite run now would be config A's numbers"; echo "under config B's label. Regen log follows as the answer:"; cat /tmp/rr_B_1.log 2>&1 | head -40; return 7
  fi
  echo "--- $T SUITE ---"
  cargo build --release -p v1-compiler-tests --tests 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error\[|panicked|assertion" | sed -E 's/^test cross_coproduct_generic_locate_test:://' > /tmp/suite_"$T".txt
  grep -c "ok$" /tmp/suite_"$T".txt | sed 's/^/'"$T"' suite passed=/'; grep -c "FAILED" /tmp/suite_"$T".txt | sed 's/^/'"$T"' suite failed=/'; grep "FAILED" /tmp/suite_"$T".txt | cut -c1-95
  grep -E "disj_and_applied|two_applied_box|bare_head|alias_and_its_applied" /tmp/suite_"$T".txt | cut -c1-95 | sed 's/^/  BOX-PAIR+ACCEPTS: /'
  echo "--- $T SELF-HOST (denominator included: an aborted run under-counts) ---"
  cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_"$T".log 2>&1
  echo "$T self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/sh_"$T".log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' /tmp/sh_"$T".log | sort -u | wc -l)"
  cp /tmp/sh_"$T".log /tmp/keep_sh_"$T".log
  bash /tmp/rebind_census.sh /tmp/sh_"$T".log --json /tmp/census_"$T".json > /tmp/census_"$T".txt 2>&1
  echo "$T census rc=$?"; grep -a "total rows=\|classes:" /tmp/census_"$T".txt | head -3 | cut -c1-300
}

run_config A carriers/my_04_types_backup.dag carriers/my_04_infer.dag
run_config B carriers/peer_arm_04_types.dag carriers/my_04_infer.dag

echo "########## THE SPLIT THE PARENT ASKED FOR (B minus A, by identity) ##########"
python3 - <<'PY'
import json,collections,re
def load(t):
    try: return json.load(open('/tmp/census_%s.json'%t))
    except Exception as e: return None
A,B=load('A'),load('B')
print("A=%s B=%s rows"%(len(A) if A is not None else 'ERR',len(B) if B is not None else 'ERR'))
if A is None or B is None: raise SystemExit
def ident(r): return (r.get('formal',''),r.get('actual',''),r.get('span',''))
ia={ident(r) for r in A}; ib={ident(r) for r in B}
print("rows only in A (ADMITTED by the shared relation): %d" % len(ia-ib))
print("rows only in B (NEWLY refused): %d" % len(ib-ia))
c=collections.Counter()
for r in A:
    if ident(r) not in ib: c[r.get('class','?')]+=1
print("  admitted, by class:",dict(c))
cop=[r for r in A if 'Coproduct(' in ident(r)[0]+ident(r)[1] and 'Node(' in ident(r)[0]+ident(r)[1]]
kept=[r for r in cop if ident(r) in ib]; adm=[r for r in cop if ident(r) not in ib]
print("COPRODUCT-vs-NODE: total=%d admitted=%d still-refused=%d"%(len(cop),len(adm),len(kept)))
def shape(r):
    f,a=ident(r)[0],ident(r)[1]
    h=lambda t: re.sub(r'<.*','',t.split('(')[-1]) if '(' in t else t
    return (h(f),h(a))
print("  still refused, by head pair:",dict(collections.Counter(shape(r) for r in kept).most_common(8)))
print("  admitted, by head pair:",dict(collections.Counter(shape(r) for r in adm).most_common(8)))
print("  STILL-REFUSED, first 12 by identity:")
for r in kept[:12]:
    f,a,s=ident(r); print("    %-38s | %-38s | %s | %s"%(f[:38],a[:38],(s or '')[:34],r.get('class')))
PY
echo "=== BEGIN-TAR (logs first: if the remote cap bites, the census JSONs are worthless without them) ==="
cd /tmp && tar -c keep_sh_A.log keep_sh_B.log census_A.json census_B.json suite_A.txt suite_B.txt 2>/dev/null | base64 -w0; echo
echo "=== END-TAR ==="
echo FRESH_DONE
