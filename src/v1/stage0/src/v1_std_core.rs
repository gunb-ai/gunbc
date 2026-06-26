use self::CallSemantics::*;
use self::Cardinality::*;
use self::CompilerDiagnostic::*;
use self::Connective::*;
use self::ExprData::*;
use self::ExprErrorKind::*;
use self::FieldAccessStyle::*;
use self::FieldValueShape::*;
use self::FunctionSizeEffect::*;
use self::InferredNode::*;
use self::MatchPattern::*;
use self::MethodSemantics::*;
use self::NodeFieldRole::*;
use self::OperationModifier::*;
use self::StringPart::*;
use self::TokenShape::*;
use self::UnaryOpKind::*;
use self::VarBindingKind::*;
use crate::std_algebra::CollectionSizeEffect::*;
use crate::std_algebra::CostShape::*;
pub use crate::std_algebra::{AlgebraFieldTemplate, CollectionSizeEffect, CostShape};
pub use crate::std_induction::SubValueRelation;
use crate::std_induction::SubValueRelation::*;
use crate::std_syntax::AlgebraFieldKind::{
    AlgAdd, AlgCompare, AlgJoin, AlgMeet, AlgMul, AlgQuotient, AlgReciprocal, AlgRemainder,
};
use crate::std_syntax::BinOp::{
    Add, And, Div, Eq, Ge, Gt, Le, Lt, Mod, Mul, Ne, NullCoalesce, Or, Sub,
};
use crate::std_syntax::LiteralValue::{LitBool, LitFloat, LitInt, LitNull, LitStr, LitSymbol};
pub use crate::std_syntax::{AlgebraFieldKind, BinOp, LiteralValue};
pub use crate::std_types::{
    container_expected_arity, container_type_arity, is_container_type, is_kernel_type,
    kernel_type_set,
};
pub use crate::std_types::{FilePath, NonEmptyStr, SourceSpan};
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub text: String,
    pub span: Rc<SourceSpan>,
    pub shape: TokenShape,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum TokenShape {
    ShKeyword,
    ShLBrace,
    ShRBrace,
    ShLParen,
    ShRParen,
    ShLBracket,
    ShRBracket,
    ShLt,
    ShGt,
    ShLe,
    ShGe,
    ShFatArrow,
    ShArrow,
    ShColon,
    ShComma,
    ShDot,
    ShDotDot,
    ShEq,
    ShEqEq,
    ShNe,
    ShPlus,
    ShMinus,
    ShStar,
    ShSlash,
    ShPercent,
    ShBang,
    ShAnd,
    ShOr,
    ShQuestion,
    ShNullCoalesce,
    ShCaret,
    ShPipe,
    ShPipeArrow,
    ShLitStr,
    ShLitInt,
    ShLitFloat,
    ShIdent,
    ShStrBegin,
    ShStrMid,
    ShStrEnd,
    ShNewline,
    ShEof,
    ShUnknown,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum Connective {
    Conj,
    Disj,
    NoConnective,
    Arrow,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum Cardinality {
    Required,
    CardOptional,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum FieldAccessStyle {
    StoredField,
    EnumAccessor,
    OptionalUnwrap,
    TupleFirst,
    TupleSecond,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum FieldValueShape {
    PlainValue,
    OptionalValue,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldSummary {
    pub access_style: FieldAccessStyle,
    pub value_shape: FieldValueShape,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum InferredNode {
    Resolved {
        node: Rc<Node>,
    },
    CompilerError {
        message: String,
        span: Rc<SourceSpan>,
    },
    TypeVariable {
        id: String,
    },
}

pub fn inferred_to_node(inferred: Rc<InferredNode>) -> Option<Rc<Node>> {
    match (*inferred).clone() {
        InferredNode::Resolved { node: n, .. } => Some(n.clone()),
        InferredNode::CompilerError { .. } => None,
        InferredNode::TypeVariable { id: _, .. } => None,
    }
}

pub fn is_compiler_error(inferred: Rc<InferredNode>) -> bool {
    match (*inferred).clone() {
        InferredNode::Resolved { node: _, .. } => false,
        InferredNode::CompilerError { .. } => true,
        InferredNode::TypeVariable { id: _, .. } => false,
    }
}

pub fn has_inferred(n: Rc<Node>) -> bool {
    (n.inferred.clone() != None)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum VarBindingKind {
    LocalValueBinding,
    FunctionValueBinding,
    VariantValueBinding { parent_enum: String },
    MatchBoundBinding,
}
impl VarBindingKind {
    pub fn parent_enum(&self) -> String {
        match self {
            VarBindingKind::LocalValueBinding => panic!("no parent_enum on unit variant"),
            VarBindingKind::FunctionValueBinding => panic!("no parent_enum on unit variant"),
            VarBindingKind::VariantValueBinding {
                parent_enum: __val, ..
            } => __val.clone(),
            VarBindingKind::MatchBoundBinding => panic!("no parent_enum on unit variant"),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum CallSemantics {
    PlainCallSemantics,
    LookupCallSemantics,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum MethodSemantics {
    PlainMethodSemantics,
    AlgebraMethodSemantics {
        method_def: Rc<Node>,
        fold_accumulator_type: Option<Rc<Node>>,
        size_effect: Option<CollectionSizeEffect>,
        cost_shape: Option<CostShape>,
        algebra_template: Option<Rc<AlgebraFieldTemplate>>,
    },
    ServiceMethodSemantics {
        service_name: String,
        op_params: Rc<Vec<Rc<Node>>>,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum ExprErrorKind {
    ParseRecoveryError,
    SemanticExprError,
    InternalExprError,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum ExprData {
    NoExprData,
    ExprLiteral {
        value: Rc<LiteralValue>,
    },
    ExprError {
        kind: ExprErrorKind,
        message: String,
    },
    ExprVar {
        binding_kind: Option<Rc<VarBindingKind>>,
    },
    ExprFieldAccess {
        summary: Option<Rc<FieldSummary>>,
    },
    ExprCall {
        call_semantics: Option<CallSemantics>,
        descent_evidence: Option<Rc<Vec<Rc<SubValueRelation>>>>,
    },
    ExprMethodCall {
        method_semantics: Option<Rc<MethodSemantics>>,
    },
    ExprMatch,
    ExprIf,
    ExprLet,
    ExprRecordLit {
        parent_enum: Option<String>,
    },
    ExprListLit,
    ExprBinOp {
        op: BinOp,
        algebra_field: Option<AlgebraFieldKind>,
    },
    ExprUnaryOp {
        op: UnaryOpKind,
    },
    ExprLambda,
    ExprStringInterp,
    ExprBlock,
    ExprCast,
    ExprForEach,
    ExprIndex,
    ExprSlice,
    ExprReturn,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum MatchPattern {
    Bind {
        name: String,
    },
    LitPattern {
        value: Rc<LiteralValue>,
    },
    VariantPattern {
        name: String,
        parent_enum: Option<String>,
        field_bindings: Rc<Vec<Rc<Node>>>,
    },
    Wildcard,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum UnaryOpKind {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum StringPart {
    Text { value: String },
    Interpolation { expr: Rc<Node> },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum OperationModifier {
    Idempotent,
    Readonly,
    Hermetic,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompileResult {
    pub files: Rc<Vec<Rc<TextFile>>>,
    pub diagnostics: Rc<Vec<Rc<ErrorNode>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextFile {
    pub path: FilePath,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum CompilerDiagnostic {
    UnresolvedImport {
        module_path: String,
        importing_module: String,
        span: Rc<SourceSpan>,
    },
    MissingExport {
        name: String,
        module_path: String,
        importing_module: String,
        span: Rc<SourceSpan>,
    },
    UnresolvedType {
        name: String,
        span: Rc<SourceSpan>,
    },
    TypeMismatch {
        expected: String,
        got: String,
        span: Rc<SourceSpan>,
    },
    ArityMismatch {
        name: String,
        expected: i64,
        got: i64,
        span: Rc<SourceSpan>,
    },
    VariantNotFound {
        variant: String,
        type_name: String,
        span: Rc<SourceSpan>,
    },
    FieldNotFound {
        field: String,
        type_name: String,
        span: Rc<SourceSpan>,
    },
    NonExhaustiveMatch {
        missing: Rc<Vec<String>>,
        span: Rc<SourceSpan>,
    },
    CircularDependency {
        modules: Rc<Vec<String>>,
        span: Rc<SourceSpan>,
    },
    DuplicateModule {
        name: String,
        span: Rc<SourceSpan>,
    },
    MissingAnnotation {
        fn_name: String,
        what: String,
        span: Rc<SourceSpan>,
    },
    ParseError {
        message: String,
        span: Rc<SourceSpan>,
    },
    InternalError {
        message: String,
        span: Rc<SourceSpan>,
    },
    ComplexityUnknown {
        func_name: String,
        reason: String,
        span: Rc<SourceSpan>,
    },
    OwnershipViolation {
        binding: String,
        fn_name: String,
        consumers: i64,
        span: Rc<SourceSpan>,
    },
    VariantCollision {
        variant: String,
        enum1: String,
        enum2: String,
        span: Rc<SourceSpan>,
    },
    SoleConstructorViolation {
        type_name: String,
        span: Rc<SourceSpan>,
    },
}
impl CompilerDiagnostic {
    pub fn span(&self) -> Rc<SourceSpan> {
        match self {
            CompilerDiagnostic::UnresolvedImport { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::MissingExport { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::UnresolvedType { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::TypeMismatch { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::ArityMismatch { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::VariantNotFound { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::FieldNotFound { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::NonExhaustiveMatch { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::CircularDependency { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::DuplicateModule { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::MissingAnnotation { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::ParseError { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::InternalError { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::ComplexityUnknown { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::OwnershipViolation { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::VariantCollision { span: __val, .. } => __val.clone(),
            CompilerDiagnostic::SoleConstructorViolation { span: __val, .. } => __val.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ErrorNode {
    pub diagnostic: Rc<CompilerDiagnostic>,
    pub module_name: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ErrorDAG {
    pub errors: Rc<Vec<Rc<ErrorNode>>>,
}

pub fn diagnostic_to_span(d: Rc<CompilerDiagnostic>) -> Rc<SourceSpan> {
    match (*d).clone() {
        CompilerDiagnostic::UnresolvedImport { span: s, .. } => s.clone(),
        CompilerDiagnostic::MissingExport { span: s, .. } => s.clone(),
        CompilerDiagnostic::UnresolvedType { span: s, .. } => s.clone(),
        CompilerDiagnostic::TypeMismatch { span: s, .. } => s.clone(),
        CompilerDiagnostic::ArityMismatch { span: s, .. } => s.clone(),
        CompilerDiagnostic::VariantNotFound { span: s, .. } => s.clone(),
        CompilerDiagnostic::FieldNotFound { span: s, .. } => s.clone(),
        CompilerDiagnostic::NonExhaustiveMatch { span: s, .. } => s.clone(),
        CompilerDiagnostic::CircularDependency { span: s, .. } => s.clone(),
        CompilerDiagnostic::DuplicateModule { span: s, .. } => s.clone(),
        CompilerDiagnostic::MissingAnnotation { span: s, .. } => s.clone(),
        CompilerDiagnostic::ParseError { span: s, .. } => s.clone(),
        CompilerDiagnostic::InternalError { span: s, .. } => s.clone(),
        CompilerDiagnostic::ComplexityUnknown { span: s, .. } => s.clone(),
        CompilerDiagnostic::OwnershipViolation { span: s, .. } => s.clone(),
        CompilerDiagnostic::VariantCollision { span: s, .. } => s.clone(),
        CompilerDiagnostic::SoleConstructorViolation { span: s, .. } => s.clone(),
    }
}

pub fn diagnostic_to_message(d: Rc<CompilerDiagnostic>) -> String {
    match (*d).clone() {
        CompilerDiagnostic::UnresolvedImport {
            module_path: m,
            importing_module: i,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat("unresolved import: module '".to_string(), m.clone()),
                    "' not found (imported by '".to_string(),
                ),
                i.clone(),
            ),
            "')".to_string(),
        ),
        CompilerDiagnostic::MissingExport {
            name: n,
            module_path: m,
            importing_module: i,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(
                        v1_rt::concat(
                            v1_rt::concat("name '".to_string(), n.clone()),
                            "' not found in module '".to_string(),
                        ),
                        m.clone(),
                    ),
                    "' (imported by '".to_string(),
                ),
                i.clone(),
            ),
            "')".to_string(),
        ),
        CompilerDiagnostic::UnresolvedType { name: n, .. } => v1_rt::concat(
            v1_rt::concat("unresolved type '".to_string(), n.clone()),
            "'".to_string(),
        ),
        CompilerDiagnostic::TypeMismatch {
            expected: e,
            got: g,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat("type mismatch: expected '".to_string(), e.clone()),
                    "', got '".to_string(),
                ),
                g.clone(),
            ),
            "'".to_string(),
        ),
        CompilerDiagnostic::ArityMismatch {
            name: n,
            expected: e,
            got: g,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(
                        v1_rt::concat("type ".to_string(), n.clone()),
                        " expects ".to_string(),
                    ),
                    (e.clone()).to_string(),
                ),
                " type arguments, got ".to_string(),
            ),
            (g.clone()).to_string(),
        ),
        CompilerDiagnostic::VariantNotFound {
            variant: v,
            type_name: t,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat("variant '".to_string(), v.clone()),
                    "' not found in type '".to_string(),
                ),
                t.clone(),
            ),
            "'".to_string(),
        ),
        CompilerDiagnostic::FieldNotFound {
            field: f,
            type_name: t,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat("field '".to_string(), f.clone()),
                    "' not found in type '".to_string(),
                ),
                t.clone(),
            ),
            "'".to_string(),
        ),
        CompilerDiagnostic::NonExhaustiveMatch { missing: ms, .. } => v1_rt::concat(
            "non-exhaustive match: missing variant(s) ".to_string(),
            ms.clone().join(&", ".to_string()),
        ),
        CompilerDiagnostic::CircularDependency { modules: ms, .. } => v1_rt::concat(
            "circular dependency detected: ".to_string(),
            ms.clone().join(&" -> ".to_string()),
        ),
        CompilerDiagnostic::DuplicateModule { name: n, .. } => v1_rt::concat(
            v1_rt::concat("duplicate module declaration: '".to_string(), n.clone()),
            "'".to_string(),
        ),
        CompilerDiagnostic::MissingAnnotation {
            fn_name: f,
            what: w,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat("function '".to_string(), f.clone()),
                    "' requires ".to_string(),
                ),
                w.clone(),
            ),
            " annotation".to_string(),
        ),
        CompilerDiagnostic::ParseError { message: m, .. } => m.clone(),
        CompilerDiagnostic::InternalError { message: m, .. } => m.clone(),
        CompilerDiagnostic::ComplexityUnknown {
            func_name: f,
            reason: r,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat("complexity: ".to_string(), f.clone()),
                ": ".to_string(),
            ),
            r.clone(),
        ),
        CompilerDiagnostic::OwnershipViolation {
            binding: b,
            fn_name: f,
            consumers: c,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(
                        v1_rt::concat(
                            v1_rt::concat("ownership: binding '".to_string(), b.clone()),
                            "' in '".to_string(),
                        ),
                        f.clone(),
                    ),
                    "' has ".to_string(),
                ),
                (c.clone()).to_string(),
            ),
            " consumers".to_string(),
        ),
        CompilerDiagnostic::SoleConstructorViolation { type_name: t, .. } => v1_rt::concat(
            v1_rt::concat("sole_constructor type '".to_string(), t.clone()),
            "' cannot be constructed outside its defining module".to_string(),
        ),
        CompilerDiagnostic::VariantCollision {
            variant: v,
            enum1: e1,
            enum2: e2,
            ..
        } => v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(
                        v1_rt::concat(
                            v1_rt::concat("variant '".to_string(), v.clone()),
                            "' appears in both '".to_string(),
                        ),
                        e1.clone(),
                    ),
                    "' and '".to_string(),
                ),
                e2.clone(),
            ),
            "'".to_string(),
        ),
    }
}

pub fn is_error_diagnostic(d: Rc<CompilerDiagnostic>) -> bool {
    true
}

pub fn is_interpreter_blocking_diagnostic(d: Rc<CompilerDiagnostic>) -> bool {
    match (*d).clone() {
        CompilerDiagnostic::ComplexityUnknown { .. } => false,
        _ => true,
    }
}

pub fn is_discovery_corpus_advisory_typecheck_diagnostic(d: Rc<CompilerDiagnostic>) -> bool {
    match (*d).clone() {
        CompilerDiagnostic::UnresolvedType { .. } => true,
        CompilerDiagnostic::TypeMismatch { .. } => true,
        CompilerDiagnostic::ArityMismatch { .. } => true,
        CompilerDiagnostic::VariantNotFound { .. } => true,
        CompilerDiagnostic::FieldNotFound { .. } => true,
        CompilerDiagnostic::NonExhaustiveMatch { .. } => true,
        CompilerDiagnostic::MissingAnnotation { .. } => true,
        CompilerDiagnostic::VariantCollision { .. } => true,
        CompilerDiagnostic::SoleConstructorViolation { .. } => true,
        _ => false,
    }
}

pub fn is_discovery_corpus_blocking_diagnostic(d: Rc<CompilerDiagnostic>) -> bool {
    if !is_interpreter_blocking_diagnostic(d.clone()) {
        false
    } else if is_discovery_corpus_advisory_typecheck_diagnostic(d.clone()) {
        false
    } else {
        true
    }
}

pub fn make_error_node(diagnostic: Rc<CompilerDiagnostic>, module_name: String) -> Rc<ErrorNode> {
    Rc::new(ErrorNode {
        diagnostic: diagnostic,
        module_name: module_name,
    })
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeclaredFuncSig {
    pub name: String,
    pub params: Rc<Vec<Rc<Node>>>,
    pub inferred: Option<Rc<Node>>,
    pub is_async: bool,
    pub output_provenance: Rc<Vec<Rc<HashMap<String, Rc<SubValueRelation>>>>>,
    pub variant_provenance:
        Rc<HashMap<String, Rc<HashMap<String, Rc<HashMap<String, Rc<SubValueRelation>>>>>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeclaredFuncEnv {
    pub signatures: Rc<HashMap<String, Rc<DeclaredFuncSig>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub name: String,
    pub ident: Option<i64>,
    pub span: Rc<SourceSpan>,
    pub ident_span: Option<Rc<SourceSpan>>,
    pub children: Rc<Vec<Rc<Node>>>,
    pub connective: Connective,
    pub params: Rc<Vec<Rc<Node>>>,
    pub inferred: Option<Rc<InferredNode>>,
    pub return_cardinality: Cardinality,
    pub uses: Rc<Vec<Rc<Node>>>,
    pub body: Option<Rc<Node>>,
    pub transport: Option<Rc<Node>>,
    pub properties: Rc<Vec<Rc<Node>>>,
    pub type_annotation: Option<Rc<Node>>,
    pub is_self_recursive: bool,
    pub has_non_tail_self_call: bool,
    pub match_pattern: Option<Rc<MatchPattern>>,
    pub expr_data: Rc<ExprData>,
}

pub fn default_ident_span(name: String, span: Rc<SourceSpan>) -> Option<Rc<SourceSpan>> {
    if (name.as_str() == "".to_string().as_str()) {
        None
    } else {
        Some(span)
    }
}

pub fn node_name_span(n: Rc<Node>) -> Rc<SourceSpan> {
    match n.ident_span.clone() {
        Some(s) => s.clone(),
        None => n.span.clone(),
    }
}

pub fn make_expr_node(
    expr_data: Rc<ExprData>,
    children: Rc<Vec<Rc<Node>>>,
    inferred: Option<Rc<InferredNode>>,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: "".to_string(),
        span: span,
        ident_span: None,
        children: children,
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: inferred,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: expr_data,
        ident: None,
    })
}

pub fn make_named_expr_node(
    name: String,
    expr_data: Rc<ExprData>,
    children: Rc<Vec<Rc<Node>>>,
    inferred: Option<Rc<InferredNode>>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.clone(),
        span: span,
        ident_span: default_ident_span(name.clone(), name_span),
        children: children,
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: inferred,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: expr_data,
        ident: None,
    })
}

pub fn make_expr_error_node(
    kind: ExprErrorKind,
    message: String,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: "".to_string(),
        span: span.clone(),
        ident_span: None,
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: Some(Rc::new(InferredNode::CompilerError {
            message: message.clone(),
            span: span.clone(),
        })),
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::ExprError {
            kind: kind,
            message: message.clone(),
        }),
        ident: None,
    })
}

pub fn make_arg_node(
    name: Option<String>,
    value: Rc<Node>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let arg_name = match name {
            Some(n) => n.clone(),
            None => "".to_string(),
        };
        Rc::new(Node {
            name: arg_name.clone(),
            span: span,
            ident_span: default_ident_span(arg_name.clone(), name_span),
            children: Rc::new(vec![value]),
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn make_arm_node(
    pattern: Rc<MatchPattern>,
    guard: Option<Rc<Node>>,
    body: Rc<Node>,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let children = match guard {
            Some(g) => Rc::new(vec![g.clone(), body]),
            None => Rc::new(vec![body]),
        };
        Rc::new(Node {
            name: "".to_string(),
            span: span,
            ident_span: None,
            children: children,
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: Some(pattern),
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn make_resource_use_node(
    name: String,
    resource: Rc<Node>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.clone(),
        span: span,
        ident_span: default_ident_span(name.clone(), name_span),
        children: Rc::new(vec![resource]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn resource_use_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn resource_use_resource(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            "malformed resource-use: missing resource".to_string(),
            n.span.clone(),
        ),
    }
}

pub fn make_field_init_node(
    name: String,
    value: Rc<Node>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.clone(),
        span: span,
        ident_span: default_ident_span(name.clone(), name_span),
        children: Rc::new(vec![value]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn make_field_binding_node(
    field_name: String,
    binding: Rc<MatchPattern>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: field_name.clone(),
        span: span,
        ident_span: default_ident_span(field_name.clone(), name_span),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: Some(binding),
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn field_binding_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn field_binding_pattern(n: Rc<Node>) -> Rc<MatchPattern> {
    match n.match_pattern.clone() {
        Some(p) => p.clone(),
        None => Rc::new(MatchPattern::Wildcard),
    }
}

pub fn make_text_part_node(text: String, span: Rc<SourceSpan>) -> Rc<Node> {
    Rc::new(Node {
        name: "".to_string(),
        span: span,
        ident_span: None,
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::ExprLiteral {
            value: Rc::new(LiteralValue::LitStr { value: text }),
        }),
        ident: None,
    })
}

pub fn make_interp_part_node(expr: Rc<Node>, span: Rc<SourceSpan>) -> Rc<Node> {
    Rc::new(Node {
        name: "".to_string(),
        span: span,
        ident_span: None,
        children: Rc::new(vec![expr]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn make_param_node(
    name: String,
    type_expr: Rc<Node>,
    default_value: Option<Rc<Node>>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let children = match default_value {
            Some(dv) => Rc::new(vec![type_expr, dv.clone()]),
            None => Rc::new(vec![type_expr]),
        };
        Rc::new(Node {
            name: name.clone(),
            span: span,
            ident_span: default_ident_span(name.clone(), name_span),
            children: children,
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn make_resolved_param_node(
    name: String,
    type_expr: Rc<Node>,
    default_value: Option<Rc<Node>>,
    properties: Rc<Vec<Rc<Node>>>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let children = match default_value {
            Some(dv) => Rc::new(vec![type_expr.clone(), dv.clone()]),
            None => Rc::new(vec![type_expr.clone()]),
        };
        Rc::new(Node {
            name: name.clone(),
            span: span,
            ident_span: default_ident_span(name.clone(), name_span),
            children: children,
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: Some(Rc::new(InferredNode::Resolved {
                node: type_expr.clone(),
            })),
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: properties,
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn param_node_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn generic_param_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn authored_name_at(
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    node: Rc<Node>,
) -> String {
    match node.ident_span.clone() {
        Some(span) => match v1_rt::map_get(&source_indices, span.file.clone()) {
            Some(index) => {
                let text = source_text_at(index.clone(), span.clone());
                if (text.clone().as_str() == "".to_string().as_str()) {
                    "".to_string()
                } else {
                    text.clone()
                }
            }
            None => {
                if ((v1_rt::string_length(&span.file.clone()) > 8)
                    && (v1_rt::substring(&span.file.clone(), 0, 8).as_str()
                        == "<kernel:".to_string().as_str()))
                {
                    v1_rt::substring(
                        &span.file.clone(),
                        8,
                        (v1_rt::string_length(&span.file.clone()) - 1),
                    )
                } else {
                    node.name.clone()
                }
            }
        },
        None => "".to_string(),
    }
}

pub fn find_child_named(
    n: Rc<Node>,
    name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    match Rc::new({
        let mut __result = Vec::new();
        for c in n.children.clone().iter().cloned() {
            if (authored_name_at(source_indices.clone(), c.clone()).as_str()
                == name.clone().as_str())
            {
                __result.push(c);
            }
        }
        __result
    })
    .first()
    .cloned()
    {
        Some(ch) => Some(ch.clone()),
        None => None,
    }
}

pub fn has_child_named(
    n: Rc<Node>,
    name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    {
        let mut __found = false;
        for c in n.children.clone().iter().cloned() {
            if (authored_name_at(source_indices.clone(), c.clone()).as_str()
                == name.clone().as_str())
            {
                __found = true;
                break;
            }
        }
        __found
    }
}

pub fn param_node_type_expr(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            "malformed param: missing type_expr".to_string(),
            n.span.clone(),
        ),
    }
}

pub fn param_node_default_value(n: Rc<Node>) -> Option<Rc<Node>> {
    if ((n.children.clone().len() as i64) > 1) {
        n.children.clone().get(1 as usize).cloned()
    } else {
        None
    }
}

pub fn param_node_span(n: Rc<Node>) -> Rc<SourceSpan> {
    n.span.clone()
}

pub fn make_field_node(
    name: String,
    type_expr: Rc<Node>,
    cardinality: Cardinality,
    default_value: Option<Rc<Node>>,
    from_key: Option<String>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let children = match default_value {
            Some(dv) => Rc::new(vec![type_expr.clone(), dv.clone()]),
            None => Rc::new(vec![type_expr.clone()]),
        };
        let props = match from_key {
            Some(fk) => Rc::new(vec![make_field_init_node(
                "from_key".to_string(),
                Rc::new(Node {
                    name: fk.clone(),
                    span: make_span(0, 0),
                    ident_span: default_ident_span(fk.clone(), make_span(0, 0)),
                    children: Rc::new(vec![]),
                    connective: Connective::NoConnective,
                    params: Rc::new(vec![]),
                    inferred: None,
                    return_cardinality: Cardinality::Required,
                    uses: Rc::new(vec![]),
                    body: None,
                    transport: None,
                    properties: Rc::new(vec![]),
                    type_annotation: None,
                    is_self_recursive: false,
                    has_non_tail_self_call: false,
                    match_pattern: None,
                    expr_data: Rc::new(ExprData::NoExprData),
                    ident: None,
                }),
                make_span(0, 0),
                make_span(0, 0),
            )]),
            None => Rc::new(vec![]),
        };
        Rc::new(Node {
            name: name.clone(),
            span: span,
            ident_span: default_ident_span(name.clone(), name_span),
            children: children,
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: cardinality,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: v1_rt::concat(props, type_expr.properties.clone()),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn field_node_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn field_node_type_expr(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            "malformed field: missing type_expr".to_string(),
            n.span.clone(),
        ),
    }
}

pub fn field_node_cardinality(n: Rc<Node>) -> Cardinality {
    n.return_cardinality.clone()
}

pub fn field_node_default_value(n: Rc<Node>) -> Option<Rc<Node>> {
    if ((n.children.clone().len() as i64) > 1) {
        n.children.clone().get(1 as usize).cloned()
    } else {
        None
    }
}

pub fn field_node_from_key(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    match find_property(
        n.properties.clone(),
        "from_key".to_string(),
        source_indices.clone(),
    ) {
        Some(p) => Some(authored_name_at(source_indices.clone(), p.clone())),
        None => None,
    }
}

pub fn field_node_span(n: Rc<Node>) -> Rc<SourceSpan> {
    n.span.clone()
}

pub fn make_variant_node(
    name: String,
    fields: Rc<Vec<Rc<Node>>>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.clone(),
        span: span,
        ident_span: default_ident_span(name.clone(), name_span),
        children: fields,
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn variant_node_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn variant_node_fields(n: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    n.children.clone()
}

pub fn variant_node_span(n: Rc<Node>) -> Rc<SourceSpan> {
    n.span.clone()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChildRole {
    pub name: String,
    pub accessor: String,
    pub position: i64,
    pub required: bool,
}

pub fn expr_child_roles() -> Rc<HashMap<String, Rc<Vec<Rc<ChildRole>>>>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, Rc<Vec<Rc<ChildRole>>>>> = {
            serde_json::from_value(serde_json::json!({"ExprFieldAccess": [{"name": "base", "accessor": "field_access_base", "position": 0, "required": true}], "ExprBinOp": [{"name": "left", "accessor": "binop_left", "position": 0, "required": true}, {"name": "right", "accessor": "binop_right", "position": 1, "required": true}], "ExprUnaryOp": [{"name": "operand", "accessor": "unaryop_operand", "position": 0, "required": true}], "ExprIf": [{"name": "condition", "accessor": "if_condition", "position": 0, "required": true}, {"name": "then", "accessor": "if_then_branch", "position": 1, "required": true}, {"name": "else", "accessor": "if_else_branch", "position": 2, "required": false}], "ExprMatch": [{"name": "scrutinee", "accessor": "match_scrutinee", "position": 0, "required": true}], "ExprLet": [{"name": "value", "accessor": "let_value", "position": 0, "required": true}, {"name": "body", "accessor": "let_body", "position": 1, "required": false}], "ExprLambda": [{"name": "body", "accessor": "lambda_body", "position": 0, "required": true}], "ExprMethodCall": [{"name": "receiver", "accessor": "method_receiver", "position": 0, "required": true}], "ExprCast": [{"name": "expr", "accessor": "cast_expr", "position": 0, "required": true}, {"name": "target", "accessor": "cast_target", "position": 1, "required": true}], "ExprForEach": [{"name": "collection", "accessor": "foreach_collection", "position": 0, "required": true}, {"name": "body", "accessor": "foreach_body", "position": 1, "required": true}], "ExprIndex": [{"name": "base", "accessor": "index_base", "position": 0, "required": true}, {"name": "index", "accessor": "index_expr", "position": 1, "required": true}], "ExprSlice": [{"name": "base", "accessor": "slice_base", "position": 0, "required": true}, {"name": "start", "accessor": "slice_start", "position": 1, "required": true}, {"name": "end", "accessor": "slice_end", "position": 2, "required": true}], "ExprReturn": [{"name": "value", "accessor": "return_value", "position": 0, "required": true}]}))
                .expect("valid data definition")
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, Rc<Vec<Rc<ChildRole>>>>>| c.clone())
}

pub fn wrapper_child_roles() -> Rc<HashMap<String, Rc<Vec<Rc<ChildRole>>>>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, Rc<Vec<Rc<ChildRole>>>>> = {
            serde_json::from_value(serde_json::json!({"Arg": [{"name": "value", "accessor": "arg_value", "position": 0, "required": true}], "Arm": [{"name": "guard", "accessor": "arm_guard", "position": 0, "required": false}, {"name": "body", "accessor": "arm_body", "position": -1, "required": true}], "FieldInit": [{"name": "value", "accessor": "field_init_node_value", "position": 0, "required": true}]}))
                .expect("valid data definition")
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, Rc<Vec<Rc<ChildRole>>>>>| c.clone())
}

pub fn is_child_accessor_in_model(name: String) -> bool {
    ({
        let mut __found = false;
        for roles in Rc::new(v1_rt::map_values(&expr_child_roles()))
            .iter()
            .cloned()
        {
            if {
                let mut __found = false;
                for r in roles.clone().iter().cloned() {
                    if (r.accessor.clone().as_str() == name.clone().as_str()) {
                        __found = true;
                        break;
                    }
                }
                __found
            } {
                __found = true;
                break;
            }
        }
        __found
    } || {
        let mut __found = false;
        for roles in Rc::new(v1_rt::map_values(&wrapper_child_roles()))
            .iter()
            .cloned()
        {
            if {
                let mut __found = false;
                for r in roles.clone().iter().cloned() {
                    if (r.accessor.clone().as_str() == name.clone().as_str()) {
                        __found = true;
                        break;
                    }
                }
                __found
            } {
                __found = true;
                break;
            }
        }
        __found
    })
}

pub fn child_roles_for_variant(variant_name: String) -> Option<Rc<Vec<Rc<ChildRole>>>> {
    v1_rt::map_get(&expr_child_roles(), variant_name)
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum NodeFieldRole {
    ChildrenListField,
    SubValueField,
    MetadataField,
}

pub fn node_field_roles() -> Rc<HashMap<String, NodeFieldRole>> {
    thread_local! {
        static CACHED: Rc<HashMap<String, NodeFieldRole>> = {
            let mut __m = HashMap::new();
            __m.insert("children".to_string(), NodeFieldRole::ChildrenListField);
            __m.insert("params".to_string(), NodeFieldRole::ChildrenListField);
            __m.insert("body".to_string(), NodeFieldRole::SubValueField);
            __m.insert("expr_data".to_string(), NodeFieldRole::SubValueField);
            __m.insert("match_pattern".to_string(), NodeFieldRole::SubValueField);
            Rc::new(__m)
        };
    }
    CACHED.with(|c: &Rc<HashMap<String, NodeFieldRole>>| c.clone())
}

pub fn is_children_list_field(field_name: String) -> bool {
    match v1_rt::lookup(&node_field_roles(), field_name) {
        v1_rt::Witness::Holds {
            value: NodeFieldRole::ChildrenListField,
            ..
        } => true,
        _ => false,
    }
}

pub fn is_sub_value_field(field_name: String) -> bool {
    match v1_rt::lookup(&node_field_roles(), field_name) {
        v1_rt::Witness::Holds {
            value: NodeFieldRole::SubValueField,
            ..
        } => true,
        v1_rt::Witness::Holds {
            value: NodeFieldRole::ChildrenListField,
            ..
        } => true,
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum FunctionSizeEffect {
    TreeSizePreserving,
    TreeSizeReducing,
    PropertyContraction { domain_size: i64 },
}
impl FunctionSizeEffect {
    pub fn domain_size(&self) -> i64 {
        match self {
            FunctionSizeEffect::TreeSizePreserving => panic!("no domain_size on unit variant"),
            FunctionSizeEffect::TreeSizeReducing => panic!("no domain_size on unit variant"),
            FunctionSizeEffect::PropertyContraction {
                domain_size: __val, ..
            } => __val.clone(),
        }
    }
}

pub fn function_size_effects() -> Rc<HashMap<String, Rc<FunctionSizeEffect>>> {
    thread_local! {
            static CACHED: Rc<HashMap<String, Rc<FunctionSizeEffect>>> = {
                let mut __m = HashMap::new();
                __m.insert("with_required_cardinality".to_string(), Rc::new(FunctionSizeEffect::PropertyContraction {
        domain_size: 2,
    }));
                __m.insert("resolved_type".to_string(), Rc::new(FunctionSizeEffect::TreeSizeReducing));
                __m.insert("param_node_type_expr".to_string(), Rc::new(FunctionSizeEffect::TreeSizeReducing));
                __m.insert("field_binding_pattern".to_string(), Rc::new(FunctionSizeEffect::TreeSizeReducing));
                __m.insert("wrapper_inner_arg".to_string(), Rc::new(FunctionSizeEffect::TreeSizeReducing));
                __m.insert("extractor_inner_arg".to_string(), Rc::new(FunctionSizeEffect::TreeSizeReducing));
                __m.insert("child_type_node".to_string(), Rc::new(FunctionSizeEffect::TreeSizeReducing));
                Rc::new(__m)
            };
        }
    CACHED.with(|c: &Rc<HashMap<String, Rc<FunctionSizeEffect>>>| c.clone())
}

pub fn is_tree_size_preserving(func_name: String) -> bool {
    match v1_rt::lookup(&function_size_effects(), func_name) {
        v1_rt::Witness::Holds { value: effect, .. } => match (*effect.clone()).clone() {
            FunctionSizeEffect::TreeSizePreserving => true,
            FunctionSizeEffect::PropertyContraction { domain_size: _, .. } => true,
            _ => false,
        },
        v1_rt::Witness::Violates { diagnostic: _, .. } => false,
    }
}

pub fn is_tree_size_reducing(func_name: String) -> bool {
    match v1_rt::lookup(&function_size_effects(), func_name) {
        v1_rt::Witness::Holds { value: effect, .. } => match (*effect.clone()).clone() {
            FunctionSizeEffect::TreeSizeReducing => true,
            _ => false,
        },
        v1_rt::Witness::Violates { diagnostic: _, .. } => false,
    }
}

pub fn is_property_contraction(func_name: String) -> bool {
    match v1_rt::lookup(&function_size_effects(), func_name) {
        v1_rt::Witness::Holds { value: effect, .. } => match (*effect.clone()).clone() {
            FunctionSizeEffect::PropertyContraction { domain_size: _, .. } => true,
            _ => false,
        },
        v1_rt::Witness::Violates { diagnostic: _, .. } => false,
    }
}

pub fn expr_child_at(texpr: Rc<Node>, index: i64, role: String) -> Rc<Node> {
    match texpr.children.clone().get(index as usize).cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            v1_rt::concat("malformed node: missing ".to_string(), role),
            texpr.span.clone(),
        ),
    }
}

pub fn arg_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    {
        let name = authored_name_at(source_indices, n);
        if (name.clone().as_str() == "".to_string().as_str()) {
            None
        } else {
            Some(name.clone())
        }
    }
}

pub fn arg_value(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            "malformed arg: missing value".to_string(),
            n.span.clone(),
        ),
    }
}

pub fn arm_pattern(n: Rc<Node>) -> Rc<MatchPattern> {
    match n.match_pattern.clone() {
        Some(p) => p.clone(),
        None => Rc::new(MatchPattern::Wildcard),
    }
}

pub fn arm_guard(n: Rc<Node>) -> Option<Rc<Node>> {
    if ((n.children.clone().len() as i64) == 2) {
        n.children.clone().first().cloned()
    } else {
        None
    }
}

pub fn arm_body(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().last().cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            "malformed arm: missing body".to_string(),
            n.span.clone(),
        ),
    }
}

pub fn field_init_node_name_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, n)
}

pub fn field_init_node_value(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
        Some(v) => v.clone(),
        None => make_expr_error_node(
            ExprErrorKind::InternalExprError,
            "malformed field-init: missing value".to_string(),
            n.span.clone(),
        ),
    }
}

pub fn if_condition(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "if condition".to_string())
}

pub fn if_then_branch(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1, "if then-branch".to_string())
}

pub fn if_else_branch(texpr: Rc<Node>) -> Option<Rc<Node>> {
    texpr.children.clone().get(2 as usize).cloned()
}

pub fn match_scrutinee(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "match scrutinee".to_string())
}

pub fn match_arm_nodes(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    Rc::new(
        texpr
            .children
            .clone()
            .iter()
            .cloned()
            .skip(1 as usize)
            .collect::<Vec<_>>(),
    )
}

pub fn binop_left(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "binop left".to_string())
}

pub fn binop_right(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1, "binop right".to_string())
}

pub fn unaryop_operand(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "unaryop operand".to_string())
}

pub fn expr_var_name_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, texpr)
}

pub fn field_access_base(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "field access base".to_string())
}

pub fn field_access_field_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, texpr)
}

pub fn expr_field_access_summary(texpr: Rc<Node>) -> Option<Rc<FieldSummary>> {
    match (*texpr.expr_data.clone()).clone() {
        ExprData::ExprFieldAccess { summary: s, .. } => s.clone(),
        _ => None,
    }
}

pub fn expr_call_func_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, texpr)
}

pub fn expr_call_descent_evidence(texpr: Rc<Node>) -> Option<Rc<Vec<Rc<SubValueRelation>>>> {
    match (*texpr.expr_data.clone()).clone() {
        ExprData::ExprCall {
            descent_evidence: de,
            ..
        } => de.clone(),
        _ => None,
    }
}

pub fn method_receiver(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "method receiver".to_string())
}

pub fn method_arg_nodes(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    Rc::new(
        texpr
            .children
            .clone()
            .iter()
            .cloned()
            .skip(1 as usize)
            .collect::<Vec<_>>(),
    )
}

pub fn expr_method_name_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, texpr)
}

pub fn expr_method_call_semantics(texpr: Rc<Node>) -> Option<Rc<MethodSemantics>> {
    match (*texpr.expr_data.clone()).clone() {
        ExprData::ExprMethodCall {
            method_semantics: ms,
            ..
        } => ms.clone(),
        _ => None,
    }
}

pub fn lambda_body(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "lambda body".to_string())
}

pub fn lambda_param_names_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Rc<Vec<String>> {
    Rc::new({
        let mut __result = Vec::new();
        for n in Rc::new(
            texpr
                .children
                .clone()
                .iter()
                .cloned()
                .skip(1 as usize)
                .collect::<Vec<_>>(),
        )
        .iter()
        .cloned()
        {
            __result.push(authored_name_at(source_indices.clone(), n.clone()));
        }
        __result
    })
}

pub fn let_value(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "let value".to_string())
}

pub fn let_body(texpr: Rc<Node>) -> Option<Rc<Node>> {
    texpr.children.clone().get(1 as usize).cloned()
}

pub fn let_binding_name_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, texpr)
}

pub fn cast_expr(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "cast expr".to_string())
}

pub fn cast_target(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1, "cast target".to_string())
}

pub fn foreach_collection(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "foreach collection".to_string())
}

pub fn foreach_body(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1, "foreach body".to_string())
}

pub fn foreach_variable_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> String {
    authored_name_at(source_indices, texpr)
}

pub fn index_base(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "index base".to_string())
}

