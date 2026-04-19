use std::fmt::Write as _;
use std::path::PathBuf;

use crate::dag::{AtomPayload, Declaration, DeclarationId, TypeConnective};

#[derive(Debug)]
pub struct MirrorOutput {
    pub types: String,
    pub diagnostics: String,
    pub serialize: String,
    pub dag_cost: String,
}

#[derive(Debug)]
pub enum MirrorError {
    Compile(String),
    MissingDeclaration(&'static str),
    InvalidDeclaration { name: &'static str, detail: String },
}

pub fn render_runtime_mirrors() -> Result<MirrorOutput, MirrorError> {
    let path = runtime_mirrors_path();
    let dag = crate::Dag::new();
    if !dag.diagnostics().is_empty() {
        return Err(MirrorError::Compile(format!(
            "{} did not bootstrap cleanly: {:?}",
            path.display(),
            dag.diagnostics()
        )));
    }

    validate_symbolic_cost_runtime(&dag)?;

    Ok(MirrorOutput {
        types: with_header("src/v3/std/substrate.dag", TYPE_SHAPE_TEMPLATE.trim()),
        diagnostics: with_header(
            "src/v3/compiler/runtime_mirrors.dag",
            &render_diagnostics_module(&dag)?,
        ),
        serialize: with_header(
            "src/v3/compiler/runtime_mirrors.dag",
            &render_record(&dag, "DagDifference", &[])?,
        ),
        dag_cost: with_header("src/v3/std/algebra.dag", DAG_COST_TEMPLATE.trim_start()),
    })
}

pub fn runtime_mirrors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("runtime_mirrors.dag")
}

fn with_header(authority: &str, body: &str) -> String {
    format!(
        "// AUTO-GENERATED from `{authority}`.\n// Regenerate instead of hand-editing.\n\n{body}\n"
    )
}

fn render_diagnostics_module(dag: &crate::Dag) -> Result<String, MirrorError> {
    let mut rendered = String::new();
    rendered.push_str(SOURCE_SPAN_TEMPLATE.trim());
    rendered.push('\n');
    rendered.push('\n');
    rendered.push_str(CORRECTION_TEMPLATE.trim());
    rendered.push('\n');
    rendered.push('\n');
    rendered.push_str(&render_sum_with_derives(
        dag,
        "CompilerDiagnostic",
        &[
            ("Int", "usize"),
            ("CompilerSourceSpan", "SourceSpan"),
            ("CompilerCorrection", "Correction"),
        ],
        "#[derive(Debug, Clone)]",
        Some("Diagnostic"),
    )?);
    rendered.push('\n');
    rendered.push('\n');
    rendered.push_str(&render_sum_with_derives(
        dag,
        "CompilerDiagnosticStyleTarget",
        &[],
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        Some("DiagnosticStyleTarget"),
    )?);
    rendered.push('\n');
    rendered.push('\n');
    rendered.push_str(&render_sum_with_derives(
        dag,
        "CompilerDiagnosticRenderError",
        &[("String", "&'static str")],
        "#[derive(Debug, Clone, PartialEq, Eq)]",
        Some("DiagnosticRenderError"),
    )?);
    Ok(rendered)
}

fn render_record(
    dag: &crate::Dag,
    name: &'static str,
    type_overrides: &[(&str, &str)],
) -> Result<String, MirrorError> {
    let declaration = find_named(dag, name)?;
    let TypeConnective::Conj { children } = &declaration.connective else {
        return Err(invalid(name, "expected record (Conj)"));
    };

    let mut rendered = String::new();
    writeln!(&mut rendered, "#[derive(Debug, Clone, PartialEq, Eq)]").unwrap();
    writeln!(&mut rendered, "pub struct {name} {{").unwrap();
    for field in children {
        writeln!(
            &mut rendered,
            "    pub {}: {},",
            field.label,
            render_type(dag, field.ty, type_overrides)?
        )
        .unwrap();
    }
    rendered.push('}');
    Ok(rendered)
}

fn render_sum_with_derives(
    dag: &crate::Dag,
    name: &'static str,
    type_overrides: &[(&str, &str)],
    derives: &str,
    output_name: Option<&str>,
) -> Result<String, MirrorError> {
    let declaration = find_named(dag, name)?;
    let TypeConnective::Disj { variants } = &declaration.connective else {
        return Err(invalid(name, "expected sum (Disj)"));
    };
    let output_name = output_name.unwrap_or(name);

    let mut rendered = String::new();
    writeln!(&mut rendered, "{derives}").unwrap();
    writeln!(&mut rendered, "pub enum {output_name} {{").unwrap();
    for variant in variants {
        let variant_decl = dag.declaration(variant.ty);
        let TypeConnective::Conj { children } = &variant_decl.connective else {
            return Err(MirrorError::InvalidDeclaration {
                name,
                detail: format!(
                    "variant `{}` does not lower to a Conj payload",
                    variant.label
                ),
            });
        };
        match children.as_slice() {
            [] => {
                writeln!(&mut rendered, "    {},", variant.label).unwrap();
            }
            [field] if field.label == "_0" => {
                writeln!(
                    &mut rendered,
                    "    {}({}),",
                    variant.label,
                    render_type(dag, field.ty, type_overrides)?
                )
                .unwrap();
            }
            fields => {
                writeln!(&mut rendered, "    {} {{", variant.label).unwrap();
                for field in fields {
                    writeln!(
                        &mut rendered,
                        "        {}: {},",
                        field.label,
                        render_type(dag, field.ty, type_overrides)?
                    )
                    .unwrap();
                }
                writeln!(&mut rendered, "    }},").unwrap();
            }
        }
    }
    rendered.push('}');
    Ok(rendered)
}

fn render_type(
    dag: &crate::Dag,
    declaration: DeclarationId,
    type_overrides: &[(&str, &str)],
) -> Result<String, MirrorError> {
    let declaration = dag.declaration(declaration);
    let Some(name) = declaration.name.as_deref() else {
        return render_unnamed_type(dag, declaration, type_overrides);
    };
    if let Some((_, replacement)) = type_overrides.iter().find(|(target, _)| *target == name) {
        return Ok((*replacement).to_string());
    }
    Ok(match name {
        "Int" => "i64".to_string(),
        "String" => "String".to_string(),
        "Bool" => "bool".to_string(),
        "TypeShape" | "PortId" | "DeclarationId" => name.to_string(),
        other => other.to_string(),
    })
}

fn render_unnamed_type(
    dag: &crate::Dag,
    declaration: &Declaration,
    type_overrides: &[(&str, &str)],
) -> Result<String, MirrorError> {
    match &declaration.connective {
        TypeConnective::Cardinality { element, bound } => match bound {
            crate::dag::CardinalityBound::AtMostOne => Ok(format!(
                "Option<{}>",
                render_type(dag, *element, type_overrides)?
            )),
            other => Err(MirrorError::InvalidDeclaration {
                name: "<anonymous>",
                detail: format!("unsupported cardinality bound in generated mirror: {other:?}"),
            }),
        },
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            let template = dag.declaration(*template);
            let template_name = template
                .name
                .as_deref()
                .ok_or_else(|| invalid("<anonymous>", "instantiation template missing name"))?;
            let rendered_args = arguments
                .iter()
                .map(|argument| render_type(dag, argument.value, type_overrides))
                .collect::<Result<Vec<_>, _>>()?;
            match template_name {
                "List" => Ok(format!("Vec<{}>", rendered_args.join(", "))),
                other => Ok(format!("{other}<{}>", rendered_args.join(", "))),
            }
        }
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => {
            render_type(dag, *next, type_overrides)
        }
        other => Err(MirrorError::InvalidDeclaration {
            name: "<anonymous>",
            detail: format!("unsupported generated field type: {other:?}"),
        }),
    }
}

