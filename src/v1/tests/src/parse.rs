use std::rc::Rc;

use crate::helpers::*;
use v1_compiler::std_types::SourceSpan;
use v1_compiler::v1_compiler_tokenize::{source_code_point, source_len, SourceRef};
use v1_compiler::v1_rt::{reset_text_lookup_chars_walked, take_text_lookup_chars_walked};
use v1_compiler::v1_std_core::{build_newline_index, source_text_at, InferredNode, TokenShape};

fn median_tokenize_secs(source: &str) -> (f64, usize) {
    const RUNS: usize = 5;
    let mut samples = Vec::with_capacity(RUNS);
    let mut last_len = 0usize;
    for _ in 0..RUNS {
        let t0 = std::time::Instant::now();
        let toks = tokenize(source);
        last_len = toks.len();
        samples.push(t0.elapsed().as_secs_f64());
    }
    samples.sort_by(|a, b| a.total_cmp(b));
    (samples[RUNS / 2], last_len)
}

fn tokenizer_source_ref(source: &str) -> Rc<SourceRef> {
    let chars = Rc::new(source.chars().map(|c| c as i64).collect::<im_rc::Vector<_>>());
    Rc::new(SourceRef {
        file: "tokenizer_lookup_flat.v3".to_string(),
        text: source.to_string(),
        source_chars: chars,
    })
}

fn source_code_point_chars_walked(source: &Rc<SourceRef>, pos: i64, lookups: usize) -> u64 {
    reset_text_lookup_chars_walked();
    for _ in 0..lookups {
        let _ = source_code_point(source.clone(), pos);
    }
    take_text_lookup_chars_walked()
}

fn source_text_at_chars_walked(
    index: &Rc<v1_compiler::v1_std_core::NewlineIndex>,
    span: &Rc<SourceSpan>,
    lookups: usize,
) -> u64 {
    reset_text_lookup_chars_walked();
    for _ in 0..lookups {
        let _ = source_text_at(index.clone(), span.clone());
    }
    take_text_lookup_chars_walked()
}

fn name_lookup_padding_fixture(k: usize, pad: usize) -> (String, Vec<Rc<SourceSpan>>) {
    let filler = "§".repeat(pad);
    let mut source = String::from("module pad_test\n");
    let mut spans = Vec::with_capacity(k);
    let file = "pad_test.dag".to_string();
    for i in 0..k {
        if i > 0 {
            source.push_str(&filler);
            source.push('\n');
        }
        let name = format!("fn_{i}");
        let start = source.chars().count() as i64;
        source.push_str(&name);
        let end = source.chars().count() as i64;
        spans.push(Rc::new(SourceSpan {
            file: file.clone(),
            start,
            end,
        }));
        source.push_str(": Int = 0\n");
    }
    (source, spans)
}

fn total_source_text_at_chars_walked(
    index: &Rc<v1_compiler::v1_std_core::NewlineIndex>,
    spans: &[Rc<SourceSpan>],
    lookups_per_span: usize,
) -> u64 {
    reset_text_lookup_chars_walked();
    for _ in 0..lookups_per_span {
        for span in spans {
            let _ = source_text_at(index.clone(), span.clone());
        }
    }
    take_text_lookup_chars_walked()
}

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
fn parse_fn_expression_body() {
    let source = r#"module test
type Color = Red | Green
fn is_red(c: Color) -> Bool =
  match c {
    Red => true
    Green => false
  }
fn code_point(c: Int) -> Int = c + 0
"#;
    assert_parses(source, "expression-bodied fn");
}

