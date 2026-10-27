#!/bin/bash
# One dispatch: seed build, witnesses green, realization/seal/declaration REDs, old->new readback.
set -u
F() { grep -o '^test fn [a-z_0-9]*' "dag/test/claim/$1.dag" | sed 's/test fn //' | paste -sd, -; }
CB() { FN="${ONLY:-$(F $1)}"; ./target/release/claim_batch --source-root dag --source-root src/v2 --entry "dag/test/claim/$1.dag" --functions "$FN" "${@:2}" > /tmp/cb.out 2>&1; rc=$?; grep -E '^(PASS|FAIL|UNOBSERVED)|error:|claim_batch:|passed|failed' /tmp/cb.out | grep -v '^\[pre-entry\]\|typed_cache\|floor-' | cut -c1-500 | head -60; echo "EXIT[$1 ${*:2}]=$rc PASS=$(grep -c '^PASS' /tmp/cb.out) FAIL=$(grep -c '^FAIL' /tmp/cb.out)"; }
STORE=dag/gunbc/deploy_transition_store.dag; DT=dag/gunbc/deploy_transition.dag
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^ D artifacts' | head -20
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
ls -la --time-style=full-iso target/release/claim_batch target/release/gunbc
echo "##### GREEN"
CB deploy_transition_witness --hermetic
CB deploy_transition_wet_witness --wet
CB transition_inhibition_store_seal_witness
cp $STORE /tmp/store.bak; cp $DT /tmp/dt.bak
echo "##### RED: mint refused (lists <root>/unlisted)"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='      let listing = Filesystem.List(path: root)\n'
assert s.count(old)==1; s=s.replace(old,'      let listing = Filesystem.List(path: join([root, "/unlisted"], ""))\n'); open(p,'w').write(s)
PY
CB deploy_transition_wet_witness --wet
cp /tmp/store.bak $STORE
echo "##### RED: realization at another location (member renamed)"
sed -i 's/member: "transition-inhibition.json" as NonEmptyStr/member: "transition-inhibition-moved.json" as NonEmptyStr/' $STORE; grep -c moved $STORE
CB deploy_transition_wet_witness --wet
cp /tmp/store.bak $STORE
echo "##### RED: the context's lease root is not the store's (lease_root from a sibling directory)"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='context: TransitionContext { inhibition: s, lease_root: s.root },'
assert s.count(old)==1; s=s.replace(old,'context: TransitionContext { inhibition: s, lease_root: join([s.root as String, "/.."], "") as NonEmptyStr },'); open(p,'w').write(s)
PY
CB deploy_transition_wet_witness --wet
cp /tmp/store.bak $STORE; rm -f /tmp/transition-lease.json
echo "##### RED: the instance-root rule admits every spelling"
cp dag/gunbc/roadmap/roadmap_dashboard_instance.dag /tmp/rdi.bak
python3 - <<'PY'
p='dag/gunbc/roadmap/roadmap_dashboard_instance.dag'; s=open(p).read()
old='  let root = instance.instance_root as String\n  let segments = split(s: root, delimiter: "/")\n'
assert s.count(old)==1; s=s.replace(old,'  let root = "/opt/gunbc"\n  let segments = split(s: root, delimiter: "/")\n'); open(p,'w').write(s)
PY
ONLY=the_context_mint_refuses_every_spelling_but_the_canonical_absolute_root,the_sealed_inhibition_sets_and_clears_in_the_unchanged_format CB deploy_transition_wet_witness --wet
cp /tmp/rdi.bak dag/gunbc/roadmap/roadmap_dashboard_instance.dag
for T in TransitionInhibitionStore TransitionContext; do
  echo "##### SEAL CONTROL: sole_constructor stripped from $T"
  sed -i "s/^type $T sole_constructor {/type $T {/" $STORE; grep -n "^type $T" $STORE
  CB transition_inhibition_store_seal_witness
  cp /tmp/store.bak $STORE
done
for T in TransitionGuard TransitionResidue; do
  echo "##### SEAL CONTROL: sole_constructor stripped from $T"
  sed -i "s/^type $T sole_constructor {/type $T {/" $DT; grep -n "^type $T" $DT
  CB transition_inhibition_store_seal_witness
  cp /tmp/dt.bak $DT
done
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
