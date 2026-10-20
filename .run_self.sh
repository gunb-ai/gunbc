set -u
export GUNBC_MEMORY_BUDGET_BYTES=20000000000
export GUNBC_BIND_MEMORY_CGROUP_BYTES=21474836480
FNS=w_alias_keyed_argument_mismatch_is_refused_at_if,w_alias_keyed_argument_mismatch_is_refused_at_literal,w_alias_keyed_argument_agreement_is_accepted_at_literal,w_applied_alias_argument_mismatch_is_refused,w_applied_alias_argument_agreement_is_accepted,w_control_without_alias_crossing_is_clean,w_control_genuine_mismatch_is_counted,w_alias_and_expansion_agree_at_if,w_alias_and_expansion_agree_at_argument
ENTRY=dag/test/claim/alias_identity_every_check_witness.dag
fixed=0
for round in 1 2 3 4; do
  (cd src/v1/stage0 && cargo build --release --bin claim_executor 2>&1 | tail -3)
  B=$(find . -path '*release/claim_executor' -type f | head -1)
  $B --required-regen --source-root dag --source-root src/v2 > regen$round.txt 2>&1; echo "round $round rc=$?"; grep -iE "error|refus|diagnostic" regen$round.txt | head -8 | cut -c1-400
  if grep -q "first_generation_equal=true" regen$round.txt; then fixed=1; break; fi
  cp -r target/stage0-regen-candidate/src/. src/v1/stage0/src/
done
echo FIXED=$fixed
(cd src/v1/stage0/src && tar czf - .) | base64 -w0 > /tmp/m.b64; echo MIRSTART; cat /tmp/m.b64; echo; echo MIREND
echo ARM_B_CLAIMS; (cd src/v1/stage0 && cargo build --release --bin claim_batch 2>&1 | tail -2)
CB=$(find . -path '*release/claim_batch' -type f | head -1)
$CB --source-root dag --source-root src/v2 --entry $ENTRY --functions $FNS 2>&1 | grep -E "^(PASS|FAIL)" ; $CB --source-root dag --source-root src/v2 --entry dag/test/claim/generic_slot_substitution_witness.dag --functions w_slot_bound_application_takes_the_later_binding,w_slot_bound_application_keeps_its_argument_and_refuses_a_mismatch,w_bare_argument_is_substituted_without_a_slot,w_list_slot_takes_the_later_binding,w_forwarded_type_variable_in_a_slot_takes_the_later_binding,w_forwarded_type_variable_keeps_its_binding_and_refuses_a_mismatch,w_a_slot_shaped_replacement_is_not_double_wrapped 2>&1 | grep -E "^(PASS|FAIL)" ; echo ARM_A_DONE
echo SEEDTEST; (cd src/v1/stage0 && cargo test --release --test generic_slot_substitution_producer 2>&1 | grep -E '^test |test result|error' | cut -c1-200)
echo MUTARM; F=src/v1/stage0/src/v1_compiler_infer.rs; cp $F /tmp/infer.good.rs; L=$(grep -n "if generic_node_is_bare(c.clone()) {" $F | head -1 | cut -d: -f1); echo mutline=$L; sed -i "${L}s/if generic_node_is_bare(c.clone()) {/if false \&\& generic_node_is_bare(c.clone()) {/" $F; (cd src/v1/stage0 && cargo test --release --test generic_slot_substitution_producer 2>&1 | grep -E '^test |test result' | cut -c1-200); cp /tmp/infer.good.rs $F
cargo build --release 2>&1 | tail -3
G=$(find . -path '*release/gunbc' -type f | head -1)
echo SELFHOST; unset GUNBC_BIND_MEMORY_CGROUP_BYTES; $G test //gunbc/instruments:self-host 2>&1 | tail -12 | cut -c1-300
