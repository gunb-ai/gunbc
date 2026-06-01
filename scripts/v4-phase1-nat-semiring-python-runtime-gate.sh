#!/usr/bin/env bash
# scripts/v4-phase1-nat-semiring-python-runtime-gate.sh
#
# L1 Python runtime receipt for fixture=phase1/nat_semiring: after py_compile (rung gate),
# execute each emitted .py under the fixture emit tree and record typed host verdicts.
#
# Authority: docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md Worksheet B
# (PY-L1-L2-RUNTIME-FIXTURE-EXECUTION); complements scripts/v4-phase1-nat-semiring-rung-gate.sh
# (py_compile-only for R0/R2). Cross-target MVP-2 stdout parity (L2) is proven via
# emit_host_bridge integration tests until per-law emit lands.
#
# Env:
#   V4_PHASE1_NAT_SEMIRING_OUT — emit output dir (same as rung gate; required)
#   V4_PHASE1_NAT_SEMIRING_PYTHON — python3 binary (default: python3)
#   V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS — per-invocation timeout (default: 300)
#   V4_PHASE1_NAT_SEMIRING_PYTHON_RUNTIME_STRICT — exit 1 on FAIL when 1

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_id="phase1/nat_semiring"
python_bin="${V4_PHASE1_NAT_SEMIRING_PYTHON:-python3}"
timeout_secs="${V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS:-300}"
strict="${V4_PHASE1_NAT_SEMIRING_PYTHON_RUNTIME_STRICT:-0}"

if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_PHASE1_NAT_SEMIRING_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-phase1-nat-semiring"
else
  out="${V4_PHASE1_NAT_SEMIRING_OUT:-}"
fi

if [[ -z "$out" ]]; then
  echo "error: V4_PHASE1_NAT_SEMIRING_OUT must point at the rung-gate emit tree" >&2
  exit 2
fi

py_tree="$out/python"
summary="${out}.python-runtime-gate-summary.txt"

if [[ ! -d "$py_tree" ]]; then
  echo "error: python emit tree missing at $py_tree (run v4-phase1-nat-semiring-rung-gate.sh first)" >&2
  exit 2
fi

if ! command -v "$python_bin" >/dev/null 2>&1; then
  echo "error: python interpreter not found: $python_bin" >&2
  if [[ "$strict" == "1" ]]; then
    exit 2
  fi
  exit 0
fi

if [[ "$timeout_secs" -gt 0 ]]; then
  timed=(timeout --preserve-status "$timeout_secs")
else
  timed=()
fi

declare -A verdict=(
  [R-L1-python-runtime-exec]=SKIP
)
blocking_receipt="none"

note_blocking() {
  local pred="$1"
  if [[ "$blocking_receipt" == "none" ]]; then
    blocking_receipt="$pred"
  fi
}

py_files=()
while IFS= read -r -d '' f; do
  py_files+=("$f")
done < <(find "$py_tree" -name '*.py' -print0 2>/dev/null || true)

if [[ "${#py_files[@]}" -eq 0 ]]; then
  verdict[R-L1-python-runtime-exec]=SKIP
  note_blocking "phase1/nat_semiring/l1/python_emit_unavailable"
else
  mkdir -p "$out/logs"
  runtime_ok=1
  for py in "${py_files[@]}"; do
    # Path-relative slug avoids log collisions when emit fans out per-law subdirs.
    rel="${py#"$py_tree"/}"
    rel="${rel#/}"
    if [[ -n "$rel" ]]; then
      log_slug="${rel//\//__}"
    else
      log_slug="$(basename "$py")"
    fi
    log="$out/logs/python_runtime_${log_slug}.log"
    set +e
    "${timed[@]}" "$python_bin" "$py" >"$log" 2>&1
    status=$?
    set -e
    if [[ "$status" -ne 0 ]]; then
      runtime_ok=0
      echo "FAIL $py exit=$status (see $log)" >>"$summary.partial" 2>/dev/null || true
    fi
  done
  rm -f "$summary.partial" 2>/dev/null || true
  if [[ "$runtime_ok" -eq 1 ]]; then
    verdict[R-L1-python-runtime-exec]=PASS
  else
    verdict[R-L1-python-runtime-exec]=FAIL
    note_blocking "phase1/nat_semiring/l1/python_runtime_exec_rejected"
  fi
fi

row_pass="FAIL"
if [[ "${verdict[R-L1-python-runtime-exec]}" == "PASS" ]]; then
  row_pass="PASS"
fi

{
  echo "fixture=${fixture_id}"
  echo "  l1_python_runtime: ${row_pass}  (python=${verdict[R-L1-python-runtime-exec]})"
  echo "blocking_receipt: ${blocking_receipt}"
  echo ""
  echo "emit_tree: ${py_tree}"
  echo "logs: ${out}/logs/"
} | tee "$summary"

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  body="$(head -6 "$summary")"
  escaped="${body//$'\n'/%0A}"
  echo "::notice title=phase1/nat_semiring python runtime gate::${escaped}"
fi

if [[ "$strict" == "1" && "$row_pass" != "PASS" ]]; then
  exit 1
fi

exit 0
