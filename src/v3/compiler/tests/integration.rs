//! Consolidated v3-compiler integration test binary.
//!
//! **Why one binary.** Rust integration tests default to one binary per
//! `tests/*.rs` file. Each binary pays a separate bootstrap + compile cost
//! on cold runners (bootstrap alone is ~200ms; the in-memory
//! `cached_compile_to_dag` helper only amortizes within-binary). Hoisting
//! every test file into this single module tree means:
//!
//! - **One bootstrap per cargo test run** — shared across every test.
//! - **Cross-test cache hits** — two tests that pass identical `(source,
//!   file)` arguments to `cached_compile_to_dag` now share the compile
//!   result. Different file markers produce distinct cache keys by design
//!   (the cache identity is the exact compile invocation).
//! - **One compile, link, and load cycle** — no 25× rustc invocations for
//!   test-binary production.
//!
//! **Module discipline.** Each file under `tests/integration/*.rs`,
//! `tests/integration/cementing/*.rs`, `tests/boundary/*.rs`, or
//! `tests/unit/*.rs` is a sibling module at this crate root, reached via
//! `#[path]` because Rust's default module resolution for a crate-root file
//! looks in the containing directory (`tests/`) rather than a same-named
//! subdirectory. Shared helpers live under `tests/integration/common/`.
//! Inside a test module, `use crate::common::…` reaches those helpers; there
//! is no per-file `mod common;` declaration.
//!
//! **Layer taxonomy (TESTING.md § test layers).** Files are partitioned
//! by directory:
//! - `tests/unit/`        — lenses, accessors, single-pass behaviors (<5ms)
//! - `tests/integration/` — multi-stage pipeline, fixed-point convergence (<100ms)
//! - `tests/boundary/`    — rustc/go/python roundtrips, emitted-module behavior (<2s)
//!
//! Each moved test file carries a `//! **Layer:** <unit|integration|boundary>`
//! header so `grep -rn '\*\*Layer:\*\*'` reports the current partition.
//! The taxonomy is the directory; the header is a human-readable echo.

#[macro_use]
#[path = "integration/common/mod.rs"]
mod common;

