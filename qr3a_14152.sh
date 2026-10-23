#!/bin/bash
# QR3a (parent msg_0e84a10b: print the FULL messages and counts per arm; msg_e0f346d2
# decision rule needs the arm-F byte-identity). NO `git checkout` and NO `git clean`
# on the runner (its HEAD is the main tip; clean once wiped the untracked seed carriers
# and this script mid-run -- lesson recorded). Every arm restores via cp from the
# seedgreen/ seed carriers; ctrl-build re-patches the tracked tree per dispatch.
# Arms:
#  A merged .dag + fold seed (this tree)          -> expect rc=1, sametext=2, bound-as=0
#  C merged .dag + PRE-FOLD seed (seedred, U=0)   -> expect the same pair (fold absent!)
#  F MAIN's current mirrors as seed + merged .dag -> the heal route; then sha-compare the
#    emitted mirrors against seedgreen/ (byte identity = my mirror conflict dissolves).
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
sig() { grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs; }
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
echo "QR3a: base=$(git rev-parse HEAD) seedU=$(sig) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) (want 5e9c37585fe088ac)"
[ -d seedgreen ] && [ -d seedred ] || { echo "QR3a: seed carriers missing"; exit 2; }
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -1; fi
dump() {
  label="$1"
  echo "########## QR3a [$label] seedU=$(sig) dagfoldU=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag) ##########"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/qr3a_$label.log 2>&1
  echo "rc=$? bound-as=$(grep -c 'bound as' /tmp/qr3a_$label.log) sametext=$(grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/qr3a_$label.log)"
  echo "--- verbatim same-text FreeMonoid diagnostics (max 2):"
  grep -m2 "Container(FreeMonoid,Primitive(element))', got 'Container" /tmp/qr3a_$label.log | cut -c1-235
  echo "--- verbatim fold diagnostics (max 4), if any:"
  grep -m4 "bound as" /tmp/qr3a_$label.log | cut -c1-235
}
restore_mine
dump A_merged_foldSeed
for f in $MIRROR_FILES; do cp "seedred/$(basename $f)" "$f"; done
dump C_merged_preFoldSeed
restore_mine
echo "########## QR3a [F heal route] main's committed mirrors as seed, merged .dag ##########"
git fetch -q origin main 2>/dev/null || true
for f in $MIRROR_FILES; do git show origin/main:$f > "$f"; done
echo "QR3a F: seeded-from-main seedU=$(sig)"
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/qr3a_F.log 2>&1
echo "rc=$? bound-as=$(grep -c 'bound as' /tmp/qr3a_F.log) sametext=$(grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/qr3a_F.log) $(grep -oE "first_generation_equal=[a-z]+" /tmp/qr3a_F.log | head -1)"
echo "QR3a F: emitted mirrors vs MY committed mirrors (sha equality = byte identity):"
for f in $MIRROR_FILES; do
  a=$(sha256sum "$f" | cut -c1-16); b=$(sha256sum "seedgreen/$(basename $f)" | cut -c1-16)
  m=$(grep -c UnifyGenericsAcc "$f")
  [ "$a" = "$b" ] && v=IDENTICAL || v=DIFFERS
  echo "QR3a F: $v emitted=$a mine=$b U=$m $(basename $f)"
done
restore_mine
echo "QR3a [restored] seedU=$(sig) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo "QR3a_DONE"
