> Part of: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) (Lane 2 Stage 2b) | Companion: [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (PR #529) | Unblocks: Lane 2 Stage 2b implementation (workflow idempotency lens); informs Lane 2 Stages 2d / 2e / 2f (downstream composition over workflow structure)

# Design DB-18 — `WorkflowEffect` substrate carrier (Stage 2b input structure)

**Design blocker:** DB-18 (new substrate carrier `WorkflowEffect` — a four-variant coproduct describing workflow control-flow shape for effect-algebra composition)
**Consumers:** Lane 2 Stage 2b workflow idempotency lens (initial consumer; `LinearEffect`-only scope). Downstream: Stage 2d symbolic cost, Stage 2e parallelism-as-lens, Stage 2f user-declared dimensions.
**Status:** Part 1 design-only — implementation and tests ship in the follow-up Part 2 PR.
**Companion:** [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (PR #529, landed 2026-04-18 as `8c7e7acdd`) reshapes the *output* side of the effect algebra: `ComposedEffect` is removed; `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` is the post-reshape algebra. DB-18 adds the *input* side: a typed carrier describing the workflow's control-flow structure above `List<OperationEffect>`. The two live on orthogonal axes — `WorkflowEffect` is the input structure the lens walks; `CompositionVerdict` is the output the algebra returns. They coexist; neither replaces the other.

---

## Summary

Stage 2b's pre-DB-18 design (`lane2-compile-time-proofs.md` §Stage 2b) assumed a workflow is a linear `List<OperationEffect>` — `compose_effects` walks the list, composes effect shapes, emits a verdict. That shape is correct for the narrow case (GCP Secret Manager upsert → STS Exchange → IAM grant, all linear) but the escalation clause in the same section already names the extension: *"if workflow structure isn't representable cleanly — e.g., control flow in a pipeline doesn't map to a linear `List<OperationEffect>` — surface. Don't stretch `compose_effects` to handle branches silently; the algebra needs to reflect branch-wise composition, which is a legitimate design extension."*

DB-18 is that legitimate design extension, moved up-front rather than deferred. The substrate gains **one new reflected coproduct** (`WorkflowEffect`), **one helper record** (`BranchArm`), and **one Track 9-style typed opaque handle** (`BoolPortRef`, the R2 third Track 9 primitive after `ParamRef` and `TransformRef`). The computation substrate is untouched (no new `Behavior` variant, no `LoopBound` change); the effect algebra (`compose_effects`, `CompositionVerdict`) is untouched. The authority site for `WorkflowEffect` values is locked to user-declared `data` declarations typed `WorkflowEffect` — no new declaration kind, no sidecar table. Stage 2b's lens matches on `LinearEffect` only and emits explicit diagnostics on the other three variants, each naming the downstream stage that will consume it. The diagnostic is a first-class `Diagnostic`, not a silent skip — the fail-closed contract from INVARIANTS §C-8 applies. `bool_port_of` (the `BoolPortRef` constructor) returning `None` is a fail-closed boundary at the caller: lowering emits `Diagnostic::BranchConditionNotBool`, never silently absorbs the absence.

`WorkflowEffect` is the input-structure carrier. `CompositionVerdict` (PR #529) is the output-verdict carrier. They meet at exactly one edge: Stage 2b's lens projects `LinearEffect { ops }` through `compose_effects(ops)` to obtain a `CompositionVerdict`; other `WorkflowEffect` variants produce diagnostics without invoking the algebra.

---

## Constraints (non-negotiable)

1. **`CompositionVerdict` authority preserved.** Post-PR #529, `compose_effects(List<OperationEffect>) -> CompositionVerdict` is the sole effect-algebra output. DB-18 does not introduce a parallel verdict carrier. `LinearEffect`'s lens delegates to `compose_effects`; other variants do not compose at Stage 2b.
2. **Five-behavior computation substrate untouched.** `Behavior = Value | Transform | Branch | Loop | Bind` remains at five variants. `WorkflowEffect` lives in the *type substrate* (declarations), not the computation substrate. Parallel lens tests (`test_thesis_five_behavior_variants`-style) stay green by construction.
3. **Bounded kernel invariant.** `WorkflowEffect` is recursive through its own variant fields; the recursion terminates at `LinearEffect.ops: List<OperationEffect>` (no further `WorkflowEffect` children; empty list is a well-defined leaf). Finite by construction because the substrate is a DAG (no cycles in `WorkflowEffect`-typed edges).
4. **Illegal states unrepresentable at the type level.** Cardinality invariants use Track 9 primitives (`NonEmptyList`, `NonSingletonList`). No `List<T>` with a "lowering guarantees ≥1" comment; no `Bool + Option<T>` pairs that admit contradictions.
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
//
// Recursive composition: BranchEffect, LoopEffect, and ParallelEffect
// each carry sub-WorkflowEffect fields, so arbitrary nesting composes.
// LinearEffect terminates the recursion at a (possibly empty) list of
// OperationEffect values — the same carrier compose_effects consumes
// post-PR-#529. The empty case is the monoidal identity.
type WorkflowEffect
  = LinearEffect { ops: List<OperationEffect> }
  | BranchEffect { arms: NonSingletonList<BranchArm> }
  | LoopEffect { body: WorkflowEffect }
  | ParallelEffect { branches: NonSingletonList<WorkflowEffect> }

// 🟢 TERMINAL. A single arm of a BranchEffect — the Bool-typed
// condition port witnessing "this arm is taken" plus the workflow
// executed when the condition holds. Separating condition from
// workflow makes the payload structurally distinct from
// ParallelEffect's NonSingletonList<WorkflowEffect>, so the Q4
// dissolution receipt passes at the payload level rather than
// requiring a separate discriminator tag.
//
// `condition: BoolPortRef` — a typed opaque handle statically
// witnessing that the referenced port is Bool-typed. Because
// BoolPortRef has no unsafe constructor (see `type BoolPortRef`
// below), a raw-literal `BranchArm { condition: <arbitrary PortId>, ... }`
// is not representable: the field type itself rejects non-Bool ports
// at the type level. No convention-level constructor discipline is
// required.
type BranchArm {
  condition: BoolPortRef
  body: WorkflowEffect
}
```

**Add `BoolPortRef` as a Track 9-style substrate integrity primitive** (`src/v3/std/substrate.dag` — alongside `NonEmptyList`, `NonSingletonList`, `ParamRef`, `TransformRef`; graduates WITH the first consumer per Track 9 discipline):

```dag
// 🟢 TERMINAL. Typed opaque handle statically witnessing that the
// referenced substrate port's declared type is Bool. The sole
// constructor `bool_port_of` validates the port's type at
// construction time; there is no unsafe escape hatch for producing
// a BoolPortRef around a non-Bool port.
//
// Track 9 graduation rationale: this is the third Track 9 primitive
// (after ParamRef and TransformRef from DB-9 R2.1). It graduates
// here because BranchArm's first consumer (DB-18 §BranchArm) needs
// a type-level witness for the Bool-typed-port invariant that
// carries the Q4 dissolution receipt distinguishing BranchEffect
// from ParallelEffect. A raw PortId would push the invariant into
// constructor convention, which is API-level rather than type-level
// enforcement — exactly the pattern Track 9 exists to dissolve.
//
// Second consumer candidate: computation-substrate Branch behavior's
// condition slot. If a Track 9 graduation review determines the
// invariant belongs there too, BoolPortRef extends rather than
// duplicating — same primitive, two consumers. Not a Part 1 commit.
type BoolPortRef

// Sole constructor. Returns None when the port's declared type is
// not Bool; callers that need to emit a Diagnostic on None do so
// themselves (fail-closed discipline at the caller boundary, not at
// the primitive boundary — same pattern as `Dag::param_of`).
fn bool_port_of(dag: Dag, port: PortId) -> BoolPortRef?

// Accessor. Recover the underlying PortId from the typed handle;
// the witness that the port is Bool-typed is carried by the handle,
// not by a duplicate inspection.
fn port_of(bool_ref: BoolPortRef) -> PortId
```

**Fail-closed boundary placement.** `bool_port_of` returning `Option` is the Track 9 primitive's internal-validation contract (matching `param_of`). The primitive does not itself emit a `Diagnostic` — emitting diagnostics is not the responsibility of a typed-handle primitive. The *lowering path that calls `bool_port_of`* is required to handle `None` by emitting `Diagnostic::BranchConditionNotBool { port, actual_type, span }` per C-8, never by silently absorbing the absence. The primitive's `Option` is internal validation; the caller's `Diagnostic` emission is the fail-closed boundary. Part 2 Acceptance item 4 names this as a review gate — a silent `None` absorption fails Part 2 review.

**Why this belongs in `std/effects.dag` rather than `std/substrate.dag`.** `WorkflowEffect` is a concept of the effect algebra — it describes input shapes that `compose_effects` (and its downstream peers) dispatch over. The substrate's own declarations (`Dag`, `Declaration`, `Behavior`, `LoopBound`, `Cluster`) describe the *computation and type substrate*; the effect algebra is a lens-writable layer above that substrate. The existing file already hosts `OperationEffect`, `EffectShape`, `IdempotencyEvidence`, `CompositionVerdict` — all effect-algebra types. `WorkflowEffect` joins that family.

**No reflected `type Dag` changes.** `WorkflowEffect` values live inside user code (as fields of workflow declarations) and inside lens output, not as a sidecar table on `Dag`. The authority for "what workflow does this call site model?" is the user's declaration; the lens walks declarations and reads their `WorkflowEffect`-typed fields. No new `Dag.workflows: List<...>` field is needed (cf. DB-9 R2.1 `Dag.clusters` sidecar which is necessary because clusters span multiple declarations; a `WorkflowEffect` is a single declaration-local value).

**No Rust mirror yet.** Because Part 1 is design-only, no `src/v3/compiler/src/dag.rs` change is proposed here. Part 2 mirrors `WorkflowEffect` and `BranchArm` in `dag.rs` under the same reflection-invariant check (`m2_field_access_binding_test.rs`) that DB-9 R2.1 used for `Cluster` / `MemberDescent` / `IntraClusterCall`. The single-authority discipline from DB-9 R2.1 carries over: lowering writes `WorkflowEffect` values; lenses and inference are pure readers.

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

The critical pair is BranchEffect vs ParallelEffect — both use `NonSingletonList<...>` at the outer shape and recurse on `WorkflowEffect` at the element level. The distinction is carried by `BranchArm { condition: BoolPortRef, body: WorkflowEffect }` versus a plain `WorkflowEffect`: a branch arm is an arm-gated-by-a-Bool-typed-port, a parallel branch is an unconditioned concurrent workflow. `condition: BoolPortRef` is the load-bearing structural fact, carried by a **typed opaque handle**. `BoolPortRef`'s sole constructor `bool_port_of(dag, port)` returns `None` for non-Bool ports and has no unsafe escape hatch, so the "port is Bool-typed" invariant is witnessed by the type itself rather than by constructor convention. A raw-literal `BranchArm { condition: <arbitrary PortId>, body: ... }` is not type-checkable; only `BranchArm { condition: <BoolPortRef>, body: ... }` is, and a `BoolPortRef` can only come from the fail-closed constructor. Pattern 2's dissolution criterion ("same shape, different label") does not apply because the `BranchArm` wrapper carries a structurally required *typed* field `ParallelEffect`'s branches do not. ✓ does not apply.

**Pattern 3 — Algebraic-form (traces to intro/elim of algebraic structures).** The four variants trace to four distinct categorical operations on effect algebras:

- `LinearEffect` = **monoidal composition (∘)**. `compose_effects` already witnesses this: a linear sequence of effects composes as `∘` with `ReadEffect` (identity on state) as the unit on the effect-shape side AND with the empty list `[]` as the unit on the carrier side. The `CompositionVerdict` is the algebra's output for this case (`compose_effects([])` = `IdempotentComposition`, the identity verdict). The carrier `List<OperationEffect>` is the free monoid on `OperationEffect`, including the empty word as the monoidal identity.
- `BranchEffect` = **coproduct (∨) / lattice meet over arms**. Exactly one arm executes at runtime; the verdict must hold for whichever arm is taken. At compile time this is a `∨`-over-arms: the workflow is idempotent iff every arm's workflow is idempotent. Structurally distinct from `∘` — composition order does not matter for arms that are alternatives, and the per-arm verdicts combine via lattice meet, not via monoidal multiplication.
- `LoopEffect` = **fixpoint (μ) / iteration**. The body is re-applied 0..N times (bound structure tracked separately, potentially in Part 2). Idempotency under iteration demands the body itself be idempotent (`f ∘ f = f`) — a strictly stronger condition than linear composition, because monoidal composition of two different non-idempotent effects can still converge, but iteration of one non-idempotent effect cannot.
- `ParallelEffect` = **concurrent product (⊗) / commutative composition**. Branches execute concurrently and require algebraic commutativity on the target state to compose safely. The verdict's validity depends on commutativity evidence (open question §1 below — Part 2 or Stage 2e may extend the carrier with a `commutativity` witness). Distinct from both `∘` (order-preserving) and `∨` (alternatives): `⊗` is unordered AND concurrent.

Each variant is the introduction form of a different categorical operation. There is no single algebraic operation the four dissolve into; mapping the set of four to a two-dimensional record (e.g., `{ordered, concurrent}`) does not recover the distinct verdict rules each variant demands. ✓ does not apply.

**Pattern 4 — Dimensional (flat enum hides M-dimensional record).** The proposed dimension would be `{control_flow: Linear | Branch | Loop | Parallel, ...}` — but this is just the coproduct tag renamed, with no second coordinate. Attempting a second coordinate produces a partial record where each variant has different required fields (LinearEffect needs `ops`; BranchEffect needs `arms` with conditions; LoopEffect needs `body`; ParallelEffect needs `branches`): these are different types of payload, not orthogonal coordinates on a shared record. A record shape with every payload field present-or-absent would admit illegal combinations (`control_flow: LinearEffect` with `arms: Some(...)`), which violates the Q1 cardinality-invariant check. ✓ does not apply.

**Conclusion:** `WorkflowEffect`'s 4-variant coproduct is structurally irreducible. The coproduct is the correct shape. Stamp: `🟢 TERMINAL` on `WorkflowEffect`, `🟢 TERMINAL` on `BranchArm`.

### Substrate principle audit (Q1–Q6, beyond Q4)

Per `feedback_substrate_principle_audit`, all six questions walk before greenlighting. Q4 is covered above; the other five are stamped here so downstream review verifies the audit ran rather than re-deriving each finding.

**Q1 — Cardinality invariants.** Does any variant admit `[]` where invariant says ≥1, or singletons where ≥2?

- `LinearEffect.ops: List<OperationEffect>` — empty is explicitly allowed as the **monoidal identity** (the unit element of the free monoid on `OperationEffect`). `compose_effects([])` already returns `IdempotentComposition`; the type accepts empty for carrier/algebra symmetry. This is also the honest representation of a no-op workflow (e.g., a branch arm whose body does nothing). No ≥1 invariant applies here; the invariant is "a list of ops (zero or more)," matching the algebra.
- `BranchEffect.arms: NonSingletonList<BranchArm>` — a branch with 0 or 1 arms is meaningless (0: no workflow; 1: just the arm itself, use the arm's `body` directly). Type rejects both.
- `LoopEffect.body: WorkflowEffect` — singleton (one body), never empty. Type is non-optional.
- `ParallelEffect.branches: NonSingletonList<WorkflowEffect>` — 0 branches is empty parallelism; 1 branch is just the branch itself, no concurrency. Type rejects both. ✓ cardinality invariants at the type level (including the correctly-permissive empty `LinearEffect` case); no "lowering guarantees" prose required.

**Q2 — Index/handle types.** Does a raw `Int` or `NodeId` encode something with a domain restriction?

- `BranchArm.condition: BoolPortRef` is a typed opaque handle (Track 9-style primitive, R2 graduation). The sole constructor `bool_port_of(dag: Dag, port: PortId) -> BoolPortRef?` validates the port's declared type is Bool; there is no unsafe escape hatch. A raw `PortId` does not typecheck in `BranchArm.condition`; the field type is `BoolPortRef`, not `PortId`. Matching the `ParamRef` / `TransformRef` pattern from DB-9 R2.1 — the handle carries both "is a valid port reference" AND "is Bool-typed," with the relation folded into the type rather than carried by constructor discipline. ✓ no raw `Int`/`NodeId`/`PortId` with comment-level or constructor-level validity; the invariant is carried by the type itself.

**Q3 — Duplicated fact.** Does Field A duplicate what's derivable from Field B?

- `WorkflowEffect` variants do not duplicate any fact carried by `OperationEffect`, `EffectShape`, or `CompositionVerdict`. `WorkflowEffect` is the *input structure* above ops; the ops themselves are authored once and referenced by `LinearEffect.ops`. `CompositionVerdict` is the *output* of the algebra — DB-18 does not pre-compute a verdict on the workflow shape. ✓ no duplication.

**Q5 — Construction authority.** Are multiple call sites independently constructing the same fact?

- **Authority site for `WorkflowEffect` values.** See §"Authority site for WorkflowEffect" below — `WorkflowEffect` values live exclusively on user-declared `data` declarations whose declared type is structurally `WorkflowEffect`. Exactly one `WorkflowEffect` per qualifying `Declaration`; no sidecar table, no parallel hosting. Lowering is the sole producer; lens and downstream consumers are pure readers.
- **Construction authority for `BranchArm`.** `BranchArm` itself is directly constructible (it is a plain record), but its validity is witnessed by the `condition: BoolPortRef` field type, NOT by a constructor. The sole constructor `bool_port_of(dag: Dag, port: PortId) -> BoolPortRef?` is the Track 9 primitive; any lowering path that obtains a `BoolPortRef` did so via `bool_port_of` and therefore either (a) emitted a `Diagnostic::BranchConditionNotBool` on `None` or (b) proceeded with a type-level Bool witness. Direct `BranchArm { condition: bool_port_ref, body: ... }` construction is fine by design because the condition field CANNOT carry an invalid port. Raw-literal `BranchArm { condition: <arbitrary PortId>, ... }` is a type error. ✓ single authority on the typed-witness construction path; no convention-level escape hatch.

**Q6 — Representation duality.** Can the same fact be expressed in two structurally different shapes that comparison treats differently?

- A workflow that is structurally a sequence with a nested branch is expressed *uniquely* as the outermost variant's shape. "Linear with branch in the middle" cannot be represented as `LinearEffect` (because `LinearEffect.ops` is `List<OperationEffect>`, not `List<WorkflowEffect>`); it must be lifted to `BranchEffect { arms: [LinearEffect{A∪B∪D}, LinearEffect{A∪C∪D}] }` — i.e., a branch between two linear paths. This is a structurally-unique canonical form, not a choice between two equivalent representations. A no-op workflow has exactly one canonical form — `LinearEffect { ops: [] }` (the R2 relaxation) — and cannot be spelled another way under the 4-variant coproduct. ✓ no representation duality.

All six audit questions stamp cleanly. The full six-question audit is recorded in this section for reviewer-bot verification rather than re-derivation at PR-review time.

### Authority site for `WorkflowEffect`

`WorkflowEffect` values are authored in exactly one place: a user-declared `data` declaration whose declared type is structurally `WorkflowEffect`. The v3 `data` declaration form (`data <name>: <type> = <value>`) — shipped in `3a.2` per ROADMAP — is the surface the user writes; lowering parses the RHS literal into a `WorkflowEffect` value and stores it on the resulting `Declaration`'s value slot.

**Authority contract (locked, Part 1):**

- **One `WorkflowEffect` per qualifying `Declaration`.** A `Declaration` whose declared type structurally matches `WorkflowEffect` carries exactly one `WorkflowEffect` value on its value slot. No sidecar table (`Dag.workflows`), no parallel hosting, no multi-workflow-per-declaration.
- **No other hosting site.** `WorkflowEffect` values do NOT appear as fields of `OperationDeclaration`, `ServiceDeclaration`, or any other declaration kind. They do NOT appear as nested fields inside `OperationEffect` or `CompositionVerdict` (the orthogonality guarantee from §"Coexistence"). They do NOT appear in `Dag.nodes` as behavior-level payloads. The only authoritative site is the value slot of a user-declared `data` declaration whose type is `WorkflowEffect`.
- **Lens discovery.** `analyze_workflow` and downstream consumers enumerate qualifying declarations:
  ```dag
  fn find_workflow_declarations(d: Dag) -> List<DeclarationId>
  // Returns the ID of every Declaration whose declared type is
  // structurally WorkflowEffect. Structural match (not name-based)
  // so aliased declared types are recovered via the standard
  // type-resolution walk.
  ```
  and read the value via a standard data-declaration value accessor (exact accessor depends on v3's current data-declaration shape; Part 2 binds against the substrate as it stands at Part 2 implementation time).
- **Q5 closure.** This is the Q5 single-authority answer: the authority site is the `data` declaration; lowering is the sole producer of `WorkflowEffect` values (by parsing the RHS literal); every reader walks the same discovery function. No alternative hosting paths, no inference-time re-derivation, no lens-time reconstruction.

**Why `data` rather than a new `WorkflowDeclaration` kind.** Both are valid; R2 picks `data` because it is zero-substrate-expansion. The v3 substrate already hosts `data` declarations — adding `WorkflowEffect` as a declarable type in that slot is pure consumer extension of an existing surface. A new `WorkflowDeclaration` kind would be a substrate addition that duplicates what `data` already provides (name + type + value) with a specific-to-workflow type constraint; the specific-to-workflow constraint is already carried by the *type* `WorkflowEffect`. Per `feedback_std_over_patterns` ("types-in-std/ dissolves CX patterns; don't enumerate special cases"), the right move is to reuse the generic `data` surface and let the type annotation carry the specificity.

**Why not a `Dag.workflows` sidecar.** Sidecar tables are justified when a fact spans multiple declarations (e.g., `Dag.clusters` — an SCC member list spans the cluster's member declarations, so a per-declaration field cannot host it). A `WorkflowEffect` is a single-declaration-local value — it describes one workflow, authored on one declaration. A sidecar would be a parallel cache of facts already stored on the declaration; the consumer could drift from the authority. Per Q3 (no duplicated facts) and Q5 (single authority), the data-declaration value slot is the correct place.

**Rejected host alternatives:** see §Rejected alternatives (R-alt-F for `WorkflowDeclaration`; R-alt-G for deferring the host decision). Field-on-`OperationDeclaration` and sidecar-table alternatives are enumerated below alongside the adopted path so newcomers don't re-propose them.

### Source-to-handle contract — how a user-authored condition becomes a `BoolPortRef`

The previous subsection locked *where* `WorkflowEffect` values live. This subsection locks *how* a user-authored `BranchArm.condition` reaches its `BoolPortRef` type. Without this rule, Part 2 could invent (a) name-keyed port recovery (violating `feedback_no_metadata_markers`), (b) a raw-id literal escape hatch (violating structural-opacity of substrate handles), or (c) lowering-time synthesis of a bool witness with no source anchor (violating `feedback_declare_facts_dont_derive`). The single legal path is locked below; the three escape hatches are named as rejected so Part 2 cannot silently adopt one.

**Contract (locked, Part 1):**

1. **Surface form.** A `BranchArm.condition` is authored as an **ordinary Bool-typed expression** in the surface language — identical to the syntax the user writes for any Bool-typed value elsewhere. No DB-18-specific syntax. No port-lookup function. Example:

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

   The user writes the condition as an expression; lowering handles the handle-wrapping.

2. **Lowering path (single authority).** When lowering encounters a `BranchArm.condition` surface-level expression:
   1. Lower the expression into a computation sub-DAG via the standard value-expression → sub-DAG path (no DB-18-specific pipeline).
   2. Take the sub-DAG's root output port.
   3. Apply `bool_port_of(dag, port)`.
   4. On `Some(bool_ref)`: construct `BranchArm { condition: bool_ref, body: <lowered body> }`.
   5. On `None` (port's declared type is not Bool): emit `Diagnostic::BranchConditionNotBool { port, actual_type, span }` with the span pointing at the source expression (not the surrounding BranchArm or WorkflowEffect). Do NOT construct a `BranchArm` on the `None` path.

   This path is the SOLE source → `BoolPortRef` recovery mechanism. No alternative.

3. **Rejected alternative source forms:**
   - **Named-port lookup** (e.g., `condition: port("my_branch_condition")`). Violates `feedback_no_metadata_markers` (string-keyed structural recovery).
   - **Raw `NodeId` / `PortId` literal** (e.g., `condition: NodeId(42)`). Violates structural opacity of substrate handles; a `BoolPortRef` must come from `bool_port_of`, not from a user-authored integer.
   - **Lowering-time synthesis without source anchor** (e.g., lowering fabricates a bool witness when the user didn't author one). Violates `feedback_declare_facts_dont_derive`.
   - **Any path that produces a `BoolPortRef` without going through `bool_port_of`.** `BoolPortRef` has no unsafe constructor; this rejection is enforced by the Track 9 primitive's own shape.

4. **Fail-closed span discipline.** The `Diagnostic::BranchConditionNotBool` span points at the source expression of the condition, not at the BranchArm or WorkflowEffect wrapper. User sees "this specific condition is not Bool-typed" with a pointer to the exact surface form that needs fixing. Part 2's Acceptance item 4 names this explicitly as a Part 2 review gate — a diagnostic pointing at the wrong span (or absent entirely) fails Part 2 review.

**Why the expression-based form (and not a dedicated port-reference syntax).** Per `feedback_std_over_patterns` (the same rationale that chose `data` over `WorkflowDeclaration` for the host): reuse existing surface. The user already writes Bool-typed expressions for every other Bool-consuming slot (`if <expr> then ... else ...`, refinements `where <expr>`, modifier predicates). Adding a DB-18-specific syntax for branch conditions would enumerate a special case. The value-expression → sub-DAG → port → `bool_port_of` path is zero-new-syntax.

### Consumer contract — Stage 2b (LinearEffect-only scope)

Stage 2b's `analyze_workflow` lens walks a `WorkflowEffect` value, dispatching per variant. Only `LinearEffect` yields a `CompositionVerdict`; the other three variants produce explicit diagnostics. Pseudocode shape (not the Part 2 implementation):

```dag
// Caller resolves `workflow_decl` to a WorkflowEffect value via the
// authority-site accessor (§"Authority site for WorkflowEffect"),
// then dispatches per variant. Both entry points coexist: the
// DeclarationId form is natural for "walk the DAG and analyze all
// workflow-typed data declarations"; the direct-WorkflowEffect form
// is natural for unit tests and sub-workflow recursion.
fn analyze_workflow(d: Dag, workflow_decl: DeclarationId) -> WorkflowIdempotencyReport {
  let workflow = read_workflow_value(d, workflow_decl)   // standard data-value accessor
  analyze_workflow_value(d, workflow)
}

fn analyze_workflow_value(d: Dag, workflow: WorkflowEffect) -> WorkflowIdempotencyReport {
  match workflow {
    LinearEffect { ops } => {
      // Delegate to the post-PR-#529 algebra. `compose_effects`
      // returns `CompositionVerdict` directly; for ops = [] it
      // returns IdempotentComposition (the monoidal identity).
      let verdict = compose_effects(ops)
      WorkflowIdempotencyReport {
        verdict: verdict,
        diagnostic: match verdict {
          IdempotentComposition => None
          BrokenBy { first_breaker } =>
            Some(Diagnostic {
              kind: WorkflowNonIdempotent { breaker: first_breaker }
              // ... span + fix info
            })
        }
      }
    }
    BranchEffect =>
      // Stage 2d owns branch composition (max-path / lattice meet
      // over arms). Stage 2b emits a diagnostic that names the
      // downstream consumer so the user (and CI) can tell the
      // difference between "this is broken" and "this isn't scope".
      report_unsupported_variant(
        variant_name: "BranchEffect",
        downstream_stage: "Lane 2 Stage 2d (branch-wise composition — symbolic cost)",
        reason: "branch-wise effect composition is not modeled at Stage 2b"
      )
    LoopEffect =>
      report_unsupported_variant(
        variant_name: "LoopEffect",
        downstream_stage: "Lane 2 Stage 2d (fixpoint bound + body convergence)",
        reason: "iterated effect composition requires body idempotency + bound evidence"
      )
    ParallelEffect =>
      report_unsupported_variant(
        variant_name: "ParallelEffect",
        downstream_stage: "Lane 2 Stage 2e (parallelism-as-lens + commutativity witness)",
        reason: "concurrent effect composition requires algebraic commutativity evidence"
      )
  }
}
```

`WorkflowIdempotencyReport`'s exact shape after DB-18 + PR #529 reconcile is left to Part 2 design — the current `{ idempotent: Bool, breaking_op: String?, evidence_chain: List<OperationEffect>, diagnostic: Diagnostic? }` shape in `lane2-compile-time-proofs.md` §Stage 2b is a pre-#529 placeholder and must be replaced with a structural carrier projecting through `CompositionVerdict` (per PR #529's pre-start-gate update; see §"Open questions" below for the exact reconciliation).

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

This is deliberate. PR #529's R3 removed `ComposedEffect { operations, verdict }` because pairing the input walk and the output verdict in a single record admitted correlated-fields incoherence. DB-18 preserves that lesson: the input structure lives on one axis (`WorkflowEffect`), the output verdict lives on another axis (`CompositionVerdict`), and nothing pairs them. Stage 2b callers that want "the walk AND the verdict" keep their own `WorkflowEffect` in scope and read the lens's `CompositionVerdict` output — two facts kept separate at their natural sites, with no correlation for the type system to enforce.

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

- `type WorkflowEffect` (new coproduct) and `type BranchArm` (new record) in `src/v3/std/effects.dag`.
- `type BoolPortRef` (new Track 9 typed opaque handle) with `fn bool_port_of` and `fn port_of` in `src/v3/std/substrate.dag`. Third Track 9 primitive after `ParamRef` / `TransformRef`.
- Authority-site convention: user-declared `data` typed `WorkflowEffect` is the host. No substrate-addition needed — the `data` declaration surface already exists.

---

## Worked example — workflow shapes under DB-18

Three example workflows, their `WorkflowEffect` encoding, and the Stage 2b verdict under this design.

**Workflow A — GCP linear chain (Stage 2b golden path).**

```
Secret.upsert(key=secret_id, value=...)   // UpsertEffect
STS.exchange(token=...)                    // ReadEffect (idempotent on state)
IAM.grant(role=...)                         // UpsertEffect
```

Encoded as:

```
WorkflowEffect::LinearEffect {
  ops: [
    OperationEffect { operation_name: "Secret.upsert", shape: IsIdempotent(UpsertEffect{..}) }
    OperationEffect { operation_name: "STS.exchange", shape: IsIdempotent(ReadEffect) }
    OperationEffect { operation_name: "IAM.grant", shape: IsIdempotent(UpsertEffect{..}) }
  ]
}
```

Stage 2b lens dispatches on `LinearEffect`, calls `compose_effects(ops)`, receives `CompositionVerdict::IdempotentComposition`. Report green. ✓ — mirrors the Stage 2b fixture from `lane2-compile-time-proofs.md` §Stage 2b Acceptance.

**Workflow B — linear chain with terminal audit log (Stage 2b red).**

```
Secret.upsert(...)                          // UpsertEffect (idempotent)
STS.exchange(...)                           // ReadEffect (idempotent)
IAM.grant(...)                              // UpsertEffect (idempotent)
AuditLog.append(event=...)                  // AppendEffect (breaking)
```

Encoded as `LinearEffect { ops: [upsert, exchange, grant, append] }`. Stage 2b calls `compose_effects`, receives `CompositionVerdict::BrokenBy { first_breaker: BreakingOperation { operation_name: "AuditLog.append", shape: IsBreaking(AppendEffect) } }`. Report red with diagnostic naming `AuditLog.append` as the breaker. ✓ — mirrors `lane2-compile-time-proofs.md` §Stage 2b Acceptance fixture 2.

**Workflow C — retry with nested branch (Stage 2b diagnostic).**

```
// port_auth_ok and port_auth_failed are PortId values authored by
// lowering. Each is wrapped via bool_port_of(dag, port) before
// landing in BranchArm.condition; a non-Bool port would surface a
// Diagnostic::BranchConditionNotBool on the lowering path and
// prevent BranchArm construction.
LoopEffect {
  body: BranchEffect {
    arms: [
      BranchArm {
        condition: bool_port_of(d, port_auth_ok).unwrap()     // Bool witness obtained
        body: LinearEffect { ops: [Secret.get] }
      }
      BranchArm {
        condition: bool_port_of(d, port_auth_failed).unwrap()
        body: LinearEffect { ops: [STS.exchange, Secret.get] }
      }
    ]
  }
}
```

Stage 2b dispatches on `LoopEffect`, emits diagnostic: *"`LoopEffect` encountered at workflow root; iterated effect composition requires body idempotency + bound evidence — consumed by Lane 2 Stage 2d (fixpoint bound + body convergence)."* No verdict. ✓ — the diagnostic path from §"Consumer contract" above; the user knows *why* Stage 2b did not verdict and *which stage will*.

---

## Acceptance

Part 1 (this design doc, no code) is accepted when the design locks:

1. `WorkflowEffect` is a four-variant coproduct (`LinearEffect | BranchEffect | LoopEffect | ParallelEffect`) with the payload shapes in §"Substrate changes." Payload shapes are locked for Part 1 scope; additive-extension fields graduate from §Open questions as downstream stages bind (e.g., `LoopEffect.bound` when Stage 2d consumes, `ParallelEffect.commutativity` when Stage 2e consumes). The Part 1 lock is the set of VARIANTS (four, exactly) and the MANDATORY fields of each variant (listed in §"Substrate changes"); optional additive fields added by later stages do not regress this lock.
2. `BranchArm { condition: BoolPortRef, body: WorkflowEffect }` is the sole structural distinction between `BranchEffect` and `ParallelEffect` payloads. `BoolPortRef` is a Track 9-style typed opaque handle whose sole constructor is `bool_port_of(dag: Dag, port: PortId) -> BoolPortRef?` — raw-literal construction of a `BranchArm` around a non-Bool port is not representable at the type level. The Q4 Pattern-2 distinction is carried by the typed witness, not by constructor discipline.
3. Q1–Q6 substrate-principle audit is stamped in-doc; Q4 dissolution receipt is stamped in-doc. Q5 single-authority is resolved by the §"Authority site for WorkflowEffect" section (not deferred).
4. `LinearEffect` is the Stage 2b consumer; the other three variants produce `Diagnostic` via `report_unsupported_variant` with the downstream stage name. Empty `LinearEffect.ops` is the monoidal identity, consumed by `compose_effects([])` = `IdempotentComposition`.
5. `CompositionVerdict` and `WorkflowEffect` coexist on orthogonal axes — no enclosing record pairs them; `LinearEffect`'s dispatch is the sole edge between them.
6. `WorkflowEffect`'s authority site is locked to user-declared `data` declarations typed `WorkflowEffect` (§"Authority site for WorkflowEffect"). No sidecar table; no parallel hosting; no deferred host-selection decision.
7. The source-to-handle contract for `BranchArm.condition` is locked (§"Source-to-handle contract"). Condition is an ordinary Bool-typed expression; lowering runs the standard expression → sub-DAG → port → `bool_port_of` path; fail-closed `Diagnostic::BranchConditionNotBool` on `None`. Named-port lookup, raw-id literals, and source-anchorless synthesis are rejected paths, not Part 2 judgment calls.
8. Part 2 pre-start gate: PR #529 merged 2026-04-18 (`8c7e7acdd`), clearing the gate. Part 2 dispatch is structurally unblocked.

Part 2 (follow-up implementation PR) must satisfy:

1. `src/v3/std/effects.dag` declares `type WorkflowEffect` and `type BranchArm` per the substrate-changes section (with `LinearEffect.ops: List<OperationEffect>`, `BranchArm.condition: BoolPortRef`).
2. `src/v3/std/substrate.dag` (or equivalent Track 9 primitive site) declares `type BoolPortRef` + `fn bool_port_of(dag, port) -> BoolPortRef?` + `fn port_of(bool_ref) -> PortId` per §"Add `BoolPortRef` as a Track 9-style substrate integrity primitive." Primitive graduation trigger: DB-18 BranchArm is the first consumer.
3. Rust mirror in `src/v3/compiler/src/dag.rs` (or equivalent sidecar file) declares `WorkflowEffect`, `BranchArm`, and `BoolPortRef` as `enum` and `struct`s; reflection-invariant test (`m2_field_access_binding_test.rs`-style) locks the two sides together.
4. **Fail-closed on `bool_port_of` returning `None`, with source-span precision.** Every lowering site that calls `bool_port_of` must emit `Diagnostic::BranchConditionNotBool { port, actual_type, span }` on `None` and must NOT construct a `BranchArm` with that port via any alternative path. The `span` must point at the source-level condition expression (not the surrounding BranchArm or WorkflowEffect), per §"Source-to-handle contract" point 4. Silent `None` absorption is a Part 2 review gate — reject any PR that pattern-matches `bool_port_of(...)` result without a Diagnostic emission branch, or whose diagnostic span points at the wrong surface form.
4b. **Single source-to-handle path.** The lowering code that produces a `BranchArm.condition` value must go through exactly one path: expression → sub-DAG → output-port → `bool_port_of`. Part 2 review rejects any PR that introduces (a) a named-port lookup helper for branch conditions, (b) a raw `NodeId` / `PortId` literal parse at the BranchArm-condition site, or (c) a synthesized bool witness without a source-level expression anchor. Per §"Source-to-handle contract" point 3.
5. `src/v3/lenses/idempotency.dag` (new) declares `analyze_workflow(d: Dag, workflow_decl: DeclarationId) -> WorkflowIdempotencyReport` (resolving the declaration's value via the data-declaration accessor) and `analyze_workflow_value(d: Dag, workflow: WorkflowEffect) -> WorkflowIdempotencyReport`, dispatching per variant. `LinearEffect` calls `compose_effects`; the other three emit `Diagnostic` via `report_unsupported_variant`.
6. `WorkflowIdempotencyReport` replaces the pre-#529 `idempotent: Bool + breaking_op: String?` draft shape with a structural carrier projecting through `CompositionVerdict` (per PR #529's pre-start-gate update to `lane2-compile-time-proofs.md` §Stage 2b). Exact shape resolved at Part 2 design (see Open question §2).
7. Stage 2b acceptance fixtures from `lane2-compile-time-proofs.md` §Stage 2b (three fixtures: GCP linear green, terminal AppendEffect red, POST-create keyless failure) pass against the lens-over-`WorkflowEffect` implementation.
8. New fixtures exercising the diagnostic paths: one `BranchEffect` workflow, one `LoopEffect` workflow, one `ParallelEffect` workflow — each producing the documented diagnostic with the named downstream stage. Additionally: one fixture exercising the fail-closed path on `bool_port_of` — a malformed `data` declaration attempting to use a non-Bool port as a branch condition must surface `Diagnostic::BranchConditionNotBool`, not silently construct an invalid `BranchArm`.
9. New fixture exercising the empty-`LinearEffect` case: a `LinearEffect { ops: [] }` workflow produces `CompositionVerdict::IdempotentComposition` (the monoidal identity).
10. `src/v3/ROADMAP.md` Lane 2 Stage 2b row updates to `✅ Shipped (DB-18 + lens)` with links to both design docs.

---

## Pre-start gate for Part 2 implementation

**Gate status: CLEARED.** PR #529 (ComposedEffect reshape, session `warm-newt-750`) merged 2026-04-18 as commit `8c7e7acdd`. Part 2 dispatch is structurally unblocked.

**Original gate rationale (preserved for record):** DB-18's `LinearEffect` consumer delegates to `compose_effects(List<OperationEffect>) -> CompositionVerdict`. That signature is defined by PR #529's R3. Starting Part 2 against the pre-#529 `compose_effects(...) -> ComposedEffect` signature would have locked DB-18 against a shape being removed — the lens would either (a) build against the doomed `ComposedEffect` and need immediate rework, or (b) anticipate the R3 shape and diverge from main until #529 landed. Both paths burned throughput for no signal. The gate prevented that divergence; #529's merge cleared it.

**STOP-AND-ESCALATE rule for Part 2 dispatch:**

If Part 2 implementation discovers any of the following, HALT and report to director chat rather than patching forward:

- `WorkflowEffect`'s 4-variant shape is insufficient — e.g., a real workflow fixture requires a fifth variant or a variant payload reshape. DB-18 locks the shape; reshape is a DB revision, not an in-flight patch.
- `BoolPortRef` does not distinguish BranchEffect from ParallelEffect structurally in practice (Q4 receipt regresses), OR the `bool_port_of` fail-closed semantics require an escape hatch for some legitimate lowering case. Same rule — reshape the substrate only through a DB revision.
- Stage 2b's `analyze_workflow` / `analyze_workflow_value` signature needs to return something other than `WorkflowIdempotencyReport` (e.g., `List<Diagnostic>` for multi-diagnostic workflows). This is an API refinement, possibly an open question for director-chat clearance; do not silently change the signature.
- The authority-site decision (user-declared `data` typed `WorkflowEffect`) turns out to be insufficient — e.g., a real fixture needs WorkflowEffect hosted somewhere `data` cannot reach. DB-18 locks the authority site; changing it is a DB revision, not an in-flight patch.
- `bool_port_of` returning `None` is silently absorbed at any lowering site (no `Diagnostic::BranchConditionNotBool` emission). This is a C-8 violation — Part 2 review must reject it.

The director chat owns the call on each of these. Silent in-flight patches destroy the structural-finding signal the pre-clearance exists to capture (per `phase-plan-2026-04-18.md` §7 "Structural-finding escalation rule").

---

## Open questions

1. **Does `ParallelEffect` need a commutativity witness field in Part 2?** The Pattern 3 algebraic-form analysis argues `ParallelEffect` is a concurrent-product (⊗) requiring algebraic commutativity on the target state. Part 1 does NOT require the witness in the carrier because Stage 2b emits a diagnostic on `ParallelEffect` (no verdict computed). When Stage 2e's parallelism lens binds as a real consumer, it will either (a) extend `ParallelEffect` with `commutativity: AlgebraRef` (or a named-alternative witness carrier) as an additive payload extension, or (b) prove that commutativity is derivable from the workflow's op-level algebra without a substrate field. Decision deferred to Stage 2e design pre-clearance — NOT patched forward in Part 2 implementation. This is an additive extension consistent with the Acceptance-item-1 language (locked VARIANTS + MANDATORY fields; optional additive fields are in-scope for later stages).

2. **Exact post-#529 `WorkflowIdempotencyReport` shape.** The pre-#529 draft shape in `lane2-compile-time-proofs.md` §Stage 2b is `{ idempotent: Bool, breaking_op: String?, evidence_chain: List<OperationEffect>, diagnostic: Diagnostic? }`. PR #529's pre-start-gate update says this must be replaced with a structural carrier projecting through `CompositionVerdict`. Candidate post-DB-18 shape: `{ verdict: CompositionVerdict, workflow_shape: WorkflowEffect, diagnostic: Diagnostic? }` — but this pairs the algebra output with the input structure in a single record, which is exactly the correlated-fields pattern PR #529 R3 rejected. Alternative: return `CompositionVerdict` OR `Diagnostic` directly from `analyze_workflow_value` (no outer record); the lens's caller pairs it with the `WorkflowEffect` they already hold. The R1 claude-review (2026-04-18) leans to the alternative (no outer record); R2 inherits that lean. Resolve at Part 2 design.

3. **Does `LoopEffect.body` need a bound carrier in Part 1?** Substrate has `LoopBound = Cardinality { count: PortId } | Descent { cluster: ClusterId }` post-DB-9 R2.1. Stage 2d (symbolic cost) will almost certainly need the bound to compute recursion depth. Part 1 does NOT include a bound field because Stage 2b emits a diagnostic on `LoopEffect` without reading bound info. Part 2 / Stage 2d can add `bound: LoopBound` as an additive extension when its consumer binds — consistent with Acceptance item 1's explicit additive-extension clause (which was tightened in R2 to prevent the doc-coherence gap ChatGPT flagged). If the director chat prefers a single shape-lock including the bound, a R3 revision can add `LoopEffect { body: WorkflowEffect, bound: LoopBound }`; decision is a scope judgment, not a correctness one.

4. **Does `LinearEffect.ops` need to be `List<WorkflowEffect>` instead of `List<OperationEffect>` to allow mixed nesting?** The worked example C demonstrates the answer: a linear sequence with a branch in the middle lifts to a `BranchEffect` of two linear paths (distributive over composition). No `LinearEffect` with a `WorkflowEffect` in the middle is needed; the structural canonical form is always a variant at the outermost shape. Locked per Q6 (no representation duality) — not open.

5. ~~**Does the substrate need a `WorkflowDeclaration` record to host `WorkflowEffect`?**~~ **CLOSED in R2.** `WorkflowEffect` values live exclusively on user-declared `data` declarations typed `WorkflowEffect`. See §"Authority site for `WorkflowEffect`" above. No new declaration kind needed; `data` is the host. This closes the Q5 single-authority question at Part 1 scope rather than deferring to Part 2.

---

## Rejected alternatives

**R0 — lens-only, derive `WorkflowEffect` from DAG shape at walk time.** Rejected in §Revision history. Core issue: re-derives workflow structure heuristically from Bind chain / Branch / Loop behavior, violating `feedback_lenses_not_passes` ("heuristic = missing physics"). A workflow's control-flow shape is a fact about the author's intent, not something the compiler should re-infer per-lens.

**R-alt-A — single `WorkflowEffect` coproduct with a kind tag + optional payload fields.** Shape: `WorkflowEffect { kind: Linear | Branch | Loop | Parallel, ops: Option<List<OperationEffect>>, arms: Option<List<WorkflowEffect>>, body: Option<WorkflowEffect> }`. Rejected because it admits illegal state combinations (`kind: Linear` with `arms: Some(...)`) — the classic `Bool + Option<T>` pattern `feedback_state_space_vs_behavioral_invariants` names. The four-variant coproduct is the state-space-sound alternative.

**R-alt-B — hierarchical inheritance: `WorkflowEffect` is-a `List<OperationEffect>` with optional control-flow overlay.** Rejected because it pre-commits to linear-as-base and overlays branches as a non-structural layer. Stage 2d's branch-wise composition would have to peel back the overlay — the overlay IS the structure. Better to encode control-flow structurally from the start (R1).

**R-alt-C — merge `BranchEffect` and `ParallelEffect` into `ChoiceEffect { kind: Exclusive | Concurrent, arms: NonSingletonList<WorkflowEffect> }`.** Rejected because it re-introduces the `kind + discriminator` pattern that Q4's dissolution receipt is supposed to eliminate. The whole point of the four-variant coproduct is that each variant traces to a distinct categorical operation; compressing two of them under a flag re-creates the compression the coproduct solves.

**R-alt-D — drop `ParallelEffect`; add it later via DB-N when Stage 2e ships.** Viable; the user's directive explicitly asked for 4 variants, so this is not the chosen path. If the director chat later prefers a 3-variant initial shape, that is an additive refinement (drop `ParallelEffect`; Stage 2e's DB adds it when it ships) and does not regress the Q4 receipt for the remaining three variants. Noted for director-chat discussion if the 4-variant commit proves too heavy for Part 1.

**R-alt-E — raw `PortId` on `BranchArm.condition` with constructor-level validation only (the R1 shape).** Rejected in R2 after the PR #531 ChatGPT + codex reviews. The raw-`PortId` shape relies on `branch_arm_of(port, body) -> Option<BranchArm>` to validate "port is Bool-typed" at construction, and requires every lowering site to obey the constructor-only-no-raw-literals convention. That is API-level enforcement: a contributor writing `BranchArm { condition: <non_bool_port>, body: ... }` directly produces a type-checkable value whose condition port is invalid. The Q4 Pattern-2 receipt leans on the condition field as the load-bearing structural distinction between `BranchEffect` and `ParallelEffect`; if that field admits invalid ports, the receipt degrades to convention-level enforcement and the coproduct dissolution argument collapses. R2 replaces this with a typed-witness (`BoolPortRef`) whose field type itself rejects invalid ports — the invariant is carried by the type, not by the constructor. Additional benefit: the `BoolPortRef` primitive is reusable for other Bool-typed-port consumers (e.g., the computation substrate's Branch behavior condition slot) as a Track 9 primitive graduation.

**R-alt-F — host `WorkflowEffect` on a new `WorkflowDeclaration` kind.** Rejected in R2 in favor of user-declared `data` typed `WorkflowEffect`. See §"Authority site for `WorkflowEffect`" for the full argument. Short version: a new declaration kind adds substrate concept without capability gain — `data` already provides name + type + value hosting, and the type annotation (`WorkflowEffect`) carries the workflow-specificity. Per `feedback_std_over_patterns`, reuse the generic surface rather than enumerating special cases.

**R-alt-G — defer authority-site selection to Part 2.** Rejected in R2 after the PR #531 codex review BLOCKING finding. Leaving "Stage 2b can read `WorkflowEffect` values from any declaration slot the user happens to declare them on" in the Open Questions is a Q5 single-authority violation: "any slot" is not a single authority. R2 locks the authority site in §"Authority site for `WorkflowEffect`" — user-declared `data` declarations typed `WorkflowEffect`, period.

---

## Cross-references

- `feedback_coproduct_dissolution` — Q4 receipt (§"Dissolution receipt" above cites patterns 1–4).
- `feedback_substrate_principle_audit` — Q1–Q6 (§"Substrate principle audit" cites all six).
- `feedback_state_space_vs_behavioral_invariants` — cardinality invariants via `NonEmptyList` / `NonSingletonList`; rejected alternative R-alt-A.
- `feedback_lenses_not_passes` — rejected alternative R0 (lens-level re-derivation).
- `feedback_no_metadata_markers` — `BranchArm.condition: BoolPortRef` is a typed opaque handle, not a string marker or raw index.
- `feedback_fail_closed_discipline` — `report_unsupported_variant` produces a `Diagnostic`, not a silent skip; `bool_port_of` returning `None` must be surfaced as `Diagnostic::BranchConditionNotBool` at the caller (Part 2 Acceptance item 4).
- `feedback_std_over_patterns` — authority-site decision reuses `data` rather than inventing `WorkflowDeclaration` (R-alt-F rejection).
- DB-9 R2.1 (`design-mutual-recursion-lowering.md`) — worked example of the six-question audit; mirrors DB-18's format. Also: the `ParamRef` / `TransformRef` Track 9 primitive pattern that `BoolPortRef` follows (R2 third primitive graduation).
- DB-16 (`design-db16-refined-generic-substitution.md`) — worked example of Part 1 design / Part 2 impl split; mirrors DB-18's scope discipline.
- PR #529 (`design-composed-effect-reshape.md`) — `CompositionVerdict` authority that DB-18's `LinearEffect` path delegates to. Merged 2026-04-18 as `8c7e7acdd`.
- ROADMAP (`../ROADMAP.md` and `../src/v3/ROADMAP.md`) — Lane 2 Stage 2b entry; this DB advances the row from planned lens-only to substrate-carrier + lens per director escalation. Track 9 line 763-772 references for primitive-graduation discipline.

---

End of DB-18 Part 1.
