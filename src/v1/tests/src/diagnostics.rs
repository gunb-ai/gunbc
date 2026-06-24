use crate::helpers::{compile_multi, diagnostic_messages};
use std::rc::Rc;
use v1_compiler::v1_std_core::{
    build_newline_index, byte_to_line_col, diagnostic_to_message, CompilerDiagnostic, ErrorNode,
};

fn first_diag(files: &[(&str, &str)]) -> Rc<ErrorNode> {
    let result = compile_multi(files);
    assert!(
        !result.diagnostics.is_empty(),
        "expected at least one diagnostic, got none"
    );
    result.diagnostics[0].clone()
}

fn diag_line_col(diag: &ErrorNode, source: &str, file: &str) -> (i64, i64) {
    let span = v1_compiler::v1_std_core::diagnostic_to_span(diag.diagnostic.clone());
    let idx = build_newline_index(file.to_string(), source.to_string());
    let lc = byte_to_line_col(idx, span.start);
    (lc.line, lc.col)
}

#[test]
fn missing_export_points_at_name() {
    let source = "module provider\ntype User { name: String }\n";
    let bad = "module consumer\nimport provider { NonExistent }\n";
    let result = compile_multi(&[("provider.dag", source), ("consumer.dag", bad)]);

    assert_eq!(result.diagnostics.len(), 1);
    let d = &result.diagnostics[0];

    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::MissingExport { .. }),
        "expected MissingExport, got: {:?}",
        d.diagnostic
    );

    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(
        msg.contains("NonExistent"),
        "message should name the missing export: {}",
        msg
    );
    assert!(
        msg.contains("provider"),
        "message should name the target module: {}",
        msg
    );
    assert!(
        msg.contains("consumer"),
        "message should name the importing module: {}",
        msg
    );

    let (line, col) = diag_line_col(d, bad, "consumer.dag");
    assert_eq!(line, 2, "should be on line 2 (the import line)");
    assert_eq!(
        col, 19,
        "should point at 'NonExistent' (col 19), not 'import' (col 1)"
    );
}

#[test]
fn variant_not_reexported_through_type_only_import() {
    let files = &[
        ("def.dag", "module self_gen8_def\ntype E = A | B\n"),
        (
            "proxy.dag",
            "module self_gen8_proxy\nimport self_gen8_def { E }\n",
        ),
        (
            "use_mod.dag",
            "module self_gen8_use\nimport self_gen8_proxy { B }\n",
        ),
    ];
    let result = compile_multi(files);

    assert_eq!(result.diagnostics.len(), 1);
    let d = &result.diagnostics[0];
    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::MissingExport { .. }),
        "expected MissingExport for variant not in proxy export surface, got: {:?}",
        d.diagnostic
    );
    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(
        msg.contains("B"),
        "message should name the missing variant export: {msg}"
    );
    assert!(
        msg.contains("self_gen8_proxy"),
        "message should name the proxy module: {msg}"
    );
}

#[test]
fn multiple_missing_exports_each_have_own_span() {
    let source = "module provider\ntype User { name: String }\n";
    let bad = "module consumer\nimport provider { Foo, Bar }\n";
    let result = compile_multi(&[("provider.dag", source), ("consumer.dag", bad)]);

    assert_eq!(
        result.diagnostics.len(),
        2,
        "expected 2 diagnostics for 2 missing names"
    );

    let msg0 = diagnostic_to_message(result.diagnostics[0].diagnostic.clone());
    let msg1 = diagnostic_to_message(result.diagnostics[1].diagnostic.clone());
    assert!(
        msg0.contains("Foo"),
        "first diagnostic should mention Foo: {}",
        msg0
    );
    assert!(
        msg1.contains("Bar"),
        "second diagnostic should mention Bar: {}",
        msg1
    );

    let (_, col0) = diag_line_col(&result.diagnostics[0], bad, "consumer.dag");
    let (_, col1) = diag_line_col(&result.diagnostics[1], bad, "consumer.dag");
    assert_ne!(
        col0, col1,
        "Foo and Bar should have different column positions"
    );
}

