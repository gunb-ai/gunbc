//! Shared `regen_parse_tables` emission: compile `parse_tables.dag` +
//! `tokenize.dag`, cross-validate rows, project into `parse_tables_generated.rs`,
//! run `rustfmt --emit stdout`.
//!
//! **Tables projected.** **SG-2c-1:** `BinaryOpRow` → `binary_op_at_level` (plus
//! shared-syntax checks below). **SG-2c-2:** `TopLevelItemKwRow` →
//! `ItemDispatchKind` + `top_level_item_dispatch` (token rows validated against
//! `tokenize.dag`'s `TokenKind` variants).
//!
//! Used by the `regen_parse_tables` binary (writes the file) and by the
//! hermetic integration snapshot test (compare in-memory only — avoids a
//! `cargo run` subprocess that blows the 2s per-test ratchet on cold CI).
//!
//! **Cross-validation against shared authority.** `dag_operators` in
//! `syntax.dag` owns the canonical (symbol, left_bp, right_bp, binop) for
//! every binary operator. Each `BinaryOpRow` in `parse_tables.dag` is
//! checked against that authority: the row's `operator_symbol` must
//! appear in `dag_operators`, and its declared `level` must be consistent
//! with the shared row's binding power (the coarse parser precedence
//! level covers a contiguous bp range — see `level_for_bp`). If the
//! shared authority shifts an operator's bp into a different level, or
//! drops a symbol, regen fails closed here. This closes the token ↔
//! symbol ↔ precedence drift surface the initial SG-2c-1 landing left
//! open (`parse_tables.dag` used to carry `(symbol, level)` without any
//! structural tie to the shared authority's bp).
//!
//! **SG-1a scaffold extension.** `syntax.dag`'s `dag_operators` body
//! still lowers as `ValueBody::Unparsed` (same scaffold `regen_tokenize`
//! extends), so this bridge reads the raw source text. When the shared
//! authority lowers structurally, the raw-text extractor folds into a
//! typed read — same dissolution trigger as `regen_tokenize`.

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

/// Compile `parse_tables.dag` and `tokenize.dag`, cross-validate rows, emit
/// `parse_tables_generated.rs`, format with `rustfmt --emit stdout`.
///
/// Validation: `BinaryOpRow` vs `tokenize.dag` + `operators.dag` + shared-syntax
/// `dag_operators`; `TopLevelItemKwRow` vs `tokenize.dag` + `Kw`/`ItemDispatchKind`
/// pairing (SG-2c-2). Does not read or write workspace paths.
pub fn render_parse_tables_generated_rs(
    parse_tables_source: &str,
    parse_tables_file: &str,
    tokenize_source: &str,
    tokenize_file: &str,
    shared_syntax_source: &str,
) -> Result<String, RenderParseTablesGeneratedError> {
    let tables_dag = compile_authority(parse_tables_source, parse_tables_file)?;
    let tokenize_dag = compile_authority(tokenize_source, tokenize_file)?;
    let shared_operators = extract_shared_operator_bps(shared_syntax_source);

    let token_variants = collect_variant_labels(&tokenize_dag, "TokenKind");
    let levels = collect_variant_labels(&tables_dag, "BinaryOpLevel");
    let rows = collect_binary_op_rows(&tables_dag, &token_variants, &levels, &shared_operators);

    let item_dispatch_variants = collect_variant_labels(&tables_dag, "ItemDispatchKind");
    let item_kw_rows = collect_top_level_item_kw_rows(&tables_dag, &token_variants);

    let rust = emit_module(&levels, &rows, &item_dispatch_variants, &item_kw_rows);
    let combined = format!("{HEADER}{rust}");
    rustfmt_stdout(&combined).map_err(RenderParseTablesGeneratedError::Rustfmt)
}

/// Coarse parser precedence level implied by a shared-authority
/// `left_bp`. The five levels in `BinaryOpLevel` each correspond to one
/// per-precedence-level parser function in `parse_parser_body.txt`,
/// which accepts a contiguous bp band:
/// - `Or` ← bp 5/6 (`||`)
/// - `And` ← bp 7/8 (`&&`)
/// - `Comparison` ← bp 9/10 or 11/12 (`== != < <= > >=`)
/// - `Additive` ← bp 13/14 (`+ -`)
/// - `Multiplicative` ← bp 15/16 (`* / %`)
///
/// If the shared authority introduces an operator at a bp outside these
/// bands, regen fails closed (`BinaryOpLevel` does not yet cover it).
fn level_for_bp(left_bp: i64) -> Option<&'static str> {
    match left_bp {
        5 => Some("Or"),
        7 => Some("And"),
        9 | 11 => Some("Comparison"),
        13 => Some("Additive"),
        15 => Some("Multiplicative"),
        _ => None,
    }
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

