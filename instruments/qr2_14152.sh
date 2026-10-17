#!/bin/bash
# QR2: the missing arm of the control matrix. At the merged tree:
#  arm A (control-of-record, already known): merged .dag + fold seed -> rc=1, 2 FreeMonoid
#  arm B (NEW, decisive): MAIN's .dag authorities + MY fold seed -> ?
#  arm C (already known via seedred): merged .dag + fold-less seed -> rc=1, 2 FreeMonoid
# B green  => the refusal needs MY .dag authorities (my change is now judging main's code).
# B refuses => the refusal lives in the corpus main already ships at this pairing; the
#              fold is not implicated at all, and the 02_parse pair is a pre-existing
#              judgment issue (its text has no 'bound as', so it is not the fold's).
# NEVER `git checkout` on the runner (its HEAD is main tip; it rewrote seed content to
# base once). main's authorities come from `git show origin/main:<path>` into files.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
sig() { grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs; }
echo "QR2: mirror-base=$(git rev-parse HEAD) seedU=$(sig) dagfoldU=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag)"
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -1; fi
# snapshot my authorities (the merged .dag files) so we can restore them
mkdir -p /tmp/myauth
for f in src/v1/04_infer.dag src/v1/tests/claim/generic_identity_census.dag; do mkdir -p /tmp/myauth/$(dirname $f); cp $f /tmp/myauth/$f; done
run() {
  label="$1"
  echo "QR2 [$label]: seedU=$(sig) dagfoldU=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag) census7arg=$(grep -c 'text:' src/v1/tests/claim/generic_identity_census.dag)"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/qr2_$label.log 2>&1
  rc=$?
  echo "QR2 [$label]: rc=$rc bound-as=$(grep -c 'bound as' /tmp/qr2_$label.log) freemonoid=$(grep -c 'Container(FreeMonoid,Primitive(element))' /tmp/qr2_$label.log)"
  grep -E "refused|first_generation_equal|type mismatch" /tmp/qr2_$label.log | head -4 | cut -c1-190
}
echo "########## arm B: main's .dag authorities + MY fold seed ##########"
for f in src/v1/04_infer.dag src/v1/tests/claim/generic_identity_census.dag; do git show origin/main:$f > $f; done
run mainDag_mySeed
echo "########## restore my authorities ##########"
for f in src/v1/04_infer.dag src/v1/tests/claim/generic_identity_census.dag; do cp /tmp/myauth/$f $f; done
run mergedDag_mySeed
echo "########## arm E (the heal arm): MAIN's 4 mirrors as the committed seed + MY merged .dag ##########"
git fetch -q origin main 2>/dev/null || true
for f in $MIRROR_FILES; do git show origin/main:$f > $f 2>/dev/null || echo "QR2: cannot read main mirror $f"; done
run mainSeed_mergedDag
echo "QR2 [mainSeed_mergedDag]: emitted-seed U=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) (a green regen WRITES the new seed: fold present => the heal route works)"
echo "QR2: tracked moves after arm E:"; git status --porcelain | grep -v '^??' | head -8
echo "QR2_DONE"
