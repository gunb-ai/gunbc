//! Regenerate `tokenize_generated.rs` from `src/v3/compiler/tokenize.dag`.
//
// Keyword and punctuation tables are read from the lowered Dag; the scanning
// algorithm is emitted as deterministic Rust (this binary is codegen only).

use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{AtomPayload, Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};
use v3_compiler::CompileError;

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/tokenize.dag` via\n\
     // `regen_tokenize`. Regenerate instead of hand-editing.\n\n";

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dag_path = manifest_dir.join("tokenize.dag");
    let source = std::fs::read_to_string(&dag_path).expect("read tokenize.dag");
    let dag = match compile_to_dag(&source, "src/v3/compiler/tokenize.dag") {
        Ok(d) => d,
        Err(CompileError::Semantic(d)) => {
            eprintln!("tokenize.dag semantic errors:");
            for (_, diag) in d.diagnostics().iter() {
                eprintln!("  {diag:?}");
            }
            panic!("compile tokenize.dag failed");
        }
        Err(other) => panic!("compile tokenize.dag: {other:?}"),
    };
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

fn validate_kind_names(dag: &Dag, keywords: &[(String, String)], puncts: &[(String, i64, String)]) {
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
                let rust_ty = rust_type_for_decl_id(dag, field.ty);
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
    let mut s = String::new();
    s.push_str("pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic> {\n");
    s.push_str("    let bytes = source.as_bytes();\n");
    s.push_str("    let mut pos: usize = 0;\n");
    s.push_str("    let mut tokens = Vec::new();\n\n");
    s.push_str("    while pos < bytes.len() {\n");
    s.push_str("        let byte = bytes[pos];\n\n");
    s.push_str("        if byte.is_ascii_whitespace() {\n");
    s.push_str("            pos += 1;\n");
    s.push_str("            continue;\n");
    s.push_str("        }\n\n");
    s.push_str("        // Line comments: `// ...` to end of line.\n");
    s.push_str("        if byte == b'/' && bytes.get(pos + 1) == Some(&b'/') {\n");
    s.push_str("            pos += 2;\n");
    s.push_str("            while pos < bytes.len() && bytes[pos] != b'\\n' {\n");
    s.push_str("                pos += 1;\n");
    s.push_str("            }\n");
    s.push_str("            continue;\n");
    s.push_str("        }\n\n");
    s.push_str("        let start = pos;\n\n");
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
    s.push_str("        if byte.is_ascii_digit() {\n");
    s.push_str("            let mut end = pos;\n");
    s.push_str("            while end < bytes.len() && bytes[end].is_ascii_digit() {\n");
    s.push_str("                end += 1;\n");
    s.push_str("            }\n");
    s.push_str("            let literal = &source[start..end];\n");
    s.push_str(
        "            let value: i64 = literal.parse().map_err(|_| Diagnostic::TokenizerError {\n",
    );
    s.push_str("                message: format!(\"invalid integer literal `{}`\", literal),\n");
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
    s.push_str("        if byte.is_ascii_alphabetic() || byte == b'_' {\n");
    s.push_str("            let mut end = pos;\n");
    s.push_str("            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {\n");
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
    s.push_str("        // String literal: \"...\"\n");
    s.push_str("        //\n");
    s.push_str("        // Minimal escape surface for bootstrap-staged structural data:\n");
    s.push_str(
        "        // `\\\"`, `\\\\`, `\\n`, `\\r`, `\\t`. Unknown `\\x` pairs preserve the\n",
    );
    s.push_str("        // old M0 behavior and stay literal as `\\` + `x`. Raw newlines\n");
    s.push_str("        // are preserved until the closing `\"`.\n");
    s.push_str("        if byte == b'\"' {\n");
    s.push_str("            let mut end = pos + 1;\n");
    s.push_str("            let mut content = String::new();\n");
    s.push_str("            let mut terminated = false;\n");
    s.push_str("            while end < bytes.len() {\n");
    s.push_str("                match bytes[end] {\n");
    s.push_str("                    b'\"' => {\n");
    s.push_str("                        terminated = true;\n");
    s.push_str("                        end += 1;\n");
    s.push_str("                        break;\n");
    s.push_str("                    }\n");
    s.push_str("                    b'\\\\' => {\n");
    s.push_str("                        let Some(escaped) = bytes.get(end + 1).copied() else {\n");
    s.push_str("                            return Err(Diagnostic::TokenizerError {\n");
    s.push_str(
        "                                message: \"unterminated string escape\".to_string(),\n",
    );
    s.push_str("                                span: SourceSpan::new(file, start as u32, (end + 1) as u32),\n");
    s.push_str("                                fixes: Vec::new(),\n");
    s.push_str("                            });\n");
    s.push_str("                        };\n");
    s.push_str("                        match escaped {\n");
    s.push_str("                            b'\"' => content.push('\"'),\n");
    s.push_str("                            b'\\\\' => content.push('\\\\'),\n");
    s.push_str("                            b'n' => content.push('\u{000A}'),\n");
    s.push_str("                            b'r' => content.push('\u{000D}'),\n");
    s.push_str("                            b't' => content.push('\u{0009}'),\n");
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
    s.push_str("                    message: \"unterminated string literal\".to_string(),\n");
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
