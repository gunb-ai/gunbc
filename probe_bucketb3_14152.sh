#!/bin/bash
# PROBE v2 -- attribute each fold rebind refusal to its DECIDING ARM.
# PURPOSE (stern-bat-401 msg_576466dc): the SAME_TEXT rows came back from probe v1 IDENTICAL in every field
# v1 prints, so the refusal was unexplained. v2 prints the FULL pairwise tree (descending THROUGH
# InferredNode::Resolved{node}, which v1 collapsed) and calls the REAL pub relations, reporting the
# DIFFERENCE between them, which localises a refusal to an arm without my copying the peer's logic.
# PRE-REGISTERED OUTCOMES, written BEFORE the run as the parent required (the cause may not be fitted to
# the data afterwards):
#   A. cross=true AND core_na=true  -> the AMBIENT text judgment refuses, not the two types = FOLD SCOPE
#      (pass the settled text judgment for each node's own module). Fix belongs in #14152.
#   B. cross=false and the tree differs below Resolved{node}, or between raw children, or in declaration
#      identity -> X1 PROPER: a descent/declaration defect in the relation. Its own PR.
#   C. cross=false and the trees are identical in every printed field, yet the relation refuses
#      -> REPORT THE UNEXPLAINED PAIR BY IDENTITY. Pick no cause, invent no predicate.
# Same harness, same pins, same carrier as probe v1 (which produced 208 refusals -- exactly the config-A
# census, so the probe is an observer, not a perturbation). Only the patcher, the marker and the gates below
# differ from probe_bucketb_14152.sh, and v1 is the proven template.
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
D=/tmp/probe-bucketb3; rm -rf $D; mkdir -p $D; LOGS=$D/logs; mkdir -p $LOGS
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
# THE CARRIER SHA IS READ FROM GIT, NEVER TYPED. A mistyped tail here (I typed 7f70285ce498c0e2b...,
# the real object is 7f70285ce498c6c08...) cost a dispatch that failed at the fetch, and the pre-push
# hook had already printed the correct value. Deriving it, and asserting every carrier path exists
# BEFORE the 20-minute regen, turns a 20-minute loss into a 3-second one.
CAR_SHA=$(git ls-remote origin refs/heads/scratch/rebind-census | awk '{print $1}')
[ -n "$CAR_SHA" ] || fail 28 "cannot resolve scratch/rebind-census"
echo "carrier sha (from ls-remote)=$CAR_SHA"
timeout 900 git fetch --depth 1 origin "$CAR_SHA" > $LOGS/fc.log 2>&1 || fail 28 "carrier fetch failed"
CAR=$(git rev-parse FETCH_HEAD)
[ "$CAR" = "$CAR_SHA" ] || fail 28 "FETCH_HEAD $CAR != $CAR_SHA"
for p in carriers/my_04_infer.dag carriers/my_04_types_backup.dag carriers/emitted_fold_v1_compiler_infer.rs patch_rebind_probe3.py; do
  git cat-file -e $CAR:$p 2>/dev/null || fail 28 "carrier $CAR lacks $p"
