# R3 Cluster M Phase 3 — Cementing-Test Family Bulk-Port Worker Brief

**Status:** PRE-AUTH CLASSIFICATION HANDOFF (G87-D5 refresh). Live cementing-looking
rows are classified with owner lane + census-delta disposition; concrete
port workers still dispatch only when the named blockers unblock.

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`). **Authority chain:** Phase 3 coordinator finalization → auto-spawn dispatch.

**Authority (cite-and-execute):**
- Phase 3 coordinator: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §2 row "Cementing-test family"
- Landed cementing pattern synthesis: [`r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md) — predicate-taxonomy reference this worker enumerates against
- Phase 2 pattern brief: [`r3-v-cluster-m-87-cementing-worker.md`](r3-v-cluster-m-87-cementing-worker.md)
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §5 cementing discipline + §C5 predicate-class table
- Standing testing discipline: [`TESTING.md`](../../TESTING.md) Band C cementing tests
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`
- Director Ask 4 ratification: strict-zero close; no closure-allowed exceptions

---

## §0. Scope

Migrate the **cementing-test family** from hand-Rust integration tests into `.dag` `TestClaim` cementing receipts using the landed #87 pattern/dispatch receipt.

Owned files/surfaces for Mgr inventory:
- `src/v3/compiler/tests/integration/cementing/`
- live gate-#87 dispatch surfaces: `src/v3/compiler/tests/dag/cementing_dispatch.dag` + `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`
- lens behavioral completion tests that assert v2-vs-v3 or expected-output parity
- replacement `.dag` cementing claims under `src/v3/compiler/tests/dag/`
- `EXPECTED_HAND_AUTHORED_TEST` rows for each migrated file

Live inventory rule: use `EXPECTED_HAND_AUTHORED_TEST` at dispatch time. The current
G87-D5 handoff classifies the complete live `tests/integration/cementing/`
slice in [`r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md)
§3. Do not copy that table here; it is the single owner-lane /
census-delta disposition surface for cementing-looking hand-Rust rows.

## §1. Migration Pattern

Use the landed cementing pattern synthesis exactly:
- v2 counterpart exists: `DifferentialEquals`
- v3-native expected output: `LensOutputEquals`
- symbolic-cost contract: `SymbolicCostExprEquals`
- helper-only or intentionally narrower rows: `Compiles` plus paired receipt
- blocked rows: named hand-Rust blocker routed to D2
- landed #87 dispatch receipt confirms the `regen.dag` registry corpus has matching `.dag` receipts

No new predicate family is in scope for this class. If an entry cannot fit the #87 predicate axis, STOP and ping the Coordinator.

## §2. Dispatch Trigger

Dispatch from the current #87 `CONSUMER_LANDED + PASSING` state after the Coordinator
confirms that the row is V2-portable at dispatch time. At the G87-D5 handoff:
- `cementing_provenance_origin_integration_test.rs` is a V2 row blocked on
  sum-typed expected-carrier authoring; owner lane: tests-as-data carrier
  completeness for sum-typed lens outputs; census delta: `-1` when the
  per-`Behavior` mirror moves into the existing `.dag` receipt.
- `complexity_lens_behavioral_completion.rs` is a V2 row blocked on
  `Gate73_ReportPredicateCarriers`; owner lane: T-LBP / gate #73
  report-predicate carrier authoring; census delta: `-1` with the new
  complexity `.dag` receipt.
- `cost_lens_symbolic_consumer_test.rs` is not a Band-C cementing-class
  row; owner lane: gate #78 host-wrapper retirement; census delta:
  `0` for this class.
- `memory_peak_cost_basis_demo.rs` is a V2 row blocked on the parser-level
  `apply_lens(... Enforce { budget: SymbolicCost { dimension: Memory, ... } })`
  consumer; owner lane: T-LAS Slice B / gate #91 and gate #94 receipt;
  census delta: `-1` with the memory-peak `.dag` receipt.

## §3. Acceptance

- Each migrated entry deletes the corresponding hand-Rust test file or removes it from `EXPECTED_HAND_AUTHORED_TEST` via generated-test replacement.
- Same-PR SG-0 rule: the replacement `.dag` claim and `EXPECTED_HAND_AUTHORED_TEST` decrement land together; no census-only or artifact-only PR.
- Each migrated entry has an equivalent `.dag` cementing `TestClaim`.
- SG-0 test census decreases by the number of migrated entries.
- Substrate/Verification signoff confirms behavioral fidelity.

## §4. STOP and Escalate

Escalate if the entry requires a new predicate, if the lens is not behaviorally complete enough to cement, if runner enumeration cannot consume the new claim, or if the proposed migration leaves a Rust shadow test behind. Rows that resist cementing because they need structural/compiler fixes route to D2 with the blocker named; they are not silently kept in V2 scope.

---

**Mgr-finalization checklist (G87-D5):**
- [x] Complete live `tests/integration/cementing/` inventory.
- [x] Mark each entry as V2-portable blocked row, non-cementing owner-lane
  row, or future `LensOutputEquals` / `Compiles` candidate.
- [x] Record owner lane and SG-0 census delta disposition in the pattern
  brief §3 and live census row comments.
- [ ] Select 1-3 pilot entries after a named blocker unblocks.
- [ ] Confirm selected pilot/bulk split against the landed #87 pattern.

**End of stub.**
