#!/bin/bash
# REGEN the 4 stage0 mirrors for this head so the RELATION (src/v1/04_infer.dag
# generic_types_unify + the (c) guard) is in the shipped seed, then re-run the whole
# cross_coproduct suite at the regenerated seed. Route proven by arm E (main mirrors +
# merged .dag -> regen rc=0). Reports EVERY tracked file regen moved instead of committing
# them (parent msg_06639145). NO git checkout / git clean on the runner.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
echo "HEAD=$(git rev-parse --short HEAD) dagRelation=$(grep -c 'generic_types_unify' src/v1/04_infer.dag) dagFormalGuard=$(grep -c 'binding_is_unsettled_formal_annotation' src/v1/04_infer.dag)"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "PRE: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12):U$(grep -c UnifyGenericsAcc $f)"; done)"
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2; fi
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen_rel.log 2>&1
rc=$?
echo "REGEN rc=$rc"
grep -E "first_generation_equal|refused|hard diagnostic" /tmp/regen_rel.log | head -5
echo "===== EVERY tracked file regen moved (report only) ====="
git status --porcelain | grep -v '^??'
echo "POST: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12):U$(grep -c UnifyGenericsAcc $f)"; done)"
echo "relation in seed now: generic_types_unify=$(grep -c generic_types_unify src/v1/stage0/src/v1_compiler_infer.rs) formal_guard=$(grep -c binding_is_unsettled_formal_annotation src/v1/stage0/src/v1_compiler_infer.rs)"
if [ "$rc" = "0" ]; then
  echo "########## SUITE at the REGENERATED seed ##########"
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error" | head -30
fi
echo "===== return mirrors base64 (so the commit is the regen product, not a hand pick) ====="
mkdir -p mirrors_rel && cp $MIRROR_FILES mirrors_rel/ 2>/dev/null
echo "=== BEGIN-TAR ==="; tar -c mirrors_rel 2>/dev/null | base64 -w0; echo "=== END-TAR ==="
restore_mine
echo "restored: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12)"; done) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo REGEN_REL_DONE
