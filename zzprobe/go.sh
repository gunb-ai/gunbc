set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
G=./target/release/gunbc
for f in zzprobe/v2.zzp_*.dag; do
  echo "=== COMPILE $f"
  $G compile --source-root dag --source-root src/v2 --source-root src/v1 --source-root zzprobe --output-dir /tmp/zzo --dependency-pool-index primary-precedence --entry $f --target dag --dry-run 2>&1 | tail -15
done
for fn in $(grep -o '^fn [a-z0-9_]*' zzprobe/runner.dag | cut -d' ' -f2); do
  echo "=== CENSUS $fn"
  $G run --source-root dag --source-root src/v2 --source-root zzprobe --entry zzprobe/runner.dag --function $fn 2>&1 | tail -c 2500
done
