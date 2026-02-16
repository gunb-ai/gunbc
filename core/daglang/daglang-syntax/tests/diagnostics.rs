use std::path::Path;

use daglang_syntax::diagnostic::DiagnosticKind;
use daglang_syntax::parser::{byte_to_line_col, parse, parse_with_file_diagnostics};

#[test]
fn byte_to_line_col_handles_multiline_offsets_and_eof_clamp() {
    let src = "module test\nfn broken(\n";

    assert_eq!(byte_to_line_col(src, 0), (1, 1));
    assert_eq!(byte_to_line_col(src, 7), (1, 8));
    assert_eq!(byte_to_line_col(src, 12), (2, 1));
    assert_eq!(
        byte_to_line_col(src, src.len() + 100),
        byte_to_line_col(src, src.len()),
        "offsets beyond EOF should clamp to EOF"
    );
}

#[test]
fn parse_error_formats_with_file_line_col() {
    let src = "module test\nfn broken( -> String {\n";
    let err = parse(src)
        .expect_err("source should fail to parse")
        .into_iter()
        .next()
        .expect("expected at least one parse error");

    let rendered = err.format_with_source(Path::new("sample.dag"), src);
    assert!(rendered.contains("sample.dag:2:12"));
}

#[test]
fn parse_error_converts_to_parse_diagnostic() {
    let src = "module test\nfn broken( -> String {\n";
    let err = parse(src)
        .expect_err("source should fail to parse")
        .into_iter()
        .next()
        .expect("expected at least one parse error");

    let diagnostic = err.to_diagnostic(Path::new("sample.dag"), src);
    assert_eq!(diagnostic.kind, DiagnosticKind::Parse);
    assert_eq!(diagnostic.file.as_ref().and_then(|f| f.to_str()), Some("sample.dag"));
    assert!(diagnostic.span.is_some(), "parse diagnostic should carry span");
    assert_eq!(diagnostic.line, Some(2));
}

#[test]
fn parse_with_file_diagnostics_preserves_lex_diagnostic_kind() {
    let src = "module test\n$\n";
    let diagnostics = parse_with_file_diagnostics(Path::new("sample.dag"), src)
        .expect_err("source should fail with lexical diagnostic");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].kind, DiagnosticKind::Lex);
    assert_eq!(
        diagnostics[0].file.as_ref().and_then(|f| f.to_str()),
        Some("sample.dag")
    );
    assert_eq!(diagnostics[0].line, Some(2));
    assert_eq!(diagnostics[0].column, Some(1));
}

#[test]
fn parse_with_file_diagnostics_preserves_parse_diagnostic_kind() {
    let src = "module test\nfn broken( -> String {\n";
    let diagnostics = parse_with_file_diagnostics(Path::new("sample.dag"), src)
        .expect_err("source should fail with parse diagnostic");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].kind, DiagnosticKind::Parse);
    assert_eq!(
        diagnostics[0].file.as_ref().and_then(|f| f.to_str()),
        Some("sample.dag")
    );
    assert_eq!(diagnostics[0].line, Some(2));
    assert_eq!(diagnostics[0].column, Some(12));
}

#[test]
fn parse_with_file_diagnostics_aggregates_multiple_lex_diagnostics() {
    let src = "module test\n$\n&\n";
    let diagnostics = parse_with_file_diagnostics(Path::new("sample.dag"), src)
        .expect_err("source should fail with lexical diagnostics");
    assert_eq!(diagnostics.len(), 2, "expected both lexical diagnostics");
    assert!(diagnostics.iter().all(|diag| diag.kind == DiagnosticKind::Lex));
    assert_eq!(diagnostics[0].line, Some(2));
    assert_eq!(diagnostics[0].column, Some(1));
    assert_eq!(diagnostics[1].line, Some(3));
    assert_eq!(diagnostics[1].column, Some(1));
}