#[path = "integration/anthropic_messages_callable_test.rs"]
mod anthropic_messages_callable_test;
#[path = "integration/anthropic_operations_test.rs"]
mod anthropic_operations_test;
#[path = "integration/anthropic_schema_lockstep_test.rs"]
mod anthropic_schema_lockstep_test;
#[path = "integration/bridge_ledger_carrier_test.rs"]
mod bridge_ledger_carrier_test;
#[path = "integration/bridge_lower_helpers_patch_zero_residual_test.rs"]
mod bridge_lower_helpers_patch_zero_residual_test;
#[path = "integration/canonical_lens_bridge_ratchet_test.rs"]
mod canonical_lens_bridge_ratchet_test;
#[path = "integration/cementing/cementing_lens_registry_dispatch_test.rs"]
mod cementing_lens_registry_dispatch_test;
#[path = "integration/e_i_lane_induction_preflight_test.rs"]
mod e_i_lane_induction_preflight_test;
#[path = "integration/extdeps_rust_primitives_loader_test.rs"]
mod extdeps_rust_primitives_loader_test;
#[path = "integration/four_fixture_regression_test.rs"]
mod four_fixture_regression_test;
#[path = "integration/idempotency_lens_instance_blocker_test.rs"]
mod idempotency_lens_instance_blocker_test;
#[path = "integration/int_literal_cardinality_test.rs"]
mod int_literal_cardinality_test;
#[path = "integration/l1_5_fixed_point_test.rs"]
mod l1_5_fixed_point_test;
#[path = "integration/lane2_stage_2a_effects_smoke.rs"]
mod lane2_stage_2a_effects_smoke;
#[path = "integration/lane2_stage_2b_db18_test.rs"]
mod lane2_stage_2b_db18_test;
#[path = "integration/lane2_stage_2c_db15_test.rs"]
mod lane2_stage_2c_db15_test;
#[path = "integration/lane2_stage_2d_symbolic_cost_test.rs"]
mod lane2_stage_2d_symbolic_cost_test;
#[path = "integration/lane2_stage_2e_parallelism_test.rs"]
mod lane2_stage_2e_parallelism_test;
#[path = "integration/lane3_stage_3b_db1_test.rs"]
mod lane3_stage_3b_db1_test;
#[path = "integration/lens_register_correspondence_test.rs"]
mod lens_register_correspondence_test;
#[path = "integration/lens_substrate_carrier_test.rs"]
mod lens_substrate_carrier_test;
#[path = "integration/m0_acceptance.rs"]
mod m0_acceptance;
#[path = "boundary/m1_3_emit_go_test.rs"]
mod m1_3_emit_go_test;
#[path = "boundary/m1_3_emit_rust_test.rs"]
mod m1_3_emit_rust_test;
#[path = "integration/m1_3_lens_cost_test.rs"]
mod m1_3_lens_cost_test;
#[path = "integration/m1_3_lens_unused_parameters_test.rs"]
mod m1_3_lens_unused_parameters_test;
#[path = "boundary/m1_4_emit_python_test.rs"]
mod m1_4_emit_python_test;
#[path = "boundary/m1_5_emit_omni_demo_test.rs"]
mod m1_5_emit_omni_demo_test;
#[path = "integration/m1_5_testgen_test.rs"]
mod m1_5_testgen_test;
#[path = "integration/m1_5_user_authored_lens_gate_test.rs"]
mod m1_5_user_authored_lens_gate_test;
#[path = "integration/m1_5_verification_test.rs"]
mod m1_5_verification_test;
#[path = "integration/m1_fn_external_body_reconciliation_test.rs"]
mod m1_fn_external_body_reconciliation_test;
#[path = "integration/m1_lens_structural_resolution_test.rs"]
mod m1_lens_structural_resolution_test;
#[path = "integration/m1_substrate_test.rs"]
mod m1_substrate_test;
#[path = "boundary/m2_emit_multi_field_struct_variant_test.rs"]
mod m2_emit_multi_field_struct_variant_test;
#[path = "integration/m2_feature_parity_test.rs"]
mod m2_feature_parity_test;
#[path = "integration/m2_field_access_binding_test.rs"]
mod m2_field_access_binding_test;
#[path = "integration/m2_lens_cost_migration_test.rs"]
mod m2_lens_cost_migration_test;
#[path = "integration/m2_lens_idempotency_emit_test.rs"]
mod m2_lens_idempotency_emit_test;
#[path = "integration/m2_lens_idempotency_migration_test.rs"]
mod m2_lens_idempotency_migration_test;
#[path = "integration/m2_lens_provenance_migration_test.rs"]
mod m2_lens_provenance_migration_test;
#[path = "integration/m2_lens_structural_resolution_migration_test.rs"]
mod m2_lens_structural_resolution_migration_test;
#[path = "integration/m2_lens_unused_parameters_migration_test.rs"]
mod m2_lens_unused_parameters_migration_test;
#[path = "integration/m2_lens_variant_payload_migration_test.rs"]
mod m2_lens_variant_payload_migration_test;
#[path = "integration/m2_substrate_inhabitance_test.rs"]
mod m2_substrate_inhabitance_test;
#[path = "integration/method_registry_test.rs"]
mod method_registry_test;
#[path = "integration/method_template_contract_test.rs"]
mod method_template_contract_test;
#[path = "integration/p0_std_render_repeat_string_test.rs"]
mod p0_std_render_repeat_string_test;
#[path = "integration/pb1_bootstrap_full_snapshot_test.rs"]
mod pb1_bootstrap_full_snapshot_test;
#[path = "integration/pb_method_template_projection_dag_emit_test.rs"]
mod pb_method_template_projection_dag_emit_test;
#[path = "integration/pb_method_template_projection_test.rs"]
mod pb_method_template_projection_test;
#[path = "integration/pipe_desugar.rs"]
mod pipe_desugar;
#[path = "integration/prereq_x_call_on_field_access_ratchet_test.rs"]
mod prereq_x_call_on_field_access_ratchet_test;
#[path = "integration/r1_release_acceptance_test.rs"]
mod r1_release_acceptance_test;
#[path = "integration/r1c_d_pb_census_gates_test.rs"]
mod r1c_d_pb_census_gates_test;
#[path = "integration/r1c_e_emit_gates_dag_test.rs"]
mod r1c_e_emit_gates_dag_test;
#[path = "integration/r1c_e_emit_gates_omni_dag_test.rs"]
mod r1c_e_emit_gates_omni_dag_test;
#[path = "integration/r2_b5_loop_construction_closure_test.rs"]
mod r2_b5_loop_construction_closure_test;
#[path = "integration/r3_free_consequences_first_batch_test.rs"]
mod r3_free_consequences_first_batch_test;
#[path = "integration/r3_free_consequences_second_batch_test.rs"]
mod r3_free_consequences_second_batch_test;
#[path = "integration/r3_verification_l4_l7_l5_skeleton_test.rs"]
mod r3_verification_l4_l7_l5_skeleton_test;
#[path = "integration/services_carrier_shape_test.rs"]
mod services_carrier_shape_test;
#[path = "integration/sg0_census_test.rs"]
mod sg0_census_test;
#[path = "integration/sg1_tokenize_authority_test.rs"]
mod sg1_tokenize_authority_test;
#[path = "integration/sg2_parse_authority_test.rs"]
mod sg2_parse_authority_test;
#[path = "integration/sg2c1_parse_tables_authority_test.rs"]
mod sg2c1_parse_tables_authority_test;
#[path = "integration/sg2c5_soft_keyword_ident_test.rs"]
mod sg2c5_soft_keyword_ident_test;
#[path = "integration/sg3_lower_parse_surface_stack_test.rs"]
mod sg3_lower_parse_surface_stack_test;
#[path = "integration/sg3_surface_reflection_consumer_test.rs"]
mod sg3_surface_reflection_consumer_test;
#[path = "integration/sg6_hand_authored_census_test.rs"]
mod sg6_hand_authored_census_test;
#[path = "integration/sg7_prep_variant_payload_freshness_test.rs"]
mod sg7_prep_variant_payload_freshness_test;
#[path = "integration/shape_a_target_source_filtering_authority_test.rs"]
mod shape_a_target_source_filtering_authority_test;
#[path = "integration/t_impossiblebugs_unenumerated_effects_test.rs"]
mod t_impossiblebugs_unenumerated_effects_test;
#[path = "integration/t_pb_b_1_dag_runner_test.rs"]
mod t_pb_b_1_dag_runner_test;
#[path = "integration/t_pb_b_brief_d_fixture_smoke_test.rs"]
mod t_pb_b_brief_d_fixture_smoke_test;
#[path = "integration/tc1_substrate_lens_eta_equivalence_deferred_test.rs"]
mod tc1_substrate_lens_eta_equivalence_deferred_test;
#[path = "integration/tc3_strong_normalization_deferred_test.rs"]
mod tc3_strong_normalization_deferred_test;
#[path = "integration/test_runner_test.rs"]
mod test_runner_test;
#[path = "integration/thesis_parallelism_test.rs"]
mod thesis_parallelism_test;
#[path = "integration/thesis_validation_test.rs"]
mod thesis_validation_test;
#[path = "integration/workflow_root_port_test.rs"]
mod workflow_root_port_test;

