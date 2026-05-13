//! **Layer:** integration
//!
//! §1.8 / gate #58 `apply_lens_self_application_demonstrated`: `EnforcedApplication` with
//! `timing_enforceable` on modeled CI `Workflow` data (`timing_lens.dag` + host consumer in
//! `enforced_lens_application.rs`).
//!
//! The pass witness is authored in `src/v3/std/t_ci_workflow_as_data_demo.dag` (typed section
//! witness `gate_58_modeled_ci_timing_measurement` + `gate_58_apply_lens_self_application_pass`) and
//! is compiled into the committed PB-1 bootstrap snapshot. Fail-closed budget arithmetic is
//! unit-tested in `enforced_lens_application.rs`.

use v3_compiler::generated_full_bootstrap_dag;

#[test]
fn apply_lens_self_application_demonstrated_bootstrap_receipt() {
    let dag = generated_full_bootstrap_dag();
    assert!(
        dag.declaration_by_name("gate_58_apply_lens_self_application_pass")
            .is_some(),
        "bootstrap must include gate #58 `EnforcedApplication<TimingMeasurement, TimingBudget>` witness"
    );
    assert!(
        dag.declaration_by_name("modeled_gunbc_ci_workflow")
            .is_some(),
        "bootstrap must include modeled CI `Workflow` (`modeled_gunbc_ci_workflow`)"
    );
    assert!(
        dag.declaration_by_name("timing_enforceable").is_some(),
        "bootstrap must include `timing_enforceable` (`timing_lens.dag`)"
    );
    assert!(
        dag.declaration_by_name("gate_58_modeled_ci_timing_measurement")
            .is_some(),
        "bootstrap must include gate #58 timing witness row"
    );
    assert!(
        dag.diagnostics().is_empty(),
        "unexpected bootstrap diagnostics: {:?}",
        dag.diagnostics()
    );
}
