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
//! **Module discipline.** Each file under `tests/integration/*.rs` is a
//! sibling module at this crate root, reached via `#[path]` because Rust's
//! default module resolution for a crate-root file looks in the containing
//! directory (`tests/`) rather than a same-named subdirectory. Shared
//! helpers live under `tests/integration/common/`. Inside a test module,
//! `use crate::common::…` reaches those helpers; there is no per-file
//! `mod common;` declaration.

#[macro_use]
#[path = "integration/common/mod.rs"]
mod common;

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
#[path = "integration/m0_acceptance.rs"]
mod m0_acceptance;
#[path = "integration/m1_3_emit_go_test.rs"]
mod m1_3_emit_go_test;
#[path = "integration/m1_3_emit_rust_test.rs"]
mod m1_3_emit_rust_test;
#[path = "integration/m1_3_lens_cost_test.rs"]
mod m1_3_lens_cost_test;
#[path = "integration/m1_3_lens_unused_parameters_test.rs"]
mod m1_3_lens_unused_parameters_test;
#[path = "integration/m1_4_emit_python_test.rs"]
mod m1_4_emit_python_test;
#[path = "integration/m1_5_testgen_test.rs"]
mod m1_5_testgen_test;
#[path = "integration/m1_5_verification_test.rs"]
mod m1_5_verification_test;
#[path = "integration/m1_fn_external_body_reconciliation_test.rs"]
mod m1_fn_external_body_reconciliation_test;
#[path = "integration/m1_lens_structural_resolution_test.rs"]
mod m1_lens_structural_resolution_test;
#[path = "integration/m1_substrate_test.rs"]
mod m1_substrate_test;
#[path = "integration/m2_emit_multi_field_struct_variant_test.rs"]
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
#[path = "integration/m2_substrate_inhabitance_test.rs"]
mod m2_substrate_inhabitance_test;
#[path = "integration/pipe_desugar.rs"]
mod pipe_desugar;
#[path = "integration/real_stdlib_parse_smoke.rs"]
mod real_stdlib_parse_smoke;
#[path = "integration/sg0_census_test.rs"]
mod sg0_census_test;
#[path = "integration/sg4_prep_infer_helpers_freshness_test.rs"]
mod sg4_prep_infer_helpers_freshness_test;
#[path = "integration/thesis_parallelism_test.rs"]
mod thesis_parallelism_test;
#[path = "integration/thesis_validation_test.rs"]
mod thesis_validation_test;

mod parse_corpus {
    use std::fs;
    use std::path::{Path, PathBuf};

    use v3_compiler::{parse_for_test, tokenize_for_test};

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
            .filter_map(|entry| entry.ok())
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
        let mut paths = vec![
            "dsl/gunbc/compiler.dag".to_string(),
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
            "# AUTO-GENERATED by `cargo test -p v3-compiler refresh_parse_corpus_manifest -- --ignored`\n\
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
    fn current_parse_corpus_matches_manifest() {
        assert_eq!(render_manifest(), PARSE_CORPUS_MANIFEST);
    }

    #[test]
    #[ignore = "helper to refresh parse_corpus_manifest.txt after intentional parser changes"]
    fn refresh_parse_corpus_manifest() {
        let manifest_path = compiler_root()
            .join("tests")
            .join("integration")
            .join("parse_corpus_manifest.txt");
        fs::write(&manifest_path, render_manifest())
            .unwrap_or_else(|err| panic!("write {} failed: {err}", manifest_path.display()));
    }

    #[test]
    fn logic_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/logic.dag"),
            "dsl/std/logic.dag",
        );
    }

    #[test]
    fn bit_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/bit.dag"),
            "dsl/std/bit.dag",
        );
    }

    #[test]
    fn algebra_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/algebra.dag"),
            "dsl/std/algebra.dag",
        );
    }

    #[test]
    fn types_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/types.dag"),
            "dsl/std/types.dag",
        );
    }

    #[test]
    fn integer_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/integer.dag"),
            "dsl/std/integer.dag",
        );
    }

    #[test]
    fn float_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/float.dag"),
            "dsl/std/float.dag",
        );
    }

    #[test]
    fn string_type_dag_parses() {
        parse_file(
            include_str!("../../../../dsl/std/string_type.dag"),
            "dsl/std/string_type.dag",
        );
    }

    #[test]
    fn list_dag_parses() {
        parse_file(include_str!("../../std/list.dag"), "src/v3/std/list.dag");
    }

    #[test]
    fn verification_dag_parses() {
        parse_file(
            include_str!("../../std/verification.dag"),
            "src/v3/std/verification.dag",
        );
    }

    #[test]
    fn effects_dag_parses() {
        parse_file(
            include_str!("../../std/effects.dag"),
            "src/v3/std/effects.dag",
        );
    }
}
