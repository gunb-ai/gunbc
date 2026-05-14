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

For any program scope (function body / region / module), the tightness lens produces a discriminated result — the variant tag + named improvement witness IS the proof relationship (no two-adjacent-class-fields shape):

```
data TightnessAnalysis
  = AlreadyTight {
      actual: AsymptoticClass,        // = tight by construction; no derivation needed
      section: SectionRef,
    }
  | Loose {
      improvement: AsymptoticStrictDominance,   // named improvement witness: dominator strictly dominates dominated (see §1.5)
      first_transformation: ClassTierTightnessTransformation,        // ≥1 enforced structurally (no empty/disconnected derivation); class-tier-only by type
      additional_transformations: List<ClassTierTightnessTransformation>,  // optional ordered tail; class-tier-only by type
      section: SectionRef,            // function/region the analysis applies to (matches EnforcedApplication.section per src/v3/std/lens_application.dag:178 — SectionRef is the type; DeclarationScope is a variant)
    }
```

The discrimination + improvement witness encodes the Loose-vs-AlreadyTight precondition into the type:
- `AlreadyTight` cannot carry a transformation (no field exists) — no spurious derivation.
- `Loose` cannot exist without ≥1 transformation (`first_transformation` is non-optional) — no empty derivation.
- `Loose` cannot exist without a named `AsymptoticStrictDominance` improvement witness — the relation `actual > tight ∧ actual ≠ tight` is a named-carrier obligation, not two adjacent fields that admit `(ClassLinear, ClassLinear)` or `(ClassLinear, ClassQuadratic)` inversions.

The four illegal states `actual == tight ∧ transformations non-empty`, `actual > tight ∧ transformations empty`, `actual == tight ∧ Loose tag`, and `actual < tight ∧ Loose tag` are all structurally impossible.

### §1.2 Transformation vocabulary

The §1.5 substrate models the patterns as two type-level-distinct coproducts (`ClassTierTightnessTransformation` + `SymbolicTierTightnessTransformation` — see §1.5 declarations). The table below enumerates the shared documentation-level vocabulary; the types are split so each lens's `Loose.first_transformation` field structurally cannot carry a wrong-tier variant:

Per-transformation tier classification (class-tier vs symbolic-tier-only) — class-tier transformations can produce an `AsymptoticStrictDominance` improvement at the lattice level; symbolic-tier-only transformations tighten the symbolic cost expression but keep the same class:

| Transformation | Pattern recognized | Tightening | Tier |
|---|---|---|---|
| `LoopHoisting` | Computation inside loop independent of loop variable | O(n*m) → O(n+m) when inner cost is non-constant | **class-tier** (e.g., n,m both ClassLinear: ClassPolynomial(2) → ClassLinear) |
| `DeadCodeElimination` | Subgraph with no consumer (compute result never read) | removes the subgraph's cost contribution entirely | **class-tier** (when dead subgraph dominates the class) |
| `ConstantBoundPropagation` | Inner-loop bound provably independent of outer-loop variable | O(n*m) → O(n) when m proved constant | **class-tier** (ClassPolynomial(2) → ClassLinear) |
| `LoopFusion` | Sequential loops with compatible iteration spaces over same data | O(n+m) → O(max(n,m)) when iteration spaces identical | **symbolic-tier only** (`n+m` and `max(n,m)` are same class; ClassLinear → ClassLinear in BoundedLattice<AsymptoticClass>) |
| `AggregationRecognition` | Explicit accumulator with associative-reduce shape | substrate-folded to declarative `sum`/`fold` op | **symbolic-tier only** (pattern recognition; folding to declarative form does not change the lattice class) |
| `MapFilterFoldFusion` | Chained collection ops sharing iteration space | O(n)+O(n)+O(n) → O(n) single-pass | **symbolic-tier only** (all chain elements + fused result are same class; ClassLinear → ClassLinear in BoundedLattice<AsymptoticClass>) |

