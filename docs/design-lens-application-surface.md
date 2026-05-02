# Lens Application Surface

> Part of: [`docs/lens-library-design.md`](lens-library-design.md), [`docs/r3-structure.md`](r3-structure.md), [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md), [`../INVARIANTS.md`](../INVARIANTS.md)
>
> **Purpose:** specify the substrate shape and authoring surface for applying lenses to arbitrary `.dag` sections. This design extends [`docs/lens-library-design.md`](lens-library-design.md) §3 (file-glob `LensApplication`) to function / module / expression / declaration scope, and resolves the violation-policy semantics under fail-closed discipline (resolved as a sum-type `ApplicationConfig = Enforce { budget, ... } | Introspect` — illegal states unrepresentable; see §2 + §3).
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

The existing `LensApplication` (file-glob) carrier stays as-is for whole-tree invariant gates. The new substrate is **`SectionedLensApplication`**, a refinement of `LensApplication` whose `applies_to` is a structural section reference rather than a path glob.

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

// ApplicationConfig is a sum, not a record-with-Optional. This makes
// "Enforce without budget" and "Introspect with budget" UNREPRESENTABLE
// at the type level — illegal states cannot exist (per
// feedback_state_space_vs_behavioral_invariants + modeling principles 2/6).
type ApplicationConfig
  = Enforce { budget: LensBudget, diagnostic_severity: DiagnosticSeverity }
  | Introspect

type DiagnosticSeverity = Error                  // C-8 fail-closed: only Error is admitted

type SectionedLensApplication {
  lens: DeclarationId           // structural reference to the lens declaration
  section: SectionRef
  config: ApplicationConfig
  span: SourceSpan              // user-authored site for diagnostic attribution
}
```

The `lens` field is a `DeclarationId` (not a string lens name). This grounds in the canonical `.dag`-lens form per [`docs/lens-library-design.md`](lens-library-design.md) §1.5.

The `LensBudget` shape is lens-instance-specific and lives in each lens's own substrate authority:

```dag
// src/v3/lenses/complexity.dag (or equivalent — already exists in some form)

type ComplexityBudget = AsymptoticClass     // O(1), O(log n), O(n), O(n log n), O(n²), ...

// src/v3/lenses/cost.dag

type CostBudget = SymbolicCost              // already declared; this is the existing cost lens output type

// src/v3/lenses/parallelism.dag

type ParallelismBudget = ParallelismMode    // Sequential | OptInIndependent
```

**Why `LensBudget` is not a single union here**: each lens's budget shape is part of its own authority. A central `LensBudget = ComplexityBudget | CostBudget | ParallelismBudget | ...` would create a roster (same failure class as `kernel_lens_set`, per [`docs/lens-library-design.md`](lens-library-design.md) §1.5). The `Enforce.budget` field is an *opaque* refinement keyed by the `lens: DeclarationId` — the lens declaration itself names the budget type, and the type-checker enforces compatibility at lens-application authoring time.

Implementation sketch for the keyed compatibility:

```dag
// in dsl/std/lens.dag (extending existing Lens<C> framework)

