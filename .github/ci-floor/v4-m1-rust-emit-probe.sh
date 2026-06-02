#!/usr/bin/env bash
# .github/ci-floor/v4-m1-rust-emit-probe.sh
#
# M1 informational probe: v2-compiler --target rust over full src/v4, then
# cargo check on emitted output. Missing compiler, v2 emit failure, and skipped
# cargo-check preconditions fail closed; V4_M1_RUST_EMIT_PROBE_STRICT controls
# whether rustc residuals from an attempted cargo check also fail the step.
#
# Authority: src/v4/workflow/ci.dag (T-24) + src/v4/TASKS.md T-24; r3 gates #98/#100 interim bridge.
# Pattern: .github/ci-floor/v4-bootstrap-viability.sh (compile + log receipt parsing).
#
# Env:
#   V2_COMPILER              — v2-compiler binary (default: target/release/gunbc)
#   V4_M1_RUST_EMIT_OUT       — rust emit output dir (default: /tmp/v4-rust-emit)
#   V4_M1_RUST_EMIT_LOG       — v2 compile log (default: ${OUT}.compile.log)
#   V4_M1_DAG_EMIT_OUT        — optional dag emit output dir for shared rust+dag closure
#   V4_M1_DAG_EMIT_LOG        — dag compile receipt log (default: ${DAG_OUT}.compile.log)
#   V4_M1_RUSTC_LOG           — cargo check log (default: ${OUT}.rustc.log)
#   V4_M1_RUST_EMIT_PROBE_STRICT — if 1, exit non-zero when rustc fails
#   V4_M1_RUSTC_TIMEOUT_SECS  — optional timeout for cargo check (CI: 600)
#   V4_M1_CARGO_CHECK_JOBS_CEILING — host-governor job ceiling (CTRL_BUILD_DYNAMIC_JOBS_MAX,
#                               default 64; modeled as m1_probe_cargo_check_jobs_ceiling in
#                               src/v4/workflow/ci.dag). Actual jobs are memory-denominated below it.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_M1_RUST_EMIT_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-rust-emit"
else
  out="${V4_M1_RUST_EMIT_OUT:-/tmp/v4-rust-emit}"
fi
# The emitted-tree cargo check must run JOBSERVER-COUPLED: cargo draws a host jobserver token per
# rustc, so parallelism fills the machine when idle and pares down under load (the host pool bounds
# rustc processes across all runners). Two coupling sources, in order:
#   1. inherited MAKEFLAGS carrying --jobserver-auth — GHA runners get this from the
#      actions-runner@.service systemd unit; raw cargo joins the pool directly (no ctrl-build on GHA).
#   2. ctrl-build — in session containers MAKEFLAGS is unset, so route through ctrl-build, which sets
#      MAKEFLAGS from CTRL_JOBSERVER_FIFO (and adds the MemAvailable picker + sccache).
# NO FALLBACK: if neither is present the probe fails closed (a missing jobserver coupling surfaces
# immediately rather than silently running an uncoupled check). Modeled in dsl/std/compute_fabric.dag.
if [[ -x /opt/cargo/bin/cargo ]]; then
  cargo_bin="/opt/cargo/bin/cargo"
else
  cargo_bin="${CARGO_BIN:-cargo}"
fi
# Treat the inherited coupling as usable only if the jobserver token source actually resolves — a
# bare `*jobserver-auth*` substring match would accept a STALE/MALFORMED auth (deleted FIFO, empty
# value, closed fds) and run raw cargo UNCOUPLED, defeating fail-closed (INVARIANTS P3). Mirrors
# ctrl-build, which drops MAKEFLAGS when the FIFO isn't readable+writable.
m1_inherited_jobserver_usable() {
  local mf="${MAKEFLAGS:-}" auth
  [[ "$mf" == *--jobserver-auth=* ]] || return 1
  auth="${mf##*--jobserver-auth=}"   # strip up to the last --jobserver-auth=
  auth="${auth%%[[:space:]]*}"       # take the token (up to next whitespace)
  case "$auth" in
    fifo:?*)
      local fifo="${auth#fifo:}"
      [[ -p "$fifo" && -r "$fifo" && -w "$fifo" ]] || return 1 ;;
    [0-9]*,[0-9]*)
      local r="${auth%%,*}" w="${auth##*,}"
      [[ -r "/proc/self/fd/$r" && -w "/proc/self/fd/$w" ]] || return 1 ;;
    *)
      return 1 ;;   # empty / malformed / unrecognized auth → not usable
  esac
  return 0
}
ctrl_build_bin=""
if m1_inherited_jobserver_usable; then
  : # validated inherited jobserver coupling (live FIFO/fds) — run raw cargo
elif command -v ctrl-build >/dev/null 2>&1; then
  ctrl_build_bin="$(command -v ctrl-build)"
else
  echo "error: M1 emit-probe requires a host jobserver coupling that is actually usable —" >&2
  echo "       inherited MAKEFLAGS=--jobserver-auth is absent/stale/malformed and ctrl-build" >&2
  echo "       is not present (no fallback)." >&2
  exit 1
