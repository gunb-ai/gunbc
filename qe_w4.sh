cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
W=dag/test/claim/optional_leaf_instance_join_witness.dag
cp $W /tmp/w_orig.dag
sed -i 's/^import std.live_tree.*/&\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }/' $W
cat >> $W <<'X'

fn qe_main() -> ProcessExit {
  let a = an_optional_leaf_joins_the_optional_instance_at_the_same_payload_and_only_that()
  let b = only_the_optionals_own_nullary_arm_adopts_an_optional_expectation()
  ExitFailure { code: (if a { 1 } else { 0 }) * 10 + (if b { 1 } else { 0 }) + 1, reason: "tens=join ones=wrap, plus one" }
}
X
target/release/gunbc run --source-root dag --source-root src/v2 --entry $W --function qe_main > /tmp/qe_w4.out 2>&1; echo "rc=$?"
tail -4 /tmp/qe_w4.out | cut -c1-300
cp /tmp/w_orig.dag $W
