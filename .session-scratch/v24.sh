set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=cba9dc575f7a8f9b2dc9d377f0127117f74b473a
R=/root/workspace/v24-root; rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
echo "BINARY_SHA256 $(sha256sum $CB | cut -d' ' -f1)"
# 1) s1wit 43 (imports floor_pure_producer_share via s1 carriers; my debt row must not break it)
SW=src/v2/test/claim/claim_coverage_census_s1_witness.dag
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $SW > /tmp/v24-s1wit.list
echo "== s1wit n=$(wc -l < /tmp/v24-s1wit.list) t=$(date +%H:%M:%S)"
timeout 1200 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $SW --functions "$(paste -sd, /tmp/v24-s1wit.list)" > /tmp/v24-s1wit.log 2>&1
echo "s1wit RC=$? PASS=$(grep -acE '^PASS ' /tmp/v24-s1wit.log) FAIL=$(grep -acE '^FAIL ' /tmp/v24-s1wit.log) t=$(date +%H:%M:%S)"
grep -aE '^FAIL ' /tmp/v24-s1wit.log | cut -c1-110
# 2) live pairing claim: runs the S1 fold over the real tree -> my new cover row must resolve, not STALE-ENROLLED
SP=src/v2/test/claim/claim_coverage_census_s1_live_pairing.dag
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' $SP > /tmp/v24-pair.list
echo "== pair n=$(wc -l < /tmp/v24-pair.list) t=$(date +%H:%M:%S)"
timeout 1400 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry $SP --functions "$(paste -sd, /tmp/v24-pair.list)" > /tmp/v24-pair.log 2>&1
echo "pair RC=$? PASS=$(grep -acE '^PASS ' /tmp/v24-pair.log) FAIL=$(grep -acE '^FAIL ' /tmp/v24-pair.log) t=$(date +%H:%M:%S)"
grep -aE '^(PASS|FAIL) ' /tmp/v24-pair.log | cut -c1-110
# 3) does my debt row's identity resolve (not STALE) in the real universe? grep the pairing log for it
grep -a 'test.claim.claim_coverage_census_witness' /tmp/v24-pair.log | head -3 || echo "[pair log silent on my identity -- fold not printing rows is expected]"
echo "ALL_DONE_V24 $(date +%H:%M:%S)"
