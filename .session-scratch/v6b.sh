set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v6b-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=a80a63e85a55354ad07f34e4ad47c70f91f0adb7
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$?"
CB=$CARGO_TARGET_DIR/release/claim_batch
run_one() {
  ENTRY="$1"; TAG="$2"; EXTRA="$3"
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' "$ENTRY" > /tmp/v6b-$TAG.list
  N=$(wc -l < /tmp/v6b-$TAG.list)
  echo "== $TAG n=$N t=$(date +%H:%M:%S)"
  timeout 900 $CB --source-root $PWD/dag --source-root $PWD/src/v2 $EXTRA --entry "$ENTRY" --functions "$(paste -sd, /tmp/v6b-$TAG.list)" > /tmp/v6b-$TAG.log 2>&1
  echo "$TAG RC=$? PASS=$(grep -acE '^PASS ' /tmp/v6b-$TAG.log) FAIL=$(grep -acE '^FAIL ' /tmp/v6b-$TAG.log)"
  grep -aE '^FAIL ' /tmp/v6b-$TAG.log | head -8 | cut -c1-120
  grep -aE 'refus|error|panic' /tmp/v6b-$TAG.log | grep -av '^PASS' | head -4 | cut -c1-140
}
run_one src/v2/test/claim/outside_discovery_roots_census_witness.dag outwit ""
run_one src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag pair ""
run_one dag/test/claim/route_gap_partition_witness.dag gap ""
run_one src/v2/test/claim/outside_discovery_roots_census_real_execution.dag outwet "--wet"
echo "ALL_DONE_V6B $(date +%H:%M:%S)"
