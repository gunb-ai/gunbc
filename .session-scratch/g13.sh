set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/g13-root
E=dag/test/claim/serving_caller_backlog_witness.dag
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 20 origin main || { echo CLONE_FAIL; exit 1; }
mkdir -p /tmp/g13-out
for SHA in 63b15ae9a7b45dc200694352f39b8d977f9f390d 3b620b17da63e0e412fcb3cbea9ad103d606046b; do
  git checkout -q $SHA 2>/dev/null || { echo "CHECKOUT_FAIL $SHA"; continue; }
  echo "TREE_OK $SHA"
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $E > /tmp/g13.list
  echo "LIST_COUNT=$(wc -l < /tmp/g13.list)"
  cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch --bin gunbc 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
  echo "-- A_THIRD_ROOT_CLAIM_BATCH $SHA t=$(date +%H:%M:%S)"
  timeout 1500 $CARGO_TARGET_DIR/release/claim_batch --source-root $PWD/dag --source-root $PWD/src/v2 --source-root $PWD/src/v1 --entry $E --functions "$(paste -sd, /tmp/g13.list)" > /tmp/g13-a-$SHA.log 2>&1
  echo "A_RC=$? A_PASS=$(grep -acE '^PASS ' /tmp/g13-a-$SHA.log) A_FAIL=$(grep -acE '^FAIL ' /tmp/g13-a-$SHA.log)"
  grep -aE 'type mismatch|harness_seat.dag:1752|PoolEvent' /tmp/g13-a-$SHA.log | head -4 | cut -c1-300
  echo "-- B_THIRD_ROOT_COMPILE $SHA t=$(date +%H:%M:%S)"
  timeout 1500 $CARGO_TARGET_DIR/release/gunbc compile --source-root $PWD/dag --source-root $PWD/src/v2 --source-root $PWD/src/v1 --entry $E --output-dir /tmp/g13-out --target dag > /tmp/g13-b-$SHA.log 2>&1
  echo "B_RC=$?"
  grep -aE 'type mismatch|harness_seat.dag:1752|PoolEvent' /tmp/g13-b-$SHA.log | head -4 | cut -c1-300
done
echo "ALL_DONE_G13 $(date +%H:%M:%S)"