mod t_demo_fixture_test {
    //! **Layer:** integration

    use std::fs;
    use std::path::PathBuf;

    use crate::common::cached_compile_to_dag;
    use v3_compiler::compile_to_dag;
    use v3_compiler::dag::Dag;
    use v3_compiler::test_runner::{ClaimResult, TestRunner};
    use v3_compiler::CompileError;

    const FIXTURE: &str = "src/v3/compiler/tests/t_demo/t_demo_fixtures.dag";

    /// Byte-sync with `t_demo_structural_cost_obligation_gate.source` in `t_demo_fixtures.dag`.
    const T_DEMO_STRUCTURAL_COST_OBLIGATION_CLAIM_SOURCE: &str = "fn pair_score(xs: List<Int>) -> Int = fold(xs, 0, |outer, x| outer + fold(xs, 0, |inner, y| inner + x + y))\nlet complexity_demo_out: Int = pair_score(cons(1, singleton(2)))\n";

    fn fixture_source() -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(FIXTURE);
        fs::read_to_string(path).expect("read T-Demo fixture skeleton")
    }

    fn compile_fixture(source: &str) -> Dag {
        cached_compile_to_dag(source, FIXTURE)
    }

    #[test]
    fn t_demo_fixture_skeleton_compiles() {
        let source = fixture_source();
        let dag = compile_fixture(&source);

        assert!(
            dag.diagnostics().is_empty(),
            "T-Demo fixture skeleton should compile without diagnostics: {:?}",
            dag.diagnostics()
        );
    }

    #[test]
    fn t_demo_canonical_suites_are_runner_visible() {
        let source = fixture_source();
        let dag = compile_fixture(&source);

        for suite_name in [
            "fixture_compiler_nerd_canonical",
            "fixture_integration_canonical",
        ] {
            let results = TestRunner::new(&dag).run_suite(suite_name);
            assert!(
                !results.is_empty(),
                "T-Demo suite `{suite_name}` should contain Day-1 Compiles claims"
            );
            assert!(
                results
                    .iter()
                    .all(|result| result.result == ClaimResult::Pass),
                "T-Demo suite `{suite_name}` should pass Day-1 Compiles claims, got {results:?}"
            );
        }
    }

    /// ROADMAP T-Demo / PR #764: `FailsWithDiagnostic.detail_contains` must pin the **sum
    /// constructor** failure (`AppendEffect()` is not an `IdempotentShape` case), not a generic
    /// `compose_effects` argument refinement message that omits `AppendEffect`.
    #[test]
    fn impossible_bug_idempotency_violation_emits_named_constructor_resolve_error() {
        let src = "let bad_compose = compose_effects([{ operation_name: \"noop\", shape: IsIdempotent(AppendEffect()) }])\n";
        let err = compile_to_dag(src, "impossible_bug_idempotency.v3")
            .expect_err("idempotency-violation witness should not compile");
        let CompileError::Semantic(dag) = err else {
            panic!("expected Semantic(Dag) handoff, got {err:?}");
        };
        let msgs: Vec<String> = dag.diagnostics().iter().map(|(_, d)| d.message()).collect();
        let needle = "named constructor `AppendEffect` is not a variant of the expected sum type";
        assert!(
            msgs.iter().any(|m| m.contains(needle)),
            "expected nullary-call lowering to reject AppendEffect as IdempotentShape payload; got: {msgs:?}"
        );
    }

    #[test]
    fn t_demo_impossible_bug_suite_r1_passes() {
        let source = fixture_source();
        let dag = compile_fixture(&source);
        let results = TestRunner::new(&dag).run_suite("impossible_bug_class_suite_r1");
        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .all(|result| result.result == ClaimResult::Pass),
            "impossible-bug suite claims should all Pass (FailsWithDiagnostic receipts only), got {results:?}"
        );
    }

    /// R1C-F T-Demo gate: a user-authored lens (`lenses.named_function_count`, the same
    /// GREEN T-LensAPI lens) detects 3 named bindings in the violating program; the
    /// `LensOutputEquals` predicate matches and the gate Passes — proving the proof
    /// surface is user-extensible (THESIS §"User-defined dimensions").
    #[test]
    fn t_demo_user_authored_lens_rejects_violating_program_passes() {
        let source = fixture_source();
        let dag = compile_fixture(&source);
        let results = TestRunner::new(&dag)
            .run_suite("demo_user_authored_lens_rejects_violating_program_suite");
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].result, ClaimResult::Pass),
            "user-authored lens demo gate should Pass (lens detected violations and matched expected count), got {:?}",
            results[0].result
        );
    }

    #[test]
    fn t_demo_structural_cost_obligation_witness_compiles_cleanly() {
        compile_to_dag(
            T_DEMO_STRUCTURAL_COST_OBLIGATION_CLAIM_SOURCE,
            "t_demo_structural_cost_obligation.v3",
        )
        .unwrap_or_else(|err| {
            panic!(
                "T-Demo structural cost witness must compile so CostBounded exercises lens_cost::cost_of, not tokenizer/parse failures: {err:?}"
            )
        });
    }

    #[test]
    fn t_demo_structural_cost_obligation_suite_observes_cost_bound_fail() {
        let source = fixture_source();
        let dag = compile_fixture(&source);
        let results = TestRunner::new(&dag).run_suite("t_demo_structural_cost_obligation_suite");
        assert_eq!(results.len(), 1);
        let ClaimResult::Fail(msg) = &results[0].result else {
            panic!(
                "structural cost obligation gate should Fail CostBounded (cost exceeds bound), got {:?}",
                results[0].result
            );
        };
        assert!(
            msg.starts_with("cost ") && msg.contains("did not satisfy bound"),
            "unexpected CostBounded failure message (expected structural bound receipt, not compile skip): {msg}"
        );
    }
}

