# Lens consolidation — the essence, the axes, and the stress-test rubric

> **The goal is to stop writing lenses.** The family has grown to ~40 distinct lenses across ~155 files; on the current trajectory it reaches 200, and 200 lens *modules* is the anti-pattern, not the win. The survey (2026-07-15) found the same handful of graph properties re-implemented under many names with forked machinery — which means the growth is mostly *redundant*, not new coverage. The target is the DESIGN §2 Realization move applied to the lenses themselves: **a small, closed set of invariant-kernels, where a new check is a DATA ROW, not a new module** — the same move that turned eleven hand-rolled caches into one `Compose` axis, and N×M parser/emitter pairs into N grammar rows. This doc fixes the **essence** of what a lens is, the **axes** the whole family reduces to (there are only four), and a **stress-test rubric** you run against any existing or proposed lens (and against the *set*) — whose *first question is always "can this be a row on an existing kernel instead of a new lens?"* It is a *rubric*, not a roster: it dissolves when the lenses are consolidated onto the shared kernel and verdict it prescribes (the mark on those carriers becomes the authority, DESIGN §6). DESIGN refs throughout: §1 (time), §2 (minimize redundancy), §3 (single authority), §4 (closed grounded acyclic substrate — a *closed* vocabulary is exactly what caps the count), §5 (fail-closed), §6 (lens-as-residue, priced in displaced cost), §7 (the compiler analyzes itself — the recursion these lenses must survive).

**The closure thesis (the whole point).** The substrate is 6 connectives + 5 behaviors — *closed*, and all variety lives in how they compose, not in new primitives. The lens set must be the same: a **closed set of ~4 invariant-kernels** (§2 below), and every conceivable check is expressed as `(projection, roots, invariant-handler) → shared verdict` **data** over one of them. New *subjects*, *grains*, and *verdicts* are rows; only a genuinely new *invariant* — and the survey found there are almost none left — warrants new machinery. If authoring a check requires a new `.dag` lens module rather than a row, that is the smell this rubric exists to catch. 200 lenses means the closure failed; a handful of kernels + a growing table of rows means it held.

---

## 1. The essence — what a lens *is*

A lens is a **pure reader over a projection of the one Node DAG that decides a graph invariant and emits a verdict on a shared lattice.** It stores nothing, so a new analysis costs zero substrate edits (§6, the residue mechanism). It is the tool of last resort, not first: DESIGN §5 is *construction first* — make the bad state **unwritable** (a single authority the realization derives from; a compile gate) — and reserve the lens for the genuinely **unstructurable residue** (undecidable, or not-yet-grounded). A lens that re-states a constraint the model already carries is itself a second representation (§2/§3), so the first question of any lens is *"why isn't this a construction wall?"*

Three consequences fix the shape:

- **One graph, many projections.** The subject is always a projection (edge-set) of the *single* Node DAG — call edges, dependency edges, cost edges, containment edges, reference edges — never a bespoke parallel fact-list that duplicates the graph.
- **One verdict, many grains.** The output lands on a *shared* verdict lattice, read at whatever grain the projection has (node, parameter, module, fix, type, test). Minting a per-lens verdict enum is the recurring fork (§3).
- **The lens is itself a substrate fact** (§7). Every quality below applies to the lens *recursively*: a lens must survive its own rubric — it must not be inert, degenerate, or vacuous.

---

## 2. The axes — the whole family reduces to four invariants + one meta-layer

The survey collapsed ~40 lenses onto a small closed set of *invariants*. Each is a DESIGN axiom mechanized; "are we missing a lens" is largely "is every axiom covered by an invariant."

