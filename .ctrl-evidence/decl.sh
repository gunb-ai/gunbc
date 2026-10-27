#!/bin/bash
# One dispatch: seed build, witnesses green, realization/seal/declaration REDs, old->new readback.
set -u
F() { grep -o '^test fn [a-z_0-9]*' "dag/test/claim/$1.dag" | sed 's/test fn //' | paste -sd, -; }
CB() { FN="${ONLY:-$(F $1)}"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry "dag/test/claim/$1.dag" --functions "$FN" "${@:2}" > /tmp/cb.out 2>&1; rc=$?; grep -E '^(PASS|FAIL|UNOBSERVED)|error:|claim_batch:|passed|failed' /tmp/cb.out | grep -v '^\[pre-entry\]\|typed_cache\|floor-' | cut -c1-500 | head -60; echo "EXIT[$1 ${*:2}]=$rc PASS=$(grep -c '^PASS' /tmp/cb.out) FAIL=$(grep -c '^FAIL' /tmp/cb.out)"; }
STORE=dag/gunbc/deploy_transition_store.dag; DT=dag/gunbc/deploy_transition.dag
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^ D artifacts' | head -20
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
ls -la --time-style=full-iso target/release/claim_batch target/release/gunbc
cp $STORE /tmp/store.bak; cp $DT /tmp/dt.bak
echo "##### DECLARATION CONTROL: transition_inhibition_slots deleted"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='data transition_inhibition_slots: TransitionInhibitionDeclaration = TransitionInhibitionDeclaration {\n  member: "transition-inhibition.json" as NonEmptyStr,\n}\n'
assert s.count(old)==1; s=s.replace(old,''); open(p,'w').write(s)
PY
for spec in "dag/test/claim/deploy_transition_witness.dag:a_written_record_decodes_back_to_the_same_fields" "dag/test/claim/deploy_transition_wet_witness.dag:two_roots_one_guard_reaches_only_its_own_instance" "dag/test/claim/transition_inhibition_store_seal_witness.dag:the_lawful_mint_route_is_admitted" "dag/gunbc/roadmap/roadmap_belt_actuate.dag:belt_transition_admission_for_instance" "dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag:rlm_launch_deployment_receipt_bound"; do
  e=${spec%%:*}; f=${spec##*:}; echo "-- $e ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $e --functions $f > /tmp/decl.out 2>&1; echo "rc=$?"
  grep -E "transition_inhibition_slots" /tmp/decl.out | grep -E "error" | sed 's/ (imported.*//' | sort | uniq -c | head -6
  grep -E "^(PASS|FAIL)" /tmp/decl.out | cut -c1-200 | head -2
done
cp /tmp/store.bak $STORE
echo "##### READBACK main (pre-change) -> sealed store"
git fetch -q --depth=1 origin 7298b01efc07 2>/dev/null || git fetch -q --depth=1 origin main; OLD=$(git rev-parse FETCH_HEAD); echo "old tree $OLD"
git worktree add -q /tmp/oldtree FETCH_HEAD && cp $PROBES/old_inhibition_writer_probe.dag /tmp/oldtree/dag/test/claim/ && cp $PROBES/new_inhibition_reader_probe.dag dag/test/claim/
grep -c "instance_root: String" /tmp/oldtree/dag/gunbc/deploy_transition.dag
D=$(mktemp -d /tmp/gunbc_readback.XXXXXX)
(cd /tmp/oldtree && $OLDPWD/target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/old_inhibition_writer_probe.dag --function write_old --arg dir=$D 2>&1 | grep -v 'pre-entry\|typed_cache\|floor-' | tail -5; echo "OLD_WRITE_EXIT=${PIPESTATUS[0]}")
ls -la $D; echo "BYTES:"; cat $D/transition-inhibition.json; echo; sha256sum $D/transition-inhibition.json
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/new_inhibition_reader_probe.dag --function read_new --arg dir=$D 2>&1 | grep -v 'pre-entry\|typed_cache\|floor-' | tail -5; echo "NEW_READ_EXIT=${PIPESTATUS[0]}"
rm -f dag/test/claim/new_inhibition_reader_probe.dag
echo "##### DONE"
