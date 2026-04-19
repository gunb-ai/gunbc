> Part of: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) (Lane 2 Stage 2b) | Companion: [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (PR #529) | Unblocks: Lane 2 Stage 2b implementation (workflow idempotency lens); informs Lane 2 Stages 2d / 2e / 2f (downstream composition over workflow structure)

# Design DB-18 — `WorkflowEffect` substrate carrier (Stage 2b input structure)

**Design blocker:** DB-18 (new substrate carrier `WorkflowEffect` — a four-variant coproduct describing workflow control-flow shape for effect-algebra composition)
**Consumers:** Lane 2 Stage 2b workflow idempotency lens (initial consumer; `LinearEffect`-only scope). Downstream: Stage 2d symbolic cost, Stage 2e parallelism-as-lens, Stage 2f user-declared dimensions.
**Status:** **R2 (Part 2) is shipped in code** — `BoolPortRef`, `Dag::bool_port_of`, `BranchArm::new`, `LinearEffect.ops: List<OperationEffect>`, reflected `lane2_workflow` on `ValueNode`/`BindNode`, substrate accessor `lane2_workflow_at`, and `Dag::lane2_workflow_effect_at(&self, root: &NodeId)`. This document describes that **live** shape (see `src/v3/std/effects.dag`, `src/v3/std/substrate.dag`, `src/v3/compiler/src/dag.rs`). **Part 3** (user `data … WorkflowEffect` authoring surface + full lowering) remains future work. Historical R1 names (`BranchPredicateRef`, `branch_arm_of`, `NonEmptyList` for linear ops) are **not** the current substrate.
**Companion:** [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (PR #529, landed 2026-04-18 as `8c7e7acdd`) reshapes the *output* side of the effect algebra: `ComposedEffect` is removed; `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` is the post-reshape algebra. DB-18 adds the *input* side: a typed carrier describing the workflow's control-flow structure above `List<OperationEffect>`. The two live on orthogonal axes — `WorkflowEffect` is the input structure the lens walks; `CompositionVerdict` is the output the algebra returns. They coexist; neither replaces the other.

---

## Summary

Stage 2b's pre-DB-18 design (`lane2-compile-time-proofs.md` §Stage 2b) assumed a workflow is a linear `List<OperationEffect>` — `compose_effects` walks the list, composes effect shapes, emits a verdict. That shape is correct for the narrow case (GCP Secret Manager upsert → STS Exchange → IAM grant, all linear) but the escalation clause in the same section already names the extension: *"if workflow structure isn't representable cleanly — e.g., control flow in a pipeline doesn't map to a linear `List<OperationEffect>` — surface. Don't stretch `compose_effects` to handle branches silently; the algebra needs to reflect branch-wise composition, which is a legitimate design extension."*

DB-18 is that legitimate design extension. The substrate gains **one new coproduct** (`WorkflowEffect`, **🟢 TERMINAL** in `effects.dag` alongside Stage 2b/2e consumers), **one helper record** (`BranchArm`), and **one typed opaque handle** (`BoolPortRef`, same pattern as `ParamRef` / `TransformRef`). The authority site for `WorkflowEffect` values is an optional `lane2_workflow: Option<Box<WorkflowEffect>>` field on both the `Value` and `Bind` computation-substrate behaviors — one workflow per root node, keyed by the root's `NodeId`. Accessors on `Dag`: `try_register_lane2_workflow_effect(root, workflow) -> bool` (writer; installs on either `Value` or `Bind`) and `lane2_workflow_effect_at(&root: &NodeId) -> Option<&WorkflowEffect>` (reader; pattern-matches both variants). The five-`Behavior` commitment and the effect algebra (`compose_effects`, `CompositionVerdict`) are untouched. Stage 2b's `analyze_workflow` matches on `LinearEffect` only and routes the other three variants through `WorkflowIdempotencyReport::IdempotencyUnsupported`. The fail-closed contract from INVARIANTS §C-8 applies at **`Dag::bool_port_of` / `Dag::bool_port_for_branch_condition_or_diagnose`**: a non-Bool branch condition must surface as `Diagnostic::BranchConditionNotBool` — never silently absorb.

`WorkflowEffect` is the input-structure carrier. `CompositionVerdict` (PR #529) is the output-verdict carrier. They meet at exactly one edge: Stage 2b's lens projects `LinearEffect { ops }` through `compose_effects(ops)` to obtain a `CompositionVerdict`; other `WorkflowEffect` variants produce diagnostics without invoking the algebra.

---

## Constraints (non-negotiable)

1. **`CompositionVerdict` authority preserved.** Post-PR #529, `compose_effects(List<OperationEffect>) -> CompositionVerdict` is the sole effect-algebra output. DB-18 does not introduce a parallel verdict carrier. `LinearEffect`'s lens delegates to `compose_effects`; other variants do not compose at Stage 2b.
2. **Five-behavior computation substrate untouched.** `Behavior = Value | Transform | Branch | Loop | Bind` remains at five variants. `WorkflowEffect` lives in the *type substrate* (declarations), not the computation substrate. Parallel lens tests (`test_thesis_five_behavior_variants`-style) stay green by construction.
3. **Bounded kernel invariant.** `WorkflowEffect` is recursive through its own variant fields; the recursion terminates at `LinearEffect.ops: List<OperationEffect>` (no `WorkflowEffect` children inside the list — only `OperationEffect`). **R2:** the empty list is the monoidal identity at the workflow-input carrier; `compose_effects` maps `[]` to `IdempotentComposition`. Finite by construction because the substrate is a DAG (no cycles in `WorkflowEffect`-typed edges).
4. **Illegal states unrepresentable at the type level.** Cardinality invariants use Track 9 primitives where required (`NonSingletonList` for multi-arm shapes). `LinearEffect.ops` intentionally uses **`List`** so `[]` is representable; branching/parallel arms still use `NonSingletonList` where ≥2 is required. No `Bool + Option<T>` pairs that admit contradictions.
5. **Fail-closed at diagnostic boundaries (INVARIANTS §C-8).** Every branch that does not lead to a `CompositionVerdict` produces a `Diagnostic` explaining which variant was encountered, which downstream stage consumes it, and — if the user can reshape the workflow into a `LinearEffect` form — how. No `Option<Verdict>`, no `Result<Verdict, ()>`, no silent skip.
6. **Stage 2a follow-up authority preserved.** The `CompositionVerdict`/`BreakingOperation`/`IdempotentShape`/`BreakingShape` partitioning landed in PR #529 is consumed by DB-18's `LinearEffect`-path without modification. DB-18 does not reshape `EffectShape`, does not re-materialize `ComposedEffect`, does not pair `CompositionVerdict` with a sibling list field (which would reintroduce the correlated-fields incoherence PR #529 dissolved).

---

## Design

### Substrate changes

**Add the `WorkflowEffect` coproduct** (`src/v3/std/effects.dag`):

```dag
// 🟢 TERMINAL. Workflow control-flow structure at the effect level.
// The compiler already distinguishes "do these ops in sequence" from
// "do one of these arms depending on a condition" from "repeat this
// body" at the computation-substrate level (Bind chains / Branch /
// Loop behaviors). This carrier hoists that distinction to the effect
// algebra so compose_effects and its downstream peers (symbolic cost,
// parallelism, user dimensions) can dispatch on shape rather than
// reconstruct it by walking the DAG.
//
// Four variants cover the four categorical operations on effect
// sequences: monoidal composition (LinearEffect, ∘), coproduct
// (BranchEffect, ∨), fixpoint (LoopEffect, μ), product/concurrent
// (ParallelEffect, ⊗). The Q4 dissolution receipt below argues each
// variant traces to a distinct algebraic operation and the coproduct
// is structurally irreducible.
type WorkflowEffect
  = LinearEffect { ops: List<OperationEffect> }
  | BranchEffect { arms: NonSingletonList<BranchArm> }
  | LoopEffect { body: WorkflowEffect }
  | ParallelEffect { branches: NonSingletonList<WorkflowEffect> }

// A single arm of a BranchEffect. `condition` is a Bool-typed port
// witness (`BoolPortRef`). Build with `BranchArm::new` after
// `Dag::bool_port_of` succeeds on the condition port.
type BranchArm {
  condition: BoolPortRef
  body: WorkflowEffect
}
```

**`BoolPortRef` — typed opaque handle** in `src/v3/compiler/src/dag.rs`, following the `ParamRef` / `TransformRef` Track 9 pattern from DB-9 R2.1:

```rust
// src/v3/compiler/src/dag.rs — R2 shipped shape.
pub struct BoolPortRef {
    port: PortId,
}

impl BoolPortRef {
    pub fn port_id(self) -> PortId { self.port }
}
```

**R2 construction path (two steps).** The Bool witness and the arm are separate: **`Dag::bool_port_of(&self, port: PortId) -> Option<BoolPortRef>`** is the sole Bool-validating constructor for a port. **`BranchArm::new(condition: BoolPortRef, body: WorkflowEffect)`** builds the arm once the witness exists. There is no combined `branch_arm_of` on `Dag` in R2.

```rust
impl Dag {
    pub fn bool_port_of(&self, port: PortId) -> Option<BoolPortRef> { /* … */ }

    pub fn bool_port_for_branch_condition_or_diagnose(
        &mut self,
        port: PortId,
        span: SourceSpan,
    ) -> Option<BoolPortRef> { /* emits BranchConditionNotBool on failure */ }
}

impl BranchArm {
    pub fn new(condition: BoolPortRef, body: WorkflowEffect) -> Self { /* … */ }
}
```

**Fail-closed boundary placement.** `bool_port_of` returns `None` when the port is not Bool-typed; call sites that have a source span use **`bool_port_for_branch_condition_or_diagnose`** to attach **`Diagnostic::BranchConditionNotBool`**. The typed-handle primitives themselves do not allocate diagnostics except via that dedicated path.

**Reflection (R2).** `WorkflowEffect` lives in `std.effects`; `ValueNode` / `BindNode` carry optional `lane2_workflow` in `substrate.dag`, and `lane2_workflow_at` is a substrate accessor realized per target (`rust.dag`, etc.). `.dag` lenses (e.g. `src/v3/lenses/idempotency.dag`) read workflow facts through that surface; the Rust analyzer remains the bootstrap oracle until emitted lens Rust is fully linked.

### Dissolution receipt — `WorkflowEffect` coproduct (four-pattern check)

Per `feedback_coproduct_dissolution`, every new coproduct must pass the four-pattern check before being stamped `🟢 TERMINAL`. Stamped here, not deferred. The full argument is required under the user's directive ("Audit Q4 receipt required").

**Pattern 1 — Fact placement (multiple consumers, different DAG locations).** All four variants attach to the same `WorkflowEffect`-typed slot at the same DAG location — a workflow declaration's root. The variant discriminates KIND OF WORKFLOW SHAPE, not WHERE THE WORKFLOW FACT LIVES. No fact-placement compression to dissolve. ✓ does not apply.

**Pattern 2 — Variant-is-data (same shape, different label).** The four variants' payloads, audited pairwise:

| Variant A | Variant B | Payload A | Payload B | Distinct? |
|---|---|---|---|---|
| `LinearEffect` | `BranchEffect` | `List<OperationEffect>` | `NonSingletonList<BranchArm>` | ✓ different element types + different cardinality contract |
| `LinearEffect` | `LoopEffect` | `List<OperationEffect>` | `WorkflowEffect` | ✓ list vs single |
| `LinearEffect` | `ParallelEffect` | `List<OperationEffect>` | `NonSingletonList<WorkflowEffect>` | ✓ different element types + different cardinality contract |
| `BranchEffect` | `LoopEffect` | `NonSingletonList<BranchArm>` | `WorkflowEffect` | ✓ list vs single |
| `BranchEffect` | `ParallelEffect` | `NonSingletonList<BranchArm>` | `NonSingletonList<WorkflowEffect>` | ✓ `BranchArm` ≠ `WorkflowEffect` (BranchArm carries `condition: BoolPortRef` that ParallelEffect branches do not; the typed-witness field is unrepresentable in ParallelEffect's element type) |
| `LoopEffect` | `ParallelEffect` | `WorkflowEffect` | `NonSingletonList<WorkflowEffect>` | ✓ single vs list |

The critical pair is BranchEffect vs ParallelEffect — both use `NonSingletonList<...>` at the outer shape and recurse on `WorkflowEffect` at the element level. The distinction is carried by `BranchArm { condition: BoolPortRef, body: WorkflowEffect }` versus a plain `WorkflowEffect`: a branch arm is an arm-gated-by-a-Bool-typed-port, a parallel branch is an unconditioned concurrent workflow. `condition: BoolPortRef` is the load-bearing structural fact, carried by a **typed opaque handle**. **`Dag::bool_port_of`** returns `None` for non-Bool ports; **`BranchArm::new`** packages a validated witness with a body. A raw-literal `BranchArm { condition: <arbitrary PortId>, body: ... }` is not type-checkable; only `BranchArm { condition: <BoolPortRef>, body: ... }` is. Pattern 2's dissolution criterion ("same shape, different label") does not apply because the `BranchArm` wrapper carries a structurally required *typed* field `ParallelEffect`'s branches do not. ✓ does not apply.

**Pattern 3 — Algebraic-form (traces to intro/elim of algebraic structures).** The four variants trace to four distinct categorical operations on effect algebras:

- `LinearEffect` = **monoidal composition (∘)**. `compose_effects` witnesses this: a linear sequence of effects composes as `∘` with `ReadEffect` (identity on state) as the unit on the effect-shape side. The `CompositionVerdict` is the algebra's output for this case. **R2:** the workflow-input carrier is `List<OperationEffect>` — the empty list is the monoidal identity (`compose_effects` on `[]` yields `IdempotentComposition`). That is distinct from **`lane2_workflow: None`** (no workflow fact on the node at all).
- `BranchEffect` = **coproduct (∨) / lattice meet over arms**. Exactly one arm executes at runtime; the verdict must hold for whichever arm is taken. At compile time this is a `∨`-over-arms: the workflow is idempotent iff every arm's workflow is idempotent. Structurally distinct from `∘` — composition order does not matter for arms that are alternatives, and the per-arm verdicts combine via lattice meet, not via monoidal multiplication.
- `LoopEffect` = **fixpoint (μ) / iteration**. The body is re-applied 0..N times (bound structure tracked separately, potentially in Part 2). Idempotency under iteration demands the body itself be idempotent (`f ∘ f = f`) — a strictly stronger condition than linear composition, because monoidal composition of two different non-idempotent effects can still converge, but iteration of one non-idempotent effect cannot.
- `ParallelEffect` = **concurrent product (⊗) / commutative composition**. Branches execute concurrently and require algebraic commutativity on the target state to compose safely. The verdict's validity depends on commutativity evidence (open question §1 below — Part 2 or Stage 2e may extend the carrier with a `commutativity` witness). Distinct from both `∘` (order-preserving) and `∨` (alternatives): `⊗` is unordered AND concurrent.

Each variant is the introduction form of a different categorical operation. There is no single algebraic operation the four dissolve into; mapping the set of four to a two-dimensional record (e.g., `{ordered, concurrent}`) does not recover the distinct verdict rules each variant demands. ✓ does not apply.

**Pattern 4 — Dimensional (flat enum hides M-dimensional record).** The proposed dimension would be `{control_flow: Linear | Branch | Loop | Parallel, ...}` — but this is just the coproduct tag renamed, with no second coordinate. Attempting a second coordinate produces a partial record where each variant has different required fields (LinearEffect needs `ops`; BranchEffect needs `arms` with conditions; LoopEffect needs `body`; ParallelEffect needs `branches`): these are different types of payload, not orthogonal coordinates on a shared record. A record shape with every payload field present-or-absent would admit illegal combinations (`control_flow: LinearEffect` with `arms: Some(...)`), which violates the Q1 cardinality-invariant check. ✓ does not apply.

**Conclusion:** `WorkflowEffect`'s 4-variant coproduct is structurally irreducible. The coproduct is the correct shape. Stamp: `🟢 TERMINAL` on `WorkflowEffect`, `🟢 TERMINAL` on `BranchArm`.

### Substrate principle audit (Q1–Q6, beyond Q4)

Per `feedback_substrate_principle_audit`, all six questions walk before greenlighting. Q4 is covered above; the other five are stamped here so downstream review verifies the audit ran rather than re-deriving each finding.

**Q1 — Cardinality invariants.** Does any variant admit `[]` where invariant says ≥1, or singletons where ≥2?

- `LinearEffect.ops: List<OperationEffect>` — **R2** allows `[]`: empty linear workflow is the monoidal identity at the workflow carrier (`IdempotentComposition` via `compose_effects`). **`lane2_workflow: None`** remains the sole representation of "no workflow on this root"; it is not the same fact as `Some(LinearEffect { ops: [] })`.
- `BranchEffect.arms: NonSingletonList<BranchArm>` — a branch with 0 or 1 arms is meaningless (0: no workflow; 1: just the arm itself, use the arm's `body` directly). Type rejects both.
- `LoopEffect.body: WorkflowEffect` — singleton (one body), never empty. Type is non-optional.
- `ParallelEffect.branches: NonSingletonList<WorkflowEffect>` — 0 branches is empty parallelism; 1 branch is just the branch itself, no concurrency. Type rejects both. ✓ cardinality invariants at the type level (including the correctly-permissive empty `LinearEffect` case); no "lowering guarantees" prose required.

**Q2 — Index/handle types.** Does a raw `Int` or `NodeId` encode something with a domain restriction?

- `BranchArm.condition: BoolPortRef` is a typed opaque handle (Track 9-style primitive). **`Dag::bool_port_of(port)`** is the sole Bool-validating port constructor; there is no unsafe escape hatch. A raw `PortId` does not typecheck in `BranchArm.condition`; the field type is `BoolPortRef`, not `PortId`. Matching the `ParamRef` / `TransformRef` pattern from DB-9 R2.1 — the handle carries both "is a valid port reference" AND "is Bool-typed," with the relation folded into the type rather than carried by constructor discipline. ✓ no raw `Int`/`NodeId`/`PortId` with comment-level or constructor-level validity; the invariant is carried by the type itself.

**Q3 — Duplicated fact.** Does Field A duplicate what's derivable from Field B?

- `WorkflowEffect` variants do not duplicate any fact carried by `OperationEffect`, `EffectShape`, or `CompositionVerdict`. `WorkflowEffect` is the *input structure* above ops; the ops themselves are authored once and referenced by `LinearEffect.ops`. `CompositionVerdict` is the *output* of the algebra — DB-18 does not pre-compute a verdict on the workflow shape. ✓ no duplication.

**Q5 — Construction authority.** Are multiple call sites independently constructing the same fact?

- **Authority site for `WorkflowEffect` values.** `ValueNode.lane2_workflow: Option<Box<WorkflowEffect>>` on the computation-substrate Value node at the workflow root. Writes go through `Dag::try_register_lane2_workflow_effect(root, workflow)`; reads go through `Dag::lane2_workflow_effect_at(&root)`. Exactly one workflow per root; no sidecar table; no parallel hosting. See §"Authority site for WorkflowEffect" below.
- **Construction authority for `BranchArm`.** After `bool_port_of` succeeds, **`BranchArm::new(condition, body)`** is the public arm constructor. The `BoolPortRef` inner field is crate-private; there is no `BranchArm { condition: PortId, ... }`. Any lowering that cannot obtain a witness must use **`bool_port_for_branch_condition_or_diagnose`** (or equivalent) so **`BranchConditionNotBool`** is emitted — never silent absorption. ✓ single authority on the typed-witness path.

**Q6 — Representation duality.** Can the same fact be expressed in two structurally different shapes that comparison treats differently?

- A workflow that is structurally a sequence with a nested branch is expressed *uniquely* as the outermost variant's shape. "Linear with branch in the middle" cannot be represented as `LinearEffect` (because `LinearEffect.ops` is `List<OperationEffect>`, not a list of `WorkflowEffect`); it must be lifted to `BranchEffect { arms: [LinearEffect{A∪B∪D}, LinearEffect{A∪C∪D}] }` — i.e., a branch between two linear paths. This is a structurally-unique canonical form, not a choice between two equivalent representations. A node without a workflow has **`lane2_workflow: None`**. That is not the same as registering **`LinearEffect { ops: [] }`** (empty linear — monoidal identity). ✓ no representation duality between `None` and empty-linear conflation.

All six audit questions stamp cleanly. The full six-question audit is recorded in this section for reviewer-bot verification rather than re-derivation at PR-review time.

### Authority site for `WorkflowEffect`

A `WorkflowEffect` value is hosted **on the computation-substrate Value or Bind behavior at the workflow root**, as an optional field (same name on both):

```rust
// src/v3/compiler/src/dag.rs — R2
pub struct ValueNode {
    // ... existing fields ...
    pub(crate) lane2_workflow: Option<Box<WorkflowEffect>>,
}

pub struct BindNode {
    // ... existing fields ...
    pub(crate) lane2_workflow: Option<Box<WorkflowEffect>>,
}
```

`Dag` accessors pattern-match both behaviors:

```rust
impl Dag {
    pub fn try_register_lane2_workflow_effect(
        &mut self,
        root: NodeId,
        workflow: WorkflowEffect,
    ) -> bool {
        match self.nodes.get_mut(root.index()) {
            Some(Behavior::Value(v)) => { v.lane2_workflow = Some(Box::new(workflow)); true }
            Some(Behavior::Bind(b))  => { b.lane2_workflow = Some(Box::new(workflow)); true }
            _ => false,  // other Behavior kinds cannot host a WorkflowEffect
        }
    }

    pub fn lane2_workflow_effect_at(&self, root: &NodeId) -> Option<&WorkflowEffect> {
        match self.node_opt(root)? {
            Behavior::Value(v) => v.lane2_workflow(),
            Behavior::Bind(b) => b.lane2_workflow(),
            _ => None,
        }
    }
}
```

One workflow per root node, keyed by `NodeId`. `workflow_idempotency::analyze_workflow` consumes through `lane2_workflow_effect_at(&workflow_root)` — one graph-local store, no parallel side table.

**Authority contract (R2, live in tree):**

- **One `WorkflowEffect` per workflow-root node.** The root must be a `Value` or `Bind` behavior; other `Behavior` variants cannot host the carrier. The root is identified by `NodeId`; the `lane2_workflow` field holds at most one `WorkflowEffect`. No `Dag.workflows: List<...>` sidecar; the carrier is attached to the node it describes.
- **Writes.** `Dag::try_register_lane2_workflow_effect` is the staging/test write path today; **future lowering** will also populate `lane2_workflow`. Re-registration overwrites in the current implementation.
- **Readers.** Every consumer reads via `Dag::lane2_workflow_effect_at(&root: &NodeId)` (matches substrate accessor emission — explicit boundary with `node_opt` / `port_opt`).
- **Reflection.** `lane2_workflow` is declared on `ValueNode` / `BindNode` in `substrate.dag` and is readable from `.dag` via `lane2_workflow_at` + per-target realizations (e.g. `rust.dag`). Part 3 remains: user **surface** authoring (`data my_flow: WorkflowEffect = …`) end-to-end.

**Why both `Value` and `Bind`** (not just `Value`)**.** A workflow root may be either: a top-level `data` declaration whose root is a `Value` behavior (the RHS expression), or a nested workflow computed as part of a `Bind` (let-binding a sub-workflow). Both cases must host a `WorkflowEffect`; the shipped impl attaches the same `lane2_workflow` field to both `Behavior` variants rather than inventing a wrapper. Other `Behavior` kinds (`Transform`, `Branch`, `Loop`) cannot host a workflow root — a `Branch` IS a control-flow node; wrapping one in `WorkflowEffect::BranchEffect` is a type error — so `try_register_lane2_workflow_effect` correctly returns `false` against them.

**Part 3 follow-up — data-declaration authoring surface.** The eventual end-user surface is a `data my_flow: WorkflowEffect = BranchEffect { arms: [ BranchArm { condition: <bool-expr>, body: <workflow-expr> }, ... ] }` declaration whose lowered computation sub-DAG registers a `WorkflowEffect` via `try_register_lane2_workflow_effect`. That path — source-form parsing, expression-to-port wiring, fail-closed diagnostic on non-Bool conditions — is **not** fully shipped. It is Part 3 work tracked against:

- Surface-to-`WorkflowEffect` lowering (`bool_port_of` / `BranchArm::new` / `bool_port_for_branch_condition_or_diagnose` at branch arms) under the data-declaration surface.
- End-to-end wiring so user-authored `BranchArm` literals reach the same constructors tests use today.

**Rejected host alternatives:** see §Rejected alternatives. `Dag.workflows` sidecar table, field-on-`OperationDeclaration`, lens-time reconstruction, new `WorkflowDeclaration` declaration kind — each is enumerated with its rejection reason so newcomers don't re-propose them.

### Source-to-handle contract (Part 3)

The `lane2_workflow` authority above specifies *where* the `WorkflowEffect` lives in the substrate today. This subsection specifies *how* a future user-authored `BranchArm.condition` will reach its `BoolPortRef` type once the Part 3 data-declaration surface lands. Today, tests and scaffolding build arms with **`bool_port_of`** + **`BranchArm::new`**, or use **`bool_port_for_branch_condition_or_diagnose`** when a diagnostic is required.

The locked-ahead contract (to prevent Part 3 from inventing escape hatches):

1. **Surface form (Part 3 target).** A `BranchArm.condition` will be authored as an **ordinary Bool-typed expression** in the surface language, identical to the syntax for any Bool-typed value elsewhere. No DB-18-specific syntax. No port-lookup function. Example:

   ```
   data my_flow: WorkflowEffect = BranchEffect {
     arms: [
       BranchArm {
         condition: auth_state == AuthOk          // Bool-typed expression
         body: LinearEffect { ops: [Secret.get] }
       }
       BranchArm {
         condition: retry_count < max_retries     // Bool-typed expression
         body: LinearEffect { ops: [STS.exchange] }
       }
     ]
   }
   ```

2. **Lowering path (Part 3, sole authority).** When lowering encounters a `BranchArm.condition` surface-level expression:
   1. Lower the expression into a computation sub-DAG via the standard value-expression → sub-DAG path (no DB-18-specific pipeline).
   2. Take the sub-DAG's root output port as a `PortId`.
   3. Obtain `BoolPortRef` via `bool_port_of` (or `bool_port_for_branch_condition_or_diagnose` when emitting diagnostics), then `BranchArm::new(witness, body)`.
   4. On success: install the `BranchArm` into the enclosing `arms` list.
   5. On failure (port not Bool): emit `Diagnostic::BranchConditionNotBool { port, actual_type, span, ... }` with the span pointing at the source expression. Do NOT construct a `BranchArm` on the failure path.

   This is the SOLE source → `BranchArm` recovery mechanism once Part 3 ships. No alternative.

3. **Rejected alternative source forms (for Part 3):**
   - **Named-port lookup** (e.g., `condition: port("my_branch_condition")`). Violates `feedback_no_metadata_markers` (string-keyed structural recovery).
   - **Raw `NodeId` / `PortId` literal** (e.g., `condition: NodeId(42)`). Violates structural opacity of substrate handles; a `BoolPortRef` must come from **`bool_port_of`** (or the diagnose wrapper), not from a user-authored integer.
   - **Lowering-time synthesis without source anchor** (e.g., lowering fabricates a bool witness when the user didn't author one). Violates `feedback_declare_facts_dont_derive`.
   - **Any path that produces a `BoolPortRef` without going through the validated constructors.** `BoolPortRef` has no public field constructor; this rejection is enforced by the Rust type's visibility — no unsafe escape hatch.

4. **Fail-closed span discipline.** The `Diagnostic::BranchConditionNotBool` span must point at the source expression of the condition, not at the `BranchArm` or `WorkflowEffect` wrapper. Part 3 review rejects any PR whose diagnostic span points at the wrong surface form, or whose lowering silently absorbs `None` without a `Diagnostic`.

**Why the expression-based form (and not a dedicated port-reference syntax) in Part 3.** Per `feedback_std_over_patterns`: reuse existing surface. The user already writes Bool-typed expressions for every other Bool-consuming slot (`if <expr> then ... else ...`, refinements `where <expr>`, modifier predicates). Adding a DB-18-specific syntax for branch conditions would enumerate a special case. The expression → sub-DAG → port → **`bool_port_of` → `BranchArm::new`** path reuses existing machinery.

### Consumer contract — Stage 2b (LinearEffect-only scope)

Stage 2b's analyzer walks a `WorkflowEffect` value, dispatching per variant. Only `LinearEffect` yields a `WorkflowCompositionVerdict` wrapping `CompositionVerdict`; the other three variants produce `IdempotencyUnsupported`. The shipped shape in `workflow_idempotency.rs`:

```rust
pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    let Some(workflow) = d.lane2_workflow_effect_at(&workflow_root) else {
        return WorkflowIdempotencyReport::IdempotencyUnsupported(IdempotencyUnsupportedDetail {
            variant_name: "Lane2WorkflowRoot".into(),
            downstream_stage: "lane2_stage2b_idempotency_lens".into(),
            reason: "no WorkflowEffect at this substrate root".into(),
        });
    };
    project_workflow_idempotency_report(workflow)
}
```

`project_workflow_idempotency_report` matches `LinearEffect { ops }` → `WorkflowCompositionVerdict(compose_operation_effects(ops))` and routes non-linear variants to `IdempotencyUnsupportedDetail` with `lane2_stage2b_idempotency_lens` as downstream stage. Callers keep the `WorkflowEffect` and the report as separate facts (PR #529 R3).

**Why the other three variants are diagnostics, not silent skips.** Per C-8 (INVARIANTS §Diagnostic severity): every detectable condition is either an error or not a diagnostic. A workflow with a `BranchEffect` root is not an error — it is an error only if Stage 2b were claiming to be exhaustive over all workflow shapes. Stage 2b explicitly is *not* exhaustive; it is linear-only. So the diagnostic is informational-framed-as-error ("this variant is out of Stage 2b's scope; see Stage 2d") rather than "your workflow is broken." Downstream stages (2d, 2e) will subsume these diagnostics when they ship, replacing them with real verdicts. The diagnostic contract is: *name the variant, name the downstream stage that will consume it, name the reason it cannot be verdicted at Stage 2b.*

**Why `LinearEffect` is the initial consumer scope.** `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` is already the algebra for linear composition. Stage 2b wires the lens to that algebra without extending it. The other three variants each demand an algebra extension (meet-over-arms, fixpoint, commutative-concurrent) that belongs to downstream stages. Gating Stage 2b to `LinearEffect` delivers the thesis claim ("idempotency is inescapable at compile time") for the 80% case without pre-committing to algebra extensions the downstream stages will design.

### Coexistence with `CompositionVerdict` (post-PR #529)

`WorkflowEffect` and `CompositionVerdict` are orthogonal axes of the effect algebra. Concretely:

| Carrier | Axis | Produced by | Consumed by |
|---|---|---|---|
| `WorkflowEffect` | Input structure — "what shape is this workflow?" | Lowering (Part 2) | Stage 2b lens, Stage 2d cost lens, Stage 2e parallelism lens, Stage 2f user-dimension lens |
| `OperationEffect` | Per-op effect shape | Lowering (Stage 2a) via `derive_op_effect` | `compose_effects`, `LinearEffect.ops` |
| `CompositionVerdict` | Output verdict — "what does this linear chain compose to?" | `compose_effects(List<OperationEffect>)` (post-PR #529) | Stage 2b lens report; any downstream diagnostic constructor |

`WorkflowEffect` never carries a `CompositionVerdict`; `CompositionVerdict` never carries a `WorkflowEffect`. The edge between them is a single lens-level match arm: `LinearEffect { ops } =>` projects `ops` through `compose_effects` to obtain `CompositionVerdict`. No enclosing record pairs them.

This is deliberate. PR #529 removed `ComposedEffect { operations, verdict }` because pairing the input walk and the output verdict in a single record admitted correlated-fields incoherence. DB-18 preserves that lesson: the input structure lives on one axis (`WorkflowEffect`), the output verdict lives on another axis (`CompositionVerdict`), and nothing pairs them. Stage 2b callers that want "the walk AND the verdict" keep their own `WorkflowEffect` in scope and read the lens's `CompositionVerdict` output — two facts kept separate at their natural sites, with no correlation for the type system to enforce.

### What DB-18 does NOT touch

- `compose_effects` signature or body. Continues to consume `List<OperationEffect>` and return `CompositionVerdict` per PR #529.
- `EffectShape` partitioning (`IdempotentShape` / `BreakingShape`). Per PR #529, already landed in Stage 2a follow-up.
- `OperationEffect`, `BreakingOperation`, `CompositionVerdict`, `IdempotencyEvidence`, `ModifierCheck`. Untouched.
- The computation substrate (`Behavior`, `LoopBound`, `Cluster`). Untouched. `Behavior` stays at five variants.
- The reflected `Dag` record. Untouched. No new sidecar table for workflows (per §"Authority site for `WorkflowEffect`" — data-declaration value slot is the host).
- Existing Track 9 primitives (`NonEmptyList`, `NonSingletonList`, `ParamRef`, `TransformRef`). Untouched by shape change; `BoolPortRef` joins them as a new peer primitive without modifying any existing one.
- `WorkflowEffectConcern` (existing record at `src/v3/std/effects.dag:669–673` post-PR #529; was `:565–569` pre-#529). This is a diagnostic-construction helper, not an input carrier; remains as-is. Part 2 may or may not project through it for the `LinearEffect` diagnostic construction.
- V2 `dsl/std/effects.dag`. V3-only, per the same scope discipline PR #529 applied.

### What DB-18 DOES add (summary)

**Part 2 — R2 shipped (DB-18 Part 2; see PR #545 and predecessors):**
- Rust enum `WorkflowEffect` + `BranchArm` + `BoolPortRef` in `src/v3/compiler/src/dag.rs`; `effects.dag` marks carriers **🟢 TERMINAL** for Stage 2b / 2e consumers.
- Fields `ValueNode.lane2_workflow` / `BindNode.lane2_workflow: Option<Box<WorkflowEffect>>` — authority site; reflected in `substrate.dag`.
- **`bool_port_of`**, **`BranchArm::new`**, **`bool_port_for_branch_condition_or_diagnose`** — R2 Bool / branch-arm construction (no `branch_arm_of`).
- `Dag::try_register_lane2_workflow_effect` + `Dag::lane2_workflow_effect_at(&self, root: &NodeId)` — writer / reader.
- `workflow_idempotency::analyze_workflow` — `LinearEffect` → verdict; non-linear → `IdempotencyUnsupported`.

**Part 3 — follow-up:**
- Data-declaration authoring surface: end-to-end lowering from `data my_flow: WorkflowEffect = …` using the same R2 constructors.
- Optional polish beyond [`m2_lens_idempotency_migration_test.rs`](../src/v3/compiler/tests/m2_lens_idempotency_migration_test.rs) (rustc round-trip vs oracle is already gated).

---

## Worked example — workflow shapes under DB-18

Three example workflows, their `WorkflowEffect` encoding, and the Stage 2b verdict under this design.

**Workflow A — GCP linear chain (Stage 2b golden path).**

```
Secret.upsert(key=secret_id, value=...)   // UpsertEffect
STS.exchange(token=...)                    // ReadEffect (idempotent on state)
IAM.grant(role=...)                         // UpsertEffect
```

Encoded as (ops is `List<OperationEffect>`):

```rust
WorkflowEffect::LinearEffect {
    ops: vec![
        OperationEffect { operation_name: "Secret.upsert".into(), shape: /* … */ },
        OperationEffect { operation_name: "STS.exchange".into(), shape: /* … */ },
        OperationEffect { operation_name: "IAM.grant".into(), shape: /* … */ },
    ],
}
```

Stage 2b dispatches on `LinearEffect`, calls `compose_operation_effects` / `compose_effects` on `ops`, receives `CompositionVerdict::IdempotentComposition` inside `WorkflowCompositionVerdict`. Report green. ✓ — mirrors Stage 2b fixtures.

**Workflow B — linear chain with terminal audit log (Stage 2b red).**

```
Secret.upsert(...)                          // UpsertEffect (idempotent)
STS.exchange(...)                           // ReadEffect (idempotent)
IAM.grant(...)                              // UpsertEffect (idempotent)
AuditLog.append(event=...)                  // AppendEffect (breaking)
```

Encoded as `LinearEffect { ops: [upsert, exchange, grant, append] }`. Stage 2b calls `compose_effects`, receives `CompositionVerdict::BrokenBy { first_breaker: BreakingOperation { operation_name: "AuditLog.append", shape: AppendEffect } }`. (`BreakingOperation.shape: BreakingShape` already narrows to the breaking subset — no `IsBreaking` wrapper.) Report red with diagnostic naming `AuditLog.append` as the breaker. ✓ — mirrors `lane2-compile-time-proofs.md` §Stage 2b Acceptance fixture 2.

**Workflow C — retry with nested branch (Stage 2b diagnostic).**

```rust
// R2: bool_port_of + BranchArm::new (tests use this pattern).
let arm_ok = BranchArm::new(
    dag.bool_port_of(port_auth_ok).expect("Bool"),
    WorkflowEffect::LinearEffect { ops: vec![secret_get] },
);
let arm_failed = BranchArm::new(
    dag.bool_port_of(port_auth_failed).expect("Bool"),
    WorkflowEffect::LinearEffect {
        ops: vec![sts_exchange, secret_get],
    },
);
let workflow = WorkflowEffect::LoopEffect {
    body: Box::new(WorkflowEffect::BranchEffect {
        arms: /* NonSingletonList of arm_ok, arm_failed */,
    }),
};
dag.try_register_lane2_workflow_effect(workflow_root, workflow);
```

Stage 2b dispatches on `LoopEffect`, emits diagnostic: *"`LoopEffect` encountered at workflow root; iterated effect composition requires body idempotency + bound evidence — consumed by Lane 2 Stage 2d (fixpoint bound + body convergence)."* No verdict. ✓ — the diagnostic path from §"Consumer contract" above; the user knows *why* Stage 2b did not verdict and *which stage will*.

---

## Acceptance

Design-contract items (locked in this doc, independent of shipping-phase):

1. `WorkflowEffect` is a four-variant coproduct (`LinearEffect | BranchEffect | LoopEffect | ParallelEffect`) with the payload shapes in §"Substrate changes." The locked invariant is the set of VARIANTS (four, exactly) and the MANDATORY fields of each variant; additive-extension fields graduate as downstream stages bind (e.g., `LoopEffect.bound` when Stage 2d consumes, `ParallelEffect.commutativity` when Stage 2e consumes) and do not regress this lock.
2. `BranchArm { condition: BoolPortRef, body: WorkflowEffect }` is the sole structural distinction between `BranchEffect` and `ParallelEffect` payloads. `BoolPortRef` is obtained via **`bool_port_of`**; arms are built with **`BranchArm::new`**. Raw `PortId` does not appear in `condition`. The Q4 Pattern-2 distinction is carried by the typed witness, not by constructor discipline.
3. Q1–Q6 substrate-principle audit is stamped in-doc; Q4 dissolution receipt is stamped in-doc. Q5 single-authority is resolved by the §"Authority site for WorkflowEffect" section.
4. `LinearEffect` is the Stage 2b consumer; the other three variants produce `WorkflowIdempotencyReport::IdempotencyUnsupported` with a variant-specific `IdempotencyUnsupportedDetail`. Empty `LinearEffect.ops` is the monoidal identity at the workflow carrier; `compose_operation_effects` maps it to `IdempotentComposition`.
5. `CompositionVerdict` and `WorkflowEffect` coexist on orthogonal axes — no enclosing record pairs them; `LinearEffect`'s dispatch is the sole edge between them.
6. `WorkflowEffect`'s authority site is `ValueNode.lane2_workflow` / `BindNode.lane2_workflow` at the workflow root. Accessors: `try_register_lane2_workflow_effect` (write) and `lane2_workflow_effect_at(&self, root: &NodeId)` (read). One workflow per root; no `Dag.workflows` sidecar.
7. Part 3 source-to-handle: expression → port → **`bool_port_of` / `bool_port_for_branch_condition_or_diagnose`** → **`BranchArm::new`**, fail-closed `BranchConditionNotBool` when the port is not Bool.
8. Algebra pre-gate: PR #529 (`CompositionVerdict`). R2 substrate: DB-18 Part 2 (PR **#545** et seq.).

**Part 2 — R2 shipped** (Rust + `effects.dag` + reflection + accessor realizations):

1. `WorkflowEffect`, `BranchArm`, `BoolPortRef` in `dag.rs`; **🟢 TERMINAL** in `effects.dag`.
2. `bool_port_of`, `BranchArm::new`, `bool_port_for_branch_condition_or_diagnose`.
3. `try_register_lane2_workflow_effect`, `lane2_workflow_effect_at(&self, root: &NodeId)`.
4. `workflow_idempotency::analyze_workflow` + `project_workflow_idempotency_report`.
5. Tests: linear green/red, empty-linear idempotent, `BranchConditionNotBool`, non-linear unsupported paths.

**Part 3 — follow-up:**

1. **Data-declaration surface** `data my_flow: WorkflowEffect = …` wired end-to-end through the same R2 constructors (today: tests + `try_register` only).
2. Optional: extend **rustc** coverage beyond [`m2_lens_idempotency_migration_test.rs`](../src/v3/compiler/tests/m2_lens_idempotency_migration_test.rs).

**Pre-start gate for Part 3 surface:** reflection for `lane2_workflow` is **landed** (substrate + specs); the remaining gate is user-facing authoring + lowering polish.

---

## STOP-AND-ESCALATE rules (Part 3 dispatch)

R2 substrate + reflection have shipped; Part 3 (data-declaration **authoring** surface) is the next dispatch under DB-18. If implementation discovers any of the following, HALT and report to director chat rather than patching forward:

- `WorkflowEffect`'s 4-variant shape is insufficient — e.g., a real workflow fixture requires a fifth variant or a variant payload reshape. DB-18 locks the shape; reshape is a DB revision, not an in-flight patch.
- `BoolPortRef` does not distinguish BranchEffect from ParallelEffect structurally in practice (Q4 receipt regresses), OR `bool_port_of` / diagnose fail-closed semantics require an escape hatch for some legitimate lowering case. Same rule — reshape the substrate only through a DB revision.
- The authority-site decision (`ValueNode.lane2_workflow` keyed by workflow-root `NodeId`) turns out to be insufficient — e.g., a real fixture needs `WorkflowEffect` hosted somewhere the computation-substrate root cannot reach. DB-18 locks the host; changing it is a DB revision, not an in-flight patch.
- A non-Bool branch condition is silently absorbed at any lowering site (no `Diagnostic::BranchConditionNotBool` emission). This is a C-8 violation — Part 3 review must reject it.
- The data-declaration surface for authoring `WorkflowEffect` requires a new FieldValue variant (i.e., the current `ValueBody` / `FieldValue` substrate cannot encode the `data my_flow: WorkflowEffect = ...` literal). This is a substrate extension, not a Part 3 patch — escalate before extending.

The director chat owns the call on each of these. Silent in-flight patches destroy the structural-finding signal the pre-clearance exists to capture (per `phase-plan-2026-04-18.md` §7 "Structural-finding escalation rule").

---

## Open questions

1. **Does `ParallelEffect` need a commutativity witness field in Part 2?** The Pattern 3 algebraic-form analysis argues `ParallelEffect` is a concurrent-product (⊗) requiring algebraic commutativity on the target state. Part 1 does NOT require the witness in the carrier because Stage 2b emits a diagnostic on `ParallelEffect` (no verdict computed). When Stage 2e's parallelism lens binds as a real consumer, it will either (a) extend `ParallelEffect` with `commutativity: AlgebraRef` (or a named-alternative witness carrier) as an additive payload extension, or (b) prove that commutativity is derivable from the workflow's op-level algebra without a substrate field. Decision deferred to Stage 2e design pre-clearance — NOT patched forward in Part 2 implementation. This is an additive extension consistent with the Acceptance-item-1 language (locked VARIANTS + MANDATORY fields; optional additive fields are in-scope for later stages).

2. **Exact `WorkflowIdempotencyReport` shape.** Must project through `CompositionVerdict` (per PR #529's pre-start-gate update to `lane2-compile-time-proofs.md` §Stage 2b). A candidate shape `{ verdict: CompositionVerdict, workflow_shape: WorkflowEffect, diagnostic: Diagnostic? }` would pair the algebra output with the input structure in a single record — the correlated-fields pattern PR #529 rejected. The alternative is to return `CompositionVerdict` OR `Diagnostic` directly from `analyze_workflow_value` (no outer record); the lens's caller pairs it with the `WorkflowEffect` they already hold. Lean: the no-outer-record alternative. Resolve at Part 2 design.

3. **Does `LoopEffect.body` need a bound carrier in Part 1?** Substrate has `LoopBound = Cardinality { count: PortId } | Descent { cluster: ClusterId }`. Stage 2d (symbolic cost) will almost certainly need the bound to compute recursion depth. Part 1 does NOT include a bound field because Stage 2b emits a diagnostic on `LoopEffect` without reading bound info. Part 2 / Stage 2d can add `bound: LoopBound` as an additive extension when its consumer binds — consistent with Acceptance item 1's additive-extension clause. If director chat prefers a single shape-lock including the bound, a future DB revision adds `LoopEffect { body: WorkflowEffect, bound: LoopBound }`; decision is a scope judgment, not a correctness one.

4. **Does `LinearEffect.ops` need to be `NonEmptyList<WorkflowEffect>` instead of `List<OperationEffect>` to allow mixed nesting?** The worked example C demonstrates the answer: a linear sequence with a branch in the middle lifts to a `BranchEffect` of two linear paths (distributive over composition). No `LinearEffect` with a `WorkflowEffect` in the middle is needed; the structural canonical form is always a variant at the outermost shape. Locked per Q6 (no representation duality) — not open.

---

## Rejected alternatives

Each entry names a shape a reader might reasonably propose and states the live reason it stays out of scope. Read these before proposing an alternative shape.

**Lens-only, derive `WorkflowEffect` from DAG shape at walk time.** Re-derives workflow structure heuristically from Bind chain / Branch / Loop behavior, violating `feedback_lenses_not_passes` ("heuristic = missing physics"). A workflow's control-flow shape is a fact about the author's intent, not something the compiler should re-infer per-lens.

**Single `WorkflowEffect` coproduct with a kind tag + optional payload fields.** Shape: `WorkflowEffect { kind: Linear | Branch | Loop | Parallel, ops: Option<List<OperationEffect>>, arms: Option<List<WorkflowEffect>>, body: Option<WorkflowEffect> }`. Admits illegal state combinations (`kind: Linear` with `arms: Some(...)`) — the classic `Bool + Option<T>` pattern `feedback_state_space_vs_behavioral_invariants` names. The four-variant coproduct is the state-space-sound shape.

**Hierarchical inheritance: `WorkflowEffect` is-a `List<OperationEffect>` with optional control-flow overlay.** Pre-commits to linear-as-base and overlays branches as a non-structural layer. Stage 2d's branch-wise composition would have to peel back the overlay — the overlay IS the structure. Control-flow is encoded structurally from the start.

**Merge `BranchEffect` and `ParallelEffect` into `ChoiceEffect { kind: Exclusive | Concurrent, arms: NonSingletonList<WorkflowEffect> }`.** Re-introduces the `kind + discriminator` pattern Q4's dissolution receipt eliminates. Each variant traces to a distinct categorical operation; compressing two of them under a flag re-creates the compression the coproduct solves.

**Drop `ParallelEffect`; add it later via DB-N when Stage 2e ships.** Viable but not chosen: the four-variant shape is locked up front so Stage 2e's DB extends the existing carrier rather than graduating it. A 3-variant initial shape would be an additive refinement and does not regress the Q4 receipt for the remaining three variants — noted as a director-chat judgment call if the 4-variant commit proves too heavy.

**Raw `PortId` on `BranchArm.condition` with constructor-level validation only.** Rejected: R2 uses **`BoolPortRef`** + **`bool_port_of`**, not raw ports. A contributor cannot type-check `BranchArm { condition: PortId, ... }` — the field is `BoolPortRef`. The Q4 receipt leans on that typed witness as the load-bearing distinction between `BranchEffect` and `ParallelEffect`.

**Host `WorkflowEffect` on a new `WorkflowDeclaration` kind or as a field on `OperationDeclaration`.** Adds a substrate concept (or conflates workflow composition with per-op effect) without capability gain. The adopted host (`ValueNode.lane2_workflow` on the computation-substrate root) attaches the workflow to the node it describes and reuses the existing `ValueNode` machinery. The Part 3 data-declaration surface for authoring workflows reuses the existing `data` surface with the `WorkflowEffect` type annotation — no new declaration kind, no specific-to-workflow hosting concept.

**Sidecar table `Dag.workflows`.** Parallel to `ValueNode.lane2_workflow`; would require a derivation to stay in sync with the field. Violates Q3 (no duplicated facts) and Q5 (single authority). Sidecars are justified when a fact spans multiple declarations (e.g., `Dag.clusters`); a `WorkflowEffect` is attached to one root node.

**Host `WorkflowEffect` on the declaration's `value_body` (FieldValue tree) directly.** Considered and rejected for the shipped shape because `ValueBody` / `FieldValue` today cannot carry opaque handles like `BoolPortRef` — a `FieldValue::Variant { constructor, payload }` literal cannot inhabit a Rust-only opaque type. The Part 3 data-declaration surface compiles the surface literal into a computation sub-DAG, obtains **`BoolPortRef`** witnesses from those ports via **`bool_port_of`**, builds **`BranchArm::new`**, and registers the final `WorkflowEffect` on the root's `lane2_workflow` field. This is the single-authority path — the data-declaration surface is the authoring surface; `ValueNode.lane2_workflow` is the authority storage; the two are connected by the Part 3 lowering pipeline.

---

## Cross-references

- `feedback_coproduct_dissolution` — Q4 receipt (§"Dissolution receipt" above cites patterns 1–4).
- `feedback_substrate_principle_audit` — Q1–Q6 (§"Substrate principle audit" cites all six).
- `feedback_state_space_vs_behavioral_invariants` — cardinality invariants via `NonEmptyList` / `NonSingletonList`; rejected alternative R-alt-A.
- `feedback_lenses_not_passes` — anchor for the lens-level re-derivation rejection (see §Rejected alternatives).
- `feedback_no_metadata_markers` — `BranchArm.condition: BoolPortRef` is a typed opaque handle, not a string marker or raw index.
- `feedback_fail_closed_discipline` — `WorkflowIdempotencyReport::IdempotencyUnsupported` carries a typed `IdempotencyUnsupportedDetail`, not a silent skip; non-Bool branch conditions surface as **`Diagnostic::BranchConditionNotBool`** (via `bool_port_for_branch_condition_or_diagnose` or equivalent).
- `feedback_std_over_patterns` — authority site reuses `ValueNode` rather than introducing a new declaration kind; Part 3 surface reuses `data` rather than introducing a workflow-specific surface.
- DB-9 R2.1 (`design-mutual-recursion-lowering.md`) — worked example of the six-question audit; mirrors DB-18's format. Also: the `ParamRef` / `TransformRef` Track 9 primitive pattern that `BoolPortRef` follows.
- DB-16 (`design-db16-refined-generic-substitution.md`) — worked example of Part 1 design / Part 2 impl split; mirrors DB-18's scope discipline.
- PR #529 (`design-composed-effect-reshape.md`) — `CompositionVerdict` authority that DB-18's `LinearEffect` path delegates to. Merged 2026-04-18 as `8c7e7acdd`.
- PR #534 / **#545** — DB-18 Part 1 / Part 2 (β + R2 reconciliation): Rust mirrors, `bool_port_of` / `BranchArm::new`, `workflow_idempotency`, reflected `lane2_workflow`, `lane2_workflow_at` realizations. Part 3 (data-declaration **authoring** surface) remains follow-up.
- ROADMAP — Lane 2 Stage 2b row records DB-18 Part 2 shipped; debt rows for reflection + R1→R2 cleared with the landing PRs.

---

End of DB-18.
