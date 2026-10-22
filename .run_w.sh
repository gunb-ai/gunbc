set -u
SHA=$1
git fetch -q origin session/calm-eagle-763 --deepen=400 2>&1|tail -1
git checkout -q -f --detach $SHA && git clean -fdq -e run_t.sh
[ "$(git rev-parse HEAD)" = "$SHA" ] || { echo "HEAD MISMATCH"; exit 3; }
echo "HEAD=$(git rev-parse HEAD)"
cargo build --release -q --manifest-path src/v1/stage0/Cargo.toml --bin claim_batch --target-dir target 2>&1 | tail -5
for w in rung_drop_superseded_standing rung_drop_declaration rung_drop_standing_partition rung_drop_shared_capability rung_drop_amendment; do
  f=dag/test/claim/${w}_witness.dag
  [ -f $f ] || { echo "NOFILE $f"; continue; }
  fns=$(grep -oE '^test fn [A-Za-z0-9_]+' $f | awk '{print $3}' | paste -sd,)
  echo "=== $w"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions "$fns" 2>&1 | grep -aE '^(PASS|FAIL)|rror|refus' | cut -c1-240
done
