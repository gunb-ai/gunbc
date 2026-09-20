set -o pipefail
export GUNBC_BIND_MEMORY_CGROUP_BYTES=$((7*1024*1024*1024))
T0=$(date +%s)
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -2
echo "build_s=$(( $(date +%s) - T0 ))"
T1=$(date +%s)
./target/release/claim_batch --entry dag/test/claim/p256_dag_ecdsa_witness_test.dag \
  --functions the_rfc6979_sample_signature_verifies \
  --eval-budget-ms 900000 2>&1 | tail -30
echo "verify_exit=$? verify_s=$(( $(date +%s) - T1 ))"
echo "EX=done"
