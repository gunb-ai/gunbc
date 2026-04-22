//! SG-2c / grammar-tables prototype: `parse_tables.dag` is load-bearing
//! authority for parser-owned tables projected into `parse_tables_generated.rs`:
//! SG-2c-1 binary-operator precedence rows, SG-2c-2 top-level item keyword
//! dispatch, SG-2c-3 type-RHS boundary keywords, SG-2c-4 bracket opener/closer
//! roles. `parse_tables_generated.rs`
//! must stay in sync with the authoring `.dag`; `binary_op_*` rows cover every
//! binary-operator token the parser dispatches on, `top_level_kw_*` rows cover
//! every keyword `parse_item` accepts, and the same rows also project the shared
//! top-level item-boundary helper used by type-RHS lookahead.
//!
//! This lane is explicitly NOT SG-2c proper (parser authority proper).
//! Full parser-algorithm dissolution is blocked on recursive list-body
//! emission over `List<Token>`; see `parse_tables.dag` header.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::parse_tables::soft_keyword_ident_spelling;
use v3_compiler::render_parse_tables_generated_rs;
use v3_compiler::tokenize::TokenKind;

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

#[test]
fn every_top_level_item_kw_row_token_variant_is_a_token_kind_variant() {
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

    let row_type_id = tables_dag
        .declaration_by_name("TopLevelItemKwRow")
        .expect("TopLevelItemKwRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(row_type_id) {
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
fn every_bracket_row_token_variant_is_a_token_kind_variant() {
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

    let row_type_id = tables_dag
        .declaration_by_name("BracketRow")
        .expect("BracketRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(row_type_id) {
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
fn every_bracket_shaped_token_kind_variant_has_a_bracket_row() {
    // Reverse drift check (catches the gap the forward pin leaves open):
    // if a new bracketing `TokenKind` variant ever lands in `tokenize.dag`
    // and no one adds a matching `BracketRow`, `bracket_role` would silently
    // return `None` and the parser's depth tracker would miscount — the
    // forward membership pin wouldn't fire because it only checks the rows
    // against a hardcoded expected set.
    //
    // Heuristic: any `TokenKind` variant whose name is `L[A-Z]…` or
    // `R[A-Z]…` (capital second letter) is bracket-shaped. That rules
    // out comparison operators like `Lt` / `Le` (lowercase second char)
    // while catching hypothetical future `LAngle` / `RAngle` etc.
    // Matches the prefix convention already enforced at regen time by
    // `bracket_role_from_token_variant`.
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

    let is_bracket_shaped = |name: &str| -> bool {
        let mut chars = name.chars();
        let first = match chars.next() {
            Some(c) => c,
            None => return false,
        };
        if first != 'L' && first != 'R' {
            return false;
        }
        matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
    };

    let bracket_shaped_token_kinds: std::collections::BTreeSet<String> = token_variants
        .iter()
        .filter(|v| is_bracket_shaped(&v.label))
        .map(|v| v.label.clone())
        .collect();

    let row_type_id = tables_dag
        .declaration_by_name("BracketRow")
        .expect("BracketRow declaration")
        .id;
    let authored: std::collections::BTreeSet<String> = tables_dag
        .declarations()
        .iter()
        .filter(|d| d.meta_tag == Some(row_type_id))
        .filter_map(|d| match &d.value_body {
            Some(ValueBody::Structural { fields }) => fields
                .iter()
                .find_map(|(k, v)| (k == "token_variant").then_some(v))
                .and_then(|v| match v {
                    FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
                    _ => None,
                }),
            _ => None,
        })
        .collect();

    let missing: Vec<&String> = bracket_shaped_token_kinds.difference(&authored).collect();
    assert!(
        missing.is_empty(),
        "TokenKind variants look bracket-shaped (`L[A-Z]…` / `R[A-Z]…`) but \
         have no matching `BracketRow` in `parse_tables.dag`: {missing:?}. Either \
         author a row, or if the variant is genuinely not a bracket, adjust the \
         bracket-shape heuristic in this test."
    );
}

#[test]
fn bracket_rows_cover_exactly_the_tokens_depth_tracking_helpers_dispatch_on() {
    // `skip_where_clause` and `rhs_is_sum` in `parse_parser_body.txt` scan
    // token spans while tracking paren/brace/bracket depth. Pinned here as
    // the closed set so SG-2c-4 dispatch fails closed if a new bracketing
    // token ever appears in `TokenKind` without a matching `BracketRow`.
    let expected: std::collections::BTreeSet<&'static str> = [
        "LParen", "LBrace", "LBracket", "RParen", "RBrace", "RBracket",
    ]
    .into_iter()
    .collect();

    let tables_dag = compile_to_dag(PARSE_TABLES_DAG, "src/v3/compiler/parse_tables.dag")
        .unwrap_or_else(|e| panic!("parse_tables.dag should compile: {e:?}"));
    let mut got: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let row_type_id = tables_dag
        .declaration_by_name("BracketRow")
        .expect("BracketRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(row_type_id) {
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
        "bracket_* rows in parse_tables.dag do not cover exactly the bracketing tokens the \
         parser's depth-tracking helpers dispatch on"
    );
}

#[test]
fn top_level_item_kw_rows_cover_exactly_the_tokens_parse_item_dispatches_on() {
    // Structural authority for which `Kw*` keywords open items is every
    // `top_level_kw_* : TopLevelItemKwRow` row in `parse_tables.dag` (`got`,
    // collected below). This literal exists as an explicit crash-on-edit pin:
    // extending `parse_item` match arms alone does **not** update `got`; you
    // must author matching rows first (then regen fills `top_level_item_dispatch`).
    let expected: std::collections::BTreeSet<&'static str> =
        ["KwLet", "KwFn", "KwType", "KwModule", "KwImport", "KwData"]
            .into_iter()
            .collect();

    let tables_dag = compile_to_dag(PARSE_TABLES_DAG, "src/v3/compiler/parse_tables.dag")
        .unwrap_or_else(|e| panic!("parse_tables.dag should compile: {e:?}"));
    let mut got: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let row_type_id = tables_dag
        .declaration_by_name("TopLevelItemKwRow")
        .expect("TopLevelItemKwRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(row_type_id) {
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
        "top_level_kw_* rows in parse_tables.dag do not cover exactly the keyword tokens \
         `parse_item` dispatches on"
    );
}

#[test]
fn soft_keyword_ident_rows_cover_exactly_the_keyword_aliases_parser_accepts_as_names() {
    // `parse_field_label` and `parse_variant` currently accept exactly one
    // soft-keyword alias as a bare name: `KwType -> "type"`. Keep the
    // generated parser-name alias table fail-closed.
    let expected: std::collections::BTreeSet<&'static str> = ["KwType"].into_iter().collect();

    let tables_dag = compile_to_dag(PARSE_TABLES_DAG, "src/v3/compiler/parse_tables.dag")
        .unwrap_or_else(|e| panic!("parse_tables.dag should compile: {e:?}"));
    let mut got: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let row_type_id = tables_dag
        .declaration_by_name("SoftKeywordIdentRow")
        .expect("SoftKeywordIdentRow declaration")
        .id;
    for decl in tables_dag.declarations() {
        if decl.meta_tag != Some(row_type_id) {
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
        "soft_keyword_ident_* rows in parse_tables.dag do not cover exactly the keyword aliases \
         the parser accepts as bare names"
    );
}

#[test]
fn soft_keyword_ident_projection_matches_authored_alias_rows() {
    assert_eq!(
        soft_keyword_ident_spelling(&TokenKind::KwType),
        Some("type")
    );
    assert_eq!(soft_keyword_ident_spelling(&TokenKind::KwLet), None);
    assert_eq!(
        soft_keyword_ident_spelling(&TokenKind::Ident(String::from("type"))),
        None
    );
}
