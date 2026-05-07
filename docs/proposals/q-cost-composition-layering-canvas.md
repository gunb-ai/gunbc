# Canvas — Q-Cost-Composition-Layering (ε substrate-shape question)

**Authority**: Director ratification of α-revised T-CostLens supersession at gunbc#828 #issuecomment-4400920572 — ε path (Rust-side cost-composition wiring; preserves `cost.dag` PROXY without introducing first-precedent .dag-side fold-over-realizations) elevated from slice-tier to canvas-tier.

**Cross-cutting parent**: PR #2171 α-revised pure-docs (cost.dag deferred-discussion + 5-option matrix + 4 findings) merged 2026-05-07.

**Sibling canvas**: `q-lens-target-context-canvas.md` (β-extended path; substrate-shape question on whether `.dag`-side lenses receive target-context).

**Status**: **canvas — Director-tier ratification needed; cost-specific factoring claim test against `feedback_abstraction_layering`**.

## The factoring claim

**ε path proposes**: cost composition factors as **(target-agnostic-shape) × (target-specific-values)**, where:
- **Target-agnostic-shape** = the abstract `SymbolicCost` algebra (7-variant: `ConstantCost`, `LinearCost`, `LogCost`, `LinearithmicCost`, `QuadraticCost`, `ExponentialCost`, `PolynomialCost(Int)`) + `Semiring<SymbolicCost>` composition (sequential / branch / iterate via `sum`/`product`)
- **Target-specific-values** = per-primitive realization-cost facts (`TypeRealization.cost: Int`, `CallableRealization.cost: Int`, etc.) loaded Rust-side at emit time

If this factoring is **structurally honest**, cost composition can happen Rust-side: the Rust consumer reads the abstract `SymbolicCost` shape from the `.dag`-side lens output, then composes target-specific values via Rust-side data-flow (HashMap-build over realization rows, lookup, multiply by abstract structural coefficients).

If this factoring is **NOT honest**, cost composition NEEDS .dag-side target-context — ε is parallel-representation debt (Rust-side wiring duplicates / bypasses the canonical Lens-framework authority).

## Test against `feedback_abstraction_layering` (Director-named anchor)

Per Director correction at gunbc#828 c#4400921658:

> `feedback_abstraction_layering` — objective concepts pure; LanguageSpec language-agnostic. Cite this as the framework anchor for whether emit-side cost-keyed composition respects abstraction layering.

**Abstraction-layering test**:

1. **Pure objective-concepts layer**: `SymbolicCost` algebra at `src/v3/std/algebra.dag:12+` carries the abstract shape. Per `feedback_abstraction_layering`, objective concepts must be pure — no language/target awareness leaks.
2. **LanguageSpec layer**: language-agnostic; describes structural per-target realization facts without committing to a specific language's idioms.
3. **Emit-side composition layer**: Rust-side `emit/{rust,python}_target.rs` reads LanguageSpec data + abstract `SymbolicCost` shape; composes per-target concrete cost via per-primitive HashMap-lookup.

**Layering compliance test**:
- ✓ Layer 1 (objective): `SymbolicCost` algebra is target-agnostic. Holds at HEAD.
- ✓ Layer 2 (LanguageSpec): per-primitive realization-cost facts are language-agnostic substrate-data. Holds at HEAD (`src/v3/spec/{rust,python,go}.dag` rows are spec-data, not Rust-specific dispatch).
- ⚠️ Layer 3 (emit-side composition): IF cost composition factors honestly, this layer is the natural home for (target-agnostic × target-specific) → concrete cost. IF factoring fails, this layer carries load-bearing logic that should have been substrate.

## Honest-factoring evidence

### Pro factoring (ε is structurally honest)

**Argument**: `SymbolicCost` is already target-agnostic. Per fierce-ram-21's grep at gunbc#2153 c#4399898495 (Slice 1 finding):

> `lenses/cost.dag` lens output is already `Lookup<SymbolicCost>`-typed; `Lookup<SymbolicCost>` MissingCost/HitCost machinery all already substrate. #38/#39 substrate-already-aligned.

Cost composition via `Semiring::sum`/`::product` is structural-fact at HEAD. The abstract algebra IS the load-bearing substrate. Per-primitive realization values are downstream data-flow that doesn't need .dag-side lens-fold authority — Rust consumer reads the abstract shape + multiplies per-primitive concrete values.

**Empirical test**: at HEAD, `lens_cost_symbolic_generated.rs` (or analogous Rust consumer) IS the natural home for emit-time cost composition. No substrate-shape change needed; ε is "make Rust-side composition explicit + document as the canonical path".

### Con factoring (ε is parallel-representation debt)

**Argument**: cost composition that reads target-realization-rows via Rust-side HashMap-build IS load-bearing logic. If Rust-side does the composition, the lens-framework abstraction stops at "abstract `SymbolicCost` shape from structural fold"; concrete cost values flow OUTSIDE the Lens<C> framework via Rust-side data-flow.

This means: **the lens-framework authority for cost composition is incomplete at the substrate layer** — Rust-side carries load-bearing semantics that should have been substrate. Per `feedback_parallel_representation_debt`, parallel-author-Rust-side-composition violates canonical-authority-consumption (Lens<C> exists; the cost-specific composition should consume it through, not bypass it).

**Empirical test**: future lenses needing target-context (e.g., emission_provenance per Q-Lens-Target-Context canvas) face the same dilemma — does each lens-with-target-context get its own Rust-side wiring? That's N parallel Rust-side compositions, each violating the lens-framework abstraction.

## Mgr-tier provisional reading

