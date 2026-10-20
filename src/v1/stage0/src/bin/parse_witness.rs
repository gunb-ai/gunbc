#![allow(clippy::disallowed_macros)]

//! Host-physics floor witness for parse/tokenize oracle suite
//! (migrated from `src/v1/tests/src/parse.rs`).
//! Exercises syntax parsing, span grounding, strict corpus parses, tokenizer
//! behavior, and lookup/performance invariants until v1.compiler.parse/tokenize
//! are witness-layer importable.

use im::HashMap;
use std::process::ExitCode;
use std::rc::Rc;
use std::time::Instant;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::std_types::SourceSpan;
use v1_compiler::v1_compiler_parse::ParseResult;
use v1_compiler::v1_compiler_tokenize::{source_code_point, source_len, SourceRef};
use v1_compiler::v1_rt::{reset_text_lookup_chars_walked, take_text_lookup_chars_walked};
use v1_compiler::v1_std_core::{
    build_newline_index, diagnostic_to_message, diagnostic_to_span, source_text_at, InferredNode,
    Token, TokenShape,
};

type WitnessCase = (&'static str, fn());

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("parse_witness: {msg}");
    ExitCode::from(1)
}

fn read_v2_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
}

fn tokenize(source: &str) -> Rc<im::Vector<Rc<Token>>> {
    v1_compiler::v1_compiler_tokenize::tokenize(
        source.to_string(),
        "test.dag".to_string(),
        v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment(),
    )
}

fn parse_source(source: &str) -> Rc<ParseResult> {
    let tokens = v1_compiler::v1_compiler_tokenize::tokenize(
        source.to_string(),
        "test.dag".to_string(),
        v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let source_index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert("test.dag".to_string(), source_index);
    v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices))
}

fn assert_parses(source: &str, label: &str) {
    let result = parse_source(source);
    if let Some(ref err) = result.error {
        panic!(
            "{} had parse error: {}",
            label,
            diagnostic_to_message(err.diagnostic.clone())
        );
    }
    assert!(result.module.is_some(), "{} produced no module", label);
}

fn assert_parses_strict(relative_path: &str) {
    let source = read_v2_file(relative_path);
    let result = parse_source(&source);
    if let Some(ref err) = result.error {
        let span = {
            let s = diagnostic_to_span(err.diagnostic.clone());
            let line = source[..s.start.max(0) as usize]
                .chars()
                .filter(|c| *c == '\n')
                .count()
                + 1;
            format!(" (line {})", line)
        };
        panic!(
            "{} had parse error: {}{}",
            relative_path,
            diagnostic_to_message(err.diagnostic.clone()),
            span
        );
    }
    assert!(
        result.module.is_some(),
        "{} produced no module and no error",
        relative_path
    );
}

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
    let chars = Rc::new(source.chars().map(|c| c as i64).collect::<im::Vector<_>>());
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

fn pipe_syntax() {
    let source = r#"module test
fn foo(items: List<Int>) -> Int {
  items |> count
}"#;
    assert_parses(source, "pipe_syntax");
}

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

fn parse_generic_fn() {
    let source = r#"module test
fn identity<T>(x: T) -> T {
  x
}"#;
    assert_parses(source, "generic fn");
}

fn parse_multi_type_param_fn() {
    let source = r#"module test
fn fold_stack<T, B>(stack: List<T>, init: B, f: fn(B, T) -> B) -> B {
  init
}"#;
    assert_parses(source, "multi-type-param generic fn");
}

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

fn parse_fn_brace_body_still_accepted() {
    let source = r#"module test
fn add(x: Int, y: Int) -> Int { x + y }
"#;
    assert_parses(source, "brace-bodied fn (legacy)");
}

