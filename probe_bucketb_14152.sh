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
D=/tmp/probe-bucketb; rm -rf $D; mkdir -p $D; LOGS=$D/logs; mkdir -p $LOGS
cd $D
timeout 900 git clone --depth 1 $URL repo > $LOGS/clone.log 2>&1 || { echo "CLONE FAILED"; tail -5 $LOGS/clone.log; exit 20; }
cd repo
timeout 600 git fetch --depth 1 origin $BASE_SHA > $LOGS/fb.log 2>&1 || { echo "BASE FETCH FAILED"; exit 25; }
git checkout -q -f FETCH_HEAD
[ "$(git rev-parse HEAD)" = "$BASE_SHA" ] || { echo "WRONG TREE"; exit 27; }
echo "tree=$BASE_SHA status=$(git status --porcelain | wc -l) (want 0)"

# ---- source = my fixed fold (carrier) so the fold's refusal exists at all ----
timeout 900 git fetch --depth 1 origin bd68035135c62c17dd5be2777ff9770955b58f50 > $LOGS/fc.log 2>&1 || exit 28
CAR=$(git rev-parse FETCH_HEAD)
git show $CAR:carriers/my_04_infer.dag > src/v1/04_infer.dag || exit 29
git show $CAR:carriers/my_04_types.dag  > src/v1/04_types.dag || exit 29
echo "carrier markers: fold=$(grep -ac UnifyGenericsAcc src/v1/04_infer.dag) fixed=$(grep -ac 'resolved.value' src/v1/04_infer.dag)"
# main's own mirrors: they match this source only after regen, so regen once to a fixed point first.
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
r=1
while [ $r -le 3 ]; do
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > $LOGS/rr_$r.log 2>&1
  echo "regen round $r rc=$?"
  if [ ! -d target/stage0-regen-candidate/src ]; then echo "NO CANDIDATE -- receipt:"; grep -a "^required-regen\|^type mismatch\|not found in\|no field" $LOGS/rr_${r}.log | head -6 | cut -c1-240; exit 7; fi
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
git show $CAR:patch_rebind_probe.py > $LOGS/patch_rebind_probe.py || exit 30
wc -c $LOGS/patch_rebind_probe.py
python3 $LOGS/patch_rebind_probe.py $SEED || exit 5
echo "probe refs in seed=$(grep -ac 'rebind_probe_print' $SEED) (want 2: the fn and the call)"
# rustfmt-parse the patched seed LOCALLY (the rule: 4 probes died on bad injections)
rustfmt --edition 2021 --emit stdout $SEED > /dev/null 2> $LOGS/rustfmt.err; f=$?
echo "rustfmt rc=$f"; head -8 $LOGS/rustfmt.err
[ "$f" != "0" ] && { echo "PATCH DOES NOT PARSE -- aborting before dispatch"; exit 6; }

cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -3
timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > $LOGS/sh.log 2>&1
echo "self-host rc=$? probeLines=$(grep -ac '^REBINDPROBE|' $LOGS/sh.log) refusals=$(grep -ac 'bound as' $LOGS/sh.log)"
cp $SEED $LOGS/seed_patched.rs
echo "=== BEGIN-TAR (log first) ==="; ( cd $LOGS && tar -c sh.log seed_patched.rs 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR ==="
cp $LOGS/seed_pristine.rs $SEED
echo PROBE_DONE
