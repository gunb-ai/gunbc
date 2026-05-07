# Canvas — Q-Lens-Target-Context (β-extended substrate-shape question)

**Authority**: Director ratification of α-revised T-CostLens supersession at gunbc#828 #issuecomment-4400920572 — β-extended path (introduce first-precedent .dag-side fold-over-realization-rows + name active-target authority shape) elevated from slice-tier to canvas-tier.

**Cross-cutting parent**: PR #2171 α-revised pure-docs (cost.dag deferred-discussion + 5-option matrix + 4 findings) merged 2026-05-07 — captures substrate-state-honest framing pending this canvas's ratification.

**Sibling canvas**: `q-cost-composition-layering-canvas.md` (ε path; abstraction-layering test for cost-specific Rust-side wiring).

**Status**: **canvas — DRAFT/DEFERRED 2026-05-07; reopens on N=2 trigger event**. Director ratified ε path for cost (sibling canvas Q-Cost-Composition-Layering ratified 2026-05-07); β-extended substrate-wide refactor case isn't proven at HEAD with N=1 (only cost). Canvas reopens when SECOND lens surfaces real target-context need beyond cost (emission_provenance per cross-cutting analysis is most likely candidate). Until then, no proposal-maturation authoring; Q-Lens-Target-Context stays in canvas-authoring queue. Closed-system disposition: if N=2 condition never fires, β-extended never fires.

## Substrate-state at HEAD (grep-verified)

`src/v3/std/lens.dag:70-77` defines the generic Lens<C> carrier:

```dag
type Lens<C> {
  name: String
  read: fn(Dag, Behavior) -> Witness<C>
  sequential: Monoid<C>
  branch: fn(C, C) -> C
  iterate: fn(C, LoopBound) -> C
  validate: fn(Dag, C) -> OptionalDiagnostic
}
```

