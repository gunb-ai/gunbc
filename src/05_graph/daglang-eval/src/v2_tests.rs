//! Tests for the v2 self-hosted compiler .dag files.
//!
//! Validates that each v2 .dag file parses correctly with the v1 parser,
//! verifies structural properties of the parsed ASTs, and exercises
//! kernel intrinsics that the v2 tokenizer depends on.
//!
//! Note: some v2 .dag files use syntax constructs (multi-statement fn bodies,
//! pipe chains, lambda expressions) that the v1 parser does not yet fully
//! support. Tests for those files use the partial-recovery parser and verify
//! what structure IS recoverable, or are `#[ignore]`d pending v1 parser
//! extensions.

use std::collections::HashMap;

use crate::expr::{LoweredExpr, LoweredFnBody, LoweredLiteral, LoweredStmt};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Locate the workspace root by walking up from CARGO_MANIFEST_DIR.
fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // daglang-eval lives at src/05_graph/daglang-eval, so workspace root is 3 levels up.
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("could not find workspace root")
        .to_path_buf()
}

/// Read a .dag file relative to the workspace root, returning None if it does
/// not exist yet (parallel work units may not have created it).
fn read_dag_file(relative_path: &str) -> Option<String> {
    let path = workspace_root().join(relative_path);
    std::fs::read_to_string(&path).ok()
}

fn empty_siblings() -> HashMap<String, LoweredFnBody> {
    HashMap::new()
}

/// Helper: build a fn body that calls a kernel intrinsic and returns the result.
fn call_and_return(name: &str, args: Vec<(Option<String>, LoweredExpr)>) -> LoweredFnBody {
    LoweredFnBody {
        stmts: vec![LoweredStmt::Return(vec![(
            "return".to_string(),
            LoweredExpr::Call {
                name: name.to_string(),
                args,
            },
        )])],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Syntax validation — parse each v2 .dag file with the v1 parser
//
// core.dag is types-only and parses cleanly with v1.
// tokenize.dag, resolve.dag, and pipeline.dag use v2 syntax extensions
// (multi-statement fn bodies, pipe chains) that v1 cannot fully parse.
// For those files, we verify the partial AST recovers key structure.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn v2_core_dag_parses_cleanly() {
    let source = read_dag_file("src/v2/00_core.dag").expect("src/v2/00_core.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(
        result.is_ok(),
        "src/v2/00_core.dag should parse without errors, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

#[test]
fn v2_tokenize_dag_partial_parse_recovers_items() {
    // tokenize.dag uses v2 syntax that v1 cannot fully parse, but the
    // partial-recovery parser should still extract the module path and
    // some top-level items.
    let source = read_dag_file("src/v2/01_tokenize.dag")
        .expect("src/v2/01_tokenize.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);

    // Module path should be recovered even if fn bodies fail.
    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("partial parse should recover module path");
    assert_eq!(module_path.node.as_dotted(), "v2.compiler.tokenize");

    // At least some imports should be recovered.
    assert!(
        !result.ast.imports.is_empty(),
        "partial parse should recover at least one import"
    );
}

#[test]
fn v2_resolve_dag_partial_parse_recovers_module_path() {
    let source = read_dag_file("src/v2/03_resolve.dag")
        .expect("src/v2/03_resolve.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);

    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("partial parse should recover module path");
    assert_eq!(module_path.node.as_dotted(), "v2.compiler.resolve");
}

#[test]
fn v2_pipeline_dag_partial_parse_recovers_module_path() {
    let source = read_dag_file("src/v2/06_pipeline.dag")
        .expect("src/v2/06_pipeline.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);

    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("partial parse should recover module path");
    assert_eq!(module_path.node.as_dotted(), "v2.compiler.pipeline");
}

#[test]
fn v2_parse_dag_parses() {
    let source = read_dag_file("src/v2/02_parse.dag")
        .expect("src/v2/02_parse.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);
    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("should recover module path");
    assert_eq!(module_path.node.as_dotted(), "v2.compiler.parse");
}

#[test]
fn v2_typecheck_dag_parses() {
    let source = read_dag_file("src/v2/04_typecheck.dag")
        .expect("src/v2/04_typecheck.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);
    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("should recover module path");
    assert_eq!(module_path.node.as_dotted(), "v2.compiler.typecheck");
}

#[test]
fn v2_emit_dag_parses() {
    let source = read_dag_file("src/v2/05_emit.dag")
        .expect("src/v2/05_emit.dag should exist");
    let result = daglang_syntax::parser::parse_to_result(&source);
    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("should recover module path");
    assert_eq!(module_path.node.as_dotted(), "v2.compiler.emit");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Structural AST validation — verify parsed ASTs have expected structure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn v2_core_dag_has_module_declaration() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());
    let module_path = result
        .ast
        .module_path
        .as_ref()
        .expect("core.dag should declare a module path");
    assert_eq!(
        module_path.node.as_dotted(),
        "v2.std.core",
        "module path should be 'v2.std.core'"
    );
}

#[test]
fn v2_core_dag_defines_token_type() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());

    let has_token_type = result.ast.items.iter().any(|item| {
        matches!(
            &item.node,
            daglang_syntax::ast::Item::TypeDef(td) if td.name == "Token"
        )
    });
    assert!(has_token_type, "core.dag should define a 'Token' type");
}

#[test]
fn v2_core_dag_defines_token_kind_sum_type() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());

    let token_kind = result.ast.items.iter().find_map(|item| {
        if let daglang_syntax::ast::Item::TypeDef(td) = &item.node {
            if td.name == "TokenKind" {
                return Some(td);
            }
        }
        None
    });
    let td = token_kind.expect("core.dag should define a 'TokenKind' type");

    match &td.body {
        daglang_syntax::ast::TypeBody::Sum(variants) => {
            assert!(
                variants.len() > 30,
                "TokenKind should have 30+ variants, got {}",
                variants.len()
            );
            let variant_names: Vec<&str> = variants.iter().map(|v| v.name.as_str()).collect();
            assert!(
                variant_names.contains(&"KwModule"),
                "TokenKind should include KwModule"
            );
            assert!(
                variant_names.contains(&"Eof"),
                "TokenKind should include Eof"
            );
            assert!(
                variant_names.contains(&"LitStr"),
                "TokenKind should include LitStr"
            );
            assert!(
                variant_names.contains(&"Ident"),
                "TokenKind should include Ident"
            );
        }
        _ => panic!("TokenKind should be a sum type"),
    }
}

#[test]
fn v2_core_dag_defines_expr_type() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());

    let has_expr = result.ast.items.iter().any(|item| {
        matches!(
            &item.node,
            daglang_syntax::ast::Item::TypeDef(td) if td.name == "Expr"
        )
    });
    assert!(has_expr, "core.dag should define an 'Expr' type");
}

#[test]
fn v2_core_dag_defines_module_type() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());

    let has_module = result.ast.items.iter().any(|item| {
        matches!(
            &item.node,
            daglang_syntax::ast::Item::TypeDef(td) if td.name == "Module"
        )
    });
    assert!(has_module, "core.dag should define a 'Module' type");
}

