#!/usr/bin/env bash
# L1 Type Knowledge Dissolution Ratchet
#
# Counts compiler-facing .dag type knowledge under `src/v3/` that should
# eventually live in substrate-first form. Run after any `.dag` change to
# ensure L1 violations don't increase.
#
# Corpus: **all** `*.dag` files under `src/v3` (recursive). (`dsl/**` is
# excluded: meta-tool `.dag` files embed the grep patterns as string data and
# would false-positive the constructor pattern.)
#
# Usage: scripts/l1-ratchet.sh [--check]
#   Without --check: prints counts
#   With --check: prints counts and fails if L1 total exceeds ratchet

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

mapfile -d '' DAG_FILES < <(find "$ROOT/src/v3" -name '*.dag' -print0)

count_matches() {
    local pattern="$1"
    local n
    if ((${#DAG_FILES[@]} == 0)); then
        echo "0"
        return
    fi
    n=$( (grep -cE "$pattern" "${DAG_FILES[@]}" || true) | awk -F: '{s+=$NF} END {print s+0}')
    echo "$n"
}

# ── L1 Violations (target: 0) ──────────────────────────────────────────

constructor_count=$(count_matches '\b(leaf_node|optional_node|container_node|tuple_node|pair_node|callable_node|error_type_node)\b')
typename_count=$(count_matches '\.name == "(Optional|Map|List|Set|Dynamic|Error|Int|String|Bool|Float|Unit|Bytes|Json|Secret|Tuple|Callable|None|Some)"')

l1_total=$((constructor_count + typename_count))

# ── Type Constructors (Layer -1, not yet modeled in std) ────────────────

connective_field_count=$(count_matches '\.connective\b')
conj_disj_count=$(count_matches '\b(Conj|Disj|NoConnective)\b')

constructor_primitive_total=$((connective_field_count + conj_disj_count))

# ── Bridge Debt (tracked, has deletion point) ───────────────────────────

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