pub fn index_expr(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1, "index expression".to_string())
}

pub fn slice_base(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "slice base".to_string())
}

pub fn slice_start(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1, "slice start".to_string())
}

pub fn slice_end(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 2, "slice end".to_string())
}

pub fn return_value(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0, "return value".to_string())
}

pub fn block_stmts(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    texpr.children.clone()
}

pub fn record_lit_type_name_at(
    texpr: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    {
        let name = authored_name_at(source_indices, texpr);
        if (name.clone().as_str() == "".to_string().as_str()) {
            None
        } else {
            Some(name.clone())
        }
    }
}

pub fn transport_url_key() -> String {
    thread_local! {
        static CACHED: String = {
            "base_url".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_path_key() -> String {
    thread_local! {
        static CACHED: String = {
            "base_path".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_auth_token_key() -> String {
    thread_local! {
        static CACHED: String = {
            "auth_token".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_auth_header_key() -> String {
    thread_local! {
        static CACHED: String = {
            "auth_header".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_auth_scheme_key() -> String {
    thread_local! {
        static CACHED: String = {
            "auth_scheme".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_method_key() -> String {
    thread_local! {
        static CACHED: String = {
            "method".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_path_template_key() -> String {
    thread_local! {
        static CACHED: String = {
            "path".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_query_key() -> String {
    thread_local! {
        static CACHED: String = {
            "query".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_body_key() -> String {
    thread_local! {
        static CACHED: String = {
            "body".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_stdin_key() -> String {
    thread_local! {
        static CACHED: String = {
            "stdin".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_response_format_key() -> String {
    thread_local! {
        static CACHED: String = {
            "response_format".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn transport_headers_key() -> String {
    thread_local! {
        static CACHED: String = {
            "headers".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn make_transport_node(
    properties: Rc<Vec<Rc<Node>>>,
    children: Rc<Vec<Rc<Node>>>,
    body: Option<Rc<Node>>,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: "".to_string(),
        span: span,
        ident_span: None,
        children: children,
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: body,
        transport: None,
        properties: properties,
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn local_transport_node(span: Rc<SourceSpan>) -> Rc<Node> {
    make_transport_node(Rc::new(vec![]), Rc::new(vec![]), None, span)
}

pub fn rest_transport_node(
    base_url: Rc<Node>,
    auth_props: Rc<Vec<Rc<Node>>>,
    headers: Rc<Vec<Rc<Node>>>,
    method: Option<Rc<Node>>,
    path: Option<Rc<Node>>,
    query: Option<Rc<Node>>,
    request_body: Option<Rc<Node>>,
    response_format: Option<Rc<Node>>,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let zero_span = make_span(0, 0);
        let url_field = make_field_init_node(
            transport_url_key(),
            base_url,
            zero_span.clone(),
            zero_span.clone(),
        );
        let method_props = match method {
            Some(m) => Rc::new(vec![make_field_init_node(
                transport_method_key(),
                m.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let path_props = match path {
            Some(p) => Rc::new(vec![make_field_init_node(
                transport_path_template_key(),
                p.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let query_props = match query {
            Some(q) => Rc::new(vec![make_field_init_node(
                transport_query_key(),
                q.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let body_props = match request_body {
            Some(b) => Rc::new(vec![make_field_init_node(
                transport_body_key(),
                b.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let rf_props = match response_format {
            Some(rf) => Rc::new(vec![make_field_init_node(
                transport_response_format_key(),
                rf.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let props = v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(
                        v1_rt::concat(
                            v1_rt::concat(
                                v1_rt::concat(Rc::new(vec![url_field]), method_props),
                                path_props,
                            ),
                            query_props,
                        ),
                        body_props,
                    ),
                    rf_props,
                ),
                auth_props,
            ),
            headers,
        );
        make_transport_node(props, Rc::new(vec![]), None, span)
    }
}

pub fn shell_transport_node(
    argv: Rc<Vec<Rc<Node>>>,
    env: Rc<Vec<Rc<Node>>>,
    stdin: Option<Rc<Node>>,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let shell_marker = Rc::new(Node {
            name: "".to_string(),
            span: span.clone(),
            ident_span: None,
            children: Rc::new(vec![]),
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        });
        let zero_span = make_span(0, 0);
        let stdin_props = match stdin {
            Some(s) => Rc::new(vec![make_field_init_node(
                transport_stdin_key(),
                s.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let all_props = v1_rt::concat(env, stdin_props);
        Rc::new(Node {
            name: "".to_string(),
            span: span.clone(),
            ident_span: None,
            children: argv,
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: Some(shell_marker),
            transport: None,
            properties: all_props,
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn file_transport_node(base_path: Rc<Node>, span: Rc<SourceSpan>) -> Rc<Node> {
    {
        let path_field = make_field_init_node(
            transport_path_key(),
            base_path,
            make_span(0, 0),
            make_span(0, 0),
        );
        make_transport_node(Rc::new(vec![path_field]), Rc::new(vec![]), None, span)
    }
}

pub fn find_property(
    props: Rc<Vec<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    match Rc::new({
        let mut __result = Vec::new();
        for p in props.iter().cloned() {
            if (field_init_node_name_at(p.clone(), source_indices.clone()).as_str()
                == prop_name.clone().as_str())
            {
                __result.push(p);
            }
        }
        __result
    })
    .first()
    .cloned()
    {
        Some(fi) => Some(field_init_node_value(fi.clone())),
        None => None,
    }
}

pub fn find_property_string(
    props: Rc<Vec<Rc<Node>>>,
    prop_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    match find_property(props, prop_name, source_indices) {
        Some(n) => match (*n.expr_data.clone()).clone() {
            ExprData::ExprLiteral { ref value, .. } => {
                let LiteralValue::LitStr { value: s, .. } = value.as_ref() else {
                    unreachable!()
                };
                Some(s.clone())
            }
            _ => None,
        },
        None => None,
    }
}

pub fn transport_base_path(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), transport_path_key(), source_indices)
}

pub fn transport_has_argv(t: Rc<Node>) -> bool {
    ((t.children.clone().len() as i64) > 0)
}

pub fn is_rest_transport(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    (transport_base_url(t, source_indices) != None)
}

pub fn is_shell_transport(t: Rc<Node>) -> bool {
    (t.body.clone() != None)
}

pub fn is_file_transport(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    (transport_base_path(t, source_indices) != None)
}

pub fn is_local_transport(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    ((!is_rest_transport(t.clone(), source_indices.clone()) && !is_shell_transport(t.clone()))
        && !is_file_transport(t.clone(), source_indices.clone()))
}

pub fn field_init_operation_modifier(
    field_init: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<OperationModifier> {
    {
        let fi_name = field_init_node_name_at(field_init, source_indices);
        if (fi_name.clone().as_str() == "idempotent".to_string().as_str()) {
            Some(OperationModifier::Idempotent)
        } else {
            if (fi_name.clone().as_str() == "readonly".to_string().as_str()) {
                Some(OperationModifier::Readonly)
            } else {
                if (fi_name.clone().as_str() == "hermetic".to_string().as_str()) {
                    Some(OperationModifier::Hermetic)
                } else {
                    None
                }
            }
        }
    }
}

pub fn operation_modifier_name(modifier: OperationModifier) -> String {
    match modifier {
        OperationModifier::Idempotent => "idempotent".to_string(),
        OperationModifier::Readonly => "readonly".to_string(),
        OperationModifier::Hermetic => "hermetic".to_string(),
    }
}

pub fn transport_base_url(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), transport_url_key(), source_indices)
}

pub fn transport_auth_token(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        t.properties.clone(),
        transport_auth_token_key(),
        source_indices,
    )
}

pub fn transport_auth_header_name(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    find_property_string(
        t.properties.clone(),
        transport_auth_header_key(),
        source_indices,
    )
}

pub fn transport_has_auth(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    match find_property(
        t.properties.clone(),
        transport_auth_token_key(),
        source_indices,
    ) {
        Some(_) => true,
        None => false,
    }
}

pub fn transport_method(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), transport_method_key(), source_indices)
}

pub fn transport_path_template(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        t.properties.clone(),
        transport_path_template_key(),
        source_indices,
    )
}

pub fn transport_query(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), transport_query_key(), source_indices)
}

pub fn transport_request_body(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), transport_body_key(), source_indices)
}

pub fn transport_stdin(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), transport_stdin_key(), source_indices)
}

pub fn transport_response_format(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        t.properties.clone(),
        transport_response_format_key(),
        source_indices,
    )
}

pub fn is_config_reserved_key(name: String) -> bool {
    ((((((((((((name.clone().as_str() == transport_url_key().as_str())
        || (name.clone().as_str() == transport_path_key().as_str()))
        || (name.clone().as_str() == transport_auth_scheme_key().as_str()))
        || (name.clone().as_str() == transport_auth_header_key().as_str()))
        || (name.clone().as_str() == transport_auth_token_key().as_str()))
        || (name.clone().as_str() == transport_method_key().as_str()))
        || (name.clone().as_str() == transport_path_template_key().as_str()))
        || (name.clone().as_str() == transport_query_key().as_str()))
        || (name.clone().as_str() == transport_body_key().as_str()))
        || (name.clone().as_str() == transport_stdin_key().as_str()))
        || (name.clone().as_str() == transport_response_format_key().as_str()))
        || (name.clone().as_str() == transport_headers_key().as_str()))
}

pub fn transport_headers(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Rc<Vec<Rc<Node>>> {
    Rc::new({
        let mut __result = Vec::new();
        for p in t.properties.clone().iter().cloned() {
            if !is_config_reserved_key(field_init_node_name_at(p.clone(), source_indices.clone())) {
                __result.push(p);
            }
        }
        __result
    })
}

pub fn transport_env(
    t: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Rc<Vec<Rc<Node>>> {
    Rc::new({
        let mut __result = Vec::new();
        for p in t.properties.clone().iter().cloned() {
            if !is_config_reserved_key(field_init_node_name_at(p.clone(), source_indices.clone())) {
                __result.push(p);
            }
        }
        __result
    })
}

pub fn map_children(node: Rc<Node>, transform: impl Fn(Rc<Node>) -> Rc<Node> + Clone) -> Rc<Node> {
    Rc::new(Node {
        name: node.name.clone(),
        ident: node.ident.clone(),
        span: node.span.clone(),
        ident_span: node.ident_span.clone(),
        children: Rc::new({
            let mut __result = Vec::new();
            for child in node.children.clone().iter().cloned() {
                __result.push(transform(child.clone()));
            }
            __result
        }),
        connective: node.connective.clone(),
        params: node.params.clone(),
        inferred: node.inferred.clone(),
        return_cardinality: node.return_cardinality.clone(),
        uses: node.uses.clone(),
        body: node.body.clone(),
        transport: node.transport.clone(),
        properties: node.properties.clone(),
        type_annotation: node.type_annotation.clone(),
        is_self_recursive: node.is_self_recursive.clone(),
        has_non_tail_self_call: node.has_non_tail_self_call.clone(),
        match_pattern: node.match_pattern.clone(),
        expr_data: node.expr_data.clone(),
    })
}

pub fn expr_has_self_call(
    texpr: Rc<Node>,
    fn_name: String,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match (*texpr.expr_data.clone()).clone() {
            ExprData::ExprCall { .. } => {
                if (expr_call_func_at(texpr.clone(), source_indices.clone()).as_str()
                    == fn_name.clone().as_str())
                {
                    true
                } else {
                    {
                        let mut __found = false;
                        for child in texpr.children.clone().iter().cloned() {
                            if expr_has_self_call(
                                child.clone(),
                                fn_name.clone(),
                                source_indices.clone(),
                            ) {
                                __found = true;
                                break;
                            }
                        }
                        __found
                    }
                }
            }
            _ => {
                let mut __found = false;
                for child in texpr.children.clone().iter().cloned() {
                    if expr_has_self_call(child.clone(), fn_name.clone(), source_indices.clone()) {
                        __found = true;
                        break;
                    }
                }
                __found
            }
        }
    })
}

pub fn expr_has_non_tail_self_call(
    texpr: Rc<Node>,
    fn_name: String,
    in_tail: bool,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match (*texpr.expr_data.clone()).clone() {
            ExprData::ExprCall { .. } => {
                if (expr_call_func_at(texpr.clone(), source_indices.clone()).as_str()
                    == fn_name.clone().as_str())
                {
                    if (in_tail.clone() == false) {
                        true
                    } else {
                        {
                            let mut __found = false;
                            for child in texpr.children.clone().iter().cloned() {
                                if expr_has_non_tail_self_call(
                                    child.clone(),
                                    fn_name.clone(),
                                    false,
                                    source_indices.clone(),
                                ) {
                                    __found = true;
                                    break;
                                }
                            }
                            __found
                        }
                    }
                } else {
                    {
                        let mut __found = false;
                        for child in texpr.children.clone().iter().cloned() {
                            if expr_has_non_tail_self_call(
                                child.clone(),
                                fn_name.clone(),
                                false,
                                source_indices.clone(),
                            ) {
                                __found = true;
                                break;
                            }
                        }
                        __found
                    }
                }
            }
            ExprData::ExprError { .. } => false,
            ExprData::ExprVar {
                binding_kind: _, ..
            } => false,
            ExprData::ExprLiteral { value: _, .. } => false,
            ExprData::ExprFieldAccess { summary: _, .. } => {
                let mut __found = false;
                for child in texpr.children.clone().iter().cloned() {
                    if expr_has_non_tail_self_call(
                        child.clone(),
                        fn_name.clone(),
                        false,
                        source_indices.clone(),
                    ) {
                        __found = true;
                        break;
                    }
                }
                __found
            }
            ExprData::ExprMethodCall {
                method_semantics: _,
                ..
            } => {
                let mut __found = false;
                for child in texpr.children.clone().iter().cloned() {
                    if expr_has_non_tail_self_call(
                        child.clone(),
                        fn_name.clone(),
                        false,
                        source_indices.clone(),
                    ) {
                        __found = true;
                        break;
                    }
                }
                __found
            }
            ExprData::ExprIf => {
                let cond_bad = expr_has_non_tail_self_call(
                    if_condition(texpr.clone()),
                    fn_name.clone(),
                    false,
                    source_indices.clone(),
                );
                let then_bad = expr_has_non_tail_self_call(
                    if_then_branch(texpr.clone()),
                    fn_name.clone(),
                    in_tail.clone(),
                    source_indices.clone(),
                );
                let else_bad = match if_else_branch(texpr.clone()) {
                    Some(e) => expr_has_non_tail_self_call(
                        e.clone(),
                        fn_name.clone(),
                        in_tail.clone(),
                        source_indices.clone(),
                    ),
                    None => false,
                };
                ((cond_bad || then_bad) || else_bad)
            }
            ExprData::ExprMatch => {
                let scrut_bad = expr_has_non_tail_self_call(
                    match_scrutinee(texpr.clone()),
                    fn_name.clone(),
                    false,
                    source_indices.clone(),
                );
                let arms_bad = {
                    let mut __found = false;
                    for arm_node in match_arm_nodes(texpr.clone()).iter().cloned() {
                        if expr_has_non_tail_self_call(
                            arm_body(arm_node.clone()),
                            fn_name.clone(),
                            in_tail.clone(),
                            source_indices.clone(),
                        ) {
                            __found = true;
                            break;
                        }
                    }
                    __found
                };
                (scrut_bad || arms_bad)
            }
            ExprData::ExprLet => {
                let val_bad = expr_has_non_tail_self_call(
                    let_value(texpr.clone()),
                    fn_name.clone(),
                    false,
                    source_indices.clone(),
                );
                let body_bad = match let_body(texpr.clone()) {
                    Some(b) => expr_has_non_tail_self_call(
                        b.clone(),
                        fn_name.clone(),
                        in_tail.clone(),
                        source_indices.clone(),
                    ),
                    None => false,
                };
                (val_bad || body_bad)
            }
            ExprData::ExprBlock => {
                let ss = texpr.children.clone();
                let ss_count = (ss.clone().len() as i64);
                if (ss_count.clone() == 0) {
                    false
                } else {
                    {
                        let init_bad = {
                            let mut __found = false;
                            for p in Rc::new({
                                let mut __result = Vec::new();
                                for p in Rc::new(
                                    ss.clone()
                                        .iter()
                                        .cloned()
                                        .enumerate()
                                        .map(|(i, v)| (i as i64, v))
                                        .collect::<Vec<_>>(),
                                )
                                .iter()
                                .cloned()
                                {
                                    if (p.0.clone() < (ss_count.clone() - 1)) {
                                        __result.push(p);
                                    }
                                }
                                __result
                            })
                            .iter()
                            .cloned()
                            {
                                if expr_has_non_tail_self_call(
                                    p.1.clone(),
                                    fn_name.clone(),
                                    false,
                                    source_indices.clone(),
                                ) {
                                    __found = true;
                                    break;
                                }
                            }
                            __found
                        };
                        let last_bad = match ss.clone().last().cloned() {
                            Some(last_expr) => expr_has_non_tail_self_call(
                                last_expr.clone(),
                                fn_name.clone(),
                                in_tail.clone(),
                                source_indices.clone(),
                            ),
                            None => false,
                        };
                        (init_bad || last_bad)
                    }
                }
            }
            ExprData::ExprReturn => {
                let mut __found = false;
                for child in texpr.children.clone().iter().cloned() {
                    if expr_has_non_tail_self_call(
                        child.clone(),
                        fn_name.clone(),
                        true,
                        source_indices.clone(),
                    ) {
                        __found = true;
                        break;
                    }
                }
                __found
            }
            ExprData::NoExprData => {
                let mut __found = false;
                for child in texpr.children.clone().iter().cloned() {
                    if expr_has_non_tail_self_call(
                        child.clone(),
                        fn_name.clone(),
                        in_tail.clone(),
                        source_indices.clone(),
                    ) {
                        __found = true;
                        break;
                    }
                }
                __found
            }
            _ => {
                let mut __found = false;
                for child in texpr.children.clone().iter().cloned() {
                    if expr_has_non_tail_self_call(
                        child.clone(),
                        fn_name.clone(),
                        false,
                        source_indices.clone(),
                    ) {
                        __found = true;
                        break;
                    }
                }
                __found
            }
        }
    })
}

pub fn service_config_properties(
    endpoint: Rc<Node>,
    auth: Option<Rc<Node>>,
    auth_input: Option<Rc<Node>>,
    auth_source: Option<Rc<Node>>,
    rate_limit: Option<Rc<Node>>,
    retry: Option<Rc<Node>>,
) -> Rc<Vec<Rc<Node>>> {
    {
        let zero_span = make_span(0, 0);
        let ep_prop = Rc::new(vec![make_field_init_node(
            "svc_endpoint".to_string(),
            endpoint,
            zero_span.clone(),
            zero_span.clone(),
        )]);
        let auth_prop = match auth {
            Some(a) => Rc::new(vec![make_field_init_node(
                "svc_auth".to_string(),
                a.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let auth_input_prop = match auth_input {
            Some(ai) => Rc::new(vec![make_field_init_node(
                "svc_auth_input".to_string(),
                ai.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let auth_source_prop = match auth_source {
            Some(src) => Rc::new(vec![make_field_init_node(
                "svc_auth_source".to_string(),
                src.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let rate_prop = match rate_limit {
            Some(r) => Rc::new(vec![make_field_init_node(
                "svc_rate_limit".to_string(),
                r.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        let retry_prop = match retry {
            Some(r) => Rc::new(vec![make_field_init_node(
                "svc_retry".to_string(),
                r.clone(),
                zero_span.clone(),
                zero_span.clone(),
            )]),
            None => Rc::new(vec![]),
        };
        v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(v1_rt::concat(ep_prop, auth_prop), auth_input_prop),
                    auth_source_prop,
                ),
                rate_prop,
            ),
            retry_prop,
        )
    }
}

pub fn has_service_config(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> bool {
    {
        let mut __found = false;
        for p in n.properties.clone().iter().cloned() {
            if (field_init_node_name_at(p.clone(), source_indices.clone()).as_str()
                == "svc_endpoint".to_string().as_str())
            {
                __found = true;
                break;
            }
        }
        __found
    }
}

pub fn service_config_endpoint(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        n.properties.clone(),
        "svc_endpoint".to_string(),
        source_indices,
    )
}

pub fn service_config_auth(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(n.properties.clone(), "svc_auth".to_string(), source_indices)
}

pub fn service_config_rate_limit(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        n.properties.clone(),
        "svc_rate_limit".to_string(),
        source_indices,
    )
}

pub fn service_config_retry(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        n.properties.clone(),
        "svc_retry".to_string(),
        source_indices,
    )
}

pub fn service_config_auth_input(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        n.properties.clone(),
        "svc_auth_input".to_string(),
        source_indices,
    )
}

pub fn service_config_auth_source(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    find_property(
        n.properties.clone(),
        "svc_auth_source".to_string(),
        source_indices,
    )
}

pub fn module_node(
    name: String,
    imports: Rc<Vec<Rc<Node>>>,
    items: Rc<Vec<Rc<Node>>>,
    span: Rc<SourceSpan>,
) -> Rc<Node> {
    Rc::new(Node {
        name: name.clone(),
        span: span.clone(),
        ident_span: default_ident_span(name.clone(), span.clone()),
        children: items,
        connective: Connective::NoConnective,
        params: imports,
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn import_node(
    module_path: String,
    is_all: bool,
    specific_names: Rc<Vec<Rc<Node>>>,
    span: Rc<SourceSpan>,
    name_span: Rc<SourceSpan>,
) -> Rc<Node> {
    {
        let wildcard_marker = if is_all {
            Some(Rc::new(Node {
                name: "".to_string(),
                span: span.clone(),
                ident_span: None,
                children: Rc::new(vec![]),
                connective: Connective::NoConnective,
                params: Rc::new(vec![]),
                inferred: None,
                return_cardinality: Cardinality::Required,
                uses: Rc::new(vec![]),
                body: None,
                transport: None,
                properties: Rc::new(vec![]),
                type_annotation: None,
                is_self_recursive: false,
                has_non_tail_self_call: false,
                match_pattern: None,
                expr_data: Rc::new(ExprData::NoExprData),
                ident: None,
            }))
        } else {
            None
        };
        Rc::new(Node {
            name: module_path,
            span: span.clone(),
            ident_span: Some(name_span),
            children: specific_names,
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: wildcard_marker,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }
}

pub fn import_is_all(n: Rc<Node>) -> bool {
    (n.body.clone() != None)
}

pub fn import_specific_names_at(
    n: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Rc<Vec<String>> {
    Rc::new({
        let mut __result = Vec::new();
        for c in n.children.clone().iter().cloned() {
            __result.push(authored_name_at(source_indices.clone(), c.clone()));
        }
        __result
    })
}

pub fn module_imports(n: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    n.params.clone()
}

pub fn module_items(n: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    n.children.clone()
}

pub fn leaf_node_with_span(name: String, span: Rc<SourceSpan>) -> Rc<Node> {
    Rc::new(Node {
        name: name.clone(),
        span: span.clone(),
        ident_span: default_ident_span(name.clone(), span.clone()),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
}

pub fn kernel_span(name: String) -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: v1_rt::concat(
            v1_rt::concat("<kernel:".to_string(), name.clone()),
            ">".to_string(),
        ),
        start: 0,
        end: v1_rt::string_length(&name),
    })
}

pub fn unit_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "Unit".to_string(),
        span: kernel_span("Unit".to_string()),
        ident_span: Some(kernel_span("Unit".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::Conj,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn bool_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "Bool".to_string(),
        span: kernel_span("Bool".to_string()),
        ident_span: Some(kernel_span("Bool".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn string_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "String".to_string(),
        span: kernel_span("String".to_string()),
        ident_span: Some(kernel_span("String".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn hash_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "Hash".to_string(),
        span: kernel_span("Hash".to_string()),
        ident_span: Some(kernel_span("Hash".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn int_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "Int".to_string(),
        span: kernel_span("Int".to_string()),
        ident_span: Some(kernel_span("Int".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn float_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "Float".to_string(),
        span: kernel_span("Float".to_string()),
        ident_span: Some(kernel_span("Float".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn none_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "None".to_string(),
        span: kernel_span("None".to_string()),
        ident_span: Some(kernel_span("None".to_string())),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn tuple_type_name() -> String {
    thread_local! {
        static CACHED: String = {
            "Tuple".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn error_type() -> Rc<Node> {
    thread_local! {
            static CACHED: Rc<Node> = {
                Rc::new(Node {
        name: "".to_string(),
        span: make_span(0, 0),
        ident_span: None,
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: Some(Rc::new(InferredNode::CompilerError {
        message: "unresolved type".to_string(),
        span: make_span(0, 0),
    })),
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::ExprError {
        kind: ExprErrorKind::SemanticExprError,
        message: "unresolved type".to_string(),
    }),
        ident: None,
    })
            };
        }
    CACHED.with(|c: &Rc<Node>| c.clone())
}

pub fn make_span(start: i64, end: i64) -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: "".to_string(),
        start: start,
        end: end,
    })
}

pub fn make_file_span(file: String, start: i64, end: i64) -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: file,
        start: start,
        end: end,
    })
}

pub fn no_span() -> Rc<SourceSpan> {
    make_span(0, 0)
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LineCol {
    pub line: i64,
    pub col: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NewlineIndex {
    pub file: String,
    pub offsets: Rc<Vec<i64>>,
    pub char_codes: Rc<Vec<i64>>,
}

pub fn build_newline_index(file: String, source: String) -> Rc<NewlineIndex> {
    {
        let char_codes = Rc::new(source.chars().map(|c| c as i64).collect::<Vec<_>>());
        let offsets = Rc::new(
            char_codes
                .clone()
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, v)| (i as i64, v))
                .collect::<Vec<_>>(),
        )
        .iter()
        .cloned()
        .fold(Rc::new(vec![]), |acc: _, pair: (i64, i64)| {
            if (pair.1.clone() == 10) {
                v1_rt::rc_list_push(acc.clone(), pair.0.clone())
            } else {
                acc.clone()
            }
        });
        Rc::new(NewlineIndex {
            file: file,
            offsets: offsets,
            char_codes: char_codes.clone(),
        })
    }
}

pub fn byte_to_line_col(index: Rc<NewlineIndex>, offset: i64) -> LineCol {
    {
        let clamped = if (offset.clone() < 0) {
            0
        } else {
            offset.clone()
        };
        let line = ((Rc::new({
            let mut __result = Vec::new();
            for o in index.offsets.clone().iter().cloned() {
                if (o.clone() < clamped.clone()) {
                    __result.push(o);
                }
            }
            __result
        })
        .len() as i64)
            + 1);
        let line_start = if (line.clone() <= 1) {
            0
        } else {
            match index
                .offsets
                .clone()
                .get((line.clone() - 2) as usize)
                .cloned()
            {
                Some(o) => (o.clone() + 1),
                None => 0,
            }
        };
        let col = ((clamped.clone() - line_start) + 1);
        LineCol {
            line: line.clone(),
            col: col,
        }
    }
}

pub fn source_line_at(index: Rc<NewlineIndex>, line: i64) -> String {
    {
        let src_len = (index.char_codes.clone().len() as i64);
        let line_start = if (line.clone() <= 1) {
            0
        } else {
            match index
                .offsets
                .clone()
                .get((line.clone() - 2) as usize)
                .cloned()
            {
                Some(o) => (o.clone() + 1),
                None => src_len.clone(),
            }
        };
        let line_end = match index
            .offsets
            .clone()
            .get((line.clone() - 1) as usize)
            .cloned()
        {
            Some(o) => o.clone(),
            None => src_len.clone(),
        };
        v1_rt::chars_to_string(&index.char_codes.clone(), line_start, line_end)
    }
}

pub fn source_text_at(index: Rc<NewlineIndex>, span: Rc<SourceSpan>) -> String {
    v1_rt::chars_to_string(
        &index.char_codes.clone(),
        span.start.clone(),
        span.end.clone(),
    )
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InternTable {
    pub strings: Rc<Vec<String>>,
    pub index: Rc<HashMap<String, i64>>,
    pub next_id: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InternResult {
    pub table: Rc<InternTable>,
    pub id: i64,
}

pub fn empty_intern_table() -> Rc<InternTable> {
    Rc::new(InternTable {
        strings: Rc::new(vec!["".to_string()]),
        index: v1_rt::rc_map_insert(v1_rt::rc_empty_map::<String, i64>(), "".to_string(), 0),
        next_id: 1,
    })
}

pub fn intern(table: Rc<InternTable>, s: String) -> Rc<InternResult> {
    // Borrow the index directly (Rc derefs to &HashMap) rather than cloning it.
    // The prior `&table.index.clone()` created a temporary Rc that lived for the
    // whole `match`, pinning the index map at refcount>=2 inside the miss arm and
    // forcing `rc_map_insert`'s `make_mut` to deep-copy it — the same O(T^2) trap
    // as the strings Vec. A plain borrow ends before the arms run, so the
    // uniquely-held map grows in place.
    match v1_rt::map_get(&table.index, s.clone()) {
        Some(id) => Rc::new(InternResult {
            table: table.clone(),
            id: id.clone(),
        }),
        None => {
            let id = table.next_id;
            // Amortized-O(1) intern: take ownership of the table's inner Rcs
            // instead of cloning them while `table` still aliases them. Cloning
            // (`table.strings.clone()`) pinned each Rc at refcount>=2, forcing
            // `rc_list_push`/`rc_map_insert`'s `Rc::make_mut` to deep-copy the
            // whole strings Vec + index map on every new string — O(T^2) over T
            // distinct strings, the dominant parse cost. Unwrapping the uniquely
            // held table hands its inner Rcs to the push/insert at refcount==1 so
            // `make_mut` grows them in place. Purity is invariant to ownership:
            // `next_id` increments identically and insertion order is unchanged,
            // so intern IDs are byte-preserved; if the table is shared we fall
            // back to a clone (today's deep-copy behavior — no speedup on that
            // path, never wrong IDs).
            let mut new_table = Rc::try_unwrap(table).unwrap_or_else(|shared| (*shared).clone());
            new_table.strings = v1_rt::rc_list_push(new_table.strings, s.clone());
            new_table.index = v1_rt::rc_map_insert(new_table.index, s, id);
            new_table.next_id = id + 1;
            Rc::new(InternResult {
                table: Rc::new(new_table),
                id,
            })
        }
    }
}

pub fn intern_str(table: Rc<InternTable>, id: i64) -> String {
    match table.strings.clone().get(id as usize).cloned() {
        Some(s) => s.clone(),
        None => "".to_string(),
    }
}

pub fn intern_find(table: Rc<InternTable>, s: String) -> Option<i64> {
    match v1_rt::map_get(&table.index.clone(), s) {
        Some(id) => Some(id.clone()),
        None => None,
    }
}

pub fn intern_find_or_empty(table: Rc<InternTable>, s: String) -> i64 {
    match v1_rt::map_get(&table.index.clone(), s) {
        Some(id) => id.clone(),
        None => 0,
    }
}

pub fn merge_intern_tables(tables: Rc<Vec<Rc<InternTable>>>) -> Rc<InternTable> {
    tables.iter().cloned().fold(
        empty_intern_table(),
        |merged: Rc<InternTable>, t: Rc<InternTable>| {
            t.strings
                .clone()
                .iter()
                .cloned()
                .fold(merged, |m: Rc<InternTable>, s: String| {
                    if (s.clone().as_str() == "".to_string().as_str()) {
                        m.clone()
                    } else {
                        intern(m.clone(), s.clone()).table.clone()
                    }
                })
        },
    )
}

pub fn is_internable_token(shape: TokenShape) -> bool {
    match shape {
        TokenShape::ShIdent => true,
        TokenShape::ShKeyword => true,
        _ => false,
    }
}

pub fn pre_intern_tokens(tokens: Rc<Vec<Rc<Token>>>, table: Rc<InternTable>) -> Rc<InternTable> {
    tokens
        .iter()
        .cloned()
        .fold(table, |t: Rc<InternTable>, tok: Rc<Token>| {
            if is_internable_token(tok.shape.clone()) {
                intern(t.clone(), tok.text.clone()).table.clone()
            } else {
                t.clone()
            }
        })
}

pub fn with_optional_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node {
        name: n.name.clone(),
        ident: n.ident.clone(),
        span: n.span.clone(),
        ident_span: n.ident_span.clone(),
        children: n.children.clone(),
        connective: n.connective.clone(),
        params: n.params.clone(),
        inferred: n.inferred.clone(),
        return_cardinality: Cardinality::CardOptional,
        uses: n.uses.clone(),
        body: n.body.clone(),
        transport: n.transport.clone(),
        properties: n.properties.clone(),
        type_annotation: n.type_annotation.clone(),
        is_self_recursive: n.is_self_recursive.clone(),
        has_non_tail_self_call: n.has_non_tail_self_call.clone(),
        match_pattern: n.match_pattern.clone(),
        expr_data: n.expr_data.clone(),
    })
}

pub fn with_required_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node {
        name: n.name.clone(),
        ident: n.ident.clone(),
        span: n.span.clone(),
        ident_span: n.ident_span.clone(),
        children: n.children.clone(),
        connective: n.connective.clone(),
        params: n.params.clone(),
        inferred: n.inferred.clone(),
        return_cardinality: Cardinality::Required,
        uses: n.uses.clone(),
        body: n.body.clone(),
        transport: n.transport.clone(),
        properties: n.properties.clone(),
        type_annotation: n.type_annotation.clone(),
        is_self_recursive: n.is_self_recursive.clone(),
        has_non_tail_self_call: n.has_non_tail_self_call.clone(),
        match_pattern: n.match_pattern.clone(),
        expr_data: n.expr_data.clone(),
    })
}
