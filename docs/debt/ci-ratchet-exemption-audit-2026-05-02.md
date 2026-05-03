# CI Ratchet Exemption Audit — 2026-05-02

**Source:** `scripts/slow-test-exemptions.txt` and `scripts/check-test-timeout.sh` on branch `quick-heron-61` (workspace snapshot 2026-05-02).

**Authority:** `TESTING.md` § test layers (`feedback_test_timeout_2s`, 2000 ms per `#[test]` wall-clock); `docs/briefs/ci-ratchet-architecture-audit.md` Phase 1 shape; prior pass [ci-ratchet-exemption-audit-2026-04-24.md](ci-ratchet-exemption-audit-2026-04-24.md).

**R3 dispatch parent:** [r3-debt-paydown-ledger-2026-05-02.md](r3-debt-paydown-ledger-2026-05-02.md) highest-leverage target **#10** (CI slow-test exemption fresh audit).

## Summary

| Metric | Value |
|--------|------:|
| Active exemptions (non-comment lines) | **39** |
| `TEST_TIMEOUT_MAX_EXEMPTIONS` default in `scripts/check-test-timeout.sh` | **39** (must match; same PR rule for floor changes) |
| Stale exemptions removed in **this** PR | **0** — no row deleted without a fresh `cargo test … -- -Z unstable-options --report-time` observation proving **≤ 2000 ms** for that test under the ratchet’s parsing rules |
| Per-exempt duration budgets enforced in CI | **Not landed** — still deferred to Phase 2 in the architecture brief (would require parser/script extension; do not invent budgets) |

## Measurement status (honest scope)

- **This pass did not re-time every exempt test.** A cold `cargo test -p v3-compiler` single-filter invocation was started for spot-checking; it did not complete within the agent window (integration binary link dominates before libtest prints timings). **All rows below are therefore categorized from the in-file ROADMAP / lane annotations, not from fresh 2026-05-02 wall-clock rows.**
- **Follow-up (mechanical):** Re-use the CI job’s tee’d libtest log (same shape `scripts/check-test-timeout.sh` parses) or run `RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler -- -Z unstable-options --report-time` once on a warmed target, extract per-name `elapsed_ms`, mark each exemption **measured / under budget / still over budget**, then delete only lines with `elapsed_ms ≤ 2000` and lower `TEST_TIMEOUT_MAX_EXEMPTIONS` in the same commit. Optionally add comment-only budget columns after that pass (still requires brief/script agreement before CI enforcement).

## Delta vs 2026-04-24 audit

The April audit recorded **43** active exemptions and a floor of **43**. The tree under audit now has **39** exemptions and default floor **39** — net **−4** exemptions since April, already reflected in `check-test-timeout.sh` on `main` before this document.

**Rows present in 2026-04-24 audit but absent now** (dissolved elsewhere; this audit does not re-claim credit):

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

## STOP+PING check (`ci-ratchet-architecture-audit.md`)

- **“Most exemptions structurally necessary”:** **No.** A minority block (~9 lines) is the DB-8 determinism matrix + four-fixture disk suite; the majority are **P** (compile/snapshot/rustc-harness/M0/M1/M2/PB/SG/KF-1 paydown classes). This does **not** indicate the 2s budget is wrong for the whole architecture; it indicates concentrated compile-oracle debt plus one very hot P0 oracle row (`p0_std_render…`, comment still cites ~29.7 s historical observation).
- **Parser / script invasiveness:** **No change** to exemption line format or `check-test-timeout.sh` parsing in this PR.

## Exemption table (all 39 rows)