The vocabulary is shared across the lens family. This lens (class-level `lens_complexity_tight` producing `TightnessAnalysis` per §1.5) emits `Loose` only when an `AsymptoticStrictDominance` improvement is derivable — restricting to the 3 class-tier transformations above. The 3 symbolic-tier-only transformations are deferred to a future symbolic-cost-tightness sibling lens (per §2 out-of-scope carve), which produces `Loose` for same-class symbolic tightening that the class-level lens correctly reports as `AlreadyTight`.

**Per openai-pro BLOCKING PR #3067 #11790 2026-05-14**: original 6-row table without tier classification admitted the design tension where `Loose.first_transformation` could carry a symbolic-tier-only variant. **Per codex BLOCKING PR #3067 #11795 2026-05-14**: original tier-by-implementation-discipline was insufficient — must be type-level enforced. Fix: §1.5 substrate splits `TightnessTransformation` into `ClassTierTightnessTransformation` (3 arms: LoopHoisting / DeadCodeElimination / ConstantBoundPropagation) and `SymbolicTierTightnessTransformation` (3 arms: LoopFusion / AggregationRecognition / MapFilterFoldFusion). The class-level `Loose.first_transformation` field is typed `ClassTierTightnessTransformation` — symbolic-tier variants are structurally non-instantiable at the type level (Practice 2 / modeling-discipline). The vocabulary in this table is shared at the documentation tier across the lens family; the types are split at the substrate tier.

### §1.3 Diagnostic

When the lens produces a `Loose` variant (variant tag + `improvement: AsymptoticStrictDominance` witness IS the precondition check — no runtime `actual > tight` comparison; structurally enforced via the discriminated TightnessAnalysis at §1.1):

```
TightnessViolation: code as written is {loose.improvement.dominator} but
structurally-derivable tight bound is {loose.improvement.dominated}.
Applicable transformations:
  [{loose.first_transformation}, ...loose.additional_transformations].
  --> {span_at_the_loose_region}
```

`AlreadyTight` variants emit no diagnostic — structurally cannot enter the violation path.

- **Severity**: `Error` (always-on for compiler-internal; per-`EnforcedTightness` declared for user programs)
- **Layer-1 kind label**: `TightnessViolation` (new diagnostic class)
- **Span**: points at the loose region of code (the function or sub-expression where the transformation would apply)

### §1.4 Enforcement tiers (operator-ratified 2026-05-14)

**Compiler-internal code** (`src/v3/*`, `dsl/std/*`): **always-on**. Every compiler-authored function ratchet-checks tightness as part of build. Any tightness violation is a build-break. This is the SELF_HOSTING.md "compiler is canonical example" framing made operational — the compiler's own code is the most-aggressively-checked codebase in the project.

