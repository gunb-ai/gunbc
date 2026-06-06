#!/usr/bin/env bash
# scripts/v4-testclaim-smoke-roster.sh
#
# Wave-A consolidated smoke-roster transport. Row authority lives in
# src/v4/test/claim/workflow/v4_roster_pilot.dag (list-based); this shell only
# projects modeled V4RosterPilotClaimRunRow blocks and invokes v2 claim-run.
#
# Env:
#   V2_COMPILER — gunbc binary (default: target/release/gunbc)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
roster="src/v4/test/claim/workflow/v4_roster_pilot.dag"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

claim_run() {
  local label="$1" entry="$2" function="$3"
  echo "::group::v4 smoke roster: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

# Project each V4RosterPilotClaimRunRow { label, entry, function } block from the roster DAG.
parse_claim_run_rows() {
  awk '
    /V4RosterPilotClaimRunRow \{/ { in_row = 1; label = ""; entry = ""; fn = "" }
    in_row && /label:/ {
      sub(/.*label: "/, "")
      sub(/".*/, "")
      label = $0
    }
    in_row && /entry:/ {
      sub(/.*entry: "/, "")
      sub(/".*/, "")
      entry = $0
    }
    in_row && /function:/ {
      sub(/.*function: "/, "")
      sub(/".*/, "")
      fn = $0
    }
    in_row && /\}/ {
      if (label != "" && entry != "" && fn != "") {
        print label "\t" entry "\t" fn
      }
      in_row = 0
    }
  ' "$root/$roster"
}

"$bin" run \
  --source-root src/v4 \
  --entry "$roster" \
  --function witness_v4_roster_pilot_declares_claim_run_rows \
  --claim-run

row_count=0
while IFS=$'\t' read -r label entry function; do
  [[ -z "$label" || -z "$entry" || -z "$function" ]] && continue
  claim_run "$label" "$entry" "$function"
  row_count=$((row_count + 1))
done < <(parse_claim_run_rows)

if [[ "$row_count" -eq 0 ]]; then
  echo "error: no V4RosterPilotClaimRunRow blocks found in $roster" >&2
  exit 2
fi

echo "::notice title=v4 smoke roster::${row_count} claim-run witness(es) passed"
