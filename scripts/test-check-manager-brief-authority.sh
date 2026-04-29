#!/usr/bin/env bash
# Self-test for scripts/check-manager-brief-authority.sh.
#
# Why: without this, future changes to MANAGER_BRIEFS or the Q1/Q2/Q4/Q5
# pattern set could silently neuter the consumer (broaden patterns until
# everything passes, drop briefs from MANAGER_BRIEFS, etc.) and the
# guardrail would become ceremonial. Per-question isolation negative
# tests prove every check is caught individually.
#
# Exit codes:
#   0 — all assertions verified
#   1 — at least one assertion failed (consumer is broken)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CONSUMER="scripts/check-manager-brief-authority.sh"
TMPDIR="$(mktemp -d)"
trap "rm -rf $TMPDIR" EXIT

# Copy the consumer + setup tmp git repo + 7 brief stubs.
mkdir -p "$TMPDIR/scripts"
cp "$CONSUMER" "$TMPDIR/scripts/"
TEST_CONSUMER="$TMPDIR/scripts/$(basename "$CONSUMER")"

mkdir -p "$TMPDIR/docs/briefs" "$TMPDIR/docs/thesis"

# Init a tmp git repo with one synthetic merge commit "(#999)" so Q4
# positive tests have something to find via git log without needing
# network access to gh.
(
  cd "$TMPDIR"
  git init -q
  git config user.email "test@test.local"
  git config user.name "Test"
  echo "seed" > seed.txt
  git add seed.txt
  git commit -q -m "test commit subject (#999)"
)

# Seed authority files referenced by the briefs.
cat > "$TMPDIR/docs/r2-structure.md" <<'EOF'
# R2 Structure

## Goal 1
EOF

cat > "$TMPDIR/docs/r3-structure.md" <<'EOF'
# R3 Structure
EOF

cat > "$TMPDIR/docs/thesis/r2-r3-thesis-mapping.md" <<'EOF'
# Thesis Mapping
EOF

# Helper: write 7 minimal briefs, optionally overriding one with custom content.
# Uses markdown-bold form (**7**) to match the format live briefs use —
# per gpt-5-5-pro review on ea33aeb9d, earlier fixtures used bare
# digits ("7 standing R2 managers") which proved Q5 for a format the
# live docs don't actually use. Live briefs write
# "Names this manager one of **7** standing R2 managers".
#
# Includes a `landed via #999` claim in the clean fixture (lowercase,
# matching live brief format) so the Q4 positive path is non-vacuous —
# per gpt-5-5-pro review on 91b5274f. The tmp git repo (test_q4 setup
# below) seeds a "(#999)" merge subject so this resolves cleanly.
write_clean_briefs() {
  local override_brief="${1:-}"
  local override_content="${2:-}"

  for name in evaluator grounding impossible-bugs modeling pure-bootstrap release substrate; do
    local brief="$TMPDIR/docs/briefs/r2-${name}-manager.md"
    if [ "r2-${name}-manager.md" = "$override_brief" ]; then
      printf "%s" "$override_content" > "$brief"
    else
      cat > "$brief" <<EOF
# R2 ${name} Manager Brief (clean stub)

References [r2-structure](../r2-structure.md) — clean.
References [r3-structure](../r3-structure.md) — clean.
References [thesis](../thesis/r2-r3-thesis-mapping.md) — clean.

Names this manager one of **7** standing R2 managers.
R3 has **10** R3 lanes.
Substrate landed via #999 (positive Q4 fixture; tmp-git seed).
EOF
    fi
  done
}

# ---------------------------------------------------------------------
# Q1 — file-existence negative test
# ---------------------------------------------------------------------

test_negative_q1_missing_file() {
  write_clean_briefs "r2-evaluator-manager.md" \
    "# Brief\n\nReferences [missing](../no-such-file.md) — should fail Q1."

  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [Q1/missing-file]: consumer passed on a fixture with a missing cited file"
    return 1
  fi
  cd "$ROOT"
  return 0
}

# ---------------------------------------------------------------------
# Q2 — anchor-existence negative test
# ---------------------------------------------------------------------

test_negative_q2_missing_anchor() {
  write_clean_briefs "r2-evaluator-manager.md" \
    "# Brief\n\nReferences [r2-structure with bogus anchor](../r2-structure.md#nonexistent-anchor) — should fail Q2."

  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [Q2/missing-anchor]: consumer passed on a fixture with a bogus anchor"
    return 1
  fi
  cd "$ROOT"
  return 0
}

# ---------------------------------------------------------------------
# Q4 — LANDED-PR-not-in-history negative test
# ---------------------------------------------------------------------
# A claim "LANDED via #88888888" must fail because the tmp repo's only
# merge commit is "(#999)". The fall-back gh API call will return null
# for an obviously-unreachable PR number against the tmp repo's origin
# (which is unset), so the assertion fires.

test_negative_q4_unreachable_pr() {
  write_clean_briefs "r2-evaluator-manager.md" \
    "# Brief\n\nClaims work LANDED via #88888888 (unreachable in tmp repo) — should fail Q4."

  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [Q4/unreachable-pr]: consumer passed on a fixture with an unreachable PR claim"
    return 1
  fi
  cd "$ROOT"
  return 0
}

