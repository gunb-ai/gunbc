#!/usr/bin/env bash
# scripts/v4-phase1-nat-semiring-python-static-gate.sh
#
# L1 Python static structural receipts for fixture=phase1/nat_semiring: after
# py_compile (rung gate R2), run pyright and mypy on the emitted Python tree.
# Distinct authorities from CPython compile and from python3 execution.
#
# Authority: docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md Worksheet A
# (PY-L1-STATIC-STRUCTURAL); modeled profiles in src/v4/extdeps/typecheckers/{pyright,mypy}.dag.
#
# Env:
#   V4_PHASE1_NAT_SEMIRING_OUT — emit output dir (same as rung gate; required)
#   V4_PHASE1_NAT_SEMIRING_PYRIGHT — pyright binary (default: pyright)
#   V4_PHASE1_NAT_SEMIRING_MYPY — mypy binary (default: mypy)
#   V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS — per-invocation timeout (default: 300)
#   V4_PHASE1_NAT_SEMIRING_PYTHON_STATIC_STRICT — exit 1 on FAIL when 1 (advisory
#     when 0: SKIP if tool missing; FAIL only when tool ran and reported errors)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_id="phase1/nat_semiring"
pyright_bin="${V4_PHASE1_NAT_SEMIRING_PYRIGHT:-pyright}"
mypy_bin="${V4_PHASE1_NAT_SEMIRING_MYPY:-mypy}"
timeout_secs="${V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS:-300}"
strict="${V4_PHASE1_NAT_SEMIRING_PYTHON_STATIC_STRICT:-0}"

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
summary="${out}.python-static-gate-summary.txt"

if [[ ! -d "$py_tree" ]]; then
  echo "error: python emit tree missing at $py_tree (run v4-phase1-nat-semiring-rung-gate.sh first)" >&2
  exit 2
fi

if [[ "$timeout_secs" -gt 0 ]]; then
  timed=(timeout --preserve-status "$timeout_secs")
else
  timed=()
fi

declare -A verdict=(
  [R-L1-pyright-static]=SKIP
  [R-L1-mypy-static]=SKIP
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

mkdir -p "$out/logs"

run_pyright() {
  if ! command -v "$pyright_bin" >/dev/null 2>&1; then
    verdict[R-L1-pyright-static]=SKIP
    note_blocking "phase1/nat_semiring/l1/pyright_unavailable"
    return
  fi
  if [[ "${#py_files[@]}" -eq 0 ]]; then
    verdict[R-L1-pyright-static]=SKIP
    note_blocking "phase1/nat_semiring/l1/python_emit_unavailable"
    return
  fi
  local log="$out/logs/python_static_pyright.log"
  set +e
  "${timed[@]}" "$pyright_bin" "$py_tree" >"$log" 2>&1
  local status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    verdict[R-L1-pyright-static]=PASS
  else
    verdict[R-L1-pyright-static]=FAIL
    note_blocking "phase1/nat_semiring/l1/pyright_static_rejected"
  fi
}

run_mypy() {
  if ! command -v "$mypy_bin" >/dev/null 2>&1; then
    verdict[R-L1-mypy-static]=SKIP
    note_blocking "phase1/nat_semiring/l1/mypy_unavailable"
    return
  fi
  if [[ "${#py_files[@]}" -eq 0 ]]; then
    verdict[R-L1-mypy-static]=SKIP
    note_blocking "phase1/nat_semiring/l1/python_emit_unavailable"
    return
  fi
  local log="$out/logs/python_static_mypy.log"
  set +e
  "${timed[@]}" "$mypy_bin" --follow-imports=skip --ignore-missing-imports "${py_files[@]}" >"$log" 2>&1
  local status=$?
  set -e
  if [[ "$status" -eq 0 ]]; then
    verdict[R-L1-mypy-static]=PASS
  else
    verdict[R-L1-mypy-static]=FAIL
    note_blocking "phase1/nat_semiring/l1/mypy_static_rejected"
  fi
}

run_pyright
run_mypy

row_pass="SKIP"
if [[ "${verdict[R-L1-pyright-static]}" == "FAIL" || "${verdict[R-L1-mypy-static]}" == "FAIL" ]]; then
  row_pass="FAIL"
elif [[ "${verdict[R-L1-pyright-static]}" == "PASS" || "${verdict[R-L1-mypy-static]}" == "PASS" ]]; then
  if [[ "${verdict[R-L1-pyright-static]}" == "SKIP" && "${verdict[R-L1-mypy-static]}" == "SKIP" ]]; then
    row_pass="SKIP"
  elif [[ "${verdict[R-L1-pyright-static]}" != "FAIL" && "${verdict[R-L1-mypy-static]}" != "FAIL" ]]; then
    row_pass="PASS"
  fi
fi

{
  echo "fixture=${fixture_id}"
  echo "  l1_python_static: ${row_pass}  (pyright=${verdict[R-L1-pyright-static]} mypy=${verdict[R-L1-mypy-static]})"
  echo "blocking_receipt: ${blocking_receipt}"
  echo ""
  echo "emit_tree: ${py_tree}"
  echo "logs: ${out}/logs/"
} | tee "$summary"

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  body="$(head -6 "$summary")"
  escaped="${body//$'\n'/%0A}"
  echo "::notice title=phase1/nat_semiring python static gate::${escaped}"
fi

if [[ "$strict" == "1" && "$row_pass" == "FAIL" ]]; then
  exit 1
fi

exit 0
