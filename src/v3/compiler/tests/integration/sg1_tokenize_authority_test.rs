//! SG-1: `tokenize.dag` is load-bearing authority; `tokenize_generated.rs` must stay in sync.

use std::collections::BTreeSet;

use v3_compiler::dag::{FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::{compile_to_dag, generated_full_bootstrap_dag};

const TOKENIZE_DAG: &str = include_str!("../../tokenize.dag");
const SHARED_SYNTAX_DAG: &str = include_str!("../../../../../dsl/extdeps/languages/dag/syntax.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/tokenize_generated.rs");

#[test]
fn tokenize_dag_compiles_cleanly() {
    compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
}

#[test]
fn std_tokenize_dag_is_present_in_bootstrap_authority() {
    let dag = generated_full_bootstrap_dag();
    let moved = [
        "Token",
        "TokenKind",
        "KeywordTokenKind",
        "PunctTokenKind",
        "LocalPunctSpec",
        "StringEscapeSpec",
    ];

    for name in moved {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("expected `{name}` in generated bootstrap Dag"));
        assert_eq!(
            decl.span.file, "src/v3/std/tokenize.dag",
            "`{name}` should be loaded from `src/v3/std/tokenize.dag` in the committed bootstrap authority"
        );
    }
}

#[test]
fn compiler_tokenize_dag_imports_moved_types_from_std() {
    let dag = compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
    let moved = [
        "Token",
        "TokenKind",
        "KeywordTokenKind",
        "PunctTokenKind",
        "LocalPunctSpec",
        "StringEscapeSpec",
    ];

    for name in moved {
        let decl = dag
            .declaration_by_name(name)
            .unwrap_or_else(|| panic!("expected `{name}` in lowered tokenizer authority"));
        assert_eq!(
            decl.span.file, "src/v3/std/tokenize.dag",
            "`{name}` should now be authored in `src/v3/std/tokenize.dag`, not compiler-local authority"
        );
    }
}

