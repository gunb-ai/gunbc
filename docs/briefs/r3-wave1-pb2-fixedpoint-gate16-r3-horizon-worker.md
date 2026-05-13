# R3 Wave-1 PB2 — FixedPoint gate #16 (`pb_self_compile_fixed_point` R3 horizon) worker brief

**Status:** DISPATCH-READY (pre-authored worker brief per `docs/r3-remaining-work-dependency-graph.md` §5 Wave-1).

**Owning manager:** R3 PB Manager (nimble-crab-786 lineage).

**Lane:** T-FixedPoint — R3 §1.8 gate **#16** — **R3 thesis facet 2** interpretation only (distinct from R1 horizon).

**Parent planning artifact (authority, not superseded):** [`r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md) — phases P0–P3, STOP conditions, Row-class A/B/C acceptance, DB-8 relationship.

**Cross-reads:** [`docs/r3-structure.md`](../r3-structure.md) §"T-FixedPoint" two-horizon clarification; [`docs/r2-structure.md`](../r2-structure.md) R1 closure criteria; [`INVARIANTS.md`](../../INVARIANTS.md) **P1**, **P2**, **P5**; [`MODELING.md`](../../MODELING.md) §M9 before any new verification predicate composition.

**Charter note:** Director msg_57631cde §1.9 (V1 sunset / PR #2748 aggregator) — **#16** R3 strengthening is named as a contributor; this worker does **not** toggle aggregator constituents alone; document any dependency edge discovered.

## Preconditions (dispatch gate — read `r3-pb-t-fixedpoint-worker.md` §"Dispatch preconditions")

Worker **must not** land `pb_self_compile_fixed_point_strong` (or rename / shadow the R1 `pb_self_compile_fixed_point` claim) until **joint** preconditions in the parent brief hold: R2-Evaluator landed; R2-Grounding-Rust+Python per joint rule; T-LensProducer-Retirement / SG-0 choreography per Director 2026-04-28 lock; Row-B materialization rule satisfied.

**Wave-1 allowed work (when any precondition is false — default):**

- **P0 pins only:** documentation + script/readiness alignment that **reduces** future P3 risk without changing R1 evaluation — e.g. `self_host_fixed_point.rs` / `db-8.md` cross-links, CI job comments, explicit "not yet dispatch-eligible" banners in `verification.dag` adjacent docs (not new `TestSuite` names).
- **Readiness audits:** confirm `FixedPointConverges` + `RatchetZero` substrate shapes still sufficient for future composition (report gaps to Substrate / Verification via PM — no variant edits from PB without substrate ack).

**Wave-1 allowed work (when PM + Director confirm joint preconditions met — rare):**

- Execute **P3** deliverables exactly as specified in `r3-pb-t-fixedpoint-worker.md` §"Acceptance gate" (Row A / B / C), preserving **two-horizon** discipline: R1 fixture and predicate name stay untouched; **add** strong suite as distinct claims.

## STOP — escalate

- Any urge to edit R1 `pb_self_compile_fixed_point` evaluation semantics to "include" the R3 bar — **forbidden**; see parent brief §"Worker discipline".
- Substrate insufficient for `FixedPointConverges` composition — signal Substrate Mgr via PM; do not extend `TestPredicate` variants from PB lane.
- SG-0 non-test > 0 blocks P3 per parent brief — do not partial-ship strong fixed-point.

## Deliverables

1. **Default path:** One PR labeled **P0/P1 readiness** with docs + optional thin Rust comments / `tracing` hooks that do **not** change Pass/Fail of R1 gates.
2. **Dispatch-eligible path:** One PR implementing parent brief **P3** + `r3-program-plan.md` §1.8 #16 status transition + ledger signal per parent brief §closure.
3. Tests: only those required by the chosen path; never remove or weaken `r1_release_acceptance_test` / `r1c_d` wiring for R1 horizon.

## Merge discipline (Mgr-self-authorized PB lane)

Same **(i)(ii)(iii)** audit-trail as PB1 brief: lane + gate, brief + authorities, SG-0 / receipt trail or explicit none.