fi
compile_log="${V4_M1_RUST_EMIT_LOG:-${out}.compile.log}"
rustc_log="${V4_M1_RUSTC_LOG:-${out}.rustc.log}"
summary="${out}.m1-probe-summary.txt"
strict="${V4_M1_RUST_EMIT_PROBE_STRICT:-0}"
dag_out="${V4_M1_DAG_EMIT_OUT:-}"
dag_log="${V4_M1_DAG_EMIT_LOG:-}"
shared_out=""
if [[ -n "$dag_out" ]]; then
  shared_out="$(dirname "$out")/v4-shared-closure"
  dag_log="${dag_log:-${dag_out}.compile.log}"
fi

if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  exit 1
fi

if [[ -n "$dag_out" ]]; then
  rm -rf "$shared_out" "$out" "$dag_out"
else
  rm -rf "$out"
fi
mkdir -p "$(dirname "$compile_log")"
if [[ -n "$dag_log" ]]; then
  mkdir -p "$(dirname "$dag_log")"
fi

emit_notice() {
  local title="$1"
  local body="$2"
  # GitHub Actions workflow commands tolerate newlines in body when escaped.
  local escaped="${body//$'\n'/%0A}"
  escaped="${escaped//\r/}"
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
    echo "::notice title=${title}::${escaped}"
  fi
}

if [[ -n "$dag_out" ]]; then
  echo "=== M1: v2-compiler compile --target rust+dag src/v4 (single source closure) ==="
else
  echo "=== M1: v2-compiler compile --target rust src/v4 ==="
fi
set +e
if [[ -n "$dag_out" ]]; then
  "$bin" compile --source-root src/v4 --output-dir "$shared_out" --target rust+dag 2>&1 | tee "$compile_log"
else
  "$bin" compile --source-root src/v4 --output-dir "$out" --target rust 2>&1 | tee "$compile_log"
fi
compile_status=${PIPESTATUS[0]}
set -e

if [[ -n "$dag_out" && -d "$shared_out/rust" && -d "$shared_out/dag" ]]; then
  mv "$shared_out/rust" "$out"
  mv "$shared_out/dag" "$dag_out"
  rmdir "$shared_out" 2>/dev/null || true
  cp "$compile_log" "$dag_log"
fi

compiled_receipt=""
if [[ -f "$compile_log" ]]; then
  compiled_receipt="$(grep -E '^compiled: [0-9]+ files emitted, [0-9]+ diagnostics$' "$compile_log" | tail -1 || true)"
fi

files_emitted=0
v2_diagnostics=0
if [[ -n "$compiled_receipt" ]]; then
  files_emitted="$(echo "$compiled_receipt" | sed -n 's/^compiled: \([0-9]*\) files emitted, \([0-9]*\) diagnostics$/\1/p')"
  v2_diagnostics="$(echo "$compiled_receipt" | sed -n 's/^compiled: \([0-9]*\) files emitted, \([0-9]*\) diagnostics$/\2/p')"
fi

rs_on_disk=0
if [[ -d "$out" ]]; then
  rs_on_disk="$(find "$out" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
fi

# v2 diagnostic lines: error[file:line:col]: message  OR  error: message
v2_error_lines=0
v2_error_categories=""
if [[ -f "$compile_log" ]]; then
  v2_error_lines="$(grep -cE '^[[:space:]]*error(\[[^]]+\])?: ' "$compile_log" 2>/dev/null || true)"
  v2_error_lines="${v2_error_lines:-0}"
  v2_error_categories="$(
    grep -oE '^[[:space:]]*error(\[[^]]+\])?: ' "$compile_log" 2>/dev/null \
      | sed 's/^[[:space:]]*//' | sort | uniq -c | sort -rn | head -20 || true
  )"
fi

rustc_attempted=false
rustc_skipped=true
rustc_skip_reason=""
rustc_status=""
rustc_error_total=0
rustc_categories=""
rustc_files_with_errors=0
if [[ "$compile_status" -eq 0 && -f "$out/Cargo.toml" ]]; then
  rustc_attempted=true
  rustc_skipped=false
  rustc_skip_reason=""
  echo "=== M1: cargo check on emitted tree ==="
  rustc_timeout="${V4_M1_RUSTC_TIMEOUT_SECS:-}"
  if [[ -n "${GITHUB_ACTIONS:-}" && -z "$rustc_timeout" ]]; then
    rustc_timeout=600
  fi
  # Parallelism is jobserver-governed: the host token pool bounds rustc processes across all runners,
  # so the check fills the machine when idle and pares down under load. Capped per-invocation at the
  # ceiling. The 2026-05-28 swap incident was a static cap × N-runners with no shared pool; the
  # jobserver bounds the host-wide total instead.
  #   V4_M1_CARGO_CHECK_JOBS_CEILING (v4.workflow.ci `m1_probe_cargo_check_jobs_ceiling`) — the
  #     per-invocation --jobs ceiling (raw/GHA path); the jobserver pares actual concurrency below it.
  cargo_check_jobs="${V4_M1_CARGO_CHECK_JOBS_CEILING:-64}"
  echo "M1 jobserver coupling: MAKEFLAGS=${MAKEFLAGS:-<empty>}; ctrl_build=${ctrl_build_bin:-<none>}; --jobs ceiling=${cargo_check_jobs}"
  check_cmd=()
  if [[ -n "$rustc_timeout" ]]; then
    check_cmd+=(timeout --preserve-status "$rustc_timeout")
  fi
  if [[ -n "$ctrl_build_bin" ]]; then
    check_cmd+=("$ctrl_build_bin" --)
  fi
  check_cmd+=("$cargo_bin" check --manifest-path "$out/Cargo.toml")
  if [[ -z "$ctrl_build_bin" ]]; then
    # Raw/GHA path: cap at the ceiling; the inherited MAKEFLAGS jobserver pares concurrency below it.
    # (The ctrl-build path uses its own MemAvailable CARGO_BUILD_JOBS picker — don't override with --jobs.)
    check_cmd+=(--jobs "$cargo_check_jobs")
  fi
  set +e
  CTRL_BUILD_DYNAMIC_JOBS_MAX="${V4_M1_CARGO_CHECK_JOBS_CEILING:-64}" \
    "${check_cmd[@]}" 2>&1 | tee "$rustc_log"
  rustc_status=${PIPESTATUS[0]}
  set -e
