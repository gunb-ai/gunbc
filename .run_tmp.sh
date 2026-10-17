set -u
export GUNBC_MEMORY_BUDGET_BYTES=20000000000
(cd src/v1/stage0/src && find . -type f | sort | xargs md5sum) > /tmp/before.md5
fixed=0
for round in 1 2 3 4; do
  (cd src/v1/stage0 && cargo build --release --bin claim_executor 2>&1 | tail -3)
  B=$(find . -path '*release/claim_executor' -type f | head -1)
  $B --required-regen --source-root dag --source-root src/v2 > regen$round.txt 2>&1; echo "round $round rc=$?"
  if grep -q "first_generation_equal=true" regen$round.txt; then fixed=1; break; fi
  cp -r target/stage0-regen-candidate/src/. src/v1/stage0/src/
done
echo FIXED=$fixed
(cd src/v1/stage0/src && find . -type f | sort | xargs md5sum) > /tmp/after.md5
CH=$(diff /tmp/before.md5 /tmp/after.md5 | grep '^>' | awk '{print $3}')
echo "CHANGED: $CH"
(cd src/v1/stage0/src && tar czf - $CH) | base64 -w0 > /tmp/m.b64; echo MIRSTART; cat /tmp/m.b64; echo; echo MIREND
(cd src/v1/stage0 && cargo build --release --bin claim_batch 2>&1 | tail -2)
CB=$(find . -path '*release/claim_batch' -type f | head -1)
GUNBC_BIND_MEMORY_CGROUP_BYTES=21474836480 $CB --source-root dag --source-root src/v2 --entry dag/test/claim/alias_identity_every_check_witness.dag --functions w_alias_and_expansion_agree_at_if,w_alias_and_expansion_agree_at_match_arm,w_alias_and_expansion_agree_at_argument,w_alias_and_expansion_agree_at_record_field,w_alias_and_expansion_agree_at_return,w_alias_and_expansion_agree_at_list_element,w_alias_and_expansion_agree_at_nested_container,w_alias_and_expansion_agree_at_function_type,w_control_without_alias_crossing_is_clean,w_control_genuine_mismatch_is_counted,w_alias_and_expansion_agree_at_bare_if,w_alias_and_expansion_agree_at_bare_match_arm,w_alias_and_expansion_agree_at_bare_argument,w_alias_and_expansion_agree_at_bare_record_field,w_alias_and_expansion_agree_at_bare_return,w_alias_and_expansion_agree_at_bare_list_element,w_alias_and_expansion_agree_at_bare_nested_container,w_alias_and_expansion_agree_at_bare_function_type 2>&1 | grep -E "^(PASS|FAIL)"
GUNBC_BIND_MEMORY_CGROUP_BYTES=21474836480 $CB --source-root dag --source-root src/v2 --entry dag/test/claim/coproduct_payload_soundness_witness.dag --functions cpp_payload_where_coproduct_required_must_refuse,cpp_payload_inside_its_own_arm_still_compiles,cpp_alias_mediated_payload_must_also_refuse,dcp_foreign_coproduct_where_parent_required_must_refuse,dcp_right_coproduct_still_compiles,dcp_primitive_in_coproduct_field_still_refuses,dcp_unresolved_name_in_coproduct_field_still_refuses,bcp_foreign_coproduct_where_bool_required_must_refuse,bcp_bool_where_bool_required_still_compiles,bcp_primitive_where_bool_required_still_refuses 2>&1 | grep -E "^(PASS|FAIL)"
GUNBC_BIND_MEMORY_CGROUP_BYTES=21474836480 $CB --source-root dag --source-root src/v2 --entry dag/test/claim/declared_type_inhabitance_list_element_witness.dag --functions w_plain_kernel_at_declared_list_element_is_refused,w_arm_payload_where_parent_declared_is_refused,w_declared_coproduct_member_is_accepted,w_undefined_name_at_the_list_element_position_still_refuses,w_reachability_control_discriminates_on_the_name,w_optional_carrier_is_undecidable_not_refused 2>&1 | grep -E "^(PASS|FAIL)"
