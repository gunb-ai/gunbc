use crate::v2_core::*;
use crate::infer_types::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParserState {
    pub pos: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseResult {
    pub module: Option<Rc<Module>>,
    pub error: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdvanceResult {
    pub token: Rc<Token>,
    pub state: Rc<ParserState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EatResult {
    pub consumed: bool,
    pub state: Rc<ParserState>,
    pub token: Option<Rc<Token>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenResult {
    pub token: Rc<Token>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NameResult {
    pub name: String,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExprResult {
    pub expr: Rc<Node>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemResult {
    pub item: Rc<Node>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeResult {
    pub type_expr: Rc<Node>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleResult {
    pub module: Rc<Module>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportResult {
    pub import: Rc<Import>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariantResult {
    pub variant: Rc<Variant>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PredResult {
    pub predicate: Rc<FieldInit>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParamResult {
    pub param: Rc<Param>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransportResult {
    pub transport: Rc<Node>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpResult {
    pub operation: Rc<OperationDef>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapResult {
    pub capability: Rc<CapabilityDef>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatternResult {
    pub pattern: Rc<MatchPattern>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArmResult {
    pub arm: Rc<MatchArm>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArgResult {
    pub arg: Rc<NamedArg>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldResult {
    pub field: Rc<Field>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldInitResult {
    pub field: Rc<FieldInit>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResUseResult {
    pub resource_use: Rc<ResourceUse>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfigResult {
    pub config: Rc<ServiceConfig>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingPower {
    pub left: i64,
    pub right: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportsResult {
    pub imports: Rc<Vec<Rc<Import>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemsResult {
    pub items: Rc<Vec<Rc<Node>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamesResult {
    pub names: Rc<Vec<String>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldsResult {
    pub fields: Rc<Vec<Rc<Field>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldInitsResult {
    pub fields: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariantsResult {
    pub variants: Rc<Vec<Rc<Variant>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PredsResult {
    pub predicates: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParamsResult {
    pub params: Rc<Vec<Rc<Param>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsesResult {
    pub uses: Rc<Vec<Rc<ResourceUse>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArgsResult {
    pub args: Rc<Vec<Rc<NamedArg>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StmtsResult {
    pub stmts: Rc<Vec<Rc<Node>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExprsResult {
    pub exprs: Rc<Vec<Rc<Node>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArmsResult {
    pub arms: Rc<Vec<Rc<MatchArm>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModsResult {
    pub modifiers: Rc<Vec<OperationModifier>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingsResult {
    pub field_bindings: Rc<Vec<Rc<FieldBinding>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

pub fn parse_recovery_expr(span: SourceSpan, message: &str) -> Rc<Node> {
    make_expr_error_node(ExprErrorKind::ParseRecoveryError, &message, span)
}

pub fn parse_recovery_placeholder() -> Rc<Node> {
    parse_recovery_expr(SourceSpan { start: 0_i64, end: 0_i64 }, "parser recovery placeholder")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OptRetResult {
    pub return_type: Option<Rc<InferredNode>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GuardResult {
    pub guard: Option<Rc<Node>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FromKeyResult {
    pub from_key: Option<String>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PostfixResult {
    pub expr: Rc<Node>,
    pub changed: bool,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LambdaCheckResult {
    pub is_lambda: bool,
    pub params: Rc<Vec<String>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdentCollectResult {
    pub success: bool,
    pub params: Rc<Vec<String>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RangeArgsResult {
    pub min_val: Option<i64>,
    pub max_val: Option<i64>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamedIntResult {
    pub arg_name: String,
    pub arg_value: i64,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceBodyResult {
    pub config: Option<Rc<ServiceConfig>>,
    pub transport: Rc<Node>,
    pub operations: Rc<Vec<Rc<OperationDef>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IOResult {
    pub inputs: Rc<Vec<Rc<Field>>>,
    pub outputs: Rc<Vec<Rc<Field>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResPropResult {
    pub properties: Rc<Vec<Rc<FieldInit>>>,
    pub capabilities: Rc<Vec<Rc<CapabilityDef>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResponsesResult {
    pub responses: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MocksResult {
    pub mocks: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExitEntriesResult {
    pub entries: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RespEntriesResult {
    pub entries: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MockEntriesResult {
    pub entries: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpBodyResult {
    pub inputs: Rc<Vec<Rc<Field>>>,
    pub outputs: Rc<Vec<Rc<Field>>>,
    pub modifier_props: Rc<Vec<Rc<FieldInit>>>,
    pub transport: Option<Rc<Node>>,
    pub exit_props: Rc<Vec<Rc<FieldInit>>>,
    pub response_props: Rc<Vec<Rc<FieldInit>>>,
    pub mock_props: Rc<Vec<Rc<FieldInit>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnitResult {
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DescResult {
    pub desc: Option<String>,
    pub state: Rc<ParserState>,
}

pub fn peek(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Option<Rc<Token>> {
    tokens.clone().get((state.pos.clone()) as usize).cloned()
}

pub fn peek_kind(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Option<Rc<TokenKind>> {
    let tok = peek(tokens.clone(), state.clone());
    match tok.as_ref().map(|__rc| __rc.as_ref()) {
    Some(t) => {
        let t = Rc::new(t.clone());
        Some(t.kind.clone())
    }
    None => {
        None
    }
}
}

pub fn at_end(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    state.pos.clone() >= ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
})
}

pub fn current_span(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> SourceSpan {
    let tok = peek(tokens.clone(), state.clone());
    match tok.as_ref().map(|__rc| __rc.as_ref()) {
    Some(t) => {
        let t = Rc::new(t.clone());
        t.span.clone()
    }
    None => {
        SourceSpan { start: 0_i64, end: 0_i64 }
    }
}
}

pub fn advance(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<AdvanceResult> {
    let tok = peek(tokens.clone(), state.clone());
    match tok.as_ref().map(|__rc| __rc.as_ref()) {
    Some(t) => {
        let t = Rc::new(t.clone());
        {
    let next = Rc::new(ParserState { pos: state.pos.clone() + 1_i64, ..(*state).clone() });
    Rc::new(AdvanceResult { token: t.clone(), state: next.clone() })
}
    }
    None => {
        {
    let eof_tok = Rc::new(Token { kind: Rc::new(TokenKind::Eof), span: SourceSpan { start: 0_i64, end: 0_i64 } });
    Rc::new(AdvanceResult { token: eof_tok.clone(), state: state.clone() })
}
    }
}
}

pub fn parse_error(msg: &str, span: SourceSpan) -> Rc<Diagnostic> {
    Rc::new(Diagnostic { severity: Severity::Error, message: msg.to_string(), span: Some(span), module_name: None, category: None })
}

pub fn has_err(err: Option<Rc<Diagnostic>>) -> bool {
    match err.as_ref().map(|__rc| __rc.as_ref()) {
    Some(_) => {
        true
    }
    None => {
        false
    }
}
}

pub fn is_ident_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Ident { name: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lit_str_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::LitStr { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lit_int_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::LitInt { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lit_float_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::LitFloat { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_str_begin_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::StrBegin { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_str_mid_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::StrMid { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_str_end_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::StrEnd { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_unknown_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Unknown { char: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_newline_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Newline => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_eof_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Eof => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_module_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwModule => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_import_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwImport => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_type_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwType => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_fn_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwFn => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_func_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwFunc => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_service_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwService => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_resource_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwResource => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_data_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwData => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_extern_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwExtern => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_interface_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwInterface => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_pipeline_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwPipeline => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_profile_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwProfile => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_pattern_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwPattern => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_let_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwLet => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_return_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwReturn => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_match_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwMatch => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_if_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwIf => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_else_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwElse => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_for_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwFor => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_in_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwIn => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_where_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwWhere => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_with_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwWith => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_true_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwTrue => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_false_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwFalse => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_none_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwNone => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_acquire_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwAcquire => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_release_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwRelease => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_capability_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwCapability => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_operation_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwOperation => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_input_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwInput => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_output_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwOutput => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_idempotent_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwIdempotent => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_readonly_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwReadonly => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_kw_hermetic_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::KwHermetic => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lbrace_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::LBrace => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_rbrace_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::RBrace => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lparen_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::LParen => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_rparen_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::RParen => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lbracket_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::LBracket => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_rbracket_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::RBracket => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_lt_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Lt => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_gt_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Gt => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_le_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Le => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_ge_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Ge => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_fat_arrow_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::FatArrow => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_arrow_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Arrow => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_colon_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Colon => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_comma_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Comma => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_dot_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Dot => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_dot_dot_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::DotDot => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_eq_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Eq => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_eq_eq_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::EqEq => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_ne_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Ne => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_plus_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Plus => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_minus_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Minus => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_star_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Star => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_slash_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Slash => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_percent_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Percent => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_bang_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Bang => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_and_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::And => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_or_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Or => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_question_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Question => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_null_coalesce_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::NullCoalesce => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_pipe_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::Pipe => {
        true
    }
    _ => {
        false
    }
}
}

pub fn is_pipe_arrow_kind(kind: Rc<TokenKind>) -> bool {
    match kind.as_ref() {
    TokenKind::PipeArrow => {
        true
    }
    _ => {
        false
    }
}
}

pub fn peek_is_ident(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_ident_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_newline(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_newline_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_eof(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_eof_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_lit_str(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_lit_str_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_lbrace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_lbrace_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_rbrace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_rbrace_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_lparen(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_lparen_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_rparen(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_rparen_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_lbracket(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_lbracket_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_rbracket(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_rbracket_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_colon(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_colon_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_comma(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_comma_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_dot(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_dot_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_dot_dot(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_dot_dot_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_eq(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_eq_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_fat_arrow(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_fat_arrow_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_arrow(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_arrow_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_lt(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_lt_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_gt(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_gt_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_pipe(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_pipe_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_pipe_arrow(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_pipe_arrow_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_question(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_question_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_module(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_module_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_import(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_import_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_type(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_type_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_fn(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_fn_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_func(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_func_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_service(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_service_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_resource(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_resource_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_data(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_data_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_extern(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_extern_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_interface(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_interface_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_pipeline(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_pipeline_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_profile(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_profile_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_pattern(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_pattern_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_let(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_let_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_return(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_return_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_match(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_match_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_if(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_if_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_else(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_else_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_for(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_for_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_in(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_in_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_where(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_where_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_with(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_with_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_true(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_true_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_false(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_false_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_idempotent(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_idempotent_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_readonly(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_readonly_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_hermetic(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_hermetic_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_capability(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_capability_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_operation(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_operation_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_input(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_input_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn peek_is_kw_output(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match peek_kind(tokens.clone(), state.clone()) {
    Some(kind) => {
        is_kw_output_kind(kind.clone())
    }
    None => {
        false
    }
}
}

pub fn kind_tag(kind: Rc<TokenKind>) -> String {
    match kind.as_ref() {
    TokenKind::KwModule => {
        "KwModule".to_string()
    }
    TokenKind::KwImport => {
        "KwImport".to_string()
    }
    TokenKind::KwType => {
        "KwType".to_string()
    }
    TokenKind::KwFn => {
        "KwFn".to_string()
    }
    TokenKind::KwFunc => {
        "KwFunc".to_string()
    }
    TokenKind::KwService => {
        "KwService".to_string()
    }
    TokenKind::KwResource => {
        "KwResource".to_string()
    }
    TokenKind::KwData => {
        "KwData".to_string()
    }
    TokenKind::KwExtern => {
        "KwExtern".to_string()
    }
    TokenKind::KwInterface => {
        "KwInterface".to_string()
    }
    TokenKind::KwPipeline => {
        "KwPipeline".to_string()
    }
    TokenKind::KwProfile => {
        "KwProfile".to_string()
    }
    TokenKind::KwPattern => {
        "KwPattern".to_string()
    }
    TokenKind::KwLet => {
        "KwLet".to_string()
    }
    TokenKind::KwReturn => {
        "KwReturn".to_string()
    }
    TokenKind::KwMatch => {
        "KwMatch".to_string()
    }
    TokenKind::KwIf => {
        "KwIf".to_string()
    }
    TokenKind::KwElse => {
        "KwElse".to_string()
    }
    TokenKind::KwFor => {
        "KwFor".to_string()
    }
    TokenKind::KwIn => {
        "KwIn".to_string()
    }
    TokenKind::KwWhere => {
        "KwWhere".to_string()
    }
    TokenKind::KwWith => {
        "KwWith".to_string()
    }
    TokenKind::KwTrue => {
        "KwTrue".to_string()
    }
    TokenKind::KwFalse => {
        "KwFalse".to_string()
    }
    TokenKind::KwNone => {
        "KwNone".to_string()
    }
    TokenKind::KwAcquire => {
        "KwAcquire".to_string()
    }
    TokenKind::KwRelease => {
        "KwRelease".to_string()
    }
    TokenKind::KwCapability => {
        "KwCapability".to_string()
    }
    TokenKind::KwOperation => {
        "KwOperation".to_string()
    }
    TokenKind::KwInput => {
        "KwInput".to_string()
    }
    TokenKind::KwOutput => {
        "KwOutput".to_string()
    }
    TokenKind::KwIdempotent => {
        "KwIdempotent".to_string()
    }
    TokenKind::KwReadonly => {
        "KwReadonly".to_string()
    }
    TokenKind::KwHermetic => {
        "KwHermetic".to_string()
    }
    TokenKind::LBrace => {
        "LBrace".to_string()
    }
    TokenKind::RBrace => {
        "RBrace".to_string()
    }
    TokenKind::LParen => {
        "LParen".to_string()
    }
    TokenKind::RParen => {
        "RParen".to_string()
    }
    TokenKind::LBracket => {
        "LBracket".to_string()
    }
    TokenKind::RBracket => {
        "RBracket".to_string()
    }
    TokenKind::Lt => {
        "Lt".to_string()
    }
    TokenKind::Gt => {
        "Gt".to_string()
    }
    TokenKind::Le => {
        "Le".to_string()
    }
    TokenKind::Ge => {
        "Ge".to_string()
    }
    TokenKind::FatArrow => {
        "FatArrow".to_string()
    }
    TokenKind::Arrow => {
        "Arrow".to_string()
    }
    TokenKind::Colon => {
        "Colon".to_string()
    }
    TokenKind::Comma => {
        "Comma".to_string()
    }
    TokenKind::Dot => {
        "Dot".to_string()
    }
    TokenKind::DotDot => {
        "DotDot".to_string()
    }
    TokenKind::Eq => {
        "Eq".to_string()
    }
    TokenKind::EqEq => {
        "EqEq".to_string()
    }
    TokenKind::Ne => {
        "Ne".to_string()
    }
    TokenKind::Plus => {
        "Plus".to_string()
    }
    TokenKind::Minus => {
        "Minus".to_string()
    }
    TokenKind::Star => {
        "Star".to_string()
    }
    TokenKind::Slash => {
        "Slash".to_string()
    }
    TokenKind::Percent => {
        "Percent".to_string()
    }
    TokenKind::Bang => {
        "Bang".to_string()
    }
    TokenKind::And => {
        "And".to_string()
    }
    TokenKind::Or => {
        "Or".to_string()
    }
    TokenKind::Question => {
        "Question".to_string()
    }
    TokenKind::NullCoalesce => {
        "NullCoalesce".to_string()
    }
    TokenKind::Pipe => {
        "Pipe".to_string()
    }
    TokenKind::PipeArrow => {
        "PipeArrow".to_string()
    }
    TokenKind::LitStr { value: _, .. } => {
        "LitStr".to_string()
    }
    TokenKind::LitInt { value: _, .. } => {
        "LitInt".to_string()
    }
    TokenKind::LitFloat { value: _, .. } => {
        "LitFloat".to_string()
    }
    TokenKind::Ident { name: _, .. } => {
        "Ident".to_string()
    }
    TokenKind::StrBegin { value: _, .. } => {
        "StrBegin".to_string()
    }
    TokenKind::StrMid { value: _, .. } => {
        "StrMid".to_string()
    }
    TokenKind::StrEnd { value: _, .. } => {
        "StrEnd".to_string()
    }
    TokenKind::Newline => {
        "Newline".to_string()
    }
    TokenKind::Eof => {
        "Eof".to_string()
    }
    TokenKind::Unknown { char: _, .. } => {
        "Unknown".to_string()
    }
}
}

pub fn kind_matches_tag(kind: Rc<TokenKind>, tag: &str) -> bool {
    if tag == "KwModule" {
    is_kw_module_kind(kind.clone())
} else {
    if tag == "KwImport" {
    is_kw_import_kind(kind.clone())
} else {
    if tag == "KwType" {
    is_kw_type_kind(kind.clone())
} else {
    if tag == "KwFn" {
    is_kw_fn_kind(kind.clone())
} else {
    if tag == "KwFunc" {
    is_kw_func_kind(kind.clone())
} else {
    if tag == "KwService" {
    is_kw_service_kind(kind.clone())
} else {
    if tag == "KwResource" {
    is_kw_resource_kind(kind.clone())
} else {
    if tag == "KwData" {
    is_kw_data_kind(kind.clone())
} else {
    if tag == "KwExtern" {
    is_kw_extern_kind(kind.clone())
} else {
    if tag == "KwInterface" {
    is_kw_interface_kind(kind.clone())
} else {
    if tag == "KwPipeline" {
    is_kw_pipeline_kind(kind.clone())
} else {
    if tag == "KwProfile" {
    is_kw_profile_kind(kind.clone())
} else {
    if tag == "KwPattern" {
    is_kw_pattern_kind(kind.clone())
} else {
    if tag == "KwLet" {
    is_kw_let_kind(kind.clone())
} else {
    if tag == "KwReturn" {
    is_kw_return_kind(kind.clone())
} else {
    if tag == "KwMatch" {
    is_kw_match_kind(kind.clone())
} else {
    if tag == "KwIf" {
    is_kw_if_kind(kind.clone())
} else {
    if tag == "KwElse" {
    is_kw_else_kind(kind.clone())
} else {
    if tag == "KwFor" {
    is_kw_for_kind(kind.clone())
} else {
    if tag == "KwIn" {
    is_kw_in_kind(kind.clone())
} else {
    if tag == "KwWhere" {
    is_kw_where_kind(kind.clone())
} else {
    if tag == "KwWith" {
    is_kw_with_kind(kind.clone())
} else {
    if tag == "KwTrue" {
    is_kw_true_kind(kind.clone())
} else {
    if tag == "KwFalse" {
    is_kw_false_kind(kind.clone())
} else {
    if tag == "KwNone" {
    is_kw_none_kind(kind.clone())
} else {
    if tag == "KwAcquire" {
    is_kw_acquire_kind(kind.clone())
} else {
    if tag == "KwRelease" {
    is_kw_release_kind(kind.clone())
} else {
    if tag == "KwCapability" {
    is_kw_capability_kind(kind.clone())
} else {
    if tag == "KwOperation" {
    is_kw_operation_kind(kind.clone())
} else {
    if tag == "KwInput" {
    is_kw_input_kind(kind.clone())
} else {
    if tag == "KwOutput" {
    is_kw_output_kind(kind.clone())
} else {
    if tag == "KwIdempotent" {
    is_kw_idempotent_kind(kind.clone())
} else {
    if tag == "KwReadonly" {
    is_kw_readonly_kind(kind.clone())
} else {
    if tag == "KwHermetic" {
    is_kw_hermetic_kind(kind.clone())
} else {
    if tag == "LBrace" {
    is_lbrace_kind(kind.clone())
} else {
    if tag == "RBrace" {
    is_rbrace_kind(kind.clone())
} else {
    if tag == "LParen" {
    is_lparen_kind(kind.clone())
} else {
    if tag == "RParen" {
    is_rparen_kind(kind.clone())
} else {
    if tag == "LBracket" {
    is_lbracket_kind(kind.clone())
} else {
    if tag == "RBracket" {
    is_rbracket_kind(kind.clone())
} else {
    if tag == "Lt" {
    is_lt_kind(kind.clone())
} else {
    if tag == "Gt" {
    is_gt_kind(kind.clone())
} else {
    if tag == "Le" {
    is_le_kind(kind.clone())
} else {
    if tag == "Ge" {
    is_ge_kind(kind.clone())
} else {
    if tag == "FatArrow" {
    is_fat_arrow_kind(kind.clone())
} else {
    if tag == "Arrow" {
    is_arrow_kind(kind.clone())
} else {
    if tag == "Colon" {
    is_colon_kind(kind.clone())
} else {
    if tag == "Comma" {
    is_comma_kind(kind.clone())
} else {
    if tag == "Dot" {
    is_dot_kind(kind.clone())
} else {
    if tag == "DotDot" {
    is_dot_dot_kind(kind.clone())
} else {
    if tag == "Eq" {
    is_eq_kind(kind.clone())
} else {
    if tag == "EqEq" {
    is_eq_eq_kind(kind.clone())
} else {
    if tag == "Ne" {
    is_ne_kind(kind.clone())
} else {
    if tag == "Plus" {
    is_plus_kind(kind.clone())
} else {
    if tag == "Minus" {
    is_minus_kind(kind.clone())
} else {
    if tag == "Star" {
    is_star_kind(kind.clone())
} else {
    if tag == "Slash" {
    is_slash_kind(kind.clone())
} else {
    if tag == "Percent" {
    is_percent_kind(kind.clone())
} else {
    if tag == "Bang" {
    is_bang_kind(kind.clone())
} else {
    if tag == "And" {
    is_and_kind(kind.clone())
} else {
    if tag == "Or" {
    is_or_kind(kind.clone())
} else {
    if tag == "Question" {
    is_question_kind(kind.clone())
} else {
    if tag == "NullCoalesce" {
    is_null_coalesce_kind(kind.clone())
} else {
    if tag == "Pipe" {
    is_pipe_kind(kind.clone())
} else {
    if tag == "PipeArrow" {
    is_pipe_arrow_kind(kind.clone())
} else {
    if tag == "Ident" {
    is_ident_kind(kind.clone())
} else {
    if tag == "LitStr" {
    is_lit_str_kind(kind.clone())
} else {
    if tag == "LitInt" {
    is_lit_int_kind(kind.clone())
} else {
    if tag == "LitFloat" {
    is_lit_float_kind(kind.clone())
} else {
    if tag == "StrBegin" {
    is_str_begin_kind(kind.clone())
} else {
    if tag == "StrMid" {
    is_str_mid_kind(kind.clone())
} else {
    if tag == "StrEnd" {
    is_str_end_kind(kind.clone())
} else {
    if tag == "Newline" {
    is_newline_kind(kind.clone())
} else {
    if tag == "Eof" {
    is_eof_kind(kind.clone())
} else {
    false
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}

pub fn expect(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, tag: &str) -> Rc<TokenResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    let matches = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(kind) => {
        let kind = Rc::new(kind.clone());
        kind_matches_tag(kind.clone(), &tag)
    }
    None => {
        false
    }
};
    if matches.clone() {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(TokenResult { token: adv.token.clone(), state: adv.state.clone(), err: None })
} else {
    let found = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(kind) => {
        let kind = Rc::new(kind.clone());
        kind_tag(kind.clone())
    }
    None => {
        "EOF".to_string()
    }
};
    Rc::new(TokenResult { token: Rc::new(Token { kind: Rc::new(TokenKind::Eof), span: SourceSpan { start: 0_i64, end: 0_i64 } }), state: state.clone(), err: Some(parse_error(&format!("expected {}, found {}", tag.to_string(), found.clone()), current_span(tokens.clone(), state.clone()))) })
}
}

pub fn expect_ident(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<NameResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { name: n, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(NameResult { name: n.clone(), state: adv.state.clone(), err: None })
}
    }
    _ => {
        {
    let found = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(kind) => {
        let kind = Rc::new(kind.clone());
        kind_tag(kind.clone())
    }
    None => {
        "EOF".to_string()
    }
};
    Rc::new(NameResult { name: "".to_string(), state: state.clone(), err: Some(parse_error(&v2_rt::concat("expected identifier, found ".to_string(), found.clone()), current_span(tokens.clone(), state.clone()))) })
}
    }
}
}

pub fn expect_name(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<NameResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { name: n, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(NameResult { name: n.clone(), state: adv.state.clone(), err: None })
}
    }
    _ => {
        {
    let kw_name = keyword_to_name(tokens.clone(), state.clone());
    match kw_name {
    Some(n) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(NameResult { name: n, state: adv.state.clone(), err: None })
}
    }
    None => {
        {
    let found = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(kind) => {
        let kind = Rc::new(kind.clone());
        kind_tag(kind.clone())
    }
    None => {
        "EOF".to_string()
    }
};
    Rc::new(NameResult { name: "".to_string(), state: state.clone(), err: Some(parse_error(&v2_rt::concat("expected name, found ".to_string(), found.clone()), current_span(tokens.clone(), state.clone()))) })
}
    }
}
}
    }
}
}

pub fn keyword_to_name(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Option<String> {
    if peek_is_kw_type(tokens.clone(), state.clone()) {
    Some("type".to_string())
} else {
    if peek_is_kw_resource(tokens.clone(), state.clone()) {
    Some("resource".to_string())
} else {
    if peek_is_kw_capability(tokens.clone(), state.clone()) {
    Some("capability".to_string())
} else {
    if peek_is_kw_operation(tokens.clone(), state.clone()) {
    Some("operation".to_string())
} else {
    if peek_is_kw_pattern(tokens.clone(), state.clone()) {
    Some("pattern".to_string())
} else {
    if peek_is_kw_input(tokens.clone(), state.clone()) {
    Some("input".to_string())
} else {
    if peek_is_kw_output(tokens.clone(), state.clone()) {
    Some("output".to_string())
} else {
    if peek_is_kw_data(tokens.clone(), state.clone()) {
    Some("data".to_string())
} else {
    if peek_is_kw_match(tokens.clone(), state.clone()) {
    Some("match".to_string())
} else {
    if peek_is_kw_service(tokens.clone(), state.clone()) {
    Some("service".to_string())
} else {
    if peek_is_kw_import(tokens.clone(), state.clone()) {
    Some("import".to_string())
} else {
    if peek_is_kw_module(tokens.clone(), state.clone()) {
    Some("module".to_string())
} else {
    if peek_is_kw_fn(tokens.clone(), state.clone()) {
    Some("fn".to_string())
} else {
    if peek_is_kw_func(tokens.clone(), state.clone()) {
    Some("func".to_string())
} else {
    if peek_is_kw_extern(tokens.clone(), state.clone()) {
    Some("extern".to_string())
} else {
    if peek_is_kw_let(tokens.clone(), state.clone()) {
    Some("let".to_string())
} else {
    if peek_is_kw_return(tokens.clone(), state.clone()) {
    Some("return".to_string())
} else {
    if peek_is_kw_if(tokens.clone(), state.clone()) {
    Some("if".to_string())
} else {
    if peek_is_kw_else(tokens.clone(), state.clone()) {
    Some("else".to_string())
} else {
    if peek_is_kw_for(tokens.clone(), state.clone()) {
    Some("for".to_string())
} else {
    if peek_is_kw_in(tokens.clone(), state.clone()) {
    Some("in".to_string())
} else {
    if peek_is_kw_where(tokens.clone(), state.clone()) {
    Some("where".to_string())
} else {
    if peek_is_kw_with(tokens.clone(), state.clone()) {
    Some("with".to_string())
} else {
    if peek_is_kw_true(tokens.clone(), state.clone()) {
    Some("true".to_string())
} else {
    if peek_is_kw_false(tokens.clone(), state.clone()) {
    Some("false".to_string())
} else {
    if peek_is_kw_interface(tokens.clone(), state.clone()) {
    Some("interface".to_string())
} else {
    if peek_is_kw_pipeline(tokens.clone(), state.clone()) {
    Some("pipeline".to_string())
} else {
    if peek_is_kw_profile(tokens.clone(), state.clone()) {
    Some("profile".to_string())
} else {
    if peek_is_kw_idempotent(tokens.clone(), state.clone()) {
    Some("idempotent".to_string())
} else {
    if peek_is_kw_readonly(tokens.clone(), state.clone()) {
    Some("readonly".to_string())
} else {
    if peek_is_kw_hermetic(tokens.clone(), state.clone()) {
    Some("hermetic".to_string())
} else {
    None
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}
}

pub fn skip_newlines(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ParserState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            if peek_is_newline(tokens.clone(), state.clone()) {
    let adv = advance(tokens.clone(), state.clone());
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        continue;
    }

} else {
    break state.clone();
};
        }
    })
}

pub fn is_continuation_kind(kind: Rc<TokenKind>) -> bool {
    ((is_pipe_arrow_kind(kind.clone()) || is_dot_kind(kind.clone())) || is_or_kind(kind.clone())) || is_and_kind(kind.clone())
}

pub fn skip_continuation_newlines(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ParserState> {
    let is_continuation = if peek_is_newline(tokens.clone(), state.clone()) {
    let s = skip_newlines(tokens.clone(), state.clone());
    match peek_kind(tokens.clone(), s.clone()) {
    Some(kind) => {
        is_continuation_kind(kind.clone())
    }
    None => {
        false
    }
}
} else {
    false
};
    if is_continuation.clone() {
    skip_newlines(tokens.clone(), state.clone())
} else {
    state.clone()
}
}

pub fn eat(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, tag: &str) -> Rc<EatResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    let matches = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(kind) => {
        let kind = Rc::new(kind.clone());
        kind_matches_tag(kind.clone(), &tag)
    }
    None => {
        false
    }
};
    if matches.clone() {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(EatResult { consumed: true, state: adv.state.clone(), token: Some(adv.token.clone()) })
} else {
    Rc::new(EatResult { consumed: false, state: state.clone(), token: None })
}
}

pub fn is_ident(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    peek_is_ident(tokens.clone(), state.clone())
}

pub fn is_keyword_name(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    match keyword_to_name(tokens.clone(), state.clone()) {
    Some(_) => {
        true
    }
    None => {
        false
    }
}
}

pub fn leaf_type_node(name: &str, span: SourceSpan) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span, children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn is_conj_with_children(n: Rc<Node>) -> bool {
    node_is_product(n.clone()) && (({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) > 0_i64)
}

pub fn child_return_type_or_empty(ch: Rc<Node>) -> Rc<Node> {
    if ch.return_type.clone().is_some() {
    match rt_node(ch.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        leaf_type_node("Unit", ch.span.clone())
    }
    NodeType::Untyped => {
        leaf_type_node("Unit", ch.span.clone())
    }
}
} else {
    leaf_type_node("", ch.span.clone())
}
}

pub fn node_return_type_to_outputs(rt: Rc<Node>) -> Rc<Vec<Rc<Field>>> {
    if is_conj_with_children(rt.clone()) {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in rt.children.iter().cloned() {
        __mapped_0.push(Rc::new(Field { name: __elem_1.name.clone(), type_expr: child_return_type_or_empty(__elem_1.clone()), cardinality: Cardinality::Required, default_value: __elem_1.body.clone(), from_key: None, span: __elem_1.span.clone() }));
    }
    Rc::new(__mapped_0)
}
} else {
    Rc::new(vec!(Rc::new(Field { name: "value".to_string(), type_expr: rt.clone(), cardinality: Cardinality::Required, default_value: None, from_key: None, span: rt.span.clone() })))
}
}

pub fn parse_dotted_ident(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<NameResult> {
    let r = expect_name(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return r.clone();
};
    parse_dotted_ident_rest(tokens.clone(), r.state.clone(), &r.name)
}

pub fn parse_dotted_ident_rest(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: &str) -> Rc<NameResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc.to_string();
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let e = eat(tokens.clone(), state.clone(), "Dot");
            if e.consumed.clone() {
    let r = expect_name(tokens.clone(), e.state.clone());
    if has_err(r.err.clone()) {
    break r.clone();
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = format!("{}.{}", acc.to_string(), r.name.clone());
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(NameResult { name: acc.to_string(), state: state.clone(), err: None });
};
        }
    })
}

pub fn parse(tokens: Rc<Vec<Rc<Token>>>) -> Rc<ParseResult> {
    let state = Rc::new(ParserState { pos: 0_i64 });
    let r = parse_module(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    Rc::new(ParseResult { module: None, error: r.err.clone() })
} else {
    Rc::new(ParseResult { module: Some(r.module.clone()), error: None })
}
}

pub fn parse_module(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ModuleResult> {
    let s = skip_newlines(tokens.clone(), state.clone());
    let start_span = current_span(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), s.clone(), "KwModule");
    if has_err(r.err.clone()) {
    return Rc::new(ModuleResult { module: Rc::new(Module { name: "".to_string(), imports: Rc::new(Vec::new()), items: Rc::new(Vec::new()), span: start_span.clone() }), state: r.state.clone(), err: r.err.clone() });
};
    let s = r.state.clone();
    let r = parse_dotted_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ModuleResult { module: Rc::new(Module { name: "".to_string(), imports: Rc::new(Vec::new()), items: Rc::new(Vec::new()), span: start_span.clone() }), state: r.state.clone(), err: r.err.clone() });
};
    let mod_name = r.name.clone();
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r = parse_imports(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ModuleResult { module: Rc::new(Module { name: "".to_string(), imports: Rc::new(Vec::new()), items: Rc::new(Vec::new()), span: start_span.clone() }), state: r.state.clone(), err: r.err.clone() });
};
    let imports = r.imports.clone();
    let s = r.state.clone();
    let r = parse_items(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ModuleResult { module: Rc::new(Module { name: "".to_string(), imports: Rc::new(Vec::new()), items: Rc::new(Vec::new()), span: start_span.clone() }), state: r.state.clone(), err: r.err.clone() });
};
    let items = r.items.clone();
    let s = r.state.clone();
    let r#mod = Rc::new(Module { name: mod_name, imports: imports.clone(), items: items.clone(), span: start_span.clone() });
    Rc::new(ModuleResult { module: r#mod.clone(), state: s.clone(), err: None })
}

pub fn parse_imports(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ImportsResult> {
    parse_imports_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_imports_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Import>>>) -> Rc<ImportsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_kw_import(tokens.clone(), s.clone()) {
    let r = parse_import(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ImportsResult { imports: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.import.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(ImportsResult { imports: acc.clone(), state: s.clone(), err: None });
};
        }
    })
}

pub fn parse_items(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemsResult> {
    parse_items_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_items_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Node>>>) -> Rc<ItemsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if at_end(tokens.clone(), s.clone()) || peek_is_eof(tokens.clone(), s.clone()) {
    break Rc::new(ItemsResult { items: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_item(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ItemsResult { items: acc.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.item.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_import(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ImportResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let err_import = Rc::new(Import { module_path: "".to_string(), names: Rc::new(ImportNames::ImportSpecific { names: Rc::new(Vec::new()) }), span: start_span.clone() });
    let r = expect(tokens.clone(), state.clone(), "KwImport");
    if has_err(r.err.clone()) {
    return Rc::new(ImportResult { import: err_import.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = r.state.clone();
    let r = parse_dotted_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ImportResult { import: err_import.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let mod_path = r.name.clone();
    let s = r.state.clone();
    let e = eat(tokens.clone(), s.clone(), "LBrace");
    if e.consumed.clone() {
    let r = parse_import_names(tokens.clone(), e.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ImportResult { import: err_import.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let names = r.names.clone();
    let s = r.state.clone();
    let r = expect(tokens.clone(), s.clone(), "RBrace");
    if has_err(r.err.clone()) {
    return Rc::new(ImportResult { import: err_import.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let imp = Rc::new(Import { module_path: mod_path, names: Rc::new(ImportNames::ImportSpecific { names: names.clone() }), span: start_span.clone() });
    Rc::new(ImportResult { import: imp.clone(), state: s.clone(), err: None })
} else {
    let s = skip_newlines(tokens.clone(), s.clone());
    let imp = Rc::new(Import { module_path: mod_path, names: Rc::new(ImportNames::ImportAll), span: start_span.clone() });
    Rc::new(ImportResult { import: imp.clone(), state: s.clone(), err: None })
}
}

pub fn parse_import_names(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<NamesResult> {
    parse_import_names_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_import_names_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<String>>) -> Rc<NamesResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) {
    break Rc::new(NamesResult { names: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_dotted_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(NamesResult { names: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let e = eat(tokens.clone(), s.clone(), "Comma");
    let s = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    s.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(name);
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_item(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let s = skip_newlines(tokens.clone(), state.clone());
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwType) => {
        parse_type_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwFn) => {
        parse_fn_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwFunc) => {
        parse_func_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwService) => {
        parse_service_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwResource) => {
        parse_resource_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwData) => {
        parse_data_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwExtern) => {
        parse_extern_decl(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwPattern) => {
        parse_func_def(tokens.clone(), s.clone())
    }
    Some(TokenKind::KwInterface) => {
        parse_func_def(tokens.clone(), s.clone())
    }
    _ => {
        Rc::new(ItemResult { item: Rc::new(Node { name: "<unknown>".to_string(), span: current_span(tokens.clone(), s.clone()), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), state: s.clone(), err: Some(parse_error("expected item declaration (type, fn, func, service, resource, data, extern, pattern, interface)", current_span(tokens.clone(), s.clone()))) })
    }
}
}

pub fn field_to_child_node(field: Rc<Field>) -> Rc<Node> {
    let ret_type = field.type_expr.clone();
    let props = match field.from_key.clone() {
    Some(key) => {
        Rc::new(vec!(Rc::new(FieldInit { name: "from_key".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: key }) }), None, field.span.clone()) })))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(Node { name: field.name.clone(), span: field.span.clone(), children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: ret_type.clone() })), return_cardinality: field.cardinality.clone(), uses: Rc::new(Vec::new()), body: field.default_value.clone(), transport: None, properties: props.clone(), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn variant_to_child_node(variant: Rc<Variant>) -> Rc<Node> {
    let children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in variant.fields.iter().cloned() {
        __mapped_0.push(field_to_child_node(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    Rc::new(Node { name: variant.name.clone(), span: variant.span.clone(), children: children.clone(), connective: if ({
    let __len_2 = variant.fields.clone().len();
    __len_2 as i64
}) > 0_i64 {
    Some(Connective::Conj)
} else {
    None
}, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn outputs_to_return_type(outputs: Rc<Vec<Rc<Field>>>, span: SourceSpan) -> Option<Rc<InferredNode>> {
    if ({
    let __len_2 = outputs.clone().len();
    __len_2 as i64
}) > 0_i64 {
    Some(Rc::new(InferredNode::Resolved { node: Rc::new(Node { name: "".to_string(), span, children: {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in outputs.iter().cloned() {
        __mapped_0.push(field_to_child_node(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
}, connective: Some(Connective::Conj), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }) }))
} else {
    None
}
}

pub fn parse_type_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), state.clone(), "KwType");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let type_params_result = parse_optional_type_params(tokens.clone(), r.state.clone());
    let type_params = type_params_result.params.clone();
    let s = skip_newlines(tokens.clone(), type_params_result.state.clone());
    let e = eat(tokens.clone(), s.clone(), "LBrace");
    if e.consumed.clone() {
    let r = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), e.state.clone()));
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: type_params.clone(), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let type_children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in r.fields.iter().cloned() {
        __mapped_0.push(field_to_child_node(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: type_children.clone(), connective: Some(Connective::Conj), params: type_params.clone(), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), r2.state.clone()), err: None })
} else {
    let eq = eat(tokens.clone(), s.clone(), "Eq");
    if eq.consumed.clone() {
    let s = skip_newlines(tokens.clone(), eq.state.clone());
    parse_type_body_after_eq(tokens.clone(), s.clone(), &name, start_span.clone(), type_params.clone())
} else {
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), connective: None, params: type_params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: leaf_type_node(&name, start_span.clone()) })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: s.clone(), err: None })
}
}
}

pub fn parse_type_body_after_eq(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, name: &str, start_span: SourceSpan, type_params: Rc<Vec<Rc<Param>>>) -> Rc<ItemResult> {
    let dummy = Rc::new(Node { name: name.to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: type_params.clone(), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    if is_ident(tokens.clone(), state.clone()) {
    let r = expect_ident(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let first_name = r.name.clone();
    let s = skip_newlines(tokens.clone(), r.state.clone());
    if peek_is_lbrace(tokens.clone(), s.clone()) || peek_is_pipe(tokens.clone(), s.clone()) {
    let r = parse_variant_fields(tokens.clone(), s.clone(), &first_name);
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let first_variant = r.variant.clone();
    let rest = parse_more_variants_acc(tokens.clone(), r.state.clone(), Rc::new(vec!(first_variant.clone())));
    if has_err(rest.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: rest.state.clone(), err: rest.err.clone() });
};
    let variants = rest.variants.clone();
    let type_children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in variants.iter().cloned() {
        __mapped_0.push(variant_to_child_node(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let item = Rc::new(Node { name: name.to_string(), span: start_span.clone(), children: type_children.clone(), connective: Some(Connective::Disj), params: type_params.clone(), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), rest.state.clone()), err: None })
} else {
    let r = finish_type_expr_from_name(tokens.clone(), s.clone(), &first_name, start_span.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let wr = try_where_clause(tokens.clone(), r.state.clone(), r.type_expr.clone(), start_span.clone());
    if has_err(wr.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: wr.state.clone(), err: wr.err.clone() });
};
    let item = Rc::new(Node { name: name.to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), connective: None, params: type_params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: wr.type_expr.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), wr.state.clone()), err: None })
}
} else {
    let r = parse_type_expr(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let wr = try_where_clause(tokens.clone(), r.state.clone(), r.type_expr.clone(), start_span.clone());
    if has_err(wr.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: wr.state.clone(), err: wr.err.clone() });
};
    let item = Rc::new(Node { name: name.to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), connective: None, params: type_params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: wr.type_expr.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), wr.state.clone()), err: None })
}
}

pub fn try_where_clause(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, base_te: Rc<Node>, start_span: SourceSpan) -> Rc<TypeResult> {
    if peek_is_kw_where(tokens.clone(), state.clone()) {
    let adv = advance(tokens.clone(), state.clone());
    let r = parse_predicates(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(TypeResult { type_expr: base_te.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let refined = Rc::new(Node { name: "Refined".to_string(), span: start_span, children: Rc::new(vec!(base_te.clone())), connective: Some(Connective::Conj), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: r.predicates.clone(), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(TypeResult { type_expr: refined.clone(), state: r.state.clone(), err: None })
} else {
    Rc::new(TypeResult { type_expr: base_te.clone(), state: state.clone(), err: None })
}
}

pub fn parse_predicates(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<PredsResult> {
    parse_predicates_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_predicates_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<FieldInit>>>) -> Rc<PredsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let r = parse_single_predicate(tokens.clone(), state);
            if has_err(r.err.clone()) {
    break Rc::new(PredsResult { predicates: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
            let acc = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.predicate.clone());
    Rc::new(__appended_0)
};
            let e = eat(tokens.clone(), r.state.clone(), "Comma");
            if e.consumed.clone() {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = skip_newlines(tokens.clone(), e.state.clone());
        let __tco_2 = acc.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(PredsResult { predicates: acc.clone(), state: r.state.clone(), err: None });
};
        }
    })
}

pub fn parse_single_predicate(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<PredResult> {
    let zero_span = SourceSpan { start: 0_i64, end: 0_i64 };
    let dummy_pred = Rc::new(FieldInit { name: "".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: false }) }), None, zero_span.clone()) });
    let r = expect_name(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let pred_name = r.name.clone();
    let s = r.state.clone();
    let e = eat(tokens.clone(), s.clone(), "LParen");
    if e.consumed.clone() {
    match pred_name.as_str() {
    "pattern" => {
        {
    let r2 = parse_expr(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), r2.state.clone(), "RParen");
    if has_err(r3.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    match r2.expr.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitStr { value: s, .. } = value.as_ref() else { unreachable!() };
        Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "Pattern".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: s.clone() }) }), None, zero_span.clone()) }), state: r3.state.clone(), err: None })
    }
    _ => {
        Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: Some(parse_error("pattern() requires a string literal argument", current_span(tokens.clone(), r3.state.clone()))) })
    }
}
}
    }
    "format" => {
        {
    let r2 = expect_ident(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), r2.state.clone(), "RParen");
    if has_err(r3.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "Format".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: r2.name.clone() }) }), None, zero_span.clone()) }), state: r3.state.clone(), err: None })
}
    }
    "brand" => {
        {
    let r2 = parse_expr(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), r2.state.clone(), "RParen");
    if has_err(r3.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    match r2.expr.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitStr { value: s, .. } = value.as_ref() else { unreachable!() };
        Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "Brand".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: s.clone() }) }), None, zero_span.clone()) }), state: r3.state.clone(), err: None })
    }
    _ => {
        Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: Some(parse_error("brand() requires a string literal argument", current_span(tokens.clone(), r3.state.clone()))) })
    }
}
}
    }
    "content" => {
        {
    let r2 = expect_ident(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), r2.state.clone(), "RParen");
    if has_err(r3.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "ContentEncoding".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: r2.name.clone() }) }), None, zero_span.clone()) }), state: r3.state.clone(), err: None })
}
    }
    "domain" => {
        {
    let r2 = expect_ident(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), r2.state.clone(), "RParen");
    if has_err(r3.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "Domain".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: r2.name.clone() }) }), None, zero_span.clone()) }), state: r3.state.clone(), err: None })
}
    }
    "range" => {
        {
    let r2 = parse_named_int_args(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), r2.state.clone(), "RParen");
    if has_err(r3.err.clone()) {
    return Rc::new(PredResult { predicate: dummy_pred.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let min_fields = if r2.min_val.clone().is_some() {
    Rc::new(vec!(Rc::new(FieldInit { name: "min".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitInt { value: r2.min_val.clone().unwrap() }) }), None, zero_span.clone()) })))
} else {
    Rc::new(Vec::new())
};
    let max_fields = if r2.max_val.clone().is_some() {
    Rc::new(vec!(Rc::new(FieldInit { name: "max".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitInt { value: r2.max_val.clone().unwrap() }) }), None, zero_span.clone()) })))
} else {
    Rc::new(Vec::new())
};
    Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "Range".to_string(), value: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: None, fields: v2_rt::concat(min_fields.clone(), max_fields.clone()), parent_enum: None }), None, zero_span.clone()) }), state: r3.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(PredResult { predicate: dummy_pred.clone(), state: e.state.clone(), err: Some(parse_error(&v2_rt::concat(v2_rt::concat("unknown where predicate `".to_string(), pred_name), "`".to_string()), current_span(tokens.clone(), e.state.clone()))) })
    }
}
} else {
    match pred_name.as_str() {
    "non_empty" => {
        Rc::new(PredResult { predicate: Rc::new(FieldInit { name: "NonEmpty".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, zero_span.clone()) }), state: s.clone(), err: None })
    }
    _ => {
        Rc::new(PredResult { predicate: dummy_pred.clone(), state: s.clone(), err: Some(parse_error(&v2_rt::concat(v2_rt::concat("unknown where predicate `".to_string(), pred_name), "`".to_string()), current_span(tokens.clone(), s.clone()))) })
    }
}
}
}

pub fn parse_named_int_args(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<RangeArgsResult> {
    let r = parse_single_named_int(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(RangeArgsResult { min_val: None, max_val: None, state: r.state.clone(), err: r.err.clone() });
};
    let e = eat(tokens.clone(), r.state.clone(), "Comma");
    if e.consumed.clone() {
    let r2 = parse_single_named_int(tokens.clone(), skip_newlines(tokens.clone(), e.state.clone()));
    if has_err(r2.err.clone()) {
    return Rc::new(RangeArgsResult { min_val: None, max_val: None, state: r2.state.clone(), err: r2.err.clone() });
};
    let min_val = if r.arg_name.clone() == "min" {
    Some(r.arg_value.clone())
} else {
    if r2.arg_name.clone() == "min" {
    Some(r2.arg_value.clone())
} else {
    None
}
};
    let max_val = if r.arg_name.clone() == "max" {
    Some(r.arg_value.clone())
} else {
    if r2.arg_name.clone() == "max" {
    Some(r2.arg_value.clone())
} else {
    None
}
};
    Rc::new(RangeArgsResult { min_val: min_val.clone(), max_val: max_val.clone(), state: r2.state.clone(), err: None })
} else {
    let min_val = if r.arg_name.clone() == "min" {
    Some(r.arg_value.clone())
} else {
    None
};
    let max_val = if r.arg_name.clone() == "max" {
    Some(r.arg_value.clone())
} else {
    None
};
    Rc::new(RangeArgsResult { min_val: min_val.clone(), max_val: max_val.clone(), state: r.state.clone(), err: None })
}
}

pub fn parse_single_named_int(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<NamedIntResult> {
    let r = expect_ident(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(NamedIntResult { arg_name: "".to_string(), arg_value: 0_i64, state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    return Rc::new(NamedIntResult { arg_name: name.clone(), arg_value: 0_i64, state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    return Rc::new(NamedIntResult { arg_name: name.clone(), arg_value: 0_i64, state: r3.state.clone(), err: r3.err.clone() });
};
    match r3.expr.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitInt { value: n, .. } = value.as_ref() else { unreachable!() };
        Rc::new(NamedIntResult { arg_name: name.clone(), arg_value: n.clone(), state: r3.state.clone(), err: None })
    }
    _ => {
        Rc::new(NamedIntResult { arg_name: name.clone(), arg_value: 0_i64, state: r3.state.clone(), err: Some(parse_error(&v2_rt::concat(v2_rt::concat("range() argument `".to_string(), name.clone()), "` requires an integer literal".to_string()), current_span(tokens.clone(), r3.state.clone()))) })
    }
}
}

pub fn parse_variant_fields(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, vname: &str) -> Rc<VariantResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let e = eat(tokens.clone(), state.clone(), "LBrace");
    if e.consumed.clone() {
    let r = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), e.state.clone()));
    if has_err(r.err.clone()) {
    return Rc::new(VariantResult { variant: Rc::new(Variant { name: vname.to_string(), fields: Rc::new(Vec::new()), span: start_span.clone() }), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(VariantResult { variant: Rc::new(Variant { name: vname.to_string(), fields: Rc::new(Vec::new()), span: start_span.clone() }), state: r2.state.clone(), err: r2.err.clone() });
};
    let v = Rc::new(Variant { name: vname.to_string(), fields: r.fields.clone(), span: start_span.clone() });
    Rc::new(VariantResult { variant: v.clone(), state: r2.state.clone(), err: None })
} else {
    let v = Rc::new(Variant { name: vname.to_string(), fields: Rc::new(Vec::new()), span: start_span.clone() });
    Rc::new(VariantResult { variant: v.clone(), state: state.clone(), err: None })
}
}

pub fn parse_more_variants(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<VariantsResult> {
    parse_more_variants_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_more_variants_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Variant>>>) -> Rc<VariantsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            let e = eat(tokens.clone(), s.clone(), "Pipe");
            if e.consumed.clone() {
    let s = skip_newlines(tokens.clone(), e.state.clone());
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(VariantsResult { variants: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_variant_fields(tokens.clone(), r.state.clone(), &r.name);
    if has_err(r2.err.clone()) {
    break Rc::new(VariantsResult { variants: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r2.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r2.variant.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(VariantsResult { variants: acc.clone(), state: s.clone(), err: None });
};
        }
    })
}

pub fn parse_type_expr(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<TypeResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let s = skip_newlines(tokens.clone(), state.clone());
        let k = peek_kind(tokens.clone(), s.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LBrace) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let inline_start = current_span(tokens.clone(), s.clone());
    let r = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), adv.state.clone()));
    if has_err(r.err.clone()) {
    return Rc::new(TypeResult { type_expr: leaf_type_node("", inline_start.clone()), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(TypeResult { type_expr: leaf_type_node("", inline_start.clone()), state: r2.state.clone(), err: r2.err.clone() });
};
    let span = current_span(tokens.clone(), s.clone());
    let te = Rc::new(Node { name: "".to_string(), span, children: {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in r.fields.iter().cloned() {
        __mapped_0.push(field_to_child_node(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
}, connective: Some(Connective::Conj), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(TypeResult { type_expr: te.clone(), state: r2.state.clone(), err: None })
}
    }
    Some(TokenKind::KwFn) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let start_span = current_span(tokens.clone(), s.clone());
    parse_callable_type_expr(tokens.clone(), adv.state.clone(), start_span)
}
    }
    Some(TokenKind::Ident { name: n, .. }) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let span = current_span(tokens.clone(), s.clone());
    finish_type_expr_from_name(tokens.clone(), adv.state.clone(), &n, span)
}
    }
    _ => {
        Rc::new(TypeResult { type_expr: leaf_type_node("", current_span(tokens.clone(), s.clone())), state: s.clone(), err: Some(parse_error("expected type expression", current_span(tokens.clone(), s.clone()))) })
    }
}
    })
}

pub fn parse_callable_type_expr(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, start_span: SourceSpan) -> Rc<TypeResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_te = leaf_type_node("Callable", start_span.clone());
        let r = expect(tokens.clone(), state.clone(), "LParen");
        if has_err(r.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let params_result = if peek_is_rparen(tokens.clone(), s.clone()) {
    Rc::new(ParamsResult { params: Rc::new(Vec::new()), state: s.clone(), err: None })
} else {
    parse_callable_param_types(tokens.clone(), s.clone(), Rc::new(Vec::new()))
};
        if has_err(params_result.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: params_result.state.clone(), err: params_result.err.clone() });
};
        let s2 = skip_newlines(tokens.clone(), params_result.state.clone());
        let r2 = expect(tokens.clone(), s2.clone(), "RParen");
        if has_err(r2.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
        let r3 = expect(tokens.clone(), r2.state.clone(), "Arrow");
        if has_err(r3.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
        let ret = parse_type_expr(tokens.clone(), r3.state.clone());
        if has_err(ret.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
        let te = callable_node(params_result.params.clone(), ret.type_expr.clone());
        let te_span = Rc::new(Node { name: te.name.clone(), span: start_span.clone(), children: te.children.clone(), connective: te.connective.clone(), params: te.params.clone(), return_type: te.return_type.clone(), return_cardinality: te.return_cardinality.clone(), uses: te.uses.clone(), body: te.body.clone(), transport: te.transport.clone(), properties: te.properties.clone(), type_annotation: te.type_annotation.clone(), config: te.config.clone(), is_self_recursive: te.is_self_recursive.clone(), has_non_tail_self_call: te.has_non_tail_self_call.clone(), expr_data: te.expr_data.clone() });
        maybe_optional(tokens.clone(), ret.state.clone(), te_span.clone(), start_span.clone())
    })
}

pub fn parse_callable_param_types(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Param>>>) -> Rc<ParamsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let r = parse_type_expr(tokens.clone(), state);
            if has_err(r.err.clone()) {
    break Rc::new(ParamsResult { params: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
            let param = Rc::new(Param { name: "".to_string(), type_expr: r.type_expr.clone(), default_value: None, span: r.type_expr.span.clone() });
            let acc = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(param.clone());
    Rc::new(__appended_0)
};
            let e = eat(tokens.clone(), r.state.clone(), "Comma");
            if e.consumed.clone() {
    let s = skip_newlines(tokens.clone(), e.state.clone());
    if peek_is_rparen(tokens.clone(), s.clone()) {
    break Rc::new(ParamsResult { params: acc.clone(), state: s.clone(), err: None });
} else {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s.clone();
        let __tco_2 = acc.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
} else {
    break Rc::new(ParamsResult { params: acc.clone(), state: r.state.clone(), err: None });
};
        }
    })
}

pub fn finish_type_expr_from_name(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, type_name: &str, start_span: SourceSpan) -> Rc<TypeResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_te = leaf_type_node(&type_name, start_span.clone());
        let e = eat(tokens.clone(), state.clone(), "Lt");
        if e.consumed.clone() {
    let r = parse_type_expr(tokens.clone(), e.state.clone());
    if has_err(r.err.clone()) {
    return r.clone();
};
    let first_arg = r.type_expr.clone();
    let s = r.state.clone();
    let type_args = collect_type_args(tokens.clone(), s.clone(), Rc::new(vec!(first_arg.clone())));
    if has_err(type_args.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: type_args.state.clone(), err: type_args.err.clone() });
};
    let r3 = expect(tokens.clone(), type_args.state.clone(), "Gt");
    if has_err(r3.err.clone()) {
    return Rc::new(TypeResult { type_expr: dummy_te.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let te = Rc::new(Node { name: type_name.to_string(), span: start_span.clone(), children: type_args.args.clone(), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    maybe_optional(tokens.clone(), r3.state.clone(), te.clone(), start_span.clone())
} else {
    let te = leaf_type_node(&type_name, start_span.clone());
    maybe_optional(tokens.clone(), state.clone(), te.clone(), start_span.clone())
}
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeParamsResult {
    pub params: Rc<Vec<Rc<Param>>>,
    pub state: Rc<ParserState>,
}

pub fn parse_optional_type_params(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<TypeParamsResult> {
    let e = eat(tokens.clone(), state.clone(), "Lt");
    if e.consumed.clone() {
    let params_result = collect_type_param_names(tokens.clone(), e.state.clone(), Rc::new(Vec::new()));
    let r = expect(tokens.clone(), params_result.state.clone(), "Gt");
    Rc::new(TypeParamsResult { params: params_result.params.clone(), state: r.state.clone() })
} else {
    Rc::new(TypeParamsResult { params: Rc::new(Vec::new()), state: state.clone() })
}
}

pub fn collect_type_param_names(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, params: Rc<Vec<Rc<Param>>>) -> Rc<TypeParamsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_params = params;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let params = __tco_p_params;
            if is_ident(tokens.clone(), state.clone()) {
    let r = expect_ident(tokens.clone(), state.clone());
    let span = current_span(tokens.clone(), r.state.clone());
    let param = Rc::new(Param { name: r.name.clone(), type_expr: leaf_type_node(&r.name, span.clone()), default_value: None, span: span.clone() });
    let next_params = {
    let __rc_1 = params;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(param.clone());
    Rc::new(__appended_0)
};
    let e = eat(tokens.clone(), r.state.clone(), "Comma");
    if e.consumed.clone() {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = e.state.clone();
        let __tco_2 = next_params.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_params = __tco_2;
        continue;
    }

} else {
    break Rc::new(TypeParamsResult { params: next_params.clone(), state: r.state.clone() });
};
} else {
    break Rc::new(TypeParamsResult { params: params.clone(), state: state.clone() });
};
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeArgsResult {
    pub args: Rc<Vec<Rc<Node>>>,
    pub state: Rc<ParserState>,
    pub err: Option<Rc<Diagnostic>>,
}

pub fn collect_type_args(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, args: Rc<Vec<Rc<Node>>>) -> Rc<TypeArgsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_args = args;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let args = __tco_p_args;
            let e = eat(tokens.clone(), state.clone(), "Comma");
            if e.consumed.clone() {
    let r = parse_type_expr(tokens.clone(), e.state.clone());
    if has_err(r.err.clone()) {
    break Rc::new(TypeArgsResult { args: args.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = {
    let __rc_1 = args;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.type_expr.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_args = __tco_2;
        continue;
    }

} else {
    break Rc::new(TypeArgsResult { args: args.clone(), state: state.clone(), err: None });
};
        }
    })
}

pub fn maybe_optional(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, te: Rc<Node>, start_span: SourceSpan) -> Rc<TypeResult> {
    let e = eat(tokens.clone(), state.clone(), "Question");
    if e.consumed.clone() {
    let ote = Rc::new(Node { name: te.name.clone(), span: te.span.clone(), children: te.children.clone(), connective: te.connective.clone(), params: te.params.clone(), return_type: te.return_type.clone(), return_cardinality: Cardinality::CardOptional, uses: te.uses.clone(), body: te.body.clone(), transport: te.transport.clone(), properties: te.properties.clone(), type_annotation: te.type_annotation.clone(), config: te.config.clone(), is_self_recursive: te.is_self_recursive.clone(), has_non_tail_self_call: te.has_non_tail_self_call.clone(), expr_data: te.expr_data.clone() });
    Rc::new(TypeResult { type_expr: ote.clone(), state: e.state.clone(), err: None })
} else {
    Rc::new(TypeResult { type_expr: te.clone(), state: state.clone(), err: None })
}
}

pub fn parse_field_list(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<FieldsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_field_list_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
    })
}

pub fn parse_field_list_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Field>>>) -> Rc<FieldsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if (peek_is_rbrace(tokens.clone(), s.clone()) || peek_is_rparen(tokens.clone(), s.clone())) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(FieldsResult { fields: acc.clone(), state: s.clone(), err: None });
} else {
    if is_ident(tokens.clone(), s.clone()) || is_keyword_name(tokens.clone(), s.clone()) {
    let r = parse_field(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(FieldsResult { fields: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = r.state.clone();
    let e = eat(tokens.clone(), s2.clone(), "Comma");
    let s3 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    s2.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s3.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.field.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(FieldsResult { fields: acc.clone(), state: s.clone(), err: None });
};
};
        }
    })
}

pub fn parse_field(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<FieldResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let start_span = current_span(tokens.clone(), state.clone());
        let dummy_field = Rc::new(Field { name: "".to_string(), type_expr: leaf_type_node("", start_span.clone()), cardinality: Cardinality::Required, default_value: None, from_key: None, span: start_span.clone() });
        let r = expect_name(tokens.clone(), state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(FieldResult { field: dummy_field.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let name = r.name.clone();
        let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
        if has_err(r2.err.clone()) {
    return Rc::new(FieldResult { field: dummy_field.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
        let r3 = parse_type_expr(tokens.clone(), r2.state.clone());
        if has_err(r3.err.clone()) {
    return Rc::new(FieldResult { field: dummy_field.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
        let te = r3.type_expr.clone();
        let s = r3.state.clone();
        let from_r = parse_optional_from_key(tokens.clone(), s.clone());
        let from_key = from_r.from_key.clone();
        let s = from_r.state.clone();
        let e = eat(tokens.clone(), s.clone(), "Eq");
        if e.consumed.clone() {
    let r4 = parse_expr(tokens.clone(), e.state.clone());
    if has_err(r4.err.clone()) {
    return Rc::new(FieldResult { field: dummy_field.clone(), state: r4.state.clone(), err: r4.err.clone() });
};
    let f = Rc::new(Field { name, type_expr: te.clone(), cardinality: te.return_cardinality.clone(), default_value: Some(r4.expr.clone()), from_key, span: start_span.clone() });
    Rc::new(FieldResult { field: f.clone(), state: r4.state.clone(), err: None })
} else {
    let f = Rc::new(Field { name, type_expr: te.clone(), cardinality: te.return_cardinality.clone(), default_value: None, from_key, span: start_span.clone() });
    Rc::new(FieldResult { field: f.clone(), state: s.clone(), err: None })
}
    })
}

pub fn parse_optional_from_key(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<FromKeyResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { ref name, .. }) if name == "from" => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let k2 = peek_kind(tokens.clone(), adv.state.clone());
    match k2.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LitStr { value: key, .. }) => {
        {
    let adv2 = advance(tokens.clone(), adv.state.clone());
    Rc::new(FromKeyResult { from_key: Some(key.clone()), state: adv2.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(FromKeyResult { from_key: None, state: state.clone(), err: None })
    }
}
}
    }
    _ => {
        Rc::new(FromKeyResult { from_key: None, state: state.clone(), err: None })
    }
}
}

pub fn parse_fn_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), state.clone(), "KwFn");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = parse_params(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let params = r.params.clone();
    let s = r.state.clone();
    let ret = parse_optional_return_type(tokens.clone(), s.clone());
    if has_err(ret.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
    let return_type = ret.return_type.clone();
    let s = ret.state.clone();
    let r = parse_block(tokens.clone(), skip_newlines(tokens.clone(), s.clone()));
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let body = r.expr.clone();
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: params.clone(), return_type: return_type.clone(), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: Some(body.clone()), connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), r.state.clone()), err: None })
}

pub fn parse_func_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let k = peek_kind(tokens.clone(), state.clone());
    let r = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwFunc) => {
        expect(tokens.clone(), state.clone(), "KwFunc")
    }
    Some(TokenKind::KwPattern) => {
        expect(tokens.clone(), state.clone(), "KwPattern")
    }
    Some(TokenKind::KwInterface) => {
        expect(tokens.clone(), state.clone(), "KwInterface")
    }
    _ => {
        expect(tokens.clone(), state.clone(), "KwFunc")
    }
};
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = parse_params(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let params = r.params.clone();
    let s = r.state.clone();
    let ret = parse_optional_return_type(tokens.clone(), s.clone());
    if has_err(ret.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
    let return_type = ret.return_type.clone();
    let s = ret.state.clone();
    let uses_r = parse_uses_clause(tokens.clone(), skip_newlines(tokens.clone(), s.clone()));
    if has_err(uses_r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: uses_r.state.clone(), err: uses_r.err.clone() });
};
    let uses = uses_r.uses.clone();
    let s = uses_r.state.clone();
    let r = parse_block(tokens.clone(), skip_newlines(tokens.clone(), s.clone()));
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let body = r.expr.clone();
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: params.clone(), return_type: return_type.clone(), return_cardinality: Cardinality::Required, uses: uses.clone(), body: Some(body.clone()), connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), r.state.clone()), err: None })
}

pub fn parse_uses_clause(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<UsesResult> {
    let s = skip_newlines(tokens.clone(), state.clone());
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { ref name, .. }) if name == "uses" => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = parse_uses_list(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    return r.clone();
};
    Rc::new(UsesResult { uses: r.uses.clone(), state: r.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(UsesResult { uses: Rc::new(Vec::new()), state: s.clone(), err: None })
    }
}
}

pub fn parse_uses_list(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<UsesResult> {
    parse_uses_list_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_uses_list_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<ResourceUse>>>) -> Rc<UsesResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let r = parse_uses_entry(tokens.clone(), state);
            if has_err(r.err.clone()) {
    break Rc::new(UsesResult { uses: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
            let acc = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.resource_use.clone());
    Rc::new(__appended_0)
};
            let s = r.state.clone();
            let e = eat(tokens.clone(), s.clone(), "Comma");
            if e.consumed.clone() {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = e.state.clone();
        let __tco_2 = acc.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(UsesResult { uses: acc.clone(), state: s.clone(), err: None });
};
        }
    })
}

pub fn parse_uses_entry(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ResUseResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(ResourceUse { name: "".to_string(), resource: leaf_type_node("", start_span.clone()), span: start_span.clone() });
    let r = expect_ident(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ResUseResult { resource_use: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    return Rc::new(ResUseResult { resource_use: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_type_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    return Rc::new(ResUseResult { resource_use: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let ru = Rc::new(ResourceUse { name, resource: r3.type_expr.clone(), span: start_span.clone() });
    Rc::new(ResUseResult { resource_use: ru.clone(), state: r3.state.clone(), err: None })
}

pub fn parse_optional_return_type(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<OptRetResult> {
    let e = eat(tokens.clone(), state.clone(), "Arrow");
    if e.consumed.clone() {
    let r = parse_type_expr(tokens.clone(), e.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(OptRetResult { return_type: None, state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(OptRetResult { return_type: Some(Rc::new(InferredNode::Resolved { node: r.type_expr.clone() })), state: r.state.clone(), err: None })
} else {
    Rc::new(OptRetResult { return_type: None, state: state.clone(), err: None })
}
}

pub fn parse_service_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), state.clone(), "KwService");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r_ns = expect_name(tokens.clone(), r.state.clone());
    if has_err(r_ns.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r_ns.state.clone(), err: r_ns.err.clone() });
};
    let namespace_root = r_ns.name.clone();
    let r = parse_dotted_ident_rest(tokens.clone(), r_ns.state.clone(), &namespace_root);
    let name = r.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), r.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r = parse_service_body(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let op_children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in r.operations.iter().cloned() {
        __mapped_0.push({
    let all_props = v2_rt::concat(v2_rt::concat(v2_rt::concat(__elem_1.modifier_props.clone(), __elem_1.response_props.clone()), __elem_1.exit_props.clone()), __elem_1.mock_props.clone());
    Rc::new(Node { name: __elem_1.name.clone(), span: __elem_1.span.clone(), children: Rc::new(Vec::new()), params: {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in __elem_1.inputs.iter().cloned() {
        __mapped_2.push(Rc::new(Param { name: __elem_3.name.clone(), type_expr: __elem_3.type_expr.clone(), default_value: __elem_3.default_value.clone(), span: __elem_3.span.clone() }));
    }
    Rc::new(__mapped_2)
}, return_type: outputs_to_return_type(__elem_1.outputs.clone(), __elem_1.span.clone()), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: __elem_1.transport.clone(), properties: all_props.clone(), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
});
    }
    Rc::new(__mapped_0)
};
    let ns_prop = Rc::new(FieldInit { name: "namespace_root".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: namespace_root.clone() }) }), None, start_span.clone()) });
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: op_children.clone(), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: Some(r.transport.clone()), properties: Rc::new(vec!(ns_prop.clone())), type_annotation: None, config: r.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), r2.state.clone()), err: None })
}

pub fn parse_service_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ServiceBodyResult> {
    parse_service_entries(tokens.clone(), state.clone(), None, local_transport_node(current_span(tokens.clone(), state.clone())), Rc::new(Vec::new()))
}

pub fn parse_service_entries(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, config: Option<Rc<ServiceConfig>>, transport: Rc<Node>, operations: Rc<Vec<Rc<OperationDef>>>) -> Rc<ServiceBodyResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_config = config;
        let mut __tco_p_transport = transport;
        let mut __tco_p_operations = operations;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let config = __tco_p_config;
            let transport = __tco_p_transport;
            let operations = __tco_p_operations;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: s.clone(), err: None });
} else {
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { ref name, .. }) if name == "config" => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_service_config_block(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r2.state.clone());
    let r3 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r3.state.clone();
        let __tco_2 = Some(r2.config.clone());
        let __tco_3 = transport.clone();
        let __tco_4 = operations.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_config = __tco_2;
        __tco_p_transport = __tco_3;
        __tco_p_operations = __tco_4;
        continue;
    }

};
    }
    Some(TokenKind::Ident { ref name, .. }) if name == "transport" => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = parse_transport_binding(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = config.clone();
        let __tco_3 = r.transport.clone();
        let __tco_4 = operations.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_config = __tco_2;
        __tco_p_transport = __tco_3;
        __tco_p_operations = __tco_4;
        continue;
    }

};
    }
    Some(TokenKind::KwOperation) => {
        {
    let r = parse_operation_def(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = config.clone();
        let __tco_3 = transport.clone();
        let __tco_4 = {
    let __rc_1 = operations;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.operation.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_config = __tco_2;
        __tco_p_transport = __tco_3;
        __tco_p_operations = __tco_4;
        continue;
    }

};
    }
    _ => {
        break Rc::new(ServiceBodyResult { config: config.clone(), transport: transport.clone(), operations: operations.clone(), state: s.clone(), err: Some(parse_error("expected config, transport, or operation in service block", current_span(tokens.clone(), s.clone()))) });
    }
};
};
        }
    })
}

pub fn parse_service_config_block(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ConfigResult> {
    parse_config_fields(tokens.clone(), state.clone(), None, None, None, None)
}

pub fn parse_config_fields(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, endpoint: Option<Rc<Node>>, auth: Option<Rc<Node>>, rate_limit: Option<Rc<Node>>, retry: Option<Rc<Node>>) -> Rc<ConfigResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_endpoint = endpoint;
        let mut __tco_p_auth = auth;
        let mut __tco_p_rate_limit = rate_limit;
        let mut __tco_p_retry = retry;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let endpoint = __tco_p_endpoint;
            let auth = __tco_p_auth;
            let rate_limit = __tco_p_rate_limit;
            let retry = __tco_p_retry;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    let cfg = Rc::new(ServiceConfig { endpoint: match endpoint.as_ref().map(|__rc| __rc.as_ref()) {
    Some(e) => {
        let e = Rc::new(e.clone());
        e.clone()
    }
    None => {
        make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "".to_string() }) }), None, SourceSpan { start: 0_i64, end: 0_i64 })
    }
}, auth: auth.clone(), rate_limit: rate_limit.clone(), retry: retry.clone() });
    break Rc::new(ConfigResult { config: cfg.clone(), state: s.clone(), err: None });
} else {
    let dummy_cfg = Rc::new(ServiceConfig { endpoint: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "".to_string() }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }), auth: None, rate_limit: None, retry: None });
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ConfigResult { config: dummy_cfg.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fname = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    break Rc::new(ConfigResult { config: dummy_cfg.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(ConfigResult { config: dummy_cfg.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let s2 = r3.state.clone();
    let e = eat(tokens.clone(), s2.clone(), "Comma");
    let s3 = if e.consumed.clone() {
    e.state.clone()
} else {
    s2.clone()
};
    match fname.as_str() {
    "endpoint" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s3.clone();
            let __tco_2 = Some(r3.expr.clone());
            let __tco_3 = auth.clone();
            let __tco_4 = rate_limit.clone();
            let __tco_5 = retry.clone();
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_endpoint = __tco_2;
            __tco_p_auth = __tco_3;
            __tco_p_rate_limit = __tco_4;
            __tco_p_retry = __tco_5;
            continue;
        }

    }
    "auth" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s3.clone();
            let __tco_2 = endpoint.clone();
            let __tco_3 = Some(r3.expr.clone());
            let __tco_4 = rate_limit.clone();
            let __tco_5 = retry.clone();
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_endpoint = __tco_2;
            __tco_p_auth = __tco_3;
            __tco_p_rate_limit = __tco_4;
            __tco_p_retry = __tco_5;
            continue;
        }

    }
    "rate_limit" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s3.clone();
            let __tco_2 = endpoint.clone();
            let __tco_3 = auth.clone();
            let __tco_4 = Some(r3.expr.clone());
            let __tco_5 = retry.clone();
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_endpoint = __tco_2;
            __tco_p_auth = __tco_3;
            __tco_p_rate_limit = __tco_4;
            __tco_p_retry = __tco_5;
            continue;
        }

    }
    "retry" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s3.clone();
            let __tco_2 = endpoint.clone();
            let __tco_3 = auth.clone();
            let __tco_4 = rate_limit.clone();
            let __tco_5 = Some(r3.expr.clone());
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_endpoint = __tco_2;
            __tco_p_auth = __tco_3;
            __tco_p_rate_limit = __tco_4;
            __tco_p_retry = __tco_5;
            continue;
        }

    }
    _ => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s3.clone();
            let __tco_2 = endpoint.clone();
            let __tco_3 = auth.clone();
            let __tco_4 = rate_limit.clone();
            let __tco_5 = retry.clone();
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_endpoint = __tco_2;
            __tco_p_auth = __tco_3;
            __tco_p_rate_limit = __tco_4;
            __tco_p_retry = __tco_5;
            continue;
        }

    }
};
};
        }
    })
}

pub fn parse_transport_binding(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<TransportResult> {
    let span = current_span(tokens.clone(), state.clone());
    let dummy = local_transport_node(span);
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { ref name, .. }) if name == "rest" => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(TransportResult { transport: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_rest_binding_body(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    return Rc::new(TransportResult { transport: r2.transport.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(TransportResult { transport: r2.transport.clone(), state: r3.state.clone(), err: None })
}
    }
    Some(TokenKind::Ident { ref name, .. }) if name == "shell" => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(TransportResult { transport: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_shell_binding_body(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    return Rc::new(TransportResult { transport: r2.transport.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(TransportResult { transport: r2.transport.clone(), state: r3.state.clone(), err: None })
}
    }
    Some(TokenKind::Ident { ref name, .. }) if name == "file" => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(TransportResult { transport: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_file_binding_body(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    return Rc::new(TransportResult { transport: r2.transport.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(TransportResult { transport: r2.transport.clone(), state: r3.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(TransportResult { transport: dummy.clone(), state: state.clone(), err: None })
    }
}
}

pub fn parse_rest_binding_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<TransportResult> {
    parse_rest_fields(tokens.clone(), state.clone(), None)
}

pub fn parse_rest_fields(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, base_url: Option<Rc<Node>>) -> Rc<TransportResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_base_url = base_url;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let base_url = __tco_p_base_url;
            let s = skip_newlines(tokens.clone(), state);
            let span = current_span(tokens.clone(), s.clone());
            let dummy = local_transport_node(span.clone());
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    let bu = match base_url.as_ref().map(|__rc| __rc.as_ref()) {
    Some(e) => {
        let e = Rc::new(e.clone());
        e.clone()
    }
    None => {
        make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "".to_string() }) }), None, SourceSpan { start: 0_i64, end: 0_i64 })
    }
};
    break Rc::new(TransportResult { transport: rest_transport_node(bu.clone(), Rc::new(Vec::new()), Rc::new(Vec::new()), span.clone()), state: s.clone(), err: None });
} else {
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fname = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let e = eat(tokens.clone(), r3.state.clone(), "Comma");
    let s2 = if e.consumed.clone() {
    e.state.clone()
} else {
    r3.state.clone()
};
    match fname.as_str() {
    "base_url" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s2.clone();
            let __tco_2 = Some(r3.expr.clone());
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_base_url = __tco_2;
            continue;
        }

    }
    _ => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s2.clone();
            let __tco_2 = base_url.clone();
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_base_url = __tco_2;
            continue;
        }

    }
};
};
        }
    })
}

pub fn parse_shell_binding_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<TransportResult> {
    parse_shell_fields(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_shell_fields(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, argv: Rc<Vec<Rc<Node>>>) -> Rc<TransportResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_argv = argv;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let argv = __tco_p_argv;
            let s = skip_newlines(tokens.clone(), state);
            let span = current_span(tokens.clone(), s.clone());
            let dummy = local_transport_node(span.clone());
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(TransportResult { transport: shell_transport_node(argv.clone(), Rc::new(Vec::new()), span.clone()), state: s.clone(), err: None });
} else {
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fname = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    match fname.as_str() {
    "argv" => {
        {
    let r3 = expect(tokens.clone(), r2.state.clone(), "LBracket");
    if has_err(r3.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let r4 = parse_expr_list_until(tokens.clone(), r3.state.clone(), "RBracket");
    if has_err(r4.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r4.state.clone(), err: r4.err.clone() });
};
    let r5 = expect(tokens.clone(), r4.state.clone(), "RBracket");
    if has_err(r5.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r5.state.clone(), err: r5.err.clone() });
};
    let e = eat(tokens.clone(), r5.state.clone(), "Comma");
    let s2 = if e.consumed.clone() {
    e.state.clone()
} else {
    r5.state.clone()
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = r4.exprs.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_argv = __tco_2;
        continue;
    }

};
    }
    _ => {
        {
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let e = eat(tokens.clone(), r3.state.clone(), "Comma");
    let s2 = if e.consumed.clone() {
    e.state.clone()
} else {
    r3.state.clone()
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = argv.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_argv = __tco_2;
        continue;
    }

};
    }
};
};
        }
    })
}

pub fn parse_file_binding_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<TransportResult> {
    parse_file_fields(tokens.clone(), state.clone(), None)
}

pub fn parse_file_fields(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, base_path: Option<Rc<Node>>) -> Rc<TransportResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_base_path = base_path;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let base_path = __tco_p_base_path;
            let s = skip_newlines(tokens.clone(), state);
            let span = current_span(tokens.clone(), s.clone());
            let dummy = local_transport_node(span.clone());
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    let bp = match base_path.as_ref().map(|__rc| __rc.as_ref()) {
    Some(e) => {
        let e = Rc::new(e.clone());
        e.clone()
    }
    None => {
        make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "".to_string() }) }), None, SourceSpan { start: 0_i64, end: 0_i64 })
    }
};
    break Rc::new(TransportResult { transport: file_transport_node(bp.clone(), span.clone()), state: s.clone(), err: None });
} else {
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fname = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(TransportResult { transport: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let e = eat(tokens.clone(), r3.state.clone(), "Comma");
    let s2 = if e.consumed.clone() {
    e.state.clone()
} else {
    r3.state.clone()
};
    match fname.as_str() {
    "path" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s2.clone();
            let __tco_2 = Some(r3.expr.clone());
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_base_path = __tco_2;
            continue;
        }

    }
    "base_path" => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s2.clone();
            let __tco_2 = Some(r3.expr.clone());
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_base_path = __tco_2;
            continue;
        }

    }
    _ => {
         {
            let __tco_0 = tokens.clone();
            let __tco_1 = s2.clone();
            let __tco_2 = base_path.clone();
            __tco_p_tokens = __tco_0;
            __tco_p_state = __tco_1;
            __tco_p_base_path = __tco_2;
            continue;
        }

    }
};
};
        }
    })
}

pub fn parse_operation_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<OpResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy_op = Rc::new(OperationDef { name: "".to_string(), inputs: Rc::new(Vec::new()), outputs: Rc::new(Vec::new()), response_props: Rc::new(Vec::new()), mock_props: Rc::new(Vec::new()), exit_props: Rc::new(Vec::new()), modifier_props: Rc::new(Vec::new()), transport: None, span: start_span.clone() });
    let r = expect(tokens.clone(), state.clone(), "KwOperation");
    if has_err(r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let s = skip_newlines(tokens.clone(), r.state.clone());
    if peek_is_lbrace(tokens.clone(), s.clone()) {
    parse_operation_v1_body(tokens.clone(), s.clone(), &name, start_span.clone())
} else {
    parse_operation_v2_inline(tokens.clone(), s.clone(), &name, start_span.clone())
}
}

pub fn parse_operation_v2_inline(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, name: &str, start_span: SourceSpan) -> Rc<OpResult> {
    let dummy_op = Rc::new(OperationDef { name: "".to_string(), inputs: Rc::new(Vec::new()), outputs: Rc::new(Vec::new()), response_props: Rc::new(Vec::new()), mock_props: Rc::new(Vec::new()), exit_props: Rc::new(Vec::new()), modifier_props: Rc::new(Vec::new()), transport: None, span: start_span.clone() });
    let mods_r = parse_operation_modifiers(tokens.clone(), state.clone());
    let modifiers = mods_r.modifiers.clone();
    let mod_props = modifiers_to_props(modifiers.clone(), start_span.clone());
    let s = mods_r.state.clone();
    let r = expect(tokens.clone(), s.clone(), "LParen");
    if has_err(r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let inputs = r.fields.clone();
    let r = expect(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "RParen");
    if has_err(r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = r.state.clone();
    let ret = parse_optional_return_type(tokens.clone(), s.clone());
    if has_err(ret.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
    let s = ret.state.clone();
    let outputs = match ret.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_return_type_to_outputs(rt.clone())
    }
    _ => {
        Rc::new(Vec::new())
    }
};
    let s = skip_newlines(tokens.clone(), s.clone());
    let resp_r = parse_optional_response_block(tokens.clone(), s.clone());
    if has_err(resp_r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: resp_r.state.clone(), err: resp_r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), resp_r.state.clone());
    let mock_r = parse_optional_mock_response_block(tokens.clone(), s.clone());
    if has_err(mock_r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: mock_r.state.clone(), err: mock_r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), mock_r.state.clone());
    let op = Rc::new(OperationDef { name: name.to_string(), inputs: inputs.clone(), outputs: outputs.clone(), response_props: resp_r.responses.clone(), mock_props: mock_r.mocks.clone(), exit_props: Rc::new(Vec::new()), modifier_props: mod_props.clone(), transport: None, span: start_span.clone() });
    Rc::new(OpResult { operation: op.clone(), state: s.clone(), err: None })
}

pub fn parse_operation_v1_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, name: &str, start_span: SourceSpan) -> Rc<OpResult> {
    let dummy_op = Rc::new(OperationDef { name: "".to_string(), inputs: Rc::new(Vec::new()), outputs: Rc::new(Vec::new()), response_props: Rc::new(Vec::new()), mock_props: Rc::new(Vec::new()), exit_props: Rc::new(Vec::new()), modifier_props: Rc::new(Vec::new()), transport: None, span: start_span.clone() });
    let r = expect(tokens.clone(), state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = parse_op_body_entries(tokens.clone(), s.clone(), Rc::new(Vec::new()), Rc::new(Vec::new()), Rc::new(Vec::new()), None, Rc::new(Vec::new()), Rc::new(Vec::new()), Rc::new(Vec::new()));
    if has_err(r2.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(OpResult { operation: dummy_op.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let op = Rc::new(OperationDef { name: name.to_string(), inputs: r2.inputs.clone(), outputs: r2.outputs.clone(), response_props: r2.response_props.clone(), mock_props: r2.mock_props.clone(), exit_props: r2.exit_props.clone(), modifier_props: r2.modifier_props.clone(), transport: r2.transport.clone(), span: start_span.clone() });
    Rc::new(OpResult { operation: op.clone(), state: skip_newlines(tokens.clone(), r3.state.clone()), err: None })
}

pub fn parse_op_body_entries(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, inputs: Rc<Vec<Rc<Field>>>, outputs: Rc<Vec<Rc<Field>>>, modifier_props: Rc<Vec<Rc<FieldInit>>>, transport: Option<Rc<Node>>, exit_props: Rc<Vec<Rc<FieldInit>>>, response_props: Rc<Vec<Rc<FieldInit>>>, mock_props: Rc<Vec<Rc<FieldInit>>>) -> Rc<OpBodyResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_inputs = inputs;
        let mut __tco_p_outputs = outputs;
        let mut __tco_p_modifier_props = modifier_props;
        let mut __tco_p_transport = transport;
        let mut __tco_p_exit_props = exit_props;
        let mut __tco_p_response_props = response_props;
        let mut __tco_p_mock_props = mock_props;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let inputs = __tco_p_inputs;
            let outputs = __tco_p_outputs;
            let modifier_props = __tco_p_modifier_props;
            let transport = __tco_p_transport;
            let exit_props = __tco_p_exit_props;
            let response_props = __tco_p_response_props;
            let mock_props = __tco_p_mock_props;
            let s = skip_newlines(tokens.clone(), state);
            let mk_result = Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: s.clone(), err: None });
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break mk_result.clone();
} else {
    let k = peek_kind(tokens.clone(), s.clone());
    let err_result = Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: s.clone(), err: None });
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwInput) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r3.state.clone();
        let __tco_2 = r2.fields.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

};
    }
    Some(TokenKind::KwOutput) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r3.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = r2.fields.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

};
    }
    Some(TokenKind::KwIdempotent) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let prop = modifier_to_prop("idempotent", current_span(tokens.clone(), s.clone()));
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = {
    let __rc_1 = modifier_props;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(prop.clone());
    Rc::new(__appended_0)
};
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

};
    }
    Some(TokenKind::KwReadonly) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let prop = modifier_to_prop("readonly", current_span(tokens.clone(), s.clone()));
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = {
    let __rc_3 = modifier_props;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(prop.clone());
    Rc::new(__appended_2)
};
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

};
    }
    Some(TokenKind::KwHermetic) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let prop = modifier_to_prop("hermetic", current_span(tokens.clone(), s.clone()));
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = {
    let __rc_5 = modifier_props;
    let mut __appended_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __appended_4.push(prop.clone());
    Rc::new(__appended_4)
};
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

};
    }
    Some(TokenKind::Ident { name: id, .. }) => {
        if id.clone() == "transport" {
    let adv = advance(tokens.clone(), s.clone());
    let r = parse_transport_binding(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = Some(r.transport.clone());
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

} else {
    if id.clone() == "exit" {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_exit_entries(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r3.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = transport.clone();
        let __tco_6 = r2.entries.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

} else {
    if id.clone() == "response" {
    let r = parse_optional_response_block(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = r.responses.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

} else {
    if id.clone() == "mock_response" {
    let r = parse_optional_mock_response_block(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = r.mocks.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

} else {
    if peek_is_colon_after_ident(tokens.clone(), s.clone()) {
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = skip_newlines(tokens.clone(), r3.state.clone());
        let __tco_2 = inputs.clone();
        let __tco_3 = outputs.clone();
        let __tco_4 = modifier_props.clone();
        let __tco_5 = transport.clone();
        let __tco_6 = exit_props.clone();
        let __tco_7 = response_props.clone();
        let __tco_8 = mock_props.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        __tco_p_modifier_props = __tco_4;
        __tco_p_transport = __tco_5;
        __tco_p_exit_props = __tco_6;
        __tco_p_response_props = __tco_7;
        __tco_p_mock_props = __tco_8;
        continue;
    }

} else {
    break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: s.clone(), err: Some(parse_error(&format!("unexpected '{}' in operation body", id.clone()), current_span(tokens.clone(), s.clone()))) });
};
};
};
};
};
    }
    _ => {
        break Rc::new(OpBodyResult { inputs: inputs.clone(), outputs: outputs.clone(), modifier_props: modifier_props.clone(), transport: transport.clone(), exit_props: exit_props.clone(), response_props: response_props.clone(), mock_props: mock_props.clone(), state: s.clone(), err: Some(parse_error("unexpected token in operation body", current_span(tokens.clone(), s.clone()))) });
    }
};
};
        }
    })
}

pub fn modifier_to_prop(name: &str, span: SourceSpan) -> Rc<FieldInit> {
    Rc::new(FieldInit { name: name.to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, span) })
}

pub fn modifiers_to_props(modifiers: Rc<Vec<OperationModifier>>, span: SourceSpan) -> Rc<Vec<Rc<FieldInit>>> {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in modifiers.iter().cloned() {
        __mapped_0.push(match __elem_1.clone() {
    OperationModifier::Idempotent => {
        modifier_to_prop("idempotent", span.clone())
    }
    OperationModifier::Readonly => {
        modifier_to_prop("readonly", span.clone())
    }
    OperationModifier::Hermetic => {
        modifier_to_prop("hermetic", span.clone())
    }
});
    }
    Rc::new(__mapped_0)
}
}

pub fn status_expr_to_str(expr: Rc<Node>) -> String {
    match expr.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        if let LiteralValue::LitInt { value: n, .. } = value.as_ref() {
int_to_string(n.clone())

} else if let LiteralValue::LitStr { value: s, .. } = value.as_ref() {
s.clone()

} else {
unreachable!()
}
    }
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        n.clone()
    }
    _ => {
        "_".to_string()
    }
}
}

pub fn int_to_string(value: i64) -> String {
    if value.clone() == 0_i64 {
    "0".to_string()
} else {
    let digits = int_to_string_acc(value.clone(), Rc::new(Vec::new()));
    {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in digits.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&"".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}
}
}

pub fn int_to_string_acc(value: i64, acc: Rc<Vec<String>>) -> Rc<Vec<String>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_value = value;
        let mut __tco_p_acc = acc;
        loop {
            let value = __tco_p_value;
            let acc = __tco_p_acc;
            if value.clone() == 0_i64 {
    break acc.clone();
} else {
    let digit = value.clone() % 10_i64;
    let rest = (value.clone() - digit.clone()) / 10_i64;
    let digit_chars = Rc::new(vec!("0".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string()));
    let ch = match {
    let mut __found_8 = None;
    for __elem_9 in ({
    let mut __enumerated_5 = Vec::new();
    for (__idx_6, __elem_7) in digit_chars.clone().iter().enumerate() {
        __enumerated_5.push((__idx_6 as i64, __elem_7.clone()));
    }
    Rc::new(__enumerated_5)
}).iter().cloned() {
        if __elem_9.0.clone() == digit.clone() {
    __found_8 = Some(__elem_9);
    break;
};
    }
    __found_8
} {
    Some(p) => {
        p.1.clone()
    }
    None => {
        "?".to_string()
    }
};
     {
        let __tco_0 = rest.clone();
        let __tco_1 = {
    let __rc_11 = acc;
    let mut __appended_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __appended_10.push(ch.clone());
    Rc::new(__appended_10)
};
        __tco_p_value = __tco_0;
        __tco_p_acc = __tco_1;
        continue;
    }

};
        }
    })
}

pub fn first_child_or_self(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().first().cloned() {
    Some(ch) => {
        ch.clone()
    }
    None => {
        n.clone()
    }
}
}

pub fn last_child_or_self(n: Rc<Node>) -> Rc<Node> {
    match n.children.clone().last().cloned() {
    Some(ch) => {
        ch.clone()
    }
    None => {
        n.clone()
    }
}
}

pub fn is_container_name(name: &str) -> bool {
    if name == "List" {
    true
} else {
    if name == "Set" {
    true
} else {
    if name == "NonEmptyList" {
    true
} else {
    if name == "NonEmptySet" {
    true
} else {
    false
}
}
}
}
}

pub fn node_to_name_str(n: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let has_children = ({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) > 0_i64;
        if node_is_optional(n.clone()) {
    v2_rt::concat("Optional_".to_string(), node_to_name_str(with_required_cardinality(n.clone())))
} else {
    if n.name.clone() == "Map" {
    if ({
    let __len_1 = n.children.clone().len();
    __len_1 as i64
}) > 1_i64 {
    v2_rt::concat("Map_".to_string(), node_to_name_str(last_child_or_self(n.clone())))
} else {
    n.name.clone()
}
} else {
    if n.name.clone() == "Refined" {
    if has_children.clone() {
    node_to_name_str(first_child_or_self(n.clone()))
} else {
    n.name.clone()
}
} else {
    if is_container_name(&n.name) {
    if has_children.clone() {
    v2_rt::concat("List_".to_string(), node_to_name_str(first_child_or_self(n.clone())))
} else {
    n.name.clone()
}
} else {
    if n.name.clone() == "" {
    if node_is_product(n.clone()) {
    "Record".to_string()
} else {
    if node_is_coproduct(n.clone()) {
    "Union".to_string()
} else {
    n.name.clone()
}
}
} else {
    n.name.clone()
}
}
}
}
}
    })
}

pub fn parse_exit_entries(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExitEntriesResult> {
    parse_exit_entries_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_exit_entries_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<FieldInit>>>) -> Rc<ExitEntriesResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(ExitEntriesResult { entries: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_status_pattern(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExitEntriesResult { entries: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let code = r.expr.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "FatArrow");
    if has_err(r2.err.clone()) {
    break Rc::new(ExitEntriesResult { entries: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_type_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(ExitEntriesResult { entries: Rc::new(Vec::new()), state: r3.state.clone(), err: r3.err.clone() });
};
    let desc_k = peek_kind(tokens.clone(), r3.state.clone());
    let desc_r = match desc_k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LitStr { value: d, .. }) => {
        {
    let adv = advance(tokens.clone(), r3.state.clone());
    Rc::new(DescResult { desc: Some(d.clone()), state: adv.state.clone() })
}
    }
    _ => {
        Rc::new(DescResult { desc: None, state: r3.state.clone() })
    }
};
    let code_str = status_expr_to_str(code.clone());
    let type_name = node_to_name_str(r3.type_expr.clone());
    let prop_name = v2_rt::concat("exit_".to_string(), code_str);
    let entry = Rc::new(FieldInit { name: prop_name.clone(), value: make_expr_node(Rc::new(ExprData::ExprVar { name: type_name, binding_kind: None }), None, r3.type_expr.span.clone()) });
    let e = eat(tokens.clone(), desc_r.state.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    desc_r.state.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(entry.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_operation_modifiers(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ModsResult> {
    parse_operation_modifiers_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_operation_modifiers_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<OperationModifier>>) -> Rc<ModsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let k = peek_kind(tokens.clone(), state.clone());
            match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwIdempotent) => {
        {
    let adv = advance(tokens.clone(), state.clone());
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(OperationModifier::Idempotent);
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
    }
    Some(TokenKind::KwReadonly) => {
        {
    let adv = advance(tokens.clone(), state.clone());
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = {
    let __rc_3 = acc;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(OperationModifier::Readonly);
    Rc::new(__appended_2)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
    }
    Some(TokenKind::KwHermetic) => {
        {
    let adv = advance(tokens.clone(), state.clone());
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = {
    let __rc_5 = acc;
    let mut __appended_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __appended_4.push(OperationModifier::Hermetic);
    Rc::new(__appended_4)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
    }
    _ => {
        break Rc::new(ModsResult { modifiers: acc.clone(), state: state.clone(), err: None });
    }
};
        }
    })
}

pub fn parse_status_pattern(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LitInt { value: n, .. }) => {
        if (state.pos.clone() + 1_i64) < ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
}) {
    let next_tok = tokens.clone().get((state.pos.clone() + 1_i64) as usize).cloned();
    match next_tok.clone() {
    Some(t) => {
        match t.kind.as_ref() {
    TokenKind::Ident { ref name, .. } if name == "xx" => {
        {
    let span = current_span(tokens.clone(), state.clone());
    let adv = advance(tokens.clone(), state.clone());
    let adv2 = advance(tokens.clone(), adv.state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: format!("{}xx", n.clone()) }) }), None, span), state: adv2.state.clone(), err: None })
}
    }
    _ => {
        {
    let span = current_span(tokens.clone(), state.clone());
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitInt { value: n.clone() }) }), None, span), state: adv.state.clone(), err: None })
}
    }
}
    }
    None => {
        {
    let span = current_span(tokens.clone(), state.clone());
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitInt { value: n.clone() }) }), None, span), state: adv.state.clone(), err: None })
}
    }
}
} else {
    let span = current_span(tokens.clone(), state.clone());
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitInt { value: n.clone() }) }), None, span), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::Ident { name: id, .. }) => {
        {
    let span = current_span(tokens.clone(), state.clone());
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: id.clone() }) }), None, span), state: adv.state.clone(), err: None })
}
    }
    _ => {
        {
    let span = current_span(tokens.clone(), state.clone());
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: "_".to_string() }) }), None, span), state: adv.state.clone(), err: None })
}
    }
}
}

pub fn parse_optional_response_block(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ResponsesResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { ref name, .. }) if name == "response" => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(ResponsesResult { responses: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_response_entries(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    return Rc::new(ResponsesResult { responses: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(ResponsesResult { responses: Rc::new(Vec::new()), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(ResponsesResult { responses: r2.entries.clone(), state: r3.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(ResponsesResult { responses: Rc::new(Vec::new()), state: state.clone(), err: None })
    }
}
}

pub fn parse_response_entries(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<RespEntriesResult> {
    parse_response_entries_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_response_entries_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<FieldInit>>>) -> Rc<RespEntriesResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(RespEntriesResult { entries: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_status_pattern(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(RespEntriesResult { entries: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let status = r.expr.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "FatArrow");
    if has_err(r2.err.clone()) {
    break Rc::new(RespEntriesResult { entries: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_type_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(RespEntriesResult { entries: Rc::new(Vec::new()), state: r3.state.clone(), err: r3.err.clone() });
};
    let status_str = status_expr_to_str(status.clone());
    let type_name = node_to_name_str(r3.type_expr.clone());
    let prop_name = v2_rt::concat("response_".to_string(), status_str);
    let entry = Rc::new(FieldInit { name: prop_name.clone(), value: make_expr_node(Rc::new(ExprData::ExprVar { name: type_name, binding_kind: None }), None, r3.type_expr.span.clone()) });
    let e = eat(tokens.clone(), r3.state.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    r3.state.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(entry.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_optional_mock_response_block(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<MocksResult> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { ref name, .. }) if name == "mock_response" => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(MocksResult { mocks: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_mock_response_entries(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    return Rc::new(MocksResult { mocks: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(MocksResult { mocks: Rc::new(Vec::new()), state: r3.state.clone(), err: r3.err.clone() });
};
    Rc::new(MocksResult { mocks: r2.entries.clone(), state: r3.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(MocksResult { mocks: Rc::new(Vec::new()), state: state.clone(), err: None })
    }
}
}

pub fn parse_mock_response_entries(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<MockEntriesResult> {
    parse_mock_response_entries_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_mock_response_entries_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<FieldInit>>>) -> Rc<MockEntriesResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(MockEntriesResult { entries: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_status_pattern(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(MockEntriesResult { entries: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let status = r.expr.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "FatArrow");
    if has_err(r2.err.clone()) {
    break Rc::new(MockEntriesResult { entries: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(MockEntriesResult { entries: Rc::new(Vec::new()), state: r3.state.clone(), err: r3.err.clone() });
};
    let body = r3.expr.clone();
    let desc_k = peek_kind(tokens.clone(), r3.state.clone());
    let desc_r = match desc_k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LitStr { value: d, .. }) => {
        {
    let adv = advance(tokens.clone(), r3.state.clone());
    Rc::new(DescResult { desc: Some(d.clone()), state: adv.state.clone() })
}
    }
    _ => {
        Rc::new(DescResult { desc: None, state: r3.state.clone() })
    }
};
    let status_str = status_expr_to_str(status.clone());
    let prop_name = v2_rt::concat("mock_".to_string(), status_str);
    let entry = Rc::new(FieldInit { name: prop_name.clone(), value: body.clone() });
    let e = eat(tokens.clone(), desc_r.state.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    desc_r.state.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(entry.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_resource_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), state.clone(), "KwResource");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), r.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = parse_resource_entries(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), Rc::new(Vec::new()), Rc::new(Vec::new()));
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = expect(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let cap_children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in r.capabilities.iter().cloned() {
        __mapped_0.push(Rc::new(Node { name: __elem_1.name.clone(), span: __elem_1.span.clone(), children: Rc::new(Vec::new()), params: {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in __elem_1.inputs.iter().cloned() {
        __mapped_2.push(Rc::new(Param { name: __elem_3.name.clone(), type_expr: __elem_3.type_expr.clone(), default_value: __elem_3.default_value.clone(), span: __elem_3.span.clone() }));
    }
    Rc::new(__mapped_2)
}, return_type: outputs_to_return_type(__elem_1.outputs.clone(), __elem_1.span.clone()), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }));
    }
    Rc::new(__mapped_0)
};
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: cap_children.clone(), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: r.properties.clone(), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), r2.state.clone()), err: None })
}

pub fn parse_resource_entries(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, properties: Rc<Vec<Rc<FieldInit>>>, capabilities: Rc<Vec<Rc<CapabilityDef>>>) -> Rc<ResPropResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_properties = properties;
        let mut __tco_p_capabilities = capabilities;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let properties = __tco_p_properties;
            let capabilities = __tco_p_capabilities;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: s.clone(), err: None });
} else {
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwCapability) => {
        {
    let r = parse_capability(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = properties.clone();
        let __tco_3 = {
    let __rc_1 = capabilities;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.capability.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_properties = __tco_2;
        __tco_p_capabilities = __tco_3;
        continue;
    }

};
    }
    Some(TokenKind::KwAcquire) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = skip_until_rbrace(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    let r3 = expect(tokens.clone(), r2.state.clone(), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = skip_newlines(tokens.clone(), r3.state.clone());
        let __tco_2 = properties.clone();
        let __tco_3 = capabilities.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_properties = __tco_2;
        __tco_p_capabilities = __tco_3;
        continue;
    }

};
    }
    Some(TokenKind::KwRelease) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = skip_until_rbrace(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    let r3 = expect(tokens.clone(), r2.state.clone(), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = skip_newlines(tokens.clone(), r3.state.clone());
        let __tco_2 = properties.clone();
        let __tco_3 = capabilities.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_properties = __tco_2;
        __tco_p_capabilities = __tco_3;
        continue;
    }

};
    }
    Some(TokenKind::Ident { name: _, .. }) => {
        if peek_is_colon_after_ident(tokens.clone(), s.clone()) {
    let r = expect_ident(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fname = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let fi = Rc::new(FieldInit { name: fname, value: r3.expr.clone() });
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = skip_newlines(tokens.clone(), r3.state.clone());
        let __tco_2 = {
    let __rc_3 = properties;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(fi.clone());
    Rc::new(__appended_2)
};
        let __tco_3 = capabilities.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_properties = __tco_2;
        __tco_p_capabilities = __tco_3;
        continue;
    }

} else {
    break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: s.clone(), err: Some(parse_error("unexpected token in resource block", current_span(tokens.clone(), s.clone()))) });
};
    }
    _ => {
        break Rc::new(ResPropResult { properties: properties.clone(), capabilities: capabilities.clone(), state: s.clone(), err: Some(parse_error("expected capability, acquire, release, or property in resource block", current_span(tokens.clone(), s.clone()))) });
    }
};
};
        }
    })
}

pub fn skip_until_rbrace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<UnitResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let s = skip_newlines(tokens.clone(), state.clone());
        if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    Rc::new(UnitResult { state: s.clone(), err: None })
} else {
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LBrace) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let inner = skip_until_rbrace(tokens.clone(), adv.state.clone());
    let r = expect(tokens.clone(), inner.state.clone(), "RBrace");
    if has_err(r.err.clone()) {
    return Rc::new(UnitResult { state: r.state.clone(), err: r.err.clone() });
};
    skip_until_rbrace(tokens.clone(), r.state.clone())
}
    }
    _ => {
        {
    let adv = advance(tokens.clone(), s.clone());
    skip_until_rbrace(tokens.clone(), adv.state.clone())
}
    }
}
}
    })
}

pub fn parse_capability(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<CapResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy_cap = Rc::new(CapabilityDef { name: "".to_string(), inputs: Rc::new(Vec::new()), outputs: Rc::new(Vec::new()), span: start_span.clone() });
    let r = expect(tokens.clone(), state.clone(), "KwCapability");
    if has_err(r.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let s = r.state.clone();
    if peek_is_lbrace(tokens.clone(), s.clone()) {
    let r2 = expect(tokens.clone(), s.clone(), "LBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let io = parse_input_output_blocks(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()));
    if has_err(io.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: io.state.clone(), err: io.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), io.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let cap = Rc::new(CapabilityDef { name, inputs: io.inputs.clone(), outputs: io.outputs.clone(), span: start_span.clone() });
    Rc::new(CapResult { capability: cap.clone(), state: skip_newlines(tokens.clone(), r3.state.clone()), err: None })
} else {
    if peek_is_lparen(tokens.clone(), s.clone()) {
    let r2 = expect(tokens.clone(), s.clone(), "LParen");
    if has_err(r2.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()));
    if has_err(r3.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let inputs = r3.fields.clone();
    let r4 = expect(tokens.clone(), skip_newlines(tokens.clone(), r3.state.clone()), "RParen");
    if has_err(r4.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: r4.state.clone(), err: r4.err.clone() });
};
    let ret = parse_optional_return_type(tokens.clone(), r4.state.clone());
    if has_err(ret.err.clone()) {
    return Rc::new(CapResult { capability: dummy_cap.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
    let outputs = match ret.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_return_type_to_outputs(rt.clone())
    }
    _ => {
        Rc::new(Vec::new())
    }
};
    let cap = Rc::new(CapabilityDef { name, inputs: inputs.clone(), outputs: outputs.clone(), span: start_span.clone() });
    Rc::new(CapResult { capability: cap.clone(), state: skip_newlines(tokens.clone(), ret.state.clone()), err: None })
} else {
    let cap = Rc::new(CapabilityDef { name, inputs: Rc::new(Vec::new()), outputs: Rc::new(Vec::new()), span: start_span.clone() });
    Rc::new(CapResult { capability: cap.clone(), state: skip_newlines(tokens.clone(), s.clone()), err: None })
}
}
}

pub fn parse_input_output_blocks(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<IOResult> {
    parse_io_blocks_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()), Rc::new(Vec::new()))
}

pub fn parse_io_blocks_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, inputs: Rc<Vec<Rc<Field>>>, outputs: Rc<Vec<Rc<Field>>>) -> Rc<IOResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_inputs = inputs;
        let mut __tco_p_outputs = outputs;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let inputs = __tco_p_inputs;
            let outputs = __tco_p_outputs;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: s.clone(), err: None });
} else {
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwInput) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r3.state.clone();
        let __tco_2 = r2.fields.clone();
        let __tco_3 = outputs.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        continue;
    }

};
    }
    Some(TokenKind::KwOutput) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let r = expect(tokens.clone(), adv.state.clone(), "LBrace");
    if has_err(r.err.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = parse_field_list(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
    if has_err(r2.err.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = expect(tokens.clone(), skip_newlines(tokens.clone(), r2.state.clone()), "RBrace");
    if has_err(r3.err.clone()) {
    break Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r3.state.clone();
        let __tco_2 = inputs.clone();
        let __tco_3 = r2.fields.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_inputs = __tco_2;
        __tco_p_outputs = __tco_3;
        continue;
    }

};
    }
    _ => {
        {
    let result = Rc::new(IOResult { inputs: inputs.clone(), outputs: outputs.clone(), state: s.clone(), err: None });
    break result.clone();
};
    }
};
};
        }
    })
}

