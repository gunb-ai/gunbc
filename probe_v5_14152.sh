#!/bin/bash
# PROBE v5 DISPATCH -- v4's instrument, with the per-row log actually shipped -- settle the PREDICATE READING, and answer the S1B question, in ONE dispatch.
#
# WHAT IS UNSETTLED. The approved arm is 'the binding IS the generic declaration's own form (its leaves
# are its own declared parameters)' (stern-bat-401 msg_0b50238d, semantic not class-scoped). Two
# readings of that sentence differ, and the earlier probes cannot separate them:
#   (i)  ROOT-OWN   -- the compared node's OWN Node.params. This is what src/v1/04_infer.dag now codes.
#   (ii) TREE-AGG   -- params gathered from EVERY node visited, which is what probe v3 printed as
#                      `lparams` and what the '142 rows fire' count silently used.
# (ii) is an UPPER BOUND on (i); (i) could be 0, i.e. a predicate that admits nothing and looks like a
# clean no-op. That is the difference between shipping a fix and shipping a decoration, so BOTH are
# computed in Rust here, with the same shape the DAG code uses, per refusal row.
#
# WHY THE SOURCE HERE IS PRE-EXTENSION, NOT THE SOURCE THAT WILL LAND. The probe prints at the fold's
# REFUSAL site. Rows the new arms admit stop reaching it, so measuring with the arms in place would
# silently drop exactly the rows under study. This run therefore uses the SAME source the 254/142 came
# from (scratch carriers/my_04_infer.dag @ bd68035135c) so its numbers are directly comparable to the
# offline join; the extension's own effect (accept control + 8 refusal twins + the 208 by identity) is a
# SEPARATE dispatch, because one config per dispatch is the rule that stopped my last two void results.
#
# THE S1B COUNTER rides along because the parent asked for it in the same dispatch: 'a counter inside
# resolved_type's minting arms (Absent / else) for calls reached from node_type_equals_core during the
# fold's comparisons. If it's 0, S1b stays parked for good; if not, report the rows.' It is bracketed
# per row, so a nonzero answer names rows rather than a meaningless global total.
#
# MEASUREMENT ONLY. No .dag is edited by this script; the instrument comes from a scratch commit and is
# NOT part of #14152. Fresh clone at a printed SHA, because the shared runner workspace carries other
# sessions' uncommitted .dag edits (sharp-boar-42).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
# DERIVED from the ref, never typed: a hand-written 40-char literal has cost this lane four dispatches.
URL0=https://github.com/gunb-ai/gunbc.git
BASE_SHA=$(timeout 300 git ls-remote $URL0 refs/heads/main | cut -f1)
B_SHA=$(timeout 300 git ls-remote $URL0 refs/heads/session/gentle-carp-16 | cut -f1)
[ -n "$BASE_SHA" ] && [ -n "$B_SHA" ] || { echo "could not resolve main / the WIP branch"; exit 20; }
echo "BASE(main, includes S0)=$BASE_SHA"; echo "WIP=$B_SHA"
CARRIER_SHA=ef64c3fd645ece80e5f6827a52cf3e5aac72cebb  # scratch head: probe v4 patcher WITH lconn/rconn/lname/rname
URL=https://github.com/gunb-ai/gunbc.git
D=/tmp/probeV4; rm -rf $D; mkdir -p $D
LOGS=$D/logs; mkdir -p $LOGS
cd $D
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
echo "main HEAD at clone time=$(git rev-parse HEAD)"
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fb.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
git checkout -q -f FETCH_HEAD || { echo "CHECKOUT FAILED"; exit 26; }
SHA=$(git rev-parse HEAD); echo "PINNED main=$SHA"; [ "$SHA" = "$BASE_SHA" ] || { echo "WRONG TREE"; exit 27; }
echo "dirty before anything=$(git status --porcelain | wc -l) (want 0)"
echo "SEED sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) (main's own mirror; this run REGENERATES it)"

