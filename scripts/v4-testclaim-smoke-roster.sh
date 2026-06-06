#!/usr/bin/env bash
# scripts/v4-testclaim-smoke-roster.sh
#
# Wave-A consolidated smoke-roster transport. Row authority lives in
# src/v4/test/claim/workflow/v4_roster_pilot.dag (`v4_roster_pilot_claim_run_rows` list);
# this shell projects only list member bindings and invokes v2 claim-run.
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

dag_string_data() {
  local name="$1"
  grep -E "^data ${name}: String = \"" "$root/$roster" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

claim_run() {
  local label="$1" entry="$2" function="$3"
  echo "::group::v4 smoke roster: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

# List member names from `v4_roster_pilot_claim_run_rows` authority (not free file scan).
list_claim_run_row_members() {
  awk '
    /data v4_roster_pilot_claim_run_rows:/ { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  v4_roster_pilot_row_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$roster"
}

# Project one list member binding: `data <name>: V4RosterPilotClaimRunRow = V4RosterPilotClaimRunRow { ... }`.
project_list_member_row() {
  local name="$1"
  awk -v n="$name" '
    $0 ~ "^data " n ": V4RosterPilotClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = "" }
    in_row && /label: "/ {
      sub(/.*label: "/, "")
      sub(/".*/, "")
      label = $0
    }
    in_row && /entry: "/ {
      sub(/.*entry: "/, "")
      sub(/".*/, "")
      entry = $0
    }
    in_row && /function: "/ {
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

expected_count="$(dag_string_data v4_roster_pilot_claim_run_row_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing v4_roster_pilot_claim_run_row_count in $roster" >&2
  exit 2
fi

row_count=0
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_list_member_row "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing V4RosterPilotClaimRunRow binding in $roster" >&2
    exit 2
  fi
  IFS=$'\t' read -r label entry function <<< "$row"
  claim_run "$label" "$entry" "$function"
  row_count=$((row_count + 1))
done < <(list_claim_run_row_members)

if [[ "$row_count" -eq 0 ]]; then
  echo "error: v4_roster_pilot_claim_run_rows has no members in $roster" >&2
  exit 2
fi

if [[ "$row_count" -ne "$expected_count" ]]; then
  echo "error: smoke roster transport projected ${row_count} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

echo "::notice title=v4 smoke roster::${row_count} claim-run witness(es) passed"
