# R3 Cluster M Phase 3 — R1C-D / R1C-E Wrapper Family Bulk-Port Worker Brief

**Status:** STUB (coordinator framework; pilot brief exists; Mgr finalization required for full class dispatch).

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`).

**Authority (cite-and-execute):**
- Phase 3 coordinator: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §2 row "R1C-D / R1C-E wrappers"
- Pilot brief: [`r3-v-cluster-m-r1c-d-e-pilot-worker.md`](r3-v-cluster-m-r1c-d-e-pilot-worker.md)
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 migration audit + §6 implementation order
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`
- Director Ask 4 ratification: strict-zero close; no closure-allowed exceptions

---

## §0. Scope

Migrate the **R1C-D / R1C-E wrapper family**: residual R1-close tests that wrap PB census gates and emit-gate checks.

Owned files/surfaces:
- `r1c_d_pb_census_gates_test.rs`
- `r1c_e_emit_gates_dag_test.rs`
- `r1c_e_emit_gates_omni_dag_test.rs`
- replacement `.dag` wrapper claims under `src/v3/compiler/tests/dag/`
- `EXPECTED_HAND_AUTHORED_TEST` rows for each migrated file

The pilot brief is the working pattern source. This class stub becomes the full-batch dispatch surface after pilot lessons are folded back into the coordinator.

## §1. Migration Pattern

Use the pilot's direct `.dag` TestClaim wrapper pattern. Preserve the exact gate names and pass/fail behavior from the R1C tests; the migration should make the wrapper data-authored, not reinterpret the R1C gate.

## §2. Dispatch Trigger

Eligible before full Phase 3 if the pilot is explicitly approved by the Coordinator. Full class dispatch waits for pilot receipt and a short lessons section in the coordinator or PR body.

## §3. Acceptance

- All R1C-D/E wrapper entries in the live census are ported or have a Coordinator-filed scope-expansion blocker.
- Pilot lessons are applied to the full class.
- Same-PR SG-0 rule: each replacement wrapper claim and census decrement land together.
- SG-0 test census decreases by the migrated count.
- R1/PB or owning lane signoff confirms fidelity.

## §4. STOP and Escalate

Escalate if a wrapper is still asserting behavior that only exists as Rust-side helper code, if a PB census check needs substrate shape not yet available, or if the port would duplicate the gate in both Rust and `.dag`. Rows needing structural/compiler fixes route to D2 with the blocker named.

---

**Mgr-finalization checklist:**
- [ ] Import pilot lessons.
- [ ] Confirm the live R1C-D/E inventory.
- [ ] Select full-class worker split, likely one worker if count remains near three.

**End of stub.**
