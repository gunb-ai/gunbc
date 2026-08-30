// THE PARSE SWEEP'S REFUSALS CARRY THE LOCATION THE DIAGNOSTIC ALREADY HELD.
//
// WHAT WENT WRONG (cost an hour). `AnnotationAttachmentRefusal` carries `origin: SourceSpan`
// and `diagnostic_to_span` reads it, but `run_dag_parse_sweep` rendered only path and message:
// a file with 52 unattachable annotations gave 52 byte-identical sentences and no way to tell
// which `//` line was at fault. A typed, LOCATED diagnostic rendered as untyped prose — the
// opposite of DESIGN section 5.
//
// ONE DEFECT, THREE INSTANCES. All three arms -- `UnattachedAtScopeEnd`, `TrailingNotModeled`,
// `BodyGrainNotModeled` -- carry `origin` and were unlocated by the one printer, so the repair
// is arm-independent: it reads `diagnostic_to_span`, which every variant answers. The third and
// fourth tests hold the other two arms so "arm-independent" is executed, not argued.
//
// WHY THE SECOND TEST IS NOT REDUNDANT. The first test cannot tell a constant-position printer
// from a fixed one. The control moves the offending annotation down a known number of lines in
// an otherwise identical file and requires the reported line to move with it, so a hardcoded
// `1:1` fails here though it passes there.

use std::path::{Path, PathBuf};

// A FREE FUNCTION RATHER THAN AN RAII GUARD: a `struct` + two `impl` blocks would add four
// hand-authored Rust items to `src/v1`, every `impl` method UNCITABLE as a `DeclarationRef`
// (`std.decl_ref` has no `ImplMethod` field) — exactly the class `gunbc.seed_growth_admission`
// reports and asks authors not to grow. Cleanup runs on the way IN, which makes the run
// deterministic: a tree left by a killed run cannot decide the next one. No `tempfile`
// dev-dependency exists in this crate and this test does not earn one; the directory is named
// after the test, so the four tests cannot collide.
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

// The fixture's refusal is a real one: a leading `//` block at end of scope names no module
// item, so `annotation_attach_resolve` finds no following subject and refuses.
fn fixture(leading_blank_lines: usize) -> String {
    format!(
        "module probe_root.probe\n\ndata probe: Bool = true\n{}\n// this annotation names no subject\n",
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
    // The annotation is on line 5 of `fixture(0)`: module, blank, data, blank, annotation.
    assert!(
        errors[0].contains("probe.dag:5:1:"),
        "the refusal must name the annotation's line and column, got: {}",
        errors[0]
    );
}

#[test]
fn parse_sweep_refusal_location_follows_the_annotation() {
    let dir = scratch_root("moved");
    let errors = sweep_errors(&dir, "moved", &fixture(7));

    assert_eq!(
        errors.len(),
        1,
        "expected exactly one refusal, got {errors:?}"
    );
    // Seven blank lines inserted above it, so the same annotation is now on line 12. A printer
    // emitting a constant position reports 5 here and fails.
    assert!(
        errors[0].contains("probe.dag:12:1:"),
        "the reported line must move with the annotation, got: {}",
        errors[0]
    );
}

// THE OTHER TWO ARMS, so "one defect, not three" is executed. Neither needs its own printer
// change; a printer locating only the tested arm would look identical to one locating all.

#[test]
fn parse_sweep_locates_a_trailing_annotation_refusal() {
    let dir = scratch_root("trailing");
    // `data probe: Bool = true ` is 24 characters, so the `//` opens at column 25 of line 3.
    let errors = sweep_errors(
        &dir,
        "trailing",
        "module probe_root.probe\n\ndata probe: Bool = true // trailing placement is not modeled\n",
    );

    assert_eq!(
        errors.len(),
        1,
        "expected exactly one refusal, got {errors:?}"
    );
    assert!(
        errors[0].contains("probe.dag:3:25:"),
        "the trailing refusal must name the annotation's line and column, got: {}",
        errors[0]
    );
}

#[test]
fn parse_sweep_locates_a_body_grain_annotation_refusal() {
    let dir = scratch_root("body");
    let errors = sweep_errors(
        &dir,
        "body",
        "module probe_root.probe\n\nfn probe() -> Bool {\n  // inside a declaration body\n  true\n}\n",
    );

    assert_eq!(
        errors.len(),
        1,
        "expected exactly one refusal, got {errors:?}"
    );
    assert!(
        errors[0].contains("probe.dag:4:3:"),
        "the body-grain refusal must name the annotation's line and column, got: {}",
        errors[0]
    );
}
