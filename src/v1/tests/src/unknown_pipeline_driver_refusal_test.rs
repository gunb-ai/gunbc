//! AN UNRECOGNISED PIPELINE-DRIVER READING REFUSES, AND THE PRE-FIX SHAPE IS THE RED CONTROL.
//!
//! `std.compiler_entry` declares exactly three drivers. `v1.compiler.emit_rust` `emit_main_rs`
//! tested two of them by name and treated every other reading as the third, so a reading that
//! matched none rendered the direct-ingest main for a corpus declaring the source-root Eval driver
//! — a silent wrong answer, rostered as
//! `gunbc.recurring_failure_mode.closed_variant_partition_read_as_a_default_arm`.
//!
//! The fixture is an ORDINARY ACCEPTED PROGRAM: the driver is spelled with its declaring module's
//! qualifier, which resolves, and which the arm selection — reading the authored name off the
//! source span — cannot match. Measured against a seed WITHOUT the wall, this same fixture emits
//! the direct-ingest main (`compile_dag_source_to_target_text`, the translator arm); with the wall
//! it emits a located `compile_error!` naming the module and the reading.
//!
//! The emission runs at the same precedence `gunbc compile` uses: under primary precedence this
//! fixture's qualified spelling draws a blocking diagnostic and the transaction refuses before any
//! main is rendered, which would measure that refusal rather than the arm selection.

use v1_compiler::cli_run::compile_entry_emission;
use v1_compiler::v1_compiler_artifact::RenderTarget;

use crate::helpers::workspace_root;

const PROBE_ENTRY: &str = "fixtures/unknown_pipeline_driver/probe.dag";

fn emitted_main() -> String {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let roots = vec![
        "dag".to_string(),
        "src/v2".to_string(),
        "fixtures/unknown_pipeline_driver".to_string(),
    ];
    let run = compile_entry_emission(&roots, PROBE_ENTRY, false, RenderTarget::Rust);
    let emission = run.emissions.first().unwrap_or_else(|| {
        panic!(
            "the probe entry must reach emission; disposition: {}",
            run.measurement_line("unknown-pipeline-driver-probe")
        )
    });
    emission
        .result
        .files
        .iter()
        .find(|f| f.path.ends_with("main.rs"))
        .unwrap_or_else(|| {
            panic!(
                "the emitted crate must carry a main.rs; {} emitted paths: {:?}",
                format!("{:?}", run.disposition),
                emission
                    .result
                    .files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect::<Vec<_>>()
            )
        })
        .content
        .clone()
}

#[test]
fn an_unrecognised_driver_reading_refuses_and_names_what_it_read() {
    let main_rs = emitted_main();
    assert!(
        main_rs.contains("compile_error!")
            && main_rs.contains("does not recognise")
            && main_rs.contains("std.compiler_entry.SourceRootEvalDriver"),
        "an unrecognised driver reading must refuse and NAME the reading, so the defect is \
         located rather than guessed; emitted main.rs:\n{main_rs}"
    );
    assert!(
        !main_rs.contains("compile_dag_source_to_target_text"),
        "RED CONTROL: before the wall this fixture rendered the DIRECT-INGEST main -- a translator \
         for a corpus declaring the source-root Eval driver, with zero diagnostics. An arm must \
         never be selected as the residue of two string comparisons; emitted main.rs:\n{main_rs}"
    );
}
