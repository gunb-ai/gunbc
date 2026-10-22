set +e
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
mk(){ echo "[$(date +%H:%M:%S)] $*"; }
PIN=f6a9994bf12e9bb7da45d7ccbbe9923718377237
R=/root/workspace/s1-root
mk "START host=$(hostname)"
rm -rf $R; mkdir -p $R
git init -q $R && cd $R || { echo NO_DIR; exit 0; }
git remote add origin https://github.com/gunb-ai/gunbc.git
timeout 600 git fetch --depth=1 origin $PIN 2>&1 | tail -1
git checkout -q FETCH_HEAD
[ "$(git rev-parse HEAD)" = "$PIN" ] || { echo HEAD_IS=$(git rev-parse HEAD); exit 0; }
mk "PIN_OK=$(git rev-parse HEAD)"
cargo build -p v1-compiler --release --features text_lookup_work_counter,interp_test_witness,test_hooks --bin claim_batch 2>&1 | tail -2
mk "BUILD_RC=$?"
BIN=target/release/claim_batch
[ -x "$BIN" ] || BIN=$CARGO_TARGET_DIR/release/claim_batch
ls -la "$BIN" || { echo NO_BIN; exit 0; }
E=src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag
echo "===== PAIRING (real-checkout claim, budget 600s, cap 1200s) ====="
timeout 1200 $BIN --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --function the_module_real_path_pairing_row_grounds_against_the_real_discovery --eval-budget-ms 600000 > /tmp/pair.log 2>&1
echo "PAIR_RC=$?"
grep -aE '^(PASS|FAIL) ' /tmp/pair.log | cut -c1-160
grep -aE 'eval_steps|cpu_ms|wall_ms|steps=' /tmp/pair.log | tail -4 | cut -c1-200
grep -aE 'panic|error|Error|\[resolve\]' /tmp/pair.log | head -4 | cut -c1-200
mk "ALL_DONE_F7"