// DIFFERENTIAL (H2): the text a parsed node CARRIES in ident_span is the SOURCE SLICE at that span,
// for every reachable node. Two independent referents: authored_name_at (the legacy reader the
// carried text replaces) and a slice cut from the source string here, by scalar offsets, with no
// compiler code. The structural `name` is NOT a referent: it differs from the slice wherever the
// parser normalizes (a dotted path is joined without the whitespace around its dots) or decodes (a
// quoted key's content has lost its quotes and escapes), and the carried text must keep the
// spelling. The source puts a non-BMP scalar before the identifiers, because spans index Unicode
// scalars and a UTF-16 or byte offset would shift the slice.
type CoreNode = Rc<v1_compiler::v1_std_core::Node>;

#[derive(Default)]
struct Reach {
    children: usize,
    params: usize,
    uses: usize,
    body: usize,
    transport: usize,
    properties: usize,
    type_annotation: usize,
    inferred: usize,
    match_pattern: usize,
}

// Every node-bearing field of Node, so no node the parser built is outside the comparison.
fn reachable_nodes(n: &CoreNode, reach: &mut Reach, out: &mut Vec<CoreNode>) {
    use v1_compiler::v1_std_core::MatchPattern;
    out.push(n.clone());
    for c in n.children.iter() {
        reach.children += 1;
        reachable_nodes(c, reach, out);
    }
    for c in n.params.iter() {
        reach.params += 1;
        reachable_nodes(c, reach, out);
    }
    for c in n.uses.iter() {
        reach.uses += 1;
        reachable_nodes(c, reach, out);
    }
    for c in n.properties.iter() {
        reach.properties += 1;
        reachable_nodes(c, reach, out);
    }
    if let Some(c) = n.body.as_ref() {
        reach.body += 1;
        reachable_nodes(c, reach, out);
    }
    if let Some(c) = n.transport.as_ref() {
        reach.transport += 1;
        reachable_nodes(c, reach, out);
    }
    if let Some(c) = n.type_annotation.as_ref() {
        reach.type_annotation += 1;
        reachable_nodes(c, reach, out);
    }
    if let Some(i) = n.inferred.as_ref() {
        if let InferredNode::Resolved { node } = &**i {
            reach.inferred += 1;
            reachable_nodes(node, reach, out);
        }
    }
    if let Some(p) = n.match_pattern.as_ref() {
        match &**p {
            MatchPattern::Bind { declaration } => {
                reach.match_pattern += 1;
                reachable_nodes(declaration, reach, out);
            }
            MatchPattern::VariantPattern { field_bindings, .. } => {
                for c in field_bindings.iter() {
                    reach.match_pattern += 1;
                    reachable_nodes(c, reach, out);
                }
            }
            _ => {}
        }
    }
}

fn indices_of(source: &str) -> Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>> {
    let index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut m = HashMap::new();
    m.insert("test.dag".to_string(), index);
    Rc::new(m)
}

// The slice of `source` at a span, by Unicode scalar offsets.
fn source_slice(source: &str, span: &SourceSpan) -> String {
    source
        .chars()
        .skip(span.start as usize)
        .take((span.end - span.start) as usize)
        .collect()
}

// The comparison the differential runs at every node: the carried text equals the source slice
// at the node's span, and equals what authored_name_at re-derives there.
fn carried_text_matches_source(
    indices: &Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
    source: &str,
    n: &CoreNode,
) -> bool {
    let ident = n.ident_span.as_ref().expect("node carries an ident");
    // A kernel or synthetic identity has no source span (the empty <synthetic> span): its contract
    // is the node's own structural name, not a source slice.
    if ident.span.file == "<synthetic>" {
        return ident.text == n.name;
    }
    ident.text == source_slice(source, &ident.span)
        && ident.text == v1_compiler::v1_std_core::authored_name_at(indices.clone(), n.clone())
}