done
echo "carrier $CAR carries all 4 required paths"
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
# PATCHER SELFTEST, BEFORE THE 20-MINUTE REGEN. The patcher is only ever invoked AFTER the regen that
# produces the seed, so any defect in the patcher itself -- argv handling, an f-string, a bad replace --
# is priced at a full dispatch. The first v2 run proved the price: it converged for 20 minutes and then
# died in __main__ on `sys.argv[2] if len(sys.argv) > 1` (guard must be > 2), an IndexError that no
# amount of remote time could have predicted but a 30ms local run catches. Cheap gate, expensive failure.
# The patcher is needed HERE, not only at the patch step after the regen: the whole point of these
# gates is to fail before the 20 minutes. (The first attempt at this gate failed with "can't open file
# .../patch_rebind_probe3.py" because the fetch that writes it lives further down the script.)
git show $CAR:patch_rebind_probe3.py > $LOGS/patch_rebind_probe3.py || fail 30 "no patcher v2 in carrier"
wc -c $LOGS/patch_rebind_probe3.py
git show $CAR:carriers/emitted_fold_v1_compiler_infer.rs > $LOGS/mirror_rehearsal.rs || fail 33 "no rehearsal mirror in carrier"
wc -c $LOGS/mirror_rehearsal.rs
python3 $LOGS/patch_rebind_probe3.py --selftest || fail 4 "PATCHER SELFTEST FAILED"
# AND rehearse against a REAL emitted mirror if one is at hand -- the selftest uses a synthetic seed, so
# it can prove the patcher runs but cannot prove the anchor exists in the real emission.
if [ -f $LOGS/mirror_rehearsal.rs ]; then
  cp $LOGS/mirror_rehearsal.rs $LOGS/rehearsal.rs
  python3 $LOGS/patch_rebind_probe3.py $LOGS/rehearsal.rs | tee -a $LOGS/rehearse.log
  grep -q "'ok': True" $LOGS/rehearse.log || fail 4 "anchor not found in the real emitted mirror"
fi

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
git show $CAR:patch_rebind_probe3.py > $LOGS/patch_rebind_probe3.py || fail 30 "no patcher v2"
wc -c $LOGS/patch_rebind_probe3.py
python3 $LOGS/patch_rebind_probe3.py $SEED || fail 5 "patcher v2 refused"
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
NP=$(grep -ac 'rebind_probe_print3(' $SEED || true)
echo "probe v3 call sites in seed=$NP (want 2: the fn def's recursive calls aside, the refusal-site call)"
[ "$NP" -ge 1 ] || fail 9 "probe call not present in seed"
timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh.log 2>&1
echo "self-host rc=$? probeLines=$(grep -ac '^REBINDV3|' $LOGS/sh.log) refusals=$(grep -ac 'bound as' $LOGS/sh.log)"
# VALIDITY GATE, pre-registered: probe v1 measured 208 refusals at this exact base/carrier, and the config-A
# census said 208/130/19. probeLines must equal refusals and refusals must be 208, or the probe perturbed
# the compiler and the arm attribution is void. Printing it is not enough: a mismatch must be loud.
PL=$(grep -ac '^REBINDV3|' $LOGS/sh.log); RF=$(grep -ac 'bound as' $LOGS/sh.log)
echo "VALIDITY: probeLines=$PL refusals=$RF (expect 208 refusals; probeLines == refusals)"
[ "$RF" = "208" ] || echo "!! refusals != 208 -- compare against config A before believing any arm count"
[ "$PL" = "$RF" ] || echo "!! probeLines != refusals: the probe did not fire at every refusal"
# Pre-registered outcome tally, printed HERE so the answer is in the dispatch log and not only in a tar.
echo "=== ARM TALLY (pre-registered outcomes) ==="
grep -a '^REBINDV3|' $LOGS/sh.log | sed -n 's/.*(|cross=[a-z]*|core_na=[a-z]*|lleafinown=[a-z]*).*/\1/p' | sort | uniq -c | sort -rn | head -12
echo "outcome A (cross=true core_na=true) = $(grep -a '^REBINDV3|' $LOGS/sh.log | grep -aFc '|cross=true|core_na=true' || true)"
echo "outcome B/C candidates (cross=false) = $(grep -a '^REBINDV3|' $LOGS/sh.log | grep -aFc '|cross=false' || true)"
echo "equals=false core=true (wrapper-only difference) = $(grep -a '^REBINDV3|' $LOGS/sh.log | grep -aFc '|lleafinown=true' || true)"
echo "textpair=true = $(grep -a '^REBINDV3|' $LOGS/sh.log | grep -aFc '|lce=0' || true)  headapp=true = $(grep -a '^REBINDV3|' $LOGS/sh.log | grep -aFc '|lcaps=0' || true)"
cp $SEED $LOGS/seed_patched.rs
echo "=== BEGIN-TAR (log first) ==="; ( cd $LOGS && tar -c sh.log seed_patched.rs 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
cp $LOGS/seed_pristine.rs $SEED
echo PROBE_DONE
