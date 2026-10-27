#!/bin/bash
set -u
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"; git status --short | grep -v '^??' | head
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
STORE=dag/gunbc/deploy_transition_store.dag; cp $STORE /tmp/store.bak
run() { for spec in "dag/gunbc/roadmap/roadmap_belt_actuate.dag:belt_transition_admission_for_instance" "dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag:rlm_launch_deployment_receipt_bound"; do
  e=${spec%%:*}; f=${spec##*:}; echo "-- $e ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $e --functions $f > /tmp/decl.out 2>&1; echo "rc=$?"
  grep -E "error" /tmp/decl.out | grep -v '^\[pre-entry\]' | sed 's/ (imported.*//' | cut -c1-300 | sort | uniq -c | head -6
  grep -E "^(PASS|FAIL|UNOBSERVED)" /tmp/decl.out | cut -c1-300 | head -2
done; }
echo "##### UNMUTATED"; run
echo "##### DECLARATION CONTROL: transition_inhibition_slots deleted"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='data transition_inhibition_slots: TransitionInhibitionDeclaration = TransitionInhibitionDeclaration {\n  member: "transition-inhibition.json" as NonEmptyStr,\n}\n'
assert s.count(old)==1; s=s.replace(old,''); open(p,'w').write(s)
PY
run
cp /tmp/store.bak $STORE
echo "##### DONE"