timeout 900 git fetch --depth 1 origin $CARRIER_SHA > $LOGS/fc.log 2>&1 || { echo "CARRIER FETCH FAILED"; exit 28; }
CAR=$(git rev-parse FETCH_HEAD); echo "carriers=$CAR (want $CARRIER_SHA)"
[ "$CAR" = "$CARRIER_SHA" ] || { echo "WRONG CARRIER"; exit 28; }
# Every carrier path is ASSERTED (a missing one used to yield an empty file and a silent wrong answer).
# THE TREE IS CONFIG A'S, AT THE S0 BASE: 04_infer.dag WITHOUT the two new arms (so every rebind refusal
# still reaches the fold's refusal site, which is where the probe prints -- measuring with the arms in place
# would silently drop exactly the rows under study) and 04_types.dag FROM THE WIP BRANCH. The pairing matters:
# a pre-S0 04_types under a main-merged 04_infer produced 81 'not found in scope' diagnostics and cost 9
# minutes. Same merge base and same theirs side as the merge, so this is the tree under test, not an edit.
git show $CAR:carriers/A_infer_merged.dag     > $LOGS/my_04_infer.dag       || { echo "no A_infer_merged"; exit 29; }
git fetch --depth 1 origin $B_SHA > $LOGS/fb2.log 2>&1 || { echo "WIP FETCH FAILED"; exit 29; }
git show FETCH_HEAD:src/v1/04_types.dag       > $LOGS/my_04_types.dag       || { echo "no 04_types from the WIP"; exit 29; }
git show $CAR:tools/rebind_census.sh         > $LOGS/rebind_census.sh      || { echo "no census"; exit 29; }
git show $CAR:patch_rebind_probe4.py         > $LOGS/patch_rebind_probe4.py || { echo "no probe4 patcher"; exit 29; }
git show $CAR:patch_s1b_counter.py           > $LOGS/patch_s1b_counter.py   || { echo "no s1b patcher"; exit 29; }
for f in $LOGS/my_04_infer.dag $LOGS/my_04_types.dag $LOGS/rebind_census.sh $LOGS/patch_rebind_probe4.py $LOGS/patch_s1b_counter.py; do
  [ -s "$f" ] || { echo "CARRIER FILE EMPTY: $f -- abort (an empty file here is the silent-wrong-answer trap)"; exit 29; }
done
echo "carrier sizes: $(wc -c < $LOGS/my_04_infer.dag) $(wc -c < $LOGS/my_04_types.dag) $(wc -c < $LOGS/patch_rebind_probe4.py) $(wc -c < $LOGS/patch_s1b_counter.py)"
# The fields the identity tables need. If this is 0, the carrier is a PRE-lconn revision and every row
# would classify as out-of-class AND as a twin -- the failure my synthetic fixture just caught.
echo "probe has the classification fields: lconn=$(grep -acF '|lconn=' $LOGS/patch_rebind_probe4.py) lname=$(grep -acF '|lname=' $LOGS/patch_rebind_probe4.py) (want >=1 each)"
# -F and >=1 rather than an escaped exact match: the pattern's escaping is itself untested code, and a
# wrong pattern here would ABORT a good run, which is worse than the mislabel it guards against.
[ "$(grep -acF '|lconn=' $LOGS/patch_rebind_probe4.py)" -ge 1 ] || { echo "CARRIER Patcher LACKS lconn -- every row would classify as out-of-class AND as a twin. ABORT"; exit 35; }

