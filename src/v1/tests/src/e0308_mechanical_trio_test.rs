//! Mechanical E0308 trio in the v1 seed emitter: Range-vs-usize (`skip().first()`),
//! String-vs-str (read-only callee borrow + `&str` signatures), and Unit-vs-Option
//! (optional tail on `-> Unit` must discard, not return `Some`/`None`).

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

fn return_sig(emitted: &str, fn_name: &str) -> String {
    let needle = format!("fn {fn_name}");
    let start = emitted
        .find(&needle)
        .unwrap_or_else(|| panic!("fixture fn `{fn_name}` was not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let body_open = rest
        .find(" {")
        .unwrap_or_else(|| panic!("no body opener for `{fn_name}`:\n{emitted}"));
    rest[..body_open].to_string()
}

#[test]
fn string_length_call_borrows_string_arg() {
    let source =
        "module strcall.fixture\n\nfn use_len(s: String) -> Int {\n  string_length(s: s)\n}\n";
    let body = fn_body(&emit(source), "use_len");
    assert!(
        body.contains("v1_rt::string_length(&"),
        "String arg to string_length must borrow at the call site, got:\n{body}"
    );
}

#[test]
fn callee_borrow_passes_str_ref_to_concat() {
    let source = "module strsig.fixture\n\nfn echo_twice(a: String, b: String) -> String {\n  concat(a, b, a)\n}\n";
    let body = fn_body(&emit(source), "echo_twice");
    assert!(
        body.contains("concat(&a") || body.contains("concat(& a"),
        "read-only String args at call sites must borrow, got:\n{body}"
    );
}

#[test]
fn list_slice_emits_usize_range_on_rc_vec() {
    let source =
        "module slice.fixture\n\nfn tail(items: List<Int>) -> List<Int> {\n  items[1..3]\n}\n";
    let body = fn_body(&emit(source), "tail");
    assert!(
        body.contains("[1 as usize..3 as usize]"),
        "list slice must emit usize range bounds, not a bare integer range, got:\n{body}"
    );
}

#[test]
fn skip_first_emits_iter_skip_not_get_index() {
    let source = "module skipfirst.fixture\n\nfn second(xs: List<Int>) -> Int? {\n  xs.skip(n: 1).first()\n}\n";
    let body = fn_body(&emit(source), "second");
    assert!(
        body.contains(".iter().cloned().skip(") && body.contains("as usize).next().cloned()"),
        "skip().first() must emit iter/skip/next, not get(index), got:\n{body}"
    );
    assert!(
        !body.contains(".get(1 as usize)"),
        "skip count must not be used as a direct index, got:\n{body}"
    );
}

#[test]
fn unit_return_discards_optional_tail() {
    let source = "module unitopt.fixture\n\nfn maybe_discard(flag: Bool) -> Unit {\n  if flag { Present { value: \"x\" } } else { none }\n}\n";
    let body = fn_body(&emit(source), "maybe_discard");
    assert!(
        !body.contains("Some(") && !body.contains("None"),
        "-> Unit must discard optional tail, not emit Some/None, got:\n{body}"
    );
    assert!(
        body.contains(';'),
        "unit-discarding body should statementize the optional if, got:\n{body}"
    );
}
