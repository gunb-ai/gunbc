set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
HEAD_SHA=8f7af1271081476ec2c2019db4d2d448c481ea52
BASE_SHA=201fe0bd10da30ff0eeabaf2eabfc1b42f5e02f1
E=dag/test/claim/claim_coverage_census_witness.dag
C=the_live_entry_exits_one_while_the_wet_lane_has_no_executor

# --- build a DIAGNOSTIC (uncommitted) binary at head: remove the located-only guard
# so EVERY TypeError carries its unwind chain "msg <- fn <- fn <- ...". ---
R=/root/workspace/v19-head; rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $HEAD_SHA || { echo CLONE_FAIL_HEAD; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL_HEAD; exit 1; }
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
echo "== claim1 head (diagnostic binary) t=$(date +%H:%M:%S)"
timeout 3300 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C" > /tmp/v19-head.log 2>&1
echo "run[head] RC=$? t=$(date +%H:%M:%S)"
grep -aE '^(PASS|FAIL|PANICKED) ' /tmp/v19-head.log || echo "[head] NO_VERDICT"

# --- same binary, fork-base tree ---
B=/root/workspace/v19-base; rm -rf $B; mkdir -p $B; cd $B
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $BASE_SHA || { echo CLONE_FAIL_BASE; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL_BASE; exit 1; }
echo "TREE[base] $(git rev-parse HEAD)"
echo "== claim1 base (same diagnostic binary) t=$(date +%H:%M:%S)"
timeout 3300 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $E --functions "$C" > /tmp/v19-base.log 2>&1
echo "run[base] RC=$? t=$(date +%H:%M:%S)"
grep -aE '^(PASS|FAIL|PANICKED) ' /tmp/v19-base.log || echo "[base] NO_VERDICT"
echo "ALL_DONE_V19 $(date +%H:%M:%S)"