/// Scratch shape for SG-2c-2 emission: `dispatch` is derived from `token_variant`
/// (`Kw`-strip rule); it is never read from the `.dag` row body (substrate carries
/// only `token_variant`).
struct TopLevelItemKwRow {
    token_variant: String,
    dispatch: String,
}

fn collect_binary_op_rows(
    dag: &Dag,
    token_variants: &[String],
    levels: &[String],
    shared_operators: &std::collections::BTreeMap<String, SharedOperatorSpec>,
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
        let level = variant_field(fields, "level", level_variants, name, "BinaryOpLevel");

        assert!(
            token_variant_set.contains(token_variant.as_str()),
            "`parse_tables.dag::{name}`: token_variant `{token_variant}` is not a \
             `TokenKind` variant in `tokenize.dag`"
        );
        assert!(
            level_set.contains(level.as_str()),
            "`parse_tables.dag::{name}`: level `{level}` is not a `BinaryOpLevel` variant"
        );
        let operator_kind = operators::from_symbol(&operator_symbol).unwrap_or_else(|| {
            panic!(
                "`parse_tables.dag::{name}`: operator_symbol `{operator_symbol}` has no \
                     `OperatorKind` mapping in `operators.dag::from_symbol`"
            )
        });
        // Structural cross-validation against the shared syntax
        // authority at `dsl/extdeps/languages/dag/syntax.dag`. Three
        // joins, closing three drift surfaces:
        //  (1) symbol ∈ `dag_operators` — operator is declared
        //      canonically; deletion there fails regen here.
        //  (2) shared `left_bp` implies the declared `level` via
        //      `level_for_bp` — precedence drift fails closed.
        //  (3) `operators.dag::from_symbol(symbol)` returns an
        //      `OperatorKind` whose leaf variant matches the shared
        //      `binop` name — `symbol → OperatorKind` drift between
        //      `operators.dag` and `dag_operators` fails closed.
        // Together, these turn `parse_tables.dag` into a structural
        // projection of the shared authority rather than a parallel
        // restatement of it.
        let shared = shared_operators.get(&operator_symbol).unwrap_or_else(|| {
            panic!(
                "`parse_tables.dag::{name}`: operator_symbol `{operator_symbol}` is not declared \
                 in `dag_operators` at `dsl/extdeps/languages/dag/syntax.dag`. Every binary-operator \
                 row must be grounded in the shared-syntax authority so drift fails closed."
            )
        });
        let expected_level = level_for_bp(shared.left_bp).unwrap_or_else(|| {
            panic!(
                "`parse_tables.dag::{name}`: shared-syntax `{operator_symbol}` has `left_bp = {}`, \
                 which falls outside any `BinaryOpLevel` bp band. Either the shared authority moved \
                 the operator's precedence or a new band is needed in `level_for_bp` + `BinaryOpLevel`.",
                shared.left_bp
            )
        });
        assert!(
            expected_level == level,
            "`parse_tables.dag::{name}`: declared level `{level}` disagrees with shared-syntax \
             authority. `dag_operators` assigns `{operator_symbol}` `left_bp = {}` which \
             implies level `{expected_level}`. Either update the row to match or reclassify \
             `left_bp = {}` in `level_for_bp`.",
            shared.left_bp,
            shared.left_bp
        );
        let actual_binop = operator_kind_binop_name(operator_kind);
        assert!(
            actual_binop == shared.binop_name,
            "`parse_tables.dag::{name}`: `operators.dag::from_symbol(\"{operator_symbol}\")` returns \
             `OperatorKind` leaf `{actual_binop}`, but `dag_operators` in \
             `dsl/extdeps/languages/dag/syntax.dag` records `binop: {}` for the same symbol. One of \
             the two authorities drifted — fix both to agree before shipping.",
            shared.binop_name
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

fn collect_top_level_item_kw_rows(dag: &Dag, token_variants: &[String]) -> Vec<TopLevelItemKwRow> {
    let token_variant_set: BTreeSet<&str> = token_variants.iter().map(String::as_str).collect();
    let dispatch_labels = collect_variant_labels(dag, "ItemDispatchKind");
    let dispatch_set: BTreeSet<&str> = dispatch_labels.iter().map(String::as_str).collect();

    let row_type_id = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("TopLevelItemKwRow"))
        .map(|d| d.id)
        .expect("TopLevelItemKwRow declaration");

    let mut rows: Vec<TopLevelItemKwRow> = Vec::new();
    let mut seen_token_variants: BTreeSet<String> = BTreeSet::new();
    for decl in dag.declarations() {
        if decl.meta_tag != Some(row_type_id) {
            continue;
        }
        let name = decl.name.as_deref().unwrap_or("<anonymous>");
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!(
                "`parse_tables.dag::{name}`: `TopLevelItemKwRow` binding must carry a structural value body"
            );
        };
        let token_variant = string_field(fields, "token_variant", name);

        assert!(
            token_variant_set.contains(token_variant.as_str()),
            "`parse_tables.dag::{name}`: token_variant `{token_variant}` is not a \
             `TokenKind` variant in `tokenize.dag`"
        );

        let dispatch =
            item_dispatch_label_from_kw_token_variant(&token_variant, name, &dispatch_set);
        assert_eq!(
            format!("Kw{dispatch}"),
            token_variant,
            "`parse_tables.dag::{name}`: token_variant `{token_variant}` must equal `Kw` + \
             `ItemDispatchKind` label `{dispatch}` (single representation; strip `Kw` for dispatch)"
        );

        assert!(
            seen_token_variants.insert(token_variant.clone()),
            "`parse_tables.dag`: duplicate top-level item row for `TokenKind::{token_variant}`"
        );

        rows.push(TopLevelItemKwRow {
            token_variant,
            dispatch,
        });
    }
    rows.sort_by(|a, b| a.token_variant.cmp(&b.token_variant));
    rows
}