fn parse_all_nodes(
    source: &str,
) -> (
    Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
    Vec<CoreNode>,
    Reach,
) {
    let result = parse_source(source);
    assert!(result.error.is_none(), "parse error");
    let module = result.module.clone().expect("module");
    let mut reach = Reach::default();
    let mut nodes = Vec::new();
    reachable_nodes(&module, &mut reach, &mut nodes);
    (indices_of(source), nodes, reach)
}

fn with_carried_text(n: &CoreNode, text: &str) -> CoreNode {
    Rc::new(v1_compiler::v1_std_core::Node {
        ident_span: n
            .ident_span
            .as_ref()
            .map(|i| v1_compiler::v1_std_core::synthetic_ident(i.span.clone(), text.to_string())),
        ..(**n).clone()
    })
}

fn carried_ident_text_equals_authored_name_at() {
    let source = "module test\n// \u{1F600}\ntype Widget = String\nfn \u{1F600}_pad() -> Int { 1 }\nfn make_widget(w: Widget) -> Widget {\n  w\n}\nservice weather.api {\n}";
    let (indices, nodes, _) = parse_all_nodes(source);
    let mut checked = 0;
    for n in &nodes {
        if n.ident_span.is_some() {
            assert!(
                carried_text_matches_source(&indices, source, n),
                "carried text differs at node {}",
                n.name
            );
            checked += 1;
        }
    }
    assert!(checked >= 6, "differential reached only {} nodes", checked);
    let victim = nodes
        .iter()
        .find(|n| n.name == "Widget" && n.ident_span.is_some())
        .expect("Widget node")
        .clone();
    assert!(
        !carried_text_matches_source(&indices, source, &with_carried_text(&victim, "Wrong")),
        "RED control did not discriminate"
    );
}

// The traversal reaches every node-bearing field: a fixture that populates children, params,
// properties, body, transport, inferred and match patterns, and a count per field proving each
// was walked. A walk over children and params alone left every node under the others uncompared.
fn carried_text_differential_reaches_every_node_bearing_field() {
    let source = "module test\n\
type Color = Red | Green\n\
type Shape = Circle { r: Int } | Dot\n\
fn radius(s: Shape) -> Int {\n  match s {\n    Circle { r } => r\n    Dot => 0\n  }\n}\n\
type Row { stdout: String from \"std\\tout\\n\" }\n\
fn is_red(c: Color) -> Bool {\n  match c {\n    Red => true\n    Green => false\n  }\n}\n\
service cron.Tab {\n  operation List {\n    requires none\n    input {}\n    output { entries: String from \"stdout\" }\n    readonly\n    transport shell { argv: [\"crontab\", \"-l\"] }\n  }\n}\n";
    let (indices, nodes, reach) = parse_all_nodes(source);
    for (field, count) in [
        ("children", reach.children),
        ("params", reach.params),
        ("body", reach.body),
        ("transport", reach.transport),
        ("properties", reach.properties),
        ("inferred", reach.inferred),
        ("match_pattern", reach.match_pattern),
    ] {
        assert!(
            count > 0,
            "the traversal never reached a node through {field}"
        );
    }
    let mut checked = 0;
    for n in &nodes {
        if n.ident_span.is_some() {
            assert!(
                carried_text_matches_source(&indices, source, n),
                "carried text differs at node {}",
                n.name
            );
            checked += 1;
        }
    }
    assert!(checked >= 12, "differential reached only {} nodes", checked);
}

// A dotted name is carried as written: the whitespace around its dots is part of the spelling the
// identifier span covers, while the structural name is the segments joined by '.'. Module header,
// import path, service name and a non-applied qualified type name all take that path.
fn carried_text_keeps_the_spelling_of_dotted_names() {
    let source = "module a . b\nimport v1 . std . core { Foo }\nservice cron . Tab {\n}\nfn f(x: std . Foo) -> Int { 1 }\n";
    let (indices, nodes, _) = parse_all_nodes(source);
    for (structural, written) in [
        ("a.b", "a . b"),
        ("v1.std.core", "v1 . std . core"),
        ("cron.Tab", "cron . Tab"),
        ("std.Foo", "std . Foo"),
    ] {
        let n = nodes
            .iter()
            .find(|n| {
                n.name == structural
                    && n.ident_span.as_ref().map(|i| i.text.as_str()) == Some(written)
            })
            .unwrap_or_else(|| panic!("no node {structural} carrying `{written}`"))
            .clone();
        assert!(carried_text_matches_source(&indices, source, &n));
        // RED: the old carried value, the structural name, is not the source slice here.
        assert!(
            !carried_text_matches_source(&indices, source, &with_carried_text(&n, structural)),
            "RED control did not discriminate for {structural}"
        );
    }
}

