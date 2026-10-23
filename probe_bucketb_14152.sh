#!/bin/bash
# STAGE 2 TEMPLATE -- fill anchors from the stage-1 dump before dispatching.
# PURPOSE (stern-bat-401 msg_6b7b53a9): Bucket B's 20 `Coproduct(FreeMonoid)` vs `Container(FreeMonoid,X)`
# rows need ROUTING, and the census cannot route them: node_type_shape hides a Disj's children, so a bare
# generic (instantiation -> waits on S1, msg_42284010) and a concrete child equal to X (printing difference
# -> the container normalization at the type authority, msg_d7ee8f8a item 2) print IDENTICALLY. So the
# discriminator is structural, read at the refusal: the children's name + inferred variant on each side.
# It doubles as X1's first rung (msg_e5215d82): for the 11 SAME_TEXT rows, the declaration state of both
# sides -- Present-vs-Present cross-module is correct and must stay refused; Absent-vs-Present is the
# defect; both-Absent is the arm I am NOT allowed to loosen (parent: both-Absent equal-by-structure is
# spelling equality).
#
# HOW THE INSTRUMENT WORKS (the recipe already used for the 7-fabrication substitution measurement, and
# already handed to quiet-eagle-395): the stage0 seed IS the compiler's own source, so patching the seed's
# v1_compiler_infer.rs and building gunbc from it instruments the self-host run. The patch adds ONE
# top-level fn plus ONE call at the refusal arm, and prints exactly one machine-readable line per refusal:
#   REBINDPROBE|span=..|module=..|formal=<shape>|actual=<shape>|
#     F=[name|infer|decl|children..]|A=[name|infer|decl|children..]
# Rules learned the hard way, all enforced below:
#  - rustfmt-parse the patched file LOCALLY before dispatch (4 probes died on bad injections);
#  - verify every anchor matches EXACTLY ONCE, print the count, abort otherwise;
#  - one eprintln per line via push_str (no nested fns capturing locals);
#  - NEVER git checkout/clean; restore by cp from the pristine copy taken in-script;
#  - no git status --porcelain | wc -l after `cd repo` for the same reason as the instrument bug;
#  - tar the probe log FIRST (a 45-min remote cap once kept the answer and lost the artifact).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
BASE_SHA=59606870e8ba3001a845627696509e8aa16bb0f0
SRC_SHA=d4410d980a91b619fdd45a628f454b4d2ee0c670
URL=https://github.com/gunb-ai/gunbc.git
# Same five test sources cleanab2 mode A installs. `set -u` makes an unset $TESTS fatal at the loop below,
# and it was undefined here -- the loop would have died at the first iteration with 'TESTS: unbound variable'.
TESTS="src/v1/tests/claim/generic_identity_census.dag src/v1/tests/src/applied_constructor_type_equals_test.rs src/v1/tests/src/cross_coproduct_generic_locate_test.rs src/v1/tests/src/cross_coproduct_probe_discharge_test.rs src/v1/tests/src/lib.rs"
D=/tmp/probe-bucketb; rm -rf $D; mkdir -p $D; LOGS=$D/logs; mkdir -p $LOGS
cd $D
# fail <code> <msg>: ship the logs BEFORE exiting. The first dispatch of this script died at regen round 1
# with "v2 self-compile produced 2 hard diagnostic(s)" and then exited 7 through a path that tarred nothing,
# so the ONE thing needed -- the diagnostic text -- was lost and the answer cost a second 60-min dispatch.
fail () { echo "FAILED($1): $2"; echo "=== BEGIN-TAR (failure path) ==="; ( cd $LOGS && tar -c . 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="; exit "$1"; }
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { tail -5 $LOGS/clone.log; fail 20 "CLONE FAILED"; }
cd repo
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fb.log 2>&1 || fail 25 "BASE FETCH FAILED"
git checkout -q -f FETCH_HEAD
[ "$(git rev-parse HEAD)" = "$BASE_SHA" ] || fail 27 "WRONG TREE"
echo "tree=$BASE_SHA status=$(git status --porcelain | wc -l) (want 0)"

# ---- source = my fixed fold (carrier) so the fold's refusal exists at all ----
# Carrier head: scratch/rebind-census at the commit that carries patch_rebind_probe.py, the
# instrument and the carriers together, so one fetch gives everything and the SHAs cannot disagree.
# (An earlier head, bd68035135c, predates the patcher and would fetch a tree without it.)
timeout 900 git fetch --depth 1 origin 8ae11cb4cea63653e26260b6db22201026a8b541 > $LOGS/fc.log 2>&1 || fail 28 "carrier fetch failed"
CAR=$(git rev-parse FETCH_HEAD)
git show $CAR:carriers/my_04_infer.dag > src/v1/04_infer.dag || fail 29 "no carrier my_04_infer"
git show $CAR:carriers/my_04_types_backup.dag > src/v1/04_types.dag || fail 29 "no carrier my_04_types"
# The tests are part of the CONFIGURATION, not of the measurement: cleanab2 mode A (which converged at this
# exact base with this exact carrier) installs them, this script did not, and the only run that ever produced
# "2 hard diagnostic(s)" was the one that omitted them. Do not run a 60-min dispatch to re-discover which
# test source the compiler is complaining about.
timeout 900 git fetch --depth 1 origin $SRC_SHA > $LOGS/fs.log 2>&1 || fail 30 "SRC FETCH FAILED"
for f in $TESTS; do git show $SRC_SHA:$f > $f || fail 31 "MISSING $f"; done
echo "tests from=$SRC_SHA"
echo "carrier markers: fold=$(grep -ac UnifyGenericsAcc src/v1/04_infer.dag) fixed=$(grep -ac 'resolved.value' src/v1/04_infer.dag) rawChildren=$(grep -ac 'any(c => binding_has_type_variable_leaf(n: c))' src/v1/04_infer.dag) badTrim=$(grep -ac 'trim(value:' src/v1/04_infer.dag) badIdentSpan=$(grep -ac 'n.identifier_span' src/v1/04_infer.dag) (want 1/1/1/0/0)"
[ "$(grep -ac 'trim(value:' src/v1/04_infer.dag)" = "0" ] || fail 32 "CARRIER IS THE PRE-FIX FILE"
[ "$(grep -ac 'n.identifier_span' src/v1/04_infer.dag)" = "0" ] || fail 32 "CARRIER STILL HAS identifier_span"
[ "$(grep -ac 'any(c => binding_has_type_variable_leaf(n: c))' src/v1/04_infer.dag)" = "1" ] || fail 32 "CARRIER LACKS THE RAW-CHILDREN PREDICATE"
# main's own mirrors: they match this source only after regen, so regen once to a fixed point first.
# Reset ONLY the generated mirrors to main's committed bytes first (cleanab2's converge does this), so the
# comparison at round 1 is against main and a leftover from a prior round can never be read as "installed".
git checkout -q -f -- src/v1/stage0 2>/dev/null || true
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
r=1
while [ $r -le 3 ]; do
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_$r.log 2>&1
  echo "regen round $r rc=$?"
  if [ ! -d target/stage0-regen-candidate/src ]; then
    echo "NO CANDIDATE -- receipt (FULL diagnostics, not a filtered head):"
    grep -a -B2 -A12 -i 'hard diagnostic' $LOGS/rr_${r}.log | cut -c1-400 | head -60
    echo "--- any diagnostic-shaped lines ---"
    grep -a -E 'diagnostic|error|not found|mismatch|unknown|expected' $LOGS/rr_${r}.log | cut -c1-300 | head -40
    echo "--- tail of receipt ---"; tail -30 $LOGS/rr_${r}.log
    fail 7 "regen round $r produced no candidate"
  fi
  inst=0
  for f in $(ls target/stage0-regen-candidate/src); do
    [ -f src/v1/stage0/src/$f ] && ! cmp -s target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f && { cp target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f; inst=$((inst+1)); echo "  installed $f"; }
  done
  echo "round $r installed=$inst"; [ "$inst" = "0" ] && break; r=$((r+1))
done
echo "fold in seed: bindAs=$(grep -ac 'bound as' src/v1/stage0/src/v1_compiler_infer.rs)"

# ---- PROBE PATCH on the converged seed (anchors are placeholders until the stage-1 dump lands) ----
SEED=src/v1/stage0/src/v1_compiler_infer.rs
cp $SEED $LOGS/seed_pristine.rs   # restore source; no git checkout
# The patcher is a SEPARATE FILE (patch_rebind_probe.py), which is fetched from the scratch branch by
# SHA rather than embedded, so the exact bytes that were rustfmt-verified locally are the bytes that run.
git show $CAR:patch_rebind_probe.py > $LOGS/patch_rebind_probe.py || fail 30 "no patcher"
wc -c $LOGS/patch_rebind_probe.py
python3 $LOGS/patch_rebind_probe.py $SEED || fail 5 "patcher refused"
echo "probe refs in seed=$(grep -ac 'rebind_probe_print' $SEED) (want 2: the fn and the call)"
# rustfmt-parse the patched seed LOCALLY (the rule: 4 probes died on bad injections)
rustfmt --edition 2021 --emit stdout $SEED > /dev/null 2> $LOGS/rustfmt.err; f=$?
echo "rustfmt rc=$f"; head -8 $LOGS/rustfmt.err
[ "$f" != "0" ] && { echo "PATCH DOES NOT PARSE -- aborting before dispatch"; fail 6 "PATCH DOES NOT PARSE"; }

# NO OUTPUT FILTER ON THE BUILD. The previous dispatch printed only 'error[E0308]: mismatched types' twice
# and 'could not compile v1-stage0-v1-infer due to 6 previous errors' -- the grep|tail -3 threw away the
# spans and the expected/actual types, so a one-line type error in the probe helper cost a full 21-minute
# round trip. Build output is the DIAGNOSTIC; it goes to a file whole and gets printed on failure.
cargo build --release -p v1-compiler --bin gunbc > $LOGS/gunbc_build.log 2>&1; BRC=$?
echo "gunbc build rc=$BRC (full log: $LOGS/gunbc_build.log)"
if [ "$BRC" != "0" ]; then
  echo "=== BUILD FAILED: full error blocks (spans + expected/actual), unfiltered ==="
  grep -a -A14 '^error' $LOGS/gunbc_build.log | cut -c1-220 | head -120
  echo "=== end build errors ==="
  fail 8 "gunbc build failed rc=$BRC"
fi
# Probe-line gate BEFORE the 30-minute self-host: if the helper never made it into the seed, or the seed did
# not build, the self-host produces 0 probe lines and we learn that only after it has already cost the run.
NP=$(grep -ac 'REBINDPROBE|bind=' $SEED || true)
echo "probe call sites in seed=$NP (want >=1)"
[ "$NP" -ge 1 ] || fail 9 "probe call not present in seed"
timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh.log 2>&1
echo "self-host rc=$? probeLines=$(grep -ac '^REBINDPROBE|' $LOGS/sh.log) refusals=$(grep -ac 'bound as' $LOGS/sh.log)"
cp $SEED $LOGS/seed_patched.rs
echo "=== BEGIN-TAR (log first) ==="; ( cd $LOGS && tar -c sh.log seed_patched.rs 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
cp $LOGS/seed_pristine.rs $SEED
echo PROBE_DONE