pub fn parse_data_def(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), state.clone(), "KwData");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = expect_ident(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = parse_type_expr(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let te = r.type_expr.clone();
    let r = expect(tokens.clone(), r.state.clone(), "Eq");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let r = parse_expr(tokens.clone(), r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: Some(r.expr.clone()), connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: Some(te.clone()), config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), r.state.clone()), err: None })
}

pub fn parse_extern_decl(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ItemResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy = Rc::new(Node { name: "".to_string(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r = expect(tokens.clone(), state.clone(), "KwExtern");
    if has_err(r.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let k = peek_kind(tokens.clone(), r.state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwFn) => {
        {
    let adv = advance(tokens.clone(), r.state.clone());
    let r2 = expect_ident(tokens.clone(), adv.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let name = r2.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r3 = parse_params(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let ret = parse_optional_return_type(tokens.clone(), r3.state.clone());
    if has_err(ret.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
    let return_type = if ret.return_type.clone().is_some() {
    ret.return_type.clone()
} else {
    Some(Rc::new(InferredNode::Resolved { node: leaf_type_node("Unit", start_span.clone()) }))
};
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: r3.params.clone(), return_type: return_type.clone(), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), ret.state.clone()), err: None })
}
    }
    Some(TokenKind::KwFunc) => {
        {
    let adv = advance(tokens.clone(), r.state.clone());
    let r2 = expect_ident(tokens.clone(), adv.state.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(ItemResult { item: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let name = r2.name.clone();
    let named_dummy = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let r3 = parse_params(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let ret = parse_optional_return_type(tokens.clone(), r3.state.clone());
    if has_err(ret.err.clone()) {
    return Rc::new(ItemResult { item: named_dummy.clone(), state: ret.state.clone(), err: ret.err.clone() });
};
    let return_type = if ret.return_type.clone().is_some() {
    ret.return_type.clone()
} else {
    Some(Rc::new(InferredNode::Resolved { node: leaf_type_node("Unit", start_span.clone()) }))
};
    let item = Rc::new(Node { name: name.clone(), span: start_span.clone(), children: Rc::new(Vec::new()), params: r3.params.clone(), return_type: return_type.clone(), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, connective: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(ItemResult { item: item.clone(), state: skip_newlines(tokens.clone(), ret.state.clone()), err: None })
}
    }
    _ => {
        Rc::new(ItemResult { item: dummy.clone(), state: r.state.clone(), err: Some(parse_error("expected fn or func after extern", current_span(tokens.clone(), r.state.clone()))) })
    }
}
}

pub fn parse_params(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ParamsResult> {
    let r = expect(tokens.clone(), state.clone(), "LParen");
    if has_err(r.err.clone()) {
    return Rc::new(ParamsResult { params: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    if peek_is_rparen(tokens.clone(), s.clone()) {
    let adv = advance(tokens.clone(), s.clone());
    Rc::new(ParamsResult { params: Rc::new(Vec::new()), state: adv.state.clone(), err: None })
} else {
    let r = parse_param_list(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return r.clone();
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s.clone(), "RParen");
    if has_err(r2.err.clone()) {
    return Rc::new(ParamsResult { params: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ParamsResult { params: r.params.clone(), state: r2.state.clone(), err: None })
}
}

pub fn parse_param_list(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ParamsResult> {
    parse_param_list_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
}

pub fn parse_param_list_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Param>>>) -> Rc<ParamsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let r = parse_param(tokens.clone(), state);
            if has_err(r.err.clone()) {
    break Rc::new(ParamsResult { params: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
            let acc = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.param.clone());
    Rc::new(__appended_0)
};
            let s = r.state.clone();
            let e = eat(tokens.clone(), s.clone(), "Comma");
            if e.consumed.clone() {
    let s2 = skip_newlines(tokens.clone(), e.state.clone());
    if peek_is_rparen(tokens.clone(), s2.clone()) {
    break Rc::new(ParamsResult { params: acc.clone(), state: s2.clone(), err: None });
} else {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = acc.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
} else {
    break Rc::new(ParamsResult { params: acc.clone(), state: s.clone(), err: None });
};
        }
    })
}

pub fn parse_param(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ParamResult> {
    let start_span = current_span(tokens.clone(), state.clone());
    let dummy_param = Rc::new(Param { name: "".to_string(), type_expr: leaf_type_node("", start_span.clone()), default_value: None, span: start_span.clone() });
    let r = expect_name(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ParamResult { param: dummy_param.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let name = r.name.clone();
    let r2 = expect(tokens.clone(), r.state.clone(), "Colon");
    if has_err(r2.err.clone()) {
    return Rc::new(ParamResult { param: dummy_param.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    let r3 = parse_type_expr(tokens.clone(), r2.state.clone());
    if has_err(r3.err.clone()) {
    return Rc::new(ParamResult { param: dummy_param.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
    let s = r3.state.clone();
    let e = eat(tokens.clone(), s.clone(), "Eq");
    if e.consumed.clone() {
    let r4 = parse_expr(tokens.clone(), e.state.clone());
    if has_err(r4.err.clone()) {
    return Rc::new(ParamResult { param: dummy_param.clone(), state: r4.state.clone(), err: r4.err.clone() });
};
    let p = Rc::new(Param { name, type_expr: r3.type_expr.clone(), default_value: Some(r4.expr.clone()), span: start_span.clone() });
    Rc::new(ParamResult { param: p.clone(), state: r4.state.clone(), err: None })
} else {
    let p = Rc::new(Param { name, type_expr: r3.type_expr.clone(), default_value: None, span: start_span.clone() });
    Rc::new(ParamResult { param: p.clone(), state: s.clone(), err: None })
}
}

pub fn parse_block(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "LBrace");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let r = parse_stmts(tokens.clone(), s.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let stmts = r.stmts.clone();
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let r = expect(tokens.clone(), s.clone(), "RBrace");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        if ({
    let __len_0 = stmts.clone().len();
    __len_0 as i64
}) == 1_i64 {
    Rc::new(ExprResult { expr: stmts.clone().first().cloned().unwrap(), state: r.state.clone(), err: None })
} else {
    let span = current_span(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: stmts.clone() }), None, span), state: r.state.clone(), err: None })
}
    })
}

pub fn parse_stmts(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<StmtsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_stmts_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
    })
}

pub fn parse_stmts_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Node>>>) -> Rc<StmtsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if (peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone())) || peek_is_eof(tokens.clone(), s.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_stmt(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.expr.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_stmt(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let k = peek_kind(tokens.clone(), state.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwLet) => {
        parse_let(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwReturn) => {
        parse_return(tokens.clone(), state.clone())
    }
    Some(TokenKind::Ident { name: _, .. }) => {
        if peek_is_eq_after_ident(tokens.clone(), state.clone()) {
    parse_bare_assignment(tokens.clone(), state.clone())
} else {
    parse_expr(tokens.clone(), state.clone())
}
    }
    _ => {
        parse_expr(tokens.clone(), state.clone())
    }
}
    })
}

pub fn peek_is_eq_after_ident(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    if (state.pos.clone() + 1_i64) < ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
}) {
    let next_tok = tokens.clone().get((state.pos.clone() + 1_i64) as usize).cloned();
    match next_tok.clone() {
    Some(t) => {
        is_eq_kind(t.kind.clone())
    }
    None => {
        false
    }
}
} else {
    false
}
}

pub fn parse_bare_assignment(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect_ident(tokens.clone(), state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let name = r.name.clone();
        let r2 = expect(tokens.clone(), r.state.clone(), "Eq");
        if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
        let r3 = parse_expr(tokens.clone(), r2.state.clone());
        if has_err(r3.err.clone()) {
    return r3.clone();
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLet { name, value: r3.expr.clone(), body: None }), None, span), state: r3.state.clone(), err: None })
    })
}

pub fn parse_expr(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_expr_bp(tokens.clone(), state.clone(), 0_i64)
    })
}

pub fn parse_expr_bp(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, min_bp: i64) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let r = parse_prefix(tokens.clone(), state.clone());
        if has_err(r.err.clone()) {
    return r.clone();
};
        let lhs = r.expr.clone();
        let s = r.state.clone();
        parse_expr_loop(tokens.clone(), s.clone(), lhs.clone(), min_bp)
    })
}

