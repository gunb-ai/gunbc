# R3 Evaluator — E7 Witness Construction Readiness / Blocker Audit

**Status:** AUDIT — docs-only. Records the exact prerequisites E7
(witness construction surface) needs from E5 (Loop), names the API
shape the first executable post-E5 slice will fill, and locks the
fail-closed boundaries E7 implementation must respect. **No Rust, no
substrate, no fixture changes land in this slice.**

**Authorities:**
- [`docs/briefs/r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
  §"Implementation Slices" → E7 spec; §"Parallelization" — "E6 and E7
  wait for the body evaluator spine." The locked E7 decisions consumed here
  are: materialize `Witness::Inhabits` / `Witness::Violates` from evaluator
  results, cover complexity / tenant-flow / IFC-style outcomes, use typed
  diagnostics, and do not parse `Witness.reason`.
- [`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) §
  `Witness<Carrier>` (line 35), `DimensionReport<Carrier>` (line 51),
  `AnalysisDimension<Carrier>` (line 73), and `Dimension<Unit, Carrier>`
  (line 89) — substrate type authorities. **No `data
  AnalysisDimension<...>` instances are live today**; `dimensions.dag`
  explicitly defers `data symbolic_cost_dimension: AnalysisDimension<SymbolicCost>`
  behind the class-5 record-body gap.
- [`src/v3/compiler/src/dimension.rs`](../../src/v3/compiler/src/dimension.rs)
  — existing Rust mirrors for `Witness<C>` / `DimensionReport<C>`,
  plus `analyze_symbolic_cost_dimension` (the Q6.5 lens-instance
  precedent the dispatch brief points at).
