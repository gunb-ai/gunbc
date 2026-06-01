#!/usr/bin/env bash
# scripts/v4-phase1-nat-semiring-rung-gate.sh
#
# Phase 1 rung gate: drives rungs 0–2 acceptance predicates over the ratified Phase 1
# fixture phase1/nat_semiring on a single module path, not the corpus-wide src/v4 sweep.
# Reports the §2.4 verdict matrix with PASS|FAIL|SKIP cell vocabulary:
#
#   fixture=phase1/nat_semiring
#     rung0: PASS|FAIL (dag=… rust=… python=… go=…)
#     rung1: PASS|FAIL (rust=…)
#     rung2: PASS|FAIL (rust=… python=… go=…)
#   blocking_receipt: <predicate id> | upstream_blocked:<predicate-id> | none
#
# Authority: docs/planning/v4-ladder-rung-specs-2026-05-30.md §2.1–§2.5 + §6 follow-up
# (parse-only R0 receipts, §2.4 SKIP/upstream_blocked semantics). Cells are PASS|FAIL|SKIP;
# rung row is PASS iff every cell is PASS, else FAIL (a row of all-SKIP is FAIL, not SKIP).
#
# Env:
#   V2_COMPILER                — v2-compiler binary (default: target/release/gunbc)
#   V4_PHASE1_NAT_SEMIRING_OUT — emit output dir (default: /tmp/v4-phase1-nat-semiring)
#   V4_PHASE1_NAT_SEMIRING_STRICT — if 1, exit non-zero on any rung failure (implies L1 strict)
#   V4_PHASE1_NAT_SEMIRING_PYTHON_RUNTIME_STRICT — if 1, L1 runtime gate fail-closed even when
#     parent STRICT=0 (merged into child export; parent exit honors either knob)
#   V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS — timeout per toolchain check (CI: 300)
#   V4_PHASE1_NAT_SEMIRING_PYTHON — python3 binary (default: python3)
#   V4_PHASE1_NAT_SEMIRING_GO     — go binary (default: go)
#   V4_PHASE1_NAT_SEMIRING_RUSTC  — rustc binary (default: rustc)
#   V4_PHASE1_NAT_SEMIRING_GOFMT  — gofmt binary (default: gofmt)
#   V4_GO_L1_NAT_SEMIRING_RECEIPT — Go L1 JSON receipt path (default: ${out}.go-l1-receipt.json)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_module_path="src/v4/test/claim/algebra_laws/nat_semiring.dag"
fixture_id="phase1/nat_semiring"
go_l1_slice_id="go_l1_nat_semiring_rung2"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_PHASE1_NAT_SEMIRING_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-phase1-nat-semiring"
else
  out="${V4_PHASE1_NAT_SEMIRING_OUT:-/tmp/v4-phase1-nat-semiring}"
fi
if [[ -x /opt/cargo/bin/cargo ]]; then
  cargo_bin="/opt/cargo/bin/cargo"
else
  cargo_bin="${CARGO_BIN:-cargo}"
fi
python_bin="${V4_PHASE1_NAT_SEMIRING_PYTHON:-python3}"
go_bin="${V4_PHASE1_NAT_SEMIRING_GO:-go}"
rustc_bin="${V4_PHASE1_NAT_SEMIRING_RUSTC:-rustc}"
gofmt_bin="${V4_PHASE1_NAT_SEMIRING_GOFMT:-gofmt}"
timeout_secs="${V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS:-300}"
strict="${V4_PHASE1_NAT_SEMIRING_STRICT:-0}"
l1_runtime_strict="${V4_PHASE1_NAT_SEMIRING_PYTHON_RUNTIME_STRICT:-0}"
if [[ "$strict" == "1" ]]; then
  l1_runtime_strict="1"
fi
summary="${out}.rung-gate-summary.txt"
go_l1_receipt="${V4_GO_L1_NAT_SEMIRING_RECEIPT:-${out}.go-l1-receipt.json}"

if [[ ! -f "$fixture_module_path" ]]; then
  echo "error: fixture module not found at $fixture_module_path" >&2
  exit 1
fi

if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  if [[ "$strict" == "1" ]]; then
    echo "::error title=phase1/nat_semiring rung gate setup::v2-compiler missing at $bin (phase1/nat_semiring/setup/v2_compiler_missing)"
    exit 2
  fi
  echo "::notice title=phase1/nat_semiring rung gate::skipped — v2-compiler missing"
  exit 0
fi

