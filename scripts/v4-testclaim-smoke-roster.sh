#!/usr/bin/env bash
# scripts/v4-testclaim-smoke-roster.sh
#
# Wave-A consolidated smoke-roster transport. Row authority is the mechanical
# projection of distributed top-level BoolWitnessClaim markers under
# src/v4/test/claim/ (scripts/v4-glob-discovery-project.sh) — not a central list.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
law_model="src/v4/test/claim/workflow/glob_discovery.dag"
project_sh="$root/scripts/v4-glob-discovery-project.sh"

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc (v2 stage0 binary) not found at $bin" >&2
  exit 2
fi

# shellcheck source=scripts/v4-glob-discovery-project.sh
source "$project_sh"

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

echo "::group::v4 smoke roster: glob discovery cardinality law"
"$bin" run \
  --source-root src/v4 \
  --entry "$law_model" \
  --function witness_glob_discovery_smoke_marker_count_is_positive \
  --claim-run
echo "::endgroup::"

v4_glob_discovery_project_distributed_markers

expected_count="$(dag_string_data "$law_model" glob_discovered_smoke_marker_count)"
if [[ -z "$expected_count" ]]; then
  echo "error: missing glob_discovered_smoke_marker_count in $law_model" >&2
  exit 2
fi

if [[ "$V4_GLOB_DISCOVERY_ROW_COUNT" -ne "$expected_count" ]]; then
  echo "error: discovery projected ${V4_GLOB_DISCOVERY_ROW_COUNT} rows; modeled count is ${expected_count}" >&2
  exit 2
fi

while IFS=$'\t' read -r label entry function; do
  [[ -z "$label" ]] && continue
  claim_run "$label" "$entry" "$function"
done < <(printf '%s' "$V4_GLOB_DISCOVERY_ROWS")

echo "::notice title=v4 smoke roster::${V4_GLOB_DISCOVERY_ROW_COUNT} claim-run witness(es) passed"
