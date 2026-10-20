// PROBE ONLY (probe/* refs; never ships). Answers stern-bat-401's executed-run question for PR-2
// prereq (b): does substituting in `transparent_alias_rhs` move the diagnostic produced at
// `variant_reference_inferred_node`?
//
// A previous dispatch of this probe FAILED on BOTH control and mutant with no ALIAS-PROBE line, i.e. it
// panicked before printing anything, and my grep pattern omitted `panicked` so the cause was lost. This
// revision panics nowhere and prints a marker before and after every step, so an early exit names itself
// instead of reading as a verdict. It asserts nothing: the verdict is the printed diagnostic set, read by
// the instrument and compared between the two trees.
use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};

const WITNESS: &str = "dag/test/claim/algebra_receiver_alias_witness.dag";
const COMPILED_GUARD: &str = "src/v1/stage0/src/v1_compiler_infer_env.rs";

fn mark(step: &str) {
    println!("ALIAS-PROBE MARK {step}");
}

/// Pull one `data NAME: String = "..."` fixture out of the witness module, un-escaping \n and \".
/// Returns None with a named reason rather than panicking, so a parse miss is visible as data.
fn fixture(name: &str) -> Option<String> {
    let path = workspace_root().join(WITNESS);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            println!("ALIAS-PROBE READ_FAIL {WITNESS} {e}");
            return None;
        }
    };
    let needle = format!("data {name}: String = \"");
    let line = match text.lines().find(|l| l.starts_with(&needle)) {
        Some(l) => l.to_string(),
        None => {
            println!("ALIAS-PROBE FIXTURE_NOT_FOUND {name}");
            return None;
        }
    };
    let body = &line[needle.len()..];
    match body.rfind('"') {
        Some(end) => Some(
            body[..end]
                .to_string()
                .replace("\\n", "\n")
                .replace("\\\"", "\""),
        ),
        None => {
            println!("ALIAS-PROBE FIXTURE_UNCLOSED {name}");
            None
        }
    }
}

fn run(label: &str, source: Option<String>) {
    let Some(source) = source else {
        println!("ALIAS-PROBE {label} SKIPPED no fixture");
        return;
    };
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    mark(&format!("{label}: resolving"));
    let sources = resolve_imports_transitively_with_source_roots("probe.dag", &source, &roots);
    mark(&format!("{label}: compiling"));
    let resolved: Rc<ResolvedPipelineResult> = compile_to_resolved(Rc::new(sources.into()));
    mark(&format!("{label}: compiled"));
    let hard: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|e| format!("{:?}", e.diagnostic))
        .filter(|d| !d.contains("UnusedDeclaration"))
        .map(|d| d.chars().take(190).collect())
        .collect();
    // The census counts BLOCKING rows only; the probe cannot see the severity policy from here, so it
    // prints every non-unused diagnostic and the class name leads each Debug string. Deliberate: a class
    // that vanishes from this superset certainly vanished from the blocking subset.
    println!("ALIAS-PROBE {label} hard={}", hard.len());
    for d in hard.iter() {
        println!("ALIAS-PROBE {label} DIAG {d}");
    }
    mark(&format!("{label}: done"));
}

#[test]
fn alias_guard_seam_probe() {
    mark(&format!(
        "start manifest_dir={}",
        env!("CARGO_MANIFEST_DIR")
    ));
    let ws = workspace_root();
    println!("ALIAS-PROBE TREE workspace_root={}", ws.display());
    match std::fs::read_to_string(ws.join(COMPILED_GUARD)) {
        Ok(text) => {
            // The mutated tree makes the params term constant-false, so this flag is what proves the two
            // trees really differ in the guard and the verdict is not comparing a tree with itself.
            println!(
                "ALIAS-PROBE TREE guard_params_term_present={} params_term_neutered={}",
                text.contains("(pre.params.clone().len() as i64) > 0"),
                text.contains("(0 as i64) > 0")
            );
        }
        Err(e) => println!("ALIAS-PROBE TREE guard_file_unreadable {COMPILED_GUARD} {e}"),
    }
    for (label, name) in [
        ("nominal_return_accept", "nominal_return_source"),
        ("alias_return_refuse", "alias_return_source"),
        ("alias_field_refuse", "alias_field_source"),
        ("alias_initializer_accept", "alias_initializer_source"),
    ] {
        run(label, fixture(name));
    }
    mark("all fixtures attempted");
}
