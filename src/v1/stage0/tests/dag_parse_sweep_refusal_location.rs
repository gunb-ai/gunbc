// THE PARSE SWEEP'S REFUSALS CARRY THE LOCATION THE DIAGNOSTIC ALREADY HELD.
//
// WHAT WENT WRONG (cost an hour). A refusal carries its span and `diagnostic_to_span` reads it,
// but `run_dag_parse_sweep` once rendered only path and message: a file with 52 refusals gave 52
// byte-identical sentences and no way to tell which line was at fault. A typed, LOCATED
// diagnostic rendered as untyped prose — the opposite of DESIGN section 5.
//
// The fixture's refusal is a parse error, located at a known line and column. It used to be an
// annotation-attachment refusal; those no longer exist (std.source_annotation attaches every
// comment to its gap), and the property under test is the printer's, not the refusal's.
//
// WHY THE SECOND TEST IS NOT REDUNDANT. The first test cannot tell a constant-position printer
// from a fixed one. The control moves the offending line down a known number of lines in an
// otherwise identical file and requires the reported line to move with it, so a hardcoded `1:1`
// fails here though it passes there.

use std::path::{Path, PathBuf};

// A FREE FUNCTION RATHER THAN AN RAII GUARD: a `struct` + two `impl` blocks would add four
// hand-authored Rust items to `src/v1`, every `impl` method UNCITABLE as a `DeclarationRef`
// (`std.decl_ref` has no `ImplMethod` field) — exactly the class `gunbc.seed_growth_admission`
// reports and asks authors not to grow. Cleanup runs on the way IN, which makes the run
// deterministic: a tree left by a killed run cannot decide the next one. No `tempfile`
// dev-dependency exists in this crate and this test does not earn one; the directory is named
// after the test, so the tests cannot collide.
fn scratch_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gunbc_parse_sweep_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch tree");
    dir
}

fn sweep_errors(dir: &Path, module: &str, body: &str) -> Vec<String> {
    let root = dir.join("probe_root");
    std::fs::create_dir_all(&root).expect("probe root");
    std::fs::write(root.join("probe.dag"), body).expect("probe source");
    match v1_compiler::cli_run::run_dag_parse_sweep(dir, &["probe_root"]) {
        Ok(sweep) => panic!(
            "{module}: sweep reported {} clean file(s); the fixture must refuse",
            sweep.parse_clean
        ),
        Err(errors) => errors,
    }
}

// The fixture's refusal is a real one: a stray `)` at the start of a line is no module item.
fn fixture(leading_blank_lines: usize) -> String {
    format!(
        "module probe_root.probe\n\ndata probe: Bool = true\n{}\n) stray\n",
        "\n".repeat(leading_blank_lines)
    )
}

#[test]
fn parse_sweep_refusal_names_the_offending_line() {
    let dir = scratch_root("located");
    let errors = sweep_errors(&dir, "located", &fixture(0));

    assert_eq!(
        errors.len(),
        1,
        "expected exactly one refusal, got {errors:?}"
    );
    // The stray `)` is on line 5 of `fixture(0)`: module, blank, data, blank, stray.
    assert!(
        errors[0].contains("probe.dag:5:1:"),
        "the refusal must name the offending line and column, got: {}",
        errors[0]
    );
}

#[test]
fn parse_sweep_refusal_location_follows_the_offending_line() {
    let dir = scratch_root("moved");
    let errors = sweep_errors(&dir, "moved", &fixture(7));

    assert_eq!(
        errors.len(),
        1,
        "expected exactly one refusal, got {errors:?}"
    );
    // Seven blank lines inserted above it, so the same stray line is now on line 12. A printer
    // emitting a constant position reports 5 here and fails.
    assert!(
        errors[0].contains("probe.dag:12:1:"),
        "the reported line must move with the offending line, got: {}",
        errors[0]
    );
}

// Out-of-closure body grammaticality belongs to the parse sweep, not to an entry compile.
// Pair A: the entry's dependency closure compiles while a sibling file's body is malformed.
// Pair B: the same population under `run_dag_parse_sweep` refuses at that body.
#[test]
fn out_of_closure_malformed_body_is_owned_by_the_parse_sweep() {
    use std::rc::Rc;
    use v1_compiler::v1_compiler_compile::{
        compile_to_resolved_with_options, default_compile_pipeline_options, SourceFile,
    };

    let entry = "module probe_root.entry\n\ndata ok: Bool = true\n";
    let outsider = "module probe_root.outsider\n\nfn body_probe(x: Int) -> Int { let = = = 9 }\n";

    let options = {
        let mut opts = (*default_compile_pipeline_options()).clone();
        opts.census_only_sources = Rc::new(
            vec![Rc::new(SourceFile {
                path: "outsider.dag".to_string(),
                content: outsider.to_string(),
            })]
            .into(),
        );
        Rc::new(opts)
    };
    let resolved = compile_to_resolved_with_options(
        Rc::new(
            vec![Rc::new(SourceFile {
                path: "entry.dag".to_string(),
                content: entry.to_string(),
            })]
            .into(),
        ),
        options,
    );
    let hard = resolved
        .diagnostics
        .iter()
        .any(|d| v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone()));
    assert!(
        !hard,
        "an entry compile must not refuse an out-of-closure body parse: {:?}",
        resolved
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );

    let dir = scratch_root("body_probe_authority");
    let root = dir.join("probe_root");
    std::fs::create_dir_all(&root).expect("probe root");
    std::fs::write(root.join("entry.dag"), entry).expect("entry");
    std::fs::write(root.join("outsider.dag"), outsider).expect("outsider");
    match v1_compiler::cli_run::run_dag_parse_sweep(&dir, &["probe_root"]) {
        Ok(sweep) => panic!(
            "parse sweep must refuse the malformed outsider body; parsed {} clean file(s)",
            sweep.parse_clean
        ),
        Err(errors) => {
            assert!(
                errors.iter().any(|e| e.contains("expected name, found Eq")
                    || e.contains("outsider.dag")),
                "sweep must locate the outsider body parse, got {errors:?}"
            );
        }
    }
}
