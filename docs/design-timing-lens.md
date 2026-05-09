# Design — Timing lens and Shared External Attachment (T-Workflow-As-Data Slice 2)

**Status:** substrate carriers landed in `src/v3/std/timing_lens.dag` (gunbc#1955).  
**Gates:** §1.8 `#54` `timing_lens_carrier_landed`, `#55` `shared_external_attachment_pattern_documented`.

## 1. Timing lens (`Lens<TimingMeasurement>`)

The structural lens carrier remains `Lens<C>` in `src/v3/std/lens.dag`. Slice 2 introduces `TimingMeasurement` as the timing dimension’s carrier type; composed analyses therefore use `Lens<TimingMeasurement>` as the parameterized lens surface (parallel to `Lens<SymbolicCost>` cost work).

`TimingMeasurement` is a closed sum: `Observed { nanoseconds } | Unobserved | Ambiguous | Stale`. The carrier uses `Unobserved` (not the bare label `Missing`) so witness expressions like `Inhabits(Unobserved)` resolve unambiguously in the `.dag` compiler; it denotes the same external “missing timing evidence” state described in program gates. It is intentionally **not** a bare scalar: absent or ambiguous runner evidence stays explicit in the type (fail-closed discipline; no silent defaults).

Supporting carriers:

- `TimingObservationSet` — batched `(subject_stable_id, measurement)` entries for correlating external timing rows with workflow subjects.
- `TimingBudget` — `max_nanoseconds` budget fact paired with enforcement / demo lanes (`LensEnforcement`-shaped consumption is out of scope for this substrate-only slice).

Scaffold functions in `timing_lens.dag` (`timing_sequential_op`, `timing_branch_op`, `timing_measurement_iterate`, `timing_lens_read`, `timing_lens_validate`) are the hooks where a future `data … : Lens<TimingMeasurement>` instance will attach; **E6 / bootstrap policy** still forbids registering concrete `Lens<…>` data rows until the function-valued structural data surface closes.

## 2. Shared External Attachment — six invariants

`WorkflowObservationAnchor` factors reusable **external** workflow facts (timing today; logs, coverage, artifacts, failures in follow-on work) away from pure program text. The Substrate Mgr stance (gunbc#1130) requires these six invariants:

1. **Stable subject identity** — attachment is keyed by a stable subject identifier (`subject_stable_id`), not by a source span or other editor-volatile coordinate.
2. **Observed-artifact identity** — the external payload is bound via an explicit digest (`artifact_digest`) so “what was read” is objective, not inferred.
3. **Producer / observer / prover separation** — three roles (`producer_id`, `observer_id`, `prover_id`) are recorded explicitly so provenance does not collapse into a single anonymous string.
4. **Attachment time and run context** — `attached_at_epoch_ns` plus `workflow_run_id` tie the fact to a concrete run, not an abstract “latest” pointer.
5. **Report states, not silent scalars** — timing outcomes use `TimingMeasurement`’s `Observed | Unobserved | Ambiguous | Stale` sum; consumers must branch on the state instead of coercing absent data to zero.
6. **Fail-closed on non-observed / non-valid states** — enforcement and lens application surfaces treat `Unobserved`, `Ambiguous`, and `Stale` as non-evidence unless a named consumer explicitly documents widening; the substrate vocabulary does not fabricate `Observed` from thin air.

Promotion to a generic `ExternalDataAnchor<Subject, Source>` waits on a second concrete consumer (Substrate Mgr disposition).

## 3. References

- `docs/r3-structure.md` — T-Workflow-As-Data row, gate bullets for `#54` / `#55`.
- `docs/briefs/r3-substrate-t-workflow-as-data-slice-1-worker.md` — Slice 2 scope list.
- `src/v3/std/lens.dag` — `Lens<C>` six-field contract.
- `src/v3/std/lens_application.dag` — `LensEnforcement` / enforceable lens application (budget pairing).
