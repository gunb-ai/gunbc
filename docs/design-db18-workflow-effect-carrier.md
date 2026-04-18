> Part of: [lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md) (Lane 2 Stage 2b) | Companion: [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (PR #529) | Unblocks: Lane 2 Stage 2b implementation (workflow idempotency lens); informs Lane 2 Stages 2d / 2e / 2f (downstream composition over workflow structure)

# Design DB-18 — `WorkflowEffect` substrate carrier (Stage 2b input structure)

**Design blocker:** DB-18 (new substrate carrier `WorkflowEffect` — a four-variant coproduct describing workflow control-flow shape for effect-algebra composition)
**Consumers:** Lane 2 Stage 2b workflow idempotency lens (initial consumer; `LinearEffect`-only scope). Downstream: Stage 2d symbolic cost, Stage 2e parallelism-as-lens, Stage 2f user-declared dimensions.
**Status:** R2 — responding to PR #531 ChatGPT review's three blocking concerns on fail-closed construction, type-level witness for the BranchEffect/ParallelEffect structural distinction, and LinearEffect carrier coverage of the monoidal identity. Part 1 design-only — implementation and tests ship in the follow-up Part 2 PR.
**Companion:** [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (PR #529, landed 2026-04-18 as `8c7e7acdd`) reshapes the *output* side of the effect algebra: `ComposedEffect` is removed; `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` is the post-reshape algebra. DB-18 adds the *input* side: a typed carrier describing the workflow's control-flow structure above `List<OperationEffect>`. The two live on orthogonal axes — `WorkflowEffect` is the input structure the lens walks; `CompositionVerdict` is the output the algebra returns. They coexist; neither replaces the other.

---

## Summary

Stage 2b's pre-DB-18 design (`lane2-compile-time-proofs.md` §Stage 2b) assumed a workflow is a linear `List<OperationEffect>` — `compose_effects` walks the list, composes effect shapes, emits a verdict. That shape is correct for the narrow case (GCP Secret Manager upsert → STS Exchange → IAM grant, all linear) but the escalation clause in the same section already names the extension: *"if workflow structure isn't representable cleanly — e.g., control flow in a pipeline doesn't map to a linear `List<OperationEffect>` — surface. Don't stretch `compose_effects` to handle branches silently; the algebra needs to reflect branch-wise composition, which is a legitimate design extension."*

DB-18 is that legitimate design extension, moved up-front rather than deferred. The substrate gains **one new reflected type** (`WorkflowEffect`) with **one helper record** (`BranchArm`); the computation substrate is untouched (no new `Behavior` variant, no `LoopBound` change); the effect algebra (`compose_effects`, `CompositionVerdict`) is untouched. Stage 2b's lens matches on `LinearEffect` only and emits explicit diagnostics on the other three variants, each naming the downstream stage that will consume it. The diagnostic is a first-class `Diagnostic`, not a silent skip — the fail-closed contract from INVARIANTS §C-8 applies.

`WorkflowEffect` is the input-structure carrier. `CompositionVerdict` (PR #529) is the output-verdict carrier. They meet at exactly one edge: Stage 2b's lens projects `LinearEffect { ops }` through `compose_effects(ops)` to obtain a `CompositionVerdict`; other `WorkflowEffect` variants produce diagnostics without invoking the algebra.

---

## Revision history

- **R0** (rejected; pre-dispatch): "Stage 2b is lens-only — model workflows as `List<OperationEffect>` produced by the lens at walk time from the DAG's Bind chain." Rejected because it re-derives workflow structure heuristically from DAG shape at lens time, violating `feedback_lenses_not_passes` ("heuristic = missing physics"). A workflow's control-flow shape is an input fact to the algebra, not something to reconstruct.
- **R1** (superseded by R2): Workflow control-flow shape lifted into the substrate as a first-class four-variant carrier `WorkflowEffect`; `BranchArm { condition: PortId, body: WorkflowEffect }` as the Pattern-2 distinguisher between `BranchEffect` and `ParallelEffect`; `LinearEffect { ops: NonEmptyList<OperationEffect> }`. ChatGPT review (2026-04-18) flagged three blocking structural concerns: (a) `branch_arm_of(...) -> Option<BranchArm>` weakens fail-closed — a non-Bool condition port is a user-reachable modeling error, not a benign "absent" case; (b) raw `PortId` admits the wrong port kind unless every producer obeys convention, keeping the BranchEffect/ParallelEffect distinction behavioral rather than type-level (directly regresses the Q4 receipt it claims to carry); (c) excluding the empty word from `LinearEffect.ops` creates asymmetry with `compose_effects(List<OperationEffect>)` — the monoidal identity is a real workflow shape (no-op paths, empty branch arms) with no honest representation.
- **R2** (current): Response to the three R1 blocking items. (a+b) Introduce `BoolPortRef` as a Track 9-style typed opaque handle whose sole constructor `bool_port_of(dag: Dag, port: PortId) -> BoolPortRef?` fails closed at construction — a non-Bool port returns `None` and the lowering caller emits a `Diagnostic` per C-8. `BranchArm.condition` is typed `BoolPortRef` (not raw `PortId`), so non-Bool conditions are unrepresentable at the type level; raw-literal construction of `BranchArm` cannot produce an invalid arm because `BoolPortRef` has no unsafe constructor. The Q4 Pattern-2 dissolution receipt is now carried by the *typed witness* rather than by constructor convention. (c) Relax `LinearEffect.ops` from `NonEmptyList<OperationEffect>` to `List<OperationEffect>`. Empty `LinearEffect` is the monoidal identity — `compose_effects([])` already returns `IdempotentComposition` (the identity verdict); rejecting it would make the carrier narrower than the algebra. This is also the canonical form for no-op paths (e.g., a branch arm whose body is "do nothing").

---

## Constraints (non-negotiable)

1. **`CompositionVerdict` authority preserved.** Post-PR #529, `compose_effects(List<OperationEffect>) -> CompositionVerdict` is the sole effect-algebra output. DB-18 does not introduce a parallel verdict carrier. `LinearEffect`'s lens delegates to `compose_effects`; other variants do not compose at Stage 2b.
2. **Five-behavior computation substrate untouched.** `Behavior = Value | Transform | Branch | Loop | Bind` remains at five variants. `WorkflowEffect` lives in the *type substrate* (declarations), not the computation substrate. Parallel lens tests (`test_thesis_five_behavior_variants`-style) stay green by construction.
3. **Bounded kernel invariant.** `WorkflowEffect` is recursive through its own variant fields; the recursion terminates at `LinearEffect.ops: NonEmptyList<OperationEffect>` (no further `WorkflowEffect` children). Finite by construction because the substrate is a DAG (no cycles in `WorkflowEffect`-typed edges).
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
// LinearEffect terminates the recursion at a non-empty list of
// OperationEffect values — the same carrier compose_effects consumes
// post-PR-#529.
type WorkflowEffect
  = LinearEffect { ops: NonEmptyList<OperationEffect> }
  | BranchEffect { arms: NonSingletonList<BranchArm> }
  | LoopEffect { body: WorkflowEffect }
  | ParallelEffect { branches: NonSingletonList<WorkflowEffect> }

// 🟢 TERMINAL. A single arm of a BranchEffect — the condition port
// witnessing "this arm is taken" plus the workflow executed when the
// condition holds. Separating condition from workflow makes the
// payload structurally distinct from ParallelEffect's
// NonSingletonList<WorkflowEffect>, so the Q4 dissolution receipt
// passes at the payload level rather than requiring a separate
// discriminator tag.
//
// `condition: PortId` — a typed substrate handle into the computation
// DAG. The port is the producer of the Bool-typed value the branch
// dispatches on. Constructor-validated: `branch_arm_of(port, body)`
// returns `None` for non-Bool-typed ports, so a malformed arm is
// unrepresentable.
type BranchArm {
  condition: PortId
  body: WorkflowEffect
}
```

**Why this belongs in `std/effects.dag` rather than `std/substrate.dag`.** `WorkflowEffect` is a concept of the effect algebra — it describes input shapes that `compose_effects` (and its downstream peers) dispatch over. The substrate's own declarations (`Dag`, `Declaration`, `Behavior`, `LoopBound`, `Cluster`) describe the *computation and type substrate*; the effect algebra is a lens-writable layer above that substrate. The existing file already hosts `OperationEffect`, `EffectShape`, `IdempotencyEvidence`, `CompositionVerdict` — all effect-algebra types. `WorkflowEffect` joins that family.

**No reflected `type Dag` changes.** `WorkflowEffect` values live inside user code (as fields of workflow declarations) and inside lens output, not as a sidecar table on `Dag`. The authority for "what workflow does this call site model?" is the user's declaration; the lens walks declarations and reads their `WorkflowEffect`-typed fields. No new `Dag.workflows: List<...>` field is needed (cf. DB-9 R2.1 `Dag.clusters` sidecar which is necessary because clusters span multiple declarations; a `WorkflowEffect` is a single declaration-local value).

**No Rust mirror yet.** Because Part 1 is design-only, no `src/v3/compiler/src/dag.rs` change is proposed here. Part 2 mirrors `WorkflowEffect` and `BranchArm` in `dag.rs` under the same reflection-invariant check (`m2_field_access_binding_test.rs`) that DB-9 R2.1 used for `Cluster` / `MemberDescent` / `IntraClusterCall`. The single-authority discipline from DB-9 R2.1 carries over: lowering writes `WorkflowEffect` values; lenses and inference are pure readers.

### Dissolution receipt — `WorkflowEffect` coproduct (four-pattern check)

Per `feedback_coproduct_dissolution`, every new coproduct must pass the four-pattern check before being stamped `🟢 TERMINAL`. Stamped here, not deferred. The full argument is required under the user's directive ("Audit Q4 receipt required").

**Pattern 1 — Fact placement (multiple consumers, different DAG locations).** All four variants attach to the same `WorkflowEffect`-typed slot at the same DAG location — a workflow declaration's root. The variant discriminates KIND OF WORKFLOW SHAPE, not WHERE THE WORKFLOW FACT LIVES. No fact-placement compression to dissolve. ✓ does not apply.

**Pattern 2 — Variant-is-data (same shape, different label).** The four variants' payloads, audited pairwise:

| Variant A | Variant B | Payload A | Payload B | Distinct? |
|---|---|---|---|---|
| `LinearEffect` | `BranchEffect` | `NonEmptyList<OperationEffect>` | `NonSingletonList<BranchArm>` | ✓ different element types + different cardinality contract |
| `LinearEffect` | `LoopEffect` | `NonEmptyList<OperationEffect>` | `WorkflowEffect` | ✓ list vs single |
| `LinearEffect` | `ParallelEffect` | `NonEmptyList<OperationEffect>` | `NonSingletonList<WorkflowEffect>` | ✓ different element types + different cardinality contract |
| `BranchEffect` | `LoopEffect` | `NonSingletonList<BranchArm>` | `WorkflowEffect` | ✓ list vs single |
| `BranchEffect` | `ParallelEffect` | `NonSingletonList<BranchArm>` | `NonSingletonList<WorkflowEffect>` | ✓ `BranchArm` ≠ `WorkflowEffect` (BranchArm carries `condition: PortId` that ParallelEffect branches do not) |
| `LoopEffect` | `ParallelEffect` | `WorkflowEffect` | `NonSingletonList<WorkflowEffect>` | ✓ single vs list |

The critical pair is BranchEffect vs ParallelEffect — both use `NonSingletonList<...>` at the outer shape and recurse on `WorkflowEffect` at the element level. The distinction is carried by `BranchArm { condition: PortId, body: WorkflowEffect }` versus a plain `WorkflowEffect`: a branch arm is an arm-gated-by-condition, a parallel branch is an unconditioned concurrent workflow. The `condition: PortId` is the load-bearing structural fact — it encodes "which arm is taken is a function of this specific DAG port's value," which is irreducibly different from "all branches execute concurrently and converge at a join." Pattern 2's dissolution criterion ("same shape, different label") does not apply because the BranchArm wrapper carries a structurally required field ParallelEffect's branches do not. ✓ does not apply.

**Pattern 3 — Algebraic-form (traces to intro/elim of algebraic structures).** The four variants trace to four distinct categorical operations on effect algebras:

- `LinearEffect` = **monoidal composition (∘)**. `compose_effects` already witnesses this: a linear sequence of effects composes as `∘` with `ReadEffect` (identity on state) as the unit. The `CompositionVerdict` is the algebra's output for this case. The carrier `NonEmptyList<OperationEffect>` is the free monoid on `OperationEffect` less the empty word.
- `BranchEffect` = **coproduct (∨) / lattice meet over arms**. Exactly one arm executes at runtime; the verdict must hold for whichever arm is taken. At compile time this is a `∨`-over-arms: the workflow is idempotent iff every arm's workflow is idempotent. Structurally distinct from `∘` — composition order does not matter for arms that are alternatives, and the per-arm verdicts combine via lattice meet, not via monoidal multiplication.
- `LoopEffect` = **fixpoint (μ) / iteration**. The body is re-applied 0..N times (bound structure tracked separately, potentially in Part 2). Idempotency under iteration demands the body itself be idempotent (`f ∘ f = f`) — a strictly stronger condition than linear composition, because monoidal composition of two different non-idempotent effects can still converge, but iteration of one non-idempotent effect cannot.
- `ParallelEffect` = **concurrent product (⊗) / commutative composition**. Branches execute concurrently and require algebraic commutativity on the target state to compose safely. The verdict's validity depends on commutativity evidence (open question §1 below — Part 2 or Stage 2e may extend the carrier with a `commutativity` witness). Distinct from both `∘` (order-preserving) and `∨` (alternatives): `⊗` is unordered AND concurrent.

Each variant is the introduction form of a different categorical operation. There is no single algebraic operation the four dissolve into; mapping the set of four to a two-dimensional record (e.g., `{ordered, concurrent}`) does not recover the distinct verdict rules each variant demands. ✓ does not apply.

**Pattern 4 — Dimensional (flat enum hides M-dimensional record).** The proposed dimension would be `{control_flow: Linear | Branch | Loop | Parallel, ...}` — but this is just the coproduct tag renamed, with no second coordinate. Attempting a second coordinate produces a partial record where each variant has different required fields (LinearEffect needs `ops`; BranchEffect needs `arms` with conditions; LoopEffect needs `body`; ParallelEffect needs `branches`): these are different types of payload, not orthogonal coordinates on a shared record. A record shape with every payload field present-or-absent would admit illegal combinations (`control_flow: LinearEffect` with `arms: Some(...)`), which violates the Q1 cardinality-invariant check. ✓ does not apply.

**Conclusion:** `WorkflowEffect`'s 4-variant coproduct is structurally irreducible. The coproduct is the correct shape. Stamp: `🟢 TERMINAL` on `WorkflowEffect`, `🟢 TERMINAL` on `BranchArm`.

### Substrate principle audit (Q1–Q6, beyond Q4)

Per `feedback_substrate_principle_audit`, all six questions walk before greenlighting. Q4 is covered above; the other five are stamped here so downstream review verifies the audit ran rather than re-deriving each finding.

**Q1 — Cardinality invariants.** Does any variant admit `[]` where invariant says ≥1, or singletons where ≥2?

- `LinearEffect.ops: NonEmptyList<OperationEffect>` — a linear workflow with zero ops is meaningless (nothing to compose). Type rejects `[]`.
- `BranchEffect.arms: NonSingletonList<BranchArm>` — a branch with 0 or 1 arms is meaningless (0: no workflow; 1: just the arm itself, use the arm's `body` directly). Type rejects both.
- `LoopEffect.body: WorkflowEffect` — singleton (one body), never empty. Type is non-optional.
- `ParallelEffect.branches: NonSingletonList<WorkflowEffect>` — 0 branches is empty parallelism; 1 branch is just the branch itself, no concurrency. Type rejects both. ✓ cardinality invariants at the type level; no "lowering guarantees" prose required.

**Q2 — Index/handle types.** Does a raw `Int` or `NodeId` encode something with a domain restriction?

- `BranchArm.condition: PortId` is a typed substrate handle. The existing `PortId` carrier already witnesses "this is a substrate port." Construction validity ("the port is Bool-typed at the condition slot") is the `branch_arm_of(port, body) -> Option<BranchArm>` responsibility, matching the `param_of` pattern from Track 9. ✓ no raw Int/NodeId with comment-level validity.

**Q3 — Duplicated fact.** Does Field A duplicate what's derivable from Field B?

- `WorkflowEffect` variants do not duplicate any fact carried by `OperationEffect`, `EffectShape`, or `CompositionVerdict`. `WorkflowEffect` is the *input structure* above ops; the ops themselves are authored once and referenced by `LinearEffect.ops`. `CompositionVerdict` is the *output* of the algebra — DB-18 does not pre-compute a verdict on the workflow shape. ✓ no duplication.

**Q5 — Construction authority.** Are multiple call sites independently constructing the same fact?

- Single authority: lowering is the sole producer of `WorkflowEffect` values (Part 2 wires this). The lens and downstream consumers are pure readers. No parallel construction path through multiple lowering sites. `branch_arm_of` is the sole `BranchArm` constructor; no raw `BranchArm { ... }` literal should appear outside that constructor. ✓ single authority.

**Q6 — Representation duality.** Can the same fact be expressed in two structurally different shapes that comparison treats differently?

- A workflow that is structurally a sequence with a nested branch is expressed *uniquely* as the outermost variant's shape. "Linear with branch in the middle" cannot be represented as `LinearEffect` (because `LinearEffect.ops` is `NonEmptyList<OperationEffect>`, not `NonEmptyList<WorkflowEffect>`); it must be lifted to `BranchEffect { arms: [LinearEffect{A∪B∪D}, LinearEffect{A∪C∪D}] }` — i.e., a branch between two linear paths. This is a structurally-unique canonical form, not a choice between two equivalent representations. ✓ no representation duality.

All six audit questions stamp cleanly. The full six-question audit is recorded in this section for reviewer-bot verification rather than re-derivation at PR-review time.

### Consumer contract — Stage 2b (LinearEffect-only scope)

Stage 2b's `analyze_workflow` lens walks a `WorkflowEffect` value, dispatching per variant. Only `LinearEffect` yields a `CompositionVerdict`; the other three variants produce explicit diagnostics. Pseudocode shape (not the Part 2 implementation):

```dag
fn analyze_workflow(d: Dag, workflow: WorkflowEffect) -> WorkflowIdempotencyReport {
  match workflow {
    LinearEffect { ops } => {
      // Delegate to the post-PR-#529 algebra. `compose_effects`
      // returns `CompositionVerdict` directly.
      let verdict = compose_effects(ops |> to_list)
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
- The reflected `Dag` record. Untouched. No new sidecar table.
- `WorkflowEffectConcern` (existing record at `src/v3/std/effects.dag:669–673` post-PR #529; was `:565–569` pre-#529). This is a diagnostic-construction helper, not an input carrier; remains as-is. Part 2 may or may not project through it for the `LinearEffect` diagnostic construction.
- V2 `dsl/std/effects.dag`. V3-only, per the same scope discipline PR #529 applied.

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
  ops: NonEmptyList {
    first: OperationEffect { operation_name: "Secret.upsert", shape: IsIdempotent(UpsertEffect{..}) }
    rest: [
      OperationEffect { operation_name: "STS.exchange", shape: IsIdempotent(ReadEffect) }
      OperationEffect { operation_name: "IAM.grant", shape: IsIdempotent(UpsertEffect{..}) }
    ]
  }
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
LoopEffect {
  body: BranchEffect {
    arms: [
      BranchArm { condition: <port_auth_ok>, body: LinearEffect{[Secret.get]} }
      BranchArm { condition: <port_auth_failed>, body: LinearEffect{[STS.exchange, Secret.get]} }
    ]
  }
}
```

Stage 2b dispatches on `LoopEffect`, emits diagnostic: *"`LoopEffect` encountered at workflow root; iterated effect composition requires body idempotency + bound evidence — consumed by Lane 2 Stage 2d (fixpoint bound + body convergence)."* No verdict. ✓ — the diagnostic path from §"Consumer contract" above; the user knows *why* Stage 2b did not verdict and *which stage will*.

---

## Acceptance

Part 1 (this design doc, no code) is accepted when the design locks:

1. `WorkflowEffect` is a four-variant coproduct (`LinearEffect | BranchEffect | LoopEffect | ParallelEffect`) with exactly the payload shapes in §"Substrate changes."
2. `BranchArm { condition: PortId, body: WorkflowEffect }` is the sole structural distinction between `BranchEffect` and `ParallelEffect` payloads at Part 1 scope.
3. Q1–Q6 substrate-principle audit is stamped in-doc; Q4 dissolution receipt is stamped in-doc.
4. `LinearEffect` is the Stage 2b consumer; the other three variants produce `Diagnostic` via `report_unsupported_variant` with the downstream stage name.
5. `CompositionVerdict` and `WorkflowEffect` coexist on orthogonal axes — no enclosing record pairs them; `LinearEffect`'s dispatch is the sole edge between them.
6. Part 2 pre-start gate (this section below) names PR #529 as the merge precondition.

Part 2 (follow-up implementation PR) must satisfy:

1. `src/v3/std/effects.dag` declares `type WorkflowEffect` and `type BranchArm` per the substrate-changes section.
2. Rust mirror in `src/v3/compiler/src/dag.rs` (or equivalent sidecar file) declares `WorkflowEffect` and `BranchArm` as `enum` and `struct`; reflection-invariant test (`m2_field_access_binding_test.rs`-style) locks the two sides together.
3. `src/v3/lenses/idempotency.dag` (new) declares `analyze_workflow(d: Dag, workflow: WorkflowEffect) -> WorkflowIdempotencyReport` and dispatches per variant. `LinearEffect` calls `compose_effects`; the other three emit `Diagnostic` via `report_unsupported_variant`.
4. `WorkflowIdempotencyReport` replaces the pre-#529 `idempotent: Bool + breaking_op: String?` draft shape with a structural carrier projecting through `CompositionVerdict` (per PR #529's pre-start-gate update to `lane2-compile-time-proofs.md` §Stage 2b). Exact shape resolved at Part 2 design (see Open question §2).
5. Stage 2b acceptance fixtures from `lane2-compile-time-proofs.md` §Stage 2b (three fixtures: GCP linear green, terminal AppendEffect red, POST-create keyless failure) pass against the lens-over-`WorkflowEffect` implementation.
6. New fixtures exercising the diagnostic paths: one `BranchEffect` workflow, one `LoopEffect` workflow, one `ParallelEffect` workflow — each producing the documented diagnostic with the named downstream stage.
7. `src/v3/ROADMAP.md` Lane 2 Stage 2b row updates to `✅ Shipped (DB-18 + lens)` with links to both design docs.

---

## Pre-start gate for Part 2 implementation

**PR #529 (ComposedEffect reshape, session `warm-newt-750`, branch `session/warm-newt-750`) MUST land before Part 2 coding begins.**

Reason: DB-18's `LinearEffect` consumer delegates to `compose_effects(List<OperationEffect>) -> CompositionVerdict`. That signature is defined by PR #529's R3. Starting Part 2 against the pre-#529 `compose_effects(...) -> ComposedEffect` signature would lock DB-18 against a shape being removed — the lens would either (a) build against the doomed `ComposedEffect` and need immediate rework, or (b) anticipate the R3 shape and diverge from main until #529 lands. Both paths burn throughput for no signal. Wait for #529's merge; then Part 2 proceeds against a stable `compose_effects` signature.

**STOP-AND-ESCALATE rule for Part 2 dispatch:**

If Part 2 implementation discovers any of the following, HALT and report to director chat rather than patching forward:

- `WorkflowEffect`'s 4-variant shape is insufficient — e.g., a real workflow fixture requires a fifth variant or a variant payload reshape. DB-18 locks the shape; reshape is a DB revision, not an in-flight patch.
- `BranchArm`'s `condition: PortId` does not distinguish BranchEffect from ParallelEffect structurally in practice (Q4 receipt regresses). Same rule — reshape the substrate only through a DB revision.
- Stage 2b's `analyze_workflow` signature needs to return something other than `WorkflowIdempotencyReport` (e.g., `List<Diagnostic>` for multi-diagnostic workflows). This is an API refinement, possibly an open question for director-chat clearance; do not silently change the signature.
- The reflected `Dag` needs a `workflows` sidecar after all (i.e., `WorkflowEffect` turns out to span multiple declarations). This is an architectural finding worth a DB revision, not a silent additional field.

The director chat owns the call on each of these. Silent in-flight patches destroy the structural-finding signal the pre-clearance exists to capture (per `phase-plan-2026-04-18.md` §7 "Structural-finding escalation rule").

---

## Open questions

1. **Does `ParallelEffect` need a commutativity witness field in Part 2?** The Pattern 3 algebraic-form analysis argues `ParallelEffect` is a concurrent-product (⊗) requiring algebraic commutativity on the target state. Part 1 does NOT require the witness in the carrier because Stage 2b emits a diagnostic on `ParallelEffect` (no verdict computed). When Stage 2e's parallelism lens binds as a real consumer, it will either (a) extend `ParallelEffect` with `commutativity: AlgebraRef` (or a named-alternative witness carrier) as an additive payload extension, or (b) prove that commutativity is derivable from the workflow's op-level algebra without a substrate field. Decision deferred to Stage 2e design pre-clearance — NOT patched forward in Part 2 implementation.

2. **Exact post-#529 `WorkflowIdempotencyReport` shape.** The pre-#529 draft shape in `lane2-compile-time-proofs.md` §Stage 2b is `{ idempotent: Bool, breaking_op: String?, evidence_chain: List<OperationEffect>, diagnostic: Diagnostic? }`. PR #529's pre-start-gate update says this must be replaced with a structural carrier projecting through `CompositionVerdict`. Candidate post-DB-18 shape: `{ verdict: CompositionVerdict, workflow_shape: WorkflowEffect, diagnostic: Diagnostic? }` — but this pairs the algebra output with the input structure in a single record, which is exactly the correlated-fields pattern PR #529 R3 rejected. Alternative: return `CompositionVerdict` OR `Diagnostic` directly from `analyze_workflow` (no outer record); the lens's caller pairs it with the `WorkflowEffect` they already hold. Resolve at Part 2 design.

3. **Does `LoopEffect.body` need a bound carrier in Part 1?** Substrate has `LoopBound = Cardinality { count: PortId } | Descent { cluster: ClusterId }` post-DB-9 R2.1. Stage 2d (symbolic cost) will almost certainly need the bound to compute recursion depth. Part 1 does NOT include a bound field because Stage 2b emits a diagnostic on `LoopEffect` without reading bound info. Part 2 / Stage 2d can add `bound: LoopBound` as an additive extension when its consumer binds — same rationale as §1. If the director chat prefers a single shape-lock including the bound, this PR can land with `LoopEffect { body: WorkflowEffect, bound: LoopBound }` from the start; decision is a scope judgment, not a correctness one.

4. **Does `LinearEffect.ops` need to be `NonEmptyList<WorkflowEffect>` instead of `NonEmptyList<OperationEffect>` to allow mixed nesting?** The worked example C demonstrates the answer: a linear sequence with a branch in the middle lifts to a `BranchEffect` of two linear paths (distributive over composition). No `LinearEffect` with a `WorkflowEffect` in the middle is needed; the structural canonical form is always a variant at the outermost shape. Locked per Q6 (no representation duality) — not open.

5. **Does the substrate need a `WorkflowDeclaration` record to host `WorkflowEffect`?** Today, `OperationDeclaration` hosts `OperationEffect`-related fields (method, path, etc.). There is no equivalent `WorkflowDeclaration` for multi-op workflows. Part 1 does NOT propose one because Stage 2b can read `WorkflowEffect` values from any declaration slot the user happens to declare them on. Part 2 may or may not introduce a `WorkflowDeclaration` depending on user-range fixture shape; this is an ergonomics question, not a correctness one. Open for Part 2 design.

---

## Rejected alternatives

**R0 — lens-only, derive `WorkflowEffect` from DAG shape at walk time.** Rejected in §Revision history. Core issue: re-derives workflow structure heuristically from Bind chain / Branch / Loop behavior, violating `feedback_lenses_not_passes` ("heuristic = missing physics"). A workflow's control-flow shape is a fact about the author's intent, not something the compiler should re-infer per-lens.

**R-alt-A — single `WorkflowEffect` coproduct with a kind tag + optional payload fields.** Shape: `WorkflowEffect { kind: Linear | Branch | Loop | Parallel, ops: Option<List<OperationEffect>>, arms: Option<List<WorkflowEffect>>, body: Option<WorkflowEffect> }`. Rejected because it admits illegal state combinations (`kind: Linear` with `arms: Some(...)`) — the classic `Bool + Option<T>` pattern `feedback_state_space_vs_behavioral_invariants` names. The four-variant coproduct is the state-space-sound alternative.

**R-alt-B — hierarchical inheritance: `WorkflowEffect` is-a `List<OperationEffect>` with optional control-flow overlay.** Rejected because it pre-commits to linear-as-base and overlays branches as a non-structural layer. Stage 2d's branch-wise composition would have to peel back the overlay — the overlay IS the structure. Better to encode control-flow structurally from the start (R1).

**R-alt-C — merge `BranchEffect` and `ParallelEffect` into `ChoiceEffect { kind: Exclusive | Concurrent, arms: NonSingletonList<WorkflowEffect> }`.** Rejected because it re-introduces the `kind + discriminator` pattern that Q4's dissolution receipt is supposed to eliminate. The whole point of the four-variant coproduct is that each variant traces to a distinct categorical operation; compressing two of them under a flag re-creates the compression the coproduct solves.

**R-alt-D — drop `ParallelEffect`; add it later via DB-N when Stage 2e ships.** Viable; the user's directive explicitly asked for 4 variants, so this is not the chosen path. If the director chat later prefers a 3-variant initial shape, that is an additive refinement (drop `ParallelEffect`; Stage 2e's DB adds it when it ships) and does not regress the Q4 receipt for the remaining three variants. Noted for director-chat discussion if the 4-variant commit proves too heavy for Part 1.

---

## Cross-references

- `feedback_coproduct_dissolution` — Q4 receipt (§"Dissolution receipt" above cites patterns 1–4).
- `feedback_substrate_principle_audit` — Q1–Q6 (§"Substrate principle audit" cites all six).
- `feedback_state_space_vs_behavioral_invariants` — cardinality invariants via `NonEmptyList` / `NonSingletonList`; rejected alternative R-alt-A.
- `feedback_lenses_not_passes` — rejected alternative R0 (lens-level re-derivation).
- `feedback_no_metadata_markers` — `BranchArm.condition: PortId` is a typed handle, not a string marker.
- `feedback_fail_closed_discipline` — `report_unsupported_variant` produces a `Diagnostic`, not a silent skip.
- DB-9 R2.1 (`design-mutual-recursion-lowering.md`) — worked example of the six-question audit; mirrors DB-18's format.
- DB-16 (`design-db16-refined-generic-substitution.md`) — worked example of Part 1 design / Part 2 impl split; mirrors DB-18's scope discipline.
- PR #529 (`design-composed-effect-reshape.md`) — `CompositionVerdict` authority that DB-18's `LinearEffect` path delegates to.
- ROADMAP (`../ROADMAP.md` and `../src/v3/ROADMAP.md`) — Lane 2 Stage 2b entry; this DB advances the row from planned lens-only to substrate-carrier + lens per director escalation.

---

End of DB-18 Part 1.