#[test]
fn parse_fn_brace_body_still_accepted() {
    let source = r#"module test
fn add(x: Int, y: Int) -> Int { x + y }
"#;
    assert_parses(source, "brace-bodied fn (legacy)");
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
fn module_node_ident_is_populated_by_parser() {
    let source = r#"module my.test.module
import std.types { String }
type Foo { value: String }
"#;
    let result = parse_source(source);
    assert!(result.error.is_none(), "parse error: {:?}", result.error);

    let module = result.module.clone().expect("module");
    assert!(
        module.ident.is_some(),
        "module ident should be Some after parsing"
    );
    assert_ne!(module.ident.unwrap(), 0, "module ident should be non-zero");

    let imports = module.params.clone();
    assert_eq!(imports.len(), 1);
    let import_node = imports[0].clone();
    assert!(import_node.ident.is_some(), "import ident should be Some");
    assert_ne!(
        import_node.ident.unwrap(),
        0,
        "import ident should be non-zero"
    );
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
    assert_parses_strict("dag/std/stack.dag");
}

#[test]
fn unicode_parses_strict() {
    assert_parses_strict("dag/std/unicode.dag");
}

#[test]
fn core_parses_strict() {
    assert_parses_strict("src/v1/00_core.dag");
}

#[test]
fn tokenize_parses_strict() {
    assert_parses_strict("src/v1/01_tokenize.dag");
}

#[test]
fn parse_parses_strict() {
    assert_parses_strict("src/v1/02_parse.dag");
}

#[test]
fn resolve_parses_strict() {
    assert_parses_strict("src/v1/03_resolve.dag");
}

#[test]
fn typecheck_parses_strict() {
    assert_parses_strict("src/v1/04_infer.dag");
}

#[test]
fn emit_parses_strict() {
    assert_parses_strict("src/v1/05_emit.dag");
}

#[test]
fn pipeline_parses_strict() {
    assert_parses_strict("src/v1/compile.dag");
}

#[test]
fn artifact_parses_strict() {
    assert_parses_strict("src/v1/artifact.dag");
}

#[test]
fn complexity_parses_strict() {
    assert_parses_strict("src/v1/complexity.dag");
}

#[test]
fn ownership_parses_strict() {
    assert_parses_strict("src/v1/ownership.dag");
}

#[test]
fn shared_behavioral_parses_strict() {
    assert_parses_strict("dag/std/behavioral.dag");
}

#[test]
fn shared_primitives_parses_strict() {
    assert_parses_strict("dag/std/primitives.dag");
}

#[test]
fn tokenizer_non_ascii_performance_regression() {
    use std::time::Instant;

    let source = read_v2_file("src/v1/tests/fixtures/non_ascii_perf.dag");
    assert!(!source.is_ascii(), "test requires non-ASCII source file");

    let ascii_source: String = source
        .chars()
        .map(|c| if c.is_ascii() { c } else { '-' })
        .collect();
    assert!(ascii_source.is_ascii());

    let start = Instant::now();
    let _ = tokenize(&ascii_source);
    let ascii_time = start.elapsed();

    let start = Instant::now();
    let _ = tokenize(&source);
    let non_ascii_time = start.elapsed();

    let ratio = non_ascii_time.as_secs_f64() / ascii_time.as_secs_f64().max(0.001);
    eprintln!(
        "tokenize non_ascii_perf.dag: ascii={:.3}s, non-ascii={:.3}s, ratio={:.1}x",
        ascii_time.as_secs_f64(),
        non_ascii_time.as_secs_f64(),
        ratio,
    );

    assert!(
        ratio < 3.0,
        "non-ASCII tokenization is {:.1}x slower than ASCII — likely O(n²) regression in v1_rt::substring",
        ratio,
    );

    assert!(
        non_ascii_time.as_secs_f64() < 2.0,
        "tokenize took {:.3}s — budget is 2s for ~48KB fixture",
        non_ascii_time.as_secs_f64(),
    );
}

#[test]
fn tokenizer_text_lookup_flat_in_file_size() {
    const LOOKUPS: usize = 1_000;
    let large_source = read_v2_file("src/v1/02_parse.dag");
    let source = tokenizer_source_ref(&large_source);
    let tail_pos = source_len(source.clone()) - 1;

    let head_walked = source_code_point_chars_walked(&source, 0, LOOKUPS);
    let tail_walked = source_code_point_chars_walked(&source, tail_pos, LOOKUPS);

    eprintln!(
        "tokenizer lookup chars walked: head={head_walked} tail={tail_walked} ({}B source, {LOOKUPS} lookups each)",
        large_source.len(),
    );

    assert_eq!(
        head_walked, LOOKUPS as u64,
        "head lookups should be one index op each"
    );
    assert_eq!(
        tail_walked, LOOKUPS as u64,
        "tail lookups should be one index op each (flat in file offset)"
    );
}

#[test]
fn source_text_at_lookup_flat_in_file_size() {
    let source = read_v2_file("src/v1/tests/fixtures/non_ascii_perf.dag");
    assert!(
        !source.is_ascii(),
        "fixture must include non-ASCII so a reintroduced substring slow path would be caught"
    );
    let index = build_newline_index("lookup_flat.dag".to_string(), source.clone());
    let char_len = source.chars().count() as i64;
    let tail = char_len - 4;
    let head_span = Rc::new(SourceSpan {
        file: "lookup_flat.dag".to_string(),
        start: 0,
        end: 4,
    });
    let tail_span = Rc::new(SourceSpan {
        file: "lookup_flat.dag".to_string(),
        start: tail,
        end: tail + 4,
    });

    const LOOKUPS: usize = 200;
    let head_walked = source_text_at_chars_walked(&index, &head_span, LOOKUPS);
    let tail_walked = source_text_at_chars_walked(&index, &tail_span, LOOKUPS);

    eprintln!(
        "source_text_at chars walked: head={head_walked} tail={tail_walked} ({}B, {LOOKUPS} lookups each)",
        source.len(),
    );

    assert_eq!(head_walked, 800, "head span 0..4 × {LOOKUPS} lookups");
    assert_eq!(
        tail_walked, head_walked,
        "tail span near EOF must walk the same as head — flat in file offset"
    );
}

#[test]
fn source_text_at_lookup_flat_in_file_padding() {
    const K: usize = 8;
    const SMALL_PAD: usize = 32;
    const LARGE_PAD: usize = SMALL_PAD * 10;

    let (small_source, small_spans) = name_lookup_padding_fixture(K, SMALL_PAD);
    let (large_source, large_spans) = name_lookup_padding_fixture(K, LARGE_PAD);
    let small_len = small_source.len();
    let large_len = large_source.len();
    assert!(
        large_len > small_len * 5,
        "large fixture should be >> small (got {large_len} vs {small_len} bytes)"
    );

    let small_index = build_newline_index("small_pad.dag".to_string(), small_source);
    let large_index = build_newline_index("large_pad.dag".to_string(), large_source);

    const LOOKUPS_PER_SPAN: usize = 50;
    let small_walked =
        total_source_text_at_chars_walked(&small_index, &small_spans, LOOKUPS_PER_SPAN);
    let large_walked =
        total_source_text_at_chars_walked(&large_index, &large_spans, LOOKUPS_PER_SPAN);

    eprintln!(
        "source_text_at K={K} names: small {small_len}B walked={small_walked} | large {large_len}B walked={large_walked}"
    );

    assert_eq!(
        small_walked, 1_600,
        "K={K} names × 4 chars × {LOOKUPS_PER_SPAN}"
    );
    assert_eq!(
        large_walked, small_walked,
        "10× padding must not change source_text_at work — flat in file length"
    );
}

#[test]
fn tokenizer_scales_linearly_with_file_size() {
    let small_source = read_v2_file("src/v1/ownership.dag"); // ~23KB
    let large_source = read_v2_file("src/v1/02_parse.dag"); // ~271KB

    let _ = tokenize(&small_source);
    let _ = tokenize(&large_source);

    let (small_time, small_count) = median_tokenize_secs(&small_source);
    let (large_time, large_count) = median_tokenize_secs(&large_source);

    let size_ratio = large_source.len() as f64 / small_source.len() as f64;
    let time_ratio = large_time / small_time.max(0.001);

    eprintln!(
        "small: {}B, {} tokens, {:.3}s (median of 5) | large: {}B, {} tokens, {:.3}s (median of 5) | size ratio: {:.1}x, time ratio: {:.1}x",
        small_source.len(), small_count, small_time,
        large_source.len(), large_count, large_time,
        size_ratio, time_ratio,
    );

    const LINEAR_MARGIN: f64 = 3.2;
    assert!(
        time_ratio < size_ratio * LINEAR_MARGIN,
        "tokenization appears super-linear: size ratio {:.1}x but time ratio {:.1}x (expected < {:.1}x)",
        size_ratio,
        time_ratio,
        size_ratio * LINEAR_MARGIN,
    );
}

#[test]
fn tokenizer_scanning_scales_linearly() {
    let small_source = read_v2_file("src/v1/ownership.dag");
    let large_source = read_v2_file("src/v1/02_parse.dag");

    let _ = tokenize(&small_source);
    let _ = tokenize(&large_source);

    let (small_time, small_count) = median_tokenize_secs(&small_source);
    let (large_time, large_count) = median_tokenize_secs(&large_source);

    let size_ratio = large_source.len() as f64 / small_source.len() as f64;
    let time_ratio = large_time / small_time.max(0.001);

    eprintln!(
        "scan-only: small: {}B, {} tokens, {:.3}s (median of 5) | large: {}B, {} tokens, {:.3}s (median of 5) | size ratio: {:.1}x, time ratio: {:.1}x",
        small_source.len(), small_count, small_time,
        large_source.len(), large_count, large_time,
        size_ratio, time_ratio,
    );

    const LINEAR_MARGIN: f64 = 3.2;
    assert!(
        time_ratio < size_ratio * LINEAR_MARGIN,
        "scanning appears super-linear: size ratio {:.1}x but time ratio {:.1}x (expected < {:.1}x)",
        size_ratio,
        time_ratio,
        size_ratio * LINEAR_MARGIN,
    );
}

#[test]
fn parser_scales_linearly_with_token_count() {
    use std::time::Instant;

    let small_source = read_v2_file("src/v1/ownership.dag");
    let large_source = read_v2_file("src/v1/02_parse.dag");

    let small_tokens = tokenize(&small_source);
    let large_tokens = tokenize(&large_source);

    let start = Instant::now();
    let _small_result = v1_compiler::v1_compiler_parse::parse(
        small_tokens.clone(),
        Rc::new(im_rc::HashMap::new()),
    );
    let small_time = start.elapsed();

    let start = Instant::now();
    let _large_result = v1_compiler::v1_compiler_parse::parse(
        large_tokens.clone(),
        Rc::new(im_rc::HashMap::new()),
    );
    let large_time = start.elapsed();

    let token_ratio = large_tokens.len() as f64 / small_tokens.len() as f64;
    let time_ratio = large_time.as_secs_f64() / small_time.as_secs_f64().max(0.001);

    eprintln!(
        "parse: small: {} tokens, {:.3}s | large: {} tokens, {:.3}s | token ratio: {:.1}x, time ratio: {:.1}x",
        small_tokens.len(), small_time.as_secs_f64(),
        large_tokens.len(), large_time.as_secs_f64(),
        token_ratio, time_ratio,
    );

    assert!(
        time_ratio < token_ratio * 2.0,
        "parsing appears super-linear: token ratio {:.1}x but time ratio {:.1}x (expected < {:.1}x)",
        token_ratio, time_ratio, token_ratio * 2.0,
    );
}

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
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShKeyword) && t.text == "fn"),
        "should contain keyword 'fn' token"
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShIdent)),
        "should contain Ident token"
    );
}

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
#[ignore = "40s — hanging in parser; triage under PERF track"]
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
        "dag/std/types.dag",
        "dag/std/error_primitives.dag",
        "dag/std/resources.dag",
        "dag/extdeps/cloud/cloud.dag",
        "dag/extdeps/cloud/gcp/errors.dag",
        "dag/extdeps/cloud/gcp/gcp.dag",
        "dag/extdeps/cloud/gcp/secret_manager.dag",
        "dag/extdeps/github/errors.dag",
        "dag/extdeps/github/github.dag",
        "dag/extdeps/github/auth.dag",
        "dag/extdeps/github/gists.dag",
        "dag/extdeps/git/git.dag",
        "dag/gunbc/auth/credentials.dag",
    ];
    for path in &files {
        assert_parses_strict(path);
    }
}

