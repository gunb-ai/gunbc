//! Regenerate `tokenize_generated.rs` from `src/v3/compiler/tokenize.dag`.
//
// Keyword and punctuation tables are read from the lowered Dag; the scanning
// algorithm is emitted as deterministic Rust (this binary is codegen only).

use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, FieldValue, TypeConnective, ValueBody};
use v3_compiler::CompileError;

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/tokenize.dag` via\n\
     // `regen_tokenize`. Regenerate instead of hand-editing.\n\n";

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dag_path = manifest_dir.join("tokenize.dag");
    let source = std::fs::read_to_string(&dag_path).expect("read tokenize.dag");
    let dag =
        compile_to_dag(&source, "src/v3/compiler/tokenize.dag").expect("compile tokenize.dag");
    let rust = generate(&dag);
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

fn generate(dag: &Dag) -> String {
    let keywords = collect_keyword_rows(dag);
    let puncts = collect_punct_rows(dag);
    validate_kind_names(dag, &keywords, &puncts);

    let mut out = String::new();
    out.push_str("use crate::diagnostics::{Diagnostic, SourceSpan};\n\n");
    out.push_str(&emit_token_kind_enum(dag));
    out.push_str(
        r#"#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

"#,
    );
    out.push_str(&emit_tokenize_fn(&keywords));
    out.push_str(&emit_punctuation_token(&puncts));
    out
}

fn token_kind_variant_labels(dag: &Dag) -> HashSet<String> {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("TokenKind"))
        .expect("TokenKind declaration");
    let TypeConnective::Disj { variants } = &decl.connective else {
        panic!("TokenKind: expected Disj");
    };
    variants.iter().map(|v| v.label.clone()).collect()
}

fn validate_kind_names(
    dag: &Dag,
    keywords: &[(String, String)],
    puncts: &[(String, i64, String)],
) {
    let allowed = token_kind_variant_labels(dag);
    for (_, kind) in keywords {
        assert!(
            allowed.contains(kind),
            "keyword kind_name `{kind}` is not a TokenKind variant"
        );
    }
    for (_, _, kind) in puncts {
        assert!(
            allowed.contains(kind),
            "punct kind_name `{kind}` is not a TokenKind variant"
        );
    }
}

fn emit_token_kind_enum(dag: &Dag) -> String {
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("TokenKind"))
        .expect("TokenKind declaration");
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
                let ty = dag.declaration(field.ty);
                let rust_ty = atom_decl_to_rust(ty);
                let field_name = &field.label;
                if field_name == "name" && rust_ty == "String" {
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

fn atom_decl_to_rust(decl: &v3_compiler::dag::Declaration) -> &'static str {
    match &decl.connective {
        TypeConnective::Atom(ap) => {
            use v3_compiler::dag::AtomPayload;
            match ap {
                AtomPayload::ResolvedByName(id) | AtomPayload::ResolvedByStructure(id) => {
                    let target = decl
                        .name
                        .as_deref()
                        .unwrap_or_else(|| panic!("anon decl {:?}", id));
                    match target {
                        "String" => "String",
                        "Int" | "Int64" => "i64",
                        other => panic!("unsupported atom target {other}"),
                    }
                }
                _ => panic!("atom_decl_to_rust: non-resolved atom"),
            }
        }
        _ => panic!("atom_decl_to_rust: expected atom"),
    }
}

fn collect_keyword_rows(dag: &Dag) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("keyword_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        let spelling = extract_string_field(fields, "spelling");
        let kind = extract_string_field(fields, "kind_name");
        rows.push((spelling, kind));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn collect_punct_rows(dag: &Dag) -> Vec<(String, i64, String)> {
    let mut rows = Vec::new();
    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("punct_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            continue;
        };
        let pattern = extract_string_field(fields, "pattern");
        let width = extract_int_field(fields, "width");
        let kind = extract_string_field(fields, "kind_name");
        rows.push((pattern, width, kind));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
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

fn emit_tokenize_fn(keywords: &[(String, String)]) -> String {
    let mut arms = String::new();
    for (spelling, kind) in keywords {
        arms.push_str(&format!(
            "                \"{}\" => TokenKind::{},\n",
            spelling, kind
        ));
    }
    format!(
        r#"pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic> {{
    let bytes = source.as_bytes();
    let mut pos: usize = 0;
    let mut tokens = Vec::new();

    while pos < bytes.len() {{
        let byte = bytes[pos];

        if byte.is_ascii_whitespace() {{
            pos += 1;
            continue;
        }}

        // Line comments: `// ...` to end of line.
        if byte == b'/' && bytes.get(pos + 1) == Some(&b'/') {{
            pos += 2;
            while pos < bytes.len() && bytes[pos] != b'\\n' {{
                pos += 1;
            }}
            continue;
        }}

        let start = pos;

        if let Some((kind, width)) = punctuation_token(bytes, pos) {{
            tokens.push(Token {{
                kind,
                span: SourceSpan::new(file, start as u32, (start + width) as u32),
            }});
            pos += width;
            continue;
        }}

        if byte.is_ascii_digit() {{
            let mut end = pos;
            while end < bytes.len() && bytes[end].is_ascii_digit() {{
                end += 1;
            }}
            let literal = &source[start..end];
            let value: i64 = literal.parse().map_err(|_| Diagnostic::TokenizerError {{
                message: format!("invalid integer literal `{{literal}}`"),
                span: SourceSpan::new(file, start as u32, end as u32),
                fixes: Vec::new(),
            }})?;
            tokens.push(Token {{
                kind: TokenKind::IntLit(value),
                span: SourceSpan::new(file, start as u32, end as u32),
            }});
            pos = end;
            continue;
        }}

        if byte.is_ascii_alphabetic() || byte == b'_' {{
            let mut end = pos;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {{
                end += 1;
            }}
            let text = &source[start..end];
            let kind = match text {{
{arms}                _ => TokenKind::Ident(text.to_string()),
            }};
            tokens.push(Token {{
                kind,
                span: SourceSpan::new(file, start as u32, end as u32),
            }});
            pos = end;
            continue;
        }}

        // String literal: "..."
        //
        // Minimal escape surface for bootstrap-staged structural data:
        // `\"`, `\\\\`, `\\n`, `\\r`, `\\t`. Unknown `\\x` pairs preserve the
        // old M0 behavior and stay literal as `\\` + `x`. Raw newlines
        // are preserved until the closing `"`.
        if byte == b'"' {{
            let mut end = pos + 1;
            let mut content = String::new();
            let mut terminated = false;
            while end < bytes.len() {{
                match bytes[end] {{
                    b'"' => {{
                        terminated = true;
                        end += 1;
                        break;
                    }}
                    b'\\\\' => {{
                        let Some(escaped) = bytes.get(end + 1).copied() else {{
                            return Err(Diagnostic::TokenizerError {{
                                message: "unterminated string escape".to_string(),
                                span: SourceSpan::new(file, start as u32, (end + 1) as u32),
                                fixes: Vec::new(),
                            }});
                        }};
                        match escaped {{
                            b'"' => content.push('"'),
                            b'\\\\' => content.push('\\\\'),
                            b'n' => content.push('\\n'),
                            b'r' => content.push('\\r'),
                            b't' => content.push('\\t'),
                            other => {{
                                content.push('\\\\');
                                content.push(other as char);
                            }}
                        }}
                        end += 2;
                    }}
                    other => {{
                        content.push(other as char);
                        end += 1;
                    }}
                }}
            }}
            if !terminated {{
                return Err(Diagnostic::TokenizerError {{
                    message: "unterminated string literal".to_string(),
                    span: SourceSpan::new(file, start as u32, end as u32),
                    fixes: Vec::new(),
                }});
            }}
            tokens.push(Token {{
                kind: TokenKind::StringLit(content),
                span: SourceSpan::new(file, start as u32, end as u32),
            }});
            pos = end;
            continue;
        }}

        return Err(Diagnostic::TokenizerError {{
            message: format!("unexpected byte `{{}}`", byte as char),
            span: SourceSpan::new(file, start as u32, (start + 1) as u32),
            fixes: Vec::new(),
        }});
    }}

    tokens.push(Token {{
        kind: TokenKind::Eof,
        span: SourceSpan::new(file, bytes.len() as u32, bytes.len() as u32),
    }});
    Ok(tokens)
}}