mod lane2_stage_2f_dimension_test {
    use v3_compiler::analyze_symbolic_cost_dimension;
    use v3_compiler::compile_to_dag;
    use v3_compiler::dag::{Behavior, Dag, DeclarationId, PortId, TypeConnective};
    use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
    use v3_compiler::DimensionReport;

    fn find_bind_port(dag: &Dag, name: &str) -> PortId {
        dag.nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|bind| bind.name == name)
            .unwrap_or_else(|| panic!("bind `{name}` not found"))
            .value
    }

    fn find_bind_root(dag: &Dag, name: &str) -> v3_compiler::dag::NodeId {
        dag.nodes()
            .iter()
            .find(|behavior| {
                behavior
                    .as_bind()
                    .map(|bind| bind.name == name)
                    .unwrap_or(false)
            })
            .map(|behavior| behavior.id())
            .unwrap_or_else(|| panic!("bind `{name}` not found"))
    }

    #[test]
    fn no_authored_analysis_dimension_carrier_constants_in_bootstrap_stdlib() {
        let dag = Dag::new();
        let dimension_template = dag
            .declaration_by_name("AnalysisDimension")
            .expect("bootstrap loads AnalysisDimension")
            .id;
        let count = dag
            .declarations()
            .iter()
            .filter(|decl| {
                decl.value_body.is_some()
                    && matches!(
                        &decl.connective,
                        TypeConnective::Instantiation { template, .. }
                            if *template == dimension_template
                    )
            })
            .count();
        assert_eq!(
            count, 0,
            "no `data _: AnalysisDimension<_> = ...` values ship until class-5 bodies unlock the receipt"
        );
    }

    fn named_decl(dag: &Dag, name: &str) -> DeclarationId {
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("bootstrap loads {name}"))
            .id
    }

    fn instantiation_parts(dag: &Dag, id: DeclarationId) -> (DeclarationId, Vec<DeclarationId>) {
        match &dag.declaration(id).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => (*template, arguments.iter().map(|arg| arg.value).collect()),
            other => panic!("expected instantiation for {id:?}, got {other:?}"),
        }
    }

    #[test]
    fn dimension_value_wrapper_carries_unit_phantom_axis() {
        let dag = Dag::new();
        let dimension = dag
            .declaration_by_name("Dimension")
            .expect("bootstrap loads Dimension");
        assert_eq!(dimension.type_params.len(), 2);
        assert_eq!(dimension.phantom_params.len(), 1);
        assert_eq!(
            dimension.phantom_params[0].parameter,
            dimension.type_params[0]
        );

        let abelian_group = named_decl(&dag, "AbelianGroup");
        let (algebra_template, algebra_args) =
            instantiation_parts(&dag, dimension.phantom_params[0].algebra);
        assert_eq!(algebra_template, abelian_group);
        assert_eq!(algebra_args, vec![dimension.type_params[0]]);

        for unit in [
            "Meters",
            "Seconds",
            "Kilograms",
            "Amperes",
            "Kelvin",
            "Moles",
            "Candela",
        ] {
            named_decl(&dag, unit);
        }

        for function in [
            "add_dimension",
            "sub_dimension",
            "mul_dimension_scalar",
            "div_dimension_scalar",
        ] {
            let decl = dag.declaration(named_decl(&dag, function));
            assert!(
                matches!(decl.connective, TypeConnective::Arrow { .. }),
                "{function} should be present in bootstrap as a callable arrow"
            );
        }
    }

    #[test]
    fn analyze_symbolic_cost_composed_matches_lens_at_workflow_root() {
        let dag = compile_to_dag("let x = 1 + 2", "lane2_2f_dim.v3").expect("compiles");
        let root = find_bind_root(&dag, "x");
        let report = analyze_symbolic_cost_dimension(&dag, root);
        let lens = match symbolic_cost_of(&dag, &find_bind_port(&dag, "x")) {
            SymbolicCostLookup::Hit(cost) => cost,
            SymbolicCostLookup::Miss => panic!("expected Hit"),
        };
        let DimensionReport::DimensionOk {
            composed,
            dimension_name,
            witnesses,
        } = report
        else {
            panic!("expected DimensionOk for well-typed program, got {report:?}");
        };
        assert_eq!(composed, lens);
        assert_eq!(dimension_name, "symbolic_cost");
        assert!(
            witnesses.len() < dag.nodes().len(),
            "dimension witnesses should be workflow-scoped, not materialized for every bootstrap behavior"
        );
    }

    #[test]
    fn dimension_report_carrier_is_pass_fail_sum_in_bootstrap() {
        let dag = Dag::new();
        let decl = dag
            .declaration_by_name("DimensionReport")
            .expect("bootstrap loads DimensionReport");
        let TypeConnective::Disj { variants } = &decl.connective else {
            panic!(
                "DimensionReport must be a pass/fail sum (Disj), got {:?}",
                decl.connective
            );
        };
        let labels: Vec<_> = variants.iter().map(|v| v.label.as_str()).collect();
        assert_eq!(
            labels,
            vec!["DimensionOk", "DimensionFail"],
            "expected DimensionOk | DimensionFail variants"
        );
        let ok_payload = dag.declaration(variants[0].ty);
        let TypeConnective::Conj { children } = &ok_payload.connective else {
            panic!("DimensionOk payload should be a record");
        };
        let ok_fields: Vec<_> = children.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            ok_fields,
            vec!["dimension_name", "composed", "witnesses"],
            "DimensionOk should carry composed only on the pass arm"
        );
        let fail_payload = dag.declaration(variants[1].ty);
        let TypeConnective::Conj {
            children: fail_children,
        } = &fail_payload.connective
        else {
            panic!("DimensionFail payload should be a record");
        };
        let fail_fields: Vec<_> = fail_children.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            fail_fields,
            vec!["dimension_name", "violations", "witnesses"],
            "DimensionFail must not admit composed; violations carry proof failure"
        );
    }
}

