//! **Layer:** integration
//!
//! R3 §1.8 gate **#95** (`opt_in_iteration_parallelism_via_lens_application_demonstrated`):
//! opt-in iteration parallelism surfaced through `parallelism_enforceable` /
//! `v3_compiler::parallelism_iteration_opt_in_enforcement_violates(&dag, indicator)` together with Lane-2
//! `v3_compiler::loop_iteration_parallel_emission_indicator` (same contract as second-batch
//! auto-loop receipts).
//!
//! This binary intentionally pins the **exported public bridge** Lane‑2 indicator ↔ opt‑in violates
//! predicate (`compile_to_dag` staged harness + `workflow_lane2_subject`, no synthetic declaration
//! injection). Mirrors gate #58 splitting: integration covers bootstrap/pass witnesses where the
//! carrier is authored in checked-in substrate; Gate #95 executable `apply_lens`/`.dag` authoring of
//! `EnforcedApplication`/`NodeScope` remains deferred (`../fixtures/` banner), while the **`check_enforced_lens_applications`**
//! consumer path (**`EnforcedApplication` row + coupling guard + indicator read**) lives in crate
//! `#[cfg(test)]` (`gate_95_parallelism_iteration_enforcement_tests` in `enforced_lens_application.rs`,
//! synthetic `push_declaration` injection — Gate #94-style internal receipt).
//!
//! Fixture companion: `../fixtures/t_las_parallelism_iteration_gate95_fixture.dag`.
//! Authority: `docs/design-lens-application-surface.md` §4.4.

use crate::common::cached_compile_any;
use v3_compiler::{
    compile_to_dag, loop_iteration_parallel_emission_indicator,
    parallelism_iteration_opt_in_enforcement_violates,
};

const FIXTURE: &str = include_str!("../fixtures/t_las_parallelism_iteration_gate95_fixture.dag");
const FIXTURE_FILE_NAME: &str = "t_las_parallelism_iteration_gate95_fixture.dag";

#[test]
fn gate_95_fixture_pins_parallelism_enforceable_carrier() {
    let dag = cached_compile_any(FIXTURE, FIXTURE_FILE_NAME);
    assert!(
        dag.declaration_by_name("parallelism_enforceable").is_some(),
        "fixture must reference `parallelism_enforceable` from `parallelism.dag`"
    );
}

#[test]
fn gate_95_opt_in_iteration_parallelism_via_lens_application_demonstrated_indicator_bridge() {
    let run = |witness_file: &'static str,
               directive: &'static str,
               indicator: i64,
               expect_violation: bool| {
        let source = format!(
            "// gunbc::r3_free_consequences::lane2_loop_witness: {directive}\n\
             import lenses.parallelism {{ parallelism_enforceable }}\n\
             fn gate95_integration_probe() -> Int = 0\n"
        );
        let dag = compile_to_dag(source.as_str(), witness_file)
            .expect("compile staged lane2 loop harness");
        let subject = dag
            .workflow_lane2_subject()
            .expect("workflow shell Bind for Lane-2 registration");
        let observed = loop_iteration_parallel_emission_indicator(&dag, subject);
        assert_eq!(observed, indicator);
        assert_eq!(
            parallelism_iteration_opt_in_enforcement_violates(&dag, observed),
            expect_violation
        );
    };

    run(
        "t_las_parallelism_gate95_integration_read_only.v3",
        "read_only",
        1,
        false,
    );
    run(
        "t_las_parallelism_gate95_integration_upsert.v3",
        "upsert_dependent",
        0,
        true,
    );
}
