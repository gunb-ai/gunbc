# R3 T-FixedPoint Worker Brief (PB)

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-04-29 by PB Manager continuation per the Pending entry in [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Sub-briefs (authored / pending)" and the lane row in [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure".

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per `r3-structure.md` §"Manager structure" Item 1).

**Lane size:** M (per `r3-structure.md` lane table).

**This is a planning artifact — not a dispatch order.** Worker dispatch is gated; see §"Dispatch preconditions" + §"STOP conditions". PB Manager re-reads this brief at gate-clear to issue worker dispatch.

## Scope

T-FixedPoint closes the **R3 thesis facet 2 horizon** of the `pb_self_compile_fixed_point` predicate: `compiler.dag` compiled by the v3 binary produces **bit-identical stage0 Rust + bit-identical emitted artifacts** under fixed-point semantics, with the in-tree hand-Rust floor at zero (per Director-locked decision 2026-04-28 in `r3-structure.md` §"Design challenge 4").

The lane delivers:
1. The strong-interpretation `.dag` `TestClaim` `pb_self_compile_fixed_point_strong` (per `r2-pure-bootstrap-manager.md` §"Acceptance" line 101) authored against the existing `FixedPointConverges` substrate variant at `src/v3/std/verification.dag:206` — same predicate name, stronger acceptance.
2. Verification that running the cycle a second time on the v3-emitted Rust produces byte-identical output (true fixed point, not just "compiles itself once").
3. Closure-ledger signal that R3 thesis facet 2 has landed.

## Two-horizon framing (load-bearing — do not collapse)

Per `r3-structure.md:59` and `r2-structure.md:296`, the predicate name `pb_self_compile_fixed_point` carries **two horizons**:

| Horizon | Acceptance | Where |
|---|---|---|
| **R1 lane gate** | Pass = current `verification.dag` + `test_runner` evaluation under R1 acceptance discipline. Made green at landing by #1050 + #1074. | `src/v3/compiler/tests/integration/r1_release_acceptance_test.rs:18` + `r1c_d_pb_census_gates_test.rs:43` |
| **R3 thesis facet 2** (this lane) | Closes under bit-identical fixed-point + SG-0 choreography per Director cascade 2026-04-28. Strong interpretation. | `pb_self_compile_fixed_point_strong` `.dag` claim authored under this lane |

**R1 close does not wait on R3.** R3's elevated bar is a separate release/thesis acceptance — **not a silent rename of the R1 predicate**. The R1 fixture remains green at its R1 acceptance; this lane authors the strong claim alongside.

**Worker discipline:** never edit the R1 fixture's predicate evaluation to incorporate the strong bar. Add the strong claim as a distinct `TestClaim` (see §"Acceptance gate"), so the R1 horizon stays untouched.

## Dependencies

Per `r3-structure.md` §"Lane structure" + §"Dependency DAG":

1. **R2-Evaluator landed** — runtime executes `compiler.dag`; without the Evaluator the fixed-point cycle has nothing to run. This is the dominant gate (7 of 10 R3 lanes share it).
2. **SG-0 non-test = 0 from T-LensProducer-Retirement** — per `r3-structure.md` §"Design challenge 4" Director-locked decision: T-FixedPoint closes under "SG-0 non-test = 0 + ≤1 first-time-bootstrap trampoline allowed per [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §`First-time bootstrap`." The trampoline lives **outside** `src/v3/`; the in-tree floor stays 0. The three lens-producer files (`lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs`) must be retired before this lane closes — they are the load-bearing residual on the SG-0 non-test census.
3. **PB-1 generated bin-shim pattern** (sub-dependency of T-LensProducer-Retirement, transitively) — needed for `regen_lens.rs` retirement to land cleanly so SG-0 reaches zero.

These dependencies are cumulative: R2-Evaluator → T-LensProducer-Retirement (XL, 3 sub-gates) → T-FixedPoint (M).

## Acceptance gate (`.dag`)

Per `r2-pure-bootstrap-manager.md` §"Acceptance" line 101 + `r3-structure.md:60`:

**`pb_self_compile_fixed_point_strong`** — authored as a `.dag` `TestClaim` consuming the existing `FixedPointConverges { compile_target, expected }` variant at `src/v3/std/verification.dag:206-209`. Acceptance:

- v3 binary compiles `compiler.dag` → emitted Rust output X.
- Recompiling `compiler.dag` from X (or running the cycle again) → output X′ where `X == X′` byte-identically.
- All Shape-A emitted artifacts (Rust today; Python/Go once R2-Grounding-{Python,Go} land — see §"Cross-lane sequencing") byte-identical between cycle N and cycle N+1.
- SG-0 `EXPECTED_HAND_AUTHORED_NON_TEST` = 0 at the time of evaluation (gate cross-reads the SG-0 census authority — does not duplicate it).

**Substrate is already in place** (`FixedPointConverges` variant exists). No substrate-introduction expected; if the strong-interpretation acceptance surfaces a substrate gap, follow `INVARIANTS.md` §P1 substrate-fact-introduction procedure (signal Substrate Manager; do not introduce ad-hoc).

## Cross-lane sequencing (Shape-A target coverage)

`r3-structure.md` lane table specifies "bit-identical stage0 Rust + bit-identical emitted artifacts" — Rust is the load-bearing target (stage0 is Rust). Python + Go bit-identical artifact gates are scoped to whichever Shape-A targets have grounded by the time this lane dispatches:

