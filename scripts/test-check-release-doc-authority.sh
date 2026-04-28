#!/usr/bin/env bash
# Self-test for scripts/check-release-doc-authority.sh.
#
# Per gpt-5-5-pro meta-review on PR #1078 (2026-04-28T03:47Z): the
# release-doc authority consumer needs at least one negative fixture
# proving live forbidden strings actually fail the check. Without
# this, a future change to RETRACTION_PATTERNS could silently neuter
# the consumer (broaden patterns until everything passes), and the
# loop's central concern — "the guardrail becomes ceremonial" — would
# return.
#
# This script:
#   1. Creates a temporary fixture file with each forbidden string
#      in a clearly-live (non-retraction) context
#   2. Runs the consumer against it
#   3. Asserts the consumer detects each violation (exit code != 0)
#   4. Cleans up
#   5. Then creates a second fixture with the same strings in
#      retraction context and asserts the consumer passes
#
# Usage:
#   bash scripts/test-check-release-doc-authority.sh
#
# Exit codes:
#   0 — both negative and positive cases verified
#   1 — at least one assertion failed (consumer is broken)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CONSUMER="scripts/check-release-doc-authority.sh"
TMPDIR="$(mktemp -d)"
trap "rm -rf $TMPDIR" EXIT

# Copy the consumer into TMPDIR so its ROOT calculation
# (cd "$(dirname "$0")/..") resolves to TMPDIR, not the real repo root.
mkdir -p "$TMPDIR/scripts"
cp "$CONSUMER" "$TMPDIR/scripts/"
TEST_CONSUMER="$TMPDIR/scripts/$(basename "$CONSUMER")"

# Set up the empty TMPDIR/docs/ tree once. Each test rewrites
# TMPDIR/docs/r2-structure.md with its own fixture content immediately
# before invoking the consumer, so tests don't interfere with each other.
mkdir -p "$TMPDIR/docs/thesis"
echo "" > "$TMPDIR/docs/r3-structure.md"
echo "" > "$TMPDIR/docs/thesis/r2-r3-thesis-mapping.md"

# ---------------------------------------------------------------
# Negative cases (per-string): each forbidden string must be caught
# in isolation. Per gpt-5-5-pro/codex review on PR #1078:
# bundling all forbidden strings into one fixture only proves
# "at least one caught" not "each caught" — future broadening that
# silently exempts a single string (e.g., @target) would still pass
# the bundled test. Per-string isolation tests prevent that.
# ---------------------------------------------------------------

# Helper: write a fixture containing exactly one live forbidden string
# in non-retraction context, run the consumer, expect non-zero exit.
test_negative_single() {
  local forbidden="$1"
  local content="$2"

  cat > "$TMPDIR/docs/r2-structure.md" <<EOF
# R2 Structure (test fixture — live forbidden string: $forbidden)

$content
EOF
  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [negative/$forbidden]: consumer passed on a fixture with live '$forbidden'"
    echo "  Expected: consumer should detect '$forbidden' in non-retraction context"
    return 1
  fi
  cd "$ROOT"
  return 0
}

test_negative_t_ground_engine() {
  test_negative_single "T-Ground-Engine" \
    "T-Ground-Engine is a live lane in this fixture."
}

test_negative_t_ground_annotation() {
  test_negative_single "T-Ground-Annotation" \
    "T-Ground-Annotation is a live program-side substrate lane."
}

test_negative_canonical_choice() {
  test_negative_single "canonical choice" \
    "When multiple inhabitants exist, the canonical choice is declared at the language level."
}

test_negative_at_target() {
  test_negative_single "@target" \
    "Users annotate program-side intent via @target syntax."
}

test_negative_decisions_locked() {
  test_negative_single "DECISIONS LOCKED" \
    "DECISIONS LOCKED 2026-04-28: Director ratified all 8 challenges as final decisions."
}

# ---------------------------------------------------------------
# Positive case: forbidden strings in retraction context should pass
# ---------------------------------------------------------------
test_positive() {
  cat > "$TMPDIR/docs/r2-structure.md" <<'EOF'
# R2 Structure (test fixture — retraction context)

## Lane structure

~~T-Ground-Engine~~ RETRACTED 2026-04-28 — replaced by 5 substrate-completion lanes per supersession.

The retracted T-Ground-Engine framing is described here for audit only.

## Decisions

🔄 SUPERSEDED 2026-04-28: prior "DECISIONS LOCKED" [retraction-context] framing was retracted.

The retracted T-Ground-Annotation lane is replaced. @target [retraction-context: annotation supersession] no longer used.

The "canonical choice" [retraction-context: documenting supersession] framing was retracted.
EOF
  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    return 0
  fi
  cd "$ROOT"
  echo "FAIL: consumer rejected a fixture with all forbidden strings in retraction context"
  echo "  Expected: consumer should pass when forbidden strings are accompanied by"
  echo "  explicit retraction markers (~~, RETRACTED, SUPERSEDED, [retraction-context])"
  return 1
}

# ---------------------------------------------------------------
# Run both tests
# ---------------------------------------------------------------

failures=0

# Per-string negative tests (5)
for test_fn in \
  test_negative_t_ground_engine \
  test_negative_t_ground_annotation \
  test_negative_canonical_choice \
  test_negative_at_target \
  test_negative_decisions_locked; do
  forbidden="${test_fn#test_negative_}"
  echo "Test (negative/$forbidden): live string should fail consumer..."
  if "$test_fn"; then
    echo "  PASS"
  else
    failures=$((failures + 1))
  fi
done

# Positive test (1) — retraction-context strings should pass
echo "Test (positive): retraction-context forbidden strings should pass consumer..."
if test_positive; then
  echo "  PASS"
else
  failures=$((failures + 1))
fi

if [ "$failures" -gt 0 ]; then
  echo ""
  echo "Self-test FAILED: $failures assertion(s) failed."
  echo "The consumer at $CONSUMER is not enforcing its declared contract."
  exit 1
fi

echo ""
echo "Self-test PASSED: consumer correctly distinguishes live forbidden strings"
echo "from retraction-context forbidden strings."