// A quoted property key is carried as its full quoted token, quotes and escapes included; the
// structural name is the decoded content. The RED tampers the carried text to the decoded content.
fn carried_text_keeps_the_spelling_of_quoted_keys_with_escapes() {
    let source = "module test\ntype Row { stdout: String from \"std\\tout\\n\" }\n";
    let (indices, nodes, _) = parse_all_nodes(source);
    let written = "\"std\\tout\\n\"";
    let n = nodes
        .iter()
        .find(|n| n.ident_span.as_ref().map(|i| i.text.as_str()) == Some(written))
        .unwrap_or_else(|| panic!("no node carrying {written}"))
        .clone();
    assert_ne!(
        n.name, written,
        "the structural name must be the decoded key"
    );
    assert!(carried_text_matches_source(&indices, source, &n));
    assert!(
        !carried_text_matches_source(&indices, source, &with_carried_text(&n, &n.name)),
        "RED control did not discriminate for the decoded key"
    );
}

// A rebuild COPIES the identity of the node it rebuilds (C4): node_ident returns the carried ident
// itself. A rebuild that recomputed through the legacy reader would re-read the source (or fall back
// to the structural name) and so would not reproduce a carried text that differs from that reading;
// the tampered node is that discriminating input, and the copy must survive it.
fn rebuild_copies_the_carried_ident_instead_of_recomputing() {
    let source = "module test\ntype Widget = String\n";
    let (indices, nodes, _) = parse_all_nodes(source);
    let victim = nodes
        .iter()
        .find(|n| n.name == "Widget" && n.ident_span.is_some())
        .expect("Widget node")
        .clone();
    let tampered = with_carried_text(&victim, "Carried");
    let copied = v1_compiler::v1_std_core::node_ident(tampered.clone())
        .expect("a node carrying an identity copies it");
    assert_eq!(
        copied.text, "Carried",
        "node_ident did not copy the carried text"
    );
    assert_eq!(copied.span, tampered.ident_span.as_ref().unwrap().span);
    // RED: the recompute path reads something else here, so a recomputing rebuild would be caught
    assert_ne!(
        v1_compiler::v1_std_core::authored_name_at(indices, tampered),
        "Carried",
        "the recompute path cannot be told apart from a copy"
    );
}

// Absence stays absent through a rebuild: a node with no identity must not acquire one, from its
// structural name or from anything else. RED: a copy that filled the gap would be Some here.
fn rebuild_keeps_an_absent_identity_absent() {
    let source = "module test\ntype Widget = String\n";
    let (_, nodes, _) = parse_all_nodes(source);
    let victim = nodes
        .iter()
        .find(|n| n.name == "Widget" && n.ident_span.is_some())
        .expect("Widget node")
        .clone();
    let stripped = Rc::new(v1_compiler::v1_std_core::Node {
        ident_span: None,
        ..(*victim).clone()
    });
    assert!(
        !stripped.name.is_empty(),
        "control: the node still has a structural name to fall back on"
    );
    assert!(
        v1_compiler::v1_std_core::node_ident(stripped.clone()).is_none(),
        "node_ident gave an identity-less node an identity"
    );
    let rebuilt = v1_compiler::v1_std_core::leaf_node_with_span(
        stripped.occurrence_identity.clone(),
        stripped.name.clone(),
        stripped.span.clone(),
        v1_compiler::v1_std_core::node_ident(stripped.clone()),
    );
    assert!(
        rebuilt.ident_span.is_none(),
        "a rebuilt node with no identity acquired one"
    );
    // positive control: the same rebuild of a carrying node keeps its identity
    let kept = v1_compiler::v1_std_core::leaf_node_with_span(
        victim.occurrence_identity.clone(),
        victim.name.clone(),
        victim.span.clone(),
        v1_compiler::v1_std_core::node_ident(victim.clone()),
    );
    assert_eq!(kept.ident_span, victim.ident_span);
}

