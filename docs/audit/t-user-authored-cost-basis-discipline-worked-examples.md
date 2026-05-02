# T-User-Authored-Cost-Basis-Discipline — worked examples audit

**Date:** 2026-05-02
**Lane:** R3 T-Lens-Application-Surface, formerly the standalone T-User-Authored-Cost-Basis-Discipline gap.
**Authority:** R3 scope expansion #1480 as recorded in [`docs/r3-structure.md`](../r3-structure.md), [`docs/design-lens-application-surface.md`](../design-lens-application-surface.md), [`docs/design-lens-framework.md`](../design-lens-framework.md), and the current cost substrate in [`src/v3/lenses/cost.dag`](../../src/v3/lenses/cost.dag).

This is a docs-only audit. It does not author `SectionedLensApplication`, a cost-basis carrier, a `data cost_lens` instance, memory-peak semantics, or broad cost-lens composition. The purpose is to make the CRDT and memory-peak worked examples precise enough that the later substrate slice has one authority for each fact.

## Decision

User-authored cost-basis discipline should stay inside T-Lens-Application-Surface, but not by stuffing every cost fact into `ApplicationConfig`.

The split is:

| Fact | Authority | Why |
|---|---|---|
| "Run lens L on section S, enforcing budget B" | lens application config (`SectionedLensApplication`, `SectionRef`, `ApplicationConfig`) | This is the user-authored application event. It owns lens identity, target section, mode, budget value, diagnostic routing, and source span. |
| "This declaration/operation has basis cost X" | the cost lens's own cost-basis declarations | This is domain evidence consumed by the cost lens. It must be typed by the basis kind (`PerWrite`, `PerCall`, `PeakMemory`, etc.) and attached to the structural subject, not hidden in the generic application carrier. |
| "Composed result is N * X, max(path costs), peak(live allocations), ..." | T-CostLens-Composition / cost-lens fold | This is lens semantics. Composition reads basis declarations plus program structure (`LoopBound`, writes/calls/effects, target realization costs) and produces `SymbolicCost` or a dimension-specific projection. |

So the application surface is a dispatcher and policy carrier; cost-basis declarations are the evidence; cost-lens composition is the algebra.

## Worked example 1 — CRDT per-write cost basis

User intent from the design doc:

```dag
apply_lens(cost, my_crdt_field, Enforce {
  budget: SymbolicCost { per_op: O_log_replicas }
  diagnostic_severity: Error
})
```

The phrase "per-write cost basis" should not mean the generic `Enforce` record owns write semantics. The later substrate should represent the example as two facts:

1. A lens application fact: apply the cost lens to the CRDT declaration or field section with an enforced budget and error severity.
2. A cost-basis fact: the CRDT field's write operation has a basis cost of `O(log replicas)`.

The cost lens then composes with program structure: if a loop writes the CRDT field `N` times, T-CostLens-Composition multiplies the loop measure by the per-write basis and reports `O(N * log replicas)`.

### Authority assignment

| Detail | Belongs in |
|---|---|
| target field/declaration | `SectionRef` inside lens application |
| fail-closed enforcement mode | `ApplicationConfig::Enforce` |
| diagnostic span | `SectionedLensApplication.span` |
| `replicas` size variable | cost-basis declaration or referenced substrate fact, not `SectionRef` |
| `PerWrite` basis kind | cost-basis declaration |
| write-count extraction | cost-lens composition over program structure |
| loop multiplication | cost-lens composition using `LoopBound` / `loop_bound_measure` |

### Tiny carrier candidate

The smallest honest future carrier is not a general "cost basis string"; it is a typed declaration owned by the cost lens, for example:

```dag
type CostBasisKind
  = PerWrite
  | PerCall
  | PeakMemory

type CostBasisDeclaration {
  subject: SectionRef
  kind: CostBasisKind
  cost: SymbolicCost
  span: SourceSpan
}
```

This audit does **not** author that carrier because `SectionRef` / `SectionedLensApplication` have not landed yet and because `PeakMemory` needs composition semantics before it is safe to put in a shared enum. The carrier candidate is a shape target for the later T-Lens-Application-Surface substrate slice.

## Worked example 2 — memory-peak cost basis

User intent from the design doc:

```dag
apply_lens(cost, my_memory_intensive_function, Enforce {
  budget: SymbolicCost { dimension: Memory, per_call: O_input_size }
  diagnostic_severity: Error
})
```

Memory peak is not the same algebra as sequential work. Work usually composes by addition over sequence and multiplication over iteration; peak memory composes by maximum over non-overlapping lifetimes and by live-range overlap over simultaneous allocations. Therefore memory-peak facts must not be treated as "just another work budget" inside generic application config.

