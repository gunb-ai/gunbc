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

#[path = "integration/cementing/cementing_lens_registry_dispatch_test.rs"]
mod cementing_lens_registry_dispatch_test;
#[path = "integration/four_fixture_regression_test.rs"]
mod four_fixture_regression_test;
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
#[path = "integration/p0_std_render_repeat_string_test.rs"]
mod p0_std_render_repeat_string_test;
#[path = "integration/e_i_lane_induction_preflight_test.rs"]
mod e_i_lane_induction_preflight_test;
#[path = "integration/pb1_bootstrap_full_snapshot_test.rs"]
mod pb1_bootstrap_full_snapshot_test;
#[path = "integration/pb1_bootstrap_std_snapshot_test.rs"]
mod pb1_bootstrap_std_snapshot_test;
#[path = "integration/pipe_desugar.rs"]
mod pipe_desugar;
#[path = "integration/r1_manual_claim_gate_test.rs"]
mod r1_manual_claim_gate_test;
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
#[path = "integration/sg3_lower_authority_test.rs"]
mod sg3_lower_authority_test;
#[path = "integration/sg3_lower_parse_surface_stack_test.rs"]
mod sg3_lower_parse_surface_stack_test;
#[path = "integration/sg3_surface_reflection_consumer_test.rs"]
mod sg3_surface_reflection_consumer_test;
#[path = "integration/sg6_hand_authored_census_test.rs"]
mod sg6_hand_authored_census_test;
#[path = "integration/sg7_prep_variant_payload_freshness_test.rs"]
mod sg7_prep_variant_payload_freshness_test;
#[path = "integration/test_runner_test.rs"]
mod test_runner_test;
#[path = "integration/testgen_structural_coverage_gate_test.rs"]
mod testgen_structural_coverage_gate_test;
#[path = "integration/thesis_parallelism_test.rs"]
mod thesis_parallelism_test;
#[path = "integration/thesis_validation_test.rs"]
mod thesis_validation_test;

mod t_demo_fixture_test {
    //! **Layer:** integration

    use std::fs;
    use std::path::PathBuf;

    use v3_compiler::compile_to_dag;
    use v3_compiler::dag::Dag;
    use v3_compiler::test_runner::{ClaimResult, TestRunner};

    const FIXTURE: &str = "src/v3/compiler/tests/t_demo/t_demo_fixtures.dag";

    fn fixture_source() -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(FIXTURE);
        fs::read_to_string(path).expect("read T-Demo fixture skeleton")
    }

    fn compile_fixture(source: &str) -> Dag {
        compile_to_dag(source, FIXTURE).expect("T-Demo fixture skeleton compiles")
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
}

mod lane2_stage_2f_dimension_test {
    use v3_compiler::analyze_symbolic_cost_dimension;
    use v3_compiler::compile_to_dag;
    use v3_compiler::dag::{Behavior, Dag, PortId, TypeConnective};
    use v3_compiler::lens_cost_symbolic::{symbolic_cost_of, SymbolicCostLookup};

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
    fn no_authored_dimension_carrier_constants_in_bootstrap_stdlib() {
        let dag = Dag::new();
        let dimension_template = dag
            .declaration_by_name("Dimension")
            .expect("bootstrap loads Dimension")
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
            "no `data _: Dimension<_> = ...` values ship until class-5 bodies unlock the receipt"
        );
    }

    #[test]
    fn analyze_symbolic_cost_composed_matches_lens_at_workflow_root() {
        let dag = compile_to_dag("let x = 1 + 2", "lane2_2f_dim.v3").expect("compiles");
        let root = find_bind_root(&dag, "x");
        let report = analyze_symbolic_cost_dimension(&dag, root);
        let lens = match symbolic_cost_of(&dag, &find_bind_port(&dag, "x")) {
            SymbolicCostLookup::FoundCost { _0: cost } => cost,
            SymbolicCostLookup::MissingCost => panic!("expected FoundCost"),
        };
        assert_eq!(report.composed, lens);
        assert_eq!(report.dimension_name, "symbolic_cost");
        assert_eq!(report.witnesses.len(), dag.nodes().len());
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
        // Keep the `dsl/std` subset aligned with the seven bootstrap
        // fixtures loaded in `bootstrap.rs`; this prep harness is a
        // snapshot of the incumbent parser over that bootstrap-facing
        // corpus, not a claim that every `dsl/std/*.dag` file parses
        // under v3 today.
        let mut paths = vec![
            "dsl/std/algebra.dag".to_string(),
            "dsl/std/bit.dag".to_string(),
            "dsl/std/float.dag".to_string(),
            "dsl/std/integer.dag".to_string(),
            "dsl/std/logic.dag".to_string(),
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
