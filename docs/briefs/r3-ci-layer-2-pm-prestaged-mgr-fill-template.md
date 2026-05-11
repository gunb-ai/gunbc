# R3 CI Layer 2 — PM pre-staged Mgr-fill template

**Status**: PM pre-staged reference doc for Verification Mgr (clever-tern-670) finalization. Director-authored worker brief (`docs/briefs/r3-ci-layer-2-path-conditional-gating-worker.md`, pending) cites this file as the starting template.

**Authority**: PM authoring; Director-cited; Mgr-fill destination. Pre-staged 2026-05-11 EOD per Director greenlight at msg_4623068b.

**Purpose**: provide the Verification Mgr with a starting grouping (slow tests by file-area) plus a `(test_pattern, dimension, required_paths_regex)` skeleton aligned to the affected-set lens per-dimension output shape (`docs/design-affected-set-lens.md` §2). Mgr finalizes inventory + path-mapping; PM only provides the template + structural alignment.

**Hard constraint** (per `feedback_parallel_representation_debt`): every entry below carries a `dimension:` field matching the affected-set lens `Dimension` enum. The mechanism today is path-regex; the semantic carrier matches the future lens output so post-dissolution `skip_*` flags compute structurally as `affected_dimensions.contains(group.dimension)`.

**Dissolution trigger**: gate `ci_uses_provable_minimal_affected_set_selection` (R3 close-blocking; `docs/design-affected-set-lens.md` §5). When the lens lands, this template + the worker output are deleted.

---

## §1. Affected-set lens `Dimension` enum (reference)

Per `docs/design-affected-set-lens.md` §2:

```
enum Dimension {
    Value,        // value-output / structural-output (emit, parse, substrate shape)
    Cost,         // cost lens / symbolic cost / realization cost
    Complexity,   // complexity lens / analyze_complexity / asymptotic class
    Effect,       // effect lens / idempotency / effect-shape
    Refinement,   // refinement type checks / int width / cardinality
}
```

Every test-group entry below carries exactly one primary `dimension:` (multi-dim tests pick the closest match; if a test genuinely spans dimensions, it falls into the "run-always" conservative bucket).

---

## §2. Slow-test inventory grouped by file-area

Source: `scripts/slow-test-exemptions.txt` (78 active entries as of 2026-05-11 EOD at sha e9c8f9896). Grouped by module prefix; counts in parens.

### Cluster A — Emission / determinism (~10)

| Module | Tests | Notes |
|---|---|---|
| `db8_*` | 1 (`db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix`) | DB-8 / ROADMAP Lane 3 Stage 3c |
| `emit_matrix_*` | 6 (`emit_matrix_module_{go,python,rust}_is_deterministic` + `emit_matrix_program_*`) | 5× emit matrix sweep |
| `four_fixture_*` | 2 (`four_fixture_disk_sources_emit_deterministically` + `four_fixture_regression_test::four_fixture_suite_shares_one_reachability_shape`) | 4-fixture corpus |
| `m1_3_emit_rust_test::*` | 6 | M1.3 Rust roundtrip rustc harness |

### Cluster B — Lane 2 Stage 2d cost migration (~6)

| Module | Tests |
|---|---|
| `lane2_stage_2d_symbolic_cost_test::*` | 6 (`branch_reports_constant_when_both_arms_constant`, `cost_dag_compiles_cleanly`, `cost_generated_module_matches_checked_in_snapshot`, `recursive_fn_body_contributes_to_loop_cost`, `recursive_fn_reports_linear_via_loop_lowering`, `transform_single_op_reports_constant`, `value_reports_constant`) |

### Cluster C — M2 lens migration (~7)

| Module | Tests |
|---|---|
| `m2_lens_cost_migration_test::*` | 3 (`complexity_dag_compiles_cleanly`, `complexity_dag_runs_end_to_end_via_rustc_harness`, `complexity_generated_module_matches_checked_in_snapshot`) |
| `m2_lens_idempotency_migration_test::*` | 1 (`idempotency_emitted_analyze_matches_oracle`) |
| `m2_lens_provenance_migration_test::*` | 1 (`lens_provenance_dag_runs_end_to_end_via_rustc_harness`) |
| `m2_lens_unused_parameters_migration_test::*` | 2 (`unused_parameters_dag_runs_end_to_end_via_rustc_harness`, `unused_parameters_dag_self_analysis_reports_zero_findings`) |

### Cluster D — T-Lens-Behavioral-Parity / gate #73 / cost lens consumer (~5)

