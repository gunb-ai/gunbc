#!/usr/bin/env bash
# Release-doc authority consumer — checks for forbidden stale concept
# names in live sections of release-control docs.
#
# Authority: docs/r2-structure.md §"Release-doc authority discipline"
# (Open call 4, added 2026-04-28 per gpt-5-5-pro meta-review of PR #1078).
#
# Why: PR #1078 review loop (22+ events, ~3h 05m, 9 codex passes,
# 5 claude, 6 openai-pro incl. 3 meta-reviews, 2 cursor as of
# 2026-04-28T04:47Z) kept catching the same P2 single-authority
# shape in new clothing — stale lane names persisting in sibling
# tables after the live framing was retracted. This script is the
# v1 mechanical consumer that turns manual grep-as-a-service into
# a CI-checkable invariant for FORBIDDEN STRINGS.
#
# It does NOT catch structured-state drift (lane counts, dependency
# graphs, thesis-claim dispositions). The 04:47 meta-review identified
# that gap when T-CostLens-Composition was added and again failed to
# cascade through dependency/thesis-mapping projections. v2 guardrail
# (structured release-state authority + mechanical projection-check)
# is named follow-up debt.
#
# Usage:
#   scripts/check-release-doc-authority.sh
#
# Exit codes:
#   0 — no violations
#   1 — at least one forbidden string in a live (non-retraction) context

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Release-control docs covered by this consumer.
#
# Scope: docs that carry live release-control authority (program / lane /
# state). Design docs that *discuss* retracted concepts in narrative
# (e.g., docs/design-emission-model.md, which explains why the engine
# framing was retracted and what replaced it) are intentionally OUT of
# scope — they're explanatory authorities, not release-control authorities.
# Including them would force every retraction-narrative line to carry an
# explicit marker, neutering the check the way claude-opus-4-7 review
# warned against (broad retraction patterns silently allow live framings).
#
# If a release-control fact starts living in a design doc, the design doc
# should be added to this list, and any retraction-narrative lines in it
# get explicit [retraction-context] markers per the discipline rule.
RELEASE_DOCS=(
  "docs/r2-structure.md"
  "docs/r3-structure.md"
  "docs/thesis/r2-r3-thesis-mapping.md"
)

# Forbidden lane/concept names. These were retracted but kept reappearing
# in sibling docs across the #1078 review loop. Adding to this list:
# any lane or concept the project explicitly retracts as live framing.
#
# Format: each entry is a literal string. The consumer checks each
# release doc and reports lines that contain the string AND are not in
# retraction context.
FORBIDDEN_STRINGS=(
  "T-Ground-Engine"
  "T-Ground-Annotation"
  "canonical choice"
  "@target"
  "DECISIONS LOCKED"
  # Added 2026-04-28 per codex finding on PR #1078 commit ca9b4c66:
  # T-Verification-L4L7 was the prior single-lane name; r3-structure.md
  # split into T-Verification-L4-L7-Direct (L4+L7 Evaluator-direct) and
  # T-Verification-L5-Corpus (L5 corpus-driven). The thesis-mapping doc
  # had stale T-Verification-L4L7 in 8 per-claim rows that the v1
  # consumer didn't catch — exactly the structured-state-drift class
  # named as v2-guardrail follow-up. Adding here keeps the v1 string
  # check fresh while v2 is in flight.
  "T-Verification-L4L7"
)

# Lines that count as "retraction context" — if a line containing a
# forbidden string also matches one of these patterns, it's allowed
# (the doc is explaining the supersession, not declaring live framing).
RETRACTION_PATTERNS=(
  # NARROW set per claude-opus-4-7 review feedback (2026-04-28):
  # broad patterns neuter the check. Retraction context must be
  # explicit markers — strikethrough, SUPERSEDED/RETRACTED/CLOSED with
  # 🔄 marker, or "the retracted X" phrasing. Lines mentioning a
  # forbidden string without one of these markers MUST get an explicit
  # marker added or be rewritten.
  "~~"                            # strikethrough markdown
  "🔄"                            # supersession/retraction/closure emoji
  "SUPERSEDED"
  "[Ss]upersedes"
  "RETRACTED"
  "[Rr]etracted"
  "CLOSED 2026"                   # explicit "CLOSED <date>" marker (Open call closures)
  "[Rr]eplaces the retracted"
  "the retracted"                 # "the retracted T-Ground-X lane"
  "supersession"
  "supersedes the prior"
  "framing was retracted"
  # Explicit author marker for legitimate retrospective prose that
  # discusses retracted concepts inline. Use this sparingly — it's
  # the author opting out of the narrow check. Form: [retraction-context]
  "\[retraction-context"  # matches [retraction-context] and [retraction-context: explanation]
)

violations=0

for doc in "${RELEASE_DOCS[@]}"; do
  if [ ! -f "$doc" ]; then
    continue
  fi

  for forbidden in "${FORBIDDEN_STRINGS[@]}"; do
    # Find all lines containing the forbidden string.
    while IFS= read -r match; do
      [ -z "$match" ] && continue

      lineno="${match%%:*}"
      content="${match#*:}"

      # Check if the line is in retraction context.
      is_retraction=0
      for pattern in "${RETRACTION_PATTERNS[@]}"; do
        if echo "$content" | grep -qiE "$pattern"; then
          is_retraction=1
          break
        fi
      done

      if [ "$is_retraction" = "0" ]; then
        echo "VIOLATION: $doc:$lineno"
        echo "  forbidden string: '$forbidden'"
        echo "  line: $content"
        echo ""
        violations=$((violations + 1))
      fi
    done < <(grep -n -F "$forbidden" "$doc" || true)
  done
done

if [ "$violations" -gt 0 ]; then
  echo "Release-doc authority check FAILED: $violations violation(s) found."
  echo ""
  echo "Each violation is a forbidden lane/concept name appearing in a live"
  echo "(non-retraction) context. Either:"
  echo "  - the lane/concept is genuinely live, in which case remove it from"
  echo "    FORBIDDEN_STRINGS in this script and update authority docs accordingly"
  echo "  - the lane/concept is retracted, in which case add a retraction marker"
  echo "    (~~strikethrough~~, 🔄 SUPERSEDED, RETRACTED, 'replaces the retracted',"
  echo "    or similar) to the line, OR remove the line entirely."
  echo ""
  echo "Authority: docs/r2-structure.md §\"Release-doc authority discipline\""
  exit 1
fi

echo "Release-doc authority check passed: no forbidden stale concept names"
echo "in live sections of release-control docs."