fn find_named<'a>(dag: &'a crate::Dag, name: &'static str) -> Result<&'a Declaration, MirrorError> {
    dag.declaration_by_name(name)
        .ok_or(MirrorError::MissingDeclaration(name))
}

fn invalid(name: &'static str, detail: impl Into<String>) -> MirrorError {
    MirrorError::InvalidDeclaration {
        name,
        detail: detail.into(),
    }
}

fn validate_symbolic_cost_runtime(dag: &crate::Dag) -> Result<(), MirrorError> {
    let size_variable = find_named(dag, "SizeVariable")?;
    match &size_variable.connective {
        TypeConnective::Conj { children }
            if children.len() == 1 && children[0].label == "source_port" => {}
        other => {
            return Err(invalid(
                "SizeVariable",
                format!("unexpected reflected shape: {other:?}"),
            ))
        }
    }

    let degree = find_named(dag, "DegreeAtLeastTwo")?;
    let TypeConnective::Disj { variants } = &degree.connective else {
        return Err(invalid("DegreeAtLeastTwo", "expected Disj"));
    };
    if variants.len() != 2
        || variants[0].label != "DegreeTwo"
        || variants[1].label != "DegreeSuccessor"
    {
        return Err(invalid(
            "DegreeAtLeastTwo",
            "unexpected symbolic degree variants",
        ));
    }

    let symbolic_cost = find_named(dag, "SymbolicCost")?;
    let TypeConnective::Disj { variants } = &symbolic_cost.connective else {
        return Err(invalid("SymbolicCost", "expected Disj"));
    };
    let labels = variants
        .iter()
        .map(|variant| variant.label.as_str())
        .collect::<Vec<_>>();
    let expected = [
        "ConstantCost",
        "LinearCost",
        "PolynomialCost",
        "ProductCost",
        "SumCost",
        "LogCost",
        "UnknownCost",
    ];
    if labels.as_slice() != expected {
        return Err(invalid(
            "SymbolicCost",
            format!("unexpected variants: {labels:?}"),
        ));
    }
    Ok(())
}

