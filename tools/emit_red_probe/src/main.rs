use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use v2_compiler::cli_run::{build_multi_entry_index, make_eval_context, resolve_entry_with_index, run_value};
use v2_compiler::v2_interpreter::Value;

fn main() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf();
    let source_root = repo.join("src/v4");
    let entry = repo.join("src/v4/test/claim/manual/inhabitant_neutralization.dag");
    let function = "inhabitant_neutralization_python_to_go_translate_reject_reason_symbol";

    let index = build_multi_entry_index(&[source_root], &HashMap::new())
        .unwrap_or_else(|e| panic!("index: {e}"));
    let (graph, nl) = resolve_entry_with_index(&index, &entry).unwrap_or_else(|e| panic!("resolve: {e}"));
    let ctx = make_eval_context(Rc::clone(&graph), nl);
    match run_value(&ctx, function) {
        Ok(Value::OutcomeAccepted { value, .. }) => match value.as_ref() {
            Value::Sym(s) => println!("REASON_SYM: {s}"),
            other => println!("REASON_VALUE: {}", ctx.format_value(other)),
        },
        Ok(Value::OutcomeRejected { diagnostics, .. }) => {
            for d in diagnostics {
                println!("REJECTED_DIAG: {}", d.message);
            }
        }
        Ok(other) => println!("OTHER: {}", ctx.format_value(&other)),
        Err(e) => eprintln!("RUNTIME_ERROR: {e}"),
    }
}
