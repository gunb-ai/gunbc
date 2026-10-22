set -u
export CARGO_TARGET_DIR=/root/workspace/cargo-target-persist
R=/root/workspace/v7-root
FEAT=text_lookup_work_counter,interp_test_witness,test_hooks
SHA=8f7af1271081476ec2c2019db4d2d448c481ea52
rm -rf $R; mkdir -p $R; cd $R
git init -q . && git remote add origin https://github.com/gunb-ai/gunbc.git
git fetch -q --depth 1 origin $SHA || { echo CLONE_FAIL; exit 1; }
git checkout -q FETCH_HEAD || { echo CHECKOUT_FAIL; exit 1; }
echo "TREE_OK $(git rev-parse HEAD)"
cargo build -q -p v1-compiler --release --features $FEAT --bin claim_batch 2>&1 | tail -2; echo "BUILD_RC=$? t=$(date +%H:%M:%S)"
CB=$CARGO_TARGET_DIR/release/claim_batch
cp dag/gunbc/claim_coverage_census_live.dag /tmp/pristine_composed.dag
# helper: extract red_gen.py from the pushed commit? not tracked — embed inline.
cat > /tmp/red_gen.py <<'PY'
import sys
E='dag/gunbc/claim_coverage_census_live.dag'
MUTS={
 'M1_drop_outside_from_findings':(
   '  let findings = concat(concat(wet_reason, s1_channel), outside_channel)',
   '  let findings = concat(wet_reason, s1_channel)'),
 'M2_outside_unread_degrades_to_held':(
   '''      _ => match outside {
        // Nothing found, S1 read clean, but the outside census could not read its listing: no observation.
        OutsideChannelUnread { cause: cause } => ComposedUnread { cause: cause }
        _ => ComposedHeld
      }''',
   '      _ => ComposedHeld'),
 'M3_outside_unread_note_dropped':(
   '''    let outside_unread_note = match outside {
      OutsideChannelUnread { cause: cause } => [unread_beside_finding_note(cause: cause)]
      _ => []
    }''',
   '    let outside_unread_note = []'),
}
mode=sys.argv[1]; key=sys.argv[2]
old,new=MUTS[key]
s=open(E).read()
if mode=='apply':
    assert s.count(old)==1, f"needle count {s.count(old)} != 1 for {key}"
    open(E,'w').write(s.replace(old,new)); print(f"APPLIED {key}")
elif mode=='revert':
    assert s.count(new)>=1, f"revert needle not found for {key}"
    open(E,'w').write(s.replace(new,old,1)); print(f"REVERTED {key}")
PY
run_s1wit() {
  TAG="$1"
  awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' src/v2/test/claim/claim_coverage_census_s1_witness.dag > /tmp/v7-$TAG.list
  N=$(wc -l < /tmp/v7-$TAG.list)
  echo "== s1wit[$TAG] n=$N t=$(date +%H:%M:%S)"
  timeout 1500 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --entry src/v2/test/claim/claim_coverage_census_s1_witness.dag --functions "$(paste -sd, /tmp/v7-$TAG.list)" > /tmp/v7-$TAG.log 2>&1
  echo "s1wit[$TAG] RC=$? PASS=$(grep -acE '^PASS ' /tmp/v7-$TAG.log) FAIL=$(grep -acE '^FAIL ' /tmp/v7-$TAG.log)"
  grep -aE '^FAIL ' /tmp/v7-$TAG.log | cut -c1-110
}
# 1) baseline s1wit green at new head
run_s1wit base
# 2) outwet at new head (the repaired detector)
awk '/^test fn /{ name=$3; sub(/\(.*/,"",name); print name }' src/v2/test/claim/outside_discovery_roots_census_real_execution.dag > /tmp/v7-outwet.list
echo "== outwet n=$(wc -l < /tmp/v7-outwet.list) t=$(date +%H:%M:%S)"
timeout 1200 $CB --source-root $PWD/dag --source-root $PWD/src/v2 --wet --entry src/v2/test/claim/outside_discovery_roots_census_real_execution.dag --functions "$(paste -sd, /tmp/v7-outwet.list)" > /tmp/v7-outwet.log 2>&1
echo "outwet RC=$? PASS=$(grep -acE '^PASS ' /tmp/v7-outwet.log) FAIL=$(grep -acE '^FAIL ' /tmp/v7-outwet.log)"
grep -aE '^FAIL ' /tmp/v7-outwet.log | cut -c1-110
# 3) three RED mutations, each must break exactly its own claims
for M in M1_drop_outside_from_findings M2_outside_unread_degrades_to_held M3_outside_unread_note_dropped; do
  python3 /tmp/red_gen.py apply $M || { echo "APPLY_FAIL $M"; break; }
  run_s1wit $M
  python3 /tmp/red_gen.py revert $M || echo "REVERT_FAIL $M"
  cmp -s dag/gunbc/claim_coverage_census_live.dag /tmp/pristine_composed.dag && echo "RESTORE_OK $M" || echo "RESTORE_DIRTY $M"
done
echo "ALL_DONE_V7 $(date +%H:%M:%S)"
