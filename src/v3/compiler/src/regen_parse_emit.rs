//! Shared `regen_parse` emission: compile `runtime_mirrors.dag`, emit `parse_generated.rs`
//! body, run `rustfmt --emit stdout`. Used by the `regen_parse` binary (writes the file)
//! and by hermetic integration tests (compare in-memory only).

use std::collections::BTreeSet;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::compile_runtime_mirrors_authority_dag;
use crate::dag::{AtomPayload, CardinalityBound, Dag, DeclarationId, TypeConnective};
use crate::CompileError;

const HEADER: &str =
    "// AUTO-GENERATED from `src/v3/compiler/runtime_mirrors.dag` (Surface carriers)\n\
     // via `regen_parse` + `parse_parser_body.txt`. Regenerate instead of hand-editing.\n\n";

/// Failure compiling the authority DAG or running `rustfmt` on the combined module text.
#[derive(Debug)]
pub enum RenderParseGeneratedError {
    Compile(Box<CompileError>),
    Rustfmt(String),
}

impl fmt::Display for RenderParseGeneratedError {
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

/// Compile [`runtime_mirrors_source`] with [`compile_runtime_mirrors_authority_dag`], splice
/// [`parser_body`], format with `rustfmt --emit stdout`. Does not read or write workspace paths.
pub fn render_parse_generated_rs(
    runtime_mirrors_source: &str,
    runtime_mirrors_file: &str,
    parser_body: &str,
) -> Result<String, RenderParseGeneratedError> {
    let dag = compile_runtime_mirrors_authority_dag(runtime_mirrors_source, runtime_mirrors_file)
        .map_err(|e| RenderParseGeneratedError::Compile(Box::new(e)))?;
    let rust = emit_parse_module(&dag, parser_body);
    let combined = format!("{HEADER}{rust}");
    rustfmt_stdout(&combined).map_err(RenderParseGeneratedError::Rustfmt)
}

fn rustfmt_stdout(combined: &str) -> Result<String, String> {
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
        .write_all(combined.as_bytes())
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
        .unwrap_or_else(|| panic!("missing `{name}` in runtime_mirrors.dag"));

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
                    "anonymous nested type as field `{field_label}` of `{parent_name}` — give it a name in runtime_mirrors.dag"
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