# Host toolchain availability is distinct from fixture rung failure. Missing host binary
# is a CI provisioning gap (setup receipt + exit 2 under STRICT=1), not a fixture R0 FAIL.
missing_tools=()
command -v "$python_bin" >/dev/null 2>&1 || missing_tools+=("$python_bin")
command -v "$go_bin"     >/dev/null 2>&1 || missing_tools+=("$go_bin")
command -v "$gofmt_bin"  >/dev/null 2>&1 || missing_tools+=("$gofmt_bin")
command -v "$rustc_bin"  >/dev/null 2>&1 || missing_tools+=("$rustc_bin")
if [[ "${#missing_tools[@]}" -gt 0 ]]; then
  echo "error: required host toolchain(s) missing: ${missing_tools[*]}" >&2
  if [[ "$strict" == "1" ]]; then
    echo "::error title=phase1/nat_semiring rung gate setup::host toolchain missing: ${missing_tools[*]} (phase1/nat_semiring/setup/host_toolchain_missing)"
    exit 2
  fi
  echo "::notice title=phase1/nat_semiring rung gate::skipped — host toolchain missing: ${missing_tools[*]}"
  exit 0
fi

rm -rf "$out"
mkdir -p "$out/rust" "$out/python" "$out/go" "$out/logs"

# Fixture-scoped entry isolation: v2-compiler treats the FIRST --source-root as the entry
# pool; subsequent --source-root values are dep pools resolved via imports. Scope the
# compile to the fixture module's transitive closure — the §2.5 "fixture-scoped" contract.
entry_root="$out/entry"
deps_root="$out/deps"
fixture_relpath="${fixture_module_path#src/v4/}"
mkdir -p "$entry_root/$(dirname "$fixture_relpath")"
cp "$fixture_module_path" "$entry_root/$fixture_relpath"
cp -R src/v4/. "$deps_root/"
rm -f "$deps_root/$fixture_relpath"

# Per-predicate verdict slots. Cell vocabulary: PASS|FAIL|SKIP (§2.4). Default SKIP until
# the predicate is actually executed and observed; emit/typecheck/compile only flip to
# PASS or FAIL when the receipt-bearing command was run end-to-end (INVARIANTS P3).
declare -A verdict=(
  [R0-dag-parse]=SKIP
  [R0-rust-parse]=SKIP
  [R0-python-parse]=SKIP
  [R0-go-parse]=SKIP
  [R1-rust-typecheck]=SKIP
  [R2-rust-compile]=SKIP
  [R2-python-compile]=SKIP
  [R2-go-compile]=SKIP
)
blocking_receipt="none"

note_blocking() {
  # First-failure-wins per §2.4: lowest rung, earliest predicate that was executed and
  # failed (or first *_emit_unavailable / upstream_blocked attribution).
  local pred="$1"
  if [[ "$blocking_receipt" == "none" ]]; then
    blocking_receipt="$pred"
  fi
}

run_step() {
  local log="$1"; shift
  if [[ "$timeout_secs" -gt 0 ]]; then
    timeout --preserve-status "$timeout_secs" "$@" 2>&1 | tee "$log"
    return "${PIPESTATUS[0]}"
  else
    "$@" 2>&1 | tee "$log"
    return "${PIPESTATUS[0]}"
  fi
}

# Bounded-command prefix for invocations that can't go through run_step (e.g. piped into
# xargs, or where stdout needs to be discarded). Keeps the §22 per-toolchain timeout
# contract uniform across every host-process boundary (INVARIANTS P3 fail-closed).
if [[ "$timeout_secs" -gt 0 ]]; then
  timed=(timeout --preserve-status "$timeout_secs")
else
  timed=()
fi

# --- Rung 0 dag-parse: v4 ingest of the fixture module accepts. ---
parse_log="$out/logs/dag_parse.log"
set +e
run_step "$parse_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/dag-parse" \
  --target dag
parse_status=$?
set -e
if [[ "$parse_status" -eq 0 ]]; then
  verdict[R0-dag-parse]=PASS
else
  verdict[R0-dag-parse]=FAIL
  note_blocking "phase1/nat_semiring/rung0/dag_parse_rejected"
fi

# --- Rust target ---
# §2.1 R0-rust-parse: parse-only receipt. Allowed: `rustc -Z parse-only` when the pinned
# toolchain supports it (nightly). FORBIDDEN: `cargo check`, `cargo build`,
# `rustc --emit=metadata`, `rustfmt --check`. If `-Z parse-only` is unavailable (stable
# rustc), this cell is SKIP with `rust_parse_driver_unavailable` — the spec accepts
# ship_disposition: GAP until Compiler Spine ratifies a stable parse driver.
rust_emit_log="$out/logs/rust_emit.log"
set +e
run_step "$rust_emit_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/rust" \
  --target rust
