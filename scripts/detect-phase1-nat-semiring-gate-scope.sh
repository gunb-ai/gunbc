#!/usr/bin/env bash
# scripts/detect-phase1-nat-semiring-gate-scope.sh
#
# Host transport for phase1/nat_semiring rung gate CI gating (Gate #103: git diff
# selection must not live in .github/workflows/*.yml — see
# scripts/workflow-path-regex-forbidden-substrings.txt).
#
# Emits should_run=true|false to GITHUB_OUTPUT when set; otherwise prints to stdout.
#
# Usage:
#   detect-phase1-nat-semiring-gate-scope.sh <event_name> <output_file>
#   detect-phase1-nat-semiring-gate-scope.sh pull_request /dev/stdout

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

event_name="${1:-pull_request}"
output_file="${2:-}"

if [[ -z "$output_file" ]]; then
  echo "usage: detect-phase1-nat-semiring-gate-scope.sh <event_name> <output_file>" >&2
  exit 2
fi

range="origin/main...HEAD"
if [[ "$event_name" == "push" && "${GITHUB_REF:-}" == "refs/heads/main" ]]; then
  range="HEAD~1..HEAD"
fi

should_run=false
if git diff --name-only "$range" | grep -qE '^(src/v4/test/claim/(algebra_laws/nat_semiring|nat_semiring/)|scripts/v4-phase1-nat-semiring)'; then
  should_run=true
fi
# Workflow-policy gate wiring: any edit to the rung gate, its prerequisites, scope
# detector, or modeled ci.dag signal — including `if:` lines that route through
# phase1_nat_semiring_fixture_scope (fail-closed self-validation; do not filter those out).
if [[ "$should_run" != true ]] && git diff --name-only "$range" | grep -qE '^(\.github/workflows/ci\.yml|src/v4/workflow/ci\.dag|dsl/gunbc/ci_github_actions_workflow\.dag)'; then
  if git diff "$range" -- .github/workflows/ci.yml src/v4/workflow/ci.dag dsl/gunbc/ci_github_actions_workflow.dag 2>/dev/null \
    | grep '^[-+]' | grep -v '^[-+][[:space:]]*#' \
    | grep -qE 'phase1/nat_semiring rungs 0-2 gate|Setup Go \(phase1/nat_semiring|v4-phase1-nat-semiring-rung-gate\.sh|V4_PHASE1_NAT_SEMIRING_STRICT|phase1_nat_semiring_rung_gate|phase1_nat_semiring_fixture_scope|detect-phase1-nat-semiring-gate-scope'; then
    should_run=true
  fi
fi

{
  echo "should_run=${should_run}"
} >>"$output_file"
