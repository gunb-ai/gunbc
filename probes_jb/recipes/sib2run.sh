for t in sib3:before sib2:after; do d=${t%%:*}; l=${t##*:}
for f in kernel_value_type_roster_witness infer_expected_type_record_instantiation_witness infer_record_construct_field_inhabitance_witness; do
  p=src/v2/test/claim/compiler/$f.dag
  fns=$(grep -o '^test fn [a-z_0-9]*' /tmp/jb/$d/$p | awk '{print $3}' | paste -sd,)
  (cd /tmp/jb/$d && /tmp/jb/maintarget/release/claim_batch --source-root dag --source-root src/v2 --entry /tmp/jb/$d/$p --functions $fns > /tmp/jb/s2_${l}_$f.out 2>&1; echo "rc=$?" >> /tmp/jb/s2_${l}_$f.out)
  echo "$l $f $(tail -1 /tmp/jb/s2_${l}_$f.out) pass=$(grep -c '^PASS' /tmp/jb/s2_${l}_$f.out) fail=$(grep -c '^FAIL' /tmp/jb/s2_${l}_$f.out)"
done; done
echo finished
