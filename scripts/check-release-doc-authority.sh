#!/usr/bin/env bash
# Release-doc authority consumer — checks for forbidden stale concept
# names in live (non-retraction-context) sections of release-control docs.
#
# Authority: docs/r2-structure.md §"Release-doc authority discipline".
#
# Scope: catches forbidden-string drift only. Structured-state drift
# (lane counts, dependency graphs, thesis-claim dispositions) is named
# v2-guardrail follow-up — see r2-structure.md for the requirements list.
#
# Exit codes:
#   0 — no violations
#   1 — at least one forbidden string in a live (non-retraction) context

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Release-control docs (carry live program / lane / state authority).
# Design docs are intentionally out of scope — broadening this list would
# force every retraction-narrative line to carry an explicit marker.
RELEASE_DOCS=(
  "docs/r2-structure.md"
  "docs/r3-structure.md"
  "docs/thesis/r2-r3-thesis-mapping.md"
)

# Forbidden lane/concept names — each entry is a literal string the
# project has retracted as live framing. Add an entry when a name is
# retracted; remove it only if the underlying concept becomes live again.
#
# Match semantics: literal (grep -F), NOT regex. Don't add regex
# metacharacters expecting them to be honored — they'll match literally.
# (RETRACTION_PATTERNS below uses grep -qiE for regex semantics; the
# asymmetry is intentional — forbidden strings are exact lane/concept
# names, retraction patterns are markers that may need character-class
# flexibility like [Rr]etracted.)
#
# Scope: this list applies only to RELEASE_DOCS above. Design docs
# (e.g., docs/design-emission-model.md, docs/design-lens-framework.md)
# are explanatory authorities and intentionally out of scope, per the
# RELEASE_DOCS scope comment. A retracted name appearing in a design
# doc's narrative does not violate this consumer; if release-control
# authority migrates into a design doc, add the design doc to
# RELEASE_DOCS at the same time.
FORBIDDEN_STRINGS=(
  "T-Ground-Engine"
  "T-Ground-Annotation"
  "canonical choice"
  "@target"
  "DECISIONS LOCKED"
  "T-Verification-L4L7"
)

# Lines that count as "retraction context" — if a line containing a
# forbidden string also matches one of these patterns, it's allowed
# (the doc is explaining the supersession, not declaring live framing).
# Patterns are deliberately narrow; broad patterns neuter the check.
RETRACTION_PATTERNS=(
  "~~"                            # strikethrough markdown
  "🔄"                            # supersession/retraction/closure emoji
  "SUPERSEDED"
  "[Ss]upersedes"
  "RETRACTED"
  "[Rr]etracted"
  "CLOSED 2026"                   # explicit CLOSED-<date> marker
  "[Rr]eplaces the retracted"
  "the retracted"
  "supersession"
  "supersedes the prior"
  "framing was retracted"
  # Explicit author marker for legitimate retrospective prose that
  # discusses retracted concepts inline. Use this sparingly — it's
  # the author opting out of the narrow check. Form: [retraction-context]
  "\[retraction-context"  # matches [retraction-context] and [retraction-context: explanation]
)

violations=0
missing_docs=()

# Fail-closed precheck: every configured release doc must exist. A
# missing doc means the consumer's declared authority surface has
# silently shrunk — exactly the failure mode the guardrail is supposed
# to prevent. If a release doc is genuinely retired, remove it from
# RELEASE_DOCS above with the same review attention as adding one.
for doc in "${RELEASE_DOCS[@]}"; do
  if [ ! -f "$doc" ]; then
    missing_docs+=("$doc")
  fi
done

if [ "${#missing_docs[@]}" -gt 0 ]; then
  echo "Release-doc authority check FAILED: ${#missing_docs[@]} configured release-control doc(s) missing."
  echo ""
  for doc in "${missing_docs[@]}"; do
    echo "  MISSING: $doc"
  done
  echo ""
  echo "Each missing doc was declared in RELEASE_DOCS in this script. The"
  echo "consumer fails closed because silently skipping a missing doc would"
  echo "let release-control authority shrink without review. Either:"
  echo "  - the doc was renamed → update RELEASE_DOCS to the new path"
  echo "  - the doc was retired → remove it from RELEASE_DOCS with"
  echo "    the same attention as adding a new release-control authority"
  exit 1
fi

for doc in "${RELEASE_DOCS[@]}"; do
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
