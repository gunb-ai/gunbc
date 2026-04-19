#!/usr/bin/env bash
#
# Per-test wall-clock ratchet (TESTING.md § test layers).
#
# Runs `cargo test -p v3-compiler -- --report-time` and fails if any single
# `#[test]` exceeded the budget (default 2000 ms, aligns with
# `feedback_test_timeout_2s`). Tests listed in the exemption file are
# tolerated with a logged warning so the ratchet can land without blocking
# known-slow tests — the exemption file IS the paydown backlog.
#
# Usage:
#   scripts/check-test-timeout.sh [budget_ms]
#
# Environment:
#   TEST_TIMEOUT_MS       Override budget (default 2000).
#   TEST_TIMEOUT_PACKAGE  Cargo package to test (default v3-compiler).
#   TEST_TIMEOUT_EXEMPT   Path to exemption file
#                         (default scripts/slow-test-exemptions.txt).

set -euo pipefail

budget_ms=${1:-${TEST_TIMEOUT_MS:-2000}}
pkg=${TEST_TIMEOUT_PACKAGE:-v3-compiler}
exempt_file=${TEST_TIMEOUT_EXEMPT:-scripts/slow-test-exemptions.txt}

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
cd "$repo_root"

log_file=$(mktemp -t test-timings.XXXXXX)
trap 'rm -f "$log_file"' EXIT

echo "Running cargo test -p $pkg -- --report-time (budget: ${budget_ms}ms per test)..."
# --report-time is stable since Rust 1.70. Libtest appends `<N.NNNs>` to each
# line for passing tests and `<N.NNNs>` for failures too. We only inspect the
# report lines, not the suite summary.
set +e
cargo test -p "$pkg" -- --report-time 2>&1 | tee "$log_file"
cargo_status=${PIPESTATUS[0]}
set -e

if [ "$cargo_status" -ne 0 ]; then
  echo "::error::cargo test failed (exit=$cargo_status) — per-test ratchet not evaluated"
  exit "$cargo_status"
fi

# Parse lines of shape:  test foo::bar_baz ... ok <0.123s>
# report-time normalizes to `<N.NNNs>`. POSIX awk (BSD on macOS,
# gawk on Ubuntu CI) lacks the `match(s, r, arr)` submatch form, so
# extract via field indexing: `$1=="test", $2=name, $3="...", $4=status, $5=<time>`.
# "ignored" tests have no timing trailer, so they fail the regex and are skipped.
violations=$(awk -v budget_ms="$budget_ms" '
  /^test[[:space:]]+[^ ]+[[:space:]]+\.\.\.[[:space:]]+(ok|FAILED)[[:space:]]+<[0-9]+\.[0-9]+s>$/ {
    name = $2
    timestr = $NF
    # timestr looks like "<1.234s>" — strip brackets and the "s" suffix.
    gsub(/[<>s]/, "", timestr)
    n = split(timestr, parts, ".")
    if (n != 2) next
    whole = parts[1] + 0
    frac = parts[2]
    while (length(frac) < 3) frac = frac "0"
    frac = substr(frac, 1, 3)
    elapsed_ms = whole * 1000 + (frac + 0)
    if (elapsed_ms > budget_ms) {
      printf "%s\t%d\n", name, elapsed_ms
    }
  }
' "$log_file")

# Load exemptions. Lines are `test::name  # reason` — the `#` and anything
# after it is a comment. Blank lines ignored.
exempt_set=""
if [ -f "$exempt_file" ]; then
  exempt_set=$(sed -e 's/#.*$//' -e 's/[[:space:]]*$//' "$exempt_file" | grep -v '^[[:space:]]*$' || true)
fi

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
