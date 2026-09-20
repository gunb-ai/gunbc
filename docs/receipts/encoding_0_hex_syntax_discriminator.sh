#!/usr/bin/env bash
# ENCODING-0 hex-syntax consolidation: the discriminating RED.
#
# The four consumers no longer carry their own copy of the 0-9a-f alphabet; they read
# extdeps.numeric.base16's digit rows. This script proves that by MUTATING THE ROWS -- the
# lowercase spelling of digit 15 becomes "F" -- and running the consumers' own witnesses. Each
# named claim must go RED. A consumer that had kept a private code-point range would stay green,
# which is exactly the bypass this cut deletes.
#
# The mutation is applied to the working tree and reverted at the end (and on failure).
set -euo pipefail
: "${CLAIM_BATCH:=target/release/claim_batch}"
ROWS=dag/extdeps/numeric/base16.dag
BACKUP=$(mktemp)
cp "$ROWS" "$BACKUP"
restore() { cp "$BACKUP" "$ROWS"; rm -f "$BACKUP"; }
trap restore EXIT

python3 - "$ROWS" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = 'Base16DigitRow { value: 15, lower: "f", upper: "F" },'
new = 'Base16DigitRow { value: 15, lower: "F", upper: "F" },'
assert old in s, "the digit-15 row is not where this discriminator expects it"
open(p, "w").write(s.replace(old, new, 1))
PY

echo "### mutated: base16 digit 15 lower spelling f -> F; every claim below must go RED"
set +e
"$CLAIM_BATCH" --claim-run --hermetic \
  --source-root dag --source-root src/v2 \
  --entry dag/test/claim/base16_rfc4648_witness_test.dag --functions the_lowercase_alphabet_is_exactly_the_digit_rows,a_digit_value_is_read_from_the_same_rows,base16_encodes_rfc4648_foobar_vector \
  --entry dag/test/claim/approval_decision_store_witness_test.dag --functions keyring_loads_sixty_four_lower_hex_and_refuses_the_rest \
  --entry dag/test/claim/network_mac_witness_test.dag --functions $(grep -oE '^test fn [a-z0-9_]+' dag/test/claim/network_mac_witness_test.dag | awk '{print $3}' | paste -sd,) \
  --entry dag/test/claim/git_upstream_model_witness_test.dag --functions $(grep -oE '^test fn [a-z0-9_]+' dag/test/claim/git_upstream_model_witness_test.dag | awk '{print $3}' | paste -sd,)
echo "### claim_batch exit: $? (expected nonzero)"
