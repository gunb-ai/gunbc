//! `sort_by` COMPUTES EACH KEY ONCE, AND THE PRE-CHANGE SHAPE IS THE RED CONTROL.
//!
//! `v1.compiler.emit_rust` `emit_rust_sort_by_method_call` used to emit
//! `__sorted.sort_by(|a, b| { let __ka = (key)(a); let __kb = (key)(b); canonical_key_cmp(&__ka, &__kb) })`,
//! running the supplied selector twice per comparison. It now decorates each element with its key once, stable-sorts on the stored key
//! with the same `canonical_key_cmp`, and undecorates, so multiplicity and tie order are those of
//! the one stable sort.
//!
//! This reads the emitted text of the probe fixture (the pattern of
//! `grounded_shared_carrier_wrap_test`); it asserts the lowering's route, not ordering behavior.
//! Ordering behavior is reached by the self-host closure that `v2-bootstrap` compiles and runs
//! through the generated `sort_by` sites. No required lane runs this Rust unit test
//! (`gunbc.rung_drop` `rust_unit_tests_off_the_merge_path`); clippy `--all-targets` compiles it.

use v1_compiler::cli_run::compile_entry_emission;
use v1_compiler::v1_compiler_artifact::RenderTarget;

use crate::helpers::workspace_root;

const PROBE_ENTRY: &str = "fixtures/sort_by_decorate_once/probe.dag";

fn emit_probe() -> String {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let roots = vec![
        "dag".to_string(),
        "src/v2".to_string(),
        "fixtures/sort_by_decorate_once".to_string(),
    ];
    let run = compile_entry_emission(&roots, PROBE_ENTRY, true, RenderTarget::Rust);
    let emission = run.emissions.first().unwrap_or_else(|| {
        panic!(
            "the probe entry must reach emission; disposition: {}",
            run.measurement_line("sort-by-decorate-once-probe")
        )
    });
    emission
        .result
        .files
        .iter()
        .find(|f| f.path.contains("test_fixture_sort_by_decorate_once"))
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

// ONE TEST: the emission needs the workspace as cwd and `set_current_dir` is process-global.
#[test]
fn sort_by_decorates_once_sorts_the_stored_key_and_undecorates() {
    let emitted = emit_probe();
    let sites = emitted.matches("__keyed").count();
    assert!(
        sites > 0,
        "the decorate/sort/undecorate lowering must be emitted; emitted:\n{emitted}"
    );
    for needle in [
        "let mut __keyed: Vec<_> =",
        "(__k, __e)",
        "__keyed.sort_by(|__x, __y| v1_rt::canonical_key_cmp(&__x.0, &__y.0))",
        "__keyed.into_iter().map(|(_, __e)| __e).collect::<Vec<_>>()",
    ] {
        assert!(
            emitted.contains(needle),
            "missing `{needle}` in the sort_by lowering; emitted:\n{emitted}"
        );
    }
    for old in [
        "let __ka =",
        "let __kb =",
        "canonical_key_cmp(&__ka, &__kb)",
    ] {
        assert!(
            !emitted.contains(old),
            "RED CONTROL: `{old}` is the pre-change shape, which applies the selector inside the \
             comparator (twice per comparison); emitted:\n{emitted}"
        );
    }
}
