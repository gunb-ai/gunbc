set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v13-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=8f7af1271081476ec2c2019db4d2d448c481ea52
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
E=dag/gunbc/claim_coverage_census_live.dag
run() { # $1 tag  $2 extra flags
  echo "== $1 t=$(date +%H:%M:%S)"
  timeout 1200 $CB --source-root $PWD/dag --source-root $PWD/src/v2 $2 --entry $E --functions claim_coverage_census_exit > /tmp/v13-$1.log 2>&1
  echo "$1 RC=$? PASS=$(grep -acE '^PASS ' /tmp/v13-$1.log) FAIL=$(grep -acE '^FAIL ' /tmp/v13-$1.log)"
  grep -aE '^(PASS|FAIL) ' /tmp/v13-$1.log | cut -c1-190
}
run dry ""
run wet "--wet"
# stub A: channel constant (replace the outside_roots_census call with a held channel)
python3 - <<'PY'
E='dag/gunbc/claim_coverage_census_live.dag'
s=open(E).read()
old='''  let outside = outside_roots_census(
    listing: tracked_claim_listing(),
    bearers: tracked_claim_bearers()
  )'''
new='''  let outside = OutsideChannelHeld'''
assert s.count(old)==1
open(E,'w').write(s.replace(old,new)); print("STUB_A_APPLIED")
PY
run dry_stubA ""
git checkout -q -- $E
# stub B: keep census, stub the two head receipt lines to constants
python3 - <<'PY'
L='src/v2/workflow/outside_discovery_roots_census_live.dag'
s=open(L).read()
import re
m=re.search(r'fn outside_roots_head_receipt_line\(\) -> String \{.*?\n\}\n', s, re.S)
assert m, "receipt fn not found"
s2=s[:m.start()]+'fn outside_roots_head_receipt_line() -> String {\n  "S1-CENSUS head stubbed"\n}\n'+s[m.end():]
open(L,'w').write(s2); print("STUB_B_APPLIED")
PY
run dry_stubB ""
git checkout -q -- $L
echo "ALL_DONE_V13 $(date +%H:%M:%S)"