/// PR-E E7 symbolic-cost-only — public-API integration coverage.
///
/// `analyze_complexity` is the named E7 entrypoint authorized by #1471
/// and landed in #1484. These tests pin that the **public crate API**
/// (`v3_compiler::analyze_complexity`) is reachable from outside the
/// `dimension` module, delegates to the existing symbolic-cost
/// analyzer, and preserves the typed `DimensionReport` /
/// `Diagnostic` partition without parsing `Witness::Violates.reason`.
mod e7_analyze_complexity_integration {
    use v3_compiler::compile_to_dag;
    use v3_compiler::dag::Behavior;
    use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};
    use v3_compiler::{analyze_complexity, analyze_symbolic_cost_dimension, DimensionReport};

    fn find_bind_root(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::NodeId {
        dag.nodes()
            .iter()
            .find(|behavior| {
                behavior
                    .as_bind()
                    .map(|bind| bind.name == name)
                    .unwrap_or(false)
            })
            .map(|behavior| behavior.id())
            .unwrap_or_else(|| panic!("bind `{name}` not found"))
    }

    fn find_bind_port(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::PortId {
        dag.nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|bind| bind.name == name)
            .unwrap_or_else(|| panic!("bind `{name}` not found"))
            .value
    }

    /// E7 §test 1 (integration form): the public `analyze_complexity`
    /// API delegates to the live `analyze_symbolic_cost_dimension` —
    /// pinned by structural equality of the resulting `DimensionOk`
    /// fields. Single-authority via the public crate surface.
    #[test]
    fn analyze_complexity_public_api_delegates_to_symbolic_cost_dimension() {
        let dag = compile_to_dag("let y = 3 + 4", "e7_int_match.v3").expect("compiles");
        let root = find_bind_root(&dag, "y");

        let via_complexity = analyze_complexity(&dag, root);
        let via_dimension = analyze_symbolic_cost_dimension(&dag, root);

        match (&via_complexity, &via_dimension) {
            (
                DimensionReport::DimensionOk {
                    dimension_name: cn,
                    composed: cc,
                    witnesses: cw,
                },
                DimensionReport::DimensionOk {
                    dimension_name: dn,
                    composed: dc,
                    witnesses: dw,
                },
            ) => {
                assert_eq!(cn, dn);
                assert_eq!(cc, dc);
                assert_eq!(cw.len(), dw.len());
                // Strengthen length-equality to per-witness content
                // equality on the typed Inhabits arm (the only arm
                // reachable on a well-typed program through the
                // public surface). SymbolicCost derives PartialEq, so
                // structural equality on the carrier is honest;
                // Violates would need Behavior PartialEq which it
                // does not derive — fail-arm equality stays in the
                // in-module unit tests where ghost-port DAGs are
                // constructible.
                for (cw_i, dw_i) in cw.iter().zip(dw.iter()) {
                    match (cw_i, dw_i) {
                        (
                            v3_compiler::Witness::Inhabits(cc),
                            v3_compiler::Witness::Inhabits(dc),
                        ) => assert_eq!(cc, dc),
                        (
                            v3_compiler::Witness::Violates { .. },
                            v3_compiler::Witness::Violates { .. },
                        ) => {
                            panic!(
                                "well-typed program produced Violates witnesses on both arms; \
                                 the symbolic-cost analyzer should not emit Violates here, \
                                 so the test must be revisited (likely a regression)."
                            );
                        }
                        other => panic!(
                            "wrapper and direct analyzer produced different witness arms: {other:?}"
                        ),
                    }
                }
            }
            other => panic!(
                "expected both DimensionOk with matching content via the public API, got {other:?}",
            ),
        }
    }

    /// E7 §test 1 (root-arg observable): a two-bind program makes the
    /// `workflow_root` argument structurally observable. The wrapper
    /// must agree with `analyze_symbolic_cost_dimension` on the
    /// non-default root specifically — a wrapper that ignored the
    /// supplied root and always picked "the only bind" or "the first
    /// bind" would diverge here on **witness-spine size**, since the
    /// reachable-behaviors filter for the second bind covers strictly
    /// more nodes than the first.
    #[test]
    fn analyze_complexity_public_api_honors_supplied_workflow_root() {
        let dag = compile_to_dag("let a = 1 + 2\nlet b = a + 3 + 4", "e7_int_two_binds.v3")
            .expect("compiles");

        let root_a = find_bind_root(&dag, "a");
        let root_b = find_bind_root(&dag, "b");
        assert_ne!(root_a, root_b, "two-bind fixture must distinguish roots");

        // For each root the public wrapper agrees with the underlying
        // analyzer on the same root.
        for selected_root in [root_a, root_b] {
            let via_complexity = analyze_complexity(&dag, selected_root);
            let via_dimension = analyze_symbolic_cost_dimension(&dag, selected_root);

            match (&via_complexity, &via_dimension) {
                (
                    DimensionReport::DimensionOk {
                        composed: cc,
                        witnesses: cw,
                        ..
                    },
                    DimensionReport::DimensionOk {
                        composed: dc,
                        witnesses: dw,
                        ..
                    },
                ) => {
                    assert_eq!(cc, dc, "wrapper must honor selected root {selected_root:?}");
                    assert_eq!(
                        cw.len(),
                        dw.len(),
                        "wrapper must produce the same witness-spine count for root {selected_root:?}",
                    );
                    // Strengthen length-equality to per-witness
                    // content equality on the typed `Inhabits` arm.
                    // SymbolicCost has PartialEq; Behavior on the
                    // Violates arm does not, so fail-arm content
                    // equality stays in the in-module unit tests.
                    for (cw_i, dw_i) in cw.iter().zip(dw.iter()) {
                        match (cw_i, dw_i) {
                            (
                                v3_compiler::Witness::Inhabits(cc),
                                v3_compiler::Witness::Inhabits(dc),
                            ) => assert_eq!(
                                cc, dc,
                                "wrapper must produce identical Inhabits content for root {selected_root:?}",
                            ),
                            (
                                v3_compiler::Witness::Violates { .. },
                                v3_compiler::Witness::Violates { .. },
                            ) => panic!(
                                "well-typed two-bind fixture should not emit Violates witnesses; \
                                 likely regression for root {selected_root:?}",
                            ),
                            other => panic!(
                                "wrapper and direct analyzer produced different witness arms for \
                                 root {selected_root:?}: {other:?}",
                            ),
                        }
                    }
                }
                other => panic!(
                    "expected both DimensionOk for selected root {selected_root:?}, got {other:?}",
                ),
            }
        }

        // The two roots reach different sets of behaviors via
        // `workflow_reachable_behavior_ids`: `b`'s slice contains
        // `a`'s slice plus the additional adds. A wrapper that
        // ignored the supplied root would return the same witness
        // count for both — this assertion catches that regression.
        let witnesses_a = match analyze_complexity(&dag, root_a) {
            DimensionReport::DimensionOk { witnesses, .. } => witnesses,
            other => panic!("expected Ok at root_a, got {other:?}"),
        };
        let witnesses_b = match analyze_complexity(&dag, root_b) {
            DimensionReport::DimensionOk { witnesses, .. } => witnesses,
            other => panic!("expected Ok at root_b, got {other:?}"),
        };
        assert!(
            witnesses_a.len() < witnesses_b.len(),
            "root `a`'s reachable spine ({}) must be strictly smaller than root `b`'s ({}); \
             a wrapper that ignored the supplied root would return equal counts here",
            witnesses_a.len(),
            witnesses_b.len(),
        );
    }

    /// E7 §test 1 (cross-check): `analyze_complexity.composed` matches
    /// the lens authority `symbolic_cost_of` at the workflow root.
    /// Confirms the wrapper preserves the lens contract.
    #[test]
    fn analyze_complexity_composed_matches_lens_at_workflow_root() {
        let dag = compile_to_dag("let z = 5 + 6", "e7_int_lens.v3").expect("compiles");
        let root = find_bind_root(&dag, "z");

        let SymbolicCostLookup::Hit(lens_cost) = symbolic_cost_of(&dag, &find_bind_port(&dag, "z"))
        else {
            panic!("lens authority must produce a Hit on a well-typed program");
        };

        let DimensionReport::DimensionOk {
            composed,
            dimension_name,
            ..
        } = analyze_complexity(&dag, root)
        else {
            panic!("expected DimensionOk on a well-typed program");
        };
        assert_eq!(composed, lens_cost);
        assert_eq!(dimension_name, "symbolic_cost");
    }

    /// E7 §test 6 (integration form, success-path scope): the public
    /// API preserves the typed `Witness<SymbolicCost>` envelope on
    /// the `DimensionOk` arm — every witness is a typed enum
    /// inhabitant the test pattern-matches without inspecting the
    /// `reason` string. Typed-diagnostic discipline on the `Fail`
    /// arm is pinned by the in-module `analyze_complexity_tests`
    /// (`src/v3/compiler/src/dimension.rs`), which constructs
    /// ghost-port DAGs via crate-private builders that the public
    /// API surface here cannot reach (the surface compiler always
    /// wires its outputs). This integration test confirms the public
    /// API does not lose the typed envelope on the success path.
    #[test]
    fn analyze_complexity_public_api_preserves_typed_witness_envelope_on_ok() {
        let dag = compile_to_dag("let w = 7 + 8", "e7_int_typed.v3").expect("compiles");
        let root = find_bind_root(&dag, "w");

        let DimensionReport::DimensionOk { witnesses, .. } = analyze_complexity(&dag, root) else {
            panic!("expected DimensionOk for well-typed program");
        };
        // Each witness is a typed enum inhabitant; the test
        // pattern-matches without ever inspecting the `reason` string
        // (which is only present on the `Violates` arm anyway).
        for witness in &witnesses {
            // Exhaustive match on the typed Witness enum; both arms
            // are acceptable inhabitants. `Violates.reason` is never
            // string-parsed by this test.
            match witness {
                v3_compiler::Witness::Inhabits(_) => {}
                v3_compiler::Witness::Violates { at, .. } => {
                    // Only structural assertions on `at`; never on
                    // `reason`.
                    let _ = at;
                }
            }
        }
    }
}

