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
# Negative case: live forbidden strings should fail the check
# ---------------------------------------------------------------
test_negative() {
  cat > "$TMPDIR/docs/r2-structure.md" <<'EOF'
# R2 Structure (test fixture — live forbidden strings)

This is a fixture intended to fail the consumer.

## Lane structure

T-Ground-Engine is a live lane in this fixture.

## Decisions locked

DECISIONS LOCKED 2026-04-28: Director ratified that T-Ground-Annotation
remains live as a parallel-authority annotation surface using @target syntax.
The canonical choice is documented inline.

(All forbidden strings appear in non-retraction context; consumer should fail.)
EOF
  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL: consumer passed on a fixture with live forbidden strings"
    echo "  Expected: consumer should detect T-Ground-Engine, T-Ground-Annotation,"
    echo "  canonical choice, @target, DECISIONS LOCKED in non-retraction context"
    return 1
  fi
  cd "$ROOT"
  return 0
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

echo "Test 1 (negative): live forbidden strings should fail consumer..."
if test_negative; then
  echo "  PASS"
else
  failures=$((failures + 1))
fi

echo "Test 2 (positive): retraction-context forbidden strings should pass consumer..."
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
