#!/usr/bin/env bash
# Compiler-std consolidation ratchet (#642).
#
# Mechanical definition from docs/thesis/compiler-std-consolidation.md:
#   1. Count `^type [A-Z]` across `src/v3/compiler/*.dag` and
#      `src/v3/lenses/*.dag`.
#   2. Subtract positive-definition rows.
#   3. Subtract exempt rows.
#   4. Fail CI if the remaining tracked total grows above the baseline.
#
# The row lists below are the explicit in-tree classification required to
# make the thesis rule actionable in CI. The script also fails closed if
# any counted `type` row on those surfaces is left unclassified.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

BASELINE_TRACKED_TOTAL=23

SURFACES=(
  src/v3/compiler/*.dag
  src/v3/lenses/*.dag
)

POSITIVE_ROWS=(
  "src/v3/compiler/pipeline.dag:13"
  "src/v3/compiler/pipeline.dag:17"
  "src/v3/compiler/pipeline.dag:19"
  "src/v3/compiler/regen.dag:38"
  "src/v3/lenses/complexity.dag:45"
  "src/v3/lenses/cost.dag:23"
  "src/v3/lenses/infer_helpers.dag:42"
  "src/v3/lenses/infer_helpers.dag:65"
  "src/v3/lenses/infer_helpers.dag:84"
  "src/v3/lenses/provenance.dag:61"
  "src/v3/lenses/structural_resolution.dag:37"
  "src/v3/lenses/structural_resolution.dag:43"
  "src/v3/lenses/unused_parameters.dag:16"
  "src/v3/lenses/variant_payload.dag:22"
  "src/v3/lenses/variant_payload.dag:36"
)

EXEMPT_ROWS=(
  "src/v3/compiler/parse_tables.dag:109"
  "src/v3/compiler/parse_tables.dag:143"
  "src/v3/compiler/parse_tables.dag:265"
  "src/v3/compiler/parse_tables.dag:340"
)

collect_type_rows() {
  local pattern
  for pattern in "${SURFACES[@]}"; do
    grep -nH -E '^type [A-Z]' $pattern 2>/dev/null || true
  done
}

TYPE_ROWS="$(collect_type_rows)"

if [ -z "$TYPE_ROWS" ]; then
  echo "compiler-std ratchet: no type rows found on the configured surfaces"
  exit 1
fi

TOTAL_TYPE_ROWS=$(printf '%s\n' "$TYPE_ROWS" | sed '/^$/d' | wc -l | tr -d ' ')
POSITIVE_COUNT=${#POSITIVE_ROWS[@]}
EXEMPT_COUNT=${#EXEMPT_ROWS[@]}
CLASSIFIED_COUNT=$((POSITIVE_COUNT + EXEMPT_COUNT))

is_listed_row() {
  local row="$1"
  shift
  local listed
  for listed in "$@"; do
    if [ "$row" = "$listed" ]; then
      return 0
    fi
  done
  return 1
}

UNCLASSIFIED_ROWS=()
while IFS= read -r row; do
  [ -n "$row" ] || continue
  row="${row%%:*}:${row#*:}"
  row="${row%%:*}:${row#*:}"
done < /dev/null

while IFS= read -r raw; do
  [ -n "$raw" ] || continue
  row_id="$(printf '%s\n' "$raw" | cut -d: -f1,2)"
  if ! is_listed_row "$row_id" "${POSITIVE_ROWS[@]}" && ! is_listed_row "$row_id" "${EXEMPT_ROWS[@]}"; then
    UNCLASSIFIED_ROWS+=("$raw")
  fi
done <<EOF
$TYPE_ROWS
EOF

if [ "${#UNCLASSIFIED_ROWS[@]}" -gt 0 ]; then
  echo "compiler-std ratchet: unclassified type rows found on tracked surfaces."
  echo "Each counted row must be classified as positive-def, exempt, or tracked debt before CI can evaluate the ratchet."
  echo
  printf '%s\n' "${UNCLASSIFIED_ROWS[@]}"
  exit 1
fi

TRACKED_TOTAL=$((TOTAL_TYPE_ROWS - CLASSIFIED_COUNT))

echo "Compiler-std consolidation ratchet"
echo "=================================="
echo "Tracked surfaces: src/v3/compiler/*.dag + src/v3/lenses/*.dag"
echo "Counted rows:     $TOTAL_TYPE_ROWS"
echo "Positive-def:     $POSITIVE_COUNT"
echo "Exempt:           $EXEMPT_COUNT"
echo "---"
echo "Tracked total:    $TRACKED_TOTAL"
echo "Baseline:         $BASELINE_TRACKED_TOTAL"

if [ "$TRACKED_TOTAL" -gt "$BASELINE_TRACKED_TOTAL" ]; then
  echo
  echo "::error::compiler-std ratchet grew: tracked total $TRACKED_TOTAL > baseline $BASELINE_TRACKED_TOTAL"
  echo "Action:"
  echo "  1. Move the new type to std/, OR"
  echo "  2. Add an explicit positive-def / exemption classification if #642 already covers it, OR"
  echo "  3. Update docs/thesis/compiler-std-consolidation.md and ROADMAP.md together if the policy itself changed."
  exit 1
fi

echo
echo "compiler-std ratchet: clean ($TRACKED_TOTAL <= $BASELINE_TRACKED_TOTAL)"
