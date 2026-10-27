#!/bin/bash
# Fifth dispatch: the declaration control's two roadmap readers, without the
# readback (already captured separately).
set -u
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
echo "##### DECLARATION CONTROL: roadmap readers"
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
