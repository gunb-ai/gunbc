set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
G=./target/release/gunbc
run1() { # file fn -> prints VERDICT
  out=$($G run --source-root dag --source-root src/v2 --entry "$1" --function "$2" 2>&1)
  if echo "$out" | grep -q "returned \`true\`"; then echo "TRUE  $2";
  elif echo "$out" | grep -q "returned \`false\`"; then echo "FALSE $2";
  else echo "OTHER $2"; echo "$out" | tail -3; fi
}
SEM=dag/test/claim/self_host_semantic_conformance_witness_test.dag
AUD=dag/test/claim/self_host_expected_value_authority_audit_witness_test.dag
SUBJ=dag/test/claim/self_host_semantic_conformance_subjects_witness_test.dag
echo "=== BASELINE (all must be TRUE) at rev $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
for f in conformance_all_routes_satisfy_authority_conforms unanimous_agreement_on_a_wrong_value_is_not_conformance legacy_observation_cannot_move_a_native_verdict legacy_divergence_alone_is_a_legacy_defect_and_never_blocks evaluator_divergence_blocks_on_route_and_not_off_it a_record_with_no_v2_observation_is_a_witness_defect a_subject_with_no_primary_authority_is_authority_missing_and_blocks effect_receipt_in_order_conforms a_reordered_effect_receipt_is_a_host_realization_defect a_dropped_effect_is_a_host_realization_defect an_unmeasured_coverage_area_blocks_the_gate_with_no_dispositions a_blocking_disposition_shuts_a_fully_measured_gate two_authorities_disagreeing_about_one_subject_conflict admission_is_derived_from_the_gate_and_carries_the_revision an_extra_unexpected_effect_is_a_host_realization_defect omitted_arm_triple_holds_only_on_the_exact_pattern a_failed_open_omitted_arm_refuses_a_clean_gate; do run1 $SEM $f; done
for f in production_partition_is_declared_unmeasured_not_measured_empty an_unmeasured_partition_carries_no_refusal_causes a_measured_partition_with_no_rows_is_refused a_clean_measured_partition_is_admitted_and_counts_both_sides an_origin_claim_without_a_basis_is_refused_by_identity a_duplicated_claim_identity_is_refused; do run1 $AUD $f; done
for f in exhaustiveness_subject_expects_the_named_refusal exhaustiveness_fixtures_discriminate exhaustiveness_authority_is_authored; do run1 $SUBJ $f; done

echo "=== BASELINE-ONLY RUN COMPLETE"
