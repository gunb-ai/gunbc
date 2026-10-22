#!/bin/bash
# Scratch: D6 = agreement RED via the .dag language rows (the lever the interpreted v1 parse route
# actually reads), plus the full decode-test panic text.
set -uo pipefail
echo "=== HEAD ==="; git rev-parse --short HEAD
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "=== BASELINE GREEN sanity ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports > /tmp/ag6g.log 2>&1
echo "green rc=$?"; grep -a 'unary-prefix' /tmp/ag6g.log | head -2 | cut -c1-200
echo "=== PATCH .dag rows 18->12, 19->13 (both languages) ==="
python3 - <<'PY'
for p in ["dag/extdeps/languages/dag/syntax.dag", "dag/extdeps/languages/rust/syntax.dag"]:
    s=open(p).read()
    n18=s.count("binding_power: 18"); n19=s.count("binding_power: 19")
    s=s.replace("binding_power: 18","binding_power: 12").replace("binding_power: 19","binding_power: 13")
    open(p,'w').write(s); print("patched",p,n18,n19)
    assert (n18,n19)==(2,1), p
PY
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
echo "=== RED (old binding, .dag lever) ==="
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag --function unary_prefix_v1_v2_agreement_reports > /tmp/ag6r.log 2>&1
echo "red rc=$?"
grep -a 'unary-prefix\|ARM\[' /tmp/ag6r.log | head -30 | cut -c1-260
git checkout -- dag/extdeps/languages/dag/syntax.dag dag/extdeps/languages/rust/syntax.dag && echo restored
echo "=== VALUES under the same .dag patch (cross-check of the RED arms) ==="
python3 - <<'PY'
for p in ["dag/extdeps/languages/dag/syntax.dag", "dag/extdeps/languages/rust/syntax.dag"]:
    s=open(p).read()
    s=s.replace("binding_power: 18","binding_power: 12").replace("binding_power: 19","binding_power: 13")
    open(p,'w').write(s)
PY
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
  --entry src/v2/test/claim/manual/unary_prefix_values.dag --function unary_prefix_value_arms_hold > /tmp/val6r.log 2>&1
echo "values-red rc=$?"
grep -a -o 'unary-prefix value arms refused.\{0,400\}' /tmp/val6r.log | head -2
git checkout -- dag/extdeps/languages/dag/syntax.dag dag/extdeps/languages/rust/syntax.dag && echo restored
echo "=== DECODE TEST full panic ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip the_loader_decodes 2>&1 | grep -a -A14 'panicked at' | head -30 | cut -c1-240