pub fn parse_expr_loop(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, lhs: Rc<Node>, min_bp: i64) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_lhs = lhs;
        let mut __tco_p_min_bp = min_bp;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let lhs = __tco_p_lhs;
            let min_bp = __tco_p_min_bp;
            if at_end(tokens.clone(), state.clone()) || peek_is_eof(tokens.clone(), state.clone()) {
    break Rc::new(ExprResult { expr: lhs.clone(), state: state.clone(), err: None });
} else {
    let s = skip_continuation_newlines(tokens.clone(), state.clone());
    let post = try_postfix(tokens.clone(), s.clone(), lhs.clone(), min_bp.clone());
    if has_err(post.err.clone()) {
    break Rc::new(ExprResult { expr: lhs.clone(), state: post.state.clone(), err: post.err.clone() });
} else {
    if post.changed.clone() {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = post.state.clone();
        let __tco_2 = post.expr.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    let bp = infix_bp(tokens.clone(), s.clone());
    match bp {
    Some(bps) => {
        if bps.left.clone() < min_bp.clone() {
    break Rc::new(ExprResult { expr: lhs.clone(), state: s.clone(), err: None });
} else {
    let adv = advance(tokens.clone(), s.clone());
    let op_kind = adv.token.kind.clone();
    if is_dot_kind(op_kind.clone()) {
    let r = expect_name(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: lhs.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), state.clone());
    let new_lhs = make_expr_node(Rc::new(ExprData::ExprFieldAccess { base: lhs.clone(), field: r.name.clone(), summary: None }), None, span);
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = new_lhs.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    if is_pipe_arrow_kind(op_kind.clone()) {
    let span = current_span(tokens.clone(), state.clone());
    let r = parse_pipe_rhs(tokens.clone(), adv.state.clone(), lhs.clone(), span);
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = r.expr.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    let rhs_state = skip_newlines(tokens.clone(), adv.state.clone());
    let r = parse_expr_bp(tokens.clone(), rhs_state.clone(), bps.right.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), state.clone());
    let binop = token_to_binop(op_kind.clone());
    let new_lhs = make_expr_node(Rc::new(ExprData::ExprBinOp { op: binop, left: lhs.clone(), right: r.expr.clone() }), None, span);
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = new_lhs.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

};
};
};
    }
    None => {
        break Rc::new(ExprResult { expr: lhs.clone(), state: s.clone(), err: None });
    }
};
};
};
};
        }
    })
}

