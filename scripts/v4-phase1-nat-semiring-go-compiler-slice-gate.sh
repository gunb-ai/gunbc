#!/usr/bin/env bash
# scripts/v4-phase1-nat-semiring-go-compiler-slice-gate.sh
#
# L1 Go compiler-slice receipt for fixture=phase1/nat_semiring: after go emit + R2-go-compile
# prerequisite, record structured JSON tying slice_id go_l1_nat_semiring_rung2 → go build verdict.
#
# Authority: docs/planning/v4-go-l1-compiler-slice-compile-worksheet-2026-06-01.md
# (GO-L1-COMPILER-SLICE-COMPILE); complements scripts/v4-phase1-nat-semiring-rung-gate.sh
# (R2-go-compile cell). Non-goals: L2 cross-target eval, L3 self-output fixed point, full
# v4-bootstrap-viability.sh.
#
# Env:
#   V4_PHASE1_NAT_SEMIRING_OUT — emit output dir (same as rung gate; required)
#   V4_PHASE1_NAT_SEMIRING_GO — go binary (default: go)
#   V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS — per-invocation timeout (default: 300)
#   V4_PHASE1_NAT_SEMIRING_GO_COMPILER_SLICE_STRICT — exit 1 on FAIL when 1 (when chained from
#     rung gate, parent sets this from GO_COMPILER_SLICE_STRICT and/or V4_PHASE1_NAT_SEMIRING_STRICT)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_id="phase1/nat_semiring"
slice_id="go_l1_nat_semiring_rung2"
go_bin="${V4_PHASE1_NAT_SEMIRING_GO:-go}"
timeout_secs="${V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS:-300}"
strict="${V4_PHASE1_NAT_SEMIRING_GO_COMPILER_SLICE_STRICT:-0}"

if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_PHASE1_NAT_SEMIRING_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-phase1-nat-semiring"
else
  out="${V4_PHASE1_NAT_SEMIRING_OUT:-}"
fi

if [[ -z "$out" ]]; then
  echo "error: V4_PHASE1_NAT_SEMIRING_OUT must point at the rung-gate emit tree" >&2
  exit 2
fi

go_tree="$out/go"
go_module_root="$go_tree"
summary="${out}.go-compiler-slice-gate-summary.txt"
receipt_json="${out}.go-compiler-slice-receipt.json"

if [[ ! -d "$go_tree" ]]; then
  echo "error: go emit tree missing at $go_tree (run v4-phase1-nat-semiring-rung-gate.sh first)" >&2
  exit 2
fi

if ! command -v "$go_bin" >/dev/null 2>&1; then
  echo "error: go toolchain not found: $go_bin" >&2
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
  [R-L1-go-compiler-slice-compile]=SKIP
)
blocking_receipt="none"
diagnostic_snippet=""

note_blocking() {
  local pred="$1"
  if [[ "$blocking_receipt" == "none" ]]; then
    blocking_receipt="$pred"
  fi
}

go_files=()
while IFS= read -r -d '' f; do
  go_files+=("$f")
done < <(find "$go_tree" -name '*.go' -print0 2>/dev/null || true)

if [[ "${#go_files[@]}" -eq 0 ]]; then
  verdict[R-L1-go-compiler-slice-compile]=SKIP
  note_blocking "phase1/nat_semiring/l1/go_emit_unavailable"
else
  mkdir -p "$out/logs"
  build_log="$out/logs/go_compiler_slice_build.log"
  set +e
  ( cd "$go_module_root" && "${timed[@]}" "$go_bin" build ./... ) >"$build_log" 2>&1
  build_status=$?
  set -e
  if [[ "$build_status" -eq 0 ]]; then
    verdict[R-L1-go-compiler-slice-compile]=PASS
  else
    verdict[R-L1-go-compiler-slice-compile]=FAIL
    # tail -20 already bounds size; omit head -c to avoid SIGPIPE under pipefail.
    diagnostic_snippet="$(tail -20 "$build_log" 2>/dev/null | sed 's/\\/\\\\/g; s/"/\\"/g' | tr '\n' ' ')"
    note_blocking "phase1/nat_semiring/l1/go_compiler_slice_build_rejected"
  fi
fi

row_pass="FAIL"
if [[ "${verdict[R-L1-go-compiler-slice-compile]}" == "PASS" ]]; then
  row_pass="PASS"
fi

# Structured L1 receipt (worksheet §10 systemic fix item 3).
{
  printf '{'
  printf '"schema":"scripts/v4-phase1-nat-semiring-go-compiler-slice-gate.sh::go_l1_compiler_slice_receipt_v1",'
  printf '"slice_id":"%s",' "$slice_id"
  printf '"fixture_id":"%s",' "$fixture_id"
  printf '"go_module_root":"%s",' "$go_module_root"
  printf '"verdict":"%s",' "$row_pass"
  if [[ -n "$diagnostic_snippet" ]]; then
    printf '"diagnostic_snippet":"%s",' "$diagnostic_snippet"
  fi
  printf '"predicate":"R-L1-go-compiler-slice-compile"'
  printf '}\n'
} >"$receipt_json"

{
  echo "fixture=${fixture_id}"
  echo "  slice_id: ${slice_id}"
  echo "  l1_go_compiler_slice: ${row_pass}  (go=${verdict[R-L1-go-compiler-slice-compile]})"
  echo "blocking_receipt: ${blocking_receipt}"
  echo ""
  echo "go_module_root: ${go_module_root}"
  echo "receipt_json: ${receipt_json}"
  echo "logs: ${out}/logs/"
} | tee "$summary"

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  body="$(head -8 "$summary")"
  escaped="${body//$'\n'/%0A}"
  echo "::notice title=phase1/nat_semiring go compiler-slice gate::${escaped}"
fi

if [[ "$strict" == "1" && "$row_pass" != "PASS" ]]; then
  exit 1
fi

exit 0
