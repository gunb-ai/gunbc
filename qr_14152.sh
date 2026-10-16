#!/bin/bash
# PARENT RULING (msg_0e84a10b / msg_fd2b5b33 / msg_812952da): before changing either
# side, establish with execution whether the two same-text FreeMonoid diagnostics at
# src/v1/02_parse.dag byte-spans 83460/84172 (lines 1960/1979) ARE the rebind refusal.
#
# CONTROL (the decisive one): run the regen at the MERGED tree twice -- once with the
# fold-carrying seed (this tree's state) and once with the PRE-FOLD seed carried in
# seedred/. The fold's refusal code does not exist in the pre-fold interpreter, so if
# the 2 diagnostics persist under BOTH, they are NOT the fold (corpus/judgment
# issue); if they vanish under the pre-fold seed, they ARE the fold's doing.
# Also count "bound as" (the fold's own message text) in each run.
#
# NEVER `git checkout` on the runner (its HEAD is not the PR head; it rewrote seed to
# base content once). Seed variants are plain files; restore with cp from seedgreen/.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
sig() { grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs; }
echo "QR: mirror-base=$(git rev-parse HEAD) seed U=$(sig)"
[ -d seedgreen ] && [ -d seedred ] || { echo "QR: seedred/seedgreen missing"; exit 2; }
if [ ! -x target/release/claim_executor ]; then
  cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2
fi
run() {
  side="$1"
  for f in $FILES; do cp "$side/$(basename $f)" "$f"; done
  echo "QR [$side]: seed U=$(sig) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
  cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -1
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/qr_$side.log 2>&1
  rc=$?
  echo "QR [$side]: regen rc=$rc  bound-as=$(grep -c 'bound as' /tmp/qr_$side.log)  freemonoid=$(grep -c 'Container(FreeMonoid,Primitive(element))' /tmp/qr_$side.log)"
  grep -E "refused|type mismatch" /tmp/qr_$side.log | head -6 | cut -c1-200
}
run seedgreen
run seedred
# restore this tree's own (green) seed
for f in $FILES; do cp "seedgreen/$(basename $f)" "$f"; done
echo "QR [restored]: seed U=$(sig) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs | cut -c1-16)"
echo "QR_DONE"