// A span whose file has no tokenized source has no obtainable text: construction refuses with a
// typed failure, naming the span, never the structural name and never the empty string. The same
// holds at the parse boundary, where every parsed identifier takes its text from that source.
fn source_backed_construction_refuses_without_the_source_authority() {
    use v1_compiler::v1_std_core::{source_index_at, SourceIndexOutcome};
    let span = Rc::new(SourceSpan {
        file: "test.dag".to_string(),
        start: 0,
        end: 6,
    });
    let empty: Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>> =
        Rc::new(im::HashMap::new());
    assert!(
        matches!(
            &*source_index_at(empty.clone(), span.clone()),
            SourceIndexOutcome::SourceIndexRefused { .. }
        ),
        "source_index_at substituted an index for a missing source"
    );
    // positive control: with the source indexed, the same span finds its authority
    let source = "module test\n";
    assert!(
        matches!(
            &*source_index_at(indices_of(source), span.clone()),
            SourceIndexOutcome::SourceIndexFound { .. }
        ),
        "source_index_at refused with the source indexed"
    );
    let tokens = v1_compiler::v1_compiler_tokenize::tokenize(
        source.to_string(),
        "test.dag".to_string(),
        v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let refused = v1_compiler::v1_compiler_parse::parse(tokens, empty);
    assert!(
        refused.module.is_none(),
        "a parse without source authority produced a module"
    );
    let err = refused.error.as_ref().expect("typed construction failure");
    assert!(
        diagnostic_to_message(err.diagnostic.clone()).contains("not indexed"),
        "the refusal does not name the missing source"
    );
}

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
    let type_ident = type_item
        .ident_span
        .clone()
        .expect("type ident span")
        .span
        .clone();
    assert_eq!(
        type_item.span.start,
        source.find("type Widget").unwrap() as i64
    );
    assert_eq!(type_ident.start, source.find("Widget").unwrap() as i64);
    assert!(type_ident.start > type_item.span.start);

    let fn_item = module.children[1].clone();
    let fn_ident = fn_item
        .ident_span
        .clone()
        .expect("fn ident span")
        .span
        .clone();
    assert_eq!(
        fn_item.span.start,
        source.find("fn make_widget").unwrap() as i64
    );
    assert_eq!(fn_ident.start, source.find("make_widget").unwrap() as i64);
    assert!(fn_ident.start > fn_item.span.start);

    let service_item = module.children[2].clone();
    let service_ident = service_item
        .ident_span
        .clone()
        .expect("service ident span")
        .span
        .clone();
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
    let rhs_ident = rhs.ident_span.clone().expect("rhs ident span").span.clone();
    assert_eq!(
        &source[rhs_ident.start as usize..rhs_ident.end as usize],
        "ResultType"
    );
    assert_eq!(rhs_ident.start, source.find("ResultType").unwrap() as i64);
    assert!(rhs_ident.start > alias_item.span.start);
}

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
    let key_ident = key_type
        .ident_span
        .clone()
        .expect("key type ident span")
        .span
        .clone();
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
        .expect("value type ident span")
        .span
        .clone();
    assert_eq!(
        &source[value_ident.start as usize..value_ident.end as usize],
        "ValueU"
    );
    assert_eq!(value_ident.start, source.find("ValueU").unwrap() as i64);
}

