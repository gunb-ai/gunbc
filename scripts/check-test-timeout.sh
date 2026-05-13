#!/usr/bin/env bash
#
# Per-test wall-clock ratchet (TESTING.md § test layers).
#
# Reads libtest `--report-time` output (lines like
# `test foo::bar ... ok <1.234s>`) from a pre-captured log file and fails
# if any single `#[test]` exceeded the budget (default 2000 ms, aligns
# with `feedback_test_timeout_2s`). Warn-only policy rows are projected from
# substrate (`dsl/gunbc/test_node_wall_clock_ratchet.dag`) via `gunbc-ci
# wall-clock-warn-manifest` (**interim toward #102** — canonical #102 is timing-fact
# authority via `TestNodeCostDimension`) into JSONL consumed here — same
# `{"test":"<token>","policy":"warn"}` transport `jq` already understands.
#
# **Naming alignment (gates #101 / #102).** Row tokens match libtest
# `--report-time` names — the same strings `TestNodeCostDimension` attaches
# to in `src/v3/std/verification.dag`.
#
# **Why parse an external log, not re-run `cargo test`.** The v3 CI job
# already runs `cargo test -p v3-compiler` once (budget 1200s) and the
# job-level timeout is 25 minutes. A second full-suite invocation from
# this ratchet would compete for the remaining job budget and could
# starve the clippy / lens / census gates on cold runners. Instead, the
# caller (CI step or local shell) runs `cargo test` itself with
# `RUSTC_BOOTSTRAP=1 ... -- -Z unstable-options --report-time`, tees the
# output to a log, and passes the path here. Local users with no
# pre-captured log get a fallback that invokes `cargo test` once.
#
# `--report-time` is unstable on the 1.93 toolchain
# (rust-lang/rust#64888); the `RUSTC_BOOTSTRAP=1 + -Z unstable-options`
# pair is the documented narrow unlock for libtest flags.
#
# **Env-scope caveat.** `RUSTC_BOOTSTRAP=1` on the outer shell is
# inherited by every child process — including the `rustc` invocations
# that the boundary tests spawn to exercise the real stable toolchain.
# That would break the boundary-layer contract (TESTING.md § test
# layers). Each `Command::new("rustc")` spawn site in the test tree
# therefore calls `.env_remove("RUSTC_BOOTSTRAP")` before spawning; see
# `tests/integration/common/mod.rs::RustcHarness::compile` and the four
# inline spawn sites in `tests/boundary/*`. Any **new** `rustc` spawn
# site must do the same. Migrate off this whole mechanism when
# `--report-time` stabilizes or when the project adopts `cargo-nextest`.
#
# Usage:
#   scripts/check-test-timeout.sh <log_file> [budget_ms]
#   scripts/check-test-timeout.sh                 # local fallback: runs cargo test itself
#
# Environment:
#   TEST_TIMEOUT_MS       Override budget (default 2000).
#   TEST_TIMEOUT_PACKAGE  Cargo package for the local fallback
#                         (default v3-compiler).
#   TEST_TIMEOUT_MANIFEST Optional path to JSONL warn manifest (self-tests /
#                         emergencies). When unset, rows come from `gunbc-ci
#                         wall-clock-warn-manifest` (requires built binary).
#   GUNBC_CI_BIN          Override path to `gunbc-ci` (default: search
#                         target/debug|release under repo root).
#   GITHUB_WORKSPACE      Passed through to `gunbc-ci` (default: repo root).
#
# **Fail-closed policy.** A test whose wall time exceeds `TEST_TIMEOUT_MS` and
# whose libtest name is **not** warn-listed fails this step. Add a row to
# `dsl/gunbc/test_node_wall_clock_ratchet.dag` in the **same PR** as any
# intentional warn-only expansion.

set -euo pipefail

log_file_arg=${1:-}
budget_ms=${2:-${TEST_TIMEOUT_MS:-2000}}
pkg=${TEST_TIMEOUT_PACKAGE:-v3-compiler}

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
cd "$repo_root"

cleanup_log=""
cleanup_manifest=""
trap '[ -n "$cleanup_log" ] && rm -f "$cleanup_log"; [ -n "$cleanup_manifest" ] && rm -f "$cleanup_manifest"' EXIT

