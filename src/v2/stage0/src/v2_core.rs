
/// Source span for diagnostic reporting.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceSpan {
    pub start: i64,
    pub end: i64,
}

/// Type alias for file paths.
pub type FilePath = String;

/// Non-empty string type (alias — validation not enforced at type level).
pub type NonEmptyStr = String;

/// Binding power for Pratt parser precedence levels.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingPower {
    pub left: i64,
    pub right: i64,
}

use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;
pub type Map<K, V> = HashMap<K, V>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Token {
    pub text: String,
    pub span: SourceSpan,
    pub shape: TokenShape,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum TokenShape {
    #[default]
    ShKwModule,
    ShKwImport,
    ShKwType,
    ShKwFn,
    ShKwFunc,
    ShKwService,
    ShKwResource,
    ShKwData,
    ShKwExtern,
    ShKwInterface,
    ShKwPipeline,
    ShKwProfile,
    ShKwPattern,
    ShKwLet,
    ShKwReturn,
    ShKwMatch,
    ShKwIf,
    ShKwElse,
    ShKwFor,
    ShKwIn,
    ShKwWhere,
    ShKwWith,
    ShKwTrue,
    ShKwFalse,
    ShKwNone,
    ShKwAcquire,
    ShKwRelease,
    ShKwCapability,
    ShKwOperation,
    ShKwInput,
    ShKwOutput,
    ShKwIdempotent,
    ShKwReadonly,
    ShKwHermetic,
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

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum Connective {
    #[default]
    Conj,
    Disj,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum Cardinality {
    #[default]
    Required,
    CardOptional,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum CollectionKind {
    #[default]
    ListKind,
    SetKind,
    NonEmptyListKind,
    NonEmptySetKind,
    MapKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Field {
    pub name: String,
    pub type_expr: Rc<Node>,
    pub cardinality: Cardinality,
    pub default_value: Option<Rc<Node>>,
    pub from_key: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Variant {
    pub name: String,
    pub fields: Rc<Vec<Rc<Field>>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Param {
    pub name: String,
    pub type_expr: Rc<Node>,
    pub default_value: Option<Rc<Node>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceUse {
    pub name: String,
    pub resource: Rc<Node>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum FieldAccessStyle {
    #[default]
    StoredField,
    EnumAccessor,
    OptionalUnwrap,
    TupleFirst,
    TupleSecond,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum FieldValueShape {
    #[default]
    PlainValue,
    OptionalValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldSummary {
    pub access_style: FieldAccessStyle,
    pub value_shape: FieldValueShape,
}

pub static KERNEL_TYPES: &[&str] = &[
    "String",
    "Int",
    "Bool",
    "Float",
    "Secret",
    "Json",
    "Unit",
    "Bytes"
];

pub fn is_kernel_type(name: &str) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in Rc::new(KERNEL_TYPES.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        if __elem_1.clone() == name {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn is_kernel_numeric(name: &str) -> bool {
    (name == "Int") || (name == "Float")
}

pub fn is_kernel_textual(name: &str) -> bool {
    (name == "String") || (name == "Secret")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferredNode {
    Resolved { node: Rc<Node> },
    CompilerError { message: String, span: SourceSpan },
}

impl Default for InferredNode {
    fn default() -> Self {
        InferredNode::Resolved { node: Default::default() }
    }
}

pub fn inferred_to_node(inferred: Rc<InferredNode>) -> Option<Rc<Node>> {
    match inferred.as_ref() {
    InferredNode::Resolved { node: n, .. } => {
        Some(n.clone())
    }
    InferredNode::CompilerError { message: _, span: _, .. } => {
        None
    }
}
}

pub fn is_compiler_error(inferred: Rc<InferredNode>) -> bool {
    match inferred.as_ref() {
    InferredNode::Resolved { node: _, .. } => {
        false
    }
    InferredNode::CompilerError { message: _, span: _, .. } => {
        true
    }
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NodeType {
    Typed { node: Rc<Node> },
    InferError { message: String, span: SourceSpan },
    #[default]
    Untyped,
}

pub fn rt_node(n: Rc<Node>) -> Rc<NodeType> {
    if n.inferred.clone().is_none() {
    Rc::new(NodeType::Untyped)
} else {
    match n.inferred.clone().unwrap().as_ref() {
    InferredNode::Resolved { node: rt, .. } => {
        Rc::new(NodeType::Typed { node: rt.clone() })
    }
    InferredNode::CompilerError { message: m, span: s, .. } => {
        Rc::new(NodeType::InferError { message: m.clone(), span: s.clone() })
    }
}
}
}

pub fn has_inferred(n: Rc<Node>) -> bool {
    n.inferred.clone().is_some()
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum IntrinsicMethod {
    #[default]
    MethodCount,
    MethodJoin,
    MethodSplit,
    MethodLast,
    MethodFirst,
    MethodEnumerate,
    MethodChars,
    MethodStringContains,
    MethodConcat,
    MethodMap,
    MethodFilter,
    MethodAny,
    MethodAll,
    MethodFlatMap,
    MethodSkip,
    MethodTake,
    MethodFold,
    MethodSortBy,
    MethodAppend,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VarBindingKind {
    #[default]
    LocalValueBinding,
    FunctionValueBinding,
    VariantValueBinding { parent_enum: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum CallSemantics {
    #[default]
    PlainCallSemantics,
    LookupCallSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LambdaSemantics {
    pub param_types: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum RuntimeBridgeMethod {
    #[default]
    BridgeGet,
    BridgeWith,
    BridgeListPush,
    BridgeMapInsert,
    BridgeMapMerge,
    BridgeMapGet,
    BridgeMapHas,
    BridgeEmitMapHas,
    BridgeMapValues,
    BridgeMapKeys,
    BridgeMapContainsKey,
    BridgeCharAt,
    BridgeStringAt,
    BridgeStringLength,
    BridgeLength,
    BridgeStartsWith,
    BridgeEndsWith,
    BridgeToString,
    BridgeTrim,
    BridgeToLower,
    BridgeToUpper,
    BridgeReplace,
    BridgeSubstring,
    BridgeToInt,
    BridgeEmptyMap,
    BridgeContains,
    BridgeReverse,
    BridgeLookup,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MethodSemantics {
    #[default]
    PlainMethodSemantics,
    IntrinsicMethodSemantics { intrinsic: IntrinsicMethod, fold_accumulator_type: Option<Rc<Node>> },
    RuntimeBridgeSemantics { method: RuntimeBridgeMethod },
    ServiceMethodSemantics { service_name: String, op_params: Rc<Vec<Rc<Param>>> },
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum ExprErrorKind {
    #[default]
    ParseRecoveryError,
    SemanticExprError,
    InternalExprError,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TransportKind {
    #[default]
    LocalTransport,
    RestTransport,
    ShellTransport,
    FileTransport,
    CustomTransport { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConfigPropertyKey {
    #[default]
    ConfigBaseUrl,
    ConfigBasePath,
    ConfigAuthScheme,
    ConfigAuthHeader,
    ConfigAuthToken,
    ConfigOther { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExprData {
    #[default]
    NoExprData,
    ExprLiteral { value: Rc<LiteralValue> },
    ExprError { kind: ExprErrorKind, message: String },
    ExprVar { name: String, binding_kind: Option<Rc<VarBindingKind>> },
    ExprFieldAccess { field: String, summary: Option<Rc<FieldSummary>> },
    ExprCall { func: String, call_semantics: Option<CallSemantics> },
    ExprMethodCall { method: String, method_semantics: Option<Rc<MethodSemantics>> },
    ExprMatch,
    ExprIf,
    ExprLet { name: String },
    ExprRecordLit { type_name: Option<String>, parent_enum: Option<String> },
    ExprListLit,
    ExprBinOp { op: BinOpKind },
    ExprUnaryOp { op: UnaryOpKind },
    ExprLambda { params: Rc<Vec<String>>, semantics: Option<Rc<LambdaSemantics>> },
    ExprStringInterp,
    ExprBlock,
    ExprCast,
    ExprForEach { variable: String },
    ExprIndex,
    ExprSlice,
    ExprReturn,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamedArg {
    pub name: Option<String>,
    pub value: Rc<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchArm {
    pub pattern: Rc<MatchPattern>,
    pub guard: Option<Rc<Node>>,
    pub body: Rc<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldInit {
    pub name: String,
    pub value: Rc<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MatchPattern {
    Bind { name: String },
    LitPattern { value: Rc<LiteralValue> },
    VariantPattern { name: String, parent_enum: Option<String>, field_bindings: Rc<Vec<Rc<FieldBinding>>> },
    #[default]
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldBinding {
    pub field_name: String,
    pub binding: Rc<MatchPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LiteralValue {
    LitStr { value: String },
    LitInt { value: i64 },
    LitFloat { value: String },
    LitBool { value: bool },
    #[default]
    LitNull,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum BinOpKind {
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BinEq,
    BinNe,
    BinLt,
    BinGt,
    BinLe,
    BinGe,
    BinAnd,
    BinOr,
    NullCoalesce,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum UnaryOpKind {
    #[default]
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringPart {
    Text { value: String },
    Interpolation { expr: Rc<Node> },
}

impl Default for StringPart {
    fn default() -> Self {
        StringPart::Text { value: Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperationDef {
    pub name: String,
    pub inputs: Rc<Vec<Rc<Field>>>,
    pub outputs: Rc<Vec<Rc<Field>>>,
    pub response_props: Rc<Vec<Rc<FieldInit>>>,
    pub mock_props: Rc<Vec<Rc<FieldInit>>>,
    pub exit_props: Rc<Vec<Rc<FieldInit>>>,
    pub modifier_props: Rc<Vec<Rc<FieldInit>>>,
    pub transport: Option<Rc<Node>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum OperationModifier {
    #[default]
    Idempotent,
    Readonly,
    Hermetic,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapabilityDef {
    pub name: String,
    pub inputs: Rc<Vec<Rc<Field>>>,
    pub outputs: Rc<Vec<Rc<Field>>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompileResult {
    pub files: Rc<Vec<Rc<TextFile>>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeclaredFuncSig {
    pub name: String,
    pub params: Rc<Vec<Rc<Param>>>,
    pub inferred: Option<Rc<Node>>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeclaredFuncEnv {
    pub signatures: Rc<HashMap<String, Rc<DeclaredFuncSig>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Node {
    pub name: String,
    pub span: SourceSpan,
    pub children: Rc<Vec<Rc<Node>>>,
    pub connective: Option<Connective>,
    pub collection_kind: Option<CollectionKind>,
    pub params: Rc<Vec<Rc<Param>>>,
    pub inferred: Option<Rc<InferredNode>>,
    pub return_cardinality: Cardinality,
    pub uses: Rc<Vec<Rc<ResourceUse>>>,
    pub body: Option<Rc<Node>>,
    pub transport: Option<Rc<Node>>,
    pub properties: Rc<Vec<Rc<FieldInit>>>,
    pub type_annotation: Option<Rc<Node>>,
    pub is_self_recursive: bool,
    pub has_non_tail_self_call: bool,
    pub match_pattern: Option<Rc<MatchPattern>>,
    pub expr_data: Rc<ExprData>,
}

pub fn make_expr_node(expr_data: Rc<ExprData>, children: Rc<Vec<Rc<Node>>>, inferred: Option<Rc<InferredNode>>, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: "".to_string(), span, children, connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data })
}

pub fn make_expr_error_node(kind: ExprErrorKind, message: &str, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: "".to_string(), span: span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: Some(Rc::new(InferredNode::CompilerError { message: message.to_string(), span: span.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::ExprError { kind, message: message.to_string() }), match_pattern: None })
}

pub fn make_arg_node(name: Option<String>, value: Rc<Node>, span: SourceSpan) -> Rc<Node> {
    let arg_name = match name {
    Some(n) => {
        n
    }
    None => {
        "".to_string()
    }
};
    Rc::new(Node { name: arg_name, span, children: Rc::new(vec!(value)), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn make_arm_node(pattern: Rc<MatchPattern>, guard: Option<Rc<Node>>, body: Rc<Node>, span: SourceSpan) -> Rc<Node> {
    let children = match guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(g) => {
        let g = Rc::new(g.clone());
        Rc::new(vec!(g.clone(), body.clone()))
    }
    None => {
        Rc::new(vec!(body.clone()))
    }
};
    Rc::new(Node { name: "".to_string(), span, children, connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: Some(pattern), expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn make_field_init_node(name: &str, value: Rc<Node>, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span, children: Rc::new(vec!(value)), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn make_text_part_node(text: &str, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: "".to_string(), span, children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: text.to_string() }) }) })
}

pub fn make_interp_part_node(expr: Rc<Node>, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: "".to_string(), span, children: Rc::new(vec!(expr)), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn expr_child_at(texpr: Rc<Node>, index: i64, role: &str) -> Rc<Node> {
    match texpr.children.clone().get((index) as usize).cloned() {
    Some(v) => {
        v
    }
    None => {
        make_expr_error_node(ExprErrorKind::InternalExprError, &v2_rt::concat("malformed node: missing ".to_string(), role.to_string()), texpr.span.clone())
    }
}
}

pub fn arg_name(n: Rc<Node>) -> Option<String> {
    if n.name.clone() == "" {
    None
} else {
    Some(n.name.clone())
}
}

pub fn arg_value(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
    Some(v) => {
        v
    }
    None => {
        make_expr_error_node(ExprErrorKind::InternalExprError, "malformed arg: missing value", n.span.clone())
    }
}
}

pub fn arm_pattern(n: Rc<Node>) -> Rc<MatchPattern> {
    match n.match_pattern.as_ref().map(|__rc| __rc.as_ref()) {
    Some(p) => {
        let p = Rc::new(p.clone());
        p.clone()
    }
    None => {
        Rc::new(MatchPattern::Wildcard)
    }
}
}

pub fn arm_guard(n: Rc<Node>) -> Option<Rc<Node>> {
    if ({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 2_i64 {
    n.children.clone().first().cloned()
} else {
    None
}
}

pub fn arm_body(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().last().cloned() {
    Some(v) => {
        v
    }
    None => {
        make_expr_error_node(ExprErrorKind::InternalExprError, "malformed arm: missing body", n.span.clone())
    }
}
}

pub fn field_init_node_name(n: Rc<Node>) -> String {
    n.name.clone()
}

pub fn field_init_node_value(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
    Some(v) => {
        v
    }
    None => {
        make_expr_error_node(ExprErrorKind::InternalExprError, "malformed field-init: missing value", n.span.clone())
    }
}
}

pub fn if_condition(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "if condition")
}

pub fn if_then_branch(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1_i64, "if then-branch")
}

pub fn if_else_branch(texpr: Rc<Node>) -> Option<Rc<Node>> {
    texpr.children.clone().get((2_i64) as usize).cloned()
}

pub fn match_scrutinee(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "match scrutinee")
}

pub fn match_arm_nodes(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    { let __s = texpr.children.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) }
}

pub fn binop_left(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "binop left")
}

pub fn binop_right(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1_i64, "binop right")
}

pub fn unaryop_operand(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "unaryop operand")
}

pub fn field_access_base(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "field access base")
}

pub fn method_receiver(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "method receiver")
}

pub fn method_arg_nodes(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    { let __s = texpr.children.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) }
}

pub fn call_arg_nodes(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    texpr.children.clone()
}

pub fn lambda_body(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "lambda body")
}

pub fn let_value(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "let value")
}

pub fn let_body(texpr: Rc<Node>) -> Option<Rc<Node>> {
    texpr.children.clone().get((1_i64) as usize).cloned()
}

pub fn cast_expr(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "cast expr")
}

pub fn cast_target(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1_i64, "cast target")
}

pub fn foreach_collection(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "foreach collection")
}

pub fn foreach_body(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1_i64, "foreach body")
}

pub fn index_base(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "index base")
}

pub fn index_expr(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1_i64, "index expression")
}

pub fn slice_base(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "slice base")
}

pub fn slice_start(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 1_i64, "slice start")
}

pub fn slice_end(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 2_i64, "slice end")
}

pub fn return_value(texpr: Rc<Node>) -> Rc<Node> {
    expr_child_at(texpr, 0_i64, "return value")
}

pub fn block_stmts(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    texpr.children.clone()
}

pub fn record_field_nodes(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    texpr.children.clone()
}

pub fn list_elements(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    texpr.children.clone()
}

pub fn string_interp_parts(texpr: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    texpr.children.clone()
}

pub fn make_transport_node(name: &str, properties: Rc<Vec<Rc<FieldInit>>>, children: Rc<Vec<Rc<Node>>>, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span, children, connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties, type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn local_transport_node(span: SourceSpan) -> Rc<Node> {
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::LocalTransport)), Rc::new(Vec::new()), Rc::new(Vec::new()), span)
}

pub fn rest_transport_node(base_url: Rc<Node>, auth_props: Rc<Vec<Rc<FieldInit>>>, headers: Rc<Vec<Rc<FieldInit>>>, span: SourceSpan) -> Rc<Node> {
    let url_field = Rc::new(FieldInit { name: config_property_name(Rc::new(ConfigPropertyKey::ConfigBaseUrl)), value: base_url });
    let props = v2_rt::concat(v2_rt::concat(Rc::new(vec!(url_field)), auth_props), headers);
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::RestTransport)), props, Rc::new(Vec::new()), span)
}

pub fn shell_transport_node(argv: Rc<Vec<Rc<Node>>>, env: Rc<Vec<Rc<FieldInit>>>, span: SourceSpan) -> Rc<Node> {
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::ShellTransport)), env, argv, span)
}

pub fn file_transport_node(base_path: Rc<Node>, span: SourceSpan) -> Rc<Node> {
    let path_field = Rc::new(FieldInit { name: config_property_name(Rc::new(ConfigPropertyKey::ConfigBasePath)), value: base_path });
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::FileTransport)), Rc::new(vec!(path_field)), Rc::new(Vec::new()), span)
}

pub fn find_property(props: Rc<Vec<Rc<FieldInit>>>, prop_name: &str) -> Option<Rc<Node>> {
    match {
    let mut __found_2 = None;
    for __elem_3 in props.iter().cloned() {
        if __elem_3.name.clone() == prop_name {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(fi) => {
        Some(fi.value.clone())
    }
    None => {
        None
    }
}
}

pub fn find_property_string(props: Rc<Vec<Rc<FieldInit>>>, prop_name: &str) -> Option<String> {
    match find_property(props, &prop_name) {
    Some(n) => {
        match n.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitStr { value: s, .. } = value.as_ref() else { unreachable!() };
        Some(s.clone())
    }
    _ => {
        None
    }
}
    }
    None => {
        None
    }
}
}

pub fn config_property_name(key: Rc<ConfigPropertyKey>) -> String {
    match key.as_ref() {
    ConfigPropertyKey::ConfigBaseUrl => {
        "base_url".to_string()
    }
    ConfigPropertyKey::ConfigBasePath => {
        "base_path".to_string()
    }
    ConfigPropertyKey::ConfigAuthScheme => {
        "auth_scheme".to_string()
    }
    ConfigPropertyKey::ConfigAuthHeader => {
        "auth_header".to_string()
    }
    ConfigPropertyKey::ConfigAuthToken => {
        "auth_token".to_string()
    }
    ConfigPropertyKey::ConfigOther { name, .. } => {
        name.clone()
    }
}
}

pub fn config_property_key(name: &str) -> Rc<ConfigPropertyKey> {
    if name == "base_url" {
    Rc::new(ConfigPropertyKey::ConfigBaseUrl)
} else {
    if name == "base_path" {
    Rc::new(ConfigPropertyKey::ConfigBasePath)
} else {
    if name == "auth_scheme" {
    Rc::new(ConfigPropertyKey::ConfigAuthScheme)
} else {
    if name == "auth_header" {
    Rc::new(ConfigPropertyKey::ConfigAuthHeader)
} else {
    if name == "auth_token" {
    Rc::new(ConfigPropertyKey::ConfigAuthToken)
} else {
    Rc::new(ConfigPropertyKey::ConfigOther { name: name.to_string() })
}
}
}
}
}
}

pub fn transport_kind_name(kind: Rc<TransportKind>) -> String {
    match kind.as_ref() {
    TransportKind::LocalTransport => {
        "local".to_string()
    }
    TransportKind::RestTransport => {
        "rest".to_string()
    }
    TransportKind::ShellTransport => {
        "shell".to_string()
    }
    TransportKind::FileTransport => {
        "file".to_string()
    }
    TransportKind::CustomTransport { name, .. } => {
        name.clone()
    }
}
}

pub fn transport_kind(t: Rc<Node>) -> Rc<TransportKind> {
    if t.name.clone() == "local" {
    Rc::new(TransportKind::LocalTransport)
} else {
    if t.name.clone() == "rest" {
    Rc::new(TransportKind::RestTransport)
} else {
    if t.name.clone() == "shell" {
    Rc::new(TransportKind::ShellTransport)
} else {
    if t.name.clone() == "file" {
    Rc::new(TransportKind::FileTransport)
} else {
    Rc::new(TransportKind::CustomTransport { name: t.name.clone() })
}
}
}
}
}

pub fn is_transport_kind(t: Rc<Node>, kind: Rc<TransportKind>) -> bool {
    transport_kind_name(transport_kind(t)) == transport_kind_name(kind)
}

pub fn field_init_operation_modifier(field_init: Rc<FieldInit>) -> Option<OperationModifier> {
    if field_init.name.clone() == "idempotent" {
    Some(OperationModifier::Idempotent)
} else {
    if field_init.name.clone() == "readonly" {
    Some(OperationModifier::Readonly)
} else {
    if field_init.name.clone() == "hermetic" {
    Some(OperationModifier::Hermetic)
} else {
    None
}
}
}
}

pub fn operation_modifier_name(modifier: OperationModifier) -> String {
    match modifier {
    OperationModifier::Idempotent => {
        "idempotent".to_string()
    }
    OperationModifier::Readonly => {
        "readonly".to_string()
    }
    OperationModifier::Hermetic => {
        "hermetic".to_string()
    }
}
}

pub fn transport_base_url(t: Rc<Node>) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), &config_property_name(Rc::new(ConfigPropertyKey::ConfigBaseUrl)))
}

pub fn transport_auth_token(t: Rc<Node>) -> Option<Rc<Node>> {
    find_property(t.properties.clone(), &config_property_name(Rc::new(ConfigPropertyKey::ConfigAuthToken)))
}

pub fn transport_auth_header_name(t: Rc<Node>) -> Option<String> {
    find_property_string(t.properties.clone(), &config_property_name(Rc::new(ConfigPropertyKey::ConfigAuthHeader)))
}

pub fn transport_has_auth(t: Rc<Node>) -> bool {
    match find_property(t.properties.clone(), &config_property_name(Rc::new(ConfigPropertyKey::ConfigAuthToken))) {
    Some(_) => {
        true
    }
    None => {
        false
    }
}
}

pub fn transport_headers(t: Rc<Node>) -> Rc<Vec<Rc<FieldInit>>> {
    {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in t.properties.iter().cloned() {
        {
let __cond = {
    let key = config_property_key(&__elem_1.name);
    ((((key.clone() != Rc::new(ConfigPropertyKey::ConfigBaseUrl)) && (key.clone() != Rc::new(ConfigPropertyKey::ConfigBasePath))) && (key.clone() != Rc::new(ConfigPropertyKey::ConfigAuthScheme))) && (key.clone() != Rc::new(ConfigPropertyKey::ConfigAuthHeader))) && (key.clone() != Rc::new(ConfigPropertyKey::ConfigAuthToken))
};
if __cond {
    __filtered_0.push(__elem_1);
}
};
    }
    Rc::new(__filtered_0)
}
}

pub fn transport_env(t: Rc<Node>) -> Rc<Vec<Rc<FieldInit>>> {
    t.properties.clone()
}

pub fn map_children(node: Rc<Node>, transform: impl Fn(Rc<Node>) -> Rc<Node>) -> Rc<Node> {
    {
    let __rc_1 = node;
    let mut __owned_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| { debug_assert!(false, "V5: expected sole ownership of `node`"); (*rc).clone() });
    let __taken_2 = std::mem::take(&mut __owned_0.children);
    __owned_0.children = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in __taken_2.iter().cloned() {
        __mapped_3.push(transform(__elem_4.clone()));
    }
    Rc::new(__mapped_3)
};
    Rc::new(__owned_0)
}
}

pub fn expr_has_self_call(texpr: Rc<Node>, fn_name: &str) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, call_semantics: _, .. } => {
        if f.clone() == fn_name {
    true
} else {
    {
    let mut __any_0 = false;
    for __elem_1 in texpr.children.iter().cloned() {
        if expr_has_self_call(__elem_1.clone(), &fn_name) {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}
    }
    _ => {
        {
    let mut __any_2 = false;
    for __elem_3 in texpr.children.iter().cloned() {
        if expr_has_self_call(__elem_3.clone(), &fn_name) {
    __any_2 = true;
    break;
};
    }
    __any_2
}
    }
}
    })
}

pub fn expr_has_non_tail_self_call(texpr: Rc<Node>, fn_name: &str, in_tail: bool) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, call_semantics: _, .. } => {
        if f.clone() == fn_name {
    if in_tail.clone() == false {
    true
} else {
    {
    let mut __any_0 = false;
    for __elem_1 in texpr.children.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_1.clone(), &fn_name, false) {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}
} else {
    {
    let mut __any_2 = false;
    for __elem_3 in texpr.children.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_3.clone(), &fn_name, false) {
    __any_2 = true;
    break;
};
    }
    __any_2
}
}
    }
    ExprData::ExprError { kind: _, message: _, .. } => {
        false
    }
    ExprData::ExprVar { name: _, binding_kind: _, .. } => {
        false
    }
    ExprData::ExprLiteral { value: _, .. } => {
        false
    }
    ExprData::ExprFieldAccess { field: _, summary: _, .. } => {
        {
    let mut __any_4 = false;
    for __elem_5 in texpr.children.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_5.clone(), &fn_name, false) {
    __any_4 = true;
    break;
};
    }
    __any_4
}
    }
    ExprData::ExprMethodCall { method: _, method_semantics: _, .. } => {
        {
    let mut __any_6 = false;
    for __elem_7 in texpr.children.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_7.clone(), &fn_name, false) {
    __any_6 = true;
    break;
};
    }
    __any_6
}
    }
    ExprData::ExprIf => {
        {
    let cond_bad = expr_has_non_tail_self_call(if_condition(texpr.clone()), &fn_name, false);
    let then_bad = expr_has_non_tail_self_call(if_then_branch(texpr.clone()), &fn_name, in_tail.clone());
    let else_bad = match if_else_branch(texpr.clone()) {
    Some(e) => {
        expr_has_non_tail_self_call(e, &fn_name, in_tail.clone())
    }
    None => {
        false
    }
};
    (cond_bad || then_bad) || else_bad
}
    }
    ExprData::ExprMatch => {
        {
    let scrut_bad = expr_has_non_tail_self_call(match_scrutinee(texpr.clone()), &fn_name, false);
    let arms_bad = {
    let mut __any_8 = false;
    for __elem_9 in match_arm_nodes(texpr.clone()).iter().cloned() {
        if expr_has_non_tail_self_call(arm_body(__elem_9.clone()), &fn_name, in_tail.clone()) {
    __any_8 = true;
    break;
};
    }
    __any_8
};
    scrut_bad || arms_bad
}
    }
    ExprData::ExprLet { name: _, .. } => {
        {
    let val_bad = expr_has_non_tail_self_call(let_value(texpr.clone()), &fn_name, false);
    let body_bad = match let_body(texpr.clone()) {
    Some(b) => {
        expr_has_non_tail_self_call(b, &fn_name, in_tail.clone())
    }
    None => {
        false
    }
};
    val_bad || body_bad
}
    }
    ExprData::ExprBlock => {
        {
    let ss = texpr.children.clone();
    let ss_count = {
    let __len_10 = ss.clone().len();
    __len_10 as i64
};
    if ss_count.clone() == 0_i64 {
    false
} else {
    let init_bad = {
    let mut __any_16 = false;
    for __elem_17 in ({
    let mut __filtered_14 = Vec::new();
    for __elem_15 in ({
    let mut __enumerated_11 = Vec::new();
    for (__idx_12, __elem_13) in ss.clone().iter().enumerate() {
        __enumerated_11.push((__idx_12 as i64, __elem_13.clone()));
    }
    Rc::new(__enumerated_11)
}).iter().cloned() {
        if __elem_15.0.clone() < (ss_count.clone() - 1_i64) {
    __filtered_14.push(__elem_15);
};
    }
    Rc::new(__filtered_14)
}).iter().cloned() {
        if expr_has_non_tail_self_call(__elem_17.1.clone(), &fn_name, false) {
    __any_16 = true;
    break;
};
    }
    __any_16
};
    let last_bad = match ss.clone().last().cloned() {
    Some(last_expr) => {
        expr_has_non_tail_self_call(last_expr, &fn_name, in_tail.clone())
    }
    None => {
        false
    }
};
    init_bad || last_bad
}
}
    }
    ExprData::NoExprData => {
        {
    let mut __any_18 = false;
    for __elem_19 in texpr.children.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_19.clone(), &fn_name, in_tail.clone()) {
    __any_18 = true;
    break;
};
    }
    __any_18
}
    }
    _ => {
        {
    let mut __any_20 = false;
    for __elem_21 in texpr.children.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_21.clone(), &fn_name, false) {
    __any_20 = true;
    break;
};
    }
    __any_20
}
    }
}
    })
}

