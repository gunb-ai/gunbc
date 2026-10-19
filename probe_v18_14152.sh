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
BASE_SHA=${BASE_SHA:-$(timeout 300 git ls-remote $URL0 refs/heads/main | cut -f1)}
B_SHA=$(timeout 300 git ls-remote $URL0 ${B_REF:-refs/heads/session/gentle-carp-16} | cut -f1)
[ -n "$BASE_SHA" ] && [ -n "$B_SHA" ] || { echo "could not resolve main / the WIP branch"; exit 20; }
echo "BASE(main, includes S0)=$BASE_SHA"; echo "WIP=$B_SHA"
# DERIVED from the ref, NEVER typed. This script died at 16 s (exit 29) carrying a hand-written 40-char
# literal for this SHA, and the commit it named did not contain carriers/A_infer_merged.dag -- the typed-SHA
# trap this lane has now paid for five times. The ref head does carry all four carrier files (checked with
# git cat-file -e on each). If the ref ever moves to a head without them, the per-file asserts below abort
# LOUDLY rather than feeding an empty file to the patcher.
CARRIER_SHA=$(timeout 300 git ls-remote $URL0 refs/heads/scratch/rebind-census | cut -f1)
[ -n "$CARRIER_SHA" ] || { echo "could not resolve scratch/rebind-census"; exit 20; }
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
# THE TREE UNDER TEST IS THE HEAD, CHECKED OUT WHOLE -- then the arms are cut from one file.
#
# Three wrong constructions died here, all the same class:
#   v8  installed a COMMITTED carrier snapshot, which predated the variant revert and refused to compile.
#   v9  installed head's 04_infer.dag + 04_types.dag into a MAIN clone -> the census CALLER stayed at main's
#       5-argument call and regen refused: "missing required argument 'text' (5 of 6 supplied)".
#   v10 installed `git diff --name-only BASE..HEAD` and copied those files, BASE being main's TIP -- which had
#       MOVED to aa761c29cfe. Files main changed after my branch point were therefore "restored" to my
#       branch's OLDER copies, silently reverting four upstream commits (#14330 alias-identity consolidation,
#       #14340, #14344, #14346) inside the tree under test. It printed "INSTALLED 49 head files", a number
#       that looks like diligence and is the defect. I killed that run rather than read a refusal count out
#       of a tree that exists nowhere.
# The invariant all three violated: the tree you measure must be a tree that EXISTS. So check the head out
# whole and change exactly ONE thing -- the arms -- through strip_arms.py, which asserts every edit.
git fetch --depth 1 origin $B_SHA > $LOGS/fb2.log 2>&1 || { echo "WIP FETCH FAILED"; exit 29; }
( cd $D/repo && git checkout -q FETCH_HEAD ) || { echo "HEAD CHECKOUT FAILED"; exit 29; }
[ "$(git -C $D/repo rev-parse HEAD)" = "$B_SHA" ] || { echo "VOID: tree is $(git -C $D/repo rev-parse HEAD), wanted $B_SHA"; exit 29; }
echo "CHECKED OUT the head whole: $B_SHA (tracked-dirty=$(git -C $D/repo status --porcelain | wc -l), want 0)"
git show $CAR:strip_arms.py           > $LOGS/strip_arms.py           || { echo "no strip_arms"; exit 29; }
git show $CAR:tools/rebind_census.sh  > $LOGS/rebind_census.sh        || { echo "no census"; exit 29; }
git show $CAR:patch_rebind_probe18.py > $LOGS/patch_rebind_probe18.py || { echo "no probe16 patcher"; exit 29; }
for f in $LOGS/strip_arms.py $LOGS/rebind_census.sh $LOGS/patch_rebind_probe18.py; do
  [ -s "$f" ] || { echo "CARRIER FILE EMPTY: $f -- abort (an empty file here is the silent-wrong-answer trap)"; exit 29; }
done
cd $D/repo
# PROVENANCE printed, not assumed: markers naming recent main commits. If one is missing, the checkout is not
# the tree I think it is, and every number below belongs to a fiction.
for probe_str in "aggregate products declare seed membership" "a_shared_cargo_target"; do
  echo "fingerprint '$probe_str': $(grep -ra "$probe_str" src/v1 dag/gunbc --include=*.dag --include=*.rs 2>/dev/null | wc -l) hit(s)"
