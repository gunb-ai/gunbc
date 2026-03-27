//! Parse-level tests for the v2 compiler.
//!
//! Covers: syntax smoke tests, strict .dag file parse audits,
//! tokenizer e2e, and parser e2e tests.
//! All tests call stage0 functions directly.

use crate::helpers::*;
use v2_compiler::v2_std_core::TokenShape;

// ── Phase 0: syntax smoke tests ─────────────────────────────────────────

#[test]
fn fn_lambda_syntax() {
    let source = r#"module test
fn foo() -> List<Int> {
  fold(enumerate([1, 2, 3]), [], fn(acc, pair) {
    if first(pair) < 2 {
      acc
    } else {
      acc
    }
  })
}"#;
    assert_parses(source, "fn_lambda_syntax");
}

#[test]
fn pipe_syntax() {
    let source = r#"module test
fn foo(items: List<Int>) -> Int {
  items |> count
}"#;
    assert_parses(source, "pipe_syntax");
}

#[test]
fn multi_stmt_if_body() {
    let source = r#"module test
fn foo(x: Int) -> Int {
  if x > 0 {
    let y = x + 1
    return y
  }
  x
}"#;
    assert_parses(source, "multi_stmt_if_body");
}

#[test]
fn match_with_variant_construct() {
    let source = r#"module test
fn foo(ch: String) -> String {
  match lookup(table, key: ch) {
    Some { value: kind } => kind
    None => Unknown { char: ch }
  }
}"#;
    assert_parses(source, "match_with_variant_construct");
}

#[test]
fn fold_with_fn_lambda_and_pipe() {
    let source = r#"module test
fn drop_last(stack: List<Int>) -> List<Int> {
  let len = count(stack)
  fold(enumerate(stack), [], fn(result, pair) {
    if first(pair) < len - 1 {
      append(result, items: last(pair))
    } else {
      result
    }
  })
}"#;
    assert_parses(source, "fold_with_fn_lambda_and_pipe");
}

#[test]
fn nested_match_with_pipe() {
    let source = r#"module test
fn foo(item: String) -> List<String> {
  match item {
    TypeDef { body: body } => {
      match body {
        Sum { variants: vs } => []
        _ => []
      }
    }
    _ => []
  }
}"#;
    assert_parses(source, "nested_match_with_pipe");
}

#[test]
fn implicit_block_match_arms() {
    let source = r#"module test
fn foo(item: String) -> String {
  match item {
    TypeDef { name: name, body: body } =>
      let x = name
      x + body

    FuncDef { name: name, params: params } =>
      let y = name
      y
  }
}"#;
    assert_parses(source, "implicit_block_match_arms");
}

#[test]
fn typecheck_match_with_itemresult() {
    let source = r#"module test
fn foo(item: String) -> String {
  match item {
    FnDef { name: name } =>
      let ret_result = resolve_type(expr: name)
      ret_result

    FuncDef { name: name, params: params, uses: uses } =>
      name
  }
}"#;
    assert_parses(source, "typecheck_match_with_itemresult");
}

// ── Phase 0: strict parse audit (.dag files) ────────────────────────────

#[test]
fn core_parses_strict() {
    assert_parses_strict("src/v2/00_core.dag");
}

#[test]
fn tokenize_parses_strict() {
    assert_parses_strict("src/v2/01_tokenize.dag");
}

#[test]
fn parse_parses_strict() {
    assert_parses_strict("src/v2/02_parse.dag");
}

#[test]
fn resolve_parses_strict() {
    assert_parses_strict("src/v2/03_resolve.dag");
}

#[test]
fn typecheck_parses_strict() {
    assert_parses_strict("src/v2/04_infer.dag");
}

#[test]
fn emit_parses_strict() {
    assert_parses_strict("src/v2/05_emit.dag");
}

#[test]
fn pipeline_parses_strict() {
    assert_parses_strict("src/v2/compile.dag");
}

#[test]
fn artifact_parses_strict() {
    assert_parses_strict("src/v2/artifact.dag");
}

#[test]
fn complexity_parses_strict() {
    assert_parses_strict("src/v2/complexity.dag");
}

#[test]
fn ownership_parses_strict() {
    assert_parses_strict("src/v2/ownership.dag");
}

