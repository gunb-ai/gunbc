//! SG-2c-1 — Regenerate `parse_tables_generated.rs` from
//! `src/v3/compiler/parse_tables.dag`.
//!
//! Scope (grammar-tables prototype, NOT SG-2c proper):
//! - reads the `BinaryOpLevel` enum declaration from `parse_tables.dag`
//! - reads each `binary_op_*: BinaryOpRow` data row
//! - cross-validates `token_variant` against `tokenize.dag`'s `TokenKind`
//!   declaration and `operator_symbol` against `operators.dag`'s
//!   `from_symbol` (drift in either direction fails closed here)
//! - emits a `BinaryOpLevel` enum and a `binary_op_at_level` helper
//!   so the per-level parser functions in `parse_parser_body.txt` stop
//!   open-coding the token → `OperatorKind` match.
//!
//! Parser control flow (cursor mechanics, recursive descent, error
//! recovery) stays in `parse_parser_body.txt` — those require
//! recursive list-body emission over `List<Token>` which v3 does not
//! yet support. See `parse_tables.dag` header for the dissolution
//! trigger.
//!
//! Dissolution trigger: once SG-2c proper ports the parser algorithm
//! itself into `.dag`, this driver either folds into a unified
//! `regen_parse` or its output becomes one of many sections emitted
//! from a full `parse.dag`.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::generated_files::GENERATED_FILES;
use v3_compiler::CompileError;

const GENERATED_FILE: &str = "src/v3/compiler/src/parse_tables_generated.rs";
const PARSE_TABLES_AUTHORITY_FILE: &str = "src/v3/compiler/parse_tables.dag";
const TOKENIZE_AUTHORITY_FILE: &str = "src/v3/compiler/tokenize.dag";

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/parse_tables.dag` via\n\
     // `regen_parse_tables`. Regenerate instead of hand-editing.\n\n";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_parse_tables` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let tables_source = std::fs::read_to_string(manifest_dir.join("parse_tables.dag"))
        .expect("read parse_tables.dag");
    let tables_dag = compile(&tables_source, PARSE_TABLES_AUTHORITY_FILE);

    let tokenize_source =
        std::fs::read_to_string(manifest_dir.join("tokenize.dag")).expect("read tokenize.dag");
    let tokenize_dag = compile(&tokenize_source, TOKENIZE_AUTHORITY_FILE);

    let token_variants = collect_variant_labels(&tokenize_dag, "TokenKind");
    let levels = collect_variant_labels(&tables_dag, "BinaryOpLevel");
    let rows = collect_binary_op_rows(&tables_dag, &token_variants, &levels);

    let rust = emit_module(&levels, &rows);
    let combined = format!("{HEADER}{rust}");
    let formatted = rustfmt(&combined);

    let out_path = manifest_dir.join("src").join("parse_tables_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write parse_tables_generated.rs");
    println!("wrote {}", out_path.display());
}

fn compile(source: &str, file: &str) -> Dag {
    compile_to_dag(source, file).unwrap_or_else(|e| match e {
        CompileError::Semantic(d) => {
            let mut msg = format!("compile {file} failed:\n");
            for (_, diag) in d.diagnostics().iter() {
                msg.push_str(&format!("  {diag:?}\n"));
            }
            panic!("{msg}");
        }
        other => panic!("compile {file}: {other:?}"),
    })
}

fn collect_variant_labels(dag: &Dag, type_name: &str) -> Vec<String> {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some(type_name))
        .unwrap_or_else(|| panic!("missing `{type_name}` declaration"));
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("`{type_name}`: expected Disj");
    };
    variants.iter().map(|v| v.label.clone()).collect()
}

struct BinaryOpRow {
    token_variant: String,
    operator_symbol: String,
    level: String,
}

fn collect_binary_op_rows(
    dag: &Dag,
    token_variants: &[String],
    levels: &[String],
) -> Vec<BinaryOpRow> {
    let token_variant_set: BTreeSet<&str> = token_variants.iter().map(String::as_str).collect();
    let level_set: BTreeSet<&str> = levels.iter().map(String::as_str).collect();

    let level_type_id = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("BinaryOpLevel"))
        .map(|d| d.id)
        .expect("BinaryOpLevel declaration");
    let TypeConnective::Disj {
        variants: level_variants,
    } = &dag.declaration(level_type_id).connective
    else {
        panic!("BinaryOpLevel should be a Disj");
    };

    let mut rows: Vec<BinaryOpRow> = Vec::new();
    let mut seen_token_variants: BTreeSet<String> = BTreeSet::new();
    for decl in dag.declarations() {
        let Some(name) = &decl.name else { continue };
        if !name.starts_with("binary_op_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        let token_variant = string_field(fields, "token_variant", name);
        let operator_symbol = string_field(fields, "operator_symbol", name);
        let level = variant_field(fields, "level", level_variants, name);

        assert!(
            token_variant_set.contains(token_variant.as_str()),
            "`parse_tables.dag::{name}`: token_variant `{token_variant}` is not a \
             `TokenKind` variant in `tokenize.dag`"
        );
        assert!(
            level_set.contains(level.as_str()),
            "`parse_tables.dag::{name}`: level `{level}` is not a `BinaryOpLevel` variant"
        );
        assert!(
            operator_kind_expr_from_symbol(&operator_symbol).is_some(),
            "`parse_tables.dag::{name}`: operator_symbol `{operator_symbol}` has no \
             `OperatorKind` mapping in `operators.dag::from_symbol`"
        );
        assert!(
            seen_token_variants.insert(token_variant.clone()),
            "`parse_tables.dag`: duplicate row for `TokenKind::{token_variant}`"
        );

        rows.push(BinaryOpRow {
            token_variant,
            operator_symbol,
            level,
        });
    }
    rows.sort_by(|a, b| a.token_variant.cmp(&b.token_variant));
    rows
}