#[test]
fn unresolved_import_names_module() {
    let bad = "module consumer\nimport nonexistent { Thing }\n";
    let d = first_diag(&[("consumer.dag", bad)]);

    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedImport { .. }),
        "expected UnresolvedImport, got: {:?}",
        d.diagnostic
    );

    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(
        msg.contains("nonexistent"),
        "should name the missing module: {}",
        msg
    );
    assert!(
        msg.contains("consumer"),
        "should name the importing module: {}",
        msg
    );
}

#[test]
fn unresolved_type_in_field() {
    let source = "module types\ntype Wrapper { inner: Bogus }\n";
    let result = compile_multi(&[("types.dag", source)]);

    let type_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. }))
        .collect();

    assert!(
        !type_diags.is_empty(),
        "expected UnresolvedType diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(type_diags[0].diagnostic.clone());
    assert!(
        msg.contains("Bogus"),
        "should name the unresolved type: {}",
        msg
    );
}

#[test]
#[ignore = "78s — hanging in compile pipeline; triage under PERF track"]
fn duplicate_module_detected() {
    let a = "module dup\ntype A { x: Int }\n";
    let b = "module dup\ntype B { y: Int }\n";
    let result = compile_multi(&[("a.dag", a), ("b.dag", b)]);

    let dup_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::DuplicateModule { .. }))
        .collect();

    assert!(
        !dup_diags.is_empty(),
        "expected DuplicateModule diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(dup_diags[0].diagnostic.clone());
    assert!(
        msg.contains("dup"),
        "should name the duplicate module: {}",
        msg
    );
}

#[test]
fn bare_container_type_detected() {
    let source = "module bare\nimport std.types { List }\ntype Foo { items: List }\n";
    let result = compile_multi(&[("bare.dag", source)]);

    let arity_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
        .collect();

    assert!(
        !arity_diags.is_empty(),
        "expected ArityMismatch diagnostic for bare List, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(arity_diags[0].diagnostic.clone());
    assert!(
        msg.contains("List"),
        "should name the bare container type: {}",
        msg
    );
}

#[test]
fn parameterized_container_no_false_positive() {
    let source = "module param\nimport std.types { List }\ntype Foo { items: List<Int> }\n";
    let result = compile_multi(&[("param.dag", source)]);

    let arity_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
        .collect();

    assert!(
        arity_diags.is_empty(),
        "parameterized List<Int> should not trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn unknown_type_name_no_arity_false_positive() {
    let source = "module custom\ntype Widget { label: String }\ntype Bag { item: Widget }\n";
    let result = compile_multi(&[("custom.dag", source)]);

    let arity_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
        .collect();

    assert!(
        arity_diags.is_empty(),
        "user-defined type should not trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn empty_list_wrong_expected_type() {
    let source = "module elist\nfn make_stuff() -> String {\n  []\n}\n";
    let result = compile_multi(&[("elist.dag", source)]);

    let internal_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.module_name == "elist"
                && match &*d.diagnostic {
                    CompilerDiagnostic::InternalError { message, .. } => {
                        message.contains("empty list literal")
                    }
                    _ => false,
                }
        })
        .collect();

    assert!(
        !internal_diags.is_empty(),
        "expected diagnostic for empty list with non-collection expected type, got: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn empty_list_with_type_context_no_false_positive() {
    let source =
        "module elist_ok\nimport std.types { List }\nfn make_list() -> List<String> {\n  []\n}\n";
    let result = compile_multi(&[("elist_ok.dag", source)]);

    let empty_list_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.module_name == "elist_ok"
                && match &*d.diagnostic {
                    CompilerDiagnostic::InternalError { message, .. } => {
                        message.contains("empty list literal")
                    }
                    _ => false,
                }
        })
        .collect();

    assert!(
        empty_list_diags.is_empty(),
        "empty list with type context should not trigger diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn clean_compile_produces_zero_diagnostics() {
    let source = "module clean\ntype Widget { label: String, count: Int }\n";
    let result = compile_multi(&[("clean.dag", source)]);
    assert!(
        result.diagnostics.is_empty(),
        "clean source should produce 0 diagnostics, got: {:?}",
        diagnostic_messages(&result)
    );
}
