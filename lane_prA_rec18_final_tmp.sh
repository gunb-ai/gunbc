#!/bin/bash
# Scratch (never lands): lane A FINAL receipts at the record-row head.
# Part 1 GREEN battery (7 claims + decode). Part 2 RED battery: patch BOTH language specs'
#   operator rows back to the FAITHFUL HISTORICAL values (-,! prefix 18->12, as 19->13, { 18->13)
#   -- claim_batch reads .dag specs at runtime, no rebuild -- values + agreement must go RED on
#   the named arms (incl. the two dotted unary record arms, review 84349's cell). Part 3 DIGEST
#   at both bindings over the full corpus (patch the MIRROR json rows, 8 hits expected) PLUS a
#   positive-control corpus of dotted record literals under a prefix: the control MUST diff
#   (proving the digest sees this family), and the corpus diff must stay the banked cast-only set.
set -uo pipefail
PIN=10a9dfa6bbc42186e82be3cdf76109eec0e37b9b
git fetch -q origin 2>/dev/null || true
CUR=$(git rev-parse HEAD)
if [ "$CUR" != "$PIN" ]; then echo "PIN: was $CUR; checkout $PIN"; git checkout -f -q "$PIN" || { echo PIN-FAILED; exit 3; }; fi
test "$(git rev-parse HEAD)" = "$PIN" || { echo PIN-MISMATCH; exit 3; }
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
P=src/v1/stage0/src
echo "=== HEAD ==="; git rev-parse --short HEAD
echo "=== PART 1 GREEN battery ==="
cargo build --release -p v1-compiler --bin claim_batch --bin unary_prefix_digest 2>&1 | tail -1
run_claim() {
  echo "STEP claim $1"; date -u +%H:%M:%S
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/cA_$1.log 2>&1
  echo "claim@$1 rc=$?"
  grep -am1 'FAIL\|resolve failed' /tmp/cA_$1.log | cut -c1-200
}
run_claim values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm
echo "STEP decode tests"; date -u +%H:%M:%S
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | grep -aE 'panicked|assertion|test result' | head -8 | cut -c1-200

echo "=== PART 2 RED battery (faithful historical rows) ==="
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
print("patched both specs to historical 12/12/13/13")
PY
run_claim values_old src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim agreement_old src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
echo "--- agreement_old failing arms ---"
grep -aE 'MISMATCH|DISAGREE|expected|!=' /tmp/cA_agreement_old.log | head -14 | cut -c1-200
git checkout -q -- dag/extdeps/languages/dag/syntax.dag dag/extdeps/languages/rust/syntax.dag
git status --porcelain dag/ | wc -l | sed 's/^/dag-dirty-after-restore=/'

echo "=== PART 3 DIGEST both bindings (mirrors) + CONTROL corpus ==="
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
./target/release/unary_prefix_digest dag src /tmp/recctl > /tmp/digest_new.txt 2>/tmp/summary_new.txt
echo "NEW: $(cat /tmp/summary_new.txt)"
grep -c "^SKIPPED" /tmp/digest_new.txt | sed 's/^/NEW skipped-lines: /'
grep "^/tmp/recctl" /tmp/digest_new.txt | sed 's/^/CTL-NEW /'
python3 - <<'PY'
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
assert hits == 8, f"expected 8 historical rows across 2 mirrors, got {hits}"
print("patched 8 mirror rows to historical")
PY
./target/release/unary_prefix_digest dag src /tmp/recctl > /tmp/digest_old.txt 2>/tmp/summary_old.txt
echo "OLD: $(cat /tmp/summary_old.txt)"
grep -c "^SKIPPED" /tmp/digest_old.txt | sed 's/^/OLD skipped-lines: /'
grep "^/tmp/recctl" /tmp/digest_old.txt | sed 's/^/CTL-OLD /'
git checkout -q -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs
git status --porcelain -uno | wc -l | sed 's/^/dirty-after-restore=/'
echo "--- CONTROL diff (MUST be non-empty; the digest sees this family) ---"
diff <(grep "^/tmp/recctl" /tmp/digest_old.txt) <(grep "^/tmp/recctl" /tmp/digest_new.txt) | sed 's/^/CTL /' | cut -c1-200
echo "--- CORPUS diff (expect ONLY the banked cast items) ---"
grep -v "^/tmp/recctl" /tmp/digest_old.txt > /tmp/dc_old.txt
grep -v "^/tmp/recctl" /tmp/digest_new.txt > /tmp/dc_new.txt
diff /tmp/dc_old.txt /tmp/dc_new.txt | sed 's/^/CORPUS /' | head -30 | cut -c1-200
echo "CORPUS-diff-lines: $(diff /tmp/dc_old.txt /tmp/dc_new.txt | grep -c '^[<>]')"
echo "=== DONE ==="
