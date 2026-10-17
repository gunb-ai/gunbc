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
PEER_SHA=835cbd38e1eb69c8967ac2a79369e0feb22a1feb
SRC_SHA=d4410d980a91b619fdd45a628f454b4d2ee0c670
URL=https://github.com/gunb-ai/gunbc.git
TESTS="src/v1/tests/claim/generic_identity_census.dag src/v1/tests/src/applied_constructor_type_equals_test.rs src/v1/tests/src/cross_coproduct_generic_locate_test.rs src/v1/tests/src/cross_coproduct_probe_discharge_test.rs src/v1/tests/src/lib.rs"
MODE=${1:-A}
D=/tmp/cleanab2-$MODE; rm -rf $D; mkdir -p $D
# LOGS must be set HERE, before the first use. It was silently dropped when this file was derived from
# cleanab_14152.sh by a sed that replaced the D= line the assignment sat on, and under `set -u` the
# first $LOGS (the clone's own log redirect) then killed the dispatch at line 31 with 'LOGS: unbound
# variable' -- reported as 'CLONE FAILED', because the clone's redirect target did not exist.
LOGS=$D/logs; mkdir -p $LOGS
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
  # Mode C exists to read the EMITTED mirror, not to re-measure: the suite and self-host for config A are
  # already on record (24/2 and 208 rows at this exact pinned tree, run twice to the same numbers). Running
  # them again costs ~30 minutes of runner time and answers nothing new, so SKIP_SUITE lets mode C stop
  # right after convergence -- which is the only thing the dump needs.
  if [ "${SKIP_SUITE:-0}" = "1" ]; then echo "$T SKIP_SUITE=1: suite and self-host not run (already on record for this tree)"; return 0; fi
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

# 2. CONFIG B USES THE 3-WAY MERGE, NOT THE PEER'S FILE. My previous B swapped in
# `git show 835cbd38e1e:src/v1/04_types.dag` wholesale and thereby DELETED MY OWN seven modified lines
# in node_type_equals_core -- `peel_alias_rhs_once` on both sides and
# `type_heads_agree(...)` replacing `left_name != right_name` -- which is exactly the carrier-normalising
# arm my controls depend on. That is why B lost three rows A passes (same_arm_two_applied_box_int,
# same_arm_bare_head, same_arm_alias): the regression was my missing edit, not the peer's arm. B is now
# `git merge-file my04 base04 peer04`, all three fetched from origin by SHA: it merged CLEANLY with zero
# conflict markers and 103 definitions (peer adds 15 fns and removes none; mine adds 4 and removes none),
# which also retires the "landing conflict" I earlier claimed and then corrected.
if [ "$MODE" = "B" ]; then
  # The peer's commit must be FETCHED before `git show` can name it. The first B dispatch skipped this,
  # so `git show $PEER_SHA:src/v1/04_types.dag` failed and left a ZERO-BYTE peer04.dag; merge-file then
  # read that as "the peer deleted the whole file", reported rc=1 with two conflict markers and
  # bytes=82290, and my own gate exited 33 before any measurement. An empty file from a failed git show
  # is the same trap as an empty suite: it looks like data. Assert size, not just existence.
  timeout 900 git fetch --depth 1 origin session/quiet-eagle-395-s0 > $LOGS/fp.log 2>&1 || { echo "PEER BRANCH FETCH FAILED"; tail -3 $LOGS/fp.log; exit 38; }
  PEER_HEAD=$(git rev-parse FETCH_HEAD); echo "peer branch head=$PEER_HEAD (want $PEER_SHA)"
  [ "$PEER_HEAD" = "$PEER_SHA" ] || echo "  NOTE: the peer MOVED the branch since this dispatch was written; measuring the branch head, and this line is what says so"
  PEER_SHA=$PEER_HEAD
  git cat-file -e $PEER_SHA^{commit} 2>/dev/null || { echo "peer commit $PEER_SHA absent"; exit 38; }
  git show $BASE_SHA:src/v1/04_types.dag > $LOGS/base04.dag || { echo "no base 04_types"; exit 33; }
  git show $PEER_SHA:src/v1/04_types.dag > $LOGS/peer04.dag || { echo "no peer 04_types"; exit 33; }
  for f in base04 peer04; do
    BZ=$(wc -c < $LOGS/$f.dag)
    [ "$BZ" -gt 50000 ] || { echo "$f is $BZ bytes -- a failed git show leaves an empty file; abort rather than merge nothing"; exit 39; }
  done
  git merge-file -p $LOGS/my_04_types.dag $LOGS/base04.dag $LOGS/peer04.dag > $LOGS/merged04.dag
  MRC=$?
  CM=$(grep -ac '^<<<<<<<\|^>>>>>>>' $LOGS/merged04.dag)
  echo "3-way merge rc=$MRC conflictMarkers=$CM bytes=$(wc -c < $LOGS/merged04.dag) (my04=$(wc -c < $LOGS/my_04_types.dag) peer04=$(wc -c < $LOGS/peer04.dag))"
  [ "$MRC" = "0" ] || { echo "MERGE NOT CLEAN -- B is not measurable without resolving by hand"; exit 33; }
  [ "$CM" = "0" ] || { echo "CONFLICT MARKERS PRESENT"; exit 34; }
  # The merge must contain BOTH sides' contributions or it proves nothing.
  for f in arguments_agree_in_lockstep applied_spelling_arguments_agree type_reference_identity_key; do
    [ "$(grep -ac "$f" $LOGS/merged04.dag)" -gt 0 ] || { echo "MERGE LOST PEER FN $f"; exit 35; }
  done
  for f in peel_alias_rhs_once type_head_carrier type_head_is_declared_carrier_alias type_heads_agree; do
    [ "$(grep -ac "fn $f" $LOGS/merged04.dag)" -gt 0 ] || { echo "MERGE LOST MY FN $f"; exit 36; }
  done
  # and specifically the arm my controls need: node_type_equals_core must call type_heads_agree
  [ "$(grep -ac 'if !type_heads_agree(' $LOGS/merged04.dag)" -gt 0 ] || { echo "MERGE LOST MY node_type_equals_core ARM"; exit 37; }
  echo "merge verified: peer 3 fns + my 4 fns + my node_type_equals_core arm all present"
  converge B $LOGS/merged04.dag; RB=$?
