#!/usr/bin/env bash
# Wave 1 §11.7.1 gate 5: fail-closed ratchet — no new shell steps on the required CI path.
# Authority: docs/planning/ci-required-surface-cut-2026-06-01.md + project_no_new_shell.
#
# Allowed `bash scripts/...` invocations in .github/workflows/ci.yml (required floor only):
#   - v4-m1-rust-emit-probe.sh
#   - v4-bootstrap-viability.sh
#   - check-ci-no-new-shell.sh (self)
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

if [[ "${1:-}" == "--self-test" ]]; then
  echo "check-ci-no-new-shell: self-test ok"
  exit 0
fi

if [[ ! -f "$CI_YML" ]]; then
  echo "error: missing $CI_YML" >&2
  exit 1
fi

violations=0
while IFS= read -r line; do
  if [[ ! "$line" =~ bash[[:space:]]+scripts/ ]]; then
    continue
  fi
  script=""
  if [[ "$line" =~ scripts/[a-zA-Z0-9_.-]+ ]]; then
    script="${BASH_REMATCH[0]}"
  fi
  [[ -z "$script" ]] && continue
  ok=0
  for a in "${allowed[@]}"; do
    if [[ "$script" == "$a" ]]; then
      ok=1
      break
    fi
  done
  if [[ "$ok" -eq 0 ]]; then
    echo "error: disallowed shell on required CI path: $script" >&2
    echo "       line: $line" >&2
    violations=$((violations + 1))
  fi
done < <(grep -E 'run:.*bash scripts/|bash scripts/' "$CI_YML" || true)

if (( violations > 0 )); then
  echo "check-ci-no-new-shell: failed ($violations violation(s))" >&2
  echo "allowed: ${allowed[*]}" >&2
  exit 1
fi

echo "check-ci-no-new-shell: ok (${#allowed[@]} scripts allowlisted)"
