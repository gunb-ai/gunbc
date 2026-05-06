# R3 T-LensProducer-Retirement — Sub-gate 1: `lens_apply.rs` retirement (skeleton)

**Status:** PROPOSAL skeleton (planning artifact, dispatch-gated). Authored 2026-04-30 by PB Manager continuation per dispatch on inbox #1149 (R3-continuation-by-design; R2 closed via #1275). **Worker dispatch is gated** — see §"Dispatch preconditions" + §"STOP conditions". This brief is read at gate-clear; it does NOT instruct implementation against runtime surfaces that don't yet exist.

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per `r3-structure.md` §"Manager structure" Item 1).

**Sub-gate identifier:** `lens_apply_dot_rs_retired` (per `docs/design-pb-runtime-interpreter.md` §5.1 sub-gate 1).

**Lane size (within T-LensProducer-Retirement XL):** L (the largest of the three retirement files at 107 KB; `reflect_program_dag_nodes_in_file` + `apply_lens_declaration` substrate-of-substrate).

## Purpose

Retire `src/v3/compiler/src/lens_apply.rs` once PB-Runtime (Item 4 per [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md)) can execute the `.dag` lens-application body that replaces `lens_apply.rs`'s two load-bearing functions:

- `reflect_program_dag_nodes_in_file(...)` — reflects program nodes into `FieldValue` form for lens consumption (per anti-bridge invariant #1: P2 cross-`Dag` reflection-coherence; same constraint named in `r2-pb-canonical-lens-bridge-disposition.md`).
- `apply_lens_declaration(...)` — folds a lens body over a reflected input.

Both currently exist as hand-authored Rust. Sub-gate 1 closes when PB-Runtime's `.dag` `evaluate(...)` (per design doc §3.2) covers the same dispatch surface and the file is deletable. Per design-doc §5.2: this is the largest single class of hand-Rust the SG-0 = 0 cascade retires.

## Owning lane

T-LensProducer-Retirement (XL, kept as one program per Director cascade Item 8) — sub-gate 1 of the three internal sub-gates. PB Manager owns dispatch; sub-gate 1 lands within the one-program lane and is reported separately to the closure ledger but does not split into a parallel lane.

## Prerequisites (cumulative; all must be live at dispatch time)

Per `design-pb-runtime-interpreter.md` §5.1 sub-gate 1 row + §3 (Item 4) + the convergence-matrix prerequisite-state table at [`docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md):

1. **R2 close** — landed (#1275; verified at branch creation 2026-04-30).
2. **R2-Evaluator stable** (PR-A through PR-E lane closed per [`docs/briefs/r2-evaluator-manager.md`](r2-evaluator-manager.md)).
3. **PR-A.1 (`Value` carrier) + PR-A.2 (`EvalFrame` / `EvalStateStack`) landed** — convergence-matrix Rows 1+2.
4. **Item 4 (PB-Runtime interpreter-as-data) landed** — `pb_runtime_evaluate` executable; convergence-matrix Row 4 forward-ref resolved on the PB-Runtime side.
5. **Convergence-matrix Row 4 TestClaim `pb_runtime_equivalent_to_evaluator_on_corpus` green** — Seed (3) one `Lens<C>` instance application demonstrates PB-Runtime's lens-application surface matches R2-Evaluator's; per [`docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md) Seed (3). **Without this, retiring `lens_apply.rs` would break canonical-lens dispatch in `test_runner.rs` per the disposition in [`r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md).**
6. **Canonical-lens-name dispatch bridge retired or migrated** — per #1183 disposition: `R1_CANONICAL_*_LENS` `include_str!`s + `lens_decl.name == Some("...")` arms in `test_runner.rs` route through PB-Runtime's structural dispatch instead. Sub-gate 1 implementation IS the dissolution path for that disposition's Path (a) "PB-Runtime interpreter-as-data."

## Acceptance shape

- **`lens_apply_dot_rs_retired`** — `src/v3/compiler/src/lens_apply.rs` deleted.
- **SG-0 census delta** — `EXPECTED_HAND_AUTHORED_NON_TEST` decreases by 1 (`lens_apply.rs` removed from the census authority at `src/v3/compiler/tests/integration/sg0_census_test.rs`).
- **Lens-application invariants preserved by construction** — every lens consumer that reached `apply_lens_declaration(...)` now routes through PB-Runtime's `evaluate(...)` (per anti-bridge invariant #4: "no parallel emit logic"; the `.dag` evaluator IS the dispatch path, not a parallel one). Behavioral equivalence proven by the convergence-matrix Row 4 TestClaim landing green pre-deletion.
- **No new `TestPredicate` variant** introduced from this lane. If the equivalence acceptance requires a substrate gap, escalate per [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure) to Substrate Manager.
- **Canonical-lens bridge ratchet** at `src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs` (#1183) lowers (or its companion deletion lands in the same PR) — that ratchet's pinned counts collapse when `lens_apply.rs` retires.

The sub-gate is reported as a `.dag` `TestClaim` to the closure ledger; the locked predicate name is `lens_apply_dot_rs_retired` per design-doc §5.1.

## STOP conditions

Worker MUST STOP and escalate to PB Manager when any of these surface during implementation:

- **PB-Runtime can't express a `lens_apply.rs` semantic.** If `reflect_program_dag_nodes_in_file` or `apply_lens_declaration` exposes a behavior PB-Runtime's `.dag` `evaluate(...)` cannot express (per anti-bridge invariant #1: PB-Runtime ≡ R2-Evaluator's runtime model expressed as `.dag`), STOP — that's a substrate-semantic decision in Evaluator Manager territory.
- **Canonical-lens bridge dependency unmet.** If `test_runner.rs`'s name-dispatched canonical lenses (`named_function_count` / `cost_of` per #1183 §"What remains") still depend on `lens_apply.rs` at retirement time, STOP — sub-gate 1 cannot land while consumers still reach the file.
- **Reflection-coherence regression.** If retiring `lens_apply.rs` breaks the P2 cross-`Dag` reflection invariant (per `r2-pb-canonical-lens-bridge-disposition.md` §"Why these survive — the substrate gap"), STOP — that's the invariant the lens-bridge disposition explicitly named, and sub-gate 1's whole job is to dissolve it.
- **Convergence Row 4 not green.** If Row 4 TestClaim `pb_runtime_equivalent_to_evaluator_on_corpus` is not yet green at dispatch, retirement would be unjustified by behavioral receipt. STOP and wait.
- **Bin-shim retirement (sub-gate 3) presupposed.** Sub-gate 1 does NOT depend on sub-gate 3; if a worker finds themselves needing `regen_lens.rs` retirement to land first, that's a misread — sub-gates 1+2 are independent of sub-gate 3 per design-doc §5.1's coupling note.

## Non-goals

- **Not** sub-gate 2 (`lens_testgen.rs`) — separate skeleton at [`r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md). Sub-gate 1 + 2 share Item 4 mechanism but each retires its own file independently.
- **Not** sub-gate 3 (`regen_lens.rs`) — distinct mechanism (Item 5 bin-shim emit, not Item 4 PB-Runtime); see [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) for the underlying pattern and [`r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md) for the sub-gate 3 skeleton.
- **Not** PB-Runtime implementation — Item 4 prerequisite per design-doc §3.
- **Not** R2-Evaluator implementation — Evaluator Manager territory per [`r2-evaluator-manager.md`](r2-evaluator-manager.md).
- **Not** `BinShim` instance authoring or carrier-shape edits — see binshim brief.
- **Not** `Lens<C>` substrate carrier-shape edits — Substrate Manager territory.
- **Not** `TestPredicate` invention — §P1 escalation route only.
- **Not** T-FixedPoint implementation — separate planning brief.
- **Not** advanced lifetime analyzer cases d/e/f folding — per the PB Manager brief's [`Owns (R3 continuation — Director cascade Item 4 + Item 8 ratified 2026-04-28)`](r2-pure-bootstrap-manager.md#program-scope-t-pb-post-r1-only) table, those land alongside retirement; this skeleton stays scoped to retirement mechanics. The lifetime-analyzer-handles-closures-of-async-self-referential-state work is its own dispatch chain.

## Cross-program signals

- **Evaluator Manager** — runtime-value model convergence (anti-bridge invariant #1; matrix Row 1). Pre-dispatch readiness signal: PR-A.1 + PR-A.2 + Item 4 landed.
- **Substrate Manager** — `Lens<C>` carrier shape; any new `TestPredicate` variant (§P1 routing).
- **R3 Release Manager** — sub-gate progress reporting per PB Manager brief §"Reporting cadence" (sub-gate progress within the one-program lane; ledger row `lens_apply_dot_rs_retired`).
- **Verification Manager (R3)** — `bridge_retirement_ledger_zero` audit; canonical-lens-bridge dispositional signal flows here too once #1183's bridge dissolves.
- **Director** — scope-change escalations (e.g., a `lens_apply.rs` semantic that proves substrate-shape-dependent rather than PB-Runtime-expressible); §P1 arbitration if Substrate disposition stalls.

## Dispatch preconditions (live-state checklist for PB Manager)

PB Manager dispatches the sub-gate 1 worker when **a single readiness check** of the R3 closure ledger (or equivalent live state) shows all of:

1. R2 close — done via #1275 ✓.
2. R2-Evaluator stable.
3. PR-A.1 + PR-A.2 carriers live on main.
4. Item 4 (PB-Runtime) landed and `pb_runtime_evaluate` resolvable.
5. Convergence Row 4 TestClaim authored and green.
6. Canonical-lens-bridge dependency surface migrated or ready to retire in the same PR.

If any of (2)–(6) is unmet, this brief stays in PROPOSAL state; PB Manager does not dispatch.

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §3 (Item 4 PB-Runtime), §5.1 (sub-gate decomposition), §5.2 (SG-0 cascade), §6 (anti-bridge invariants).
- Parent manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md#program-scope-t-pb-post-r1-only) — R3 continuation table row for T-LensProducer-Retirement.
- T-LensProducer-Retirement parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure".
- Convergence matrix: [`docs/briefs/r2-pb-runtime-evaluator-convergence-matrix.md`](r2-pb-runtime-evaluator-convergence-matrix.md) Rows 1+2+4.
- Corpus seed audit (Row-4 expansion): [`docs/briefs/r3-pb-runtime-equivalence-corpus-seed-audit.md`](r3-pb-runtime-equivalence-corpus-seed-audit.md) Seed (3) `Lens<C>` instance.
- Canonical-lens bridge disposition (#1183): [`docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`](r2-pb-canonical-lens-bridge-disposition.md) — sub-gate 1 IS the dissolution path for the canonical-lens-name dispatch bridge.
- Sibling sub-gate skeletons: [`r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md), [`r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Sibling PB R3 briefs: [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md), [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md).
- SG-0 census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs` (`EXPECTED_HAND_AUTHORED_NON_TEST`).
- File targeted for retirement (do not edit until dispatch): `src/v3/compiler/src/lens_apply.rs`.
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