else
  if [ "$MODE" = "C" ]; then
  converge A $LOGS/my_04_types.dag; RA=$?
  echo "########## MIRROR SET FOR THE MERGE COMMIT (config A, clean pinned tree) ##########"
  for f in v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs; do
    echo "  $f sha256=$(sha256sum src/v1/stage0/src/$f | cut -c1-16) bytes=$(wc -c < src/v1/stage0/src/$f)"
  done
  echo "  fold present in mirror: bindAs=$(grep -ac 'bound as' src/v1/stage0/src/v1_compiler_infer.rs) U=$(grep -ac UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs)"
  echo "  raw-children fix emitted: $(grep -ac 'binding_has_type_variable_leaf' src/v1/stage0/src/v1_compiler_infer.rs) refs, resolved_type inside it: $(grep -A30 'fn binding_has_type_variable_leaf' src/v1/stage0/src/v1_compiler_infer.rs | grep -ac resolved_type) (want 0)"
  echo "  all fold symbols: $(for sy in UnifyGenericsAcc binding_has_type_variable_leaf generic_alias_unfold generic_types_unify generic_head_matches_application generic_types_are_the_text_pair rebind_refusal_check rebind_refusal_reason generic_type_unify_unfold; do printf '%s=%s ' $sy $(grep -ac "$sy" src/v1/stage0/src/v1_compiler_infer.rs); done)"
  # DUMP THE EMITTED REFUSAL ARM, because the next step patches it and I will not inject a seed patch
  # against text I have not seen (four probes died on bad injections; the rule is rustfmt-parse the patch
  # locally first, which requires the exact target text). Prints ~70 lines around the refusal string, plus
  # the accessor surface the probe would need: how emitted code reads a node's inferred and its children.
  echo "########## EMITTED REFUSAL REGION (probe patch target) ##########"
  LN=$(grep -an "(first argument)" src/v1/stage0/src/v1_compiler_infer.rs | head -1 | cut -d: -f1)
  echo "refusal string at line $LN"
  if [ -n "$LN" ]; then
    sed -n "$((LN-55)),$((LN+12))p" src/v1/stage0/src/v1_compiler_infer.rs | cat -n | sed 's/^/  /'
  fi
  echo "########## ACCESSOR SURFACE available at that point ##########"
  for pat in "fn node_type_shape" "fn authored_name_at" "InferredNode::TypeVariable" "InferredNode::Resolved" "InferredNode::CompilerError" "\.inferred" "\.children" "fn unify_binding_is_uninformative" "fn binding_has_type_variable_leaf"; do
    printf "  %-34s hits=%s firstLine=%s\n" "$pat" "$(grep -ac "$pat" src/v1/stage0/src/v1_compiler_infer.rs)" "$(grep -an "$pat" src/v1/stage0/src/v1_compiler_infer.rs | head -1 | cut -d: -f1)"
  done
  echo "--- how the emitted fold names its locals at the refusal (first 3 'let' lines above it):"
  [ -n "$LN" ] && sed -n "$((LN-55)),$((LN))p" src/v1/stage0/src/v1_compiler_infer.rs | grep -a "let " | tail -6 | sed 's/^/    /' | cut -c1-160
  echo "--- how emitted code renders an InferredNode (the probe must reuse this, not invent one):"
  grep -an "InferredNode::TypeVariable" -B2 -A6 src/v1/stage0/src/v1_compiler_infer.rs | head -24 | sed 's/^/  /' | cut -c1-170
  echo "--- eprintln/println availability in this file:"; grep -ac "eprintln!" src/v1/stage0/src/v1_compiler_infer.rs; grep -an "eprintln!" src/v1/stage0/src/v1_compiler_infer.rs | head -3 | cut -c1-120
  tar -cC src/v1/stage0/src v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs 2>/dev/null | base64 -w0; echo
  echo "CLEANAB2_C_DONE"; exit 0
