use std::path::Path;

use daglang_syntax::diagnostic::DiagnosticKind;
use daglang_syntax::parser::{byte_to_line_col, parse};

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