| invariant | question it asks | DESIGN § | representative lenses | shared mechanism (today / target) |
|---|---|---|---|---|
| **plurality / cardinality of authority** — `0` inert · `1` healthy · `≥2` degenerate | how many authorities claim one identity? | §2 + §3 | inert_carrier, inert_lens, doc_reachability, wiring_liveness, unused_parameters, non_fold_residue, structural_resolution *(0-end)* · grounding, fact_density, unit_modeling *(leaf-grounding = the same axis at the leaf)* · duplicate_computation, fact_cardinality, subsumption, structural_similarity, simulated_relationship, identical_variant_payload, no_dual_representation, **vacuity** *(≥2-end)* | `LadderVerdict` (`dag/std/materialization_ladder.dag`) — but wired only for computation today; **target = the general cardinality authority** |
| **exhaustiveness / coverage** | is every member of a closed set covered? | §4 + §5 | mock_totality, mutation_adequacy, coverage, complexity_linearity_audit | `missing_coverage` set-difference (`v2.lens.coverage`) |
| **cost-bound** | is the asymptotic cost modeled and minimal? | §6 | cost → complexity → {complexity_lowering, synthesis, complexity_accumulator_copy} | one `symbolic_cost_fold` (`cost.dag`) read four directions over the `AsymptoticClass` lattice |
| **discrimination** | does this check carry information (a red ≠ every green)? | §5 | discrimination, mutation_adequacy, **vacuity** | red-input-separation (`discrimination.dag`); the anti-vacuous `universe_nonempty` control |
| *meta:* **enforcement registry** | is every lens enrolled, contracted, live, self-applying? | §7 | registry → contract → gate → receipts, standing_intent | `LensRegistryEntryV0` → `LensContract` ⇄ `StandingIntent` (live, fail-closed) |

**The one axis to internalize:** plurality. `0` (inertness — *nothing* points at the node) and `≥2` (degeneracy/redundancy — *many* things claim one concept) are the **two ends of a single cardinality axis** on a node's authority-count; the healthy middle is exactly **`1`** (`std.disposition.SingleAuthority`). Reach-count (in-edges) and authority-count (identity-edges) are two readings of the same "how many" question. The tell that this is real and un-unified: **~6 lenses already carry `dissolves_to: SingleAuthority`** — they self-declare they are waiting to be grounded onto this one authority. The `materialization_ladder`'s plurality primitive (`0×→DeadComputation`, `1→Recompute`, `≥2→AuthoredDuplication`) *is* that lattice; it has simply not yet absorbed the 0-end or the non-computation grains.

**Where the three commonly-conflated names actually sit:** `complexity` = the cost axis (its own axis, one fold). `degenerate`/`vacuity` = the **≥2-end** of the plurality axis (dual authority), at test grain. `intent_linearity` = *not* a peer of those — it is the **half-built meta-composer** of the ≥2-end detectors across representations (it dispatches `simulated_relationship`/`structural_similarity`/`complexity_lowering`/`fn_family`/`cost` by an `(axis × representation)` table), and its declared-but-empty `Representation.ImportGraph` arm is exactly the missing **reachability** (0-end) handler.

---

## 3. The stress-test rubric — the qualities of a well-designed *lens*

Run these questions against any lens. Each is a DESIGN consequence and each has a concrete failure the survey observed. A "no" is a design debt, not necessarily a blocker — but an *unnamed* "no" is. **Question 0 gates all the rest — if it fails, there is no new lens to review.**

