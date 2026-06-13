//! G2: alias field access through parametric carriers (Measure / ByteSize).

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::compile_to_resolved;
use v2_compiler::v2_std_core::diagnostic_to_message;

use crate::helpers::{
    compile_dag_resolved, resolve_imports_transitively_with_source_roots, workspace_root,
};

fn hard_diagnostic_messages(
    resolved: &v2_compiler::v2_compiler_compile::ResolvedPipelineResult,
) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

#[test]
fn generic_alias_field_access_resolves_through_expansion() {
    let src = r#"
module m

import std.nat { Nat }

type Box<T> {
  value: T
}

type NatBox = Box<Nat>

fn get(b: NatBox) -> Nat {
  b.value
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.is_empty(),
        "generic alias field access should resolve, got: {msgs:?}"
    );
}

#[test]
fn measure_dag_v2_loads_without_field_errors() {
    let entry = "dsl/std/measure.dag";
    let content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    let sources = resolve_imports_transitively_with_source_roots(
        entry,
        &content,
        &[workspace_root().join("dsl")],
    );
    let msgs = hard_diagnostic_messages(&compile_to_resolved(Rc::new(sources)));
    assert!(
        msgs.is_empty(),
        "measure.dag should load on v2, got diagnostics: {msgs:?}"
    );
}
