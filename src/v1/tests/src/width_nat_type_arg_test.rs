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
    let content = read_v2_file("dag/std/integer.dag");
    let sources = resolve_imports_transitively_with_source_roots(
        "dag/std/integer.dag",
        &content,
        &source_roots(),
    );
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let msgs = blocking_diagnostics(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "integer.dag should resolve on v2 after gate #60 width-nat parse: {msgs:?}"
    );
}

#[test]
fn v1_std_float_dag_resolves_with_literal_machine_width() {
    let content = read_v2_file("dag/std/float.dag");
    let sources = resolve_imports_transitively_with_source_roots(
        "dag/std/float.dag",
        &content,
        &source_roots(),
    );
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let msgs = blocking_diagnostics(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "float.dag should resolve on v2 after gate #60 width-nat parse: {msgs:?}"
    );
}

#[test]
#[ignore = "fix is in the .dag emitter (05_emit_rust) on this branch, but the in-process lib (committed stage0 seed) is not yet regenerated, so compile_sources here still uses the un-peeled emitter; un-ignore after the seed regen lands (2-stage bootstrap), per Track A step 3"]
fn machine_width_phantom_arg_rust_emit_peels_literal_width_to_unit() {
    use v1_compiler::cli_run;
    use v1_compiler::v1_compiler_artifact::RenderTarget;
    use v1_compiler::v1_compiler_compile::compile_sources;

    let roots: Vec<String> = crate::helpers::source_roots()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let entry = crate::helpers::workspace_root()
        .join("dag/std/integer.dag")
        .to_string_lossy()
        .to_string();
    let sources = cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {entry}: {e}"));
    let result = compile_sources(std::rc::Rc::new(sources), RenderTarget::Rust);
    let integer_rs = result
        .files
        .iter()
        .find(|f| f.path.contains("std_integer.rs"))
        .map(|f| f.content.as_str())
        .unwrap_or("");
    assert!(
        integer_rs.contains("MachineWidth<()>"),
        "literal MachineWidth<N> should peel to MachineWidth<()> in Rust emit, got:\n{integer_rs}"
    );
    assert!(
        !integer_rs.contains("MachineWidth<>"),
        "peeled MachineWidth must not emit empty angle brackets (E0107), got:\n{integer_rs}"
    );
    assert!(
        integer_rs.contains("MachineWidth<PointerWidth>"),
        "PointerWidth token should stay as a type argument, got:\n{integer_rs}"
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
