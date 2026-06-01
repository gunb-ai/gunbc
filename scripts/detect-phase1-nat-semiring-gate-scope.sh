#!/usr/bin/env bash
# scripts/detect-phase1-nat-semiring-gate-scope.sh
#
# Host transport for phase1/nat_semiring rung gate CI gating (Gate #103: git diff
# selection must not live in .github/workflows/*.yml — see
# scripts/workflow-path-regex-forbidden-substrings.txt).
#
# Emits to GITHUB_OUTPUT (or the given file):
#   should_run=true|false  — run the full fixture-scoped rung gate (emit + rustc/…)
#   policy_check=true|false — run the lightweight workflow-policy ratchet only
#
# Full gate: fixture paths, or workflow edits to gate *execution* (script invoke,
# STRICT env, modeled ci.dag rung signal). Scope-detector / if-routing narrowing
# alone triggers policy_check (Gate #103 inventory + detector smoke), not the
# substrate-red rung gate (ladder §6 appendix).
#
# Usage:
#   detect-phase1-nat-semiring-gate-scope.sh <event_name> <output_file>

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
policy_check=false

if git diff --name-only "$range" | grep -qE '^(src/v4/test/claim/(algebra_laws/nat_semiring|nat_semiring/)|scripts/v4-phase1-nat-semiring-rung-gate\.sh|scripts/v4-phase1-nat-semiring-python-runtime-gate\.sh)'; then
  should_run=true
fi

workflow_paths='^(\.github/workflows/ci\.yml|src/v4/workflow/ci\.dag|dsl/gunbc/ci_github_actions_workflow\.dag|scripts/detect-phase1-nat-semiring-gate-scope\.sh)'
if git diff --name-only "$range" | grep -qE "$workflow_paths"; then
  policy_check=true
  if [[ "$should_run" != true ]]; then
    if git diff "$range" -- .github/workflows/ci.yml src/v4/workflow/ci.dag dsl/gunbc/ci_github_actions_workflow.dag 2>/dev/null \
      | grep '^[-+]' | grep -v '^[-+][[:space:]]*#' \
      | grep -qE 'v4-phase1-nat-semiring-rung-gate\.sh|v4-phase1-nat-semiring-python-runtime-gate\.sh|V4_PHASE1_NAT_SEMIRING_STRICT|V4_PHASE1_NAT_SEMIRING_PYTHON_RUNTIME_STRICT|phase1_nat_semiring_rung_gate|phase1_nat_semiring_python_runtime'; then
      should_run=true
    fi
  fi
fi

{
  echo "should_run=${should_run}"
  echo "policy_check=${policy_check}"
} >>"$output_file"
