set -u
run(){ SHA=$1
git fetch -q origin session/calm-eagle-763 --deepen=400 2>&1|tail -1
git checkout -q -f --detach $SHA && git clean -fdq
[ "$(git rev-parse HEAD)" = "$SHA" ] || { echo "HEAD MISMATCH"; exit 3; }
echo "##### HEAD=$(git rev-parse HEAD)"
cargo build --release -q --manifest-path src/v1/stage0/Cargo.toml --bin claim_batch --target-dir target 2>&1 | tail -5
for w in rung_drop_superseded_standing_witness rung_drop_declaration_witness rung_drop_standing_partition_witness rung_drop_shared_capability_witness rung_drop_amendment_witness whole_generated_artifact_adjudication primitive_runtime_body_witness namespace_cut_subject_observation_witness; do
  f=dag/test/claim/$w.dag
  fns=$(grep -oE '^test fn [A-Za-z0-9_]+' $f | awk '{print $3}' | paste -sd,)
  echo "=== $w"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions "$fns" 2>&1 | grep -aE '^(PASS|FAIL)|rror|refus' | cut -c1-200
done; }
run $1; run $2