0. **Row, not a lens? (the closure gate.)** Can this check be a **data row** — `(projection, roots, invariant-handler) → shared verdict` — on an *existing* kernel, instead of a new `.dag` module? A new *subject / grain / verdict* is always a row. A new module is warranted **only** by a genuinely new *invariant*, and there are ~four (§2). If you are about to author a lens module, name which of the four invariants it needs that no existing kernel provides — if you can't, it's a row. *(§2 Realization / §4 closed vocabulary. This is the question that stops the count at a handful instead of 200.)*
1. **Construction-first?** Could this be a construction wall (single authority the realization derives from, or a compile gate) instead of a post-hoc reader at all? A lens is justified only for the *unstructurable residue* (undecidable, or not-yet-grounded). *(§5. Tell: the check can be satisfied by editing the declaration while the realizer still lies.)*
2. **One projection of the one DAG?** Does it read a projection of the shared Node graph, not a bespoke parallel fact-list? *(§2/§4. Fail seen: several lenses roll their own flat `List<…Fact>` where a `fold_node`/`dependency_lens` projection existed.)*
3. **One shared verdict lattice?** Does it emit onto the single verdict authority (the plurality/cardinality lattice + `prescribes → Recompute|Memoize|Share`), not a minted per-lens enum? *(§3. Fail seen: `LadderVerdict` vs `LensVerdict` vs `DuplicateComputationViolation` — three vocabularies for one idea.)*
4. **One shared kernel, N handlers?** Is the mechanism a *row/handler* on a shared kernel (a `GraphInvariant<Projection>`, `symbolic_cost_fold`, `anti_unify_chain`), so a new lens costs a row, not a new engine? *(§2 Realization. Fail seen: effect/idempotency/ownership/parallelism = one engine forked 4×; structural_similarity ≡ simulated_relationship; inert_lens ≡ doc_reachability.)*
5. **Fail-closed — refuse, never widen?** Is every unresolved case a *typed, located, counted* refusal, never an absorbing fallback (rerun-all / scan-all / always-run)? *(§5. The failure arm must refuse; a widen is fail-open wearing fail-closed's name.)*
6. **Discriminating RED control, proven by execution?** Is there a red input that goes red when the checked behavior is wrong, distinct from every green — proven by *running* a consumer, not a grep/typecheck? And an anti-vacuous control (nonempty universe)? *(§5, and the recursion: a lens that cannot itself go red is vacuous — it must pass its own vacuity/discrimination check.)*
7. **Decidability honest — wall vs ratchet?** Does it declare whether it is a decidable *wall* or an undecidable *advisory ratchet*, and never let "never" masquerade as a wall? *(§5's "never" trap. Carrier: `ConstructionJustification` — `WallNow` / `WallAfterGrounding{dissolves_to}` / `RatchetForever`.)*
8. **Enrolled and live — not inert?** Is it in the lens registry (`LensIdV0`), contracted (`LensContract`), and reached by a fail-closed witness? An inert lens is a lie. *(§6/§7. Fail seen: the registry knows 10 lenses; ~15 real ones — incl. vacuity, duplicate_computation, intent_linearity — aren't enrolled and so cannot be contracted or gated.)*
9. **Self-applying where possible?** Does the lens hold over the lenses themselves (the §7 fractal)? A degeneracy lens must catch a degenerate lens; a cost lens must bound its own cost. *(Fail seen: `near_miss_vacuous_not_parallel` / `sibling_degenerate_not_hollow` are themselves vacuous `X != Y` constant-comparisons — the vacuity lens would flag its own taxonomy witnesses.)*
10. **Denominated in displaced cost, with a dissolution trigger?** Is it justified by a real pain it removes (§6), not elegance (the purity trap)? If staged/host-fed, does it name a dissolve-on and a `dissolves_to` authority? *(§6. A lens priced in elegance is unbounded self-referential work.)*

---

## 4. The qualities of a well-designed lens *set* (systemic — the recursion, §7)

The set must itself pass the rubric it enforces:

- **Axis coverage, no gaps.** Every DESIGN axiom is mechanized by some invariant (§2/§3 → plurality; §4 → exhaustiveness/acyclicity; §5 → discrimination + fail-closed; §6 → cost). "Are we missing a lens" = "is an axiom uncovered." *Current gap:* **acyclicity** (the syllogism — every claim reaches an axiom, no orphan, no cycle) is only a plan (`axiom_syllogism_lens.dag`), not built; it is the third face of the graph kernel (§6 below).
- **No forked machinery across the set.** The redundancy lens applied to the lenses: no two lenses may share an invariant with duplicated engines. *(The set currently fails its own §2 — see §5 stress-test.)*
- **One verdict authority, one kernel per invariant.** Exactly-one, not per-lens.
- **Registry-complete.** Every lens enrolled, or it cannot be gated — the registry is *literally* where "colocate them" happens.
- **The set is non-degenerate.** The lens set passes its own plurality lens (no inert lens, no two lenses that are one modulo an argument).

---

## 5. Stress-test of the *current* set (the rubric applied — concrete failures the survey found)

| failure | rubric § | fix |
|---|---|---|
| effect / idempotency / ownership / parallelism = one `DependencyKindClassifier<T>` forked 4× (differ only in the verdict enum) | 4 | one parameterized lens; the enum is a row |
| structural_similarity ≡ simulated_relationship (one `anti_unify_chain` engine, forked by one predicate + threshold) | 4 | one lens, predicate as a parameter |
| inert_lens ≡ doc_reachability (one reachability check, forked per medium; the `universe_nonempty` control is verbatim in both) | 4 | one `GraphInvariant<Projection>` reachability handler, two projections |
| two verdict vocabularies (`LadderVerdict` vs `LensVerdict`) + `fact_cardinality`'s internal bare-alias nickname | 3 | collapse to `LadderVerdict`; delete the nickname |
| `complexity_linearity_audit` / `witness_cost_locality` misfiled under "complexity" (they are exhaustiveness / reachability) | 2 | rehome by *invariant*, not by name |
| the registry knows 10 lenses; ~15 real ones aren't enrolled (vacuity, duplicate_computation, intent_linearity, inert_lens, structural_similarity, …) | 8 | enroll all; only then can they be contracted/gated |
| the ≥2-end lenses each mint a bespoke verdict yet self-declare `dissolves_to: SingleAuthority` | 3 | ground them onto the plurality authority |
| `intent_linearity`'s `Representation.ImportGraph` arm declared with no rule | 4 / §4-gap | fill it with the reachability handler → it becomes the kernel |
| coverage-taxonomy witnesses (`near_miss_vacuous_not_parallel`, `sibling_degenerate_not_hollow`) are themselves vacuous `X != Y` constants | 9 | the self-applying check catches them |

---

## 6. Consolidation direction (the target — where the set is going)

Two moves, highest leverage first. Neither is the vacuity lens in isolation — vacuity lands as *one row* of the first.

- **A. The plurality/cardinality authority.** Generalize `LadderVerdict` from `plurality-of-demand-per-computation-identity` to **plurality-of-authority-per-identity**, absorbing the **0-end** (the inert family) and the non-computation grains. ~15 lenses ground onto one lattice (`0 = Dead/Inert · 1 = SingleAuthority · ≥2 = AuthoredDuplication`), with per-substrate *detectors* feeding it (the `materialization_ladder` already runs this "detectors → one verdict" architecture: `v2.std.materialize`, `v2.lens.duplicate_computation`). Vacuity is the authored-fact detector; inert-carrier/inert-lens/doc-reachability are the 0-end detectors.
- **B. The `GraphInvariant<Projection>` kernel.** The §2 Realization pattern ("one kernel, N handlers") applied to the *lenses themselves* — the §7 recursion the project has not yet done here. One kernel — *"project the Node DAG to a graph from roots; assert an invariant over a nonempty universe"* — with three handlers: **reachability** (0-end/inert), **no-duplication** (≥2-end/degenerate), **acyclicity** (the syllogism, currently unbuilt). `intent_linearity` is the half-built version (it has the no-duplication handlers and an empty reachability arm); growing it — or lifting a kernel out of it — is the concrete first step. The verbatim `universe_nonempty` control shared by inert_lens and doc_reachability is the literal seam.

The end-state: a lens is authored as **(projection, roots, invariant-handler) → shared verdict**, registered in one registry, gated by one contract, and it survives this rubric — including recursively. When that holds, this doc dissolves into the carriers (the kernel, the verdict, the registry, the rubric-as-`StandingIntent`).

---

## 7. How to use this doc (the operator's stated intent)

Run §3 (per-lens) and §4 (per-set) against a proposed or existing lens as a **stress test**: a "no" that is *named* (a dissolve-on, a `RatchetForever`, an exemption) is acceptable; a "no" that is *silent* is the debt. The value of the doc is not the taxonomy — it is the ten questions in §3 and the five set-properties in §4, which any *future* lens must answer before it joins the set, so the family stays minimal instead of accreting the forks §5 catalogues.

## Related

- [vacuity lens](vacuity-lens-design.md) — the ≥2-end (dual authority) at test grain; one detector-row of §6.A, not a standalone concept.
- [inert-layer lens](inert-layer-lens.md) — the 0-end (reachability); "one rule, N substrates" is §6.B for the reachability face.
- [duplicate-work graph lens](duplicate-work-graph-lens-design.md) — the `materialization_ladder` computation instance; the verdict authority §6.A generalizes.
- [enforcement intent](enforcement-intent-design.md) — the meta-registry (`StandingIntent` ⇄ `LensContract` ⇄ receipt) that §4/§8 makes registry-complete.

## Dissolution trigger (DESIGN §6)

Delete this doc when the consolidation it prescribes has landed and is self-describing in the carriers: the plurality/cardinality verdict is one authority that the 0-end and ≥2-end lenses feed (§6.A), the `GraphInvariant<Projection>` kernel carries the three handlers with `intent_linearity`'s reachability arm filled (§6.B), the registry is complete, and the §3/§4 rubric lives as a `StandingIntent` gated fail-closed — at which point the lens set enforces its own well-formedness by execution and this prose is superseded (the rubric never dissolves; this doc does, per DESIGN §6's mark-on-carrier-is-authority).
