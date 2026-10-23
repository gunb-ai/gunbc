#!/bin/bash
# Two questions in one dispatch, both on the SHIPPED seed (no mirror patching):
#  [1] DIRECT-CALL CONTROLS: are the four new fold-grain controls sound? Expect the two
#      ACCEPT rows RED (the predicate is in src/v1/04_infer.dag but not yet in the seed)
#      and the two REJECT twins GREEN. GREEN-when-it-should-be-RED, or RED-when-it-
#      should-be-GREEN, is an unsound control, not a finding about the fold.
#  [2] WHY REGEN REFUSED at this head (rc=1): list the hard diagnostics with file:span and
#      message, to tell the KNOWN pre-existing same-text pair at src/v1/02_parse.dag
#      (X2's boundary: v1.compiler.infer structured_application_site_type_mismatch_with_
#      peeled, a name-only checker emitting a pair its own shapes call equal) apart from
#      anything my predicate introduced.
# NO git checkout / git clean on the runner.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
MIRROR_FILES="src/v1/stage0/src/v1_compiler_infer.rs src/v1/stage0/src/v1_compiler_infer_types.rs src/v1/stage0/src/v1_compiler_emit_rust.rs src/v1/stage0/src/v1_tests_claim_generic_identity_census.rs"
restore_mine() { for f in $MIRROR_FILES; do cp "seedgreen/$(basename $f)" "$f"; done; }
restore_mine
echo "seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) sha=$(sha256sum src/v1/stage0/src/v1_compiler_infer.rs|cut -c1-16) (want 36 / 5e9c37585fe088ac)"
echo "HEAD=$(git rev-parse --short HEAD) dag: relation=$(grep -c generic_types_unify src/v1/04_infer.dag) formalGuard=$(grep -c binding_is_unsettled_formal_annotation src/v1/04_infer.dag) tvLeaf=$(grep -c binding_has_type_variable_leaf src/v1/04_infer.dag)"
echo "########## [1] DIRECT-CALL CONTROLS on the shipped seed ##########"
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate_test::fold_ 2>&1 | grep -E "^test |test result|^error|panicked|assertion|must refuse|unsettled|cannot falsify|genuine misbinding" | head -30
echo "########## [2] REGEN round diagnostics at this head ##########"
if [ ! -x target/release/claim_executor ]; then cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2; fi
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/r2.log 2>&1
echo "rc=$?"
echo "--- tail of the regen output:"; tail -20 /tmp/r2.log | cut -c1-200
echo "--- diagnostic / refusal lines:"; grep -iE "hard diagnostic|diagnostic|refus|first_generation_equal" /tmp/r2.log | tail -8 | cut -c1-200
echo "--- 02_parse same-text pair count:"; grep -c "expected 'Container(FreeMonoid,Primitive(element))', got 'Container(FreeMonoid,Primitive(element))'" /tmp/r2.log
echo "--- rebind (bound as) refusals in the regen round:"; grep -c "bound as" /tmp/r2.log
restore_mine
echo "restored seedU=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs) tracked-diff=$(git status --porcelain | grep -vc '^??')"
echo DC_DONE
