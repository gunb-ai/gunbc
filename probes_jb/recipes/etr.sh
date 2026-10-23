for t in sib4 e16 m31; do
 d=/tmp/jb/$t; cd $d
 for f in infer_expected_type_record_instantiation_witness infer_record_construct_field_inhabitance_witness; do cp /tmp/jb/sib4/src/v2/test/claim/compiler/$f.dag src/v2/test/claim/compiler/$f.dag; done
 /tmp/jb/maintarget/release/claim_batch --source-root dag --source-root src/v2 --entry $d/src/v2/test/claim/compiler/infer_expected_type_record_instantiation_witness.dag --functions etr_a_phantom_binder_is_fixed_by_the_declared_type_holds,etr_a_nested_construct_typed_only_by_its_context_is_decided_at_its_declared_return_holds > /tmp/jb/etr_$t.out 2>&1; echo rc=$? >> /tmp/jb/etr_$t.out
 git checkout -q -- src/v2/test 2>/dev/null
done; echo done > /tmp/jb/etr.done
