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

# Category 1: Connective field + Conj/Disj references
connective_count=$(grep -c '\bconnective\b' "${DAG_FILES[@]}" 2>/dev/null | awk -F: '{s+=$NF} END {print s+0}')

# Category 2: Type constructors (leaf_node, optional_node, container_node, tuple_node, etc.)
constructor_count=$(grep -cE '\b(leaf_node|optional_node|container_node|tuple_node|pair_node|callable_node|error_type_node)\b' "${DAG_FILES[@]}" 2>/dev/null | awk -F: '{s+=$NF} END {print s+0}')

# Category 3: Type-name string comparisons
typename_count=$(grep -cE '\.name == "(Optional|Map|List|Set|Dynamic|Error|Int|String|Bool|Float|Unit|Bytes|Json|Secret|Tuple|Callable|None|Some)"' "${DAG_FILES[@]}" 2>/dev/null | awk -F: '{s+=$NF} END {print s+0}')

# Category 4: node_is_* predicate calls
predicate_count=$(grep -cE '\bnode_is_\w+\b' "${DAG_FILES[@]}" 2>/dev/null | awk -F: '{s+=$NF} END {print s+0}')

# Category 5: builtin_type_kind() calls
builtin_count=$(grep -c '\bbuiltin_type_kind\b' "${DAG_FILES[@]}" 2>/dev/null | awk -F: '{s+=$NF} END {print s+0}')

total=$((connective_count + constructor_count + typename_count + predicate_count + builtin_count))

echo "L1 Type Knowledge Violations"
echo "============================"
echo "  Connective field + Conj/Disj:  $connective_count"
echo "  Type constructors:             $constructor_count"
echo "  Type-name comparisons:         $typename_count"
echo "  node_is_* predicates:          $predicate_count"
echo "  builtin_type_kind() calls:     $builtin_count"
echo "  ---"
echo "  Total:                         $total"

# Ratchet: total must not exceed this value
L1_RATCHET=454

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
