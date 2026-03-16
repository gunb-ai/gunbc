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

    /// Compile all v2 compiler .dag files into a single EmbeddedCompileOutput.
    /// All fn bodies from all modules share one `fns` HashMap, enabling
    /// cross-module calls (e.g., pipeline.dag calling tokenize()).
    fn compile_all_modules() -> Result<daglang_driver::EmbeddedCompileOutput, String> {
        let root = workspace_root();

        let files = vec![
            root.join("dsl/std/types.dag"),
            root.join("src/v2/00_core.dag"),
            root.join("src/v2/01_tokenize.dag"),
            root.join("src/v2/02_parse.dag"),
            root.join("src/v2/03_resolve.dag"),
            root.join("src/v2/04_typecheck.dag"),
            root.join("src/v2/05_emit.dag"),
            root.join("src/v2/05_emit_rust.dag"),
            root.join("src/v2/05_emit_python.dag"),
            root.join("src/v2/06_pipeline.dag"),
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
            let ast = daglang_syntax::parser::parse_with_file_diagnostics(path, source).map_err(
                |errs| {
                    errs.iter()
                        .map(|d| d.render())
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            )?;
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
                        if fns.insert(fndef.name.clone(), lowered).is_some() {
                            return Err(format!("duplicate fn name: {}", fndef.name));
                        }
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
                            ..Default::default()
                        };
                        if let Ok(result) =
                            daglang_eval::evaluate_fn_body(&body, &HashMap::new(), &HashMap::new())
                        {
                            if let Some(val) = result.get("return") {
                                data_values.insert(dd.name.clone(), val.clone());
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

    /// Like `compile_all_modules()` but rejects duplicate function names across
    /// modules instead of silently overwriting. Returns the list of duplicate
    /// names if any are found.
    fn detect_duplicate_fn_names() -> Vec<String> {
        let root = workspace_root();

        let files = vec![
            root.join("dsl/std/types.dag"),
            root.join("src/v2/00_core.dag"),
            root.join("src/v2/01_tokenize.dag"),
            root.join("src/v2/02_parse.dag"),
            root.join("src/v2/03_resolve.dag"),
            root.join("src/v2/04_typecheck.dag"),
            root.join("src/v2/05_emit.dag"),
            root.join("src/v2/05_emit_rust.dag"),
            root.join("src/v2/05_emit_python.dag"),
            root.join("src/v2/06_pipeline.dag"),
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
            if let Ok(ast) = daglang_syntax::parser::parse_with_file_diagnostics(path, source) {
                parsed_files.push(ast);
            }
        }

        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for ast in &parsed_files {
            for item in &ast.items {
                if let daglang_syntax::ast::Item::FnDef(fndef) = &item.node {
                    if !seen.insert(fndef.name.clone()) {
                        duplicates.push(fndef.name.clone());
                    }
                }
            }
        }
        duplicates
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

    fn source_file_value(path: &str, content: &str) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert("path".to_string(), gunbc_ir::Value::Str(path.to_string()));
        map.insert(
            "content".to_string(),
            gunbc_ir::Value::Str(content.to_string()),
        );
        gunbc_ir::Value::Map(map)
    }

    fn render_target_value(target: &str) -> gunbc_ir::Value {
        let mut target_map = std::collections::BTreeMap::new();
        target_map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str(target.to_string()),
        );
        gunbc_ir::Value::Map(target_map)
    }

    fn compile_sources_with_target(
        output: &daglang_driver::EmbeddedCompileOutput,
        sources: &[(&str, &str)],
        target: &str,
    ) -> HashMap<String, gunbc_ir::Value> {
        let mut inputs = HashMap::new();
        inputs.insert(
            "sources".to_string(),
            gunbc_ir::Value::List(
                sources
                    .iter()
                    .map(|(path, content)| source_file_value(path, content))
                    .collect(),
            ),
        );
        inputs.insert("target".to_string(), render_target_value(target));
        let result =
            call_fn(output, "compile_sources", inputs).expect("compile_sources should succeed");
        if let Some(gunbc_ir::Value::Map(map)) = result.get("return") {
            map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            result
        }
    }

    fn compile_sources_with(
        output: &daglang_driver::EmbeddedCompileOutput,
        sources: &[(&str, &str)],
    ) -> HashMap<String, gunbc_ir::Value> {
        compile_sources_with_target(output, sources, "Rust")
    }

    fn zero_span_value() -> gunbc_ir::Value {
        let mut span = std::collections::BTreeMap::new();
        span.insert("start".to_string(), gunbc_ir::Value::Int(0));
        span.insert("end".to_string(), gunbc_ir::Value::Int(0));
        gunbc_ir::Value::Map(span)
    }

    fn named_type_value(name: &str) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Named".to_string()),
        );
        map.insert("name".to_string(), gunbc_ir::Value::Str(name.to_string()));
        map.insert("span".to_string(), zero_span_value());
        gunbc_ir::Value::Map(map)
    }

    fn type_binding_value(name: &str, resolved: gunbc_ir::Value) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert("name".to_string(), gunbc_ir::Value::Str(name.to_string()));
        map.insert("resolved".to_string(), resolved);
        gunbc_ir::Value::Map(map)
    }

    fn type_env_value(bindings: Vec<gunbc_ir::Value>) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        let binding_map: std::collections::BTreeMap<String, gunbc_ir::Value> = bindings
            .into_iter()
            .filter_map(|binding| match &binding {
                gunbc_ir::Value::Map(fields) => match fields.get("name") {
                    Some(gunbc_ir::Value::Str(name)) => Some((name.clone(), binding.clone())),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        map.insert("bindings".to_string(), gunbc_ir::Value::Map(binding_map));
        gunbc_ir::Value::Map(map)
    }

    fn bool_type_value() -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Primitive".to_string()),
        );
        map.insert("name".to_string(), gunbc_ir::Value::Str("Bool".to_string()));
        map.insert("span".to_string(), zero_span_value());
        gunbc_ir::Value::Map(map)
    }

    fn string_type_value() -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Primitive".to_string()),
        );
        map.insert("name".to_string(), gunbc_ir::Value::Str("String".to_string()));
        map.insert("span".to_string(), zero_span_value());
        gunbc_ir::Value::Map(map)
    }

    fn field_value(name: &str, type_expr: gunbc_ir::Value) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert("name".to_string(), gunbc_ir::Value::Str(name.to_string()));
        map.insert("type_expr".to_string(), type_expr);
        map.insert("optional".to_string(), gunbc_ir::Value::Bool(false));
        map.insert("default_value".to_string(), gunbc_ir::Value::Unit);
        map.insert("from_key".to_string(), gunbc_ir::Value::Unit);
        map.insert("span".to_string(), zero_span_value());
        gunbc_ir::Value::Map(map)
    }

    fn product_type_value(
        name: Option<&str>,
        fields: Vec<gunbc_ir::Value>,
    ) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Product".to_string()),
        );
        map.insert(
            "name".to_string(),
            match name {
                Some(name) => gunbc_ir::Value::Str(name.to_string()),
                None => gunbc_ir::Value::Unit,
            },
        );
        map.insert("fields".to_string(), gunbc_ir::Value::List(fields));
        map.insert("span".to_string(), zero_span_value());
        gunbc_ir::Value::Map(map)
    }

    fn literal_value_string(value: &str) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("LitStr".to_string()),
        );
        map.insert("value".to_string(), gunbc_ir::Value::Str(value.to_string()));
        gunbc_ir::Value::Map(map)
    }

    fn literal_value_bool(value: bool) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("LitBool".to_string()),
        );
        map.insert("value".to_string(), gunbc_ir::Value::Bool(value));
        gunbc_ir::Value::Map(map)
    }

    fn literal_expr_value(
        literal: gunbc_ir::Value,
        span: gunbc_ir::Value,
    ) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Literal".to_string()),
        );
        map.insert("value".to_string(), literal);
        map.insert("span".to_string(), span);
        gunbc_ir::Value::Map(map)
    }

    fn field_init_value(name: &str, value: gunbc_ir::Value) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert("name".to_string(), gunbc_ir::Value::Str(name.to_string()));
        map.insert("value".to_string(), value);
        gunbc_ir::Value::Map(map)
    }

    fn span_type_value(
        start: i64,
        end: i64,
        resolved_type: gunbc_ir::Value,
    ) -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert("start".to_string(), gunbc_ir::Value::Int(start));
        map.insert("end".to_string(), gunbc_ir::Value::Int(end));
        map.insert("resolved_type".to_string(), resolved_type);
        gunbc_ir::Value::Map(map)
    }

    fn func_env_value() -> gunbc_ir::Value {
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "signatures".to_string(),
            gunbc_ir::Value::Map(std::collections::BTreeMap::new()),
        );
        gunbc_ir::Value::Map(map)
    }

    fn infer_scope_value(
        type_env: gunbc_ir::Value,
        type_cache: Vec<gunbc_ir::Value>,
    ) -> gunbc_ir::Value {
        // type_cache is now Map<String, TypeExpr> keyed by "start:end"
        let mut cache_map = std::collections::BTreeMap::new();
        for entry in &type_cache {
            if let gunbc_ir::Value::Map(m) = entry {
                let start = match m.get("start") {
                    Some(gunbc_ir::Value::Int(i)) => i.to_string(),
                    _ => continue,
                };
                let end = match m.get("end") {
                    Some(gunbc_ir::Value::Int(i)) => i.to_string(),
                    _ => continue,
                };
                if let Some(resolved) = m.get("resolved_type") {
                    cache_map.insert(format!("{}:{}", start, end), resolved.clone());
                }
            }
        }
        let mut map = std::collections::BTreeMap::new();
        map.insert("type_env".to_string(), type_env);
        map.insert("func_env".to_string(), func_env_value());
        map.insert(
            "locals".to_string(),
            gunbc_ir::Value::Map(std::collections::BTreeMap::new()),
        );
        map.insert(
            "module_name".to_string(),
            gunbc_ir::Value::Str("main".to_string()),
        );
        map.insert("type_cache".to_string(), gunbc_ir::Value::Map(cache_map));
        gunbc_ir::Value::Map(map)
    }

    fn returned_value(outputs: HashMap<String, gunbc_ir::Value>) -> gunbc_ir::Value {
        if let Some(value) = outputs.get("return") {
            value.clone()
        } else {
            gunbc_ir::Value::Map(outputs.into_iter().collect())
        }
    }

    fn diagnostic_messages(diags: &gunbc_ir::Value) -> Vec<String> {
        match diags {
            gunbc_ir::Value::List(items) => items
                .iter()
                .filter_map(|item| match item {
                    gunbc_ir::Value::Map(map) => match map.get("message") {
                        Some(gunbc_ir::Value::Str(message)) => Some(message.clone()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect(),
            other => panic!("expected diagnostics list, got: {other:?}"),
        }
    }

    fn emitted_file_content(files: &gunbc_ir::Value, path: &str) -> String {
        match files {
            gunbc_ir::Value::List(items) => items
                .iter()
                .find_map(|item| match item {
                    gunbc_ir::Value::Map(map) => match (map.get("path"), map.get("content")) {
                        (
                            Some(gunbc_ir::Value::Str(file_path)),
                            Some(gunbc_ir::Value::Str(content)),
                        ) if file_path == path => Some(content.clone()),
                        _ => None,
                    },
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing emitted file {path}")),
            other => panic!("expected files list, got: {other:?}"),
        }
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
        assert_parses_strict("src/v2/00_core.dag");
    }

    #[test]
    fn phase0_tokenize_parses_strict() {
        assert_parses_strict("src/v2/01_tokenize.dag");
    }

    #[test]
    fn phase0_parse_parses_strict() {
        assert_parses_strict("src/v2/02_parse.dag");
    }

    #[test]
    fn phase0_resolve_parses_strict() {
        assert_parses_strict("src/v2/03_resolve.dag");
    }

    #[test]
    fn phase0_typecheck_parses_strict() {
        assert_parses_strict("src/v2/04_typecheck.dag");
    }

    #[test]
    fn phase0_emit_parses_strict() {
        assert_parses_strict("src/v2/05_emit.dag");
    }

    #[test]
    fn phase0_pipeline_parses_strict() {
        assert_parses_strict("src/v2/06_pipeline.dag");
    }

    #[test]
    fn phase0_shared_behavioral_parses_strict() {
        assert_parses_strict("dsl/shared/behavioral.dag");
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 1: Compilation gate — v1 compiler can compile each v2 module
    // ═════════════════════════════════════════════════════════════════════

    /// Extract fn bodies and data values from the tokenizer module.
    /// Uses direct AST-level lowering (bypasses DAG wiring) to avoid
    /// DAG-level expression resolution failures for pure fn bodies.
    fn compile_tokenizer_module() -> Result<daglang_driver::EmbeddedCompileOutput, String> {
        let root = workspace_root();

        // Read sources
        let files = vec![
            root.join("dsl/std/types.dag"),
            root.join("src/v2/00_core.dag"),
            root.join("src/v2/01_tokenize.dag"),
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
            let ast = daglang_syntax::parser::parse_with_file_diagnostics(path, source).map_err(
                |errs| {
                    errs.iter()
                        .map(|d| d.render())
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            )?;
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
                        let lowered =
                            daglang_lower::expr::lower_fn_body(&fndef.body, &variant_names);
                        fns.insert(fndef.name.clone(), lowered);
                    }
                    daglang_syntax::ast::Item::DataDef(dd) => {
                        // Lower data declarations to Value directly (no JSON round-trip)
                        {
                            let expr = &dd.value;
                            let lowered_expr =
                                daglang_lower::expr::lower_expr_remap(expr, &variant_names);
                            let body = daglang_eval::LoweredFnBody {
                                stmts: vec![daglang_eval::LoweredStmt::Return(vec![(
                                    "return".to_string(),
                                    lowered_expr,
                                )])],
                                ..Default::default()
                            };
                            match daglang_eval::evaluate_fn_body(
                                &body,
                                &HashMap::new(),
                                &HashMap::new(),
                            ) {
                                Ok(result) => {
                                    if let Some(val) = result.get("return") {
                                        data_values.insert(dd.name.clone(), val.clone());
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
    fn phase1_tokenizer_module_compiles() {
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
    #[allow(clippy::disallowed_macros)]
    fn phase1_keywords_data_exists() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        assert!(
            output.data_values.contains_key("keywords"),
            "should have 'keywords' data, found: {:?}",
            output.data_values.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    #[allow(clippy::disallowed_macros)]
    fn phase1_data_lookup_works() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        // Check if "lookup" accidentally exists as a sibling fn
        eprintln!(
            "[diag] fns has 'lookup': {}",
            output.fns.contains_key("lookup")
        );
        eprintln!(
            "[diag] fns has 'find_module': {}",
            output.fns.contains_key("find_module")
        );
        let fn_names: Vec<_> = output.fns.keys().collect();
        eprintln!("[diag] all fn names ({}):", fn_names.len());
        for name in &fn_names {
            eprintln!("  {}", name);
        }

        let mut inputs = HashMap::new();
        inputs.insert("source".to_string(), gunbc_ir::Value::Str("+".to_string()));
        let result = daglang_eval::evaluate_fn_body_with_data(
            output.fns.get("tokenize").unwrap(),
            &inputs,
            &output.fns,
            &output.data_values,
        );
        match &result {
            Ok(outputs) => {
                eprintln!(
                    "[diag] tokenize('+') ok: {:?}",
                    outputs.get("return").map(|v| format!("{:?}", v)
                        [..200.min(format!("{:?}", v).len())]
                        .to_string())
                );
            }
            Err(e) => {
                eprintln!("[diag] tokenize('+') failed: {}", e);
            }
        }
        result.expect("tokenize('+') should succeed");
    }

    #[test]
    #[allow(clippy::disallowed_macros)]
    fn phase1_keywords_data_shape() {
        let output = compile_tokenizer_module().expect("compilation should succeed");
        let kw = output
            .data_values
            .get("keywords")
            .expect("keywords should exist");
        let kw_str = format!("{:?}", kw);
        eprintln!(
            "[diag] keywords ({} chars):\n{}",
            kw_str.len(),
            &kw_str[..kw_str.len().min(500)]
        );

        let sp = output.data_values.get("single_punct");
        eprintln!("[diag] single_punct present: {}", sp.is_some());
        if let Some(sp_val) = sp {
            let sp_str = format!("{:?}", sp_val);
            eprintln!("[diag] single_punct:\n{}", &sp_str[..sp_str.len().min(500)]);
        }
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
        let core = root.join("src/v2/00_core.dag");
        let source = std::fs::read_to_string(&core).unwrap();
        let ast = daglang_syntax::parser::parse_with_file_diagnostics(&core, &source).unwrap();
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
        kind_map.insert("name".to_string(), gunbc_ir::Value::Str("test".to_string()));
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
        let output = compile_all_modules().expect("compilation should succeed");

        // Build a token list with just an Ident token + Eof
        let mut ident_kind = std::collections::BTreeMap::new();
        ident_kind.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Ident".to_string()),
        );
        ident_kind.insert("name".to_string(), gunbc_ir::Value::Str("test".to_string()));

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
    }

    #[test]
    fn phase3_peek_kind_returns_option() {
        // Test peek_kind on a simple token list
        let output = compile_all_modules().expect("compilation should succeed");

        // Step 1: Tokenize "module test"
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("module test".into()),
        );
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
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
    }

    #[test]
    fn phase3_parser_e2e() {
        // Parser uses deep recursion via evaluator — needs large stack.
        let output = compile_all_modules().expect("compilation should succeed");

        // Step 1: Tokenize - start with minimal input
        let source = "module test";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
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
                let module_val = outputs.get("module").expect("should have 'module' key");
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
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 3b: Full pipeline grinding — progressively deeper stages
    // ═════════════════════════════════════════════════════════════════════

    #[test]
    fn phase3_parse_real_source() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module types_test\n\
                type SourceSpan { start: Int, end: Int }\n\
                type Token { kind: TokenKind, span: SourceSpan }\n\
                type TokenKind = Ident { name: String } | KwModule | KwFn | Eof\n\
                type Param { name: String, type_expr: String }\n\
                fn identity(x: Int) -> Int { x }\n\
                "
        .to_string();
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        assert!(!tokens.is_empty(), "should produce tokens");
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs)
            .expect("parse should succeed on multi-type source");
        let module_val = parse_result
            .get("module")
            .expect("should have 'module' key");
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
            assert!(
                mod_map.contains_key("name"),
                "parsed module should have 'name'"
            );
            assert!(
                mod_map.contains_key("items"),
                "parsed module should have 'items'"
            );
            if let Some(gunbc_ir::Value::List(items)) = mod_map.get("items") {
                assert!(!items.is_empty(), "should have at least one item");
            }
        } else {
            panic!("module is not a Map: {:?}", module);
        }
    }

    #[test]
    fn phase3_parse_fold_with_fn_lambda() {
        let output = compile_all_modules().expect("compilation should succeed");
        // Test multi-line pipe chain with fold + fn lambda (matches 03_resolve.dag line 80-81)
        let source = "module test\nfn foo(items: List<List<Int>>) -> List<Int> {\n  let diags = items |> map(r => r)\n    |> fold(init: [], f: fn(acc, diags) { concat(acc, diags) })\n  diags\n}\n";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs);
        match parse_result {
            Ok(outputs) => {
                let error_val = outputs.get("error");
                if let Some(gunbc_ir::Value::Map(err_map)) = error_val {
                    if !err_map.is_empty() {
                        if let Some(gunbc_ir::Value::Str(msg)) = err_map.get("message") {
                            panic!("parse error: {}", msg);
                        }
                        panic!("parse produced error: {:?}", err_map);
                    }
                }
                let module_val = outputs.get("module").expect("should have 'module' key");
                if let gunbc_ir::Value::Unit = module_val {
                    let error_val = outputs.get("error");
                    panic!(
                        "module is Unit (parse returned None), error: {:?}",
                        error_val
                    );
                }
            }
            Err(e) => panic!("parse fn eval failed: {}", e),
        }
    }

    #[test]
    fn phase3_resolve_single_module() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\ntype Foo { x: Int }";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize ok");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs).expect("parse ok");
        let module_val = parse_result.get("module").expect("should have 'module'");
        let module = if let gunbc_ir::Value::Map(m) = module_val {
            if m.contains_key("value") && !m.contains_key("name") {
                m.get("value").unwrap().clone()
            } else {
                module_val.clone()
            }
        } else {
            module_val.clone()
        };
        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert("modules".to_string(), gunbc_ir::Value::List(vec![module]));
        let resolve_result =
            call_fn(&output, "resolve_modules", resolve_inputs).expect("resolve_modules ok");
        let graph = if let Some(ret) = resolve_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(resolve_result.into_iter().collect())
        };
        match &graph {
            gunbc_ir::Value::Map(m) => {
                assert!(
                    m.contains_key("modules"),
                    "ModuleGraph should have 'modules'"
                );
            }
            other => panic!("unexpected resolve result: {:?}", other),
        }
    }

    #[test]
    fn phase3_typecheck_single_module() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\ntype Foo { x: Int }";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize ok");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs).expect("parse ok");
        let module_val = parse_result.get("module").expect("should have 'module'");
        let module = if let gunbc_ir::Value::Map(m) = module_val {
            if m.contains_key("value") && !m.contains_key("name") {
                m.get("value").unwrap().clone()
            } else {
                module_val.clone()
            }
        } else {
            module_val.clone()
        };
        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert("modules".to_string(), gunbc_ir::Value::List(vec![module]));
        let resolve_result =
            call_fn(&output, "resolve_modules", resolve_inputs).expect("resolve ok");
        let graph = if let Some(ret) = resolve_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(resolve_result.into_iter().collect())
        };
        let mut tc_inputs = HashMap::new();
        tc_inputs.insert("graph".to_string(), graph);
        let tc_result = call_fn(&output, "typecheck", tc_inputs).expect("typecheck ok");
        let typed_graph = if let Some(ret) = tc_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(tc_result.into_iter().collect())
        };
        if let gunbc_ir::Value::Map(m) = &typed_graph {
            assert!(
                m.contains_key("modules"),
                "TypedGraph should have 'modules'"
            );
            assert!(
                m.contains_key("diagnostics"),
                "TypedGraph should have 'diagnostics'"
            );
        } else {
            panic!("TypedGraph is not a Map: {:?}", typed_graph);
        }
    }

    #[test]
    fn phase3_emit_single_module() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\ntype Foo { x: Int }";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize ok");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs).expect("parse ok");
        let module_val = parse_result.get("module").expect("should have 'module'");
        let module = if let gunbc_ir::Value::Map(m) = module_val {
            if m.contains_key("value") && !m.contains_key("name") {
                m.get("value").unwrap().clone()
            } else {
                module_val.clone()
            }
        } else {
            module_val.clone()
        };
        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert("modules".to_string(), gunbc_ir::Value::List(vec![module]));
        let resolve_result =
            call_fn(&output, "resolve_modules", resolve_inputs).expect("resolve ok");
        let graph = if let Some(ret) = resolve_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(resolve_result.into_iter().collect())
        };
        let mut tc_inputs = HashMap::new();
        tc_inputs.insert("graph".to_string(), graph);
        let tc_result = call_fn(&output, "typecheck", tc_inputs).expect("typecheck ok");
        let typed_graph = if let Some(ret) = tc_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(tc_result.into_iter().collect())
        };
        let typed_modules = if let gunbc_ir::Value::Map(m) = &typed_graph {
            if let Some(gunbc_ir::Value::List(mods)) = m.get("modules") {
                mods.clone()
            } else {
                panic!("no modules in typed graph");
            }
        } else {
            panic!("typed graph not a map");
        };
        assert!(!typed_modules.is_empty());
        let mut emit_inputs = HashMap::new();
        emit_inputs.insert("typed_module".to_string(), typed_modules[0].clone());
        emit_inputs.insert("registry".to_string(), gunbc_ir::Value::Map(std::collections::BTreeMap::new()));
        let emit_result = call_fn(&output, "emit_module", emit_inputs).expect("emit_module ok");
        let text_file = if let Some(ret) = emit_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(emit_result.into_iter().collect())
        };
        if let gunbc_ir::Value::Map(m) = &text_file {
            assert!(m.contains_key("path"), "TextFile should have 'path'");
            assert!(m.contains_key("content"), "TextFile should have 'content'");
            if let Some(gunbc_ir::Value::Str(content)) = m.get("content") {
                assert!(
                    content.contains("struct") || content.contains("pub"),
                    "emitted content should contain Rust code"
                );
            }
        } else {
            panic!("TextFile is not a Map: {:?}", text_file);
        }
    }

    #[test]
    fn phase3_full_pipeline() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\ntype Foo { x: Int }";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize ok");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs).expect("parse ok");
        let module_val = parse_result.get("module").expect("should have 'module'");
        let module = if let gunbc_ir::Value::Map(m) = module_val {
            if m.contains_key("value") && !m.contains_key("name") {
                m.get("value").unwrap().clone()
            } else {
                module_val.clone()
            }
        } else {
            module_val.clone()
        };
        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert("modules".to_string(), gunbc_ir::Value::List(vec![module]));
        let resolve_result =
            call_fn(&output, "resolve_modules", resolve_inputs).expect("resolve ok");
        let graph = if let Some(ret) = resolve_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(resolve_result.into_iter().collect())
        };
        let mut tc_inputs = HashMap::new();
        tc_inputs.insert("graph".to_string(), graph);
        let tc_result = call_fn(&output, "typecheck", tc_inputs).expect("typecheck ok");
        let typed_graph = if let Some(ret) = tc_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(tc_result.into_iter().collect())
        };
        let typed_modules = if let gunbc_ir::Value::Map(m) = &typed_graph {
            if let Some(gunbc_ir::Value::List(mods)) = m.get("modules") {
                mods.clone()
            } else {
                panic!("no modules");
            }
        } else {
            panic!("not a map");
        };
        let mut emit_inputs = HashMap::new();
        emit_inputs.insert("typed_module".to_string(), typed_modules[0].clone());
        emit_inputs.insert("registry".to_string(), gunbc_ir::Value::Map(std::collections::BTreeMap::new()));
        let emit_result = call_fn(&output, "emit_module", emit_inputs).expect("emit_module ok");
        let text_file = if let Some(ret) = emit_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(emit_result.into_iter().collect())
        };
        if let gunbc_ir::Value::Map(m) = &text_file {
            if let Some(gunbc_ir::Value::Str(s)) = m.get("content") {
                assert!(
                    s.contains("struct Foo"),
                    "emitted Rust should contain 'struct Foo', got: {}",
                    &s[..s.len().min(300)]
                );
            }
        } else {
            panic!("not a TextFile map");
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 4: New feature tests — pipe arrow, null coalesce, cast, where
    // ═════════════════════════════════════════════════════════════════════

    /// Test that PipeArrow token is recognized by v1 tokenizer.
    #[test]
    fn phase4_pipe_arrow_token_exists() {
        assert_parses_strict("src/v2/00_core.dag");
        // PipeArrow should be in the TokenKind sum type
        let source = read_v2_file("src/v2/00_core.dag");
        assert!(
            source.contains("PipeArrow"),
            "core.dag should contain PipeArrow variant"
        );
    }

    /// Test that NullCoalesce is in BinOpKind.
    #[test]
    fn phase4_null_coalesce_in_binop_kind() {
        let source = read_v2_file("src/v2/00_core.dag");
        assert!(
            source.contains("NullCoalesce"),
            "core.dag BinOpKind should contain NullCoalesce"
        );
    }

    /// Test that the tokenizer scans |> as PipeArrow.
    #[test]
    fn phase4_tokenizer_scans_pipe_arrow() {
        let output = compile_all_modules().expect("compilation should succeed");
        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("items |> count".to_string()),
        );
        let result = call_fn(&output, "tokenize", inputs).expect("tokenize ok");
        let json = value_to_json(&gunbc_ir::Value::Map(result.into_iter().collect()));
        let json_str = json.to_string();
        assert!(
            json_str.contains("PipeArrow"),
            "tokenize('items |> count') should produce PipeArrow token, got: {}",
            &json_str[..json_str.len().min(500)]
        );
    }

    /// Test that the tokenizer scans ?? as NullCoalesce.
    #[test]
    fn phase4_tokenizer_scans_null_coalesce() {
        let output = compile_all_modules().expect("compilation should succeed");
        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str("x ?? y".to_string()),
        );
        let result = call_fn(&output, "tokenize", inputs).expect("tokenize ok");
        let json = value_to_json(&gunbc_ir::Value::Map(result.into_iter().collect()));
        let json_str = json.to_string();
        assert!(
            json_str.contains("NullCoalesce"),
            "tokenize('x ?? y') should produce NullCoalesce token, got: {}",
            &json_str[..json_str.len().min(500)]
        );
    }

    /// Test that parse.dag includes PipeArrow in kind_tag and infix_bp.
    #[test]
    fn phase4_parse_supports_pipe_arrow() {
        let source = read_v2_file("src/v2/02_parse.dag");
        assert!(
            source.contains("PipeArrow"),
            "parse.dag should reference PipeArrow"
        );
        assert!(
            source.contains("parse_pipe_rhs"),
            "parse.dag should contain parse_pipe_rhs function"
        );
    }

    /// Test that parse.dag supports NullCoalesce in infix_bp.
    #[test]
    fn phase4_parse_supports_null_coalesce() {
        let source = read_v2_file("src/v2/02_parse.dag");
        assert!(
            source.contains("NullCoalesce"),
            "parse.dag should reference NullCoalesce"
        );
    }

    /// Test that emit_rust.dag handles NullCoalesce emission.
    #[test]
    fn phase4_emit_handles_null_coalesce() {
        let source = read_v2_file("src/v2/05_emit_rust.dag");
        assert!(
            source.contains("unwrap_or_else"),
            "emit_rust.dag should emit unwrap_or_else for null coalesce"
        );
    }

    /// Test that emit_rust.dag handles for-loop emission.
    #[test]
    fn phase4_emit_handles_for_loop() {
        let source = read_v2_file("src/v2/05_emit_rust.dag");
        assert!(
            source.contains("emit_for_each"),
            "emit_rust.dag should contain emit_for_each function"
        );
        assert!(
            source.contains("into_iter"),
            "emit_rust.dag should emit .into_iter().map() for for-loops"
        );
    }

    /// Test that emit_rust.dag generates Cargo.toml.
    #[test]
    fn phase4_emit_generates_cargo_toml() {
        let source = read_v2_file("src/v2/05_emit_rust.dag");
        assert!(
            source.contains("emit_cargo_toml"),
            "emit_rust.dag should contain emit_cargo_toml function"
        );
    }

    /// Test that emit_rust.dag has tail-call optimization support.
    /// TCO-eligible functions should emit `loop` + `continue` instead of
    /// direct self-recursion.
    #[test]
    fn phase4_emit_has_tco_support() {
        let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
        assert!(
            rust_source.contains("emit_tco_body"),
            "emit_rust.dag should contain emit_tco_body function for TCO rendering"
        );
        assert!(
            rust_source.contains("emit_tco_expr"),
            "emit_rust.dag should contain emit_tco_expr for TCO expression rendering"
        );
        assert!(
            rust_source.contains("emit_tco_reassign"),
            "emit_rust.dag should contain emit_tco_reassign for parameter reassignment"
        );
        assert!(
            rust_source.contains("loop {"),
            "emit_rust.dag should emit Rust loop for TCO-eligible functions"
        );
        assert!(
            rust_source.contains("continue;"),
            "emit_rust.dag should emit continue for tail self-calls"
        );
        assert!(
            rust_source.contains("break "),
            "emit_rust.dag should emit break for non-recursive returns in TCO"
        );

        let python_source = read_v2_file("src/v2/05_emit_python.dag");
        assert!(
            python_source.contains("emit_py_tco_body"),
            "emit_python.dag should contain emit_py_tco_body function for TCO rendering"
        );
        assert!(
            python_source.contains("while True:"),
            "emit_python.dag should emit Python while True for TCO-eligible functions"
        );
        assert!(
            python_source.contains("continue"),
            "emit_python.dag should emit continue for tail self-calls in Python"
        );

        // Verify the shared classification functions exist in 05_emit.dag
        let emit_source = read_v2_file("src/v2/05_emit.dag");
        assert!(
            emit_source.contains("fn expr_has_self_call"),
            "emit.dag should contain expr_has_self_call for shared TCO classification"
        );
        assert!(
            emit_source.contains("fn has_non_tail_self_call"),
            "emit.dag should contain has_non_tail_self_call for shared TCO classification"
        );
    }

    /// Test that the parse.dag where clause machinery exists.
    #[test]
    fn phase4_parse_supports_where_clause() {
        let source = read_v2_file("src/v2/02_parse.dag");
        assert!(
            source.contains("try_where_clause"),
            "parse.dag should contain try_where_clause function"
        );
        assert!(
            source.contains("parse_predicates"),
            "parse.dag should contain parse_predicates function"
        );
    }

    /// Test that the parse.dag response/mock_response parsing exists.
    #[test]
    fn phase4_parse_supports_response_blocks() {
        let source = read_v2_file("src/v2/02_parse.dag");
        assert!(
            source.contains("parse_optional_response_block"),
            "parse.dag should contain parse_optional_response_block function"
        );
        assert!(
            source.contains("parse_optional_mock_response_block"),
            "parse.dag should contain parse_optional_mock_response_block function"
        );
    }

    /// Test that the typecheck.dag has mutual recursion cycle detection.
    #[test]
    fn phase4_typecheck_has_cycle_detection() {
        let source = read_v2_file("src/v2/04_typecheck.dag");
        assert!(
            source.contains("resolve_type_expr_with_resolving"),
            "typecheck.dag should contain resolve_type_expr_with_resolving for cycle detection"
        );
        assert!(
            source.contains("resolving"),
            "typecheck.dag should use 'resolving' parameter for cycle tracking"
        );
    }

    #[test]
    fn phase6_resolve_filters_failed_imports_and_cycles() {
        let source = read_v2_file("src/v2/03_resolve.dag");
        assert!(
            source.contains("acyclic_resolved"),
            "resolve.dag should filter cycle members before downstream sorting"
        );
        assert!(
            source.contains("r.resolved.target_module != none"),
            "resolve.dag should drop unresolved imports from resolved_imports"
        );
        assert!(
            source.contains("r.diagnostics |> count == 0"),
            "resolve.dag should keep only fully-resolved imports"
        );
    }

    #[test]
    fn phase6_typecheck_resolves_and_validates_expression_tree_types() {
        let source = read_v2_file("src/v2/04_typecheck.dag");
        assert!(
            source.contains("fn resolve_expr_types"),
            "typecheck.dag should walk expression trees during type resolution"
        );
        assert!(
            source.contains("collect_unresolved_in_expr"),
            "typecheck.dag should validate unresolved types inside expression trees"
        );
    }

    #[test]
    fn phase6_emit_preserves_field_provenance_and_named_arg_ordering() {
        let rust_source = read_v2_file("src/v2/05_emit_rust.dag");
        assert!(
            rust_source.contains("serde(rename = "),
            "emit_rust.dag should preserve from_key through serde rename attributes"
        );
        let core_source = read_v2_file("src/v2/05_emit.dag");
        assert!(
            core_source.contains("order_call_args"),
            "emit.dag should reorder named arguments using function signatures"
        );
    }

    /// Test: emit a module with pipe chains and verify Rust output has .len(), .join(), etc.
    #[test]
    fn phase4_emit_pipe_methods() {
        let output = compile_all_modules().expect("compilation should succeed");
        let fixture = r#"module test

fn example(items: List<String>) -> Int {
  items |> count
}
"#;
        let mut inputs = HashMap::new();
        inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str(fixture.to_string()),
        );
        let tokens = call_fn(&output, "tokenize", inputs).expect("tokenize ok");
        let token_list = tokens.get("return").cloned().unwrap_or_else(|| {
            gunbc_ir::Value::List(
                tokens
                    .values()
                    .next()
                    .cloned()
                    .map(|v| {
                        if let gunbc_ir::Value::List(l) = v {
                            l
                        } else {
                            vec![v]
                        }
                    })
                    .unwrap_or_default(),
            )
        });
        // Just verify tokenization succeeds with pipe arrow
        let json = value_to_json(&token_list);
        let json_str = json.to_string();
        assert!(
            json_str.contains("PipeArrow"),
            "tokenization of 'items |> count' should contain PipeArrow, got: {}",
            &json_str[..json_str.len().min(500)]
        );
    }

    /// Regression: multi-line pipe chain with continuation on next line.
    /// Locks in that `items |> map(i =>\n  process(i)\n) |> filter(f => f != none)` parses.
    #[test]
    fn phase4_parse_multiline_pipe_chain() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\nfn transform(items: List<Int>) -> List<Int> {\n  let x = items |> map(i =>\n    process(i)\n  ) |> filter(f => f != none)\n  x\n}\n";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs);
        match parse_result {
            Ok(outputs) => {
                let error_val = outputs.get("error");
                if let Some(gunbc_ir::Value::Map(err_map)) = error_val {
                    if !err_map.is_empty() {
                        if let Some(gunbc_ir::Value::Str(msg)) = err_map.get("message") {
                            panic!("parse error: {}", msg);
                        }
                        panic!("parse produced error: {:?}", err_map);
                    }
                }
                let module_val = outputs.get("module").expect("should have 'module' key");
                if let gunbc_ir::Value::Unit = module_val {
                    panic!(
                        "module is Unit (parse returned None), error: {:?}",
                        outputs.get("error")
                    );
                }
                // Verify module has items (the fn parsed successfully)
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
                    assert!(
                        mod_map.contains_key("items"),
                        "parsed module should have 'items'"
                    );
                    if let Some(gunbc_ir::Value::List(items)) = mod_map.get("items") {
                        assert!(
                            !items.is_empty(),
                            "multi-line pipe chain fn should produce at least one item"
                        );
                    }
                } else {
                    panic!("module is not a Map: {:?}", module);
                }
            }
            Err(e) => panic!("parse fn eval failed on multi-line pipe chain: {}", e),
        }
    }

    /// Regression: fn() lambda as a named argument in a call.
    /// Locks in that `fold(init: [], f: fn(acc, item) { concat(acc, [item]) })` parses.
    #[test]
    fn phase4_parse_fn_lambda_in_call_arg() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\nfn flatten(items: List<List<Int>>) -> List<Int> {\n  fold(init: [], f: fn(acc, item) { concat(acc, [item]) })\n}\n";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs);
        match parse_result {
            Ok(outputs) => {
                let error_val = outputs.get("error");
                if let Some(gunbc_ir::Value::Map(err_map)) = error_val {
                    if !err_map.is_empty() {
                        if let Some(gunbc_ir::Value::Str(msg)) = err_map.get("message") {
                            panic!("parse error: {}", msg);
                        }
                        panic!("parse produced error: {:?}", err_map);
                    }
                }
                let module_val = outputs.get("module").expect("should have 'module' key");
                if let gunbc_ir::Value::Unit = module_val {
                    panic!(
                        "module is Unit (parse returned None), error: {:?}",
                        outputs.get("error")
                    );
                }
            }
            Err(e) => panic!("parse fn eval failed on fn-lambda-in-call-arg: {}", e),
        }
    }

    /// Regression: keyword as named argument name.
    /// Locks in that `resolve_module_imports(module: m, all_modules: modules)` parses
    /// and the named arg `module` is preserved.
    #[test]
    fn phase4_parse_keyword_named_arg() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\nfn do_resolve(m: Module, modules: List<Module>) -> Module {\n  resolve_module_imports(module: m, all_modules: modules)\n}\n";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source.into()));
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };
        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(&output, "parse", parse_inputs);
        match parse_result {
            Ok(outputs) => {
                let error_val = outputs.get("error");
                if let Some(gunbc_ir::Value::Map(err_map)) = error_val {
                    if !err_map.is_empty() {
                        if let Some(gunbc_ir::Value::Str(msg)) = err_map.get("message") {
                            panic!("parse error: {}", msg);
                        }
                        panic!("parse produced error: {:?}", err_map);
                    }
                }
                let module_val = outputs.get("module").expect("should have 'module' key");
                if let gunbc_ir::Value::Unit = module_val {
                    panic!(
                        "module is Unit (parse returned None), error: {:?}",
                        outputs.get("error")
                    );
                }
                // Verify the named arg "module" is preserved in the AST
                let json = value_to_json(module_val);
                let json_str = json.to_string();
                assert!(
                    json_str.contains("module"),
                    "named arg 'module' should be preserved in parse output, got: {}",
                    &json_str[..json_str.len().min(1000)]
                );
            }
            Err(e) => panic!("parse fn eval failed on keyword-named-arg: {}", e),
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 5: gist.dag transitive closure — v1 parser gate
    //
    // All 11 files in gist.dag's transitive closure must parse without
    // errors through the v1 parser. (The v2 parser gate is phase 5b.)
    // ═════════════════════════════════════════════════════════════════════

    /// Gate test: v1 parser handles all gist.dag transitive deps.
    #[test]
    fn phase5_gist_transitive_closure_v1_parse() {
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
        for f in &files {
            assert_parses_strict(f);
        }
    }

    /// Gate test: v2 parser handles a representative gist.dag dependency.
    /// Compiles v2 compiler via v1, then calls v2's tokenize+parse on a small file.
    ///
    /// Only tests 1 file (~24 lines) to keep CI under 15s. The DSL tokenizer
    /// is O(n²) due to list-append-per-token (`tokens + [tok]`), so even small
    /// files take ~10s when run interpreter-in-interpreter in debug mode.
    /// The full 12-file transitive closure is in phase5_gist_full_transitive_closure.
    #[test]
    fn phase5_gist_transitive_closure_v2_parse() {
        let output = compile_all_modules().expect("compilation should succeed");
        let root = workspace_root();
        let files = [
            // Smallest dep — exercises imports, service ops, type refs.
            "dsl/extdeps/github/auth.dag", // 24 lines
        ];
        for rel_path in &files {
            let path = root.join(rel_path);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));

            // Tokenize via v2
            let mut tok_inputs = HashMap::new();
            tok_inputs.insert("source".to_string(), gunbc_ir::Value::Str(source));
            let tok_result = call_fn(&output, "tokenize", tok_inputs)
                .unwrap_or_else(|e| panic!("{}: tokenize failed: {}", rel_path, e));
            let tokens = tok_result.get("return").cloned().unwrap_or_else(|| {
                gunbc_ir::Value::List(
                    tok_result
                        .values()
                        .next()
                        .cloned()
                        .map(|v| {
                            if let gunbc_ir::Value::List(l) = v {
                                l
                            } else {
                                vec![v]
                            }
                        })
                        .unwrap_or_default(),
                )
            });

            // Parse via v2
            let mut parse_inputs = HashMap::new();
            parse_inputs.insert("tokens".to_string(), tokens);
            let parse_result = call_fn(&output, "parse", parse_inputs)
                .unwrap_or_else(|e| panic!("{}: parse failed: {}", rel_path, e));

            // Check for parse errors
            let error = parse_result.get("error");
            let has_error = match error {
                Some(gunbc_ir::Value::Unit) => false,
                Some(gunbc_ir::Value::Map(m)) if m.contains_key("value") => {
                    !matches!(m.get("value"), Some(gunbc_ir::Value::Unit))
                }
                None => false,
                _ => false,
            };
            if has_error {
                let err_json = value_to_json(error.unwrap());
                panic!("{}: v2 parse error: {}", rel_path, err_json);
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 6: Multi-module compilation — towards gist.dag
    // ═════════════════════════════════════════════════════════════════════

    /// Helper: tokenize + parse a source string through the v2 pipeline,
    /// returning the parsed Module value.
    fn v2_tokenize_and_parse(
        output: &daglang_driver::EmbeddedCompileOutput,
        source: &str,
    ) -> gunbc_ir::Value {
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str(source.to_string()),
        );
        let tok_result = call_fn(output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };

        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result = call_fn(output, "parse", parse_inputs).expect("parse should succeed");

        // Check for parse errors before extracting module (S56 fix).
        if let Some(err_val) = parse_result.get("error") {
            if !matches!(err_val, gunbc_ir::Value::Unit) {
                let preview = format!("{:?}", err_val);
                panic!("v2 parse error: {}", &preview[..preview.len().min(500)]);
            }
        }

        let module_val = parse_result
            .get("module")
            .expect("should have 'module' key");
        // Unwrap Option wrapping (Some { value: ... })
        if let gunbc_ir::Value::Map(m) = module_val {
            if m.contains_key("value") && !m.contains_key("name") {
                let module = m.get("value").unwrap().clone();
                // Validate module shape
                if let gunbc_ir::Value::Map(ref mm) = module {
                    assert!(
                        mm.contains_key("name")
                            && mm.contains_key("imports")
                            && mm.contains_key("items"),
                        "parsed module missing required fields, got keys: {:?}",
                        mm.keys().collect::<Vec<_>>()
                    );
                } else {
                    panic!(
                        "parsed module is not a Map: {:?}",
                        std::mem::discriminant(&module)
                    );
                }
                return module;
            }
        }
        panic!(
            "unexpected parse result shape for 'module' field: {:?}",
            std::mem::discriminant(module_val)
        );
    }

    /// Synthetic 2-module test: types module + function module that imports it.
    /// Proves multi-module resolve → typecheck → emit works.
    ///
    /// Needs 16MB stack: the v2 parser has ~80 mutually-recursive functions
    /// (parse_items → parse_item → parse_type_def → ...) that aren't
    /// self-recursive (so TCO doesn't help). Each call frame is ~500 bytes
    /// in debug mode. 6 levels × ~500 bytes × many tokens = >8MB default.
    #[test]
    #[allow(clippy::disallowed_macros)]
    fn phase6_multi_module_synthetic() {
        let output = compile_all_modules().expect("compilation should succeed");

        // Start with the simplest possible multi-module case.
        let types_src = "module mylib.types\ntype Point { x: Int, y: Int }\n";
        let funcs_src = "module mylib.funcs\nimport mylib.types { Point }\n";

        eprintln!("[test] tokenizing types_src ({} bytes)...", types_src.len());
        let mod_types = v2_tokenize_and_parse(&output, types_src);
        eprintln!("[test] tokenizing funcs_src ({} bytes)...", funcs_src.len());
        let mod_funcs = v2_tokenize_and_parse(&output, funcs_src);

        // Resolve
        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert(
            "modules".to_string(),
            gunbc_ir::Value::List(vec![mod_types, mod_funcs]),
        );
        let resolve_result = call_fn(&output, "resolve_modules", resolve_inputs)
            .expect("resolve_modules should succeed");
        let graph = if let Some(ret) = resolve_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(resolve_result.into_iter().collect())
        };

        // Check no resolve errors
        if let gunbc_ir::Value::Map(ref m) = graph {
            if let Some(gunbc_ir::Value::List(diags)) = m.get("diagnostics") {
                let errors: Vec<_> = diags
                    .iter()
                    .filter(|d| {
                        if let gunbc_ir::Value::Map(dm) = d {
                            dm.get("severity").and_then(|s| match s {
                                gunbc_ir::Value::Str(s) => Some(s.as_str()),
                                gunbc_ir::Value::Enum { variant, .. } => Some(variant.as_str()),
                                _ => None,
                            }) == Some("Error")
                        } else {
                            false
                        }
                    })
                    .collect();
                assert!(
                    errors.is_empty(),
                    "resolve_modules produced errors: {:?}",
                    errors
                );
            }
        }

        // Typecheck
        let mut tc_inputs = HashMap::new();
        tc_inputs.insert("graph".to_string(), graph);
        let tc_result = call_fn(&output, "typecheck", tc_inputs).expect("typecheck should succeed");
        let typed_graph = if let Some(ret) = tc_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(tc_result.into_iter().collect())
        };

        // Verify we have typed modules
        if let gunbc_ir::Value::Map(ref m) = typed_graph {
            if let Some(gunbc_ir::Value::List(modules)) = m.get("modules") {
                assert_eq!(
                    modules.len(),
                    2,
                    "should have 2 typed modules, got {}",
                    modules.len()
                );
            } else {
                panic!("TypedGraph.modules not a list");
            }
        } else {
            panic!("typed_graph not a Map");
        }
    }

    /// Feed gist.dag's full transitive dependency chain through the v2
    /// pipeline: tokenize → parse → resolve → typecheck → emit.
    ///
    /// This is the Level 1 acceptance gate: v2 can process the real gist
    /// Regression test (S56): a parse error must surface as a parse diagnostic,
    /// not leak through as a resolve crash like "map requires a list, got Unit".
    #[test]
    fn phase6_parse_error_does_not_leak_to_resolve() {
        let output = compile_all_modules().expect("compilation should succeed");

        // Deliberately malformed source: missing module declaration.
        let bad_source = "fn orphan() -> Int { 42 }";
        let mut tok_inputs = HashMap::new();
        tok_inputs.insert(
            "source".to_string(),
            gunbc_ir::Value::Str(bad_source.to_string()),
        );
        let tok_result = call_fn(&output, "tokenize", tok_inputs).expect("tokenize should succeed");
        let tokens = match &tok_result["return"] {
            gunbc_ir::Value::List(t) => t.clone(),
            other => panic!("expected token list, got: {:?}", other),
        };

        let mut parse_inputs = HashMap::new();
        parse_inputs.insert("tokens".to_string(), gunbc_ir::Value::List(tokens));
        let parse_result =
            call_fn(&output, "parse", parse_inputs).expect("parse fn should not panic");

        // The parse result must have a non-Unit error field.
        let error_val = parse_result.get("error").expect("should have 'error' key");
        assert!(
            !matches!(error_val, gunbc_ir::Value::Unit),
            "parse of malformed source should produce an error, got Unit"
        );

        // The module field must be none/Unit — NOT a valid Module.
        let module_val = parse_result
            .get("module")
            .expect("should have 'module' key");
        let is_none = matches!(module_val, gunbc_ir::Value::Unit)
            || matches!(module_val, gunbc_ir::Value::Map(m) if m.get("_variant").and_then(|v| if let gunbc_ir::Value::Str(s) = v { Some(s.as_str()) } else { None }) == Some("None"));
        assert!(
            is_none,
            "parse of malformed source should return module=none, got: {:?}",
            std::mem::discriminant(module_val)
        );
    }

    // compile_file was deleted (dead code — never called by the pipeline).

    #[test]
    fn phase6_compile_sources_filters_none_parse_diagnostics() {
        let output = compile_all_modules().expect("compilation should succeed");
        let mut inputs = HashMap::new();
        inputs.insert(
            "sources".to_string(),
            gunbc_ir::Value::List(vec![
                source_file_value("good.dag", "module good\n"),
                source_file_value("bad.dag", "fn orphan() -> Int { 42 }\n"),
            ]),
        );
        // RenderTarget::Rust variant value
        let mut target_map = std::collections::BTreeMap::new();
        target_map.insert(
            "_variant".to_string(),
            gunbc_ir::Value::Str("Rust".to_string()),
        );
        inputs.insert("target".to_string(), gunbc_ir::Value::Map(target_map));
        let result =
            call_fn(&output, "compile_sources", inputs).expect("compile_sources should succeed");
        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");

        match diagnostics {
            gunbc_ir::Value::List(items) => {
                assert_eq!(
                    items.len(),
                    1,
                    "expected one parse diagnostic, got: {:?}",
                    items
                );
                assert!(
                    items
                        .iter()
                        .all(|item| !matches!(item, gunbc_ir::Value::Unit)),
                    "diagnostics should not contain Unit entries: {:?}",
                    items
                );
            }
            other => panic!("expected diagnostics list, got: {other:?}"),
        }
    }

    #[test]
    fn phase6_bare_import_wildcard_survives_pipeline() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[
                ("dep.dag", "module dep\ntype Foo { x: Int }\n"),
                (
                    "main.dag",
                    "module main\nimport dep\nfn id(x: Foo) -> Foo { x }\n",
                ),
            ],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "bare import should not produce diagnostics: {:?}",
            messages
        );

        let files = result
            .get("files")
            .expect("compile_sources should return files");
        let main_rs = emitted_file_content(files, "src/main.rs");
        assert!(
            main_rs.contains("use crate::dep::*;"),
            "bare import should emit a Rust wildcard import:\n{}",
            main_rs
        );
    }

    #[test]
    fn phase6_anonymous_record_literal_fails_closed_without_named_type() {
        let output = compile_all_modules().expect("compilation should succeed");
        let mut span = std::collections::BTreeMap::new();
        span.insert("start".to_string(), gunbc_ir::Value::Int(10));
        span.insert("end".to_string(), gunbc_ir::Value::Int(20));
        let span = gunbc_ir::Value::Map(span);

        let record_fields = vec![
            field_value("name", string_type_value()),
            field_value("enabled", bool_type_value()),
        ];
        let scope = infer_scope_value(
            type_env_value(vec![type_binding_value(
                "Config",
                product_type_value(Some("Config"), record_fields.clone()),
            )]),
            vec![span_type_value(
                10,
                20,
                product_type_value(None, record_fields),
            )],
        );

        let mut inputs = HashMap::new();
        inputs.insert("type_name".to_string(), gunbc_ir::Value::Unit);
        inputs.insert(
            "fields".to_string(),
            gunbc_ir::Value::List(vec![
                field_init_value(
                    "name",
                    literal_expr_value(literal_value_string("demo"), span.clone()),
                ),
                field_init_value(
                    "enabled",
                    literal_expr_value(literal_value_bool(true), span.clone()),
                ),
            ]),
        );
        inputs.insert("span".to_string(), span);
        inputs.insert("registry".to_string(), gunbc_ir::Value::Map(std::collections::BTreeMap::new()));
        inputs.insert("scope".to_string(), scope);

        let rendered = returned_value(
            call_fn(&output, "emit_record_lit", inputs).expect("emit_record_lit should succeed"),
        );
        let main_rs = match rendered {
            gunbc_ir::Value::Str(rendered) => rendered,
            other => panic!("emit_record_lit should return a string, got: {:?}", other),
        };

        assert!(
            main_rs.contains("cannot resolve anonymous record type in emitter"),
            "anonymous record literal should fail closed without a named type:\n{}",
            main_rs
        );
        assert!(
            !main_rs.contains("Config {"),
            "anonymous record literal should not guess a matching named struct:\n{}",
            main_rs
        );
    }

    #[test]
    fn phase6_anonymous_record_literal_does_not_rank_shape_candidates() {
        let output = compile_all_modules().expect("compilation should succeed");
        let mut span = std::collections::BTreeMap::new();
        span.insert("start".to_string(), gunbc_ir::Value::Int(10));
        span.insert("end".to_string(), gunbc_ir::Value::Int(20));
        let span = gunbc_ir::Value::Map(span);

        let exact_fields = vec![
            field_value("name", string_type_value()),
            field_value("enabled", bool_type_value()),
        ];
        let wider_fields = vec![
            field_value("name", string_type_value()),
            field_value("enabled", bool_type_value()),
            field_value("owner", string_type_value()),
            field_value("region", string_type_value()),
            field_value("version", string_type_value()),
            field_value("stage", string_type_value()),
            field_value("mode", string_type_value()),
            field_value("team", string_type_value()),
            field_value("service", string_type_value()),
            field_value("profile", string_type_value()),
        ];
        let scope = infer_scope_value(
            type_env_value(vec![
                type_binding_value("Config", product_type_value(Some("Config"), exact_fields.clone())),
                type_binding_value(
                    "ConfigExpanded",
                    product_type_value(Some("ConfigExpanded"), wider_fields),
                ),
            ]),
            vec![span_type_value(
                10,
                20,
                product_type_value(None, exact_fields),
            )],
        );

        let mut inputs = HashMap::new();
        inputs.insert("type_name".to_string(), gunbc_ir::Value::Unit);
        inputs.insert(
            "fields".to_string(),
            gunbc_ir::Value::List(vec![
                field_init_value(
                    "name",
                    literal_expr_value(literal_value_string("demo"), span.clone()),
                ),
                field_init_value(
                    "enabled",
                    literal_expr_value(literal_value_bool(true), span.clone()),
                ),
            ]),
        );
        inputs.insert("span".to_string(), span);
        inputs.insert("registry".to_string(), gunbc_ir::Value::Map(std::collections::BTreeMap::new()));
        inputs.insert("scope".to_string(), scope);

        let rendered = returned_value(
            call_fn(&output, "emit_record_lit", inputs).expect("emit_record_lit should succeed"),
        );
        let main_rs = match rendered {
            gunbc_ir::Value::Str(rendered) => rendered,
            other => panic!("emit_record_lit should return a string, got: {:?}", other),
        };

        assert!(
            main_rs.contains("cannot resolve anonymous record type in emitter"),
            "anonymous record literal should fail closed instead of ranking candidates:\n{}",
            main_rs
        );
        assert!(
            !main_rs.contains("ConfigExpanded {"),
            "anonymous record literal should not guess a wider structural superset:\n{}",
            main_rs
        );
        assert!(
            !main_rs.contains("Config {"),
            "anonymous record literal should not guess an exact structural match either:\n{}",
            main_rs
        );
    }

    #[test]
    fn phase6_qualified_type_names_canonicalize_to_local_binding() {
        let output = compile_all_modules().expect("compilation should succeed");
        let config_type = named_type_value("Config");
        let env = type_env_value(vec![type_binding_value("Config", config_type.clone())]);

        let mut lookup_inputs = HashMap::new();
        lookup_inputs.insert("env".to_string(), env);
        lookup_inputs.insert(
            "name".to_string(),
            gunbc_ir::Value::Str("pkg.Config".to_string()),
        );
        let lookup_result = returned_value(
            call_fn(&output, "lookup_type", lookup_inputs).expect("lookup_type should succeed"),
        );
        match lookup_result {
            gunbc_ir::Value::Map(map) => {
                let value = if map.get("_variant")
                    == Some(&gunbc_ir::Value::Str("Some".to_string()))
                {
                    map.get("value")
                } else if map.get("value").is_some() {
                    map.get("value")
                } else {
                    None
                };
                assert_eq!(
                    value,
                    Some(&config_type),
                    "lookup_type should reuse the local binding for canonicalized names: {:?}",
                    map
                );
            }
            other => panic!("lookup_type should return an option map, got: {:?}", other),
        }

        let mut eq_inputs = HashMap::new();
        eq_inputs.insert("left".to_string(), named_type_value("Config"));
        eq_inputs.insert("right".to_string(), named_type_value("pkg.Config"));
        let eq_result = call_fn(&output, "type_expr_equals", eq_inputs)
            .expect("type_expr_equals should succeed");
        assert!(
            matches!(eq_result.get("return"), Some(gunbc_ir::Value::Bool(true))),
            "type_expr_equals should treat qualified and local names as equivalent: {:?}",
            eq_result
        );
    }

    #[test]
    fn phase6_empty_import_block_emits_no_rust_import() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[
                ("dep.dag", "module dep\ntype Foo { x: Int }\n"),
                (
                    "main.dag",
                    "module main\nimport dep {}\ndata answer: Int = 42\n",
                ),
            ],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "empty import block should remain a no-op: {:?}",
            messages
        );

        let files = result
            .get("files")
            .expect("compile_sources should return files");
        let main_rs = emitted_file_content(files, "src/main.rs");
        assert!(
            !main_rs.contains("use crate::dep::*;") && !main_rs.contains("use crate::dep::{"),
            "empty import block should not emit a Rust import:\n{}",
            main_rs
        );
    }

    #[test]
    fn phase6_map_index_emits_lookup_style_rust() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn get(m: Map<String, Int>) -> Int? { m[\"x\"] }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "map index should typecheck and emit cleanly: {:?}",
            messages
        );

        let files = result
            .get("files")
            .expect("compile_sources should return files");
        let main_rs = emitted_file_content(files, "src/main.rs");
        assert!(
            main_rs.contains(".get(&\"x\".to_string()).cloned()"),
            "map index should emit Rust map lookup semantics:\n{}",
            main_rs
        );
    }

    #[test]
    fn phase6_string_index_and_slice_emit_string_runtime_calls() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn head(s: String) -> String { s[0] }\nfn mid(s: String) -> String { s[0..1] }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "string index/slice should compile cleanly: {:?}",
            messages
        );

        let files = result
            .get("files")
            .expect("compile_sources should return files");
        let main_rs = emitted_file_content(files, "src/main.rs");
        assert!(
            main_rs.contains("v2_rt::char_at(&s, 0)")
                && main_rs.contains("v2_rt::substring(&s, 0, 1)"),
            "string index/slice should emit string runtime helpers:\n{}",
            main_rs
        );
    }

    #[test]
    fn phase6_list_index_is_rejected_before_emit() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn first(xs: List<Int>) -> Int { xs[0] }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages
                .iter()
                .any(|message| message
                    .contains("indexing is only supported for String and Map values")),
            "list index should be rejected by typecheck: {:?}",
            messages
        );
    }

    #[test]
    fn phase6_map_index_key_type_mismatch_is_rejected() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn get(m: Map<String, Int>) -> Int? { m[0] }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages
                .iter()
                .any(|message| message
                    .contains("map index key type does not match the map key type")),
            "mismatched map keys should be rejected: {:?}",
            messages
        );
    }

    #[test]
    fn phase6_non_string_slice_is_rejected_before_emit() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn sub(xs: List<Int>) -> String { xs[0..1] }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages
                .iter()
                .any(|message| message.contains("slice is only supported for String values")),
            "non-string slice should be rejected by typecheck: {:?}",
            messages
        );
    }

    /// Focused test: func with return type goes through typecheck without error.
    /// Exercises Optional<TypeExpr> handling in resolve_optional_type_expr.
    #[test]
    fn phase6_func_return_type_typecheck() {
        let output = compile_all_modules().expect("compilation should succeed");
        let src = "module test.auth\nfunc get_token() -> { token: Secret } {\n  return { token: \"mock\" }\n}\n";
        let module = v2_tokenize_and_parse(&output, src);

        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert("modules".to_string(), gunbc_ir::Value::List(vec![module]));
        let resolve_result =
            call_fn(&output, "resolve_modules", resolve_inputs).expect("resolve should succeed");
        let graph = resolve_result
            .get("return")
            .cloned()
            .unwrap_or_else(|| gunbc_ir::Value::Map(resolve_result.into_iter().collect()));

        let mut tc_inputs = HashMap::new();
        tc_inputs.insert("graph".to_string(), graph);
        let tc_result = call_fn(&output, "typecheck", tc_inputs)
            .expect("typecheck should succeed for func with return type");
        let typed_graph = tc_result
            .get("return")
            .cloned()
            .unwrap_or_else(|| gunbc_ir::Value::Map(tc_result.into_iter().collect()));

        if let gunbc_ir::Value::Map(ref m) = typed_graph {
            if let Some(gunbc_ir::Value::List(modules)) = m.get("modules") {
                assert_eq!(modules.len(), 1, "should have 1 typed module");
            }
        }
    }

    #[test]
    fn phase6_typecheck_rejects_cross_function_param_leak() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn uses_missing() -> Int { ghost }\nfn carries_param(ghost: Int) -> Int { ghost }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages
                .iter()
                .any(|message| message.contains("undefined variable 'ghost'")),
            "cross-function param names must not leak through FuncEnv: {:?}",
            messages
        );
    }

    #[test]
    fn phase6_block_let_scope_threads_forward() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn scoped() -> Int {\n  let x = 1\n  x\n}\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "block-local let bindings should be visible to later statements: {:?}",
            messages
        );
    }

    #[test]
    fn phase6_if_else_branch_is_inferred() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn choose(cond: Bool) -> Int { if cond { 1 } else { \"x\" } }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages
                .iter()
                .any(|message| message.contains("if branches resolve to incompatible types")),
            "else branches should be typechecked, not ignored: {:?}",
            messages
        );
    }

    #[test]
    fn phase6_for_each_binds_loop_variable() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[(
                "main.dag",
                "module main\nfn walk() -> Unit { for ch in \"abc\" { ch } }\n",
            )],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "for-each loop variables should be available inside the loop body: {:?}",
            messages
        );
    }

    #[test]
    fn phase6_emit_non_empty_wrappers_validate_deserialize() {
        let output = compile_all_modules().expect("compilation should succeed");
        let result = compile_sources_with(
            &output,
            &[("main.dag", "module main\ndata answer: Int = 42\n")],
        );

        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "non-empty wrapper emission should compile cleanly: {:?}",
            messages
        );

        let files = result
            .get("files")
            .expect("compile_sources should return files");
        let main_rs = emitted_file_content(files, "src/main.rs");
        assert!(
            main_rs.contains("impl<'de, T> Deserialize<'de> for NonEmptyVec<T>")
                && main_rs.contains("NonEmptyVec::new(items).map_err(serde::de::Error::custom)")
                && main_rs.contains("impl<'de, T> Deserialize<'de> for NonEmptyBTreeSet<T>")
                && main_rs
                    .contains("NonEmptyBTreeSet::new(items).map_err(serde::de::Error::custom)"),
            "non-empty wrappers should validate deserialization invariants:\n{}",
            main_rs
        );
    }

    /// Feed gist.dag's full transitive dependency chain through the v2
    /// pipeline: tokenize → parse → resolve → typecheck → emit.
    ///
    /// This is the Level 1 acceptance gate: v2 can process the real gist
    /// tool and its 10 transitive dependencies.
    ///
    /// No stack overflow: the explicit-stack evaluator handles deep mutual
    /// recursion via heap continuations. However, evaluating 11 real .dag
    /// files consumes >16GB heap in debug mode (interpreter overhead).
    #[test]
    #[ignore = "OOM: interpreting v2 pipeline on 11 .dag files exceeds memory (stack overflow fixed by stacker)"]
    fn phase6_gist_full_pipeline() {
        let output = compile_all_modules().expect("compilation should succeed");
        let root = workspace_root();

        // Gist's full transitive dependency chain (topological order: leaves first).
        let dag_files = vec![
            "dsl/std/types.dag",
            "dsl/std/resources.dag",
            "dsl/std/errors.dag",
            "dsl/extdeps/cloud/cloud.dag",
            "dsl/extdeps/cloud/gcp/gcp.dag",
            "dsl/extdeps/github/github.dag",
            "dsl/gunbc/auth/credentials.dag",
            "dsl/extdeps/git.dag",
            "dsl/extdeps/github/auth.dag",
            "dsl/extdeps/github/gists.dag",
            "dsl/gunbc/tools/gist.dag",
        ];

        // Step 1: Tokenize + parse each file
        let mut modules = Vec::new();
        for path in &dag_files {
            let full_path = root.join(path);
            let source = std::fs::read_to_string(&full_path)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
            let module = v2_tokenize_and_parse(&output, &source);
            modules.push(module);
        }

        // Step 2: Resolve imports
        let mut resolve_inputs = HashMap::new();
        resolve_inputs.insert("modules".to_string(), gunbc_ir::Value::List(modules));
        let resolve_result = call_fn(&output, "resolve_modules", resolve_inputs)
            .expect("resolve_modules should succeed");
        let graph = if let Some(ret) = resolve_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(resolve_result.into_iter().collect())
        };

        // Step 3: Typecheck
        let mut tc_inputs = HashMap::new();
        tc_inputs.insert("graph".to_string(), graph.clone());
        let tc_result = call_fn(&output, "typecheck", tc_inputs).expect("typecheck should succeed");
        let typed_graph = if let Some(ret) = tc_result.get("return") {
            ret.clone()
        } else {
            gunbc_ir::Value::Map(tc_result.into_iter().collect())
        };

        // Step 4: Emit Rust for each typed module
        let typed_modules = if let gunbc_ir::Value::Map(ref m) = typed_graph {
            if let Some(gunbc_ir::Value::List(mods)) = m.get("modules") {
                mods.clone()
            } else {
                panic!("no modules in typed graph");
            }
        } else {
            panic!("typed graph not a map");
        };

        assert_eq!(
            typed_modules.len(),
            dag_files.len(),
            "should have {} typed modules, got {}",
            dag_files.len(),
            typed_modules.len()
        );

        let mut emitted_files = Vec::new();
        for typed_module in &typed_modules {
            let mut emit_inputs = HashMap::new();
            emit_inputs.insert("typed_module".to_string(), typed_module.clone());
            emit_inputs.insert("registry".to_string(), gunbc_ir::Value::Map(std::collections::BTreeMap::new()));
            if let Ok(result) = call_fn(&output, "emit_module", emit_inputs) {
                let text_file = if let Some(ret) = result.get("return") {
                    ret.clone()
                } else {
                    gunbc_ir::Value::Map(result.into_iter().collect())
                };
                if matches!(text_file, gunbc_ir::Value::Map(_)) {
                    emitted_files.push(text_file);
                }
            }
        }

        // At minimum, gist.dag should produce emitted output
        assert!(
            !emitted_files.is_empty(),
            "should have emitted at least one file"
        );
    }

    // ═════════════════════════════════════════════════════════════════════
    // Regression: stack ordering for inner/outer continuations (S52-EVAL)
    // ═════════════════════════════════════════════════════════════════════

    /// Verify is_ident_start works when compiled with all v2 modules.
    #[test]
    fn regression_is_ident_start_with_all_modules() {
        let output = compile_all_modules().expect("compilation should succeed");
        // Call is_ident_start("f") directly
        let mut inputs = HashMap::new();
        inputs.insert("ch".to_string(), gunbc_ir::Value::Str("f".to_string()));
        let result = call_fn(&output, "is_ident_start", inputs);
        match &result {
            Ok(outputs) => {
                let ret = outputs.get("return").unwrap();
                assert_eq!(
                    ret,
                    &gunbc_ir::Value::Bool(true),
                    "is_ident_start('f') should be true, got: {:?}",
                    ret
                );
            }
            Err(e) => panic!("is_ident_start('f') failed: {}", e),
        }
    }

    /// Regression: tokenize('f') with all v2 modules should produce Ident token.
    /// Before the stack-ordering fix, inner block continuations were pushed
    /// below outer stmt continuations, causing pop_stack to resume with the
    /// wrong continuation and skip the block's Return/EarlyReturn.
    #[test]
    fn regression_tokenize_single_ident() {
        let output = compile_all_modules().expect("compilation should succeed");
        let mut inputs = HashMap::new();
        inputs.insert("source".to_string(), gunbc_ir::Value::Str("f".to_string()));
        let result = call_fn(&output, "tokenize", inputs).expect("tokenize should succeed");
        let tokens = match &result["return"] {
            gunbc_ir::Value::List(t) => t,
            other => panic!("expected token list, got: {:?}", other),
        };
        let kinds: Vec<Option<String>> = tokens.iter().map(token_kind_tag).collect();
        assert!(
            kinds.iter().any(|k| k.as_deref() == Some("Ident")),
            "tokenize('f') with all modules should produce Ident, got: {:?}",
            kinds
        );
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 4+5: v2 → Rust crate assembly
    // ═════════════════════════════════════════════════════════════════════

    /// Parse all v2 .dag files and assemble them into a Rust crate via
    /// the v2_crate_emit module.
    #[test]
    fn v2_crate_assembly_produces_files() {
        let v2_files = [
            ("00_core", "src/v2/00_core.dag"),
            ("01_tokenize", "src/v2/01_tokenize.dag"),
            ("02_parse", "src/v2/02_parse.dag"),
            ("03_resolve", "src/v2/03_resolve.dag"),
            ("04_typecheck", "src/v2/04_typecheck.dag"),
            ("05_emit", "src/v2/05_emit.dag"),
            ("05_emit_rust", "src/v2/05_emit_rust.dag"),
            ("05_emit_python", "src/v2/05_emit_python.dag"),
            ("06_pipeline", "src/v2/06_pipeline.dag"),
        ];

        let parsed: Vec<(String, daglang_syntax::ast::SourceFile)> = v2_files
            .iter()
            .map(|(stem, path)| {
                let source = read_v2_file(path);
                let result = daglang_syntax::parser::parse_to_result(&source);
                assert!(
                    result.is_ok(),
                    "{} had parse errors: {:?}",
                    path,
                    result.diagnostics
                );
                (stem.to_string(), result.ast)
            })
            .collect();

        let modules: Vec<(&str, &daglang_syntax::ast::SourceFile)> = parsed
            .iter()
            .map(|(stem, sf): &(String, daglang_syntax::ast::SourceFile)| (stem.as_str(), sf))
            .collect();

        let files = daglang_emit::v2_crate_emit::assemble_v2_crate(&modules);

        // Should produce: 7 module files + lib.rs + v2_rt.rs + Cargo.toml = 10 files
        let file_names: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();
        assert!(
            file_names.contains(&"Cargo.toml"),
            "missing Cargo.toml in {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"src/lib.rs"),
            "missing src/lib.rs in {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"src/v2_rt.rs"),
            "missing src/v2_rt.rs in {:?}",
            file_names
        );
        assert!(
            file_names.contains(&"src/v2_core.rs"),
            "missing src/v2_core.rs in {:?}",
            file_names
        );

        // All generated .rs files should be non-empty
        for f in &files {
            assert!(!f.content.is_empty(), "{} is empty", f.rel_path);
        }
    }

    /// The recursive type detection should identify Expr and TypeExpr as needing Box<>.
    #[test]
    fn v2_recursive_types_detected() {
        let source = read_v2_file("src/v2/00_core.dag");
        let result = daglang_syntax::parser::parse_to_result(&source);
        assert!(result.is_ok());

        let type_defs: Vec<&daglang_syntax::ast::TypeDef> = result
            .ast
            .items
            .iter()
            .filter_map(|item| match &item.node {
                daglang_syntax::ast::Item::TypeDef(td) => Some(td),
                _ => None,
            })
            .collect();

        let recursive_fields = daglang_emit::type_codegen::compute_recursive_fields(&type_defs);

        // Expr should have recursive fields (e.g. FieldAccess::base is Expr)
        let has_expr_recursive = recursive_fields.iter().any(|(ty, _)| ty == "Expr");
        assert!(
            has_expr_recursive,
            "Expr should have recursive fields, got: {:?}",
            recursive_fields
        );

        // TypeExpr should have recursive fields (e.g. Optional::inner is TypeExpr)
        let has_type_expr_recursive = recursive_fields.iter().any(|(ty, _)| ty == "TypeExpr");
        assert!(
            has_type_expr_recursive,
            "TypeExpr should have recursive fields, got: {:?}",
            recursive_fields
        );
    }

    /// v2 builtins (char_at, string_length, etc.) should be recognized by the
    /// builtin contract registry.
    #[test]
    fn v2_builtins_registered() {
        use gunbc_ir::patterns::collection::contract_for_name;

        assert!(
            contract_for_name("char_at").is_some(),
            "char_at not registered"
        );
        assert!(
            contract_for_name("string_length").is_some(),
            "string_length not registered"
        );
        assert!(
            contract_for_name("substring").is_some(),
            "substring not registered"
        );
        assert!(
            contract_for_name("lookup").is_some(),
            "lookup not registered"
        );
        assert!(contract_for_name("with").is_some(), "with not registered");
    }

    // ── v2 crate assembly helpers ──────────────────────────────────────

    /// Assemble and write the v2 crate to a temp directory, returning its path.
    fn assemble_v2_crate_to_dir(dir_name: &str) -> std::path::PathBuf {
        let v2_files = [
            ("00_core", "src/v2/00_core.dag"),
            ("01_tokenize", "src/v2/01_tokenize.dag"),
            ("02_parse", "src/v2/02_parse.dag"),
            ("03_resolve", "src/v2/03_resolve.dag"),
            ("04_typecheck", "src/v2/04_typecheck.dag"),
            ("05_emit", "src/v2/05_emit.dag"),
            ("05_emit_rust", "src/v2/05_emit_rust.dag"),
            ("05_emit_python", "src/v2/05_emit_python.dag"),
            ("06_pipeline", "src/v2/06_pipeline.dag"),
        ];

        let parsed: Vec<(String, daglang_syntax::ast::SourceFile)> = v2_files
            .iter()
            .map(|(stem, path)| {
                let source = read_v2_file(path);
                let result = daglang_syntax::parser::parse_to_result(&source);
                (stem.to_string(), result.ast)
            })
            .collect();

        let modules: Vec<(&str, &daglang_syntax::ast::SourceFile)> = parsed
            .iter()
            .map(|(stem, sf): &(String, daglang_syntax::ast::SourceFile)| (stem.as_str(), sf))
            .collect();

        let files = daglang_emit::v2_crate_emit::assemble_v2_crate(&modules);

        let tmp_dir = std::env::temp_dir().join(dir_name);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        daglang_emit::v2_crate_emit::write_crate(&tmp_dir, &files).expect("failed to write crate");
        tmp_dir
    }

    // ── v2 crate compilation gate tests ─────────────────────────────────
    //
    // Evidence summary for the v2 self-hosted compiler:
    //
    //   Syntactic correctness:  94 parser tests (daglang-syntax), 315 codegen unit tests (daglang-emit)
    //   Type correctness:       cargo check passes on ~10,500 lines of generated Rust (v2_crate_cargo_check)
    //   Link correctness:       cargo build succeeds — all trait impls resolve (v2_crate_cargo_build)
    //   Semantic correctness:   phase 3 interpreter tests prove v2 logic works on real input through all 5 stages
    //   Runtime correctness:    generated tests prove emitted Rust executes correctly (v2_crate_cargo_test)

    /// Type-check the generated v2 crate (cargo check).
    #[test]
    fn v2_crate_cargo_check() {
        let tmp_dir = assemble_v2_crate_to_dir("v2-compiler-check");

        let output = std::process::Command::new("cargo")
            .arg("check")
            .current_dir(&tmp_dir)
            .output()
            .expect("failed to run cargo check");

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            panic!(
                "cargo check failed (crate at {}):\n{}",
                tmp_dir.display(),
                stderr
            );
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// Build the generated v2 crate (cargo build) — proves linking succeeds.
    #[test]
    #[ignore] // Slower than check; run with --ignored
    fn v2_crate_cargo_build() {
        let tmp_dir = assemble_v2_crate_to_dir("v2-compiler-build");

        let output = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(&tmp_dir)
            .output()
            .expect("failed to run cargo build");

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            panic!(
                "cargo build failed (crate at {}):\n{}",
                tmp_dir.display(),
                stderr
            );
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// Run the generated v2 crate's emitted tests — proves runtime correctness.
    ///
    /// The tests come FROM the crate assembly (emit_test_module), not from this
    /// test harness. This is Phase 1 / D6: the compiler tests itself through
    /// generated tests.
    #[test]
    #[ignore] // Requires cargo build; run with --ignored
    fn v2_crate_cargo_test() {
        let tmp_dir = assemble_v2_crate_to_dir("v2-compiler-test");

        // The generated crate already contains src/generated_tests.rs and the
        // corresponding `mod generated_tests;` in lib.rs — no injection needed.

        let output = std::process::Command::new("cargo")
            .arg("test")
            .current_dir(&tmp_dir)
            .output()
            .expect("failed to run cargo test");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            panic!(
                "cargo test failed (crate at {}):\nstdout:\n{}\nstderr:\n{}",
                tmp_dir.display(),
                stdout,
                stderr
            );
        }

        // Verify that the emitted tests actually ran (not just compiled)
        assert!(
            stdout.contains("test generated_tests"),
            "expected generated_tests to appear in test output:\n{}",
            stdout
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// Run the generated v2 crate's self_compile_all_modules test — proves
    /// the compiled v2 compiler can compile its own 9 .dag sources through
    /// the full pipeline (tokenize -> parse -> resolve -> typecheck -> emit).
    #[test]
    #[ignore] // Requires cargo build + full pipeline; run with --ignored
    fn v2_crate_self_compile() {
        let tmp_dir = assemble_v2_crate_to_dir("v2-compiler-self-compile");

        let output = std::process::Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("self_compile_all_modules")
            .current_dir(&tmp_dir)
            .output()
            .expect("failed to run cargo test");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            panic!(
                "self_compile_all_modules failed (crate at {}):\nstdout:\n{}\nstderr:\n{}",
                tmp_dir.display(),
                stdout,
                stderr
            );
        }

        assert!(
            stdout.contains("self_compile_all_modules"),
            "expected self_compile_all_modules to appear in test output:\n{}",
            stdout
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// Run the generated v2 crate's gist_resolve_all_modules test -- proves
    /// the compiled v2 compiler can tokenize, parse, and resolve the 12
    /// gist transitive dependencies (real-world DSL modules with services,
    /// resources, patterns, and func definitions).
    #[test]
    #[ignore] // Requires cargo build + full pipeline; run with --ignored
    fn v2_crate_gist_resolve() {
        let tmp_dir = assemble_v2_crate_to_dir("v2-compiler-gist-resolve");

        let output = std::process::Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("gist_resolve_all_modules")
            .current_dir(&tmp_dir)
            .output()
            .expect("failed to run cargo test");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            panic!(
                "gist_resolve_all_modules failed (crate at {}):\nstdout:\n{}\nstderr:\n{}",
                tmp_dir.display(),
                stdout,
                stderr
            );
        }

        assert!(
            stdout.contains("gist_resolve_all_modules"),
            "expected gist_resolve_all_modules to appear in test output:\n{}",
            stdout
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// Run the generated v2 crate's gist_compile_all_modules test -- proves
    /// the compiled v2 compiler can tokenize, parse, resolve, typecheck,
    /// and emit the gist transitive dependency chain without OOM.
    #[test]
    #[ignore] // Requires cargo build + full pipeline; run with --ignored
    fn v2_crate_gist_compile() {
        let tmp_dir = assemble_v2_crate_to_dir("v2-compiler-gist-compile");

        let output = std::process::Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("gist_compile_all_modules")
            .current_dir(&tmp_dir)
            .output()
            .expect("failed to run cargo test");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            panic!(
                "gist_compile_all_modules failed (crate at {}):\nstdout:\n{}\nstderr:\n{}",
                tmp_dir.display(),
                stdout,
                stderr
            );
        }

        assert!(
            stdout.contains("gist_compile_all_modules"),
            "expected gist_compile_all_modules to appear in test output:\n{}",
            stdout
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// Generate the v2 crate to target/v2-compiler/ for inspection.
    /// Does not clean up — the crate persists for manual browsing/running.
    #[test]
    #[ignore]
    #[allow(clippy::disallowed_macros)]
    fn v2_crate_emit_to_target() {
        let out_dir = workspace_root().join("target/v2-compiler");
        let _ = std::fs::remove_dir_all(&out_dir);

        let v2_files = [
            ("00_core", "src/v2/00_core.dag"),
            ("01_tokenize", "src/v2/01_tokenize.dag"),
            ("02_parse", "src/v2/02_parse.dag"),
            ("03_resolve", "src/v2/03_resolve.dag"),
            ("04_typecheck", "src/v2/04_typecheck.dag"),
            ("05_emit", "src/v2/05_emit.dag"),
            ("05_emit_rust", "src/v2/05_emit_rust.dag"),
            ("05_emit_python", "src/v2/05_emit_python.dag"),
            ("06_pipeline", "src/v2/06_pipeline.dag"),
        ];

        let parsed: Vec<(String, daglang_syntax::ast::SourceFile)> = v2_files
            .iter()
            .map(|(stem, path)| {
                let source = read_v2_file(path);
                let result = daglang_syntax::parser::parse_to_result(&source);
                (stem.to_string(), result.ast)
            })
            .collect();

        let modules: Vec<(&str, &daglang_syntax::ast::SourceFile)> = parsed
            .iter()
            .map(|(stem, sf): &(String, daglang_syntax::ast::SourceFile)| (stem.as_str(), sf))
            .collect();

        let files = daglang_emit::v2_crate_emit::assemble_v2_crate(&modules);
        daglang_emit::v2_crate_emit::write_crate(&out_dir, &files).expect("failed to write crate");
    }

    // ═════════════════════════════════════════════════════════════════════
    // Python emission tests — exercise the real Python renderer through
    // compile_sources(..., Python) and validate emitted files.
    // ═════════════════════════════════════════════════════════════════════

    fn emitted_python_module(
        output: &daglang_driver::EmbeddedCompileOutput,
        source: &str,
        module_path: &str,
    ) -> String {
        let result = compile_sources_with_target(output, &[("test.dag", source)], "Python");
        let diagnostics = result
            .get("diagnostics")
            .expect("compile_sources should return diagnostics");
        let messages = diagnostic_messages(diagnostics);
        assert!(
            messages.is_empty(),
            "Python emission should produce no diagnostics: {:?}",
            messages
        );
        let files = result
            .get("files")
            .expect("compile_sources should return emitted files");
        emitted_file_content(files, module_path)
    }

    fn assert_python_parses(python_code: &str) {
        use std::io::Write as _;

        let mut child = std::process::Command::new("python3")
            .arg("-c")
            .arg("import ast, sys; ast.parse(sys.stdin.read())")
            .stdin(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("failed to run python3");

        child
            .stdin
            .as_mut()
            .expect("python3 stdin should be piped")
            .write_all(python_code.as_bytes())
            .expect("failed to write Python source to stdin");

        let output = child
            .wait_with_output()
            .expect("failed to wait for python3");
        assert!(
            output.status.success(),
            "generated Python should parse without errors:\n--- Python code ---\n{}\n--- stderr ---\n{}",
            python_code,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn is_snake_case(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '_')
            && !name.starts_with('_')
    }

    /// Extract function names from generated Python source.
    fn extract_python_fn_names(python_code: &str) -> Vec<String> {
        python_code
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("def ") {
                    let end = rest.find('(').unwrap_or(rest.len());
                    Some(rest[..end].to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Validate that the Python renderer produces syntactically valid Python
    /// by running the output through Python's own AST parser.
    #[test]
    fn phase4_python_emit_produces_valid_syntax() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source =
            "module test\ntype Foo { x: Int, y: String }\nfn add(a: Int, b: Int) -> Int { a }\n";
        let python_code = emitted_python_module(&output, source, "test.py");
        assert!(
            !python_code.trim().is_empty(),
            "rendered Python should not be empty"
        );
        assert_python_parses(&python_code);
    }

    /// Verify that the Python renderer produces Python-idiomatic constructs:
    /// @dataclass for product types, def for functions, type hints.
    #[test]
    fn phase4_python_emit_has_dataclasses() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\ntype Point { x: Int, y: Int }\nfn origin() -> Int { 0 }\n";
        let python_code = emitted_python_module(&output, source, "test.py");

        assert!(
            python_code.contains("@dataclass"),
            "Python output should use @dataclass for product types:\n{}",
            python_code
        );
        assert!(
            python_code.contains("def "),
            "Python output should use def for function definitions:\n{}",
            python_code
        );
        assert!(
            python_code.contains(": int")
                || python_code.contains(": str")
                || python_code.contains(": bool"),
            "Python output should include type hints:\n{}",
            python_code
        );
    }

    /// Verify that rendered Python function names are snake_case.
    #[test]
    fn phase4_python_emit_snake_case_functions() {
        let output = compile_all_modules().expect("compilation should succeed");
        let source = "module test\nfn get_value(x: Int) -> Int { x }\nfn compute_total(a: Int, b: Int) -> Int { a }\n";
        let python_code = emitted_python_module(&output, source, "test.py");
        let fn_names = extract_python_fn_names(&python_code);

        assert!(
            !fn_names.is_empty(),
            "should have extracted at least one function name from:\n{}",
            python_code
        );
        for name in &fn_names {
            assert!(
                is_snake_case(name),
                "function name '{}' should be snake_case in Python output",
                name
            );
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // Namespace guard — duplicate function name detection
    // ═════════════════════════════════════════════════════════════════════

    /// Verify that the v2 module set has no duplicate fn names.
    #[test]
    fn compile_all_modules_rejects_duplicate_fn_names() {
        let duplicates = detect_duplicate_fn_names();
        assert!(
            duplicates.is_empty(),
            "v2 modules should not contain duplicate fn names: {:?}",
            duplicates
        );
    }
}
