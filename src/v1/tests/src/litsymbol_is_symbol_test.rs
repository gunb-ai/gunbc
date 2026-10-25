use crate::helpers::compile_dag;
use std::rc::Rc;
use v1_compiler::std_syntax::LiteralValue;
use v1_compiler::v1_compiler_infer_types::infer_literal_node;
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn blocking(source: &str) -> Vec<String> {
    compile_dag(source)
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

#[test]
fn infer_literal_node_types_litsymbol_as_symbol() {
    let n = infer_literal_node(Rc::new(LiteralValue::LitSymbol {
        value: "tag".to_string(),
    }));
    assert_eq!(
        n.name.as_str(),
        "Symbol",
        "RED: infer_literal_node(LitSymbol) must mint symbol_type, not string_type"
    );
}

#[test]
fn infer_literal_node_types_litstr_as_string() {
    let n = infer_literal_node(Rc::new(LiteralValue::LitStr {
        value: "tag".to_string(),
    }));
    assert_eq!(n.name.as_str(), "String");
}

// Producer flip only. `-> String { ^tag }` still compiles: conformance is
// the explicit symbol_at_text exemption in kernel_value_declared_type_mismatch_bounded,
// not a declared Symbol→String conversion and not node_type_compatible treating the names as equal.
#[test]
fn two_carets_still_compare() {
    let msgs = blocking("module litsymbol.eq\nfn f() -> Bool { ^tag == ^tag }\n");
    assert!(msgs.is_empty(), "two carets must still compare: {msgs:?}");
}

#[test]
fn quoted_string_still_inhabits_string() {
    let msgs = blocking("module litsymbol.litstr\nfn f() -> String { \"tag\" }\n");
    assert!(msgs.is_empty(), "LitStr is still String: {msgs:?}");
}

#[test]
fn caret_still_inhabits_declared_string() {
    let msgs = blocking("module litsymbol.caret_at_string\nfn f() -> String { ^tag }\n");
    assert!(
        msgs.is_empty(),
        "caret-to-String is still accepted until conformance judges Symbol: {msgs:?}"
    );
}
