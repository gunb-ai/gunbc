#!/bin/bash
# Dispatch G at this head: the declaration control ALONE, with wider greps.
set -u
STORE=dag/gunbc/deploy_transition_store.dag
echo "HEAD $(git rev-parse HEAD 2>/dev/null)"
cargo build --release -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | tail -1
cp $STORE /tmp/store.bak
echo "##### DECLARATION CONTROL: transition_inhibition_slots deleted"
python3 - <<'PY'
p='dag/gunbc/deploy_transition_store.dag'; s=open(p).read()
old='data transition_inhibition_slots: TransitionInhibitionDeclaration = TransitionInhibitionDeclaration {\n  member: "transition-inhibition.json" as NonEmptyStr,\n}\n'
assert s.count(old)==1, "decl not found once"
open(p,'w').write(s.replace(old,''))
print("declaration deleted from", p)
PY
for spec in "dag/test/claim/deploy_transition_witness.dag:the_record_is_written_where_the_declaration_says" "dag/test/claim/deploy_transition_wet_witness.dag:a_written_record_decodes_back_to_the_same_fields" "dag/test/claim/transition_inhibition_store_seal_witness.dag:the_lawful_mint_route_is_admitted" "dag/gunbc/roadmap/roadmap_belt_actuate.dag:belt_transition_admission_for_instance" "dag/gunbc/roadmap/roadmap_launch_deployment_cli.dag:rlm_launch_deployment_receipt_bound"; do
  e=${spec%%:*}; f=${spec##*:}; echo "-- $e ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $e --functions $f > /tmp/decl.out 2>&1; echo "rc=$?"
  grep -E "transition_inhibition_slots|not found in module" /tmp/decl.out | sed 's/ (imported.*//' | sort | uniq -c | head -6
  grep -E "^(PASS|FAIL)" /tmp/decl.out | cut -c1-200 | head -2
done
cp /tmp/store.bak $STORE
git checkout -- dag/gunbc/deploy_transition_store.dag 2>/dev/null
echo "##### DONE"
