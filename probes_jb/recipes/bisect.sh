cd /tmp/jb/biswt
for c in 9e9982624a6 4201736f30f 3e73b5d8ae6 e72ab1f 691df7d c0c4b50 1f79b5e 0ac3102; do
  git checkout -q --detach $c || { echo "$c checkout fail" >> /tmp/jb/bisect.out; continue; }
  /tmp/jb/maintarget/release/claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/compiler/infer_optional_at_required_witness.dag --functions oar_arrow_nullary_is_accepted_holds,oar_the_lift_is_still_accepted_on_the_route_holds,oar_an_optional_returned_at_a_declared_int_refuses_holds > /tmp/jb/bis_$c.log 2>&1
  echo "$c rc=$? $(git rev-parse --short HEAD) $(grep -E '^(PASS|FAIL)' /tmp/jb/bis_$c.log | tr '\n' ' ')" >> /tmp/jb/bisect.out
done
git checkout -q --detach 0ac3102d1b0
echo finished >> /tmp/jb/bisect.out
