//! R3 gate #60 Slice Z — v2 parser accepts literal-Nat width indices in `<…>` type args.

use std::rc::Rc;

use crate::helpers::{
    compile_dag_resolved, parse_source, read_v2_file,
    resolve_imports_transitively_with_source_roots, source_roots,
};
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_std_core::diagnostic_to_message;

fn blocking_diagnostics(
    resolved: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

#[test]
fn parse_type_angle_arg_accepts_literal_nat_width() {
    let source = r#"
module width_nat_parse_test
type Box = MachineWidth<64>
"#;
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "MachineWidth<64> should parse: {:?}",
        result.error
    );
}

#[test]
fn parse_type_angle_arg_rejects_bare_literal_type() {
    let source = r#"
module bare_width_nat_reject
type Box = 64
"#;
    let result = parse_source(source);
    assert!(
        result.error.is_some(),
        "standalone literal type position must stay a parse error"
    );
}

#[test]
fn v1_std_integer_dag_resolves_with_literal_machine_width() {
    let content = read_v2_file("dsl/std/integer.dag");
    let sources = resolve_imports_transitively_with_source_roots(
        "dsl/std/integer.dag",
        &content,
        &source_roots(),
    );
    let resolved = compile_to_resolved(Rc::new(sources));
    let msgs = blocking_diagnostics(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "integer.dag should resolve on v2 after gate #60 width-nat parse: {msgs:?}"
    );
}

#[test]
fn v1_std_float_dag_resolves_with_literal_machine_width() {
    let content = read_v2_file("dsl/std/float.dag");
    let sources = resolve_imports_transitively_with_source_roots(
        "dsl/std/float.dag",
        &content,
        &source_roots(),
    );
    let resolved = compile_to_resolved(Rc::new(sources));
    let msgs = blocking_diagnostics(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "float.dag should resolve on v2 after gate #60 width-nat parse: {msgs:?}"
    );
}

#[test]
fn machine_width_use_site_resolves_without_unresolved_type() {
    let source = r#"
module width_nat_infer_test
import std.machine_constraints { MachineWidth }
type Word = MachineWidth<8>
"#;
    let resolved = compile_dag_resolved(source);
    let msgs = blocking_diagnostics(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "MachineWidth<8> use site should resolve: {msgs:?}"
    );
}