if [ -n "${TEST_TIMEOUT_MANIFEST:-}" ]; then
  manifest=${TEST_TIMEOUT_MANIFEST}
else
  gunbc_ci_bin=${GUNBC_CI_BIN:-}
  if [ -z "$gunbc_ci_bin" ]; then
    for c in "$repo_root/target/debug/gunbc-ci" "$repo_root/target/release/gunbc-ci"; do
      if [ -x "$c" ]; then
        gunbc_ci_bin=$c
        break
      fi
    done
  fi
  if [ -z "$gunbc_ci_bin" ]; then
    echo "::error::gunbc-ci binary not found (build with: cargo build -p v3-compiler --bin gunbc-ci, or set GUNBC_CI_BIN / TEST_TIMEOUT_MANIFEST)."
    exit 2
  fi
  manifest=$(mktemp -t wall-clock-warn.XXXXXX.jsonl)
  cleanup_manifest=$manifest
  export GITHUB_WORKSPACE=${GITHUB_WORKSPACE:-$repo_root}
  if ! "$gunbc_ci_bin" wall-clock-warn-manifest >"$manifest"; then
    echo "::error::gunbc-ci wall-clock-warn-manifest failed"
    exit 2
  fi
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "::error::jq is required to parse warn manifest (install jq)."
  exit 2
fi

if [ ! -r "$manifest" ]; then
  echo "::error::test-node wall-clock manifest not readable: $manifest"
  exit 2
fi

if [ -n "$log_file_arg" ]; then
  log_file=$log_file_arg
  if [ ! -r "$log_file" ]; then
    echo "::error::log file not readable: $log_file"
    exit 2
  fi
  echo "Reading pre-captured libtest timings from $log_file (budget: ${budget_ms}ms per test)..."
else
  # Local fallback: re-run cargo test to capture timings. Not for CI —
  # the CI step reuses the log from the existing full-suite run.
  log_file=$(mktemp -t test-timings.XXXXXX)
  cleanup_log=$log_file
  echo "Running RUSTC_BOOTSTRAP=1 cargo test -p $pkg -- -Z unstable-options --report-time (budget: ${budget_ms}ms per test)..."
  set +e
  RUSTC_BOOTSTRAP=1 cargo test -p "$pkg" -- -Z unstable-options --report-time 2>&1 | tee "$log_file"
  cargo_status=${PIPESTATUS[0]}
  set -e
  if [ "$cargo_status" -ne 0 ]; then
    echo "::error::cargo test failed (exit=$cargo_status) — per-test ratchet not evaluated"
    exit "$cargo_status"
  fi
fi

# Parse lines of shape:  test foo::bar_baz ... ok <0.123s>  (fraction optional, e.g. <2s>)
# report-time normalizes to `<N.NNNs>`. POSIX awk (BSD on macOS,
# gawk on Ubuntu CI) lacks the `match(s, r, arr)` submatch form, so
# extract via field indexing: `$1=="test", $2=name, $3="...", $4=status, $5=<time>`.
# "ignored" tests have no timing trailer, so they fail the regex and are skipped.
#
# The awk script emits a trailing `__PARSED_COUNT=<n>` line so the caller
# can **fail closed** when zero test-result lines were parsed — protects
# against silent false-green if libtest output drifts (the format is
# explicitly unstable, rust-lang/rust#64888).
#
# Normalize captured CI logs: GitHub Actions sets `CARGO_TERM_COLOR=always`
# at the workflow level, and libtest lines can theoretically pick up leading
# ANSI escapes or CRLF line endings depending on how the stream is tee'd.
# Strip those before matching the `^test ... <N.NNNs>$` shape.
if command -v perl >/dev/null 2>&1; then
  awk_input() {
    # `set -o pipefail` is enabled for this script: if `perl` exits non-zero
    # (IO/regex edge case on a tee'd CI log), fall back to the raw file so the
    # ratchet still runs instead of aborting before diagnostics.
    perl -pe 's/\r\n/\n/g; s/\r/\n/g; s/\e\[[0-9;]*m//g' "$log_file" 2>/dev/null || cat "$log_file"
  }
