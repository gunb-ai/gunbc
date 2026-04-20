#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
COMPILER_DIR = ROOT / "src" / "v3" / "compiler"
SRC_DIR = COMPILER_DIR / "src"
AUTHORITY_PATH = COMPILER_DIR / "runtime_mirrors.dag"
SUBSTRATE_PATH = ROOT / "src" / "v3" / "std" / "substrate.dag"


TYPE_SHAPE_IMPL_TEMPLATE = """impl TypeShape {
    pub fn new(declaration: DeclarationId) -> Self {
        Self { declaration }
    }
}

impl From<DeclarationId> for TypeShape {
    fn from(declaration: DeclarationId) -> Self {
        Self { declaration }
    }
}
"""


SOURCE_SPAN_TEMPLATE = """#[derive(Debug, Clone, PartialEq, Eq)]
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
"""


CORRECTION_TEMPLATE = """#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    pub description: String,
    pub span: SourceSpan,
    pub new_source: String,
}
"""


DAG_COST_TEMPLATE = """#[derive(Debug, Clone, PartialEq, Eq)]
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
"""


SERIALIZE_FUNCTIONS_TEMPLATE = """pub fn serialize_dag(dag: &Dag) -> Vec<u8> {
    let mut out = String::new();
    for declaration in dag.declarations() {
        out.push_str(&format!(
            \"DECL {} {:?}\\n\",
            declaration.id.raw(),
            declaration
        ));
    }
    for behavior in dag.nodes() {
        out.push_str(&serialize_behavior(behavior));
    }
    for port in dag.ports() {
        out.push_str(&format!(\"PORT {} {:?}\\n\", port.id().raw(), port));
    }
    let mut diagnostics: Vec<_> = dag.diagnostics().iter().collect();
    diagnostics.sort_by_key(|(port, _)| port.raw());
    for (port, diagnostic) in diagnostics {
        out.push_str(&format!(
            \"DIAG port={} {}\\n\",
            port.raw(),
            render_diagnostic(diagnostic)
        ));
    }
    out.into_bytes()
}

pub fn first_difference(lhs: &Dag, rhs: &Dag) -> Option<DagDifference> {
    let lhs_decls = lhs.declarations();
    let rhs_decls = rhs.declarations();
    if lhs_decls.len() != rhs_decls.len() {
        return Some(DagDifference {
            detail: format!(
                \"declaration count mismatch: pass1={}, pass2={}\",
                lhs_decls.len(),
                rhs_decls.len()
            ),
        });
    }
    for (left, right) in lhs_decls.iter().zip(rhs_decls.iter()) {
        if format!(\"{left:?}\") != format!(\"{right:?}\") {
            let name = left
                .name
                .as_deref()
                .or(right.name.as_deref())
                .unwrap_or(\"<anonymous>\");
            return Some(DagDifference {
                detail: format!(
                    \"declaration {} `{}` diverged: pass1=`{:?}`, pass2=`{:?}`\",
                    left.id.raw(),
                    name,
                    left,
                    right
                ),
            });
        }
    }

    let lhs_nodes = lhs.nodes();
    let rhs_nodes = rhs.nodes();
    if lhs_nodes.len() != rhs_nodes.len() {
        return Some(DagDifference {
            detail: format!(
                \"behavior count mismatch: pass1={}, pass2={}\",
                lhs_nodes.len(),
                rhs_nodes.len()
            ),
        });
    }
    for (left, right) in lhs_nodes.iter().zip(rhs_nodes.iter()) {
        if format!(\"{left:?}\") != format!(\"{right:?}\") {
            return Some(DagDifference {
                detail: format!(\"behavior diverged: pass1=`{:?}`, pass2=`{:?}`\", left, right),
            });
        }
    }

    let lhs_ports = lhs.ports();
    let rhs_ports = rhs.ports();
    if lhs_ports.len() != rhs_ports.len() {
        return Some(DagDifference {
            detail: format!(
                \"port count mismatch: pass1={}, pass2={}\",
                lhs_ports.len(),
                rhs_ports.len()
            ),
        });
    }
    for (left, right) in lhs_ports.iter().zip(rhs_ports.iter()) {
        if format!(\"{left:?}\") != format!(\"{right:?}\") {
            return Some(DagDifference {
                detail: format!(\"port diverged: pass1=`{:?}`, pass2=`{:?}`\", left, right),
            });
        }
    }

    let mut lhs_diags: Vec<_> = lhs.diagnostics().iter().collect();
    let mut rhs_diags: Vec<_> = rhs.diagnostics().iter().collect();
    lhs_diags.sort_by_key(|(port, _)| port.raw());
    rhs_diags.sort_by_key(|(port, _)| port.raw());
    if lhs_diags.len() != rhs_diags.len() {
        return Some(DagDifference {
            detail: format!(
                \"diagnostic count mismatch: pass1={}, pass2={}\",
                lhs_diags.len(),
                rhs_diags.len()
            ),
        });
    }
    for ((left_port, left_diag), (right_port, right_diag)) in lhs_diags.iter().zip(rhs_diags.iter())
    {
        let left = format!(
            \"DIAG port={} {}\",
            left_port.raw(),
            render_diagnostic(left_diag)
        );
        let right = format!(
            \"DIAG port={} {}\",
            right_port.raw(),
            render_diagnostic(right_diag)
        );
        if left != right {
            return Some(DagDifference {
                detail: format!(\"diagnostic diverged: pass1=`{left}`, pass2=`{right}`\"),
            });
        }
    }

    None
}

fn serialize_behavior(behavior: &Behavior) -> String {
    format!(\"BEHAV {:?}\\n\", behavior)
}

fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    format!(\"{diagnostic:?}\")
}
"""


