# R3 Cluster M (Tests-As-Data-Completeness) Sequencing Plan — 2026-05-09

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier sequencing plan. **Director ratifies before dispatch** per operator directive 2026-05-09 ("course correct; existing plan stays canonical; staffing is not a concern; this is planning/correction") + Director ratification at gunbc#846 #issuecomment-4412008376.
**Parent docs**:
- [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) — load-bearing finding: Cluster M is critical-path for PB-0 closure
- [`docs/audit/r3-cluster-analysis-2026-05-09.md`](r3-cluster-analysis-2026-05-09.md) — original 15-cluster decomposition (this plan corrects Cluster M classification)
- [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 gates #84/#85/#86/#87
- [`THESIS.md`](../../THESIS.md):298 + [`ROADMAP.md`](../../ROADMAP.md):88 (T-PB-B: zero Rust-authored tests)

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

```
Phase 1 (parallel substrate-shape):
  ┌─ #85 ForAll/Exists quantifier substrate
  │   (carrier in dsl/std/quantifier.dag or similar)
  │
  └─ #86 ProgramGenerator carrier
      (carrier in dsl/std/test_generation.dag or similar)

Phase 2 (consumer-discipline):
  #87 cementing-test discipline
  ↑ consumes #85 (ForAll quantifier for "for-all-programs" claims)
  ↑ consumes #86 (ProgramGenerator for representative-input claims)
  ↑ consumes T-Tests-As-Data runner infrastructure (TestClaim execution path)

Phase 3 (bulk port):
  #84 every Rust test ports
  ↑ consumes #87 cementing-test discipline as the migration pattern
  ↑ consumes #85/#86 carriers for new TestClaim authoring
  → bulk-port mechanism collapses ~80-90 hand-Rust test entries
  → SG-0 EXPECTED_HAND_AUTHORED_TEST count drops to 0
```

### §1.3 Why this sequencing

- **#85 + #86 must land first** (substrate carriers): without them, #87 cementing-test discipline cannot express the per-lens claims that current hand-Rust cementing tests assert. Phase 1 is the substrate-introduction.
- **#87 must land before bulk port** (#84): the cementing-test discipline IS the migration pattern. Hand-Rust cementing tests (e.g., `complexity_lens_behavioral_completion.rs`, `cost_lens_behavioral_completion.rs`) are the largest single class in `EXPECTED_HAND_AUTHORED_TEST`; #87 dissolves them via a single discipline-pattern landing + bulk migration.
- **#84 closes when bulk port runs to zero**: state-check gate; not authored as a single PR but as the convergence of bulk-migration PRs. Closes when SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count = 0 (or carries only Director-allocated exceptions).

## §2. Lane-Mgr partition

| Phase | Gate | Owner Mgr | Partner | Authoring scope |
|---|---|---|---|---|
| 1a | #85 ForAll/Exists quantifier substrate | **Substrate Mgr** (warm-wolf-698 / #2068) | Verification (consumer wiring) | substrate carrier in `dsl/std/`; need substrate canvas first |
| 1b | #86 ProgramGenerator carrier | **Substrate Mgr** (warm-wolf-698 / #2068) | Verification (consumer wiring) | substrate carrier in `dsl/std/`; need substrate canvas first |
| 2 | #87 cementing-test discipline | **Verification Mgr** (wise-bear-525 / #2075) | Substrate (#85/#86 consumer) | discipline pattern + first cementing-test migration receipt |
| 3 | #84 every Rust test ports | **Verification Mgr** (wise-bear-525 / #2075) | Substrate (carrier consumer) + multi-Mgr (test ownership distributed) | bulk-port coordinator role; per-test-class migration brief queue |

**Per Director directive (operator: "staffing is not a concern")**: lane Mgrs dispatch as many parallel workers as needed. Substrate Mgr can author #85 + #86 substrate canvases in parallel (different carrier shapes, no shared substrate dependency). Verification Mgr partners with Substrate consumer-wiring work-side as #85/#86 carriers land.

## §3. Phase 1 substrate canvases (substrate-canvas-author asks)

**Critical**: gates #85 + #86 are substrate-shape introductions — they introduce new carriers in `dsl/std/`. Per `feedback_audit_adjacent_authority_first` + `feedback_grep_substrate_before_naming_ratification`, substrate-shape introductions need **substrate canvases authored before worker briefs** so Director can ratify carrier shape + naming.

### §3.1 #85 ForAll/Exists quantifier substrate canvas

**Substrate canvas needed** (Substrate Mgr authors):
- Carrier shape: how does ForAll<P, T> / Exists<P, T> embed into Node/Conj/Disj/Cardinality/Bit per `feedback_compiler_is_dag_processor`?
- Predicate body type: closed-DSL term (decidable per `feedback_decidability_invariant`) or runtime evaluation against ProgramGenerator?
- Naming: `ForAllPrograms<C>`, `ForAllInhabitants<T>`, `Exists<P>` — grep `dsl/std/` first
- Adjacency: existing `BinaryDimensionReportEquals` (Pattern-A) family is the precedent; quantifier substrate generalizes per-(algebra, inhabitant) iteration to "for all P matching predicate"
- Pass-condition wiring: how does `every_rust_test_ports_to_dag_or_generated` count quantifier-driven tests?

**Director ratification needed before brief authoring** (substrate-canvas-tier).

### §3.2 #86 ProgramGenerator carrier canvas

**Substrate canvas needed** (Substrate Mgr authors):
- Carrier shape: ProgramGenerator<C> as `.dag` data — what does it produce? Concrete `Dag` instances? `Node` trees? Constrained by what predicate?
- Composition with #85 quantifier: ForAllPrograms<P, ProgramGenerator<C>> means "for all programs from generator G matching predicate P"
- Examples / fixtures: representative test cases the generator must cover (e.g., 2+ algebraic constructs per #1.6 demonstration discipline minimum bar)
- Naming: `ProgramGenerator<C>`, `ProgramShapeFamily<S>`, `TestProgramSeed` — grep `dsl/std/`
- Adjacency: existing fixture patterns in `tests/fixtures/`; do those become structured `.dag` data via this carrier?

**Director ratification needed before brief authoring** (substrate-canvas-tier).

## §4. Phase 2 brief authoring scope (#87)

**Verification Mgr authors worker brief** (PM-authorable now; substrate canvases for #85/#86 not blocking):

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
- Receipt: SG-0 census `EXPECTED_HAND_AUTHORED_TEST` count drops to 0 (or Director-allocated exceptions only)

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

**Phase 1 (parallel)**: 1-2 weeks per substrate canvas + carrier authoring (Substrate Mgr can run #85/#86 parallel with worker pairs)
**Phase 2**: 1-2 weeks for #87 discipline pattern + first migration receipt
**Phase 3**: 2-4 weeks for bulk-port (parallel per-class workers; staffing-not-a-concern)

**Total Cluster M close**: **4-8 weeks** from Phase 1 dispatch start. Fits in 8-12 week R3 window if dispatch starts immediately.

**Risk**: substrate canvases for #85/#86 may surface design questions that need Director ratification (not just Mgr-tier disposition). Each canvas-tier ratification adds 1-3 days to Phase 1 critical path. Account for 2-3 ratification cycles in worst case.

## §7. Cross-Mgr coordination requirements

- **Substrate Mgr ↔ Verification Mgr**: Phase 1 carriers (#85/#86) consumed by Phase 2 discipline (#87); coordinate consumer-readiness signals.
- **Verification Mgr ↔ all lane Mgrs**: Phase 3 bulk-port touches tests across all lanes (m1/m2 boundary, T-Free-Consequences, T-Lens-*, etc.). Per-class briefs may need lane-Mgr partner sign-off on test-ownership.
- **PB Mgr cross-reference**: some Phase 3 classes ("R3 PB Row-4 corpus seed", "Differential equality PB-Runtime tests") consume PB Item 4 disposition — coordinate with PB Mgr lane-state.

## §8. Dispatch readiness checklist (for Director ratification)

### §8.1 Existing brief inventory (grep-verified at HEAD)

[`docs/briefs/r3-v-tests-as-data-v1-worker.md`](../briefs/r3-v-tests-as-data-v1-worker.md) is **PRE-AUTH DISPATCH-READY** (tier-1 queue #1859) and covers all four gates (#84/#85/#86/#87). Brief shape: "single worker coordinates lane closure."

**Director ratification needed**: which dispatch shape under operator "staffing not a concern" directive?
- **(α)** Existing single-coordinator brief — one worker authors all four gates serialized within lane
- **(β)** Re-shape to 4 parallel-worker briefs — Substrate Mgr partner on #85 + #86, Verification Mgr partner on #87 + #84
- **(γ)** Hybrid — single coordinator authors #87/#84 (Verification scope) + Substrate Mgr authors substrate canvases for #85/#86 in parallel

**PM recommendation**: **(γ)** — separates substrate-shape introduction (Substrate authority) from consumer-discipline (Verification authority). Aligns with §2 lane-Mgr partition. Existing brief becomes the Verification-side coordinator brief; Substrate Mgr authors substrate canvases independently.

### §8.2 Dispatch sequence after Director ratifies plan + dispatch shape

- [ ] Substrate Mgr authors #85 substrate canvas → Director ratifies → worker brief → dispatch
- [ ] Substrate Mgr authors #86 substrate canvas → Director ratifies → worker brief → dispatch (parallel with #85)
- [ ] Existing `r3-v-tests-as-data-v1-worker.md` cited as Verification-side coordinator brief; dispatched when #85/#86 partial-land (or per Director ratification timing)
- [ ] §1.8 ledger Status updated as each gate transitions DECLARED → CONSUMER_LANDED → PASSING

## §9. Open questions for Director ratification

1. **Substrate canvas authoring authority**: do #85 + #86 canvases route through Substrate Mgr standing authority, or do they need Director-tier canvas-tier ratification (operator directive said "operator-tier spawn-authority lacks tooling" but didn't address canvas-tier authoring)?

2. **Phase 3 bulk-port discipline**: is bulk-port a single Verification Mgr coordinator role (PM recommendation) or distributed per-lane (each lane Mgr migrates their own tests)? Both are viable; PM defaults to coordinator for cleaner sequencing tracking.

3. **#84 closure criterion under Director-allocated exceptions**: Director Option 2 ratification (cross_target_coverage_carrier_test.rs etc. stays hand-Rust until testgen covers) — does Phase 3 close fold those into Director-allocated exceptions, or does testgen need to cover them too? Affects whether #84 can fire while exceptions persist.

---

**End of plan.**