# Per gpt-5-5-pro review on 91b5274f: live briefs use title-case
# "Landed via #N" in addition to UPPERCASE/lowercase forms; verify the
# case-insensitive Q4 regex catches it. Q4 is now grep -oEi.
test_negative_q4_unreachable_pr_titlecase() {
  write_clean_briefs "r2-evaluator-manager.md" \
    "# Brief\n\nLanded via #88888887 (title-case, unreachable in tmp repo) — should fail Q4."

  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [Q4/unreachable-pr-titlecase]: consumer passed on title-case 'Landed via #N' with unreachable PR"
    return 1
  fi
  cd "$ROOT"
  return 0
}

# ---------------------------------------------------------------------
# Q5 — cross-brief projection mismatch negative test
# ---------------------------------------------------------------------
# Have one brief say "5 standing R2 managers" while others say 7.
# The mismatch must be reported.

test_negative_q5_count_mismatch() {
  # Default briefs say "**7** standing R2 managers"; override one to
  # say "**5**" — markdown-bold form matches live briefs (per
  # gpt-5-5-pro review). Pre-fix this test passed silently because
  # the regex required bare digits and missed the bolded count
  # entirely on both fixtures, leaving counts_seen empty (no
  # mismatch to flag).
  write_clean_briefs "r2-evaluator-manager.md" \
    "# Brief\n\nReferences [r2-structure](../r2-structure.md).\n\nNames this manager one of **5** standing R2 managers (intentional drift)."

  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [Q5/count-mismatch]: consumer passed on a fixture with mismatched manager counts (markdown-bold form)"
    return 1
  fi
  cd "$ROOT"
  return 0
}

# ---------------------------------------------------------------------
# Q5 — divergent-from-canonical negative test
# ---------------------------------------------------------------------
# All briefs agree on "5 standing R2 managers" — agreement is preserved
# but it diverges from canonical (7).

test_negative_q5_divergent_from_canonical() {
  # All briefs agree on **5** standing R2 managers — agreement is
  # preserved but diverges from canonical (7). Markdown-bold form
  # matches live brief format.
  for name in evaluator grounding impossible-bugs modeling pure-bootstrap release substrate; do
    cat > "$TMPDIR/docs/briefs/r2-${name}-manager.md" <<EOF
# R2 ${name} Manager Brief
References [r2-structure](../r2-structure.md).
Names this manager one of **5** standing R2 managers (all briefs agree but diverges from canonical 7).
EOF
  done

  cd "$TMPDIR"
  if bash "$TEST_CONSUMER" >/dev/null 2>&1; then
    cd "$ROOT"
    echo "FAIL [Q5/divergent-from-canonical]: consumer passed on a fixture where all briefs agree on a non-canonical value (markdown-bold form)"
    return 1
  fi
  cd "$ROOT"
  return 0
}

# ---------------------------------------------------------------------
# Fail-closed precheck — missing brief test
# ---------------------------------------------------------------------

test_negative_missing_brief_fails_closed() {
  write_clean_briefs
  rm "$TMPDIR/docs/briefs/r2-evaluator-manager.md"

  cd "$TMPDIR"
  local output
  if output="$(bash "$TEST_CONSUMER" 2>&1)"; then
    exit_code=0
  else
    exit_code=$?
  fi
  cd "$ROOT"

  if [ "$exit_code" -eq 0 ]; then
    echo "FAIL [missing-brief]: consumer passed with one brief missing"
    return 1
  fi
  if ! echo "$output" | grep -q "MISSING: docs/briefs/r2-evaluator-manager.md"; then
    echo "FAIL [missing-brief]: consumer failed but didn't name the missing brief"
    echo "  Expected output to contain: 'MISSING: docs/briefs/r2-evaluator-manager.md'"
    echo "  Actual output:"
    echo "$output" | sed 's/^/    /'
    return 1
  fi
  return 0
}

# ---------------------------------------------------------------------
# Positive case — all 7 briefs clean, all 4 questions pass
# ---------------------------------------------------------------------

test_positive_all_clean() {
  write_clean_briefs

  cd "$TMPDIR"
  local output exit_code
  if output="$(bash "$TEST_CONSUMER" 2>&1)"; then
    exit_code=0
  else
    exit_code=$?
  fi
  cd "$ROOT"

  if [ "$exit_code" -ne 0 ]; then
    echo "FAIL [positive]: consumer rejected a clean fixture"
    echo "  output:"
    echo "$output" | sed 's/^/    /'
    return 1
  fi
  if ! echo "$output" | grep -q "no violations"; then
    echo "FAIL [positive]: consumer didn't print success message on clean fixture"
    return 1
  fi
  return 0
}

# ---------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------

failures=0

for test_fn in \
  test_negative_q1_missing_file \
  test_negative_q2_missing_anchor \
  test_negative_q4_unreachable_pr \
  test_negative_q4_unreachable_pr_titlecase \
  test_negative_q5_count_mismatch \
  test_negative_q5_divergent_from_canonical \
  test_negative_missing_brief_fails_closed; do
  echo "Test ($test_fn): negative case should fail consumer..."
  if "$test_fn"; then
    echo "  PASS"
  else
    failures=$((failures + 1))
  fi
done

echo "Test (positive): clean fixture should pass consumer..."
if test_positive_all_clean; then
  echo "  PASS"
else
  failures=$((failures + 1))
fi

if [ "$failures" -gt 0 ]; then
  echo ""
  echo "Self-test FAILED: $failures contract assertion(s) failed."
  echo "The consumer at $CONSUMER is not enforcing its declared contract."
  exit 1
fi

echo ""
echo "Self-test PASSED: 8 contract assertions verified"
echo "  (6 negative — Q1/Q2/Q4×2/Q5×2 + missing-brief; 1 positive — clean fixture w/ landed-PR)"
