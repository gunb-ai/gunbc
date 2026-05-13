# R3 Cluster M Phase 3 — L4/L7/L5 Skeleton Family Bulk-Port Worker Brief

**Status:** STUB (coordinator framework; Mgr finalization required before dispatch).

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`).

**Authority (cite-and-execute):**
- Phase 3 coordinator: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §2 row "L4/L7/L5 skeleton"
- Pattern-A / Verification Cluster G context: [`r3-v-pattern-a-coverage-rollup.md`](r3-v-pattern-a-coverage-rollup.md), [`r3-v-l4-l7-direct-worker.md`](r3-v-l4-l7-direct-worker.md), [`r3-v-l5-corpus-worker.md`](r3-v-l5-corpus-worker.md)
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §3 migration audit + §6 implementation order
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs`
- Director Ask 4 ratification: strict-zero close; no closure-allowed exceptions

---

## §0. Scope

Migrate the **L4/L7/L5 skeleton family**: verification skeleton tests and Pattern-A strict-fire/deferred wrappers that currently stand in Rust while the underlying claims move to `.dag`.

Owned files/surfaces:
- `r3_verification_l4_l7_l5_skeleton_test.rs`
- `tc1_substrate_lens_eta_equivalence_deferred_test.rs`
- `tc1_substrate_lens_eta_equivalence_strict_fire_test.rs`
- `tc2_church_rosser_strict_fire_test.rs`
- `tc3_strong_normalization_deferred_test.rs`
- `tc3_strong_normalization_strict_fire_test.rs`
- replacement `.dag` TestClaims / corpus metadata under `src/v3/compiler/tests/dag/`
- `EXPECTED_HAND_AUTHORED_TEST` rows for each migrated file

Some TC entries overlap with the Generic DimensionReport class. Coordinator finalization must choose one owning class per entry to avoid double-dispatch.

## §1. Migration Pattern

For Pattern-A DimensionReport entries, prefer the Generic DimensionReport runner-discipline pattern. For L4/L7/L5 skeleton entries that encode coverage state rather than a DimensionReport comparison, express the coverage obligation as `.dag` `TestClaim` data or generated corpus metadata.

## §2. Dispatch Trigger

Dispatch after Verification Cluster G has a stable Pattern-A consumer path for the selected entries. Do not dispatch the same TC file from both this class and the Generic DimensionReport class.

## §3. Acceptance

- Each migrated entry has exactly one owning class and one replacement data/generated-test artifact.
- Same-PR SG-0 rule: replacement artifact, class ownership, and census decrement land together.
- Verification Mgr signs off on L4/L7/L5 behavioral fidelity.
- SG-0 test census decreases by the migrated count.
- Pattern-A strict-fire/deferred semantics are preserved.

## §4. STOP and Escalate

Escalate if an entry's current Rust skeleton is still representing a not-yet-landed semantic carrier, if the class boundary overlaps unresolved DimensionReport ownership, or if strict-fire semantics would be weakened. Rows that need structural/compiler fixes before cementing route to D2 with the blocker named.

---

**Mgr-finalization checklist:**
- [ ] De-duplicate overlap with Generic DimensionReport class.
- [ ] Identify skeleton-only entries vs Pattern-A DimensionReport entries.
- [ ] Confirm Cluster G owner signoff route.

**End of stub.**
