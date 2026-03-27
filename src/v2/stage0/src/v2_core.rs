
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Module {
    pub name: String,
    pub imports: Rc<Vec<Rc<Import>>>,
    pub items: Rc<Vec<Rc<Node>>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ImportNames {
    #[default]
    ImportAll,
    ImportSpecific { names: Rc<Vec<String>> },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Import {
    pub module_path: String,
    pub names: Rc<ImportNames>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum Connective {
    Conj,
    Disj,
    #[default]
    NoConnective,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum Cardinality {
    #[default]
    Required,
    CardOptional,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum CollectionKind {
    ListKind,
    SetKind,
    NonEmptyListKind,
    NonEmptySetKind,
    MapKind,
    #[default]
    NoCollection,
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

pub fn node_has_compiler_error(n: &Node) -> bool {
    matches!(n.inferred.as_ref().map(|rc| rc.as_ref()), Some(InferredNode::CompilerError { .. }))
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
    ExprFieldAccess { base: Rc<Node>, field: String, summary: Option<Rc<FieldSummary>> },
    ExprCall { func: String, args: Rc<Vec<Rc<NamedArg>>>, call_semantics: Option<CallSemantics> },
    ExprMethodCall { receiver: Rc<Node>, method: String, args: Rc<Vec<Rc<NamedArg>>>, method_semantics: Option<Rc<MethodSemantics>> },
    ExprMatch { scrutinee: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>> },
    ExprIf { condition: Rc<Node>, then_branch: Rc<Node>, else_branch: Option<Rc<Node>> },
    ExprLet { name: String, value: Rc<Node>, body: Option<Rc<Node>> },
    ExprRecordLit { type_name: Option<String>, fields: Rc<Vec<Rc<FieldInit>>>, parent_enum: Option<String> },
    ExprListLit { elements: Rc<Vec<Rc<Node>>> },
    ExprBinOp { op: BinOpKind, left: Rc<Node>, right: Rc<Node> },
    ExprUnaryOp { op: UnaryOpKind, operand: Rc<Node> },
    ExprLambda { params: Rc<Vec<String>>, body: Rc<Node>, semantics: Option<Rc<LambdaSemantics>> },
    ExprStringInterp { parts: Rc<Vec<Rc<StringPart>>> },
    ExprBlock { stmts: Rc<Vec<Rc<Node>>> },
    ExprCast { expr: Rc<Node>, target: Rc<Node> },
    ExprForEach { variable: String, collection: Rc<Node>, body: Rc<Node> },
    ExprIndex { base: Rc<Node>, index: Rc<Node> },
    ExprSlice { base: Rc<Node>, start: Rc<Node>, end: Rc<Node> },
    ExprReturn { value: Rc<Node> },
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
pub struct ServiceConfig {
    pub endpoint: Rc<Node>,
    pub auth: Option<Rc<Node>>,
    pub rate_limit: Option<Rc<Node>>,
    pub retry: Option<Rc<Node>>,
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
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum ErrorCategory {
    #[default]
    UnresolvedName,
    TypeMismatch,
    FieldNotFound,
    VariantNotFound,
    AmbiguousCall,
    InvalidOperation,
    CascadeError,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Option<SourceSpan>,
    pub module_name: Option<String>,
    pub category: Option<ErrorCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum Severity {
    #[default]
    Error,
    Warning,
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
    pub connective: Connective,
    pub collection_kind: CollectionKind,
    pub params: Rc<Vec<Rc<Param>>>,
    pub inferred: Option<Rc<InferredNode>>,
    pub return_cardinality: Cardinality,
    pub uses: Rc<Vec<Rc<ResourceUse>>>,
    pub body: Option<Rc<Node>>,
    pub transport: Option<Rc<Node>>,
    pub properties: Rc<Vec<Rc<FieldInit>>>,
    pub type_annotation: Option<Rc<Node>>,
    pub config: Option<Rc<ServiceConfig>>,
    pub is_self_recursive: bool,
    pub has_non_tail_self_call: bool,
    pub expr_data: Rc<ExprData>,
}

pub fn make_expr_node(expr_data: Rc<ExprData>, inferred: Option<Rc<InferredNode>>, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: "".to_string(), span, children: Rc::new(Vec::new()), connective: Connective::NoConnective, collection_kind: CollectionKind::NoCollection, params: Rc::new(Vec::new()), inferred: inferred.clone(), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: expr_data.clone() })
}

pub fn make_expr_error_node(kind: ExprErrorKind, message: &str, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: "".to_string(), span: span.clone(), children: Rc::new(Vec::new()), connective: Connective::NoConnective, collection_kind: CollectionKind::NoCollection, params: Rc::new(Vec::new()), inferred: Some(Rc::new(InferredNode::CompilerError { message: message.to_string(), span: span.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::ExprError { kind, message: message.to_string() }) })
}

pub fn make_transport_node(name: &str, properties: Rc<Vec<Rc<FieldInit>>>, children: Rc<Vec<Rc<Node>>>, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span, children: children.clone(), connective: Connective::NoConnective, collection_kind: CollectionKind::NoCollection, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: properties.clone(), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn local_transport_node(span: SourceSpan) -> Rc<Node> {
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::LocalTransport)), Rc::new(Vec::new()), Rc::new(Vec::new()), span)
}

pub fn rest_transport_node(base_url: Rc<Node>, auth_props: Rc<Vec<Rc<FieldInit>>>, headers: Rc<Vec<Rc<FieldInit>>>, span: SourceSpan) -> Rc<Node> {
    let url_field = Rc::new(FieldInit { name: config_property_name(Rc::new(ConfigPropertyKey::ConfigBaseUrl)), value: base_url.clone() });
    let props = v2_rt::concat(v2_rt::concat(Rc::new(vec!(url_field.clone())), auth_props.clone()), headers.clone());
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::RestTransport)), props.clone(), Rc::new(Vec::new()), span)
}

pub fn shell_transport_node(argv: Rc<Vec<Rc<Node>>>, env: Rc<Vec<Rc<FieldInit>>>, span: SourceSpan) -> Rc<Node> {
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::ShellTransport)), env.clone(), argv.clone(), span)
}

