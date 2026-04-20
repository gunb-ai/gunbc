//! Regenerate `parse_generated.rs` from `src/v3/compiler/parse.dag` plus the
//! checked-in recursive-descent body fragment (`parse_parser_body.txt`).

use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{AtomPayload, CardinalityBound, Dag, DeclarationId, TypeConnective};
use v3_compiler::generated_files::GENERATED_FILES;
use v3_compiler::CompileError;

const GENERATED_FILE: &str = "src/v3/compiler/src/parse_generated.rs";
const PARSE_AUTHORITY_FILE: &str = "src/v3/compiler/parse.dag";
const PARSER_BODY_REL: &str = "parse_parser_body.txt";

const HEADER: &str = "// AUTO-GENERATED from `src/v3/compiler/parse.dag` via\n\
     // `regen_parse`. Regenerate instead of hand-editing.\n\n";

fn main() {
    assert!(
        GENERATED_FILES.contains(&GENERATED_FILE),
        "`regen_parse` writes `{GENERATED_FILE}` but that path is not \
         registered in `REGEN_OUTPUTS` in `src/v3/compiler/build.rs`."
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dag_path = manifest_dir.join("parse.dag");
    let source = std::fs::read_to_string(&dag_path).expect("read parse.dag");
    let dag = compile_authority_dag(&source, PARSE_AUTHORITY_FILE);
    let body_path = manifest_dir.join(PARSER_BODY_REL);
    let parser_body = std::fs::read_to_string(&body_path)
        .unwrap_or_else(|e| panic!("read parser body fragment `{}`: {e}", body_path.display()));

    let rust = emit_parse_module(&dag, &parser_body);
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

    let out_path = manifest_dir.join("src").join("parse_generated.rs");
    std::fs::write(&out_path, &formatted).expect("write parse_generated.rs");
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

fn emit_parse_module(dag: &Dag, parser_body: &str) -> String {
    let mut out = String::new();
    out.push_str("use crate::diagnostics::{Diagnostic, SourceSpan};\n");
    out.push_str("use crate::operators::OperatorKind;\n");
    out.push_str("use crate::tokenize::{Token, TokenKind};\n\n");
    out.push_str(&emit_surface_types(dag));
    out.push_str(
        r#"impl SurfaceType {
    pub fn span(&self) -> &SourceSpan {
        match self {
            SurfaceType::Named { span, .. }
            | SurfaceType::Parameterized { span, .. }
            | SurfaceType::Optional { span, .. }
            | SurfaceType::Arrow { span, .. } => span,
        }
    }
}

"#,
    );
    out.push_str(parser_body);
    out
}

/// Surface AST types declared in `parse.dag`, emitted as Rust `struct` /
/// `enum` definitions. `List<T>` becomes `Vec<T>`; recursive `SurfaceType` /
/// `SurfaceExpr` edges that require indirection in Rust use `Box<…>` where
/// the handwritten parser did.
fn emit_surface_types(dag: &Dag) -> String {
    let root_names = [
        "SurfaceLiteral",
        "SurfaceField",
        "VariantPayload",
        "SurfaceVariant",
        "SurfaceType",
        "SurfacePatternField",
        "SurfacePattern",
        "SurfaceRecordField",
        "SurfaceExpr",
        "SurfaceMatchArm",
        "SurfaceParam",
        "SurfaceItem",
        "SurfaceModule",
    ];
    let mut emitted = BTreeSet::new();
    let mut out = String::new();
    for name in root_names {
        emit_named_declaration(dag, name, &mut emitted, &mut out);
    }
    out.push('\n');
    out
}

fn emit_named_declaration(dag: &Dag, name: &str, emitted: &mut BTreeSet<String>, out: &mut String) {
    if emitted.contains(name) {
        return;
    }
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing `{name}` in parse.dag"));

    match &decl.connective {
        TypeConnective::Conj { children } => {
            emitted.insert(name.to_string());
            out.push_str("#[derive(Debug, Clone)]\n");
            out.push_str(&format!("pub struct {name} {{\n"));
            for field in children {
                let rust_ty = rust_type_for_field(dag, field.ty, name, &field.label, false);
                out.push_str(&format!("    pub {}: {rust_ty},\n", field.label));
            }
            out.push_str("}\n\n");
        }
        TypeConnective::Disj { variants } => {
            emitted.insert(name.to_string());
            out.push_str("#[derive(Debug, Clone)]\n");
            out.push_str(&format!("pub enum {name} {{\n"));
            for v in variants {
                let payload = dag.declaration(v.ty);
                match &payload.connective {
                    TypeConnective::Conj { children } if children.is_empty() => {
                        out.push_str(&format!("    {},\n", v.label));
                    }
                    TypeConnective::Conj { children }
                        if children.len() == 1 && children[0].label == "_0" =>
                    {
                        let rust_ty = rust_type_for_field(dag, children[0].ty, name, "_0", false);
                        out.push_str(&format!("    {}({rust_ty}),\n", v.label));
                    }
                    TypeConnective::Conj { children } => {
                        out.push_str(&format!("    {} {{\n", v.label));
                        for field in children {
                            let rust_ty =
                                rust_type_for_field(dag, field.ty, name, &field.label, false);
                            out.push_str(&format!("        {}: {rust_ty},\n", field.label));
                        }
                        out.push_str("    },\n");
                    }
                    other => panic!("{name}::{}: unexpected payload {other:?}", v.label),
                }
            }
            out.push_str("}\n\n");
        }
        other => panic!("{name}: unsupported connective {other:?}"),
    }
}

fn rust_type_for_field(
    dag: &Dag,
    id: DeclarationId,
    parent_name: &str,
    field_label: &str,
    in_list: bool,
) -> String {
    let decl = dag.declaration(id);
    if let Some(mapped) = map_substrate_scalar_to_rust(dag, id) {
        return mapped;
    }
    let base = match &decl.connective {
        TypeConnective::Atom(ap) => match ap {
            AtomPayload::ResolvedByName(inner) | AtomPayload::ResolvedByStructure(inner) => {
                return rust_type_for_field(dag, *inner, parent_name, field_label, in_list);
            }
            AtomPayload::TypeParam(label) => label.clone(),
            AtomPayload::Literal(_) | AtomPayload::UnresolvedIdentifier(_) => {
                panic!("unexpected atom in field type: {ap:?}")
            }
        },
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let template_decl = dag.declaration(*template);
            let template_name = template_decl
                .name
                .as_deref()
                .unwrap_or_else(|| panic!("anonymous list template"));
            if template_name == "String" {
                return "String".to_string();
            }
            if template_name == "List"
                || (template_name == "FreeMonoid" && arguments.len() == 1)
            {
                assert_eq!(arguments.len(), 1, "List/FreeMonoid arity");
                let elem = arguments[0].value;
                let inner = rust_type_for_field(dag, elem, parent_name, field_label, true);
                return format!("Vec<{inner}>");
            }
            if matches!(template_name, "Int" | "Int64") {
                return "i64".to_string();
            }
            if template_name == "Bool" {
                return "bool".to_string();
            }
            if template_name == "String" {
                return "String".to_string();
            }
            panic!("unsupported instantiation `{template_name}`");
        }
        TypeConnective::Cardinality { element, bound } => {
            let inner = rust_type_for_field(dag, *element, parent_name, field_label, in_list);
            return match bound {
                CardinalityBound::AtMostOne => format!("Option<{inner}>"),
                CardinalityBound::Unbounded => format!("Vec<{inner}>"),
                CardinalityBound::Exact(n) => {
                    panic!("unsupported exact cardinality {n} for field `{field_label}`")
                }
            };
        }
        TypeConnective::Conj { .. } | TypeConnective::Disj { .. } => {
            decl.name.clone().unwrap_or_else(|| {
                panic!(
                    "anonymous nested type as field `{field_label}` of `{parent_name}` — give it a name in parse.dag"
                )
            })
        }
        other => panic!("unsupported field connective: {other:?}"),
    };

    let needs_box = !in_list && needs_box_edge(parent_name, field_label, &base);
    let inner = if base == "Int" {
        "i64".to_string()
    } else if base == "Bool" {
        "bool".to_string()
    } else if base == "String" || base == "SourceSpan" || base == "OperatorKind" {
        base
    } else {
        base.clone()
    };
    if needs_box {
        format!("Box<{inner}>")
    } else {
        inner
    }
}

