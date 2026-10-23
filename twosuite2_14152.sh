#!/bin/bash
# THE TWO Box CONTROLS, in BOTH configurations. Why it is cheap: a COMPILE GATE runs first and
# aborts before any regen. Three earlier dispatches printed NOTHING for their suite block; the cause
# was my own fold_disj returning Node where the helper returns Rc<Node> (E0308), and my test filter
# ('^test |test result') could not print a compile error, so a broken build looked like an empty
# pass. A filter that cannot show failure is not an instrument. Here: cargo check runs alone first,
# its RAW tail is printed unfiltered, and failure exits 4 before the 20-minute regen.
#
# CONFIG A baseline: keep/converged3 mirrors -- my relation + my two fixes, NO PR #14256 arm.
# CONFIG B arm: 04_types.dag = keep/peer_arm_04_types.dag (main's file + commit 92e2b987796's two
#   hunks, hand-applied verbatim; the patch does not apply to my base), regenerated into the seed.
# MARKERS ARE DISCRIMINATING ON PURPOSE. 'right_struct' ALONE IS NOT A PEER MARKER: main already
# uses right_struct in its own arms, and an earlier run of mine claimed 'arm in seed = 4' from that
# loose grep and was wrong. The peer arm's emitted signature is the conjunction itself,
# 'left_applied && (right_leaf || right_struct)'; main's own neighbouring let left_applied
# (src/v1/04_types.dag:1274) decides applied by (left.children |> count) > 0, so it cannot produce
# that text. A=0, B>=1, and a mismatch stops the run.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
CAND=target/stage0-regen-candidate/src
ARMMARK='left_applied && (right_leaf || right_struct)'
cp keep/my_04_infer.dag src/v1/04_infer.dag
cp keep/my_04_types_backup.dag src/v1/04_types.dag

echo "########## GATE: does the test crate compile at all? (raw tail, unfiltered) ##########"
for f in $MIRRORS; do cp "keep/converged3/$f" "$SEED/$f"; done
# PIPESTATUS, not $?: 'cargo check | tail' leaves $? as TAIL's status, which is always 0, so a gate
# written that way passes on a broken crate and spends the regen round anyway -- the exact failure
# that made three earlier dispatches report an empty suite.
cargo check -p v1-compiler-tests --tests > /tmp/gate.log 2>&1
gate=$?
tail -25 /tmp/gate.log
echo "check rc=$gate"
if [ "$gate" != "0" ]; then echo "TEST CRATE DOES NOT COMPILE -- refusing to spend a regen round on it"; exit 4; fi

suite() {
  cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 \
    | grep -E "^test |test result|^error|error\[|panicked|assertion|left:|right:" | head -44
}

echo "########## CONFIG A: baseline (converged3, no PR #14256 arm) ##########"
echo "A seed: armMark=$(grep -c "$ARMMARK" $SEED/v1_compiler_infer_types.rs) (want 0) myRelation=$(grep -c generic_head_matches_application $SEED/v1_compiler_infer.rs) fold=$(grep -o UnifyGenericsAcc $SEED/v1_compiler_infer.rs | wc -l)"
cargo build --release -p v1-compiler-tests --tests 2>&1 | grep -E "^error|error\[|Finished" | tail -3
suite
echo "--- the two Box controls by name, config A:"
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -aE "disj_and_applied" | head -4

echo "########## CONFIG B: PR #14256's arm regenerated into the seed ##########"
cp keep/peer_arm_04_types.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
echo "B dag: peerLet=$(grep -c 'let left_applied = left.connective == NoConnective && !left_leaf' src/v1/04_types.dag) peerArm=$(grep -c 'left_applied && (right_leaf || right_struct)' src/v1/04_types.dag) (want 1,1)"
i=1
while [ $i -le 3 ]; do
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/tb_$i.log 2>&1
  echo "=== B round $i rc=$? diag=$(grep -acE '^type mismatch|^call shape|arm decides' /tmp/tb_$i.log)"
  grep -aE "^type mismatch|^call shape|arm decides" /tmp/tb_$i.log | head -4 | cut -c1-190
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
am=$(grep -c "$ARMMARK" $SEED/v1_compiler_infer_types.rs)
echo "B seed: armMark=$am (want >=1) myRelation=$(grep -c generic_head_matches_application $SEED/v1_compiler_infer.rs) fold=$(grep -o UnifyGenericsAcc $SEED/v1_compiler_infer.rs | wc -l)"
if [ "$am" -lt 1 ]; then echo "ARM DID NOT REACH THE SEED -- the config B result would be meaningless"; exit 5; fi
cargo build --release -p v1-compiler-tests --tests 2>&1 | grep -E "^error|error\[|Finished" | tail -3
suite
echo "--- the two Box controls by name, config B:"
cargo test -p v1-compiler-tests --lib cross_coproduct_generic_locate 2>&1 | grep -aE "disj_and_applied" | head -4

echo "########## SELF-HOST at config B, with its denominator ##########"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | grep -E "^error|error\[|Finished" | tail -2
timeout 2100 target/release/gunbc test //gunbc/instruments:self-host > /tmp/tb_sh.log 2>&1
echo "self-host rc=$? rebind=$(grep -ac 'bound as' /tmp/tb_sh.log) files=$(grep -aoE 'src/[a-z0-9/_.]+\.dag' /tmp/tb_sh.log | sort -u | wc -l)"
cp /tmp/tb_sh.log selfhost_B2.log
echo "=== BEGIN-TAR-LOG ==="; ( cd . && tar -cz selfhost_B2.log 2>/dev/null | base64 -w0 ); echo; echo "=== END-TAR-LOG ==="
cp keep/my_04_types_backup.dag src/v1/04_types.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
cp keep/my_04_infer.dag src/v1/04_infer.dag
echo "restored: peerMarkers=$(grep -c 'let left_applied = left.connective == NoConnective && !left_leaf' src/v1/04_types.dag) (want 0) myRelation=$(grep -c generic_head_matches_application src/v1/04_infer.dag) (want 1)"
echo TWOSUITE2_DONE
