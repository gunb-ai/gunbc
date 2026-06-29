# Plan — complexity/linearity lens family whole-corpus (audit-first)

**Status:** audit landed · enforcement phased · **DESIGN.md + carriers are authority** (§2 horizontal + §1 cost axis; the expressibility-frontier frame is [expressibility-frontier.md](expressibility-frontier.md)). Linked from `ROADMAP.md §3` and [self-applying-lenses.md](self-applying-lenses.md).

**Verified against the live tree 2026-06-29** (executable census + floor witness). Line numbers are receipts; re-check before acting.

## 0. Verdict — one family, three representations, one blocker

The **complexity/linearity lens family** is the §1 time-axis pair (run-time cost vs change-time redundancy) unified under `v2.lens.intent_linearity` as a single registry engine ([self-applying-lenses.md](self-applying-lenses.md)). Detection is **total** over the kernel (`cost.dag` U2; anti-unification rows). The gate's **reach** is not: whole-corpus enforcement of `TermChain` / fn-body subjects is **blocked on fn-body reflection** — `fn_arrow_decl_facts_live()` enumerates fn *signatures* (params + output type), not resolved fn *bodies* as `Node`. Until that bridge lands (same dissolution trigger as `concept_index` host self-host: gunbc#5364), the honest posture is **roster + unit fixtures for bodies**, **live gates for import-graph**, and an **audit witness** that encodes what is wired today.

**Not inert.** The fail-closed-lockdown census (2026-06-21) labeled `complexity`/`cost` "inert" — meaning *no gate runs them on the tree*, not that the modules are unreached by floor discovery. Live census: **15+** discovered `*_test.dag` files import family lenses; the consolidated family floor witness (`family_floor_audit_holds_test.dag`) imports every top-level family module so the §6 inert-lens backstop stays green on this family.

## 1. Family inventory (authoritative modules)

| module | axis | representation | construction class | floor today |
| --- | --- | --- | --- | --- |
| `v2.lens.intent_linearity` | both (registry) | TermChain · TypeForest · ImportGraph | `WallAfterGrounding` | lens_unit (4 tests) + family audit |
| `v2.lens.simulated_relationship` | ChangeTime | TermChain (row 1: unroll→fold) | `WallAfterGrounding` | lens_unit discriminators |
| `v2.lens.structural_similarity` | ChangeTime | TypeForest (dup type→generic) | `WallAfterGrounding` | consumed by intent_linearity row; no standalone gate |
| `v2.lens.complexity_lowering` | RunTime | cost-recurrence catalog | `WallAfterGrounding` | catalog_sound + hoist unit tests |
| `v2.lens.cost` | RunTime | `fold_node` cost catamorphism | `WallNow` | unit tests under `v2/lens/cost/` |
| `v2.lens.complexity` | RunTime | budget dominance over bodies | `RatchetForever` (global optimality residue) | roster gate (4 subjects) + unit tests |
| `v2.lens.fact_density` | ChangeTime (under-side) | hollow carrier | `WallAfterGrounding` | separate compile/run gates; conceptual dual, not a registry row |

**Registry rows (intent_linearity):** 3 `LinearityRule` rows (`unroll_to_fold`, `complexity_reducible`, `dup_type_to_generic`) + 1 `ParallelRepresentationRule` (`consumed_input_closure` / ImportGraph). Authority: `intent_linearity.dag:58-81`.

**Complexity budget roster:** 4 subjects (`add`, `bind`, `branch`, `loop`) via `source_bridged_*_subject_producer` + `subject_complexity_budget_roster.dag`. Authority: `complexity_gate/`.

## 2. Expressibility-frontier partition (per representation)