fn stack_parses_strict() {
    assert_parses_strict("dag/std/stack.dag");
}

fn core_parses_strict() {
    assert_parses_strict("src/v1/00_core.dag");
}

fn tokenize_parses_strict() {
    assert_parses_strict("src/v1/01_tokenize.dag");
}

fn parse_parses_strict() {
    assert_parses_strict("src/v1/02_parse.dag");
}

fn resolve_parses_strict() {
    assert_parses_strict("src/v1/03_resolve.dag");
}

fn typecheck_parses_strict() {
    assert_parses_strict("src/v1/04_infer.dag");
}

fn emit_parses_strict() {
    assert_parses_strict("src/v1/05_emit.dag");
}

fn pipeline_parses_strict() {
    assert_parses_strict("src/v1/compile.dag");
}

fn artifact_parses_strict() {
    assert_parses_strict("src/v1/artifact.dag");
}

fn complexity_parses_strict() {
    assert_parses_strict("src/v1/complexity.dag");
}

fn ownership_parses_strict() {
    assert_parses_strict("src/v1/ownership.dag");
}

fn shared_behavioral_parses_strict() {
    assert_parses_strict("dag/std/behavioral.dag");
}

fn shared_primitives_parses_strict() {
    assert_parses_strict("dag/std/primitives.dag");
}

fn tokenizer_non_ascii_performance_regression() {
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

fn parser_scales_linearly_with_token_count() {
    let small_source = read_v2_file("src/v1/ownership.dag");
    let large_source = read_v2_file("src/v1/02_parse.dag");

    let small_tokens = tokenize(&small_source);
    let large_tokens = tokenize(&large_source);

    let start = Instant::now();
    let _small_result =
        v1_compiler::v1_compiler_parse::parse(small_tokens.clone(), indices_of(&small_source));
    let small_time = start.elapsed();

    let start = Instant::now();
    let _large_result =
        v1_compiler::v1_compiler_parse::parse(large_tokens.clone(), indices_of(&large_source));
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

fn tokenizer_smoke() {
    let tokens = tokenize("fn add(a: Int) -> Int { a }");
    assert!(!tokens.is_empty(), "should produce at least one token");
}

fn tokenizer_empty_input() {
    let tokens = tokenize("");
    assert!(
        !tokens.is_empty(),
        "empty input should still produce Eof token"
    );
}

fn tokenizer_keywords() {
    let tokens = tokenize("module fn type import data");
    assert!(
        tokens.len() >= 5,
        "should produce at least 5 keyword tokens, got {}",
        tokens.len()
    );
}

fn tokenizer_two_char_operators() {
    let tokens = tokenize("== -> => !=");
    assert!(
        tokens.len() >= 4,
        "should produce at least 4 operator tokens, got {}",
        tokens.len()
    );
}

fn tokenizer_scans_pipe_arrow() {
    let tokens = tokenize("items |> count");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShPipeArrow)),
        "should contain PipeArrow token"
    );
}

fn tokenizer_scans_null_coalesce() {
    let tokens = tokenize("x ?? y");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.shape, TokenShape::ShNullCoalesce)),
        "should contain NullCoalesce token"
    );
}

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

fn parser_e2e() {
    let result = parse_source("module test");
    assert!(result.error.is_none(), "should parse 'module test'");
    let module = result.module.as_ref().expect("should produce module");
    assert_eq!(module.name, "test");
}

fn uppercase_condition_does_not_consume_then_branch_as_record_literal() {
    let source = "module test\nfn choose(flag: Bool) -> Int {\n  if Present { 1 } else { 2 }\n}";
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "uppercase condition should leave the following brace for the then-branch: {:?}",
        result.error
    );
}

