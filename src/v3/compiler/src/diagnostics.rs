// Diagnostics and the enforced mark_unresolved API.
//
// The load-bearing guarantee:
//   Port.value_type == None  iff  DiagnosticTable contains PortId
//
// Enforcement is structural: Dag::clear_port_type is pub(crate) and
// is ONLY called from Dag::mark_unresolved (which also writes the
// diagnostic entry atomically). No other call path can null a port.
//
// G2 guardrail: SourceSpan always carries a non-empty `file` field.
// Cross-artifact diagnostics (M2+ multi-file, multi-language) rely
// on spans being addressable across files. Do NOT replace this with
// a bare (line, col) pair — that forecloses the thesis goal.
//
// ────────────────────────────────────────────────────────────────
// DEFERRED DISSOLUTION: Diagnostic enum is a scaffold.
//
// Current variants (M0.5): TokenizerError, ParseError, TypeMismatch,
// ArityMismatch, ResolveError. The v3 target shape per
// docs/v3-modeling-analysis.md §CompilerDiagnostic is a 5-field
// record:
//
//   Diagnostic {
//     span: SourceSpan,
//     category: DiagnosticCategory,  // Type | Cardinality | ...
//     subject: DiagnosticSubject,    // typed ref to thing flagged
//     detail: DiagnosticDetail,      // category-specific payload
//     correction: Option<Correction>,// the literal fixed code
//     producing_node: Option<NodeId>,
//   }
//
// The `correction` field is the thesis claim "show the correct code"
// (THESIS.md §"Error handling"). Every diagnostic that can structurally
// compute a fix must carry one. Design: SELF_HOSTING.md §14.6.
// Roundtrip test pattern: break → diagnose → apply correction →
// recompile → zero diagnostics. Lands at L1.5.
//
// TRIGGER for dissolution: when typed DeclarationId / FieldRef references
// replace the name + span carriers below. M1(2.5) delivered the
// DeclarationId substrate; the subject/detail refactor is next.
//
// Extension from 3→5 variants at M0.5 is deferred dissolution with
// justification: the fail-closed invariant (C-8) requires that every
// detectable failure produces a diagnostic, and M0's infer + lower
// stages reach failure categories (arity, resolve) that TypeMismatch
// does not cover. Until Subject/Detail infrastructure exists,
// extension is the least-bad interim form.
//
// DO NOT add a severity field. C-8: all diagnostics are errors.
// DO NOT add a warning variant. DO NOT add a "maybe wrong" state.
// If a condition is detected and it's wrong, emit the diagnostic
// and fail. If it's not wrong, emit nothing. There is no middle.
// ────────────────────────────────────────────────────────────────

use std::collections::HashMap;

use crate::dag::{
    AtomPayload, CardinalityBound, Dag, DeclarationId, Field, FieldValue, PortId, TypeConnective,
    ValueBody,
};
use crate::types::TypeShape;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: String,
    pub byte_start: u32,
    pub byte_end: u32,
}

