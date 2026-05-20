#!/usr/bin/env bash
# scripts/v4-bootstrap-resolve-posture-gate.sh
#
# 🟡 gated — feature:v4-bootstrap-ci-resolve-posture-bridge — scaffold:TASKS T-8 / INVARIANTS §P2 host-process boundary
# Owner lane: T-8 closeout (CP-1b); operator-authorized CI bridge only.
# Dissolve-on-arrival: delete this script and the paired CI step when either (a) v2-compiler exposes
#   a typed resolve-only gate with its own exit code / structured receipt, or (b) v4 emit reaches
#   `compiled:` under ubicloud-standard-8 without host SIGTERM (TASKS T-22 / emit stability).
# Exit condition: removal when dissolution trigger (a) or (b) is met on main CI for 14 consecutive days.
#
# Separate gate from v4-bootstrap-viability.sh — does not reinterpret compile success.
# Requires V4_BOOTSTRAP_ALLOW_RESOLVE_POSTURE_BRIDGE=1 (set only in ci.yml bridge step).

set -euo pipefail

if [[ "${V4_BOOTSTRAP_ALLOW_RESOLVE_POSTURE_BRIDGE:-}" != "1" ]]; then
  echo "error: resolve-posture bridge refused (unset V4_BOOTSTRAP_ALLOW_RESOLVE_POSTURE_BRIDGE)" >&2
  exit 1
fi

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

log="${V4_BOOTSTRAP_LOG:-/tmp/v4-stage1.log}"
receipt="${V4_BOOTSTRAP_RECEIPT:-/tmp/v4-stage1/bootstrap-resolve-posture-receipt.json}"

if [[ ! -f "$log" ]]; then
  echo "error: compile log missing at $log (run v4-bootstrap-viability.sh first)" >&2
  exit 1
fi

compiler_exit="${V4_BOOTSTRAP_COMPILER_EXIT:-}"
if [[ -z "$compiler_exit" ]]; then
  echo "error: V4_BOOTSTRAP_COMPILER_EXIT required (compile step exit status)" >&2
  exit 1
fi

if [[ "$compiler_exit" != "124" && "$compiler_exit" != "143" ]]; then
  echo "error: resolve-posture bridge only applies after timeout/SIGTERM (got $compiler_exit)" >&2
  exit 1
fi

resolved_line_ok=false
if grep -qE '^resolved [0-9]+ sources \(transitive import closure\)$' "$log"; then
  resolved_line_ok=true
fi

error_count=0
if grep -qE '^error:' "$log"; then
  error_count=$(grep -cE '^error:' "$log" || true)
fi

mkdir -p "$(dirname "$receipt")"
# Single-authority structured receipt (consumers: this gate + CI artifact readers).
cat >"$receipt" <<EOF
{
  "version": 1,
  "gate": "v4-bootstrap-resolve-posture",
  "compiler_exit_status": ${compiler_exit},
  "resolved_line_ok": ${resolved_line_ok},
  "compiler_error_line_count": ${error_count}
}
EOF

if [[ "$resolved_line_ok" != "true" || "$error_count" -ne 0 ]]; then
  echo "error: resolve-posture receipt failed (see $receipt)" >&2
  cat "$receipt" >&2
  exit 1
fi

echo "::warning::v4 bootstrap resolve-posture bridge: compile exit ${compiler_exit}; full emit receipt deferred (dissolve-on-arrival per script header)." >&2
echo "Bootstrap resolve-posture OK — receipt $receipt"
exit 0
