#!/usr/bin/env bash
# scripts/v4-testclaim-grounding-typescript-pilot.sh
#
# Wave-A grounding_typescript partial additive transport. Authority for named test rows lives in
# src/v4/test/claim/workflow/v4_grounding_typescript_pilot.dag; this shell only reads
# the modeled entry/function names and invokes v2 claim-run.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
roster="src/v4/test/claim/workflow/v4_grounding_typescript_pilot.dag"

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
  echo "::group::v4 grounding_typescript pilot: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

roster_entry="$(dag_string_data v4_grounding_typescript_pilot_entry)"
if [[ -z "$roster_entry" ]]; then
  echo "error: missing v4_grounding_typescript_pilot_entry in $roster" >&2
  exit 2
fi

"$bin" run \
  --source-root src/v4 \
  --entry "$roster" \
  --function witness_v4_grounding_typescript_pilot_declares_rows \
  --claim-run

claim_run \
  "SG-1 symbol carrier" \
  "$roster_entry" \
  "$(dag_string_data v4_grounding_typescript_pilot_symbol_carrier_fn)"
claim_run \
  "SG-5 absence fail-closed" \
  "$roster_entry" \
  "$(dag_string_data v4_grounding_typescript_pilot_absence_fail_closed_fn)"

echo "::notice title=v4 grounding_typescript pilot::discriminating claim-run witnesses passed"