#[test]
fn v2_core_dag_defines_compile_result_type() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());

    let has_compile_result = result.ast.items.iter().any(|item| {
        matches!(
            &item.node,
            daglang_syntax::ast::Item::TypeDef(td) if td.name == "CompileResult"
        )
    });
    assert!(
        has_compile_result,
        "core.dag should define a 'CompileResult' type"
    );
}

#[test]
fn v2_core_dag_imports_std_types() {
    let source = read_dag_file("src/v2/00_core.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);
    assert!(result.is_ok());

    let imports_std = result
        .ast
        .imports
        .iter()
        .any(|imp| imp.node.path.as_dotted() == "std.types");
    assert!(imports_std, "core.dag should import from std.types");
}

#[test]
fn v2_tokenize_dag_imports_core_types() {
    // Even though tokenize.dag has parse errors, the partial parser should
    // recover the imports section.
    let source = read_dag_file("src/v2/01_tokenize.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);

    let imports_core = result
        .ast
        .imports
        .iter()
        .any(|imp| imp.node.path.as_dotted() == "v2.std.core");
    assert!(
        imports_core,
        "tokenize.dag should import from v2.std.core"
    );
}

#[test]
fn v2_tokenize_dag_has_data_declarations() {
    // Data declarations (keywords, single_punct) appear before fn bodies,
    // so the partial parser should recover them.
    let source = read_dag_file("src/v2/01_tokenize.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);

    let data_names: Vec<&str> = result
        .ast
        .items
        .iter()
        .filter_map(|item| {
            if let daglang_syntax::ast::Item::DataDef(dd) = &item.node {
                Some(dd.name.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(
        data_names.contains(&"keywords"),
        "tokenize.dag should have 'keywords' data declaration, found: {:?}",
        data_names
    );
    assert!(
        data_names.contains(&"single_punct"),
        "tokenize.dag should have 'single_punct' data declaration, found: {:?}",
        data_names
    );
}

#[test]
fn v2_resolve_dag_partial_parse_recovers_types() {
    // The type declarations at the top of resolve.dag should be recoverable
    // even though the fn bodies may fail.
    let source = read_dag_file("src/v2/03_resolve.dag").unwrap();
    let result = daglang_syntax::parser::parse_to_result(&source);

    let type_names: Vec<&str> = result
        .ast
        .items
        .iter()
        .filter_map(|item| {
            if let daglang_syntax::ast::Item::TypeDef(td) = &item.node {
                Some(td.name.as_str())
            } else {
                None
            }
        })
        .collect();

    assert!(
        type_names.contains(&"ModuleGraph"),
        "resolve.dag should define 'ModuleGraph', found: {:?}",
        type_names
    );
    assert!(
        type_names.contains(&"ResolvedModule"),
        "resolve.dag should define 'ResolvedModule', found: {:?}",
        type_names
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Kernel intrinsic tests — validate intrinsics the v2 tokenizer depends on
//
// The v2 tokenizer uses: char_at, string_length, substring, parse_int,
// scan_while, skip_horizontal_ws, scan_to_eol. These tests verify the
// Rust implementations in the evaluator match expected behavior.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn kernel_char_at() {
    let body = call_and_return(
        "char_at",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
            ),
            (
                Some("pos".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(1)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Str("e".to_string()));
}

#[test]
fn kernel_char_at_first() {
    let body = call_and_return(
        "char_at",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
            ),
            (
                Some("pos".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Str("h".to_string()));
}

#[test]
fn kernel_char_at_last() {
    let body = call_and_return(
        "char_at",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
            ),
            (
                Some("pos".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(4)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Str("o".to_string()));
}

#[test]
fn kernel_char_at_out_of_bounds_returns_unit() {
    // The v2 tokenizer uses char_at to check if we've reached EOF.
    // Out-of-bounds returns Unit (not empty string).
    let body = call_and_return(
        "char_at",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hi".to_string())),
            ),
            (
                Some("pos".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(5)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Unit);
}

#[test]
fn kernel_string_length() {
    let body = call_and_return(
        "string_length",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Int(5));
}

#[test]
fn kernel_string_length_empty() {
    let body = call_and_return(
        "string_length",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String(String::new())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Int(0));
}

#[test]
fn kernel_string_length_unicode() {
    // string_length counts characters, not bytes.
    let body = call_and_return(
        "string_length",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String("cafe\u{0301}".to_string())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // "cafe\u{0301}" has 5 characters: c, a, f, e, combining accent
    assert_eq!(result["return"], gunbc_ir::Value::Int(5));
}

#[test]
fn kernel_substring() {
    let body = call_and_return(
        "substring",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello world".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
            (
                Some("end".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(5)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Str("hello".to_string()));
}

#[test]
fn kernel_substring_mid() {
    let body = call_and_return(
        "substring",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello world".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(6)),
            ),
            (
                Some("end".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(11)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Str("world".to_string()));
}

#[test]
fn kernel_substring_empty() {
    let body = call_and_return(
        "substring",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(2)),
            ),
            (
                Some("end".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(2)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Str("".to_string()));
}

#[test]
fn kernel_parse_int() {
    let body = call_and_return(
        "parse_int",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String("42".to_string())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Int(42));
}

#[test]
fn kernel_parse_int_negative() {
    let body = call_and_return(
        "parse_int",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String("-7".to_string())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Int(-7));
}

#[test]
fn kernel_parse_int_zero() {
    let body = call_and_return(
        "parse_int",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String("0".to_string())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    assert_eq!(result["return"], gunbc_ir::Value::Int(0));
}

#[test]
fn kernel_parse_int_invalid() {
    let body = call_and_return(
        "parse_int",
        vec![(
            Some("s".to_string()),
            LoweredExpr::Literal(LoweredLiteral::String("abc".to_string())),
        )],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings());
    assert!(result.is_err(), "parse_int('abc') should return an error");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Scan intrinsic tests — scan_to_eol, skip_horizontal_ws
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn kernel_scan_to_eol() {
    let body = call_and_return(
        "scan_to_eol",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello\nworld".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // scan_to_eol should return the position of the newline (5).
    assert_eq!(result["return"], gunbc_ir::Value::Int(5));
}

#[test]
fn kernel_scan_to_eol_no_newline() {
    let body = call_and_return(
        "scan_to_eol",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // No newline, should return string length (5).
    assert_eq!(result["return"], gunbc_ir::Value::Int(5));
}

#[test]
fn kernel_skip_horizontal_ws() {
    let body = call_and_return(
        "skip_horizontal_ws",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("   hello".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // Should skip 3 spaces, returning position 3.
    assert_eq!(result["return"], gunbc_ir::Value::Int(3));
}

#[test]
fn kernel_skip_horizontal_ws_tabs() {
    let body = call_and_return(
        "skip_horizontal_ws",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("\t\thello".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // Should skip 2 tabs, returning position 2.
    assert_eq!(result["return"], gunbc_ir::Value::Int(2));
}

#[test]
fn kernel_skip_horizontal_ws_no_ws() {
    let body = call_and_return(
        "skip_horizontal_ws",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("hello".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // No whitespace to skip, should return start position (0).
    assert_eq!(result["return"], gunbc_ir::Value::Int(0));
}

#[test]
fn kernel_skip_horizontal_ws_does_not_skip_newlines() {
    let body = call_and_return(
        "skip_horizontal_ws",
        vec![
            (
                Some("s".to_string()),
                LoweredExpr::Literal(LoweredLiteral::String("\nhello".to_string())),
            ),
            (
                Some("start".to_string()),
                LoweredExpr::Literal(LoweredLiteral::Int(0)),
            ),
        ],
    );
    let result = crate::evaluate_fn_body(&body, &HashMap::new(), &empty_siblings()).unwrap();
    // Newlines are NOT horizontal whitespace, should return 0.
    assert_eq!(result["return"], gunbc_ir::Value::Int(0));
}
