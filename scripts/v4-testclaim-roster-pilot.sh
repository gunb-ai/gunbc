#!/usr/bin/env bash
# scripts/v4-testclaim-roster-pilot.sh
#
# Wave-A roster pilot transport. Authority for promoted rows lives in
# src/v4/test/claim/workflow/v4_roster_pilot.dag; this shell only reads the
# modeled entry/function names and invokes v2 claim-run.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
roster="src/v4/test/claim/workflow/v4_roster_pilot.dag"

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
  echo "::group::v4 roster pilot: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

roster_entry="$(dag_string_data v4_roster_pilot_edit_locus_entry)"
if [[ -z "$roster_entry" ]]; then
  echo "error: missing v4_roster_pilot_edit_locus_entry in $roster" >&2
  exit 2
fi

"$bin" run \
  --source-root src/v4 \
  --entry "$roster" \
  --function witness_v4_roster_pilot_declares_edit_locus_rows \
  --claim-run

claim_run \
  "edit-locus narrow resolution" \
  "$roster_entry" \
  "$(dag_string_data v4_roster_pilot_edit_locus_narrow_resolution_fn)"
claim_run \
  "edit-locus fail closed" \
  "$roster_entry" \
  "$(dag_string_data v4_roster_pilot_edit_locus_fail_closed_fn)"

# Exemplar E1: v4.std.text carrier witnesses (smoke→witness migration).
std_text_entry="$(dag_string_data v4_roster_pilot_std_text_entry)"
if [[ -z "$std_text_entry" ]]; then
  echo "error: missing v4_roster_pilot_std_text_entry in $roster" >&2
  exit 2
fi

"$bin" run \
  --source-root src/v4 \
  --entry "$roster" \
  --function witness_v4_roster_pilot_declares_std_text_rows \
  --claim-run

claim_run \
  "std_text String FreeMonoid<Char> carrier" \
  "$std_text_entry" \
  "$(dag_string_data v4_roster_pilot_std_text_freemonoid_carrier_fn)"
claim_run \
  "std_text HostStringText constructor/projection round-trip" \
  "$std_text_entry" \
  "$(dag_string_data v4_roster_pilot_std_text_roundtrip_fn)"
claim_run \
  "std_text HostStringText empty round-trip (projection returns stored text)" \
  "$std_text_entry" \
  "$(dag_string_data v4_roster_pilot_std_text_empty_roundtrip_fn)"
claim_run \
  "std_text HostStringText record field text:String" \
  "$std_text_entry" \
  "$(dag_string_data v4_roster_pilot_std_text_field_text_fn)"

echo "::notice title=v4 roster pilot::edit_locus + std_text witnesses passed"