else
  if [[ "$compile_status" -ne 0 ]]; then
    rustc_skip_reason="compile_failed (exit ${compile_status})"
  elif [[ ! -f "$out/Cargo.toml" ]]; then
    rustc_skip_reason="no Cargo.toml in emit output"
  else
    rustc_skip_reason="unknown"
  fi
  echo "=== M1: skipping cargo check (${rustc_skip_reason}) ===" | tee "$rustc_log"
fi

if [[ -f "$rustc_log" ]]; then
  # grep exits 1 on zero matches; with pipefail that must not abort the probe
  # before summary + non-strict exit 0 (modeled non_blocking / INVARIANTS P3/P5).
  rustc_error_total="$(grep -cE '^error\[E[0-9]+\]:' "$rustc_log" 2>/dev/null || true)"
  rustc_error_total="${rustc_error_total:-0}"
  rustc_categories="$(
    grep -oE 'error\[E[0-9]+\]:' "$rustc_log" 2>/dev/null \
      | sort | uniq -c | sort -rn | head -25 || true
  )"
  rustc_files_with_errors="$(
    grep -oE '\-\-> src/[^:]+\.rs' "$rustc_log" 2>/dev/null \
      | sed 's/^--> //' | sort -u | wc -l | tr -d ' ' || true
  )"
  rustc_files_with_errors="${rustc_files_with_errors:-0}"
fi

{
  echo "M1 v4 full-tree rust emit probe"
  echo "==========================="
  echo "v2 compile exit: ${compile_status}"
  echo "v2 receipt: ${compiled_receipt:-<missing>}"
  echo "v2 stderr error lines: ${v2_error_lines}"
  echo ".rs files on disk: ${rs_on_disk}"
  echo ""
  echo "cargo check attempted: ${rustc_attempted}"
  echo "cargo check skipped: ${rustc_skipped}"
  if [[ "$rustc_attempted" == "true" ]]; then
    echo "cargo check exit: ${rustc_status}"
  else
    echo "cargo check skip_reason: ${rustc_skip_reason}"
  fi
  echo "rustc error[E####] lines: ${rustc_error_total}"
  echo "distinct .rs files with rustc errors: ${rustc_files_with_errors}"
  echo ""
  if [[ -n "$v2_error_categories" ]]; then
    echo "v2 error prefix histogram (top 20):"
    echo "$v2_error_categories"
    echo ""
  fi
  if [[ -n "$rustc_categories" ]]; then
    echo "rustc error code histogram (top 25):"
    echo "$rustc_categories"
    echo ""
  fi
  echo "logs: compile=${compile_log} rustc=${rustc_log}"
} | tee "$summary"

probe_body="$(head -20 "$summary")"
emit_notice "M1 v4 rust emit probe" "$probe_body"

if [[ "$compile_status" -ne 0 ]]; then
  emit_notice "M1 compile failed" "exit=${compile_status}; see ${compile_log}"
elif [[ "$v2_diagnostics" != "0" ]]; then
  emit_notice "M1 compile diagnostics" "${compiled_receipt:-see log}"
fi

if [[ "$rustc_attempted" == "true" && "${rustc_status:-0}" -ne 0 ]]; then
  emit_notice "M1 rustc gap surface" "cargo check exit=${rustc_status}; ${rustc_error_total} error lines; ${rustc_files_with_errors} files"
fi

echo "=== M1 probe summary written to ${summary} ==="

if [[ "$compile_status" -ne 0 ]]; then
  exit "$compile_status"
fi

if [[ "$rustc_skipped" == "true" ]]; then
  echo "error: M1 probe requires cargo check after successful compile (skip_reason=${rustc_skip_reason})" >&2
  exit 1
fi

if [[ "$strict" == "1" && "$rustc_attempted" == "true" && "${rustc_status:-0}" -ne 0 ]]; then
  exit "$rustc_status"
fi

exit 0