fn map_substrate_scalar_to_rust(dag: &Dag, mut id: DeclarationId) -> Option<String> {
    for _ in 0..64 {
        let decl = dag.declaration(id);
        if let Some(name) = decl.name.as_deref() {
            match name {
                "String" => return Some("String".to_string()),
                "Int" | "Int64" => return Some("i64".to_string()),
                "Bool" => return Some("bool".to_string()),
                "SourceSpan" => return Some("SourceSpan".to_string()),
                "OperatorKind" => return Some("OperatorKind".to_string()),
                _ => {}
            }
        }
        match &decl.connective {
            TypeConnective::Atom(AtomPayload::ResolvedByName(inner))
            | TypeConnective::Atom(AtomPayload::ResolvedByStructure(inner)) => {
                id = *inner;
            }
            _ => break,
        }
    }
    None
}

fn needs_box_edge(parent: &str, field: &str, rhs: &str) -> bool {
    matches!(
        (parent, field, rhs),
        ("SurfaceType", "inner", "SurfaceType")
            | ("SurfaceType", "output", "SurfaceType")
            | ("SurfaceExpr", "body", "SurfaceExpr")
            | ("SurfaceExpr", "cond", "SurfaceExpr")
            | ("SurfaceExpr", "then_branch", "SurfaceExpr")
            | ("SurfaceExpr", "else_branch", "SurfaceExpr")
            | ("SurfaceExpr", "scrutinee", "SurfaceExpr")
    )
}
