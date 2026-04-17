#!/usr/bin/env bash
# Banked-dissolutions ratchet (Lane 1 Stage 1a).
#
# Scans lane and phase docs for shapes that the design-blocker set
# (DB-1…DB-10) has formally rejected. The single authority is the
# `FORBIDDEN=(...)` array in docs/post-l15-phase-plan.md § "Banked
# dissolutions — rejected shapes" — this script PARSES that block
# directly instead of hardcoding a second list.
#
# Exempt from the scan:
#   docs/design-*.md         — DB docs record the rejections
#   docs/post-l15-phase-plan.md — master plan carries the ratchet itself
#
# Usage: scripts/check-banked-dissolutions.sh
#   Exits 0 when clean; exits 1 with a report on any match.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

MASTER_PLAN="docs/post-l15-phase-plan.md"

if [ ! -f "$MASTER_PLAN" ]; then
  echo "banked-dissolutions: missing master plan $MASTER_PLAN" >&2
  exit 1
fi

# Parse the authoritative FORBIDDEN array from the master plan. awk
# isolates the lines between `FORBIDDEN=(` and `)`; the shell regex
# pulls each quoted string out. The master plan's block is the single
# source of truth — see the plan's § Banked dissolutions for rationale.
FORBIDDEN=()
while IFS= read -r line; do
  while [[ $line =~ \"([^\"]+)\" ]]; do
    FORBIDDEN+=("${BASH_REMATCH[1]}")
    line="${line/${BASH_REMATCH[0]}/}"
  done
done < <(awk '/^FORBIDDEN=\($/,/^\)$/' "$MASTER_PLAN")

if [ "${#FORBIDDEN[@]}" -eq 0 ]; then
  echo "banked-dissolutions: could not extract FORBIDDEN array from $MASTER_PLAN" >&2
  echo "The master plan's § Banked dissolutions block must contain a bash-style \`FORBIDDEN=(...)\` array." >&2
  exit 1
fi

shopt -s nullglob
FILES=()
for pattern in docs/lane*.md docs/phase*.md; do
  for file in $pattern; do
    case "$file" in
      docs/post-l15-phase-plan.md) ;;
      docs/design-*.md) ;;
      *) FILES+=("$file") ;;
    esac
  done
done
shopt -u nullglob

if [ ${#FILES[@]} -eq 0 ]; then
  echo "banked-dissolutions: no lane/phase docs to scan"
  exit 0
fi

violations=0
for pat in "${FORBIDDEN[@]}"; do
  hits=$(grep -nHF -- "$pat" "${FILES[@]}" 2>/dev/null || true)
  if [ -n "$hits" ]; then
    if [ "$violations" -eq 0 ]; then
      echo "banked-dissolutions ratchet: forbidden shapes found in lane/phase docs."
      echo "Authority: $MASTER_PLAN § Banked dissolutions."
      echo
    fi
    echo "--- forbidden: $pat ---"
    echo "$hits"
    echo
    violations=$((violations + 1))
  fi
done

if [ "$violations" -gt 0 ]; then
  echo "banked-dissolutions ratchet: $violations forbidden shape(s) found."
  echo "Fix: delete the restatement and reference the DB doc instead."
  exit 1
fi

echo "banked-dissolutions ratchet: clean (${#FILES[@]} docs scanned, ${#FORBIDDEN[@]} forbidden shapes from $MASTER_PLAN)"
