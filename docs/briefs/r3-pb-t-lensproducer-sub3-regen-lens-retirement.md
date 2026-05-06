# R3 T-LensProducer-Retirement — Sub-gate 3: `regen_lens.rs` retirement (skeleton)

**Status:** PROPOSAL skeleton (planning artifact, dispatch-gated). Authored 2026-04-30 by PB Manager continuation per dispatch on inbox #1149 (R3-continuation-by-design; R2 closed via #1275). Worker dispatch is gated; this brief is read at gate-clear and does NOT instruct implementation against runtime surfaces or `BinShim` instances that don't yet exist.

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation).

**Sub-gate identifier:** `regen_lens_dot_rs_retired` (per `docs/design-pb-runtime-interpreter.md` §5.1 sub-gate 3).

**Lane size (within T-LensProducer-Retirement XL):** S (~9 KB; bin-shim shape, not full reflection/application substrate).

## Purpose

Retire `src/v3/compiler/src/bin/regen_lens.rs` once the **Item 5 bin-shim emit pattern** lands per [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4. Per design-doc §5.1 sub-gate 3 row: "`regen_lens` ships as `data regen_lens_shim: BinShim = { ... }` + emitted Rust; equivalence-verified vs the current hand-Rust shim."

Sub-gate 3 is **independent of sub-gates 1 + 2** per design-doc §5.1's coupling note. Sub-gates 1+2 consume Item 4 (PB-Runtime); sub-gate 3 consumes Item 5 (bin-shim emit pattern). They share the parent lane (T-LensProducer-Retirement XL) but not the dissolution mechanism. Sub-gate 3 is also the canonical first slice of the broader bin-shim retirement program (per [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) §"First slice — `regen_lens.rs`").

## Owning lane

T-LensProducer-Retirement (XL, one program) — sub-gate 3. Implementation is dispatched off the BinShim retirement program ([`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md)) — sub-gate 3 IS the BinShim brief's first slice. PB Manager owns dispatch.

## Prerequisites (cumulative)

Per `design-pb-runtime-interpreter.md` §5.1 sub-gate 3 + §4 (Item 5) + the BinShim retirement brief's §"Dependencies / gates":

1. **R2 close** — landed (#1275).
2. **R2-Evaluator stable**.
3. **Item 4 (PB-Runtime interpreter-as-data) landed** — needed transitively because Item 5's emitter uses the same fold-over-substrate pattern as other `.dag` emitters (per anti-bridge invariant #4: "no parallel emit logic").
4. **Substrate-owned `BinShim` carrier type live on main** — `type BinShim { ... }` declared per design-doc §4.2. Substrate Manager owns the carrier-type shape; PB owns instance authoring.
5. **`std.process.ProcessExit` substrate live** — design-doc §4.2 names this as the structural contract for translating `.dag` program return values to host process exit codes.
6. **BinShim emit pattern landed** — the `.dag` emitter program that translates `BinShim` declarations to Rust files (per design-doc §4.2; analogous to `dsl/extdeps/languages/rust/emit.dag`). This is the BinShim retirement brief's deliverable 2.
7. **`data regen_lens_shim: BinShim = { ... }` instance authored** — under `dsl/std/runtime/bin_shims/regen_lens.dag` (path TBD per BinShim brief). This is the BinShim retirement brief's deliverable 1.

Sub-gate 3 has **no Row-4-equivalence dependency** (different mechanism) — its acceptance receipt is the BinShim equivalence fixture per design-doc §7.2, NOT the convergence-matrix Row 4.

## Acceptance shape

- **`regen_lens_dot_rs_retired`** — `src/v3/compiler/src/bin/regen_lens.rs` deleted.
- **SG-0 census delta** — `EXPECTED_HAND_AUTHORED_NON_TEST` decreases by 1 (`bin/regen_lens.rs` removed from the census authority); `GENERATED_FILES` (via `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`) grows by 1 to register the emitted shim path. Updates land atomically in the same PR.
- **Behavioral equivalence fixture green** — `regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust` (per design-doc §7.2; locked TestClaim name). Behavioral equivalence, NOT byte-identity (per anti-bridge invariant #1 framing in §7.2: hand-authored Rust and `.dag`-emitted Rust differ in formatting / `// AUTO-GENERATED` header).
- **Generated file is not editable authority** — emitted `regen_lens.rs` carries `// AUTO-GENERATED from <path> — DO NOT EDIT.` header per design-doc §4.2. SG-0 census already enforces "hand-authored `// AUTO-GENERATED` header does not slip through."
- **Closed-set discipline preserved** — sub-gate 3 closes one element; the broader closed-set discipline (`no_new_bin_shim_hand_rust` per design-doc §7.3) remains a separate gate dependent on Substrate Manager's §7.3 `CensusListConstant`/filter disposition.

## STOP conditions

Worker MUST STOP and escalate when:

- **`BinShim` carrier-shape pressure.** If authoring `data regen_lens_shim: BinShim = { ... }` reveals the locked carrier shape cannot express `regen_lens`'s pipeline composition, STOP — that's substrate-owned per `design-pb-runtime-interpreter.md` §5.4 boundary; route to Substrate Manager via [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
- **Emit-pattern divergence.** If the BinShim emitter authoring path needs to diverge from `dsl/extdeps/languages/rust/emit.dag` shape (per anti-bridge invariant #4: "the `BinShim` emitter is one of many `.dag` emitters; its shape mirrors `dsl/extdeps/languages/rust/emit.dag`"), STOP — that's a sign the carrier or pattern is wrong.
- **Equivalence fixture authoring needs new substrate.** If the §7.2 fixture cannot compose from existing `TestPredicate` variants at `src/v3/std/verification.dag`, STOP — §P1 escalation, not in-this-lane substrate authoring.
- **§7.3 closed-set fixture pressure.** Sub-gate 3 retires one element; if the worker finds themselves trying to author the broader `no_new_bin_shim_hand_rust` gate (which depends on Substrate Manager's §7.3 disposition), STOP — that's outside sub-gate 3's scope.
- **SG-0 census drift the wrong way.** Per `feedback_ratchet_only_down`: if the retirement PR doesn't net-decrease the hand-Rust bin-shim count, that's a defect.

## Non-goals

- **Not** sub-gates 1 + 2 (`lens_apply.rs`, `lens_testgen.rs`) — different mechanism; see [`r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](r3-pb-t-lensproducer-sub1-lens-apply-retirement.md) and [`r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md).
- **Not** the broader BinShim retirement program — see [`r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md). Sub-gate 3 is only the `regen_lens.rs` first slice; subsequent bin-shims (other `regen_*` drivers, `self_host_fixed_point.rs`-shaped shims) follow under the BinShim brief's program.
- **Not** the `BinShim` carrier-type shape edits — Substrate Manager territory per design-doc §5.4.
- **Not** the `.dag` BinShim instance authoring (`data regen_lens_shim: BinShim`) — that's the BinShim brief's deliverable 1; consumed here at dispatch.
- **Not** the BinShim emitter authoring — BinShim brief's deliverable 2; consumed here at dispatch.
- **Not** `no_new_bin_shim_hand_rust` (§7.3) authoring — Substrate-Manager-prereq lane.
- **Not** T-FixedPoint, PB-Runtime implementation, R2-Evaluator implementation, advanced lifetime analyzer cases.

## Cross-program signals

- **Evaluator Manager** — Item 4 readiness (transitive prereq for the emit pattern).
- **Substrate Manager** — `BinShim` carrier-type shape; `std.process.ProcessExit`; §7.3 `CensusListConstant`/filter disposition (informationally — §7.3 is not sub-gate 3's deliverable).
- **R3 Release Manager** — sub-gate progress reporting; ledger row `regen_lens_dot_rs_retired`.
- **Director** — scope-change escalations (e.g., a bin-shim semantic that needs Director-approved continuing exception); §P1 arbitration.

## Dispatch preconditions (live-state checklist for PB Manager)

PB Manager dispatches the sub-gate 3 worker when:

1. R2 close — done via #1275 ✓.
2. R2-Evaluator stable.
3. Item 4 (PB-Runtime) landed (transitive emit-pattern prereq).
4. Substrate-owned `BinShim` carrier type live on main.
5. `std.process.ProcessExit` live.
6. BinShim emit pattern landed (BinShim brief's deliverable 2).
7. `data regen_lens_shim: BinShim` instance authored (BinShim brief's deliverable 1).

If any of (2)–(7) is unmet, brief stays in PROPOSAL state.

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4 (Item 5 emit pattern), §4.3 (dissolution path), §5.1 (sub-gate decomposition), §5.2 (SG-0 cascade), §6 (anti-bridge invariants), §7.2 (BinShim equivalence fixture).
- Parent manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md#owns-r3-continuation--director-cascade-item-4--item-8-ratified-2026-04-28) — R3 continuation table rows for T-LensProducer-Retirement and BinShim instances + emit pattern + retirement dispatch.
- BinShim retirement program (mechanism this sub-gate consumes): [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) — sub-gate 3 is this brief's "First slice — `regen_lens.rs`."
- First-target retirement readiness checklist (planning only): [`docs/briefs/r3-pb-regen-lens-first-binshim-target-retirement-readiness.md`](r3-pb-regen-lens-first-binshim-target-retirement-readiness.md).
- T-LensProducer-Retirement parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure".
- Sibling sub-gate skeletons: [`r3-pb-t-lensproducer-sub1-lens-apply-retirement.md`](r3-pb-t-lensproducer-sub1-lens-apply-retirement.md), [`r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md`](r3-pb-t-lensproducer-sub2-lens-testgen-retirement.md).
- Sibling PB R3 brief: [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md).
- SG-0 census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs` (`EXPECTED_HAND_AUTHORED_NON_TEST`); `src/v3/compiler/build.rs` `REGEN_OUTPUTS`.
- File targeted for retirement (do not edit until dispatch): `src/v3/compiler/src/bin/regen_lens.rs`.
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
