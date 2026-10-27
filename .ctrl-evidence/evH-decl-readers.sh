#!/bin/bash
# Dispatch H at this head: the declaration control's hermetic readers, with the RIGHT claim names.
# The declaration's readers are: the mint (3 sites in the store), the hermetic witness's import +
# two claims that read `transition_inhibition_slots.member`, and nothing else in the repo.
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
print("declaration deleted")
PY
for f in a_duplicated_field_refuses_rather_than_picking_one a_listing_match_is_line_exact_and_not_a_substring a_written_record_decodes_back_to_the_same_fields; do
  echo "-- deploy_transition_witness ($f)"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/deploy_transition_witness.dag --functions $f > /tmp/decl.out 2>&1; echo "rc=$?"
  grep -E "transition_inhibition_slots" /tmp/decl.out | grep -E "error" | sed 's/ (imported.*//' | sort | uniq -c | head -5
  grep -E "^(PASS|FAIL)" /tmp/decl.out | cut -c1-240 | head -2
done
cp /tmp/store.bak $STORE
echo "##### DONE"
