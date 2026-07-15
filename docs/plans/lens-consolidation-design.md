# Lens consolidation — the essence, the axes, and the stress-test rubric

> **The goal is to stop writing lenses.** The family has grown to ~40 distinct lenses across ~155 files; on the current trajectory it reaches 200, and 200 lens *modules* is the anti-pattern, not the win. The survey (2026-07-15) found the same handful of graph properties re-implemented under many names with forked machinery — which means the growth is mostly *redundant*, not new coverage. The target is the DESIGN §2 Realization move applied to the lenses themselves: **a small, closed set of invariant-kernels, where a new check is a DATA ROW, not a new module** — the same move that turned eleven hand-rolled caches into one `Compose` axis, and N×M parser/emitter pairs into N grammar rows. This doc fixes the **essence** of what a lens is, the **invariants** the whole family reduces to (a small closed set — with the discipline *consolidate mechanisms, not meanings*, so distinct verdicts are **not** forced into one lattice), and a **stress-test rubric** you run against any existing or proposed lens (and against the *set*) — whose *first question is always "can this be a row on an existing kernel instead of a new lens?"* It is a *rubric*, not a roster: it dissolves when the lenses are consolidated onto the shared kernel and verdict it prescribes (the mark on those carriers becomes the authority, DESIGN §6). DESIGN refs throughout: §1 (time), §2 (minimize redundancy), §3 (single authority), §4 (closed grounded acyclic substrate — a *closed* vocabulary is exactly what caps the count), §5 (fail-closed), §6 (lens-as-residue, priced in displaced cost), §7 (the compiler analyzes itself — the recursion these lenses must survive).

**The closure thesis (the whole point).** The substrate is 6 connectives + 5 behaviors — *closed*, and all variety lives in how they compose, not in new primitives. The lens set must be the same: a **closed set of invariant-kernels** (§2 below — a handful, though *not* one; "consolidate mechanisms, not meanings"), and every conceivable check is expressed as `(projection, roots, invariant-handler) → verdict` **data** over one of them. New *subjects* and *grains* are always rows; a new *verdict* is a row only when it shares an existing kernel's mechanism (else it is a genuinely new invariant, which is rare — the survey found few). If authoring a check requires a new `.dag` lens *module* rather than a row, that is the smell this rubric exists to catch. 200 lens modules means the closure failed; a handful of kernels + a growing table of rows means it held.

---

## 1. The essence — what a lens *is*

A lens is a **pure reader over a projection of the one Node DAG that decides a graph invariant and emits a verdict on a shared lattice.** It stores nothing, so a new analysis costs zero substrate edits (§6, the residue mechanism). It is the tool of last resort, not first: DESIGN §5 is *construction first* — make the bad state **unwritable** (a single authority the realization derives from; a compile gate) — and reserve the lens for the genuinely **unstructurable residue** (undecidable, or not-yet-grounded). A lens that re-states a constraint the model already carries is itself a second representation (§2/§3), so the first question of any lens is *"why isn't this a construction wall?"*

Three consequences fix the shape:

- **One graph, many projections.** The subject is always a projection (edge-set) of the *single* Node DAG — call edges, dependency edges, cost edges, containment edges, reference edges — never a bespoke parallel fact-list that duplicates the graph.
- **One verdict per invariant, many grains.** The output lands on the verdict authority *for its invariant* (`ReachVerdict`, `LadderVerdict`, an oracle-provenance verdict, …), read at whatever grain the projection has (node, parameter, module, fix, type, test). Minting a *needless* per-lens enum where the invariant's verdict already exists is the recurring fork (§3); but *merging* verdicts whose remedies differ is the opposite error — consolidate mechanisms, not meanings.
- **The lens is itself a substrate fact** (§7). Every quality below applies to the lens *recursively*: a lens must survive its own rubric — it must not be inert, degenerate, or vacuous.

---

## 2. The invariants — a small closed set (consolidate mechanisms, not meanings)

The survey collapsed ~40 lenses onto a small closed set of *invariants*. Each is a DESIGN axiom mechanized; "are we missing a lens" is largely "is every axiom covered by an invariant."

