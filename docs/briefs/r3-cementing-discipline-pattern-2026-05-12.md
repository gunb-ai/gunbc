# R3 Cementing-Discipline Pattern Brief — Wave-1 V1 (2026-05-12)

**Owner**: Verification Mgr (clever-tern-670). Authored by tidy-wolf-223.

**Role**: Wave-1 deliverable on the dependency-graph dispatch plan
([`docs/r3-remaining-work-dependency-graph.md`](../r3-remaining-work-dependency-graph.md))
gating Wave-2 **#84 Phase 3 bulk-port** (6 per-class workers, ~80–90 SG-0
dissolutions). This brief codifies the cementing-discipline pattern that
the #84 Phase 3 workers replicate.

**Status**: pattern is **POST-LANDED** — `§1.8` row **#87
`lens_cementing_test_discipline_complete`** is **CONSUMER_LANDED +
PASSING** ([`docs/r3-program-plan.md`](../r3-program-plan.md) row #87 +
PR #2639 + PR #2757). This brief is a synthesis surface, not a
fresh-design ask. Substantive authority lives upstream; the brief
cites-and-routes.

**Substrate-of-truth (do not restate; cite-and-execute)**:
- [`TESTING.md`](../../TESTING.md) §"Cementing tests (Band C — lens subsumption)" — canonical pattern prose.
- [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §5 + §C5 — design authority (cementing v2 oracle + predicate-class table).
- [`docs/v3-lens-capability-register.md`](../v3-lens-capability-register.md) — register prose + Band-C inventory.
- `src/v3/std/verification.dag` — `lens_capability_register_rows` data + `TestPredicate` variants.
- `src/v3/compiler/regen.dag` — `LensRegistryEntry` registry rows (gate-#87 §Acceptance corpus).
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs` — `R3_GATE_87_CEMENTING_REGEN_SUITES` runner table (single-authority inventory).
- `src/v3/compiler/tests/dag/cementing_dispatch.dag` — Band-C v2-COMPLETE dispatch receipts.
- Prior dispatch brief: [`r3-cluster-m-dispatch-verification-discipline-87-2026-05-09.md`](r3-cluster-m-dispatch-verification-discipline-87-2026-05-09.md).
- Prior worker brief (pre-#87-PASSING; **superseded for pattern facts** by this brief + TESTING.md): [`r3-v-cluster-m-87-cementing-worker.md`](r3-v-cluster-m-87-cementing-worker.md).

---

## §0. What the pattern is (one paragraph)

Every `regen.dag` `LensRegistryEntry` whose row is **BEHAVIORALLY COMPLETE**
carries a paired cementing receipt: a `.dag` `TestClaim` in
`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`
asserting v2-vs-v3 equality (real v2 counterpart) or a pinned behavioral
contract (v3-native). The runner table
`R3_GATE_87_CEMENTING_REGEN_SUITES` is the single-authority inventory:
`cementing_dispatch.dag`, `t_pb_b_1_dag_runner_test`, and
`r3_gate_87_lens_cementing_regen_receipts_test` all read from it. A new
register row cannot ship without a matching runner-table row and
`.dag` harness.

Cementing-discipline (Band-C, TESTING.md) is the **superset pattern**:
any lens-subsumption claim — register row, prose, or brief — needs a
behavioral regression that would fail under silent semantic drift.
Gate #87 PASSING is the exhaustive receipt over the `regen.dag`
corpus; non-`regen` lenses cement via Band-C / register ratchets
outside gate-#87 scope (see §3 below).

## §1. Single-authority surfaces (drift-fails the predicate)

Four artifacts must align row-for-row. Drift between any pair fails
`CementingDispatchMatchesProjection` /
`r3_gate_87_lens_cementing_regen_receipts_test`:

1. `src/v3/compiler/regen.dag` — `LensRegistryEntry` rows (the *what*).
2. `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs::R3_GATE_87_CEMENTING_REGEN_SUITES` — runner inventory (the *how-it-runs*).
3. `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` — per-lens claim file (the *receipt*).
4. `src/v3/compiler/tests/dag/cementing_dispatch.dag::cementing_band_c_v2_complete_receipts` — Band-C dispatch list (the *cross-check*).

`src/v3/std/verification.dag::lens_capability_register_rows` and
`docs/v3-lens-capability-register.md` are the prose-and-data mirrors;
they must agree with (1).

**Worker rule**: editing any one of these without paired edits across
the set is a STOP+PING. The drift test is named exactly so workers can
re-run locally: `cargo test -p v3-compiler r3_gate_87`.

## §2. Predicate-class taxonomy (Phase 3 slicing axis)

`#84` Phase 3 bulk-port allocates **6 per-class workers**. The class
axis is the `TestPredicate` variant chosen, which is determined by
register-row shape:

| Class | Predicate | Trigger | Receipt shape |
|-------|-----------|---------|---------------|
| **C-DiffEq** | `DifferentialEquals { subject_ref, oracle_ref, input_ref }` | Real v2 counterpart (register row names non-`N/A` v2 lens). | v2 oracle + v3 lens run on shared `.v3` fixture; carrier equality on published shape. |
| **C-LensOutEq** | `LensOutputEquals { lens_ref, input_ref, expected_ref }` | v3-native (register row v2 column = `None (v3-native)` / `N/A`) with full-carrier authorability. | Frozen-expected carrier declaration co-located with the lens. |
| **C-SymCostEq** | `SymbolicCostExprEquals { lens_ref, input_ref, expected_expr }` | Cost-lens family rows whose contract is a `SymbolicCost` shape. | Frozen `SymbolicCost` expr (or Int projection where full carriers not yet authorable). |
| **C-IntProj** | `IntEquals`/equivalent narrow projection | Full carrier not yet authorable (substrate-prereq named); narrows to scalar projection. | Scalar projection pinned + named blocker comment citing missing carrier. |
| **C-CompilesHelper** | `Compiles` | **Helper-only** rows (`infer_helpers`, `lower_helpers`, `variant_payload`, etc.) — register **N/A** / partial scope. | `.dag` `Compiles` + paired Rust compile receipt in `r3_gate_87_lens_cementing_regen_receipts_test`. |
| **C-HandRustBlocker** | (Rust receipt under `tests/integration/cementing/`) | Carrier-prereq blocks `.dag` expression (e.g., `SymbolicCost` nested-carrier — PR #2769 dissolving this for cost; `ComplexitySummary` for complexity #80; `MemoryPeak*` for #94). | Temporary Rust module in `SG-0 EXPECTED_HAND_AUTHORED_TEST` with named blocker; flips to C-LensOutEq / C-DiffEq when blocker dissolves. |

**Predicate-axis distinctions (do not mix)**:
- `BinaryDimensionReportEquals` belongs to Pattern-A DimensionReport
  comparisons (TC1/TC2/TC3 family) — **different axis, different gate**.
- `ProgramGenerator` / `Quantifier` / `QuantifiedTestClaim` (Phase 1
  #85/#86) is the property-based "every program in family X satisfies
  P" axis — **orthogonal to per-lens cementing** (cementing is
  single-fixture per claim).

**Inventing new `TestPredicate` variants is out of scope (STOP+PING).**

## §3. Where cementing lives (corpus split)

- **In gate #87 §Acceptance (`regen.dag` corpus)**: every `LensRegistryEntry`. Receipts in `tests/dag/t_r3_gate_87_cementing_regen_*.dag`. Exhaustive over that corpus; #87 PASSING means this set is complete + drift-checked.
- **Outside gate #87** (Band-C / register-ratchet, not gate-#87 receipts):
  - `tests/integration/cementing/cementing_provenance_origin_integration_test.rs` — v3-native provenance, integration-shape.
  - `tests/integration/cementing/complexity_lens_behavioral_completion.rs` — T-LBP gate #80 cementing (blocker: full `ComplexitySummary` carrier).
  - `tests/integration/cementing/cost_lens_symbolic_consumer_test.rs` — gate #80 cost-symbolic cementing (blocker: `M1_2_8_STRUCTURAL_SYMBOLIC_COST_DATA`; **in-flight dissolution: PR #2769** under sibling worker tidy-gull-504).
  - `tests/integration/cementing/memory_peak_cost_basis_demo.rs` — T-LAS gate #94 demonstration.
  
  These four remain in `EXPECTED_HAND_AUTHORED_TEST`. Each carries a
  named blocker comment. As blockers dissolve, the module ports to
  `tests/dag/cementing_<lens>.dag` and the SG-0 census decrements by
  one (PR body: `SG-0 hand-path delta: -1`).

## §4. Phase 3 bulk-port worker template (#84 Wave-2 dispatch)

Each Phase 3 worker owns **one C-class slice from §2** across the SG-0
hand-Rust cementing surface (currently 4 modules in
`tests/integration/cementing/` + cementing-shape hand-Rust elsewhere
in `EXPECTED_HAND_AUTHORED_TEST` — exact count is worker-grep at
dispatch against `sg0_census_test.rs`). Per-worker acceptance:

1. Port each in-class hand-Rust cementing module to a `.dag` claim per §2 shape, deleting (not stub-replacing) the Rust file.
2. If the row is gate-#87-corpus-eligible (`regen.dag` row): add a `t_r3_gate_87_cementing_regen_<lens>.dag` harness **and** extend `R3_GATE_87_CEMENTING_REGEN_SUITES` (single-authority surface).
3. If non-corpus: land under `tests/dag/cementing_<lens>.dag` with a register-ratchet receipt.
4. SG-0 census `EXPECTED_HAND_AUTHORED_TEST` decrements by one per port (PR body: `SG-0 hand-path delta: -N`).
5. Drift gate `cargo test -p v3-compiler r3_gate_87` green on BuildBuddy.
6. Same-PR rule: register-row promotion + cementing receipt land together (TESTING.md §Cementing §"Dispatch").

**Per-worker out-of-scope (STOP+PING)**:
- Inventing new `TestPredicate` variants (§2 fixed taxonomy).
- Porting hand-Rust modules whose lens row is **not** BEHAVIORALLY COMPLETE — that's T-Lens-Behavioral-Parity scope; cementing lands with the COMPLETE flip, not before.
- Modifying `lens_capability_register_rows` shape — Substrate Mgr authority.

## §5. Coupling to Cluster M Phase 1 (#85 `SuiteClaim` wrap)

Phase 1 → cementing coupling is **wrapper-level only**: when #85
lands the `SuiteClaim` consumer (per `r3-program-plan.md` row #85
note), existing `TestSuite.claims: List<TestClaim>` sites mechanically
migrate to wrap claims in `Enumerated(...)` (design §6 line 344).
Cementing `TestClaim`s participate in that mechanical wrap as a
follow-on; the per-claim predicate shape (§2) does **not** change.
Phase 3 bulk-port may proceed independently of #85 dispatch.

## §6. Receipt + ledger updates

- **#87**: already CONSUMER_LANDED + PASSING; this brief does not move row #87.
- **#84 `every_rust_test_ports_to_dag_or_generated`**: each Phase 3 worker PR moves SG-0 census down; row #84 stays DECLARED until census exhaustion + carrier ratchets confirm zero hand-cementing residual outside named-blocker modules.
- **§1.8 ledger**: Verification Mgr advances row #84 per §10 cadence as Phase 3 PRs land.

## §7. Cross-Mgr signoff

Per Phase 2 dispatch overlay §6 (still load-bearing): when a Phase 3
bulk-port PR lands, the original lane Mgr for the migrated lens
reviews behavioral fidelity (Substrate Mgr for cost/complexity/etc.;
Verification Mgr for verification-tier rows). This signoff workflow
scales to per-class fan-out; the brief encodes it once, Phase 3
workers cite it.

## §8. Velocity context

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](../audit/r3-pb0-velocity-walk-2026-05-09.md)
§3.3 + Phase 2 dispatch overlay §5: cementing-test class is the
single largest SG-0-dissolution class (~20–25 entries direct, more
under cascade as blocker carriers land). Phase 3 bulk-port is the
substantial bulk-dissolution sub-event en route to #84 close. With
#87 PASSING already, the Phase 3 fan-out has no remaining
substrate-prereq dependency at the **pattern** level — only per-row
substrate-prereq dependencies for individual C-HandRustBlocker
modules (§3) whose blocker tickets are tracked in the named-blocker
comments at the call site.

---

**End of brief.**