**User programs**: opt-in via `EnforcedTightness` data declaration (CONCRETE non-generic carrier; Output type-locked to `TightnessAnalysis` per codex BLOCKING PR #3067 2026-05-14 — see §1.5 for the carrier shape + example use-site at §1.5 instantiation block). Backwards-compatible with existing programs; users opt their functions in as they're ready.

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
// Per openai-pro BLOCKING PR #3067 2026-05-14: bare `List<NodeId>` for
// role-specific node-pairs was too loose (admitted wrong-arity/role
// instantiations); proof obligations stated in comments were not type-enforced.
// Fix: role-specific named NodeId fields (NOT bare List<NodeId>) + typed
// proof-witness carriers (one per variant, NOT a parallel coproduct).

// Per codex BLOCKING PR #3067 #11795 2026-05-14: original single
// `TightnessTransformation` coproduct admitted `Loose.first_transformation`
// pairing with the 3 symbolic-tier-only variants (LoopFusion /
// AggregationRecognition / MapFilterFoldFusion). The §1.2 tier classification
// described the constraint, but it was only Practice-6 API enforcement
// (lens-implementation construction discipline), not Practice-2 structural
// enforcement. Fix: split the coproduct into TWO TYPE-LEVEL DISTINCT
// coproducts so the class-level lens cannot accept a symbolic-tier variant
// at the type level. The class-level Loose carrier references
// `ClassTierTightnessTransformation` (3 arms); the future symbolic-cost-
// tightness sibling lens (per §2 out-of-scope carve) will reference
// `SymbolicTierTightnessTransformation` (3 arms). No common parent type —
// vocabulary is shared at the §1.2 documentation level, not the type-system
// level, because the two lenses' Loose variants are structurally different.

type ClassTierTightnessTransformation       // produces AsymptoticStrictDominance in BoundedLattice<AsymptoticClass>
  = LoopHoisting {
      enclosing_loop_node: NodeId             // role: outer loop containing the invariant subgraph
      invariant_subgraph_node: NodeId         // role: subgraph proved loop-invariant
      independent_size_variables: List<SizeVariable>  // size-vars proved independent of loop var
      invariance_witness: LoopInvarianceWitness  // structural witness: subgraph reads no loop-var; shape TBD post-Gap-11
    }
  | DeadCodeElimination {
      dead_subgraph_node: NodeId              // role: subgraph with no downstream Port consumer
      no_consumer_witness: NoConsumerWitness  // structural witness: Port consumption walk confirms zero consumers; shape TBD post-Gap-11
    }
  | ConstantBoundPropagation {
      outer_loop_node: NodeId                 // role: outer loop over variable size
      inner_loop_node: NodeId                 // role: inner loop with constant-bound
      inner_bound: SymbolicCost               // proved variable-independent of outer-loop SizeVariable
      bound_independence_witness: ConstantBoundWitness  // structural witness: inner_bound has no SizeVariable dependency on outer; shape TBD post-Gap-11
    }

type SymbolicTierTightnessTransformation    // produces same-class symbolic-cost tightening; future symbolic-cost-tightness sibling lens carrier
  = LoopFusion {
      // Loop fusion = SIBLING/SEQUENTIAL loops with compatible iteration spaces
      // (NOT outer/inner nested-loop relationship — that's ConstantBoundPropagation's
      // shape). Per openai-pro BLOCKING PR #3067 2026-05-14: outer/inner naming
      // misled toward nested-loop semantics; corrected to sequential first/second.
      first_loop_node: NodeId                 // role: first sequential loop in fusion sequence
      second_loop_node: NodeId                // role: second sequential loop (compatible iteration space)
      space_a: SymbolicCost                   // first-loop iteration-space cost expression
      space_b: SymbolicCost                   // second-loop iteration-space cost expression
      equivalence_witness: IterationSpaceEquivalenceWitness  // structural witness: space_a ≡ space_b + sequential-not-nested + no inter-loop dependency-order blocker; shape TBD post-Gap-11
    }
  | AggregationRecognition {
      accumulator_subgraph_node: NodeId       // role: subgraph implementing the accumulator pattern
      associative_op_node: NodeId             // role: +/min/max operation node at reduce-point
      associativity_witness: AssociativeReduceWitness  // structural witness: op is associative per algebra.dag laws; shape TBD post-Gap-11
    }
  | MapFilterFoldFusion {
      // Pipeline chain has minimum cardinality 2 (single map/filter/fold doesn't
      // fuse). Per openai-pro BLOCKING PR #3067 2026-05-14: bare `List<NodeId>`
      // admitted 0/1 nodes + non-pipeline nodes + duplicates + wrong ordering
      // via prose-only comment. Fix: structural ≥2 enforcement via first +
      // second + rest decomposition (rest is empty for exactly-2 chains).
      first_pipeline_node: NodeId             // role: first map/filter/fold node in chain (ordered position 1)
      second_pipeline_node: NodeId            // role: second map/filter/fold node (ordered position 2)
      additional_pipeline_nodes: List<NodeId> // role: optional ordered tail (positions 3, 4, ...) — empty for exactly-2 chains
      shared_iteration_cost: SymbolicCost     // common iteration-space cost across chain elements
      shared_space_witness: SharedIterationSpaceWitness  // structural witness: chain elements share iteration space + are all map/filter/fold operation nodes + ordering preserved; shape TBD post-Gap-11
    }

// 🟡 SCAFFOLD per-variant proof-witness types — concrete shapes finalize
// post-Gap-11 SymbolicCost / ProductCost / SumCost composition + lens-tier
// implementation surface. Each witness type encodes ONE specific structural
// proof obligation; NOT a generic Proof<Evidence> coproduct (avoids the
// parallel-evidence-admits-invalid-pairings class codex BLOCKING #11751
// flagged). Witness construction is lens-side; consumers receive the witness
// as a structurally-valid proof receipt, not a "compiler-said-so" promise.
//
// Concrete shapes ratified per Substrate Mgr canvas during PB-X-tightness-lens
// implementation worker dispatch (post-Gap-11). Current SCAFFOLD shape:
//
//   type IterationSpaceEquivalenceWitness { /* TBD per Gap-11 SymbolicCost equivalence-decidability algorithm */ }
//   type LoopInvarianceWitness            { /* TBD per Port read-set analysis output */ }
//   type NoConsumerWitness                { /* TBD per Port consumption-walk algorithm output */ }
//   type ConstantBoundWitness             { /* TBD per SymbolicCost variable-independence analysis */ }
//   type AssociativeReduceWitness         { /* TBD per algebra.dag associativity-decidability surface */ }
//   type SharedIterationSpaceWitness      { /* TBD per chain-fusion algorithm output */ }
//
// All 6 carriers are 🟡 SCAFFOLD; SCAFFOLD-→-TERMINAL trigger = Gap 11 lands +
// lens-implementation worker dispatches resolve concrete witness fields.

// Type-enforced pairing: each variant arm carries the EXACT role-named fields
// + per-variant proof-witness type applicable to that transformation.
// LoopFusion cannot pair with NoConsumer evidence; ConstantBoundPropagation
// cannot pair with AssociativeReduce evidence. No parallel TransformationEvidence
// coproduct + no bare List<NodeId> admitting wrong arities.

// 🟡 SCAFFOLD per Gap 11 SymbolicCost composition trigger — named witness
// carrier for "asymptotic strict dominance" (the relation
// `asymptotic_dominates(dominator, dominated) ∧ dominator ≠ dominated`).
//
// Grounds against existing substrate:
//   - AsymptoticClass inhabits BoundedLattice<AsymptoticClass> per
//     src/v3/std/algebra.dag:418 — partial order is defined by the lattice.
//   - asymptotic_dominates(a, b) at src/v3/std/algebra.dag:428 — implements
//     the lattice's `a ≥ b` relation (reflexive). Strict dominance = `≥ ∧ ≠`.
//
// Per openai-pro BLOCKING PR #3067 #11790 2026-05-14: previous Loose carrier
// had `actual: AsymptoticClass` + `tight: AsymptoticClass` as TWO ADJACENT
// FIELDS with no structural proof of strict dominance. Admitted illegal pairs:
//   - `Loose { actual: ClassLinear, tight: ClassLinear, ... }` (equal — not strict)
//   - `Loose { actual: ClassLinear, tight: ClassQuadratic, ... }` (inverted — not dominance)
//   - `Loose { actual: ClassUnknown, tight: ClassConstant, ... }` (incomparable
//     in some readings — relies on lens construction discipline alone)
// Fix: name the relation as a carrier. Role-named fields `dominator` (= former
// `actual`) and `dominated` (= former `tight`) make the ordering explicit.
//
// SCAFFOLD content: current shape has named fields ordered by role but no
// type-level proof of the dominance relation between them (no AsymptoticClass-
// pair-witnesses tier in std yet). Construction discipline = lens-side
// (Practice 6 API enforcement): `lens_complexity_tight` constructs
// `AsymptoticStrictDominance` only when `asymptotic_dominates(dominator,
// dominated) ∧ dominator ≠ dominated` is structurally verified against
// SymbolicCost composition. Consumers receive the named witness as a
// proof-relation receipt (NOT a "compiler-said-so" promise), same discipline
// as the 6 transformation-evidence witnesses below.
//
// SCAFFOLD → TERMINAL trigger: Gap 11 SymbolicCost composition + lattice-
// strict-ordering proof shape ratified per Substrate Mgr canvas. Concrete
// post-Gap-11 shape: likely carries a `SymbolicCostDifferenceWitness` or
// equivalent lattice-strict-ordering proof carrier per the chosen Gap 11
// composition algebra.
type AsymptoticStrictDominance {
  dominator: AsymptoticClass             // role: strictly larger class (= former `actual`)
  dominated: AsymptoticClass             // role: strictly smaller class (= former `tight`)
  // Future SCAFFOLD-→-TERMINAL field (post-Gap-11): strict_dominance_proof: SymbolicCostStrictDominanceWitness
}

// 🟢 TERMINAL at the tightness-analysis scope. Discriminated result —
// variant tag + named improvement witness IS the derivation predicate
// (AlreadyTight ≡ actual == tight; Loose ≡ asymptotic_dominates(actual, tight)
// ∧ actual ≠ tight, expressed structurally via AsymptoticStrictDominance).
//
// Per openai-pro BLOCKING PR #3067 #11790 2026-05-14: previous shape with
// adjacent `actual`/`tight` fields admitted four illegal states (equal pair,
// inverted pair, equal-pair with Loose tag, inverted-pair with Loose tag).
// Fix: replace adjacent class fields with named AsymptoticStrictDominance
// improvement witness above. Plus prior fix (openai-pro #11789): discriminate
// the result + structural ≥1 transformation enforcement.
type TightnessAnalysis
  = AlreadyTight {
      actual: AsymptoticClass             // = tight by construction; no separate tight field
      section: SectionRef                 // per src/v3/std/lens_application.dag:66-68 + 178 — matches EnforcedApplication.section
    }
  | Loose {
      improvement: AsymptoticStrictDominance  // named witness: dominator strictly dominates dominated (see above)
      first_transformation: ClassTierTightnessTransformation         // ≥1 enforced — no empty derivation possible; class-tier-only by type (codex BLOCKING #11795)
      additional_transformations: List<ClassTierTightnessTransformation>  // ordered tail; empty for single-transformation derivations; class-tier-only by type
      section: SectionRef                 // per src/v3/std/lens_application.dag:66-68 + 178
    }

// Self-comparison carrier — STRUCTURALLY DISTINCT from EnforcedApplication
// (which is user-budget comparison). Tightness is self-comparison: lens
// produces discriminated TightnessAnalysis (AlreadyTight carries actual only;
// Loose carries actual + tight + ≥1 transformation derivation); enforcement
// dispatches on the variant tag. No user-declared budget field.
//
// Per codex BLOCKING #11751 PR #3067 2026-05-14: previous 3-param mirror of
// EnforcedApplication<Output, Budget, Projected> created structural mismatch
// — Budget type param was unused (no budget field) → admits invalid states.
// Director ratification msg_d45523da had locked-in 3-param mirror as compromise;
// codex correctly flagged that the compromise is incomplete. Resolved here:
// distinct carrier shape for self-comparison; lens carrier is Lens<Output> (NOT
// EnforceableLens which is budget-shaped).
// Concrete (NON-generic) carrier — Output type-locked to `TightnessAnalysis`.
// Per codex BLOCKING PR #3067 2026-05-14: previous `EnforcedTightness<Output>`
// generic over any `Lens<Output>` was too permissive — admitted invalid
// instantiations like `EnforcedTightness<ComplexitySummary>` (where
// ComplexitySummary lacks the AlreadyTight|Loose discrimination the
// enforcement logic dispatches on). Prose-only "Output MUST be
// TightnessAnalysis-like" contract is NOT type-enforcement; concretization
// locks the contract at the type level.
//
// Tightness-style enforcement for OTHER lens output domains (e.g., future
// timing-tightness, memory-tightness) creates its own concrete carrier
// (e.g., `EnforcedTimingTightness { lens: Lens<TimingTightnessAnalysis>, ... }`),
// NOT a generic over `Lens<Output>`. This per-domain carrier discipline matches
// how `EnforcedApplication<Output, Budget, Projected>` family members handle
// per-domain budget enforcement at the type level rather than via prose contract.
type EnforcedTightness {
  lens: Lens<TightnessAnalysis>       // type-locked to TightnessAnalysis (discriminated AlreadyTight | Loose per §1.1 / §1.5)
  section: SectionRef                 // per src/v3/std/lens_application.dag:66-68 (SectionRef type, DeclarationScope is a variant)
  diagnostic_severity: DiagnosticSeverity  // src/v3/std/lens_application.dag:84 (single-variant Error per feedback_fail_closed_discipline + INVARIANTS C-8); NOT dsl/std/behavioral.dag::Severity (4-variant unrelated to lens discipline)
  span: SourceSpan
  //
  // Enforcement logic (lens-internal): the lens produces a discriminated
  // TightnessAnalysis (AlreadyTight | Loose). On AlreadyTight: no-op. On Loose:
  // emit TightnessViolation diagnostic at span citing
  // `Loose.improvement.dominator` + `Loose.improvement.dominated` +
  // `Loose.first_transformation` + `Loose.additional_transformations`. No
  // runtime `actual > tight` comparison needed at enforcement-time — the
  // variant tag + AsymptoticStrictDominance witness IS the precondition check
  // (per openai-pro BLOCKING PR #3067 #11789 + #11790 2026-05-14 + §1.1
  // discriminated shape).
  //
  // Semantic distinction from EnforcedApplication<Output, Budget, Projected>:
  //   - EnforcedApplication carries a USER-DECLARED `budget: Budget` field; user
  //     authority on the constraint; lens checks observed Projected ≤ budget.
  //   - EnforcedTightness has NO budget field; COMPILER is authority on what the
  //     constraint should be; lens internally derives both sides of the comparison.
  //   - Same axes (constraint + observed), different authority source (user vs compiler).
}
```

Plus a new lens-instance declaration grounded in the live 6-field `Lens<C>` substrate at `src/v3/std/lens.dag:70` (name + read + sequential + branch + iterate + validate). Mirrors the `timing_lens` declaration pattern at `src/v3/std/timing_lens.dag:423-430` — function-body fields are 🟡 SCAFFOLD per implementation-tier dispatch (post-Gap-11), but the carrier-shape inhabitance is structural at design-tier:

```
// 🟡 SCAFFOLD function-body fields per implementation-tier dispatch (post-Gap-11).
// Carrier inhabits Lens<TightnessAnalysis> structurally at design-tier.
data tightness_lens_sequential: Monoid<TightnessAnalysis> = {
  op: tightness_sequential_op           // TBD: (TightnessAnalysis, TightnessAnalysis) -> TightnessAnalysis
                                        //   Law: AlreadyTight ⊕ AlreadyTight = AlreadyTight;
                                        //   either-side Loose absorbs (joining improvement.dominator/dominated
                                        //   via BoundedLattice<AsymptoticClass> ≥; concatenating
                                        //   transformation lists in order; reuses join_asymptotic_class
                                        //   at src/v3/std/algebra.dag:514).
  identity: AlreadyTight {              // identity for monoid; empty subgraph is tight at the smallest class
    actual: ClassConstant
    section: <empty-section-ref>        // TBD per substrate definition
  }
}

// Inhabits the live 6-field Lens<C> substrate at src/v3/std/lens.dag:70.
data lens_complexity_tight: Lens<TightnessAnalysis> = {
  name: "complexity_tightness"
  read: tightness_lens_read              // TBD: fn(Dag, Behavior) -> Witness<TightnessAnalysis>
                                          //   Per-behavior classification — produces AlreadyTight or Loose
                                          //   depending on whether a TightnessTransformation derives a
                                          //   strictly-dominated class via Gap 11 SymbolicCost composition.
  sequential: tightness_lens_sequential  // monoid above — sequential composition law
  branch: tightness_branch_op            // TBD: fn(TightnessAnalysis, TightnessAnalysis) -> TightnessAnalysis
                                          //   Branch combination — max-dominator across branches; transformations
                                          //   union under the maximum branch (BoundedLattice<AsymptoticClass> join).
  iterate: tightness_iterate             // TBD: fn(TightnessAnalysis, LoopBound) -> TightnessAnalysis
                                          //   Loop amplification — multiply symbolic cost by LoopBound;
                                          //   reclassify; lift improvement.dominator/dominated under composition.
  validate: tightness_lens_validate      // TBD: fn(Dag, TightnessAnalysis) -> OptionalDiagnostic
                                          //   Emit TightnessViolation when result is Loose (per §1.3 diagnostic);
                                          //   Optional.None when AlreadyTight (structurally no-op).
}
```

Applying via `fold_lens<TightnessAnalysis>(lens_complexity_tight, dag)` (framework function at `src/v3/std/lens.dag:6`) produces a `DimensionReport<TightnessAnalysis>`. EnforcedTightness enforcement (§1.5 above) dispatches on the contained variant — AlreadyTight is no-op; Loose triggers `validate` → TightnessViolation diagnostic.

Example use-site declaration (1-param self-comparison shape; lens is `Lens<TightnessAnalysis>` data-instance grounded above, not `EnforceableLens`):

```
data witness_tightness: EnforcedTightness = {
  lens: lens_complexity_tight              // produces discriminated TightnessAnalysis (AlreadyTight | Loose); Loose carries actual + tight + ≥1 transformation
  section: DeclarationScope { declaration: my_function }
  diagnostic_severity: Error
  span: { file: "...", start: ..., end: ... }
}
```

The `DeclarationScope { declaration: my_function }` is a VALUE of type `SectionRef` (per the variant at lens_application.dag:67) — uses the variant constructor at value-position; the field type at type-position is `SectionRef`.

## §2. Out of scope

Cross-algorithm optimality (algorithm synthesis — e.g., bubble sort → merge sort, naive matmul → Strassen) is **NOT** in scope. That requires algorithm synthesis or pattern-recognition + transformation library at semantic-equivalence-tier; major research-tier feature beyond lens-tier scope.

Tightness lens is **same-algorithm-only**: it reasons about the program AS WRITTEN and applies semantics-preserving transformations to derive the tight bound. It doesn't propose alternate algorithms.

**Symbolic-tier-only tightening is also out of scope for THIS lens** (deferred to a future sibling lens at the symbolic-cost-tightness tier): the 3 symbolic-tier-only transformations per §1.2 tier classification — `LoopFusion`, `AggregationRecognition`, `MapFilterFoldFusion` — tighten the symbolic cost expression but stay in the same AsymptoticClass arm of the BoundedLattice<AsymptoticClass> at `src/v3/std/algebra.dag:418`. The class-level lens correctly reports `AlreadyTight` for those cases by construction. A symbolic-cost-tightness sibling lens (separate `data` instance + separate `TightnessAnalysis`-like carrier parameterized over SymbolicCost rather than AsymptoticClass) will carry those cases; pre-Gap-11 substrate scope is intentional.

## §3. Prerequisites

1. **§1.8 gate #79 `complexity_lens_behaviorally_complete`** — currently SATISFIED-BY-CONSTRUCTION via temporary Rust cementing receipt; full behavioral completion requires `ComplexitySummary` TestClaim literals + ProductCost/SumCost composition (Gap 11).
2. **Close-plan Gap 11 LogCost / ProductCost / SumCost composition** — without this, `actual_class` collapses to ClassUnknown for composite expressions; `tight_class` would inherit the same limitation. Tightness lens consumes Gap 11's composition algebra.
3. **`ClassTierTightnessTransformation` + `SymbolicTierTightnessTransformation` substrate** — two new `.dag` coproducts per §1.5 (split landed per codex BLOCKING PR #3067 #11795); needs ratification per `feedback_grep_carrier_semantic_before_ratification` 4-axis audit. Sibling lens (symbolic-cost-tightness) consumes the `SymbolicTier...` carrier; this lens (class-level) consumes only the `ClassTier...` carrier.
4. **Lens dispatch infrastructure** — `lens_complexity_tight` is declared as a `data` instance of `Lens<TightnessAnalysis>` (6-field carrier per `src/v3/std/lens.dag:70` — see §1.5 declaration) and registers in `lens_capability_register`; framework application via `fold_lens<TightnessAnalysis>` at `src/v3/std/lens.dag:6`. Consumer wiring follows the established lens-fold pipeline (no parallel custom dispatcher).

## §4. Close criterion (substrate-debt-shaped predicate)

Per `feedback_no_textual_enforcement_bridges` — close criterion is a substrate-fact-at-HEAD predicate, not narrative:

```
# Predicate at gate close — one fixture per class-tier TightnessTransformation
# arm per §1.2 (LoopHoisting, DeadCodeElimination, ConstantBoundPropagation;
# symbolic-tier-only arms LoopFusion/AggregationRecognition/MapFilterFoldFusion
# are carved to a future sibling lens and have no class-level fixture):
cargo test --release -p v3-compiler --test integration complexity_tightness_compile_error_demonstrated
# returns: PASS with at least one fixture for EACH class-tier transformation,
# each fixture demonstrating:
#   - lens produces TightnessAnalysis::Loose variant (per §1.1 discriminated shape)
#   - Loose.improvement = AsymptoticStrictDominance { dominator: <looser_class>, dominated: <tight_class>, ... }
#   - Loose.first_transformation = <the-specific-class-tier-arm> { ... }
#   - Error/TightnessViolation diagnostic produced at fixture span citing
#     improvement.dominator/dominated + transformations
#
# Required fixture set:
#   - fixture demonstrating LoopHoisting: e.g., O(n*m) loop with m-cost subgraph
#     proven loop-invariant; lens reports Loose { improvement: { dominator:
#     ClassPolynomial(2), dominated: ClassLinear }, first_transformation:
#     LoopHoisting { ... } }
#   - fixture demonstrating DeadCodeElimination: e.g., quadratic subgraph
#     producing a Port no downstream node reads; lens reports Loose {
#     improvement: { dominator: ClassPolynomial(2), dominated: ClassLinear },
#     first_transformation: DeadCodeElimination { ... } }
#   - fixture demonstrating ConstantBoundPropagation: e.g., nested loop with
#     inner bound provably constant; lens reports Loose { improvement: {
#     dominator: ClassPolynomial(2), dominated: ClassLinear },
#     first_transformation: ConstantBoundPropagation { ... } }
#
# Carved out from class-level fixture requirement (deferred to future
# symbolic-cost-tightness sibling lens per §2):
#   - LoopFusion: no class-level Loose constructible — both operands
#     same-class by lattice (algebra.dag:418)
#   - AggregationRecognition: no class-level dominance from pattern
#     recognition alone (no lattice arm change)
#   - MapFilterFoldFusion: no class-level Loose constructible — chained ops
#     and fused single-pass are same-class
```

Plus compiler-internal-code build invariant:

```
# Every compiler-authored function (.dag or Rust) ratchet-checked tightness-clean at HEAD:
cargo test --release -p v3-compiler --test integration compiler_internal_code_tightness_clean
# returns: PASS — every src/v3/* and dsl/std/* function produces TightnessAnalysis::AlreadyTight
# (no Loose variants permitted in compiler-internal code)
```

## §5. PM-recommended gate-row shape

Two options for §1.8 placement:

**Option A — Sub-promise under existing gate #79**:
- Gate #79 `complexity_lens_behaviorally_complete` expanded scope to include tightness lens behavior
- Sub-promise wording: "complexity lens produces both `ComplexitySummary` (actual class) AND `TightnessAnalysis` (discriminated `AlreadyTight | Loose` per §1.1 — Loose carries named `AsymptoticStrictDominance` improvement witness + ≥1-enforced transformation derivation); EnforcedApplication + EnforcedTightness both fail-closed on violations (Loose variant emits TightnessViolation; AlreadyTight is no-op)"
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
