# R3 CI Layer 2 — PM pre-staged Mgr-fill template

**Status**: PM pre-staged reference doc for Verification Mgr (clever-tern-670) finalization. Director-authored worker brief (`docs/briefs/r3-ci-layer-2-path-conditional-gating-worker.md`, pending) cites this file as the starting template.

**Authority**: PM authoring; Director-cited; Mgr-fill destination. Pre-staged 2026-05-11 EOD per Director greenlight at msg_4623068b.

**Purpose**: provide the Verification Mgr with a starting grouping (slow tests by file-area) plus a `(test_pattern, dimensions, required_paths_regex)` skeleton aligned to the affected-set lens per-dimension output shape (`docs/design-affected-set-lens.md` §2). Mgr finalizes inventory + path-mapping; PM only provides the template + structural alignment.

**Hard constraint** (per `feedback_parallel_representation_debt` + locked design): every entry below carries a `dimensions:` **field** (Set<Dimension>) matching the affected-set lens `Dimension` enum. **Multi-dimension carriage is REQUIRED, not optional**: a consumer that reads both Cost AND Complexity must carry `dimensions: [Cost, Complexity]` — narrowing to a single "primary" dimension is a semantic-contract violation against `docs/design-affected-set-lens.md` §2 (full affected-set = union across every dimension the consumer reads).

**Boolean polarity (load-bearing)**: `skip_<cluster> = true` means "no relevant change for this cluster — safe to skip its tests." `skip_<cluster> = false` means "relevant change detected — must run." The CI consumer wires `if: needs.changes.outputs.skip_<cluster> != 'true'` (run when skip is NOT true). Post-dissolution mapping under the affected-set lens:

```
skip_<cluster> = (affected_dimensions ∩ row.dimensions) = ∅
              ⇔ no affected dim that this cluster reads
              ⇔ safe to skip
```

Equivalently: `run_<cluster> = (affected_dimensions ∩ row.dimensions) ≠ ∅`. The boolean polarity must match the CI gate semantics; setting `skip=true` on non-empty intersection would silently skip affected tests (TESTING.md violation, openai-pro REQUEST_CHANGES caught an earlier inversion).

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

Every test-group entry below carries a `dimensions:` field that is a **Set<Dimension>** — the full set of dimensions the test/consumer reads, NOT a single "primary." Per `docs/design-affected-set-lens.md` §2:

```
affected_set(Dag_before, Dag_after) =
  ⋃ over dim in {Value, Cost, Complexity, Effect, Refinement}
    affected_set(Dag_before, Dag_after, dim)
```

