#!/usr/bin/env bash
#
# Per-test wall-clock ratchet (TESTING.md § test layers).
#
# Reads libtest `--report-time` output (lines like
# `test foo::bar ... ok <1.234s>`) from a pre-captured log file and fails
# if any single `#[test]` exceeded the budget (default 2000 ms, aligns
# with `feedback_test_timeout_2s`). Tests listed in the exemption file
# are tolerated with a logged warning so the ratchet can land without
# blocking known-slow tests — the exemption file IS the paydown backlog.
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
#   TEST_TIMEOUT_EXEMPT   Path to exemption file
#                         (default scripts/slow-test-exemptions.txt).
#   TEST_TIMEOUT_MAX_EXEMPTIONS
#                         Ratchet floor for active exemption entries
#                         (default 42, captured 2026-04-24). Lower this
#                         value in the same PR that removes exemptions.

set -euo pipefail

log_file_arg=${1:-}
budget_ms=${2:-${TEST_TIMEOUT_MS:-2000}}
pkg=${TEST_TIMEOUT_PACKAGE:-v3-compiler}
exempt_file=${TEST_TIMEOUT_EXEMPT:-scripts/slow-test-exemptions.txt}
max_exemptions=${TEST_TIMEOUT_MAX_EXEMPTIONS:-42}

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
cd "$repo_root"

cleanup_log=""
trap '[ -n "$cleanup_log" ] && rm -f "$cleanup_log"' EXIT

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

# Load exemptions. Lines are `test::name  # reason` — the `#` and anything
# after it is a comment. Blank lines ignored.
exempt_set=""
if [ -f "$exempt_file" ]; then
  exempt_set=$(sed -e 's/^[[:space:]]*//' -e 's/#.*$//' -e 's/[[:space:]]*$//' "$exempt_file" | grep -v '^[[:space:]]*$' || true)
fi

exempt_count=0
if [ -n "$exempt_set" ]; then
  exempt_count=$(printf '%s\n' "$exempt_set" | wc -l | tr -d '[:space:]')
fi

if ! printf '%s\n' "$max_exemptions" | grep -Eq '^[0-9]+$'; then
  echo "::error::TEST_TIMEOUT_MAX_EXEMPTIONS must be a non-negative integer, got: $max_exemptions"
  exit 2
fi

if [ "$exempt_count" -ne "$max_exemptions" ]; then
  if [ "$exempt_count" -gt "$max_exemptions" ]; then
    echo "::error::slow-test exemption count grew to ${exempt_count}; ratchet floor is ${max_exemptions}."
    echo "Remove at least $((exempt_count - max_exemptions)) exemption(s)."
  else
    echo "::error::slow-test exemption count shrank to ${exempt_count}; ratchet floor is still ${max_exemptions}."
    echo "Lower TEST_TIMEOUT_MAX_EXEMPTIONS in the same PR that deletes exemptions."
  fi
  exit 1
fi
echo "Slow-test exemption count: ${exempt_count}/${max_exemptions}."

unexpected=""
warned=""
if [ -n "$violations" ]; then
  while IFS=$'\t' read -r name elapsed_ms; do
    if [ -z "$name" ]; then continue; fi
    if [ -n "$exempt_set" ] && echo "$exempt_set" | grep -Fxq "$name"; then
      warned+=$(printf '%s\t%s\n' "$name" "$elapsed_ms")
      warned+=$'\n'
    else
      unexpected+=$(printf '%s\t%s\n' "$name" "$elapsed_ms")
      unexpected+=$'\n'
    fi
  done <<< "$violations"
fi

if [ -n "$warned" ]; then
  echo "::warning::${budget_ms}ms ratchet: exempt tests exceeded budget (paydown backlog in $exempt_file):"
  printf '%s' "$warned" | awk -F'\t' 'NF==2 {printf "  %s — %sms\n", $1, $2}'
fi

if [ -n "$unexpected" ]; then
  echo "::error::${budget_ms}ms ratchet: tests exceeded budget (not in exemption list):"
  printf '%s' "$unexpected" | awk -F'\t' 'NF==2 {printf "  %s — %sms\n", $1, $2}'
  echo ""
  echo "Options:"
  echo "  1. Speed up the test (share bootstrap via OnceLock, shrink fixtures,"
  echo "     collapse fine-grained cases)."
  echo "  2. Add to $exempt_file with a reason and a ROADMAP/task reference."
  echo "  3. Mark #[ignore]-by-default if the coverage is redundant."
  exit 1
fi

echo "Per-test ratchet clean: no tests exceeded ${budget_ms}ms."