**`read` signature carries no target-context**. Per-primitive realization-cost reading (T-CostLens-Composition gate #37) requires the lens to know WHICH target it's emitting for to pick the right realization row. Realizations are consumed Rust-side at emit time only (`emit/{rust,python}_target.rs` builds `HashMap<DeclarationId, …Binding>` from data-row scan); zero `.dag`-side fold-over-`List<CallableRealization>` / `List<TypeRealization>` / `List<MethodTemplateContract>` precedent at HEAD.

**Lens instances at HEAD** (`src/v3/lenses/`): complexity, cost, dag_shape, effect_enumeration, emission_provenance, idempotency, named_function_count, parallelism, provenance, structural_resolution, unused_parameters, variant_payload, lens_composition_associative_witness, infer_helpers, lower_helpers — 15+ lens instances using the generic Lens<C> carrier.

## Question (binary)

**Should .dag-side lenses receive target-context for target-keyed reading?**

This is a substrate-fact-introduction (P1 procedure): introducing first-precedent `.dag`-side fold-over-realization-rows substrate-consumption pattern. Yes-or-no decision; affects every future lens with target-context needs (not just cost).

## Options matrix

### Option (i) — `LanguageSpec` parameter to lens fold

Refactor `Lens<C>::read` from `fn(Dag, Behavior) -> Witness<C>` to `fn(Dag, Behavior, LanguageSpec) -> Witness<C>`. All 15+ existing lens instances thread `LanguageSpec` parameter; consumers pass active-target at lens-application time.

**Pro**:
- Most additive at lens-instance level (instances that don't use target-context can ignore the parameter)
- Matches Rust-side `active_language` cache discipline (per fierce-ram-21's grep at gunbc#2153 c#4399898495)
- Single substrate-shape change; cascade is bounded (15 lens instances + their consumers)
- Future lenses that need target-context can use it without further substrate refactoring

**Con**:
- Threading cost across 15 existing lens instances (even if they ignore the parameter, signature change cascades)
- Forces target-context awareness on lens-fold-pass authority even when only one lens (cost) needs it
- Lens<C> generic refactor requires its own canvas + worker brief (per `feedback_pre_authored_brief_queue`)

### Option (ii) — Per-target lens instances

Author per-target lens instances: `cost_lens_rust`, `cost_lens_python`, `cost_lens_go` (and analogous for any other target-context-needing lens). Lens-fold-pass dispatches on target at consumer level; each lens instance is target-specific.

**Pro**:
- No `Lens<C>` carrier-shape refactor needed
- Existing 14 non-cost lenses untouched
- Each lens instance is target-specific by construction

**Con**:
- **Explosion-by-target**: N targets × M target-context-needing lenses = N×M lens instances (3 targets × 1 cost-needing lens currently = 3 instances; grows to 9 if 3 lenses need target-context)
- Lens-fold-pass authority must dispatch on target — substrate-side dispatch logic
- Per-lens-per-target authoring duplicates lens shape; parallel-representation debt
- Violates `feedback_parallel_representation_debt` (canonical-authority-consumption rule — Lens<C> exists; per-target instances multiply rather than consume)

**Status**: structurally weakest option; named for completeness.

### Option (iii) — Global accessor (REJECTED)

New substrate accessor `current_language_spec(): LanguageSpec` callable from inside lens fold body.

**Pro**: minimal surface change; one new accessor.

**Con**:
- **Global-state anti-pattern** per `feedback_state_space_vs_behavioral_invariants` (lens output should be a function of explicit inputs, not implicit global state)
- Hidden dispatch state at fold-time; breaks lens compositionality
- No structural visibility into "which target" at lens-output read time

**Rejected** per anti-pattern framing; named only as the third option of the matrix per template completeness.

## Mgr-tier provisional preference

**Option (i)** — `LanguageSpec` parameter to lens fold. Most structurally correct: bounded substrate-shape change; matches Rust-side discipline; preserves Lens<C> generic; future-proofs additional target-context-needing lenses.

The threading cost across 15 existing lens instances is real but manageable per `feedback_construction_over_ratchets` (model first; threading is mechanical). Each non-target-context-using instance can simply ignore the parameter (`fn(_dag, _behavior, _lang_spec) -> Witness<C>`).

If Director ratifies (i), the cascade:
1. Lens<C> carrier-shape refactor PR (separate Q-PAFS-template canvas + worker brief — substrate-fact-introduction territory)
2. 15 existing lens instances thread `LanguageSpec` parameter (mechanical migration; `_lang_spec` for non-using instances)
3. T-CostLens follow-on slice authors `target_realization_cost_for_callable` helper consuming `LanguageSpec` per-primitive lookup
4. Cementing receipts + capability-register row + §10.3 row updates per α-narrow-eventually shape (post-canvas-ratification)

## Cross-cutting question

**Which other lenses beyond cost plausibly need target-context?**

Grep + analysis of `src/v3/lenses/` instances:
- **complexity.dag**: structural-fold over algebra; no target-context need (complexity IS target-agnostic)
- **dag_shape.dag**: structural; no target need
- **effect_enumeration.dag**: structural-effect axis; no target need
- **emission_provenance.dag**: emission-side reading — POSSIBLY target-context need
- **idempotency.dag**: structural; no target need
- **named_function_count.dag**: structural; no target need
- **parallelism.dag**: R4-carved; structural at HEAD
- **provenance.dag**: structural; no target need
- **structural_resolution.dag**: structural; no target need
- **unused_parameters.dag**: structural; no target need
- **variant_payload.dag**: structural; no target need
- **lens_composition_associative_witness.dag**: meta-lens; no target need

**Net finding**: cost.dag is the load-bearing target-context-needing lens; emission_provenance.dag is a secondary candidate (emission-side framing; target-keyed emission may need lookup). Other lenses are target-agnostic.

This means option (i)'s threading cost is mostly mechanical (14 of 15 instances don't substantively use `LanguageSpec`). Option (ii)'s explosion-by-target is bounded too (cost × 3 targets + emission_provenance × 3 targets = 6 per-target instances if (ii) chosen) — but still violates parallel-representation discipline.

**Mgr lean reaffirmed**: option (i).

## Director ratification ask

1. **Pick (i) / (ii) / (iii)** (or surface fourth option). Mgr provisional preference: **(i)**.
2. **Confirm Lens<C> generic refactor scope**: if (i) ratified, refactor lands as separate canvas + worker-brief cycle (per Q-PAFS template) BEFORE T-CostLens follow-on slice. Confirm sequencing.
3. **`LanguageSpec` parameter shape**: if (i), confirm whether `LanguageSpec` is the right type-level parameter (vs `&LanguageSpec` reference-shape, vs `LanguageSpecId` lookup-key). DSL conventions favor value-shape where feasible; verify at refactor brief time.
4. **Cross-cutting impact ratification**: confirm `emission_provenance.dag` is the secondary target-context candidate (not exhaustive — invite Director scrutiny on lens-instance survey).

## On ratification — sequencing

Post-ratification cascade:
1. **Q-Lens-Target-Context PROPOSAL doc** lands per Q-PAFS template (this canvas matures into proposal)
2. **Lens<C> refactor canvas + worker brief** authored next (separate cycle if (i) ratified)
3. **Lens<C> refactor PR** — Mgr-tier or worker dispatch; threading 15 lens instances
4. **T-CostLens follow-on slice** authors `target_realization_cost_for_callable` helper consuming new `LanguageSpec` parameter; cementing receipts + capability-register + §10.3 row updates per α-narrow-eventually shape
5. **Verification ratchet authoring** sequences against gate transitions (#37/#40/#70 advancement)

## Sibling canvas (ε path)

`q-cost-composition-layering-canvas.md` — addresses whether cost composition factoring (target-agnostic-shape × target-specific-values) is structurally honest, allowing Rust-side wiring without `.dag`-side first-precedent. If ε is ratified instead of (i), the substrate-shape decision IS that target-context belongs emit-side; `.dag`-side carrier holds abstract cost shape; concrete values flow through Rust-side composition.

The deeper cross-cutting question — does target-context belong .dag-side (β-extended, this canvas) or emit-side (ε, sibling canvas) — is the load-bearing axis. Director's ratification should consider both canvases as a pair.

## Framework discipline anchors

Citing per Director correction at gunbc#828 c#4400774036 + c#4400921658:

- **`feedback_same_slice_dissolution_discipline`**: bridge-with-named-dissolution anti-pattern. β-extended (this canvas's path) was rejected at slice-tier because the named dissolution ("when γ-extended Lens<C> refactor lands, fold helper into Lens<C>::read") was future-scope-not-same-slice. This canvas authors the substrate move at canvas-tier so that follow-on slice's same-slice acceptance is structurally grounded.
- **`feedback_parallel_representation_debt`**: canonical-authority-consumption rule. Option (ii) per-target instances violates this (Lens<C> exists; per-target instances multiply rather than consume). Option (i) honors by refactoring the canonical authority itself.
- **`feedback_abstraction_layering`** (sibling canvas anchor): see Q-Cost-Composition-Layering for cost-specific abstraction-layering test.

## Worker pin (post-ratification)

Lens<C> refactor (if (i) ratified): substrate-fact-introduction precedent owners — valiant-ibex-312 OR smart-ram-167. Worker dispatched per substantial substrate change.

T-CostLens follow-on slice: fierce-ram-21 (continuity from α-revised).

## Cross-Mgr coordination

- **Verification Mgr (#2075 / wise-bear-525)**: ratchet authoring on §1.6 NYI → executable transitions (cost-lens BEHAVIORALLY COMPLETE gate #80, T-CostLens gates #37/#40/#70) sequences post-canvas-ratification.
- **Evaluator Mgr (#2065 / crisp-bat-13)**: lens consumers downstream — Pattern A predicates (TC1/TC2/TC3) consume lens output; target-context propagation impacts their predicate structure.
- **Grounding Mgr (#1944)**: target-realization data is Grounding lane authority; lens fold consuming target-keyed lookups means cross-Mgr data-flow dependency.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director α-revised supersession at gunbc#828 #issuecomment-4400920572 + canvas authoring queue per gunbc#2068 #issuecomment-4401221750.
