//! THE GROUNDED FREEMONOID CARRIER IS WRAPPED ONCE, AND THE PRE-FIX SHAPE IS THE RED CONTROL.
//!
//! `Empty {}` for a seed-host container emits the grounded SHARED value
//! (`v1.compiler.emit_rust` `emit_freemonoid_empty_rc_value`, `Rc::new(vec![])`). The two
//! record-literal sites then applied the sharing constructor a second time because the carrier's
//! parent enum is a shared type, so `Cons { head: x, tail: Empty {} }` rendered
//! `(*Rc::new(Rc::new(vec![]))).clone()` — which rustc refuses with `Rc<Vector<T>>` where
//! `Vector<T>` was expected. Measured on the required-v2-native lane's emitted closure: 43 of its
//! 152 rustc errors were this one shape, every one a roster data row of the form
//! `Cons { head: "...", tail: Empty {} }` in `v2.workflow.floor_expected_red` /
//! `v2.workflow.floor_route_gap` — the admission's own rosters, which the lane must compile.
//!
//! WHERE THIS EXECUTES, STATED RATHER THAN ASSUMED. No CI step runs a Rust unit test today
//! (`gunbc.rung_drop` `rust_unit_tests_off_the_merge_path`); the clippy `--all-targets` step
//! COMPILES this target, so a shape change that stops type-checking is caught, and the assertions
//! below execute under any local `cargo test`. The executing evidence on a REQUIRED lane is the
//! required-v2-native job's own cargo build of the emitted closure, which refused before this wall
//! and compiles after it.

use v1_compiler::cli_run::compile_entry_emission;
use v1_compiler::v1_compiler_artifact::RenderTarget;

use crate::helpers::workspace_root;

const PROBE_ENTRY: &str = "src/v1/tests/fixtures/grounded_shared_carrier/probe.dag";

fn emit_probe() -> String {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    // The fixture's own directory is a source root beside the corpus roots: the entry must be
    // keyed into the module graph, and it deliberately lives outside dag/ and src/v2/ so no lens,
    // census or floor discovery enrols a probe that exists only to be read as emitted text.
    let roots = vec![
        "dag".to_string(),
        "src/v2".to_string(),
        "src/v1/tests/fixtures/grounded_shared_carrier".to_string(),
    ];
    let run = compile_entry_emission(&roots, PROBE_ENTRY, true, RenderTarget::Rust);
    let emission = run.emissions.first().unwrap_or_else(|| {
        panic!(
            "the probe entry must reach emission; disposition: {}",
            run.measurement_line("grounded-shared-carrier-probe")
        )
    });
    emission
        .result
        .files
        .iter()
        .find(|f| f.path.contains("test_fixture_grounded_shared_carrier"))
        .unwrap_or_else(|| {
            panic!(
                "the probe module must emit a file; emitted paths: {:?}",
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

// ONE TEST, BOTH ASSERTIONS, because the emission needs the workspace as its working directory and
// `set_current_dir` is process-global: two tests asserting over the same emission would race each
// other in the same process rather than measure anything twice.
#[test]
fn the_grounded_empty_is_shared_once_and_never_twice() {
    let emitted = emit_probe();
    assert!(
        emitted.contains("(*Rc::new(vec![])).clone()"),
        "Cons must deref the carrier the grounded empty already shares; emitted:\n{emitted}"
    );
    assert!(
        !emitted.contains("Rc::new(Rc::new(vec![]))"),
        "RED CONTROL: the record-literal sites must not wrap the already-shared grounded empty a \
         second time - that shape is what rustc refuses; emitted:\n{emitted}"
    );
}