pub fn infix_bp(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Option<BindingPower> {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::NullCoalesce) => {
        Some(BindingPower { left: 3_i64, right: 4_i64 })
    }
    Some(TokenKind::Or) => {
        Some(BindingPower { left: 5_i64, right: 6_i64 })
    }
    Some(TokenKind::And) => {
        Some(BindingPower { left: 7_i64, right: 8_i64 })
    }
    Some(TokenKind::EqEq) => {
        Some(BindingPower { left: 9_i64, right: 10_i64 })
    }
    Some(TokenKind::Ne) => {
        Some(BindingPower { left: 9_i64, right: 10_i64 })
    }
    Some(TokenKind::Lt) => {
        Some(BindingPower { left: 11_i64, right: 12_i64 })
    }
    Some(TokenKind::Gt) => {
        Some(BindingPower { left: 11_i64, right: 12_i64 })
    }
    Some(TokenKind::Le) => {
        Some(BindingPower { left: 11_i64, right: 12_i64 })
    }
    Some(TokenKind::Ge) => {
        Some(BindingPower { left: 11_i64, right: 12_i64 })
    }
    Some(TokenKind::Plus) => {
        Some(BindingPower { left: 13_i64, right: 14_i64 })
    }
    Some(TokenKind::Minus) => {
        Some(BindingPower { left: 13_i64, right: 14_i64 })
    }
    Some(TokenKind::Star) => {
        Some(BindingPower { left: 15_i64, right: 16_i64 })
    }
    Some(TokenKind::Slash) => {
        Some(BindingPower { left: 15_i64, right: 16_i64 })
    }
    Some(TokenKind::Percent) => {
        Some(BindingPower { left: 15_i64, right: 16_i64 })
    }
    Some(TokenKind::PipeArrow) => {
        Some(BindingPower { left: 17_i64, right: 18_i64 })
    }
    Some(TokenKind::Dot) => {
        Some(BindingPower { left: 19_i64, right: 20_i64 })
    }
    _ => {
        None
    }
}
}