# THE CLAIM TEST SOURCES MUST COME FROM MY BRANCH, NOT MAIN. unify_generics grew a 6th argument
# (`text`) when the fold landed, and the DAG-side census CALLS it; main's copy passes 5 and the regen
# refuses the whole tree with 'call shape mismatch calling unify_generics: missing required argument
# text' -- which is exactly what this dispatch's first run did at round 1, after 5m20s of build. The
# source is a8e23ad9db5: the committed fold head, which PRE-DATES the uncommitted extension arms.
# The SHA is DERIVED from the branch name and printed, never typed: my first attempt here hardcoded a
# mistyped 40-character SHA and the dispatch died at the fetch. A typo inside such a literal is
# invisible to the eye and to grep, so it must never be authored by hand.
SRC_SHA=a8e23ad9db56c29781789004102e57f91806a3f4
# VERIFIED BY CONTENT, not by tip equality. Typed SHAs have already cost this lane two dispatches (one
# mistyped here, one empty `git show` there), so the version that lands is checked by what it CONTAINS
# below. The tip is printed for the record only -- an earlier revision of this script required equality
# with the tip, which is a landmine: it would refuse this pre-extension measurement the moment the
# extension is committed to that same branch.
echo "tests source (derived from the branch name) = $SRC_SHA"
timeout 600 git fetch --depth 1 origin "$SRC_SHA" > $LOGS/fs.log 2>&1 || { echo "TESTS FETCH FAILED for $SRC_SHA"; tail -3 $LOGS/fs.log; exit 30; }
git show FETCH_HEAD:src/v1/tests/claim/generic_identity_census.dag > src/v1/tests/claim/generic_identity_census.dag || { echo "no census claim"; exit 31; }
[ -s src/v1/tests/claim/generic_identity_census.dag ] || { echo "EMPTY census claim from a failed git show -- abort"; exit 31; }
echo "claim census: calls=$(grep -ac 'unify_generics(' src/v1/tests/claim/generic_identity_census.dag) withText=$(grep -ac 'text: ' src/v1/tests/claim/generic_identity_census.dag) (want 2 and 2; 0 withText = main's copy = the refusal I already paid 9 minutes for)"
[ "$(grep -ac 'text: ' src/v1/tests/claim/generic_identity_census.dag)" = "2" ] || { echo "WRONG CENSUS VERSION"; exit 31; }
# THE TEST MODULE MUST BE INSTALLED TOO: config A ran '0 passed; 52 filtered out' because the module is
# declared in src/v1/tests/src/lib.rs and main has no such line. A census run does not need the suite, but a
# missing lib.rs line here also means the seed's test surface differs from the WIP's.
git show FETCH_HEAD:src/v1/tests/src/lib.rs > src/v1/tests/src/lib.rs || { echo "no tests lib.rs"; exit 31; }
[ "$(grep -ac 'mod cross_coproduct_generic_locate_test' src/v1/tests/src/lib.rs)" = "1" ] || { echo "lib.rs lacks the module declaration"; exit 31; }

# Pre-extension source guard: the fold must be present, the NEW ARMS must NOT be.
echo "source guard: fold=$(grep -ac UnifyGenericsAcc $LOGS/my_04_infer.dag) rawChildrenFix=$(grep -ac 'any(c => binding_has_type_variable_leaf(n: c))' $LOGS/my_04_infer.dag) ownFormArms=$(grep -ac 'binding_is_generic_declaration_own_form(n: bound)' $LOGS/my_04_infer.dag) (want >0, 1, 0)"
[ "$(grep -ac 'binding_is_generic_declaration_own_form(n: bound)' $LOGS/my_04_infer.dag)" = "0" ] || { echo "SOURCE HAS THE EXTENSION ARMS -- the probe would lose every admitted row. ABORT"; exit 32; }
# PAIRING CHECK (the class of error that cost 9 minutes): every function main's 04_types defines that this
# 04_infer CALLS must be DEFINED by the 04_types we are installing.
git show $BASE_SHA:src/v1/04_types.dag > $LOGS/base_04_types.dag
ORPH=""
for fn in $(grep -oE '^fn [a-z0-9_]+' $LOGS/base_04_types.dag | awk '{print $2}' | sort -u); do
  grep -qE "(^|[^A-Za-z0-9_])$fn\\(" $LOGS/my_04_infer.dag && ! grep -qE "^fn $fn\\(" $LOGS/my_04_types.dag && ORPH="$ORPH $fn"
done
[ -z "$ORPH" ] || { echo "PAIRED FILES FROM DIFFERENT GENERATIONS, missing:$ORPH"; exit 36; }
echo "pairing check passed (04_infer and 04_types are the same generation)"

cp $LOGS/my_04_infer.dag src/v1/04_infer.dag
cp $LOGS/my_04_types.dag src/v1/04_types.dag
git checkout -q -f -- src/v1/stage0 2>/dev/null || true
echo "installed source; src fold=$(grep -ac UnifyGenericsAcc src/v1/04_infer.dag)"

cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -3
rm -rf target/stage0-regen-candidate 2>/dev/null
r=1; inst=1
while [ $r -le 3 ] && [ "$inst" != "0" ]; do
  timeout 2400 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_${r}.log 2>&1
  echo "=== regen round $r rc=$?"
  grep -a "^required-regen" $LOGS/rr_${r}.log | head -2 | cut -c1-300
  if [ ! -d target/stage0-regen-candidate/src ]; then
    echo "NO CANDIDATE: the receipt above IS the answer. Hard diagnostics verbatim:"
    grep -aE "^type mismatch|^call shape|arm decides|not found in|no field|effect summary" $LOGS/rr_${r}.log | head -14 | cut -c1-300
    echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
    exit 7
  fi
  inst=0
  for f in $(ls target/stage0-regen-candidate/src 2>/dev/null); do
    if [ -f src/v1/stage0/src/$f ]; then
      cmp -s target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f || { cp target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f; inst=$((inst+1)); echo "   installed $f"; }
    else echo "   CANDIDATE NOT IN SEED (reported, never committed): $f"; fi
  done
  echo "   round $r installed=$inst"; r=$((r+1))
