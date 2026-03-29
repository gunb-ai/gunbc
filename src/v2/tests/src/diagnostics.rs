//! Diagnostic output quality tests.
//!
//! These tests verify that compiler diagnostics are:
//! 1. Typed (correct CompilerDiagnostic variant)
//! 2. Precisely located (file:line:col points at the right token)
//! 3. Actionable (message tells the user what to fix)

use std::rc::Rc;
use v2_compiler::v2_std_core::{
    CompilerDiagnostic, ErrorNode, byte_to_line_col, build_newline_index,
    diagnostic_to_message,
};
use crate::helpers::{compile_multi, diagnostic_messages};

// ── Helpers ─────────────────────────────────────────────────────────────

/// Extract the first diagnostic from a compilation result, panicking if none.
fn first_diag(files: &[(&str, &str)]) -> Rc<ErrorNode> {
    let result = compile_multi(files);
    assert!(
        !result.diagnostics.is_empty(),
        "expected at least one diagnostic, got none"
    );
    result.diagnostics[0].clone()
}

/// Resolve a diagnostic's span to (line, col) using the source text.
fn diag_line_col(diag: &ErrorNode, source: &str, file: &str) -> (i64, i64) {
    let span = v2_compiler::v2_std_core::diagnostic_to_span(diag.diagnostic.clone());
    let idx = build_newline_index(file.to_string(), source.to_string());
    let lc = byte_to_line_col(idx, span.start);
    (lc.line, lc.col)
}

// ── Missing export: precise span on the missing name ────────────────────

#[test]
fn missing_export_points_at_name() {
    let source = "module provider\ntype User { name: String }\n";
    let bad = "module consumer\nimport provider { NonExistent }\n";
    let result = compile_multi(&[("provider.dag", source), ("consumer.dag", bad)]);

    assert_eq!(result.diagnostics.len(), 1);
    let d = &result.diagnostics[0];

    // Correct variant.
    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::MissingExport { .. }),
        "expected MissingExport, got: {:?}", d.diagnostic
    );

    // Message includes name, module, and importer.
    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(msg.contains("NonExistent"), "message should name the missing export: {}", msg);
    assert!(msg.contains("provider"), "message should name the target module: {}", msg);
    assert!(msg.contains("consumer"), "message should name the importing module: {}", msg);

    // Span points at "NonExistent", not at "import".
    let (line, col) = diag_line_col(d, bad, "consumer.dag");
    assert_eq!(line, 2, "should be on line 2 (the import line)");
    assert_eq!(col, 19, "should point at 'NonExistent' (col 19), not 'import' (col 1)");
}

#[test]
fn multiple_missing_exports_each_have_own_span() {
    let source = "module provider\ntype User { name: String }\n";
    let bad = "module consumer\nimport provider { Foo, Bar }\n";
    let result = compile_multi(&[("provider.dag", source), ("consumer.dag", bad)]);

    assert_eq!(result.diagnostics.len(), 2, "expected 2 diagnostics for 2 missing names");

    let msg0 = diagnostic_to_message(result.diagnostics[0].diagnostic.clone());
    let msg1 = diagnostic_to_message(result.diagnostics[1].diagnostic.clone());
    assert!(msg0.contains("Foo"), "first diagnostic should mention Foo: {}", msg0);
    assert!(msg1.contains("Bar"), "second diagnostic should mention Bar: {}", msg1);

    // Different columns.
    let (_, col0) = diag_line_col(&result.diagnostics[0], bad, "consumer.dag");
    let (_, col1) = diag_line_col(&result.diagnostics[1], bad, "consumer.dag");
    assert_ne!(col0, col1, "Foo and Bar should have different column positions");
}

// ── Unresolved import: module not found ─────────────────────────────────

#[test]
fn unresolved_import_names_module() {
    let bad = "module consumer\nimport nonexistent { Thing }\n";
    let d = first_diag(&[("consumer.dag", bad)]);

    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedImport { .. }),
        "expected UnresolvedImport, got: {:?}", d.diagnostic
    );

    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(msg.contains("nonexistent"), "should name the missing module: {}", msg);
    assert!(msg.contains("consumer"), "should name the importing module: {}", msg);
}

// ── Unresolved type ─────────────────────────────────────────────────────

#[test]
fn unresolved_type_in_field() {
    let source = "module types\ntype Wrapper { inner: Bogus }\n";
    let result = compile_multi(&[("types.dag", source)]);

    let type_diags: Vec<_> = result.diagnostics.iter().filter(|d| {
        matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. })
    }).collect();

    assert!(
        !type_diags.is_empty(),
        "expected UnresolvedType diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(type_diags[0].diagnostic.clone());
    assert!(msg.contains("Bogus"), "should name the unresolved type: {}", msg);
}

// ── Duplicate module ────────────────────────────────────────────────────

#[test]
fn duplicate_module_detected() {
    let a = "module dup\ntype A { x: Int }\n";
    let b = "module dup\ntype B { y: Int }\n";
    let result = compile_multi(&[("a.dag", a), ("b.dag", b)]);

    let dup_diags: Vec<_> = result.diagnostics.iter().filter(|d| {
        matches!(&*d.diagnostic, CompilerDiagnostic::DuplicateModule { .. })
    }).collect();

    assert!(
        !dup_diags.is_empty(),
        "expected DuplicateModule diagnostic, got: {:?}",
        diagnostic_messages(&result)
    );

    let msg = diagnostic_to_message(dup_diags[0].diagnostic.clone());
    assert!(msg.contains("dup"), "should name the duplicate module: {}", msg);
}

// ── No false positives ─────────────────────────────────────────────────

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
