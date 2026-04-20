// AUTO-GENERATED from `src/v3/compiler/runtime_mirrors.dag`.
// Regenerate instead of hand-editing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceModule {
    pub items: Vec<SurfaceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceParam {
    pub name: String,
    pub ty: SurfaceType,
    pub refinement: Option<SurfaceExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceField {
    pub name: String,
    pub ty: SurfaceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceVariant {
    pub name: String,
    pub payload: VariantPayload,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceRecordField {
    pub name: String,
    pub value: SurfaceExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceMatchArm {
    pub pattern: SurfacePattern,
    pub body: SurfaceExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfacePatternField {
    pub name: String,
    pub binding: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantPayload {
    Positional(Vec<SurfaceType>),
    Record(Vec<SurfaceField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceType {
    Named {
        name: String,
        span: SourceSpan,
    },
    Parameterized {
        name: String,
        args: Vec<SurfaceType>,
        span: SourceSpan,
    },
    Optional {
        inner: Box<SurfaceType>,
        span: SourceSpan,
    },
    Arrow {
        inputs: Vec<SurfaceType>,
        output: Box<SurfaceType>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfacePattern {
    BareVariant {
        name: String,
        span: SourceSpan,
    },
    VariantWith {
        name: String,
        binding: String,
        span: SourceSpan,
    },
    VariantFields {
        name: String,
        fields: Vec<SurfacePatternField>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceLiteral {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceExpr {
    Literal {
        value: SurfaceLiteral,
        span: SourceSpan,
    },
    Var {
        name: String,
        span: SourceSpan,
    },
    Path {
        segments: Vec<String>,
        segment_spans: Vec<SourceSpan>,
        span: SourceSpan,
    },
    Call {
        target: String,
        args: Vec<SurfaceExpr>,
        span: SourceSpan,
    },
    VariantRecord {
        target: String,
        fields: Vec<SurfaceRecordField>,
        span: SourceSpan,
    },
    Operator {
        op: OperatorKind,
        args: Vec<SurfaceExpr>,
        span: SourceSpan,
    },
    Lambda {
        params: Vec<String>,
        body: Box<SurfaceExpr>,
        span: SourceSpan,
    },
    If {
        cond: Box<SurfaceExpr>,
        then_branch: Box<SurfaceExpr>,
        else_branch: Box<SurfaceExpr>,
        span: SourceSpan,
    },
    Match {
        scrutinee: Box<SurfaceExpr>,
        arms: Vec<SurfaceMatchArm>,
        span: SourceSpan,
    },
    Record {
        fields: Vec<SurfaceRecordField>,
        span: SourceSpan,
    },
    List {
        elements: Vec<SurfaceExpr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceItem {
    Let {
        name: String,
        type_ann: Option<SurfaceType>,
        expr: SurfaceExpr,
    },
    Fn {
        name: String,
        type_params: Vec<String>,
        params: Vec<SurfaceParam>,
        return_type: SurfaceType,
        body: SurfaceExpr,
        span: SourceSpan,
    },
    FnExternalBody {
        name: String,
        type_params: Vec<String>,
        params: Vec<SurfaceParam>,
        return_type: SurfaceType,
        body_span: SourceSpan,
        span: SourceSpan,
    },
    Data {
        name: String,
        ty: SurfaceType,
        body: Option<SurfaceExpr>,
        body_span: SourceSpan,
        span: SourceSpan,
    },
    Module {
        path: Vec<String>,
        span: SourceSpan,
    },
    Import {
        path: Vec<String>,
        names: Vec<String>,
        span: SourceSpan,
    },
    TypeAtom {
        name: String,
        type_params: Vec<String>,
        span: SourceSpan,
    },
    TypeRecord {
        name: String,
        type_params: Vec<String>,
        fields: Vec<SurfaceField>,
        span: SourceSpan,
    },
    TypeSum {
        name: String,
        type_params: Vec<String>,
        variants: Vec<SurfaceVariant>,
        span: SourceSpan,
    },
    TypeAlias {
        name: String,
        type_params: Vec<String>,
        target: SurfaceType,
        span: SourceSpan,
    },
}

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
                op: *op,
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