pub fn token_to_binop(kind: Rc<TokenKind>) -> BinOpKind {
    match kind.as_ref() {
    TokenKind::Plus => {
        BinOpKind::Add
    }
    TokenKind::Minus => {
        BinOpKind::Sub
    }
    TokenKind::Star => {
        BinOpKind::Mul
    }
    TokenKind::Slash => {
        BinOpKind::Div
    }
    TokenKind::Percent => {
        BinOpKind::Mod
    }
    TokenKind::EqEq => {
        BinOpKind::BinEq
    }
    TokenKind::Ne => {
        BinOpKind::BinNe
    }
    TokenKind::Lt => {
        BinOpKind::BinLt
    }
    TokenKind::Gt => {
        BinOpKind::BinGt
    }
    TokenKind::Le => {
        BinOpKind::BinLe
    }
    TokenKind::Ge => {
        BinOpKind::BinGe
    }
    TokenKind::And => {
        BinOpKind::BinAnd
    }
    TokenKind::Or => {
        BinOpKind::BinOr
    }
    TokenKind::NullCoalesce => {
        BinOpKind::NullCoalesce
    }
    _ => {
        BinOpKind::Add
    }
}
}

pub fn parse_pipe_rhs(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, receiver: Rc<Node>, span: SourceSpan) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_expr = parse_recovery_placeholder();
        let r = expect_name(tokens.clone(), state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let method = r.name.clone();
        let s = r.state.clone();
        let s = skip_newlines(tokens.clone(), s.clone());
        if peek_is_lparen(tokens.clone(), s.clone()) {
    let r2 = parse_call_args(tokens.clone(), s.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprMethodCall { receiver: receiver.clone(), method, args: r2.args.clone(), method_semantics: None }), None, span), state: r2.state.clone(), err: None })
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprMethodCall { receiver: receiver.clone(), method, args: Rc::new(Vec::new()), method_semantics: None }), None, span), state: s.clone(), err: None })
}
    })
}

