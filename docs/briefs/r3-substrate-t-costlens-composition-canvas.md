# Canvas — Substrate T-CostLens-Composition (`Lens<SymbolicCost>` instance shape)

**Sub-issue**: gunbc#1957 (parented under #1939 Substrate Mgr lane).
**Authority**: `docs/r3-program-plan.md` §10.3 T-CostLens-Composition row at line 399 ("(TBD from Substrate canvas)" — explicitly Substrate Mgr canvas territory); `docs/r3-design-schedule-2026-05-06.md:72` cost-lens-as-discriminator framing.
**Closure predicate**: §1.8 gates #37-40 (structural-fold + thesis-unification + no-separate-cost-dimension + executable predicate) + #70 demonstration.
**Status**: **canvas — Director-tier ratification needed on composition shape before worker brief authoring**.

## Adjacent substrate (grep-verified at HEAD)

- `src/v3/std/lens.dag:70-77` defines the generic `Lens<C>` carrier (`read`, `sequential: Monoid<C>`, `branch`, `iterate`, `validate`). Already-landed; NOT subject to redesign.
- `src/v3/std/algebra.dag:12+` defines `SymbolicCost` 7-variant coproduct (`ProductCost`, `SumCost`, etc.) + `Semiring<SymbolicCost>` inhabitance. Algebra-cost side already-substrate.
- `src/v3/std/lookup.dag:48-60` scaffolds `Lookup<SymbolicCost>` + `MissingCost` lens-boundary-fallback shape.
- `src/v3/std/machine_constraints.dag` (per memory + #1933) defines `MachineWidth<bits>` substrate; target-realization side has structural facts available.
- `src/v3/std/dimensions.dag:10` notes `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost>` is **deferred** — that's part of the canvas territory.

## Scope

T-CostLens-Composition lands a **single `Lens<SymbolicCost>` instance** (or near-equivalent) that composes algebra-cost (already-substrate via `Semiring<SymbolicCost>`) with target-realization-cost (read via `MachineConstraint<C>` / target language spec) end-to-end. Per Director's "cost-lens-as-discriminator" framing: the cost lens orders faithful-representation alternatives by per-primitive realization cost; Grounding selects lowest-cost faithful representation.

**Key invariant** (gate #39 `no_coercion_cost_dimension`): NO separate cost dimension for coercion vs realization vs algebra — one `SymbolicCost` algebra, all three sources feed into it via `Semiring` operations.

## Carrier-composition options

### Option α — Two separate lenses, externally joined

`Lens<SymbolicCost>` reads algebra-cost only (structural-fold over Behavior + algebra inhabitance); separate `Lens<RealizationCost>` reads target-realization-cost from LanguageSpec; composition happens at the consumer (Grounding).

**Pro**: Each lens has a single concern; algebra-cost is target-independent (purer).
**Con**: Violates gate #39 by construction — TWO cost dimensions, joined by convention not by carrier shape. `coercion_cost_equals_complexity_by_construction` (gate #38) would have to be a derived theorem rather than structural-by-construction. Rejected.

### Option β — Single `Lens<SymbolicCost>` with composed witness in `read`

`Lens<SymbolicCost>::read(dag, behavior) -> Witness<SymbolicCost>` reads BOTH algebra-cost AND target-realization-cost, composing them via `Semiring<SymbolicCost>` (sum for sequential operations, product for branch/iterate as appropriate). Target-realization-cost obtained via per-primitive lookup keyed on `MachineConstraint<C>` instances at the Behavior's primitives.

**Pro**: Single cost dimension by construction (gate #39 satisfied structurally); `coercion_cost_equals_complexity_by_construction` is true by construction (gate #38) because coercion is just another operation feeding into the same algebra. End-to-end composition is the carrier's `read` signature.
**Con**: `read` becomes target-aware — needs target-language-spec context threaded through. May force Lens<C> generic's `read: fn(Dag, Behavior) -> Witness<C>` to gain a third parameter (target-context) — substrate-impacting refactor of the generic lens carrier, OR cost lens carries a captured target reference (closure-shape, less structural).

### Option γ — `Lens<SymbolicCost>` reads structural; target-realization via `Lookup<SymbolicCost>`

`Lens<SymbolicCost>::read` reads structural-cost via algebra-fold (Behavior + `SymbolicCost` algebra inhabitance); target-realization-cost composed via existing `Lookup<SymbolicCost>` infrastructure at `lookup.dag:48-60`. The lookup is keyed on per-primitive identity (target-spec-derived); when present, lookup-cost composes into the lens output via `Semiring<SymbolicCost>::sum`.

**Pro**: Reuses existing `Lookup<SymbolicCost>` substrate (per `feedback_audit_adjacent_authority_first` — already authored). Generic `Lens<C>` carrier unchanged; target-realization-cost enters via Lookup's `MissingCost` lens-boundary-fallback shape (already-substrate). Single-cost-dimension preserved (Lookup output IS `SymbolicCost`). `cost_lens_reads_target_realization` (gate #37) is satisfied by Lookup composition; `no_coercion_cost_dimension` (gate #39) is satisfied because Lookup output composes into the same algebra.
**Con**: Lookup-via-Lens-during-read is slightly indirect; consumers must know Lookup is part of the cost computation (vs being lens-internal). Mitigation: lens implementation hides Lookup composition; consumers see only `Lens<SymbolicCost>::read` interface.

## Mgr-tier recommendation

Provisional **γ**: composes via existing `Lookup<SymbolicCost>` substrate (already-authored at `lookup.dag:48-60`); preserves generic `Lens<C>` carrier shape unchanged; satisfies all 4 §1.8 gates by construction (#37 via Lookup, #38 via Semiring composition, #39 via single algebra, #40 via the carrier's typed `read` output). Aligns with `feedback_audit_adjacent_authority_first` + `feedback_compositional_not_templating`.

**β** is second-best if Lookup composition turns out to be insufficient for end-to-end realization-cost reading at canvas-implementation time (e.g., target-realization-cost requires more context than a per-primitive lookup can carry).

**α rejected** — violates gate #39 by construction.

## Director ratification ask

1. **Pick α / β / γ** (or surface fourth option). Provisional Mgr recommendation: **γ** (Lookup composition).
2. Confirm `Lens<C>` generic at `lens.dag:70-77` is authoritative starting point (no carrier-shape refactor in T-CostLens scope). Per option γ this is unchanged; per option β it would refactor.
3. Confirm `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost>` (currently deferred per `dimensions.dag:10`) lands as part of T-CostLens-Composition or stays separately deferred to a Dimensions sub-lane.

## On ratification — worker brief scope

Will author execution brief covering:
- `Lens<SymbolicCost>` instance authoring in DSL (likely `src/v3/lenses/cost.dag` per existing convention; verify-via-grep at brief time)
- `read` implementation per chosen option (γ Lookup composition / β composed witness)
- `cost_lens_demonstration` (gate #70): ≥2 algebra-instances composed + ≥1 recursive call + observable cost-bound output — fixture program + executor wiring same-slice
- 4 §1.8 gates (#37-40) advance to executable status
- `data symbolic_cost_dimension` lands or stays deferred per question 3

## Worker pin (Mgr disposition)

Substrate-fact-introduction precedent owners — valiant-ibex-312 (delivered IntPlatform/UIntPlatform, S5 candidate) OR smart-ram-167. Final pin at dispatch.

## Auto-spawn caveat

Per Director's standing note + cache-staleness cluster ctrl#217: HOLD dispatch on this canvas's worker brief until auto-spawn fix lands per L-sized substrate-fact-introduction threshold.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 post-#2105 merge per Director endorsement of pre-staging T-CostLens-Composition canvas-shape.