A test/consumer is "affected" (must run) when **any** dimension it reads has a proven delta — set intersection, not single-value match. Narrowing multi-dim consumers to a single primary dimension would silently skip tests when only the non-primary dimension changes (semantic-contract violation; codex REQUEST_CHANGES on PR #2721 caught this in an earlier revision of this template).

---

## §2. Slow-test inventory grouped by file-area

Source: `scripts/slow-test-exemptions.txt` (78 active entries as of 2026-05-11 EOD at sha e9c8f9896). Grouped by module prefix; counts in parens.

### Cluster A — Emission / determinism (15)

| Module | Tests | Notes |
|---|---|---|
| `db8_*` | 1 (`db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix`) | DB-8 / ROADMAP Lane 3 Stage 3c |
| `emit_matrix_*` | 6 (`emit_matrix_module_{go,python,rust}_is_deterministic` + `emit_matrix_program_*`) | 6× emit matrix sweep (3 module + 3 program) |
| `four_fixture_*` | 2 (`four_fixture_disk_sources_emit_deterministically` + `four_fixture_regression_test::four_fixture_suite_shares_one_reachability_shape`) | 4-fixture corpus |
| `m1_3_emit_rust_test::*` | 6 | M1.3 Rust roundtrip rustc harness |

### Cluster B — Lane 2 Stage 2d cost migration (7)

| Module | Tests |
|---|---|
| `lane2_stage_2d_symbolic_cost_test::*` | 7 (`branch_reports_constant_when_both_arms_constant`, `cost_dag_compiles_cleanly`, `cost_generated_module_matches_checked_in_snapshot`, `recursive_fn_body_contributes_to_loop_cost`, `recursive_fn_reports_linear_via_loop_lowering`, `transform_single_op_reports_constant`, `value_reports_constant`) |

### Cluster C — M2 lens migration (~7)

| Module | Tests |
|---|---|
| `m2_lens_cost_migration_test::*` | 3 (`complexity_dag_compiles_cleanly`, `complexity_dag_runs_end_to_end_via_rustc_harness`, `complexity_generated_module_matches_checked_in_snapshot`) |
| `m2_lens_idempotency_migration_test::*` | 1 (`idempotency_emitted_analyze_matches_oracle`) |
| `m2_lens_provenance_migration_test::*` | 1 (`lens_provenance_dag_runs_end_to_end_via_rustc_harness`) |
| `m2_lens_unused_parameters_migration_test::*` | 2 (`unused_parameters_dag_runs_end_to_end_via_rustc_harness`, `unused_parameters_dag_self_analysis_reports_zero_findings`) |

### Cluster D — T-Lens-Behavioral-Parity / gate #73 / cost lens consumer (9)

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

Each row: `(test_pattern, dimensions, required_paths_regex, confidence, dissolution_note)`.

`dimensions` is a **Set<Dimension>** — the full set of dimensions the test/consumer reads. Per the lens design §2, post-dissolution skip computation is `skip_<cluster> = (affected_dimensions ∩ group.dimensions) = ∅` (skip when intersection IS empty / no affected dim that this cluster reads). Equivalently: `run = (intersection ≠ ∅)`. Where PM marks dimensions for a row, the set is the **complete read-set as best derived from the test name + comments**, not a primary; Mgr should expand the set if consumer tracing reveals more dimensions read.

**Conservative fail-closed default**: any group where Mgr is unsure of paths → mark `required_paths_regex: .*` (always-run). Similarly, when in doubt about dimensions, **add more dimensions to the set, never narrow** — over-running is cheap during the bridge-debt period; miss-running violates the locked-design contract.

**Cluster aggregation predicate** (load-bearing per codex BLOCKING on earlier revision): a cluster has multiple rows (e.g., Cluster A has 4 modules). The cluster-level `skip_<cluster>` boolean is computed by **conjunction** over rows — skip the cluster ONLY when ALL rows are unaffected:

```
skip_<cluster> = ∀ row ∈ cluster :
  (changed_files ∩ row.required_paths_regex matches) = ∅
```

Equivalently: `run_<cluster> = ∃ row ∈ cluster : row affected`. Aggregating with `any-row-empty` (i.e., disjunction) would skip the cluster when only ONE row is affected — silently skipping the OTHER affected rows. Aggregation must be `all-rows-empty` (conjunction).

**Path-mapping verification discipline** (load-bearing per codex BLOCKING on earlier revision): the `required_paths_regex` values below are **PM best-effort manual authoring** — they have NOT been verified against the live source tree at the cited inventory sha (`e9c8f9896`). Workers MUST validate each regex against the source tree at HEAD before implementing the CI gate. Any regex that does not match a path actually present in the tree, OR that a Mgr cannot quickly verify, MUST be replaced with `.*` (always-run) — the conservative fail-closed default. The `confidence` column indicates PM's subjective certainty per row; treat `low` rows as `.*` candidates first, `medium` as audit-then-decide, `high` as audit-but-likely-fine.

| Cluster | test_pattern | dimensions | required_paths_regex | confidence | dissolution_note |
|---|---|---|---|---|---|
| A | `db8_rust_emit_avoids_time_paths_and_float_hooks_on_program_matrix` | `[Value]` | `^(dsl/extdeps/rust.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/tests/integration/db8_.*\.rs)$` | high | gate ci_uses_provable_minimal_affected_set_selection lands → lens emits per-Node delta; consumer reads only Value |
| A | `emit_matrix_(module|program)_(go|python|rust)_is_deterministic` | `[Value]` | `^(dsl/extdeps/.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/src/lens_.*\.rs|src/v3/compiler/tests/integration/emit_matrix.*\.rs)$` | high | same |
| A | `four_fixture_.*` | `[Value]` | `^(dsl/extdeps/.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/tests/integration/four_fixture.*\.rs|tests/.*/four_fixture.*\.dag)$` | medium | same |
| A | `m1_3_emit_rust_test::rustc_roundtrip_.*` | `[Value]` | `^(dsl/extdeps/rust.*\.dag|src/v3/compiler/src/emit.*\.rs|src/v3/compiler/tests/integration/m1_3_emit_rust_test\.rs)$` | high | same |
| B | `lane2_stage_2d_symbolic_cost_test::.*` | `[Cost]` | `^(src/v3/lenses/cost(_target_realization)?\.dag|src/v3/compiler/src/lens_cost.*\.rs|src/v3/compiler/tests/integration/lane2_stage_2d.*\.rs)$` | high | gate lands → lens emits delta; consumer reads only Cost. **Lens-path correction** (codex BLOCKING line 152 fix): `src/v3/lenses/cost.dag` is the live authority, not stale `dsl/std/cost*.dag` |
| C | `m2_lens_cost_migration_test::complexity_.*` | `[Cost, Complexity]` | `[Mgr-fill]` (likely: complexity.dag + lens_complexity*.rs + lens migration substrate) | medium | gate lands → set intersection trigger if either dim has delta |
| C | `m2_lens_idempotency_migration_test::.*` | `[Effect]` | `^(src/v3/lenses/idempotency\.dag|src/v3/compiler/src/lens_idempotency.*\.rs|src/v3/compiler/tests/integration/m2_lens_idempotency.*\.rs)$` | medium | same. **Lens-path correction**: live authority at `src/v3/lenses/idempotency.dag` |
| C | `m2_lens_provenance_migration_test::.*` | `[Value]` | `^(src/v3/lenses/(provenance|emission_provenance)\.dag|src/v3/compiler/src/lens_provenance.*\.rs|src/v3/compiler/tests/integration/m2_lens_provenance.*\.rs)$` | medium | same. **Lens-path correction**: live authority at `src/v3/lenses/provenance.dag` + `src/v3/lenses/emission_provenance.dag` |
| C | `m2_lens_unused_parameters_migration_test::.*` | `[Value]` | `^(src/v3/lenses/unused_parameters\.dag|src/v3/compiler/src/lens_unused_parameters.*\.rs|src/v3/compiler/tests/integration/m2_lens_unused_parameters.*\.rs)$` | medium | same. **Lens-path correction**: live authority at `src/v3/lenses/unused_parameters.dag` |
| D | `complexity_lens_behavioral_completion::.*` | `[Complexity, Cost]` | `[Mgr-fill]` (likely: lens_complexity + lens_cost + cementing substrate) | medium | reads cementing summary for both Complexity and Cost dims |
| D | `cost_lens_symbolic_consumer_test::.*` | `[Cost]` | `[Mgr-fill]` (likely: cost.dag + lens_cost*.rs + E-P substrate) | medium | gate lands → Cost |
| D | `lens_behavioral_parity_demonstration_test::r3_gate_73_.*` | `[Complexity, Cost]` | `[Mgr-fill]` (gate #73 LBP — needs cementing-test substrate dep) | medium | LBP demonstration reads both certainty (Complexity) + symbolic-cost (Cost) |
| D | `lens_cost_target_realization_test::.*` | `[Cost, Value]` | `[Mgr-fill]` (cost-target-realization + rustc harness) | medium | realization composes Cost projection with Value emission |
| E | `dimension::analyze_complexity_tests::.*` | `[Complexity]` | `^(src/v3/lenses/(complexity|cost)\.dag|src/v3/compiler/src/(analyze_complexity|dimension).*\.rs|src/v3/compiler/tests/integration/dimension.*\.rs)$` | medium | gate lands → Complexity. **Lens-path correction**: live authority at `src/v3/lenses/complexity.dag` + `src/v3/lenses/cost.dag` |
| E | `dimension::fail_closed_tests::.*` | `[Complexity]` | `^(src/v3/lenses/(complexity|cost)\.dag|src/v3/compiler/src/(analyze_complexity|dimension).*\.rs)$` | medium | same |
| E | `e7_analyze_complexity_integration::.*` | `[Complexity]` | `^(src/v3/lenses/(complexity|cost)\.dag|src/v3/compiler/src/(analyze_complexity|dimension|public_api).*\.rs)$` | medium | same |
| E | `lane2_stage_2f_dimension_test::.*` | `[Complexity, Cost]` | `^(src/v3/lenses/(complexity|cost)\.dag|src/v3/compiler/src/(lens_complexity|dimension).*\.rs)$` | medium | "composed matches lens" reads BOTH analyze_symbolic_cost (Cost) and Complexity composition |
| F | `m0_acceptance::.*` | `[Value, Cost, Complexity, Effect, Refinement]` | `.*` (conservative — compile-boundary is broad) | low | compile-boundary touches every dim; run-always until lens lands |
| F | `m1_substrate_test::.*` | `[Value]` | `^(dsl/std/.*\.dag|src/v3/compiler/src/(parse|substrate).*\.rs|src/v3/compiler/tests/integration/m1_substrate.*\.rs)$` | medium | substrate reflects parse Value shape |
| F | `tc1_substrate_lens_eta_equivalence_strict_fire_test::.*` | `[Value]` | `[Mgr-fill]` (substrate eta-equivalence — needs substrate-lens dep map) | medium | gate #11 lands; eta-equivalence is a Value-relation |
| F | `t_pb_b_1_dag_runner_test::r3_gate_87_.*` | `[Cost]` | `[Mgr-fill]` (cementing-test substrate + regen harness) | medium | cementing oracle frozen-Cost lens output; gate #87 Cluster M Phase 2 lands → dissolves |
| F | `bootstrap::tests::kernel_bool_path_a_.*` | `[Value]` | `^(src/v3/compiler/src/bootstrap.*\.rs|dsl/std/types\.dag|dsl/std/logic\.dag)$` | medium | dissolves with Lane 1e-2b Path A close. **Path correction**: `dsl/std/boolean_algebra.*\.dag` was stale (no such file); replaced with `dsl/std/logic.dag` (boolean-algebra concepts live there per source tree) |
| G | `t_las_complexity_contract_compile_error_test::.*` | `[Complexity]` | `^(src/v3/lenses/complexity\.dag|src/v3/compiler/src/(las|t_las).*\.rs|src/v3/compiler/tests/integration/t_las.*\.rs)$` | medium | gate #92 lands; compile-error on Complexity contract violation. **Lens-path correction**: live authority at `src/v3/lenses/complexity.dag` |
| G | `t_las_crdt_cost_basis_demo_test::.*` | `[Cost, Effect, Value]` | `.*` | **low (Mgr-fill)** | CRDT cost-basis reads multi-dim (Cost log budget + Effect replica merge + Value replica state). Path-dependency for `las`/`crdt` substrate not yet PM-traced; defaulting to `.*` per conservative discipline until Mgr audits the T-LAS substrate-deps |
| G | `r3_free_consequences_second_batch_test::.*` | `[Cost, Value]` | `[Mgr-fill]` (free-consequences second batch — cross-target optimization + symbolic-cost) | medium | cross-target optimization reads BOTH Cost (symbolic_cost_witness) and Value (constant_fold consistency); gates #43-#52 (free-consequences) land |
| H | `sg2_parse_authority_test::.*` | `[Value]` | `^(src/v3/std/parse_surface\.dag|src/v3/compiler/parse_tables\.dag|src/v3/compiler/src/parse.*\.rs|src/v3/compiler/tests/integration/sg2_.*\.rs)$` | medium | SG-2 parser-staging close. **Path correction**: parse files live at `src/v3/std/parse_surface.dag` + `src/v3/compiler/parse_tables.dag`, not stale `dsl/std/parse*.dag` |
| H | `sg2c1_parse_tables_authority_test::.*` | `[Value]` | `^(src/v3/compiler/parse_tables\.dag|src/v3/(compiler|std)/tokenize\.dag|src/v3/compiler/src/parse.*\.rs)$` | medium | SG-2c-1 close. **Path correction**: live authorities at `src/v3/compiler/parse_tables.dag` + `src/v3/(compiler|std)/tokenize.dag`, not stale `dsl/std/parse_tables.*`. **Regex correction** (openai-pro APPROVE_WITH_COMMENTS on 45fc195a): unescaped pipe for alternation; prior `\|` was a markdown-cell escape that would have been interpreted as literal `compiler|std` by a regex engine, not alternation between `compiler` and `std` |
| H | `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_.*` | `[Value]` | `^(scripts/regen.*|src/v3/compiler/src/(regen|cli).*\.rs|dsl/std/.*\.dag)$` | medium | SG-6 close |
| H | `r1c_e_emit_gates_dag_test::.*` | `[Value]` | `[Mgr-fill]` (R1C-E `.dag` wrapper) | low | R1C-E closure |
| H | `r3_verification_l4_l7_l5_skeleton_test::r3_verification_l4_.*` | `[Value]` | `[Mgr-fill]` (R3-V L4 direct consumer) | medium | gates #43+ L4-L7 lane |
| H | `r3_verification_l4_l7_l5_skeleton_test::r3_verification_l7_.*` | `[Value]` | `[Mgr-fill]` (gate #10 algebraic-law matrix) | medium | gate #10 close |
| H | `t_ci_workflow_as_data_demo_test::.*` | `[Value, Cost]` | `^(src/v3/std/workflows\.dag|src/v3/compiler/src/(workflow|evaluator).*\.rs|src/v3/compiler/tests/integration/t_ci_workflow.*\.rs)$` | medium | workflow-as-data demo reads both Value (workflow shape) + Cost (DimensionReport timing dim evaluates). **Path correction**: live authority at `src/v3/std/workflows.dag`, not stale `dsl/std/workflow*.dag` |
| I | `t_demo_fixture_test::.*` | `[Value]` | `^(tests/.*/t_demo.*\.dag|src/v3/compiler/src/(test_runner|runner).*\.rs|src/v3/compiler/tests/integration/t_demo.*\.rs)$` | medium | T-Demo lane close |
| I | `thesis_validation_test::kf_1_.*` | `[Value, Cost, Complexity, Effect, Refinement]` | `.*` (conservative — thesis-level validation is broad) | low | thesis-level validation reads every dim |
| I | `common::cached_compile::tests::.*` | `[Value]` | `^(src/v3/compiler/src/cached_compile.*\.rs|src/v3/compiler/tests/common/cached_compile.*\.rs)$` | high | TESTING.md paydown |
| I | `int_literal_cardinality_test::int_refinement_overflow_is_proven_.*` | `[Refinement]` | `^(dsl/std/int.*\.dag|src/v3/compiler/src/(refinement|int_literal).*\.rs|src/v3/compiler/tests/integration/int_literal.*\.rs)$` | high | gate #21 close |
| I | `m1_5_verification_test::symbolic_cost_expr_equals_smoke_.*` | `[Cost]` | `[Mgr-fill]` (M1.5 verification smoke — needs symbolic-cost equality harness dep map) | medium | M1.5 close |

---

## §4. Open questions for Mgr-fill

1. **Multi-dimension rows (D, G, F-`m0_acceptance`, I-`thesis_validation_test`)**: these carry sets like `[Cost, Complexity]` or the full 5-dim set. Mgr should NOT narrow these sets — per the locked design, a test that reads N dimensions is in the affected-set when ANY of those N dimensions has a delta (union semantics). The earlier draft of this template ("primary dimension is sufficient") was a semantic-contract violation caught by codex REQUEST_CHANGES on PR #2721 and is fixed in this revision. **When in doubt about a dim, ADD it to the set, never narrow.**

2. **Conservative-run-always candidates**: `m0_acceptance::*` and `thesis_validation_test::*` carry the full 5-dim set + `.*` regex (always-run). Mgr decides if these justify the cost (~3-5s each cold) or warrant tighter scoping. PM read: keep `.*` until the lens lands; the safety margin is cheap.

3. **`[Mgr-fill]` rows count**: 12 rows out of ~36 need Mgr to derive `required_paths_regex` from consumer tracing. PM left these blank where derivation requires deeper substrate knowledge (substrate-lens deps, R3-V L4/L7 direct-consumer maps, R1C-E `.dag` wrapper internals, free-consequences cross-target topology).

4. **Mechanism — per-row regex + cluster-level aggregation**: each row contributes a per-row intersection check `(changed_files ∩ row.required_paths_regex matches) = ∅`. The cluster-level boolean **aggregates by conjunction** (all-rows-empty), NOT disjunction:

   ```
   skip_<cluster> = ∀ row ∈ cluster : (changed_files ∩ row.required_paths_regex) = ∅
   ```

   Equivalently: `run_<cluster> = ∃ row ∈ cluster : row affected`. Aggregating by disjunction (any-row-empty) would silently skip the cluster when only ONE row is affected — running affected tests is mandatory per locked-design contract. **Conjunction is load-bearing; do not invert.**

   Dimension-set is the **structural carrier for post-dissolution lens substitution**: when the lens lands, per-row paths flip to per-row dimension-intersection, but the cluster-level conjunction over rows stays the same:

   ```
   skip_<cluster> = ∀ row ∈ cluster : (affected_dimensions ∩ row.dimensions) = ∅
   ```

   Same polarity, same aggregation. The CI consumer is a shell snippet in `ci.yml`:
   ```yaml
   - name: <Cluster X test step>
     if: needs.changes.outputs.skip_<cluster> != 'true'
     run: cargo test -p v3-compiler --test integration <module_filter>
   ```

5. **PM-best-effort regex verification** (per codex BLOCKING on earlier revision): the `required_paths_regex` values in §3 are PM-authored without source-tree verification at the cited inventory sha (`e9c8f9896`). Workers MUST validate each regex against live source tree at HEAD before CI implementation. Unverified or unverifiable regexes → replace with `.*` per the conservative fail-closed default. Treat the `confidence` column as audit priority: `low` → `.*` first, `medium` → audit then decide, `high` → audit but likely fine.

6. **Pilot cluster selection**: which cluster does Mgr prototype first? PM recommendation: **Cluster B (Lane 2 Stage 2d symbolic cost)** — high confidence in path-regex, contained module, 7 tests, **single-dimension set `[Cost]`** (no multi-dim union complexity for the pilot). Lowest risk, highest learning per LOC.

---

## §5. Acceptance (Mgr fills, Director ratifies)

Mgr-fill complete when:

- [ ] Every `[Mgr-fill]` placeholder in §3 is replaced with a concrete `required_paths_regex` OR explicitly marked `.*` (always-run) with a one-line "why conservative".
- [ ] Each cluster has a pilot-PR worker brief authored (or batched into the Director-authored Layer 2 brief).
- [ ] `changes` job in `ci.yml` extended with per-cluster `skip_<cluster>` boolean outputs.
- [ ] At least one cluster (pilot) has its `if: needs.changes.outputs.skip_<cluster> != 'true'` gate landed and CI-validated against a representative test case (docs-only PR skips; in-cluster code change runs).
- [ ] All path-mapping entries have a `dimensions:` field that is a Set<Dimension> with every member of the set in `{Value, Cost, Complexity, Effect, Refinement}`. **Single-element sets are valid (e.g., `[Cost]`); narrowing a known-multi-dim consumer to a single-element set is not.**
- [ ] Post-dissolution mapping verified: each row's per-row formula reads `(affected_dimensions ∩ row.dimensions) = ∅` (skip when intersection IS empty / nothing affected), NOT `≠ ∅`. The CI gate is `if: skip_<cluster> != 'true'` (run when skip is false); inverting the polarity silently skips affected tests.
- [ ] **Cluster-level aggregation verified**: `skip_<cluster>` aggregates per-row results by conjunction (`∀ row : row-skip-empty`). NOT disjunction (`∃ row : row-skip-empty`). Disjunction silently skips affected rows when only one row is unaffected.
- [ ] **Path regex validation against live source tree**: every concrete (non-`.*`) `required_paths_regex` validated against source tree at HEAD before CI implementation. Unverified entries replaced with `.*` per conservative default. Validation record kept (PR description or commit message citing the validation pass).

---

## §6. STOP triggers (Mgr aborts and surfaces to Director)

- Any cluster needs a *new* substrate carrier to express path-dependency → STOP. Surface to Director: this is the lens substrate, not a bridge.
- Any path-mapping entry has a `dimensions:` element outside `{Value, Cost, Complexity, Effect, Refinement}` → STOP. Surface to Director: this is a coproduct-dissolution candidate.
- Any row tempted to use a single-dimension `dimensions:` set when consumer tracing reveals multi-dim reads → STOP and EXPAND the set. Narrowing is a semantic-contract violation (codex REQUEST_CHANGES on PR #2721 caught one such instance; do not regress).
- Any cluster's `required_paths_regex` requires depending on test-output (not just changed files) → STOP. That's the lens, not the bridge.
- Any cluster aggregation tempted to use **disjunction** (`∃ row : row-skip-empty`) instead of **conjunction** (`∀ row : row-skip-empty`) → STOP. Disjunction silently skips affected rows; conjunction is load-bearing (codex BLOCKING on PR #2721 caught this gap; aggregation predicate now explicit at §3 + §4 + §5).
- Any concrete `required_paths_regex` adopted without source-tree validation at HEAD → STOP and replace with `.*` per conservative fail-closed default. PM-best-effort regexes in §3 are NOT pre-validated; Mgr must validate before CI implementation (codex BLOCKING on PR #2721 caught this gap).

---

## §7. Cross-references

- **Layer 1 (predecessor)**: PR #2718 — docs-only-skip `v3` via `changes` job + `code` boolean output. Layer 2 extends the same job with per-group flags.
- **Affected-set lens (dissolution)**: `docs/design-affected-set-lens.md` (locked design), PR #2713 (Verification Mgr canvas rebuild).
- **Routing decision**: msg_a77c7f42 (Director ratification of Verification Mgr ownership) — gunbc#828 c4425726922 carries PM's 3-ask scaffold data.
- **Director-authored worker brief**: `docs/briefs/r3-ci-layer-2-path-conditional-gating-worker.md` (pending; ETA per Director ~23:00-23:30Z 2026-05-11).
- **Memory**: `feedback_parallel_representation_debt` (single-Mgr ownership rationale); `feedback_intro_rate_vs_residual_share` (intro-class composition framing).

---

**End of pre-staged Mgr-fill template.** PM signing off; Mgr inherits via Director-cited brief.
