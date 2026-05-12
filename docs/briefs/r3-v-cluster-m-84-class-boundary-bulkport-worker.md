# R3 Cluster M Phase 3 — Boundary-Test Family Bulk-Port Worker Brief

**Status:** STUB (coordinator framework; Mgr finalization required before dispatch).

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`).

**Authority (cite-and-execute):**
- Phase 3 coordinator: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §2 row "Boundary tests"
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 migration audit + §6 implementation order
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`
- Director Ask 4 ratification: strict-zero close; no closure-allowed exceptions

---

## §0. Scope

Migrate the **boundary-test family**: milestone/boundary tests such as `m1_*`, `m2_*`, and boundary directory emission tests that currently encode acceptance behavior in Rust.

Owned files/surfaces for Mgr inventory:
- `src/v3/compiler/tests/boundary/m1_3_emit_go_test.rs`
- `src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs`
- `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs`
- `src/v3/compiler/tests/boundary/m1_5_emit_omni_demo_test.rs`
- `src/v3/compiler/tests/boundary/m2_emit_multi_field_struct_variant_test.rs`
- integration tests with `m1_` / `m2_` prefixes
- replacement `.dag` `ExecuteCommand` / TestClaim files or generated target-language tests
- `EXPECTED_HAND_AUTHORED_TEST` rows for each migrated file

**[Mgr-fill]: complete live inventory** from `EXPECTED_HAND_AUTHORED_TEST` at dispatch time.

## §1. Migration Pattern

Boundary tests should become `.dag` TestClaims or generated target-language tests depending on what the original Rust assertion observes:
- compiler behavior / acceptance verdict: `.dag` `TestClaim`
- emitted target artifact behavior: generated target-language test code
- cross-target equivalence: `.dag` suite invoking the existing runner discipline

The worker preserves the exact boundary contract; it must not soften acceptance semantics to make the migration easier.

## §2. Dispatch Trigger

Dispatch after the runner can execute the needed TestClaim shape for the selected boundary class. Individual boundary pilots can start earlier when the assertion is a direct DB-15-style `TestClaim`.

## §3. Acceptance

- Each migrated boundary test has an equivalent data/generated-test assertion.
- Same-PR SG-0 rule: the replacement artifact and `EXPECTED_HAND_AUTHORED_TEST` decrement land together.
- The original lane owner for the boundary milestone signs off on behavioral fidelity.
- SG-0 test census decreases by the number of migrated entries.
- No new hand-Rust helper layer is introduced.

## §4. STOP and Escalate

Escalate if the boundary assertion depends on host-specific side effects, target execution not represented by the runner, or a missing generated-test backend. Rows that need compiler/runtime fixes before cementing route to D2 with the blocker named. These are Cluster M scope-expansion asks, not closure exceptions.

---

**Mgr-finalization checklist:**
- [ ] Split inventory by milestone owner (`m1`, `m2`, omni, target-specific boundary).
- [ ] Choose direct-claim pilots before generated-test pilots.
- [ ] Confirm lane-Mgr signoff route for each sub-batch.

**End of stub.**
