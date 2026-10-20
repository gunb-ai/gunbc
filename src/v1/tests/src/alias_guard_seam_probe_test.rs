// PROBE ONLY (probe/* refs; never ships). Answers stern-bat-401's executed-run question for PR-2
// prereq (b): does substituting in `transparent_alias_rhs` move the diagnostic produced at
// `variant_reference_inferred_node`?
//
// The four programs are NOT retyped here -- they are read out of the authority that CI already runs,
// dag/test/claim/algebra_receiver_alias_witness.dag, so the probe cannot drift from the fixture the
// admission row enrolls (DESIGN 6: cite the producer that re-derives it). Two of them refuse today
// (return position `-> List<Int>`, field position `values: List<Int>`), two accept.
//
// Run at kind=probe against (a) an untouched clone of the head = the control, and (b) the mutant ref
// where the `params > 0` guard term in transparent_alias_rhs is removed = the experiment. The verdict
// is the printed diagnostic multiset per program, compared by identity: if the guard were the boundary,
// the mutant would make the two refusing programs go clean.
use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};
use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};

const WITNESS: &str = "dag/test/claim/algebra_receiver_alias_witness.dag";

/// Pull one `data NAME: String = "..."` fixture out of the witness module, un-escaping \n and \".
fn fixture(name: &str) -> String {
    let path = workspace_root().join(WITNESS);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read authority {WITNESS}: {e}"));
    let needle = format!("data {name}: String = \"");
    let line = text
        .lines()
        .find(|l| l.starts_with(&needle))
        .unwrap_or_else(|| panic!("{name} is not a data string in {WITNESS}"));
    let body = &line[needle.len()..];
    let body = body[..body.rfind('"').unwrap_or(body.len())].to_string();
    body.replace("\\n", "\n").replace("\\\"", "\"")
}

fn run(label: &str, source: &str) {
    let ws = workspace_root();
    let roots = [ws.join("src/v2"), ws.join("dag")];
    let sources = resolve_imports_transitively_with_source_roots("probe.dag", source, &roots);
    let resolved: Rc<ResolvedPipelineResult> = compile_to_resolved(Rc::new(sources.into()));
    let hard: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|e| format!("{:?}", e.diagnostic))
        .filter(|d| !d.starts_with("UnusedDeclaration") && !d.contains("UnusedDeclaration"))
        .map(|d| d.chars().take(190).collect())
        .collect();
    // The census counts BLOCKING rows only; the probe cannot see the severity policy from here, so it
    // prints every non-unused diagnostic and the class name leads each Debug string. That is deliberate:
    // if a class disappears from this superset it certainly disappeared from the blocking subset.
    println!("ALIAS-PROBE {label} hard={}", hard.len());
    for d in hard.iter() {
        println!("ALIAS-PROBE {label} DIAG {d}");
    }
}

#[test]
fn alias_guard_seam_probe() {
    // guard must be readable in the tree we compiled, else the run says nothing about it
    let env_rs = workspace_root().join("src/v1/stage0/src/v1_compiler_infer_env.rs");
    let text = std::fs::read_to_string(&env_rs).expect("compiled guard file missing");
    let has_guard = text.contains("(pre.params.clone().len() as i64) > 0");
    println!("ALIAS-PROBE TREE guard_present_in_compiled_mirror={has_guard}");
    for (label, name) in [
        ("nominal_return_accept", "nominal_return_source"),
        ("alias_return_refuse", "alias_return_source"),
        ("alias_field_refuse", "alias_field_source"),
        ("alias_initializer_accept", "alias_initializer_source"),
    ] {
        run(label, &fixture(name));
    }
}
