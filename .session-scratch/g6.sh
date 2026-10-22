set +e
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
mk(){ echo "[$(date +%H:%M:%S)] $*"; }
PIN=11d6c58f7525a9d100e3c5b76c81fbfeed378444
R=/root/workspace/main-root
mk "START host=$(hostname)"
rm -rf $R; mkdir -p $R
git init -q $R && cd $R || { echo NO_DIR; exit 0; }
git remote add origin https://github.com/gunb-ai/gunbc.git
timeout 600 git fetch --depth=1 origin $PIN 2>&1 | tail -1
git checkout -q FETCH_HEAD
[ "$(git rev-parse HEAD)" = "$PIN" ] || { echo HEAD_IS=$(git rev-parse HEAD); exit 0; }
mk "MAIN_OK=$(git rev-parse HEAD)"
cargo build -p v1-compiler --release --features text_lookup_work_counter,interp_test_witness,test_hooks --bin claim_batch 2>&1 | tail -2
mk "BUILD_RC=$?"
BIN=target/release/claim_batch
[ -x "$BIN" ] || BIN=$CARGO_TARGET_DIR/release/claim_batch
E=dag/test/claim/ledger_row_coherence.dag
SR="--source-root $PWD/dag --source-root $PWD/src/v2"
echo "===== PROBE1 print-entry-closure (900s cap) ====="
( time timeout 900 $BIN $SR --entry $E --print-entry-closure ) > /tmp/closure.log 2>&1
echo "P1_RC=$? LINES=$(wc -l < /tmp/closure.log)"
head -12 /tmp/closure.log | cut -c1-160
tail -6 /tmp/closure.log | cut -c1-160
mk "P1 done"
echo "===== PROBE2 one claim, eval-budget 60s (240s cap) ====="
timeout 240 $BIN $SR --entry $E --function a_fully_rendered_ledger_is_coherent --eval-budget-ms 60000 > /tmp/b1.log 2>&1
echo "P2_RC=$?"; head -30 /tmp/b1.log | cut -c1-200
mk "P2 done"
echo "===== PROBE3 sibling control: witness-style entry cheap claim (120s cap) ====="
timeout 120 $BIN $SR --entry dag/test/claim/claim_coverage_census_witness.dag --function a_source_a_required_job_executes_is_covered_and_the_census_holds > /tmp/w1.log 2>&1
echo "P3_RC=$?"; head -5 /tmp/w1.log | cut -c1-200
mk "P3 done"
mk "ALL_DONE_F6"
