//! Consolidated v3-compiler integration test binary.
//!
//! **Why one binary.** Rust integration tests default to one binary per
//! `tests/*.rs` file. Each binary pays a separate bootstrap + compile cost
//! on cold runners (bootstrap alone is ~200ms; the in-memory
//! `cached_compile_to_dag` helper only amortizes within-binary). Hoisting
//! every test file into this single module tree means:
//!
//! - **One bootstrap per cargo test run** — shared across every test.
//! - **Cross-test cache hits** — two tests compiling `"let x = 1"` with
//!   different file markers now share a key (per-binary OnceLock is now
//!   also per-run).
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
#[path = "integration/thesis_parallelism_test.rs"]
mod thesis_parallelism_test;
#[path = "integration/thesis_validation_test.rs"]
mod thesis_validation_test;