impl SourceSpan {
    pub fn new(file: impl Into<String>, byte_start: u32, byte_end: u32) -> Self {
        Self {
            file: file.into(),
            byte_start,
            byte_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    pub description: String,
    // Retained even though the current source-level renderer only
    // prints replacement text. Future fix surfaces can use this to
    // point at the precise range the correction applies to without
    // changing the carrier shape.
    pub span: SourceSpan,
    pub new_source: String,
}

#[derive(Debug, Clone)]
pub enum Diagnostic {
    TokenizerError {
        message: String,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    ParseError {
        message: String,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    TypeMismatch {
        expected: TypeShape,
        actual: TypeShape,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    ArityMismatch {
        function: String,
        expected: usize,
        actual: usize,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    ResolveError {
        name: String,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
    /// DB-18 R2 — branch condition port did not resolve to `Bool` at lowering.
    BranchConditionNotBool {
        port: PortId,
        actual_type: Option<TypeShape>,
        span: SourceSpan,
        fixes: Vec<Correction>,
    },
}

impl Diagnostic {
    pub fn span(&self) -> &SourceSpan {
        match self {
            Diagnostic::TokenizerError { span, .. }
            | Diagnostic::ParseError { span, .. }
            | Diagnostic::TypeMismatch { span, .. }
            | Diagnostic::ArityMismatch { span, .. }
            | Diagnostic::ResolveError { span, .. }
            | Diagnostic::BranchConditionNotBool { span, .. } => span,
        }
    }

    pub fn fixes(&self) -> &[Correction] {
        match self {
            Diagnostic::TokenizerError { fixes, .. }
            | Diagnostic::ParseError { fixes, .. }
            | Diagnostic::TypeMismatch { fixes, .. }
            | Diagnostic::ArityMismatch { fixes, .. }
            | Diagnostic::ResolveError { fixes, .. }
            | Diagnostic::BranchConditionNotBool { fixes, .. } => fixes,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Diagnostic::TokenizerError { message, .. } | Diagnostic::ParseError { message, .. } => {
                message.clone()
            }
            Diagnostic::TypeMismatch {
                expected, actual, ..
            } => format!("expected {expected:?}, got {actual:?}"),
            Diagnostic::ArityMismatch {
                function,
                expected,
                actual,
                ..
            } => format!("{function} expected {expected}, got {actual}"),
            Diagnostic::ResolveError { name, .. } => name.clone(),
            Diagnostic::BranchConditionNotBool { actual_type, .. } => match actual_type {
                Some(ty) => format!("branch condition port is not Bool (got {ty:?})"),
                None => "branch condition port is not Bool (type not resolved)".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStyleTarget {
    Rust,
    Go,
    Python,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticRenderError {
    MissingLanguageSpec(&'static str),
    MissingCleanEmissionContract(&'static str),
    MalformedCleanEmissionContract {
        declaration: DeclarationId,
        detail: &'static str,
    },
    MissingCorrectionStyle(&'static str),
    MalformedCorrectionStyle {
        declaration: DeclarationId,
        detail: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CorrectionStyleBinding {
    indent_unit: String,
    line_ending: String,
    string_quote: String,
    trailing_semicolon: bool,
}

impl CorrectionStyleBinding {
    fn build(dag: &Dag, declaration: DeclarationId) -> Result<Self, DiagnosticRenderError> {
        let fields = match dag.declaration(declaration).value_body.as_ref() {
            Some(crate::dag::ValueBody::Structural { fields }) => fields,
            _ => {
                return Err(DiagnosticRenderError::MalformedCorrectionStyle {
                    declaration,
                    detail: "correction_style declaration must carry a Structural value_body",
                });
            }
        };
        Ok(Self {
            indent_unit: require_string(fields, "indent_unit", declaration)?,
            line_ending: require_string(fields, "line_ending", declaration)?,
            string_quote: require_string(fields, "string_quote", declaration)?.replace("%Q", "\""),
            trailing_semicolon: require_bool(fields, "trailing_semicolon", declaration)?,
        })
    }
}

pub fn render_diagnostic_for_target(
    dag: &Dag,
    target: DiagnosticStyleTarget,
    diagnostic: &Diagnostic,
) -> Result<String, DiagnosticRenderError> {
    let language_spec = language_spec_for_target(dag, target)?;
    render_diagnostic_for_language_spec(dag, language_spec, diagnostic)
}

pub fn render_diagnostic_for_language_spec(
    dag: &Dag,
    language_spec: DeclarationId,
    diagnostic: &Diagnostic,
) -> Result<String, DiagnosticRenderError> {
    let declaration = correction_style_for_language_spec(dag, language_spec)?;
    let style = CorrectionStyleBinding::build(dag, declaration)?;
    Ok(render_diagnostic_with_style(diagnostic, &style))
}

fn language_spec_for_target(
    dag: &Dag,
    target: DiagnosticStyleTarget,
) -> Result<DeclarationId, DiagnosticRenderError> {
    let (language_spec, missing_name) = match target {
        DiagnosticStyleTarget::Rust => (dag.rust_language_spec(), "rust_language"),
        DiagnosticStyleTarget::Go => (dag.go_language_spec(), "go_language"),
        DiagnosticStyleTarget::Python => (dag.python_language_spec(), "python_language"),
    };
    language_spec.ok_or(DiagnosticRenderError::MissingLanguageSpec(missing_name))
}

fn correction_style_for_language_spec(
    dag: &Dag,
    language_spec: DeclarationId,
) -> Result<DeclarationId, DiagnosticRenderError> {
    let clean_emission_decl = dag
        .target_syntax_bundle_for_language(language_spec)
        .map(|bundle| bundle.clean_emission_spec)
        .ok_or(DiagnosticRenderError::MissingCleanEmissionContract(
            "clean_emission.correction_style",
        ))?;
    let Some(ValueBody::Structural { fields }) = &dag.declaration(clean_emission_decl).value_body
    else {
        return Err(DiagnosticRenderError::MalformedCleanEmissionContract {
            declaration: clean_emission_decl,
            detail: "clean emission declaration must carry a structural value_body",
        });
    };
    let value = fields
        .iter()
        .find(|(label, _)| label == "correction_style")
        .map(|(_, value)| value)
        .ok_or(DiagnosticRenderError::MissingCorrectionStyle(
            "clean_emission.correction_style",
        ))?;
    match value {
        FieldValue::Reference(declaration) => Ok(*declaration),
        _ => Err(DiagnosticRenderError::MalformedCleanEmissionContract {
            declaration: clean_emission_decl,
            detail: "clean emission correction_style field must be a declaration reference",
        }),
    }
}

fn render_diagnostic_with_style(diagnostic: &Diagnostic, style: &CorrectionStyleBinding) -> String {
    let mut lines = vec![format!(
        "ERROR at {}:{}-{}: {}",
        diagnostic.span().file,
        diagnostic.span().byte_start,
        diagnostic.span().byte_end,
        diagnostic.message()
    )];
    for (index, fix) in diagnostic.fixes().iter().enumerate() {
        lines.push(format!("FIX (option {}): {}", index + 1, fix.description));
        lines.push(render_correction_source(fix, style));
    }
    lines.join(&style.line_ending)
}

fn render_correction_source(fix: &Correction, style: &CorrectionStyleBinding) -> String {
    let suffix = if style.trailing_semicolon { ";" } else { "" };
    let normalized = fix.new_source.replace("\r\n", "\n").replace('\r', "\n");
    if !normalized.contains('\n') {
        return format!(
            "{}{}{}{}{}",
            style.indent_unit,
            style.string_quote,
            escape_for_string_literal(&normalized, &style.string_quote),
            style.string_quote,
            suffix
        );
    }

    let body = normalized
        .split('\n')
        .map(|line| {
            format!(
                "{}{}",
                style.indent_unit,
                escape_for_string_literal(line, &style.string_quote)
            )
        })
        .collect::<Vec<_>>()
        .join(&style.line_ending);
    format!(
        "{}{}{}{}",
        style.string_quote, style.line_ending, body, style.string_quote
    ) + suffix
}

fn escape_for_string_literal(source: &str, string_quote: &str) -> String {
    let escaped = source.replace('\\', "\\\\");
    if string_quote.is_empty() {
        return escaped;
    }
    escaped.replace(string_quote, &format!("\\{string_quote}"))
}

fn require_string(
    fields: &[(String, FieldValue)],
    label: &'static str,
    declaration: DeclarationId,
) -> Result<String, DiagnosticRenderError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style is missing a required String field",
        })?;
    match value {
        FieldValue::Literal(crate::dag::LiteralBits::String(value)) => Ok(value.clone()),
        _ => Err(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style String field must be a string literal",
        }),
    }
}

fn require_bool(
    fields: &[(String, FieldValue)],
    label: &'static str,
    declaration: DeclarationId,
) -> Result<bool, DiagnosticRenderError> {
    let value = fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .map(|(_, value)| value)
        .ok_or(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style is missing a required Bool field",
        })?;
    match value {
        FieldValue::Literal(crate::dag::LiteralBits::Bool(value)) => Ok(*value),
        _ => Err(DiagnosticRenderError::MalformedCorrectionStyle {
            declaration,
            detail: "correction_style Bool field must be a bool literal",
        }),
    }
}

pub(crate) fn declaration_display_name(dag: &Dag, declaration: DeclarationId) -> String {
    dag.declaration(declaration)
        .name
        .clone()
        .unwrap_or_else(|| format!("declaration#{}", declaration.raw()))
}

pub(crate) fn example_source_for_decl(dag: &Dag, declaration: DeclarationId) -> Option<String> {
    example_source_for_decl_inner(dag, declaration, 0)
}

fn example_source_for_decl_inner(
    dag: &Dag,
    declaration: DeclarationId,
    depth: usize,
) -> Option<String> {
    if depth >= 8 {
        return None;
    }
    let decl = dag.declaration(declaration);
    match decl.name.as_deref() {
        Some("Int") => return Some("1".to_string()),
        Some("Bool") => return Some("true".to_string()),
        Some("String") => return Some("\"x\"".to_string()),
        Some("List") => return Some("[]".to_string()),
        _ => {}
    }
    match &decl.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            example_source_for_decl_inner(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation { template, .. } => {
            if dag.declaration(*template).name.as_deref() == Some("List") {
                return Some("[]".to_string());
            }
            example_source_for_decl_inner(dag, *template, depth + 1)
        }
        TypeConnective::Conj { children } => {
            let fields: Option<Vec<_>> = children
                .iter()
                .map(|field| {
                    Some(format!(
                        "{}: {}",
                        field.label,
                        example_source_for_decl_inner(dag, field.ty, depth + 1)?
                    ))
                })
                .collect();
            Some(format!("{{ {} }}", fields?.join(", ")))
        }
        TypeConnective::Disj { variants } => variants
            .iter()
            .find_map(|variant| render_variant_witness(dag, variant, depth + 1)),
        TypeConnective::Cardinality {
            bound: CardinalityBound::AtMostOne,
            ..
        } => Some("None".to_string()),
        _ => None,
    }
}

fn render_variant_witness(dag: &Dag, variant: &Field, depth: usize) -> Option<String> {
    let TypeConnective::Conj { children } = &dag.declaration(variant.ty).connective else {
        return None;
    };
    if children.is_empty() {
        return Some(variant.label.clone());
    }
    // Constructor invocation in the current `.dag` surface is
    // positional (`Variant(arg0, arg1)`), even when the payload
    // declaration's fields are named. Preserve declaration-derived
    // child order here, but do not invent an unsupported
    // `Variant { field: value }` surface until lowering accepts that
    // shape structurally.
    let payload: Option<Vec<_>> = children
        .iter()
        .map(|field| example_source_for_decl_inner(dag, field.ty, depth + 1))
        .collect();
    Some(format!("{}({})", variant.label, payload?.join(", ")))
}

pub(crate) fn witness_correction_for_decl(
    dag: &Dag,
    declaration: DeclarationId,
    span: SourceSpan,
    description: impl Into<String>,
) -> Option<Correction> {
    Some(Correction {
        description: description.into(),
        span,
        new_source: example_source_for_decl(dag, declaration)?,
    })
}

#[derive(Debug, Default, Clone)]
pub struct DiagnosticTable {
    entries: HashMap<PortId, Diagnostic>,
}

impl DiagnosticTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, port: PortId) -> bool {
        self.entries.contains_key(&port)
    }

    pub fn get(&self, port: PortId) -> Option<&Diagnostic> {
        self.entries.get(&port)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterate `(port, diagnostic)` pairs. Used by tests and callers
    /// that need to scan all diagnostics for a specific kind.
    pub fn iter(&self) -> impl Iterator<Item = (PortId, &Diagnostic)> {
        self.entries.iter().map(|(p, d)| (*p, d))
    }

    /// Insert a diagnostic entry for a port. pub(crate) because the
    /// only callers are Dag::mark_unresolved (which atomically also
    /// clears the port's value_type) and should never be called
    /// independently from outside the crate.
    pub(crate) fn insert(&mut self, port: PortId, diagnostic: Diagnostic) {
        self.entries.insert(port, diagnostic);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_emission_contracts_reference_named_correction_styles() {
        let dag = Dag::new();
        let assert_reference =
            |clean_decl: DeclarationId, expected_style_name: &str, expected_name: &str| {
                let fields = match dag
                    .declaration(clean_decl)
                    .value_body
                    .as_ref()
                    .expect("clean emission carries value_body")
                {
                    crate::dag::ValueBody::Structural { fields } => fields,
                    other => panic!("clean emission must be structural, got {other:?}"),
                };
                let value = fields
                    .iter()
                    .find(|(label, _)| label == "correction_style")
                    .map(|(_, value)| value)
                    .expect("correction_style field exists");
                match value {
                    FieldValue::Reference(actual) => assert_eq!(
                        dag.declaration(*actual).name.as_deref(),
                        Some(expected_style_name),
                        "{expected_name} should point at its named correction style"
                    ),
                    other => panic!("correction_style must be a Reference, got {other:?}"),
                }
            };
        assert_reference(
            dag.rust_clean_emission_spec().expect("rust clean emission"),
            "rust_correction_style",
            "rust_clean_emission",
        );
        assert_reference(
            dag.go_clean_emission_spec().expect("go clean emission"),
            "go_correction_style",
            "go_clean_emission",
        );
        assert_reference(
            dag.python_clean_emission_spec()
                .expect("python clean emission"),
            "python_correction_style",
            "python_clean_emission",
        );
    }

    #[test]
    fn target_syntax_bundles_pair_each_language_with_its_clean_emission_contract() {
        let dag = Dag::new();
        let assert_bundle = |language_spec: DeclarationId,
                             expected_clean_emission: DeclarationId| {
            let bundle = dag
                .target_syntax_bundle_for_language(language_spec)
                .expect("bundle should exist for cached language spec");
            assert_eq!(bundle.language_spec, language_spec);
            assert_eq!(bundle.clean_emission_spec, expected_clean_emission);
        };
        assert_bundle(
            dag.rust_language_spec().expect("rust language"),
            dag.rust_clean_emission_spec().expect("rust clean emission"),
        );
        assert_bundle(
            dag.go_language_spec().expect("go language"),
            dag.go_clean_emission_spec().expect("go clean emission"),
        );
        assert_bundle(
            dag.python_language_spec().expect("python language"),
            dag.python_clean_emission_spec()
                .expect("python clean emission"),
        );
    }

    #[test]
    fn render_rust_diagnostic_uses_rust_correction_style() {
        let dag = Dag::new();
        let rendered = render_diagnostic_for_target(
            &dag,
            DiagnosticStyleTarget::Rust,
            &Diagnostic::ResolveError {
                name: "field `c` does not exist on `Point`".to_string(),
                span: SourceSpan::new("field.v3", 12, 19),
                fixes: vec![Correction {
                    description: "did you mean `point.a`?".to_string(),
                    span: SourceSpan::new("field.v3", 12, 19),
                    new_source: "point.a".to_string(),
                }],
            },
        )
        .expect("render");
        assert!(rendered.contains("FIX (option 1): did you mean `point.a`?"));
        assert!(rendered.contains("\n    \"point.a\";"));
    }

    #[test]
    fn render_go_diagnostic_uses_go_correction_style() {
        let dag = Dag::new();
        let rendered = render_diagnostic_for_target(
            &dag,
            DiagnosticStyleTarget::Go,
            &Diagnostic::ResolveError {
                name: "field `c` does not exist on `Point`".to_string(),
                span: SourceSpan::new("field.v3", 12, 19),
                fixes: vec![Correction {
                    description: "did you mean `point.a`?".to_string(),
                    span: SourceSpan::new("field.v3", 12, 19),
                    new_source: "point.a".to_string(),
                }],
            },
        )
        .expect("render");
        assert!(rendered.contains("FIX (option 1): did you mean `point.a`?"));
        assert!(rendered.contains("\n\t\"point.a\""));
        assert!(!rendered.contains("\n\t\"point.a\";"));
    }

    #[test]
    fn render_python_diagnostic_uses_python_correction_style() {
        let dag = Dag::new();
        let rendered = render_diagnostic_for_target(
            &dag,
            DiagnosticStyleTarget::Python,
            &Diagnostic::ResolveError {
                name: "field `c` does not exist on `Point`".to_string(),
                span: SourceSpan::new("field.v3", 12, 19),
                fixes: vec![Correction {
                    description: "did you mean `point.a`?".to_string(),
                    span: SourceSpan::new("field.v3", 12, 19),
                    new_source: "point.a".to_string(),
                }],
            },
        )
        .expect("render");
        assert!(rendered.contains("FIX (option 1): did you mean `point.a`?"));
        assert!(rendered.contains("\n    \"point.a\""));
        assert!(!rendered.contains("\n    \"point.a\";"));
    }

    #[test]
    fn render_rust_diagnostic_escapes_quotes_and_backslashes_in_fix_source() {
        let dag = Dag::new();
        let rendered = render_diagnostic_for_target(
            &dag,
            DiagnosticStyleTarget::Rust,
            &Diagnostic::ResolveError {
                name: "field `path` does not exist on `Config`".to_string(),
                span: SourceSpan::new("field.v3", 12, 19),
                fixes: vec![Correction {
                    description: "did you mean `config.path`?".to_string(),
                    span: SourceSpan::new("field.v3", 12, 19),
                    new_source: "config[\"path\\\\name\"]".to_string(),
                }],
            },
        )
        .expect("render");
        assert!(rendered.contains("\n    \"config[\\\"path\\\\\\\\name\\\"]\";"));
    }

    #[test]
    fn render_multiline_fix_escapes_quotes_and_backslashes_in_each_line() {
        let dag = Dag::new();
        let rendered = render_diagnostic_for_target(
            &dag,
            DiagnosticStyleTarget::Python,
            &Diagnostic::ResolveError {
                name: "field `path` does not exist on `Config`".to_string(),
                span: SourceSpan::new("field.v3", 12, 19),
                fixes: vec![Correction {
                    description: "did you mean `config.path`?".to_string(),
                    span: SourceSpan::new("field.v3", 12, 19),
                    new_source: "config[\n\"path\\\\name\"\n]".to_string(),
                }],
            },
        )
        .expect("render");
        assert!(rendered.contains("\"\n    config[\n    \\\"path\\\\\\\\name\\\"\n    ]\""));
    }
}