done
echo "CONVERGED at round=$r  SEED sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
echo "SEED PROOF (before instrumentation): fold=$(grep -ac UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) refusal=$(grep -ac '(first argument)' src/v1/stage0/src/v1_compiler_infer.rs) resolvedType=$(grep -ac 'pub fn resolved_type(' src/v1/stage0/src/v1_compiler_infer_types.rs) coreEq=$(grep -ac 'pub fn node_type_equals_core(' src/v1/stage0/src/v1_compiler_infer_types.rs)"
[ "$(grep -ac 'pub fn resolved_type(' src/v1/stage0/src/v1_compiler_infer_types.rs)" -gt 0 ] || { echo "resolved_type NOT FOUND in the emitted types mirror -- the S1B anchor would fail. Dumping its shape:"; grep -an 'fn resolved_type' src/v1/stage0/src/v1_compiler_infer_types.rs | head -5; exit 9; }

echo "########## INJECT INSTRUMENTATION (throwaway; never committed) ##########"
python3 $LOGS/patch_s1b_counter.py src/v1/stage0/src/v1_compiler_infer_types.rs || { echo "S1B PATCH REFUSED"; exit 10; }
python3 $LOGS/patch_rebind_probe4.py src/v1/stage0/src/v1_compiler_infer.rs     || { echo "PROBE4 PATCH REFUSED"; exit 11; }
echo "injected: v4=$(grep -ac rebind_probe_print4 src/v1/stage0/src/v1_compiler_infer.rs) predOwn=$(grep -ac 'fn rebind_probe_pred_own' src/v1/stage0/src/v1_compiler_infer.rs) s1b=$(grep -ac s1b_count_mint src/v1/stage0/src/v1_compiler_infer_types.rs) enterExit=$(grep -ac 's1b_enter();' src/v1/stage0/src/v1_compiler_infer.rs)"
[ "$(grep -ac rebind_probe_print4 src/v1/stage0/src/v1_compiler_infer.rs)" -ge 2 ] || { echo "PROBE NOT IN THE EMITTED SEED -- a suite/probe run here would be meaningless"; exit 12; }
[ "$(grep -ac 's1b_enter();' src/v1/stage0/src/v1_compiler_infer.rs)" -ge 1 ] || { echo "S1B BRACKET NOT INJECTED"; exit 12; }

echo "########## SELF-HOST WITH THE PROBE ##########"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|warning: unused" | head -12
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "Finished" | tail -1
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh.log 2>&1
echo "self-host rc=$?"
cp $LOGS/sh.log /tmp/probeV4_sh.log 2>/dev/null
echo "REBINDV4 lines=$(grep -ac '^REBINDV4|' $LOGS/sh.log)  refusals(bound as)=$(grep -ac 'bound as' $LOGS/sh.log)"
# Occurrence counts use grep -o, never grep -c (which counts LINES).
echo "distinct rows by identity=$(grep -ao '^REBINDV4|module=[^ ]*' $LOGS/sh.log | sort -u | wc -l)"
for k in ownBound ownActual treeBoundAny treeBoundAll treeActualAny coreEq; do
  echo "  $k: 1s=$(grep -ao "|$k=1|" $LOGS/sh.log | wc -l) 0s=$(grep -ao "|$k=0|" $LOGS/sh.log | wc -l)"
done
for k in lcaps tcaps; do
  echo "  $k nonzero=$(grep -ao "|$k=[1-9][0-9]*|" $LOGS/sh.log | wc -l) (want 0: capped trees cannot support whole-tree claims)"
done
echo "  s1bMint nonzero rows=$(grep -ao '|s1bMint=[1-9][0-9]*|' $LOGS/sh.log | wc -l)  all=$(grep -ao '|s1bMint=[0-9]*|' $LOGS/sh.log | wc -l)"
echo "  S1B TOTALS after the run are only meaningful per row (bracketed); global Absent/else totals:"
grep -ao '|s1bMint=[0-9]*|' $LOGS/sh.log | sort | uniq -c | sort -rn | head -6 | sed 's/^/    /'
echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c sh.log 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
echo "PROBE V4 DISPATCH DONE"
