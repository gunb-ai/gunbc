cd /tmp/jb/pfm
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
timeout 1500 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $PWD/src/v2/test/claim/emit/module_member_emission.dag --functions a_where_refinement_is_not_refused_for_missing_facts_holds,a_refinement_with_a_lowered_carrier_emits_a_transparent_alias_holds,a_record_beside_a_function_emits_as_a_struct_holds > /tmp/pf.log 2>&1
echo "rc=$?"; git rev-parse HEAD; sha256sum target/release/claim_batch|cut -c1-16
grep -vE '^\[pre-entry\]|typecheck' /tmp/pf.log | cut -c1-500 | tail -15
echo ==END
