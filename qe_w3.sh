cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
W=dag/test/claim/optional_leaf_instance_join_witness.dag
cp $W /tmp/w_orig.dag
sed -i 's/^import std.live_tree.*/&\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }/' $W
cat >> $W <<'X'

fn qe_main() -> ProcessExit {
  ExitFailure { code: total_blocking_count_for(source: optional_nullary_refuse_source) * 100 + total_blocking_count_for(source: optional_nullary_accept_source), reason: "nullary refuse*100+accept" }
}
X
target/release/gunbc run --source-root dag --source-root src/v2 --entry $W --function qe_main > /tmp/qe_w3.out 2>&1; echo "rc=$?"
tail -3 /tmp/qe_w3.out | cut -c1-300
cp /tmp/w_orig.dag $W
