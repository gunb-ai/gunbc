//! Regenerate `tokenize_generated.rs` from `src/v3/compiler/tokenize.dag`.
//!
//! Scanner controls and tokenizer-local punctuation come from the lowered
//! tokenizer Dag, while token types are imported from `src/v3/std/tokenize.dag`.
//! Dedicated keywords are derived from the lowered shared syntax authority at
//! `dsl/extdeps/languages/dag/syntax.dag`. Shared operators still use the
//! bounded raw-source bridge until `dag_operators` lowers structurally.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    AtomPayload, Dag, DeclarationId, FieldValue, LiteralBits, TypeConnective, ValueBody,
};
use v3_compiler::generated_files::GENERATED_FILES;
use v3_compiler::CompileError;

const GENERATED_FILE: &str = "src/v3/compiler/src/tokenize_generated.rs";
const TOKENIZE_AUTHORITY_FILE: &str = "src/v3/compiler/tokenize.dag";
const TOKEN_STD_AUTHORITY_FILE: &str = "src/v3/std/tokenize.dag";
const SHARED_SYNTAX_FILE: &str = "dsl/extdeps/languages/dag/syntax.dag";

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/tokenize.dag` via\n\
     // `regen_tokenize`. Regenerate instead of hand-editing.\n\n";

fn main() {
    // Single-authority gate: the output path this driver writes must
    // be registered in `REGEN_OUTPUTS` (surfaced as
    // `v3_compiler::generated_files::GENERATED_FILES`). SG-0 treats that
    // manifest as the sole producer-owned partition; writing to a path
    // outside the manifest would silently drift the census.
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_tokenize` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`. \
         Add the path to `REGEN_OUTPUTS` so the two authorities stay in \
         lockstep. Both are SG-0's producer-owned manifest; writing \
         to a path outside the manifest would be silent drift."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dag_path = manifest_dir.join("tokenize.dag");
    let source = std::fs::read_to_string(&dag_path).expect("read tokenize.dag");
    let dag = compile_authority_dag(&source, TOKENIZE_AUTHORITY_FILE);
    let shared_syntax_source = read_shared_syntax_source(&manifest_dir);
    let shared_syntax_dag = compile_shared_syntax_dag(&shared_syntax_source);
    let shared_syntax =
        SharedSyntaxAuthority::from_authority(&shared_syntax_dag, &shared_syntax_source);
    let rust = generate(&dag, &shared_syntax);
    let combined = format!("{HEADER}{rust}");

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
        .write_all(combined.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("rustfmt");
    assert!(output.status.success(), "rustfmt failed");
    let formatted = String::from_utf8(output.stdout).expect("utf8");

    let out_path = manifest_dir.join("src").join("tokenize_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write tokenize_generated.rs");
    println!("wrote {}", out_path.display());
}

fn compile_authority_dag(source: &str, file: &str) -> Dag {
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

fn read_shared_syntax_source(manifest_dir: &std::path::Path) -> String {
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("src/v3/compiler should have a repo root ancestor");
    let syntax_path = repo_root.join(SHARED_SYNTAX_FILE);
    std::fs::read_to_string(&syntax_path).unwrap_or_else(|e| {
        panic!(
            "read shared syntax authority `{}` from {}: {e}",
            SHARED_SYNTAX_FILE,
            syntax_path.display()
        )
    })
}

fn compile_shared_syntax_dag(source: &str) -> Dag {
    match compile_to_dag(source, SHARED_SYNTAX_FILE) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("compile {SHARED_SYNTAX_FILE}: {other:?}"),
    }
}

fn generate(dag: &Dag, shared_syntax: &SharedSyntaxAuthority) -> String {
    let keywords = collect_keyword_rows(dag, shared_syntax);
    let ascii_scan_order = collect_ascii_scan_order(dag);
    let puncts = collect_punct_rows(dag, shared_syntax);
    let line_comment_prefix = string_data_named(dag, "line_comment_prefix");
    let string_delim = string_data_named(dag, "string_literal_delimiter");
    assert_eq!(
        string_delim.len(),
        1,
        "string_literal_delimiter must be one byte, got {string_delim:?}"
    );
    let diag_unterm_esc = string_data_named(dag, "diagnostic_unterminated_string_escape");
    let diag_unterm_lit = string_data_named(dag, "diagnostic_unterminated_string_literal");
    let diag_int_pre = string_data_named(dag, "diagnostic_invalid_integer_literal_prefix");
    let diag_int_suf = string_data_named(dag, "diagnostic_invalid_integer_literal_suffix");
    let escapes = collect_string_escape_rows(dag);
    let minus_infix_labels = collect_minus_infix_only_after_token_kinds(dag);

    let mut out = String::new();
    out.push_str("use crate::diagnostics::{Diagnostic, SourceSpan};\n");
    out.push_str(&emit_char_scanner_class_scaffolding(&ascii_scan_order));
    out.push_str(&emit_token_kind_enum(dag));
    out.push_str(
        r#"#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

"#,
    );
    out.push_str(&emit_tokenize_fn(
        &keywords,
        &line_comment_prefix,
        string_delim.as_bytes()[0],
        &diag_unterm_esc,
        &diag_unterm_lit,
        &diag_int_pre,
        &diag_int_suf,
        &escapes,
        &ascii_scan_order,
        &minus_infix_labels,
    ));
    out.push_str(&emit_punctuation_token(&puncts));
    out
}

fn collect_minus_infix_only_after_token_kinds(dag: &Dag) -> Vec<String> {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("minus_infix_only_after_token_kinds"))
        .unwrap_or_else(|| {
            panic!(
                "missing `minus_infix_only_after_token_kinds` data in `{TOKENIZE_AUTHORITY_FILE}`"
            )
        });
    let Some(ValueBody::List(values)) = decl.value_body.as_ref() else {
        panic!(
            "`minus_infix_only_after_token_kinds` in `{TOKENIZE_AUTHORITY_FILE}` must be a list"
        );
    };
    let mut out = Vec::new();
    for value in values {
        match value {
            FieldValue::Literal(LiteralBits::String(s)) => out.push(s.clone()),
            other => panic!(
                "`minus_infix_only_after_token_kinds`: expected string literal elements, got {other:?}"
            ),
        }
    }
    assert!(
        !out.is_empty(),
        "`minus_infix_only_after_token_kinds` must not be empty"
    );
    out
}

fn token_kind_pattern_for_minus_disambiguation(label: &str) -> &'static str {
    match label {
        "Ident" => "TokenKind::Ident(_)",
        "IntLit" => "TokenKind::IntLit(_)",
        "StringLit" => "TokenKind::StringLit(_)",
        "KwTrue" => "TokenKind::KwTrue",
        "KwFalse" => "TokenKind::KwFalse",
        "RParen" => "TokenKind::RParen",
        "RBracket" => "TokenKind::RBracket",
        "RBrace" => "TokenKind::RBrace",
        other => panic!(
            "unsupported `{other}` in `minus_infix_only_after_token_kinds`; \
             add a `TokenKind` arm to `token_kind_pattern_for_minus_disambiguation` \
             in `regen_tokenize.rs`"
        ),
    }
}

fn emit_minus_prefixed_decimal_allowed(labels: &[String]) -> String {
    assert!(
        !labels.is_empty(),
        "`minus_infix_only_after_token_kinds` must name at least one variant"
    );
    let mut inner = String::new();
    for (idx, label) in labels.iter().enumerate() {
        let pat = token_kind_pattern_for_minus_disambiguation(label);
        if idx == 0 {
            inner.push_str(pat);
        } else {
            inner.push_str("\n                | ");
            inner.push_str(pat);
        }
    }
    format!(
        "fn minus_prefixed_decimal_allowed(prev: Option<&TokenKind>) -> bool {{\n\
    !matches!(\n\
        prev,\n\
        Some(\n\
            {inner}\n\
        )\n\
    )\n\
}}\n\n"
    )
}

fn emit_char_scanner_class_scaffolding(scan_order: &[String]) -> String {
    assert!(
        scan_order.len() == 4,
        "`ascii_scan_order` in `tokenize.dag` must list exactly 4 class names"
    );
    assert!(
        scan_order
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == scan_order.len(),
        "`ascii_scan_order` in `tokenize.dag` must not contain duplicates"
    );

    let mut out = String::new();
    out.push_str("\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    out.push_str("pub(crate) enum ScannerCharClass {\n");
    for class in scan_order {
        out.push_str(&format!("    {class},\n"));
    }
    out.push_str("}\n\n");

    out.push_str("#[inline]\n");
    out.push_str("pub(crate) fn byte_matches(byte: u8, class: ScannerCharClass) -> bool {\n");
    out.push_str("    match class {\n");
    for class in scan_order {
        let expr = ascii_scan_class_predicate(class);
        out.push_str(&format!("        ScannerCharClass::{class} => {expr},\n"));
    }
    out.push_str("    }\n}\n\n");

    out
}

fn ascii_scan_class_predicate(class_name: &str) -> &'static str {
    // Interim bridge: `ascii_scan_order` supplies structural scanner order, but
    // class predicate bodies remain here until `std.unicode::char_in_class` is
    // structurally consumable by the tokenizer generator.
    match class_name {
        "Whitespace" => "matches!(byte, b'\\t' | b'\\n' | b'\\x0c' | b'\\r' | b' ')",
        "Digit" => "byte.is_ascii_digit()",
        "IdentStart" => "byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || byte == 0x5f",
        "IdentContinue" => "byte.is_ascii_alphanumeric() || byte == 0x5f",
        _ => panic!("unsupported scanner class `{class_name}` in `ascii_scan_order`"),
    }
}

fn collect_ascii_scan_order(dag: &Dag) -> Vec<String> {
    let scan_decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("ascii_scan_order"))
        .unwrap_or_else(|| panic!("missing `ascii_scan_order` in `{TOKENIZE_AUTHORITY_FILE}`"));

    let char_class_decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("CharClass"))
        .unwrap_or_else(|| panic!("missing `CharClass` in `{TOKENIZE_AUTHORITY_FILE}`"));

    let TypeConnective::Disj {
        variants: char_class_variants,
    } = &char_class_decl.connective
    else {
        panic!("`CharClass` should be a disj declaration");
    };

    let Some(ValueBody::List(values)) = scan_decl.value_body.as_ref() else {
        panic!("`ascii_scan_order` in `{TOKENIZE_AUTHORITY_FILE}` must be a list");
    };

    let mut out = Vec::new();
    for value in values {
        let FieldValue::Variant {
            constructor,
            payload,
        } = value
        else {
            panic!("`ascii_scan_order` elements must be constructor values");
        };
        assert!(
            payload.is_empty(),
            "`ascii_scan_order` class entries must be nullary constructors"
        );
        let label = char_class_variants
            .iter()
            .find(|field| field.ty == *constructor)
            .map(|field| field.label.clone())
            .unwrap_or_else(|| {
                panic!(
                    "`ascii_scan_order` contains constructor {:?} not owned by `CharClass`",
                    constructor
                )
            });
        out.push(label);
    }

    let expected = ["Whitespace", "Digit", "IdentStart", "IdentContinue"];
    for class in &expected {
        assert!(
            out.contains(&class.to_string()),
            "`ascii_scan_order` in `{TOKENIZE_AUTHORITY_FILE}` must include `{class}`"
        );
    }

    out
}

fn string_data_named(dag: &Dag, expected_name: &str) -> String {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some(expected_name))
        .unwrap_or_else(|| panic!("missing `{expected_name}` data in `{TOKENIZE_AUTHORITY_FILE}`"));
    match &decl.value_body {
        Some(ValueBody::Scalar(LiteralBits::String(s))) => s.clone(),
        Some(ValueBody::Structural { fields }) => {
            if let Some((_, FieldValue::Literal(LiteralBits::String(s)))) = fields.first() {
                s.clone()
            } else {
                panic!("`{expected_name}`: expected string scalar or single string field")
            }
        }
        other => panic!("`{expected_name}`: expected string data, got {other:?}"),
    }
}

fn collect_string_escape_rows(dag: &Dag) -> Vec<(u8, i64)> {
    let mut rows = Vec::new();
    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("string_escape_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        let suffix = extract_string_field(fields, "suffix");
        let codepoint = extract_int_field(fields, "output_codepoint");
        assert_eq!(suffix.len(), 1, "{name}: escape suffix must be one byte");
        rows.push((suffix.as_bytes()[0], codepoint));
    }
    rows.sort_by_key(|x| x.0);
    rows
}

fn rust_byte_string_literal(prefix: &str) -> String {
    let mut out = String::from("b\"");
    for b in prefix.bytes() {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            32..=126 => out.push(b as char),
            _ => panic!("non-printable byte in line_comment_prefix: {b:?}"),
        }
    }
    out.push('"');
    out
}

fn rust_string_literal_for_rust_source(s: &str) -> String {
    format!("{s:?}")
}

fn emit_token_kind_enum(dag: &Dag) -> String {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("TokenKind"))
        .unwrap_or_else(|| {
            panic!(
                "missing `TokenKind` declaration while compiling `{TOKENIZE_AUTHORITY_FILE}`; \
                 expected it to be imported from `{TOKEN_STD_AUTHORITY_FILE}`"
            )
        });
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("TokenKind: expected Disj");
    };
    let mut lines = vec![
        "#[derive(Debug, Clone, PartialEq, Eq)]".to_string(),
        "pub enum TokenKind {".to_string(),
    ];
    for v in variants {
        let payload = dag.declaration(v.ty);
        let arm = match &payload.connective {
            TypeConnective::Conj { children } if children.is_empty() => format!("    {},", v.label),
            TypeConnective::Conj { children } if children.len() == 1 => {
                let field = &children[0];
                let rust_ty = rust_type_for_decl_id(dag, field.ty);
                let field_name = &field.label;
                if field_name == "_0" {
                    format!("    {}({rust_ty}),", v.label)
                } else if field_name == "name" && rust_ty == "String" {
                    format!("    {}(String),", v.label)
                } else if field_name == "value" && rust_ty == "Int" {
                    format!("    {}(i64),", v.label)
                } else {
                    format!("    {} {{ {}: {rust_ty} }},", v.label, field_name)
                }
            }
            TypeConnective::Atom(atom) => {
                use v3_compiler::dag::AtomPayload;
                match atom {
                    AtomPayload::Literal(_) => {
                        panic!("unexpected literal atom for variant {}", v.label)
                    }
                    AtomPayload::UnresolvedIdentifier(_) => panic!("unexpected unresolved id"),
                    AtomPayload::ResolvedByStructure(_) | AtomPayload::ResolvedByName(_) => {
                        panic!("unexpected resolved atom for {}", v.label)
                    }
                    AtomPayload::TypeParam(_) => panic!("unexpected type param"),
                }
            }
            other => panic!(
                "TokenKind variant {}: unexpected payload {other:?}",
                v.label
            ),
        };
        lines.push(arm);
    }
    lines.push("}".to_string());
    lines.push(String::new());
    lines.join("\n")
}

fn rust_type_for_decl_id(dag: &Dag, id: DeclarationId) -> String {
    let decl = dag.declaration(id);
    if let Some(name) = &decl.name {
        match name.as_str() {
            "String" => return "String".to_string(),
            "Int" | "Int64" => return "i64".to_string(),
            _ => {}
        }
    }
    match &decl.connective {
        TypeConnective::Atom(ap) => match ap {
            AtomPayload::ResolvedByName(inner) | AtomPayload::ResolvedByStructure(inner) => {
                rust_type_for_decl_id(dag, *inner)
            }
            _ => panic!("rust_type_for_decl_id: unexpected atom {ap:?}"),
        },
        TypeConnective::Instantiation { template, .. } => {
            let template_decl = dag.declaration(*template);
            match template_decl.name.as_deref() {
                Some("String") => "String".to_string(),
                Some("Int") | Some("Int64") => "i64".to_string(),
                other => panic!("unsupported instantiated type {other:?}"),
            }
        }
        other => panic!("rust_type_for_decl_id: unsupported connective for field type: {other:?}"),
    }
}

fn collect_keyword_rows(dag: &Dag, shared_syntax: &SharedSyntaxAuthority) -> Vec<(String, String)> {
    let shared_keywords: BTreeSet<_> = shared_syntax.keywords.iter().cloned().collect();
    let mut rows = collect_variant_labels(dag, "KeywordTokenKind")
        .into_iter()
        .map(|kind| {
            let spelling = keyword_spelling_for_token_kind(&kind);
            assert!(
                shared_keywords.contains(&spelling),
                "`KeywordTokenKind::{kind}` expects keyword `{spelling}`, \
                 but `{SHARED_SYNTAX_FILE}` does not declare it in `dag_keyword_set`"
            );
            assert_token_kind_variant_exists(dag, &kind, "KeywordTokenKind");
            (spelling, kind)
        })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn collect_punct_rows(dag: &Dag, shared_syntax: &SharedSyntaxAuthority) -> Vec<(String, String)> {
    let punct_variants: BTreeSet<_> = collect_variant_labels(dag, "PunctTokenKind")
        .into_iter()
        .collect();
    let mut rows = Vec::new();
    let mut covered_kinds = BTreeSet::new();

    for pattern in &shared_syntax.operators {
        match classify_shared_operator_for_tokenizer(pattern, &shared_syntax.v3_supported_operators)
        {
            SharedOperatorTokenizerBoundary::Tokenized { kind } => {
                assert!(
                    punct_variants.contains(kind),
                    "shared syntax operator `{pattern}` is classified as tokenizer punctuation \
                     `PunctTokenKind::{kind}`, but `{TOKENIZE_AUTHORITY_FILE}` does not declare \
                     that variant"
                );
                assert_token_kind_variant_exists(dag, kind, "PunctTokenKind");
                assert!(
                    covered_kinds.insert(kind.to_string()),
                    "shared syntax operator `{pattern}` maps to duplicate punctuation kind \
                     `PunctTokenKind::{kind}`"
                );
                rows.push((pattern.clone(), kind.to_string()));
            }
            SharedOperatorTokenizerBoundary::ParserOnlyDebt { reason } => {
                assert!(
                    !reason.is_empty(),
                    "parser-only shared operator `{pattern}` should carry a dissolution note"
                );
            }
        }
    }

    let shared_operator_patterns: BTreeSet<_> = shared_syntax.operators.iter().cloned().collect();
    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("local_punct_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        let pattern = extract_string_field(fields, "pattern");
        assert!(
            !shared_operator_patterns.contains(&pattern),
            "tokenizer-local punctuation row `{name}` duplicates shared operator `{pattern}` \
             from `{SHARED_SYNTAX_FILE}`"
        );
        let kind = extract_nullary_variant_field(fields, "kind", "PunctTokenKind", dag);
        assert_token_kind_variant_exists(dag, &kind, "PunctTokenKind");
        assert!(
            covered_kinds.insert(kind.clone()),
            "duplicate punctuation kind `PunctTokenKind::{kind}` across shared/local authority inputs"
        );
        rows.push((pattern, kind));
    }

    let mut seen_patterns = BTreeSet::new();
    for (pattern, _) in &rows {
        assert!(
            seen_patterns.insert(pattern.clone()),
            "duplicate punctuation pattern `{pattern}` across shared/local authority inputs"
        );
    }
    let missing_kinds: Vec<_> = punct_variants.difference(&covered_kinds).cloned().collect();
    assert!(
        missing_kinds.is_empty(),
        "`{TOKENIZE_AUTHORITY_FILE}` declares `PunctTokenKind` variants {:?} that are not covered \
         by either the SG-1a shared-operator bridge or `local_punct_*` rows. Every punctuation \
         kind must come from exactly one authority input so drift fails closed.",
        missing_kinds
    );
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
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
    variants
        .iter()
        .map(|variant| variant.label.clone())
        .collect()
}

fn keyword_spelling_for_token_kind(kind: &str) -> String {
    kind.strip_prefix("Kw")
        .unwrap_or_else(|| panic!("keyword token kind `{kind}` should start with `Kw`"))
        .to_ascii_lowercase()
}

// SG-1a operator bridge: shared operators still lower through raw-source reads.
// `v3_supported_dag_operators` is the single support/exclusion projection; this
// table only maps supported symbols to their current tokenizer variant.
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
            "`v3_supported_dag_operators` includes `{other}`, but the SG-1a tokenizer bridge has \
             no TokenKind mapping for it. Add tokenizer/parser/operator support or remove it from \
             the v3 projection."
        ),
    }
}

struct SharedSyntaxAuthority {
    keywords: Vec<String>,
    operators: Vec<String>,
    v3_supported_operators: BTreeSet<String>,
}

impl SharedSyntaxAuthority {
    fn from_authority(dag: &Dag, source: &str) -> Self {
        let keywords = match data_body_named(dag, "dag_keyword_set") {
            ValueBody::Map(entries) => entries
                .entries()
                .iter()
                .map(|(key, _)| key.clone())
                .collect(),
            other => panic!("dag_keyword_set: expected ValueBody::Map, got {other:?}"),
        };
        let operators = parse_named_string_fields(
            extract_balanced_section(source, "data dag_operators", '[', ']'),
            "symbol",
        );
        let v3_supported_operators: BTreeSet<_> = parse_all_string_literals(
            extract_balanced_section(source, "data v3_supported_dag_operators", '{', '}'),
        )
        .into_iter()
        .collect();
        Self {
            keywords,
            operators,
            v3_supported_operators,
        }
    }
}

fn data_body_named<'a>(dag: &'a Dag, expected_name: &str) -> &'a ValueBody {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some(expected_name))
        .unwrap_or_else(|| panic!("missing `{expected_name}` data in `{SHARED_SYNTAX_FILE}`"));
    decl.value_body
        .as_ref()
        .unwrap_or_else(|| panic!("`{expected_name}` has no lowered value body"))
}

fn extract_balanced_section<'a>(source: &'a str, anchor: &str, open: char, close: char) -> &'a str {
    let anchor_idx = source
        .find(anchor)
        .unwrap_or_else(|| panic!("missing `{anchor}` in `{SHARED_SYNTAX_FILE}`"));
    let tail = &source[anchor_idx..];
    let open_rel = tail
        .find(open)
        .unwrap_or_else(|| panic!("missing `{open}` after `{anchor}` in `{SHARED_SYNTAX_FILE}`"));
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
    panic!(
        "unterminated `{}` section `{anchor}` in `{SHARED_SYNTAX_FILE}`",
        open
    );
}

fn parse_named_string_fields(section: &str, field_name: &str) -> Vec<String> {
    let needle = format!("{field_name}:");
    let mut out = Vec::new();
    let mut rest = section;
    while let Some(idx) = rest.find(&needle) {
        let after_field = &rest[idx + needle.len()..];
        let quote_idx = after_field.find('"').unwrap_or_else(|| {
            panic!("missing string literal for `{field_name}` in `{SHARED_SYNTAX_FILE}`")
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
    panic!("unterminated string literal while parsing `{SHARED_SYNTAX_FILE}`");
}

fn extract_nullary_variant_field(
    fields: &[(String, FieldValue)],
    key: &str,
    expected_type_name: &str,
    dag: &Dag,
) -> String {
    let fv = fields
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v))
        .unwrap_or_else(|| panic!("missing field {key}"));
    let FieldValue::Variant {
        constructor,
        payload,
    } = fv
    else {
        panic!("field {key}: expected nullary variant value");
    };
    assert!(
        payload.is_empty(),
        "field {key}: expected nullary variant, got payload of len {}",
        payload.len()
    );

    let expected_type_decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some(expected_type_name))
        .unwrap_or_else(|| panic!("missing `{expected_type_name}` declaration"));
    let TypeConnective::Disj { variants } = &expected_type_decl.connective else {
        panic!("`{expected_type_name}`: expected Disj");
    };
    variants
        .iter()
        .find(|field| field.ty == *constructor)
        .map(|field| field.label.clone())
        .unwrap_or_else(|| {
            panic!(
                "field {key}: constructor id {:?} is not a variant of `{expected_type_name}`",
                constructor
            )
        })
}

fn assert_token_kind_variant_exists(dag: &Dag, label: &str, source_type_name: &str) {
    let token_kind_decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("TokenKind"))
        .unwrap_or_else(|| panic!("missing `TokenKind` declaration"));
    let TypeConnective::Disj { variants } = &token_kind_decl.connective else {
        panic!("`TokenKind`: expected Disj");
    };
    assert!(
        variants.iter().any(|field| field.label == label),
        "`{source_type_name}` variant `{label}` has no matching `TokenKind` variant"
    );
}

fn extract_string_field(fields: &[(String, FieldValue)], key: &str) -> String {
    let fv = fields
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v))
        .unwrap_or_else(|| panic!("missing field {key}"));
    match fv {
        FieldValue::Literal(v3_compiler::dag::LiteralBits::String(s)) => s.clone(),
        _ => panic!("field {key}: expected string literal"),
    }
}

fn extract_int_field(fields: &[(String, FieldValue)], key: &str) -> i64 {
    let fv = fields
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v))
        .unwrap_or_else(|| panic!("missing field {key}"));
    match fv {
        FieldValue::Literal(v3_compiler::dag::LiteralBits::Int(n)) => *n,
        _ => panic!("field {key}: expected int literal"),
    }
}

#[allow(clippy::too_many_arguments)] // Codegen: one projection bundle per `tokenize.dag` surface.
fn emit_tokenize_fn(
    keywords: &[(String, String)],
    line_comment_prefix: &str,
    string_delim_byte: u8,
    diag_unterm_esc: &str,
    diag_unterm_lit: &str,
    diag_int_pre: &str,
    diag_int_suf: &str,
    escapes: &[(u8, i64)],
    ascii_scan_order: &[String],
    minus_infix_only_after: &[String],
) -> String {
    let ensure_classes = |required: &[&str]| {
        for name in required {
            assert!(
                ascii_scan_order.iter().any(|class| class == name),
                "`ascii_scan_order` in tokenize.dag missing `{name}`"
            );
        }
    };
    ensure_classes(&["Whitespace", "Digit", "IdentStart", "IdentContinue"]);

    let mut arms = String::new();
    for (spelling, kind) in keywords {
        arms.push_str(&format!(
            "                \"{}\" => TokenKind::{},\n",
            spelling, kind
        ));
    }
    let line_comment_lit = rust_byte_string_literal(line_comment_prefix);
    let diag_esc = rust_string_literal_for_rust_source(diag_unterm_esc);
    let int_pre = rust_string_literal_for_rust_source(diag_int_pre);
    let int_suf = rust_string_literal_for_rust_source(diag_int_suf);
    let delim_lit = rust_byte_literal(string_delim_byte);

    let mut escape_arms = String::new();
    for (suf, cp) in escapes {
        escape_arms.push_str(&format!(
            "                            {} => content.push(core::char::from_u32({}_u32).unwrap()),\n",
            rust_byte_literal(*suf),
            cp
        ));
    }

    let mut s = String::new();
    s.push_str(&emit_minus_prefixed_decimal_allowed(minus_infix_only_after));
    s.push_str("pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic> {\n");
    s.push_str("    let bytes = source.as_bytes();\n");
    s.push_str("    let mut pos: usize = 0;\n");
    s.push_str("    let mut tokens = Vec::new();\n\n");
    s.push_str(&format!(
        "    const LINE_COMMENT_PREFIX: &[u8] = {};\n",
        line_comment_lit
    ));
    s.push_str("    while pos < bytes.len() {\n");
    s.push_str("        let byte = bytes[pos];\n\n");
    s.push_str("        let start = pos;\n\n");
    s.push_str(
        "        if byte == b'-'\n            && bytes.get(pos + 1).is_some_and(|b| byte_matches(*b, ScannerCharClass::Digit))\n            && minus_prefixed_decimal_allowed(tokens.last().map(|t: &Token| &t.kind))\n        {\n",
    );
    s.push_str("            let mut end = pos + 1;\n");
    s.push_str(
        "            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::Digit) {\n",
    );
    s.push_str("                end += 1;\n");
    s.push_str("            }\n");
    s.push_str("            let literal = &source[pos + 1..end];\n");
    s.push_str(
        "            let magnitude: u128 = literal.parse().map_err(|_| Diagnostic::TokenizerError {\n",
    );
    s.push_str(&format!(
        "                message: format!(\"{{}}{{}}{{}}\", {}, literal, {}),\n",
        int_pre, int_suf
    ));
    s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
    s.push_str("                fixes: Vec::new(),\n");
    s.push_str("            })?;\n");
    s.push_str("            const SIGNED_DECIMAL_I64_ABS_MIN: u128 = 9223372036854775808;\n");
    s.push_str("            let value: i64 = match magnitude {\n");
    s.push_str("                0 => 0,\n");
    s.push_str("                m if m <= i64::MAX as u128 => -(m as i64),\n");
    s.push_str("                m if m == SIGNED_DECIMAL_I64_ABS_MIN => i64::MIN,\n");
    s.push_str("                _ => {\n");
    s.push_str("                    return Err(Diagnostic::TokenizerError {\n");
    s.push_str(
        "                        message: format!(\"integer literal out of range for i64: `-{}`\", literal),\n",
    );
    s.push_str("                        span: SourceSpan::new(file, start as u32, end as u32),\n");
    s.push_str("                        fixes: Vec::new(),\n");
    s.push_str("                    });\n");
    s.push_str("                }\n");
    s.push_str("            };\n");
    s.push_str("            tokens.push(Token {\n");
    s.push_str("                kind: TokenKind::IntLit(value),\n");
    s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
    s.push_str("            });\n");
    s.push_str("            pos = end;\n");
    s.push_str("            continue;\n");
    s.push_str("        }\n\n");
    for class in ascii_scan_order {
        if class == "Whitespace" {
            s.push_str("        if byte_matches(byte, ScannerCharClass::Whitespace) {\n");
            s.push_str("            pos += 1;\n");
            s.push_str("            continue;\n");
            s.push_str("        }\n\n");
        } else if class == "Digit" {
            s.push_str("        if byte_matches(byte, ScannerCharClass::Digit) {\n");
            s.push_str("            let mut end = pos;\n");
            s.push_str(
                "            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::Digit) {\n",
            );
            s.push_str("                end += 1;\n");
            s.push_str("            }\n");
            s.push_str("            let literal = &source[start..end];\n");
            s.push_str(
                "            let value: i64 = literal.parse().map_err(|_| Diagnostic::TokenizerError {\n",
            );
            s.push_str(&format!(
                "                message: format!(\"{{}}{{}}{{}}\", {}, literal, {}),\n",
                int_pre, int_suf
            ));
            s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
            s.push_str("                fixes: Vec::new(),\n");
            s.push_str("            })?;\n");
            s.push_str("            tokens.push(Token {\n");
            s.push_str("                kind: TokenKind::IntLit(value),\n");
            s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
            s.push_str("            });\n");
            s.push_str("            pos = end;\n");
            s.push_str("            continue;\n");
            s.push_str("        }\n\n");
        } else if class == "IdentStart" {
            s.push_str("        if byte_matches(byte, ScannerCharClass::IdentStart) {\n");
            s.push_str("            let mut end = pos;\n");
            s.push_str(
                "            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::IdentContinue) {\n",
            );
            s.push_str("                end += 1;\n");
            s.push_str("            }\n");
            s.push_str("            let text = &source[start..end];\n");
            s.push_str("            let kind = match text {\n");
            s.push_str(&arms);
            s.push_str("                _ => TokenKind::Ident(text.to_string()),\n");
            s.push_str("            };\n");
            s.push_str("            tokens.push(Token {\n");
            s.push_str("                kind,\n");
            s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
            s.push_str("            });\n");
            s.push_str("            pos = end;\n");
            s.push_str("            continue;\n");
            s.push_str("        }\n\n");
        } else if class == "IdentContinue" {
            s.push_str("        if byte_matches(byte, ScannerCharClass::IdentContinue) {\n");
            s.push_str("            let mut end = pos;\n");
            s.push_str(
                "            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::IdentContinue) {\n",
            );
            s.push_str("                end += 1;\n");
            s.push_str("            }\n");
            s.push_str("            let text = &source[start..end];\n");
            s.push_str("            let kind = match text {\n");
            s.push_str(&arms);
            s.push_str("                _ => TokenKind::Ident(text.to_string()),\n");
            s.push_str("            };\n");
            s.push_str("            tokens.push(Token {\n");
            s.push_str("                kind,\n");
            s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
            s.push_str("            });\n");
            s.push_str("            pos = end;\n");
            s.push_str("            continue;\n");
            s.push_str("        }\n\n");
        } else {
            panic!(
                "unsupported scanner class `{}` in `ascii_scan_order`",
                class
            );
        }
    }
    s.push_str("        // Line comment prefix from `tokenize.dag` (`line_comment_prefix`).\n");
    s.push_str("        if bytes.len() >= pos + LINE_COMMENT_PREFIX.len()\n");
    s.push_str(
        "            && bytes[pos..pos + LINE_COMMENT_PREFIX.len()].eq(LINE_COMMENT_PREFIX)\n",
    );
    s.push_str("        {\n");
    s.push_str("            pos += LINE_COMMENT_PREFIX.len();\n");
    s.push_str("            while pos < bytes.len() && bytes[pos] != b'\\n' {\n");
    s.push_str("                pos += 1;\n");
    s.push_str("            }\n");
    s.push_str("            continue;\n");
    s.push_str("        }\n\n");
    s.push_str("        if let Some((kind, width)) = punctuation_token(bytes, pos) {\n");
    s.push_str("            tokens.push(Token {\n");
    s.push_str("                kind,\n");
    s.push_str(
        "                span: SourceSpan::new(file, start as u32, (start + width) as u32),\n",
    );
    s.push_str("            });\n");
    s.push_str("            pos += width;\n");
    s.push_str("            continue;\n");
    s.push_str("        }\n\n");
    s.push_str(
        "        // String literal (`string_literal_delimiter` + `StringEscapeSpec` rows).\n",
    );
    s.push_str(&format!("        if byte == {} {{\n", delim_lit));
    s.push_str("            let mut end = pos + 1;\n");
    s.push_str("            let mut content = String::new();\n");
    s.push_str("            let mut terminated = false;\n");
    s.push_str("            while end < bytes.len() {\n");
    s.push_str("                match bytes[end] {\n");
    s.push_str(&format!("                    {} => {{\n", delim_lit));
    s.push_str("                        terminated = true;\n");
    s.push_str("                        end += 1;\n");
    s.push_str("                        break;\n");
    s.push_str("                    }\n");
    s.push_str("                    b'\\\\' => {\n");
    s.push_str("                        let Some(escaped) = bytes.get(end + 1).copied() else {\n");
    s.push_str("                            return Err(Diagnostic::TokenizerError {\n");
    s.push_str(&format!(
        "                                message: {}.to_string(),\n",
        diag_esc
    ));
    s.push_str("                                span: SourceSpan::new(file, start as u32, (end + 1) as u32),\n");
    s.push_str("                                fixes: Vec::new(),\n");
    s.push_str("                            });\n");
    s.push_str("                        };\n");
    s.push_str("                        match escaped {\n");
    s.push_str(&escape_arms);
    s.push_str("                            other => {\n");
    s.push_str("                                content.push('\\\\');\n");
    s.push_str("                                content.push(other as char);\n");
    s.push_str("                            }\n");
    s.push_str("                        }\n");
    s.push_str("                        end += 2;\n");
    s.push_str("                    }\n");
    s.push_str("                    other => {\n");
    s.push_str("                        content.push(other as char);\n");
    s.push_str("                        end += 1;\n");
    s.push_str("                    }\n");
    s.push_str("                }\n");
    s.push_str("            }\n");
    s.push_str("            if !terminated {\n");
    s.push_str("                return Err(Diagnostic::TokenizerError {\n");
    s.push_str(&format!(
        "                    message: {}.to_string(),\n",
        rust_string_literal_for_rust_source(diag_unterm_lit)
    ));
    s.push_str("                    span: SourceSpan::new(file, start as u32, end as u32),\n");
    s.push_str("                    fixes: Vec::new(),\n");
    s.push_str("                });\n");
    s.push_str("            }\n");
    s.push_str("            tokens.push(Token {\n");
    s.push_str("                kind: TokenKind::StringLit(content),\n");
    s.push_str("                span: SourceSpan::new(file, start as u32, end as u32),\n");
    s.push_str("            });\n");
    s.push_str("            pos = end;\n");
    s.push_str("            continue;\n");
    s.push_str("        }\n\n");
    s.push_str("        return Err(Diagnostic::TokenizerError {\n");
    s.push_str("            message: format!(\"unexpected byte `{}`\", byte as char),\n");
    s.push_str("            span: SourceSpan::new(file, start as u32, (start + 1) as u32),\n");
    s.push_str("            fixes: Vec::new(),\n");
    s.push_str("        });\n");
    s.push_str("    }\n\n");
    s.push_str("    tokens.push(Token {\n");
    s.push_str("        kind: TokenKind::Eof,\n");
    s.push_str("        span: SourceSpan::new(file, bytes.len() as u32, bytes.len() as u32),\n");
    s.push_str("    });\n");
    s.push_str("    Ok(tokens)\n");
    s.push_str("}\n\n");
    s
}

fn emit_punctuation_token(puncts: &[(String, String)]) -> String {
    let mut two = Vec::new();
    let mut one = Vec::new();
    for (pat, kind) in puncts {
        let bs: Vec<u8> = pat.bytes().collect();
        match bs.as_slice() {
            [a, b] => two.push((*a, *b, kind.clone())),
            [a] => one.push((*a, kind.clone())),
            _ => panic!("unsupported punct width {} for {pat}", bs.len()),
        };
    }
    two.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    one.sort_by_key(|x| x.0);

    let mut arms = String::new();
    for (a, b, kind) in &two {
        arms.push_str(&format!(
            "        ({}, Some({})) => Some((TokenKind::{}, 2)),\n",
            rust_byte_literal(*a),
            rust_byte_literal(*b),
            kind
        ));
    }
    for (a, kind) in &one {
        arms.push_str(&format!(
            "        ({}, _) => Some((TokenKind::{}, 1)),\n",
            rust_byte_literal(*a),
            kind
        ));
    }
    arms.push_str("        _ => None,\n");

    format!(
        "fn punctuation_token(bytes: &[u8], pos: usize) -> Option<(TokenKind, usize)> {{\n\
    let first = bytes[pos];\n\
    let second = bytes.get(pos + 1).copied();\n\
    match (first, second) {{\n\
{arms}    }}\n\
}}\n"
    )
}

fn rust_byte_literal(b: u8) -> String {
    match b {
        b'\'' => "b'\\''".to_string(),
        b'\\' => "b'\\\\'".to_string(),
        _ => format!("b'{}'", b as char),
    }
}
