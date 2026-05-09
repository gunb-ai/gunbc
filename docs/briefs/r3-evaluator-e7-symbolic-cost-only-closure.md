# R3 Evaluator — E7 Symbolic-Cost-Only Closure & Downstream Handoff

**Status:** CLOSURE NOTE — docs-only. Records the landed E7
symbolic-cost-only surface, names what downstream consumers can rely
on today, and enumerates the exact gates that block the next E7
implementation slice. **No Rust, no substrate, no fixture changes
land in this slice.**

**Scope ratchet.** This brief closes the symbolic-cost-only
sub-program inside PR-E E7. It does **not** broaden into
`analyze_with_evaluator`, `LensRunnerView<C>`, TenantFlow/IFC, witness
fold over multiple lenses, or any Bool-as-Disj / Substrate / runner
work. Those wait on the gates §below.

## Landed surface

| Slice                                                 | PR     | What it adds                                                                                                  |
|------------------------------------------------------|--------|---------------------------------------------------------------------------------------------------------------|
| Parent E7 readiness audit                             | #1452  | Locks the E7 design surface; names `analyze_with_evaluator` as post-E5 work; sets STOP+PING boundaries.       |
| Symbolic-cost-only follow-on readiness                | #1471  | Locks the `analyze_complexity` wrapper signature; six-test plan; pre-E5 by design (lens-spine, not body-eval).|
| `analyze_complexity` first executable slice           | #1484  | `pub fn analyze_complexity(dag, workflow_root) -> DimensionReport<SymbolicCost>` thin wrapper over the live `analyze_symbolic_cost_dimension`; five in-module tests covering happy path, fail-arm, sentinel, and typed-diagnostic discipline. |
| Public-API integration coverage                       | #1503  | Three integration tests (`tests/integration.rs::e7_analyze_complexity_integration`) over the public crate API: delegation pin, lens cross-check, success-path typed witness envelope. |
| Root-selection observability follow-up                | #1505  | Adds `analyze_complexity_public_api_honors_supplied_workflow_root` over a two-bind program; per-root delegation includes per-witness `Inhabits(SymbolicCost)` content equality.                                  |

## What downstream consumers can rely on today

After all five PRs merged, consumers outside the `dimension` module
(downstream R3 lanes, lens producers, future analyzers) can rely on:

1. **`v3_compiler::analyze_complexity(dag: &Dag, workflow_root: NodeId) -> DimensionReport<SymbolicCost>`** as the named, public, single-authority entrypoint for symbolic-cost analysis. Re-exported from the crate root (`src/v3/compiler/src/lib.rs::analyze_complexity`).
2. **Single-authority delegation** to `analyze_symbolic_cost_dimension`. The wrapper has no parallel implementation; a regression that diverged the wrapper from the underlying analyzer would fail the integration delegation test.
3. **`workflow_root` is observable.** Per #1505: distinct roots produce distinct reachable-spine sizes (and per-witness contents on the `Inhabits` arm). A wrapper that ignored the root would fail the regression.
4. **Typed `DimensionReport<C>` envelope.** Coproduct `DimensionOk { dimension_name, composed, witnesses } | DimensionFail { dimension_name, violations: Vec<Diagnostic>, witnesses }` (`src/v3/std/dimensions.dag:51-61`, mirrored at `src/v3/compiler/src/dimension.rs:58-69`). Pass/fail partition is structural; consumers must pattern-match the variant.
5. **Typed `Witness<C>` per behavior.** `Inhabits(C) | Violates { reason: String, at: Behavior }`. `Witness::Violates.reason` is **human-facing only** — consumers must not parse the string. Diagnostic typing flows through `DimensionFail.violations: Vec<Diagnostic>`, where each entry is a typed `Diagnostic` enum variant.
6. **No fabricated carriers on failure.** R2 fail-closed: `DimensionFail` never carries a `composed: C`. Consumers can rely on the absence of that field as the structural failure signal.
7. **Pre-E5 by design.** `analyze_complexity` consumes the lens-spine path (`symbolic_cost_of` walks the DAG structurally), not `evaluate_body`. Programs containing `Behavior::Loop` produce a defined symbolic-cost result today even though `eval_node` dispatches Loop only via E5. Lens-spine remains the single-authority complexity entrypoint until `analyze_with_evaluator` lands.

## What the next E7 implementation slice is gated on

The substrate / API gates §below block any further executable E7
work. Each gate has been routed by parent disposition; this list is
the consolidated handoff index.

| Gate                                              | Routing                                                                       | Unblocks                                                                                |
|---------------------------------------------------|-------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------|
| **`LensRunnerView<C>` adapter**                   | Substrate-routed per parent disposition + parent E7 readiness audit (#1452).  | `analyze_with_evaluator(dag, root, dimension)` — the body-evaluator-driven generalization. |
| **`TenantFlow` carrier**                          | Substrate.                                                                    | `pub fn analyze_tenant_flow(dag, root) -> DimensionReport<TenantFlow>`.                  |
| **`IfcLabel` carrier**                            | Substrate.                                                                    | `pub fn analyze_ifc(dag, root) -> DimensionReport<IfcLabel>`.                            |
| **Additional `AnalysisDimension<C>` data instances** | Class-5 record-body gap per `dimensions.dag:10` "**Deferred:**" line.       | Witness-monoid lifting / report aggregation across multiple lenses.                      |
| **Typed cost-missing `Diagnostic` variant**       | INVARIANTS §P1 substrate-fact-introduction (Q6.5 lens-instance path).         | Drop the `Diagnostic::ParseError` reuse for symbolic-cost violations; clean diagnostic surface for downstream Q6.5 consumers. |
| **Bool-as-Disj reification bridge**               | Substrate #1130.                                                              | Body-evaluator-driven analyses over Bool branches (PR-E E4 + analyze_with_evaluator).    |

Any worker who wants to ship the next E7 implementation slice must
either (a) consume one of these gates after it lands, or (b) STOP+PING
to escalate before crossing a fence.

## Out of scope (this brief)

- New Rust implementation, including any analyzer wrapper beyond
  `analyze_complexity`.
- New `Witness` / `DimensionReport` / `Diagnostic` variants.
- Any `TenantFlow` / `IfcLabel` placeholder.
- Any change to `dimension::analyze_symbolic_cost_dimension` semantics.
- E6 (Bind) fold, E5 (Loop) widening, runner / substrate / Bool work.
- New test fixtures.

## STOP+PING boundary (consumers using the landed surface)

Downstream consumers writing code against `analyze_complexity` must
STOP+PING (rather than locally work around) when:

- The shape of `DimensionReport`'s arms drifts from the locked
  partition (would indicate a substrate change without coordination).
- A consumer needs to distinguish the kind of cost-missing failure
  beyond the typed `Diagnostic::ParseError` envelope today emits — that
  needs the typed `Diagnostic::CostMissing`-style variant routed
  through P1, not local consumer-side string parsing of the `message`
  field.
- A consumer needs cost analysis over a program that requires
  `analyze_with_evaluator` semantics (recursive composition through
  `evaluate_body`) — that needs the `LensRunnerView<C>` gate.

## Closure receipt

This brief closes the symbolic-cost-only sub-program of PR-E E7. The
five PRs above land the entire near-term executable surface; the
next E7 implementation slice is genuinely gated by substrate work
that does not belong in PR-E. Workers reaching this brief looking
for the next E7 task should consult the gates table above to find
which substrate slice unblocks their target, or consult the parent
E7 readiness audit (#1452) for the broader design surface.
