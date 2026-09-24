use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

/// A map-literal key is its DECODED value, so a key of one double quote or one backslash must be
/// escaped before it is quoted. Unescaped, `"\"": 34` rendered `""".to_string()` and rustc's lexer
/// stayed inside a string for the rest of the emitted file (extdeps.languages.yaml.ingest).
#[test]
fn map_literal_keys_holding_a_quote_or_a_backslash_are_escaped() {
    let source = "module esc.keys\n\
                  data probe: Map<String, Int> = { \"\\\"\": 34, \"\\\\\": 92, \"q\": 1 }\n";
    let result = compile_sources(
        Rc::new(im::vector![Rc::new(SourceFile {
            path: "esc/keys.dag".to_string(),
            content: source.to_string(),
        })]),
        RenderTarget::Rust,
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let joined = result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains(r#"__m.insert("\"".to_string(), 34);"#),
        "{joined}"
    );
    assert!(
        joined.contains(r#"__m.insert("\\".to_string(), 92);"#),
        "{joined}"
    );
    assert!(
        joined.contains(r#"__m.insert("q".to_string(), 1);"#),
        "{joined}"
    );
    assert!(
        !joined.contains(r#"__m.insert(""".to_string()"#),
        "{joined}"
    );
}
