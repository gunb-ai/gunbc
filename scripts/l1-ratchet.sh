#!/usr/bin/env bash
# L1 Type Knowledge Dissolution Ratchet
#
# Counts compiler-side type knowledge that should eventually live in .dag.
# Run after any .dag change to ensure L1 violations don't increase.
#
# Two sections:
#   L1 violations — type-world knowledge the compiler should not have
#   Structural primitives — graph primitives that are permanent (not violations)
#
# Usage: scripts/l1-ratchet.sh [--check]
#   Without --check: prints counts
#   With --check: prints counts and fails if L1 total exceeds ratchet

set -euo pipefail

V2_DIR="$(cd "$(dirname "$0")/../src/v2" && pwd)"
DAG_FILES=("$V2_DIR"/*.dag)

count_matches() {
    local pattern="$1"
    local n
    n=$( (grep -cE "$pattern" "${DAG_FILES[@]}" || true) | awk -F: '{s+=$NF} END {print s+0}')
    echo "$n"
}

# ── L1 Violations (target: 0) ──────────────────────────────────────────

# Type constructors (leaf_node, container_node, map_node, tuple_node, etc.)
constructor_count=$(count_matches '\b(leaf_node|optional_node|container_node|tuple_node|pair_node|callable_node|error_type_node)\b')

# Type-name string comparisons (.name == "Optional", "Map", etc.)
typename_count=$(count_matches '\.name == "(Optional|Map|List|Set|Dynamic|Error|Int|String|Bool|Float|Unit|Bytes|Json|Secret|Tuple|Callable|None|Some)"')

l1_total=$((constructor_count + typename_count))

# ── Type Constructors (Layer -1, not yet modeled in std) ────────────────

# .connective direct field access (Product/Coproduct constructor reads)
connective_field_count=$(count_matches '\.connective\b')

# Conj/Disj/NoConnective references (constructor enum — dissolves when
# Product/Coproduct are modeled in dsl/std/constructors.dag)
conj_disj_count=$(count_matches '\b(Conj|Disj|NoConnective)\b')

constructor_primitive_total=$((connective_field_count + conj_disj_count))

# ── Bridge Debt (tracked, has deletion point) ───────────────────────────

# CollectionKind references (bridge — dissolves when method algebras land)
collection_kind_count=$(count_matches '\b(collection_kind|ListKind|SetKind|NonEmptyListKind|NonEmptySetKind|MapKind|NoCollection)\b')

echo "L1 Type Knowledge Ratchet"
echo "========================="
echo ""
echo "  L1 Violations (target: 0)"
echo "  -------------------------"
echo "  Type constructors:             $constructor_count"
echo "  Type-name comparisons:         $typename_count"
echo "  ---"
echo "  L1 Total:                      $l1_total"
echo ""
echo "  Type Constructors (Layer -1, not yet in std)"
echo "  ---------------------------------------------"
echo "  .connective access:            $connective_field_count"
echo "  Conj/Disj/NoConnective:        $conj_disj_count"
echo "  ---"
echo "  Constructor Total:             $constructor_primitive_total"
echo ""
echo "  Bridge Debt (has deletion point)"
echo "  --------------------------------"
echo "  CollectionKind:                $collection_kind_count"

# L1 gate: type constructors and name comparisons must be 0.
# Constructor functions (container_node, tuple_node, callable_node, etc.)
# and type-name string comparisons were dissolved in PR #352.
# This is now a hard gate — any reintroduction is a regression.

if [[ "${1:-}" == "--check" ]]; then
    if (( l1_total > 0 )); then
        echo ""
        echo "FAIL: L1 gate violated ($l1_total > 0)"
        echo "Type constructors and name comparisons must stay at 0."
        exit 1
    else
        echo ""
        echo "OK: L1 gate clean ($l1_total == 0)"
    fi
fi