- E0 / E1 / E2 / E4 landed (PRs #1371, #1387, #1374, #1426); E3 landed.
  E5 (Loop) NOT YET LANDED — `Behavior::Loop` returns
  `EvalError::UnsupportedBehavior { behavior: "Loop" }` in
  `eval_node`, regression-tested at `lib.rs::loop_behavior_fails_closed`.
  `Behavior::Bind` is also still fail-closed; it is body-evaluator coverage,
  not the E6 lens-fold slice.

## State at HEAD

- **Substrate `Witness` / `DimensionReport` / dimension types**:
  `Witness<Carrier>` and `DimensionReport<Carrier>` are declared in
  `dimensions.dag` and mirrored in Rust at `dimension.rs:46-69` with the
  same coproduct partition. `AnalysisDimension<Carrier>` and
  `Dimension<Unit, Carrier>` are type authorities in `dimensions.dag`, but no
  concrete `AnalysisDimension` data record is live; E7 must not pretend one
  exists until class-5 record bodies or an explicit host bridge lands.
- **Existing analyzer**:
  `dimension.rs::analyze_symbolic_cost_dimension(Dag, NodeId)
  -> DimensionReport<SymbolicCost>` walks `behavior_spine_in_node_order`
  (not the body evaluator) and emits `Witness::Inhabits` /
  `Witness::Violates` from `SymbolicCostLookup` outcomes. This is the
  Q6.5 lens-instance precedent E7 should generalize.
- **Body evaluator**: `evaluate_body` (E0/E1) plus per-`Behavior`
  evaluators E1 (Value), E3 (Transform), E4 (Branch). E5 (Loop)
  returns `UnsupportedBehavior`; `Bind` also returns `UnsupportedBehavior`.
  This audit treats both as prerequisites for representative E7 execution
  rather than importing state from an unlanded branch.
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

`LensRunnerView<C>` is a proposed Rust-side adapter for the first
implementation slice, not a claim that `AnalysisDimension` data instances are
already constructible. It consumes the live `Witness<C>` /
`DimensionReport<C>` mirrors and exposes
`witness_of_value(&Behavior, Value) -> Witness<C>` per representative
dimension until the substrate can instantiate `AnalysisDimension` records.
E7 will start with three concrete implementations (one per representative
lens):

- `ComplexityDimension` — wraps the existing **symbolic-cost** path
  (`crate::lens_cost_symbolic::symbolic_cost_of` →
  `Lookup<SymbolicCost>` per `lens_cost_symbolic_generated.rs:9`,
  exposed as `SymbolicCostLookup` per `dimension.rs:22`). Converts
  `SymbolicCostLookup::Hit(cost)` → `Witness::Inhabits(cost)` and
  `SymbolicCostLookup::Miss` → `Witness::Violates` exactly as the
  existing `analyze_symbolic_cost_dimension` (`dimension.rs:158-215`)
  already does. The integer-domain `cost_of` / `CostLookup<i64>`
  path returns `i64` not `SymbolicCost` and is **not** the carrier
  authority for `DimensionReport<SymbolicCost>`; `ComplexityDimension`
  must consume the symbolic-cost authority, not the integer-cost one.
- `TenantFlowDimension` — **not live today**. The first executable slice must
  either consume a landed evaluator-side carrier or stay readiness-only; it
  must not use `()` as a silent stand-in for tenant flow.
- `IfcDimension` — **not live today**. The first executable slice must consume
  a landed IFC label carrier or stay readiness-only; it must not fabricate a
  local label space.

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
`evaluate_body` instead of `behavior_spine` directly. Composition must use the
representative dimension's declared compose/identity authority when that
authority is live; until concrete `AnalysisDimension` data records are
instantiable, the first implementation slice may only use already-landed
host authority such as the symbolic-cost analyzer path. On any
`Witness::Violates`, returns `DimensionFail { violations, witnesses }` —
never fabricates a `composed: C` (R2 fail-closed, per `dimensions.dag:48-60`).

### 3. Per-lens public entrypoints

```text
pub fn analyze_complexity(dag: &Dag, root: NodeId) -> DimensionReport<SymbolicCost>;
pub fn analyze_tenant_flow(dag: &Dag, root: NodeId) -> DimensionReport<TenantFlow>;
pub fn analyze_ifc(dag: &Dag, root: NodeId) -> DimensionReport<IfcLabel>;
```

Each is a thin wrapper over `analyze_with_evaluator` with the
appropriate `LensRunnerView<C>` instance **after** the corresponding carrier
types are live. Until `TenantFlow` and `IfcLabel` exist, those entrypoints are
named targets, not implementation-ready functions.

## Acceptance — first executable post-E5 slice

Tests the first E7 PR must include (per dispatch brief §E7
Acceptance):

1. **Complexity** — analyze a small program with bounded `Loop`,
   assert `DimensionReport::DimensionOk { composed: SymbolicCost::… }`
   matches `analyze_symbolic_cost_dimension`'s result. (Cross-check
   against the existing analyzer's Q6.5 precedent.)
2. **Tenant-flow** — after the `TenantFlow` carrier exists, analyze a program
   with two tenants; assert `DimensionFail` when a cross-tenant edge exists,
   `DimensionOk` otherwise.
3. **IFC** — after the `IfcLabel` carrier exists, analyze a program with mixed
   High/Low labels; assert `DimensionFail` with typed diagnostics when a Low
   edge consumes a High value.
4. **Typed-diagnostic discipline** — typed assertion lives on
   `DimensionFail.violations: List<Diagnostic>`: every entry is
   asserted by **typed pattern match on the `Diagnostic` enum**, not
   string parsing. `Witness::Violates.reason` is `String` per the
   substrate (`dimensions.dag:35-37`) and the Rust mirror
   (`dimension.rs:46-49`); it is **human-facing only** and tests
   assert at most that it is non-empty / non-fabricated, never that it
   matches a substring or pattern. The typed-diagnostic axis is the
   `DimensionFail.violations` list, not a field on `Witness::Violates`.
5. **Fail-closed propagation** — a program with `Behavior::Loop` of
   `LoopBound::Descent { cluster }` returns `DimensionFail` whose
   `violations` list contains the typed E5 residual `Diagnostic`, not
   a fabricated carrier.
6. **No bridge fabrication** — when `evaluate_body` returns
   `EvalError::UnboundPort` or other E0/E1/E2 fail-closed cases, the
   witness wrapper produces `Witness::Violates { reason, at }` (with
   `reason` a human-facing string) **and** emits a typed `Diagnostic`
   into the eventual `DimensionFail.violations` list. It must never
   return `Witness::Inhabits` over an arbitrary default. The two
   carriers — `Witness` for per-behavior partition, `Diagnostic` for
   typed cause — are coordinate, not redundant; per
   `dimensions.dag:25-32`, "no evidence" and "evidence of violation"
   cannot masquerade as each other.

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
- `Behavior::Bind` body-evaluator coverage — hard prerequisite for programs
  whose selected branch / loop bodies need binding semantics; this is not E6
  lens-fold work.
- PR-B.0 / PR-B.1 — body evaluator semantics + fail-closed catalog.
- `src/v3/std/dimensions.dag` — Witness / DimensionReport /
  AnalysisDimension / Dimension substrate type authority; concrete
  `AnalysisDimension` data instances are deferred in that file.
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
3. **Per-lens carrier types**: at minimum `TenantFlow` / `IfcLabel`, or a
   Director-approved decision that the first executable E7 slice is
   symbolic-cost-only. No silent stand-in carrier.