fn string_field(fields: &[(String, FieldValue)], key: &str, name: &str) -> String {
    match fields.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
        Some(FieldValue::Literal(LiteralBits::String(s))) => s.clone(),
        other => panic!("`{name}`: field `{key}` should be a string literal, got {other:?}"),
    }
}

fn variant_field(
    fields: &[(String, FieldValue)],
    key: &str,
    level_variants: &[v3_compiler::dag::Field],
    name: &str,
) -> String {
    let fv = fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("`{name}`: missing field `{key}`"));
    let FieldValue::Variant {
        constructor,
        payload,
    } = fv
    else {
        panic!("`{name}`: field `{key}` should be a variant value, got {fv:?}");
    };
    assert!(
        payload.is_empty(),
        "`{name}`: field `{key}` expects a nullary variant"
    );
    level_variants
        .iter()
        .find(|v| v.ty == *constructor)
        .map(|v| v.label.clone())
        .unwrap_or_else(|| panic!("`{name}`: field `{key}` constructor {constructor:?} is not a BinaryOpLevel variant"))
}

fn operator_kind_expr_from_symbol(symbol: &str) -> Option<&'static str> {
    match symbol {
        "+" => Some("OperatorKind::Arithmetic(ArithmeticOp::Add)"),
        "-" => Some("OperatorKind::Arithmetic(ArithmeticOp::Sub)"),
        "*" => Some("OperatorKind::Arithmetic(ArithmeticOp::Mul)"),
        "/" => Some("OperatorKind::Arithmetic(ArithmeticOp::Div)"),
        "==" => Some("OperatorKind::Comparison(ComparisonOp::Eq)"),
        "!=" => Some("OperatorKind::Comparison(ComparisonOp::Ne)"),
        "<" => Some("OperatorKind::Comparison(ComparisonOp::Lt)"),
        "<=" => Some("OperatorKind::Comparison(ComparisonOp::Le)"),
        ">" => Some("OperatorKind::Comparison(ComparisonOp::Gt)"),
        ">=" => Some("OperatorKind::Comparison(ComparisonOp::Ge)"),
        "&&" => Some("OperatorKind::Logical(LogicalOp::And)"),
        "||" => Some("OperatorKind::Logical(LogicalOp::Or)"),
        _ => None,
    }
}

fn emit_module(levels: &[String], rows: &[BinaryOpRow]) -> String {
    let mut s = String::new();
    s.push_str("use crate::dag::{ArithmeticOp, ComparisonOp, LogicalOp, OperatorKind};\n");
    s.push_str("use crate::tokenize::TokenKind;\n\n");

    s.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    s.push_str("pub enum BinaryOpLevel {\n");
    for level in levels {
        s.push_str(&format!("    {level},\n"));
    }
    s.push_str("}\n\n");

    s.push_str(
        "/// Token-to-`OperatorKind` map for the parser's per-precedence-level\n\
         /// functions. Authored as `binary_op_*: BinaryOpRow` rows in\n\
         /// `src/v3/compiler/parse_tables.dag`; regenerated by `regen_parse_tables`.\n",
    );
    s.push_str(
        "pub fn binary_op_at_level(tk: &TokenKind, level: BinaryOpLevel) -> Option<OperatorKind> {\n",
    );
    s.push_str("    match (tk, level) {\n");
    for row in rows {
        let op = operator_kind_expr_from_symbol(&row.operator_symbol)
            .expect("validated in collect_binary_op_rows");
        s.push_str(&format!(
            "        (TokenKind::{}, BinaryOpLevel::{}) => Some({}),\n",
            row.token_variant, row.level, op
        ));
    }
    s.push_str("        _ => None,\n");
    s.push_str("    }\n");
    s.push_str("}\n");
    s
}

fn rustfmt(source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn rustfmt");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(source.as_bytes())
        .expect("write rustfmt stdin");
    let output = child.wait_with_output().expect("rustfmt wait");
    assert!(
        output.status.success(),
        "rustfmt failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("rustfmt output not utf8")
}
