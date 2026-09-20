#!/usr/bin/env bash
# ENCODING-0 hex-syntax consolidation (gunbc#11782): the discriminating RED, ASSERTED.
#
# The four consumers no longer carry their own copy of the 0-9a-f alphabet; they read
# extdeps.numeric.base16's digit rows. This proves it by MUTATING THE ROWS -- digit 15's lowercase
# spelling becomes "F" -- and requiring each named claim to change verdict. A consumer that had kept
# a private code-point range would stay green, which is the bypass this cut deletes.
#
# IT ASSERTS, and exits nonzero when an expectation is unmet. An earlier revision only printed
# claim_batch's exit code and returned 0 whether the claims went red or stayed green, which is the
# decoration DESIGN section 4b(1) forbids of a discriminating RED (review 68970). Both halves of that
# obligation run here: the ACCEPTED POSITIVE CONTROL (every named claim green unmutated) and the
# DISCRIMINATING RED (the subject claims red under the mutation), plus an UNAFFECTED control that
# must stay green in both phases -- a MAC address carrying no "f" -- so a mutation that simply broke
# the corpus could not pass as discrimination.
#
# SCAFFOLD, and the trigger that retires it: `gunbc test <label>` is the modeled route for invoking a
# measurement, and DESIGN's Building & checks rules that a new one is A ROW rather than a flag. This
# file dissolves when an instrument row expresses "mutate this declaration, require these claims to
# flip" -- the mutation and its expected verdicts then live in the model. It exists meanwhile
# because the receipt gate (gunbc#11742) landed before that row.
set -uo pipefail
: "${CLAIM_BATCH:=target/release/claim_batch}"
ROWS=dag/extdeps/numeric/base16.dag
BACKUP=$(mktemp)
cp "$ROWS" "$BACKUP"
restore() { cp "$BACKUP" "$ROWS"; rm -f "$BACKUP"; }
trap restore EXIT

# The claims this cut's consumers own, named rather than discovered: each reads the alphabet rows.
BASE16=dag/test/claim/base16_rfc4648_witness_test.dag
KEYRING=dag/test/claim/approval_decision_store_witness_test.dag
MAC=dag/test/claim/network_mac_witness_test.dag
GIT=dag/test/claim/git_upstream_model_witness_test.dag
SUBJECTS=(
  the_lowercase_alphabet_is_exactly_the_digit_rows
  a_digit_value_is_read_from_the_same_rows
  base16_encodes_rfc4648_foobar_vector
  keyring_loads_sixty_four_lower_hex_and_refuses_the_rest
  the_all_ones_octet_survives_the_round_trip
  witness_git_object_id_text_has_one_canonical_spelling
)
# Carries no "f": it must stay green in BOTH phases, so a corpus-wide break cannot masquerade as
# discrimination.
UNAFFECTED=the_leading_zero_octet_survives_the_round_trip

run_phase() {
  "$CLAIM_BATCH" --claim-run --hermetic --source-root dag --source-root src/v2 \
    --entry "$BASE16" --functions the_lowercase_alphabet_is_exactly_the_digit_rows,a_digit_value_is_read_from_the_same_rows,base16_encodes_rfc4648_foobar_vector \
    --entry "$KEYRING" --functions keyring_loads_sixty_four_lower_hex_and_refuses_the_rest \
    --entry "$MAC" --functions the_all_ones_octet_survives_the_round_trip,the_leading_zero_octet_survives_the_round_trip \
    --entry "$GIT" --functions witness_git_object_id_text_has_one_canonical_spelling
}

expect() { # expect <verdict> <claim> <output-file>
  if grep -qE "^$1 $2\$" "$3"; then
    echo "  ok: $2 is $1"
  else
    echo "  UNMET: expected $2 to be $1; got: $(grep -E "^(PASS|FAIL) $2\$" "$3" || echo '<no verdict>')"
    failures=$((failures + 1))
  fi
}

failures=0
GREEN=$(mktemp); RED=$(mktemp)

echo "### phase 1: unmutated -- every named claim must be green (the accepted positive control)"
run_phase > "$GREEN" 2>&1
for c in "${SUBJECTS[@]}"; do expect PASS "$c" "$GREEN"; done
expect PASS "$UNAFFECTED" "$GREEN"

echo "### phase 2: base16 digit 15 lowercase f -> F -- every subject claim must go RED"
python3 - "$ROWS" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = 'Base16DigitRow { value: 15, lower: "f", upper: "F" },'
new = 'Base16DigitRow { value: 15, lower: "F", upper: "F" },'
assert old in s, "the digit-15 row is not where this discriminator expects it"
open(p, "w").write(s.replace(old, new, 1))
PY
run_phase > "$RED" 2>&1
for c in "${SUBJECTS[@]}"; do expect FAIL "$c" "$RED"; done
expect PASS "$UNAFFECTED" "$RED"

rm -f "$GREEN" "$RED"
if [ "$failures" -ne 0 ]; then
  echo "### DISCRIMINATOR FAILED: $failures unmet expectation(s)"
  exit 1
fi
echo "### discriminator held: every subject claim flipped, the unaffected control stayed green"
