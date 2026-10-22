set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v8-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=8f7af1271081476ec2c2019db4d2d448c481ea52
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
E=dag/test/claim/claim_coverage_census_witness.dag
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $E > /tmp/v8-all.list
head -3 /tmp/v8-all.list > /tmp/v8-g1.list
sed -n '4,5p' /tmp/v8-all.list > /tmp/v8-g2.list
sed -n '6,7p' /tmp/v8-all.list > /tmp/v8-g3.list
for G in g1 g2 g3; do
  echo "== wetwit[$G] t=$(date +%H:%M:%S)"
  timeout 1200 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$(paste -sd, /tmp/v8-$G.list)" > /tmp/v8-$G.log 2>&1
  echo "wetwit[$G] RC=$? PASS=$(grep -acE '^PASS ' /tmp/v8-$G.log) FAIL=$(grep -acE '^FAIL ' /tmp/v8-$G.log)"
  grep -aE '^(PASS|FAIL) ' /tmp/v8-$G.log | cut -c1-110
done
echo "ALL_DONE_V8 $(date +%H:%M:%S)"
