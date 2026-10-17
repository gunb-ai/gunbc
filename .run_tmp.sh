set -u
export GUNBC_MEMORY_BUDGET_BYTES=20000000000
fixed=0
for round in 1 2 3 4; do
  (cd src/v1/stage0 && cargo build --release --bin claim_executor 2>&1 | tail -3)
  B=$(find . -path '*release/claim_executor' -type f | head -1)
  $B --required-regen --source-root dag --source-root src/v2 > regen$round.txt 2>&1; echo "round $round rc=$?"
  grep -E "first_generation_equal=|FAIL" regen$round.txt | head -5
  if grep -q "first_generation_equal=true" regen$round.txt; then fixed=1; break; fi
  cp -r target/stage0-regen-candidate/src/. src/v1/stage0/src/
done
echo FIXED=$fixed
git add -A src/v1/stage0 >/dev/null; git --no-pager diff --cached --stat | tail -3
git --no-pager diff --cached | gzip | base64 -w0 > /tmp/diff.b64; echo "DIFFSTART"; cat /tmp/diff.b64; echo; echo "DIFFEND"
(cd src/v1/stage0 && cargo build --release --bin claim_batch 2>&1 | tail -2)
CB=$(find . -path '*release/claim_batch' -type f | head -1)
for f in alias_identity_every_check_witness coproduct_payload_soundness_witness declared_type_inhabitance_list_element_witness; do
  echo "FILE $f"; GUNBC_BIND_MEMORY_CGROUP_BYTES=21474836480 $CB --source-root dag --source-root src/v2 --entry dag/test/claim/$f.dag 2>&1 | grep -E "^(PASS|FAIL)"
done
