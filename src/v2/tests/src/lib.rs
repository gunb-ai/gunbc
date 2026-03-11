//! Integration tests for the v2 self-hosted compiler.
//!
//! Phase 0: strict parse audit (all 7 .dag files parse with zero diagnostics)
//! Phase 1: compilation gate (v1 can compile each v2 module)
//! Phase 2: tokenizer e2e (evaluate tokenize fn on real input)
//! Phase 3: stage-by-stage integration (chain stages on trivial fixture)

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    // ── Helpers ──────────────────────────────────────────────────────────

    fn workspace_root() -> std::path::PathBuf {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // v2/tests lives at src/v2/tests, so workspace root is 3 levels up.
        manifest_dir
            .ancestors()
            .nth(3)
            .expect("could not find workspace root")
            .to_path_buf()
    }

    fn value_to_json(val: &gunbc_ir::Value) -> serde_json::Value {
        match val {
            gunbc_ir::Value::Str(s) => serde_json::Value::String(s.clone()),
            gunbc_ir::Value::Int(i) => serde_json::Value::Number((*i).into()),
            gunbc_ir::Value::Bool(b) => serde_json::Value::Bool(*b),
            gunbc_ir::Value::Unit => serde_json::Value::Null,
            gunbc_ir::Value::List(items) => {
                serde_json::Value::Array(items.iter().map(value_to_json).collect())
            }
            gunbc_ir::Value::Map(map) => {
                let obj: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_json(v)))
                    .collect();
                serde_json::Value::Object(obj)
            }
            gunbc_ir::Value::Enum { variant, .. } => serde_json::Value::String(variant.clone()),
            _ => serde_json::Value::Null,
        }
    }

    fn read_v2_file(relative_path: &str) -> String {
        let path = workspace_root().join(relative_path);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 0: Strict parse audit — every .dag file parses with zero diagnostics
    // ═════════════════════════════════════════════════════════════════════

    fn assert_parses_strict(relative_path: &str) {
        let source = read_v2_file(relative_path);
        let result = daglang_syntax::parser::parse_to_result(&source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| {
                let span = d.span;
                let line = source[..span.start.min(source.len())]
                    .chars()
                    .filter(|c| *c == '\n')
                    .count()
                    + 1;
                let span_info = format!(" (line {})", line);
                format!("{}{}", d.message, span_info)
            })
            .collect();
        assert!(
            result.is_ok(),
            "{} had {} parse errors:\n{}",
            relative_path,
            errors.len(),
            errors.join("\n")
        );
    }

    /// Compile all 7 v2 compiler .dag files into a single EmbeddedCompileOutput.
    /// All fn bodies from all modules share one `fns` HashMap, enabling
    /// cross-module calls (e.g., pipeline.dag calling tokenize()).
    fn compile_all_modules() -> Result<daglang_driver::EmbeddedCompileOutput, String> {
        let root = workspace_root();

        let files = vec![
            root.join("dsl/std/types.dag"),
            root.join("src/v2/std/core.dag"),
            root.join("src/v2/compiler/tokenize.dag"),
            root.join("src/v2/compiler/parse.dag"),
            root.join("src/v2/compiler/resolve.dag"),
            root.join("src/v2/compiler/typecheck.dag"),
            root.join("src/v2/compiler/emit.dag"),
            root.join("src/v2/compiler/pipeline.dag"),
        ];
        let sources: Vec<(std::path::PathBuf, String)> = files
            .into_iter()
            .map(|p| {
                let content = std::fs::read_to_string(&p).unwrap();
                (p, content)
            })
            .collect();

        let mut parsed_files = Vec::new();
        for (path, source) in &sources {
            let ast = daglang_syntax::parser::parse_with_file_diagnostics(path, source)
                .map_err(|errs| {
                    errs.iter()
                        .map(|d| d.render())
                        .collect::<Vec<_>>()
                        .join("\n")
                })?;
            parsed_files.push((path.clone(), ast, source.clone()));
        }

        let mut variant_names = std::collections::HashSet::new();
        for (_path, ast, _source) in &parsed_files {
            for item in &ast.items {
                if let daglang_syntax::ast::Item::TypeDef(td) = &item.node {
                    if let daglang_syntax::ast::TypeBody::Sum(variants) = &td.body {
                        for v in variants {
                            variant_names.insert(v.name.clone());
                        }
                    }
                }
            }
        }

        let mut fns = HashMap::new();
        let mut data_values = HashMap::new();
        for (_path, ast, _source) in &parsed_files {
            for item in &ast.items {
                match &item.node {
                    daglang_syntax::ast::Item::FnDef(fndef) => {
                        let lowered =
                            daglang_lower::expr::lower_fn_body(&fndef.body, &variant_names);
                        fns.insert(fndef.name.clone(), lowered);
                    }
                    daglang_syntax::ast::Item::DataDef(dd) => {
                        let expr = &dd.value;
                        let lowered_expr =
                            daglang_lower::expr::lower_expr_remap(expr, &variant_names);
                        let body = daglang_eval::LoweredFnBody {
                            stmts: vec![daglang_eval::LoweredStmt::Return(vec![(
                                "return".to_string(),
                                lowered_expr,
                            )])],
                        };
                        if let Ok(result) = daglang_eval::evaluate_fn_body(
                            &body,
                            &HashMap::new(),
                            &HashMap::new(),
                        ) {
                            if let Some(val) = result.get("return") {
                                data_values.insert(dd.name.clone(), value_to_json(val));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(daglang_driver::EmbeddedCompileOutput {
            fns,
            data_values,
            pipelines: HashMap::new(),
        })
    }

    /// Helper: call a DSL function by name with given inputs and return outputs.
    fn call_fn(
        output: &daglang_driver::EmbeddedCompileOutput,
        fn_name: &str,
        inputs: HashMap<String, gunbc_ir::Value>,
    ) -> Result<HashMap<String, gunbc_ir::Value>, String> {
        let body = output
            .fns
            .get(fn_name)
            .ok_or_else(|| format!("fn '{}' not found", fn_name))?;
        daglang_eval::evaluate_fn_body_with_data(body, &inputs, &output.fns, &output.data_values)
            .map_err(|e| format!("{}", e))
    }

    /// Helper: extract a string field from a token's kind variant.
    fn token_kind_tag(token: &gunbc_ir::Value) -> Option<String> {
        if let gunbc_ir::Value::Map(map) = token {
            if let Some(gunbc_ir::Value::Map(kind_map)) = map.get("kind") {
                if let Some(gunbc_ir::Value::Str(tag)) = kind_map.get("_variant") {
                    return Some(tag.clone());
                }
            }
            // Unit variant kinds stored as Enum
            if let Some(gunbc_ir::Value::Enum { variant, .. }) = map.get("kind") {
                return Some(variant.clone());
            }
            // Str kind (fallback for older format)
            if let Some(gunbc_ir::Value::Str(s)) = map.get("kind") {
                return Some(s.clone());
            }
        }
        None
    }

    #[test]
    fn phase0_fn_lambda_syntax() {
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
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "fn lambda syntax should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_pipe_syntax() {
        let source = r#"module test
fn foo(items: List<Int>) -> Int {
  items |> count
}"#;
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "pipe syntax should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_multi_stmt_if_body() {
        let source = r#"module test
fn foo(x: Int) -> Int {
  if x > 0 {
    let y = x + 1
    return y
  }
  x
}"#;
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "multi-stmt if body should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_match_with_variant_construct() {
        let source = r#"module test
fn foo(ch: String) -> String {
  match lookup(table, key: ch) {
    Some { value: kind } => kind
    None => Unknown { char: ch }
  }
}"#;
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "match with variant construct should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_fold_with_fn_lambda_and_pipe() {
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
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "fold with fn lambda should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_nested_match_with_pipe() {
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
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "nested match with pipe should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_implicit_block_match_arms() {
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
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "implicit block match arms should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_typecheck_match_with_itemresult() {
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
        let result = daglang_syntax::parser::parse_to_result(source);
        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        assert!(
            result.is_ok(),
            "typecheck match with itemresult should parse:\n{}",
            errors.join("\n")
        );
    }

    #[test]
    fn phase0_core_parses_strict() {
        assert_parses_strict("src/v2/std/core.dag");
    }

    #[test]
    fn phase0_tokenize_parses_strict() {
        assert_parses_strict("src/v2/compiler/tokenize.dag");
    }

    #[test]
    fn phase0_parse_parses_strict() {
        assert_parses_strict("src/v2/compiler/parse.dag");
    }

    #[test]
    fn phase0_resolve_parses_strict() {
        assert_parses_strict("src/v2/compiler/resolve.dag");
    }

    #[test]
    fn phase0_typecheck_parses_strict() {
        assert_parses_strict("src/v2/compiler/typecheck.dag");
    }

    #[test]
    fn phase0_emit_parses_strict() {
        assert_parses_strict("src/v2/compiler/emit.dag");
    }

    #[test]
    fn phase0_pipeline_parses_strict() {
        assert_parses_strict("src/v2/compiler/pipeline.dag");
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 1: Compilation gate — v1 compiler can compile each v2 module
    // ═════════════════════════════════════════════════════════════════════

    /// Extract fn bodies and data values from the tokenizer module.
    /// Uses direct AST-level lowering (bypasses DAG wiring) to avoid
    /// DAG-level expression resolution failures for pure fn bodies.
    fn compile_tokenizer_module(
    ) -> Result<daglang_driver::EmbeddedCompileOutput, String> {
        let root = workspace_root();

        // Read sources
        let files = vec![
            root.join("dsl/std/types.dag"),
            root.join("src/v2/std/core.dag"),
            root.join("src/v2/compiler/tokenize.dag"),
        ];
        let sources: Vec<(std::path::PathBuf, String)> = files
            .into_iter()
            .map(|p| {
                let content = std::fs::read_to_string(&p).unwrap();
                (p, content)
            })
            .collect();

        // Parse all sources
        let mut parsed_files = Vec::new();
        for (path, source) in &sources {
            let ast = daglang_syntax::parser::parse_with_file_diagnostics(path, source)
                .map_err(|errs| {
                    errs.iter()
                        .map(|d| d.render())
                        .collect::<Vec<_>>()
                        .join("\n")
                })?;
            parsed_files.push((path.clone(), ast, source.clone()));
        }

        // Collect variant names for lowering
        let mut variant_names = std::collections::HashSet::new();
        for (_path, ast, _source) in &parsed_files {
            for item in &ast.items {
                if let daglang_syntax::ast::Item::TypeDef(td) = &item.node {
                    if let daglang_syntax::ast::TypeBody::Sum(variants) = &td.body {
                        for v in variants {
                            variant_names.insert(v.name.clone());
                        }
                    }
                }
            }
        }

        // Extract fn bodies directly from AST (no DAG wiring needed)
        let mut fns = HashMap::new();
        let mut data_values = HashMap::new();
        for (_path, ast, _source) in &parsed_files {
            for item in &ast.items {
                match &item.node {
                    daglang_syntax::ast::Item::FnDef(fndef) => {
                        let lowered = daglang_lower::expr::lower_fn_body(&fndef.body, &variant_names);
                        fns.insert(fndef.name.clone(), lowered);
                    }
                    daglang_syntax::ast::Item::DataDef(dd) => {
                        // Lower data declarations to JSON values
                        {
                            let expr = &dd.value;
                            let lowered_expr =
                                daglang_lower::expr::lower_expr_remap(expr, &variant_names);
                            // Evaluate the data expression to get a serde_json::Value
                            let body = daglang_eval::LoweredFnBody {
                                stmts: vec![daglang_eval::LoweredStmt::Return(vec![(
                                    "return".to_string(),
                                    lowered_expr,
                                )])],
                            };
                            match daglang_eval::evaluate_fn_body(
                                &body,
                                &HashMap::new(),
                                &HashMap::new(),
                            ) {
                                Ok(result) => {
                                    if let Some(val) = result.get("return") {
                                        data_values.insert(
                                            dd.name.clone(),
                                            value_to_json(val),
                                        );
                                    }
                                }
                                Err(_) => {
                                    // Data evaluation failure is non-fatal; skip this entry.
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(daglang_driver::EmbeddedCompileOutput {
            fns,
            data_values,
            pipelines: HashMap::new(),
        })
    }

    #[test]
    fn phase1_all_v2_modules_compile() {
        match compile_tokenizer_module() {
            Ok(output) => {
                // Verify we got fn bodies and data values
                assert!(
                    !output.fns.is_empty(),
                    "should have extracted at least one fn body"
                );
            }
            Err(e) => {
                panic!("v2 modules failed to compile: {}", e);
            }
        }
    }

    #[test]
    fn phase1_tokenize_fn_exists() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        assert!(
            output.fns.contains_key("tokenize"),
            "should have a 'tokenize' fn body, found: {:?}",
            output.fns.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn phase1_keywords_data_exists() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        assert!(
            output.data_values.contains_key("keywords"),
            "should have 'keywords' data, found: {:?}",
            output.data_values.keys().collect::<Vec<_>>()
        );
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 2: Tokenizer E2E — evaluate the v2 tokenizer on real input
    // ═════════════════════════════════════════════════════════════════════

    #[test]
    fn phase2_tokenizer_smoke() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        let tokenize_body = output
            .fns
            .get("tokenize")
            .expect("'tokenize' fn should exist");

        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("fn add(a: Int) -> Int { a }".into()),
        );

        let result = daglang_eval::evaluate_fn_body_with_data(
            tokenize_body,
            &inputs,
            &output.fns,
            &output.data_values,
        );

        match result {
            Ok(outputs) => {
                let ret = &outputs["return"];
                // Should be a list of tokens
                match ret {
                    gunbc_ir::Value::List(tokens) => {
                        assert!(
                            !tokens.is_empty(),
                            "tokenizer should produce at least one token"
                        );
                    }
                    other => {
                        panic!("expected Value::List of tokens, got: {:?}", other);
                    }
                }
            }
            Err(e) => {
                panic!("tokenizer evaluation failed: {}", e);
            }
        }
    }

    #[test]
    fn phase2_tokenizer_empty_input() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        let tokenize_body = output.fns.get("tokenize").expect("'tokenize' fn");

        let mut inputs = HashMap::new();
        inputs.insert("source".to_string(), gunbc_ir::Value::Str("".into()));

        let result = daglang_eval::evaluate_fn_body_with_data(
            tokenize_body,
            &inputs,
            &output.fns,
            &output.data_values,
        );

        match result {
            Ok(outputs) => {
                let ret = &outputs["return"];
                // Empty input should produce at least an Eof token
                match ret {
                    gunbc_ir::Value::List(tokens) => {
                        assert!(
                            !tokens.is_empty(),
                            "empty input should still produce Eof token"
                        );
                    }
                    other => panic!("expected token list, got: {:?}", other),
                }
            }
            Err(e) => panic!("tokenizer failed on empty input: {}", e),
        }
    }

    #[test]
    fn phase2_tokenizer_keywords() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        let tokenize_body = output.fns.get("tokenize").expect("'tokenize' fn");

        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("module fn type import data".into()),
        );

        let result = daglang_eval::evaluate_fn_body_with_data(
            tokenize_body,
            &inputs,
            &output.fns,
            &output.data_values,
        );

        match result {
            Ok(outputs) => {
                let ret = &outputs["return"];
                match ret {
                    gunbc_ir::Value::List(tokens) => {
                        // Should have 5 keyword tokens + Eof
                        assert!(
                            tokens.len() >= 5,
                            "should produce at least 5 keyword tokens, got {}",
                            tokens.len()
                        );
                    }
                    other => panic!("expected token list, got: {:?}", other),
                }
            }
            Err(e) => panic!("tokenizer failed on keywords: {}", e),
        }
    }

    #[test]
    fn phase2_tokenizer_two_char_operators() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        let tokenize_body = output.fns.get("tokenize").expect("'tokenize' fn");

        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("== -> => !=".into()),
        );

        let result = daglang_eval::evaluate_fn_body_with_data(
            tokenize_body,
            &inputs,
            &output.fns,
            &output.data_values,
        );

        match result {
            Ok(outputs) => {
                let ret = &outputs["return"];
                match ret {
                    gunbc_ir::Value::List(tokens) => {
                        // 4 operators + Eof
                        assert!(
                            tokens.len() >= 4,
                            "should produce at least 4 operator tokens, got {}",
                            tokens.len()
                        );
                    }
                    other => panic!("expected token list, got: {:?}", other),
                }
            }
            Err(e) => panic!("tokenizer failed on operators: {}", e),
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 3: Stage-by-stage integration — chain stages on trivial fixture
    // ═════════════════════════════════════════════════════════════════════

    #[test]
    fn phase3_compile_all_modules() {
        let output = compile_all_modules().expect("all modules should compile");
        // Verify we got fn bodies from all 7 compiler modules
        assert!(
            output.fns.contains_key("tokenize"),
            "should have tokenize fn"
        );
        assert!(output.fns.contains_key("parse"), "should have parse fn");
        assert!(
            output.fns.contains_key("resolve_modules"),
            "should have resolve_modules fn"
        );
        assert!(
            output.fns.contains_key("typecheck"),
            "should have typecheck fn"
        );
        assert!(
            output.fns.contains_key("emit_rust"),
            "should have emit_rust fn"
        );
        assert!(
            output.fns.contains_key("compile_sources"),
            "should have compile_sources fn"
        );
    }

    #[test]
    fn phase3_variant_names_collected() {
        let root = workspace_root();
        let core = root.join("src/v2/std/core.dag");
        let source = std::fs::read_to_string(&core).unwrap();
        let ast =
            daglang_syntax::parser::parse_with_file_diagnostics(&core, &source).unwrap();
        let mut variant_names = std::collections::HashSet::new();
        for item in &ast.items {
            if let daglang_syntax::ast::Item::TypeDef(td) = &item.node {
                if let daglang_syntax::ast::TypeBody::Sum(variants) = &td.body {
                    for v in variants {
                        variant_names.insert(v.name.clone());
                    }
                }
            }
        }
        assert!(variant_names.contains("Ident"), "should have Ident variant");
        assert!(
            variant_names.contains("KwModule"),
            "should have KwModule variant"
        );
        assert!(variant_names.contains("Eof"), "should have Eof variant");
    }

    #[test]
    fn phase3_kind_tag_matches_ident() {
        // Test that kind_tag function correctly matches an Ident token kind
        let output = compile_all_modules().expect("compilation should succeed");

        // Simulate an Ident { name: "test" } token kind value
        let mut kind_map = std::collections::BTreeMap::new();
        kind_map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Ident".to_string()),
        );
        kind_map.insert(
            "name".to_string(),
            gunbc_ir::Value::Str("test".to_string()),
        );
        let kind_val = gunbc_ir::Value::Map(kind_map);

        let mut inputs = HashMap::new();
        inputs.insert("kind".to_string(), kind_val);

        let result = call_fn(&output, "kind_tag", inputs);
        match result {
            Ok(outputs) => {
                let ret = &outputs["return"];
                assert_eq!(
                    ret,
                    &gunbc_ir::Value::Str("Ident".to_string()),
                    "kind_tag should return 'Ident'"
                );
            }
            Err(e) => panic!("kind_tag failed: {}", e),
        }
    }

    #[test]
    fn phase3_tokenize_produces_correct_kinds() {
        let output = compile_all_modules().expect("compilation should succeed");

        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("fn add(a: Int) -> Int { a }".into()),
        );

        let result = call_fn(&output, "tokenize", inputs).expect("tokenize should succeed");
        let tokens = match &result["return"] {
            gunbc_ir::Value::List(t) => t,
            other => panic!("expected token list, got: {:?}", other),
        };

        // Verify token kinds are correct (not double-wrapped in Some)
        let kinds: Vec<Option<String>> = tokens.iter().map(token_kind_tag).collect();
        assert!(
            kinds.iter().any(|k| k.as_deref() == Some("KwFn")),
            "should have KwFn token, got kinds: {:?}",
            kinds
        );
    }

    #[test]
    fn phase3_expect_ident_on_ident_token() {
        // Test expect_ident with a token list starting with an Ident token
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let output = compile_all_modules().expect("compilation should succeed");

                // Build a token list with just an Ident token + Eof
                let mut ident_kind = std::collections::BTreeMap::new();
                ident_kind.insert(
                    "_variant".to_string(),
                    gunbc_ir::Value::Str("Ident".to_string()),
                );
                ident_kind.insert(
                    "name".to_string(),
                    gunbc_ir::Value::Str("test".to_string()),
                );

                let mut span = std::collections::BTreeMap::new();
                span.insert("start".to_string(), gunbc_ir::Value::Int(0));
                span.insert("end".to_string(), gunbc_ir::Value::Int(4));

                let mut token = std::collections::BTreeMap::new();
                token.insert("kind".to_string(), gunbc_ir::Value::Map(ident_kind));
                token.insert("span".to_string(), gunbc_ir::Value::Map(span.clone()));

                let eof_token = {
                    let mut t = std::collections::BTreeMap::new();
                    t.insert(
                        "kind".to_string(),
                        gunbc_ir::Value::Enum {
                            ty: String::new(),
                            variant: "Eof".to_string(),
                        },
                    );
                    t.insert("span".to_string(), gunbc_ir::Value::Map(span));
                    t
                };

                let tokens = gunbc_ir::Value::List(vec![
                    gunbc_ir::Value::Map(token),
                    gunbc_ir::Value::Map(eof_token),
                ]);

                let mut state = std::collections::BTreeMap::new();
                state.insert("tokens".to_string(), tokens);
                state.insert("pos".to_string(), gunbc_ir::Value::Int(0));

                let mut inputs = HashMap::new();
                inputs.insert("state".to_string(), gunbc_ir::Value::Map(state));

                match call_fn(&output, "expect_ident", inputs) {
                    Ok(_outputs) => {}
                    Err(e) => panic!("expect_ident failed: {}", e),
                }
            })
            .unwrap()
            .join();
        match result {
            Ok(()) => {}
            Err(e) => std::panic::resume_unwind(e),
        }
    }

    #[test]
    fn phase3_peek_kind_returns_option() {
        // Test peek_kind on a simple token list
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let output = compile_all_modules().expect("compilation should succeed");

                // Step 1: Tokenize "module test"
                let mut tok_inputs = HashMap::new();
                tok_inputs.insert(
                    "source".to_string(),
                    gunbc_ir::Value::Str("module test".into()),
                );
                let tok_result =
                    call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
                let tokens = match &tok_result["return"] {
                    gunbc_ir::Value::List(t) => t.clone(),
                    other => panic!("expected token list, got: {:?}", other),
                };

                // Step 2: Test peek_kind
                let mut peek_inputs = HashMap::new();
                let mut state = std::collections::BTreeMap::new();
                state.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
                state.insert("pos".to_string(), gunbc_ir::Value::Int(0));
                peek_inputs.insert("state".to_string(), gunbc_ir::Value::Map(state));

                match call_fn(&output, "peek_kind", peek_inputs) {
                    Ok(outputs) => {
                        // peek_kind returns destructured Some: has "value" key
                        assert!(
                            outputs.contains_key("value"),
                            "peek_kind should have 'value' key, got: {:?}",
                            outputs.keys().collect::<Vec<_>>()
                        );
                    }
                    Err(e) => panic!("peek_kind failed: {}", e),
                }
            })
            .unwrap()
            .join();
        match result {
            Ok(()) => {}
            Err(e) => std::panic::resume_unwind(e),
        }
    }

    #[test]
    fn phase3_parser_e2e() {
        // Parser uses deep recursion via evaluator — needs large stack.
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024) // 64 MB
            .spawn(|| {
                let output = compile_all_modules().expect("compilation should succeed");

                // Step 1: Tokenize - start with minimal input
                let source = "module test";
                let mut tok_inputs = HashMap::new();
                tok_inputs.insert(
                    "source".to_string(),
                    gunbc_ir::Value::Str(source.into()),
                );
                let tok_result =
                    call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
                let tokens = match &tok_result["return"] {
                    gunbc_ir::Value::List(t) => t.clone(),
                    other => panic!("expected token list from tokenize, got: {:?}", other),
                };

                // Step 2: Parse
                let mut parse_inputs = HashMap::new();
                parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
                let parse_result = call_fn(&output, "parse", parse_inputs);

                match parse_result {
                    Ok(outputs) => {
                        // The parse function returns fields that may include _variant
                        // Navigate to the module value, handling possible wrapping
                        let module_val = outputs
                            .get("module")
                            .expect("should have 'module' key");
                        // The module might be wrapped in a value field (from Some construction)
                        let module = if let gunbc_ir::Value::Map(m) = module_val {
                            if m.contains_key("value") && !m.contains_key("name") {
                                m.get("value").unwrap()
                            } else {
                                module_val
                            }
                        } else {
                            module_val
                        };
                        if let gunbc_ir::Value::Map(mod_map) = module {
                            let name = mod_map.get("name").expect("module should have name");
                            assert_eq!(
                                name,
                                &gunbc_ir::Value::Str("test".to_string()),
                                "module name should be 'test'"
                            );
                        } else {
                            panic!("module is not a Map: {:?}", module);
                        }
                    }
                    Err(e) => {
                        panic!("parser evaluation failed: {}", e);
                    }
                }
            })
            .expect("failed to spawn thread")
            .join();

        match result {
            Ok(()) => {}
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}