#[test]
fn keyword_as_field_name_allowed() {
    let keywords = [
        "type",
        "fn",
        "func",
        "module",
        "import",
        "service",
        "resource",
        "data",
        "interface",
        "pipeline",
        "pattern",
        "profile",
        "let",
        "return",
        "match",
        "if",
        "else",
        "for",
        "in",
        "where",
        "with",
        "capability",
        "operation",
        "input",
        "output",
        "idempotent",
        "readonly",
        "hermetic",
    ];
    for kw in &keywords {
        let source = format!("module test\ntype Rec {{ {}: String }}", kw);
        let result = parse_source(&source);
        assert!(
            result.error.is_none(),
            "keyword '{}' should be allowed as field name, got error: {:?}",
            kw,
            result
                .error
                .as_ref()
                .map(|e| { v1_compiler::v1_std_core::diagnostic_to_message(e.diagnostic.clone()) })
        );
    }
}

#[test]
fn keyword_as_field_name_forbidden() {
    let forbidden = ["true", "false", "none", "null", "acquire", "release"];
    for kw in &forbidden {
        let source = format!("module test\ntype Rec {{ {}: String }}", kw);
        let result = parse_source(&source);
        assert!(
            result.error.is_some(),
            "keyword '{}' should NOT be allowed as field name, but parse succeeded",
            kw,
        );
    }
}