HEADER_TEMPLATE = "// AUTO-GENERATED from `{authority}`.\n// Regenerate instead of hand-editing.\n\n{body}\n"


@dataclass
class RecordDef:
    name: str
    fields: list[tuple[str, str]]


@dataclass
class VariantDef:
    name: str
    kind: str
    payload: str | None = None
    fields: list[tuple[str, str]] | None = None


def parse_inline_fields(source: str) -> list[tuple[str, str]]:
    source = source.strip()
    if not source:
        return []
    fields: list[tuple[str, str]] = []
    for raw_field in source.split(","):
        field = raw_field.strip()
        if not field:
            continue
        label, ty = field.split(":", 1)
        fields.append((label.strip(), ty.strip()))
    return fields


def parse_types(path: Path) -> tuple[dict[str, RecordDef], dict[str, list[VariantDef]]]:
    lines = path.read_text().splitlines()
    records: dict[str, RecordDef] = {}
    sums: dict[str, list[VariantDef]] = {}
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if not line.startswith("type "):
            i += 1
            continue
        if "{" in line and "=" not in line:
            name = re.match(r"type\s+(\w+)(?:<[^>]+>)?\s*\{", line).group(1)
            i += 1
            fields: list[tuple[str, str]] = []
            while lines[i].strip() != "}":
                field_line = lines[i].strip()
                if field_line and ":" in field_line:
                    label, ty = field_line.split(":", 1)
                    fields.append((label.strip(), ty.strip()))
                i += 1
            records[name] = RecordDef(name=name, fields=fields)
            i += 1
            continue

        name = re.match(r"type\s+(\w+)(?:<[^>]+>)?", line).group(1)
        variants: list[VariantDef] = []
        i += 1
        while i < len(lines):
            stripped = lines[i].strip()
            if not stripped or stripped.startswith("type "):
                break
            if not stripped.startswith(("=", "|")):
                i += 1
                continue
            variant_src = stripped[1:].strip()
            tuple_match = re.match(r"(\w+)\((.+)\)", variant_src)
            record_match = re.match(r"(\w+)\s*\{", variant_src)
            unit_match = re.match(r"(\w+)$", variant_src)
            if tuple_match:
                variants.append(
                    VariantDef(
                        name=tuple_match.group(1),
                        kind="tuple",
                        payload=tuple_match.group(2).strip(),
                    )
                )
                i += 1
            elif record_match:
                variant_name = record_match.group(1)
                inline_fields = re.match(r"\w+\s*\{(.+)\}\s*$", variant_src)
                if inline_fields:
                    fields = parse_inline_fields(inline_fields.group(1))
                    i += 1
                else:
                    i += 1
                    fields = []
                    while lines[i].strip() != "}":
                        field_line = lines[i].strip()
                        if field_line and ":" in field_line:
                            label, ty = field_line.split(":", 1)
                            fields.append((label.strip(), ty.strip()))
                        i += 1
                    i += 1
                variants.append(VariantDef(name=variant_name, kind="record", fields=fields))
            elif unit_match:
                variants.append(VariantDef(name=unit_match.group(1), kind="unit"))
                i += 1
            else:
                raise ValueError(f"unparsed variant line: {lines[i]}")
        sums[name] = variants
    return records, sums


