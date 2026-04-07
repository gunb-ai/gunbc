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

# Ratchet: L1 violations must not exceed this value.
# 2026-03-28: 51→64 (enrich_kernel_type adds name checks for algebra
# field population of scalars + collections; deletion target once types
# are loaded from .dag declarations via FF-9)
# 2026-03-29: 64→66 (D3 ExprData String dissolution + D1 dissolution)
# 2026-03-29: 66→69 (merge from main: LintModel + FF-9 additions)
# 2026-03-30: 70→21 (data-table enrichment, structural access checks,
# and nominal/callable constructor consolidation)
# 2026-04-02: 22→24 (Bootstrap C: pattern_subject uses n.name=="Dynamic"/"Error",
# infer uses exp.name=="Map" — documented D6 debt)
# 2026-04-03: 24→32 (Bootstrap C: n_is_special Callable/Dynamic/Error in resolve,
# n.name=="List" in access, hardcoded True/False in patterns — tracked for M4)
# 2026-04-04: 31→30 (fix: CallableOf reuses existing callable_node, no net increase)
# 2026-04-05: 30→31 (TLC-1 zero-arg callable dispatch adds rt.name=="Callable" check)
# 2026-04-05: 31→33 (has_fn_fields: arity check→callable predicate, 2 new rt.name=="Callable" sites)
# 2026-04-06: 33→32 (TLC-1: ExprVar→ExprCall normalization dissolves is_zero_arg_callable_ref)
# 2026-04-06: 32→36 (Phase C infrastructure: has_type_variable, template_return_has_variables, unify/apply/build match on AlgebraTypeTemplate)
# 2026-04-06: 36→37 (Phase C bare-container enrichment: is_container_type+arity in apply_type_substitution)
# 2026-04-06: 37→36 (M4: Arrow connective dissolves n.name=="Callable" in render_node_type)
# 2026-04-06: 36→34 (M4: dissolve remaining 3 Callable name checks — emit_info, resolve)
# 2026-04-06: 34→31 (M4: dissolve 3 Tuple name checks — structural is_pair + tuple_type_name constant)
# 2026-04-06: 31→28 (M4: dissolve Dynamic/Error sentinel name checks — structural CompilerError check)
L1_RATCHET=28

if [[ "${1:-}" == "--check" ]]; then
    if (( l1_total > L1_RATCHET )); then
        echo ""
        echo "FAIL: L1 ratchet exceeded ($l1_total > $L1_RATCHET)"
        echo "Lower the ratchet or fix violations before committing."
        exit 1
    else
        echo ""
        echo "OK: L1 $l1_total <= $L1_RATCHET ratchet"
    fi
fi