fn uppercase_rhs_condition_does_not_consume_then_branch_as_record_literal() {
    let source =
        "module test\nfn choose(flag: Bool) -> Int {\n  if flag == Present { 1 } else { 2 }\n}";
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "uppercase condition RHS should leave the following brace for the then-branch: {:?}",
        result.error
    );
}

fn parse_real_source() {
    let source =
        "module test\ntype Foo { x: Int }\ntype Bar = Foo\nfn identity(x: Int) -> Int { x }";
    let result = parse_source(source);
    assert!(result.error.is_none(), "should parse multi-item source");
    let module = result.module.as_ref().expect("should produce module");
    assert!(!module.children.is_empty(), "should have items");
}

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

fn parse_fn_lambda_in_call_arg() {
    let source = "module test\nfn transform(items: List<Int>) -> List<Int> {\n  fold(init: [], f: fn(acc, item) {\n    append(acc, items: item)\n  })\n}";
    let result = parse_source(source);
    assert!(
        result.error.is_none(),
        "fn lambda in call arg should parse: {:?}",
        result.error
    );
}

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
        "dag/extdeps/git.dag",
        "dag/gunbc/auth/credentials.dag",
    ];
    for path in &files {
        assert_parses_strict(path);
    }
}

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
                .map(|e| { diagnostic_to_message(e.diagnostic.clone()) })
        );
    }
}

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

fn floor_suite() -> Vec<WitnessCase> {
    vec![
        ("fn_lambda_syntax", fn_lambda_syntax),
        ("pipe_syntax", pipe_syntax),
        ("multi_stmt_if_body", multi_stmt_if_body),
        ("match_with_variant_construct", match_with_variant_construct),
        ("fold_with_fn_lambda_and_pipe", fold_with_fn_lambda_and_pipe),
        ("nested_match_with_pipe", nested_match_with_pipe),
        ("implicit_block_match_arms", implicit_block_match_arms),
        (
            "typecheck_match_with_itemresult",
            typecheck_match_with_itemresult,
        ),
        ("parse_generic_fn", parse_generic_fn),
        ("parse_multi_type_param_fn", parse_multi_type_param_fn),
        ("parse_fn_expression_body", parse_fn_expression_body),
        (
            "parse_fn_brace_body_still_accepted",
            parse_fn_brace_body_still_accepted,
        ),
        (
            "carried_ident_text_equals_authored_name_at",
            carried_ident_text_equals_authored_name_at,
        ),
        (
            "carried_text_differential_reaches_every_node_bearing_field",
            carried_text_differential_reaches_every_node_bearing_field,
        ),
        (
            "carried_text_keeps_the_spelling_of_dotted_names",
            carried_text_keeps_the_spelling_of_dotted_names,
        ),
        (
            "carried_text_keeps_the_spelling_of_quoted_keys_with_escapes",
            carried_text_keeps_the_spelling_of_quoted_keys_with_escapes,
        ),
        (
            "rebuild_copies_the_carried_ident_instead_of_recomputing",
            rebuild_copies_the_carried_ident_instead_of_recomputing,
        ),
        (
            "rebuild_keeps_an_absent_identity_absent",
            rebuild_keeps_an_absent_identity_absent,
        ),
        (
            "source_backed_construction_refuses_without_the_source_authority",
            source_backed_construction_refuses_without_the_source_authority,
        ),
        (
            "item_ident_spans_point_at_identifiers_not_keywords",
            item_ident_spans_point_at_identifiers_not_keywords,
        ),
        (
            "module_node_ident_is_populated_by_parser",
            module_node_ident_is_populated_by_parser,
        ),
        (
            "type_alias_rhs_ident_span_points_at_authored_type_name",
            type_alias_rhs_ident_span_points_at_authored_type_name,
        ),
        (
            "type_param_spans_point_at_identifiers_not_delimiters",
            type_param_spans_point_at_identifiers_not_delimiters,
        ),
        ("stack_parses_strict", stack_parses_strict),
        ("core_parses_strict", core_parses_strict),
        ("tokenize_parses_strict", tokenize_parses_strict),
        ("parse_parses_strict", parse_parses_strict),
        ("resolve_parses_strict", resolve_parses_strict),
        ("typecheck_parses_strict", typecheck_parses_strict),
        ("emit_parses_strict", emit_parses_strict),
        ("pipeline_parses_strict", pipeline_parses_strict),
        ("artifact_parses_strict", artifact_parses_strict),
        ("complexity_parses_strict", complexity_parses_strict),
        ("ownership_parses_strict", ownership_parses_strict),
        (
            "shared_behavioral_parses_strict",
            shared_behavioral_parses_strict,
        ),
        (
            "shared_primitives_parses_strict",
            shared_primitives_parses_strict,
        ),
        ("tokenizer_smoke", tokenizer_smoke),
        ("tokenizer_empty_input", tokenizer_empty_input),
        ("tokenizer_keywords", tokenizer_keywords),
        ("tokenizer_two_char_operators", tokenizer_two_char_operators),
        ("tokenizer_scans_pipe_arrow", tokenizer_scans_pipe_arrow),
        (
            "tokenizer_scans_null_coalesce",
            tokenizer_scans_null_coalesce,
        ),
        (
            "tokenize_produces_correct_kinds",
            tokenize_produces_correct_kinds,
        ),
        ("parser_e2e", parser_e2e),
        (
            "uppercase_condition_does_not_consume_then_branch_as_record_literal",
            uppercase_condition_does_not_consume_then_branch_as_record_literal,
        ),
        (
            "uppercase_rhs_condition_does_not_consume_then_branch_as_record_literal",
            uppercase_rhs_condition_does_not_consume_then_branch_as_record_literal,
        ),
        ("parse_real_source", parse_real_source),
        ("parse_fold_with_fn_lambda", parse_fold_with_fn_lambda),
        ("parse_fn_lambda_in_call_arg", parse_fn_lambda_in_call_arg),
        (
            "gist_transitive_closure_parse",
            gist_transitive_closure_parse,
        ),
        (
            "keyword_as_field_name_allowed",
            keyword_as_field_name_allowed,
        ),
        (
            "keyword_as_field_name_forbidden",
            keyword_as_field_name_forbidden,
        ),
    ]
}

