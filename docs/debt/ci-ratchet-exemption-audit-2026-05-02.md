# CI Ratchet Exemption Audit — 2026-05-02

**Source:** `scripts/slow-test-exemptions.txt` and `scripts/check-test-timeout.sh` (workspace snapshot **2026-05-02**). For provenance after merge to `main`, use the commit that last touched this file (`git log -1 --format=%H -- docs/debt/ci-ratchet-exemption-audit-2026-05-02.md`); PR branch names are dispatch metadata only.

**Authority:** `TESTING.md` § test layers (`feedback_test_timeout_2s`, 2000 ms per `#[test]` wall-clock); `docs/briefs/ci-ratchet-architecture-audit.md` Phase 1 shape; prior pass [ci-ratchet-exemption-audit-2026-04-24.md](ci-ratchet-exemption-audit-2026-04-24.md).

**R3 dispatch parent:** [r3-debt-paydown-ledger-2026-05-02.md](r3-debt-paydown-ledger-2026-05-02.md) highest-leverage target **#10** (CI slow-test exemption fresh audit).

## Summary

| Metric | Value |
|--------|------:|
| Active exemptions (non-comment lines) | **39** |
| `TEST_TIMEOUT_MAX_EXEMPTIONS` default in `scripts/check-test-timeout.sh` | **39** (must match; same PR rule for floor changes) |
| Stale exemptions removed in **this** PR | **0** — no row deleted without a fresh `cargo test … -- -Z unstable-options --report-time` observation proving **≤ 2000 ms** for that test under the ratchet’s parsing rules |
| Per-exempt duration budgets enforced in CI | **Not landed** — still deferred to Phase 2 in the architecture brief (would require parser/script extension; do not invent budgets) |

**Scope (read this first):** This PR is a **bounded routing / scope audit** over the current exemption list and `check-test-timeout.sh` floor — categories, owners, STOP+PING, and explicit follow-ups. It is **not** a completed **Phase 1** pass from `docs/briefs/ci-ratchet-architecture-audit.md` (that phase expects per-test wall-clock re-measurement and stale-row deletion where proven `≤ 2000 ms`). No stale exemptions were removed here because no row was re-proven under the 2s authority in this pass.

## Measurement status (honest scope)

- **This pass did not re-time every exempt test.** A cold `cargo test -p v3-compiler` single-filter invocation was started for spot-checking; it did not complete within the agent window (integration binary link dominates before libtest prints timings). **All rows below are therefore categorized from the in-file ROADMAP / lane annotations, not from fresh 2026-05-02 wall-clock rows.**
- **Follow-up (mechanical):** Re-use the CI job’s tee’d libtest log (same shape `scripts/check-test-timeout.sh` parses) or run `RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler -- -Z unstable-options --report-time` once on a warmed target, extract per-name `elapsed_ms`, mark each exemption **measured / under budget / still over budget**, then delete only lines with `elapsed_ms ≤ 2000` and lower `TEST_TIMEOUT_MAX_EXEMPTIONS` in the same commit. Optionally add comment-only budget columns after that pass (still requires brief/script agreement before CI enforcement).

## Delta vs 2026-04-24 audit

The April audit recorded **43** active exemptions and a floor of **43**. The tree under audit now has **39** exemptions and default floor **39**. **Arithmetic is intentional:** five tokens listed below were **removed** from the exemption list on `main` between 2026-04-24 and this snapshot, and **one** new token was **added** (`r1c_e_emit_gates_dag_test::…`). Net change: **5 − 1 = −4** rows (43 → 39); that net is already reflected in `check-test-timeout.sh` before this document landed.

**Rows present in 2026-04-24 audit but absent now** (retired from the exemption list elsewhere; this audit does not re-claim credit):