| representation | decidable membership? | whole-corpus today? | blocker |
| --- | --- | --- | --- |
| **ImportGraph** | yes (`import_closure_live`) | **fixture only** (conformance closure) | needs per-module entry enumeration over `witness_layer_roots` — host bridge or v2 self-host compile-graph (#5364) |
| **TypeForest** | yes (`type_decls_anti_unify`) | **no** | `enumerate_concepts()` covers type decls, not cross-module duplicate clustering at scale; no whole-corpus gate |
| **TermChain** (change-time redundancy) | yes (anti-unify) | **no** | fn-body reflection |
| **TermChain** (run-time budget) | yes (`complexity_lens` ∘ `cost_lens`) | **roster (4)** | fn-body reflection |
| **global optimality** | no (Rice) | advisory forever | `complexity.dag` `RatchetForever` — correct |

Region ③ must not be priced as ① ([expressibility-frontier.md](expressibility-frontier.md) §4). The family's honest wall stops at anti-unification + finite lowering catalog; synthesis/optimality stays ratchet.

## 3. Floor wiring census (live, 2026-06-29)

| gate / witness | scope | discriminating? |
| --- | --- | --- |
| `complexity_gate/budget_roster_completeness_test` | 4-subject roster | yes (semantic RED on unrated budget) |
| `complexity_gate/source_bridged_*_budget_test` (×4) | per-subject budget dominate | yes |
| `intent_linearity/lens_unit/*` (×4) | registry discriminators + import graph algebra | yes |
| `module_graph/import_closure_live_test` | single conformance fixture | yes (under/over declared) |
| `simulated_relationship/lens_unit/discriminators_test` | chain_is_simulated | yes |
| `v2/lens/complexity/*_test` (×4) | cost-fold receipts | yes |
| `v2/lens/cost/*_test` (×3) | cost catamorphism | yes |
| `complexity_linearity/family_floor_audit_holds_test` | **family aggregate** (this audit) | yes (registry row count + catalog sound + roster import) |
| **whole-corpus fn bodies** | — | **missing** |

**Fn corpus without bodies:** `fn_arrow_decl_facts_live()` is live and used by `wiring_liveness_corpus_test` — but wiring liveness reads param/output structure only. Complexity/linearity on fn *bodies* cannot piggyback on it.

## 4. Enforcement phases (ordered)

1. **Audit (this doc + floor witness)** — partition family, name blocker, keep inert-lens hygiene green on all seven modules. *Done when `family_floor_audit_holds_test` is floor-discovered and green.*
2. **ImportGraph whole-corpus** — extend `import_closure_is_clean_live` / `dag_import_closure_equals_declared_*` from one conformance entry to every module under `witness_layer_roots` (host-fed path list beside `module_declaration_facts_live`, same seam as doc-graph reachability).
3. **TypeForest whole-corpus** — cluster `concept_decl_facts_live` shapes; flag `forest_is_redundant` clusters; roster exceptions for intentional parallel rows (e.g. `Compose<Int, MachineWidth<N>>` axis).
4. **TermChain whole-corpus** — `fn_body_facts_live() → Node` (or equivalent) feeding `complexity_lens` + `chain_is_redundant`; dissolves curated roster into enumeration. **Gated on fn-body reflection** (ROADMAP §3; same trigger as testgen affected-set completeness).
5. **Producer-applier** — registry rows that compute minimal form + write API ([self-applying-lenses.md](self-applying-lenses.md)); depends on emit + filesystem write + resolve-on-walk.

Phases 2–4 are independent of phase 5. Phase 4 is the ROADMAP §3 milestone ("complexity gates the whole codebase").

## 5. Relationship to adjacent lanes

- **[fail-closed-lockdown.md](fail-closed-lockdown.md)** — complexity violations are the same "modeled not enforced" class; this audit is the §3-specialized instance of CI-coverage-completeness for the family.
- **[self-applying-lenses.md](self-applying-lenses.md)** — consolidation map + bucket A/B/C classification; this audit does not duplicate it — it grounds that map in **floor receipts** and names the whole-corpus gap.
- **[wiring-liveness-preflight.md](wiring-liveness-preflight.md)** — wiring-liveness already runs whole-corpus over `fn_arrow_decl_facts_live`; proves the enumeration seam exists for *signatures*; bodies are the missing column.
- **[algebraic-rewrite-optimization.md](algebraic-rewrite-optimization.md)** — run-time rewrite *construction* is §5 expansion; §3 stays validation (budget dominate).

## Dissolution trigger (DESIGN §6)

Delete this doc when whole-corpus enforcement is live for every decidable representation in §2 (ImportGraph + TypeForest + TermChain budget/redundancy over all fn bodies under `witness_layer_roots`), with discriminating floor witnesses per representation, and the family audit witness has dissolved into those gates — at which point the executable gates are the authority and this audit prose is redundant.