pub fn file_transport_node(base_path: Rc<Node>, span: SourceSpan) -> Rc<Node> {
    let path_field = Rc::new(FieldInit { name: config_property_name(Rc::new(ConfigPropertyKey::ConfigBasePath)), value: base_path.clone() });
    make_transport_node(&transport_kind_name(Rc::new(TransportKind::FileTransport)), Rc::new(vec!(path_field.clone())), Rc::new(Vec::new()), span)
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
    match find_property(props.clone(), &prop_name) {
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
    transport_kind_name(transport_kind(t.clone())) == transport_kind_name(kind.clone())
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

pub fn expr_children(node: Rc<Node>) -> Rc<Vec<Rc<Node>>> {
    match node.expr_data.as_ref() {
    ExprData::ExprFieldAccess { base: b, field: _, summary: _, .. } => {
        Rc::new(vec!(b.clone()))
    }
    ExprData::ExprCall { func: _, args: a, call_semantics: _, .. } => {
        {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in a.iter().cloned() {
        __mapped_0.push(__elem_1.value.clone());
    }
    Rc::new(__mapped_0)
}
    }
    ExprData::ExprMethodCall { receiver: r, method: _, args: a, method_semantics: _, .. } => {
        v2_rt::concat(Rc::new(vec!(r.clone())), {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in a.iter().cloned() {
        __mapped_2.push(__elem_3.value.clone());
    }
    Rc::new(__mapped_2)
})
    }
    ExprData::ExprMatch { scrutinee: s, arms, .. } => {
        {
    let arm_nodes = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in arms.iter().cloned() {
        __flat_mapped_4.extend(({
    let guard_nodes = match __elem_5.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(g) => {
        let g = Rc::new(g.clone());
        Rc::new(vec!(g.clone()))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    {
    let __rc_7 = guard_nodes;
    let mut __appended_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __appended_6.push(__elem_5.body.clone());
    Rc::new(__appended_6)
}
}).iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    v2_rt::concat(Rc::new(vec!(s.clone())), arm_nodes.clone())
}
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(el) => {
        let el = Rc::new(el.clone());
        Rc::new(vec!(c.clone(), t.clone(), el.clone()))
    }
    None => {
        Rc::new(vec!(c.clone(), t.clone()))
    }
}
    }
    ExprData::ExprLet { name: _, value: v, body: b, .. } => {
        match b.as_ref().map(|__rc| __rc.as_ref()) {
    Some(body) => {
        let body = Rc::new(body.clone());
        Rc::new(vec!(v.clone(), body.clone()))
    }
    None => {
        Rc::new(vec!(v.clone()))
    }
}
    }
    ExprData::ExprRecordLit { type_name: _, fields: fs, parent_enum: _, .. } => {
        {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in fs.iter().cloned() {
        __mapped_8.push(__elem_9.value.clone());
    }
    Rc::new(__mapped_8)
}
    }
    ExprData::ExprListLit { elements: els, .. } => {
        els.clone()
    }
    ExprData::ExprBinOp { op: _, left: l, right: r, .. } => {
        Rc::new(vec!(l.clone(), r.clone()))
    }
    ExprData::ExprUnaryOp { op: _, operand: o, .. } => {
        Rc::new(vec!(o.clone()))
    }
    ExprData::ExprLambda { params: _, body: b, semantics: _, .. } => {
        Rc::new(vec!(b.clone()))
    }
    ExprData::ExprStringInterp { parts: ps, .. } => {
        {
    let mut __flat_mapped_10 = Vec::new();
    for __elem_11 in ps.iter().cloned() {
        __flat_mapped_10.extend((match __elem_11.as_ref() {
    StringPart::Interpolation { expr: e, .. } => {
        Rc::new(vec!(e.clone()))
    }
    StringPart::Text { value: _, .. } => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_10)
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        ss.clone()
    }
    ExprData::ExprCast { expr: e, target: t, .. } => {
        Rc::new(vec!(e.clone(), t.clone()))
    }
    ExprData::ExprForEach { variable: _, collection: c, body: b, .. } => {
        Rc::new(vec!(c.clone(), b.clone()))
    }
    ExprData::ExprIndex { base: b, index: i, .. } => {
        Rc::new(vec!(b.clone(), i.clone()))
    }
    ExprData::ExprSlice { base: b, start: s, end: e, .. } => {
        Rc::new(vec!(b.clone(), s.clone(), e.clone()))
    }
    ExprData::ExprReturn { value: v, .. } => {
        Rc::new(vec!(v.clone()))
    }
    _ => {
        Rc::new(Vec::new())
    }
}
}

pub fn with_expr_data(node: Rc<Node>, expr_data: Rc<ExprData>) -> Rc<Node> {
    Rc::new(Node { name: node.name.clone(), span: node.span.clone(), children: node.children.clone(), connective: node.connective.clone(), collection_kind: node.collection_kind.clone(), params: node.params.clone(), inferred: node.inferred.clone(), return_cardinality: node.return_cardinality.clone(), uses: node.uses.clone(), body: node.body.clone(), transport: node.transport.clone(), properties: node.properties.clone(), type_annotation: node.type_annotation.clone(), config: node.config.clone(), is_self_recursive: node.is_self_recursive.clone(), has_non_tail_self_call: node.has_non_tail_self_call.clone(), expr_data: expr_data.clone() })
}

pub fn map_expr_children(expr_node: Rc<Node>, transform: impl Fn(Rc<Node>) -> Rc<Node>) -> Rc<Node> {
    match expr_node.expr_data.as_ref() {
    ExprData::ExprFieldAccess { base: b, field: f, summary, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprFieldAccess { base: transform(b.clone()), field: f.clone(), summary: summary.clone() }))
    }
    ExprData::ExprCall { func, args, call_semantics, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprCall { func: func.clone(), args: {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(Rc::new(NamedArg { name: __elem_1.name.clone(), value: transform(__elem_1.value.clone()) }));
    }
    Rc::new(__mapped_0)
}, call_semantics: call_semantics.clone() }))
    }
    ExprData::ExprMethodCall { receiver, method, args, method_semantics, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprMethodCall { receiver: transform(receiver.clone()), method: method.clone(), args: {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in args.iter().cloned() {
        __mapped_2.push(Rc::new(NamedArg { name: __elem_3.name.clone(), value: transform(__elem_3.value.clone()) }));
    }
    Rc::new(__mapped_2)
}, method_semantics: method_semantics.clone() }))
    }
    ExprData::ExprMatch { scrutinee, arms, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprMatch { scrutinee: transform(scrutinee.clone()), arms: {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in arms.iter().cloned() {
        __mapped_4.push(Rc::new(MatchArm { pattern: __elem_5.pattern.clone(), guard: match __elem_5.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(guard) => {
        let guard = Rc::new(guard.clone());
        Some(transform(guard.clone()))
    }
    None => {
        None
    }
}, body: transform(__elem_5.body.clone()) }));
    }
    Rc::new(__mapped_4)
} }))
    }
    ExprData::ExprIf { condition, then_branch, else_branch, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprIf { condition: transform(condition.clone()), then_branch: transform(then_branch.clone()), else_branch: match else_branch.as_ref().map(|__rc| __rc.as_ref()) {
    Some(branch) => {
        let branch = Rc::new(branch.clone());
        Some(transform(branch.clone()))
    }
    None => {
        None
    }
} }))
    }
    ExprData::ExprLet { name, value, body, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprLet { name: name.clone(), value: transform(value.clone()), body: match body.as_ref().map(|__rc| __rc.as_ref()) {
    Some(inner) => {
        let inner = Rc::new(inner.clone());
        Some(transform(inner.clone()))
    }
    None => {
        None
    }
} }))
    }
    ExprData::ExprRecordLit { type_name, fields, parent_enum, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprRecordLit { type_name: type_name.clone(), fields: {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in fields.iter().cloned() {
        __mapped_6.push(Rc::new(FieldInit { name: __elem_7.name.clone(), value: transform(__elem_7.value.clone()) }));
    }
    Rc::new(__mapped_6)
}, parent_enum: parent_enum.clone() }))
    }
    ExprData::ExprListLit { elements, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprListLit { elements: {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in elements.iter().cloned() {
        __mapped_8.push(transform(__elem_9.clone()));
    }
    Rc::new(__mapped_8)
} }))
    }
    ExprData::ExprBinOp { op, left, right, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprBinOp { op: op.clone(), left: transform(left.clone()), right: transform(right.clone()) }))
    }
    ExprData::ExprUnaryOp { op, operand, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprUnaryOp { op: op.clone(), operand: transform(operand.clone()) }))
    }
    ExprData::ExprLambda { params, body, semantics, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprLambda { params: params.clone(), body: transform(body.clone()), semantics: semantics.clone() }))
    }
    ExprData::ExprStringInterp { parts, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprStringInterp { parts: {
    let mut __mapped_10 = Vec::new();
    for __elem_11 in parts.iter().cloned() {
        __mapped_10.push(match __elem_11.as_ref() {
    StringPart::Text { value, .. } => {
        Rc::new(StringPart::Text { value: value.clone() })
    }
    StringPart::Interpolation { expr, .. } => {
        Rc::new(StringPart::Interpolation { expr: transform(expr.clone()) })
    }
});
    }
    Rc::new(__mapped_10)
} }))
    }
    ExprData::ExprBlock { stmts, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprBlock { stmts: {
    let mut __mapped_12 = Vec::new();
    for __elem_13 in stmts.iter().cloned() {
        __mapped_12.push(transform(__elem_13.clone()));
    }
    Rc::new(__mapped_12)
} }))
    }
    ExprData::ExprCast { expr, target, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprCast { expr: transform(expr.clone()), target: transform(target.clone()) }))
    }
    ExprData::ExprForEach { variable, collection, body, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprForEach { variable: variable.clone(), collection: transform(collection.clone()), body: transform(body.clone()) }))
    }
    ExprData::ExprIndex { base, index, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprIndex { base: transform(base.clone()), index: transform(index.clone()) }))
    }
    ExprData::ExprSlice { base, start, end, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprSlice { base: transform(base.clone()), start: transform(start.clone()), end: transform(end.clone()) }))
    }
    ExprData::ExprReturn { value, .. } => {
        with_expr_data(expr_node.clone(), Rc::new(ExprData::ExprReturn { value: transform(value.clone()) }))
    }
    _ => {
        expr_node.clone()
    }
}
}