**Consolidate mechanisms, not meanings (review correction, 2026-07-15).** The earlier draft of this doc collapsed several *distinct invariants* onto one "plurality 0/1/≥2 authority-count" lattice. That is wrong and unsafe: **reachability is not cardinality.** A live root has in-degree 0 (healthy, not inert); an *unreachable cycle* gives every node in-degree 1 (dead, not healthy); a healthy shared diamond has in-degree 2 (not degenerate). Inertness is **root-relative reachability** — a global fixpoint (`inert-layer-lens.md:5-23`), not a local count. So the consolidation is at the **mechanism** layer (a shared typed `Count<Relation>` primitive, a shared `fold_node`, a shared `GraphInvariant<Projection>` kernel), and the **verdicts stay distinct** (`ReachVerdict`, `LadderVerdict`, oracle/authority verdicts, conformance verdicts) — because their *remedies* differ (a dead cycle is deleted; a duplicated computation is `Share`d; a vacuous test needs an independent oracle, **not** `Share`). Merging the meanings produces false positives and wrong fixes.

| invariant (distinct verdict) | question | DESIGN § | lenses | shared *mechanism* (verdict kept separate) |
|---|---|---|---|---|
| **root-relative reachability / acyclicity** | is every node reached from a root; no orphan, no cycle? *(NOT in-degree count)* | §4/§6 | inert_carrier, inert_lens, doc_reachability, wiring_liveness, unused_parameters, structural_resolution, non_fold_residue; the **syllogism** (unbuilt, `axiom_syllogism_lens.dag`) | `GraphInvariant<Projection>` reachability/acyclicity handler → `ReachVerdict` |
| **duplicated materialization** | ≥2 materializations of one identity, decided by frame + nature? | §2 | duplicate_computation, subsumption | frame+nature-aware `LadderVerdict` + `Count<Relation>` |
| **authority / oracle provenance** | does a value/test have exactly one authority, or an independent oracle? | §3/§5 | **vacuity** (own `VacuityEvidence`), fact_cardinality, no_dual_representation, structural_similarity/simulated_relationship, identical_variant_payload | provenance evidence; own verdicts — **not** `LadderVerdict` |
| **relational conformance** | does *declared* match *witnessed*? | §3/§5 | dependency_fidelity, interface_summary (declared-use arity) | declared-vs-witnessed relation (`dependency_fidelity.dag:12-29`) |
| **algebraic composition** | does the property compose over the fold (determinism / effect / termination)? | §4 | determinism, effect, idempotency, ownership, parallelism | `fold_node` algebraic compose; distinct per-property verdicts (`determinism.dag:63-85`) |
| **cost-bound / symbolic recurrence** | asymptotic cost modeled & minimal? | §6 | cost→complexity→{lowering, synthesis, accumulator_copy} | one `symbolic_cost_fold` over `AsymptoticClass` |
| **exhaustiveness / coverage** | every member of a closed set covered? | §4/§5 | mock_totality, mutation_adequacy, coverage | `missing_coverage` set-difference |
| **discrimination** | does the check carry information (a red observation ≠ every green)? | §5 | discrimination, vacuity (observation relation) | red/observation separation (`discrimination.dag`) |
| **construction / review residue** | is a leaf grounded, or anemic? | §2/§3 | grounding, fact_density, unit_modeling | construction gate or review residue (*conceptual*, not mechanical, dual of redundancy — `self-applying-lenses.md:108-115`) |
| *meta:* **enforcement registry** | every lens enrolled, contracted, live? | §7 | registry → contract → gate → receipts, standing_intent | `LensContract` ⇄ `StandingIntent` |
| *not lenses:* **infrastructure** | — | — | affected_set, testgen, reference/module producers | selection/generation plumbing, not invariants |

**Where the three commonly-conflated names actually sit (corrected):** `complexity` = the cost/recurrence axis (its own fold). `vacuity` = the **authority/oracle-provenance** axis at test grain (its *own* verdict `VacuityEvidence`, not the duplication verdict — its remedy is an independent oracle, not `Share`; see [vacuity design](vacuity-lens-design.md)). `intent_linearity` = a **meta-composer** that dispatches the *no-duplication* detectors (`simulated_relationship`/`structural_similarity`/`complexity_lowering`/`fn_family`/`cost`) by an `(axis × representation)` table; its empty `Representation.ImportGraph` arm is where the **reachability** handler would slot — so it is a candidate host for the `GraphInvariant` kernel, *not* proof that reachability and duplication are one axis (they are not).

---

## 3. The stress-test rubric — the qualities of a well-designed *lens*

Run these questions against any lens. Each is a DESIGN consequence and each has a concrete failure the survey observed. A "no" is a design debt, not necessarily a blocker — but an *unnamed* "no" is. **Question 0 gates all the rest — if it fails, there is no new lens to review.**