pub fn parse_prefix(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let k = peek_kind(tokens.clone(), state.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Bang) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let span = current_span(tokens.clone(), state.clone());
    let r = parse_expr_bp(tokens.clone(), adv.state.clone(), 12_i64);
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprUnaryOp { op: UnaryOpKind::Not, operand: r.expr.clone() }), None, span), state: r.state.clone(), err: None })
}
    }
    Some(TokenKind::Minus) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let span = current_span(tokens.clone(), state.clone());
    let r = parse_expr_bp(tokens.clone(), adv.state.clone(), 12_i64);
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprUnaryOp { op: UnaryOpKind::Neg, operand: r.expr.clone() }), None, span), state: r.state.clone(), err: None })
}
    }
    _ => {
        parse_primary(tokens.clone(), state.clone())
    }
}
    })
}

pub fn parse_primary(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let k = peek_kind(tokens.clone(), state.clone());
        let span = current_span(tokens.clone(), state.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwTrue) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, span.clone()), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::KwFalse) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: false }) }), None, span.clone()), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::KwNone) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitNull) }), None, span.clone()), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::LitInt { value: n, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitInt { value: n.clone() }) }), None, span.clone()), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::LitFloat { value: f, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitFloat { value: f.clone() }) }), None, span.clone()), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::LitStr { value: s, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: s.clone() }) }), None, span.clone()), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::StrBegin { value: s, .. }) => {
        parse_string_interp(tokens.clone(), state.clone())
    }
    Some(TokenKind::Ident { name: n, .. }) => {
        parse_ident_expr(tokens.clone(), state.clone(), &n)
    }
    Some(TokenKind::LParen) => {
        parse_paren_expr(tokens.clone(), state.clone())
    }
    Some(TokenKind::LBracket) => {
        parse_list_literal(tokens.clone(), state.clone())
    }
    Some(TokenKind::LBrace) => {
        parse_brace_expr(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwMatch) => {
        parse_match(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwIf) => {
        parse_if(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwFor) => {
        parse_for(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwLet) => {
        parse_let(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwReturn) => {
        parse_return(tokens.clone(), state.clone())
    }
    Some(TokenKind::KwFn) => {
        parse_fn_lambda(tokens.clone(), state.clone())
    }
    _ => {
        {
    let kw_name = keyword_to_name(tokens.clone(), state.clone());
    match kw_name {
    Some(n) => {
        parse_ident_expr(tokens.clone(), state.clone(), &n)
    }
    None => {
        {
    let tag = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(kind) => {
        let kind = Rc::new(kind.clone());
        kind_tag(kind.clone())
    }
    None => {
        "EOF".to_string()
    }
};
    Rc::new(ExprResult { expr: parse_recovery_expr(span.clone(), &format!("expected expression, found {}", tag.clone())), state: state.clone(), err: Some(parse_error(&format!("expected expression, found {}", tag.clone()), span.clone())) })
}
    }
}
}
    }
}
    })
}

pub fn parse_lambda_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let s = skip_newlines(tokens.clone(), state.clone());
        let k = peek_kind(tokens.clone(), s.clone());
        let is_block = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwLet) => {
        true
    }
    Some(TokenKind::KwReturn) => {
        true
    }
    _ => {
        false
    }
};
        if is_block.clone() {
    let r = parse_lambda_stmts(tokens.clone(), s.clone(), Rc::new(Vec::new()));
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: parse_recovery_placeholder(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), s.clone());
    if ({
    let __len_0 = r.stmts.clone().len();
    __len_0 as i64
}) == 1_i64 {
    Rc::new(ExprResult { expr: r.stmts.clone().first().cloned().unwrap(), state: r.state.clone(), err: None })
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: r.stmts.clone() }), None, span), state: r.state.clone(), err: None })
}
} else {
    parse_expr(tokens.clone(), s.clone())
}
    })
}

