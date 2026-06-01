#!/usr/bin/env bash
# Wave 1 §11.7.1 gate 5: fail-closed ratchet — no new shell steps on the required CI path.
# Authority: docs/planning/ci-required-surface-cut-2026-06-01.md + project_no_new_shell.
#
# Scans .github/workflows/ci.yml for every scripts/<name>.sh reference (any shell
# spelling: bash/sh/./-c). Only the three Wave-1 allowlisted transports may appear.
#
# Usage: ./scripts/check-ci-no-new-shell.sh [--self-test]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

CI_YML=".github/workflows/ci.yml"

allowed=(
  "scripts/v4-m1-rust-emit-probe.sh"
  "scripts/v4-bootstrap-viability.sh"
  "scripts/check-ci-no-new-shell.sh"
)

is_allowlisted() {
  local script="$1"
  local a
  for a in "${allowed[@]}"; do
    if [[ "$script" == "$a" ]]; then
      return 0
    fi
  done
  return 1
}

# Every scripts/<name>.sh token on a line (all occurrences).
scripts_on_line() {
  local line="$1"
  local rest="$line"
  local token
  while [[ "$rest" =~ scripts/[a-zA-Z0-9_.-]+ ]]; do
    token="${BASH_REMATCH[0]}"
    printf '%s\n' "$token"
    rest="${rest#*"$token"}"
  done
}

scan_lines() {
  local line script
  violations=0
  while IFS= read -r line; do
    if [[ ! "$line" =~ scripts/ ]]; then
      continue
    fi
    while IFS= read -r script; do
      [[ -z "$script" ]] && continue
      if ! is_allowlisted "$script"; then
        echo "error: disallowed scripts/ reference on required CI path: $script" >&2
        echo "       line: $line" >&2
        violations=$((violations + 1))
      fi
    done < <(scripts_on_line "$line")
  done
}

scan_ci_yml() {
  scan_lines < "$CI_YML"
}

self_test_scan_must_fail() {
  local label="$1"
  local probe="$2"
  local before=$violations
  scan_lines <<< "$probe"
  if [[ "$violations" -le "$before" ]]; then
    echo "check-ci-no-new-shell: self-test failed ($label — expected violation)" >&2
    exit 1
  fi
  violations=$before
}

if [[ "${1:-}" == "--self-test" ]]; then
  violations=0
  # Multi-token on one line (prior bypass).
  probe='run: bash scripts/v4-m1-rust-emit-probe.sh && bash scripts/evil-bypass.sh'
  count=0
  while IFS= read -r script; do
    [[ -z "$script" ]] && continue
    count=$((count + 1))
  done < <(scripts_on_line "$probe")
  if [[ "$count" -ne 2 ]]; then
    echo "check-ci-no-new-shell: self-test failed (expected 2 scripts on probe line, got $count)" >&2
    exit 1
  fi
  self_test_scan_must_fail "sh scripts/" 'run: sh scripts/evil-bypass.sh'
  self_test_scan_must_fail "./scripts/" 'run: ./scripts/evil-bypass.sh'
  self_test_scan_must_fail "bash ./scripts/" 'run: bash ./scripts/evil-bypass.sh'
  self_test_scan_must_fail "bash -c scripts/" "run: bash -c 'scripts/evil-bypass.sh'"
  self_test_scan_must_fail "bash -e scripts/" 'run: bash -e scripts/evil-bypass.sh'
  echo "check-ci-no-new-shell: self-test ok"
  exit 0
fi

if [[ ! -f "$CI_YML" ]]; then
  echo "error: missing $CI_YML" >&2
  exit 1
fi

violations=0
scan_ci_yml

if (( violations > 0 )); then
  echo "check-ci-no-new-shell: failed ($violations violation(s))" >&2
  echo "allowed: ${allowed[*]}" >&2
  exit 1
fi

echo "check-ci-no-new-shell: ok (${#allowed[@]} scripts allowlisted)"
