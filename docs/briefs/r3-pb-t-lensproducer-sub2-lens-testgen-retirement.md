# R3 T-LensProducer-Retirement — Sub-gate 2: `lens_testgen.rs` retirement (skeleton)

**Status:** PROPOSAL skeleton (planning artifact, dispatch-gated). Authored 2026-04-30 by PB Manager continuation per dispatch on inbox #1149 (R3-continuation-by-design; R2 closed via #1275). Worker dispatch is gated; this brief is read at gate-clear and does NOT instruct implementation against runtime surfaces that don't yet exist.

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation).

**Sub-gate identifier:** `lens_testgen_dot_rs_retired` (per `docs/design-pb-runtime-interpreter.md` §5.1 sub-gate 2).

**Lane size (within T-LensProducer-Retirement XL):** M (~31 KB; testgen-shaped reflection rather than full lens application).

## Purpose

Retire `src/v3/compiler/src/lens_testgen.rs` once PB-Runtime (Item 4) can execute the `.dag` testgen body that replaces it. Per design-doc §5.1 sub-gate 2: "PB-Runtime can execute the `.dag` testgen body that replaces `lens_testgen.rs`." Sub-gate 2 shares Item 4's load-bearing PB-Runtime mechanism with sub-gate 1 — they are tightly coupled (per design-doc §5.1's coupling note: "sub-gates 1 + 2 are tightly coupled (same dissolution mechanism); sub-gate 3 is independent (different mechanism)"). They retire independently against the same mechanism — sub-gate 1 closes lens-application; sub-gate 2 closes testgen.

## Owning lane

T-LensProducer-Retirement (XL, one program) — sub-gate 2. PB Manager owns dispatch; sub-gate 2's progress is reported separately to the closure ledger but does not split into a parallel lane.

## Prerequisites (cumulative)

Same Item-4 prerequisite chain as sub-gate 1, with one difference: sub-gate 2 does NOT depend on the canonical-lens-name dispatch bridge (#1183) the way sub-gate 1 does — testgen consumers are different from `LensOutputEquals` consumers. Per [`docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md) Rows 1+2+4 + design-doc §3:

1. **R2 close** — landed (#1275).
2. **R2-Evaluator stable**.
3. **PR-A.1 (`Value` carrier) + PR-A.2 (`EvalFrame` / `EvalStateStack`) landed**.
4. **Item 4 (PB-Runtime interpreter-as-data) landed**.
5. **Convergence-matrix Row 4 TestClaim `pb_runtime_equivalent_to_evaluator_on_corpus` green** — same evidence base as sub-gate 1; the equivalence demonstrates PB-Runtime's testgen-applicable surface matches R2-Evaluator's.

Sub-gate 2 has **no Item-5 dependency** (that's sub-gate 3's territory).

### Cadence note (not a prerequisite)

Sub-gates 1+2 share Item 4's mechanism. Sub-gate 2 does **not** require sub-gate 1 to land first as a structural prerequisite, but PB Manager may bundle them in adjacent dispatches since the dissolution mechanism is shared (per `r2-pure-bootstrap-manager.md` §"T-LensProducer-Retirement XL framing kept" — the lane stays as one program). This is dispatch cadence, not gating.

## Acceptance shape

- **`lens_testgen_dot_rs_retired`** — `src/v3/compiler/src/lens_testgen.rs` deleted.
- **SG-0 census delta** — `EXPECTED_HAND_AUTHORED_NON_TEST` decreases by 1 (`lens_testgen.rs` removed from the census authority at `src/v3/compiler/tests/integration/sg0_census_test.rs`).
- **Testgen invariants preserved by construction** — every testgen consumer that reached `lens_testgen.rs` now routes through PB-Runtime's `evaluate(...)` of the equivalent `.dag` testgen body (per anti-bridge invariant #4: "no parallel emit logic"; one canonical mechanism).
- **No new `TestPredicate` variant** — escalate per [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure) if a substrate gap surfaces.

The sub-gate is reported as a `.dag` `TestClaim` to the closure ledger; the locked predicate name is `lens_testgen_dot_rs_retired` per design-doc §5.1.

## STOP conditions

Worker MUST STOP and escalate when:

- **PB-Runtime can't express a `lens_testgen.rs` semantic.** If testgen exposes a behavior PB-Runtime's `.dag` `evaluate(...)` cannot express, STOP — Evaluator Manager territory.
- **Testgen consumer surface unmapped at dispatch.** If `lens_testgen.rs` consumers across `test_runner.rs` / `m1_5_testgen_test.rs` / fixtures aren't fully mapped to PB-Runtime equivalents at dispatch time, STOP — sub-gate 2 cannot land while consumers retain Rust-side dependencies.
- **Convergence Row 4 not green.** Same gate as sub-gate 1 — without behavioral receipt, retirement is unjustified.
- **Sub-gate 1 mid-flight blocking.** If sub-gate 1 is in mid-implementation when sub-gate 2 dispatch fires, PB Manager may serialize. Not a structural STOP, but a dispatch-cadence note.

## Non-goals

- **Not** sub-gate 1 (`lens_apply.rs` retirement) — separate skeleton at [`r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](r3-pb-t-lensproducer-sub1-lens-apply-retirement.md). Sub-gates 1+2 share Item 4 mechanism but each retires its own file.
- **Not** sub-gate 3 (`regen_lens.rs` retirement) — distinct mechanism (Item 5 bin-shim emit).
- **Not** PB-Runtime implementation, R2-Evaluator implementation, `BinShim` work, `Lens<C>` carrier-shape edits, `TestPredicate` invention, T-FixedPoint, advanced lifetime analyzer cases.

## Cross-program signals

- **Evaluator Manager** — Item 4 readiness gate.
- **Substrate Manager** — §P1 routing if substrate gap surfaces.
- **R3 Release Manager** — sub-gate progress reporting; ledger row `lens_testgen_dot_rs_retired`.
- **Verification Manager (R3)** — bridge-retirement audit if testgen consumes any bridge surface.
- **Director** — scope-change / §P1 arbitration.

## Dispatch preconditions (live-state checklist for PB Manager)

PB Manager dispatches the sub-gate 2 worker when:

1. R2 close — done via #1275 ✓.
2. R2-Evaluator stable.
3. PR-A.1 + PR-A.2 carriers live on main.
4. Item 4 (PB-Runtime) landed.
5. Convergence Row 4 TestClaim authored and green.
6. Testgen consumer surface mapped (per STOP condition above).

If any of (2)–(6) is unmet, brief stays in PROPOSAL state.

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §3 (Item 4), §5.1 (sub-gate decomposition), §5.2 (SG-0 cascade), §6 (anti-bridge invariants).
- Parent manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md#program-scope-t-pb-post-r1-only) — R3 continuation table row for T-LensProducer-Retirement.
- T-LensProducer-Retirement parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure".
- Convergence matrix: [`docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md) Rows 1+2+4.
- Corpus seed audit: [`docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md).
- Sibling sub-gate skeletons: [`r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](r3-pb-t-lensproducer-sub1-lens-apply-retirement.md), [`r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Sibling PB R3 briefs: [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md), [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md).
- SG-0 census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs` (`EXPECTED_HAND_AUTHORED_NON_TEST`).
- File targeted for retirement (do not edit until dispatch): `src/v3/compiler/src/lens_testgen.rs`.
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