The factoring claim is **probably honest for cost specifically** but **probably NOT generalizable**:

- **Cost-specific honest**: `SymbolicCost` algebra is genuinely target-agnostic; per-primitive realization-cost is genuinely target-specific data; Rust-side composition is the natural home per `feedback_abstraction_layering`.
- **Not generalizable**: emission_provenance (and possibly other future lenses) may have target-keyed structural-needs that DON'T factor as cleanly. ε for cost works; ε for emission_provenance may not.

**Mgr provisional reading**: ε is the right framing **IF** cost is the only target-context-needing lens at HEAD AND the factoring honestly holds for cost specifically. Per Q-Lens-Target-Context canvas's cross-cutting analysis, cost.dag IS load-bearing target-context-needing; emission_provenance.dag is only secondary. If only cost has the need + the factoring is structurally honest for cost, ε is the more constrained path.

If multiple lenses surface target-context needs over time, ε's per-lens Rust-side wiring becomes parallel-representation debt → β-extended (Q-Lens-Target-Context option (i)) becomes the right path.

## Directorratification ask

1. **Test the factoring**: is `SymbolicCost` × per-primitive-realization-cost an honest factoring per `feedback_abstraction_layering`? Or does it carry hidden parallel-authority?
2. **Generalization scope**: if ε is ratified for cost, what's the principle for future target-context-needing lenses? (Per-lens Rust-side wiring? Or Q-Lens-Target-Context cascade if N>1?)
3. **Cost-specific vs cross-cutting decision**: ε is the more-constrained path IF cost is the singular need. If emission_provenance or other lenses surface, β-extended (option (i) from Q-Lens-Target-Context) is the right path. **Director ratifies one of**:
   - **(ε)** Cost-specific Rust-side composition; preserves `cost.dag` PROXY; future lenses re-evaluate independently.
   - **(β-extended option (i))** Lens<C> refactor for `LanguageSpec` parameter; .dag-side composes via Lookup<SymbolicCost>; future lenses use the new shape uniformly.
   - **(both, sequenced)**: ε now for cost; β-extended later if/when N>1 target-context-needing lenses surface.

## On ratification — sequencing

**If ε ratified**:
1. **Q-Cost-Composition-Layering PROPOSAL doc** matures (this canvas).
2. **T-CostLens follow-on slice** Rust-side wiring: `lens_cost_symbolic_generated.rs` (or analog) authors per-primitive realization-cost composition; `cost.dag` lens output stays abstract `SymbolicCost`-typed; PROXY status either retained or refined to "BEHAVIORALLY COMPLETE for abstract-shape; concrete composition Rust-side per ε ratification".
3. **§10.3 row refresh** + capability-register row updated per ε disposition.
4. **Cementing receipts**: gates #37 + #40 + #70 advance via Rust-side composition demonstrating end-to-end cost reading. #70 demonstration fixture lands per ε framing.
5. **Future lenses with target-context need**: re-ratify per-lens (re-open canvas if N>1).

**If β-extended (option (i)) ratified instead**: see Q-Lens-Target-Context canvas for sequencing (Lens<C> refactor → 15-instance threading → T-CostLens follow-on slice consumes new shape).

**If both sequenced**: ε now for cost as transitional; β-extended later as canonical when need-count surfaces.

## Framework discipline anchors

- **`feedback_abstraction_layering`** (Director-named for this canvas): objective concepts pure; LanguageSpec language-agnostic. Test: does ε's emit-side composition respect the layer separation, or does it leak target-specific logic into objective-concepts-layer? Mgr provisional reading: ε respects the layering IF the factoring is honest for cost.
- **`feedback_parallel_representation_debt`**: canonical-authority-consumption rule. Test: does ε's Rust-side wiring duplicate or bypass Lens<C> framework? Mgr provisional reading: ε is bounded to one consumer (cost) at HEAD; doesn't yet violate. Re-evaluates if N>1.
- **`feedback_same_slice_dissolution_discipline`**: bridge-with-named-dissolution anti-pattern. Test: does ε ship a bridge with a named-future-dissolution to β-extended? Mgr provisional reading: NO if Director ratifies ε as canonical-for-cost (not as "transitional until β-extended"). YES if Director ratifies "both sequenced" — that becomes a named-future-dissolution.

## Sibling canvas

**Q-Lens-Target-Context** addresses the broader cross-cutting question: should `.dag`-side lenses receive target-context generally? This canvas (Q-Cost-Composition-Layering) addresses the cost-specific factoring claim. The two canvases together give Director ratification surface across both substrate-shape decisions.

**Deeper cross-cutting axis**: target-context belongs `.dag`-side (β-extended) or emit-side (ε)? The two canvases articulate the choices; Director ratifies the load-bearing shape.

## Worker pin (post-ratification)

If ε ratified: T-CostLens follow-on slice — fierce-ram-21 (continuity from α-revised); Rust-side wiring is implementation-tier, not substrate-fact-introduction.

If β-extended ratified: Lens<C> refactor + downstream slices — see Q-Lens-Target-Context.

## Cross-Mgr coordination

- **Verification Mgr (#2075)**: ratchet authoring on §1.6 NYI → executable transitions sequences post-canvas-ratification.
- **Grounding Mgr (#1944)**: target-realization data is Grounding lane authority. Under ε, Rust-side consumer reads the data; under β-extended, lens fold reads via `LanguageSpec` parameter. Cross-Mgr coordination on data-availability shape.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director α-revised supersession at gunbc#828 #issuecomment-4400920572 + framework anchor correction at c#4400921658. Sibling to `q-lens-target-context-canvas.md` (β-extended path).
