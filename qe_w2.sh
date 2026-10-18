cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
D=src/v1/stage0/src
W=dag/test/claim/optional_leaf_instance_join_witness.dag
cp $W /tmp/w_orig.dag
sed -i 's/^import std.live_tree.*/&\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }/' $W
cat >> $W <<'X'

fn qe_main() -> ProcessExit {
  if an_optional_leaf_joins_the_optional_instance_at_the_same_payload_and_only_that() && only_the_optionals_own_nullary_arm_adopts_an_optional_expectation() { ExitSuccess } else { ExitFailure { code: 3, reason: "witness false" } }
}
X
/opt/cargo/bin/cargo build --release --bin gunbc 2>&1 | tail -1
target/release/gunbc run --source-root dag --source-root src/v2 --entry $W --function qe_main > /tmp/qe_w2_head.out 2>&1; echo "rc_head=$?"
tail -3 /tmp/qe_w2_head.out | cut -c1-300
cp /tmp/w_orig.dag $W
