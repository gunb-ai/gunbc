# R3 Cluster M Phase 3 — Generic DimensionReport / Runner-Discipline Family Bulk-Port Worker Brief

**Status:** DIRECTOR-SCAFFOLD (authored under standing authority per `feedback_director_mgr_energy_input` 2026-05-11; awaiting Mgr-tier finalization). **NOT CONSUMER-GATED** — Phase-2 runner-side TestClaim pattern landed via PR #2639 (CONSUMER_LANDED; full #87 PASSING not required for this class per `feedback_construction_over_ratchets` ratification at Director msg_eb2372c7).

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`). **Authority chain:** Director scaffold-fill (zesty-bear-812 msg_eb2372c7) → Verification Mgr finalization → auto-spawn dispatch.

**Authority (cite-and-execute):**
- Phase 3 coordinator: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §2 row "Generic DimensionReport / runner-discipline family"
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 migration audit + §5 cementing + §6 implementation order
- Phase-2 pattern site (load-bearing reference): `src/v3/compiler/tests/integration/t_pb_b_1_dag_runner_test.rs:257-357` (`R3_GATE_87_CEMENTING_REGEN_SUITES`, derived inventory, `run_suite_all_pass_with_expected_claim_names`)
- Phase-2 receipt-discipline site: `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs:13-24` (P5 receipt discipline) + `:122-132` (registry names match fixture inventory)
- Associated `.dag` harnesses: `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs:573-583` (free-consequences batches), `:639-655` (TC1/TC2/TC3 Pattern-A DimensionReport), `:584-587` (paired Rust receipt)
- Director Ask 4 ratification: strict-zero close

---

## §0. Scope

Migrate the **generic DimensionReport / runner-discipline family** — Rust integration tests that wrap `.dag` `TestClaim` evaluation via the runner-side discipline (suite enumeration, derived inventory, expected-claim-names match) — to extend the Phase-2 pattern landed at PR #2639 across the full DimensionReport surface.

Owned files/surfaces:
- DimensionReport and runner-discipline Rust tests under `src/v3/compiler/tests/integration/`
- replacement `.dag` TestClaims under `src/v3/compiler/tests/dag/`
- runner-side suite enumeration constants and expected-claim-name checks
- `EXPECTED_HAND_AUTHORED_TEST` rows for each migrated file

**Seed inventory (10 of estimated 20-25 entries; Mgr finalizes full list)** per Verification Mgr msg_755c3f43:

Primary census anchors: `sg0_census_test.rs:573-583` (free-consequences), `:639-655` (TC1/TC2/TC3 Pattern-A), `:584-587` (paired Rust receipt), runner-side at `t_pb_b_1_dag_runner_test.rs:257-357`.

| # | File (under `src/v3/compiler/tests/integration/`) | Class shape |
|---|---|---|
| 1 | `r3_free_consequences_first_batch_test.rs` | Free-Consequences DimensionReport |
| 2 | `r3_free_consequences_second_batch_test.rs` | Free-Consequences DimensionReport |
| 3 | `tc1_substrate_lens_eta_equivalence_deferred_test.rs` | TC1 Pattern-A DimensionReport (deferred) |
| 4 | `tc1_substrate_lens_eta_equivalence_strict_fire_test.rs` | TC1 Pattern-A DimensionReport (strict-fire) |
| 5 | `tc2_church_rosser_strict_fire_test.rs` | TC2 Pattern-A DimensionReport (strict-fire) |
| 6 | `tc3_strong_normalization_deferred_test.rs` | TC3 Pattern-A DimensionReport (deferred) |
| 7 | `tc3_strong_normalization_strict_fire_test.rs` | TC3 Pattern-A DimensionReport (strict-fire) |
| 8 | `test_runner_test.rs` | Runner-discipline framework |
| 9 | `t_pb_b_1_dag_runner_test.rs` | Runner-side #87 table pattern (Phase-2 reference; **may stay as runner-side scaffolding** — see §3.2) |
| 10 | `r3_gate_87_lens_cementing_regen_receipts_test.rs` | Phase-2 receipt-discipline reference (**likely stays** as Phase-2 pattern site) |

**[Mgr-fill]: full 20-25 entry list** — runner-discipline framework entries (test_runner_test.rs lineage) and remaining TC4-TC7 / TC-X DimensionReport tests fill the residual.

## §1. Migration pattern (extend Phase-2 PR #2639 receipt)

Phase-2 pattern at `t_pb_b_1_dag_runner_test.rs:257-357` established the canonical shape:
1. `.dag` declares the `TestClaim` (e.g., `cementing_regen_cost_target_realization_runs_all_pass`)
2. Runner-side derives inventory from `.dag` suite enumeration
3. `run_suite_all_pass_with_expected_claim_names(...)` evaluates the suite + checks registry names match fixture inventory
4. Receipt-discipline at `r3_gate_87_lens_cementing_regen_receipts_test.rs:13-24` proves P5 receipt parity

**Worker action per entry**:
- Extract the DimensionReport assertion from the hand-Rust test
- Express as `.dag` `TestClaim` under `src/v3/compiler/tests/dag/` (matching the `t_r3_gate_87_cementing_regen_*.dag` shape)
- Add the suite to the runner-side enumeration (extending the `R3_GATE_87_CEMENTING_REGEN_SUITES` pattern with a class-appropriate constant — `R3_FREE_CONSEQUENCES_SUITES`, `R3_TC_DIMREPORT_SUITES`, etc.)
- Delete the hand-Rust `_test.rs` file
- Update SG-0 census (remove entry from `EXPECTED_HAND_AUTHORED_TEST`)

**[Mgr-fill]: per-entry migration shape detail** — TC1/TC2/TC3 deferred-vs-strict-fire variants may share suite enumeration; Free-Consequences first/second-batch may bundle.

## §2. Phase-2 pattern reuse vs extension

The Phase-2 pattern landed for the **cementing-test family** (#87 lineage). This brief extends it to the DimensionReport class. Two extension axes:

**(a) Mechanical extension** (most entries): same `run_suite_all_pass_with_expected_claim_names` shape; new `_SUITES` constant per class; `.dag` `TestClaim` matches existing fixture inventory pattern.

**(b) Suite-enumeration extension** (some entries): if the runner-side derived inventory needs to grow beyond `R3_GATE_87_CEMENTING_REGEN_SUITES`, the suite-enumeration discipline itself extends. This is **runner-substrate work** — escalate to coordinator if scope exceeds mechanical extension.

## §3. Hard constraints

1. **Phase-2 pattern is the load-bearing predicate** — do not invent a new TestClaim or runner discipline. If the existing pattern doesn't cover an entry's assertion shape, STOP and escalate.

2. **Two entries likely STAY as runner-side scaffolding** (not migrated; the framework itself):
   - `t_pb_b_1_dag_runner_test.rs` — the runner framework that hosts all the migrated suites (this file is the consumer; cannot port itself)
   - `r3_gate_87_lens_cementing_regen_receipts_test.rs` — Phase-2 receipt-discipline reference

   These entries should be flagged in the Mgr-finalization step as "Cluster M scope expansion required" — they need a separate decision per Director Ask 4 ratification (NOT closure-allowed; if they cannot migrate, the scope expands to find a path, NOT carve).

3. **Hand-Rust budget: zero net** (subject to §3.2). Each migration deletes a `_test.rs` file. The 2 framework-class entries (§3.2) are out-of-scope for THIS bulk-port; they need substrate-shape resolution.

4. **`.dag` TestClaim under `src/v3/compiler/tests/dag/`** matching the `t_r3_gate_87_cementing_regen_*.dag` naming convention (e.g., `t_r3_free_consequences_first_batch.dag`, `t_r3_tc1_strict_fire.dag`).

5. **Behavioral fidelity** — DimensionReport assertion verdicts must match. Lane-Mgr signoff (Free-Consequences lane / TC-class lane) on the PR.

6. **No closure-allowed carve-outs** per Director Ask 4. If §3.2 framework entries cannot migrate, that's a Coordinator escalation, not a per-test exception.

## §4. Acceptance

The bulk-port PR-set is acceptable when:
- `EXPECTED_HAND_AUTHORED_TEST` SG-0 census decreases by **N** (one per migrated entry; 18-23 for full class minus the 2 framework entries from §3.2)
- Same-PR SG-0 rule: each replacement `.dag` claim and census decrement land together.
- Each migrated entry has a corresponding `.dag` `TestClaim` under `tests/dag/` evaluating to the same Pass/Fail verdict as the original Rust assertion
- Runner-side suite enumeration constants extend cleanly (no parallel suite-enumeration mechanisms introduced)
- Receipt-discipline (per Phase-2 pattern) preserved
- Lane-Mgr signoff on behavioral fidelity
- 2 framework-class entries (§3.2) explicitly noted in PR body as scope-expansion candidates

## §5. Decomposition

Recommended split (subject to Mgr judgment):
- **Pilot** (1-2 entries): pick the most Phase-2-pattern-uniform (e.g., `tc1_substrate_lens_eta_equivalence_strict_fire_test.rs` — Pattern-A DimensionReport, single suite); validate mechanical extension shape
- **TC-family wave** (5-7 entries): TC1/TC2/TC3 deferred + strict-fire variants in parallel; shared suite enumeration likely
- **Free-Consequences wave** (2-4 entries): first/second-batch pairing; possibly bundles with TC-X residuals
- **Long-tail / runner-discipline wave** (remaining): escalate per §3.2 framework decisions
- **Framework decision** (2 entries from §3.2): coordinator-tier escalation; NOT in mechanical bulk-port scope

## §6. STOP and escalate

Escalate via dashboard-message to Verification Mgr if:
- An entry's assertion shape can't extend Phase-2 pattern mechanically (substrate gap)
- Runner-side suite enumeration would need to gain a new discipline (out-of-scope substrate work)
- Framework entries (§3.2) need migration path — Coordinator decides on scope expansion vs runner-side scaffolding hold
- A `.dag` TestClaim cannot express the DimensionReport assertion
- SG-0 census wouldn't decrease
- A row resists cementing because it needs structural/compiler fixes; route to D2 with the blocker named

Do not push a workaround PR for any of these.

---

**Mgr-finalization checklist** (before flipping to PRE-AUTH DISPATCH-READY):
- [ ] Complete 20-25 entry inventory (sweep TC4-TC7 / TC-X DimensionReport residuals; runner-discipline framework files)
- [ ] Per-entry classification: mechanical extension vs framework-scope-expansion
- [ ] §3.2 framework-entries decision — Coordinator escalation path determined
- [ ] Pilot dispatch selection (1-2 entries; recommend TC1 strict-fire as canonical mechanical extension)

**End of scaffold.** Director-tier shape established + Phase-2 pattern citations grounded in PR #2639 / `t_pb_b_1_dag_runner_test.rs` / `r3_gate_87_*` references; Mgr fills inventory + per-entry classification + pilot selection.
