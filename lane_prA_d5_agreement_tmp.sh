#!/bin/bash
# Scratch: D5 = agreement claim GREEN, then RED under the historical rows; decode tests.
set -uo pipefail
echo "=== HEAD ==="; git rev-parse --short HEAD
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_ag() { # $1 label
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports > /tmp/ag_$1.log 2>&1
  echo "agreement@$1 rc=$?"
  grep -aE 'unary-prefix|resolve failed|error:' /tmp/ag_$1.log | grep -av 'typecheck\|floor' | head -14 | cut -c1-300
}
echo "=== GREEN (declared rows) ==="
run_ag green
echo "=== RED (patch rows 18->12, 19->13) ==="
python3 - <<'PY'
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s=open(p).read()
    n18=s.count(r'binding_power\": 18'); n19=s.count(r'binding_power\": 19')
    s=s.replace(r'binding_power\": 18', r'binding_power\": 12').replace(r'binding_power\": 19', r'binding_power\": 13')
    open(p,'w').write(s); print("patched",p,n18,n19)
PY
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_ag red
git checkout -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs && echo restored
echo "=== DECODE TESTS ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|left:|right:|FAILED|test result' | head -20 | cut -c1-200