type Lens<C> {
  // existing fields: read fn, etc.
  budget_type: DeclarationId        // the budget carrier this lens accepts
}
```

The lens declaration names its `budget_type`. When the type-checker sees a `SectionedLensApplication` with `config = Enforce { budget: b, ... }`, it verifies `b inhabits lens.budget_type`. Any mismatch is a fail-closed diagnostic at authoring time. (No type-checking is needed for the `Introspect` variant — there is no budget to compatibility-check; introspection runs the lens and emits its value, no comparison.)

## §3. Violation policy under fail-closed discipline

The user reframe initially named the violation-policy axis as `CompileError | Warning | Silent`. Under [`../INVARIANTS.md`](../INVARIANTS.md) C-8 (fail-closed compilation) and `feedback_fail_closed_discipline`:

- **CompileError**: every detected violation is a Diagnostic at compile time. *Admitted.*
- **Warning**: violations are logged but compilation succeeds. *Forbidden* — C-8 prohibits warnings as a steady state; the only legitimate "non-blocking detection" is introspection (no enforcement at all).
- **Silent**: violations are silently dropped. *Forbidden* — same C-8 violation; "silent None" is named in `feedback_fail_closed_discipline` as exactly the pattern banned.

The fail-closed-compatible enumeration is therefore binary, not ternary — and the budget is bundled INTO the `Enforce` variant by construction (per §2 above), making "Enforce without budget" and "Introspect with budget" structurally unrepresentable:

```dag
type ApplicationConfig
  = Enforce { budget: LensBudget, diagnostic_severity: DiagnosticSeverity }   // produces a compile-time Diagnostic on violation
  | Introspect                                                                  // computes lens value; no budget; no diagnostic
```

The pairing of `budget` with `Enforce` (and its absence in `Introspect`) is a state-space invariant, not a behavioral one. The type-checker has no `Enforce + None` or `Introspect + Some(...)` combination to reject — those states cannot be constructed (per `feedback_state_space_vs_behavioral_invariants`).

`DiagnosticSeverity` is a single-variant nominal carrier (only `Error`). It exists as a refinement-extension point so future *non-fail-closed* surfaces (if ever added — none currently planned) can extend without changing the `Enforce` carrier shape. Per `feedback_no_annotations` and `feedback_no_metadata_markers`, the single-variant carrier is structurally honest: there is exactly one severity, named.

### §3.1 Why no "Warning" mode (load-bearing rationale)

C-8 is the canonical fail-closed compilation rule. A "warning" mode at lens application would:

1. **Allow violations as steady state**, normalizing the bridge-as-steady-state P5 pattern.
2. **Diverge from the rest of the compiler** (no other surface has warnings; introducing them at lens application creates parallel disposition authority).
3. **Drift toward "annotation as advice"**, which contradicts `feedback_no_annotations` (lens applications should be first-class structural facts, not advisory markers).

If a user wants to "see the lens value but not enforce", that is `Introspect` mode — the lens runs, produces a value, and the value is available for downstream lenses (or for human reading via debug surfaces). No diagnostic, no enforcement, no warning. This is a different operation than enforcement-with-a-warning; it has clean semantics.

### §3.2 Default policy for complexity contracts

Per Director ratification 2026-05-02 + user directive: the default policy for complexity contracts is **opt-out** (compiler enforces by default; explicit waiver required for exceptions). The substrate shape for the default + waiver:

**Default**: when a function declaration has no explicit `SectionedLensApplication` for the complexity lens, the compiler applies a default budget — currently `O(unbounded)` (no budget), meaning every function is introspected by default but not enforced. Once T-Lens-Behavioral-Parity ships behavioral complexity, the default flips to *enforced with implicit budget derived from the function's body* — i.e., the compiler infers the actual complexity class and rejects regressions.

**Waiver**: when a user wants to override the default (accept a function whose complexity exceeds the inferred / user-named budget), they author a structural waiver:

```dag
data my_function_complexity_waiver: ComplexityBudgetWaiver = {
  function: my_function          // DeclarationId reference
  budget: O_log_n                // the budget being waived
  justification: "Inputs always small (≤16 items per Domain.MaxItems); measured wall time stable across versions"
}
```

This is a `.dag` declaration, not an annotation (per `feedback_no_annotations`). The waiver is itself subject to the same lens infrastructure — `lens_unused_parameters` could (eventually) flag stale waivers, `lens_structural_duplicates` flags duplicate waivers, etc.

**Why a separate `ComplexityBudgetWaiver` carrier, not just `SectionedLensApplication { config: Introspect }`**: a waiver is *not* "the user wants introspection" — it is "the user accepts a known violation, with justification". The justification is load-bearing (per `feedback_reason_not_label` — encode the stable reason, not the volatile label). Conflating waivers with introspection would lose the justification.

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
data __apply_lens_my_search_function: SectionedLensApplication = {
  lens: complexity                                     // DeclarationId of the complexity lens
  section: DeclarationScope { declaration: my_search_function }
  config: Enforce { budget: O_log_n, diagnostic_severity: Error }
  span: <user-authored span>
}
```

