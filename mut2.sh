cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
M=src/v1/stage0/src/v1_compiler_infer_types.rs
W=dag/test/claim/optional_leaf_instance_join_witness.dag
sed -i 's/^    if (instance.return_cardinality.clone() != Cardinality::Required) {$/    if true {/' $M
git diff --stat -- $M
sed -i 's/^import std.live_tree.*/&\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }/' $W
cat >> $W <<'X'

fn qe_main() -> ProcessExit {
  if an_optional_leaf_joins_the_optional_instance_at_the_same_payload_and_only_that() { ExitSuccess } else { ExitFailure { code: 3, reason: "DID-NOT-HOLD: optional leaf joins Optional<P> at the same payload" } }
}
X
/opt/cargo/bin/cargo build --release --bin gunbc 2>&1 | tail -1
target/release/gunbc run --source-root dag --source-root src/v2 --entry $W --function qe_main > /tmp/m2.out 2>&1; echo "rc_mut=$?"
grep -a 'DID-NOT-HOLD' /tmp/m2.out | head -3
