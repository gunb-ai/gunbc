# R3 CI cold-v3 rebuild coordinator (Phase-3-pattern; per-cut child workers)

**Status:** DIRECTOR-SCAFFOLD (authored under standing authority per `feedback_director_mgr_energy_input` + PM greenlight msg_07f73de0 2026-05-12 + Brian operator greenlight at gunbc#846 reply ~01:25Z). Pre-authored DISPATCH-READY scaffold; activation triggers on empirical post-#2723 cold-v3 measurement.

**Owner:** R3 Verification Mgr (`clever-tern-670`) — coordinator role per Phase 3 pattern. **Authority chain:** Brian operator greenlight (#846 reply) → PM dispatch ratification (msg_07f73de0) → Director scaffold-fill (this file) → Verification Mgr canvas finalization + per-cut child worker dispatch.

**Activation trigger:** first post-#2723 cold-v3 wall-clock measurement on a code-touching PR. Decision branch:
- Wall **>20min**: dispatch additional cuts (second-cut session per separate brief; defer rebuild)
- Wall **10-20min**: dispatch rebuild ALONGSIDE possible second-cut (parallel; cuts give immediate relief while rebuilds deliver permanent fix)
- Wall **≤10min**: rebuild can de-prioritize OR proceed at slower cadence (Brian's target met by cuts alone)

PM relays wall-clock measurement to coordinator on first qualifying PR cycle.

**Depends on**: PR #2723 (hot-fix CUT phase) merged at `d98d1e04b` ✓

---

## §0. Scope

**Rebuild each `hot-fix-2026-05-12`-tagged cut test under OnceLock/cached_compile/shared-fixture amortization, restoring the test under <2s wall budget + removing the `#[ignore]` attribute + retiring the slow-test-exemptions.txt row + decrementing TEST_TIMEOUT_MAX_EXEMPTIONS in the same PR**.

Per-cut tests in scope (verified at main HEAD `d98d1e04b` from PR #2723 + slow-test-exemptions.txt `hot-fix-2026-05-12` markers):

| # | Test path | Lane | Cold-CI wall (per row comment) | Cluster |
|---|---|---|---|---|
| 1 | `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` | PB / Lane 3 Stage 3c | DB-8 5x re-emit | A — DB-8 emit matrix |
| 2 | `emit_matrix_module_go_is_deterministic` | PB / Lane 3 Stage 3c | DB-8 module re-emit | A — DB-8 emit matrix |
| 3 | `emit_matrix_module_python_is_deterministic` | PB / Lane 3 Stage 3c | DB-8 module re-emit | A — DB-8 emit matrix |
| 4 | `emit_matrix_module_rust_is_deterministic` | PB / Lane 3 Stage 3c | DB-8 module re-emit | A — DB-8 emit matrix |
| 5 | `emit_matrix_program_go_is_deterministic` | PB / Lane 3 Stage 3c | DB-8 program re-emit | A — DB-8 emit matrix |
| 6 | `emit_matrix_program_python_is_deterministic` | PB / Lane 3 Stage 3c | DB-8 program re-emit | A — DB-8 emit matrix |
| 7 | `emit_matrix_program_rust_is_deterministic` | PB / Lane 3 Stage 3c | DB-8 program re-emit | A — DB-8 emit matrix |
| 8 | `four_fixture_disk_sources_emit_deterministically` | PB / Lane 3 Stage 3c | four-fixture disk corpus | A — DB-8 emit matrix |
| 9 | `m1_5_testgen_test::testgen_lens_emits_claims_as_structural_testclaim_values` | Substrate (M1_5_DESIGN authority) | exhaustive sweep | B — m1_5_testgen |
| 10 | `m1_5_testgen_test::testgen_generated_claims_execute_against_compile_boundary` | Substrate | exhaustive sweep | B — m1_5_testgen |
| 11 | `m1_5_testgen_test::representative_generated_claims_cover_predicate_families` | Substrate | bootstrapped corpus query | B — m1_5_testgen |
| 12 | `m1_5_testgen_test::representative_generated_claims_execute_against_compile_boundary` | Substrate | representative execution | B — m1_5_testgen |
| 13 | `r3_verification_l4_l7_l5_skeleton_test::r3_verification_l7_algebraic_law_matrix_has_current_runner_receipts` | Verification (R3 gate #10) | ~2.4s | C — R3-V L4/L7/L5 |
| 14 | `lens_behavioral_parity_demonstration_test::r3_gate_73_demonstrates_complexity_certainty_is_proven` | Verification (R3 gate #73) | LBP cementing | D — LBP demo (gate #73) |
| 15 | `lens_behavioral_parity_demonstration_test::r3_gate_73_demonstrates_symbolic_cost_is_keyed_by_countdown_parameter` | Verification (R3 gate #73) | LBP cementing | D — LBP demo (gate #73) |
| 16 | `t_ci_workflow_as_data_demo_test::ci_workflow_as_data_demo_timing_dimension_report_evaluates_via_evaluator` | Verification (gunbc#1956 T-WAD) | ~3s wall | E — T-Workflow-As-Data |
| 17 | `t_las_complexity_contract_compile_error_test::complexity_violation_compile_error_demonstrated` | T-LAS (gate #92; **ownership gap — see §6**) | ~14s wall **(heaviest single test)** | F — T-LAS complexity contract |
| 18 | `t_pb_b_1_dag_runner_test::r3_gate_87_cementing_regen_lens_suites_pass_through_runner` | Verification (R3 gate #87) | ~3.7s wall | G — Gate #87 cementing |
| 19 | `tc1_substrate_lens_eta_equivalence_strict_fire_test::tc1_strict_fire_suite_has_canonical_executable_claim_with_valid_binary_shape` | Substrate (R3 gate #11) | **~140s wall (heaviest cold-CI contributor)** | H — TC1 strict-fire |
| 20 | `determinism_test::audit_rust_emit_text` (lineage) | PB / determinism | TBD wall | I — determinism audit |

20 cut tests = 20 rebuild sub-issues (Mgr discretion to merge sub-issues at cluster granularity if economical).

## §1. Mechanism (per-cut child worker pattern)

Each cut test gets a dedicated rebuild sub-issue. Worker scope per sub-issue:

1. **Establish baseline**: run the cut test (temporarily un-ignore locally; `cargo test --test integration <pattern> -- --include-ignored --report-time`) + record wall-clock + identify amortization opportunity (cold `compile_to_dag` repeated? bootstrap fixture re-instantiated? lens authority recomputed?)
2. **Refactor under amortization**:
   - **`OnceLock` for shared bootstrap state**: if multiple tests instantiate the same `Dag::new()` / `compile_to_dag(...)` from the same fixture, lift the result into a `static SHARED_DAG: OnceLock<Dag>` accessed by all related tests
   - **`cached_compile_to_dag` for fixture reuse**: per the existing `cached_compile_to_dag` helper at `src/v3/compiler/tests/integration/common/cached_compile.rs` (which is part of the harness/test-selection-machinery shared-infra class per Layer 2 brief #2719 §2)
   - **Shared-fixture amortization**: collapse N-way matrix tests onto 1 shared fixture build with N assertion blocks
3. **Verify <2s wall** (per-test ratchet): `RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler <pattern> -- -Z unstable-options --report-time` shows ≤2000ms on cold ubuntu-latest sole-filter run
4. **Re-enable + retire exemption**:
   - Remove `#[ignore = "hot-fix-2026-05-12 ..."]` attribute from the test function
   - Delete the test's row from `scripts/slow-test-exemptions.txt`
   - Decrement `TEST_TIMEOUT_MAX_EXEMPTIONS` in `scripts/check-test-timeout.sh` by 1 (same PR)
   - PR body cites this scaffold + the operator-tier hot-fix lineage

## §2. Hard constraints

1. **Preserve test semantics**: rebuild MUST NOT change the test's behavioral assertion. The whole point is the test was correct content + slow execution; rebuild makes it fast execution + same content. Lane-Mgr signoff on behavioral fidelity per #2722 §6 cross-coordinator pattern.
2. **Ratchet-down per PR**: each PR retires AT LEAST 1 exemption + decrements ratchet by AT LEAST 1. Per `feedback_ratchet_only_down`. No batch-rebuild PR that lands multiple cuts but only retires one row.
3. **OnceLock/cached_compile/shared-fixture ONLY**: no other amortization mechanism (mocking, fixture stubs, behavioral approximation). The structural fix is bootstrap-state-amortization; deviations escalate per §7.
4. **Shared-fixture helpers require P5 receipt + SG-0-neutrality** (per Brian BLOCKING #1 + codex BLOCKING #9XXX absorption 2026-05-12): any line-expansion or new-file in `src/v3/compiler/tests/integration/common/*` (or anywhere under `src/v3/`) triggers INVARIANTS.md **P5 (Pure Bootstrap zero)** receipt obligation. The shared-fixture helper carve-out is **NOT** an exemption from P5; helpers are still hand-Rust under `src/v3/`. **Per-PR P5 receipt**: PR body cites (a) the helper line/file change with concrete LOC delta, (b) the dissolution path for the helper (e.g., "helper retires when the cluster's rebuild-pattern lands in `.dag` TestClaim authority"), and (c) SG-0 census-delta computation showing net ≤ 0 per `feedback_pb_zero_is_r3_close_target`. **SG-0-neutrality enforcement** (per cursor #9834 absorption 2026-05-12 — SG-0 census and exemption list are SEPARATE mechanisms; the two MUST NOT be conflated): helpers may add lines but the PR's net SG-0 census delta MUST be ≤ 0. SG-0 census counts hand-Rust files/lines per `sg0_census_test.rs` (EXPECTED_HAND_AUTHORED_NON_TEST + EXPECTED_HAND_AUTHORED_TEST + EXPECTED_HAND_AUTHORED_FRAGMENTS); exemption-row retirement reduces `slow-test-exemptions.txt` line count and is a DIFFERENT bookkeeping. The correct offset path: helper-LOC additions in `common/*` MUST be offset by EQUAL-or-greater LOC reductions in the per-test files that consume the helper (shared fixture extraction → per-test setup boilerplate dropped from each cluster member). Exemption-row retirement + ratchet-down are independent obligations (per constraint #2) and do NOT count toward SG-0 census-delta. If helper expansion would push SG-0 census net positive even after consumer-side LOC offset, escalate per §5 — that's a substrate-shape signal that the rebuild approach needs different mechanism.
5. **Per-cluster behavioral fidelity**: tests in same cluster (e.g., 7 DB-8 emit matrix entries) MAY share the same fixture; verify each test still asserts its specific axis (module vs program; go vs python vs rust).
6. **No re-enabling without ratchet-down**: a PR that removes `#[ignore]` without retiring the exemption row + decrementing ratchet IS a fail-open shape — exemption stays in ledger but test runs unchecked. Reject in review.

## §3. Acceptance

The cold-v3 rebuild PR-set is acceptable when:

- One cut test rebuilds + re-enables per PR (or per-cluster batch where Mgr discretion permits)
- Each PR removes `#[ignore]` + retires exemption row + decrements `TEST_TIMEOUT_MAX_EXEMPTIONS` in lockstep
- `RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler <pattern> -- --report-time` shows ≤2000ms wall on cold ubuntu-latest
- Lane-Mgr signoff cited in PR body (test semantics preserved)
- PR body cites this scaffold + #2722 hot-fix lineage + the specific cut's slow-test-exemptions row dissolution comment for cross-link
- After all 20 cuts rebuild: cold-v3 wall ≤10min on code-touching PRs (Brian's stated target); **ratchet floor recomputed DYNAMICALLY from live state at activation** (per Brian BLOCKING #2 + codex BLOCKING #9XXX absorption 2026-05-12): starts at current main HEAD's `TEST_TIMEOUT_MAX_EXEMPTIONS` value (84 at dfbc0100a per `scripts/check-test-timeout.sh`; verify via `grep -E '^max_exemptions|TEST_TIMEOUT_MAX_EXEMPTIONS:-' scripts/check-test-timeout.sh` at Mgr finalization). Each rebuild PR decrements floor by N where N = cuts rebuilt that PR. Post-all-20-rebuild target = `(value at activation) − 20` (e.g., 84 − 20 = 64 if activated at current state). **Removed**: the "≤80 pre-hot-fix baseline" framing. The 80 floor was ITSELF stale debt — the 16 non-hot-fix-tagged exemption rows have separate paydown owners with their own ratchet-down lifecycle; freezing the post-rebuild goal at 80 would preserve that stale debt. Long-run target per `feedback_pb_zero_is_r3_close_target` is 0 exemptions; rebuild lifecycle delivers the hot-fix-tagged subset, not a frozen historical baseline.

## §4. Decomposition (Mgr-fill)

Recommended dispatch sequence:

- **Phase 1 pilot** (1 cluster; ~half-day): pick the simplest mechanically — Cluster A (DB-8 emit matrix) is the most amortizable (7 nearly-identical tests; shared fixture trivial). Pilot rebuilds 1-2 entries to validate the OnceLock pattern + ratchet-down protocol.
- **Phase 2 high-impact cluster** (1 cluster; ~1 day): Cluster H (TC1 strict-fire, ~140s) is the single heaviest contributor — rebuilding this alone drops cold-v3 by minutes. High-signal early dispatch.
- **Phase 3 parallel rollout** (all remaining clusters): once pilot pattern validates + Cluster H heavy-test pattern proves, dispatch per-cluster workers in parallel per Phase 3-pattern (similar to Cluster M #84 bulk-port dispatch via #2715/#2716).
- **Phase 4 ratchet sweep** (cleanup PR): once all 20 cuts re-enabled, single PR drops `TEST_TIMEOUT_MAX_EXEMPTIONS` to **`(value at activation) − 20`** (e.g., 64 at current state of 84 — NOT ≤80; the 80 baseline was itself stale debt per §3 acceptance bullet's correction; 16 non-hot-fix-tagged exemption rows have separate paydown owners with their own ratchet-down lifecycle) + drops cold-CI `--timeout` budget from 4000s to ≤2000s (per PM ratchet-down candidate at msg_07f73de0). **Per `feedback_pb_zero_is_r3_close_target`**: long-run target is 0 exemptions; this Phase 4 cleanup delivers the hot-fix-tagged subset's contribution, not a frozen historical baseline.

**Cluster routing** (cross-Mgr coordination per #2722 §6 pattern):
- Clusters A (DB-8) + I (determinism): **PB Mgr (warm-dove-618)** — Lane 3 Stage 3c authority
- Cluster B (m1_5_testgen): **Substrate Mgr (warm-wolf-698)** — M1_5_DESIGN authority
- Clusters C/D/E/G (R3-V L4/L7/L5 + LBP + T-WAD + gate #87): **Verification Mgr (clever-tern-670)** — Verification lane (this brief's coordinator)
- Cluster F (T-LAS complexity contract): **T-LAS Mgr seat absence** — see §6 gap; Director routes operator-tier
- Cluster H (TC1 strict-fire): **Substrate Mgr** (R3 gate #11; substrate-adjacent) OR dedicated session

## §5. STOP and escalate

Escalate via dashboard-message to Verification Mgr (`clever-tern-670`) if:

- A cut test's behavior cannot be preserved under amortization (test semantics genuinely require fresh bootstrap state) — escalate as substrate-shape question, NOT a per-PR fix
- A cut's cold-CI wall exceeds 2s even under OnceLock/cached_compile/shared-fixture amortization — substrate gap; needs Director surface to decide whether to (a) accept exemption permanently, (b) restructure the test, or (c) deeper substrate fix
- A cluster's shared-fixture refactoring would change the test's behavioral assertion — escalate to lane Mgr for signoff before PR
- Cross-Mgr ownership unclear (esp. T-LAS Cluster F) — surface to Director for routing
- Ratchet floor reduction blocked by other exemption rows that aren't `hot-fix-2026-05-12`-tagged — those are non-hot-fix exemptions that may need separate paydown
- Re-enabled test surfaces a real regression (test fails post-rebuild) — that's a substantive correctness signal, NOT a rebuild discipline issue

## §6. Cross-coordinator notes

**T-LAS Mgr seat gap** (Cluster F): the `complexity_violation_compile_error_demonstrated` test (ROADMAP gate #92, ~14s wall) has no standing Mgr lane. Per PB Mgr's declination (msg_b280b6a6) and Substrate Mgr's existing cost-lens Miss dispatch load, dedicated session under Director auspices is the cleanest route. Director surfaces to operator via end-of-overnight digest (item #9) for ownership confirmation when rebuild dispatch triggers.

**Phase 3 #84 cluster overlap**: clusters C/D/E/G (Verification lane gate-evidence cuts) MAY converge with Cluster M #84 bulk-port work — those tests are explicit Cluster M Phase 3 reflected-Dag / DimensionReport rebuild candidates per the Phase 3 scaffolds I authored earlier (commit `64776ea12`). Verification Mgr coordinates whether rebuild lands as Layer 2 rebuild PR OR as Cluster M Phase 3 PR (the choice is a Mgr-tier sequencing question; Director defers to clever-tern-670).

**Layer 2 brief #2719 dependency**: this rebuild scaffold cites the Layer 2 brief at #2719 §2 for the harness/test-selection-machinery shared-infra class. Layer 2 itself is bridge-debt for R4 lens (per design-affected-set-lens.md §5); the rebuild work is INDEPENDENT — rebuild is structural (OnceLock amortization) regardless of CI gating layer.

---

**Mgr-finalization checklist** (before flipping to PRE-AUTH DISPATCH-READY per cluster):

- [ ] Confirm activation trigger fired (first post-#2723 cold-v3 wall-clock measurement; PM relays)
- [ ] Pilot cluster (Cluster A DB-8 emit matrix) — validate OnceLock pattern + ratchet-down protocol on 1-2 entries
- [ ] Per-cluster dispatch order set (default: A → H → parallel rollout C-G + B + I + F → ratchet sweep)
- [ ] Cluster F (T-LAS) ownership routed via operator-tier confirmation
- [ ] Cross-Mgr signoff requirement documented per #2722 §6 pattern

**End of scaffold.** Director-tier shape established; Verification Mgr canvases per cluster + dispatches per-cut workers in Phase 3 pattern.

---

## Authority footer

- **Brian operator greenlight**: gunbc#846 reply ~01:25Z (per PM msg_07f73de0 quote: "greenlight for rebuild session — operator-routable when ready")
- **Hot-fix lineage**: PR #2722 (cut sub-issue) → PR #2723 (cut PR; merged `d98d1e04b` 01:22:31Z)
- **Cut-PR Director conformance read**: https://github.com/gunb-ai/gunbc/pull/2723#issuecomment-4426470021
- **Phase 3-pattern reference**: `docs/briefs/r3-v-cluster-m-84-class-reflected-dag-bulkport-worker.md` + `docs/briefs/r3-v-cluster-m-84-class-generic-dimreport-bulkport-worker.md` (commit landed via PR #2708 squash 2026-05-11; Director scaffold-fill pattern)
- **Operator-tier-merge-bypass-precedent**: `feedback_operator_tier_merge_bypass_precedent` — applies per-PR if rebuild PRs encounter dashboard parser-lag with substantive review unanimous