**Compiler-side processing**: during the lens fold (existing infrastructure per [`docs/design-lens-framework.md`](design-lens-framework.md)), the compiler iterates over every `SectionedLensApplication` in the program. For each `Enforce` application, it runs the named lens against the named section, compares against the budget, and emits a Diagnostic if the lens value exceeds the budget.

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
2. **New**: walk every `SectionedLensApplication` declaration, apply the named lens to the named section *with the given budget*, emit a Diagnostic if `Enforce` mode and budget exceeded, OR record the lens value if `Introspect` mode.

The new step is structurally identical to the existing fold — same `Lens<C>` reader, same `Witness<C>` output type — with an additional budget comparison and Diagnostic emission. The implementation cost is O(1) lens-applications-per-program (typically tens to hundreds per project, not millions).

### §5.1 Default-policy default-application synthesis

For the *default* application of the complexity lens (per §3.2), the compiler synthesizes `SectionedLensApplication` records implicitly during the lens fold — one per function declaration, with `config: Enforce { budget: <inferred from body>, diagnostic_severity: Error }`. These synthesized records are not stored in the `.dag` source; they exist only during the fold pass.

**Why synthesized rather than authored**: writing a default complexity application for every function would inflate the source. The synthesis is structural (every Declaration of arrow-connective shape gets one) and can be turned off per-declaration via an explicit user-authored introspection-mode application:

```dag
apply_lens(complexity, my_function, Introspect)
```

This explicit application overrides the synthesized default for `my_function`. The "explicit overrides synthesized" rule resolves at the lens-fold layer (look up explicit applications first; synthesize only if absent).

## §6. Cross-program coordination

This lane is **cross-program** between Substrate Manager and Verification Manager (per [`docs/r3-structure.md`](r3-structure.md) lane 16):

- **Substrate Manager owns**: the `SectionedLensApplication` / `SectionRef` / `ApplicationConfig` carriers in `src/v3/std/lens_application.dag`; the compiler-side lens-fold integration; the per-lens `budget_type` declaration extension.
- **Verification Manager owns**: the closure-gate TestClaims (`complexity_violation_compile_error_demonstrated`, `crdt_cost_basis_demonstrated`, `memory_peak_cost_basis_demonstrated`, `opt_in_iteration_parallelism_via_lens_application_demonstrated`); cross-target equivalence on lens-application semantics (does Rust-emitted code respect the budget in the same way Python-emitted does?).

The split mirrors the existing T-CostLens-Composition split: substrate authors carriers + fold semantics, Verification asserts the demonstrations.

## §7. Cascade gates

Per [`docs/r3-structure.md`](r3-structure.md):

- **Internal cascade**: T-Lens-Behavioral-Parity must reach BEHAVIORALLY COMPLETE before T-Lens-Application-Surface dispatches. Reason: a `complexity_violation_compile_error_demonstrated` TestClaim requires the complexity lens to actually compute correct asymptotic classes (not just the depth proxy currently shipped). Likewise for cost / parallelism / effect_enumeration. Behavioral parity gives lens-application substantive semantics.
- **External cascade**: R2-Evaluator landed (per the standard R3 worker-dispatch precondition).

Pre-cascade *design-doc* work is permitted (this doc); pre-cascade *substrate work* (carriers landing, fold-integration code) waits for T-Lens-Behavioral-Parity COMPLETE.

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

### §8.3 Default-application budget inference — RESOLVED (conditionally): regression-detection class, gated on T-Lens-Behavioral-Parity COMPLETE

