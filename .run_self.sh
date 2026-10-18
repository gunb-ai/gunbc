set -u
export GUNBC_MEMORY_BUDGET_BYTES=20000000000
fixed=0
for round in 1 2 3 4; do
  (cd src/v1/stage0 && cargo build --release --bin claim_executor 2>&1 | tail -3)
  B=$(find . -path '*release/claim_executor' -type f | head -1)
  $B --required-regen --source-root dag --source-root src/v2 > regen$round.txt 2>&1; echo "round $round rc=$?"; grep -iE "error|refus|diagnostic" regen$round.txt | head -8 | cut -c1-400
  if grep -q "first_generation_equal=true" regen$round.txt; then fixed=1; break; fi
  cp -r target/stage0-regen-candidate/src/. src/v1/stage0/src/
done
echo FIXED=$fixed
(cd src/v1/stage0/src && tar czf - .) | base64 -w0 > /tmp/m.b64; echo MIRSTART; cat /tmp/m.b64; echo; echo MIREND
cargo build --release 2>&1 | tail -3
G=$(find . -path '*release/gunbc' -type f | head -1)
$G test //gunbc/instruments:self-host 2>&1 | tail -40