pub fn diagnostic_node(severity: &str, message: &str, span: SourceSpan, module_name: Option<String>, category: Option<String>) -> Rc<Node> {
    let sev_prop = Rc::new(FieldInit { name: "severity".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: severity.to_string() }) }), Rc::new(Vec::new()), None, SourceSpan { start: 0_i64, end: 0_i64 }) });
    let mod_prop = match module_name {
    Some(mn) => {
        Rc::new(vec!(Rc::new(FieldInit { name: "module_name".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: mn }) }), Rc::new(Vec::new()), None, SourceSpan { start: 0_i64, end: 0_i64 }) })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let cat_prop = match category {
    Some(cat) => {
        Rc::new(vec!(Rc::new(FieldInit { name: "category".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: cat }) }), Rc::new(Vec::new()), None, SourceSpan { start: 0_i64, end: 0_i64 }) })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(Node { name: message.to_string(), span, children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: v2_rt::concat(v2_rt::concat(Rc::new(vec!(sev_prop)), mod_prop), cat_prop), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn is_diagnostic_node(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "severity" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn diagnostic_is_error(n: Rc<Node>) -> bool {
    diagnostic_severity(n) == "error"
}

pub fn diagnostic_severity(n: Rc<Node>) -> String {
    let sev_prop = {
    let mut __found_2 = None;
    for __elem_3 in n.properties.iter().cloned() {
        if __elem_3.name.clone() == "severity" {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
};
    match sev_prop {
    Some(prop) => {
        match prop.value.expr_data.as_ref() {
    ExprData::ExprLiteral { value: lit, .. } => {
        match lit.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        s.clone()
    }
    _ => {
        "error".to_string()
    }
}
    }
    _ => {
        "error".to_string()
    }
}
    }
    None => {
        "error".to_string()
    }
}
}

pub fn diagnostic_message(n: Rc<Node>) -> String {
    n.name.clone()
}

pub fn diagnostic_module_name(n: Rc<Node>) -> Option<String> {
    let mod_prop = {
    let mut __found_2 = None;
    for __elem_3 in n.properties.iter().cloned() {
        if __elem_3.name.clone() == "module_name" {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
};
    match mod_prop {
    Some(prop) => {
        match prop.value.expr_data.as_ref() {
    ExprData::ExprLiteral { value: lit, .. } => {
        match lit.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        Some(s.clone())
    }
    _ => {
        None
    }
}
    }
    _ => {
        None
    }
}
    }
    None => {
        None
    }
}
}

pub fn diagnostic_category(n: Rc<Node>) -> Option<String> {
    let cat_prop = {
    let mut __found_2 = None;
    for __elem_3 in n.properties.iter().cloned() {
        if __elem_3.name.clone() == "category" {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
};
    match cat_prop {
    Some(prop) => {
        match prop.value.expr_data.as_ref() {
    ExprData::ExprLiteral { value: lit, .. } => {
        match lit.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        Some(s.clone())
    }
    _ => {
        None
    }
}
    }
    _ => {
        None
    }
}
    }
    None => {
        None
    }
}
}

pub fn diagnostic_span(n: Rc<Node>) -> SourceSpan {
    n.span.clone()
}

pub fn service_config_properties(endpoint: Rc<Node>, auth: Option<Rc<Node>>, rate_limit: Option<Rc<Node>>, retry: Option<Rc<Node>>) -> Rc<Vec<Rc<FieldInit>>> {
    let ep_prop = Rc::new(vec!(Rc::new(FieldInit { name: "svc_endpoint".to_string(), value: endpoint })));
    let auth_prop = match auth.as_ref().map(|__rc| __rc.as_ref()) {
    Some(a) => {
        let a = Rc::new(a.clone());
        Rc::new(vec!(Rc::new(FieldInit { name: "svc_auth".to_string(), value: a.clone() })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let rate_prop = match rate_limit.as_ref().map(|__rc| __rc.as_ref()) {
    Some(r) => {
        let r = Rc::new(r.clone());
        Rc::new(vec!(Rc::new(FieldInit { name: "svc_rate_limit".to_string(), value: r.clone() })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let retry_prop = match retry.as_ref().map(|__rc| __rc.as_ref()) {
    Some(r) => {
        let r = Rc::new(r.clone());
        Rc::new(vec!(Rc::new(FieldInit { name: "svc_retry".to_string(), value: r.clone() })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(ep_prop, auth_prop), rate_prop), retry_prop)
}

pub fn has_service_config(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "svc_endpoint" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn service_config_endpoint(n: Rc<Node>) -> Option<Rc<Node>> {
    find_property(n.properties.clone(), "svc_endpoint")
}

pub fn service_config_auth(n: Rc<Node>) -> Option<Rc<Node>> {
    find_property(n.properties.clone(), "svc_auth")
}

pub fn service_config_rate_limit(n: Rc<Node>) -> Option<Rc<Node>> {
    find_property(n.properties.clone(), "svc_rate_limit")
}

pub fn service_config_retry(n: Rc<Node>) -> Option<Rc<Node>> {
    find_property(n.properties.clone(), "svc_retry")
}

pub fn module_node(name: &str, imports: Rc<Vec<Rc<Node>>>, items: Rc<Vec<Rc<Node>>>, span: SourceSpan) -> Rc<Node> {
    let marker = Rc::new(FieldInit { name: "__is_module".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "true".to_string() }) }), Rc::new(Vec::new()), None, SourceSpan { start: 0_i64, end: 0_i64 }) });
    Rc::new(Node { name: name.to_string(), span, children: v2_rt::concat(imports, items), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(vec!(marker)), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn import_node(module_path: &str, is_all: bool, specific_names: Rc<Vec<String>>, span: SourceSpan) -> Rc<Node> {
    let import_prop = Rc::new(FieldInit { name: "__is_import".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "true".to_string() }) }), Rc::new(Vec::new()), None, SourceSpan { start: 0_i64, end: 0_i64 }) });
    let all_prop = if is_all {
    Rc::new(vec!(Rc::new(FieldInit { name: "__import_all".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "true".to_string() }) }), Rc::new(Vec::new()), None, SourceSpan { start: 0_i64, end: 0_i64 }) })))
} else {
    Rc::new(Vec::new())
};
    let name_children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in specific_names.iter().cloned() {
        __mapped_0.push(Rc::new(Node { name: __elem_1.clone(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }));
    }
    Rc::new(__mapped_0)
};
    Rc::new(Node { name: module_path.to_string(), span, children: name_children, connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: v2_rt::concat(Rc::new(vec!(import_prop)), all_prop), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn is_module_node(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "__is_module" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn is_import_node(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "__is_import" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn import_is_all(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "__import_all" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn import_specific_names(n: Rc<Node>) -> Rc<Vec<String>> {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in n.children.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
}
}

pub fn module_imports(n: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in n.children.iter().cloned() {
        if is_import_node(__elem_1.clone()) {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
}
}

pub fn module_items(n: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in n.children.iter().cloned() {
        if is_import_node(__elem_1.clone()) == false {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
}
}

pub fn is_token_node(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "__is_token" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn leaf_node(name: &str) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn no_span() -> SourceSpan {
    SourceSpan { start: 0_i64, end: 0_i64 }
}

pub fn with_optional_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: Cardinality::CardOptional, uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), match_pattern: n.match_pattern.clone(), expr_data: n.expr_data.clone() })
}

pub fn with_required_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: Cardinality::Required, uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), match_pattern: n.match_pattern.clone(), expr_data: n.expr_data.clone() })
}

pub fn node_is_product(n: Rc<Node>) -> bool {
    (n.connective.clone().is_some()) && (n.connective.clone() == Some(Connective::Conj))
}

pub fn node_is_coproduct(n: Rc<Node>) -> bool {
    (n.connective.clone().is_some()) && (n.connective.clone() == Some(Connective::Disj))
}

pub fn node_has_structure(n: Rc<Node>) -> bool {
    node_is_product(n.clone()) || node_is_coproduct(n.clone())
}

