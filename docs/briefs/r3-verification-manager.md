# R3 Verification Manager Brief

**Status:** PROPOSAL / pre-dispatch. Authored for R3 spin-up per [`docs/r3-structure.md`](../r3-structure.md) §"Manager structure" Item 2 and Director R3 verification dispatch. Worker implementation remains gated on the prerequisite landings named below.

**Owning manager:** R3 Verification Manager.

## Orient before reading

- **R3 structure authority:** [`docs/r3-structure.md`](../r3-structure.md). Verification owns `T-Verification-L4-L7-Direct`, `T-Verification-L5-Corpus`, and the ledger-only gate for `T-Bridge-Retirement`.
- **Formal-grounding absorption:** this manager also tracks the R3 formal-grounding TestClaim bundle (TC1 / TC2 / TC3) because those claims are verification-surface obligations, even when their declarative shape was authored by Substrate, Evaluator, or PB work before manager spawn.
- **Not in scope:** L6 structural-form coverage (R2 Grounding), cost-lens authoring (Substrate continuation), or actual bridge-retirement implementation (distributed to Substrate / PB natural owners).

## Program scope

R3 Verification closes the runtime-verification surface of the thesis:

| Lane / gate | Status at authoring | Scope |
|---|---|---|
| **T-V-L4-L7-Direct** | STANDBY | L4 emit/eval match plus L7 algebraic-law witnesses. Evaluator-direct runtime harness; consumes `DifferentialEquals` / `AlgebraicLaw` style primitives but is not a `Lens<C>` instance. |
| **T-V-L5-Corpus** | STANDBY | L5 cross-target consistency over the certification corpus. Sequentially gated on L4/L7 Direct landing and all required Shape A grounding. |
| **T-FormalGrounding-Verification** | TRACKING / author-now-fire-later | TC1 eta-equivalence, TC2 evaluation-order independence, TC3 strong normalization. Maintains claim state, substrate gaps, and transition rules. |
| **`bridge_retirement_ledger_zero`** | TRACKING | Unified five-bridge map audit. Verification owns ledger truth, not the bridge retirements themselves. |

## Dispatch preconditions

`T-V-L4-L7-Direct` dispatch waits for:

1. R2-Evaluator landing with the body evaluator and witness construction surfaces needed by L4 and L7.
2. R2-Evaluator PR-A.3 strategy / memoization carriers no longer blocked by the single-variant sum parser gap.
3. The body-evaluator baseline strong enough to execute certification-corpus programs rather than fixture-only stubs.
4. R2-T-Substrate-Lens-Primitive available where claims consume witness/lens substrate as inputs.

`T-V-L5-Corpus` dispatch waits for:

1. `T-V-L4-L7-Direct` corpus exists and is green.
2. R2-Grounding Rust + Python + Go Shape A targets are closed for the target set under test.
3. Cross-target equivalence harness primitives from [`r2-pr-d-cross-target-equivalence-harness-primitives.md`](r2-pr-d-cross-target-equivalence-harness-primitives.md) have strict target receipts, not only slice-0/scaffold claims.

Formal-grounding claim activation waits on the per-claim prerequisites in [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md).

## Owned deliverables

- Worker brief for L4/L7 Direct: [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md).
- Worker brief for L5 Corpus: [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md).
- Formal-grounding TestClaim bundle: [`r3-v-formal-grounding-tc-bundle.md`](r3-v-formal-grounding-tc-bundle.md).
- Ledger audit rows for the five bridge retirements, including current partial/blocked disposition.

## Acceptance gates

- `l4_emit_eval_match` — for every certification-corpus program and materialized target, emitted target behavior algebraically equals `.dag` evaluation output.
- `l7_algebraic_laws_witnessed` — algebraic laws in `dsl/std/algebra.dag` have Evaluator-constructed witnesses rather than host-only assertions.
- `l5_cross_target_consistency` — for every certification-corpus program, Rust/Python/Go emitted programs produce algebraically equivalent behavior.
- `tc1_substrate_lens_eta_equivalence` — existing deferred fixture remains scoped and valid until strict activation.
- `evaluation_order_independent_lens_results` — TC2 deferred fixture strengthens to strict strategy-output equality once executable strategies exist.
- `every_typed_dag_program_terminates_in_bounded_steps` — TC3 remains text-form until a substrate quantifier / proof encoding exists.
- `bridge_retirement_ledger_zero` — all five named bridge rows read retired/zero with their natural-owner ratchets green.

## Bridge-retirement ledger audit

| Bridge | Natural owner | Current disposition |
|---|---|---|
| `SourceSpan.file` participation checks | Substrate | R3-deferred. Production participation still keyed by path/span in lens reflection, lower, and emit paths; structural prerequisites are module/compilation-unit identity plus typed authority / emit-scope carriers. |
| `mark_bootstrap_secret_nominal_opacity()` | Substrate | Retired on main; source-level nominal-opacity authority is live, and `bridge_mark_bootstrap_secret_nominal_opacity_retired` covers the row. |
| Canonical lens-name dispatch | PB | Partially retired. Remaining canonical lens bytes, name-dispatch arms, and name-keyed lookups are pinned by `canonical_lens_bridge_ratchet_test`; full close waits on PB-Runtime interpreter-as-data or typed lens registry substrate. |
| `include_str!` side channels | PB | Open for `pipeline_authority`: runtime ordering reads `PipelineStageBinding`, but `compile` remains `ArrowBody::Unparsed`; full retirement waits on derivation or a structural compile-body witness. |
| `patch_lower_helpers_*` residual | PB | Lower-helper post-process bridge is zero-residual and ratcheted by `bridge_lower_helpers_patch_zero_residual_test`; other exact-string patching classes are outside that row. |

## Reporting cadence

- Lane close and bridge-ledger state changes go to the R3 Release / closure ledger owner when authored.
- Cross-program bridge blockers go to the natural owner (Substrate or PB) with the specific row and unblock condition.
- TC substrate gaps are escalated through `INVARIANTS.md` §P1; Verification does not invent staging variants to force early claims.

## Cross-refs

- [`docs/r3-structure.md`](../r3-structure.md) — R3 lane authority and bridge map.
- [`docs/design-emission-model.md`](../design-emission-model.md) — L4/L5/L7 verification semantics and no-engine discipline.
- [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md) — Evaluator prerequisites and TC2 hook.
- [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) — PB-owned bridge rows.
- [`docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md) — canonical lens bridge state.
