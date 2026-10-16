set -o pipefail
export GUNBC_MEMORY_BUDGET_BYTES=2000000000
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -3
for W in dag/test/claim/verification_evidence_addressing_witness.dag dag/test/claim/roadmap/roadmap_attempt_continuation_witness.dag; do
  F=$(grep -o "^test fn [a-z_0-9]*" $W | awk '{print $3}' | paste -sd,)
  echo "== $W"
  ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $W --functions "$F" 2>&1 | grep -a "^PASS\|^FAIL\|^ERROR\|error:" | cut -c1-300
done