pub fn expr_has_self_call(texpr: Rc<Node>, fn_name: &str) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: _, call_semantics: _, .. } => {
        if f.clone() == fn_name {
    true
} else {
    {
    let mut __any_0 = false;
    for __elem_1 in expr_children(texpr.clone()).iter().cloned() {
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
    for __elem_3 in expr_children(texpr.clone()).iter().cloned() {
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
    ExprData::ExprCall { func: f, args: a, call_semantics: _, .. } => {
        if f.clone() == fn_name {
    if in_tail.clone() == false {
    true
} else {
    {
    let mut __any_0 = false;
    for __elem_1 in a.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_1.value.clone(), &fn_name, false) {
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
    for __elem_3 in a.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_3.value.clone(), &fn_name, false) {
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
    ExprData::ExprFieldAccess { base: b, field: _, summary: _, .. } => {
        expr_has_non_tail_self_call(b.clone(), &fn_name, false)
    }
    ExprData::ExprMethodCall { receiver: r, method: _, args: a, method_semantics: _, .. } => {
        expr_has_non_tail_self_call(r.clone(), &fn_name, false) || ({
    let mut __any_4 = false;
    for __elem_5 in a.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_5.value.clone(), &fn_name, false) {
    __any_4 = true;
    break;
};
    }
    __any_4
})
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        (expr_has_non_tail_self_call(c.clone(), &fn_name, false) || expr_has_non_tail_self_call(t.clone(), &fn_name, in_tail.clone())) || (match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        expr_has_non_tail_self_call(eb.clone(), &fn_name, in_tail.clone())
    }
    None => {
        false
    }
})
    }
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        expr_has_non_tail_self_call(s.clone(), &fn_name, false) || ({
    let mut __any_6 = false;
    for __elem_7 in arm_list.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_7.body.clone(), &fn_name, in_tail.clone()) {
    __any_6 = true;
    break;
};
    }
    __any_6
})
    }
    ExprData::ExprLet { name: _, value: v, body: bd, .. } => {
        expr_has_non_tail_self_call(v.clone(), &fn_name, false) || (match bd.as_ref().map(|__rc| __rc.as_ref()) {
    Some(b) => {
        let b = Rc::new(b.clone());
        expr_has_non_tail_self_call(b.clone(), &fn_name, in_tail.clone())
    }
    None => {
        false
    }
})
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        {
    let ss_count = {
    let __len_8 = ss.clone().len();
    __len_8 as i64
};
    if ss_count.clone() == 0_i64 {
    false
} else {
    let init_bad = {
    let mut __any_14 = false;
    for __elem_15 in ({
    let mut __filtered_12 = Vec::new();
    for __elem_13 in ({
    let mut __enumerated_9 = Vec::new();
    for (__idx_10, __elem_11) in ss.clone().iter().enumerate() {
        __enumerated_9.push((__idx_10 as i64, __elem_11.clone()));
    }
    Rc::new(__enumerated_9)
}).iter().cloned() {
        if __elem_13.0.clone() < (ss_count.clone() - 1_i64) {
    __filtered_12.push(__elem_13);
};
    }
    Rc::new(__filtered_12)
}).iter().cloned() {
        if expr_has_non_tail_self_call(__elem_15.1.clone(), &fn_name, false) {
    __any_14 = true;
    break;
};
    }
    __any_14
};
    let last_bad = match ss.clone().last().cloned() {
    Some(last_expr) => {
        expr_has_non_tail_self_call(last_expr.clone(), &fn_name, in_tail.clone())
    }
    None => {
        false
    }
};
    init_bad.clone() || last_bad.clone()
}
}
    }
    ExprData::ExprBinOp { op: _, left: l, right: r, .. } => {
        expr_has_non_tail_self_call(l.clone(), &fn_name, false) || expr_has_non_tail_self_call(r.clone(), &fn_name, false)
    }
    ExprData::ExprUnaryOp { op: _, operand: e, .. } => {
        expr_has_non_tail_self_call(e.clone(), &fn_name, false)
    }
    ExprData::ExprLambda { params: _, body: bd, semantics: _, .. } => {
        expr_has_non_tail_self_call(bd.clone(), &fn_name, false)
    }
    ExprData::ExprRecordLit { type_name: _, fields: fs, parent_enum: _, .. } => {
        {
    let mut __any_16 = false;
    for __elem_17 in fs.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_17.value.clone(), &fn_name, false) {
    __any_16 = true;
    break;
};
    }
    __any_16
}
    }
    ExprData::ExprListLit { elements: els, .. } => {
        {
    let mut __any_18 = false;
    for __elem_19 in els.iter().cloned() {
        if expr_has_non_tail_self_call(__elem_19.clone(), &fn_name, false) {
    __any_18 = true;
    break;
};
    }
    __any_18
}
    }
    ExprData::ExprStringInterp { parts: ps, .. } => {
        {
    let mut __any_20 = false;
    for __elem_21 in ps.iter().cloned() {
        if match __elem_21.as_ref() {
    StringPart::Interpolation { expr: e, .. } => {
        expr_has_non_tail_self_call(e.clone(), &fn_name, false)
    }
    StringPart::Text { value: _, .. } => {
        false
    }
} {
    __any_20 = true;
    break;
};
    }
    __any_20
}
    }
    ExprData::ExprCast { expr: e, target: _, .. } => {
        expr_has_non_tail_self_call(e.clone(), &fn_name, false)
    }
    ExprData::ExprForEach { variable: _, collection: c, body: bd, .. } => {
        expr_has_non_tail_self_call(c.clone(), &fn_name, false) || expr_has_non_tail_self_call(bd.clone(), &fn_name, false)
    }
    ExprData::ExprIndex { base: b, index: i, .. } => {
        expr_has_non_tail_self_call(b.clone(), &fn_name, false) || expr_has_non_tail_self_call(i.clone(), &fn_name, false)
    }
    ExprData::ExprSlice { base: b, start: s, end: e, .. } => {
        (expr_has_non_tail_self_call(b.clone(), &fn_name, false) || expr_has_non_tail_self_call(s.clone(), &fn_name, false)) || expr_has_non_tail_self_call(e.clone(), &fn_name, false)
    }
    ExprData::ExprReturn { value: v, .. } => {
        expr_has_non_tail_self_call(v.clone(), &fn_name, false)
    }
    ExprData::NoExprData => {
        false
    }
}
    })
}