#[test]
fn tokenize_generated_module_matches_checked_in_snapshot() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src").join("tokenize_generated.rs");
    let fresh =
        std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
            .current_dir(&manifest_dir)
            .args(["run", "-q", "-p", "v3-compiler", "--bin", "regen_tokenize"])
            .output()
            .expect("spawn regen_tokenize");
    assert!(
        fresh.status.success(),
        "regen_tokenize failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let regen = std::fs::read_to_string(&out_path).expect("read regenerated tokenize_generated.rs");
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in tokenize_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_tokenize`"
    );
}

#[test]
fn tokenize_keyword_subset_derives_from_shared_syntax_authority() {
    let dag = compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
    let shared_keywords: BTreeSet<_> = parse_map_string_keys(extract_balanced_section(
        SHARED_SYNTAX_DAG,
        "data dag_keyword_set",
        '{',
        '}',
    ))
    .into_iter()
    .collect();

    assert!(
        !TOKENIZE_DAG.contains("\ndata keyword_"),
        "shared keyword spellings should not remain authored in tokenize.dag"
    );

    let keyword_kind_decl = dag
        .declaration_by_name("KeywordTokenKind")
        .expect("KeywordTokenKind declaration");
    let TypeConnective::Disj {
        variants: keyword_variants,
    } = &keyword_kind_decl.connective
    else {
        panic!("KeywordTokenKind should lower to a Disj");
    };

    for variant in keyword_variants {
        let spelling = keyword_spelling_for_token_kind(&variant.label);
        assert!(
            shared_keywords.contains(&spelling),
            "`KeywordTokenKind::{}` expects keyword `{spelling}` from shared syntax authority",
            variant.label
        );
    }
}

#[test]
fn tokenize_local_punct_rows_are_structural_and_disjoint_from_shared_operator_authority() {
    let dag = compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
    let shared_operators: BTreeSet<_> = parse_named_string_fields(
        extract_balanced_section(SHARED_SYNTAX_DAG, "data dag_operators", '[', ']'),
        "symbol",
    )
    .into_iter()
    .collect();

    assert!(
        !TOKENIZE_DAG.contains("\ndata punct_"),
        "shared operator spellings should not remain authored in tokenize.dag"
    );

    let punct_kind_decl = dag
        .declaration_by_name("PunctTokenKind")
        .expect("PunctTokenKind declaration");
    let TypeConnective::Disj {
        variants: punct_variants,
    } = &punct_kind_decl.connective
    else {
        panic!("PunctTokenKind should lower to a Disj");
    };

    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("local_punct_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!("token row `{name}` should carry a structural body");
        };

        assert!(
            fields
                .iter()
                .all(|(label, _)| label != "kind_name" && label != "width"),
            "token row `{name}` should not carry string `kind_name` or redundant `width` fields"
        );

        let pattern = fields
            .iter()
            .find(|(label, _)| label == "pattern")
            .and_then(|(_, value)| match value {
                FieldValue::Literal(LiteralBits::String(pattern)) => Some(pattern.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("token row `{name}` should carry a string `pattern` field"));
        assert!(
            !shared_operators.contains(&pattern),
            "token row `{name}` should not duplicate shared operator `{pattern}`"
        );

        let kind_field = fields
            .iter()
            .find(|(label, _)| label == "kind")
            .unwrap_or_else(|| panic!("token row `{name}` should carry a `kind` field"));
        let FieldValue::Variant {
            constructor,
            payload,
        } = &kind_field.1
        else {
            panic!("token row `{name}` should store `kind` as a structural TokenKind variant");
        };
        assert!(
            payload.is_empty(),
            "token row `{name}` should store only nullary TokenKind variants"
        );
        assert!(
            punct_variants
                .iter()
                .any(|variant| variant.ty == *constructor),
            "token row `{name}` kind constructor should be a variant of `PunctTokenKind`"
        );
    }
}

#[test]
fn shared_syntax_keyword_map_is_structural_while_operator_bridge_remains_bounded() {
    let lowered = match compile_to_dag(SHARED_SYNTAX_DAG, "dsl/extdeps/languages/dag/syntax.dag") {
        Ok(dag) => dag,
        Err(v3_compiler::CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("shared syntax authority should still lower far enough to inspect scaffold state: {other:?}"),
    };

    let keywords = lowered
        .declaration_by_name("dag_keyword_set")
        .unwrap_or_else(|| panic!("shared syntax authority is missing `dag_keyword_set`"));
    assert!(
        matches!(keywords.value_body, Some(ValueBody::Map(_))),
        "`dag_keyword_set` should lower structurally as ValueBody::Map so regen_tokenize derives from the lowered Dag"
    );

    let operators = lowered
        .declaration_by_name("dag_operators")
        .unwrap_or_else(|| panic!("shared syntax authority is missing `dag_operators`"));
    assert!(
        matches!(operators.value_body, Some(ValueBody::Unparsed(_))),
        "`dag_operators` still uses the bounded SG-1a raw-source bridge; once it lowers structurally, derive shared operators from the lowered Dag and update this ratchet"
    );
}

#[test]
fn shared_operator_boundary_is_explicit_and_fail_closed() {
    let dag = compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
    let shared_operators = parse_named_string_fields(
        extract_balanced_section(SHARED_SYNTAX_DAG, "data dag_operators", '[', ']'),
        "symbol",
    );
    let v3_supported_operators: BTreeSet<_> = parse_map_string_keys(extract_balanced_section(
        SHARED_SYNTAX_DAG,
        "data v3_supported_dag_operators",
        '{',
        '}',
    ))
    .into_iter()
    .collect();
    let punct_kind_decl = dag
        .declaration_by_name("PunctTokenKind")
        .expect("PunctTokenKind declaration");
    let TypeConnective::Disj {
        variants: punct_variants,
    } = &punct_kind_decl.connective
    else {
        panic!("PunctTokenKind should lower to a Disj");
    };
    let punct_variant_labels: BTreeSet<_> = punct_variants
        .iter()
        .map(|variant| variant.label.clone())
        .collect();

    let mut shared_tokenized_kinds = BTreeSet::new();
    for pattern in &shared_operators {
        match classify_shared_operator_for_tokenizer(pattern, &v3_supported_operators) {
            SharedOperatorTokenizerBoundary::Tokenized { kind } => {
                assert!(
                    punct_variant_labels.contains(kind),
                    "shared operator `{pattern}` expects `PunctTokenKind::{kind}`"
                );
                shared_tokenized_kinds.insert(kind.to_string());
            }
            SharedOperatorTokenizerBoundary::ParserOnlyDebt { reason } => {
                assert!(
                    !reason.is_empty(),
                    "parser-only shared operator `{pattern}` should document the v3 boundary"
                );
            }
        }
    }

    let mut covered_punct_kinds = shared_tokenized_kinds;
    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("local_punct_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!("token row `{name}` should carry a structural body");
        };
        let kind = fields
            .iter()
            .find(|(label, _)| label == "kind")
            .and_then(|(_, value)| match value {
                FieldValue::Variant {
                    constructor,
                    payload,
                } if payload.is_empty() => Some(*constructor),
                _ => None,
            })
            .unwrap_or_else(|| panic!("token row `{name}` should carry a nullary `kind` variant"));
        let kind_label = punct_variants
            .iter()
            .find(|variant| variant.ty == kind)
            .map(|variant| variant.label.clone())
            .unwrap_or_else(|| {
                panic!("token row `{name}` kind should be a `PunctTokenKind` variant")
            });
        assert!(
            covered_punct_kinds.insert(kind_label.clone()),
            "punctuation kind `{kind_label}` should be covered by exactly one shared/local source"
        );
    }

    assert_eq!(
        covered_punct_kinds, punct_variant_labels,
        "every `PunctTokenKind` variant should come from either the shared-operator subset or \
         a `local_punct_*` row"
    );
}

fn keyword_spelling_for_token_kind(kind: &str) -> String {
    kind.strip_prefix("Kw")
        .unwrap_or_else(|| panic!("keyword token kind `{kind}` should start with `Kw`"))
        .to_ascii_lowercase()
}

enum SharedOperatorTokenizerBoundary {
    Tokenized { kind: &'static str },
    ParserOnlyDebt { reason: &'static str },
}

fn classify_shared_operator_for_tokenizer(
    pattern: &str,
    v3_supported: &BTreeSet<String>,
) -> SharedOperatorTokenizerBoundary {
    if !v3_supported.contains(pattern) {
        return SharedOperatorTokenizerBoundary::ParserOnlyDebt {
            reason: "operator is declared in external dag_operators but excluded from \
                     v3_supported_dag_operators until v3 parses it end-to-end",
        };
    }
    match pattern {
        "==" => SharedOperatorTokenizerBoundary::Tokenized { kind: "EqEq" },
        "!=" => SharedOperatorTokenizerBoundary::Tokenized { kind: "NotEq" },
        "<" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Lt" },
        "<=" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Le" },
        ">" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Gt" },
        ">=" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Ge" },
        "+" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Plus" },
        "-" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Minus" },
        "*" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Star" },
        "/" => SharedOperatorTokenizerBoundary::Tokenized { kind: "Slash" },
        "&&" => SharedOperatorTokenizerBoundary::Tokenized { kind: "AmpAmp" },
        "||" => SharedOperatorTokenizerBoundary::Tokenized { kind: "PipePipe" },
        "|>" => SharedOperatorTokenizerBoundary::Tokenized { kind: "PipeArrow" },
        "." => SharedOperatorTokenizerBoundary::Tokenized { kind: "Dot" },
        other => panic!(
            "`v3_supported_dag_operators` includes `{other}`, but the SG-1 tokenizer bridge has \
             no TokenKind mapping for it"
        ),
    }
}

fn extract_balanced_section<'a>(source: &'a str, anchor: &str, open: char, close: char) -> &'a str {
    let anchor_idx = source
        .find(anchor)
        .unwrap_or_else(|| panic!("missing `{anchor}` in shared syntax fixture"));
    let tail = &source[anchor_idx..];
    let open_rel = tail
        .find(open)
        .unwrap_or_else(|| panic!("missing `{open}` after `{anchor}` in shared syntax fixture"));
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
    panic!("unterminated `{anchor}` section in shared syntax fixture");
}

fn parse_map_string_keys(section: &str) -> Vec<String> {
    parse_all_string_literals(section)
}

fn parse_named_string_fields(section: &str, field_name: &str) -> Vec<String> {
    let needle = format!("{field_name}:");
    let mut out = Vec::new();
    let mut rest = section;
    while let Some(idx) = rest.find(&needle) {
        let after_field = &rest[idx + needle.len()..];
        let quote_idx = after_field.find('"').unwrap_or_else(|| {
            panic!("missing string literal for `{field_name}` in shared syntax fixture")
        });
        let (value, consumed) = parse_string_literal(&after_field[quote_idx..]);
        out.push(value);
        rest = &after_field[quote_idx + consumed..];
    }
    out
}

fn parse_all_string_literals(section: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = section;
    while let Some(idx) = rest.find('"') {
        let (value, consumed) = parse_string_literal(&rest[idx..]);
        out.push(value);
        rest = &rest[idx + consumed..];
    }
    out
}

fn parse_string_literal(source: &str) -> (String, usize) {
    assert!(
        source.starts_with('"'),
        "string literal parser expects to start at a quote"
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
    panic!("unterminated string literal in shared syntax fixture");
}
