cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
D=src/v1/stage0/src
W=dag/test/claim/optional_leaf_instance_join_witness.dag
cp $W /tmp/w_orig.dag
sed -i 's/^import std.live_tree.*/&\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }/' $W
cat >> $W <<'X'

fn qe_main() -> ProcessExit {
  if an_optional_leaf_joins_the_optional_instance_at_the_same_payload_and_only_that() { ExitSuccess } else { ExitFailure { code: 3, reason: "witness false" } }
}
X
for which in head base; do
  if [ $which = base ]; then git checkout fb6f04c403 -- $D; fi
  /opt/cargo/bin/cargo build --release --bin gunbc 2>&1 | tail -1
  target/release/gunbc run --source-root dag --source-root src/v2 --entry $W --function qe_main > /tmp/qe_w_$which.out 2>&1; echo "rc_$which=$?"
  tail -4 /tmp/qe_w_$which.out | cut -c1-300
done
