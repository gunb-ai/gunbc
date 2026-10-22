set -u
export CARGO_TARGET_DIR=/root/workspace/diag-target
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
R=/root/workspace/v20-diag; rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin 8f7af1271081476ec2c2019db4d2d448c481ea52 || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
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
cp $CB /root/workspace/v20-claim_batch-diag
echo "COPIED /root/workspace/v20-claim_batch-diag"
echo "ALL_DONE_V20A $(date +%H:%M:%S)"
