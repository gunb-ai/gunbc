#!/usr/bin/env bash
# Release-doc authority consumer — checks for forbidden stale concept
# names in live sections of release-control docs.
#
# Authority: docs/r2-structure.md §"Release-doc authority discipline"
# (Open call 4, added 2026-04-28 per gpt-5-5-pro meta-review of PR #1078).
#
# Why: PR #1078 review loop (9 events, 83 minutes, 5 codex passes)
# kept catching the same P2 single-authority shape in new clothing —
# stale lane names persisting in sibling tables after the live framing
# was retracted. This script is the mechanical consumer that turns
# manual grep-as-a-service into a CI-checkable invariant.
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
RELEASE_DOCS=(
  "docs/r2-structure.md"
  "docs/r3-structure.md"
  "docs/design-emission-model.md"
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
)

# Lines that count as "retraction context" — if a line containing a
# forbidden string also matches one of these patterns, it's allowed
# (the doc is explaining the supersession, not declaring live framing).
RETRACTION_PATTERNS=(
  "supersede"
  "[Ss]upersedes"
  "SUPERSEDED"
  "retract"
  "[Rr]etracted"
  "RETRACTED"
  "retraction"
  "[Rr]eplaces"
  "[Rr]eplace.*with"
  "rename of"
  "rename"
  "→"          # transformation arrow
  "5 lanes"    # transformation count
  "5 substrate-completion lanes"
  "five lanes"
  "the prior"
  "framing was hiding"
  "was hiding"
  "was hidden"
  "framing was the wrong"
  "framing was retracted"
  "engine framing"  # describing the retracted framing, not declaring it
  "no separate.*engine"  # explicitly negating
  "No engine"
  "Engine reframe"
  "engine-reframe"
  "engine-shaped"
  "engine policy"  # describing the anti-pattern
  "the Engine framing"
  "no canonical"
  "canonical-choice"
  "fabrication"
  "anti-pattern"
  "~~"  # strikethrough markers
  "framing"  # generic reference to a framing, usually descriptive
  "row.*replace\|replace.*row"
  "substrate-completion lane"
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
