// AUTO-GENERATED from `src/v3/std/parse_surface.dag`.
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
        inhabits: Option<SurfaceType>,
        span: SourceSpan,
    },
    TypeAlias {
        name: String,
        type_params: Vec<String>,
        target: SurfaceType,
        refinement: Option<SurfaceExpr>,
        span: SourceSpan,
    },
}
