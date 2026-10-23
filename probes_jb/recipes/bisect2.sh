cd /tmp/jb/biswt
for c in e5719328d0c^ e5719328d0c 947458435a0 947458435a0^; do
  git checkout -q -f --detach $c || { echo "$c checkout fail" >> /tmp/jb/bisect2.out; continue; }
  s=$(git rev-parse --short HEAD)
  /tmp/jb/maintarget/release/claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/compiler/infer_optional_at_required_witness.dag --functions oar_arrow_nullary_is_accepted_holds,oar_an_optional_returned_at_a_declared_int_refuses_holds > /tmp/jb/bis2_$s.log 2>&1
  echo "$c rc=$? $s $(grep -E '^(PASS|FAIL)' /tmp/jb/bis2_$s.log | tr '\n' ' ')" >> /tmp/jb/bisect2.out
done
echo finished >> /tmp/jb/bisect2.out
