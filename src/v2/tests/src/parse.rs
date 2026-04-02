//! Parse-level tests for the v2 compiler.
//!
//! Covers: syntax smoke tests, strict .dag file parse audits,
//! tokenizer e2e, and parser e2e tests.
//! All tests call stage0 functions directly.

use crate::helpers::*;
use std::rc::Rc;
use v2_compiler::v2_compiler_parse::{child_type_or_error, return_type_node_to_outputs};
use v2_compiler::v2_std_core::{
    field_node_type_expr, leaf_node, Cardinality, Connective, ExprData, Node, SourceSpan,
};
use v2_compiler::v2_std_core::{InferredNode, TokenShape};

fn zero_span() -> Rc<SourceSpan> {
    SourceSpan::new(0, 0)
}

fn synthetic_node(
    name: &str,
    children: Vec<Rc<Node>>,
    connective: Option<Connective>,
    inferred: Option<Rc<InferredNode>>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.to_string(),
        span: zero_span(),
        ident_span: None,
        children,
        connective,
        params: vec![],
        inferred,
        return_cardinality: Cardinality::Required,
        uses: vec![],
        body: None,
        transport: None,
        properties: vec![],
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
    })
}

#[test]
fn child_type_or_error_fails_closed_for_untyped_child() {
    let child = synthetic_node("value", vec![], None, None);
    let ty = child_type_or_error(child);
    assert_eq!(ty.name, "Error");
}

#[test]
fn return_type_node_to_outputs_refuses_partial_product_types() {
    let typed_child = synthetic_node(
        "ok",
        vec![],
        None,
        Some(Rc::new(InferredNode::Resolved {
            node: leaf_node("String".to_string()),
        })),
    );
    let untyped_child = synthetic_node("bad", vec![], None, None);
    let product = synthetic_node(
        "Outputs",
        vec![typed_child, untyped_child],
        Some(Connective::Conj),
        None,
    );
    let outputs = return_type_node_to_outputs(product);
    assert!(outputs.is_empty(), "partial products must fail closed");
}

#[test]
fn return_type_node_to_outputs_preserves_fully_typed_products() {
    let first = synthetic_node(
        "name",
        vec![],
        None,
        Some(Rc::new(InferredNode::Resolved {
            node: leaf_node("String".to_string()),
        })),
    );
    let second = synthetic_node(
        "count",
        vec![],
        None,
        Some(Rc::new(InferredNode::Resolved {
            node: leaf_node("Int".to_string()),
        })),
    );
    let product = synthetic_node("Outputs", vec![first, second], Some(Connective::Conj), None);
    let outputs = return_type_node_to_outputs(product);
    assert_eq!(outputs.len(), 2);
    assert_eq!(field_node_type_expr(outputs[0].clone()).name, "String");
    assert_eq!(field_node_type_expr(outputs[1].clone()).name, "Int");
}

#[test]
fn inline_service_operation_preserves_scalar_return_annotation() {
    let source = r#"module test
service demo.Api {
  operation Ping(city: String) -> String
}"#;
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "parse error: {:?}",
        result.error.as_ref().map(|e| e.diagnostic.clone())
    );

    let module = result.module.clone().expect("module");
    let service = module.children[0].clone();
    let operation = service.children[0].clone();
    let inferred = operation.inferred.clone().expect("operation inferred");
    match inferred.as_ref() {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "String");
            assert!(node.children.is_empty(), "inline return should stay scalar");
        }
        other => panic!("expected resolved operation return, got {:?}", other),
    }
}

#[test]
fn inline_resource_capability_preserves_scalar_return_annotation() {
    let source = r#"module test
resource Filesystem {
  capability read(path: String) -> String
}"#;
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "parse error: {:?}",
        result.error.as_ref().map(|e| e.diagnostic.clone())
    );

    let module = result.module.clone().expect("module");
    let resource = module.children[0].clone();
    let capability = resource.children[0].clone();
    let inferred = capability.inferred.clone().expect("capability inferred");
    match inferred.as_ref() {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "String");
            assert!(node.children.is_empty(), "inline return should stay scalar");
        }
        other => panic!("expected resolved capability return, got {:?}", other),
    }
}

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
fn string_interpolation_accepts_literal_expression_starts() {
    let source = r#"module interp_literal_starts
fn demo() -> String {
  "{1} {"x"} {[1, 2]}"
}"#;
    assert_parses(
        source,
        "string_interpolation_accepts_literal_expression_starts",
    );
}