| Exemption token (removed since April audit) |
|---|
| `pb1_bootstrap_full_snapshot_test::generated_full_bootstrap_snapshot_matches_runtime_full_bootstrap` |
| `pb1_bootstrap_full_snapshot_test::generated_full_bootstrap_without_parse_surface_matches_runtime_variant` |
| `pb1_bootstrap_std_snapshot_test::generated_std_bootstrap_snapshot_matches_runtime_std_bootstrap` |
| `sg2_parse_authority_test::parse_generated_module_matches_checked_in_snapshot` |
| `sg3_lower_authority_test::lower_generated_module_matches_regen_snapshot` |

**New since April audit**

| Exemption | Note |
|-----------|------|
| `r1c_e_emit_gates_dag_test::r1c_e_emit_gates_suite_passes_through_runner` | Runner/harness edge; comment cites ~2166 ms on PR #1233; paydown: R1C-E shared setup / runner lane. |

## Category rubric

| Code | Meaning |
|------|---------|
| **P** | Paydown backlog — slow for known test-shape reasons; shrink via shared setup, cache warming, harness decomposition, or lane work named in the exemption comment. |
| **S** | Structural for now — intentional multi-emit / matrix / corpus work tied to DB-8 determinism lane until a cheaper equivalent exists. |
| **U** | Unknown / unmeasured this pass — owner path is documented in-file; wall-clock **not** re-verified 2026-05-02. |

## Category totals (from annotations; all **U** for wall-clock)

| Category | Count | Notes |
|----------|------:|-------|
| **P** | 31 | Includes M0/M1/M2/PB/R1C-E/P0/SG/KF-1/bootstrap paydown classes. |
| **S** | 8 | `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` + six `emit_matrix_*_is_deterministic` + `four_fixture_disk_sources_emit_deterministically`. |

## STOP+PING check (`ci-ratchet-architecture-audit.md`)

- **“Most exemptions structurally necessary”:** **No.** Eight lines are the DB-8 determinism matrix + four-fixture **disk** determinism row; the remainder are **P** (compile/snapshot/rustc-harness/M0/M1/M2/PB/SG/KF-1 paydown classes). This does **not** indicate the 2s budget is wrong for the whole architecture; it indicates concentrated compile-oracle debt plus one very hot P0 oracle row (`p0_std_render…`, comment still cites ~29.7 s historical observation).
- **Parser / script invasiveness:** **No change** to exemption line format or `check-test-timeout.sh` parsing in this PR.

## Exemption table (39 tests; compact ranges)

