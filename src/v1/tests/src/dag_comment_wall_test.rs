use std::collections::HashMap;
use std::rc::Rc;

use v1_compiler::v1_compiler_parse::parse_with_table;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table};

fn try_parse(source: &str) -> bool {
    let tokens = tokenize(source.to_string(), "<wall-test>".to_string());
    let nl = build_newline_index("<wall-test>".to_string(), source.to_string());
    let mut si = HashMap::new();
    si.insert(nl.file.clone(), nl.clone());
    let parsed = parse_with_table(tokens, Rc::new(si), empty_intern_table());
    parsed.result.module.is_some()
}

// Control: clean source must still parse (the wall must not break valid code).
#[test]
fn clean_source_parses() {
    assert!(
        try_parse("module v1.test.wall\n\nfn f() -> Bool { true }"),
        "clean source (no comments) must still parse — wall broke valid code"
    );
}

// Wall: // outside a string literal lexes as two div tokens → parse error.
#[test]
fn line_comment_fails_to_parse() {
    assert!(
        !try_parse("module v1.test.wall\n\nfn f() -> Bool { true } // comment"),
        "// outside a string must not parse — comment-skip was not fully removed"
    );
}

// Wall: /* */ outside a string literal leaves untokenisable characters → parse error.
#[test]
fn block_comment_fails_to_parse() {
    assert!(
        !try_parse("module v1.test.wall\n\n/* comment */\nfn f() -> Bool { true }"),
        "/* */ outside a string must not parse — comment-skip was not fully removed"
    );
}

// Strings: // inside a string literal must remain valid (not part of the wall).
#[test]
fn slash_slash_inside_string_literal_parses() {
    assert!(
        try_parse(
            r#"module v1.test.wall

fn url() -> String { "https://example.com" }"#
        ),
        "// inside a string literal must remain valid after the comment wall"
    );
}