- If R2-Grounding-Python + R2-Grounding-Go are already landed at dispatch time, the `pb_self_compile_fixed_point_strong` claim covers Rust + Python + Go.
- If only Rust is grounded, the lane closes on Rust-only fixed-point; the Python/Go extension follows once those targets land (no separate lane — extends this gate's claim list).

This avoids artificially gating T-FixedPoint on T-Verification-L5-Corpus (cross-target equivalence is L5's lane; T-FixedPoint is single-target byte-identity per cycle).

## Non-goals

T-FixedPoint **does not** own:

1. **The R1 horizon of `pb_self_compile_fixed_point`** — that's R1's; closed at R1 close. Worker must not modify the R1 fixture's predicate evaluation.
2. **Lens-producer retirement work** — that's T-LensProducer-Retirement (PB Manager R3 lane, separate brief). T-FixedPoint **consumes** the SG-0=0 signal; it does not produce it.
3. **Cross-target algebraic equivalence (L5)** — that's T-Verification-L5-Corpus (Verification Manager R3 lane). Different acceptance: byte-identity (this lane) vs algebraic-equivalence over a corpus (L5).
4. **Tier 3 mirror dissolution** — that's T-Tier3-Dissolution (PB Manager R3 lane).
5. **Bridge retirement** — that's T-Bridge-Retirement distribution (PB owns 3 of 5; tracked separately under `bridge_retirement_ledger_zero`).
6. **Performance budgets** — `r3-structure.md` §"Design challenge 7" Director decision: perf is post-R3 unless someone authors a budget claim with concrete numbers. T-FixedPoint deliverable is structural close; do not author perf gates here.

## Dispatch preconditions

Per `r3-structure.md` §"R3 worker dispatch precondition": the **joint precondition** is "R2-Evaluator landed AND R2-Grounding-Rust+Python landed" — Director-discretionary brief authoring may begin during R2 final week (this is what authorizes this planning artifact today), but **worker dispatch waits**. For T-FixedPoint specifically, dispatch additionally requires:

1. R2 close signal (R2 Release Manager closure ledger).
2. R2-Evaluator landed and stable (R2 lane closed).
3. T-LensProducer-Retirement (XL) closed — all three sub-gates (`lens_apply_dot_rs_retired`, `lens_testgen_dot_rs_retired`, `regen_lens_dot_rs_retired`) green; SG-0 non-test census = 0.
4. PB Manager (continuation) confirms SG-0=0 reading on a clean main before issuing dispatch.

If any of (1)-(4) is not met, this brief stays in PROPOSAL state; PB Manager does not dispatch.

## STOP conditions

Worker MUST STOP and escalate to PB Manager (which escalates to Director if cross-program) when:

- **R1 fixture pressure:** any change required to `r1_release_acceptance_test.rs` predicate evaluation or `verification.dag` `FixedPointConverges` variant shape — that's a substrate or R1-horizon edit; not in this lane's scope. Surface as a substrate gap.
- **SG-0 census drift:** the SG-0 `EXPECTED_HAND_AUTHORED_NON_TEST` count is non-zero at evaluation time — T-LensProducer-Retirement is incomplete; this lane is not yet dispatchable.
- **Bit-identity fails for a structural reason** (e.g., emitter non-determinism: HashMap iteration order, timestamps, absolute paths in emitted output) — that's an emitter dissolution, not a fixed-point-acceptance edit. Surface to PB Manager; do not paper over with normalization in the gate.
- **Trampoline expansion:** the "≤1 first-time-bootstrap trampoline" boundary tightens or expands — that's a Director-level cascade-decision change, not a worker call.
- **Substrate gap:** any need to introduce a new `TestPredicate` variant or extend `FixedPointConverges` — follow `INVARIANTS.md` §P1; do not author the variant in this lane.

## Cross-program signals

- **Lane open:** PB Manager → R2 Release Manager closure ledger (R3 continuation readiness signal).
- **Lane close:** PB Manager → R3 Release Manager (when authored) for R3 closure ledger; → Director for R3 thesis facet 2 closure announcement; updates `docs/thesis/r2-r3-thesis-mapping.md` row 136 status from ⏳ to ✅.
- **No upstream production:** T-FixedPoint consumes; it does not produce carriers other managers consume.

## Cross-refs

- Parent manager: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owns (R3 continuation)" + §"Acceptance" `pb_self_compile_fixed_point_strong`
- Lane authority: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-FixedPoint row + §"Design challenge 4" Director-locked SG-0 decision
- Two-horizon authority: [`docs/r2-structure.md`](../r2-structure.md) §"R1 closure criteria" + `r3-structure.md:60` two-horizon clarification
- Thesis-facet mapping: [`docs/thesis/r2-r3-thesis-mapping.md`](../thesis/r2-r3-thesis-mapping.md) row 136 (Facet 2)
- SG-0 floor authority: [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §`First-time bootstrap` (≤1 trampoline rule)
- Substrate variant: `src/v3/std/verification.dag:206-209` (`FixedPointConverges`)
- R1 fixture (do not edit): `src/v3/compiler/tests/integration/r1_release_acceptance_test.rs:18`
- Existing test scaffolding (reference, not the strong gate): `src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs`, `src/v3/compiler/tests/integration/r1c_d_pb_census_gates_test.rs`
- Sibling R3-PB lane brief (gating dependency): T-LensProducer-Retirement worker briefs (pending — see `r2-pure-bootstrap-manager.md` §"Sub-briefs … Pending")
