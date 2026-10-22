#!/bin/bash
# Scratch (never lands): D2 = verification battery over the REGENERATED mirror.
# GREEN battery -> decode tests -> old-binding RED battery (patch rows 18->12, 19->13: the
# faithful historical parser) -> digest A/B full corpus with loud skips -> restore.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current

run_claim() { # $1 label $2 entry $3 fn [maxbytes]
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/claim_$1.log 2>&1
  local rc=$?
  echo "claim@$1 rc=$rc"
  local n=${4:-1800}
  head -c "$n" /tmp/claim_$1.log | tr '\n' '|'
  echo
}

echo "=== BUILD ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1

echo "=== GREEN BATTERY (declared-table parser; rc=0 expected) ==="
run_claim G_values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim G_values_emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim G_agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim G_readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim G_roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim G_dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim G_surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm

echo "=== DECODE TESTS (item 4) ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | tail -6

echo "=== OLD-BINDING RED BATTERY (patch declared rows back to the historical powers) ==="
python3 - <<'PY'
hits = {}
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s = open(p).read()
    n18 = s.count(r'binding_power\": 18'); n19 = s.count(r'binding_power\": 19')
    s = s.replace(r'binding_power\": 18', r'binding_power\": 12').replace(r'binding_power\": 19', r'binding_power\": 13')
    open(p, "w").write(s)
    hits[p] = (n18, n19)
print("patched", hits)
assert all(v == (2, 1) for v in hits.values()), "expected 2 prefix rows + 1 as row per language mirror"
PY
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1
run_claim R_values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold 3000
run_claim R_agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports 6000

echo "=== RESTORE ==="
git checkout -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs && echo restored
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1

echo "=== DIGEST NEW (declared rows) ==="
./target/release/unary_prefix_digest dag src > /tmp/digest_new.txt 2>/tmp/summary_new.txt
echo "NEW: $(cat /tmp/summary_new.txt)"
grep -c '^SKIPPED' /tmp/digest_new.txt | sed 's/^/NEW skipped-lines: /'
grep '^SKIPPED' /tmp/digest_new.txt | head -5

python3 - <<'PY'
hits = {}
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s = open(p).read()
    n18 = s.count(r'binding_power\": 18'); n19 = s.count(r'binding_power\": 19')
    s = s.replace(r'binding_power\": 18', r'binding_power\": 12').replace(r'binding_power\": 19', r'binding_power\": 13')
    open(p, "w").write(s)
    hits[p] = (n18, n19)
print("patched-old", hits)
assert all(v == (2, 1) for v in hits.values())
PY
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
echo "=== DIGEST OLD (historical powers) ==="
./target/release/unary_prefix_digest dag src > /tmp/digest_old.txt 2>/tmp/summary_old.txt
echo "OLD: $(cat /tmp/summary_old.txt)"
grep -c '^SKIPPED' /tmp/digest_old.txt | sed 's/^/OLD skipped-lines: /'
git checkout -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs && echo restored

echo "=== DIGEST DIFF (acceptance artifact: re-spelled sites + any cast-site meaning change) ==="
diff /tmp/digest_old.txt /tmp/digest_new.txt > /tmp/digest_diff.txt || true
grep -cE "^[<>]" /tmp/digest_diff.txt | sed 's/^/diff-lines: /'
grep -E "^[<>]" /tmp/digest_diff.txt | cut -c1-200 | head -80
echo "=== DONE ==="
