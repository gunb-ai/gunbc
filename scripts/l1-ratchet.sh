#!/usr/bin/env bash
# L1 Type Knowledge Dissolution Ratchet
#
# Counts compiler-side type knowledge that should eventually live in .dag.
# Run after any .dag change to ensure L1 violations don't increase.
#
# Usage: scripts/l1-ratchet.sh [--check]
#   Without --check: prints counts
#   With --check: prints counts and fails if total exceeds ratchet

set -euo pipefail

V2_DIR="$(cd "$(dirname "$0")/../src/v2" && pwd)"
DAG_FILES=("$V2_DIR"/*.dag)

count_matches() {
    local pattern="$1"
    local n
    n=$( (grep -cE "$pattern" "${DAG_FILES[@]}" || true) | awk -F: '{s+=$NF} END {print s+0}')
    echo "$n"
}

# Category 1a: .connective direct field access
connective_field_count=$(count_matches '\.connective\b')

# Category 1b: Conj/Disj references (connective shape matching)
conj_disj_count=$(count_matches '\b(Conj|Disj)\b')

# Category 2: Type constructors (leaf_node, optional_node, container_node, tuple_node, etc.)
constructor_count=$(count_matches '\b(leaf_node|optional_node|container_node|tuple_node|pair_node|callable_node|error_type_node)\b')

# Category 3: Type-name string comparisons
typename_count=$(count_matches '\.name == "(Optional|Map|List|Set|Dynamic|Error|Int|String|Bool|Float|Unit|Bytes|Json|Secret|Tuple|Callable|None|Some)"')

# Category 4: node_is_* predicate calls
predicate_count=$(count_matches '\bnode_is_\w+\b')

# Category 5: classify_type_structure calls in emit
classify_count=$(count_matches '\bclassify_type_structure\b')

# Category 6: builtin_type_kind() calls (target: 0, achieved)
builtin_count=$(count_matches '\bbuiltin_type_kind\b')

total=$((connective_field_count + conj_disj_count + constructor_count + typename_count + predicate_count + classify_count + builtin_count))

echo "L1 Type Knowledge Violations"
echo "============================"
echo "  .connective direct access:     $connective_field_count"
echo "  Conj/Disj references:          $conj_disj_count"
echo "  Type constructors:             $constructor_count"
echo "  Type-name comparisons:         $typename_count"
echo "  node_is_* predicates:          $predicate_count"
echo "  classify_type_structure calls:  $classify_count"
echo "  builtin_type_kind() calls:     $builtin_count"
echo "  ---"
echo "  Total:                         $total"

# Ratchet: total must not exceed this value.
# Categories and total are aligned with ROADMAP.md § Architectural Ratchet.
L1_RATCHET=373

if [[ "${1:-}" == "--check" ]]; then
    if (( total > L1_RATCHET )); then
        echo ""
        echo "FAIL: L1 ratchet exceeded ($total > $L1_RATCHET)"
        echo "Lower the ratchet or fix violations before committing."
        exit 1
    else
        echo ""
        echo "OK: $total <= $L1_RATCHET ratchet"
    fi
fi