def parse_runtime_mirrors() -> tuple[dict[str, RecordDef], dict[str, list[VariantDef]]]:
    return parse_types(AUTHORITY_PATH)


def rust_type(source: str, overrides: dict[str, str] | None = None) -> str:
    overrides = overrides or {}
    if source in overrides:
        return overrides[source]
    mapping = {
        "String": "String",
        "Bool": "bool",
        "Int": "i64",
        "NodeId": "NodeId",
        "PortId": "PortId",
        "ClusterId": "ClusterId",
        "DeclarationId": "DeclarationId",
        "ParamRef": "ParamRef",
        "TransformRef": "TransformRef",
        "TypeShape": "TypeShape",
        "CompilerSourceSpan": "SourceSpan",
        "CompilerCorrection": "Correction",
    }
    if source in mapping:
        return mapping[source]
    if source.endswith("?"):
        return f"Option<{rust_type(source[:-1], overrides)}>"
    generic_match = re.fullmatch(r"(\w+)<(.+)>", source)
    if generic_match:
        container = generic_match.group(1)
        inner = rust_type(generic_match.group(2), overrides)
        if container == "List":
            return f"Vec<{inner}>"
        return f"{container}<{inner}>"
    return source


def render_record(
    record: RecordDef,
    output_name: str | None = None,
    derives: str = "#[derive(Debug, Clone, PartialEq, Eq)]",
    field_name_overrides: dict[str, str] | None = None,
) -> str:
    output_name = output_name or record.name
    field_name_overrides = field_name_overrides or {}
    lines = [derives, f"pub struct {output_name} {{"]
    for label, ty in record.fields:
        output_label = field_name_overrides.get(label, label)
        lines.append(f"    pub {output_label}: {rust_type(ty)},")
    lines.append("}")
    return "\n".join(lines)


def render_sum(
    name: str,
    variants: list[VariantDef],
    derives: str,
    output_name: str | None = None,
    overrides: dict[str, str] | None = None,
    variant_name_overrides: dict[str, str] | None = None,
) -> str:
    output_name = output_name or name
    variant_name_overrides = variant_name_overrides or {}
    lines = [derives, f"pub enum {output_name} {{"]
    for variant in variants:
        variant_name = variant_name_overrides.get(variant.name, variant.name)
        if variant.kind == "unit":
            lines.append(f"    {variant_name},")
        elif variant.kind == "tuple":
            lines.append(f"    {variant_name}({rust_type(variant.payload, overrides)}),")
        elif variant.kind == "record":
            lines.append(f"    {variant_name} {{")
            for label, ty in variant.fields or []:
                lines.append(f"        {label}: {rust_type(ty, overrides)},")
            lines.append("    },")
        else:
            raise ValueError(f"unsupported variant kind {variant.kind}")
    lines.append("}")
    return "\n".join(lines)


def render_diagnostics_module(records: dict[str, RecordDef], sums: dict[str, list[VariantDef]]) -> str:
    del records, sums
    return "\n\n".join([SOURCE_SPAN_TEMPLATE.strip(), CORRECTION_TEMPLATE.strip()])


def render_types_module(substrate_records: dict[str, RecordDef]) -> str:
    return "\n\n".join(
        [
            render_record(
                substrate_records["TypeShape"],
                derives="#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]",
            ),
            TYPE_SHAPE_IMPL_TEMPLATE.strip(),
        ]
    )


def parse_surface_record_output_name(name: str) -> str:
    return name


