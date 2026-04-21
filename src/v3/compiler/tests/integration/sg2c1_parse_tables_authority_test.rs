//! SG-2c-1 grammar-tables prototype: `parse_tables.dag` is load-bearing
//! authority for the binary-operator-at-precedence-level map the parser
//! consumes. `parse_tables_generated.rs` must stay in sync with the
//! authoring `.dag`, and the `binary_op_*` data rows must cover every
//! binary-operator `TokenKind` that the parser's per-precedence-level
//! functions actually dispatch on.
//!
//! This lane is explicitly NOT SG-2c proper (parser authority proper).
//! Full parser-algorithm dissolution is blocked on recursive list-body
//! emission over `List<Token>`; see `parse_tables.dag` header.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::render_parse_tables_generated_rs;

const PARSE_TABLES_DAG: &str = include_str!("../../parse_tables.dag");
const TOKENIZE_DAG: &str = include_str!("../../tokenize.dag");
const SHARED_SYNTAX_DAG: &str = include_str!("../../../../../dsl/extdeps/languages/dag/syntax.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/parse_tables_generated.rs");

#[test]
fn parse_tables_dag_compiles_cleanly() {
    compile_to_dag(PARSE_TABLES_DAG, "src/v3/compiler/parse_tables.dag")
        .unwrap_or_else(|e| panic!("parse_tables.dag should compile: {e:?}"));
}

#[test]
fn parse_tables_generated_module_matches_checked_in_snapshot() {
    let regen = render_parse_tables_generated_rs(
        PARSE_TABLES_DAG,
        "src/v3/compiler/parse_tables.dag",
        TOKENIZE_DAG,
        "src/v3/compiler/tokenize.dag",
        SHARED_SYNTAX_DAG,
    )
    .unwrap_or_else(|e| panic!("render parse_tables_generated.rs in-process: {e}"));
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in parse_tables_generated.rs is stale; run \
         `cargo run -p v3-compiler --bin regen_parse_tables`"
    );
}

#[test]
fn every_binary_op_row_token_variant_is_a_token_kind_variant() {
    let tables_dag = compile_to_dag(PARSE_TABLES_DAG, "src/v3/compiler/parse_tables.dag")
        .unwrap_or_else(|e| panic!("parse_tables.dag should compile: {e:?}"));
    let tokenize_dag = compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));

    let token_kind_decl = tokenize_dag
        .declaration_by_name("TokenKind")
        .expect("TokenKind declaration in tokenize.dag");
    let TypeConnective::Disj {
        variants: token_variants,
    } = &token_kind_decl.connective
    else {
        panic!("TokenKind should lower to a Disj");
    };
    let token_variant_names: std::collections::BTreeSet<String> =
        token_variants.iter().map(|v| v.label.clone()).collect();

    let binary_op_row_type_id = tables_dag
        .declaration_by_name("BinaryOpRow")
        .expect("BinaryOpRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(binary_op_row_type_id) {
            continue;
        }
        let name = decl.name.as_deref().unwrap_or("<anonymous>");
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        let token_variant = fields
            .iter()
            .find_map(|(k, v)| (k == "token_variant").then_some(v))
            .and_then(|v| match v {
                FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("`{name}`: missing string `token_variant` field"));
        assert!(
            token_variant_names.contains(&token_variant),
            "`parse_tables.dag::{name}`: token_variant `{token_variant}` is not a \
             `TokenKind` variant in `tokenize.dag`"
        );
    }
}

#[test]
fn binary_op_rows_cover_every_operator_token_the_parser_dispatches_on() {
    // The parser's per-precedence-level functions (`parse_logical_or`,
    // `parse_logical_and`, `parse_comparison`, `parse_additive`,
    // `parse_term`) dispatch on exactly these twelve `TokenKind`
    // variants. If a new binary operator token ever joins that set,
    // `parse_tables.dag` must grow a row for it or the parser will
    // silently skip it at runtime. Pinned here so the authority
    // boundary fails closed.
    let expected: std::collections::BTreeSet<&'static str> = [
        "PipePipe", "AmpAmp", "EqEq", "NotEq", "Lt", "Le", "Gt", "Ge", "Plus", "Minus", "Star",
        "Slash",
    ]
    .into_iter()
    .collect();

    let tables_dag = compile_to_dag(PARSE_TABLES_DAG, "src/v3/compiler/parse_tables.dag")
        .unwrap_or_else(|e| panic!("parse_tables.dag should compile: {e:?}"));
    let mut got: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let binary_op_row_type_id = tables_dag
        .declaration_by_name("BinaryOpRow")
        .expect("BinaryOpRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(binary_op_row_type_id) {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        if let Some(FieldValue::Literal(LiteralBits::String(s))) = fields
            .iter()
            .find_map(|(k, v)| (k == "token_variant").then_some(v))
        {
            got.insert(s.clone());
        }
    }
    let got_ref: std::collections::BTreeSet<&str> = got.iter().map(String::as_str).collect();
    assert_eq!(
        expected, got_ref,
        "binary_op_* rows in parse_tables.dag do not cover exactly the operator tokens the \
         parser dispatches on"
    );
}