rust_emit_status=$?
set -e
rust_rs_count=0
if [[ -d "$out/rust" ]]; then
  rust_rs_count="$(find "$out/rust" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
fi
if [[ "$rust_emit_status" -ne 0 ]]; then
  verdict[R0-rust-parse]=FAIL
  note_blocking "phase1/nat_semiring/rung0/rust_emit_parse_rejected"
elif [[ ! -f "$out/rust/Cargo.toml" || "${rust_rs_count:-0}" -lt 1 ]]; then
  # No emit artifact: SKIP, not FAIL (§2.1 emit-unavailable carve-out).
  verdict[R0-rust-parse]=SKIP
  note_blocking "phase1/nat_semiring/rung0/rust_emit_unavailable"
else
  # Probe nightly `-Z parse-only` support on this rustc; otherwise GAP/SKIP.
  rust_parse_probe_log="$out/logs/rust_parse_probe.log"
  set +e
  "${timed[@]}" "$rustc_bin" -Z parse-only --edition=2021 --crate-type lib /dev/null >"$rust_parse_probe_log" 2>&1
  rust_parse_probe_status=$?
  set -e
  # A toolchain that recognises -Z parse-only accepts it; stable rustc rejects -Z flags
  # entirely with "the option `Z` is only accepted on the nightly compiler".
  if grep -qE 'only accepted on the nightly compiler|requires -Zunstable-options' "$rust_parse_probe_log" 2>/dev/null; then
    verdict[R0-rust-parse]=SKIP
    note_blocking "phase1/nat_semiring/rung0/rust_parse_driver_unavailable"
  else
    rust_parse_log="$out/logs/rust_parse.log"
    : >"$rust_parse_log"
    rust_parse_ok=1
    while IFS= read -r -d '' rs; do
      set +e
      "${timed[@]}" "$rustc_bin" -Z parse-only --edition=2021 --crate-type lib "$rs" >>"$rust_parse_log" 2>&1
      rs_status=$?
      set -e
      if [[ "$rs_status" -ne 0 ]]; then
        rust_parse_ok=0
      fi
    done < <(find "$out/rust" -name '*.rs' -print0 2>/dev/null)
    if [[ "$rust_parse_ok" -eq 1 ]]; then
      verdict[R0-rust-parse]=PASS
    else
      verdict[R0-rust-parse]=FAIL
      note_blocking "phase1/nat_semiring/rung0/rust_emit_parse_rejected"
    fi
  fi
fi

# R1-rust-typecheck: prerequisite §2.4 — runs only when R0-rust-parse is PASS.
if [[ "${verdict[R0-rust-parse]}" == "PASS" ]]; then
  rust_check_log="$out/logs/rust_check.log"
  set +e
  run_step "$rust_check_log" "$cargo_bin" check --jobs 4 --manifest-path "$out/rust/Cargo.toml"
  rust_check_status=$?
  set -e
  if [[ "$rust_check_status" -eq 0 ]]; then
    verdict[R1-rust-typecheck]=PASS
  else
    verdict[R1-rust-typecheck]=FAIL
    note_blocking "phase1/nat_semiring/rung1/rust_typecheck_failed"
  fi
else
  verdict[R1-rust-typecheck]=SKIP
  note_blocking "upstream_blocked:R0-rust-parse"
fi

# R2-rust-compile: §2.3 rung 2 Rust ⊇ rung 1; runs only when R1 is PASS.
if [[ "${verdict[R1-rust-typecheck]}" == "PASS" ]]; then
  verdict[R2-rust-compile]=PASS
else
  verdict[R2-rust-compile]=SKIP
  # note_blocking is first-wins; this is a no-op when R1 / R0 already set blocking_receipt.
  note_blocking "upstream_blocked:R1-rust-typecheck"
fi

# --- Python target ---
# §2.2 explicitly accepts `python3 -m py_compile` as the surface for both R0-python-parse
# and R2-python-compile in Phase 1; both predicates flip on the same receipt.
py_emit_log="$out/logs/python_emit.log"
set +e
run_step "$py_emit_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/python" \
  --target python
py_emit_status=$?
set -e
py_file_count=0
if [[ -d "$out/python" ]]; then
  py_file_count="$(find "$out/python" -name '*.py' 2>/dev/null | wc -l | tr -d ' ')"
fi
if [[ "$py_emit_status" -ne 0 ]]; then
  verdict[R0-python-parse]=FAIL
  verdict[R2-python-compile]=SKIP
  note_blocking "phase1/nat_semiring/rung0/python_emit_parse_rejected"