def parse_surface_field_type(record_name: str, label: str, ty: str) -> str:
    if record_name == "SurfaceParam" and label == "refinement":
        return "Option<SurfaceExpr>"
    if record_name == "SurfaceRecordField" and label == "value":
        return "SurfaceExpr"
    if record_name == "SurfaceMatchArm" and label == "body":
        return "SurfaceExpr"
    return rust_type(ty)


def render_parse_surface_record(record: RecordDef) -> str:
    lines = ["#[derive(Debug, Clone, PartialEq, Eq)]", f"pub struct {record.name} {{"]
    for label, ty in record.fields:
        lines.append(
            f"    pub {label}: {parse_surface_field_type(record.name, label, ty)},"
        )
    lines.append("}")
    return "\n".join(lines)


def render_parse_surface_sum(name: str, variants: list[VariantDef]) -> str:
    lines = ["#[derive(Debug, Clone, PartialEq, Eq)]", f"pub enum {name} {{"]
    for variant in variants:
        if variant.kind == "unit":
            lines.append(f"    {variant.name},")
        elif variant.kind == "tuple":
            lines.append(f"    {variant.name}({rust_type(variant.payload)}),")
        elif variant.kind == "record":
            lines.append(f"    {variant.name} {{")
            for label, ty in variant.fields or []:
                rust_ty = rust_type(ty)
                if name == "SurfaceType" and label in ("inner", "output"):
                    rust_ty = f"Box<{rust_ty}>"
                elif name == "SurfaceExpr" and label in (
                    "body",
                    "cond",
                    "then_branch",
                    "else_branch",
                    "scrutinee",
                ):
                    rust_ty = f"Box<{rust_ty}>"
                lines.append(f"        {label}: {rust_ty},")
            lines.append("    },")
        else:
            raise ValueError(f"unsupported variant kind {variant.kind}")
    lines.append("}")
    return "\n".join(lines)