| # | Exemption(s) | Cat | Paydown / routing |
|---|----------------|-----|-------------------|
| 1 | `bootstrap::tests::malformed_pipeline_stage_attaches_diagnostic` | P | Bootstrap fixture / test-infra caching; not TM-0. |
| 2 | `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` | S | DB-8 Lane 3 Stage 3c / determinism matrix; 5× re-emit by design. |
| 3–8 | Six `emit_matrix_{module,program}_{go,python,rust}_is_deterministic` | S | DB-8 module/program re-emit matrix. |
| 9 | `four_fixture_disk_sources_emit_deterministically` | S | Four-fixture disk corpus determinism. |
| 10 | `four_fixture_regression_test::four_fixture_suite_shares_one_reachability_shape` | P | Shared `compile_to_dag` warming / pressure-corpus paydown. |
| 11 | `common::cached_compile::tests::cached_compile_any_returns_clean_dag_when_key_warmed_by_strict` | P | TESTING.md “Mocks over compile”; real compile remains. |
| 12–14 | `lane2_stage_2d_symbolic_cost_test::{branch_reports_constant_when_both_arms_constant,cost_dag_compiles_cleanly,cost_generated_module_matches_checked_in_snapshot}` | P | Lane 2 Stage 2d; shared bootstrap / snapshot + rustfmt paydown. |
| 15–16 | `m0_acceptance::{compile_boundary_is_fail_closed,post_sweep_port_state_matches_diagnostic_table}` | P | M0 acceptance compile-boundary shared setup. |
| 17 | `m1_substrate_test::referenced_port_walk_real_helper_stack_compiles` | P | SG-3f helper-stack compile; parse/lower cutover. |
| 18–23 | Six `m1_3_emit_rust_test::rustc_roundtrip_*` / `rustc_roundtrip_generic_list_fold_prints_one` | P | SG-7 rustc harness + emitter lane. |
| 24–26 | `m2_lens_cost_migration_test::{complexity_dag_compiles_cleanly,complexity_dag_runs_end_to_end_via_rustc_harness,complexity_generated_module_matches_checked_in_snapshot}` | P | Lens migration harness paydown. |
| 27 | `m2_lens_idempotency_migration_test::idempotency_emitted_analyze_matches_oracle` | P | Lane 2 Stage 2b oracle. |
| 28 | `m2_lens_provenance_migration_test::lens_provenance_dag_runs_end_to_end_via_rustc_harness` | P | Lens harness paydown. |
| 29–30 | `m2_lens_unused_parameters_migration_test::{unused_parameters_dag_runs_end_to_end_via_rustc_harness,unused_parameters_dag_self_analysis_reports_zero_findings}` | P | Lens harness paydown. |
| 31 | `pb1_bootstrap_full_snapshot_test::full_bootstrap_extends_std_snapshot` | P | PB-1 snapshot compare; shared compile warming. |
| 32 | `r1c_e_emit_gates_dag_test::r1c_e_emit_gates_suite_passes_through_runner` | P | R1C-E runner harness; shared-setup work. |
| 33 | `bootstrap::tests::kernel_bool_path_a_attaches_diagnostic_when_boolean_algebra_unresolvable` | P | Bool kernel Path A; bridge dissolution + shared `Dag::new` warming. |
| 34 | `p0_std_render_repeat_string_test::std_render_repeat_string_and_indent_text_match_interpreter` | P | P0 / v2 oracle; shared v2-oracle bootstrap cache (`OnceLock<ResolvedPipelineResult>` or TM-0 lane). |
| 35 | `sg1_tokenize_authority_test::tokenize_generated_module_matches_checked_in_snapshot` | P | SG-1 tokenize snapshot lane. |
| 36 | `sg2_parse_authority_test::parse_surface_dag_compiles_cleanly_for_regen_parse` | P | SG-2 parse surface compile; exemption comment still names removed sibling test — tidy comment in a follow-up if confusing. |
| 37 | `sg2c1_parse_tables_authority_test::parse_tables_generated_module_matches_checked_in_snapshot` | P | SG-2c-1 parse-tables snapshot. |
| 38 | `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` | P | SG-6 CLI subprocess smoke. |
| 39 | `thesis_validation_test::kf_1_structural_list_operation_ordering_holds` | P | KF-1 thesis validation decomposition (TESTING.md). |

The compact table’s ranges expand to **39** test tokens — the same count as non-comment lines in `scripts/slow-test-exemptions.txt` and the `TEST_TIMEOUT_MAX_EXEMPTIONS` default.

## R3 per-PR debt receipt (this PR)

- **Disposition:** **Debt found + routed** (bounded **scope / routing** audit — not a completed Phase 1 per-test timing audit).
- **Debt addressed (partial):** ROADMAP bullet *CI ratchet architecture — PARTIAL* — fresh audit artifact documents the current **39**-exemption floor and routes remaining work (measured timing pass, optional per-exempt budgets).
- **Debt found (routed):** No exemptions proven stale under the 2s authority in this pass; mechanical re-timing from CI or warmed local `cargo test` logs remains the next executable slice.
- **Dissolved debt rows:** None — no line removed from `scripts/slow-test-exemptions.txt` without a ≤2000 ms proof.
- **Documentation:** Stale ROADMAP counts **43**/**43** → **39**/**39** aligned with `check-test-timeout.sh` (same PR as this audit).
