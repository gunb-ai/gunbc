#!/bin/bash
# QR3/QR4 (parent ruling msg_0e84a10b: "Establish first: are those two diagnostics the rebind
# refusal itself (print the full message and cause)? ... Report the answer to me before
# changing either side.") The control matrix already says the 02_parse pair is NOT the
# fold's own message text (bound-as=0 in every arm, incl. with a fold-less seed). This
# dispatch prints the FULL messages + counts per arm, so nothing is inferred from shape:
#   arm B: main's .dag authorities + MY fold seed        -> 844 diagnostics (fold judging main's corpus)
#   arm C: merged .dag + PRE-FOLD seed (seedred, U=0)    -> the 2 FreeMonoid pair
#   arm A: merged .dag + fold seed (this tree)           -> the 2 FreeMonoid pair
# CLEAN FIRST: arm E of the last run left a regenerated seed on the runner with no tracked
# diff; discard tracked modifications, then re-apply my committed mirrors from seedgreen/.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
sig() { grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs; }
echo "QR3: base=$(git rev-parse HEAD)"
git checkout -- . 2>/dev/null; git clean -q -x -d --force -- . 2>/dev/null
for f in $MIRROR_FILES; do cp seedgreen/$(basename $f) $f; done
echo "QR3: restored my mirrors seedU=$(sig) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16) (want 5e9c37585fe088ac)"
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -1; fi
mkdir -p /tmp/myauth
for f in src/v1/04_infer.dag src/v1/tests/claim/generic_identity_census.dag; do mkdir -p /tmp/myauth/$(dirname $f); cp $f /tmp/myauth/$f; done
dump() {
  label="$1"
  echo "########## QR3 [$label] seedU=$(sig) dagfoldU=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag) ##########"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/qr3_$label.log 2>&1
  echo "rc=$? bound-as=$(grep -c 'bound as' /tmp/qr3_$label.log) sametext=$(grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/qr3_$label.log)"
  echo "--- full text of the two same-text FreeMonoid diagnostics:"
  grep -m2 "Container(FreeMonoid,Primitive(element))', got 'Container" /tmp/qr3_$label.log | cut -c1-230
  echo "--- first 6 fold-message diagnostics (full text), if any:"
  grep -m6 "bound as" /tmp/qr3_$label.log | cut -c1-230
  echo "--- diagnostic class counts:"
  grep -oE "^[a-z][a-z ]+: " /tmp/qr3_$label.log | sort | uniq -c | sort -rn | head -6
}
dump mergedDag_foldSeed
for f in $MIRROR_FILES; do cp seedred/$(basename $f) $f; done
dump mergedDag_preFoldSeed
for f in $MIRROR_FILES; do cp seedgreen/$(basename $f) $f; done
for f in src/v1/04_infer.dag src/v1/tests/claim/generic_identity_census.dag; do git show origin/main:$f > $f; done
dump mainDag_foldSeed
echo "########## QR4 arm F: MAIN's current mirrors as the seed + merged .dag (the heal route) ##########"
for f in $MIRROR_FILES; do git show origin/main:$f > $f; done
echo "QR4: seeded-from-main seedU=$(sig)"
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/qr4_F.log 2>&1
echo "rc=$? bound-as=$(grep -c 'bound as' /tmp/qr4_F.log) sametext=$(grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/qr4_F.log) first_gen=$(grep -oE "first_generation_equal=[a-z]+" /tmp/qr4_F.log | head -1)"
echo "QR4: emitted mirrors vs MY committed mirrors (seedgreen/) -- sha equality is the byte-identity test:"
for f in $MIRROR_FILES; do
  a=$(sha256sum "$f" | cut -c1-16); b=$(sha256sum "seedgreen/$(basename $f)" | cut -c1-16)
  m=$(grep -c UnifyGenericsAcc "$f")
  [ "$a" = "$b" ] && v=IDENTICAL || v=DIFFERS
  echo "QR4: $v emitted=$a mine=$b U=$m $(basename $f)"
done
git checkout -- src/v1/stage0/ 2>/dev/null; for f in $MIRROR_FILES; do cp seedgreen/$(basename $f) $f; done
for f in src/v1/04_infer.dag src/v1/tests/claim/generic_identity_census.dag; do cp /tmp/myauth/$f $f; done
for f in $MIRROR_FILES; do cp seedgreen/$(basename $f) $f; done
echo "QR3 [restored] seedU=$(sig) dagfoldU=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo "QR3_DONE"
