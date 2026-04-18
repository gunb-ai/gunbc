> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) (Lane 2 master)

# Lane 2 — Compile-time proofs

**Lane:** 2 (of 3)
**Size:** XL (six stages), overlaps Lane 1 after 1b lands
**Status:** Plan. No code changes yet.

---

## Thesis mandate

From THESIS.md (paraphrased):

> Correctness is not one property — it is many orthogonal dimensions: termination, type safety, ownership, side effects, purity, idempotence, space bounds. In traditional systems these are separate tools that you opt into. In gunbc, they are **inescapable properties of the system**, like conservation laws in physics. You don't opt into gravity.

Today's state (per THESIS.md:1186–1293 matrix):

| Property | Declared | Lattice / composed | Gate | Compile-time enforced? |
|---|---|---|---|---|
| Termination | ✅ | ✅ | ✅ | ✅ |
| Type safety | ✅ | — | ✅ | ✅ |
| Ownership | ✅ | 🟡 partial | 🟡 partial | 🟡 partial (Phase 2 in Lane 1a) |
| **Idempotence** | ✅ `dsl/std/effects.dag` | ✅ `compose_effects` | ❌ | ❌ not wired |
| **Side effects** | ✅ `std/behavioral.dag` | ❌ | ❌ | ❌ |
| **Space bounds** | ❌ | ❌ | ❌ | ❌ |
| **Structural cost** | ✅ `complexity.dag` | ✅ forward-fold | ✅ | ✅ structural only |
| **Symbolic cost (O(n))** | ❌ | ❌ | ❌ | ❌ (NOT YET IMPLEMENTED, L2 M1) |
| **Parallelism** | 🟡 structurally tested | ❌ lens output | ❌ | ❌ (structure detected, diagnostic missing) |

This lane closes the "❌ not wired" gaps across six stages. After Lane 2 completes, every row in the table above reads "compile-time enforced: ✅" — except side effects (bigger scope, separate future work) and space bounds (truly deferred — no known thesis blocker gates on it).

---

## Stages

### Stage 2a — v3 effects algebra port (S)

**Scope:** port `dsl/std/effects.dag` → `src/v3/std/effects.dag`. Structural carry-over only.

**Boundary note (cleared):** Stage 2a plus all three follow-up refinements have landed. The final shape responds to two review rounds that each pushed state-space soundness one layer deeper.

`DerivedOpEffect { method, path_template, shape }` was collapsed into `OperationEffect { operation_name, shape }` in PR #521. The `method` / `path_template` fields were carried as bootstrap-local staging glue but never consumed downstream — both modifier falsification and obligation generation project through `shape` alone, and `ReadEffect` already encodes "method was GET/HEAD/OPTIONS" structurally. `derive_op_effect` now returns `OperationEffect?`, so derivation outputs the same shape `compose_effects` composes.

`ComposedEffect { operations, idempotent, breaking_operation }` was removed entirely; `EffectShape` partitioned; `compose_effects` now returns `CompositionVerdict` directly. The final R3 shape:

- `type IdempotentShape = ReadEffect | UpsertEffect { key_source } | DeleteEffect { key_source }`
- `type BreakingShape = CreateEffect { cause } | AppendEffect`
- `type EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`
- `type BreakingOperation { operation_name, shape: BreakingShape }`
- `type CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }`
- `fn compose_effects(effects: List<OperationEffect>) -> CompositionVerdict`
- No `ComposedEffect`. The evidence chain stays on the caller's side as the input `List<OperationEffect>`; the verdict is the output.

Design: [design-composed-effect-reshape.md](./design-composed-effect-reshape.md) (R3-final). v3-only per the same scope discipline PR #521 used. The partition means `is_idempotent_effect` reads the outer variant directly; `operation_is_breaking` does too; `classify_idempotent_disagreement` narrows its argument to `BreakingShape` (dead-arm cleanup). Dropping the record means there is no correlated-fields invariant for the type to fail to enforce.

