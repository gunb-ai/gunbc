#!/bin/bash
# REGEN the 4 stage0 mirrors at this head (relation + (c) guard + the approved
# TypeVariable-leaf predicate), then measure at the REGENERATED seed in the SAME dispatch:
#   (1) the whole cross_coproduct suite (the accept/refuse split on the exact head), and
#   (2) the self-host rebind count -- the 0-bound-as evidence parent requires.
# Regen writes the mirrors itself; every tracked file it moves is REPORTED, not committed
# blindly (parent msg_06639145). NO git checkout / git clean on the runner.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "HEAD=$(git rev-parse --short HEAD) dag: relation=$(grep -c generic_types_unify src/v1/04_infer.dag) formalGuard=$(grep -c binding_is_unsettled_formal_annotation src/v1/04_infer.dag) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/04_infer.dag)"
echo "PRE: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12):U$(grep -c UnifyGenericsAcc $f)"; done)"
echo "########## (0) VACUITY CHECK on the two new controls, SHIPPED seed (pre-regen) ##########"
echo "If the ACCEPT control passes here it is vacuous on this seed (the predicate is not in the"
echo "shipped binary yet), which is exactly what we want to see stated; the REJECT twin must pass."
cargo test -p v1-compiler-tests --lib binding_still_carrying_a_type_variable 2>&1 | grep -E "^test |test result|^error|panicked" | head -6
cargo test -p v1-compiler-tests --lib settled_map_argument_pair 2>&1 | grep -E "^test |test result|^error|panicked" | head -6
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2; fi
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen2.log 2>&1
rc=$?
echo "REGEN rc=$rc"; grep -E "first_generation_equal|refused|hard diagnostic" /tmp/regen2.log | head -4
echo "===== EVERY tracked file regen moved (reported) ====="
git status --porcelain | grep -v '^??'
echo "POST: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12):U$(grep -c UnifyGenericsAcc $f)"; done)"
echo "seed now carries: relation=$(grep -c generic_types_unify src/v1/stage0/src/v1_compiler_infer.rs) formalGuard=$(grep -c binding_is_unsettled_formal_annotation src/v1/stage0/src/v1_compiler_infer.rs) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/stage0/src/v1_compiler_infer.rs)"
if [ "$rc" != "0" ]; then
  echo "REGEN REFUSED -- showing first diagnostics:"; grep -E "error|refus" /tmp/regen2.log | head -12
else
  echo "########## (1) SUITE at the REGENERATED seed ##########"
  cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error" | head -34
  echo "########## (2) SELF-HOST rebind count at the regenerated head ##########"
  timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_regen.log 2>&1
  echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/sh_regen.log) sametext=$(grep -ac "expected 'type variable" /tmp/sh_regen.log)"
  cp /tmp/sh_regen.log sh_regen.log
fi
echo "===== return the regen product as tar+base64 ====="
mkdir -p mirrors_rel && cp $MIRROR_FILES mirrors_rel/ 2>/dev/null
echo "=== BEGIN-TAR ==="; tar -c mirrors_rel 2>/dev/null | base64 -w0; echo "=== END-TAR ==="
restore_mine
echo "restored: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12)"; done) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo REGEN2_DONE
