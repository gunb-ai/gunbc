use std::collections::HashMap;
use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, compile_to_resolved, SourceFile};
use v1_compiler::v1_compiler_parse::{
    parse_caret_expr, parse_expr, parse_module, token_stream_new, ParseContext, TokenStream,
};
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{
    diagnostic_to_message, empty_intern_table, is_error_diagnostic, ExprData, TokenShape,
};

fn tokenize_expr(src: &str) -> Rc<Vec<Rc<v1_compiler::v1_std_core::Token>>> {
    tokenize(src.to_string(), "test.dag".to_string())
}

fn token_stream_remaining(stream: &TokenStream) -> &[Rc<v1_compiler::v1_std_core::Token>] {
    stream
        .all
        .get(stream.pos as usize..)
        .unwrap_or(&[])
}

fn parse_ctx() -> Rc<ParseContext> {
    Rc::new(ParseContext {
        source_indices: Rc::new(HashMap::new()),
        intern_table: empty_intern_table(),
    })
}

#[test]
fn caret_tokenizes_as_sh_caret() {
    let tokens = tokenize_expr("^foo");
    let first = tokens.first().unwrap();
    assert_eq!(first.shape, TokenShape::ShCaret, "text={:?}", first.text);
    let second = tokens.get(1).unwrap();
    assert_eq!(second.shape, TokenShape::ShIdent);
}

#[test]
fn caret_paren_tokenizes_as_caret_then_lparen() {
    let tokens = tokenize_expr("^(1)");
    assert_eq!(tokens.first().unwrap().shape, TokenShape::ShCaret);
    assert_eq!(tokens.get(1).unwrap().shape, TokenShape::ShLParen);
}

#[test]
fn parse_caret_ident_produces_literal() {
    let tokens = tokenize_expr("^foo_tag");
    let r = parse_caret_expr(token_stream_new(tokens), parse_ctx());
    assert!(r.err.is_none(), "{:?}", r.err);
    match &*r.expr.expr_data {
        ExprData::ExprLiteral { .. } => {}
        other => panic!("expected literal, got {other:?}"),
    }
}

#[test]
fn parse_caret_paren_produces_discriminant_call() {
    let tokens = tokenize_expr("^(1)");
    let r = parse_caret_expr(token_stream_new(tokens), parse_ctx());
    assert!(r.err.is_none(), "{:?}", r.err);
    match &*r.expr.expr_data {
        ExprData::ExprCall { .. } => assert_eq!(r.expr.name, "discriminant"),
        other => panic!(
            "expected discriminant call, got {other:?} name={}",
            r.expr.name
        ),
    }
}

#[test]
fn parse_expr_caret_paren_full_pipeline() {
    let tokens = tokenize_expr("^(1)");
    let r = parse_expr(token_stream_new(tokens), parse_ctx());
    assert!(r.err.is_none(), "{:?}", r.err);
    let non_eof: Vec<_> = token_stream_remaining(&r.tokens)
        .iter()
        .filter(|t| t.shape != TokenShape::ShEof)
        .collect();
    assert!(non_eof.is_empty(), "leftover tokens: {non_eof:?}");
    match &*r.expr.expr_data {
        ExprData::ExprCall { .. } => assert_eq!(r.expr.name, "discriminant"),
        other => panic!("expected call, got {other:?} name={}", r.expr.name),
    }
}

#[test]
fn parse_expr_caret_var_arg_produces_discriminant_call() {
    let tokens = tokenize_expr("^(alpha)");
    let r = parse_expr(token_stream_new(tokens), parse_ctx());
    assert!(r.err.is_none(), "{:?}", r.err);
    match &*r.expr.expr_data {
        ExprData::ExprCall { .. } => assert_eq!(r.expr.name, "discriminant"),
        other => panic!("expected call, got {other:?} name={}", r.expr.name),
    }
}

#[test]
fn parse_module_let_caret_paren() {
    let src = r#"module caret.probe5b

fn probe() -> Bool {
  let sugar = ^(1)
  true
}
"#;
    let tokens = tokenize(src.to_string(), "caret_probe5b.dag".to_string());
    let r = parse_module(token_stream_new(tokens), parse_ctx());
    assert!(r.err.is_none(), "{:?}", r.err);
}

#[test]
fn compile_to_resolved_caret_probe5b_has_no_caret_function_error() {
    let src = r#"module caret.probe5b

fn probe() -> Bool {
  let sugar = ^(1)
  true
}
"#;
    let sources = Rc::new(vec![Rc::new(SourceFile {
        path: "caret_probe5b.dag".to_string(),
        content: src.to_string(),
    })]);
    let result = compile_to_resolved(sources);
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        !msgs.iter().any(|m| m.contains("function '^'")),
        "unexpected diagnostics: {msgs:?}"
    );
}

fn compile_rust_sources(content: &str, path: &str) -> String {
    let sources = Rc::new(vec![Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })]);
    let result = compile_sources(sources, RenderTarget::Rust);
    let errors: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn emit_caret_ident_symbol_literal() {
    let src = r#"module caret.emit_tag

fn caret_tag() -> Bool {
  let t = ^caret_emit_tag
  t == ^caret_emit_tag
}
"#;
    let emitted = compile_rust_sources(src, "caret_emit_tag.dag");
    assert!(
        emitted.contains(r#""caret_emit_tag""#),
        "expected opaque symbol spelling in emitted Rust, got:\n{emitted}"
    );
}

#[test]
fn emit_caret_paren_discriminant_sugar() {
    let src = r#"module caret.emit_disc

type CaretShape
  = CaretAlpha { x: Int }

fn caret_disc() -> String {
  let v = CaretAlpha { x: 1 }
  discriminant(v: ^(v))
}
"#;
    let emitted = compile_rust_sources(src, "caret_emit_disc.dag");
    assert!(
        emitted.contains("CaretAlpha"),
        "expected discriminant lowering to reference ctor arm, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("function '^'") && !emitted.contains("compile_error"),
        "unexpected caret/diag artifacts in emitted Rust:\n{emitted}"
    );
}