Copy (post-reshape, R3):
- `IdempotentShape = ReadEffect | UpsertEffect | DeleteEffect`
- `BreakingShape = CreateEffect | AppendEffect`
- `EffectShape = IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`
- `KeySource` (for upsert/delete key derivation)
- `IdempotencyEvidence = LatticeEffect | IdentityEffect | NonIdempotent`
- `OperationEffect { operation_name, shape: EffectShape }`
- `BreakingOperation { operation_name, shape: BreakingShape }`
- `CompositionVerdict = IdempotentComposition | BrokenBy { first_breaker: BreakingOperation }`
- `is_idempotent_effect`, `operation_is_breaking`, `operation_to_breaker`, `compose_effects`
- `derive_effect_shape(method, path)` — HTTP method + path → EffectShape
- `generate_idempotency_obligations`
- `check_modifier_vs_derivation`

**What does NOT come over:** v2-specific imports, parser quirks, any Rust bridge code. Pure `.dag` declarations.

**Acceptance:** `src/v3/std/effects.dag` compiles cleanly in v3; minimal smoke test asserts parse + 5 representative function signatures.

**Escalation:** if any v2 effects type uses a construct v3 doesn't yet parse (unlikely given L1 completeness, but check first). If blocked, surface — don't half-port.

### Stage 2b — Workflow idempotency lens (L)

**Scope:** create `src/v3/lenses/idempotency.dag`. Walks a pipeline (sequence of service operations), composes effects, emits diagnostic on chain break.

API shape:
```
fn analyze_workflow(d: Dag, workflow: NodeId) -> WorkflowIdempotencyReport

type WorkflowIdempotencyReport {
  idempotent: Bool
  breaking_op: String?             // name of first non-idempotent op, if any
  evidence_chain: List<OperationEffect>
  diagnostic: Diagnostic?
}
```

Lens reads each operation's declared `idempotent` modifier AND derives from path+method, then cross-checks via `check_modifier_vs_derivation`. Diagnostic fires when:
- Declared idempotent but derivation disagrees (`Disagrees` case)
- Workflow composition breaks because a single op is non-idempotent (`POST /logs` in a retry context)
- Modifier claims `readonly` but method is write

**Acceptance:**
- Fixture: GCP Secret Manager upsert + STS Exchange + IAM grant → all idempotent → report green
- Fixture: above + `POST /audit_log` at the end → report red, naming `POST /audit_log` as breaking op
- Fixture: `POST /secrets/create` (no path key) inside a retry loop → compile fails with specific diagnostic

**Escalation:** if workflow structure isn't representable cleanly — e.g., control flow in a pipeline doesn't map to a linear `List<OperationEffect>` — surface. Don't stretch `compose_effects` to handle branches silently; the algebra needs to reflect branch-wise composition, which is a legitimate design extension.

<<<<<<< HEAD
**Input structure (DB-18) — `WorkflowEffect` substrate carrier.** The escalation clause above named the extension that DB-18 now locks in: workflow control-flow shape is promoted from "something the lens reconstructs from DAG shape at walk time" (a heuristic, which `feedback_lenses_not_passes` rejects) to a first-class substrate carrier `WorkflowEffect` that lowering produces and lenses read structurally. Shape:

```
type WorkflowEffect
  = LinearEffect { ops: NonEmptyList<OperationEffect> }
  | BranchEffect { arms: NonSingletonList<BranchArm> }
  | LoopEffect { body: WorkflowEffect }
  | ParallelEffect { branches: NonSingletonList<WorkflowEffect> }

type BranchArm { condition: PortId; body: WorkflowEffect }
```

