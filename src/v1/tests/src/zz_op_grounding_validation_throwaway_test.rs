use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, v2_layer_roots, workspace_root};

const ENTRY: &str =
    "src/v2/compiler/manual/target_operation_grounding_emit_typescript_test.dag";

const WITNESSES: &[&str] = &[
    "op_grounding_emits_subtraction",
    "op_grounding_emits_less_than",
    "op_grounding_emits_equality",
    "op_grounding_wrong_token_discriminates",
    "op_grounding_catalog_miss_rejects",
    "op_grounding_wire_roundtrips_all",
    "op_grounding_wire_discriminates",
    "op_grounding_ordering_predicate_accepts_units",
    "op_grounding_equality_predicate_accepts_units",
];

fn sources() -> Vec<Rc<SourceFile>> {
    let content = std::fs::read_to_string(workspace_root().join(ENTRY)).expect("read entry");
    resolve_imports_transitively_with_source_roots(ENTRY, &content, &v2_layer_roots())
        .iter()
        .map(|s| {
            Rc::new(SourceFile {
                path: s.path.clone(),
                content: s.content.clone(),
            })
        })
        .collect()
}

#[test]
fn op_grounding_witnesses_all_green() {
    let resolved = compile_to_resolved(Rc::new(sources()));
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "resolve diagnostics: {msgs:?}"
    );
    let graph = resolved.graph.as_ref().expect("graph");
    for w in WITNESSES {
        let v = v1_interpreter::run(graph, resolved.source_indices.clone(), w)
            .unwrap_or_else(|e| panic!("run {w}: {e:?}"));
        match v {
            Value::Bool(true) => {}
            other => panic!("witness {w} returned {other:?}, expected Bool(true)"),
        }
    }
}
