#!/usr/bin/env bash
# Self-test for scripts/check-test-timeout.sh.
#
# Pins the T-WAD Slice 6 / gate #102 behavior: `check-test-timeout.sh` accepts
# `TEST_TIMEOUT_MANIFEST` JSONL (self-test fixture) and enforces warn vs fail
# semantics. Production path projects rows from substrate via `gunbc-ci`.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CONSUMER="$ROOT/scripts/check-test-timeout.sh"
TMPDIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT

MANIFEST="$TMPDIR/test-node-wall-clock-ratchet.jsonl"
LOG="$TMPDIR/libtest.log"

cat > "$MANIFEST" <<'EOF'
{"test":"slow_warned_test","policy":"warn"}
{"test":"not_warn_policy","policy":"fail"}
EOF

write_log() {
  cat > "$LOG" <<'EOF'
test fast_test ... ok <0.001s>
test slow_warned_test ... ok <2.500s>
EOF
}

test_warn_manifest_allows_known_slow_test() {
  write_log
  local output
  if ! output=$(TEST_TIMEOUT_MANIFEST="$MANIFEST" bash "$CONSUMER" "$LOG" 2000 2>&1); then
    echo "FAIL [known-warn]: consumer rejected a manifest warn-policy test"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
  if [[ "$output" != *"warn-policy tests exceeded budget"* ]]; then
    echo "FAIL [known-warn]: consumer passed but did not report warn-policy backlog"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
}

test_unknown_slow_test_fails_closed() {
  write_log
  cat >> "$LOG" <<'EOF'
test slow_unknown_test ... ok <2.100s>
EOF

  local output exit_code
  if output=$(TEST_TIMEOUT_MANIFEST="$MANIFEST" bash "$CONSUMER" "$LOG" 2000 2>&1); then
    exit_code=0
  else
    exit_code=$?
  fi

  if [ "$exit_code" -eq 0 ]; then
    echo "FAIL [unknown-slow]: consumer passed with an over-budget test absent from JSONL"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
  if [[ "$output" != *"slow_unknown_test"* ]] || [[ "$output" != *"not warn-listed in manifest"* ]]; then
    echo "FAIL [unknown-slow]: consumer failed but did not name the missing slow test"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
}

test_non_warn_manifest_policy_fails_closed() {
  write_log
  cat >> "$LOG" <<'EOF'
test not_warn_policy ... ok <2.100s>
EOF

  local output exit_code
  if output=$(TEST_TIMEOUT_MANIFEST="$MANIFEST" bash "$CONSUMER" "$LOG" 2000 2>&1); then
    exit_code=0
  else
    exit_code=$?
  fi

  if [ "$exit_code" -eq 0 ]; then
    echo "FAIL [non-warn-policy]: consumer treated a non-warn manifest row as warn-listed"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
  if [[ "$output" != *"not_warn_policy"* ]] || [[ "$output" != *"not warn-listed in manifest"* ]]; then
    echo "FAIL [non-warn-policy]: consumer failed but did not name the non-warn slow test"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
}

test_zero_parsed_lines_fails_closed() {
  printf 'not libtest output\n' > "$LOG"

  local output exit_code
  if output=$(TEST_TIMEOUT_MANIFEST="$MANIFEST" bash "$CONSUMER" "$LOG" 2000 2>&1); then
    exit_code=0
  else
    exit_code=$?
  fi

  if [ "$exit_code" -eq 0 ]; then
    echo "FAIL [zero-parsed]: consumer passed on a log with no libtest timing lines"
    return 1
  fi
  if [[ "$output" != *"zero test-result lines parsed"* ]]; then
    echo "FAIL [zero-parsed]: consumer failed but did not report parser drift guard"
    printf '%s\n' "$output" | sed 's/^/  | /'
    return 1
  fi
}

failures=0

for test_fn in \
  test_warn_manifest_allows_known_slow_test \
  test_unknown_slow_test_fails_closed \
  test_non_warn_manifest_policy_fails_closed \
  test_zero_parsed_lines_fails_closed; do
  echo "Test: $test_fn..."
  if "$test_fn"; then
    echo "  PASS"
  else
    failures=$((failures + 1))
  fi
done

if [ "$failures" -ne 0 ]; then
  echo "FAIL: $failures check-test-timeout self-test(s) failed"
  exit 1
fi

echo "PASS: check-test-timeout warn-manifest behavior verified"
