#!/usr/bin/env bash
# Self-test for scripts/check-release-doc-authority.sh.
#
# Why: without this, a future change to RETRACTION_PATTERNS could
# silently neuter the consumer (broaden patterns until everything
# passes) and the guardrail would become ceremonial. Per-string
# isolation negative tests prove every forbidden string is caught
# individually; bundling would only prove "at least one caught."
#
# Exit codes:
#   0 — all negative + positive assertions verified
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

# Negative cases: each forbidden string in isolation must fail the
# consumer (proves every string is caught individually, not just "at
# least one caught"). Helper writes one live forbidden string in
# non-retraction context and asserts non-zero exit.
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

test_negative_t_verification_l4l7() {
  test_negative_single "T-Verification-L4L7" \
    "T-Verification-L4L7 verifies the no-engine claim via runtime evaluation."
}

# Fail-closed-on-missing-doc: if a configured release doc is missing,
# the consumer must fail with a diagnostic naming it. Silently skipping
# would let release-control authority shrink without review.
test_missing_doc_fails_closed() {
  # Set up a fixture where r2-structure.md exists but r3-structure.md is
  # deleted (simulating a rename or accidental deletion).
  cat > "$TMPDIR/docs/r2-structure.md" <<'EOF'
# R2 Structure (test fixture — clean; no forbidden strings)
EOF
  rm -f "$TMPDIR/docs/r3-structure.md"
  # Keep thesis-mapping.md and r2-structure.md present so the only
  # missing doc is r3-structure.md.
  cat > "$TMPDIR/docs/thesis/r2-r3-thesis-mapping.md" <<'EOF'
# Thesis Mapping (test fixture — clean; no forbidden strings)
EOF
  cd "$TMPDIR"
  # Shield from `set -e`: the consumer is EXPECTED to exit non-zero
  # here (that's the whole point of this test). Capture both output
  # and exit code via an `if` form, which puts the command in a
  # context where errexit is suppressed. Per codex BLOCKING on
  # `a9326224`: a bare `output=$(...)` in errexit mode would abort
  # the whole self-test on the expected failure path.
  if output=$(bash "$TEST_CONSUMER" 2>&1); then
    exit_code=0
  else
    exit_code=$?
  fi
  cd "$ROOT"

  # Restore r3-structure.md for subsequent tests.
  echo "" > "$TMPDIR/docs/r3-structure.md"

  if [ "$exit_code" -eq 0 ]; then
    echo "FAIL [missing-doc]: consumer passed with r3-structure.md missing"
    echo "  Expected: consumer should fail-closed and name the missing doc"
    return 1
  fi
  if ! echo "$output" | grep -q "MISSING: docs/r3-structure.md"; then
    echo "FAIL [missing-doc]: consumer failed but didn't name the missing doc"
    echo "  Expected output to contain: 'MISSING: docs/r3-structure.md'"
    echo "  Actual output:"
    echo "$output" | sed 's/^/    /'
    return 1
  fi
  return 0
}

# Positive case: forbidden strings in retraction context should pass.
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

failures=0

for test_fn in \
  test_negative_t_ground_engine \
  test_negative_t_ground_annotation \
  test_negative_canonical_choice \
  test_negative_at_target \
  test_negative_decisions_locked \
  test_negative_t_verification_l4l7; do
  forbidden="${test_fn#test_negative_}"
  echo "Test (negative/$forbidden): live string should fail consumer..."
  if "$test_fn"; then
    echo "  PASS"
  else
    failures=$((failures + 1))
  fi
done

echo "Test (missing-doc): consumer must fail-closed when a configured doc is missing..."
if test_missing_doc_fails_closed; then
  echo "  PASS"
else
  failures=$((failures + 1))
fi

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
