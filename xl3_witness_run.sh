set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -3
G=./target/release/gunbc
run1() { # file fn -> prints VERDICT
  out=$($G run --source-root dag --entry "$1" --function "$2" 2>&1)
  if echo "$out" | grep -q "returned \`true\`"; then echo "TRUE  $2";
  elif echo "$out" | grep -q "returned \`false\`"; then echo "FALSE $2";
  else echo "OTHER $2"; echo "$out" | tail -3; fi
}
SEM=dag/test/claim/semantic_conformance_witness_test.dag
AUD=dag/test/claim/expected_value_authority_audit_witness_test.dag
echo "=== BASELINE (all must be TRUE) at rev $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
for f in conformance_all_routes_satisfy_authority_conforms unanimous_agreement_on_a_wrong_value_is_not_conformance legacy_observation_cannot_move_a_native_verdict legacy_divergence_alone_is_a_legacy_defect_and_never_blocks evaluator_divergence_blocks_on_route_and_not_off_it a_record_with_no_v2_observation_is_a_witness_defect a_subject_with_no_primary_authority_is_authority_missing_and_blocks effect_receipt_in_order_conforms a_reordered_effect_receipt_is_a_host_realization_defect a_dropped_effect_is_a_host_realization_defect an_unmeasured_coverage_area_blocks_the_gate_with_no_dispositions a_blocking_disposition_shuts_a_fully_measured_gate two_authorities_disagreeing_about_one_subject_conflict admission_is_derived_from_the_gate_and_carries_the_revision; do run1 $SEM $f; done
for f in production_partition_is_declared_unmeasured_not_measured_empty an_unmeasured_partition_carries_no_refusal_causes a_measured_partition_with_no_rows_is_refused a_clean_measured_partition_is_admitted_and_counts_both_sides an_origin_claim_without_a_basis_is_refused_by_identity a_duplicated_claim_identity_is_refused; do run1 $AUD $f; done

SC=dag/gunbc/semantic_conformance.dag
AC=dag/gunbc/expected_value_authority_audit.dag
sha_before_sc=$(sha256sum $SC); sha_before_ac=$(sha256sum $AC)

echo "=== MUTATION 1: authority made vacuous (ExpectsValue always satisfied) -> unanimous-agreement witness must go FALSE"
python3 - <<PY
s=open("$SC").read()
s=s.replace("ExpectsValue { digest: d } => observed_result_eq(a: observed, b: ProducedValue { digest: d })",
            "ExpectsValue { digest: d } => observed_was_run(observed: observed)",1)
open("$SC","w").write(s)
PY
run1 $SEM unanimous_agreement_on_a_wrong_value_is_not_conformance
git checkout -- $SC

echo "=== MUTATION 2: LegacyV1Defect made blocking -> legacy-never-blocks witness must go FALSE"
python3 - <<PY
s=open("$SC").read()
s=s.replace("    LegacyV1Defect { detail: _ } => false","    LegacyV1Defect { detail: _ } => true",1)
open("$SC","w").write(s)
PY
run1 $SEM legacy_divergence_alone_is_a_legacy_defect_and_never_blocks
git checkout -- $SC

echo "=== MUTATION 3: effect receipt length check dropped -> dropped-effect witness must go FALSE"
python3 - <<PY
s=open("$SC").read()
s=s.replace("""  list_length(items: expected) == list_length(items: receipt)
    && fold(""","""  fold(""",1)
open("$SC","w").write(s)
PY
run1 $SEM a_dropped_effect_is_a_host_realization_defect
git checkout -- $SC

echo "=== MUTATION 4: basis check inverted in audit refusals -> origin-without-basis witness must go FALSE"
python3 - <<PY
s=open("$AC").read()
s=s.replace("if !basis_is_available(b: r.basis) && !origin_is_unmeasured(o: r.origin) {",
            "if false {",1)
open("$AC","w").write(s)
PY
run1 $AUD an_origin_claim_without_a_basis_is_refused_by_identity
git checkout -- $AC

echo "=== BYTE-RESTORE PROOF"
[ "$(sha256sum $SC)" = "$sha_before_sc" ] && echo "semantic_conformance.dag restored byte-identical" || echo "RESTORE FAILED $SC"
[ "$(sha256sum $AC)" = "$sha_before_ac" ] && echo "expected_value_authority_audit.dag restored byte-identical" || echo "RESTORE FAILED $AC"