PARSE_SURFACE_CONVERSIONS_TEMPLATE = """
impl From<&crate::parse::SurfaceModule> for SurfaceModule {
    fn from(value: &crate::parse::SurfaceModule) -> Self {
        Self {
            items: value.items.iter().map(SurfaceItem::from).collect(),
        }
    }
}

impl From<&crate::parse::SurfaceParam> for SurfaceParam {
    fn from(value: &crate::parse::SurfaceParam) -> Self {
        Self {
            name: value.name.clone(),
            ty: SurfaceType::from(&value.ty),
            refinement: value.refinement.as_ref().map(SurfaceExpr::from),
        }
    }
}

impl From<&crate::parse::SurfaceField> for SurfaceField {
    fn from(value: &crate::parse::SurfaceField) -> Self {
        Self {
            name: value.name.clone(),
            ty: SurfaceType::from(&value.ty),
        }
    }
}

impl From<&crate::parse::SurfaceVariant> for SurfaceVariant {
    fn from(value: &crate::parse::SurfaceVariant) -> Self {
        Self {
            name: value.name.clone(),
            payload: VariantPayload::from(&value.payload),
            span: value.span.clone(),
        }
    }
}

impl From<&crate::parse::VariantPayload> for VariantPayload {
    fn from(value: &crate::parse::VariantPayload) -> Self {
        match value {
            crate::parse::VariantPayload::Positional(types) => {
                Self::Positional(types.iter().map(SurfaceType::from).collect())
            }
            crate::parse::VariantPayload::Record(fields) => {
                Self::Record(fields.iter().map(SurfaceField::from).collect())
            }
        }
    }
}

impl From<&crate::parse::SurfaceType> for SurfaceType {
    fn from(value: &crate::parse::SurfaceType) -> Self {
        match value {
            crate::parse::SurfaceType::Named { name, span } => Self::Named {
                name: name.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceType::Parameterized { name, args, span } => Self::Parameterized {
                name: name.clone(),
                args: args.iter().map(SurfaceType::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceType::Optional { inner, span } => Self::Optional {
                inner: Box::new(SurfaceType::from(inner.as_ref())),
                span: span.clone(),
            },
            crate::parse::SurfaceType::Arrow {
                inputs,
                output,
                span,
            } => Self::Arrow {
                inputs: inputs.iter().map(SurfaceType::from).collect(),
                output: Box::new(SurfaceType::from(output.as_ref())),
                span: span.clone(),
            },
        }
    }
}

impl From<&crate::parse::SurfaceRecordField> for SurfaceRecordField {
    fn from(value: &crate::parse::SurfaceRecordField) -> Self {
        Self {
            name: value.name.clone(),
            value: SurfaceExpr::from(&value.value),
            span: value.span.clone(),
        }
    }
}

impl From<&crate::parse::SurfaceMatchArm> for SurfaceMatchArm {
    fn from(value: &crate::parse::SurfaceMatchArm) -> Self {
        Self {
            pattern: SurfacePattern::from(&value.pattern),
            body: SurfaceExpr::from(&value.body),
            span: value.span.clone(),
        }
    }
}

impl From<&crate::parse::SurfacePatternField> for SurfacePatternField {
    fn from(value: &crate::parse::SurfacePatternField) -> Self {
        Self {
            name: value.name.clone(),
            binding: value.binding.clone(),
            span: value.span.clone(),
        }
    }
}

impl From<&crate::parse::SurfacePattern> for SurfacePattern {
    fn from(value: &crate::parse::SurfacePattern) -> Self {
        match value {
            crate::parse::SurfacePattern::BareVariant { name, span } => Self::BareVariant {
                name: name.clone(),
                span: span.clone(),
            },
            crate::parse::SurfacePattern::VariantWith {
                name,
                binding,
                span,
            } => Self::VariantWith {
                name: name.clone(),
                binding: binding.clone(),
                span: span.clone(),
            },
            crate::parse::SurfacePattern::VariantFields { name, fields, span } => {
                Self::VariantFields {
                    name: name.clone(),
                    fields: fields.iter().map(SurfacePatternField::from).collect(),
                    span: span.clone(),
                }
            }
        }
    }
}

impl From<&crate::parse::SurfaceLiteral> for SurfaceLiteral {
    fn from(value: &crate::parse::SurfaceLiteral) -> Self {
        match value {
            crate::parse::SurfaceLiteral::Int(value) => Self::Int(*value),
            crate::parse::SurfaceLiteral::Bool(value) => Self::Bool(*value),
            crate::parse::SurfaceLiteral::String(value) => Self::String(value.clone()),
        }
    }
}

impl From<&crate::parse::SurfaceExpr> for SurfaceExpr {
    fn from(value: &crate::parse::SurfaceExpr) -> Self {
        match value {
            crate::parse::SurfaceExpr::Literal { value, span } => Self::Literal {
                value: SurfaceLiteral::from(value),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::Var { name, span } => Self::Var {
                name: name.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::Path {
                segments,
                segment_spans,
                span,
            } => Self::Path {
                segments: segments.clone(),
                segment_spans: segment_spans.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::Call { target, args, span } => Self::Call {
                target: target.clone(),
                args: args.iter().map(SurfaceExpr::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::VariantRecord { target, fields, span } => {
                Self::VariantRecord {
                    target: target.clone(),
                    fields: fields.iter().map(SurfaceRecordField::from).collect(),
                    span: span.clone(),
                }
            }
            crate::parse::SurfaceExpr::Operator { op, args, span } => Self::Operator {
                op: op.clone(),
                args: args.iter().map(SurfaceExpr::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::Lambda { params, body, span } => Self::Lambda {
                params: params.clone(),
                body: Box::new(SurfaceExpr::from(body.as_ref())),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => Self::If {
                cond: Box::new(SurfaceExpr::from(cond.as_ref())),
                then_branch: Box::new(SurfaceExpr::from(then_branch.as_ref())),
                else_branch: Box::new(SurfaceExpr::from(else_branch.as_ref())),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::Match {
                scrutinee,
                arms,
                span,
            } => Self::Match {
                scrutinee: Box::new(SurfaceExpr::from(scrutinee.as_ref())),
                arms: arms.iter().map(SurfaceMatchArm::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::Record { fields, span } => Self::Record {
                fields: fields.iter().map(SurfaceRecordField::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceExpr::List { elements, span } => Self::List {
                elements: elements.iter().map(SurfaceExpr::from).collect(),
                span: span.clone(),
            },
        }
    }
}

impl From<&crate::parse::SurfaceItem> for SurfaceItem {
    fn from(value: &crate::parse::SurfaceItem) -> Self {
        match value {
            crate::parse::SurfaceItem::Let {
                name,
                type_ann,
                expr,
            } => Self::Let {
                name: name.clone(),
                type_ann: type_ann.as_ref().map(SurfaceType::from),
                expr: SurfaceExpr::from(expr),
            },
            crate::parse::SurfaceItem::Fn {
                name,
                type_params,
                params,
                return_type,
                body,
                span,
            } => Self::Fn {
                name: name.clone(),
                type_params: type_params.clone(),
                params: params.iter().map(SurfaceParam::from).collect(),
                return_type: SurfaceType::from(return_type),
                body: SurfaceExpr::from(body),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::FnExternalBody {
                name,
                type_params,
                params,
                return_type,
                body_span,
                span,
            } => Self::FnExternalBody {
                name: name.clone(),
                type_params: type_params.clone(),
                params: params.iter().map(SurfaceParam::from).collect(),
                return_type: SurfaceType::from(return_type),
                body_span: body_span.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::Data {
                name,
                ty,
                body,
                body_span,
                span,
            } => Self::Data {
                name: name.clone(),
                ty: SurfaceType::from(ty),
                body: body.as_ref().map(SurfaceExpr::from),
                body_span: body_span.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::Module { path, span } => Self::Module {
                path: path.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::Import { path, names, span } => Self::Import {
                path: path.clone(),
                names: names.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::TypeAtom {
                name,
                type_params,
                span,
            } => Self::TypeAtom {
                name: name.clone(),
                type_params: type_params.clone(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::TypeRecord {
                name,
                type_params,
                fields,
                span,
            } => Self::TypeRecord {
                name: name.clone(),
                type_params: type_params.clone(),
                fields: fields.iter().map(SurfaceField::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::TypeSum {
                name,
                type_params,
                variants,
                span,
            } => Self::TypeSum {
                name: name.clone(),
                type_params: type_params.clone(),
                variants: variants.iter().map(SurfaceVariant::from).collect(),
                span: span.clone(),
            },
            crate::parse::SurfaceItem::TypeAlias {
                name,
                type_params,
                target,
                span,
            } => Self::TypeAlias {
                name: name.clone(),
                type_params: type_params.clone(),
                target: SurfaceType::from(target),
                span: span.clone(),
            },
        }
    }
}
"""


