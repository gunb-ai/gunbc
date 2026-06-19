#!/usr/bin/env bash
# scripts/lib/witness_layer_roots.sh — shell projection of gunbc.ci_layer_roots.witness_layer_roots (§3).
#
# Authority: dsl/gunbc/ci_layer_roots.dag (do not fork root order here).
# DISSOLUTION: delete when self-host closure hosts read witness_layer_roots at runtime
#   (see v2.compiler.self_host.closure_witness_layer_roots + ci_floor_plan).

set -euo pipefail

witness_layer_roots_authority_relpath() {
  echo "dsl/gunbc/ci_layer_roots.dag"
}

# Prints one repo-relative root per line (e.g. dsl, src/v2).
witness_layer_roots_from_authority() {
  local repo_root="${1:?repo root required}"
  local auth="${repo_root}/$(witness_layer_roots_authority_relpath)"
  if [[ ! -f "$auth" ]]; then
    echo "witness_layer_roots: missing authority ${auth}" >&2
    return 1
  fi
  local line
  line="$(grep -E '^data witness_layer_roots:' "$auth" | head -1)"
  if [[ -z "$line" ]]; then
    echo "witness_layer_roots: row not found in ${auth}" >&2
    return 1
  fi
  local inner="${line#*=[}"
  inner="${inner%]}"
  inner="${inner#*[}"
  local part
  IFS=',' read -ra parts <<<"$inner"
  for part in "${parts[@]}"; do
    part="${part//\"/}"
    part="${part// /}"
    if [[ -n "$part" ]]; then
      echo "$part"
    fi
  done
}

# Populates WITNESS_LAYER_ROOTS (repo-relative names) in caller scope.
witness_layer_roots_load() {
  local repo_root="${1:?repo root required}"
  WITNESS_LAYER_ROOTS=()
  local r
  while IFS= read -r r; do
    WITNESS_LAYER_ROOTS+=("$r")
  done < <(witness_layer_roots_from_authority "$repo_root")
}