#[test]
fn shared_behavioral_parses_strict() {
    assert_parses_strict("dsl/std/behavioral.dag");
}

#[test]
fn shared_primitives_parses_strict() {
    assert_parses_strict("dsl/std/primitives.dag");
}

// ── Phase 2: tokenizer e2e ──────────────────────────────────────────────

#[test]
fn tokenizer_smoke() {
    let tokens = tokenize("fn add(a: Int) -> Int { a }");
    assert!(!tokens.is_empty(), "should produce at least one token");
}

#[test]
fn tokenizer_empty_input() {
    let tokens = tokenize("");
    assert!(
        !tokens.is_empty(),
        "empty input should still produce Eof token"
    );
}

#[test]
fn tokenizer_keywords() {
    let tokens = tokenize("module fn type import data");
    assert!(
        tokens.len() >= 5,
        "should produce at least 5 keyword tokens, got {}",
        tokens.len()
    );
}

#[test]
fn tokenizer_two_char_operators() {
    let tokens = tokenize("== -> => !=");
    assert!(
        tokens.len() >= 4,
        "should produce at least 4 operator tokens, got {}",
        tokens.len()
    );
}

#[test]
fn tokenizer_scans_pipe_arrow() {
    let tokens = tokenize("items |> count");
    assert!(
        tokens.iter().any(|t| matches!(t.shape, TokenShape::ShPipeArrow)),
        "should contain PipeArrow token"
    );
}

#[test]
fn tokenizer_scans_null_coalesce() {
    let tokens = tokenize("x ?? y");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShNullCoalesce)),
        "should contain NullCoalesce token"
    );
}

#[test]
fn tokenize_produces_correct_kinds() {
    let tokens = tokenize("fn add(a: Int) -> Int { a }");
    assert!(
        tokens.iter().any(|t| matches!(t.shape, TokenShape::ShKwFn)),
        "should contain KwFn token"
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShIdent)),
        "should contain Ident token"
    );
}

// ── Phase 3/4: parse-level tests ────────────────────────────────────────

#[test]
fn parser_e2e() {
    let result = parse_source("module test");
    assert!(result.error.is_none(), "should parse 'module test'");
    let module = result.module.as_ref().expect("should produce module");
    assert_eq!(module.name, "test");
}

#[test]
fn parse_real_source() {
    let source =
        "module test\ntype Foo { x: Int }\ntype Bar = Foo\nfn identity(x: Int) -> Int { x }";
    let result = parse_source(source);
    assert!(result.error.is_none(), "should parse multi-item source");
    let module = result.module.as_ref().expect("should produce module");
    assert!(!module.children.is_empty(), "should have items");
}

#[test]
fn parse_fold_with_fn_lambda() {
    let source = r#"module test
fn transform(items: List<Int>) -> List<Int> {
  fold(items, [], fn(acc, item) {
    append(acc, items: item)
  })
}"#;
    let result = parse_source(source);
    assert!(result.error.is_none(), "fold with fn lambda should parse");
}

#[test]
fn parse_multiline_pipe_chain() {
    let source = "module test\nfn transform(items: List<Int>) -> List<Int> {\n  let x = items |> map(i =>\n    process(i)\n  ) |> filter(f => f != none)\n  x\n}\n";
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "multiline pipe chain should parse: {:?}",
        result.error
    );
}

#[test]
fn parse_fn_lambda_in_call_arg() {
    let source = "module test\nfn transform(items: List<Int>) -> List<Int> {\n  fold(init: [], f: fn(acc, item) {\n    append(acc, items: item)\n  })\n}";
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "fn lambda in call arg should parse: {:?}",
        result.error
    );
}

#[test]
fn gist_transitive_closure_parse() {
    let files = [
        "dsl/std/types.dag",
        "dsl/std/errors.dag",
        "dsl/std/resources.dag",
        "dsl/extdeps/cloud/cloud.dag",
        "dsl/extdeps/cloud/gcp/gcp.dag",
        "dsl/extdeps/github/github.dag",
        "dsl/extdeps/github/auth.dag",
        "dsl/extdeps/github/gists.dag",
        "dsl/extdeps/git.dag",
        "dsl/gunbc/auth/credentials.dag",
        "dsl/gunbc/tools/gist.dag",
    ];
    for path in &files {
        assert_parses_strict(path);
    }
}