elif [[ "${py_file_count:-0}" -lt 1 ]]; then
  verdict[R0-python-parse]=SKIP
  verdict[R2-python-compile]=SKIP
  note_blocking "phase1/nat_semiring/rung0/python_emit_unavailable"
else
  py_check_log="$out/logs/python_check.log"
  set +e
  find "$out/python" -name '*.py' -print0 2>/dev/null \
    | xargs -0 "$python_bin" -m py_compile 2>&1 | tee "$py_check_log"
  py_check_status=${PIPESTATUS[1]}
  set -e
  if [[ "$py_check_status" -eq 0 ]]; then
    verdict[R0-python-parse]=PASS
    verdict[R2-python-compile]=PASS
  else
    verdict[R0-python-parse]=FAIL
    verdict[R2-python-compile]=SKIP
    note_blocking "phase1/nat_semiring/rung0/python_emit_parse_rejected"
  fi
fi

# --- Go target ---
# §2.1 R0-go-parse: parse-only. Allowed: `gofmt -e` (reports parse/syntax errors only,
# does not build). FORBIDDEN for R0: `go build`, `go test -c`. R2-go-compile uses
# `go build` and runs only when R0-go-parse is PASS (§2.4 prerequisite).
go_emit_log="$out/logs/go_emit.log"
set +e
run_step "$go_emit_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/go" \
  --target go
go_emit_status=$?
set -e
go_file_count=0
if [[ -d "$out/go" ]]; then
  go_file_count="$(find "$out/go" -name '*.go' 2>/dev/null | wc -l | tr -d ' ')"
fi
if [[ "$go_emit_status" -ne 0 ]]; then
  verdict[R0-go-parse]=FAIL
  note_blocking "phase1/nat_semiring/rung0/go_emit_parse_rejected"
elif [[ "${go_file_count:-0}" -lt 1 ]]; then
  verdict[R0-go-parse]=SKIP
  note_blocking "phase1/nat_semiring/rung0/go_emit_unavailable"
else
  go_parse_log="$out/logs/go_parse.log"
  set +e
  find "$out/go" -name '*.go' -print0 2>/dev/null \
    | xargs -0 "${timed[@]}" "$gofmt_bin" -e >/dev/null 2>"$go_parse_log"
  go_parse_status=$?
  set -e
  if [[ "$go_parse_status" -eq 0 ]]; then
    verdict[R0-go-parse]=PASS
  else
    verdict[R0-go-parse]=FAIL
    note_blocking "phase1/nat_semiring/rung0/go_emit_parse_rejected"
  fi
fi

if [[ "${verdict[R0-go-parse]}" == "PASS" ]]; then
  go_build_log="$out/logs/go_build.log"
  set +e
  ( cd "$out/go" && run_step "$go_build_log" "$go_bin" build ./... )
  go_build_status=$?
  set -e
  if [[ "$go_build_status" -eq 0 ]]; then
    verdict[R2-go-compile]=PASS
  else
    verdict[R2-go-compile]=FAIL
    note_blocking "phase1/nat_semiring/rung2/go_compile_failed"
  fi
else
  verdict[R2-go-compile]=SKIP
  note_blocking "upstream_blocked:R0-go-parse"
fi

go_l1_diagnostic_source="$out/logs/go_build.log"
go_l1_blocking_receipt="none"
case "${verdict[R2-go-compile]}" in
  PASS) go_l1_blocking_receipt="none" ;;
  FAIL) go_l1_blocking_receipt="phase1/nat_semiring/rung2/go_compile_failed" ;;
  *) go_l1_blocking_receipt="upstream_blocked:R0-go-parse" ;;
esac
if [[ "${verdict[R0-go-parse]}" != "PASS" ]]; then
  go_l1_diagnostic_source="$out/logs/go_parse.log"
fi
if [[ "$go_emit_status" -ne 0 || "${go_file_count:-0}" -lt 1 ]]; then
  go_l1_diagnostic_source="$out/logs/go_emit.log"
  if [[ "$go_emit_status" -ne 0 ]]; then
    go_l1_blocking_receipt="phase1/nat_semiring/rung0/go_emit_parse_rejected"
  else
    go_l1_blocking_receipt="phase1/nat_semiring/rung0/go_emit_unavailable"
  fi
fi
mkdir -p "$(dirname "$go_l1_receipt")"
"$python_bin" - "$go_l1_receipt" "$go_l1_slice_id" "$out/go" "${verdict[R2-go-compile]}" "$go_l1_blocking_receipt" "$go_l1_diagnostic_source" <<'PY'
import json
import pathlib
import sys

