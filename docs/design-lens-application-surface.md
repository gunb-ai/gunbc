# Lens Application Surface

> Part of: [`docs/lens-library-design.md`](lens-library-design.md), [`docs/r3-structure.md`](r3-structure.md), [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md), [`../INVARIANTS.md`](../INVARIANTS.md)
>
> **Purpose:** specify the substrate shape and authoring surface for applying lenses to arbitrary `.dag` sections. This design extends [`docs/lens-library-design.md`](lens-library-design.md) §3 (file-glob `LensApplication`) to function / module / expression / declaration scope, and resolves the violation-policy semantics under fail-closed discipline (resolved as **two separate top-level carriers** — `EnforcedApplication<Output, Budget>` and `IntrospectApplication<Output>` — sidestepping per-variant generics that v3's `.dag` substrate cannot currently express; see §2 + §3).
>
> **Authority discipline:** this is a R3 design doc; the implementation lane is **T-Lens-Application-Surface** (see [`docs/r3-structure.md`](r3-structure.md) lane table). This doc resolves the design questions that block lane dispatch.

## What this document is

[`docs/lens-library-design.md`](lens-library-design.md) §3 specified `LensApplication` with file-glob scope (`applies_to: ["src/v3/compiler/src/**/*.rs", ...]`) and a binary `severity: error | warning` policy. That shape works for *invariant-enforcement* lenses applied to whole source trees (layer opacity over `dsl/std/`, structural duplicates over the manifest) but does not support:

- **Per-function complexity contracts** ("this function should be O(log n)")
- **Per-data-declaration cost basis** ("CRDT field has a per-write cost basis")
- **Per-loop opt-in parallelism** ("this iteration is independence-provable, opt in to parallel emission")
- **Per-expression introspection** ("what is the cost of this subexpression?")

Each of these is a *user-authored* lens application against a structurally-identified section of `.dag`, not a project-wide invariant gate. The user reframe (2026-05-02) names this generalization: `apply_lens(lens, section, config)` where `section` is a structural reference to any `.dag` declaration scope and `config` carries a budget plus a violation policy.

This document specifies the substrate shape for that generalization.

## §1. Scope extension: from file-globs to `.dag` sections

The existing `LensApplication` (file-glob) carrier stays as-is for whole-tree invariant gates. The new substrate consists of **two separate top-level carriers** — `EnforcedApplication` and `IntrospectApplication` — each refining `LensApplication`'s `applies_to` from a path glob to a structural section reference. Together they form the "sectioned lens application" surface; throughout this doc, "the SectionedLensApplication surface" refers to either carrier.

### §1.1 Why structural references, not name-based references

A `.dag` lens application that says `apply_lens(complexity, "my_function", budget)` would be a **name-based** reference (load-bearing per `feedback_naming_is_aliasing` and `feedback_no_metadata_markers`: names are aliases for declarations, not the declarations themselves). Per `feedback_compositional_not_templating` and the existing `feedback_naming_is_aliasing` rule, the lens application must point at the **structural declaration** (the `DeclarationId`), with the name appearing only as a surface-syntax shorthand the parser resolves.

The substrate carrier therefore stores `DeclarationId`, not `String`. The surface syntax can spell `apply_lens(complexity, my_function, budget)` and the parser resolves `my_function` to its `DeclarationId` at the lens-application's introduction site (same resolve path as any other `.dag` reference).

### §1.2 Section granularity

The user reframe names four scopes: *function / module / expression / declaration*. The substrate encoding:

| Scope | Substrate carrier | DeclarationId resolves to |
|---|---|---|
| Function | `DeclarationId` (`Declaration` of arrow-connective shape) | the function declaration |
| Module | `DeclarationId` (the module-as-declaration; existing `module foo` syntax already lowers to a Declaration carrier) | the module declaration |
| Declaration (data / type / etc.) | `DeclarationId` (any Declaration) | the named declaration |
| Expression | `NodeId` (a Node within a Declaration's body sub-DAG) | the specific Node |

The first three uniformly use `DeclarationId`. **Expression scope is the exception** — it requires `NodeId` because Node-level expressions live inside a Declaration's body sub-DAG and have no DeclarationId of their own. The `SectionRef` carrier is therefore a disjoint sum:

```dag
type SectionRef
  = DeclarationScope { declaration: DeclarationId }
  | NodeScope        { declaration: DeclarationId, node: NodeId }
```

`NodeScope` carries both `DeclarationId` (for context — which declaration's body the node lives in) and `NodeId` (the specific node). `DeclarationScope` covers function / module / declaration cases uniformly.

**Why this shape, not "section is just NodeId"**: declarations are the unit at which lenses naturally compose (a complexity lens on a function reads the function's body sub-DAG; a complexity lens on an expression reads only that expression's sub-tree). Bundling DeclarationId into NodeScope keeps the lens-fold composition explicit — every lens application names the enclosing declaration even when narrowing to an expression.

## §2. Substrate carrier shape

The full carrier triple:

```dag
// src/v3/std/lens_application.dag

type SectionRef
  = DeclarationScope { declaration: DeclarationId }
  | NodeScope        { declaration: DeclarationId, node: NodeId }

type DiagnosticSeverity = Error                  // C-8 fail-closed: only Error is admitted

// LensEnforcement<Output, Budget> is the per-lens projection AND
// violation relation. Both are required for the fold-pass enforcement
// check; the projection alone would leave "did the observed value
// exceed the declared budget?" as API-level convention.
//
// - project: extracts the budget-comparable coordinate from the lens
//   output (identity for lenses where Output = Budget; e.g.,
//   summary.asymptotic_class for complexity).
// - violates: per-lens violation relation. Given the user's declared
//   budget and the projected lens-output, returns true iff the projected
//   value EXCEEDS the budget. Each lens's enforcement declares its own
//   violation semantics structurally (lattice ordering for complexity;
//   dominance for cost; mode-mismatch for parallelism).
type LensEnforcement<Output, Budget> {
  project: Output -> Budget
  violates: (declared: Budget, observed: Budget) -> Bool
}

// EnforceableLens<Output, Budget> packages a lens with its CANONICAL
// enforcement (projection + violation relation) into ONE substrate
// authority. Each lens declares its EnforceableLens once; users
// referencing apply_lens cite the EnforceableLens (not lens +
// enforcement separately).
//
// PARSER-LEVEL UNIQUENESS INVARIANT: the parser enforces "at most
// ONE EnforceableLens<C, B> declaration per (Lens<C>, Budget B) pair
// in the program". User code that declares a second EnforceableLens
// referencing the same lens with a type-compatible Budget fails
// parse-time with a Diagnostic naming both declarations as the
// duplicate-canonical-enforcement violation. This is the same shape
// as v3's other single-authority parser invariants (e.g., one
// declaration per name; one BoundedLattice instance per type).
// EnforcedApplication.enforceable_lens references resolve unambiguously
// to the SOLE canonical EnforceableLens for that (lens, Budget) tuple.
//
// Why parser-level, not type-level: full type-level enforcement
// (preventing user code from EVER declaring a second EnforceableLens<C, B>)
// would require existentials or singleton inhabitance — substrate
// features v3 does not fully express today (per `src/v3/std/lens.dag`
// and substrate.dag inhabitance support is sparse). The parser-level
// uniqueness invariant achieves the same single-authority outcome
// (no two competing canonical enforcements for a lens/budget pair)
// at the only construction site v3 currently supports as a structural
// gate. Future substrate hardening (existentials / dependent inhabitance)
// would let this lift to type-level.
type EnforceableLens<Output, Budget> {
  lens: Lens<Output>
  enforcement: LensEnforcement<Output, Budget>
}

// EnforcedApplication<Output, Budget> is the carrier for enforce-mode
// lens applications. References ONE EnforceableLens (the bundled
// authority); cannot pair against arbitrary lens / enforcement
// combinations.
type EnforcedApplication<Output, Budget> {
  enforceable_lens: EnforceableLens<Output, Budget>  // bundled lens+enforcement authority
  section: SectionRef
  budget: Budget                                      // matches enforceable_lens.enforcement.project's target
  diagnostic_severity: DiagnosticSeverity
  span: SourceSpan                                    // user-authored site for diagnostic attribution
}

// IntrospectApplication<Output> is the carrier for introspection-mode
// applications. Parametric in lens output ONLY — no budget, no
// enforcement projection. Per the reviewer-flagged P2 / illegal-states-
// unrepresentable concern: an Introspect application MUST NOT carry
// enforcement metadata it cannot consume.
type IntrospectApplication<Output> {
  lens: Lens<Output>
  section: SectionRef
  span: SourceSpan
}

// EnforcedApplication and IntrospectApplication are TWO separate
// top-level carriers — NOT variants of a single sum. v3's `.dag`
// substrate parameterizes sum types uniformly across variants (each
// variant shares the parent's type parameter set), so a sum where
// Enforce binds Budget but Introspect doesn't is not currently
// expressible. Two separate top-level declarations sidesteps the
// per-variant-generic issue entirely.
//
// The lens-fold pass walks both lists separately and emits Diagnostics
// from EnforcedApplication walks while recording values from
// IntrospectApplication walks. Each top-level declaration in `.dag`
// source is one or the other; user authoring chooses at the apply_lens
// site which form to emit (parsed accordingly).
//
// "SectionedLensApplication" as a noun refers to either form
// collectively (the user-authored surface); it is NOT a sum-type
// declaration in the substrate. The substrate has just the two carriers
// above. Cross-references to "the SectionedLensApplication surface"
// mean "EnforcedApplication and IntrospectApplication taken together".
```

Per `feedback_state_space_vs_behavioral_invariants` + modeling principles 2/6: **Enforce-without-budget / Introspect-with-budget / Introspect-with-enforcement / lens-projection-budget mismatch** are all illegal states; all unrepresentable in the carrier shape. Each top-level carrier carries exactly the type parameters its operation requires; user authoring chooses which top-level form to emit at the `apply_lens` site.

The `Lens<C>` framework (per R2-T-Substrate-Lens-Primitive) parametrizes lenses by their **lens-output carrier** `C` — the type returned by `read(d) -> Lookup<C>`. Per-lens carriers + their enforcement projections:

```dag
// src/v3/lenses/complexity.dag
data complexity_lens: Lens<ComplexitySummary> = ...   // rich output: work/span/asymptotic_class/work_certainty/span_certainty
data complexity_enforcement: LensEnforcement<ComplexitySummary, AsymptoticClass> = {
  project: |summary| summary.asymptotic_class           // budget compares against class only
  violates: |declared, observed|                         // per-lens violation relation
    asymptotic_class_lattice.lt(declared, observed)      // observed exceeds declared in dominance order
}
data complexity_enforceable: EnforceableLens<ComplexitySummary, AsymptoticClass> = {
  lens: complexity_lens
  enforcement: complexity_enforcement                    // canonical pairing — referenced by apply_lens
}

// src/v3/lenses/cost.dag
data cost_lens: Lens<SymbolicCost> = ...              // output IS the budget type
data cost_enforcement: LensEnforcement<SymbolicCost, SymbolicCost> = {
  project: |c| c                                        // identity projection
  violates: |declared, observed|                         // observed dominates declared (worse cost)
    dominates(observed, declared) && !dominates(declared, observed)
}
data cost_enforceable: EnforceableLens<SymbolicCost, SymbolicCost> = {
  lens: cost_lens
  enforcement: cost_enforcement
}

// src/v3/lenses/parallelism.dag
data parallelism_lens: Lens<ParallelismMode> = ...    // output IS the budget type
data parallelism_enforcement: LensEnforcement<ParallelismMode, ParallelismMode> = {
  project: |m| m                                        // identity projection
  violates: |declared, observed|                         // mode-mismatch: user opted in to parallel but lens couldn't prove
    match (declared, observed) {
      (OptInIndependent, Sequential) => True
      _ => False
    }
}
data parallelism_enforceable: EnforceableLens<ParallelismMode, ParallelismMode> = {
  lens: parallelism_lens
  enforcement: parallelism_enforcement
}
```

The `EnforceableLens` declarations are the canonical pairings — apply_lens declarations reference them by name. The standalone `Lens` and `LensEnforcement` declarations exist for `Lens<C>`-only consumers (general lens-fold over substrate without enforcement) and for declared-but-not-yet-bundled enforcement types respectively, but the apply_lens substrate consumes only the bundled `EnforceableLens` form.

**Why the projection rather than a single carrier**: the lens output for complexity is rich (`ComplexitySummary { work, span, asymptotic_class, work_certainty, span_certainty }` — per complexity-lens §1.7, certainty is per-coordinate to avoid collapsing per-dimension proof-tightness facts) — required by the lens-fold composition (per `compose_summary_*` in complexity-lens §3.1). The budget is the user's contract — typically a single class, not a full summary. Forcing budget = output would make users author `Enforce { budget: ComplexitySummary { ... } }` — over-constrained for the common "function should be O(log n)" case. Forcing output = budget would drop work/span/per-coordinate-certainty facts the lens-fold composition needs. The projection separates the two concerns: lens output stays rich (load-bearing for composition); budget stays simple (load-bearing for user authoring).

Lens-output / projection / budget compatibility is **structural by construction**: each `EnforcedApplication<Output, Budget>` declaration ties lens / enforcement / budget via two shared type parameters. A `Lens<ComplexitySummary>` can only pair with a `LensEnforcement<ComplexitySummary, B>` (some Budget B) and a `budget: B` of the same B. Mismatched triples (e.g., complexity-lens with a `SymbolicCost` budget; complexity-lens with cost's identity projection) are unrepresentable in the carrier — the type system rejects them at parse/inference time, not via a separate type-checker rule. (No `budget_type: DeclarationId` field on `Lens<C>` is needed; the type parameters `Output` and `Budget` ARE the structural authority on the `EnforcedApplication` carrier.) Each `IntrospectApplication<Output>` declaration carries only `Output` — no budget, no enforcement projection — so introspection-mode applications cannot accidentally carry irrelevant enforcement metadata.

## §3. Violation policy under fail-closed discipline

The user reframe initially named the violation-policy axis as `CompileError | Warning | Silent`. Under [`../INVARIANTS.md`](../INVARIANTS.md) C-8 (fail-closed compilation) and `feedback_fail_closed_discipline`:

- **CompileError**: every detected violation is a Diagnostic at compile time. *Admitted.*
- **Warning**: violations are logged but compilation succeeds. *Forbidden* — C-8 prohibits warnings as a steady state; the only legitimate "non-blocking detection" is introspection (no enforcement at all).
- **Silent**: violations are silently dropped. *Forbidden* — same C-8 violation; "silent None" is named in `feedback_fail_closed_discipline` as exactly the pattern banned.

The fail-closed-compatible enumeration is therefore binary, not ternary — and the budget + enforcement metadata are bundled INTO `EnforcedApplication<Output, Budget>` by construction (per §2 above). The `IntrospectApplication<Output>` carrier carries only `(lens, section, span)` — no budget, no enforcement projection. The two are **separate top-level declarations** in `.dag`; "Enforce without budget", "Introspect with budget", and "Introspect with enforcement metadata" are all structurally unrepresentable.

The pairing of `budget` + `enforcement` with `Enforce` (and their absence in `Introspect`) is a state-space invariant, not a behavioral one. The type-checker has no illegal combination to reject — those states cannot be constructed (per `feedback_state_space_vs_behavioral_invariants`). Equally, the per-variant type parameters tie `EnforcedApplication.budget` to the lens's declared output type via `LensEnforcement<Output, Budget>` — lens/projection/budget mismatch is also unrepresentable.

`DiagnosticSeverity` is a single-variant nominal carrier (only `Error`). It exists as a refinement-extension point so future *non-fail-closed* surfaces (if ever added — none currently planned) can extend without changing the `Enforce` carrier shape. Per `feedback_no_annotations` and `feedback_no_metadata_markers`, the single-variant carrier is structurally honest: there is exactly one severity, named.

### §3.1 Why no "Warning" mode (load-bearing rationale)

C-8 is the canonical fail-closed compilation rule. A "warning" mode at lens application would:

1. **Allow violations as steady state**, normalizing the bridge-as-steady-state P5 pattern.
2. **Diverge from the rest of the compiler** (no other surface has warnings; introducing them at lens application creates parallel disposition authority).
3. **Drift toward "annotation as advice"**, which contradicts `feedback_no_annotations` (lens applications should be first-class structural facts, not advisory markers).

If a user wants to "see the lens value but not enforce", that is `Introspect` mode — the lens runs, produces a value, and the value is available for downstream lenses (or for human reading via debug surfaces). No diagnostic, no enforcement, no warning. This is a different operation than enforcement-with-a-warning; it has clean semantics.

### §3.2 Default policy for complexity contracts — user-driven

Per Director ratification 2026-05-02 + user directive ("if we have suboptimal algorithms, we throw a compiler error"): complexity contracts are **user-authored**, with explicit budgets. There is no implicit auto-inferred baseline (resolution per §8.3 — auto-inferred baseline lacks a persisted authority and would lose the regression fact on every recompile).

**Default for unannotated functions**: the compiler emits an implicit `Introspect`-mode application during the lens fold — lens value is computed and surfaced for inspection; no enforcement, no compile error. (Per §5.1: this synthesis is fold-pass-only; not stored in source.)

**Enforcement opt-in**: the user explicitly authors `apply_lens(complexity, fn, Enforce { budget: <chosen class>, diagnostic_severity: Error })` for functions where they want to commit to a complexity contract. Compile errors fire when the function's actual complexity exceeds the named budget.

**Waiver**: when a user previously authored an `Enforce` budget but later wants to relax it (the function legitimately needs a worse class for a justified reason), they author a structural waiver:

```dag
data my_function_complexity_waiver: ComplexityBudgetWaiver = {
  function: my_function          // DeclarationId reference
  budget: O_log_n                // the budget being waived
  justification: "Inputs always small (≤16 items per Domain.MaxItems); measured wall time stable across versions"
}
```

This is a `.dag` declaration, not an annotation (per `feedback_no_annotations`). The waiver is itself subject to the same lens infrastructure — `lens_unused_parameters` could (eventually) flag stale waivers, `lens_structural_duplicates` flags duplicate waivers, etc.

**Why a separate `ComplexityBudgetWaiver` carrier, not just an `Introspect` application**: a waiver is *not* "the user wants introspection" — it is "the user accepts a known violation, with justification". The justification is load-bearing (per `feedback_reason_not_label` — encode the stable reason, not the volatile label). Conflating waivers with introspection would lose the justification.

## §4. Worked examples (the 4 demonstrations per Director ratification)

The lens-application surface ships with four worked examples covering orthogonal axes (per Director ratification 2026-05-02 — these are the 4 demonstrations the lane closure-gate enumerates).

**TestClaim shape for all four demonstrations**: each closure-gate test is a `TestClaim` per DB-15 ([`docs/design-test-infra.md`](design-test-infra.md)) — *enumerated, not quantified*. The demonstrations exercise specific, named programs (one per closure gate) with specific predicates (compile-error firing / lens-output equality / emission-shape matching). Property-based `QuantifiedTestClaim` form (per [`docs/design-tests-as-data-completeness.md`](design-tests-as-data-completeness.md) §2.2) is *not* used here — the closure gates test concrete demonstration programs, not universally-quantified properties. (Property tests over the lens-application substrate itself live in T-Tests-As-Data-Completeness scope, not T-Lens-Application-Surface scope.)

### §4.1 Complexity-contract compile error

**User intent**: "this function should be O(log n); fail compilation if it isn't."

```dag
// in user .dag source

apply_lens(complexity, my_search_function, Enforce {
  budget: O_log_n
  diagnostic_severity: Error
})
```

**Substrate after parsing**:

```dag
data __apply_lens_my_search_function: EnforcedApplication<ComplexitySummary, AsymptoticClass> = {
  enforceable_lens: complexity_enforceable            // bundled lens+enforcement (per §2)
  section: DeclarationScope { declaration: my_search_function }
  budget: O_log_n
  diagnostic_severity: Error
  span: <user-authored span>
}
```

**Compiler-side processing**: during the lens fold (existing infrastructure per [`docs/design-lens-framework.md`](design-lens-framework.md)), the compiler iterates over every `EnforcedApplication` declaration in the program. For each, it (a) runs the named lens against the named section to obtain the rich output, (b) calls `enforcement.project(output)` to get the budget-comparable value, (c) calls `enforcement.violates(declared_budget, projected)` to decide, (d) emits a Diagnostic if `violates` returns true. The violation relation is per-lens substrate authority (declared in the per-lens `LensEnforcement`); the fold-pass dispatch reads it directly. (`IntrospectApplication` declarations are walked separately; see §5.)

**Closure gate**: `complexity_violation_compile_error_demonstrated` — a TestClaim that constructs a function with O(n²) body + a lens application requiring O(log n) + asserts a Diagnostic is produced.

### §4.2 CRDT cost basis

**User intent**: "this CRDT data declaration has a per-write cost basis of O(log replicas); cost lens reads this when composing."

**Worked-example discipline**: the audit
[`docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md`](audit/t-user-authored-cost-basis-discipline-worked-examples.md)
splits this into two facts: the lens application owns `(lens, section, config, span)`, while a cost-lens-owned cost-basis declaration owns the `PerWrite` basis evidence. T-CostLens-Composition derives `N * log(replicas)` from writes and loop structure; the derived cost is not a second user-authored basis fact.

```dag
apply_lens(cost, my_crdt_field, Enforce {
  budget: SymbolicCost { per_op: O_log_replicas }
  diagnostic_severity: Error
})
```

**Substrate after parsing**: same shape as §4.1 with `lens: cost`, `section: DeclarationScope { declaration: my_crdt_field }`.

**Compiler-side processing**: the cost lens reads the CRDT field's per-write cost. When composing the field's cost with surrounding cost-lens applications (per T-CostLens-Composition), the per-op budget participates in the composition. A program that writes to the CRDT field N times in a loop sees O(N · log replicas) per the composition rules.

**Closure gate**: `crdt_cost_basis_demonstrated`.

### §4.3 Memory-peak cost basis

**User intent**: "this function's memory peak is O(input size); cost lens with the memory dimension reads this."

**Worked-example discipline**: the audit
[`docs/audit/t-user-authored-cost-basis-discipline-worked-examples.md`](audit/t-user-authored-cost-basis-discipline-worked-examples.md)
keeps the application config separate from memory-basis evidence. The later memory-peak demonstration must declare peak composition semantics (for example max/live-overlap behavior) in the cost-lens authority; generic application config does not decide peak algebra.

```dag
apply_lens(cost, my_memory_intensive_function, Enforce {
  budget: SymbolicCost { dimension: Memory, per_call: O_input_size }
  diagnostic_severity: Error
})
```

**Substrate after parsing**: same shape. The cost lens carries multi-dimension `SymbolicCost` (per existing `Dimension<SymbolicCost>` substrate from T-CostLens-Composition).

**Compiler-side processing**: the cost lens with `Dimension::Memory` reads only the memory portion of `SymbolicCost`. Time-dimension and memory-dimension are independent — both lens applications can apply to the same function with different budgets per dimension.

**Closure gate**: `memory_peak_cost_basis_demonstrated`.

### §4.4 Opt-in cross-iteration parallelism

**User intent**: "this loop's iterations are independence-provable; opt in to parallel emission."

```dag
apply_lens(parallelism, my_loop_expression, Enforce {
  budget: ParallelismMode::OptInIndependent
  diagnostic_severity: Error
})
```

**Substrate after parsing**: `section: NodeScope { declaration: enclosing_function, node: my_loop_node }` (this is the only example using `NodeScope` rather than `DeclarationScope`, because loop expressions are Node-level, not Declaration-level).

**Compiler-side processing**: the parallelism lens (per T-Lens-Behavioral-Parity slice 3) checks whether the loop's iterations are independence-provable (no cross-iteration `Bind` dependencies). If so, the emitter generates parallel iteration code; if not, the diagnostic fires (the user requested opt-in but the lens cannot prove independence — failed contract).

**Closure gate**: `opt_in_iteration_parallelism_via_lens_application_demonstrated`.

## §5. Lens-fold integration

The compiler already has a lens-fold pass (per [`docs/design-lens-framework.md`](design-lens-framework.md) and the active `Lens<C>` framework). The lens-application surface adds one new step to that fold:

1. **Existing**: walk every `Lens<C>` instance, apply to every program node, accumulate `Witness<C>`.
2. **New**: walk every `EnforcedApplication` declaration AND every `IntrospectApplication` declaration (two separate walks; not one sum-walk). For each `EnforcedApplication { enforceable_lens, section, budget, ... }`: run `enforceable_lens.lens` to get output; compute `projected = enforceable_lens.enforcement.project(output)`; if `enforceable_lens.enforcement.violates(budget, projected)` returns true, emit a Diagnostic. For each `IntrospectApplication { lens, ... }`: run lens, record output for downstream lens composition + debug surfaces; no comparison.

The new step is structurally identical to the existing fold — same `Lens<C>` reader, same `Witness<C>` output type — with an additional budget comparison and Diagnostic emission. The implementation cost is O(1) lens-applications-per-program (typically tens to hundreds per project, not millions).

### §5.1 Default-application synthesis (Introspect-only)

For unannotated functions (per §3.2), the compiler synthesizes `IntrospectApplication<ComplexitySummary>` records implicitly during the lens fold — one per function declaration, as `IntrospectApplication { lens: complexity_lens, section: DeclarationScope { ... }, span: <synthesized> }`. These synthesized records are not stored in the `.dag` source; they exist only during the fold pass.

The synthesizer **never** emits `Enforce` mode for unannotated functions — auto-inferred budgets lack persisted authority (per §8.3). Enforcement requires explicit user authoring of `apply_lens(complexity, fn, Enforce { budget: <class>, ... })`. The synthesizer's role is purely to ensure every function has at least Introspect coverage so the lens-fold pass produces a value for every port (for downstream lens composition + debug surfaces).

**Why fold-pass synthesis rather than no-synthesis**: per `feedback_compositional_not_templating`, the lens fold is a structural walk — every declaration produces a value. The Introspect synthesis ensures the walk is total over function declarations without requiring explicit user authoring for every function.

User-authored `apply_lens(complexity, my_function, Introspect)` is functionally identical to the synthesized default; the explicit form is only useful for documentation purposes (signaling "I considered enforcement and chose introspection").

## §6. Cross-program coordination

This lane is **cross-program** between Substrate Manager and Verification Manager (per [`docs/r3-structure.md`](r3-structure.md) lane 16):

- **Substrate Manager owns**: the `EnforceableLens<Output, Budget>` / `EnforcedApplication<Output, Budget>` / `IntrospectApplication<Output>` / `SectionRef` / `LensEnforcement<Output, Budget>` parametric carriers in `src/v3/std/lens_application.dag` (EnforceableLens is the bundled authority that EnforcedApplication references — packaging lens + enforcement into ONE substrate authority per P2 single-authority discipline); the compiler-side lens-fold integration (two separate walks — Enforce list reads `enforceable_lens.lens/enforcement`, Introspect list reads `lens` only); per-lens `EnforceableLens` declarations co-located with each lens — these are the canonical bundled authorities apply_lens references (one per lens — complexity, cost, parallelism, effect_enumeration). (No `Lens<C>.budget_type` field — the type parameters Output + Budget are the structural authority on each separate carrier; the EnforceableLens bundle prevents non-canonical lens/enforcement pairings at the application site.)
- **Verification Manager owns**: the closure-gate TestClaims (`complexity_violation_compile_error_demonstrated`, `crdt_cost_basis_demonstrated`, `memory_peak_cost_basis_demonstrated`) for **R3**; plus **`opt_in_iteration_parallelism_via_lens_application_demonstrated`** on the **R4** horizon (C1 — schedules with parallelism lens completeness per §7); cross-target equivalence on lens-application semantics (does Rust-emitted code respect the budget in the same way Python-emitted does?).

The split mirrors the existing T-CostLens-Composition split: substrate authors carriers + fold semantics, Verification asserts the demonstrations.

## §7. Cascade gates

Per [`docs/r3-structure.md`](r3-structure.md) + [`docs/r4-carve-out-routing.md`](r4-carve-out-routing.md):

- **Substantive-semantics principle:** each worked example requires its corresponding lens to be **behaviorally substantive** — a `complexity_violation_compile_error_demonstrated` TestClaim requires the complexity lens to compute correct asymptotic classes (not just the depth proxy currently shipped). Likewise cost / parallelism for their respective demos.
- **R3 program reconciliation (T-LBP option (b) RATIFIED 2026-05-06; AMENDED 2026-05-09 carve-promotion-IN-R3 per Director ratification at gunbc#846 #issuecomment-4412330468 + (a) at #issuecomment-4412380947):** R3 T-LBP closes **all 4 lenses** behavioral completeness — complexity + cost (original (b) scope) + parallelism + effect_enumeration (carve-promoted from R4 C1/C2 into Cluster F per [`docs/audit/r3-cluster-f-sequencing-plan-2026-05-09.md`](audit/r3-cluster-f-sequencing-plan-2026-05-09.md)). R3 T-Lens-Application-Surface lands substrate **88–91** + demos **92–95** after relevant lens completeness + register full-scope (prior C3 narrowed-scope DISSOLVED).
- **Gate #95 / §4.4:** `opt_in_iteration_parallelism_via_lens_application_demonstrated` requires the **parallelism** lens implementation referenced in §4.4 (T-LBP slice 3). **R3-LOAD-BEARING (carve-promoted-IN-R3 2026-05-09)** within Cluster F sub-phase F-γ.1 — cascade-gates on F-α (parallelism BEHAVIORALLY COMPLETE #81) + T-LAS Slice B substrate landing (#91 per-lens LensEnforcement projection). Prior R4-CARVED (C1 cascade) status DISSOLVED.
- **External cascade:** R2-Evaluator landed (per the standard R3 worker-dispatch precondition).

Pre-cascade *design-doc* work is permitted (this doc); pre-cascade *substrate work* for carriers consumed by **R3** demos **92–95** waits for relevant T-LBP slices COMPLETE per Cluster F sequencing plan.

## §8. Resolved design questions

Five design questions surfaced during authoring. Per `feedback_design_before_implement` ("resolve all design questions before implementation"), each is resolved here rather than deferred to lane dispatch.

### §8.1 Module-level lens application semantics — RESOLVED: aggregate-across-module

**Question:** When a user writes `apply_lens(complexity, my_module, budget)`, does the budget apply to *each function in the module* or to the *aggregate cost across the module*?

**Resolved:** aggregate-across-module. Each module-level application is one budget for the whole module's cost composition. Per-function budgets require explicit per-function applications.

**Why:** the alternative (budget applies to each function) would mean the same `apply_lens(complexity, my_module, O_log_n)` semantically expands to N independent applications (one per function in the module). That violates §8.2 single-authority discipline (each function gets a budget without an authoring receipt) and conflates module-scope with function-scope (different SectionRef variants with the same semantics). Aggregate-across-module preserves the structural fact that "module" is a different scope than "every function in module" — they are distinct concepts and should have distinct semantics.

**Implementation note:** the lens-fold pass, when processing a `DeclarationScope { declaration: <module DeclarationId> }`, reads the module's aggregate cost composition (fold over all top-level declarations within the module) rather than synthesizing per-function budgets.

### §8.2 Multiple applications on the same section — RESOLVED: fail-closed-reject

**Question:** What happens when a user writes two `apply_lens(complexity, ...)` applications targeting the same function with different budgets?

**Resolved:** type-checker rejects. Same `(lens, section)` pair appearing twice with `Enforce` mode in both is a fail-closed authoring violation per P2 (single-authority discipline).

**Permitted exception:** one `Enforce` + multiple `Introspect` applications on the same section are admitted. Introspect mode produces no budget, only a lens value reading; multiple readings of the same section by the same lens have identical results (lens is a pure function), so the duplication is structurally idempotent rather than authority-conflicting.

**Implementation note:** at parse time, the type-checker maintains a per-`(lens, section)` count of `Enforce` mode applications. Count > 1 fires a Diagnostic naming both authoring sites. `Introspect` mode is unbounded.

### §8.3 Default-application semantics — RESOLVED: user-driven contracts; no implicit baseline

**Question:** For functions without an explicit `apply_lens(complexity, ...)` declaration, what does the compiler do?

**Resolved:** the compiler emits an implicit `Introspect`-mode application — no enforcement, just lens-value computation for inspection. **There is NO implicit Enforce-with-inferred-budget mode.** The user gets enforcement only by explicitly authoring `apply_lens(complexity, fn, Enforce { budget: <user-named class>, ... })`.

**Why no implicit Enforce-with-inferred-baseline** (per codex BLOCKING on PR #1488 sha 265d8ef7): an inferred baseline computed from the current function body has no persisted authority. On the next compile, the synthesizer recomputes from the new (potentially-worse) body, getting the new class as both "current value" and "baseline" — they always agree, and no regression ever fires. The fact the design claimed to enforce (regression detection) was actually lost on every recompile. P2 single-authority + facts-flow-forward: a regression baseline must be a *declared authority* (in source), not an *ephemeral recomputation*.

**Why not synthesizer-emitted source declarations** (alternative considered): the synthesizer could write the inferred baseline back to source as an `apply_lens` declaration the user checks in. This is generated-code-on-disk, forbidden by `feedback_no_generated_code_on_disk`. It also changes the user's source file as a side effect of compilation — surprising and reversible-by-edit only.

**The structural answer**: complexity contracts are user-authored. The user decides which functions have a budget and what it is. The compiler's role is (a) introspection (compute and report) for unannotated functions, (b) enforcement for explicitly-authored `Enforce` applications. No magic baseline.

**Re-framing "opt-out for complexity"** (per the original user directive 2026-05-02 "if we have suboptimal algorithms, we throw a compiler error"): "opt-out" means the user *can* opt out of enforcement (via `Introspect` or via no application at all); the *compile error* fires when the user *opts in* with a budget the actual function exceeds. The compiler does not infer suboptimality; the user names what they consider acceptable, and divergence from the named contract fires.

**`ComplexityBudgetWaiver`** retains its original purpose: when the user authored `apply_lens(complexity, fn, Enforce { budget: O_log_n, ... })` and later wants to relax it (the function legitimately needs O(n) for a justified reason), the waiver structurally records the exception with `justification`. The waiver is for accepted-known-violations of explicit user contracts, not for absent-implicit-baseline contracts.

### §8.4 Waiver lifecycle — RESOLVED: separate-lens future scope, dissolution-trigger named

**Question:** `ComplexityBudgetWaiver` declarations need a path to dissolution — a stale waiver should be detected.

**Resolved:** stale-waiver detection is a future lens (`lens_stale_waivers`) tracked outside T-Lens-Application-Surface scope. **Dissolution trigger named explicitly per P5 scaffold-discipline**: when `lens_stale_waivers` ships, every `ComplexityBudgetWaiver` whose target's actual lens value no longer exceeds the budget is a stale-waiver finding. The user must either delete the waiver (if no longer needed) or update its `justification` field (if still load-bearing). The lens-fold pass enforces no semantics on stale waivers until that future lens lands; pre-`lens_stale_waivers`, waivers are honored unconditionally.

**Why split out:** `lens_stale_waivers` is a separate lens with its own substrate input (every waiver) and its own enforcement output (per-waiver staleness Diagnostic). Bundling it into T-Lens-Application-Surface scope would inflate this lane; the dissolution trigger is named so the future lens has clear input semantics.

**Tracking:** add `lens_stale_waivers` to [`docs/lens-library-design.md`](lens-library-design.md) §6 (future lenses) when this design doc lands.

### §8.5 Cross-section composition — RESOLVED: read declared budget, not computed class

**Question:** When function `f` is `apply_lens(complexity, f, O_log_n)`-ed and another function `g` calls `f`, does `g`'s complexity lens read `f`'s declared budget or its actual computed class?

**Resolved:** read the declared budget. The budget is the user's contract; reading the computed class would let `g`'s emission depend on `f`'s implementation rather than its contract.

**Why:** reading computed class would mean a refactor of `f` that lowers its complexity (e.g., O(n log n) → O(n)) would propagate into `g`'s lens result *automatically*. That makes `g`'s emission silently dependent on `f`'s implementation choices. The declared budget is the abstraction barrier — `g` reads "f's contract is O(log n)" and composes against that. If `f`'s implementation diverges from its declared budget, that is `f`'s own Diagnostic (the lens application on `f` fires), not `g`'s.

**Implementation note:** the lens-fold pass, when computing `g`'s complexity composition, looks up `f`'s applications. If `f` has an `EnforcedApplication` declaration: read `EnforcedApplication.budget` as the declared contract. If `f` has only an `IntrospectApplication` (or no application at all): the lens-fold reads the computed class as fallback (the absence of an enforced contract means there is no abstraction barrier; introspection-only carries the lens value but no budget).

**Bridge-as-steady-state avoidance:** this resolution preserves the cost-of-change=1 principle (changing `f`'s implementation does not require re-checking every caller's lens result) and keeps abstraction-barrier semantics consistent with the rest of the substrate (a function's *signature* is what callers depend on; the body is private detail).

---

All five questions resolved. Implementation can proceed without further Director ratification on these specific points. Cascade gates: **R3** LAS substrate+demos **88–94** wait on **complexity+cost** T-LBP COMPLETE + **C3**; **#95** waits on **R4** parallelism parity (**C1**). External dependency: R2-Evaluator landed.

## §9. Relationship to existing authority

This design doc extends:

- [`docs/lens-library-design.md`](lens-library-design.md) §3 — the existing file-glob `LensApplication`. **No changes to existing carrier**; this doc adds `EnforcedApplication` + `IntrospectApplication` as sibling carriers with structural section references.
- [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) — the lens-capability register tracking PROXY/STUB/PARTIAL/COMPLETE status per lens. **R3 (post-carve-promotion 2026-05-09):** LAS demos **92–95** assume the relevant lens reaches BEHAVIORALLY COMPLETE first per Cluster F sequencing plan; **`opt_in_iteration_parallelism_via_lens_application_demonstrated` (#95)** assumes **parallelism** COMPLETE (R3-load-bearing within Cluster F sub-phase F-α; prior R4-carved C1 framing DISSOLVED). **R3 horizon:** register reaches ZERO PROXY / ZERO STUB across all four behavioral lenses (complexity + cost + parallelism + effect_enum) per Director carve-promotion ratification 2026-05-09 c#4412330468 (prior R4-horizon framing DISSOLVED).
- [`docs/design-lens-framework.md`](design-lens-framework.md) — the `Lens<C>` framework. This design adds one fold-pass extension (lens-application discovery + budget comparison) but does not modify the underlying `Lens<C>` shape.
- [`../INVARIANTS.md`](../INVARIANTS.md) C-8 (fail-closed compilation) — load-bearing for §3 (no Warning, no Silent policies).
- [`../INVARIANTS.md`](../INVARIANTS.md) P2 (boundary discipline) — load-bearing for §8.2 (single-authority for `(lens, section)` pairs).
- [`../INVARIANTS.md`](../INVARIANTS.md) P5 (progress is dissolution) — load-bearing for §8.4 (waivers must have dissolution paths).

This document does NOT modify:

- The existing `Lens<C>` carrier shape (per T-Substrate-Lens-Primitive — that is R2 work already complete).
- The existing file-glob `LensApplication` carrier (per [`docs/lens-library-design.md`](lens-library-design.md) §3 — sibling, not replacement).
- Per-lens budget types — each lens declares its own `LensEnforcement<Output, Budget>` projection (per §2); this doc only specifies the parametric dispatch carriers `EnforcedApplication<Output, Budget>` + `IntrospectApplication<Output>` (two separate top-level types). The lens-output type stays whatever the lens chooses (rich `ComplexitySummary` for complexity; identity-projected `SymbolicCost` for cost).

## §10. Implementation order (sketch)

Within T-Lens-Application-Surface lane (per [`docs/r3-structure.md`](r3-structure.md) closure gates):

1. **Substrate carriers landing** (`lens_application_carrier_landed`, `section_ref_substrate_landed`, `lens_enforcement_carrier_landed`). Author `src/v3/std/lens_application.dag` per §2 — **two separate top-level carriers**: `EnforcedApplication<Output, Budget>` and `IntrospectApplication<Output>`, plus `SectionRef` and `LensEnforcement<Output, Budget>`. NO sum-type wrapping the two (per §2 — v3 `.dag` substrate cannot currently express per-variant generics). Each top-level carrier holds exactly its required parameters (Enforce: Output + Budget; Introspect: Output only). Co-locate per-lens `LensEnforcement` declarations with each lens (complexity → AsymptoticClass projection; cost / parallelism → identity). Type-checker integration: `(lens, section)` single-authority enforcement at parse time. (Lens/projection/budget compatibility is structural via shared Output and Budget parameters per top-level carrier; Introspect cannot accidentally carry enforcement metadata.)
2. **Fold-pass integration** (`enforce_violation_routing_landed`). Extend the lens-fold pass to walk `EnforcedApplication` declarations + emit Diagnostics on budget violations (via `enforcement.violates`); separately walk `IntrospectApplication` declarations + record lens values for downstream composition / debug surfaces. Two separate walks (no sum to dispatch on). Synthesized default-application per §5.1.
3. **Worked example #1: complexity contract** (`complexity_violation_compile_error_demonstrated`). TestClaim per §4.1.
4. **Worked example #2: CRDT cost basis** (`crdt_cost_basis_demonstrated`). TestClaim per §4.2.
5. **Worked example #3: memory-peak cost basis** (`memory_peak_cost_basis_demonstrated`). TestClaim per §4.3.
6. **Worked example #4: opt-in parallelism** (`opt_in_iteration_parallelism_via_lens_application_demonstrated`). TestClaim per §4.4 — **R4 horizon**: requires **`parallelism_lens_behaviorally_complete`** (C1) per §7; schedules **after** worked examples **1–3** (plan gates **92–94**) land for **R3**.

Steps 1-2 are sequential (carriers must exist before fold-pass consumes them). Steps 3-5 are parallel-dispatchable for **R3** (each is an independent worked example referencing the same fold-pass infrastructure). Step **6** schedules with **R4** parallelism parity — **not** R3-conjunction load-bearing (§7).

Total estimate (per L-XL sizing in the lane row): substrate carriers + fold-pass = M-L; 4 worked examples = M each in parallel = L overall. End-to-end: 4-6 weeks worker time at standard R3 cadence.

---

**This document is a design spec, not a ship target.** It resolves the structural design questions blocking T-Lens-Application-Surface lane dispatch. The lane runs when cascade gates clear per §**7** (**R3** vs **R4** split for option **(b)**) + R2-Evaluator landed. All §8 design questions resolved in-doc; no Director ratification required before substrate authoring begins within that split.
