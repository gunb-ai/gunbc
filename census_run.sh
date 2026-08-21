set -euo pipefail
cargo build --release -p v1-compiler --bin gunbc >/dev/null 2>&1 || { echo BUILD_FAIL; exit 1; }
mkdir -p target/carrier-realization-census
set +e
./target/release/gunbc run --source-root dag --source-root src/v1 --source-root src/v2 \
  --entry src/v1/tests/claim/carrier_realization_census.dag --function census_write_smoke_receipt > /tmp/run.log 2>&1
echo "RUN_EXIT=$?"
set -e
T=target/carrier-realization-census/smoke.tsv
if [ ! -s "$T" ]; then echo "NO_RECEIPT_FILE"; tail -5 /tmp/run.log; echo "===RUN_END==="; exit 0; fi
echo "RECEIPT_LINES=$(wc -l < $T)"
echo "=== outcome histogram (from the FILE) ==="
awk -F'\t' 'NR>1{print $8}' "$T" | sort | uniq -c | sort -rn
echo "=== String occurrences by resolved decl_file x outcome ==="
awk -F'\t' 'NR>1 && $4=="String"{print $5"\t"$8}' "$T" | sort | uniq -c | sort -rn
echo "=== every divergence row ==="
awk -F'\t' 'NR>1 && $8=="DivergesWithExactIdentity"' "$T"
echo "===RUN_END==="
