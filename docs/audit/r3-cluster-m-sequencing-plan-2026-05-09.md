# R3 Cluster M (Tests-As-Data-Completeness) Sequencing Plan — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier sequencing plan. **Director ratifies before dispatch** per operator directive 2026-05-09 ("course correct; existing plan stays canonical; staffing is not a concern; this is planning/correction") + Director ratification at gunbc#846 #issuecomment-4412008376.
**Parent docs**:
- [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) — load-bearing finding: Cluster M is critical-path for PB-0 closure
- [`docs/audit/r3-cluster-analysis-2026-05-09.md`](r3-cluster-analysis-2026-05-09.md) — original 15-cluster decomposition (this plan corrects Cluster M classification)
- [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 gates #84/#85/#86/#87
- [`THESIS.md`](../../THESIS.md) "Pure Bootstrap to Zero" framing ("0 hand-maintained") + [`ROADMAP.md`](../../ROADMAP.md) **T-PB-B** lane row (`pb_rust_tests_outside_residual_zero` predicate — "zero Rust-authored tests exist") (section anchors per `feedback_section_anchors_over_line_numbers` — line numbers drift)

---

## §0. Why this plan exists

PR #2358 audit established that **~80-90 of 101 SG-0 test entries dissolve via Cluster M bulk event**, and that Cluster M's gates (#84/#85/#86/#87) are all DECLARED with no active worker at HEAD. Director ratified Cluster M as critical-path-load-bearing for PB-0 closure (which is itself the load-bearing R3-close criterion).

This plan partitions Cluster M into a sequenced 3-phase dispatch program with lane-Mgr partition + brief-authoring scope.

## §1. Cluster M sub-gate analysis

### §1.1 The four gates

| # | Gate ID | Family | Owner Lane | Pass condition (per §1.8) |
|---|---|---|---|---|
| 84 | `every_rust_test_ports_to_dag_or_generated` | state-check | T-Tests-As-Data-Completeness | "thesis facet 3; every Rust test ports" — `EXPECTED_HAND_AUTHORED_TEST` count = 0 |
| 85 | `forall_exists_quantifier_substrate_landed` | substrate-shape | T-Tests-As-Data-Completeness | ForAll / Exists quantifier substrate landed in `dsl/std/` |
| 86 | `program_generator_carrier_landed` | substrate-shape | T-Tests-As-Data-Completeness | ProgramGenerator substrate carrier landed in `dsl/std/` |
| 87 | `lens_cementing_test_discipline_complete` | state-check | T-Tests-As-Data-Completeness | every `.dag` lens has cementing test against frozen v2-oracle |

### §1.2 Dependency structure

**Authority correction 2026-05-09 (codex BLOCKING #3 on PR #2362 sha `60279789`)**: prior framing of #87 as \"consumes #85/#86 carriers\" conflated cementing axis (LensRegistry projection ratchet) with property-based axis (program-family claims). Cementing uses existing 🟢 TERMINAL `DifferentialEquals`/`LensOutputEquals` predicate variants per locked design §C5; these are available on main today, no #85/#86 dependency at the predicate level. Coupling exists at the `SuiteClaim` wrapper level only (post-#85): existing `TestSuite.claims` migrate to wrap in `Enumerated(...)` per design §6 line 344 — backward-compatible.

```
Phase 1 (parallel substrate-shape per locked design §2.1+§2.2; 5 carriers):
  ┌─ #85 quantifier surface
  │   `Quantifier`, `QuantifiedTestClaim`, `SuiteClaim` in `src/v3/std/verification.dag`
  │   (per design §2.2)
  │
  └─ #86 generator surface
      `ProgramGenerator`, `ProgramShape` in `src/v3/std/verification.dag`
      (per design §2.1)

Phase 2 (cementing-test discipline; INDEPENDENT of #85/#86 at predicate level):
  #87 cementing-test discipline
  ↑ uses existing DB-15 TestClaim + DifferentialEquals/LensOutputEquals
    (🟢 TERMINAL predicate variants; available on main today per design §1)
  ↑ couples to Phase 1 only at SuiteClaim wrapper level (Enumerated(...) wrap;
    mechanical post-#85, no predicate-level coupling)
  Property-based axis (#85/#86 ProgramGenerator/Quantifier) is for "every
    program in family X satisfies P" claims — orthogonal to cementing's
    per-LensRegistry-row v2-vs-v3 same-source comparison.

Phase 3 (bulk port):
  #84 every Rust test ports
  ↑ consumes #87 cementing-test discipline as the migration pattern for
    cementing-test family (~20-25 entries — largest class)
  ↑ consumes #85/#86 carriers for property-based bulk migrations (where
    program-family claims supersede single-source TestClaims)
  ↑ consumes existing DB-15 TestClaim infrastructure for non-cementing,
    non-property-based bulk migrations
  → bulk-port mechanism collapses ~80-90 hand-Rust test entries
  → SG-0 EXPECTED_HAND_AUTHORED_TEST count drops to 0
```

### §1.3 Why this sequencing

- **#85 + #86 land for property-based axis** (program-family claims). Per locked design §6 line 344: 5 carriers (`ProgramGenerator`, `ProgramShape`, `Quantifier`, `QuantifiedTestClaim`, `SuiteClaim`) extend `src/v3/std/verification.dag`; existing `TestSuite.claims` migrate to wrap in `Enumerated(...)`.
- **#87 cementing discipline can dispatch independently** of #85/#86 — cementing uses existing 🟢 TERMINAL `DifferentialEquals`/`LensOutputEquals` predicates per design §C5. The only Phase 1 coupling is the `SuiteClaim` wrapper migration (mechanical post-#85). Hand-Rust cementing tests (e.g., `complexity_lens_behavioral_completion.rs`, `cost_lens_behavioral_completion.rs`) are the largest single class in `EXPECTED_HAND_AUTHORED_TEST`; #87 dissolves them via discipline-pattern landing + bulk migration using existing predicate infrastructure.
- **#84 closes when bulk port runs to zero — strict**: state-check gate; not authored as a single PR but as the convergence of bulk-migration PRs. Closes when SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count = 0. **Per codex BLOCKING on PR #2361 sha `b925b174` (2026-05-09)**: temporary exception handling (e.g., Director Option 2 timed-carry tests like `cross_target_coverage_carrier_test.rs`) is **NOT folded into close condition** — exceptions remain blockers / non-close risk until SG-0 test partition is actually zero. R3-honest-close requires the count itself to reach zero, not "zero except exceptions." Director-allocated timed-carries must dissolve via testgen-coverage migration before #84 fires; otherwise #84 remains DECLARED.

## §2. Lane-Mgr partition

| Phase | Gate | Owner Mgr | Partner | Authoring scope |
|---|---|---|---|---|
| 1a | #85 `Quantifier` + `QuantifiedTestClaim` + `SuiteClaim` carriers | **Substrate Mgr** (warm-wolf-698 / #2068) | Verification (consumer wiring) | extend `src/v3/std/verification.dag` per locked design `docs/design-tests-as-data-completeness.md` §2.2; no canvas needed (design-doc resolves shape) |
| 1b | #86 `ProgramGenerator` + `ProgramShape` carriers | **Substrate Mgr** (warm-wolf-698 / #2068) | Verification (consumer wiring) | extend `src/v3/std/verification.dag` per locked design `docs/design-tests-as-data-completeness.md` §2.1; no canvas needed (design-doc resolves shape) |
| 2 | #87 cementing-test discipline | **Verification Mgr** (wise-bear-525 / #2075) | Substrate (SuiteClaim wrapper migration only post-#85) | discipline pattern + first cementing-test migration receipt; uses existing DB-15 TestClaim + DifferentialEquals/LensOutputEquals per design §C5 (no #85/#86 predicate-level dependency) |
| 3 | #84 every Rust test ports | **Verification Mgr** (wise-bear-525 / #2075) | Substrate (carrier consumer) + multi-Mgr (test ownership distributed) | bulk-port coordinator role; per-test-class migration brief queue |

**Per Director directive (operator: "staffing is not a concern")**: lane Mgrs dispatch as many parallel workers as needed. Substrate Mgr can land #85 + #86 carriers in parallel per locked design `docs/design-tests-as-data-completeness.md` §2.1 + §2.2 (different carrier shapes, no shared substrate dependency; no canvas-tier ratification — design-doc resolves shape per §1 Authority discipline). Verification Mgr partners with Substrate consumer-wiring work-side as #85/#86 carriers land.

## §3. Phase 1 substrate carrier landings (per locked design)

**Authority correction 2026-05-09 (codex BLOCKING on PR #2361 sha `c6c3fb96`)**: gates #85 + #86 substrate carriers are **already canonically defined** in [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §2.1 + §2.2. Per design-doc preamble: "All §8 design questions resolved in-doc per `feedback_design_before_implement` — **no Director ratification required before lane dispatch** (only standard cascade gates: R2-Evaluator landed; existing TestClaim infrastructure from DB-15 R2)." Single canonical authority per INVARIANTS P2.

Prior framing of §3.1 / §3.2 as "substrate canvas needed; Director ratification needed before brief authoring" was a duplicate-authority anti-pattern (codex BLOCKING'd correctly). Reframed below to cite locked design + scope to lane-dispatch worker briefs only.

### §3.1 #85 `Quantifier` + `QuantifiedTestClaim` carriers (per design §2.2)

**Locked carrier shape** (per `docs/design-tests-as-data-completeness.md` §2.2): `Quantifier` is a closed two-variant sum (`ForAll | Exists`) — exhausting structurally meaningful quantifications over a `ProgramGenerator`'s output. `QuantifiedTestClaim { generator: ProgramGenerator, quantifier: Quantifier, ... }` lives **alongside** `TestClaim` (not as replacement) — covers the property-based axis where the existing `TestClaim` covers single-source enumerated tests.

**Substrate landing** (Substrate Mgr authoring under standing authority): extend `src/v3/std/verification.dag` to add `Quantifier` + `QuantifiedTestClaim` per design §2.2 spec. **No Director ratification needed** (locked design); standard cascade gates only (R2-Evaluator landed; existing DB-15 TestClaim infrastructure).

### §3.2 #86 `ProgramGenerator` carrier (per design §2.1)

**Locked carrier shape** (per `docs/design-tests-as-data-completeness.md` §2.1): `ProgramGenerator` is a structural reference to a generator declaration — **not** a roster of "shape kinds" (which would replicate the closed-roster failure flagged by `lens-library-design.md` §1.5). The generator body is itself a `.dag` declaration producing program shapes; `ProgramGenerator` references it structurally per design §2.1 (Rust signature in design doc).

**Substrate landing** (Substrate Mgr authoring under standing authority): extend `src/v3/std/verification.dag` to add `ProgramGenerator` carrier per design §2.1 spec. Composition with §3.1 #85: `QuantifiedTestClaim.generator` field references `ProgramGenerator`. **No Director ratification needed** (locked design); standard cascade gates only.

## §4. Phase 2 brief authoring scope (#87)

**Verification Mgr authors worker brief** (PM-authorable now; #85/#86 carrier landings per locked design not blocking):

- Scope: cementing-test discipline pattern application to existing hand-Rust cementing tests
- First migration target: smallest hand-Rust cementing test (e.g., `cementing/cementing_lens_registry_dispatch_test.rs`) — proof-of-concept migration
- Pattern: hand-Rust `#[test] fn test_X` with v2-oracle assert → `.dag` `TestClaim` with frozen-snapshot `BinaryDimensionReportEquals` against captured baseline
- Discipline: every `.dag` lens has at least one cementing test in `.dag` form (#87 Pass condition)
- Receipt: state-check gate fires when audit confirms 100% lens coverage in cementing-test form

PM authors the brief draft; Verification Mgr (wise-bear-525) consumes as Mgr-tier dispatch when Phase 1 lands.

## §5. Phase 3 brief authoring scope (#84)

**Verification Mgr authors bulk-port coordinator brief + per-class worker briefs** (PM authors coordinator brief; per-class briefs follow once Phase 2 pattern proves):

### §5.1 Coordinator brief
- Scope: track 80-90 hand-Rust test entries in `EXPECTED_HAND_AUTHORED_TEST` partitioned by dissolution-trigger class
- Per-class brief authoring queue: each "Dissolves when..." comment in `sg0_census_test.rs` becomes a brief for that class
- Bulk-port dispatch: parallel worker dispatch per class (operator directive: staffing not a concern)
- Receipt: SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count drops to 0 — strict (per codex BLOCKING 2026-05-09; no Director-allocated-exception fold-into-close)

### §5.2 Per-class brief queue (estimated from velocity-walk §2.1)

| Class | Approx count | Worker brief scope |
|---|---|---|
| Cementing-test family (post-#87 pattern) | ~20-25 | Apply #87 pattern to remaining lens cementing tests |
| Reflected-Dag structural assertion family | ~25-30 | Migrate to ProgramGenerator-driven TestClaims (#86 consumer) |
| Generic DimensionReport / runner-discipline family | ~20-25 | Migrate to runner-side TestClaim (post-#87) |
| Boundary tests (m1_*, m2_*, etc.) | ~10 | Each migrates per testgen-coverage discipline |
| R1C-D / R1C-E wrappers | ~3 | R1-close residuals; may be eligible NOW |
| L4/L7/L5 skeleton | ~5 | Verification Cluster G overlap; may bundle |

Each class is a parallel-dispatchable worker batch.

## §6. Velocity projection

**Phase 1 (parallel)**: 1-2 weeks for #85/#86 carrier landings per locked design `docs/design-tests-as-data-completeness.md` §2.1 + §2.2 (Substrate Mgr can run #85/#86 parallel with worker pairs; no canvas-tier ratification — design-doc resolves shape).
**Phase 2**: 1-2 weeks for #87 discipline pattern + first migration receipt
**Phase 3**: 2-4 weeks for bulk-port (parallel per-class workers; staffing-not-a-concern)

**Total Cluster M close**: **4-8 weeks** from Phase 1 dispatch start. Fits in 8-12 week R3 window if dispatch starts immediately.

**Risk**: locked design `docs/design-tests-as-data-completeness.md` §2.1 + §2.2 already resolves carrier shape; risk of unexpected design questions surfacing during landing is bounded. STOP-and-PING via Substrate Mgr inbox if migration shape surprises arise (per `feedback_construction_over_ratchets`).

## §7. Cross-Mgr coordination requirements

- **Substrate Mgr ↔ Verification Mgr**: Phase 1 carriers (#85/#86) consumed by Phase 2 discipline (#87); coordinate consumer-readiness signals.
- **Verification Mgr ↔ all lane Mgrs**: Phase 3 bulk-port touches tests across all lanes (m1/m2 boundary, T-Free-Consequences, T-Lens-*, etc.). Per-class briefs may need lane-Mgr partner sign-off on test-ownership.
- **PB Mgr cross-reference**: some Phase 3 classes ("R3 PB Row-4 corpus seed", "Differential equality PB-Runtime tests") consume PB Item 4 disposition — coordinate with PB Mgr lane-state.

## §8. Dispatch readiness checklist (for Director ratification)

### §8.1 Existing brief inventory (grep-verified at HEAD)

[`docs/briefs/r3-v-tests-as-data-v1-worker.md`](../briefs/r3-v-tests-as-data-v1-worker.md) is **PRE-AUTH DISPATCH-READY** (tier-1 queue #1859) and covers all four gates (#84/#85/#86/#87). Brief shape: "single worker coordinates lane closure."

**Director ratification (Director answered Ask 1 with (γ) at gunbc#846 #issuecomment-4412309986)**: dispatch shape ratified.
- **(α)** Existing single-coordinator brief — one worker authors all four gates serialized within lane
- **(β)** Re-shape to 4 parallel-worker briefs — Substrate Mgr partner on #85 + #86, Verification Mgr partner on #87 + #84
- **(γ) RATIFIED**: Hybrid — single coordinator authors #87/#84 (Verification scope) + Substrate Mgr lands #85/#86 carriers per locked design `docs/design-tests-as-data-completeness.md` §2.1+§2.2 in parallel.

### §8.2 Dispatch sequence

**Authority correction 2026-05-09 (codex BLOCKING #4 on PR #2362 sha `60279789`)**: prior framing said \"Substrate Mgr authors #85 substrate canvas → Director ratifies\" — duplicate-authority anti-pattern (locked design §1: \"no Director ratification required before lane dispatch\").

- [ ] Substrate Mgr lands #85 carriers (`Quantifier`, `QuantifiedTestClaim`, `SuiteClaim`) per locked design §2.2 → worker dispatch under standing authority
- [ ] Substrate Mgr lands #86 carriers (`ProgramGenerator`, `ProgramShape`) per locked design §2.1 → worker dispatch under standing authority (parallel with #85)
- [ ] Verification Mgr dispatches #87 cementing-test discipline pattern using existing DB-15 TestClaim + DifferentialEquals/LensOutputEquals predicates per design §C5 — **independent of #85/#86 dispatch** (cementing axis is orthogonal to property-based axis); existing `r3-v-tests-as-data-v1-worker.md` cited as Verification-side coordinator brief
- [ ] Post-#85 SuiteClaim landing: existing `TestSuite.claims: List<TestClaim>` sites mechanically wrap in `Enumerated(...)` per design §6 line 344 (backward-compatible)
- [ ] §1.8 ledger Status updated as each gate transitions DECLARED → CONSUMER_LANDED → PASSING

## §9. Open questions

All prior questions RESOLVED:

1. ~~**Substrate canvas authoring authority**~~: **RESOLVED 2026-05-09** per codex BLOCKING #1 on PR #2361 sha `c6c3fb96`. Locked design §1 Authority discipline: \"no Director ratification required before lane dispatch.\" #85/#86 are carrier landings per design §2.1/§2.2, not canvas-tier substrate-shape introductions. Substrate Mgr standing authority dispatches under existing pre-auth.

2. ~~**Phase 3 bulk-port discipline**~~: **RESOLVED 2026-05-09** per Director answer at gunbc#846 #issuecomment-4412309986 (Ask 3): \"Verification Mgr coordinator\". Single coordinator role; lane Mgr signoff workflow on per-class migrations.

3. ~~**#84 closure criterion under Director-allocated exceptions**~~: **RESOLVED 2026-05-09** per codex BLOCKING on PR #2361 sha `b925b174` (addressed at sha `5631cac33`). Strict-zero close-condition adopted: Director Option 2 timed-carry tests (`cross_target_coverage_carrier_test.rs`, `method_template_contract_test.rs`, etc.) are **NOT** folded into close-condition; they remain blockers / non-close-risk until they migrate to testgen-coverage. R3-honest-close requires actual zero, not "zero except exceptions." See §1.3 + §5.1 for canonical close-condition language.

---

**End of plan.**
