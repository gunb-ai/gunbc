set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
R=/root/workspace/v23a-root; rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin 8f7af1271081476ec2c2019db4d2d448c481ea52 || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE[head] $(git rev-parse HEAD)"
python3 - <<'PY'
f='src/v1/stage0/src/v1_interpreter.rs'
s=open(f).read()
old='''            if msg.contains(" [at ") && msg.matches(" <- ").count() < 12 =>'''
new='''            if msg.matches(" <- ").count() < 12 =>'''
assert s.count(old)==1, f"guard needle count {s.count(old)}"
open(f,'w').write(s.replace(old,new)); print("DIAGNOSTIC_GUARD_REMOVED")
PY
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
echo "BINARY_SHA256 $(sha256sum $CB | cut -d' ' -f1)"
E=dag/test/claim/claim_coverage_census_witness.dag
C1=the_live_entry_exits_one_while_the_wet_lane_has_no_executor
C2=the_live_executor_labels_are_the_required_workflow_step_labels
echo "== v9-exact: two live claims, one process t=$(date +%H:%M:%S)"
timeout 2900 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C1,$C2" > /tmp/v23a.log 2>&1
echo "run RC=$? t=$(date +%H:%M:%S)"
grep -aE '^(PASS|FAIL|PANICKED) ' /tmp/v23a.log || echo "[head] NO_VERDICT"
n=$(grep -an '^FAIL the_live_entry' /tmp/v23a.log | head -1 | cut -d: -f1)
if [ -n "$n" ]; then echo "---- raw FAIL region (UNCUT) ----"; sed -n "${n},$((n+14))p" /tmp/v23a.log | cat -v | sed 's/\x1b\[[0-9;]*m//g'; fi
echo "ALL_DONE_V23A $(date +%H:%M:%S)"