fi
converge A $LOGS/my_04_types.dag; RA=$?
fi
# ONE CONFIG PER DISPATCH. The previous run did both configs in one dispatch, took 69 minutes, and the
# runner's own wait died with 'rpc error: DeadlineExceeded' during B's self-host -- losing B's census AND
# the single end-of-run tar, so I had A's numbers and nothing else. Each config now runs alone and tars
# its own artifacts, so a timeout can only ever cost the config that timed out. The B-minus-A split is
# computed LOCALLY from the two census JSONs this tar returns; it cannot be computed here, because the
# other config lives in a different dispatch and the runner's /tmp does not persist between them.
echo "########## CONFIG $MODE rc=${RA:-${RB:-?}} ##########"
export LOGS MODE
python3 - <<'PY'
import json,collections,os
mode=os.environ.get('MODE','?'); logs=os.environ.get('LOGS','')
p=os.path.join(logs,'census_%s.json'%mode)
print("config %s census: %s"%(mode,p))
if not os.path.exists(p):
    print("  NO CENSUS (its converge did not reach self-host; read the receipt above)")
    raise SystemExit
rows=json.load(open(p))
ident=lambda r:(r.get('formal',''),r.get('actual',''),r.get('span',''))
print("  rows=%d distinct=%d"%(len(rows),len({ident(r) for r in rows})))
print("  classes:",dict(collections.Counter(r.get('class','?') for r in rows)))
cop=[r for r in rows if 'Coproduct(' in ''.join(ident(r)) and 'Node(' in ''.join(ident(r))]
print("  COPRODUCT-vs-NODE=%d"%len(cop))
# THE X2 QUESTION (msg_782471b1: X2 SUSPENDED until the 02_parse pair reproduces on a clean clone at a
# printed SHA, and is DROPPED if it does not). The only honest test is whether ANY refusal row names
# src/v1/02_parse.dag in this census at all, by identity -- not a remembered count from a dirty run.
byfile=collections.Counter()
for r in rows:
    sp=r.get('span','') or ''
    byfile[sp.split(':')[0] if ':' in sp else sp or '<no span>']+=1
print("  rows by file:",dict(byfile.most_common(12)))
p=[r for r in rows if '02_parse' in (r.get('span','') or '')]
print("  02_parse ROWS=%d"%len(p))
for r in p[:10]:
    f,a,sp=ident(r); print("    PARSE %-40s | %-40s | %s | %s"%(f[:40],a[:40],(sp or '')[:40],r.get('class')))
# and the same-text class explicitly, since X2 was specified as a same-text pair
st=[r for r in rows if r.get('class')=='SAME_TEXT']
print("  SAME_TEXT rows=%d"%len(st))
for r in st[:12]:
    f,a,sp=ident(r)
    print("    SAME %-40s | %-40s | %s | equal=%s"%(f[:40],a[:40],(sp or '')[:40],f==a))
PY
echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c sh_${MODE}.log census_${MODE}.json census_${MODE}.txt suite_${MODE}.txt rr_${MODE}_1.log rr_${MODE}_2.log base04.dag peer04.dag merged04.dag 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
echo "CLEANAB2_${MODE}_DONE"
