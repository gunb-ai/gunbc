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

use std::fs;
use std::path::PathBuf;

use crate::common::cached_compile_outcome;
use crate::common::CachedCompileOutcome;

use v3_compiler::Diagnostic;
use v3_compiler::{
    check_enforced_lens_applications, gate_58_test_raise_modeled_ci_timing_measurement_duration_ns,
    generated_full_bootstrap_dag,
};

const VIOLATION_FIXTURE_REL: &str =
    "src/v3/compiler/tests/fixtures/t_gate_58_timing_enforcement_budget_violation.dag";
const VIOLATION_FILE_NAME: &str = "t_gate_58_timing_enforcement_budget_violation.dag";

#[test]
fn apply_lens_self_application_demonstrated_bootstrap_receipt() {
    let mut dag = generated_full_bootstrap_dag();
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

    check_enforced_lens_applications(&mut dag);
    assert!(
        dag.diagnostics().is_empty(),
        "post-infer timing `EnforcedApplication` check must stay clean on the gate #58 pass witness; got {:?}",
        dag.diagnostics()
    );

    // Executable receipt (INVARIANTS P3 / gate #58): perturb the lowered witness above the pass
    // budget and prove [`check_enforced_lens_applications`] evaluates timing enforcement on the
    // live bootstrap graph (not merely declaration presence + a no-op pass).
    const PASS_BUDGET_MAX_NS: u64 = 1_000_000_000;
    const OVER_BUDGET_NS: u64 = PASS_BUDGET_MAX_NS + 1;
    gate_58_test_raise_modeled_ci_timing_measurement_duration_ns(&mut dag, OVER_BUDGET_NS)
        .expect("raise gate #58 modeled timing measurement over pass budget");
    check_enforced_lens_applications(&mut dag);
    let violation_receipts: Vec<_> = dag
        .diagnostics()
        .iter()
        .map(|(_, d)| d)
        .filter(|d| {
            d.layer1_kind_label() == "ParseError"
                && d.message().contains("timing budget ceiling")
                && d.message()
                    .contains(&format!("max_ns={PASS_BUDGET_MAX_NS}"))
                && d.message()
                    .contains(&format!("usage_max_ns={OVER_BUDGET_NS}"))
        })
        .collect();
    assert_eq!(
        violation_receipts.len(),
        1,
        "expected exactly one timing budget violation ParseError after witness perturbation; diagnostics={:?}",
        dag.diagnostics()
            .iter()
            .map(|(_, d)| d.message())
            .collect::<Vec<_>>()
    );
}

#[test]
fn apply_lens_self_application_timing_enforcement_executable_budget_violation() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(VIOLATION_FIXTURE_REL);
    let source =
        fs::read_to_string(&path).expect("read gate #58 timing enforcement violation fixture");
    std::thread::Builder::new()
        .name("t-gate-58-timing-enforcement-violation".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let CachedCompileOutcome::Semantic(dag) =
                cached_compile_outcome(&source, VIOLATION_FILE_NAME)
            else {
                panic!(
                    "fixture must fail closed with semantic diagnostics (timing budget exceeded)"
                );
            };
            let receipts: Vec<_> = dag
                .diagnostics()
                .iter()
                .map(|(_, d)| (d.layer1_kind_label().to_string(), d.message()))
                .collect();
            let ok = receipts.iter().any(|(kind, msg)| {
                kind == "ParseError"
                    && msg.contains("lens enforcement violation")
                    && msg.contains("timing budget ceiling")
                    && msg.contains("999")
            });
            assert!(
                ok,
                "expected timing lens enforcement ParseError with budget ceiling + observed usage; got {receipts:?}"
            );
        })
        .expect("spawn gate #58 enforcement compile")
        .join()
        .expect("gate #58 enforcement compile thread panicked");
}
