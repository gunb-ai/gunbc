export GUNBC_MEMORY_BUDGET_BYTES=30064771072
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
for f in "$@"; do
  fns=$(grep -h '^test fn' $f | sed -E 's/^test fn ([A-Za-z0-9_]+).*/\1/' | paste -sd,)
  timeout 1500 ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $fns > /tmp/o.txt 2>&1; rc=$?
  echo "=== $f rc=$rc"
  grep -vE "floor-phase|floor-drain|floor-reach|floor-shared|^\[witness\]|resolve-split|assembly-split|cost-partition|◷|✓|◐" /tmp/o.txt | cut -c1-400 | tail -12
done
