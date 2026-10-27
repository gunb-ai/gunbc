#!/bin/bash
# Fourth dispatch: the two receipts the 1h wall keeps cutting off —
# the old->new byte readback, and the two roadmap readers of the declaration.
set -u
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^ D artifacts' | head -10
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
echo "##### READBACK main (pre-change) -> sealed store"
git fetch -q --depth=1 origin 7298b01efc07 2>/dev/null || git fetch -q --depth=1 origin main; OLD=$(git rev-parse FETCH_HEAD); echo "old tree $OLD"
rm -rf /tmp/oldtree
git worktree add -q /tmp/oldtree FETCH_HEAD && cp $PROBES/old_inhibition_writer_probe.dag /tmp/oldtree/dag/test/claim/ && cp $PROBES/new_inhibition_reader_probe.dag dag/test/claim/
echo "old signature arity (instance_root: String lines): $(grep -c 'instance_root: String' /tmp/oldtree/dag/gunbc/deploy_transition.dag)"
D=$(mktemp -d /tmp/gunbc_readback.XXXXXX)
(cd /tmp/oldtree && $OLDPWD/target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/old_inhibition_writer_probe.dag --function write_old --arg dir=$D 2>&1 | grep -v 'pre-entry\|typed_cache\|floor-' | tail -5; echo "OLD_WRITE_EXIT=${PIPESTATUS[0]}")
ls -la $D; echo "BYTES:"; cat $D/transition-inhibition.json; echo; sha256sum $D/transition-inhibition.json
./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/new_inhibition_reader_probe.dag --function read_new --arg dir=$D 2>&1 | grep -v 'pre-entry\|typed_cache\|floor-' | tail -5; echo "NEW_READ_EXIT=${PIPESTATUS[0]}"
rm -f dag/test/claim/new_inhibition_reader_probe.dag
echo "##### ROADMAP READERS under the declaration control"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='data transition_inhibition_slots: TransitionInhibitionDeclaration = TransitionInhibitionDeclaration {\n  member: "transition-inhibition.json" as NonEmptyStr,\n}\n'
assert s.count(old)==1; s=s.replace(old,''); open(p,'w').write(s)
PY
for spec in "dag/gunbc/roadmap/roadmap_belt_actuate.dag:belt_transition_admission_for_instance" "dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag:rlm_launch_deployment_receipt_bound"; do
  e=${spec%%:*}; f=${spec##*:}; echo "-- $e ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $e --functions $f > /tmp/decl.out 2>&1; echo "rc=$?"
  grep -E "transition_inhibition_slots" /tmp/decl.out | grep -E "error" | sed 's/ (imported.*//' | sort | uniq -c | head -6
  grep -E "^(PASS|FAIL)" /tmp/decl.out | cut -c1-200 | head -2
done
git checkout -- dag/gunbc/deploy_transition_store.dag
echo "##### DONE"
