---
status: PM-authored design substrate (deep-wolf-155)
authority_parent: Operator briansrls 2026-05-14 ratification — "compiler-derived-optimal enforcement, not user-budget enforcement" + IN-R3 scope-expansion ratified via AskUserQuestion
director_ratification: PENDING (gate-row shape: sub-promise under #79 OR new §1.8 row)
authoring_date: 2026-05-14
prereq_gates: §1.8 #79 complexity_lens_behaviorally_complete + close-plan Gap 11 LogCost/ProductCost/SumCost composition
---

# Design — Complexity Tightness Lens

## §0. Operator framing

Operator 2026-05-14 distinguished two complexity-checking shapes:

1. **User-budget enforcement (current)**: User declares budget intent ("≤ ClassLinear"); compiler observes actual; errors if actual exceeds declared budget. Current `EnforcedApplication<ComplexitySummary, AsymptoticClass, AsymptoticClass>` shape.
2. **Compiler-derived-optimal enforcement (wanted)**: User writes code with no budget (or any budget); compiler proves a STRUCTURALLY-EQUIVALENT TIGHTER bound is derivable via semantics-preserving transformations; errors if actual is loose against the derivable tight bound.

Operator quote (paraphrased): "what i wanted was — something that was written as classquadratic — that the compiler can infer should actually be classlinear — and we error — not like 'we want this code to be classlinear and its classquadratic.'"

This doc specifies the compiler-derived-optimal shape as a NEW lens-tier feature: **Structural Tightness Lens**.

## §1. Feature spec

### §1.1 Lens output

For any program scope (function body / region / module), the tightness lens produces:

```
data TightnessAnalysis = {
  actual: AsymptoticClass,            // complexity as-written (current lens output post-Gap-11)
  tight: AsymptoticClass,             // complexity achievable via semantics-preserving transformations
  transformations: List<TightnessTransformation>,  // named transformations bridging actual → tight
  scope: DeclarationScope,            // function/region the analysis applies to
}
```

### §1.2 Transformation vocabulary

`TightnessTransformation` enumerates the patterns the lens RECOGNIZES (without mutating code — purely deriving a "would-be-tighter-if-applied" judgment):

| Transformation | Pattern recognized | Tightening |
|---|---|---|
| `LoopFusion` | Sequential loops with compatible iteration spaces over same data | O(n+m) → O(max(n,m)) when iteration spaces identical |
| `LoopHoisting` | Computation inside loop independent of loop variable | strips inner-cost factor from outer-loop product |
| `DeadCodeElimination` | Subgraph with no consumer (compute result never read) | removes the subgraph's cost contribution entirely |
| `ConstantBoundPropagation` | Inner-loop bound provably independent of outer-loop variable | O(n*m) → O(n) when m proved constant |
| `AggregationRecognition` | Explicit accumulator with associative-reduce shape | substrate-folded to declarative `sum`/`fold` op |
| `MapFilterFoldFusion` | Chained collection ops sharing iteration space | O(n)+O(n)+O(n) → O(n) single-pass |

### §1.3 Diagnostic

When `asymptotic_dominates(actual, tight)` is True AND actual ≠ tight:

```
TightnessViolation: code as written is {actual_class} but structurally-derivable
tight bound is {tight_class}. Applicable transformations: [{transformation_list}].
  --> {span_at_the_loose_region}
```

- **Severity**: `Error` (always-on for compiler-internal; per-`EnforcedTightness` declared for user programs)
- **Layer-1 kind label**: `TightnessViolation` (new diagnostic class)
- **Span**: points at the loose region of code (the function or sub-expression where the transformation would apply)

### §1.4 Enforcement tiers (operator-ratified 2026-05-14)

**Compiler-internal code** (`src/v3/*`, `dsl/std/*`): **always-on**. Every compiler-authored function ratchet-checks tightness as part of build. Any tightness violation is a build-break. This is the SELF_HOSTING.md "compiler is canonical example" framing made operational — the compiler's own code is the most-aggressively-checked codebase in the project.

**User programs**: opt-in via `EnforcedTightness<ComplexitySummary>` data declaration (analogous to current `EnforcedApplication`). Backwards-compatible with existing programs; users opt their functions in as they're ready.

### §1.5 Substrate carriers

New `.dag` declarations needed:

```
// In src/v3/std/complexity_tightness.dag (or analogous):

data TightnessTransformation =
  | LoopFusion
  | LoopHoisting
  | DeadCodeElimination
  | ConstantBoundPropagation
  | AggregationRecognition
  | MapFilterFoldFusion

data TightnessAnalysis = {
  actual: AsymptoticClass
  tight: AsymptoticClass
  transformations: List<TightnessTransformation>
  scope: DeclarationScope
}

data EnforcedTightness<L> = {
  lens: L                              // the tightness lens (lens_complexity_tight)
  section: DeclarationScope            // the function or region
  diagnostic_severity: Severity        // Error / Warning per program
  span: SourceSpan                     // the EnforcedTightness application site
}
```

Plus a new lens declaration:

```
lens lens_complexity_tight: (Dag) -> TightnessAnalysis
```

## §2. Out of scope

Cross-algorithm optimality (algorithm synthesis — e.g., bubble sort → merge sort, naive matmul → Strassen) is **NOT** in scope. That requires algorithm synthesis or pattern-recognition + transformation library at semantic-equivalence-tier; major research-tier feature beyond lens-tier scope.

Tightness lens is **same-algorithm-only**: it reasons about the program AS WRITTEN and applies semantics-preserving transformations to derive the tight bound. It doesn't propose alternate algorithms.

## §3. Prerequisites

1. **§1.8 gate #79 `complexity_lens_behaviorally_complete`** — currently SATISFIED-BY-CONSTRUCTION via temporary Rust cementing receipt; full behavioral completion requires `ComplexitySummary` TestClaim literals + ProductCost/SumCost composition (Gap 11).
2. **Close-plan Gap 11 LogCost / ProductCost / SumCost composition** — without this, `actual_class` collapses to ClassUnknown for composite expressions; `tight_class` would inherit the same limitation. Tightness lens consumes Gap 11's composition algebra.
3. **`TightnessTransformation` substrate** — new `.dag` carriers per §1.5; needs ratification per `feedback_grep_carrier_semantic_before_ratification` 4-axis audit.
4. **Lens dispatch infrastructure** — `lens_complexity_tight` registers as a new lens in `lens_capability_register`; consumer wiring in `analyze_complexity_tight_dimension` (or analogous Rust generated function until ported to `.dag`).

## §4. Close criterion (substrate-debt-shaped predicate)

Per `feedback_no_textual_enforcement_bridges` — close criterion is a substrate-fact-at-HEAD predicate, not narrative:

```
# Predicate at gate close:
cargo test --release -p v3-compiler --test integration complexity_tightness_compile_error_demonstrated
# returns: PASS with at least 1 fixture demonstrating:
#   - code structurally classified as ClassQuadratic
#   - tight_class proved as ClassLinear via ConstantBoundPropagation transformation
#   - Error/TightnessViolation diagnostic produced at fixture span
```

Plus compiler-internal-code build invariant:

```
# Every compiler-authored function (.dag or Rust) ratchet-checked tightness-clean at HEAD:
cargo test --release -p v3-compiler --test integration compiler_internal_code_tightness_clean
# returns: PASS — no tightness violation in src/v3/* or dsl/std/*
```

## §5. PM-recommended gate-row shape

Two options for §1.8 placement:

**Option A — Sub-promise under existing gate #79**:
- Gate #79 `complexity_lens_behaviorally_complete` expanded scope to include tightness lens behavior
- Sub-promise wording: "complexity lens produces both `ComplexitySummary` (actual class) AND `TightnessAnalysis` (tight class + transformations); EnforcedApplication + EnforcedTightness both fail-closed on violations"
- Pro: doesn't expand §1.8 row count; folds into existing lens-behavioral-parity work
- Con: gate #79 already has substantial scope (symbolic CostExpr + work/span split + asymptotic classification + cementing receipt); adding tightness may blur the gate's success criterion

**Option B — New §1.8 row** (e.g., #79b or #92b):
- New row: `complexity_tightness_lens_enforces_structural_tightness`
- Clean separation: gate #79 = forward-direction lens behavior; new gate = compiler-derived-optimal enforcement
- Pro: clean scope discrimination; each gate has a single success criterion
- Con: expands §1.8 row count by 1 (currently 106 → 107)

PM lean: **Option B** for clean scope discrimination + per `feedback_grep_section_18_for_sibling_failure_ledger_homes_before_scope_broadening` discipline (separate failure modes deserve separate ledger rows).

## §6. Sequencing dependency in dispatch plan

If ratified IN-R3:
- **Upstream**: Gap 11 (LogCost/ProductCost/SumCost composition) must land before tightness lens can be implemented; otherwise `actual_class` is ClassUnknown collapse
- **Parallel**: substrate carrier ratification (§1.5) can author in parallel with Gap 11 work
- **Downstream**: compiler-internal-code tightness-clean ratchet becomes a build-time gate post-implementation; user-program opt-in via `EnforcedTightness` lands as separate worker dispatch

This affects close-plan Phase 2 corrective sweep. If folded as sub-promise under #79, no new Phase needed. If new §1.8 row, Phase 2.2 (§1.8 PB-X gate-row insertions) absorbs the new row authoring; Phase 2.3 (Track A taxonomy reclassification) gets a new (b)-class category for tightness-lens-related entries.

## §7. Adjacent — what this is NOT

- **NOT a refactoring/rewrite recommender**: the lens REPORTS transformations but doesn't apply them. User refactors manually (or future-feature applies them automatically).
- **NOT a profiler / runtime-measurement tool**: this is static analysis on the DAG, not measurement of actual runtime.
- **NOT a complexity contract for inputs**: this is internal-implementation tightness, not input-shape contracts.
- **NOT algorithm synthesis**: see §2; cross-algorithm optimality is explicitly out of scope.

## §8. Authority chain

- Operator framing 2026-05-14: closure audit questionnaire question on complexity lens behavior
- Operator ratification 2026-05-14: IN-R3 scope-expansion via AskUserQuestion ("NEW R3 close gap (sub-promise under gate #79 or new gate #79b)")
- Operator ratification 2026-05-14: enforcement-tier shape via AskUserQuestion ("Always-on for the compiler's own code; opt-in for user programs")
- PM design substrate authoring (this doc)
- Director-tier ratification (PENDING) on gate-row shape + close-plan foldering
