#!/usr/bin/env bash
# Compiler-std consolidation ratchet (#642).
#
# Mechanical definition from docs/thesis/compiler-std-consolidation.md:
#   1. Count `^type [A-Z]` across `src/v3/compiler/*.dag` and
#      `src/v3/lenses/*.dag`.
#   2. Subtract positive-definition rows.
#   3. Subtract exempt rows.
#   4. Fail CI unless the remaining tracked total matches the ratcheted
#      baseline exactly.
#
# The inventories below classify counted declarations by `file:type-name`
# rather than `file:line`, so the gate keys off declaration identity
# instead of incidental source layout. The script still fails closed if
# any counted `type` declaration on those surfaces is left unclassified.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

BASELINE_TRACKED_TOTAL=6

SURFACES=(
  src/v3/compiler/*.dag
  src/v3/lenses/*.dag
)

POSITIVE_ROWS=(
  "src/v3/compiler/pipeline.dag:CompilerHostRealization"
  "src/v3/compiler/pipeline.dag:PipelineSnapshotKind"
  "src/v3/compiler/pipeline.dag:PipelineStageBinding"
  "src/v3/compiler/regen.dag:LensRegistryEntry"
  "src/v3/lenses/complexity.dag:CostEntry"
  "src/v3/lenses/cost.dag:SymbolicCostEntry"
  "src/v3/lenses/infer_helpers.dag:TemplateArgumentBinding"
  "src/v3/lenses/provenance.dag:Origin"
  "src/v3/lenses/structural_resolution.dag:UnresolvedArrowBody"
  "src/v3/lenses/structural_resolution.dag:NameKeyedReference"
  "src/v3/lenses/unused_parameters.dag:UnusedParameter"
  "src/v3/lenses/variant_payload.dag:VariantPayloadShape"
  "src/v3/lenses/variant_payload.dag:VariantPayloadShapeLookup"
)

EXEMPT_ROWS=(
  "src/v3/compiler/parse_tables.dag:BinaryOpLevel"
  "src/v3/compiler/parse_tables.dag:BinaryOpRow"
  "src/v3/compiler/parse_tables.dag:TopLevelItemKwRow"
  "src/v3/compiler/parse_tables.dag:SoftKeywordIdentRow"
  "src/v3/compiler/parse_tables.dag:BracketRow"
  "src/v3/compiler/parse_tables.dag:PrimaryPrefixRow"
)

TRACKED_ROWS=(
  "src/v3/lenses/complexity.dag:CostLookup"
  "src/v3/lenses/cost.dag:SymbolicCostLookup"
  "src/v3/lenses/infer_helpers.dag:TemplateArgumentLookup"
  "src/v3/lenses/infer_helpers.dag:TemplateArgumentsMatch"
  "src/v3/lenses/infer_helpers.dag:TemplateArgumentCursor"
  "src/v3/lenses/infer_helpers.dag:NormalizedInstantiationArgs"
)

collect_type_rows() {
  local pattern
  for pattern in "${SURFACES[@]}"; do
    grep -nH -E '^type [A-Z]' $pattern 2>/dev/null || true
  done
}

row_key() {
  local raw="$1"
  local file rest type_name
  file="${raw%%:*}"
  rest="${raw#*:}"
  rest="${rest#*:type }"
  type_name="${rest%%[ {=]*}"
  printf '%s:%s\n' "$file" "$type_name"
}

TYPE_ROWS="$(collect_type_rows)"

if [ -z "$TYPE_ROWS" ]; then
  echo "compiler-std ratchet: no type rows found on the configured surfaces"
  exit 1
fi

TOTAL_TYPE_ROWS=$(printf '%s\n' "$TYPE_ROWS" | sed '/^$/d' | wc -l | tr -d ' ')
TRACKED_COUNT=${#TRACKED_ROWS[@]}
POSITIVE_COUNT=${#POSITIVE_ROWS[@]}
EXEMPT_COUNT=${#EXEMPT_ROWS[@]}
CLASSIFIED_COUNT=$((TRACKED_COUNT + POSITIVE_COUNT + EXEMPT_COUNT))

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

row_count_in_type_rows() {
  local target="$1"
  local count=0
  while IFS= read -r raw; do
    [ -n "$raw" ] || continue
    if [ "$(row_key "$raw")" = "$target" ]; then
      count=$((count + 1))
    fi
  done <<EOF
$TYPE_ROWS
EOF
  printf '%s\n' "$count"
}

verify_inventory_rows_exist_once() {
  local label="$1"
  shift
  local listed count
  local missing_or_duplicate=()
  for listed in "$@"; do
    count="$(row_count_in_type_rows "$listed")"
    if [ "$count" -ne 1 ]; then
      missing_or_duplicate+=("$listed (matches in live set: $count)")
    fi
  done
  if [ "${#missing_or_duplicate[@]}" -gt 0 ]; then
    echo "compiler-std ratchet: $label inventory drift."
    echo "Each classified declaration must exist exactly once in the live counted set."
    printf '%s\n' "${missing_or_duplicate[@]}"
    exit 1
  fi
}

verify_inventory_rows_exist_once "tracked" "${TRACKED_ROWS[@]}"
verify_inventory_rows_exist_once "positive-def" "${POSITIVE_ROWS[@]}"
verify_inventory_rows_exist_once "exempt" "${EXEMPT_ROWS[@]}"

UNCLASSIFIED_ROWS=()
while IFS= read -r raw; do
  [ -n "$raw" ] || continue
  row_id="$(row_key "$raw")"
  if ! is_listed_row "$row_id" "${TRACKED_ROWS[@]}" \
    && ! is_listed_row "$row_id" "${POSITIVE_ROWS[@]}" \
    && ! is_listed_row "$row_id" "${EXEMPT_ROWS[@]}"; then
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

if [ "$CLASSIFIED_COUNT" -ne "$TOTAL_TYPE_ROWS" ]; then
  echo "compiler-std ratchet: row classification drift."
  echo "Counted rows: $TOTAL_TYPE_ROWS"
  echo "Tracked rows: $TRACKED_COUNT"
  echo "Positive-def rows: $POSITIVE_COUNT"
  echo "Exempt rows: $EXEMPT_COUNT"
  echo "The explicit row inventory must cover the full counted surface."
  exit 1
fi

TRACKED_TOTAL=$TRACKED_COUNT

echo "Compiler-std consolidation ratchet"
echo "=================================="
echo "Tracked surfaces: src/v3/compiler/*.dag + src/v3/lenses/*.dag"
echo "Counted rows:     $TOTAL_TYPE_ROWS"
echo "Tracked debt:     $TRACKED_COUNT"
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
  echo "  2. Add an explicit positive-def / exemption classification if #642 already covers it."
  exit 1
fi

if [ "$TRACKED_TOTAL" -lt "$BASELINE_TRACKED_TOTAL" ]; then
  echo
  echo "::error::compiler-std ratchet improved without lowering the baseline: tracked total $TRACKED_TOTAL < baseline $BASELINE_TRACKED_TOTAL"
  echo "Action:"
  echo "  1. Lower BASELINE_TRACKED_TOTAL in this script to the new tracked total, AND"
  echo "  2. Update the matching baseline / exemption count in docs/thesis/compiler-std-consolidation.md and ROADMAP.md in the same PR."
  exit 1
fi

echo
echo "compiler-std ratchet: clean ($TRACKED_TOTAL == $BASELINE_TRACKED_TOTAL)"