0. **Row, not a lens? (the closure gate.)** Can this check be a **data row** — `(projection, roots, invariant-handler) → verdict` — on an *existing* kernel, instead of a new `.dag` module? A new *subject / grain* is always a row; a new *verdict* is a row when it reuses an existing kernel's mechanism. A new module is warranted **only** by a genuinely new *invariant* (a new mechanism *and* a distinct verdict/remedy — §2). If you are about to author a lens module, name the invariant it needs that no existing kernel provides — if you can't, it's a row. *(§2 Realization / §4 closed vocabulary. This is the question that stops the count at a handful instead of 200.)*
1. **Construction-first?** Could this be a construction wall (single authority the realization derives from, or a compile gate) instead of a post-hoc reader at all? A lens is justified only for the *unstructurable residue* (undecidable, or not-yet-grounded). *(§5. Tell: the check can be satisfied by editing the declaration while the realizer still lies.)*
2. **One projection of the one DAG?** Does it read a projection of the shared Node graph, not a bespoke parallel fact-list? *(§2/§4. Fail seen: several lenses roll their own flat `List<…Fact>` where a `fold_node`/`dependency_lens` projection existed.)*
3. **One verdict *per mechanism* — distinct where remedies differ?** Does it emit onto the shared verdict authority *for its invariant* (`LadderVerdict` for duplication, `ReachVerdict` for reachability, the oracle authority for provenance), rather than a *needlessly* minted per-lens enum — while keeping verdicts with *different remedies* distinct? *(§3, "consolidate mechanisms, not meanings." Fail seen: `DuplicateComputationViolation`/`LensVerdict` are needless forks of `LadderVerdict`; but folding vacuity's remedy into `AuthoredDuplication→Share` would be the *opposite* error — `Share`ing `f(x)==f(x)` makes it more tautological.)*
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

- **Invariant coverage, no gaps.** Every DESIGN axiom is mechanized by some invariant (§2 → duplicated-materialization; §3 → authority/oracle-provenance + relational-conformance; §4 → reachability/acyclicity + exhaustiveness + algebraic-composition; §5 → discrimination + fail-closed; §6 → cost/recurrence). "Are we missing a lens" = "is an axiom uncovered." *Current gap:* **acyclicity** (the syllogism — every claim reaches an axiom, no orphan, no cycle) is only a plan (`axiom_syllogism_lens.dag`), not built; it is a handler on the graph kernel (§6 below).
- **No forked machinery across the set.** The redundancy discipline applied to the lenses: no two lenses may share an invariant *mechanism* with duplicated engines — but distinct verdicts are allowed and required (consolidate mechanisms, not meanings). *(The set currently fails the mechanism half — see §5 stress-test.)*
- **One kernel per mechanism; verdicts stay distinct.** One reachability engine, one anti-unification engine, one cost fold — not per-lens — while `ReachVerdict`/`LadderVerdict`/oracle verdicts remain separate.
- **Registry-complete.** Every lens enrolled, or it cannot be gated — the registry is *literally* where "colocate them" happens.
- **The set is non-degenerate.** The lens set passes its own lenses (no inert lens; no two lenses that are one engine modulo an argument).

---

## 5. Stress-test of the *current* set (the rubric applied — concrete failures the survey found)

| failure | rubric § | fix |
|---|---|---|
| effect / idempotency / ownership / parallelism = one `DependencyKindClassifier<T>` forked 4× (differ only in the verdict enum) | 4 | one parameterized lens; the enum is a row |
| structural_similarity ≡ simulated_relationship (one `anti_unify_chain` engine, forked by one predicate + threshold) | 4 | one lens, predicate as a parameter |
| inert_lens ≡ doc_reachability (one reachability check, forked per medium; the `universe_nonempty` control is verbatim in both) | 4 | one `GraphInvariant<Projection>` reachability handler, two projections |
| two verdict vocabularies for the *same* duplication mechanism (`LadderVerdict` vs `LensVerdict`) + `fact_cardinality`'s internal bare-alias nickname | 3 | one duplication verdict; delete the nickname *(keep oracle/reach verdicts separate — that is not this fork)* |
| `complexity_linearity_audit` / `witness_cost_locality` misfiled under "complexity" (they are exhaustiveness / reachability) | 2 | rehome by *invariant*, not by name |
| the registry knows 10 lenses; ~15 real ones aren't enrolled (vacuity, duplicate_computation, intent_linearity, inert_lens, structural_similarity, …) | 8 | enroll all; only then can they be contracted/gated |
| the redundancy lenses each mint a bespoke verdict yet self-declare `dissolves_to: SingleAuthority` | 3 | ground *duplication* lenses onto `LadderVerdict`, *provenance* lenses onto their own oracle authority — **not** one merged "plurality authority" (that was the review's false-merge) |
| `intent_linearity`'s `Representation.ImportGraph` arm declared with no rule | 4 / §4-gap | fill it with the reachability handler → it becomes a host for the graph kernel |
| coverage-taxonomy witnesses (`near_miss_vacuous_not_parallel`, `sibling_degenerate_not_hollow`) are themselves vacuous `X != Y` constants | 9 | the self-applying check catches them |

---

## 6. Consolidation direction (the target — where the set is going)

Two moves, highest leverage first. Vacuity is not the driver — it lands as one *provenance* row, on its own verdict.

- **A. The `GraphInvariant<Projection>` kernel (share the mechanism, keep verdicts distinct).** The §2 Realization pattern ("one kernel, N handlers") applied to the *lenses themselves* — the §7 recursion the project has not yet done here. One kernel — *"project the Node DAG to a graph from roots; assert an invariant over a nonempty universe"* — with **distinct handlers and distinct verdicts**: **reachability** (`ReachVerdict` — root-relative, a global fixpoint, *not* an in-degree count), **acyclicity** (the syllogism, `axiom_syllogism_lens.dag`, unbuilt), and the anti-unification/**no-duplication** engine. These are *different invariants that share the graph-projection + `Count<Relation>` + `universe_nonempty` machinery* — not ends of one axis (the review's central correction). `intent_linearity` (which has the no-duplication handlers and an empty `ImportGraph`/reachability arm) is a candidate host; the verbatim `universe_nonempty` control shared by `inert_lens` and `doc_reachability` is the literal seam to lift the kernel from.
- **B. One verdict authority per mechanism.** Collapse the *duplication* verdict forks onto `LadderVerdict` (`duplicate_computation`'s `DuplicateComputationViolation`, the `LensVerdict` split, `fact_cardinality`'s nickname), collapse the classifier-quartet onto one `DependencyKindClassifier<T>`, and give the *provenance* lenses (vacuity, no-dual-representation) their own oracle/authority verdict — **not** `LadderVerdict`, because the remedy differs (independent oracle / construction / deletion, never `Share`). Then enroll every lens in the registry so the contracts can gate.

The end-state: a lens is authored as **(projection, roots, invariant-handler) → shared verdict**, registered in one registry, gated by one contract, and it survives this rubric — including recursively. When that holds, this doc dissolves into the carriers (the kernel, the verdict, the registry, the rubric-as-`StandingIntent`).

---

## 7. Wave structure (dependency-ordered)

```
W0 (de-fork, no gating) ──▶ W1 (kernel + verdict boundaries) ──▶ W2 (new invariants)
                                     │                                    │
                                     └──────────────┬─────────────────────┘
                                                    ▼
   external lanes ─(SymbolIndex/parsed-producer, v1-deletion, fn-body reflection)─▶ W3 (gated migrations)
                                                    │
                                                    ▼
                                          W4 (self-enforcement + doc dissolution)
```

- **Wave 0 — de-fork (no dependencies; pure deletion; reversible).** The immediate-value burst; each collapses forks the survey proved identical. `W0.1` classifier-quartet → one `DependencyKindClassifier<T>`. `W0.2` structural_similarity ≡ simulated_relationship → one engine. `W0.3` verdict de-forks (`duplicate_computation`→`LadderVerdict`, delete `fact_cardinality` nickname, reconcile `LensVerdict`). `W0.4` rehome mislabeled (`complexity_linearity_audit`, `witness_cost_locality`). *Value: collapses ~8 lenses → ~3; proves the direction.*
- **Wave 1 — kernel + verdict boundaries (keystone; deps: W0 for a smaller surface).** `W1.1` lift `GraphInvariant<Projection>` from `wiring_liveness`'s existing pure-`.dag` reach engine + the `universe_nonempty` seam; repoint the pure reachability lenses. `W1.2` fix the verdict-per-mechanism carriers (`ReachVerdict` / `LadderVerdict` / oracle-provenance / conformance / algebraic — distinct, per "consolidate mechanisms not meanings").
- **Wave 2 — new invariants (deps: W1 to host handlers).** `W2.1` the acyclicity/syllogism handler (the one truly-missing invariant). `W2.2` vacuity — `TestClaimObservationRelation` + `VacuityEvidence` + classifier as an advisory census (walls deferred to W3).
- **Wave 3 — gated migrations (deps: external lanes, not effort).** Each is a one-row migration when its lane clears. `W3.1` repoint the host-count reachability lenses (inert_lens/doc_reachability/non_fold_residue/inert_carrier) off `cli_run.rs` onto the kernel — **gated on v1-deletion / parsed-producer**. `W3.2` vacuity `ProvenDuplicate` walls — **gated on `symbol_index_fill` provenance/reference edges**. `W3.3` registry completeness (enroll ~15 lenses + contracts + receipts) — the receipt half **gated on fn-body reflection** (`decl_facts`).
- **Wave 4 — self-enforcement closure (deps: W1–W3).** `W4.1` model the §3/§4 rubric — *especially Question 0* — as a `StandingIntent` gated fail-closed, so a new `.dag` lens *module* not justified by a genuinely-new invariant **fails CI**. This is the move that stops the count structurally, not by diligence. `W4.2` dissolve the design docs (this one, [vacuity](vacuity-lens-design.md), [inert-layer](inert-layer-lens.md)) as the carriers become self-describing (§6 mark-on-carrier).

Wave 0 and the Wave 1 kernel-lift can overlap; W3's three sub-lanes are independent and each fires when its external dependency clears. Nothing in W0–W2 touches `src/v1` or waits on the gated lanes.

## 8. Definition of done (a wall, not an empty roster)

The project is **not** done when "every lens is migrated" — that is a moving target, and "no new lens ever" is the §5 *"never" trap* (a ratchet masquerading as a wall). It is done when **the set enforces its own well-formedness by execution — non-conformance becomes unwritable.** Two distinct done-lines:

**Structural done (the real answer to "200 lenses" — achievable at W4.1, ahead of the long tail).** A conjunction of *live, fail-closed, RED-controlled* lenses over the lens corpus itself (the §7 recursion):
1. **Question 0 is a gate** — a new lens *module* that isn't a genuinely-new invariant fails CI (the rubric as a `StandingIntent`). *RED control: a synthetic redundant lens module must be refused.*
2. **No forked mechanism** — no two lenses share an invariant with duplicated engines. *RED control: re-introducing a deleted fork goes red.*
3. **Registry-complete** — every lens module enrolled + contracted + live-receipted; an un-enrolled lens is refused. *(extends the #5433 inert-lens backstop from wiring to enrollment.)*

Once these three are green-by-execution, the count *cannot* grow into 200 by construction — even with migrations still on the frontier. That is the deliverable.

**Migration done (the long tail — a declared frontier, never a completion claim).** Each not-yet-migrated lens is a **counted, typed, dissolve-on-tracked frontier row** (the seed-retained pattern, §7), not a silent gap — so "not yet migrated" is observable and prioritizable, never confused with "done." The frontier empties as W3's external lanes clear; it may legitimately never hit zero (a genuinely new invariant can always appear), which is *exactly why* the done-line is the self-enforcing gate above, not `roster == 0`. When the frontier is empty **and** the gate is live, the design docs dissolve (W4.2) and the lenses are the only authority left.

**One-line test of doneness:** *can someone add a redundant or unenrolled lens and have CI stay green?* When the answer is a fail-closed "no," the project is structurally done, whatever the migration frontier still holds.

## 9. How to use this doc (the operator's stated intent)

Run §3 (per-lens) and §4 (per-set) against a proposed or existing lens as a **stress test**: a "no" that is *named* (a dissolve-on, a `RatchetForever`, an exemption) is acceptable; a "no" that is *silent* is the debt. The value of the doc is not the taxonomy — it is the ten questions in §3 and the five set-properties in §4, which any *future* lens must answer before it joins the set, so the family stays minimal instead of accreting the forks §5 catalogues.

## Related

- [vacuity lens](vacuity-lens-design.md) — the **authority/oracle-provenance** invariant at test grain; its *own* `VacuityEvidence` verdict (remedy = independent oracle, not `Share`), not the duplication verdict.
- [inert-layer lens](inert-layer-lens.md) — the **root-relative reachability** invariant; "one rule, N substrates" is the reachability handler of the §6.A graph kernel.
- [duplicate-work graph lens](duplicate-work-graph-lens-design.md) — the `materialization_ladder` **duplicated-materialization** instance; `LadderVerdict` is the duplication authority §6.B collapses the forks onto.
- [enforcement intent](enforcement-intent-design.md) — the meta-registry (`StandingIntent` ⇄ `LensContract` ⇄ receipt) that §4 makes registry-complete.

## Dissolution trigger (DESIGN §6)

Delete this doc when the consolidation it prescribes has landed and is self-describing in the carriers: the `GraphInvariant<Projection>` kernel carries its distinct handlers (reachability/acyclicity/no-duplication) with `intent_linearity`'s reachability arm filled (§6.A), each mechanism has one verdict authority while distinct verdicts stay distinct (§6.B), the registry is complete, and the §3/§4 rubric lives as a `StandingIntent` gated fail-closed — at which point the lens set enforces its own well-formedness by execution and this prose is superseded (the rubric never dissolves; this doc does, per DESIGN §6's mark-on-carrier-is-authority).