else
  awk_input() {
    tr -d '\r' <"$log_file"
  }
fi
awk_output=$(awk_input | awk -v budget_ms="$budget_ms" '
  /^test[[:space:]]+[^ ]+[[:space:]]+\.\.\.[[:space:]]+(ok|FAILED)[[:space:]]+<[0-9]+(\.[0-9]*)?s>$/ {
    parsed_count++
    name = $2
    timestr = $NF
    # timestr looks like "<1.234s>" or "<2s>" — strip brackets and the "s" suffix.
    gsub(/[<>s]/, "", timestr)
    n = split(timestr, parts, ".")
    if (n == 1) {
      whole = parts[1] + 0
      frac_ms = 0
    } else if (n == 2) {
      whole = parts[1] + 0
      frac = parts[2]
      while (length(frac) < 3) frac = frac "0"
      frac_ms = substr(frac, 1, 3) + 0
    } else {
      next
    }
    elapsed_ms = whole * 1000 + frac_ms
    if (elapsed_ms > budget_ms) {
      printf "%s\t%d\n", name, elapsed_ms
    }
  }
  END {
    printf "__PARSED_COUNT=%d\n", parsed_count
  }
')

parsed_count=$(printf '%s\n' "$awk_output" | awk -F= '/^__PARSED_COUNT=/ {print $2}')
violations=$(printf '%s\n' "$awk_output" | awk '!/^__PARSED_COUNT=/ {print}' | awk 'NF')

if [ -z "$parsed_count" ] || [ "$parsed_count" -eq 0 ]; then
  echo "::error::zero test-result lines parsed from $log_file — libtest --report-time format may have drifted (tracking rust-lang/rust#64888). Failing closed rather than emit a silent green ratchet."
  echo "First 20 lines of the log:"
  head -20 "$log_file" | sed 's/^/  | /'
  exit 1
fi
echo "Parsed ${parsed_count} test-result lines from timing log."

warn_list=$(mktemp)
trap '[ -n "$cleanup_log" ] && rm -f "$cleanup_log"; [ -n "$cleanup_manifest" ] && rm -f "$cleanup_manifest"; rm -f "$warn_list"' EXIT
jq -r 'select(.policy == "warn") | .test' "$manifest" | sort -u >"$warn_list"

unexpected=""
warned=""
if [ -n "$violations" ]; then
  while IFS=$'\t' read -r name elapsed_ms; do
    if [ -z "$name" ]; then continue; fi
    if grep -Fxq "$name" "$warn_list"; then
      warned+=$(printf '%s\t%s\n' "$name" "$elapsed_ms")
      warned+=$'\n'
    else
      unexpected+=$(printf '%s\t%s\n' "$name" "$elapsed_ms")
      unexpected+=$'\n'
    fi
  done <<< "$violations"
fi

warn_source="warn-policy manifest"
if [ -z "${TEST_TIMEOUT_MANIFEST:-}" ]; then
  warn_source="warn-policy manifest (projected from dsl/gunbc/test_node_wall_clock_ratchet.dag)"
fi

if [ -n "$warned" ]; then
  echo "::warning::${budget_ms}ms ratchet: warn-policy tests exceeded budget ($warn_source):"
  printf '%s' "$warned" | awk -F'\t' 'NF==2 {printf "  %s — %sms\n", $1, $2}'
fi

if [ -n "$unexpected" ]; then
  echo "::error::${budget_ms}ms ratchet: tests exceeded budget (not warn-listed in manifest):"
  printf '%s' "$unexpected" | awk -F'\t' 'NF==2 {printf "  %s — %sms\n", $1, $2}'
  echo ""
  echo "Options:"
  echo "  1. Speed up the test (share bootstrap via OnceLock, shrink fixtures,"
  echo "     collapse fine-grained cases)."
  echo '  2. Add a WallClockWarnLibtestToken row to dsl/gunbc/test_node_wall_clock_ratchet.dag (same PR as the policy intent).'
  echo "  3. Mark #[ignore]-by-default if the coverage is redundant."
  exit 1
fi

echo "Per-test ratchet clean: no tests exceeded ${budget_ms}ms."
