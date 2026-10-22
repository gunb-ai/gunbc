#!/bin/bash
# Scratch (never lands; mirrors untracked): prA MEGA lane = RED(old mirror) -> REGEN ->
# GREEN(new mirror) -> decode tests -> digest A/B, one dispatch.
# Pre-regen the stage0 mirror still carries the historical parser (literal min_bp 12 at the
# prefix arms, literal postfix gates 21/13/14), so the RED battery needs NO mutation: it runs
# the defect. Post-regen it is the declared-table parser (prefix 18; postfix ( 21, [ 21, as 19,
# { 13). Digest A/B reproduces the old binding by patching the declared rows 18->12 and 19->13
# in the two language mirrors (measurement only, restored).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current
git status --porcelain | head -20

run_claim() { # $1 label $2 entry $3 fn
  ./target/release/claim_batch --source-root src --source-root src/v1 --source-root src/v2 --source-root dag \
    --entry "$2" --function "$3" > /tmp/claim_$1.log 2>&1
  local rc=$?
  echo "claim@$1 rc=$rc"
  grep -aE "✗|refus|DISAGREE|ARM\[|want |panic" /tmp/claim_$1.log | head -30
  return 0
}

echo "=== 1. BUILD (pre-regen mirror = historical parser) ==="
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc --bin claim_batch 2>&1 | tail -1

echo "=== 2. RED BATTERY (historical parser; refusals are the defect's face) ==="
run_claim RED_values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim RED_agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports

echo "=== 4. REGEN to fixed point ==="
PREV=-1
for ROUND in 1 2 3; do
  echo "--- REGEN ROUND $ROUND ---"
  target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/regen$ROUND.log 2>&1
  echo "regen-round-cost rc=$?"; grep -aE "changed_paths|RebuildScopeRefused" /tmp/regen$ROUND.log | head -2
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/heal_candidate_stage0.dag --function heal_stage0_overlap_refusal_wet > /tmp/heal$ROUND.log 2>&1
  echo "heal rc=$?"
  target/release/gunbc run --source-root "$ROOT/dag" --source-root "$ROOT/src/v2" --entry dag/gunbc/instruments/seed_build_artifacts_regen.dag --function main_wet > /tmp/regenw$ROUND.log 2>&1
  echo "seed-regen rc=$?"
  WRITTEN=$(git status --porcelain -- src/v1/stage0/src | wc -l | tr -d ' ')
  echo "round $ROUND mirror-dirty=$WRITTEN"
  if [ "$WRITTEN" = "$PREV" ]; then echo "FIXED-POINT round $ROUND wrote 0"; break; fi
  PREV=$WRITTEN
done

echo "=== 4b. DECODE TESTS (item 4: migration; compiled against the REGENERATED mirror, which
=== is the only state where ParseEnvironment carries postfix_operators at all) ==="
cargo test --release -p v1-compiler --test parse_environment_decode_roundtrip 2>&1 | tail -10

echo "=== 5. WITNESSES (declared reads present; literals gone) ==="
echo "find_prefix_binding_power: $(grep -c find_prefix_binding_power src/v1/stage0/src/v1_compiler_parse.rs)"
echo "declared_postfix_gate_admits: $(grep -c declared_postfix_gate_admits src/v1/stage0/src/v1_compiler_parse.rs)"
echo "literal-min-bp prefix arms: $(grep -c 'min_bp: 1[0-9]' src/v1/stage0/src/v1_compiler_parse.rs)"
echo "literal postfix gates: $(grep -cE '21 < min_bp|21 >= min_bp|13 < min_bp|14 <= min_bp' src/v1/stage0/src/v1_compiler_parse.rs)"
echo "postfix_operators std_syntax: $(grep -c postfix_operators src/v1/stage0/src/std_syntax.rs)"
git diff --stat -- src/v1/stage0/src | tail -6

echo "=== 6. PATCH (mirror bring-back; runner has no push credentials) ==="
git add -A src/v1/stage0/src
echo "BEGIN-PATCH"; git diff --cached | base64 -w0; echo; echo "END-PATCH"
git reset -q -- src/v1/stage0/src

echo "=== 7. REBUILD over regenerated mirror ==="
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | tail -1

echo "=== 8. GREEN BATTERY (declared-table parser; all rc=0 expected) ==="
run_claim GREEN_values src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_value_arms_hold
run_claim GREEN_values_emitted src/v2/test/claim/manual/unary_prefix_values.dag unary_prefix_emitted_text_follows_the_tree
run_claim GREEN_agreement src/v2/test/claim/manual/unary_prefix_v1_v2_agreement.dag unary_prefix_v1_v2_agreement_reports
run_claim GREEN_readers src/v2/test/claim/lex_token_class_membership_readers.dag lex_token_class_membership_readers_hold
run_claim GREEN_roster dag/test/claim/v1_interpreter_primitive_roster_acceptance.dag v1_interpreter_primitive_roster_acceptance_contract_holds
run_claim GREEN_dispatch dag/test/claim/v1_interpreter_primitive_dispatch_authority_acceptance.dag v1_interpreter_primitive_dispatch_authority_acceptance_contract_holds
run_claim GREEN_surface dag/test/claim/v1_interpreter_primitive_surface_witness.dag derived_surface_excludes_deleted_bridge_arm

echo "=== 9. DIGEST A/B over FULL corpus ==="
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
./target/release/unary_prefix_digest dag src > /tmp/digest_new.txt 2>/tmp/summary_new.txt
echo "NEW: $(cat /tmp/summary_new.txt)"
grep -c "^SKIPPED" /tmp/digest_new.txt | sed 's/^/NEW skipped-lines: /'

python3 - <<'PY'
hits = {}
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s = open(p).read()
    n18 = s.count(r'binding_power\": 18'); n19 = s.count(r'binding_power\": 19')
    s = s.replace(r'binding_power\": 18', r'binding_power\": 12').replace(r'binding_power\": 19', r'binding_power\": 13')
    open(p, "w").write(s)
    hits[p] = (n18, n19)
print(hits)
assert all(v == (2, 1) for v in hits.values()), "expected 2 prefix rows + 1 as row per language mirror"
PY

cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
./target/release/unary_prefix_digest dag src > /tmp/digest_old.txt 2>/tmp/summary_old.txt
echo "OLD: $(cat /tmp/summary_old.txt)"
grep -c "^SKIPPED" /tmp/digest_old.txt | sed 's/^/OLD skipped-lines: /'

git checkout -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs && echo restored

echo "=== DIGEST DIFF (the acceptance artifact) ==="
diff /tmp/digest_old.txt /tmp/digest_new.txt > /tmp/digest_diff.txt || true
grep -cE "^[<>]" /tmp/digest_diff.txt | sed 's/^/diff-lines: /'
grep -E "^[<>]" /tmp/digest_diff.txt | grep -vc "unary_prefix" | sed 's/^/non-claim diff-lines: /'
grep -E "^[<>]" /tmp/digest_diff.txt | cut -c1-160 | head -60
echo "=== DONE ==="