def render_parse_surface_module(records: dict[str, RecordDef], sums: dict[str, list[VariantDef]]) -> str:
    parts = [
        render_parse_surface_record(records["SurfaceModule"]),
        render_parse_surface_record(records["SurfaceParam"]),
        render_parse_surface_record(records["SurfaceField"]),
        render_parse_surface_record(records["SurfaceVariant"]),
        render_parse_surface_record(records["SurfaceRecordField"]),
        render_parse_surface_record(records["SurfaceMatchArm"]),
        render_parse_surface_record(records["SurfacePatternField"]),
        render_parse_surface_sum("VariantPayload", sums["VariantPayload"]),
        render_parse_surface_sum("SurfaceType", sums["SurfaceType"]),
        render_parse_surface_sum("SurfacePattern", sums["SurfacePattern"]),
        render_parse_surface_sum("SurfaceLiteral", sums["SurfaceLiteral"]),
        render_parse_surface_sum("SurfaceExpr", sums["SurfaceExpr"]),
        render_parse_surface_sum("SurfaceItem", sums["SurfaceItem"]),
        PARSE_SURFACE_CONVERSIONS_TEMPLATE.strip(),
    ]
    return "\n\n".join(parts)


def render_dag_scalar_module(records: dict[str, RecordDef], sums: dict[str, list[VariantDef]]) -> str:
    parts = [
        render_sum(
            "LiteralBits",
            sums["LiteralBits"],
            "#[derive(Debug, Clone, PartialEq, Eq)]",
            output_name="LiteralBits",
            variant_name_overrides={"LitInt": "Int", "LitBool": "Bool", "LitString": "String"},
        ),
        render_sum(
            "CardinalityBound",
            sums["CardinalityBound"],
            "#[derive(Debug, Clone, PartialEq, Eq)]",
            output_name="CardinalityBound",
            overrides={"Int": "u32"},
        ),
        render_record(
            records["TemplateArgument"],
            output_name="TemplateArgument",
            derives="#[derive(Debug, Clone)]",
        ),
        render_sum(
            "PortState",
            sums["PortState"],
            "#[derive(Debug, Clone, PartialEq, Eq)]",
            output_name="PortState",
        ),
    ]
    return "\n\n".join(parts)


