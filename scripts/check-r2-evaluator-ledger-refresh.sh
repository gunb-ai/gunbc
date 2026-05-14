#!/usr/bin/env bash
set -euo pipefail

ledger="docs/r2-closure-ledger.md"

require_row_status() {
  local gate="$1"
  local expected="$2"
  local row

  row="$(grep -F "| \`${gate}\` |" "$ledger" || true)"
  if [[ -z "$row" ]]; then
    echo "missing R2 Evaluator ledger row for ${gate}" >&2
    exit 1
  fi

  IFS='|' read -r _ _ _ _ _ status _ _ <<<"$row"
  status="$(printf '%s' "$status" | xargs)"
  if [[ "$status" != "$expected" ]]; then
    echo "unexpected status for ${gate}: got '${status}', expected '${expected}'" >&2
    exit 1
  fi

  if [[ "$row" != *"HEAD refresh 2026-05-14"* ]]; then
    echo "R2 Evaluator ledger row for ${gate} lacks HEAD refresh marker" >&2
    exit 1
  fi
}

# Pinned matrix for the 2026-05-14 R3 Evaluator refresh. Any ratified
# intermediate or all-green transition must update these expectations in
# the same change that updates the ledger evidence.
require_row_status "runtime_value_model_structural" "green"
require_row_status "body_evaluator_structural" "green"
require_row_status "lens_application_complete_reflection" "in-flight"
require_row_status "witness_construction_structural" "in-flight"
require_row_status "cross_target_equivalence_harness_structural" "green"
