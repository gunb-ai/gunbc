set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/g10-root
E=dag/test/claim/serving_caller_backlog_witness.dag
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin 6ef957f196c04e91d7e5374720adaf2413c51a1c && git checkout -q FETCH_HEAD || { echo CLONE_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
mkdir -p /tmp/g10-out
# derive 33 claim names from tree
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $E > /tmp/g10.list
echo "LIST_COUNT=$(wc -l < /tmp/g10.list)"

echo "== A_PLAIN_FEATURES_CLAIM_BATCH 6ef957f1 t=$(date +%H:%M:%S)"
cargo build -q -p v1-compiler --release --bin claim_batch 2>&1 | tail -2; echo "BUILD_A_RC=$?"
timeout 1200 $CARGO_TARGET_DIR/release/claim_batch --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$(paste -sd, /tmp/g10.list)" > /tmp/g10-a.log 2>&1
echo "A_RC=$? lines=$(wc -l < /tmp/g10-a.log)"
echo "A_PASS=$(grep -acE '^PASS ' /tmp/g10-a.log) A_FAIL=$(grep -acE '^FAIL ' /tmp/g10-a.log)"
grep -aE 'type mismatch|refus|error|panic' /tmp/g10-a.log | head -5 | cut -c1-260

echo "== B_GUNBC_COMPILE 6ef957f1 t=$(date +%H:%M:%S)"
cargo build -q -p v1-compiler --release --features $FEAT --bin gunbc 2>&1 | tail -1; echo "BUILD_B_RC=$?"
timeout 1200 $CARGO_TARGET_DIR/release/gunbc compile --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --output-dir /tmp/g10-out --target dag > /tmp/g10-b.log 2>&1
echo "B_RC=$? lines=$(wc -l < /tmp/g10-b.log)"
grep -aE 'type mismatch|refus|error|panic' /tmp/g10-b.log | head -5 | cut -c1-260

echo "== C_GUNBC_RUN_CLAIMRUN 6ef957f1 t=$(date +%H:%M:%S)"
ARGS=""; while read -r n; do ARGS="$ARGS --functions $n"; done < /tmp/g10.list
timeout 1800 $CARGO_TARGET_DIR/release/gunbc run --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --claim-run $ARGS > /tmp/g10-c.log 2>&1
echo "C_RC=$? lines=$(wc -l < /tmp/g10-c.log)"
echo "C_PASS=$(grep -acE '^PASS ' /tmp/g10-c.log) C_FAIL=$(grep -acE '^FAIL ' /tmp/g10-c.log)"
grep -aE 'type mismatch|refus|error|panic' /tmp/g10-c.log | head -5 | cut -c1-260
echo "ALL_DONE_G10 $(date +%H:%M:%S)"
