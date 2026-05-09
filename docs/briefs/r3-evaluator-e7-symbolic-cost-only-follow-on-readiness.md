# R3 Evaluator — E7 Symbolic-Cost-Only Follow-On Readiness

**Status:** AUDIT — docs/test-plan only. Locks the narrow first
executable E7 slice that ships **symbolic-cost-only** before E5
(Loop), `TenantFlow`, or `IfcLabel` carriers land. **No Rust
implementation, no test fixtures, no substrate changes in this slice.**

**Authorities:**
- [`docs/briefs/r3-evaluator-e7-witness-construction-readiness-audit.md`](r3-evaluator-e7-witness-construction-readiness-audit.md)
  (#1452, merged) — parent E7 readiness brief; this brief is the
  narrow follow-on it points at for the symbolic-cost-only entrypoint.
- [`docs/briefs/r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md)
  §E7 — locked acceptance: typed `Diagnostic`, no string parsing of
  `Witness.reason`, fail-closed propagation.
- [`src/v3/std/dimensions.dag`](../../src/v3/std/dimensions.dag) §
  `Witness<Carrier>` (line 35), `DimensionReport<Carrier>` (line 51).
- [`src/v3/compiler/src/dimension.rs::analyze_symbolic_cost_dimension`](../../src/v3/compiler/src/dimension.rs)
  (lines 158-215) — the live symbolic-cost analyzer; wrapper-target
  for the `analyze_complexity` public entrypoint.
- [`src/v3/compiler/src/lens_cost_symbolic_generated.rs:9`](../../src/v3/compiler/src/lens_cost_symbolic_generated.rs)
  — `pub fn symbolic_cost_of(p0: &Dag, p1: &PortId) -> Lookup<SymbolicCost>`,
  the carrier authority `analyze_symbolic_cost_dimension` consumes.

## Why symbolic-cost-only ships before E5 (Loop)

The parent E7 readiness audit framed implementation around an
`analyze_with_evaluator(dag, root, dimension)` that consumes
`evaluate_body` outputs. That path is correctly gated on E5 (Loop)
because lens fold over recursive programs traverses `Loop` nodes.

**Symbolic-cost-only does not need that path.** The existing
`analyze_symbolic_cost_dimension` iterates the reachable behaviors
from `workflow_root` in `d.nodes()` order, filtered by
`workflow_reachable_behavior_ids(d, workflow_root)` (`dimension.rs:163-178`),
and consumes `symbolic_cost_of` for each behavior's result port.
This is the same traversal pattern `behavior_spine_in_node_order`
documents, but the implementation is the inline `d.nodes()` loop
rather than a call to that helper. The
generated symbolic-cost lens itself (`lens_cost_symbolic_generated.rs`)
walks the program DAG structurally — it is **not** the body
evaluator and does **not** dispatch through `eval_node`. So programs
containing `Behavior::Loop` produce a `SymbolicCostLookup::Hit(cost)`
through the lens-spine path even while `eval_node` would
fail-closed `UnsupportedBehavior` on the same node.

The first executable E7 slice can therefore ship today as a thin
public entrypoint over the lens-spine path. When E5 lands, the
`analyze_with_evaluator` form becomes the canonical entrypoint and
the lens-spine wrapper either dissolves or stays as the
not-evaluator-driven path for cost (per `dimension.rs` §preamble).

## First executable slice — `analyze_complexity`

### Public entrypoint signature

```text
pub fn analyze_complexity(
    dag: &Dag,
    workflow_root: NodeId,
) -> DimensionReport<SymbolicCost>;
```

**Implementation:** thin wrapper that delegates to
`analyze_symbolic_cost_dimension(dag, workflow_root)` —
single-authority over the symbolic-cost lens-spine path. **No new
code paths.** The wrapper exists so the E7 public surface is named
the way the dispatch brief / parent readiness audit lock it
(`analyze_complexity` / `analyze_tenant_flow` / `analyze_ifc`),
without introducing a parallel analyzer.

### What does NOT ship in this slice

- No `analyze_tenant_flow` — `TenantFlow` carrier not live.
- No `analyze_ifc` — `IfcLabel` carrier not live.
- No `analyze_with_evaluator` — gated on E5 (Loop).
- No `LensRunnerView<C>` trait — that's the post-E5 generalization
  the parent audit named.
- No new `Witness` / `DimensionReport` variant.
- No new `EvalError` variant.
- No bridge from `evaluate_body` to `Witness<C>` — that's the
  post-E5 work.

### Diagnostic contract (what's already there)

`analyze_symbolic_cost_dimension` already produces typed
`Diagnostic::ParseError` entries in `DimensionFail.violations`
(`dimension.rs:198-219`). The `analyze_complexity` wrapper
inherits this — typed `Diagnostic` is already in place; **no
string parsing of `Witness.reason` is required by tests** (the
existing `Witness::Violates.reason` is a human-facing string per
`dimensions.dag:37` and the audit's discipline). Tests assert by
typed pattern match on `DimensionReport::DimensionOk { composed, .. }`
or `DimensionFail { violations, .. }` and on `Diagnostic` enum
variants.

**Open follow-up (not a blocker for this slice):** the existing
analyzer reuses `Diagnostic::ParseError` as the typed envelope for
symbolic-cost violations. A dedicated `Diagnostic::CostMissing` or
similar variant is more honest, but adding it is a substrate change
routed through the Q6.5 lens-instance path per the parent E7 audit's
STOP+PING. This slice does **not** introduce that variant; it
documents the call-out so a later slice can route it cleanly.

## Test plan — first executable slice

Six tests, all over the public `analyze_complexity` entrypoint. No
new test fixtures beyond what the existing `dimension.rs` analyzer
already exercises in its tests (the wrapper has the same input/output
shape as `analyze_symbolic_cost_dimension`).

1. **`analyze_complexity_returns_ok_for_known_workflow`** — pick a
   bounded program with all behaviors covered by `symbolic_cost_of`;
   assert `DimensionReport::DimensionOk { dimension_name:
   "symbolic_cost", composed: SymbolicCost::… }` matches the
   existing `analyze_symbolic_cost_dimension` result for the same
   `(dag, workflow_root)`. Cross-check against the live analyzer to
   pin single-authority.
2. **`analyze_complexity_fails_closed_on_missing_cost`** — program
   that hits `SymbolicCostLookup::Miss` at some behavior; assert
   `DimensionReport::DimensionFail { violations: vec![..], .. }`
   with at least one `Diagnostic::ParseError { message, span, .. }`
   matching the missing-cost message shape (typed pattern match on
   `Diagnostic` enum variant + structural `span` check; **no string
   parsing of `Witness.reason`**).
3. **`analyze_complexity_fail_does_not_fabricate_composed`** —
   `DimensionFail` arm verified to carry `violations` and
   `witnesses`; assert by exhaustive pattern that `composed` field
   is unreachable on the `DimensionFail` arm (substrate guarantees
   the partition; this test pins the wrapper preserves it).
4. **`analyze_complexity_includes_loop_node_under_lens_spine`** —
   program with a `Behavior::Loop` whose result port has a
   well-defined symbolic cost; assert `DimensionOk` with the
   computed cost. Pins that **the wrapper does NOT depend on E5
   (Loop) `eval_node` coverage** — `Behavior::Loop` is reached via
   the lens-spine, not `evaluate_body`. (If this test fails when
   E5 lands and `eval_node` is wired in for Loop, the wrapper has
   silently switched to the evaluator-driven path; the test
   protects single-authority.)
5. **`analyze_complexity_witness_reason_is_not_machine_parsed`** —
   negative test asserting that no test in this module asserts
   `Witness::Violates.reason` matches a substring or regex.
   Implemented as a small grep-style assertion over the test
   module's own source via `include_str!` if convenient, or as a
   prose comment + reviewer-enforced rule. (Not a runtime test;
   listed for discipline.)
6. **`analyze_complexity_diagnostic_is_typed_only`** — assert each
   entry in `DimensionFail.violations` is a `Diagnostic` enum
   variant via exhaustive pattern; no `to_string()` /
   `format!("{}", …)` / regex inspection of `message` field beyond
   non-empty checks for human-facing copy.

## Hard prerequisite for this slice

**None today.** The lens-spine path
(`analyze_symbolic_cost_dimension` + `symbolic_cost_of`) is live and
covers symbolic-cost over all five `Behavior` variants including
`Loop`. The slice is a thin wrapper; the wrapper itself has zero new
substrate or evaluator dependencies.

## Hard prerequisites that REMAIN gated (for the post-this-slice E7 work)

These are the **next** layers of E7, NOT in this slice:

- **E5 (Loop) `eval_node` coverage** — for the `analyze_with_evaluator`
  path that consumes `evaluate_body` instead of the lens-spine.
- **`Behavior::Bind` `eval_node` coverage** — same path; both Loop and
  Bind are currently `EvalError::UnsupportedBehavior`.
- **`TenantFlow` carrier** — for `analyze_tenant_flow` entrypoint.
- **`IfcLabel` carrier** — for `analyze_ifc` entrypoint.
- **`Diagnostic::CostMissing` (or substrate-routed equivalent)** —
  optional follow-up to drop the `Diagnostic::ParseError` reuse.
- **Bool-as-Disj reification bridge** (Substrate #1130) — for
  evaluator-driven analyses over Bool branches.

## STOP+PING boundary (this slice, when implemented)

- **No new `analyze_*` entrypoint** beyond `analyze_complexity`.
- **No `analyze_with_evaluator`** — that's the post-E5 form.
- **No new `Diagnostic` variant** — reuse existing
  `Diagnostic::ParseError` as the analyzer already does; routing a
  dedicated variant is a separate substrate slice.
- **No new `Witness` / `DimensionReport` variant** — TERMINAL per
  `dimensions.dag:35,51`.
- **No `LensRunnerView<C>` trait** — that's the post-E5
  generalization.
- **No `TenantFlow` / `IfcLabel` carrier introduction** — wait for
  the substrate slices.
- **No string parsing of `Witness.reason`** in any test or wrapper.

## Acceptance gates (this brief)

- ✅ `analyze_complexity` public-entrypoint signature locked.
- ✅ Six tests enumerated with explicit typed-pattern + non-string-
  parsing discipline.
- ✅ Hard prerequisite for this slice = NONE; pre-E5 by design.
- ✅ Post-this-slice prerequisites named and routed (E5, Bind,
  TenantFlow, IfcLabel, CostMissing, Bool bridge).
- ✅ STOP+PING boundary names every shape change a worker must not
  make silently while implementing.
- ✅ Docs/test-plan only; no Rust, no fixtures, no substrate.

## Out of scope (this brief)

- Implementing `analyze_complexity`.
- Implementing any post-E5 follow-on.
- Editing `dimension.rs::analyze_symbolic_cost_dimension`.
- Authoring TenantFlow / IfcLabel carriers.
- Routing `Diagnostic::CostMissing`.
- Touching the Bool-as-Disj bridge.
- Editing parent E7 readiness audit (#1452) — its content stands.