pub fn diagnostic_node(severity: Severity, message: &str, span: SourceSpan, module_name: Option<String>) -> Rc<Node> {
    let sev_name = match severity {
    Severity::Error => {
        "error".to_string()
    }
    Severity::Warning => {
        "warning".to_string()
    }
};
    let sev_prop = Rc::new(FieldInit { name: "severity".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: sev_name.clone() }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) });
    let mod_prop = match module_name {
    Some(mn) => {
        Rc::new(vec!(Rc::new(FieldInit { name: "module_name".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: mn }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(Node { name: message.to_string(), span, children: Rc::new(Vec::new()), connective: Connective::NoConnective, collection_kind: CollectionKind::NoCollection, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: v2_rt::concat(Rc::new(vec!(sev_prop.clone())), mod_prop.clone()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
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

pub fn is_module_node(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "is_module" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn is_token_node(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "is_token" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn leaf_node(name: &str) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: Connective::NoConnective, collection_kind: CollectionKind::NoCollection, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn no_span() -> SourceSpan {
    SourceSpan { start: 0_i64, end: 0_i64 }
}

pub fn with_optional_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: Cardinality::CardOptional, uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), expr_data: n.expr_data.clone() })
}

pub fn with_required_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: Cardinality::Required, uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), expr_data: n.expr_data.clone() })
}