mod parse_stage4_prep {
    use std::fs;
    use std::path::{Path, PathBuf};

    use v3_compiler::{parse_for_test, tokenize_for_test};

    // SG-2 parser staging: corpus manifest snapshots the runtime parse surface
    // (`parse_generated.rs` = `parse_surface.dag` carriers + `parse_parser_body.txt` algorithm)
    // for structural parity — not a claim of full `.dag` parse-rule authority.
    const PARSE_CORPUS_MANIFEST: &str = include_str!("integration/parse_corpus_manifest.txt");

    fn compiler_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn repo_root() -> PathBuf {
        compiler_root()
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("src/v3/compiler has repo-root ancestors")
            .to_path_buf()
    }

    fn collect_rel_paths(dir: &Path, rel_prefix: &str, ext: &str) -> Vec<String> {
        let mut entries: Vec<String> = fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("read_dir {} failed: {err}", dir.display()))
            .map(|entry| {
                entry.unwrap_or_else(|err| panic!("read_dir entry {} failed: {err}", dir.display()))
            })
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(ext))
            .map(|path| {
                let file_name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .expect("utf-8 fixture name");
                format!("{rel_prefix}/{file_name}")
            })
            .collect();
        entries.sort();
        entries
    }

    fn parse_corpus_paths() -> Vec<String> {
        let compiler_root = compiler_root();
        // Keep the `dsl/std` subset aligned with the bootstrap fixtures
        // loaded in `bootstrap_regen_fresh.rs::std_fixtures`; this prep
        // harness is a snapshot of the incumbent parser over that
        // bootstrap-facing corpus, not a claim that every
        // `dsl/std/*.dag` file parses under v3 today.
        let mut paths = vec![
            "dsl/std/algebra.dag".to_string(),
            "dsl/std/bit.dag".to_string(),
            "dsl/std/float.dag".to_string(),
            "dsl/std/integer.dag".to_string(),
            "dsl/std/logic.dag".to_string(),
            "dsl/std/magnitude.dag".to_string(),
            "dsl/std/nat.dag".to_string(),
            "dsl/std/rational.dag".to_string(),
            "dsl/std/string_type.dag".to_string(),
            "dsl/std/types.dag".to_string(),
        ];
        paths.extend(collect_rel_paths(
            &compiler_root.join("../std"),
            "src/v3/std",
            "dag",
        ));
        paths.extend(collect_rel_paths(
            &compiler_root.join("../spec"),
            "src/v3/spec",
            "dag",
        ));
        paths.extend(collect_rel_paths(&compiler_root, "src/v3/compiler", "dag"));
        paths.extend(collect_rel_paths(
            &compiler_root.join("tests/four_fixture_pressure"),
            "src/v3/compiler/tests/four_fixture_pressure",
            "v3",
        ));
        paths.sort();
        paths
    }

    fn fnv1a64(bytes: &[u8]) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    fn render_surface(path: &str) -> (usize, usize, u64) {
        let source = fs::read_to_string(repo_root().join(path))
            .unwrap_or_else(|err| panic!("read fixture `{path}` failed: {err}"));
        let tokens = tokenize_for_test(&source, path)
            .unwrap_or_else(|diag| panic!("tokenize `{path}` failed: {diag:?}"));
        let surface = parse_for_test(&tokens, path)
            .unwrap_or_else(|diag| panic!("parse `{path}` failed: {diag:?}"));
        let rendered = format!("{surface:#?}");
        (
            surface.items.len(),
            rendered.len(),
            fnv1a64(rendered.as_bytes()),
        )
    }

    fn render_manifest() -> String {
        let mut rendered = String::from(
            "# AUTO-GENERATED by `cargo test -p v3-compiler refresh_handwritten_parse_snapshot_manifest -- --ignored`\n\
             # SG-2 parser staging: snapshots generated-parser surface output over the parse corpus.\n\
             # path\\titems\\tdebug_bytes\\tfnv1a64\n",
        );
        for path in parse_corpus_paths() {
            let (items, debug_bytes, hash) = render_surface(&path);
            rendered.push_str(&format!("{path}\t{items}\t{debug_bytes}\t{hash:016x}\n"));
        }
        rendered
    }

    fn parse_file(source: &str, name: &str) {
        let tokens = tokenize_for_test(source, name)
            .unwrap_or_else(|diag| panic!("tokenize {name} failed: {diag:?}"));
        let _module = parse_for_test(&tokens, name)
            .unwrap_or_else(|diag| panic!("parse {name} failed: {diag:?}"));
    }

    #[test]
    fn handwritten_parse_snapshot_matches_manifest() {
        assert_eq!(render_manifest(), PARSE_CORPUS_MANIFEST);
    }

    #[test]
    #[ignore = "helper to refresh parse_corpus_manifest.txt after intentional handwritten parser changes"]
    fn refresh_handwritten_parse_snapshot_manifest() {
        let manifest_path = compiler_root()
            .join("tests")
            .join("integration")
            .join("parse_corpus_manifest.txt");
        fs::write(&manifest_path, render_manifest())
            .unwrap_or_else(|err| panic!("write {} failed: {err}", manifest_path.display()));
    }

    #[test]
    fn handwritten_parser_accepts_logic_dag() {
        parse_file(
            include_str!("../../../../dsl/std/logic.dag"),
            "dsl/std/logic.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_bit_dag() {
        parse_file(
            include_str!("../../../../dsl/std/bit.dag"),
            "dsl/std/bit.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_algebra_dag() {
        parse_file(
            include_str!("../../../../dsl/std/algebra.dag"),
            "dsl/std/algebra.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_types_dag() {
        parse_file(
            include_str!("../../../../dsl/std/types.dag"),
            "dsl/std/types.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_integer_dag() {
        parse_file(
            include_str!("../../../../dsl/std/integer.dag"),
            "dsl/std/integer.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_rational_dag() {
        parse_file(
            include_str!("../../../../dsl/std/rational.dag"),
            "dsl/std/rational.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_magnitude_dag() {
        parse_file(
            include_str!("../../../../dsl/std/magnitude.dag"),
            "dsl/std/magnitude.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_nat_dag() {
        parse_file(
            include_str!("../../../../dsl/std/nat.dag"),
            "dsl/std/nat.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_float_dag() {
        parse_file(
            include_str!("../../../../dsl/std/float.dag"),
            "dsl/std/float.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_string_type_dag() {
        parse_file(
            include_str!("../../../../dsl/std/string_type.dag"),
            "dsl/std/string_type.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_v3_list_dag() {
        parse_file(include_str!("../../std/list.dag"), "src/v3/std/list.dag");
    }

    #[test]
    fn handwritten_parser_accepts_v3_verification_dag() {
        parse_file(
            include_str!("../../std/verification.dag"),
            "src/v3/std/verification.dag",
        );
    }

    #[test]
    fn handwritten_parser_accepts_v3_effects_dag() {
        parse_file(
            include_str!("../../std/effects.dag"),
            "src/v3/std/effects.dag",
        );
    }
}
