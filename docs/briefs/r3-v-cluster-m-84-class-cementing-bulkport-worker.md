# R3 Cluster M Phase 3 — Cementing-Test Family Bulk-Port Worker Brief

**Status:** STUB (coordinator framework; Mgr finalization required before dispatch).

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

**[Mgr-fill]: complete live inventory** from `EXPECTED_HAND_AUTHORED_TEST` at dispatch time.

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

Dispatch from the current #87 `CONSUMER_LANDED + PASSING` state after the Coordinator fills live inventory, per-entry migration shape, and selected pilot/bulk split for this class.

## §3. Acceptance

- Each migrated entry deletes the corresponding hand-Rust test file or removes it from `EXPECTED_HAND_AUTHORED_TEST` via generated-test replacement.
- Same-PR SG-0 rule: the replacement `.dag` claim and `EXPECTED_HAND_AUTHORED_TEST` decrement land together; no census-only or artifact-only PR.
- Each migrated entry has an equivalent `.dag` cementing `TestClaim`.
- SG-0 test census decreases by the number of migrated entries.
- Substrate/Verification signoff confirms behavioral fidelity.

## §4. STOP and Escalate

Escalate if the entry requires a new predicate, if the lens is not behaviorally complete enough to cement, if runner enumeration cannot consume the new claim, or if the proposed migration leaves a Rust shadow test behind. Rows that resist cementing because they need structural/compiler fixes route to D2 with the blocker named; they are not silently kept in V2 scope.

---

**Mgr-finalization checklist:**
- [ ] Complete live inventory.
- [ ] Mark each entry DifferentialEquals vs LensOutputEquals vs substrate-gap.
- [ ] Select 1-3 pilot entries.
- [ ] Confirm selected pilot/bulk split against the landed #87 pattern.

**End of stub.**