fn perf_suite() -> Vec<WitnessCase> {
    vec![
        (
            "tokenizer_non_ascii_performance_regression",
            tokenizer_non_ascii_performance_regression,
        ),
        (
            "tokenizer_text_lookup_flat_in_file_size",
            tokenizer_text_lookup_flat_in_file_size,
        ),
        (
            "source_text_at_lookup_flat_in_file_size",
            source_text_at_lookup_flat_in_file_size,
        ),
        (
            "source_text_at_lookup_flat_in_file_padding",
            source_text_at_lookup_flat_in_file_padding,
        ),
        (
            "tokenizer_scales_linearly_with_file_size",
            tokenizer_scales_linearly_with_file_size,
        ),
        (
            "tokenizer_scanning_scales_linearly",
            tokenizer_scanning_scales_linearly,
        ),
        (
            "parser_scales_linearly_with_token_count",
            parser_scales_linearly_with_token_count,
        ),
    ]
}

fn all_suite() -> Vec<WitnessCase> {
    let mut tests = floor_suite();
    tests.extend(perf_suite());
    tests
}

fn run_suite(tests: &[WitnessCase]) -> ExitCode {
    for (name, test) in tests {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let suite = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "--suite".to_string());
    let suite_name = if suite == "--suite" {
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "floor".to_string())
    } else {
        suite
    };

    let tests: Vec<WitnessCase> = match suite_name.as_str() {
        "floor" => floor_suite(),
        "perf" => perf_suite(),
        "all" => all_suite(),
        other => return fail(format!("unknown suite {other:?}; expected floor|perf|all")),
    };

    run_suite(&tests)
}