| Module | Tests |
|---|---|
| `complexity_lens_behavioral_completion::*` | 2 (`literal_bind_cements_constant_complexity_summary`, `recursive_countdown_cements_linear_work_and_span`) |
| `cost_lens_symbolic_consumer_test::*` | 3 (`literal_bind_pins_symbolic_cost_of_constant_on_fixture`, `recursive_countdown_pins_symbolic_cost_linear_and_sizevar_on_fixture`, `e_p_sub_value_relation_per_call_landed_cost_lens_routes_through_per_call_pattern_query`) |
| `lens_behavioral_parity_demonstration_test::*` | 2 (`r3_gate_73_demonstrates_complexity_certainty_is_proven`, `r3_gate_73_demonstrates_symbolic_cost_is_keyed_by_countdown_parameter`) |
| `lens_cost_target_realization_test::*` | 2 (`cost_lens_composes_symbolic_cost_with_rust_type_realization_row`, `cost_lens_demonstration_composes_representative_rust_program_cost`) |

### Cluster E — Dimension / E7 analyze_complexity / Stage 2f (~12)

| Module | Tests |
|---|---|
| `dimension::analyze_complexity_tests::*` | 5 (all 5 `analyze_complexity_*` tests) |
| `dimension::fail_closed_tests::*` | 2 (`missing_symbolic_cost_surfaces_*`, `transform_workflow_root_still_backward_reaches_operand_ports`) |
| `e7_analyze_complexity_integration::*` | 4 (4 `analyze_complexity_public_api_*` tests) |
| `lane2_stage_2f_dimension_test::*` | 1 (`analyze_symbolic_cost_composed_matches_lens_at_workflow_root`) |

### Cluster F — Substrate / R3 gate / cementing (~6)