pub fn parse_lambda_stmts(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Node>>>) -> Rc<StmtsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if ((peek_is_rparen(tokens.clone(), s.clone()) || peek_is_rbrace(tokens.clone(), s.clone())) || at_end(tokens.clone(), s.clone())) || peek_is_eof(tokens.clone(), s.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_stmt(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.expr.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_ident_expr(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, name: &str) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let adv = advance(tokens.clone(), state.clone());
        let s = adv.state.clone();
        if peek_is_fat_arrow(tokens.clone(), s.clone()) {
    let adv2 = advance(tokens.clone(), s.clone());
    let r = parse_lambda_body(tokens.clone(), adv2.state.clone());
    if has_err(r.err.clone()) {
    return r.clone();
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLambda { params: Rc::new(vec!(name.to_string())), body: r.expr.clone(), semantics: None }), None, span), state: r.state.clone(), err: None })
} else {
    if is_uppercase_start(&name) && peek_is_lbrace(tokens.clone(), s.clone()) {
    parse_record_literal(tokens.clone(), s.clone(), &name, span)
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprVar { name: name.to_string(), binding_kind: None }), None, span), state: s.clone(), err: None })
}
}
    })
}

pub fn is_uppercase_start(name: &str) -> bool {
    let ch = v2_rt::char_at(&name, 0_i64);
    (ch.clone() >= "A".to_string()) && (ch.clone() <= "Z".to_string())
}

pub fn try_postfix(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, lhs: Rc<Node>, min_bp: i64) -> Rc<PostfixResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let k = peek_kind(tokens.clone(), state.clone());
        let span = current_span(tokens.clone(), state.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LParen) => {
        if 14_i64 < min_bp {
    Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
} else {
    let r = parse_call_args(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: r.state.clone(), err: r.err.clone() });
};
    let call_expr = make_call_expr(lhs.clone(), r.args.clone(), span);
    Rc::new(PostfixResult { expr: call_expr.clone(), changed: true, state: r.state.clone(), err: None })
}
    }
    Some(TokenKind::Ident { ref name, .. }) if name == "as" => {
        if 13_i64 < min_bp {
    Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
} else {
    let adv = advance(tokens.clone(), state.clone());
    let r = parse_type_expr(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(PostfixResult { expr: make_expr_node(Rc::new(ExprData::ExprCast { expr: lhs.clone(), target: r.type_expr.clone() }), None, span), changed: true, state: r.state.clone(), err: None })
}
    }
    Some(TokenKind::LBracket) => {
        if 14_i64 < min_bp {
    Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
} else {
    let r = parse_index_or_slice(tokens.clone(), state.clone(), lhs.clone(), span);
    if has_err(r.err.clone()) {
    return Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(PostfixResult { expr: r.expr.clone(), changed: true, state: r.state.clone(), err: None })
}
    }
    Some(TokenKind::LBrace) => {
        match lhs.expr_data.as_ref() {
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        if is_uppercase_start(&n) && (14_i64 <= min_bp) {
    Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
} else {
    if is_uppercase_start(&n) {
    let r = parse_record_literal(tokens.clone(), state.clone(), &n, lhs.span.clone());
    if has_err(r.err.clone()) {
    return Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(PostfixResult { expr: r.expr.clone(), changed: true, state: r.state.clone(), err: None })
} else {
    Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
}
}
    }
    _ => {
        Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
    }
}
    }
    _ => {
        Rc::new(PostfixResult { expr: lhs.clone(), changed: false, state: state.clone(), err: None })
    }
}
    })
}

pub fn make_call_expr(lhs: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, span: SourceSpan) -> Rc<Node> {
    match lhs.expr_data.as_ref() {
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        make_expr_node(Rc::new(ExprData::ExprCall { func: n.clone(), args: args.clone(), call_semantics: None }), None, span)
    }
    ExprData::ExprFieldAccess { base: b, field: f, summary: _, .. } => {
        make_expr_node(Rc::new(ExprData::ExprMethodCall { receiver: b.clone(), method: f.clone(), args: args.clone(), method_semantics: None }), None, span)
    }
    _ => {
        make_expr_node(Rc::new(ExprData::ExprCall { func: "<expr>".to_string(), args: args.clone(), call_semantics: None }), None, span)
    }
}
}

pub fn parse_index_or_slice(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, base: Rc<Node>, span: SourceSpan) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "LBracket");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = r.state.clone();
        let r = parse_expr(tokens.clone(), s.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let first_expr = r.expr.clone();
        let s = r.state.clone();
        if peek_is_dot_dot(tokens.clone(), s.clone()) {
    let adv = advance(tokens.clone(), s.clone());
    let r = parse_expr(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let end_expr = r.expr.clone();
    let s = r.state.clone();
    let r = expect(tokens.clone(), s.clone(), "RBracket");
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprSlice { base: base.clone(), start: first_expr.clone(), end: end_expr.clone() }), None, span), state: r.state.clone(), err: None })
} else {
    let r = expect(tokens.clone(), s.clone(), "RBracket");
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprIndex { base: base.clone(), index: first_expr.clone() }), None, span), state: r.state.clone(), err: None })
}
    })
}

pub fn parse_call_args(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ArgsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let r = expect(tokens.clone(), state.clone(), "LParen");
        if has_err(r.err.clone()) {
    return Rc::new(ArgsResult { args: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        if peek_is_rparen(tokens.clone(), s.clone()) {
    let adv = advance(tokens.clone(), s.clone());
    Rc::new(ArgsResult { args: Rc::new(Vec::new()), state: adv.state.clone(), err: None })
} else {
    let r = parse_arg_list(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return r.clone();
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s.clone(), "RParen");
    if has_err(r2.err.clone()) {
    return Rc::new(ArgsResult { args: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ArgsResult { args: r.args.clone(), state: r2.state.clone(), err: None })
}
    })
}

pub fn parse_arg_list(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ArgsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_arg_list_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
    })
}

pub fn parse_arg_list_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<NamedArg>>>) -> Rc<ArgsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            let r = parse_single_arg(tokens.clone(), s.clone());
            if has_err(r.err.clone()) {
    break Rc::new(ArgsResult { args: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
            let acc = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.arg.clone());
    Rc::new(__appended_0)
};
            let s = r.state.clone();
            let e = eat(tokens.clone(), s.clone(), "Comma");
            if e.consumed.clone() {
    let s2 = skip_newlines(tokens.clone(), e.state.clone());
    if peek_is_rparen(tokens.clone(), s2.clone()) {
    break Rc::new(ArgsResult { args: acc.clone(), state: s2.clone(), err: None });
} else {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = acc.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
} else {
    break Rc::new(ArgsResult { args: acc.clone(), state: s.clone(), err: None });
};
        }
    })
}

pub fn parse_single_arg(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ArgResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_arg = Rc::new(NamedArg { name: None, value: parse_recovery_placeholder() });
        let is_name_token = is_ident(tokens.clone(), state.clone()) || is_keyword_name(tokens.clone(), state.clone());
        if is_name_token.clone() {
    let save_pos = state.pos.clone();
    let name_r = expect_name(tokens.clone(), state.clone());
    if has_err(name_r.err.clone()) {
    let r = parse_expr(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ArgResult { arg: dummy_arg.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let arg = Rc::new(NamedArg { name: None, value: r.expr.clone() });
    Rc::new(ArgResult { arg: arg.clone(), state: r.state.clone(), err: None })
} else {
    if peek_is_colon(tokens.clone(), name_r.state.clone()) {
    let adv = advance(tokens.clone(), name_r.state.clone());
    let r = parse_expr(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ArgResult { arg: dummy_arg.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let arg = Rc::new(NamedArg { name: Some(name_r.name.clone()), value: r.expr.clone() });
    Rc::new(ArgResult { arg: arg.clone(), state: r.state.clone(), err: None })
} else {
    let restored = Rc::new(ParserState { pos: save_pos, ..(*state).clone() });
    let r = parse_expr(tokens.clone(), restored.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ArgResult { arg: dummy_arg.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let arg = Rc::new(NamedArg { name: None, value: r.expr.clone() });
    Rc::new(ArgResult { arg: arg.clone(), state: r.state.clone(), err: None })
}
}
} else {
    let r = parse_expr(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ArgResult { arg: dummy_arg.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let arg = Rc::new(NamedArg { name: None, value: r.expr.clone() });
    Rc::new(ArgResult { arg: arg.clone(), state: r.state.clone(), err: None })
}
    })
}

pub fn parse_match(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "KwMatch");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_expr_no_brace(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return r.clone();
};
        let scrutinee = r.expr.clone();
        let r = expect(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "LBrace");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_match_arms(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let arms = r.arms.clone();
        let r = expect(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "RBrace");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprMatch { scrutinee: scrutinee.clone(), arms: arms.clone() }), None, span), state: r.state.clone(), err: None })
    })
}

pub fn parse_expr_no_brace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_expr_bp_no_brace(tokens.clone(), state.clone(), 0_i64)
    })
}

pub fn parse_expr_bp_no_brace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, min_bp: i64) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let r = parse_prefix(tokens.clone(), state.clone());
        if has_err(r.err.clone()) {
    return r.clone();
};
        let lhs = r.expr.clone();
        parse_expr_loop_no_brace(tokens.clone(), r.state.clone(), lhs.clone(), min_bp)
    })
}

pub fn parse_expr_loop_no_brace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, lhs: Rc<Node>, min_bp: i64) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_lhs = lhs;
        let mut __tco_p_min_bp = min_bp;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let lhs = __tco_p_lhs;
            let min_bp = __tco_p_min_bp;
            if (at_end(tokens.clone(), state.clone()) || peek_is_eof(tokens.clone(), state.clone())) || peek_is_lbrace(tokens.clone(), state.clone()) {
    break Rc::new(ExprResult { expr: lhs.clone(), state: state.clone(), err: None });
} else {
    let s = skip_continuation_newlines(tokens.clone(), state.clone());
    if peek_is_lparen(tokens.clone(), s.clone()) && (14_i64 >= min_bp.clone()) {
    let r = parse_call_args(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: lhs.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), s.clone());
    let new_lhs = make_call_expr(lhs.clone(), r.args.clone(), span);
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = new_lhs.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    if peek_is_lbracket(tokens.clone(), s.clone()) && (14_i64 >= min_bp.clone()) {
    let span = current_span(tokens.clone(), s.clone());
    let r = parse_index_or_slice(tokens.clone(), s.clone(), lhs.clone(), span);
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = r.expr.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    let bp = infix_bp(tokens.clone(), s.clone());
    match bp {
    Some(bps) => {
        if bps.left.clone() < min_bp.clone() {
    break Rc::new(ExprResult { expr: lhs.clone(), state: s.clone(), err: None });
} else {
    let adv = advance(tokens.clone(), s.clone());
    let op_kind = adv.token.kind.clone();
    if is_dot_kind(op_kind.clone()) {
    let r = expect_name(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: lhs.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), s.clone());
    let new_lhs = make_expr_node(Rc::new(ExprData::ExprFieldAccess { base: lhs.clone(), field: r.name.clone(), summary: None }), None, span);
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = new_lhs.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    if is_pipe_arrow_kind(op_kind.clone()) {
    let span = current_span(tokens.clone(), s.clone());
    let r = parse_pipe_rhs(tokens.clone(), adv.state.clone(), lhs.clone(), span);
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = r.expr.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

} else {
    let rhs_state = skip_newlines(tokens.clone(), adv.state.clone());
    let r = parse_expr_bp_no_brace(tokens.clone(), rhs_state.clone(), bps.right.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), s.clone());
    let binop = token_to_binop(op_kind.clone());
    let new_lhs = make_expr_node(Rc::new(ExprData::ExprBinOp { op: binop, left: lhs.clone(), right: r.expr.clone() }), None, span);
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = new_lhs.clone();
        let __tco_3 = min_bp.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_lhs = __tco_2;
        __tco_p_min_bp = __tco_3;
        continue;
    }

};
};
};
    }
    None => {
        break Rc::new(ExprResult { expr: lhs.clone(), state: s.clone(), err: None });
    }
};
};
};
};
        }
    })
}

pub fn parse_match_arms(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ArmsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_match_arms_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
    })
}

pub fn parse_match_arms_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<MatchArm>>>) -> Rc<ArmsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(ArmsResult { arms: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_match_arm(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ArmsResult { arms: acc.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let e = eat(tokens.clone(), r.state.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    r.state.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.arm.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_match_arm(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ArmResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_arm = Rc::new(MatchArm { pattern: Rc::new(MatchPattern::Wildcard), guard: None, body: parse_recovery_placeholder() });
        let r = parse_pattern(tokens.clone(), state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ArmResult { arm: dummy_arm.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let pat = r.pattern.clone();
        let s = r.state.clone();
        let guard_r = parse_optional_guard(tokens.clone(), s.clone());
        if has_err(guard_r.err.clone()) {
    return Rc::new(ArmResult { arm: dummy_arm.clone(), state: guard_r.state.clone(), err: guard_r.err.clone() });
};
        let guard = guard_r.guard.clone();
        let s = guard_r.state.clone();
        let r = expect(tokens.clone(), s.clone(), "FatArrow");
        if has_err(r.err.clone()) {
    return Rc::new(ArmResult { arm: dummy_arm.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let r = if peek_is_lbrace(tokens.clone(), s.clone()) {
    parse_block(tokens.clone(), s.clone())
} else {
    parse_match_arm_body(tokens.clone(), s.clone())
};
        if has_err(r.err.clone()) {
    return Rc::new(ArmResult { arm: dummy_arm.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let arm = Rc::new(MatchArm { pattern: pat.clone(), guard: guard.clone(), body: r.expr.clone() });
        Rc::new(ArmResult { arm: arm.clone(), state: r.state.clone(), err: None })
    })
}

pub fn parse_match_arm_body(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let k = peek_kind(tokens.clone(), state.clone());
        let is_block = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwLet) => {
        true
    }
    Some(TokenKind::KwReturn) => {
        true
    }
    _ => {
        false
    }
};
        if is_block.clone() {
    let r = parse_match_arm_stmts(tokens.clone(), state.clone(), Rc::new(Vec::new()));
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: parse_recovery_placeholder(), state: r.state.clone(), err: r.err.clone() });
};
    let span = current_span(tokens.clone(), state.clone());
    if ({
    let __len_0 = r.stmts.clone().len();
    __len_0 as i64
}) == 1_i64 {
    Rc::new(ExprResult { expr: r.stmts.clone().first().cloned().unwrap(), state: r.state.clone(), err: None })
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: r.stmts.clone() }), None, span), state: r.state.clone(), err: None })
}
} else {
    parse_expr(tokens.clone(), state.clone())
}
    })
}

pub fn parse_match_arm_stmts(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<Node>>>) -> Rc<StmtsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if (peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone())) || peek_is_eof(tokens.clone(), s.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: s.clone(), err: None });
} else {
    if looks_like_arm_start(tokens.clone(), s.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_stmt(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(StmtsResult { stmts: acc.clone(), state: r.state.clone(), err: r.err.clone() });
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = r.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.expr.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
};
        }
    })
}

pub fn looks_like_arm_start(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { name: n, .. }) => {
        if n.clone() == "_" {
    peek_is_fat_arrow_at(tokens.clone(), state.clone(), 1_i64)
} else {
    if is_uppercase_start(&n) {
    if peek_is_fat_arrow_at(tokens.clone(), state.clone(), 1_i64) {
    true
} else {
    if peek_is_tag_at(tokens.clone(), state.clone(), 1_i64, "LBrace") {
    scan_for_fat_arrow_after_braces(tokens.clone(), state.clone(), 2_i64)
} else {
    false
}
}
} else {
    false
}
}
    }
    _ => {
        false
    }
}
}

pub fn peek_is_fat_arrow_at(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, offset: i64) -> bool {
    if (state.pos.clone() + offset.clone()) < ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
}) {
    let tok = tokens.clone().get((state.pos.clone() + offset.clone()) as usize).cloned();
    match tok.clone() {
    Some(t) => {
        is_fat_arrow_kind(t.kind.clone())
    }
    None => {
        false
    }
}
} else {
    false
}
}

pub fn peek_is_tag_at(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, offset: i64, tag: &str) -> bool {
    if (state.pos.clone() + offset.clone()) < ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
}) {
    let tok = tokens.clone().get((state.pos.clone() + offset.clone()) as usize).cloned();
    match tok.clone() {
    Some(t) => {
        kind_matches_tag(t.kind.clone(), &tag)
    }
    None => {
        false
    }
}
} else {
    false
}
}

pub fn scan_for_fat_arrow_after_braces(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, start_offset: i64) -> bool {
    scan_braces_depth(tokens.clone(), state.clone(), state.pos.clone() + start_offset, 1_i64)
}

pub fn scan_braces_depth(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, idx: i64, depth: i64) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_idx = idx;
        let mut __tco_p_depth = depth;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let idx = __tco_p_idx;
            let depth = __tco_p_depth;
            if (idx.clone() >= ({
    let __len_1 = tokens.clone().len();
    __len_1 as i64
})) || (depth.clone() <= 0_i64) {
    if depth.clone() == 0_i64 {
    if idx.clone() < ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
}) {
    let tok = tokens.clone().get((idx.clone()) as usize).cloned();
    match tok.clone() {
    Some(t) => {
        break is_fat_arrow_kind(t.kind.clone());
    }
    None => {
        break false;
    }
};
} else {
    break false;
};
} else {
    break false;
};
} else {
    let tok = tokens.clone().get((idx.clone()) as usize).cloned();
    match tok.clone() {
    Some(t) => {
        if is_lbrace_kind(t.kind.clone()) {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = state.clone();
        let __tco_2 = idx.clone() + 1_i64;
        let __tco_3 = depth.clone() + 1_i64;
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_idx = __tco_2;
        __tco_p_depth = __tco_3;
        continue;
    }

} else {
    if is_rbrace_kind(t.kind.clone()) {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = state.clone();
        let __tco_2 = idx.clone() + 1_i64;
        let __tco_3 = depth.clone() - 1_i64;
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_idx = __tco_2;
        __tco_p_depth = __tco_3;
        continue;
    }

} else {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = state.clone();
        let __tco_2 = idx.clone() + 1_i64;
        let __tco_3 = depth.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_idx = __tco_2;
        __tco_p_depth = __tco_3;
        continue;
    }

};
};
    }
    None => {
        break false;
    }
};
};
        }
    })
}

pub fn parse_optional_guard(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<GuardResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if peek_is_kw_if(tokens.clone(), state.clone()) {
    let adv = advance(tokens.clone(), state.clone());
    let r = parse_expr(tokens.clone(), adv.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(GuardResult { guard: None, state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(GuardResult { guard: Some(r.expr.clone()), state: r.state.clone(), err: None })
} else {
    Rc::new(GuardResult { guard: None, state: state.clone(), err: None })
}
    })
}

pub fn parse_pattern(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<PatternResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let k = peek_kind(tokens.clone(), state.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { name: n, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    if n.clone() == "_" {
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::Wildcard), state: adv.state.clone(), err: None })
} else {
    if is_uppercase_start(&n) {
    parse_variant_pattern(tokens.clone(), adv.state.clone(), &n)
} else {
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::Bind { name: n.clone() }), state: adv.state.clone(), err: None })
}
}
}
    }
    Some(TokenKind::KwTrue) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::LitPattern { value: Rc::new(LiteralValue::LitBool { value: true }) }), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::KwFalse) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::LitPattern { value: Rc::new(LiteralValue::LitBool { value: false }) }), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::KwNone) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::LitPattern { value: Rc::new(LiteralValue::LitNull) }), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::LitInt { value: n, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::LitPattern { value: Rc::new(LiteralValue::LitInt { value: n.clone() }) }), state: adv.state.clone(), err: None })
}
    }
    Some(TokenKind::LitStr { value: s, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::LitPattern { value: Rc::new(LiteralValue::LitStr { value: s.clone() }) }), state: adv.state.clone(), err: None })
}
    }
    _ => {
        Rc::new(PatternResult { pattern: Rc::new(MatchPattern::Wildcard), state: state.clone(), err: None })
    }
}
    })
}

pub fn parse_variant_pattern(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, name: &str) -> Rc<PatternResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if peek_is_lbrace(tokens.clone(), state.clone()) {
    let adv = advance(tokens.clone(), state.clone());
    let r = parse_variant_bindings_brace(tokens.clone(), skip_newlines(tokens.clone(), adv.state.clone()));
    if has_err(r.err.clone()) {
    return Rc::new(PatternResult { pattern: Rc::new(MatchPattern::Wildcard), state: r.state.clone(), err: r.err.clone() });
};
    let r2 = expect(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(PatternResult { pattern: Rc::new(MatchPattern::Wildcard), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::VariantPattern { name: name.to_string(), parent_enum: None, field_bindings: r.field_bindings.clone() }), state: r2.state.clone(), err: None })
} else {
    Rc::new(PatternResult { pattern: Rc::new(MatchPattern::VariantPattern { name: name.to_string(), parent_enum: None, field_bindings: Rc::new(Vec::new()) }), state: state.clone(), err: None })
}
    })
}

