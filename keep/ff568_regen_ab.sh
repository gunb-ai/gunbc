set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:$PATH"
export RUSTUP_HOME=/opt/rustup

HEAD_SHA=$(git rev-parse HEAD | cut -c1-12)
echo "=== HEAD on runner: $HEAD_SHA ==="
echo "fixmarker_dag=$(grep -c 'THE VALUE SHAPE IS NOT THE WALL' src/v1/04_infer.dag)"
echo "gate_dag=$(grep -o 'actual_base.children |> count) > 0' src/v1/04_infer.dag | wc -l)"
echo "gate_seed=$(grep -o 'actual_base.children.clone().len()' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
echo "inhabited_join_types_seed=$(grep -o 'inhabited_join_types' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
md5sum src/v1/stage0/src/v1_compiler_infer.rs

echo "=== BUILD claim_executor FROM current committed seed ==="
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -3

echo "=== ROUND A: required-regen ==="
timeout 1500 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/ff568_regenA.log 2>&1
echo "rcA=$?"
grep -E "refused:|drift|FAIL|first_generation_equal" /tmp/ff568_regenA.log | head -12

echo "=== ROUND A candidate vs committed ==="
CHANGED=""
for f in target/stage0-regen-candidate/src/*.rs; do
  b=$(basename "$f")
  if [ -f "src/v1/stage0/src/$b" ]; then
    cmp -s "$f" "src/v1/stage0/src/$b" || { echo "DIFF $b"; CHANGED="$CHANGED $b"; }
  fi
done
echo "CHANGED_A:[$CHANGED]"
for b in $CHANGED; do cp "target/stage0-regen-candidate/src/$b" "src/v1/stage0/src/$b"; done
[ -n "$CHANGED" ] && echo "INSTALLED round-A" || echo "ROUND A installed nothing"

echo "=== VERITY after A ==="
echo "gate_seed=$(grep -o 'actual_base.children.clone().len()' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
echo "inhabited_join_types_seed=$(grep -o 'inhabited_join_types' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
md5sum src/v1/stage0/src/v1_compiler_infer.rs

echo "=== TRANSPORT EARLY: diff of installed mirrors vs HEAD ==="
echo "=== BEGIN-DIFF ==="
git diff -- src/v1/stage0/src/ | gzip -9 | base64 -w0
echo ""
echo "=== END-DIFF ==="

echo "=== REBUILD claim_executor FROM installed seed ==="
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -3

echo "=== ROUND B: convergence check ==="
rm -rf target/stage0-regen-candidate
timeout 1500 target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/ff568_regenB.log 2>&1
echo "rcB=$?"
grep -E "refused:|drift|FAIL|first_generation_equal" /tmp/ff568_regenB.log | head -12
CHANGED_B=""
for f in target/stage0-regen-candidate/src/*.rs; do
  b=$(basename "$f")
  if [ -f "src/v1/stage0/src/$b" ]; then
    cmp -s "$f" "src/v1/stage0/src/$b" || { echo "DIFF $b"; CHANGED_B="$CHANGED_B $b"; }
  fi
done
echo "CHANGED_B:[$CHANGED_B]"
if [ -n "$CHANGED_B" ]; then
  for b in $CHANGED_B; do cp "target/stage0-regen-candidate/src/$b" "src/v1/stage0/src/$b"; done
  echo "NOT CONVERGED - round B installed"
  echo "=== BEGIN-DIFF-B ==="
  git diff -- src/v1/stage0/src/ | gzip -9 | base64 -w0
  echo ""
  echo "=== END-DIFF-B ==="
else
  echo "CONVERGED at round B (installed nothing)"
fi
echo "=== FINAL VERITY ==="
echo "gate_seed=$(grep -o 'actual_base.children.clone().len()' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
echo "inhabited_join_types_seed=$(grep -o 'inhabited_join_types' src/v1/stage0/src/v1_compiler_infer.rs | wc -l)"
md5sum src/v1/stage0/src/v1_compiler_infer.rs
git diff --stat -- src/v1/stage0/src/ | cat
echo "=== ROUND-A/B SCRIPT DONE ==="
