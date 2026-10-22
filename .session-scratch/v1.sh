set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v1-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=7f159b5d9a62bcfdb787126ab5da1246be2353d9
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD) t=$(date +%H:%M:%S)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch --bin gunbc 2>&1 | tail -3; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
run_one() {
  local ENTRY="$1"; local TAG="$2"; local EXTRA="$3"
  local LIST=/tmp/v1-$TAG.list
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' "$ENTRY" > "$LIST"
  local N=$(wc -l < "$LIST")
  echo "== $TAG entry=$ENTRY n=$N t=$(date +%H:%M:%S)"
  timeout 1500 $CB --source-root $PWD/dag --source-root $PWD/src/v2 $EXTRA --entry "$ENTRY" --functions "$(paste -sd, "$LIST")" > /tmp/v1-$TAG.log 2>&1
  echo "$TAG RC=$? PASS=$(grep -acE '^PASS ' /tmp/v1-$TAG.log) FAIL=$(grep -acE '^FAIL ' /tmp/v1-$TAG.log) TOTAL=$N"
  grep -aE '^FAIL ' /tmp/v1-$TAG.log | head -12 | cut -c1-120
}
run_one src/v2/test/claim/claim_coverage_census_s1_witness.dag s1wit ""
run_one dag/test/claim/claim_coverage_census_witness.dag wetwit ""
run_one src/v2/test/claim/outside_discovery_roots_census_witness.dag outwit ""
run_one src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag pair ""
run_one src/v2/test/claim/outside_discovery_roots_census_real_execution.dag outwet "--wet"
echo "ALL_DONE_V1 $(date +%H:%M:%S)"
