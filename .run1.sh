set -uo pipefail
echo "HEAD=$(git rev-parse HEAD 2>/dev/null || echo nogit)"
cargo build --release -p v1-compiler --bin claim_batch --target-dir /tmp/cb-tgt 2>&1 | tail -3
B=/tmp/cb-tgt/release/claim_batch
sha256sum $B
run() { f=$1; fns=$(grep -oP '^test fn \K\w+' $f | paste -sd,); echo "=== WITNESS $f"; $B --claim-run --hermetic --source-root dag --source-root src/v2 --entry $f --functions "$fns" 2>&1 | tail -40; echo "=== EXIT $f ${PIPESTATUS[0]}"; }
for f in "$@"; do run $f; done
echo "=== CLOSURE"
$B --source-root dag --source-root src/v2 --entry dag/gunbc/machine_intake/dimm_physical_orientation.dag --print-entry-closure 2>&1 | grep -i mtcollins; echo "=== ROWCLOSURE"; $B --source-root dag --source-root src/v2 --entry dag/gunbc/machine_intake/dimm_row_orientation.dag --print-entry-closure 2>&1 | grep -ic mtcollins1
