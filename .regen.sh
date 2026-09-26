export GUNBC_MEMORY_BUDGET_BYTES=24000000000
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -1
for round in 1 2 3; do
  echo "=== ROUND $round"
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen_$round.log 2>&1; echo "rc=$?"
  grep -E "first_generation_equal=|FAIL generated|drift|error|refus" /tmp/regen_$round.log | tail -8 | cut -c1-220
  line=$(grep -E "first_generation_equal=" /tmp/regen_$round.log | tail -1)
  [ -z "$line" ] && { tail -8 /tmp/regen_$round.log | cut -c1-220; echo "NO VERDICT LINE -- stop"; break; }
  echo "$line" | grep -q "first_generation_equal=true" && { echo "FIXED POINT"; break; }
  drifted=$(grep -oE "FAIL generated surface drift: .*" /tmp/regen_$round.log | sed 's/FAIL generated surface drift: //' | tr ',' ' ')
  for f in $drifted; do echo "copy $f"; cp target/stage0-regen-candidate/src/$f src/v1/stage0/src/$f; done
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -1
done
echo "=== DIFF"; git diff --stat -- src/v1/stage0/src
echo "=== B64"; git diff -- src/v1/stage0/src | gzip -9 | base64 -w0; echo