def render_dag_branch_module(records: dict[str, RecordDef], sums: dict[str, list[VariantDef]]) -> str:
    parts = [
        render_sum(
            "BranchPattern",
            sums["BranchPattern"],
            "#[derive(Debug, Clone)]",
            output_name="BranchPattern",
        ),
        render_record(
            records["PayloadBinding"],
            output_name="PayloadBinding",
            derives="#[derive(Debug, Clone)]",
        ),
        render_record(
            records["BranchPath"],
            output_name="Path",
            derives="#[derive(Debug, Clone)]",
            field_name_overrides={"result_port": "output"},
        ),
    ]
    return "\n\n".join(parts)


def render_dag_cluster_module(records: dict[str, RecordDef], sums: dict[str, list[VariantDef]]) -> str:
    parts = [
        render_record(
            records["MemberDescent"],
            output_name="MemberDescent",
        ),
        render_record(
            records["IntraClusterCall"],
            output_name="IntraClusterCall",
        ),
        render_record(records["Cluster"], output_name="Cluster"),
        render_sum(
            "LoopBound",
            sums["LoopBound"],
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
            output_name="LoopBound",
        ),
    ]
    return "\n\n".join(parts)


def format_with_header(authority: str, body: str) -> str:
    return HEADER_TEMPLATE.format(authority=authority, body=body.rstrip())


def expected_outputs() -> dict[Path, str]:
    records, sums = parse_runtime_mirrors()
    substrate_records, _ = parse_types(SUBSTRATE_PATH)
    _, substrate_sums = parse_types(SUBSTRATE_PATH)
    return {
        SRC_DIR / "types_generated.rs": format_with_header(
            "src/v3/std/substrate.dag", render_types_module(substrate_records)
        ),
        SRC_DIR / "diagnostics_generated.rs": format_with_header(
            "src/v3/std/substrate.dag, src/v3/std/diagnostics.dag",
            render_diagnostics_module(records, sums),
        ),
        SRC_DIR / "serialize_generated.rs": format_with_header(
            "src/v3/compiler/runtime_mirrors.dag",
            render_record(records["DagDifference"]) + "\n\n" + SERIALIZE_FUNCTIONS_TEMPLATE.rstrip(),
        ),
        SRC_DIR / "parse_surface_generated.rs": format_with_header(
            "src/v3/compiler/runtime_mirrors.dag",
            render_parse_surface_module(records, sums),
        ),
        SRC_DIR / "dag_scalar_generated.rs": format_with_header(
            "src/v3/std/substrate.dag",
            render_dag_scalar_module(substrate_records, substrate_sums),
        ),
        SRC_DIR / "dag_branch_generated.rs": format_with_header(
            "src/v3/std/substrate.dag",
            render_dag_branch_module(substrate_records, substrate_sums),
        ),
        SRC_DIR / "dag_cluster_generated.rs": format_with_header(
            "src/v3/std/substrate.dag",
            render_dag_cluster_module(substrate_records, substrate_sums),
        ),
        SRC_DIR / "dag_cost_generated.rs": format_with_header(
            "src/v3/std/algebra.dag", DAG_COST_TEMPLATE
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="fail if generated files are stale")
    args = parser.parse_args()

    outputs = expected_outputs()
    stale = False
    for path, expected in outputs.items():
        if args.check:
            actual = path.read_text()
            if actual != expected:
                print(f"stale: {path.relative_to(ROOT)}", file=sys.stderr)
                stale = True
        else:
            path.write_text(expected)
            print(f"wrote {path}")
    return 1 if stale else 0


if __name__ == "__main__":
    raise SystemExit(main())
