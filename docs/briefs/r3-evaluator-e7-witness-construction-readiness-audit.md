# R3 Evaluator — E7 Witness Construction Readiness / Blocker Audit

**Status:** AUDIT — docs-only. Records the exact prerequisites E7
(witness construction surface) needs from E5 (Loop), names the API
shape the first executable post-E5 slice will fill, and locks the
fail-closed boundaries E7 implementation must respect. **No Rust, no
substrate, no fixture changes land in this slice.**

**Authorities:**
- [`docs/briefs/r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
  §"Implementation Slices" → E7 spec; §"Parallelization" — "E6 and E7
  wait for the body evaluator spine."
- [`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) §
  `Witness<Carrier>` (line 35), `DimensionReport<Carrier>` (line 51),
  `AnalysisDimension<Carrier>` (line 73) — substrate authorities.
- [`src/v3/compiler/src/dimension.rs`](../../src/v3/compiler/src/dimension.rs)
  — existing Rust mirrors for `Witness<C>` / `DimensionReport<C>`,
  plus `analyze_symbolic_cost_dimension` (the Q6.5 lens-instance
  precedent the dispatch brief points at).
- E0 / E1 / E2 / E4 landed (PRs #1371, #1387, #1374, #1426); E3 landed.
  E5 (Loop) NOT YET LANDED — `Behavior::Loop` returns
  `EvalError::UnsupportedBehavior { behavior: "Loop" }` in
  `eval_node`, regression-tested at `lib.rs::loop_behavior_fails_closed`.

## State at HEAD

- **Substrate `Witness` / `DimensionReport` / `AnalysisDimension`**:
  declared in `dimensions.dag`, mirrored in Rust at `dimension.rs:46-69`
  with the same coproduct partition.
- **Existing analyzer**:
  `dimension.rs::analyze_symbolic_cost_dimension(Dag, NodeId)
  -> DimensionReport<SymbolicCost>` walks `behavior_spine_in_node_order`
  (not the body evaluator) and emits `Witness::Inhabits` /
  `Witness::Violates` from `SymbolicCostLookup` outcomes. This is the
  Q6.5 lens-instance precedent E7 should generalize.
- **Body evaluator**: `evaluate_body` (E0/E1) plus per-`Behavior`
  evaluators E1 (Value), E3 (Transform), E4 (Branch). E5 (Loop)
  returns `UnsupportedBehavior`; E6 (Bind) likewise.
- **No `Witness`-from-`Value` bridge**: `evaluate_body` returns a
  runtime `Value`, not a `Witness<C>`. Nothing in the tree today
  composes a `DimensionReport` from `evaluate_body` results for
  representative lenses. That is exactly what E7 must add.

## Hard prerequisite — E5 (Loop)

Per the dispatch parallelization §, "E6 and E7 wait for the body
evaluator spine." Loop is the missing arm:

- Lens analyses over recursive `.dag` programs traverse `Loop` nodes
  (the substrate's bounded-iteration carrier); without `eval_loop`,
  `evaluate_body` fail-closes `UnsupportedBehavior` on any program
  that contains one.
- Witness construction depends on the body evaluator producing
  defined `Value`s for every behavior in the analyzed program. With
  Loop unhandled, witness construction over realistic programs
  trivially fails-closed (`UnsupportedBehavior` propagates).

E7 implementation cannot ship a representative-lens witness fold
until E5 lands `eval_loop` covering at least the
`LoopBound::Cardinality { count }` arm. (`LoopBound::Descent` is a
named fail-closed residual per PR-B.0 / PR-B.1; E7 inherits that
residual without expanding it.)

## E7 implementation scope (post-E5)

Three layered functions, in ship order:

### 1. `witness_for_behavior` (per-behavior)

Bridge a single behavior's `evaluate_body` result to a `Witness<C>`:

```text
fn witness_for_behavior<C>(
    dag: &Dag,
    behavior: &Behavior,
    state: &mut EvalStateStack<Value>,
    strategy: &EvalStrategy,
    dimension: &impl LensRunnerView<C>,
) -> Witness<C>;
```

`LensRunnerView<C>` is a Rust-side trait that mirrors the
substrate's `AnalysisDimension<Carrier>.witness_of` field — it
exposes a single `witness_of_value(&Behavior, Value) -> Witness<C>`
method per dimension. E7 will start with three concrete
implementations (one per representative lens):

- `ComplexityDimension` — wraps the existing `cost_of` / `compute_costs`
  path; converts `CostLookup::Hit(_)` → `Inhabits(SymbolicCost)`,
  `CostLookup::Miss` → `Violates`.
- `TenantFlowDimension` — placeholder until lens-instance details are
  named; first slice may ship as a structural stub returning
  `Inhabits(())` for behaviors with no tenant flow and `Violates` for
  cross-tenant edges.
- `IfcDimension` — same shape; first slice ships a minimal IFC
  classification per behavior.

Each per-dimension implementation is a small free-function fold over
the existing generated lens code (`lens_cost_generated.rs` etc.)
plus the body evaluator's `Value` outputs at result ports.

### 2. `analyze_with_evaluator` (per-program)

Compose per-behavior witnesses across the workflow spine:

```text
fn analyze_with_evaluator<C>(
    dag: &Dag,
    workflow_root: NodeId,
    dimension: &impl LensRunnerView<C>,
) -> DimensionReport<C>;
```

Mirrors `analyze_symbolic_cost_dimension`'s structure but consumes
`evaluate_body` instead of `behavior_spine` directly. Composes via
the dimension's `compose` / `identity` (Rust trait methods mirroring
substrate `AnalysisDimension`'s `compose: fn(C, C) -> C` and
`identity: C`). On any `Witness::Violates`, returns
`DimensionFail { violations, witnesses }` — never fabricates a
`composed: C` (R2 fail-closed, per `dimensions.dag:48-50`).

### 3. Per-lens public entrypoints

```text
pub fn analyze_complexity(dag: &Dag, root: NodeId) -> DimensionReport<SymbolicCost>;
pub fn analyze_tenant_flow(dag: &Dag, root: NodeId) -> DimensionReport<TenantFlow>;
pub fn analyze_ifc(dag: &Dag, root: NodeId) -> DimensionReport<IfcLabel>;
```

Each is a thin wrapper over `analyze_with_evaluator` with the
appropriate `LensRunnerView<C>` instance. These are the entrypoints
E7 acceptance tests target.

## Acceptance — first executable post-E5 slice

Tests the first E7 PR must include (per dispatch brief §E7
Acceptance):

1. **Complexity** — analyze a small program with bounded `Loop`,
   assert `DimensionReport::DimensionOk { composed: SymbolicCost::… }`
   matches `analyze_symbolic_cost_dimension`'s result. (Cross-check
   against the existing analyzer's Q6.5 precedent.)
2. **Tenant-flow** — analyze a program with two tenants; assert
   `DimensionFail` when a cross-tenant edge exists, `DimensionOk`
   otherwise.
3. **IFC** — analyze a program with mixed High/Low labels; assert
   `DimensionFail { violations: [Diagnostic::IfcLeak { … }] }`
   when a Low edge consumes a High value.
4. **Typed-diagnostic discipline** — every `Witness::Violates.reason`
   and every `DimensionFail.violations[i]` is asserted by **typed
   pattern match**, not string parsing. `Diagnostic` is the typed
   envelope; reason strings are human-facing only.
5. **Fail-closed propagation** — a program with `Behavior::Loop` of
   `LoopBound::Descent { cluster }` returns `DimensionFail` with the
   E5 residual diagnostic, not a fabricated carrier.
6. **No bridge fabrication** — when `evaluate_body` returns
   `EvalError::UnboundPort` or other E0/E1/E2 fail-closed cases, the
   witness wrapper propagates `Witness::Violates` with a typed
   diagnostic, never `Witness::Inhabits` over an arbitrary default.

## STOP+PING boundary (E7 itself, when implemented)

- **No new `Witness` variant or `DimensionReport` variant** — those
  are TERMINAL per `dimensions.dag:35,51` and the Rust mirror's
  comment.
- **No string parsing of `Witness.reason`** in tests or downstream
  consumers — typed diagnostic only (per dispatch brief §E7
  Acceptance: "with typed diagnostics and no string parsing of
  `Witness.reason`").
- **No lens-local diagnostic kind** without routing through the Q6.5
  lens-instance path — per dispatch STOP+PING: "if a lens-local
  diagnostic kind requires substrate-owned closed sum extension
  instead of the Q6.5 lens-instance path" → escalate.
- **No expansion of E5 (Loop) semantics** in the E7 PR. If a test
  uses a Loop construct and E5 is missing or partial, the test
  fail-closes through `EvalError::UnsupportedBehavior` and E7's
  diagnostic propagation; E7 must not silently implement Loop to
  make a test pass.
- **No Bool-as-Disj bridge** — same routing as E4 (Substrate #1130).
- **No new substrate carrier**, no new `Value` variant, no new
  `EvalError` variant beyond those already named in PR-B.1's
  fail-closed catalog.

## Cross-references

- PR-E E0 (#1371) — body-evaluator API contract; defines
  `EvalDiagnostic` / `EvalError` shape that E7 propagates.
- PR-E E1 / E2 / E3 / E4 (#1387, #1374, E3 PR, #1426) — consumed
  through `evaluate_body`.
- PR-E E5 (Loop) — hard prerequisite; **NOT YET LANDED**.
- PR-B.0 / PR-B.1 — body evaluator semantics + fail-closed catalog.
- `src/v3/std/dimensions.dag` — Witness / DimensionReport /
  AnalysisDimension substrate authority.
- `src/v3/compiler/src/dimension.rs::analyze_symbolic_cost_dimension`
  — Q6.5 lens-instance precedent the E7 generalization mirrors.

## Out of scope (this audit)

- Implementing `witness_for_behavior`, `analyze_with_evaluator`,
  `analyze_complexity` / `analyze_tenant_flow` / `analyze_ifc`.
- Implementing E5 (Loop).
- Authoring the `LensRunnerView<C>` trait.
- Defining `TenantFlow` / `IfcLabel` carrier types.
- Tests / fixtures.
- Substrate carrier additions.

## What unblocks E7 implementation

1. **E5 lands**: `eval_loop` over `LoopBound::Cardinality` covers
   non-descent loops; `LoopBound::Descent` stays fail-closed
   residual (per PR-B.1).
2. **Worker dispatch**: Director routes the E7 implementation slice
   on a fresh branch, consuming this audit's API contract.
3. **Per-lens carrier types**: at minimum `TenantFlow` / `IfcLabel`
   (or a stand-in for the first slice) — these can land inside E7
   implementation if scoped narrowly, since they are evaluator-side
   carriers, not substrate.
