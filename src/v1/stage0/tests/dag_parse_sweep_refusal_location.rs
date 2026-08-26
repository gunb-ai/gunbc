// THE PARSE SWEEP'S REFUSALS CARRY THE LOCATION THE DIAGNOSTIC ALREADY HELD.
//
// WHAT WENT WRONG AND WHY IT COST AN HOUR. `AnnotationAttachmentRefusal` carries `origin:
// SourceSpan` and `diagnostic_to_span` reads it, but `run_dag_parse_sweep` rendered only the
// path and the message. A file with 52 unattachable annotations therefore produced 52
// byte-identical sentences and no way to tell which `//` line was at fault. The span was
// computed and dropped at the last step -- a typed, LOCATED diagnostic rendered as untyped
// prose, which is the opposite of what DESIGN section 5 requires.
//
// ONE DEFECT, THREE INSTANCES. `AnnotationAttachmentRefusal` has three arms --
// `UnattachedAtScopeEnd`, `TrailingNotModeled`, `BodyGrainNotModeled` -- and all three carry
// `origin`. They were all unlocated for the same single reason (one printer, not three), so the
// repair is arm-independent: it reads `diagnostic_to_span`, which every variant answers. The
// third and fourth tests hold the other two arms so that "arm-independent" is executed rather
// than argued.
//
// WHY THE SECOND TEST IS NOT REDUNDANT. A printer that emits a constant position is
// indistinguishable from a fixed one by the first test alone. The control moves the offending
// annotation down a known number of lines in an otherwise identical file and requires the
// reported line to move with it, so a hardcoded `1:1` fails here even though it passes there.

use std::path::{Path, PathBuf};

// A FREE FUNCTION RATHER THAN AN RAII GUARD, and the reason is the seed's own accounting: a
// `struct` + two `impl` blocks would add four more hand-authored Rust items to `src/v1`, and
// every `impl` method among them is UNCITABLE as a `DeclarationRef` (`std.decl_ref` has no
// `ImplMethod` field), so the guard would have grown exactly the class
// `gunbc.seed_growth_admission` reports separately and asks authors not to grow. Cleanup runs
// on the way IN, which is what makes the run deterministic anyway: a tree left behind by a
// killed run cannot make the next one pass or fail for the wrong reason. There is no
// `tempfile` dev-dependency in this crate and this test does not earn one; the directory is
// named after the test, so the four tests cannot collide.
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
        Ok(n) => panic!("{module}: sweep reported {n} clean file(s); the fixture must refuse"),
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

// THE OTHER TWO ARMS, so the claim that this is one defect rather than one of three is executed.
// Neither needs its own printer change; both are here because a printer that located only the
// arm someone tested would look identical to a printer that located all of them.

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