**Question:** For the synthesized default complexity application, what does "budget inferred from the body" mean exactly?

**Resolved:** budget = computed asymptotic class at the time of synthesis. Any subsequent change that *increases* the class fires a Diagnostic (regression detection). This is the "implicit baseline" semantics — the compiler records what the function's complexity *was* and rejects changes that worsen it without an explicit user authoring update (either an explicit budget or an explicit waiver).

**Cascade gate:** this resolution depends on T-Lens-Behavioral-Parity being COMPLETE for the complexity lens (asymptotic-class computation is reliable). Until that lane closes, the default complexity application is `Introspect`-mode only — the compiler records the lens value without enforcement, and surfaces it for inspection. When T-Lens-Behavioral-Parity COMPLETE lands, the default flips from `Introspect` to `Enforce { regression_baseline: <computed class> }`.

**Implementation note:** the cascade-flip is itself substrate work — a `LensApplication` field `regression_baseline_pinned: AsymptoticClass?` records the at-synthesis baseline; when present, regressions fire. When absent (pre-cascade), the application is introspection-only.

### §8.4 Waiver lifecycle — RESOLVED: separate-lens future scope, dissolution-trigger named

**Question:** `ComplexityBudgetWaiver` declarations need a path to dissolution — a stale waiver should be detected.

**Resolved:** stale-waiver detection is a future lens (`lens_stale_waivers`) tracked outside T-Lens-Application-Surface scope. **Dissolution trigger named explicitly per P5 scaffold-discipline**: when `lens_stale_waivers` ships, every `ComplexityBudgetWaiver` whose target's actual lens value no longer exceeds the budget is a stale-waiver finding. The user must either delete the waiver (if no longer needed) or update its `justification` field (if still load-bearing). The lens-fold pass enforces no semantics on stale waivers until that future lens lands; pre-`lens_stale_waivers`, waivers are honored unconditionally.

**Why split out:** `lens_stale_waivers` is a separate lens with its own substrate input (every waiver) and its own enforcement output (per-waiver staleness Diagnostic). Bundling it into T-Lens-Application-Surface scope would inflate this lane; the dissolution trigger is named so the future lens has clear input semantics.

**Tracking:** add `lens_stale_waivers` to [`docs/lens-library-design.md`](lens-library-design.md) §6 (future lenses) when this design doc lands.

### §8.5 Cross-section composition — RESOLVED: read declared budget, not computed class

**Question:** When function `f` is `apply_lens(complexity, f, O_log_n)`-ed and another function `g` calls `f`, does `g`'s complexity lens read `f`'s declared budget or its actual computed class?

**Resolved:** read the declared budget. The budget is the user's contract; reading the computed class would let `g`'s emission depend on `f`'s implementation rather than its contract.

**Why:** reading computed class would mean a refactor of `f` that lowers its complexity (e.g., O(n log n) → O(n)) would propagate into `g`'s lens result *automatically*. That makes `g`'s emission silently dependent on `f`'s implementation choices. The declared budget is the abstraction barrier — `g` reads "f's contract is O(log n)" and composes against that. If `f`'s implementation diverges from its declared budget, that is `f`'s own Diagnostic (the lens application on `f` fires), not `g`'s.

**Implementation note:** the lens-fold pass, when computing `g`'s complexity composition, looks up `f`'s `SectionedLensApplication` (if any) and reads `config.budget`. If `f` has no enforced application, the lens-fold reads the computed class as fallback (the absence of a contract means there is no abstraction barrier).