#[test]
fn string_interpolation_leaves_literal_braces_and_quantifiers_alone() {
    let source = r#"module interp_literal_braces
type ProjectId = String where pattern("^[a-z][a-z0-9-]{4,28}[a-z0-9]$")

fn braces() -> String {
  "{"
}
"#;
    assert_parses(
        source,
        "string_interpolation_leaves_literal_braces_and_quantifiers_alone",
    );
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

// ── M1 regression: generic function syntax ──────────────────────────────

#[test]
fn parse_generic_fn() {
    let source = r#"module test
fn identity<T>(x: T) -> T {
  x
}"#;
    assert_parses(source, "generic fn");
}

#[test]
fn parse_multi_type_param_fn() {
    let source = r#"module test
fn fold_stack<T, B>(stack: List<T>, init: B, f: fn(B, T) -> B) -> B {
  init
}"#;
    assert_parses(source, "multi-type-param generic fn");
}

#[test]
fn item_ident_spans_point_at_identifiers_not_keywords() {
    let source = r#"module test
type Widget = String
fn make_widget() -> Widget {
  Widget
}
service weather.api {
}"#;
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "parse error: {:?}",
        result.error.as_ref().map(|e| e.diagnostic.clone())
    );

    let module = result.module.clone().expect("module");
    assert_eq!(module.children.len(), 3, "expected three top-level items");

    let type_item = module.children[0].clone();
    let type_ident = type_item.ident_span.clone().expect("type ident span");
    assert_eq!(
        type_item.span.start,
        source.find("type Widget").unwrap() as i64
    );
    assert_eq!(type_ident.start, source.find("Widget").unwrap() as i64);
    assert!(type_ident.start > type_item.span.start);

    let fn_item = module.children[1].clone();
    let fn_ident = fn_item.ident_span.clone().expect("fn ident span");
    assert_eq!(
        fn_item.span.start,
        source.find("fn make_widget").unwrap() as i64
    );
    assert_eq!(fn_ident.start, source.find("make_widget").unwrap() as i64);
    assert!(fn_ident.start > fn_item.span.start);

    let service_item = module.children[2].clone();
    let service_ident = service_item.ident_span.clone().expect("service ident span");
    assert_eq!(
        service_item.span.start,
        source.find("service weather.api").unwrap() as i64
    );
    assert_eq!(
        service_ident.start,
        source.find("weather.api").unwrap() as i64
    );
    assert!(service_ident.start > service_item.span.start);
}

#[test]
fn type_alias_rhs_ident_span_points_at_authored_type_name() {
    let source = r#"module test
type Alias = ResultType
"#;
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "parse error: {:?}",
        result.error.as_ref().map(|e| e.diagnostic.clone())
    );

    let module = result.module.clone().expect("module");
    let alias_item = module.children[0].clone();
    let rhs = match alias_item
        .inferred
        .as_ref()
        .expect("alias inferred")
        .as_ref()
    {
        InferredNode::Resolved { node, .. } => node.clone(),
        other => panic!("expected resolved alias rhs, got {:?}", other),
    };
    let rhs_ident = rhs.ident_span.clone().expect("rhs ident span");
    assert_eq!(
        &source[rhs_ident.start as usize..rhs_ident.end as usize],
        "ResultType"
    );
    assert_eq!(rhs_ident.start, source.find("ResultType").unwrap() as i64);
    assert!(rhs_ident.start > alias_item.span.start);
}

#[test]
fn type_param_spans_point_at_identifiers_not_delimiters() {
    let source = r#"module test
type Pair<KeyT, ValueU> = Map<KeyT, ValueU>
"#;
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "parse error: {:?}",
        result.error.as_ref().map(|e| e.diagnostic.clone())
    );

    let module = result.module.clone().expect("module");
    let type_item = module.children[0].clone();
    assert_eq!(type_item.params.len(), 2, "expected two type params");

    let key_param = type_item.params[0].clone();
    let key_type = key_param.children[0].clone();
    let key_ident = key_type.ident_span.clone().expect("key type ident span");
    assert_eq!(
        &source[key_ident.start as usize..key_ident.end as usize],
        "KeyT"
    );
    assert_eq!(key_ident.start, source.find("KeyT").unwrap() as i64);

    let value_param = type_item.params[1].clone();
    let value_type = value_param.children[0].clone();
    let value_ident = value_type
        .ident_span
        .clone()
        .expect("value type ident span");
    assert_eq!(
        &source[value_ident.start as usize..value_ident.end as usize],
        "ValueU"
    );
    assert_eq!(value_ident.start, source.find("ValueU").unwrap() as i64);
}

#[test]
fn stack_parses_strict() {
    assert_parses_strict("dsl/std/stack.dag");
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
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShPipeArrow)),
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
