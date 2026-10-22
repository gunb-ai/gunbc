set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/g12-root
E=dag/test/claim/serving_caller_backlog_witness.dag
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 20 origin main || { echo CLONE_FAIL; exit 1; }
mkdir -p /tmp/g12-out
for SHA in c368c16a4ab2115a62508a0f3d559656aaa029be b611d83472b8a12d5c94e4deee6f1514d701a6b9 63b15ae9a7b45dc200694352f39b8d977f9f390d; do
  git checkout -q $SHA 2>/dev/null || { echo "CHECKOUT_FAIL $SHA"; continue; }
  echo "TREE_OK $SHA $(git rev-parse HEAD)"
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $E > /tmp/g12.list
  echo "LIST_COUNT=$(wc -l < /tmp/g12.list)"
  cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch --bin gunbc 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
  timeout 1200 $CARGO_TARGET_DIR/release/claim_batch --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$(paste -sd, /tmp/g12.list)" > /tmp/g12-cb-$SHA.log 2>&1
  echo "CB_RC=$? CB_PASS=$(grep -acE '^PASS ' /tmp/g12-cb-$SHA.log) CB_FAIL=$(grep -acE '^FAIL ' /tmp/g12-cb-$SHA.log)"
  grep -aE 'type mismatch' /tmp/g12-cb-$SHA.log | head -3 | cut -c1-300
  timeout 1200 $CARGO_TARGET_DIR/release/gunbc compile --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --output-dir /tmp/g12-out --target dag > /tmp/g12-co-$SHA.log 2>&1
  echo "COMPILE_RC=$?"
  grep -aE 'type mismatch|harness_seat' /tmp/g12-co-$SHA.log | head -4 | cut -c1-300
done
echo "ALL_DONE_G11 $(date +%H:%M:%S)"
