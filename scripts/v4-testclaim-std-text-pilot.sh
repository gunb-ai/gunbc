#!/usr/bin/env bash
# scripts/v4-testclaim-std-text-pilot.sh
#
# Exemplar E1 v4.std.text smoke→witness migration transport. Authority for named test rows lives
# in src/v4/test/claim/workflow/v4_std_text_pilot.dag; this shell only reads the modeled
# entry/function names and invokes v2 claim-run.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
roster="src/v4/test/claim/workflow/v4_std_text_pilot.dag"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$roster" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

claim_run() {
  local label="$1" entry="$2" function="$3"
  echo "::group::v4 std_text pilot: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

roster_entry="$(dag_string_data v4_std_text_pilot_entry)"
if [[ -z "$roster_entry" ]]; then
  echo "error: missing v4_std_text_pilot_entry in $roster" >&2
  exit 2
fi

"$bin" run \
  --source-root src/v4 \
  --entry "$roster" \
  --function witness_v4_std_text_pilot_declares_rows \
  --claim-run

claim_run \
  "String FreeMonoid<Char> carrier" \
  "$roster_entry" \
  "$(dag_string_data v4_std_text_pilot_freemonoid_carrier_fn)"
claim_run \
  "HostStringText constructor/projection round-trip" \
  "$roster_entry" \
  "$(dag_string_data v4_std_text_pilot_roundtrip_fn)"
claim_run \
  "HostStringText empty round-trip (projection returns stored text)" \
  "$roster_entry" \
  "$(dag_string_data v4_std_text_pilot_empty_roundtrip_fn)"
claim_run \
  "HostStringText record field text:String" \
  "$roster_entry" \
  "$(dag_string_data v4_std_text_pilot_field_text_fn)"

echo "::notice title=v4 std_text pilot::discriminating claim-run witnesses passed"