| Module | Tests |
|---|---|
| `m0_acceptance::*` | 2 (`compile_boundary_is_fail_closed`, `post_sweep_port_state_matches_diagnostic_table`) |
| `m1_substrate_test::*` | 1 (`referenced_port_walk_real_helper_stack_compiles`) |
| `tc1_substrate_lens_eta_equivalence_strict_fire_test::*` | 1 (gate #11) |
| `t_pb_b_1_dag_runner_test::r3_gate_87_cementing_regen_lens_suites_pass_through_runner` | 1 (gate #87 / Cluster M Phase 2) |
| `bootstrap::tests::kernel_bool_path_a_attaches_diagnostic_when_boolean_algebra_unresolvable` | 1 (Lane 1e-2b) |

### Cluster G — T-LAS / CRDT / free consequences (~10)

| Module | Tests |
|---|---|
| `t_las_complexity_contract_compile_error_test::*` | 1 (gate #92 / issue #1952) |
| `t_las_crdt_cost_basis_demo_test::*` | 6 (all 6 `crdt_cost_basis_*` tests) |
| `r3_free_consequences_second_batch_test::*` | 3 (3 `cross_target_optimization_*` / `r3_free_consequences_second_batch_*`) |

### Cluster H — Parser / regen / R1C-E / R3-V (~7)

| Module | Tests |
|---|---|
| `sg2_parse_authority_test::parse_surface_dag_compiles_cleanly_for_regen_parse` | 1 |
| `sg2c1_parse_tables_authority_test::parse_tables_generated_module_matches_checked_in_snapshot` | 1 |
| `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` | 1 |
| `r1c_e_emit_gates_dag_test::*` | 1 (R1C-E `.dag` wrapper) |
| `r3_verification_l4_l7_l5_skeleton_test::*` | 2 (R3-V L4 W1 + R3-V L7 gate #10) |
| `t_ci_workflow_as_data_demo_test::*` | 1 (T-Workflow-As-Data) |

### Cluster I — Demo / thesis / cached_compile / refinement (~5)

| Module | Tests |
|---|---|
| `t_demo_fixture_test::t_demo_canonical_suites_are_runner_visible` | 1 |
| `thesis_validation_test::kf_1_structural_list_operation_ordering_holds` | 1 |
| `common::cached_compile::tests::cached_compile_any_returns_clean_dag_when_key_warmed_by_strict` | 1 |
| `int_literal_cardinality_test::int_refinement_overflow_is_proven_parametric_for_representable_widths` | 1 (gate #21) |
| `m1_5_verification_test::symbolic_cost_expr_equals_smoke_suite_passes` | 1 (M1.5) |

---

## §3. Path-mapping skeleton (Mgr-fill table)

Each row: `(test_pattern, dimension, required_paths_regex, confidence, dissolution_note)`.

PM partial-fills `dimension` where high confidence; Mgr finalizes `required_paths_regex`. Where PM marks `[Mgr-fill]`, the path-mapping requires deeper consumer-tracing than PM has bandwidth for (Mgr owns the lens canvas + Phase 3 brief, has the context).

**Conservative fail-closed default**: any group where Mgr is unsure of paths → mark `required_paths_regex: .*` (always-run). Dissolution is structural via the lens; bridge-debt period favors over-running over miss-running.

| Cluster | test_pattern | dimension | required_paths_regex | confidence | dissolution_note |
|---|---|---|---|---|---|
| A | `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` | Value | `^(dsl/extdeps/rust.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/tests/integration/db8_.*\.rs)$` | high | gate ci_uses_provable_minimal_affected_set_selection lands → lens.affected_dimensions.contains(Value) |
| A | `emit_matrix_(module|program)_(go|python|rust)_is_deterministic` | Value | `^(dsl/extdeps/.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/src/lens_.*\.rs|src/v3/compiler/tests/integration/emit_matrix.*\.rs)$` | high | same |
| A | `four_fixture_.*` | Value | `^(dsl/extdeps/.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/tests/integration/four_fixture.*\.rs|tests/.*/four_fixture.*\.dag)$` | medium | same |
| A | `m1_3_emit_rust_test::rustc_roundtrip_.*` | Value | `^(dsl/extdeps/rust.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/tests/integration/m1_3_emit_rust_test\.rs)$` | high | same |
| B | `lane2_stage_2d_symbolic_cost_test::.*` | Cost | `^(dsl/std/lens_cost.*\.dag|src/v3/compiler/src/lens_cost.*\.rs|dsl/std/cost.*\.dag|src/v3/compiler/tests/integration/lane2_stage_2d.*\.rs)$` | high | gate lands → lens.affected_dimensions.contains(Cost) |
| C | `m2_lens_cost_migration_test::complexity_.*` | Cost | `[Mgr-fill]` (likely: complexity.dag + lens_complexity*.rs + lens migration substrate) | medium | gate lands → Cost ∪ Complexity (note multi-dim) |
| C | `m2_lens_idempotency_migration_test::.*` | Effect | `^(dsl/std/lens_idempotency.*\.dag|src/v3/compiler/src/lens_idempotency.*\.rs|src/v3/compiler/tests/integration/m2_lens_idempotency.*\.rs)$` | high | same |
| C | `m2_lens_provenance_migration_test::.*` | Value | `^(dsl/std/lens_provenance.*\.dag|src/v3/compiler/src/lens_provenance.*\.rs|src/v3/compiler/tests/integration/m2_lens_provenance.*\.rs)$` | high | same |
| C | `m2_lens_unused_parameters_migration_test::.*` | Value | `^(dsl/std/lens_unused_parameters.*\.dag|src/v3/compiler/src/lens_unused_parameters.*\.rs|src/v3/compiler/tests/integration/m2_lens_unused_parameters.*\.rs)$` | high | same |
| D | `complexity_lens_behavioral_completion::.*` | Complexity | `[Mgr-fill]` (likely: lens_complexity + lens_cost + cementing substrate) | medium | gate lands → Complexity |
| D | `cost_lens_symbolic_consumer_test::.*` | Cost | `[Mgr-fill]` (likely: cost.dag + lens_cost*.rs + E-P substrate) | medium | gate lands → Cost |
| D | `lens_behavioral_parity_demonstration_test::r3_gate_73_.*` | Complexity | `[Mgr-fill]` (gate #73 LBP — needs cementing-test substrate dep) | medium | gate lands → Complexity ∪ Cost |
| D | `lens_cost_target_realization_test::.*` | Cost | `[Mgr-fill]` (cost-target-realization + rustc harness) | medium | gate lands → Cost ∪ Value |
| E | `dimension::analyze_complexity_tests::.*` | Complexity | `^(dsl/std/(complexity|cost|symbolic_cost).*\.dag|src/v3/compiler/src/(analyze_complexity|dimension).*\.rs|src/v3/compiler/tests/integration/dimension.*\.rs)$` | high | gate lands → Complexity |
| E | `dimension::fail_closed_tests::.*` | Complexity | `^(dsl/std/(complexity|cost).*\.dag|src/v3/compiler/src/(analyze_complexity|dimension).*\.rs)$` | medium | same |
| E | `e7_analyze_complexity_integration::.*` | Complexity | `^(dsl/std/(complexity|cost).*\.dag|src/v3/compiler/src/(analyze_complexity|dimension|public_api).*\.rs)$` | high | same |
| E | `lane2_stage_2f_dimension_test::.*` | Complexity | `^(dsl/std/(complexity|cost).*\.dag|src/v3/compiler/src/(lens_complexity|dimension).*\.rs)$` | high | same |
| F | `m0_acceptance::.*` | Value | `.*` (conservative — compile-boundary is broad) | low | run-always; gate lands → lens picks structural affected-set |
| F | `m1_substrate_test::.*` | Value | `^(dsl/std/.*\.dag|src/v3/compiler/src/(parse|substrate).*\.rs|src/v3/compiler/tests/integration/m1_substrate.*\.rs)$` | medium | gate lands → Value |
| F | `tc1_substrate_lens_eta_equivalence_strict_fire_test::.*` | Value | `[Mgr-fill]` (substrate eta-equivalence — needs substrate-lens dep map) | medium | gate #11 lands; consider TC1 closure path |
| F | `t_pb_b_1_dag_runner_test::r3_gate_87_.*` | Cost | `[Mgr-fill]` (cementing-test substrate + regen harness) | medium | gate #87 Cluster M Phase 2 lands → dissolves naturally |
| F | `bootstrap::tests::kernel_bool_path_a_.*` | Value | `^(src/v3/compiler/src/bootstrap.*\.rs|dsl/std/types\.dag|dsl/std/boolean_algebra.*\.dag)$` | high | dissolves with Lane 1e-2b Path A close |
| G | `t_las_complexity_contract_compile_error_test::.*` | Complexity | `^(dsl/std/complexity.*\.dag|src/v3/compiler/src/(las|t_las).*\.rs|src/v3/compiler/tests/integration/t_las.*\.rs)$` | medium | gate #92 lands |
| G | `t_las_crdt_cost_basis_demo_test::.*` | Cost | `^(dsl/std/(cost|las|crdt).*\.dag|src/v3/compiler/src/(las|t_las|crdt).*\.rs|src/v3/compiler/tests/integration/t_las.*\.rs)$` | medium | T-LAS CRDT closure |
| G | `r3_free_consequences_second_batch_test::.*` | Cost | `[Mgr-fill]` (free-consequences second batch — cross-target optimization + symbolic-cost) | medium | gates #43-#52 (free-consequences) land |
| H | `sg2_parse_authority_test::.*` | Value | `^(dsl/std/parse.*\.dag|src/v3/compiler/src/parse.*\.rs|src/v3/compiler/tests/integration/sg2_.*\.rs)$` | high | SG-2 parser-staging close |
| H | `sg2c1_parse_tables_authority_test::.*` | Value | `^(dsl/std/parse_tables.*\.dag|dsl/std/tokenize.*\.dag|src/v3/compiler/src/parse.*\.rs)$` | high | SG-2c-1 close |
| H | `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_.*` | Value | `^(scripts/regen.*|src/v3/compiler/src/(regen|cli).*\.rs|dsl/std/.*\.dag)$` | medium | SG-6 close |
| H | `r1c_e_emit_gates_dag_test::.*` | Value | `[Mgr-fill]` (R1C-E `.dag` wrapper) | low | R1C-E closure |
| H | `r3_verification_l4_l7_l5_skeleton_test::r3_verification_l4_.*` | Value | `[Mgr-fill]` (R3-V L4 direct consumer) | medium | gates #43+ L4-L7 lane |
| H | `r3_verification_l4_l7_l5_skeleton_test::r3_verification_l7_.*` | Value | `[Mgr-fill]` (gate #10 algebraic-law matrix) | medium | gate #10 close |
| H | `t_ci_workflow_as_data_demo_test::.*` | Value | `^(dsl/std/workflow.*\.dag|src/v3/compiler/src/(workflow|evaluator).*\.rs|src/v3/compiler/tests/integration/t_ci_workflow.*\.rs)$` | medium | T-Workflow-As-Data close |
| I | `t_demo_fixture_test::.*` | Value | `^(tests/.*/t_demo.*\.dag|src/v3/compiler/src/(test_runner|runner).*\.rs|src/v3/compiler/tests/integration/t_demo.*\.rs)$` | medium | T-Demo lane close |
| I | `thesis_validation_test::kf_1_.*` | Value | `.*` (conservative — thesis-level validation is broad) | low | run-always; gate lands → lens picks |
| I | `common::cached_compile::tests::.*` | Value | `^(src/v3/compiler/src/cached_compile.*\.rs|src/v3/compiler/tests/common/cached_compile.*\.rs)$` | high | TESTING.md paydown |
| I | `int_literal_cardinality_test::int_refinement_overflow_is_proven_.*` | Refinement | `^(dsl/std/int.*\.dag|src/v3/compiler/src/(refinement|int_literal).*\.rs|src/v3/compiler/tests/integration/int_literal.*\.rs)$` | high | gate #21 close |
| I | `m1_5_verification_test::symbolic_cost_expr_equals_smoke_.*` | Value | `[Mgr-fill]` (M1.5 verification smoke — needs symbolic-cost equality harness dep map) | medium | M1.5 close |

---

## §4. Open questions for Mgr-fill

1. **Multi-dimension tests**: rows marked Cost-or-Complexity (e.g., D-cluster LBP, G-cluster free-consequences) — does Mgr split into separate rows per dimension, or accept "primary dimension + conservative regex covers both"? Recommendation: primary dimension is sufficient because the path-regex naturally pulls in both axes' source files.

2. **Conservative-run-always candidates**: `m0_acceptance::*` and `thesis_validation_test::*` are marked `.*` (always-run). Mgr decides if these justify the cost (~3-5s each cold) or warrant a tighter regex. PM read: keep `.*` until the lens lands; the safety margin is cheap.

3. **`[Mgr-fill]` rows count**: 12 rows out of ~36 need Mgr to derive `required_paths_regex` from consumer tracing. PM left these blank where derivation requires deeper substrate knowledge (substrate-lens deps, R3-V L4/L7 direct-consumer maps, R1C-E `.dag` wrapper internals, free-consequences cross-target topology).

4. **Mechanism — single regex per row vs `paths-ignore`-style**: this template uses a single positive regex per row (required_paths_regex). GitHub Actions doesn't natively support per-step path filtering, so the CI consumer is a shell snippet in `ci.yml`:
   ```yaml
   - name: <Cluster X test step>
     if: needs.changes.outputs.skip_<cluster> != 'true'
     run: cargo test -p v3-compiler --test integration <module_filter>
   ```
   The `changes` job computes each `skip_<cluster>` boolean by checking if any changed file matches the cluster's `required_paths_regex`.

5. **Pilot cluster selection**: which cluster does Mgr prototype first? PM recommendation: **Cluster B (Lane 2 Stage 2d symbolic cost)** — high confidence in path-regex, contained module, ~6 tests, single dimension (Cost). Lowest risk, highest learning per LOC.

---

## §5. Acceptance (Mgr fills, Director ratifies)

Mgr-fill complete when:

- [ ] Every `[Mgr-fill]` placeholder in §3 is replaced with a concrete `required_paths_regex` OR explicitly marked `.*` (always-run) with a one-line "why conservative".
- [ ] Each cluster has a pilot-PR worker brief authored (or batched into the Director-authored Layer 2 brief).
- [ ] `changes` job in `ci.yml` extended with per-cluster `skip_<cluster>` boolean outputs.
- [ ] At least one cluster (pilot) has its `if: needs.changes.outputs.skip_<cluster> != 'true'` gate landed and CI-validated against a representative test case (docs-only PR skips; in-cluster code change runs).
- [ ] All path-mapping entries have `dimension:` field aligned to the affected-set lens `Dimension` enum.

---

## §6. STOP triggers (Mgr aborts and surfaces to Director)

- Any cluster needs a *new* substrate carrier to express path-dependency → STOP. Surface to Director: this is the lens substrate, not a bridge.
- Any path-mapping entry has `dimension:` outside `{Value, Cost, Complexity, Effect, Refinement}` → STOP. Surface to Director: this is a coproduct-dissolution candidate.
- Any cluster's `required_paths_regex` requires depending on test-output (not just changed files) → STOP. That's the lens, not the bridge.

---

## §7. Cross-references

- **Layer 1 (predecessor)**: PR #2718 — docs-only-skip `v3` via `changes` job + `code` boolean output. Layer 2 extends the same job with per-group flags.
- **Affected-set lens (dissolution)**: `docs/design-affected-set-lens.md` (locked design), PR #2713 (Verification Mgr canvas rebuild).
- **Routing decision**: msg_a77c7f42 (Director ratification of Verification Mgr ownership) — gunbc#828 c4425726922 carries PM's 3-ask scaffold data.
- **Director-authored worker brief**: `docs/briefs/r3-ci-layer-2-path-conditional-gating-worker.md` (pending; ETA per Director ~23:00-23:30Z 2026-05-11).
- **Memory**: `feedback_parallel_representation_debt` (single-Mgr ownership rationale); `feedback_intro_rate_vs_residual_share` (intro-class composition framing).

---

**End of pre-staged Mgr-fill template.** PM signing off; Mgr inherits via Director-cited brief.
