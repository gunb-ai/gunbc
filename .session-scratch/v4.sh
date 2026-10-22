set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v4-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=dcbb13831de0fcbdaa71b2a9df804362cd592d4a
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$?"
CB=$CARGO_TARGET_DIR/release/claim_batch
run_one() {
  ENTRY="$1"; TAG="$2"; EXTRA="$3"
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' "$ENTRY" > /tmp/v4-$TAG.list
  N=$(wc -l < /tmp/v4-$TAG.list)
  echo "== $TAG n=$N t=$(date +%H:%M:%S)"
  timeout 2400 $CB --source-root $PWD/dag --source-root $PWD/src/v2 $EXTRA --entry "$ENTRY" --functions "$(paste -sd, /tmp/v4-$TAG.list)" > /tmp/v4-$TAG.log 2>&1
  echo "$TAG RC=$? PASS=$(grep -acE '^PASS ' /tmp/v4-$TAG.log) FAIL=$(grep -acE '^FAIL ' /tmp/v4-$TAG.log)"
  echo "---- $TAG LOG ----"; head -40 /tmp/v4-$TAG.log; echo "---- $TAG END ----"
}
run_one src/v2/test/claim/claim_coverage_census_s1_witness.dag s1wit ""
run_one dag/test/claim/claim_coverage_census_witness.dag wetwit ""
run_one src/v2/test/claim/outside_discovery_roots_census_witness.dag outwit ""
run_one src/v2/test/claim/outside_discovery_roots_census_real_execution.dag outwet "--wet"
run_one dag/test/claim/route_gap_partition_witness.dag gap ""
run_one src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag pair ""
echo "ALL_DONE_V4 $(date +%H:%M:%S)"
