//! Shared `regen_parse` emission: compile `src/v3/std/parse_surface.dag`, emit
//! `parse_generated.rs` body, run `rustfmt --emit stdout`. Used by the
//! `regen_parse` binary (writes the file) and by hermetic integration tests
//! (compare in-memory only).

use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::compile_parse_surface_std_authority_dag;
use crate::CompileError;

const HEADER: &str = "// AUTO-GENERATED from `src/v3/std/parse_surface.dag` (Surface carriers)\n\
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

/// Compile [`parse_surface_source`] with [`compile_parse_surface_std_authority_dag`], splice
/// [`parser_body`], format with `rustfmt --emit stdout`. Does not read or write workspace paths.
pub fn render_parse_generated_rs(
    parse_surface_source: &str,
    parse_surface_file: &str,
    parser_body: &str,
) -> Result<String, RenderParseGeneratedError> {
    let _dag = compile_parse_surface_std_authority_dag(parse_surface_source, parse_surface_file)
        .map_err(|e| RenderParseGeneratedError::Compile(Box::new(e)))?;
    let rust = emit_parse_module(parser_body);
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

fn emit_parse_module(parser_body: &str) -> String {
    let mut out = String::new();
    out.push_str("use crate::diagnostics::{Diagnostic, SourceSpan};\n");
    out.push_str(
        "pub use crate::parse_surface::{SurfaceExpr, SurfaceField, SurfaceItem, SurfaceLiteral, \
         SurfaceMatchArm, SurfaceModule, SurfaceParam, SurfacePattern, SurfacePatternField, \
         SurfaceRecordField, SurfaceType, SurfaceVariant, VariantPayload};\n",
    );
    out.push_str(
        "use crate::parse_tables::{binary_op_at_level, bracket_role, is_type_rhs_boundary_keyword, \
         primary_atom_class, primary_prefix_dispatch, soft_keyword_ident_spelling, \
         top_level_item_dispatch, \
         BinaryOpLevel, BracketRole, ItemDispatchKind, PrimaryAtomClass, PrimaryPrefixDispatch};\n",
    );
    out.push_str("use crate::tokenize::{Token, TokenKind};\n\n");
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
