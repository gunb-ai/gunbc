#!/bin/bash
# Scratch (never lands): D3 = extract the failing claim reports + decode-test panics.
set -uo pipefail
echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "=== AGREEMENT (declared binding) ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports \
  > /tmp/ag.log 2>&1
echo "rc=$?"
grep -aE 'ARM\[|agreement|DISAGREE|refus' /tmp/ag.log | head -30 | cut -c1-220
echo "=== AGREEMENT (old binding: patch rows) ==="
python3 - <<'PY'
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s = open(p).read()
    n18 = s.count(r'binding_power\": 18'); n19 = s.count(r'binding_power\": 19')
    s = s.replace(r'binding_power\": 18', r'binding_power\": 12').replace(r'binding_power\": 19', r'binding_power\": 13')
    open(p, "w").write(s)
    print("patched", p, n18, n19)
PY
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports \
  > /tmp/agold.log 2>&1
echo "rc=$?"
grep -aE 'ARM\[|agreement|DISAGREE|refus' /tmp/agold.log | head -40 | cut -c1-220
git checkout -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs && echo restored
echo "=== DECODE TEST PANICS ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|left|right|FAILED|test result|revision|refus' | head -30 | cut -c1-220
