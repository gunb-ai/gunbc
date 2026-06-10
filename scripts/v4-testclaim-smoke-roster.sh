#!/usr/bin/env bash
# scripts/v4-testclaim-smoke-roster.sh
#
# Wave-A consolidated smoke-roster transport. Row authority lives in
# src/v4/test/claim/workflow/glob_discovery.dag
# (`glob_discovered_smoke_bool_witness_unified_claims` list); this shell projects
# distributed BoolWitnessClaim markers and invokes v2 claim-run.
#
# Env:
#   V2_COMPILER — gunbc binary (default: target/release/gunbc)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
discovery="src/v4/test/claim/workflow/glob_discovery.dag"
claims_root="src/v4/test/claim"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

dag_string_data() {
  local file="$1" name="$2"
  grep -E "^data ${name}: String = \"" "$root/$file" \
    | sed -n "s/^data ${name}: String = \"\\(.*\\)\"/\\1/p" \
    | head -1
}

claim_run() {
  local label="$1" entry="$2" function="$3"
  echo "::group::v4 smoke roster: ${label}"
  "$bin" run --source-root src/v4 --entry "$entry" --function "$function" --claim-run
  echo "::endgroup::"
}

list_claim_members() {
  awk '
    /data glob_discovered_smoke_bool_witness_unified_claims:/ { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  unified_claim_/ {
      gsub(/^  /, "")
      gsub(/,.*/, "")
      print
    }
  ' "$root/$discovery"
}

project_unified_claim_member() {
  local name="$1"
  local file
  file="$(grep -rl "^data ${name}: UnifiedTestClaim" "$root/$claims_root" | head -1)"
  if [[ -z "$file" ]]; then
    return 1
  fi
  local rel="${file#"$root"/}"
  awk -v n="$name" '
    $0 ~ "^data " n ": UnifiedTestClaim" { in_row = 1; entry = ""; fn = "" }
    in_row && /entry: "/ {
      sub(/.*entry: "/, "")
      sub(/".*/, "")
      entry = $0
    }
    in_row && /function: / {
      sub(/.*function: /, "")
      sub(/[[:space:]].*/, "")
      fn = $0
    }
    in_row && /\}/ {
      if (entry != "" && fn != "") {
        print entry "\t" fn
      }
      in_row = 0
    }
  ' "$file"
}

"$bin" run \
  --source-root src/v4 \
  --entry "$discovery" \
  --function witness_glob_discovery_declares_smoke_claim_run_rows \
  --claim-run

expected_count="$(dag_string_data "$discovery" glob_discovered_claim_run_row_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing glob_discovered_claim_run_row_count in $discovery" >&2
  exit 2
fi

row_count=0
while IFS= read -r member; do
  [[ -z "$member" ]] && continue
  row="$(project_unified_claim_member "$member")"
  if [[ -z "$row" ]]; then
    echo "error: list member $member missing BoolWitnessClaim binding under $claims_root" >&2
    exit 2
  fi
  IFS=$'\t' read -r entry function <<< "$row"
  label="${member#unified_claim_}"
  claim_run "$label" "$entry" "$function"
  row_count=$((row_count + 1))
done < <(list_claim_members)

if [[ "$row_count" -eq 0 ]]; then
  echo "error: glob_discovered_smoke_bool_witness_unified_claims has no members in $discovery" >&2
  exit 2
fi

if [[ "$row_count" -ne "$expected_count" ]]; then
  echo "error: smoke roster transport projected ${row_count} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

echo "::notice title=v4 smoke roster::${row_count} claim-run witness(es) passed"
