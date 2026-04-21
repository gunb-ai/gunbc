//! Shared `regen_parse_tables` emission: compile `parse_tables.dag` +
//! `tokenize.dag`, project into the SG-2c-1 grammar-tables Rust module,
//! run `rustfmt --emit stdout`. Used by the `regen_parse_tables` binary
//! (writes the file) and by the hermetic integration snapshot test
//! (compare in-memory only — avoids a `cargo run` subprocess that blows
//! the 2s per-test ratchet on cold CI).

use std::collections::BTreeSet;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::compile_to_dag;
use crate::dag::{
    ArithmeticOp, ComparisonOp, Dag, Field, FieldValue, LiteralBits, LogicalOp, OperatorKind,
    TypeConnective, ValueBody,
};
use crate::operators;
use crate::CompileError;

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/parse_tables.dag` via\n\
     // `regen_parse_tables`. Regenerate instead of hand-editing.\n\n";

/// Failure compiling either authority DAG or running `rustfmt` on the combined module text.
#[derive(Debug)]
pub enum RenderParseTablesGeneratedError {
    Compile(Box<CompileError>),
    Rustfmt(String),
}

impl fmt::Display for RenderParseTablesGeneratedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rustfmt(msg) => write!(f, "{msg}"),
            Self::Compile(e) => match e.as_ref() {
                CompileError::Semantic(d) => {
                    writeln!(f, "compile failed:")?;
                    for (_, diag) in d.diagnostics().iter() {
                        writeln!(f, "  {diag:?}")?;
                    }
                    Ok(())
                }
                other => write!(f, "{other:?}"),
            },
        }
    }
}

/// Compile `parse_tables.dag` and `tokenize.dag`, cross-validate the rows
/// against `TokenKind` + `operators.dag::from_symbol`, emit the projection,
/// format with `rustfmt --emit stdout`. Does not read or write workspace paths.
pub fn render_parse_tables_generated_rs(
    parse_tables_source: &str,
    parse_tables_file: &str,
    tokenize_source: &str,
    tokenize_file: &str,
) -> Result<String, RenderParseTablesGeneratedError> {
    let tables_dag = compile_authority(parse_tables_source, parse_tables_file)?;
    let tokenize_dag = compile_authority(tokenize_source, tokenize_file)?;

    let token_variants = collect_variant_labels(&tokenize_dag, "TokenKind");
    let levels = collect_variant_labels(&tables_dag, "BinaryOpLevel");
    let rows = collect_binary_op_rows(&tables_dag, &token_variants, &levels);

    let rust = emit_module(&levels, &rows);
    let combined = format!("{HEADER}{rust}");
    rustfmt_stdout(&combined).map_err(RenderParseTablesGeneratedError::Rustfmt)
}

fn compile_authority(source: &str, file: &str) -> Result<Dag, RenderParseTablesGeneratedError> {
    compile_to_dag(source, file).map_err(|e| RenderParseTablesGeneratedError::Compile(Box::new(e)))
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

    // Discover rows by typed fact (`decl.meta_tag == BinaryOpRow`),
    // not by `name.starts_with("binary_op_")`. Mirrors the registry
    // pattern `regen_lens` uses against `LensRegistryEntry`; avoids
    // baking a name-prefix calling convention into the consumer,
    // which would let a mistyped row escape (a `BinaryOpRow` bound
    // to a name that happens not to start with `binary_op_`) or a
    // correctly-prefixed row of the wrong type sneak in.
    let binary_op_row_type_id = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("BinaryOpRow"))
        .map(|d| d.id)
        .expect("BinaryOpRow declaration");

    let mut rows: Vec<BinaryOpRow> = Vec::new();
    let mut seen_token_variants: BTreeSet<String> = BTreeSet::new();
    for decl in dag.declarations() {
        if decl.meta_tag != Some(binary_op_row_type_id) {
            continue;
        }
        let name = decl.name.as_deref().unwrap_or("<anonymous>");
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!("`parse_tables.dag::{name}`: `BinaryOpRow` binding must carry a structural value body");
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
            operators::from_symbol(&operator_symbol).is_some(),
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
    level_variants: &[Field],
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
        .unwrap_or_else(|| {
            panic!(
                "`{name}`: field `{key}` constructor {constructor:?} is not a BinaryOpLevel variant"
            )
        })
}

/// Render an `OperatorKind` as the Rust source expression that constructs it.
/// Structural projection — reads the value, emits the matching constructor
/// path — with no symbol → operator table. The symbol → `OperatorKind`
/// authority is `operators.dag::from_symbol`; this driver consumes that
/// projection via [`operators::from_symbol`] and only formats the result.
fn operator_kind_expr(op: OperatorKind) -> String {
    match op {
        OperatorKind::Arithmetic(a) => format!(
            "OperatorKind::Arithmetic(ArithmeticOp::{})",
            arithmetic_variant(a)
        ),
        OperatorKind::Comparison(c) => format!(
            "OperatorKind::Comparison(ComparisonOp::{})",
            comparison_variant(c)
        ),
        OperatorKind::Logical(l) => {
            format!("OperatorKind::Logical(LogicalOp::{})", logical_variant(l))
        }
    }
}

fn arithmetic_variant(a: ArithmeticOp) -> &'static str {
    match a {
        ArithmeticOp::Add => "Add",
        ArithmeticOp::Sub => "Sub",
        ArithmeticOp::Mul => "Mul",
        ArithmeticOp::Div => "Div",
    }
}

fn comparison_variant(c: ComparisonOp) -> &'static str {
    match c {
        ComparisonOp::Eq => "Eq",
        ComparisonOp::Ne => "Ne",
        ComparisonOp::Lt => "Lt",
        ComparisonOp::Le => "Le",
        ComparisonOp::Gt => "Gt",
        ComparisonOp::Ge => "Ge",
    }
}

fn logical_variant(l: LogicalOp) -> &'static str {
    match l {
        LogicalOp::And => "And",
        LogicalOp::Or => "Or",
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
        let op = operators::from_symbol(&row.operator_symbol)
            .expect("validated in collect_binary_op_rows");
        s.push_str(&format!(
            "        (TokenKind::{}, BinaryOpLevel::{}) => Some({}),\n",
            row.token_variant,
            row.level,
            operator_kind_expr(op)
        ));
    }
    s.push_str("        _ => None,\n");
    s.push_str("    }\n");
    s.push_str("}\n");
    s
}

fn rustfmt_stdout(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(source.as_bytes())
        .map_err(|e| format!("write rustfmt stdin: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("rustfmt stdout utf-8: {e}"))
}
