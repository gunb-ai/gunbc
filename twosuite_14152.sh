#!/bin/bash
# THE TWO NEW CONTROLS, measured in BOTH configurations, suite-only (no self-host, so it fits the
# 45-minute cap). WHY A SECOND RUN: the previous dispatch printed NOTHING for its suite block. The
# cause is my own filter -- I grepped '^test |test result' and a compile error prints neither, so a
# build failure looked like an empty pass. Same class as the 'Product(Unit)' and 'PROD-UNIT' filters
# that reported clean zeros. This script's test filter includes '^error' and the cargo error lines,
# and prints the raw tail of the test build if no test line appears at all.
#
# CONFIG A (baseline): mirrors = keep/converged3 (my relation + my two fixes, NO peer arm).
# CONFIG B (arm): 04_types.dag = keep/peer_arm_04_types.dag (main's file + commit 92e2b987796's two
#   hunks, hand-applied verbatim because the patch does not apply to my base) regenerated through
#   the emitter, so the arm is in the COMPILED SEED and is verified there by its emitted signature.
# Both run the SAME test file, which now carries:
#   fold_refuses_disj_and_applied_spellings_of_one_name_with_different_arguments  (Box<Int> vs Disj Box<Bool>)
#   fold_accepts_disj_and_applied_spellings_of_one_name_with_the_same_arguments   (Box<Int> vs Disj Box<Int>)
# The pair is the answer to PR #14256's review question 'any pair the arm admits whose arguments
# DIFFER': the census cannot see it, because node_type_shape prints a Disj node's NAME ONLY
# (src/v1/04_types.dag:948) and its children never, so 'Coproduct(Box)' and 'Coproduct(Box<Bool>)'
# are the same string in every log.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
suite() {
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 \
    | grep -E "^test |test result|^error|error\[|cannot find|expected|mismatched types" | head -40
}
cp keep/my_04_infer.dag src/v1/04_infer.dag
cp keep/my_04_types_backup.dag src/v1/04_types.dag

echo "########## CONFIG A: baseline, converged3 mirrors, no peer arm ##########"
for f in $MIRRORS; do cp "keep/converged3/$f" "$SEED/$f"; done
echo "A seed: arm-signature=$(grep -c 'right_struct' $SEED/v1_compiler_infer_types.rs) (want 0) myRelation=$(grep -c generic_head_matches_application $SEED/v1_compiler_infer.rs) fold=$(grep -o UnifyGenericsAcc $SEED/v1_compiler_infer.rs | wc -l)"
cargo build --release -p v1-compiler-tests --tests 2>&1 | grep -E "^error|error\[|Finished" | tail -4
suite

echo "########## CONFIG B: apply PR #14256's arm, regenerate, then the same suite ##########"
cp keep/peer_arm_04_types.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
echo "B dag: peerLet=$(grep -c 'let left_applied = left.connective == NoConnective && !left_leaf' src/v1/04_types.dag) peerArm=$(grep -c 'left_applied && (right_leaf || right_struct)' src/v1/04_types.dag) (want 1,1)"
i=1
while [ $i -le 3 ]; do
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/pb_$i.log 2>&1
  echo "=== B round $i rc=$? diag=$(grep -acE '^type mismatch|^call shape|arm decides' /tmp/pb_$i.log)"
  grep -aE "^type mismatch|^call shape|arm decides" /tmp/pb_$i.log | head -4 | cut -c1-190
  [ -d "$CAND" ] || break
  inst=0
  for f in $(ls "$CAND" 2>/dev/null); do
    [ -f "$SEED/$f" ] || continue
    cmp -s "$CAND/$f" "$SEED/$f" || { cp "$CAND/$f" "$SEED/$f"; inst=$((inst+1)); echo "  installed $f"; }
  done
  echo "B round $i installed=$inst"
  [ "$inst" -eq 0 ] && break
  i=$((i+1))
done
echo "B seed: arm-signature=$(grep -c 'right_struct' $SEED/v1_compiler_infer_types.rs) (want >=1) myRelation=$(grep -c generic_head_matches_application $SEED/v1_compiler_infer.rs) fold=$(grep -o UnifyGenericsAcc $SEED/v1_compiler_infer.rs | wc -l)"
cargo build --release -p v1-compiler-tests --tests 2>&1 | grep -E "^error|error\[|Finished" | tail -4
suite

echo "########## SELF-HOST at CONFIG B (the peer-arm census denominator) ##########"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
timeout 2400 target/release/gunbc test //gunbc/instruments:self-host > /tmp/pb_sh.log 2>&1
echo "self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/pb_sh.log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' /tmp/pb_sh.log | sort -u | wc -l)"
cp /tmp/pb_sh.log selfhost_B.log
echo "=== BEGIN-TAR-LOGS ==="; ( cd . && tar -cz selfhost_B.log 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR-LOGS ==="
cp keep/my_04_types_backup.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: 04_types=$(sha256sum src/v1/04_types.dag|cut -c1-12) peerMarkers=$(grep -c 'let left_applied = left.connective == NoConnective && !left_leaf' src/v1/04_types.dag) (want 0)"
echo TWOSUITE_DONE
