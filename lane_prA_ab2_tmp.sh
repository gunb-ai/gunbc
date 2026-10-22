#!/bin/bash
# Scratch (never lands): lane A FINAL receipts, take 2 at e17b6a4a11e.
# Take 1 (je034bfb6) had a measurement flaw on two channels: values_old rc=0 and the empty CTL
# digest diff are NOT receipts of the historical binding -- the values claim's parse and the
# digest bin use the seed's rows COMPILED INTO THE BINARY from the stage0 mirrors, so patching
# without rebuilding re-measures the declared binding. This lane patches BOTH authorities (.dag
# specs and stage0 mirrors) and REBUILDS the binaries between bindings.
set -uo pipefail
PIN=e17b6a4a11ec5cd143fdf566b36c43adb9e7beff
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR; checkout $PIN"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
test "$(git rev-parse HEAD)" = "$PIN" || { echo PIN-MISMATCH; exit 3; }
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
echo "=== HEAD ==="; git rev-parse --short HEAD

mkdir -p /tmp/recctl
cat > /tmp/recctl/ctl_dotted_unary.dag <<'EOF'
module rec.ctl.dotted.unary

fn a(n: Int) -> Int {
  -Foo.Bar { x: 1 }
}
EOF
cat > /tmp/recctl/ctl_dotted_bang.dag <<'EOF'
module rec.ctl.dotted.bang

fn a(n: Int) -> Int {
  !Foo.Bar { x: 1 }
}
EOF
cat > /tmp/recctl/ctl_dotted_infix.dag <<'EOF'
module rec.ctl.dotted.infix

fn a(n: Int) -> Int {
  n + Foo.Bar { x: 1 }
}
EOF
cat > /tmp/recctl/ctl_plain_unary.dag <<'EOF'
module rec.ctl.plain.unary

fn a(n: Int) -> Int {
  -Foo { x: 1 }
}
EOF

run_claim() {
  echo "STEP claim $1 @ $BIND"; date -u +%H:%M:%S
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/cB_$1_$BIND.log 2>&1
  echo "claim@$1/$BIND rc=$?"
  grep -a "MISMATCH\|DISAGREE" /tmp/cB_$1_$BIND.log | head -14 | cut -c1-200
}
run_digest() {
  echo "STEP digest @ $BIND"; date -u +%H:%M:%S
  ./target/release/unary_prefix_digest dag src /tmp/recctl > /tmp/digest_$BIND.txt 2>/tmp/summary_$BIND.txt
  echo "SUMMARY-$BIND: $(cat /tmp/summary_$BIND.txt)"
  grep -c "^SKIPPED" /tmp/digest_$BIND.txt | sed 's/^/SKIPPED-lines /'
  grep "^/tmp/recctl" /tmp/digest_$BIND.txt | sed "s/^/CTL-$BIND /"
}

echo "############ BINDING declared (rows 18/18/19/18) ############"
BIND=new
cargo build --release -p v1-compiler --bin claim_batch --bin unary_prefix_digest 2>&1 | tail -1
run_claim values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_digest
echo "STEP decode @ new"; date -u +%H:%M:%S
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|test result' | head -8 | cut -c1-200

echo "############ BINDING historical (rows 12/12/13/13) ############"
BIND=old
python3 - <<'PY'
for p in ["dag/extdeps/languages/dag/syntax.dag", "dag/extdeps/languages/rust/syntax.dag"]:
    s = open(p).read()
    for a, b in [('symbol: "-", binding_power: 18', 'symbol: "-", binding_power: 12'),
                 ('symbol: "!", binding_power: 18', 'symbol: "!", binding_power: 12'),
                 ('symbol: "as", binding_power: 19', 'symbol: "as", binding_power: 13'),
                 ('symbol: "{", binding_power: 18', 'symbol: "{", binding_power: 13')]:
        assert s.count(a) == 1, (p, a, s.count(a))
        s = s.replace(a, b)
    open(p, "w").write(s)
hits = 0
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s = open(p).read()
    for a, b in [('symbol\\": \\"-\\", \\"binding_power\\": 18', 'symbol\\": \\"-\\", \\"binding_power\\": 12'),
                 ('symbol\\": \\"!\\", \\"binding_power\\": 18', 'symbol\\": \\"!\\", \\"binding_power\\": 12'),
                 ('symbol\\": \\"as\\", \\"binding_power\\": 19', 'symbol\\": \\"as\\", \\"binding_power\\": 13'),
                 ('symbol\\": \\"{\\", \\"binding_power\\": 18', 'symbol\\": \\"{\\", \\"binding_power\\": 13')]:
        n = s.count(a)
        assert n == 1, (p, a, n)
        s = s.replace(a, b); hits += 1
    open(p, "w").write(s)
assert hits == 8, f"expected 8 mirror rows, got {hits}"
print("patched both .dag specs and both mirrors to historical 12/12/13/13")
PY
cargo build --release -p v1-compiler --bin claim_batch --bin unary_prefix_digest 2>&1 | tail -1
run_claim values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_digest
echo "STEP decode @ old"; date -u +%H:%M:%S
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|test result' | head -8 | cut -c1-200

git checkout -q -- dag/extdeps/languages/dag/syntax.dag dag/extdeps/languages/rust/syntax.dag src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs
echo "dirty-after-restore=$(git status --porcelain -uno | wc -l)"
echo "--- CONTROL diff (MUST be non-empty) ---"
diff <(grep "^/tmp/recctl" /tmp/digest_old.txt) <(grep "^/tmp/recctl" /tmp/digest_new.txt) | sed 's/^/CTL /' | cut -c1-200
echo "--- CORPUS diff (expect ONLY the banked cast items) ---"
grep -v "^/tmp/recctl" /tmp/digest_old.txt > /tmp/de_old.txt
grep -v "^/tmp/recctl" /tmp/digest_new.txt > /tmp/de_new.txt
diff /tmp/de_old.txt /tmp/de_new.txt | sed 's/^/CORPUS /' | head -30 | cut -c1-200
echo "CORPUS-diff-lines: $(diff /tmp/de_old.txt /tmp/de_new.txt | grep -c '^[<>]')"
echo "=== DONE ==="