const DAG_COST_TEMPLATE: &str = r#"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegreeAtLeastTwo {
    DegreeTwo,
    DegreeSuccessor { previous: Box<DegreeAtLeastTwo> },
}

impl DegreeAtLeastTwo {
    pub const TWO: Self = Self::DegreeTwo;

    pub fn new(value: i64) -> Option<Self> {
        match value {
            2 => Some(Self::DegreeTwo),
            v if v > 2 => Some(Self::DegreeSuccessor {
                previous: Box::new(Self::new(v - 1)?),
            }),
            _ => None,
        }
    }

    pub fn raw(&self) -> i64 {
        match self {
            Self::DegreeTwo => 2,
            Self::DegreeSuccessor { previous } => previous.raw() + 1,
        }
    }
}

type BoxedSymbolicCostList = NonSingletonList<Box<SymbolicCost>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolicCost {
    ConstantCost { _0: i64 },
    LinearCost { _0: SizeVariable },
    PolynomialCost {
        var: SizeVariable,
        degree: DegreeAtLeastTwo,
    },
    ProductCost { _0: BoxedSymbolicCostList },
    SumCost { _0: BoxedSymbolicCostList },
    LogCost { _0: SizeVariable },
    UnknownCost { _0: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizeVariable {
    pub source_port: PortId,
}

pub fn sequential(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::SumCost {
        _0: boxed_cost_list_pair(a, b),
    })
}

pub fn iterate(bound: SymbolicCost, body: SymbolicCost) -> SymbolicCost {
    normalize(SymbolicCost::ProductCost {
        _0: boxed_cost_list_pair(bound, body),
    })
}

pub fn max_path(paths: &[SymbolicCost]) -> SymbolicCost {
    paths
        .iter()
        .fold(SymbolicCost::ConstantCost { _0: 0 }, |acc, candidate| {
            if dominates(candidate, &acc) {
                candidate.clone()
            } else if dominates(&acc, candidate) {
                acc
            } else {
                sequential(acc, candidate.clone())
            }
        })
}

pub fn normalize(cost: SymbolicCost) -> SymbolicCost {
    match cost {
        SymbolicCost::SumCost { _0: terms } => {
            reduce_sum(drop_zero_terms(boxed_terms_to_vec(&terms)))
        }
        SymbolicCost::ProductCost { _0: terms } => {
            reduce_product(drop_zero_terms(boxed_terms_to_vec(&terms)))
        }
        other => other,
    }
}

fn boxed_cost_list_pair(a: SymbolicCost, b: SymbolicCost) -> BoxedSymbolicCostList {
    BoxedSymbolicCostList {
        first: Box::new(a),
        second: Box::new(b),
        rest: Vec::new(),
    }
}

fn boxed_terms_to_vec(terms: &BoxedSymbolicCostList) -> Vec<SymbolicCost> {
    terms.iter().map(|term| term.as_ref().clone()).collect()
}

fn drop_zero_terms(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    terms
        .into_iter()
        .filter(|t| !matches!(t, SymbolicCost::ConstantCost { _0: 0 }))
        .collect()
}

fn reduce_sum(mut terms: Vec<SymbolicCost>) -> SymbolicCost {
    terms = drop_dominated_in_sum(terms);
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        _ => SymbolicCost::SumCost {
            _0: boxed_cost_list_from_vec(terms),
        },
    }
}