/// `TopLevelItemKwRow.token_variant` is authoritative; `ItemDispatchKind`
/// matches `strip_prefix("Kw")` — same rule as `parse_tables.dag` header.
fn item_dispatch_label_from_kw_token_variant(
    token_variant: &str,
    decl_name: &str,
    dispatch_set: &BTreeSet<&str>,
) -> String {
    let Some(rest) = token_variant.strip_prefix("Kw") else {
        panic!(
            "`parse_tables.dag::{decl_name}`: token_variant `{token_variant}` must start with \
             `Kw` so `ItemDispatchKind` is derivable per modeling-discipline §4 / INVARIANTS no-duplicate-representations"
        );
    };
    assert!(
        !rest.is_empty(),
        "`parse_tables.dag::{decl_name}`: token_variant `{token_variant}` has nothing after `Kw`"
    );
    assert!(
        dispatch_set.contains(rest),
        "`parse_tables.dag::{decl_name}`: `{token_variant}` implies dispatch `{rest}`, which is \
         not an `ItemDispatchKind` variant"
    );
    rest.to_string()
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
    variants: &[Field],
    name: &str,
    type_label: &str,
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
    variants
        .iter()
        .find(|v| v.ty == *constructor)
        .map(|v| v.label.clone())
        .unwrap_or_else(|| {
            panic!(
                "`{name}`: field `{key}` constructor {constructor:?} is not a {type_label} variant"
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

fn emit_module(
    levels: &[String],
    rows: &[BinaryOpRow],
    item_dispatch_variants: &[String],
    item_kw_rows: &[TopLevelItemKwRow],
) -> String {
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
    s.push_str("}\n\n");

    s.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    s.push_str("pub enum ItemDispatchKind {\n");
    for v in item_dispatch_variants {
        s.push_str(&format!("    {v},\n"));
    }
    s.push_str("}\n\n");

    s.push_str(
        "/// Top-level item keyword (`parse_item`) → dispatch class.\n\
         /// Rows are `TopLevelItemKwRow { token_variant }` only; `ItemDispatchKind` is\n\
         /// `strip_prefix(\"Kw\")` — see authority header in `parse_tables.dag`.\n",
    );
    s.push_str("pub fn top_level_item_dispatch(kind: &TokenKind) -> Option<ItemDispatchKind> {\n");
    s.push_str("    match kind {\n");
    for row in item_kw_rows {
        s.push_str(&format!(
            "        TokenKind::{} => Some(ItemDispatchKind::{}),\n",
            row.token_variant, row.dispatch
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

/// SG-1a scaffold extension: read `data dag_operators: List<OperatorSpec>`
/// directly from `syntax.dag` source text because that body still lowers
/// as `ValueBody::Unparsed` (same state `regen_tokenize` encountered).
/// A trimmed view of `OperatorSpec` sufficient for regen
/// cross-validation against `parse_tables.dag`.
pub(crate) struct SharedOperatorSpec {
    pub left_bp: i64,
    /// Variant name from the shared-authority `binop` field, e.g.
    /// `"Or"`, `"Add"`, `"Eq"`. Matches the leaf variant label
    /// inside the corresponding `OperatorKind` (see
    /// `operator_kind_binop_name`), which is the structural join
    /// that closes `symbol → OperatorKind` drift between
    /// `operators.dag` and `dag_operators`.
    pub binop_name: String,
}

/// Returns a symbol → (left_bp, binop_name) map. Fails closed on malformed input.
/// Dissolution trigger: when `dag_operators` lowers structurally, swap
/// this extractor for a typed read — same lane as `regen_tokenize`'s
/// `assert_shared_syntax_raw_source_scaffold_still_required`.
fn extract_shared_operator_bps(
    source: &str,
) -> std::collections::BTreeMap<String, SharedOperatorSpec> {
    let section = extract_balanced_section(source, "data dag_operators", '[', ']');
    let mut out = std::collections::BTreeMap::new();
    // Each entry is `OperatorSpec { symbol: "X", left_bp: N, right_bp: M, binop: Y, ... }`.
    // Parse sequentially: symbol (string) → left_bp (int) → binop (bare identifier).
    let mut rest = section;
    loop {
        let Some(sym_idx) = rest.find("symbol:") else {
            break;
        };
        let after_sym = &rest[sym_idx + "symbol:".len()..];
        let quote_idx = after_sym
            .find('"')
            .expect("missing string literal for `symbol` in `dag_operators`");
        let (symbol, consumed) = parse_string_literal(&after_sym[quote_idx..]);
        let tail = &after_sym[quote_idx + consumed..];

        let bp_idx = tail
            .find("left_bp:")
            .expect("OperatorSpec missing `left_bp` field");
        let after_bp = &tail[bp_idx + "left_bp:".len()..];
        let after_bp_trimmed = after_bp.trim_start();
        let num_end = after_bp_trimmed
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_bp_trimmed.len());
        let bp: i64 = after_bp_trimmed[..num_end]
            .parse()
            .unwrap_or_else(|_| panic!("malformed left_bp for symbol `{symbol}`"));
        let tail = &after_bp_trimmed[num_end..];

        let binop_idx = tail
            .find("binop:")
            .expect("OperatorSpec missing `binop` field");
        let after_binop = &tail[binop_idx + "binop:".len()..];
        let after_binop_trimmed = after_binop.trim_start();
        let ident_end = after_binop_trimmed
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .unwrap_or(after_binop_trimmed.len());
        let binop_name = after_binop_trimmed[..ident_end].to_string();

        assert!(
            out.insert(
                symbol.clone(),
                SharedOperatorSpec {
                    left_bp: bp,
                    binop_name,
                },
            )
            .is_none(),
            "duplicate `dag_operators` row for symbol `{symbol}`"
        );
        rest = &after_binop_trimmed[ident_end..];
    }
    out
}

/// Variant-name projection of an `OperatorKind` — the leaf label inside
/// the nested enum (e.g. `Or`, `Add`, `Eq`). The shared-syntax authority's
/// `OperatorSpec.binop` uses the same vocabulary, so comparing on this
/// label is the structural join that closes `symbol → OperatorKind`
/// drift between `operators.dag` and `dag_operators` — without a
/// parallel name-to-name table (the projection is a pure value read).
fn operator_kind_binop_name(op: OperatorKind) -> &'static str {
    match op {
        OperatorKind::Arithmetic(a) => arithmetic_variant(a),
        OperatorKind::Comparison(c) => comparison_variant(c),
        OperatorKind::Logical(l) => logical_variant(l),
    }
}

fn extract_balanced_section<'a>(source: &'a str, anchor: &str, open: char, close: char) -> &'a str {
    let anchor_idx = source
        .find(anchor)
        .unwrap_or_else(|| panic!("missing `{anchor}` in shared syntax authority"));
    let tail = &source[anchor_idx..];
    let open_rel = tail
        .find(open)
        .unwrap_or_else(|| panic!("missing `{open}` after `{anchor}`"));
    let start = anchor_idx + open_rel;
    let mut depth = 0usize;
    for (offset, ch) in source[start..].char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return &source[start + open.len_utf8()..start + offset];
            }
        }
    }
    panic!("unterminated `{open}` section `{anchor}`");
}

fn parse_string_literal(source: &str) -> (String, usize) {
    assert!(
        source.starts_with('"'),
        "string literal expects to start at a quote"
    );
    let mut out = String::new();
    let mut escaped = false;
    for (idx, ch) in source[1..].char_indices() {
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return (out, idx + 2),
            other => out.push(other),
        }
    }
    panic!("unterminated string literal in shared syntax authority");
}