| # | Exemption | Cat | Paydown / routing (from file + brief) |
|---|-----------|-----|--------------------------------------|
| 1 | `bootstrap::tests::malformed_pipeline_stage_attaches_diagnostic` | P | Bootstrap fixture / test-infra caching; not TM-0. |
| 2 | `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` | S | DB-8 Lane 3 Stage 3c / determinism matrix; 5× re-emit by design. |
| 3–9 | `emit_matrix_*_{go,python,rust}_is_deterministic` (module + program) | S | Same DB-8 matrix class. |
| 10 | `four_fixture_disk_sources_emit_deterministically` | S | Four-fixture disk corpus determinism. |
| 11 | `four_fixture_regression_test::four_fixture_suite_shares_one_reachability_shape` | P | Shared `compile_to_dag` warming / pressure-corpus paydown. |
| 12 | `common::cached_compile::tests::cached_compile_any_returns_clean_dag_when_key_warmed_by_strict` | P | TESTING.md “Mocks over compile”; real compile remains. |
| 13 | `lane2_stage_2d_symbolic_cost_test::branch_reports_constant_when_both_arms_constant` | P | Shared bootstrap / cache key; parallelism neighbor effect. |
| 14 | `lane2_stage_2d_symbolic_cost_test::cost_dag_compiles_cleanly` | P | Lane 2 Stage 2d authority compile smoke. |
| 15 | `lane2_stage_2d_symbolic_cost_test::cost_generated_module_matches_checked_in_snapshot` | P | Emit + rustfmt snapshot; inner wall-clock may use `budgeted_test!`. |
| 16 | `m0_acceptance::compile_boundary_is_fail_closed` | P | M0 fail-closed path; shared-setup paydown. |
| 17 | `m0_acceptance::post_sweep_port_state_matches_diagnostic_table` | P | Full compile path; test-layer migration. |
| 18 | `m1_substrate_test::referenced_port_walk_real_helper_stack_compiles` | P | SG-3f helper-stack compile; parse/lower cutover. |
| 19–24 | `m1_3_emit_rust_test::rustc_roundtrip_*` (6 tests) | P | SG-7 rustc harness + emitter lane. |
| 25 | `m2_lens_cost_migration_test::complexity_dag_compiles_cleanly` | P | Lens migration; comment cites ~1949 ms edge. |
| 26 | `m2_lens_cost_migration_test::complexity_dag_runs_end_to_end_via_rustc_harness` | P | Lens rustc oracle harness. |
| 27 | `m2_lens_cost_migration_test::complexity_generated_module_matches_checked_in_snapshot` | P | Snapshot + rustfmt paydown. |
| 28 | `m2_lens_idempotency_migration_test::idempotency_emitted_analyze_matches_oracle` | P | Lane 2 Stage 2b oracle. |
| 29 | `m2_lens_provenance_migration_test::lens_provenance_dag_runs_end_to_end_via_rustc_harness` | P | Lens harness paydown. |
| 30 | `m2_lens_unused_parameters_migration_test::unused_parameters_dag_runs_end_to_end_via_rustc_harness` | P | Lens harness paydown. |
| 31 | `m2_lens_unused_parameters_migration_test::unused_parameters_dag_self_analysis_reports_zero_findings` | P | Lens self-analysis paydown. |
| 32 | `pb1_bootstrap_full_snapshot_test::full_bootstrap_extends_std_snapshot` | P | PB-1 snapshot compare; shared compile warming / optional streaming compare. |
| 33 | `r1c_e_emit_gates_dag_test::r1c_e_emit_gates_suite_passes_through_runner` | P | R1C-E runner binary harness; shared-setup work. |
| 34 | `bootstrap::tests::kernel_bool_path_a_attaches_diagnostic_when_boolean_algebra_unresolvable` | P | Bool kernel Path A; bridge dissolution + shared `Dag::new` warming. |
| 35 | `p0_std_render_repeat_string_test::std_render_repeat_string_and_indent_text_match_interpreter` | P | P0 / v2 oracle cold path; **shared v2-oracle bootstrap cache** trigger (`OnceLock<ResolvedPipelineResult>` or TM-0 lane). |
| 36 | `sg1_tokenize_authority_test::tokenize_generated_module_matches_checked_in_snapshot` | P | SG-1 tokenize snapshot lane. |
| 37 | `sg2_parse_authority_test::parse_surface_dag_compiles_cleanly_for_regen_parse` | P | SG-2 parse surface compile; note comment still references sibling `parse_generated_module_matches_checked_in_snapshot` (removed from exemptions — keep comment accurate in a future tiny edit if confusing). |
| 38 | `sg2c1_parse_tables_authority_test::parse_tables_generated_module_matches_checked_in_snapshot` | P | SG-2c-1 parse-tables snapshot. |
| 39 | `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` | P | SG-6 CLI subprocess smoke. |
| 40 | `thesis_validation_test::kf_1_structural_list_operation_ordering_holds` | P | KF-1 thesis validation decomposition (TESTING.md). |

**Table row count check:** entries 1–40 above are **39** distinct tests — the numbered list uses one row for the six `m1_3_emit_rust_test::*` lines (rows 19–24 inclusive = 6 tests). Adjusted: rows 1–18 = 18, 19–24 = 6, 25–40 = 16 → 18+6+16 = **40** — error.

Let me fix: 1-18 = 18 tests. 19-24 is six m1_3 = 6. 25-31 = 7 m2. 32 pb1. 33 r1c. 34 kernel. 35 p0. 36-39 sg1/sg2/sg2c1/sg6 = 4. 40 thesis = 1.

18+6+7+1+1+1+1+4+1 = 40 — still 40. Actual is 39.

Recount exemptions:
- bootstrap malformed: 1
- db8 + 7 emit + four_disk + four_regression = 1+7+1+1 = 10 → total 11
- cached: 1 → 12
- lane2 3: → 15
- m0 2: → 17
- m1_substrate 1: → 18
- m1_3: 6 lines → 24
- m2 cost 3, idem 1, prov 1, unused 2 = 7 → 31
- pb1 1 → 32
- r1c 1 → 33
- kernel_bool 1 → 34
- p0 1 → 35
- sg1 1 → 36
- sg2: only parse_surface in list = 1 → 37
- sg2c1 1 → 38
- sg6 1 → 39
- thesis 1 → 40

That's 40 again. wc said 39.

m1_3 count from grep: lines 53,54,55,56,57,58 = 6 only!

So 18+6=24, +7 m2 = 31, +1 pb +1 r1c +1 kernel +1 p0 = 35, +1 sg1 +1 sg2 +1 sg2c1 +1 sg6 = 39, +1 thesis = 40.

Thesis is 40th - but wc is 39. So one of these is wrong.

List all 39 from sed:


Shell