pub fn parse_variant_bindings_brace(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<BindingsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_variant_bindings_brace_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
    })
}

pub fn parse_variant_bindings_brace_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<FieldBinding>>>) -> Rc<BindingsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(BindingsResult { field_bindings: acc.clone(), state: s.clone(), err: None });
} else {
    let r = expect_name(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(BindingsResult { field_bindings: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let field_name = r.name.clone();
    let s = r.state.clone();
    let e = eat(tokens.clone(), s.clone(), "Colon");
    if e.consumed.clone() {
    let r2 = parse_pattern(tokens.clone(), e.state.clone());
    if has_err(r2.err.clone()) {
    break Rc::new(BindingsResult { field_bindings: Rc::new(Vec::new()), state: r2.state.clone(), err: r2.err.clone() });
};
    let s2 = r2.state.clone();
    let e2 = eat(tokens.clone(), s2.clone(), "Comma");
    let s3 = skip_newlines(tokens.clone(), if e2.consumed.clone() {
    e2.state.clone()
} else {
    s2.clone()
});
    let fb = Rc::new(FieldBinding { field_name: field_name.clone(), binding: r2.pattern.clone() });
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s3.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(fb.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    let e2 = eat(tokens.clone(), s.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e2.consumed.clone() {
    e2.state.clone()
} else {
    s.clone()
});
    let fb = Rc::new(FieldBinding { field_name: field_name.clone(), binding: Rc::new(MatchPattern::Bind { name: field_name.clone() }) });
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = {
    let __rc_3 = acc;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(fb.clone());
    Rc::new(__appended_2)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
};
        }
    })
}

pub fn parse_if(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "KwIf");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_expr_no_brace(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let condition = r.expr.clone();
        let r = parse_block(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let then_branch = r.expr.clone();
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let e = eat(tokens.clone(), s.clone(), "KwElse");
        if e.consumed.clone() {
    let s = skip_newlines(tokens.clone(), e.state.clone());
    if peek_is_kw_if(tokens.clone(), s.clone()) {
    let r2 = parse_if(tokens.clone(), s.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: r2.expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprIf { condition: condition.clone(), then_branch: then_branch.clone(), else_branch: Some(r2.expr.clone()) }), None, span), state: r2.state.clone(), err: None })
} else {
    let r2 = parse_block(tokens.clone(), s.clone());
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: r2.expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprIf { condition: condition.clone(), then_branch: then_branch.clone(), else_branch: Some(r2.expr.clone()) }), None, span), state: r2.state.clone(), err: None })
}
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprIf { condition: condition.clone(), then_branch: then_branch.clone(), else_branch: None }), None, span), state: s.clone(), err: None })
}
    })
}

pub fn parse_let(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "KwLet");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = expect_ident(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let name = r.name.clone();
        let r = expect(tokens.clone(), r.state.clone(), "Eq");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_expr(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return r.clone();
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLet { name, value: r.expr.clone(), body: None }), None, span), state: r.state.clone(), err: None })
    })
}

pub fn parse_return(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "KwReturn");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_expr(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return r.clone();
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprReturn { value: r.expr.clone() }), None, span), state: r.state.clone(), err: None })
    })
}

pub fn parse_for(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "KwFor");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = expect_ident(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let var_name = r.name.clone();
        let r = expect(tokens.clone(), r.state.clone(), "KwIn");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_expr_no_brace(tokens.clone(), r.state.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let collection = r.expr.clone();
        let r = parse_block(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()));
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let body = r.expr.clone();
        let for_expr = make_expr_node(Rc::new(ExprData::ExprForEach { variable: var_name, collection: collection.clone(), body: body.clone() }), None, span);
        Rc::new(ExprResult { expr: for_expr.clone(), state: r.state.clone(), err: None })
    })
}

pub fn parse_record_literal(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, name: &str, span: SourceSpan) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "LBrace");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let r = parse_field_init_list(tokens.clone(), s.clone());
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        let r2 = expect(tokens.clone(), s.clone(), "RBrace");
        if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: Some(name.to_string()), fields: r.fields.clone(), parent_enum: None }), None, span), state: r2.state.clone(), err: None })
    })
}

pub fn parse_field_init_list(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<FieldInitsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_field_init_list_acc(tokens.clone(), state.clone(), Rc::new(Vec::new()))
    })
}

pub fn parse_field_init_list_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<Rc<FieldInit>>>) -> Rc<FieldInitsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rbrace(tokens.clone(), s.clone()) || at_end(tokens.clone(), s.clone()) {
    break Rc::new(FieldInitsResult { fields: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_field_init(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(FieldInitsResult { fields: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let e = eat(tokens.clone(), r.state.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    r.state.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.field.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

};
        }
    })
}

pub fn parse_field_init(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<FieldInitResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let dummy_fi = Rc::new(FieldInit { name: "".to_string(), value: parse_recovery_placeholder() });
        if is_ident(tokens.clone(), state.clone()) || is_keyword_name(tokens.clone(), state.clone()) {
    let name_r = expect_name(tokens.clone(), state.clone());
    if has_err(name_r.err.clone()) {
    return Rc::new(FieldInitResult { field: dummy_fi.clone(), state: name_r.state.clone(), err: name_r.err.clone() });
};
    let n = name_r.name.clone();
    if peek_is_colon(tokens.clone(), name_r.state.clone()) {
    let adv2 = advance(tokens.clone(), name_r.state.clone());
    let r = parse_expr(tokens.clone(), adv2.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(FieldInitResult { field: dummy_fi.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fi = Rc::new(FieldInit { name: n.clone(), value: r.expr.clone() });
    Rc::new(FieldInitResult { field: fi.clone(), state: r.state.clone(), err: None })
} else {
    let fi = Rc::new(FieldInit { name: n.clone(), value: make_expr_node(Rc::new(ExprData::ExprVar { name: n.clone(), binding_kind: None }), None, current_span(tokens.clone(), state.clone())) });
    Rc::new(FieldInitResult { field: fi.clone(), state: name_r.state.clone(), err: None })
}
} else {
    if peek_is_lit_str(tokens.clone(), state.clone()) && peek_is_colon_after_ident(tokens.clone(), state.clone()) {
    let k = peek_kind(tokens.clone(), state.clone());
    let str_name = match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::LitStr { value: sv, .. }) => {
        sv.clone()
    }
    _ => {
        "_".to_string()
    }
};
    let adv = advance(tokens.clone(), state.clone());
    let adv2 = advance(tokens.clone(), adv.state.clone());
    let r = parse_expr(tokens.clone(), adv2.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(FieldInitResult { field: dummy_fi.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fi = Rc::new(FieldInit { name: str_name.clone(), value: r.expr.clone() });
    Rc::new(FieldInitResult { field: fi.clone(), state: r.state.clone(), err: None })
} else {
    let r = parse_expr(tokens.clone(), state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(FieldInitResult { field: dummy_fi.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let fi = Rc::new(FieldInit { name: "_".to_string(), value: r.expr.clone() });
    Rc::new(FieldInitResult { field: fi.clone(), state: r.state.clone(), err: None })
}
}
    })
}

pub fn parse_list_literal(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "LBracket");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r = parse_expr_list_until(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "RBracket");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let r2 = expect(tokens.clone(), skip_newlines(tokens.clone(), r.state.clone()), "RBracket");
        if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprListLit { elements: r.exprs.clone() }), None, span), state: r2.state.clone(), err: None })
    })
}

pub fn parse_expr_list_until(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, end_tag: &str) -> Rc<ExprsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        parse_expr_list_until_acc(tokens.clone(), state.clone(), &end_tag, Rc::new(Vec::new()))
    })
}

pub fn parse_expr_list_until_acc(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, end_tag: &str, acc: Rc<Vec<Rc<Node>>>) -> Rc<ExprsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_end_tag = end_tag.to_string();
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let end_tag = __tco_p_end_tag;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            let at_end_tag = match peek_kind(tokens.clone(), s.clone()) {
    Some(kind) => {
        kind_matches_tag(kind.clone(), &end_tag)
    }
    None => {
        false
    }
};
            if at_end_tag.clone() || at_end(tokens.clone(), s.clone()) {
    break Rc::new(ExprsResult { exprs: acc.clone(), state: s.clone(), err: None });
} else {
    let r = parse_expr(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    break Rc::new(ExprsResult { exprs: Rc::new(Vec::new()), state: r.state.clone(), err: r.err.clone() });
};
    let e = eat(tokens.clone(), r.state.clone(), "Comma");
    let s2 = skip_newlines(tokens.clone(), if e.consumed.clone() {
    e.state.clone()
} else {
    r.state.clone()
});
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = s2.clone();
        let __tco_2 = end_tag;
        let __tco_3 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(r.expr.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_end_tag = __tco_2;
        __tco_p_acc = __tco_3;
        continue;
    }

};
        }
    })
}

pub fn parse_paren_expr(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let r = expect(tokens.clone(), state.clone(), "LParen");
        if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r.state.clone());
        if peek_is_rparen(tokens.clone(), s.clone()) {
    let adv = advance(tokens.clone(), s.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: None, fields: Rc::new(Vec::new()), parent_enum: None }), None, span), state: adv.state.clone(), err: None })
} else {
    let lambda_r = try_lambda_params(tokens.clone(), s.clone());
    if lambda_r.is_lambda.clone() {
    let r = parse_lambda_body(tokens.clone(), lambda_r.state.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLambda { params: lambda_r.params.clone(), body: r.expr.clone(), semantics: None }), None, span), state: r.state.clone(), err: None })
} else {
    let r = parse_expr(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: r.expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s.clone(), "RParen");
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ExprResult { expr: r.expr.clone(), state: r2.state.clone(), err: None })
}
}
    })
}

pub fn parse_fn_lambda(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy = parse_recovery_placeholder();
        let r1 = expect(tokens.clone(), state.clone(), "KwFn");
        if has_err(r1.err.clone()) {
    return Rc::new(ExprResult { expr: dummy.clone(), state: r1.state.clone(), err: r1.err.clone() });
};
        let r2 = expect(tokens.clone(), r1.state.clone(), "LParen");
        if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
        let params_r = collect_fn_lambda_params(tokens.clone(), r2.state.clone(), Rc::new(Vec::new()));
        let r3 = expect(tokens.clone(), params_r.state.clone(), "RParen");
        if has_err(r3.err.clone()) {
    return Rc::new(ExprResult { expr: dummy.clone(), state: r3.state.clone(), err: r3.err.clone() });
};
        let s = skip_newlines(tokens.clone(), r3.state.clone());
        let body_r = parse_brace_expr(tokens.clone(), s.clone());
        if has_err(body_r.err.clone()) {
    return body_r.clone();
};
        Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprLambda { params: params_r.params.clone(), body: body_r.expr.clone(), semantics: None }), None, span), state: body_r.state.clone(), err: None })
    })
}

pub fn collect_fn_lambda_params(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<String>>) -> Rc<IdentCollectResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            let s = skip_newlines(tokens.clone(), state);
            if peek_is_rparen(tokens.clone(), s.clone()) {
    break Rc::new(IdentCollectResult { success: true, params: acc.clone(), state: s.clone(), err: None });
} else {
    let name_r = expect_ident(tokens.clone(), s.clone());
    if has_err(name_r.err.clone()) {
    break Rc::new(IdentCollectResult { success: false, params: acc.clone(), state: name_r.state.clone(), err: name_r.err.clone() });
} else {
    let s2 = skip_newlines(tokens.clone(), name_r.state.clone());
    if peek_is_comma(tokens.clone(), s2.clone()) {
    let adv = advance(tokens.clone(), s2.clone());
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(name_r.name.clone());
    Rc::new(__appended_0)
};
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(IdentCollectResult { success: true, params: {
    let __rc_3 = acc;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(name_r.name.clone());
    Rc::new(__appended_2)
}, state: s2.clone(), err: None });
};
};
};
        }
    })
}

pub fn try_lambda_params(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<LambdaCheckResult> {
    let save_pos = state.pos.clone();
    let r = collect_lambda_idents(tokens.clone(), state.clone(), Rc::new(Vec::new()));
    if r.success.clone() && (({
    let __len_0 = r.params.clone().len();
    __len_0 as i64
}) >= 2_i64) {
    if peek_is_rparen(tokens.clone(), r.state.clone()) {
    let adv = advance(tokens.clone(), r.state.clone());
    if peek_is_fat_arrow(tokens.clone(), adv.state.clone()) {
    let adv2 = advance(tokens.clone(), adv.state.clone());
    Rc::new(LambdaCheckResult { is_lambda: true, params: r.params.clone(), state: adv2.state.clone(), err: None })
} else {
    let restored = Rc::new(ParserState { pos: save_pos, ..(*state).clone() });
    Rc::new(LambdaCheckResult { is_lambda: false, params: Rc::new(Vec::new()), state: restored.clone(), err: None })
}
} else {
    let restored = Rc::new(ParserState { pos: save_pos, ..(*state).clone() });
    Rc::new(LambdaCheckResult { is_lambda: false, params: Rc::new(Vec::new()), state: restored.clone(), err: None })
}
} else {
    let restored = Rc::new(ParserState { pos: save_pos, ..(*state).clone() });
    Rc::new(LambdaCheckResult { is_lambda: false, params: Rc::new(Vec::new()), state: restored.clone(), err: None })
}
}

pub fn collect_lambda_idents(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, acc: Rc<Vec<String>>) -> Rc<IdentCollectResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_acc = acc;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let acc = __tco_p_acc;
            if is_ident(tokens.clone(), state.clone()) {
    let k = peek_kind(tokens.clone(), state.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::Ident { name: n, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let new_acc = {
    let __rc_1 = acc;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(n.clone());
    Rc::new(__appended_0)
};
    let e = eat(tokens.clone(), adv.state.clone(), "Comma");
    if e.consumed.clone() {
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = e.state.clone();
        let __tco_2 = new_acc.clone();
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_acc = __tco_2;
        continue;
    }

} else {
    break Rc::new(IdentCollectResult { success: true, params: new_acc.clone(), state: adv.state.clone(), err: None });
};
};
    }
    _ => {
        break Rc::new(IdentCollectResult { success: false, params: acc.clone(), state: state.clone(), err: None });
    }
};
} else {
    break Rc::new(IdentCollectResult { success: false, params: acc.clone(), state: state.clone(), err: None });
};
        }
    })
}

pub fn parse_string_interp(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let k = peek_kind(tokens.clone(), state.clone());
        match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::StrBegin { value: prefix, .. }) => {
        {
    let adv = advance(tokens.clone(), state.clone());
    let parts_init = if v2_rt::string_length(&prefix) > 0_i64 {
    Rc::new(vec!(Rc::new(StringPart::Text { value: prefix.clone() })))
} else {
    Rc::new(Vec::new())
};
    parse_interp_parts(tokens.clone(), adv.state.clone(), parts_init.clone(), span.clone())
}
    }
    _ => {
        Rc::new(ExprResult { expr: parse_recovery_expr(span.clone(), "expected string interpolation"), state: state.clone(), err: Some(parse_error("expected string interpolation", span.clone())) })
    }
}
    })
}

pub fn parse_interp_parts(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>, parts: Rc<Vec<Rc<StringPart>>>, span: SourceSpan) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_tokens = tokens;
        let mut __tco_p_state = state;
        let mut __tco_p_parts = parts;
        let mut __tco_p_span = span;
        loop {
            let tokens = __tco_p_tokens;
            let state = __tco_p_state;
            let parts = __tco_p_parts;
            let span = __tco_p_span;
            let r = parse_expr(tokens.clone(), state);
            if has_err(r.err.clone()) {
    break r.clone();
};
            let new_parts = {
    let __rc_1 = parts;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(Rc::new(StringPart::Interpolation { expr: r.expr.clone() }));
    Rc::new(__appended_0)
};
            let s = r.state.clone();
            let k = peek_kind(tokens.clone(), s.clone());
            match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::StrMid { value: mid, .. }) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let mid_parts = if v2_rt::string_length(&mid) > 0_i64 {
    {
    let __rc_3 = new_parts;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(Rc::new(StringPart::Text { value: mid.clone() }));
    Rc::new(__appended_2)
}
} else {
    new_parts.clone()
};
     {
        let __tco_0 = tokens.clone();
        let __tco_1 = adv.state.clone();
        let __tco_2 = mid_parts.clone();
        let __tco_3 = span;
        __tco_p_tokens = __tco_0;
        __tco_p_state = __tco_1;
        __tco_p_parts = __tco_2;
        __tco_p_span = __tco_3;
        continue;
    }

};
    }
    Some(TokenKind::StrEnd { value: suffix, .. }) => {
        {
    let adv = advance(tokens.clone(), s.clone());
    let final_parts = if v2_rt::string_length(&suffix) > 0_i64 {
    {
    let __rc_5 = new_parts;
    let mut __appended_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __appended_4.push(Rc::new(StringPart::Text { value: suffix.clone() }));
    Rc::new(__appended_4)
}
} else {
    new_parts.clone()
};
    break Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprStringInterp { parts: final_parts.clone() }), None, span), state: adv.state.clone(), err: None });
};
    }
    _ => {
        break Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprStringInterp { parts: new_parts.clone() }), None, span), state: s.clone(), err: None });
    }
};
        }
    })
}

pub fn parse_brace_expr(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> Rc<ExprResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let span = current_span(tokens.clone(), state.clone());
        let dummy_expr = parse_recovery_placeholder();
        let adv = advance(tokens.clone(), state.clone());
        let s = skip_newlines(tokens.clone(), adv.state.clone());
        if peek_is_rbrace(tokens.clone(), s.clone()) {
    let adv2 = advance(tokens.clone(), s.clone());
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: None, fields: Rc::new(Vec::new()), parent_enum: None }), None, span), state: adv2.state.clone(), err: None })
} else {
    let k = peek_kind(tokens.clone(), s.clone());
    match k.as_ref().map(|__rc| __rc.as_ref()) {
    Some(TokenKind::KwLet) => {
        {
    let r = parse_stmts(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    if ({
    let __len_0 = r.stmts.clone().len();
    __len_0 as i64
}) == 1_i64 {
    Rc::new(ExprResult { expr: r.stmts.clone().first().cloned().unwrap(), state: r2.state.clone(), err: None })
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: r.stmts.clone() }), None, span), state: r2.state.clone(), err: None })
}
}
    }
    Some(TokenKind::KwReturn) => {
        {
    let r = parse_stmts(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    if ({
    let __len_1 = r.stmts.clone().len();
    __len_1 as i64
}) == 1_i64 {
    Rc::new(ExprResult { expr: r.stmts.clone().first().cloned().unwrap(), state: r2.state.clone(), err: None })
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: r.stmts.clone() }), None, span), state: r2.state.clone(), err: None })
}
}
    }
    _ => {
        {
    let ident_or_keyword = is_ident(tokens.clone(), s.clone()) || is_keyword_name(tokens.clone(), s.clone());
    if ident_or_keyword.clone() && peek_is_colon_after_ident(tokens.clone(), s.clone()) {
    let r = parse_field_init_list(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: None, fields: r.fields.clone(), parent_enum: None }), None, span), state: r2.state.clone(), err: None })
} else {
    if peek_is_lit_str(tokens.clone(), s.clone()) && peek_is_colon_after_ident(tokens.clone(), s.clone()) {
    let r = parse_field_init_list(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: None, fields: r.fields.clone(), parent_enum: None }), None, span), state: r2.state.clone(), err: None })
} else {
    let r = parse_stmts(tokens.clone(), s.clone());
    if has_err(r.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r.state.clone(), err: r.err.clone() });
};
    let s2 = skip_newlines(tokens.clone(), r.state.clone());
    let r2 = expect(tokens.clone(), s2.clone(), "RBrace");
    if has_err(r2.err.clone()) {
    return Rc::new(ExprResult { expr: dummy_expr.clone(), state: r2.state.clone(), err: r2.err.clone() });
};
    if ({
    let __len_2 = r.stmts.clone().len();
    __len_2 as i64
}) == 1_i64 {
    Rc::new(ExprResult { expr: r.stmts.clone().first().cloned().unwrap(), state: r2.state.clone(), err: None })
} else {
    Rc::new(ExprResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: r.stmts.clone() }), None, span), state: r2.state.clone(), err: None })
}
}
}
}
    }
}
}
    })
}

pub fn peek_is_colon_after_ident(tokens: Rc<Vec<Rc<Token>>>, state: Rc<ParserState>) -> bool {
    if (state.pos.clone() + 1_i64) < ({
    let __len_0 = tokens.clone().len();
    __len_0 as i64
}) {
    let next_tok = tokens.clone().get((state.pos.clone() + 1_i64) as usize).cloned();
    match next_tok.clone() {
    Some(t) => {
        is_colon_kind(t.kind.clone())
    }
    None => {
        false
    }
}
} else {
    false
}
}