Stage 2b is the initial consumer with **`LinearEffect`-only scope**: the lens matches on `LinearEffect` and delegates to `compose_effects(ops |> to_list) -> CompositionVerdict` (post-PR #529). The other three variants emit explicit fail-closed diagnostics (C-8) naming the downstream stage that will consume them — `BranchEffect` → Stage 2d branch-wise composition, `LoopEffect` → Stage 2d fixpoint / body convergence, `ParallelEffect` → Stage 2e parallelism-as-lens with commutativity witness. No silent skip; a `WorkflowEffect` the Stage 2b lens cannot verdict produces a `Diagnostic` explaining which variant, which downstream stage, and why. The `analyze_workflow` signature above becomes `fn analyze_workflow(d: Dag, workflow: WorkflowEffect) -> WorkflowIdempotencyReport`; the NodeId-indexed form in the draft above is the pre-DB-18 sketch. `WorkflowEffect` and `CompositionVerdict` (post-PR #529) coexist on orthogonal axes — `WorkflowEffect` is the input-structure carrier, `CompositionVerdict` is the output-verdict carrier; no enclosing record pairs them (preserving the lesson PR #529 R3 locked).

Design: [design-db18-workflow-effect-carrier.md](./design-db18-workflow-effect-carrier.md) — Part 1 locks the 4-variant shape, the `BranchArm` structural distinction, the Q4 dissolution receipt, and the Q1–Q6 substrate-principle audit. **Part 2 implementation is gated on PR #529 landing** (see Pre-start gate below); Part 2 ships `type WorkflowEffect` in `src/v3/std/effects.dag`, the Rust mirror in `src/v3/compiler/src/dag.rs`, the lens in `src/v3/lenses/idempotency.dag`, and reconciles `WorkflowIdempotencyReport` with `CompositionVerdict` (see DB-18 §Open questions).

**Pre-start gate:** Stage 2b does not start consuming `ComposedEffect` as if it were stable substrate. The Stage 2a `DerivedOpEffect` collapse has landed (derivation now emits `OperationEffect`, which is the same shape `compose_effects` walks). The remaining Stage 2a follow-up is `ComposedEffect`: it must stop encoding workflow verdicts as duplicated summary fields before Stage 2b consumers depend on it.
=======
**Pre-start gate:** cleared. All Stage 2a follow-ups have landed — `DerivedOpEffect` collapsed into `OperationEffect` (PR #521); `EffectShape` partitioned into `IsIdempotent(IdempotentShape) | IsBreaking(BreakingShape)`; `BrokenBy`'s payload narrowed to `BreakingOperation { shape: BreakingShape }`; `ComposedEffect` removed. `compose_effects(effects: List<OperationEffect>) -> CompositionVerdict` is the algebra's output. Stage 2b consumes `CompositionVerdict` directly; callers pair it with their own input `List<OperationEffect>` for diagnostic rendering. The Stage 2b `WorkflowIdempotencyReport` shape above is one layer higher (a lens report with diagnostic), not a duplication of the algebra verdict — when Stage 2b implements it, the report must match on `CompositionVerdict` and project through `BrokenBy.first_breaker.shape: BreakingShape`. The `breaking_op: String?` field in the draft `WorkflowIdempotencyReport` shape above is a design placeholder from before the Stage 2a partition landed; Stage 2b's implementer should replace it with a structural carrier derived from `CompositionVerdict` rather than copy the flat shape. The implementer must not reintroduce a `ComposedEffect`-style record that pairs `CompositionVerdict` with a sibling list field, since that reintroduces the correlated-fields incoherence R3 just removed.

**Stage 2b / Track 17a design axis — witness vs copy.** `BreakingOperation` today is a *copy* of the originating `OperationEffect`, not a carrier-relative witness into `compose_effects`'s input list. This is a deliberate Track 9 deferral: `ElementRef<T>`-style handles "land when a concrete consumer needs it, not speculatively" (`ROADMAP.md:763-772`). When the Stage 2b lens or a Track 17a consumer first needs to render the breaker's position in the workflow or structurally tie the verdict back to its evidence chain, the handle graduation belongs in that PR — not retrofitted into Stage 2a. Until that graduation, consumers that need position-in-workflow can search the caller's input list by name; the verdict itself is sound (sum variants coherent, `BreakingShape` narrowing intact).
>>>>>>> origin/main

### Stage 2c — Test obligation materialization (M)

> **Load-bearing for Lane 2's acceptance.** Stage 2c is the bridge between "compile-time proof" and "we can demonstrate the proof." If 2c slips or gets descoped, Lane 2's acceptance gate degrades from *"compile-time-enforced + runtime-validated"* to *"compile-time-enforced only"* — the thesis claim that idempotency is "inescapable" weakens because we can't point at runnable tests proving it. Implementer must not silently descope this stage; if it needs more time, escalate per the plan's escalation protocol.

**Scope:** `generate_idempotency_obligations` today returns `List<IdempotencyTestObligation>`. Today nothing consumes it. L2c wires it to actual emitted tests.

For each idempotent op, emit (via Lane 1e's generic walker):
```rust
#[test]
fn idempotency_{op_name}() {
    let state1 = run(op_name, test_input);
    let state2 = run(op_name, test_input);
    assert_eq!(state1, state2, "op `{op_name}` claimed idempotent but f(x) != f(f(x))");
}
```

Works through mock harness for cloud ops — ops have `mock_response` already declared in extdeps.

**Acceptance:**
- Fixture: GCP STS.Exchange → generated test passes against mock
- Fixture: fabricated non-idempotent op declared `idempotent` → generated test FAILS, catching the mismatch
- Generated test count matches obligation count from `generate_idempotency_obligations`

**Escalation:** if mock harness is insufficient (e.g., real network needed to express some class of idempotency), surface — shouldn't invent a local mock framework. The extdeps mock_response surface should cover all declared ops; if not, extdeps needs extension.

### Stage 2d — Symbolic cost bounds (M)

**Scope:** L2 M1 from the thesis validation doc. Structural cost (Lane 1a's forward-fold) reports op counts; symbolic cost reports asymptotic complexity as O(f(n)) where n is input size.

New lens `src/v3/lenses/complexity_symbolic.dag`:
```
fn symbolic_cost(d: Dag, port: PortId) -> SymbolicCost

type SymbolicCost
  = Constant(Int)              // O(1)
  | Linear(VariableRef)        // O(n)
  | Polynomial(VariableRef, Int)  // O(n^k)
  | NestedLoop(List<SymbolicCost>) // O(n*m), etc.
```

Recognizes:
- `fold` with constant lambda body → Linear(list_size_var)
- `fold` inside `fold` where inner closes over outer's list → Polynomial(outer_var, 2)
- Recursion with smaller structural argument → polynomial bound from recursion depth

Emits diagnostic for unexpected complexity (e.g., hidden O(n²) via captured list in lambda).

**Acceptance:**
- Unignores `kf_1_lambda_body_cost_contributes_to_fold` (test passes, because symbolic cost correctly attributes lambda body × N iterations)
- Thesis doc example `all_pairs` reports `O(|items|²)` as a **lens output** queryable via `symbolic_cost(d, port)`; IDE tooling may surface this to users, but the compiler does NOT emit a diagnostic for high-complexity code unless a `where cost_bounded(...)` declaration is violated (in which case it IS an error)
- Thesis doc "dead work detection" (sort before commutative fold): returns a structured `DeadWorkReport` from the lens as queryable data. Also NOT a warning — per INVARIANTS.md:410-417 there is no warning severity. If the user declares `where no_dead_work`, violation is an error; otherwise the report is informational data only.

**Severity discipline**: neither symbolic cost nor dead-work detection emits soft warnings. INVARIANTS.md §Diagnostic-severity is explicit: a reportable condition is either an error OR not a diagnostic. Cost analysis results are DATA consumed by downstream tools (IDE, CI reports, optimization lenses). The user opts into errors via refinement predicates (`where cost_bounded(...)`, `where no_dead_work`) which ARE compile-time errors when violated.

**Escalation:** if recognition rules for O(n²) require solving arbitrary symbolic arithmetic, scope tighter — recognize the thesis doc's two patterns (nested fold, sort-before-commutative) and document what's not recognized. Don't build a full symbolic math library.

### Stage 2e — Parallelism-as-lens (S)

**Scope:** `thesis_parallelism_test.rs` already asserts structural parallelism via `has_transitive_dependency`. Stage 2e promotes that structural fact to a lens output users can see.

New lens `src/v3/lenses/parallelism.dag`:
```
fn analyze_parallelism(d: Dag, workflow: NodeId) -> List<ParallelizationOpportunity>

type ParallelizationOpportunity
  = IndependentBindings(NodeId, NodeId)     // "a and b have no dep, run in parallel"
  | MapPromotion(NodeId)                     // "this fold is actually a map"
  | CommutativeReduction(NodeId, AlgebraRef) // "this fold reduces on a commutative monoid — tree-reducible"
```

Unignores `parallel_fold_on_commutative_monoid_is_reducible`.

**Acceptance:**
- Structural parallelism tests from `thesis_parallelism_test.rs` now emit a corresponding `ParallelizationOpportunity` entry in the lens's output
- Commutative monoid fold produces `CommutativeReduction` entry
- Lens output is **data, not a diagnostic**. Per INVARIANTS.md:410-417 there is no "info" severity. IDE tooling reads the lens output and may surface "this fold is parallelizable" as an inline hint; the compiler itself treats the output as structured data, not as a diagnostic

**Severity discipline**: parallelism-as-lens output is DATA. Not a warning, not an info, not a diagnostic. Downstream tooling (IDE, docs generation, compile-time parallelization passes) consume it. If user code declares `where parallel_reducible` and the analysis disagrees, THAT declaration mismatch IS an error — but "this could be parallelized" on unconstrained code is just information the lens produces.

**Escalation:** if commutative-monoid detection requires algebra-awareness the `.dag` compiler doesn't have yet (operator-on-declared-Monoid-instance lookup), surface. That primitive might live in algebra.dag; if it doesn't, it's a prerequisite.

### Stage 2f — User-declared dimensions (S)

**Scope:** the infrastructure from 2b–2e (lens walks workflow, composes algebra, emits diagnostic) has a common shape. Generalize so users can add new compile-time dimensions via `.dag` declaration alone.

Type shape is **locked in [DB-3](./design-dimension-abstraction.md)** — `Dimension<Carrier>` with `name`, `witness_of`, `compose`, `identity`, `break_diagnostic` fields. See DB-3 for full signature, algebraic requirements (monoid laws on compose), and instance declarations.

Ship idempotency (2b) and symbolic-cost (2d) as Dimension instances. **Parallelism (2e) is NOT a Dimension instance** — per DB-3's resolved Open Question §1, parallelism composes over dependency structure rather than per-operation monoidal evidence and therefore stays as an ordinary lens. Document how users declare new *monoidal* dimensions (e.g., "resource exhaustion" on cloud ops). Structural lenses that aren't monoidal stay as lenses without claiming the Dimension interface.

**Acceptance:**
- Idempotency lens (2b) rewritten to consume the `Dimension` abstraction — same behavior, less bespoke code
- Example user dimension in a test: `memory_bounded: Dimension` on cloud ops, compile gate fires if workflow exceeds declared bound
- Design doc for how to add a new Dimension

**Escalation:** if the generalization requires significant rework of 2b's lens, defer to a follow-up — don't break 2b. This stage is the stretch; if it feels forced, ship 2b–2e as-is and write the design doc for future extension.

---

## Cross-cutting acceptance (Lane 2 done when)

- [ ] v2's 16 idempotency tests have v3 equivalents that pass
- [ ] A declared non-idempotent workflow fails compile with specific diagnostic
- [ ] `kf_1_lambda_body_cost_contributes_to_fold` unignored and passing
- [ ] `parallel_fold_on_commutative_monoid_is_reducible` unignored and passing
- [ ] At least one user-declared dimension example working end-to-end
- [ ] Every Lane 2 lens reads substrate via Lane 1 Stage 1b's accessors — zero `find_port`-style helpers in Lane 2 code

---

## Dependencies

- **Requires Lane 1 Stage 1b complete** — Lane 2 lenses consume keyed substrate accessors, not reconstruct. This is the hard gate that delays Lane 2 start.
- **Does NOT require Lane 1 complete** — Lane 2 runs in parallel with Lane 1 Stages 1c–1f once substrate accessors are in.
- **Blocks nothing in Lane 3** — Lane 3 is self-hosting, not property proofs.

---

## Size

XL aggregate (six stages), overlaps Lane 1 starting after 1b. Per-stage sizes:
- 2a: S
- 2b: L
- 2c: M
- 2d: M
- 2e: S
- 2f: S

Some internal parallelism possible if 2+ implementers.
