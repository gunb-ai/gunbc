#!/bin/bash
# ONE dispatch, in order, at the fixed head:
#  [A] RED-BEFORE: the discriminating direct-call control on the SHIPPED seed. It must FAIL
#      now (the shipped fold overwrites a settled binding with a variable-bearing actual)
#      and pass after the regen carries the predicate. If it PASSES now, the predicate is
#      inert at fold grain and that is the finding.
#  [B] REGEN the 4 stage0 mirrors (route proven by arm E). Report every tracked file moved.
#  [C] SUITE at the regenerated seed + [D] SELF-HOST rebind count (the 0-bound-as evidence).
# NO git checkout / git clean on the runner.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "HEAD=$(git rev-parse --short HEAD) dag: relation=$(grep -c generic_types_unify src/v1/04_infer.dag) formalGuard=$(grep -c binding_is_unsettled_formal_annotation src/v1/04_infer.dag) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/04_infer.dag)"
echo "########## [A] RED-BEFORE on the shipped seed ##########"
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate_test::fold_ 2>&1 | grep -E "^test |test result|panicked|overwrites|assertion|left|right" | head -20
echo "########## [B] REGEN ##########"
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2; fi
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen3.log 2>&1
rc=$?
echo "REGEN rc=$rc"; grep -E "first_generation_equal|refused|hard diagnostic" /tmp/regen3.log | head -3
echo "--- hard diagnostics (all, verbatim):"; grep -E "^type mismatch|^call shape|^error|expected '" /tmp/regen3.log | head -20 | cut -c1-190
echo "--- files regen moved:"; git status --porcelain | grep -v '^??'
echo "POST: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12):U$(grep -c UnifyGenericsAcc $f)"; done)"
echo "seed carries: relation=$(grep -c generic_types_unify src/v1/stage0/src/v1_compiler_infer.rs) formalGuard=$(grep -c binding_is_unsettled_formal_annotation src/v1/stage0/src/v1_compiler_infer.rs) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/stage0/src/v1_compiler_infer.rs)"
if [ "$rc" = "0" ]; then
  echo "########## [C] SUITE at the regenerated seed ##########"
  cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -E "^test |test result|^error" | head -34
  echo "########## [D] SELF-HOST at the regenerated head ##########"
  timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh3.log 2>&1
  echo "rc=$? rebind=$(grep -ac 'bound as' /tmp/sh3.log)"
  cp /tmp/sh3.log sh_regen3.log
  mkdir -p mirrors_rel3 && cp $MIRROR_FILES mirrors_rel3/ 2>/dev/null
  echo "=== BEGIN-TAR ==="; tar -c mirrors_rel3 2>/dev/null | base64 -w0; echo "=== END-TAR ==="
fi
restore_mine
echo "restored: $(for f in $MIRROR_FILES; do echo "$(basename $f)=$(sha256sum $f|cut -c1-12)"; done) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo REGEN3_DONE