fn reduce_product(terms: Vec<SymbolicCost>) -> SymbolicCost {
    match terms.len() {
        0 => SymbolicCost::ConstantCost { _0: 0 },
        1 => terms.into_iter().next().unwrap(),
        2 => {
            let mut iter = terms.into_iter();
            let a = iter.next().unwrap();
            let b = iter.next().unwrap();
            combine_binary_product(a, b)
        }
        _ => SymbolicCost::ProductCost {
            _0: boxed_cost_list_from_vec(terms),
        },
    }
}

fn boxed_cost_list_from_vec(terms: Vec<SymbolicCost>) -> BoxedSymbolicCostList {
    NonSingletonList::from_vec(terms.into_iter().map(Box::new).collect()).unwrap()
}

fn combine_binary_product(a: SymbolicCost, b: SymbolicCost) -> SymbolicCost {
    if let (SymbolicCost::LinearCost { _0: va }, SymbolicCost::LinearCost { _0: vb }) = (&a, &b) {
        if va == vb {
            return SymbolicCost::PolynomialCost {
                var: va.clone(),
                degree: DegreeAtLeastTwo::TWO,
            };
        }
    }
    SymbolicCost::ProductCost {
        _0: boxed_cost_list_pair(a, b),
    }
}

fn drop_dominated_in_sum(terms: Vec<SymbolicCost>) -> Vec<SymbolicCost> {
    let mut keep: Vec<SymbolicCost> = Vec::with_capacity(terms.len());
    for term in terms {
        let term_dominated = keep.iter().any(|k| dominates(k, &term));
        if term_dominated {
            continue;
        }
        keep.retain(|k| !dominates(&term, k));
        keep.push(term);
    }
    keep
}

pub fn dominates(a: &SymbolicCost, b: &SymbolicCost) -> bool {
    match a {
        SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::ConstantCost { .. } => matches!(b, SymbolicCost::ConstantCost { .. }),
        SymbolicCost::LinearCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost { var: _, degree: _ } => false,
            _ => false,
        },
        SymbolicCost::PolynomialCost {
            var: va,
            degree: ka,
        } => match b {
            SymbolicCost::ConstantCost { .. } | SymbolicCost::LogCost { .. } => true,
            SymbolicCost::LinearCost { _0: vb } => va == vb,
            SymbolicCost::PolynomialCost {
                var: vb,
                degree: kb,
            } => va == vb && ka.raw() >= kb.raw(),
            _ => false,
        },
        SymbolicCost::LogCost { _0: va } => match b {
            SymbolicCost::ConstantCost { .. } => true,
            SymbolicCost::LogCost { _0: vb } => va == vb,
            _ => false,
        },
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            terms.iter().any(|child| dominates(child.as_ref(), b))
        }
    }
}
"#;

const TYPE_SHAPE_TEMPLATE: &str = r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeShape {
    pub declaration: DeclarationId,
}

impl TypeShape {
    pub fn new(declaration: DeclarationId) -> Self {
        Self { declaration }
    }
}

impl From<DeclarationId> for TypeShape {
    fn from(declaration: DeclarationId) -> Self {
        Self { declaration }
    }
}
"#;

const SOURCE_SPAN_TEMPLATE: &str = r#"
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
"#;

const CORRECTION_TEMPLATE: &str = r#"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    pub description: String,
    pub span: SourceSpan,
    pub new_source: String,
}
"#;
