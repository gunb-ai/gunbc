cd /tmp/jb/repath
for f in kernel_value_type_roster_witness generic_closure_outcome_construct_probe free_semigroup_empty_producer_witness infer_expected_type_record_instantiation_witness infer_record_construct_field_inhabitance_witness; do
  p=src/v2/test/claim/compiler/$f.dag
  fns=$(grep -o '^test fn [a-z_0-9]*' $p | awk '{print $3}' | paste -sd,)
  /tmp/jb/maintarget/release/claim_batch --source-root dag --source-root src/v2 --entry /tmp/jb/repath/$p --functions $fns > /tmp/jb/sib_$f.out 2>&1
  echo "$f rc=$? pass=$(grep -c '^PASS' /tmp/jb/sib_$f.out) fail=$(grep -c '^FAIL' /tmp/jb/sib_$f.out) tests=$(echo $fns | tr ',' '\n' | wc -l)" >> /tmp/jb/sib.out
done
echo finished >> /tmp/jb/sib.out