"#
    )
}

fn emit_punctuation_token(puncts: &[(String, i64, String)]) -> String {
    let mut two = Vec::new();
    let mut one = Vec::new();
    for (pat, width, kind) in puncts {
        match *width {
            2 => {
                let bs: Vec<u8> = pat.bytes().collect();
                assert_eq!(bs.len(), 2, "pattern {pat}");
                two.push((bs[0], bs[1], kind.clone()));
            }
            1 => {
                let bs: Vec<u8> = pat.bytes().collect();
                assert_eq!(bs.len(), 1, "pattern {pat}");
                one.push((bs[0], kind.clone()));
            }
            w => panic!("unsupported punct width {w} for {pat}"),
        }
    }
    two.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    one.sort_by_key(|x| x.0);

    let mut arms = String::new();
    for (a, b, kind) in &two {
        arms.push_str(&format!(
            "        (b'{}', Some(b'{}')) => Some((TokenKind::{}, 2)),\n",
            byte_lit_char(*a),
            byte_lit_char(*b),
            kind
        ));
    }
    for (a, kind) in &one {
        arms.push_str(&format!(
            "        (b'{}', _) => Some((TokenKind::{}, 1)),\n",
            byte_lit_char(*a),
            kind
        ));
    }
    arms.push_str("        _ => None,\n");

    format!(
        r#"fn punctuation_token(bytes: &[u8], pos: usize) -> Option<(TokenKind, usize)> {{
    let first = bytes[pos];
    let second = bytes.get(pos + 1).copied();
    match (first, second) {{
{arms}    }}
}}
"#,
        arms = arms
    )
}

fn byte_lit_char(b: u8) -> char {
    match b {
        b'\'' => '\'',
        b'\\' => '\\',
        _ => b as char,
    }
}