The later substrate should again split two facts:

1. Lens application: apply the cost/memory lens to the function section with an enforced peak budget.
2. Cost-basis declaration: the function or allocation pattern has a `PeakMemory` / `PerCall` basis of `O(input size)`.

T-CostLens-Composition, or a memory-specific cost-lens slice under the same lane, owns the peak algebra. It must say whether sequential calls use `max`, whether allocations overlap, and how branch/loop peaks compose. The application surface does not decide that.

### Authority assignment

| Detail | Belongs in |
|---|---|
| target function | `SectionRef` inside lens application |
| user-enforced budget | `ApplicationConfig::Enforce` |
| memory dimension / peak basis kind | cost-basis declaration owned by the cost lens |
| input-size variable | cost-basis declaration or existing size-variable substrate |
| peak-vs-work algebra | cost-lens composition |
| allocation lifetime / overlap evidence | cost-lens composition inputs, not application config |

### Sequencing caution

`docs/design-lens-framework.md` originally treated memory-peak as a stretch instance because pure monoidal composition was enough for the first three framework examples. The 2026-05-02 R3 expansion pulls memory-peak into T-Lens-Application-Surface as a worked example, but that does not make the peak algebra free. The worked example can land only after the cost lens has an explicit memory/peak composition rule or after the lane names a memory-specific lens instance. Until then, the application carrier can accept the authoring event, but the demonstration gate `memory_peak_cost_basis_demonstrated` must remain blocked.

## 6Q audit

### Q1 - Carrier invariants

**PASS for docs-only.** The proposed split keeps application facts small and total: one lens, one section, one mode. Cost-basis declarations can then enforce basis-kind-specific invariants without making `ApplicationConfig` a bag of optional fields.

### Q2 - Index / handle types

**PASS.** The target subject should be `SectionRef` (`DeclarationId` or `NodeId` with declaration context), not a string name. Size variables and loop measures should reuse existing substrate handles (`PortId`, `SizeVariable`, `LoopBound` accessors) rather than introduce name-keyed side tables.

### Q3 - Duplicated fact

**BLOCKER unless split as above.** If `ApplicationConfig.Enforce.budget` is treated as both "budget threshold" and "basis declaration," the same cost fact lives in two conceptual places: the application policy and the cost-lens evidence. The discipline is: budget threshold in config; basis evidence in a cost-basis declaration.

### Q4 - Coproduct compression

**PASS with future carrier caution.** `PerWrite`, `PerCall`, and `PeakMemory` are not aliases. They compose differently and should be variants only if the cost lens owns a shared `CostBasisKind` sum with variant-specific interpretation. A central union outside the cost lens would recreate the budget-roster problem rejected by `docs/design-lens-application-surface.md`.

### Q5 - Construction authority

**PASS if authored structurally.** Users author lens applications and cost-basis declarations in `.dag`; the parser/type-checker resolves names to `SectionRef`. The compiler may synthesize default applications for complexity regression detection, but it should not synthesize CRDT or memory basis declarations without an authoring receipt.

### Q6 - Representation duality

**BLOCKER unless composition owns derived facts.** The examples must not store both "per-write O(log replicas)" and "looped write O(N log replicas)" as peer user-authored facts. The former is basis evidence; the latter is a derived lens result. Likewise, "peak memory O(input)" must not be stored as both a function budget and an allocation-overlap result.

## Deliverable decision

**Proceed docs-first only.** No tiny carrier is safe to author before `SectionRef` / `SectionedLensApplication` land and before memory-peak composition semantics are explicit. The next substrate slice should first land the application carriers from `docs/design-lens-application-surface.md`, then add a cost-lens-owned cost-basis declaration carrier with at least CRDT `PerWrite` and memory `PeakMemory` worked examples.

## Follow-up gates

- `lens_application_carrier_landed`: lands `SectionRef`, `ApplicationConfig`, and `SectionedLensApplication`.
- `cost_basis_declaration_carrier_landed`: lands the cost-lens-owned basis declaration shape; should not be a central `LensBudget` union.
- `crdt_cost_basis_demonstrated`: proves per-write basis composes with write count / loop measure.
- `memory_peak_cost_basis_demonstrated`: proves peak algebra is declared, not inferred from work-cost composition.
- `no_parallel_cost_basis_authority`: ratchets that derived composed costs are not stored as user-authored basis declarations.

## Non-goals

- No `data cost_lens: Lens<SymbolicCost>`.
- No broad cost-lens instance implementation.
- No generic `LensBudget = Cost | Complexity | Parallelism | ...` roster.
- No annotations or metadata markers.
- No target-runtime measurement semantics.
- No parser, tokenizer, or syntax work in this audit slice.
