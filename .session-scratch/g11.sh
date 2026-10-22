set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/g11-root
E=dag/test/claim/serving_caller_backlog_witness.dag
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 20 origin main || { echo CLONE_FAIL; exit 1; }
mkdir -p /tmp/g11-out
for SHA in d8cf288b844cd03dd65ade0b001a14b0a4f3c910 f9b9595ffb78f973a567184b3345a71dd7773bc2 0ed9cbec4a34f69716f2d29831793aee128bc9d2; do
  git checkout -q $SHA 2>/dev/null || { echo "CHECKOUT_FAIL $SHA"; continue; }
  echo "TREE_OK $SHA $(git rev-parse HEAD)"
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $E > /tmp/g11.list
  echo "LIST_COUNT=$(wc -l < /tmp/g11.list)"
  cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch --bin gunbc 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
  timeout 1200 $CARGO_TARGET_DIR/release/claim_batch --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$(paste -sd, /tmp/g11.list)" > /tmp/g11-cb-$SHA.log 2>&1
  echo "CB_RC=$? CB_PASS=$(grep -acE '^PASS ' /tmp/g11-cb-$SHA.log) CB_FAIL=$(grep -acE '^FAIL ' /tmp/g11-cb-$SHA.log)"
  grep -aE 'type mismatch' /tmp/g11-cb-$SHA.log | head -3 | cut -c1-300
  timeout 1200 $CARGO_TARGET_DIR/release/gunbc compile --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --output-dir /tmp/g11-out --target dag > /tmp/g11-co-$SHA.log 2>&1
  echo "COMPILE_RC=$?"
  grep -aE 'type mismatch|harness_seat' /tmp/g11-co-$SHA.log | head -4 | cut -c1-300
done
echo "ALL_DONE_G11 $(date +%H:%M:%S)"
