#[test]
fn malformed_inputs_return_errors_without_panicking() {
    let malformed_sources = [
        "module bad\nfn",
        "module bad\nimport",
        "module bad\ntype",
        "module bad\n@",
    ];

    for source in malformed_sources {
        let result = std::panic::catch_unwind(|| daglang_syntax::parser::parse(source));
        assert!(
            result.is_ok(),
            "parser should not panic for malformed source"
        );
        assert!(
            result.unwrap().is_err(),
            "malformed source should return parser diagnostics: {source:?}"
        );
    }
}

#[test]
fn lexer_unknown_character_surfaces_as_parser_diagnostic() {
    let result = daglang_syntax::parser::parse("module bad\n$");
    assert!(result.is_err(), "expected parse failure for unknown token");
    let errors = result.err().unwrap();
    assert!(
        errors
            .iter()
            .any(|err| err.message.contains("unexpected character '$'")),
        "expected lexical diagnostic to be surfaced through parser: {errors:?}"
    );
}

#[test]
fn parser_recovers_to_next_top_level_item_and_reports_multiple_errors() {
    let source = r#"
module bad
import
fn broken(
type Broken =
"#;

    let errors = daglang_syntax::parser::parse(source)
        .expect_err("malformed source should produce parse errors");
    assert!(
        errors.len() >= 2,
        "expected multiple diagnostics via top-level recovery, got: {errors:?}"
    );
}