receipt_path, slice_id, go_module_root, verdict, blocking_receipt, diagnostic_source = sys.argv[1:7]
diagnostic_snippet = None
diagnostic_path = pathlib.Path(diagnostic_source)
if verdict != "PASS" and diagnostic_path.exists():
    text = diagnostic_path.read_text(encoding="utf-8", errors="replace").strip()
    if text:
        diagnostic_snippet = "\n".join(text.splitlines()[-20:])

payload = {
    "schema": "scripts/v4-phase1-nat-semiring-rung-gate.sh::go_l1_compile_receipt_v1",
    "slice_id": slice_id,
    "fixture": "phase1/nat_semiring",
    "predicate": "R2-go-compile",
    "go_module_root": go_module_root,
    "verdict": verdict,
    "blocking_receipt": blocking_receipt,
    "diagnostic_snippet": diagnostic_snippet,
}
pathlib.Path(receipt_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

# --- Rung roll-up (§2.4): row PASS iff every cell PASS; otherwise FAIL (all-SKIP → FAIL). ---
row_aggregate() {
  local row_pass="PASS"
  local p
  for p in "$@"; do
    if [[ "${verdict[$p]}" != "PASS" ]]; then row_pass="FAIL"; fi
  done
  echo "$row_pass"
}
rung0_pass="$(row_aggregate R0-dag-parse R0-rust-parse R0-python-parse R0-go-parse)"
rung1_pass="$(row_aggregate R1-rust-typecheck)"
rung2_pass="$(row_aggregate R2-rust-compile R2-python-compile R2-go-compile)"

l1_python_runtime_pass="SKIP"
# L1 runtime exec requires R2-python-compile PASS (py_compile receipt); R0 alone is insufficient.
if [[ "${verdict[R2-python-compile]}" == "PASS" ]]; then
  export V4_PHASE1_NAT_SEMIRING_OUT="$out"
  export V4_PHASE1_NAT_SEMIRING_PYTHON="$python_bin"
  export V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS="$timeout_secs"
  export V4_PHASE1_NAT_SEMIRING_PYTHON_RUNTIME_STRICT="$l1_runtime_strict"
  set +e
  bash "${root}/scripts/v4-phase1-nat-semiring-python-runtime-gate.sh"
  l1_status=$?
  set -e
  if [[ -f "${out}.python-runtime-gate-summary.txt" ]]; then
    l1_line="$(grep -E '^  l1_python_runtime:' "${out}.python-runtime-gate-summary.txt" || true)"
    if [[ "$l1_line" =~ l1_python_runtime:\ PASS ]]; then
      l1_python_runtime_pass="PASS"
    elif [[ "$l1_line" =~ l1_python_runtime:\ FAIL ]]; then
      l1_python_runtime_pass="FAIL"
      note_blocking "phase1/nat_semiring/l1/python_runtime_exec_rejected"
    fi
  fi
  if [[ "$l1_runtime_strict" == "1" && "$l1_status" -ne 0 ]]; then
    l1_python_runtime_pass="FAIL"
    note_blocking "phase1/nat_semiring/l1/python_runtime_exec_rejected"
  fi
fi

{
  echo "fixture=${fixture_id}"
  echo "  rung0: ${rung0_pass}  (dag=${verdict[R0-dag-parse]} rust=${verdict[R0-rust-parse]} python=${verdict[R0-python-parse]} go=${verdict[R0-go-parse]})"
  echo "  rung1: ${rung1_pass}  (rust=${verdict[R1-rust-typecheck]})"
  echo "  rung2: ${rung2_pass}  (rust=${verdict[R2-rust-compile]} python=${verdict[R2-python-compile]} go=${verdict[R2-go-compile]})"
  echo "  l1_python_runtime: ${l1_python_runtime_pass}  (python exec after py_compile; see ${out}.python-runtime-gate-summary.txt)"
  echo "blocking_receipt: ${blocking_receipt}"
  echo "go_l1_receipt: ${go_l1_receipt}"
  echo ""
  echo "logs: ${out}/logs/"
} | tee "$summary"

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  body="$(head -10 "$summary")"
  escaped="${body//$'\n'/%0A}"
  echo "::notice title=phase1/nat_semiring rung gate::${escaped}"
fi

if [[ "$strict" == "1" ]]; then
  if [[ "$rung0_pass" != "PASS" || "$rung1_pass" != "PASS" || "$rung2_pass" != "PASS" || "$l1_python_runtime_pass" == "FAIL" ]]; then
    exit 1
  fi
elif [[ "$l1_runtime_strict" == "1" && "$l1_python_runtime_pass" == "FAIL" ]]; then
  exit 1
fi

exit 0
