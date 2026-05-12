# R3 Cluster M Phase 3 — Reflected-Dag Structural Assertion Family Bulk-Port Worker Brief

**Status:** DIRECTOR-SCAFFOLD (authored under standing authority per `feedback_director_mgr_energy_input` 2026-05-11; awaiting Mgr-tier finalization). **CONSUMER-GATED** on `TestSuite.claims` flip from `List<TestClaim>` → `List<SuiteClaim>` (staged trigger at `src/v3/std/verification.dag:394-399`); substrate carriers are landed (per Verification Mgr `clever-tern-670` msg_755c3f43).

**Owner:** worker TBD on dispatch. **Coordinator:** R3 Verification Mgr (`clever-tern-670`). **Authority chain:** Director scaffold-fill (zesty-bear-812 msg_eb2372c7) → Verification Mgr finalization → auto-spawn dispatch.

**Authority (cite-and-execute):**
- Phase 3 coordinator: [`r3-v-cluster-m-84-bulkport-coordinator.md`](r3-v-cluster-m-84-bulkport-coordinator.md) §2 row "Reflected-Dag structural assertion family"
- Locked design: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §2.1 (ProgramGenerator consumer pattern) + §3 migration audit + §6 implementation order
- Substrate carriers (landed): `src/v3/std/verification.dag:118-133` (`ProgramGenerator`, `ProgramShape`); `src/v3/std/verification.dag:379-402` (`Quantifier`, `QuantifiedTestClaim`, `SuiteClaim`)
- Census authority: `src/v3/compiler/tests/integration/sg0_census_test.rs` (line ranges in §0 below)
- Director Ask 4 ratification: strict-zero close (no closure-allowed exceptions); SG-0 `EXPECTED_HAND_AUTHORED_TEST` = 0
- Sequencing plan: [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §5.2

---

## §0. Scope

Migrate the **reflected-Dag structural assertion family** — Rust integration tests that observe structural facts (filename existence, std/ row authority, substrate carrier shape, bootstrap reflection) — to `.dag` TestClaim form so the assertion becomes data not code.

Owned files/surfaces:
- reflected/std/row-authority Rust tests under `src/v3/compiler/tests/integration/`
- replacement `.dag` TestClaims under `src/v3/compiler/tests/dag/`
- `ProgramGenerator` / `ProgramShape` / `QuantifiedTestClaim` / `SuiteClaim` consumers
- `EXPECTED_HAND_AUTHORED_TEST` rows for each migrated file

**Seed inventory (16 of estimated 25-30 entries; Mgr finalizes full list)** per Verification Mgr msg_755c3f43:

Primary census comment blocks: `sg0_census_test.rs:370-378` (`cross_target_coverage_carrier_test`), `461-528` (lens/method/projection family), adjacent reflected/std rows at `622-632` + `659-667`.

| # | File (under `src/v3/compiler/tests/integration/`) | Reflection target |
|---|---|---|
| 1 | `cross_target_coverage_carrier_test.rs` | substrate carrier shape |
| 2 | `lens_substrate_carrier_test.rs` | lens-substrate carrier |
| 3 | `method_registry_test.rs` | method-registry authority |
| 4 | `method_template_contract_test.rs` | method-template contract |
| 5 | `pb_method_template_projection_test.rs` | PB method-template projection |
| 6 | `services_carrier_shape_test.rs` | service carrier shape |
| 7 | `sg1_tokenize_authority_test.rs` | tokenize authority row |
| 8 | `sg2_parse_authority_test.rs` | parse authority row |
| 9 | `sg2c1_parse_tables_authority_test.rs` | parse-tables authority |
| 10 | `sg2c5_soft_keyword_ident_test.rs` | soft-keyword identifier |
| 11 | `sg3_lower_parse_surface_stack_test.rs` | lower parse-surface stack |
| 12 | `sg3_surface_reflection_consumer_test.rs` | surface-reflection consumer |
| 13 | `sg6_hand_authored_census_test.rs` | hand-authored census (self-referential) |
| 14 | `sg7_prep_variant_payload_freshness_test.rs` | prep variant-payload freshness |
| 15 | `timing_lens_substrate_carrier_test.rs` | timing-lens substrate carrier |
| 16 | `value_body_substrate_mirror_isomorphism_test.rs` | value-body substrate-mirror isomorphism |

**[Mgr-fill]: full 25-30 entry list** — L1/M1/M2 substrate/lens migration tests likely round out the class; second-pass sweep required before final dispatch.

## §1. Migration pattern

Each test asserts a **structural fact** that can be re-expressed as a `TestClaim` consuming the landed `ProgramGenerator` / `ProgramShape` / `QuantifiedTestClaim` substrate (post-`TestSuite.claims` flip).

Migration shape per locked design §2.1:
- Identify the structural observable (e.g., "every `lens_*.dag` declares a `LensRegistryEntry`")
- Express as `QuantifiedTestClaim` with appropriate `Quantifier` (`ForAll` / `Exists`)
- Wrap the test program as `ProgramShape` via `ProgramGenerator`
- Replace hand-Rust `assert_eq!`/`assert!` with the `.dag` `TestClaim` evaluation

Each entry's specific migration shape **[Mgr-fill]** — depends on which structural axis the test observes.

## §2. CONSUMER-GATED status

Per Verification Mgr status pass (msg_755c3f43):
- ✓ `ProgramGenerator`, `ProgramShape`, `Quantifier`, `QuantifiedTestClaim`, `SuiteClaim` carriers **landed** in `src/v3/std/verification.dag`
- ✗ `TestSuite.claims` field still typed `List<TestClaim>` (not `List<SuiteClaim>`) at `verification.dag:404-407`; staged trigger at `:394-399`
- ✗ Bootstrap verification of full wrapper/runner path exists in `m1_5_verification_test.rs:81-111` but full consumer pipeline not yet flipped

**Worker action**: STOP and PING via dashboard-message to Verification Mgr if migration of a specific entry requires the `List<SuiteClaim>` consumer path before it lands. The first 1-2 entries should be dispatched as **pilots** to validate the migration pattern under the current substrate state; full-class parallel dispatch waits for the consumer-flip event.

## §3. Hard constraints

1. **No new substrate carriers.** Use the landed `ProgramGenerator` / `ProgramShape` / `Quantifier` / `QuantifiedTestClaim` / `SuiteClaim` set. New carrier requests STOP-and-PING to Coordinator.
2. **Hand-Rust budget: zero.** Each migration deletes a `_test.rs` file (SG-0 decrement = `-N`). Net positive hand-Rust additions are a worker-stop signal.
3. **`.dag` TestClaim under `src/v3/compiler/tests/dag/`** — the migrated test lives in `.dag` form.
4. **No closure-allowed carve-outs** (per Director Ask 4 ratification). If a specific entry cannot migrate, escalate as Cluster M scope expansion (substrate-shape ask), NOT as a per-test exception.
5. **Behavioral fidelity required**: each migration preserves the exact structural assertion behavior. Lane-Mgr signoff per Phase 3 coordinator §0 (Verification Mgr coordinates; lane Mgrs review fidelity).

## §4. Acceptance

The bulk-port PR-set is acceptable when:
- `EXPECTED_HAND_AUTHORED_TEST` SG-0 census decreases by **N** (one per migrated entry; 25-30 for full class)
- Same-PR SG-0 rule: each replacement `.dag` structural claim and census decrement land together.
- Each migrated entry has a corresponding `.dag` `TestClaim` evaluating to the same Pass/Fail verdict as the original Rust assertion
- Lane-Mgr (per-test-owning lane) signs off on behavioral fidelity in the PR
- No new substrate carriers introduced
- The deleted `_test.rs` paths are removed from `EXPECTED_HAND_AUTHORED_TEST` in the same PR

## §5. Decomposition

Recommended split (subject to Mgr judgment per Phase 3 dispatch latency observations):
- **Pilot wave** (3-5 entries): pick the most reflective-shape-uniform tests (e.g., `sg1_tokenize_authority_test.rs`, `sg2_parse_authority_test.rs`, `sg2c1_parse_tables_authority_test.rs` — SG-row authority assertions follow a single pattern)
- **Class wave** (10-15 entries): parallel-dispatch by sub-axis (carrier-shape vs row-authority vs reflection-consumer)
- **Long-tail** (remaining): per-entry escalation review if migration requires substrate shape

## §6. STOP and escalate

Escalate via dashboard-message to Verification Mgr if:
- A migration requires the `TestSuite.claims = List<SuiteClaim>` consumer path before consumer-flip event
- A migration requires a new substrate carrier
- A `.dag` TestClaim cannot express the structural assertion (substrate gap)
- A migration triggers behavioral drift (lane-Mgr signoff fails)
- SG-0 census wouldn't decrease (net-zero or positive delta) — this means migration shape is wrong
- The row resists cementing because a structural/compiler fix is required; route to D2 with the blocker named

Do not push a workaround PR for any of these.

---

**Mgr-finalization checklist** (before flipping to PRE-AUTH DISPATCH-READY):
- [ ] Complete the 25-30 entry inventory (sweep L1/M1/M2 substrate/lens migration tests)
- [ ] Per-entry assertion-shape classification (carrier-shape vs row-authority vs reflection-consumer)
- [ ] Pilot wave selection (3-5 entries) + first-PR dispatch
- [ ] Confirm consumer-flip event timeline with Substrate Mgr (`warm-wolf-698`) for the post-pilot waves

**End of scaffold.** Director-tier shape established; Mgr fills inventory + per-entry detail + pilot selection.