**Bridge-as-steady-state avoidance:** this resolution preserves the cost-of-change=1 principle (changing `f`'s implementation does not require re-checking every caller's lens result) and keeps abstraction-barrier semantics consistent with the rest of the substrate (a function's *signature* is what callers depend on; the body is private detail).

---

All five questions resolved. Implementation can proceed without further Director ratification on these specific points. Cascade gates (T-Lens-Behavioral-Parity COMPLETE for §8.3 enforcement-flip) and external dependencies (R2-Evaluator landed) remain as the only outstanding preconditions.

## §9. Relationship to existing authority

This design doc extends:

- [`docs/lens-library-design.md`](lens-library-design.md) §3 — the existing file-glob `LensApplication`. **No changes to existing carrier**; this doc adds `SectionedLensApplication` as a sibling carrier with structural section references.
- [`docs/v3-lens-capability-register.md`](v3-lens-capability-register.md) — the lens-capability register tracking PROXY/STUB/PARTIAL/COMPLETE status per lens. This design assumes T-Lens-Behavioral-Parity has driven all four target lenses to COMPLETE before T-Lens-Application-Surface implementation begins.
- [`docs/design-lens-framework.md`](design-lens-framework.md) — the `Lens<C>` framework. This design adds one fold-pass extension (lens-application discovery + budget comparison) but does not modify the underlying `Lens<C>` shape.
- [`../INVARIANTS.md`](../INVARIANTS.md) C-8 (fail-closed compilation) — load-bearing for §3 (no Warning, no Silent policies).
- [`../INVARIANTS.md`](../INVARIANTS.md) P2 (boundary discipline) — load-bearing for §8.2 (single-authority for `(lens, section)` pairs).
- [`../INVARIANTS.md`](../INVARIANTS.md) P5 (progress is dissolution) — load-bearing for §8.4 (waivers must have dissolution paths).

This document does NOT modify:

- The existing `Lens<C>` carrier shape (per T-Substrate-Lens-Primitive — that is R2 work already complete).
- The existing file-glob `LensApplication` carrier (per [`docs/lens-library-design.md`](lens-library-design.md) §3 — sibling, not replacement).
- Per-lens budget types (each lens owns its own `LensBudget` definition; this doc only specifies the dispatch carrier `ApplicationConfig`).

## §10. Implementation order (sketch)

Within T-Lens-Application-Surface lane (per [`docs/r3-structure.md`](r3-structure.md) closure gates):

1. **Substrate carriers landing** (`lens_application_carrier_landed`, `section_ref_substrate_landed`). Author `src/v3/std/lens_application.dag` per §2. Type-checker integration: `Lens<C>.budget_type` field added; `(lens, section)` single-authority enforcement at parse time.
2. **Fold-pass integration** (`application_config_violation_policy_routing`). Extend the lens-fold pass to walk `SectionedLensApplication` declarations + emit Diagnostics on `Enforce`-mode budget violations + record lens values on `Introspect`-mode. Synthesized default-application per §5.1.
3. **Worked example #1: complexity contract** (`complexity_violation_compile_error_demonstrated`). TestClaim per §4.1.
4. **Worked example #2: CRDT cost basis** (`crdt_cost_basis_demonstrated`). TestClaim per §4.2.
5. **Worked example #3: memory-peak cost basis** (`memory_peak_cost_basis_demonstrated`). TestClaim per §4.3.
6. **Worked example #4: opt-in parallelism** (`opt_in_iteration_parallelism_via_lens_application_demonstrated`). TestClaim per §4.4.

Steps 1-2 are sequential (carriers must exist before fold-pass consumes them). Steps 3-6 are parallel-dispatchable (each is an independent worked example referencing the same fold-pass infrastructure).

Total estimate (per L-XL sizing in the lane row): substrate carriers + fold-pass = M-L; 4 worked examples = M each in parallel = L overall. End-to-end: 4-6 weeks worker time at standard R3 cadence.

---

**This document is a design spec, not a ship target.** It resolves the structural design questions blocking T-Lens-Application-Surface lane dispatch. The lane itself runs once cascade gates clear (T-Lens-Behavioral-Parity COMPLETE + R2-Evaluator landed). All §8 design questions resolved in-doc; no Director ratification required before substrate authoring begins.
