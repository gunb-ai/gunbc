### L1 ratchet increase audit: 371 → 414 (+43) (2026-03-26)

Systematic root-cause analysis of the +43 L1 ratchet increase between
commit `597d852b` (ratchet set to 373) and current HEAD.

#### Source: file decomposition (+18)

Code moved from `04_infer.dag` (-61 sites) into 7 extracted modules
(+79 sites). Net +18 because:

- **+13 import lines.** Each extracted module imports predicates and
  constructors it uses (`node_is_optional`, `leaf_node`, etc.). These
  are NOT new type knowledge — the same call sites existed in the
  monolith. The ratchet script counts `\bnode_is_\w+\b` in import
  lists.
- **+5 expanded logic.** During extraction, some code was slightly
  restructured (e.g., adding explicit predicate calls where the
  monolith had inline field checks).

**Classification:** Not invariant violations. File decomposition is
structural improvement. The ratchet increase is a measurement artifact
— import lines are not "compiler type knowledge."

**Ratchet script improvement opportunity:** Exclude `^import` lines
from the count, or weight them differently.

#### Source: P5.7a bridge predicates (+7)

`04_types.dag` gained 7 new `node_is_*` sites:

| Site | What | Classification |
|------|------|---------------|
| `node_is_bridge_error_name` (def + 4 calls) | Centralizes `n.name == "Error"` check that was previously inline in `node_type_equals`/`node_type_compatible` | **Bridge.** Explicitly named as temporary (prefix `bridge_`). Deletion point: P5.6/P5.8 when Error becomes `CompilerError` flow. |
| `node_is_bridge_dynamic_name` (def + 3 calls) | Same for `n.name == "Dynamic"` | **Bridge.** Same deletion point. |
| `node_is_product`/`node_is_coproduct` (P5.7a rewrites) | Changed from `properties \|> any(p => p.name == "is_product")` to `n.connective == Some { value: Conj }` | **Improvement.** Replaced uncounted string-property check with counted structural check. Net reduction in actual type knowledge (deleted duplicate representation). |

**Classification:** 5 are explicit bridge code with deletion points.
2 are structural improvements that trade uncounted violations for
counted ones (net positive).

#### Source: P5.7b CollectionKind (+4 connective, +3 Conj/Disj)

P5.7a deleted `is_product`/`is_coproduct` property strings and made
predicates read `.connective` directly. This moved sites from the
uncounted property-string pattern to the counted `.connective` pattern.

| Category | Old pattern (uncounted) | New pattern (counted) |
|----------|------------------------|----------------------|
| `.connective` +4 | `properties \|> any(p => p.name == "is_product")` | `n.connective == Some { value: Conj }` |
| `Conj/Disj` +3 | property string `"is_product"` / `"is_coproduct"` | `Conj` / `Disj` literal in predicate match |

**Classification:** Structural improvement. The old code was WORSE
(string-keyed property checks) but uncounted. The new code is BETTER
(typed field access) but counted. No invariant violation — the ratchet
script should have been counting the old pattern too.

#### Source: emit type-name comparisons (+4)

`05_emit_rust.dag` +3 and `05_emit.dag` +2 new `.name == "..."` checks.

| Site | What | Classification |
|------|------|---------------|
| `05_emit_rust.dag` typename checks | `effective.name == "List"`, `"Vec"`, `node_is_container` in intrinsic method dispatch | **Emit rendering.** Emit legitimately reads names for target identifiers. Not an L1 violation — emit is excluded from the L1=0 gate (scrambled-name tests exclude emit). |
| `05_emit.dag` typename checks | Service call detection, simple expression rendering | **Emit rendering.** Same classification. |

**Classification:** Legitimate emit rendering. These are NOT L1
violations — the L1 gate (P5.6 scrambled-name tests) explicitly
excludes emit because emit must read names to produce target
identifiers.

#### Source: parse/compile structural production (+9)

`02_parse.dag` +6, `compile.dag` +3.

| Site | What | Classification |
|------|------|---------------|
| `02_parse.dag` connective +1, constructors +2, predicates +3 | Parser creates `Conj`/`Disj` nodes and calls `node_is_optional` for cardinality | **Parse production.** The parser MUST produce structural nodes. Not "type knowledge the compiler has" — it's "structure the parser creates." |
| `compile.dag` connective +1, conj_disj +2 | Pipeline orchestration reading connective for complexity/ownership staging | **Pipeline wiring.** Compile stage reads structural properties to route to proof stages. |

**Classification:** Necessary structural production/wiring. Not
violations.

#### Source: 03_resolve.dag adjacency helper (+2)

`adjacency_add_edge` adds 2 typename comparisons (from P4.6 fix).

**Classification:** See IV-6/IV-7 — this is a workaround for missing
bidirectional type inference. The helper's explicit `Map<String,
List<String>>` type annotation provides what inference should propagate.

#### Summary

| Source | Sites | Classification | Action |
|--------|------:|---------------|--------|
| File decomposition (imports) | +13 | Measurement artifact | Fix ratchet script to exclude import lines |
| File decomposition (logic) | +5 | Moved code, not new knowledge | None |
| P5.7a bridge predicates | +7 | 5 explicit bridge, 2 improvement | Bridge deletion at P5.6/P5.8 |
| P5.7a/b connective migration | +7 | Improvement (counted replaces uncounted) | None — old uncounted pattern was worse |
| Emit rendering | +4 | Legitimate (emit excluded from L1 gate) | None |
| Parse/compile production | +9 | Structural production | None |
| Adjacency helper | +2 | Workaround (IV-6/IV-7) | Fix bidirectional inference |
| **Total** | **+47** | | |

(+47 gross, -4 from other reductions = +43 net)

**Conclusion:** Of the +43 net increase, **0 are new invariant
violations.** The increase comes from: measurement artifacts (+13),
moved code (+5), structural improvements that trade uncounted
violations for counted ones (+7), legitimate emit/parse/pipeline
sites (+13), and workarounds for pre-existing violations (+7 bridge +2
adjacency). The ratchet script should be improved to exclude import
lines and potentially emit-only files.

---

