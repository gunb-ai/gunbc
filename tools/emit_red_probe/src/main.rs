//! One-off: print translate/emit rejection diagnostic for python→go neutralized fixture.
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use v2_compiler::cli_run::{build_multi_entry_index, make_eval_context, resolve_entry_with_index, run_claim};
use v2_compiler::v2_std_core::diagnostic_to_message;

fn main() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf();
    let source_root = repo.join("src/v4");
    let entry = repo.join("src/v4/test/claim/manual/inhabitant_neutralization.dag");
    let function = "inhabitant_neutralization_python_to_go_translate_reject_reason_probe";

    let index = build_multi_entry_index(&[source_root.clone()], &HashMap::new())
        .unwrap_or_else(|e| panic!("index: {e}"));
    let (graph, nl) = resolve_entry_with_index(&index, &entry).unwrap_or_else(|e| panic!("resolve: {e}"));
    let ctx = make_eval_context(Rc::clone(&graph), nl);
    match run_claim(&ctx, function) {
        Ok(v2_compiler::cli_run::ClaimOutcome::Bool(true)) => {
            println!("PROBE_BOOL: true");
        }
        Ok(v2_compiler::cli_run::ClaimOutcome::Bool(false)) => {
            println!("PROBE_BOOL: false");
        }
        Ok(other) => println!("PROBE_OUTCOME: {:?}", std::mem::discriminant(&other)),
        Err(diags) => {
            for d in diags {
                println!("DIAG: {}", diagnostic_to_message(d.diagnostic.clone()));
            }
        }
    }
}
