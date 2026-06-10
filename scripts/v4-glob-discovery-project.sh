#!/usr/bin/env bash
# scripts/v4-glob-discovery-project.sh
#
# Mechanical projection of distributed top-level BoolWitnessClaim markers:
#   data unified_claim_*: UnifiedTestClaim = BoolWitnessClaim { witness: ... }
# co-located in claim corpus modules (owned declaration — not a central roster list).
#
# Sourced by smoke-roster and substrate-equivalence gates. When executed directly,
# prints TSV rows: label<TAB>entry<TAB>function

set -euo pipefail

v4_glob_discovery_default_claims_root() {
  local root="${1:-}"
  if [[ -n "$root" ]]; then
    printf '%s/test/claim' "$root"
    return
  fi
  local repo
  repo="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
  printf '%s/src/v4/test/claim' "$repo"
}

project_marker_file() {
  local file="$1"
  awk -v path="$file" '
    /^data unified_claim_[A-Za-z0-9_]+: UnifiedTestClaim = BoolWitnessClaim/ {
      label = $2
      sub(/:.*/, "", label)
      in_marker = 1
      entry = ""
      fn = ""
    }
    in_marker && /entry: "/ {
      sub(/.*entry: "/, "")
      sub(/".*/, "")
      entry = $0
    }
    in_marker && /function: / {
      sub(/.*function: /, "")
      sub(/[[:space:]].*/, "")
      fn = $0
    }
    in_marker && /\}/ {
      if (entry != "" && fn != "") {
        sub(/^unified_claim_/, "", label)
        print label "\t" entry "\t" fn
      }
      in_marker = 0
    }
  ' "$file"
}

# Args: [claims_root]
# Sets: V4_GLOB_DISCOVERY_ROWS (newline-delimited TSV), V4_GLOB_DISCOVERY_ROW_COUNT
# Exits 2 when projection is empty (fail-closed).
v4_glob_discovery_project_distributed_markers() {
  local claims_root
  claims_root="$(v4_glob_discovery_default_claims_root "${1:-}")"
  V4_GLOB_DISCOVERY_ROWS=""
  V4_GLOB_DISCOVERY_ROW_COUNT=0

  local file
  while IFS= read -r file; do
    [[ -z "$file" ]] && continue
    [[ "$file" == *"/impossible_bug/"* ]] && continue
    while IFS=$'\t' read -r label entry function; do
      [[ -z "$label" ]] && continue
      V4_GLOB_DISCOVERY_ROWS+="${label}"$'\t'"${entry}"$'\t'"${function}"$'\n'
      V4_GLOB_DISCOVERY_ROW_COUNT=$((V4_GLOB_DISCOVERY_ROW_COUNT + 1))
    done < <(project_marker_file "$file")
  done < <(
    find "$claims_root" -type f -name '*.dag' -print \
      | LC_ALL=C sort \
      | while IFS= read -r candidate; do
          grep -qE '^data unified_claim_[A-Za-z0-9_]+: UnifiedTestClaim = BoolWitnessClaim' "$candidate" \
            && printf '%s\n' "$candidate"
        done
  )

  if [[ "$V4_GLOB_DISCOVERY_ROW_COUNT" -eq 0 ]]; then
    echo "error: glob discovery projection is empty under ${claims_root}" >&2
    return 2
  fi
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  v4_glob_discovery_project_distributed_markers "${1:-}"
  printf '%s' "$V4_GLOB_DISCOVERY_ROWS"
fi
