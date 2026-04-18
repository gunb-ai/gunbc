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

**Boundary note:** `DerivedOpEffect { method, path_template, shape }` is tolerated in Stage 2a only as bootstrap-local staging glue between HTTP surface facts and the ported effects algebra. It is NOT the durable boundary for downstream consumers. Before Track 17a REST wiring or Stage 2b teaches consumers that this is canonical, the follow-up must either:
- collapse `DerivedOpEffect` into the existing authority carriers that actually survive the lane boundary, or
- classify it explicitly as scaffold debt with a named dissolution trigger if it must persist longer.

Lane 2 does not get to silently normalize around this wider record shape.

Copy:
- `EffectShape = ReadEffect | UpsertEffect | DeleteEffect | CreateEffect | AppendEffect`
- `KeySource` (for upsert/delete key derivation)
- `IdempotencyEvidence = LatticeEffect | IdentityEffect | NonIdempotent`
- `is_idempotent_effect`, `compose_effects`, `ComposedEffect`, `OperationEffect`
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

**Pre-start gate:** Stage 2b does not start consuming `DerivedOpEffect` as if it were stable substrate. The Stage 2a follow-up above must land first: either the record collapses to the durable authority shape, or it gains an explicit scaffold classification plus a named dissolution trigger that bounds how long it can survive.

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
