# Design — Timing lens and Shared External Attachment (T-Workflow-As-Data Slice 2)

**Status:** substrate carriers landed in `src/v3/std/timing_lens.dag` (gunbc#1955).  
**Gates:** §1.8 `#54` `timing_lens_carrier_landed`, `#55` `shared_external_attachment_pattern_documented`.

## 1. Timing lens (`Lens<TimingMeasurement>`)

The structural lens carrier remains `Lens<C>` in `src/v3/std/lens.dag`. Slice 2 introduces `TimingMeasurement` as the timing dimension’s carrier type; composed analyses therefore use `Lens<TimingMeasurement>` as the parameterized lens surface (parallel to `Lens<SymbolicCost>` cost work).

### 1.1 `Nanoseconds` nominal coordinate

Wall-clock durations and POSIX-style epoch coordinates are carried by a dedicated record `Nanoseconds { count: Nat }` rather than encoding “this `Int` is nanoseconds” only in field names. **`Nat`** is the same non-negative counting authority used for nanosecond magnitudes on `PerfBaselineMeasurement` in `src/v3/std/substrate.dag` (`median_ns`, `p99_delta_ns`); negative magnitudes are therefore uninhabitable in the carrier shape (P2 / M9 alignment for the magnitude coordinate). **🟡 SCAFFOLD** remains appropriate for **range / scale / signed-epoch** questions: upper bounds, `Measure<Time, S>` grounding, and signed pre-epoch instants stay **deferred** per Q-Unit-4 option (c), matching `dsl/std/measure.dag` until Grounding consumers attach checked bounds.

### 1.2 `TimingMeasurement` report coproduct (Practice 4 dissolution)

**PRACTICE 4 CHECKPOINT: 🟡 SCAFFOLD** — declared inline on `TimingMeasurement` in `src/v3/std/timing_lens.dag` (emoji classification + four-pattern table per `docs/modeling-discipline.md#4-coproduct-dissolution`).

`TimingMeasurement` is a closed sum encoding **report state folded with the timing fact** (`Observed { duration: Nanoseconds } | Unobserved | Ambiguous | Stale`). Practice 4 four-pattern dissolution:

1. **Fact placement** — PASS: each variant names one external observation outcome; no duplicate authorities.
2. **Variant-is-data** — PASS: `Observed` carries structured duration; other arms are unit observation facts.
3. **Algebraic form** — PASS: explicit sum type (no stringly encoding of outcomes).
4. **Dimensional** — N/A at this scaffold layer; dimensional semantics for time are carried by the `Nanoseconds` nominal (non-negative magnitude via `Nat`; further refinement deferred per §1.1).

The carrier uses `Unobserved` (not the bare label `Missing`) so witness sites avoid ambiguous resolution against other `Missing`-shaped substrate names; it denotes the same external “missing timing evidence” state described in program gates.

Supporting carriers:

- `TimingObservationSet` — batched `(subject_stable_id, measurement)` entries for correlating external timing rows with workflow subjects.
- `TimingBudget` — `max: Nanoseconds` budget fact paired with enforcement / demo lanes (`LensEnforcement`-shaped consumption is out of scope for this substrate-only slice).

Scaffold functions in `timing_lens.dag` (`timing_sequential_op`, `timing_branch_op`, `timing_measurement_iterate`, `timing_lens_read`, `timing_lens_validate`) are the hooks where a future `data … : Lens<TimingMeasurement>` instance will attach; **E6 / bootstrap policy** still forbids registering concrete `Lens<…>` data rows until the function-valued structural data surface closes. `timing_lens_read` uses `Inhabits(timing_measurement_unobserved())` so the witness payload is constructed through a `TimingMeasurement` arrow (mirrors the `Inhabits(_seed_list_provenance_empty())` pattern in `emission_provenance.dag`).

## 2. Shared External Attachment — six invariants

`WorkflowObservationAnchor` factors reusable **external** workflow facts (timing today; logs, coverage, artifacts, failures in follow-on work) away from pure program text. The Substrate Mgr stance (gunbc#1130) requires these six invariants:

1. **Stable subject identity** — attachment is keyed by a stable subject identifier (`subject_stable_id`), not by a source span or other editor-volatile coordinate.
2. **Observed-artifact identity** — the external payload is bound via an explicit digest (`artifact_digest`) so “what was read” is objective, not inferred.
3. **Producer / observer / prover separation** — three roles (`producer_id`, `observer_id`, `prover_id`) are recorded explicitly so provenance does not collapse into a single anonymous string.
4. **Attachment time and run context** — `attached_at_ns: Nanoseconds` plus `workflow_run_id` tie the fact to a concrete run; `Nanoseconds` is the same nominal coordinate used for durations (SI-nanosecond magnitude as `Nat`; signed-epoch / range refinements deferred per §1.1).
5. **Report states, not silent scalars** — timing outcomes use `TimingMeasurement`’s `Observed | Unobserved | Ambiguous | Stale` sum; consumers must branch on the state instead of coercing absent data to zero.
6. **Fail-closed on non-observed / non-valid states** — enforcement and lens application surfaces treat `Unobserved`, `Ambiguous`, and `Stale` as non-evidence unless a named consumer explicitly documents widening; the substrate vocabulary does not fabricate `Observed` from thin air.

Promotion to a generic `ExternalDataAnchor<Subject, Source>` waits on a second concrete consumer (Substrate Mgr disposition).

## 3. References

- `docs/briefs/r3-substrate-t-wad-slice-2-timing-lens-canvas.md` — Director / Mgr shape questions (ratification pending).
- `docs/r3-structure.md` — T-Workflow-As-Data row, gate bullets for `#54` / `#55`.
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — Slice 2 scope list.
- `dsl/std/measure.dag` — Q-Unit time / scale authority (phantom `Measure<Time, S>`; refinement deferral precedent).
- `src/v3/std/lens.dag` — `Lens<C>` six-field contract.
- `src/v3/std/lens_application.dag` — `LensEnforcement` / enforceable lens application (budget pairing).
