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
  section: SectionRef,                // function/region the analysis applies to (matches EnforcedApplication.section per src/v3/std/lens_application.dag:178 — SectionRef is the type; DeclarationScope is a variant)
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

**User programs**: opt-in via `EnforcedTightness<TightnessAnalysis>` data declaration (1-param self-comparison carrier; structurally distinct from EnforcedApplication's 3-param user-budget shape per codex BLOCKING #11751 PR #3067 resolution 2026-05-14 — see §1.5 for the carrier shape + example use-site at §1.5 instantiation block). Backwards-compatible with existing programs; users opt their functions in as they're ready.

### §1.5 Substrate carriers

New `.dag` declarations needed (grounded against existing lens-application substrate at `src/v3/std/lens_application.dag:176-182`; `EnforcedTightness` is intentionally a structurally distinct 1-param self-comparison carrier, not a mirror of `EnforcedApplication`'s 3-param user-budget shape):

```
// In src/v3/std/complexity_tightness.dag (or analogous):

// 🟡 SCAFFOLD until Gap 11 LogCost/ProductCost/SumCost composition lands.
//
// Coproduct classification per `feedback_coproduct_dissolution` 4-pattern audit
// (addresses openai-pro BLOCKING PR #3067 2026-05-14 — modeling-discipline
// requirement that new N≥2 coproducts carry classification + dissolution-attempt
// record + named SCAFFOLD trigger):
//
// **Pattern**: STRUCTURE — variants are different structural shapes of the same
// role ("semantics-preserving transformations the lens recognizes in the DAG to
// derive a tighter bound"). Each variant identifies a distinct structural
// pattern (sequential-loop / loop-invariant-subgraph / dead-subgraph /
// constant-bound-inner-loop / associative-reduce / chained-collection-ops).
//
// **4 dissolution attempts walked-and-rejected before settling on this coproduct**:
//
// Attempt 1 — single `Transformation` type with `String` label: REJECTED per
//   INVARIANTS.md P1 (Modeling Faithfulness). A string label is not structural;
//   downstream consumers cannot programmatically verify which transformation
//   applies. Same class as `feedback_opaque_strings_attract_heuristics`.
//
// Attempt 2 — Refinement-class hierarchy (`Transformation` refines into named
//   subtypes): REJECTED — transformations don't have a refinement relation.
//   LoopFusion is not a refinement of LoopHoisting (they're parallel structural
//   patterns over disjoint DAG shapes, not subtype-shaped).
//
// Attempt 3 — Algebra (sum of primitive operations on DAG): REJECTED —
//   transformations aren't algebraically composable in a meaningful sense.
//   LoopFusion + LoopHoisting is not a sum-shaped algebraic operation;
//   the variants are parallel-applicable choices over distinct structural
//   patterns, not summands of a primitive-operation algebra.
//
// Attempt 4 — Parametric `Refinement<Evidence>`: REJECTED — per-variant
//   evidence payload differs structurally (LoopFusion needs
//   IterationSpaceEquivalence with 2 spaces; LoopHoisting needs
//   LoopInvariance with variable-independence facts; DeadCodeElimination
//   needs NoConsumer with port-consumption facts; etc.). Cannot be a uniform
//   parametric refinement; the evidence shape IS the variant discriminator.
//
// **Named SCAFFOLD-→-TERMINAL trigger**: Gap 11 LogCost/ProductCost/SumCost
// composition lands AND per-variant evidence-payload fields finalize against
// the composition algebra. At that point: revisit + upgrade to 🟢 TERMINAL.
// If during finalization the structural pattern reveals additional variants
// (e.g., AssociativeCommutativeFold as a sibling of AggregationRecognition,
// or LoopSwap as a sibling of LoopFusion), they enter as new arms per the
// same Structure-pattern discrimination — coproduct stays open to new
// structural-pattern variants discovered post-Gap-11.
//
// Per-variant evidence-payload INLINED into each variant arm (NOT a parallel
// TransformationEvidence coproduct, per codex BLOCKING #11751 PR #3067 2026-05-14:
// parallel coproducts admit invalid pairings — e.g., LoopFusion arm could pair
// with NoConsumer evidence). Inlining makes the pairing type-enforced.
//
// All variant payloads cite LIVE substrate types per `feedback_corrections_must_grep_verify_source`
// (codex BLOCKING #11751 PR #3067 2026-05-14):
//   - `SymbolicCost` per `src/v3/std/algebra.dag:190` (NOT `SymbolicCostExpr` — non-live)
//   - `NodeId` per `src/v3/std/substrate.dag:5` (NOT `NodeRef` — non-live)
//   - `SizeVariable` per existing T-CostLens substrate (cited in close-plan §1.8 row #80)
type TightnessTransformation
  = LoopFusion {
      affected_nodes: List<NodeId>            // outer_loop + inner_loop NodeId pair
      space_a: SymbolicCost                   // outer-loop iteration-space cost expression
      space_b: SymbolicCost                   // inner-loop iteration-space cost expression
      // Lens proves space_a + space_b are equivalent before constructing this variant
    }
  | LoopHoisting {
      affected_nodes: List<NodeId>            // enclosing-loop + invariant-subgraph NodeId pair
      independent_size_variables: List<SizeVariable>  // size-vars proved independent of loop var
    }
  | DeadCodeElimination {
      affected_nodes: List<NodeId>            // unconsumed-subgraph NodeIds
      // Lens proves zero downstream Port reads subgraph output (port_consumption walk)
    }
  | ConstantBoundPropagation {
      affected_nodes: List<NodeId>            // outer_loop + inner_loop NodeId pair
      inner_bound: SymbolicCost               // proved variable-independent of outer-loop SizeVariable
    }
  | AggregationRecognition {
      affected_nodes: List<NodeId>            // accumulator-pattern NodeIds
      associative_op_node: NodeId             // +/min/max operation node at reduce-point
    }
  | MapFilterFoldFusion {
      affected_nodes: List<NodeId>            // chain pipeline NodeIds (≥2)
      shared_iteration_cost: SymbolicCost     // common iteration-space cost across chain elements
    }

// Type-enforced pairing: each variant arm carries the EXACT evidence shape
// applicable to that transformation. LoopFusion cannot pair with NoConsumer
// evidence; ConstantBoundPropagation cannot pair with AssociativeReduce evidence.
// No parallel TransformationEvidence coproduct needed.

// 🟢 TERMINAL at the tightness-analysis scope. Bundles lens output (actual + tight
// asymptotic classes), the bridging transformation list with evidence proofs,
// and the section the analysis applies to.
type TightnessAnalysis = {
  actual: AsymptoticClass
  tight: AsymptoticClass
  transformations: List<TightnessTransformation>
  section: SectionRef
  // Note: section: SectionRef (NOT DeclarationScope which is a variant of SectionRef)
  // per src/v3/std/lens_application.dag:66-68 + 178 — matches EnforcedApplication.section
}

// Self-comparison carrier — STRUCTURALLY DISTINCT from EnforcedApplication
// (which is user-budget comparison). Tightness is self-comparison: lens
// produces TightnessAnalysis carrying both projection axes (actual + tight);
// enforcement compares them internally. No user-declared budget field.
//
// Per codex BLOCKING #11751 PR #3067 2026-05-14: previous 3-param mirror of
// EnforcedApplication<Output, Budget, Projected> created structural mismatch
// — Budget type param was unused (no budget field) → admits invalid states.
// Director ratification msg_d45523da had locked-in 3-param mirror as compromise;
// codex correctly flagged that the compromise is incomplete. Resolved here:
// distinct carrier shape for self-comparison; lens carrier is Lens<Output> (NOT
// EnforceableLens which is budget-shaped).
type EnforcedTightness<Output> {
  lens: Lens<Output>                  // generic lens; NOT EnforceableLens (which is user-budget shape)
  section: SectionRef                 // per src/v3/std/lens_application.dag:66-68 (SectionRef type, DeclarationScope is a variant)
  diagnostic_severity: DiagnosticSeverity  // src/v3/std/lens_application.dag:84 (single-variant Error per feedback_fail_closed_discipline + INVARIANTS C-8); NOT dsl/std/behavioral.dag::Severity (4-variant unrelated to lens discipline)
  span: SourceSpan
  //
  // Output type contract: Output MUST be a TightnessAnalysis-like carrier with
  // two compiler-derived projection axes (actual + tight) over a common comparison
  // type (e.g., AsymptoticClass for complexity). Comparison is internal to the
  // lens's enforcement logic, NOT user-declared.
  //
  // For complexity-tightness instantiation: Output = TightnessAnalysis
  //   - TightnessAnalysis.actual: AsymptoticClass (computed from program structure)
  //   - TightnessAnalysis.tight: AsymptoticClass (compiler-derived tighter bound)
  //   - Comparison: asymptotic_dominates(actual, tight) — if True AND actual ≠ tight,
  //     emit TightnessViolation diagnostic at span.
  //
  // Semantic distinction from EnforcedApplication<Output, Budget, Projected>:
  //   - EnforcedApplication carries a USER-DECLARED `budget: Budget` field; user
  //     authority on the constraint; lens checks observed Projected ≤ budget.
  //   - EnforcedTightness has NO budget field; COMPILER is authority on what the
  //     constraint should be; lens internally derives both sides of the comparison.
  //   - Same axes (constraint + observed), different authority source (user vs compiler).
}
```

Plus a new lens declaration:

```
lens lens_complexity_tight: (Dag) -> TightnessAnalysis
```

Example use-site declaration (1-param self-comparison shape; lens is `Lens<TightnessAnalysis>` not `EnforceableLens`):

```
data witness_tightness: EnforcedTightness<TightnessAnalysis> = {
  lens: lens_complexity_tight              // produces TightnessAnalysis with actual + tight projections
  section: DeclarationScope { declaration: my_function }
  diagnostic_severity: Error
  span: { file: "...", start: ..., end: ... }
}
```

The `DeclarationScope { declaration: my_function }` is a VALUE of type `SectionRef` (per the variant at lens_application.dag:67) — uses the variant constructor at value-position; the field type at type-position is `SectionRef`.

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
