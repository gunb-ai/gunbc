# Coercion/Modeling Adoption Measurement: Pipeline Stages

**Status:** measurement report (read-only — no implementation in this lane)  
**Work item:** `node://adhoc-d11d2756-63a`  
**Session:** calm-badger-881  
**Gating PRs:** #4585 (find_witness completion rules), fold_node catamorphism in std/node.dag  

## Summary

Now that `find_witness` (#4585) and `fold_node` exist as first-class v4 std primitives, this
report measures HOW CLOSE each v4 pipeline stage is to consuming them vs hand-rolling coercion
or tree-traversal logic.

**Verdict:** `06_translate` is the only stage that has crossed the threshold. All upstream stages
(02–05) are structurally pre-coercion — by design for some (parse, normalize), by gap for others
(04_infer, 03_resolve).

---

## Per-Stage Measurement

### 06_translate.dag (4490L) — ADOPTED ✅

**find_witness:** used (imports `CandidateSet`; delegates via `coercion_fold_with_declared_priority`)  
**fold_node:** 4 call sites (L272, L4255, L4407, L4420 — excludes 1 import line, 1 comment)  
**Hand-rolled coercion:** none — `coerce_grounded_node` routes through `std/coercion.dag` which
lifts `FindWitnessResult → CoercionResult`. The 49 hits of `coerce*` in this file are call-sites
of `coerce_grounded_node` and `translate_coerced_*`, not bypass logic.  
**Gap:** none for current T-10 scope. Future: `translate_coerced_shell_at_use_site_from_source`
hand-matches on node structure (L3894–3936); could fold when Outcome-bearing fold_node algebra
lands (Ratified Q4 gate in std/diagnostic.dag).

---

### 04_infer.dag (566L) — PARTIAL 🟡

**find_witness:** 0 call sites  
**fold_node:** 2 call sites (L267, L537 — excludes 1 import line, 1 comment)  
**Hand-rolled coercion:** none — infer does not perform coercion; it _produces_ `CanonicalGrounding`
(the source facts) that translate's `coerce_grounded_node` later consumes.  
**Gap:** By design. `04_infer` is the grounding authority, not a coercion consumer. The
`canonical_grounding_admits_infer_facts` admission gate uses `well_formed` not `find_witness`.
If a call-arg type-compatibility check (enforcement AIM, see memory) ever lands in infer,
it _should_ call `find_witness` (or `coercion_fold`) to check assignability — that is the
remaining adoption gap. Currently blocked on enforcement AIM operator GO.

---

### 03_resolve.dag (590L) — PARTIAL 🟡

**find_witness:** 0 call sites  
**fold_node:** 1 call site (L145 — `harvest_direct_atom_binding`; excludes 1 import line)  
**Hand-rolled coercion:** none — resolve is name-resolution, not coercion  
**Match-on-node-kind:** 55 hits  
**Gap:** No coercion gap; fold_node adoption is partial. The 55 node-kind matches are in
resolution predicates that manually walk the tree. These are candidates for `fold_node`
algebra replacement when the traverse-node-outcome-algebra feature gate clears (Ratified Q4).
Not a coercion gap.

---

### 03_normalize.dag (224L) — NOT APPLICABLE for coercion; PARTIAL for fold_node 🟡

**find_witness:** 0 (N/A — sugar dissolution, not coercion)  
**fold_node:** 0  
**Match-on-node-kind:** 17 hits  
**Gap:** The 17 node-kind matches in `classify_sugar` and `normalize_sugar` are sugar-to-kernel
rewrites. They are structurally tree-recursive but the rewrite semantics differ from a fold
algebra. A `fold_node` refactor is possible but only with an Outcome-carrying algebra
(blocked on Ratified Q4). Not a coercion gap.

---

### 02_parse.dag (893L) — NOT APPLICABLE ⬜

**find_witness:** 0 (N/A — parser produces AST, no coercion decisions)  
**fold_node:** 0  
**Match-on-node-kind:** 7 hits (grammar rule matches in parse_expr)  
**Gap:** None for coercion. The 7 node matches are parse dispatch, not coercion. fold_node
does not apply to a recursive descent parser producing nodes bottom-up.

---

### 05_emit.dag (42L) — STUB ⬜

**find_witness:** 0  
**fold_node:** 0  
**Gap:** Stage is a thin orchestration wrapper: `serialize_source_for_emitted` + `emit`. No
coercion or node-traversal logic in scope. Downstream of translate's coercion.

---

### emit_host.dag — ORCHESTRATOR ⬜

**find_witness:** 0  
**fold_node:** 0  
**Gap:** Orchestration only (emit → eval → compare receipts). Not a coercion consumer.

---

### 04_types (v2) / lower (v3 prototype) — OUT OF SCOPE for v4 primitives ⬜

`v2/04_types.dag` and `v3/lenses/lower_helpers.dag` run against v2/v3 std, which does not
expose `find_witness` or `fold_node`. Adoption is impossible until those stages migrate to v4
std. v2 has 0 find_witness + 0 fold_node across all 23 .dag files (by design — frozen bootstrap
seed, get-off-v3 goal).

---

## Adoption Scoreboard

| Stage | find_witness | fold_node | Coercion gap | fold gap |
|---|---|---|---|---|
| 06_translate | ✅ via coercion.dag | ✅ 4 sites | none | partial (Q4 gate) |
| 04_infer | ⬜ N/A (grounding producer) | 🟡 2 sites | enforcement AIM (op-GO gated) | none |
| 03_resolve | ⬜ N/A (name resolution) | 🟡 1 site | none | partial (Q4 gate) |
| 03_normalize | ⬜ N/A (sugar dissolution) | ⬜ 0 sites | none | partial (Q4 gate) |
| 02_parse | ⬜ N/A | ⬜ 0 sites | none | none |
| 05_emit | ⬜ N/A | ⬜ 0 sites | none | none |
| v2/\* | ⬜ frozen bootstrap | ⬜ frozen | unblockable until v4 migration | same |

---

## Key Findings

1. **`find_witness` adoption is complete for the one stage that performs coercion.** No upstream
   stage has a coercion decision to route through it yet. The next adoption site is the
   enforcement call-arg check in `04_infer`, which is blocked on operator GO for the AIM design.

2. **`fold_node` adoption is partial in 03_resolve and 04_infer.** Both import and use it, but
   the majority of node-kind matches are still manual. Full fold adoption for these stages
   requires the Outcome-bearing fold algebra (Ratified Q4 gate in std/diagnostic.dag).

3. **No stage has hand-rolled coercion logic that should instead call `find_witness`.** The
   measurement finds no bypass — only staged adoption.

4. **v2 stages are permanently 0-adoption** until the get-off-v3 migration routes them through
   v4 std. This is by design (frozen bootstrap seed).

---

## Next Adoption Site

The next `find_witness` consumer in the pipeline is the direct-call arg-compatibility check
in `04_infer`. Its current status is tracked in `ROADMAP.md` under the `PD-3-DOGFOOD` row
(scaffold deletion criterion: `direct_call_arg_mismatch_diags` passes with zero false-positive
diags on v4/compiler substrate). Implementation design is out of scope for this measurement
report; consult ROADMAP.md for the checkable authority.
