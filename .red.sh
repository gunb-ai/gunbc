export GUNBC_MEMORY_BUDGET_BYTES=20000000000
(cd src/v1/stage0 && cargo build --release --bin claim_batch 2>&1 | tail -2)
CB=$(find . -path '*release/claim_batch' -type f | head -1)
$CB --source-root dag --source-root src/v2 --entry dag/test/claim/generic_slot_substitution_witness.dag --functions w_slot_bound_application_takes_the_later_binding,w_slot_bound_application_keeps_its_argument_and_refuses_a_mismatch,w_bare_argument_is_substituted_without_a_slot,w_list_slot_takes_the_later_binding 2>&1 | grep -E "w_slot|w_bare|w_list|PASS|FAIL" | cut -c1-250