done
[ "$(grep -ac 'binding_is_generic_declaration_own_form(n: bound)' src/v1/04_infer.dag)" = "1" ] || { echo "HEAD DOES NOT CARRY THE ARMS -- nothing to strip; abort rather than measure the same tree twice"; exit 29; }
grep -q 'text: TextJudgment' src/v1/04_infer.dag || { echo "the head's unify_generics has no text param -- the tree is not what this probe expects"; exit 29; }
cal_args=$(grep -o 'unify_generics([^)]*' src/v1/tests/claim/generic_identity_census.dag | head -1 | tr ',' '\n' | grep -ac ':')
fn_args=$(grep -m1 'fn unify_generics(' src/v1/04_infer.dag | tr ',' '\n' | grep -ac ':')
echo "CALLER AGREEMENT (one tree, so these must agree): census supplies ~$cal_args args, callee declares ~$fn_args params"
[ "$cal_args" = "$fn_args" ] || echo "WARNING: caller/callee counts differ -- the census would refuse to compile (this mismatch killed v9)"
cp src/v1/04_infer.dag $LOGS/head_04_infer.dag
python3 $LOGS/strip_arms.py $LOGS/head_04_infer.dag $LOGS/my_04_infer.dag || { echo "STRIP REFUSED -- abort"; exit 29; }
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
# PRE-FLIGHT the marker before calling the patcher, so a missing marker is a named failure rather than a
# Python traceback two lines from a "PROBE9 PATCH REFUSED" (v11 lost 20 minutes to an IndexError that read
# like a probe-design problem but was a CLI bug in my own patcher).
mkn=$(grep -ac "diagnostics: Rc::new(vec!\[type_mismatch_error(" src/v1/stage0/src/v1_compiler_infer.rs)
echo "pre-flight: diagnostic marker count in the regenerated seed = $mkn (want 1)"
[ "$mkn" = "1" ] || { echo "VOID: marker count $mkn -- the refusal arm is not uniquely locatable, refusing to guess"; exit 12; }
python3 $LOGS/patch_rebind_probe18.py src/v1/stage0/src/v1_compiler_infer.rs    || { echo "PROBE18 PATCH REFUSED"; exit 11; }
# The helper is typechecked against the REAL emitted field sets before the 20-minute build, because probe v1
# lost 21 minutes to an E0308 that a 30-second local rustc pass shows. It is fetched by ref, not by a
# guessed path. Non-fatal: the remote build is still the real gate; this only front-loads the cheap failure.
git show $CAR:instruments/stubcheck.py > $LOGS/stubcheck.py || echo "no stubcheck to fetch (continuing)"
if [ -s $LOGS/stubcheck.py ]; then
  timeout 900 python3 $LOGS/stubcheck.py $LOGS/patch_rebind_probe18.py 2>&1 | tail -10
  echo "stubcheck rc=$? (a FAIL here means the injected code will not build; the build below would fail the same way, minutes later)"
fi
echo "injected: v4=$(grep -ac rebind18_print src/v1/stage0/src/v1_compiler_infer.rs) predOwn=$(grep -ac 'fn rebind18_print' src/v1/stage0/src/v1_compiler_infer.rs) s1b=$(grep -ac s1b_count_mint src/v1/stage0/src/v1_compiler_infer_types.rs) enterExit=$(grep -ac 's1b_enter();' src/v1/stage0/src/v1_compiler_infer.rs)"
[ "$(grep -ac rebind18_print src/v1/stage0/src/v1_compiler_infer.rs)" -ge 2 ] || { echo "PROBE NOT IN THE EMITTED SEED -- a suite/probe run here would be meaningless"; exit 12; }
# (the S1B bracket guard that stood here is REMOVED: probe v9 dropped the s1b counter on purpose -- it
# measured 0 minting events on all 254 rows and S1b stays parked per stern-bat-401. The guard was a v10
# leftover demanding instrumentation this probe deliberately does not carry, and it killed a 20-minute
# run after everything that mattered had already succeeded.)

echo "########## SELF-HOST WITH THE PROBE ##########"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|warning: unused" | head -12
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "Finished" | tail -1
timeout 3000 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh.log 2>&1
echo "self-host rc=$?"
cp $LOGS/sh.log /tmp/probeV18_sh.log 2>/dev/null
echo "REBINDV8 lines=$(grep -ac '^REBIND18|' $LOGS/sh.log)  refusals(bound as)=$(grep -ac 'bound as' $LOGS/sh.log)"
# Occurrence counts use grep -o, never grep -c (which counts LINES).
echo "distinct rows by identity=$(grep -ao '^REBIND18|module=[^ ]*' $LOGS/sh.log | sort -u | wc -l)"
for k in ownBound ownActual treeBoundAny treeBoundAll treeActualAny coreEq; do
  echo "  $k: 1s=$(grep -ao "|$k=1|" $LOGS/sh.log | wc -l) 0s=$(grep -ao "|$k=0|" $LOGS/sh.log | wc -l)"
done
for k in lcaps tcaps; do
  echo "  $k nonzero=$(grep -ao "|$k=[1-9][0-9]*|" $LOGS/sh.log | wc -l) (want 0: capped trees cannot support whole-tree claims)"
done
echo "  s1bMint nonzero rows=$(grep -ao '|s1bMint=[1-9][0-9]*|' $LOGS/sh.log | wc -l)  all=$(grep -ao '|s1bMint=[0-9]*|' $LOGS/sh.log | wc -l)"
echo "  S1B TOTALS after the run are only meaningful per row (bracketed); global Absent/else totals:"
grep -ao '|s1bMint=[0-9]*|' $LOGS/sh.log | sort | uniq -c | sort -rn | head -6 | sed 's/^/    /'
# BOTH logs by name. sh.log alone would drop the second closure the parent's gate requires, and a closure
# whose log never arrives is indistinguishable from a closure that emitted nothing -- which reads as green.
echo "=== BEGIN-TAR ==="; ( cd $LOGS && tar -c sh.log native.log 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
echo "PROBE V4 DISPATCH DONE"
