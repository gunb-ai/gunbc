//! Probe tests for the E0308 mechanical trio — delete after fixes land in
//! e0308_mechanical_trio_test.rs.

use crate::helpers::compile_dag_target;
use v1_compiler::v1_compiler_artifact::RenderTarget;

fn emit(source: &str) -> String {
    compile_dag_target(source, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fn_body(emitted: &str, name: &str) -> String {
    let needle = format!("fn {name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fn `{name}` not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest[needle.len()..]
        .find("\npub fn ")
        .map(|i| i + needle.len())
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn probe_string_length_borrows_string_arg() {
    let source = "module strcall.fixture\n\nfn use_len(s: String) -> Int {\n  string_length(s: s)\n}\n";
    let body = fn_body(&emit(source), "use_len");
    eprintln!("EMITTED:\n{body}");
}

#[test]
fn probe_list_slice_emission() {
    let source = "module slice.fixture\n\nfn tail(items: List<Int>) -> List<Int> {\n  items[1..3]\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    eprintln!("FILES: {:?}", result.files.iter().map(|f| f.path.clone()).collect::<Vec<_>>());
    eprintln!("CONTENT:\n{}", emit(source));
    let body = fn_body(&emit(source), "tail");
    eprintln!("EMITTED:\n{body}");
}

#[test]
fn probe_unit_return_with_optional_tail() {
    let source = "module unitopt.fixture\n\nfn maybe_discard(flag: Bool) -> Unit {\n  if flag { Present { value: \"x\" } } else { none }\n}\n";
    let result = compile_dag_target(source, RenderTarget::Rust);
    eprintln!("FILES: {:?}", result.files.iter().map(|f| f.path.clone()).collect::<Vec<_>>());
    eprintln!("CONTENT:\n{}", emit(source));
    let body = fn_body(&emit(source), "maybe_discard");
    eprintln!("EMITTED:\n{body}");
}
