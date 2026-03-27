use crate::v2_core::*;
use crate::tokenize::*;
use crate::parse::*;
use crate::resolve::*;
use crate::normalize::*;
use crate::infer_items::*;
use crate::infer::*;
use crate::emit::*;
use crate::emit_rust::*;
use crate::emit_python::*;
use crate::emit_go::*;
use crate::complexity::*;
use crate::ownership::*;
use crate::artifact::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PipelineResult {
    pub files: Rc<Vec<Rc<TextFile>>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
    pub complexity: Rc<ComplexityReport>,
    pub ownership: Rc<Vec<Rc<OwnershipProof>>>,
    pub artifact_plan: Rc<ArtifactPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrontendResult {
    pub graph: Option<Rc<ModuleGraph>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn extract_func_entries(typed: Rc<ResolvedGraph>) -> Rc<Vec<Rc<FuncEntry>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in __elem_1.items.iter().cloned() {
        if __elem_3.body.clone().is_some() {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_4.push(Rc::new(FuncEntry { name: __elem_5.name.clone(), body: __elem_5.body.clone().unwrap(), params: __elem_5.params.clone() }));
    }
    Rc::new(__mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn extract_ownership_proofs(typed: Rc<ResolvedGraph>) -> Rc<Vec<Rc<OwnershipProof>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in __elem_1.items.iter().cloned() {
        if __elem_3.body.clone().is_some() {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_4.push(analyze_ownership(&__elem_5.name, __elem_5.params.clone(), __elem_5.body.clone().unwrap()));
    }
    Rc::new(__mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn ownership_diagnostics(proofs: Rc<Vec<Rc<OwnershipProof>>>) -> Rc<Vec<Rc<Node>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in proofs.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in __elem_1.decisions.iter().cloned() {
        __flat_mapped_2.extend((match __elem_3.as_ref() {
    OwnershipDecision::SharedError { binding, consumer_count: count, sites, .. } => {
        Rc::new(vec!(diagnostic_node("warning", &v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("ownership: binding '".to_string(), binding.clone()), "' in ".to_string()), __elem_1.func_name.clone()), " has ".to_string()), v2_rt::to_string(count.clone())), " consumers -- cannot guarantee O(1) mutation (".to_string()), {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in sites.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
}), ")".to_string()), no_span(), Some(__elem_1.func_name.clone()), None)))
    }
    _ => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn empty_artifact_plan() -> Rc<ArtifactPlan> {
    Rc::new(ArtifactPlan { artifacts: Rc::new(Vec::new()), boundaries: Rc::new(Vec::new()) })
}

pub fn compile_bundle_error(message: &str) -> Rc<Node> {
    diagnostic_node("error", &message, no_span(), None, None)
}

pub fn emit_artifact(typed: Rc<ResolvedGraph>, artifact: Rc<Artifact>) -> Rc<EmitResult> {
    match artifact.target.clone() {
    RenderTarget::Rust => {
        emit_rust(typed.clone())
    }
    RenderTarget::Python => {
        emit_python(typed.clone())
    }
    RenderTarget::Go => {
        emit_go(typed.clone())
    }
    RenderTarget::Dag => {
        emit_dag_artifact(typed.clone())
    }
}
}

pub fn json_quote(s: &str) -> String {
    v2_rt::concat(v2_rt::concat("\"".to_string(), escape_json_string(&s)), "\"".to_string())
}

pub fn json_list(items: Rc<Vec<String>>) -> String {
    v2_rt::concat(v2_rt::concat("[".to_string(), {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in items.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&", ".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}), "]".to_string())
}

pub fn json_optional_string(value: Option<String>) -> String {
    match value {
    Some(inner) => {
        json_quote(&inner)
    }
    None => {
        "null".to_string()
    }
}
}

pub fn json_optional_node(value: Option<Rc<Node>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(inner) => {
        let inner = Rc::new(inner.clone());
        serialize_node(inner.clone())
    }
    None => {
        "null".to_string()
    }
}
    })
}

pub fn json_optional_inferred_node(value: Option<Rc<InferredNode>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(inner) => {
        let inner = Rc::new(inner.clone());
        serialize_inferred_node(inner.clone())
    }
    None => {
        "null".to_string()
    }
}
    })
}

pub fn json_optional_span(value: Option<SourceSpan>) -> String {
    match value {
    Some(inner) => {
        serialize_span(inner)
    }
    None => {
        "null".to_string()
    }
}
}

pub fn json_bool(value: bool) -> String {
    if value {
    "true".to_string()
} else {
    "false".to_string()
}
}

pub fn connective_name(value: Connective) -> String {
    match value {
    Connective::Conj => {
        "Conj".to_string()
    }
    Connective::Disj => {
        "Disj".to_string()
    }
}
}

pub fn cardinality_name(value: Cardinality) -> String {
    match value {
    Cardinality::Required => {
        "Required".to_string()
    }
    Cardinality::CardOptional => {
        "CardOptional".to_string()
    }
}
}

pub fn field_access_style_name(value: FieldAccessStyle) -> String {
    match value {
    FieldAccessStyle::StoredField => {
        "StoredField".to_string()
    }
    FieldAccessStyle::EnumAccessor => {
        "EnumAccessor".to_string()
    }
    FieldAccessStyle::OptionalUnwrap => {
        "OptionalUnwrap".to_string()
    }
    FieldAccessStyle::TupleFirst => {
        "TupleFirst".to_string()
    }
    FieldAccessStyle::TupleSecond => {
        "TupleSecond".to_string()
    }
}
}

pub fn field_value_shape_name(value: FieldValueShape) -> String {
    match value {
    FieldValueShape::PlainValue => {
        "PlainValue".to_string()
    }
    FieldValueShape::OptionalValue => {
        "OptionalValue".to_string()
    }
}
}

pub fn var_binding_kind_name(value: Rc<VarBindingKind>) -> String {
    match value.as_ref() {
    VarBindingKind::LocalValueBinding => {
        "LocalValueBinding".to_string()
    }
    VarBindingKind::FunctionValueBinding => {
        "FunctionValueBinding".to_string()
    }
    VarBindingKind::VariantValueBinding { parent_enum: _, .. } => {
        "VariantValueBinding".to_string()
    }
}
}

pub fn call_semantics_name(value: CallSemantics) -> String {
    match value {
    CallSemantics::PlainCallSemantics => {
        "PlainCallSemantics".to_string()
    }
    CallSemantics::LookupCallSemantics => {
        "LookupCallSemantics".to_string()
    }
}
}

pub fn expr_error_kind_name(value: ExprErrorKind) -> String {
    match value {
    ExprErrorKind::ParseRecoveryError => {
        "ParseRecoveryError".to_string()
    }
    ExprErrorKind::SemanticExprError => {
        "SemanticExprError".to_string()
    }
    ExprErrorKind::InternalExprError => {
        "InternalExprError".to_string()
    }
}
}

pub fn bin_op_name(value: BinOpKind) -> String {
    match value {
    BinOpKind::Add => {
        "Add".to_string()
    }
    BinOpKind::Sub => {
        "Sub".to_string()
    }
    BinOpKind::Mul => {
        "Mul".to_string()
    }
    BinOpKind::Div => {
        "Div".to_string()
    }
    BinOpKind::Mod => {
        "Mod".to_string()
    }
    BinOpKind::BinEq => {
        "BinEq".to_string()
    }
    BinOpKind::BinNe => {
        "BinNe".to_string()
    }
    BinOpKind::BinLt => {
        "BinLt".to_string()
    }
    BinOpKind::BinGt => {
        "BinGt".to_string()
    }
    BinOpKind::BinLe => {
        "BinLe".to_string()
    }
    BinOpKind::BinGe => {
        "BinGe".to_string()
    }
    BinOpKind::BinAnd => {
        "BinAnd".to_string()
    }
    BinOpKind::BinOr => {
        "BinOr".to_string()
    }
    BinOpKind::NullCoalesce => {
        "NullCoalesce".to_string()
    }
}
}

pub fn unary_op_name(value: UnaryOpKind) -> String {
    match value {
    UnaryOpKind::Not => {
        "Not".to_string()
    }
    UnaryOpKind::Neg => {
        "Neg".to_string()
    }
}
}

pub fn serialize_span(span: SourceSpan) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"start\": ".to_string(), v2_rt::to_string(span.start.clone())), ", \"end\": ".to_string()), v2_rt::to_string(span.end.clone())), "}".to_string())
}

pub fn serialize_import_node(imp: Rc<Node>) -> String {
    let names_json = if import_is_all(imp.clone()) {
    "{\"kind\": \"ImportAll\"}".to_string()
} else {
    v2_rt::concat(v2_rt::concat("{\"kind\": \"ImportSpecific\", \"names\": ".to_string(), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in import_specific_names(imp.clone()).iter().cloned() {
        __mapped_2.push(json_quote(&__elem_3));
    }
    Rc::new(__mapped_2)
})), "}".to_string())
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"module_path\": ".to_string(), json_quote(&imp.name)), ", \"names\": ".to_string()), names_json.clone()), ", \"span\": ".to_string()), serialize_span(imp.span.clone())), "}".to_string())
}

pub fn serialize_field_summary(summary: Rc<FieldSummary>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"access_style\": ".to_string(), json_quote(&field_access_style_name(summary.access_style.clone()))), ", \"value_shape\": ".to_string()), json_quote(&field_value_shape_name(summary.value_shape.clone()))), "}".to_string())
}

pub fn serialize_literal(value: Rc<LiteralValue>) -> String {
    match value.as_ref() {
    LiteralValue::LitStr { value: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"LitStr\", \"value\": ".to_string(), json_quote(&inner)), "}".to_string())
    }
    LiteralValue::LitInt { value: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"LitInt\", \"value\": ".to_string(), v2_rt::to_string(inner.clone())), "}".to_string())
    }
    LiteralValue::LitFloat { value: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"LitFloat\", \"value\": ".to_string(), json_quote(&inner)), "}".to_string())
    }
    LiteralValue::LitBool { value: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"LitBool\", \"value\": ".to_string(), json_bool(inner.clone())), "}".to_string())
    }
    LiteralValue::LitNull => {
        "{\"kind\": \"LitNull\"}".to_string()
    }
}
}

pub fn serialize_field_binding(binding: Rc<FieldBinding>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"field_name\": ".to_string(), json_quote(&binding.field_name)), ", \"binding\": ".to_string()), serialize_match_pattern(binding.binding.clone())), "}".to_string())
    })
}

pub fn serialize_match_pattern(pattern: Rc<MatchPattern>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::Bind { name: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"Bind\", \"name\": ".to_string(), json_quote(&inner)), "}".to_string())
    }
    MatchPattern::LitPattern { value: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"LitPattern\", \"value\": ".to_string(), serialize_literal(inner.clone())), "}".to_string())
    }
    MatchPattern::VariantPattern { name: inner, parent_enum, field_bindings, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"VariantPattern\", \"name\": ".to_string(), json_quote(&inner)), ", \"parent_enum\": ".to_string()), json_optional_string(parent_enum.clone())), ", \"field_bindings\": ".to_string()), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in field_bindings.iter().cloned() {
        __mapped_2.push(serialize_field_binding(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), "}".to_string())
    }
    MatchPattern::Wildcard => {
        "{\"kind\": \"Wildcard\"}".to_string()
    }
}
    })
}

pub fn serialize_named_arg(arg: Rc<NamedArg>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_optional_string(arg.name.clone())), ", \"value\": ".to_string()), serialize_node(arg.value.clone())), "}".to_string())
}

pub fn serialize_match_arm(arm: Rc<MatchArm>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"pattern\": ".to_string(), serialize_match_pattern(arm.pattern.clone())), ", \"guard\": ".to_string()), json_optional_node(arm.guard.clone())), ", \"body\": ".to_string()), serialize_node(arm.body.clone())), "}".to_string())
}

pub fn serialize_field_init(field_init: Rc<FieldInit>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_quote(&field_init.name)), ", \"value\": ".to_string()), serialize_node(field_init.value.clone())), "}".to_string())
    })
}

pub fn serialize_string_part(part: Rc<StringPart>) -> String {
    match part.as_ref() {
    StringPart::Text { value: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"Text\", \"value\": ".to_string(), json_quote(&inner)), "}".to_string())
    }
    StringPart::Interpolation { expr: inner, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"Interpolation\", \"expr\": ".to_string(), serialize_node(inner.clone())), "}".to_string())
    }
}
}

pub fn serialize_call_semantics(value: Option<CallSemantics>) -> String {
    match value {
    Some(inner) => {
        v2_rt::concat(v2_rt::concat("{\"kind\": ".to_string(), json_quote(&call_semantics_name(inner))), "}".to_string())
    }
    None => {
        "null".to_string()
    }
}
}

pub fn serialize_lambda_semantics(value: Option<Rc<LambdaSemantics>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(inner) => {
        let inner = Rc::new(inner.clone());
        v2_rt::concat(v2_rt::concat("{\"param_types\": ".to_string(), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in inner.param_types.iter().cloned() {
        __mapped_2.push(serialize_node(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), "}".to_string())
    }
    None => {
        "null".to_string()
    }
}
    })
}

pub fn serialize_method_semantics(value: Option<Rc<MethodSemantics>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(MethodSemantics::PlainMethodSemantics) => {
        "{\"kind\": \"PlainMethodSemantics\"}".to_string()
    }
    Some(MethodSemantics::IntrinsicMethodSemantics { intrinsic: _, fold_accumulator_type, .. }) => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"IntrinsicMethodSemantics\", \"fold_accumulator_type\": ".to_string(), json_optional_node(fold_accumulator_type.clone())), "}".to_string())
    }
    Some(MethodSemantics::RuntimeBridgeSemantics { method: _, .. }) => {
        "{\"kind\": \"RuntimeBridgeSemantics\"}".to_string()
    }
    Some(MethodSemantics::ServiceMethodSemantics { service_name, op_params, .. }) => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ServiceMethodSemantics\", \"service_name\": ".to_string(), json_quote(&service_name)), ", \"op_params\": ".to_string()), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in op_params.iter().cloned() {
        __mapped_2.push(serialize_param(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), "}".to_string())
    }
    None => {
        "null".to_string()
    }
}
    })
}

pub fn serialize_expr_data(expr_node: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let ch = expr_node.children.clone();
        match expr_node.expr_data.as_ref() {
    ExprData::NoExprData => {
        "{\"kind\": \"NoExprData\"}".to_string()
    }
    ExprData::ExprLiteral { value, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprLiteral\", \"value\": ".to_string(), serialize_literal(value.clone())), "}".to_string())
    }
    ExprData::ExprError { kind, message, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprError\", \"error_kind\": ".to_string(), json_quote(&expr_error_kind_name(kind.clone()))), ", \"message\": ".to_string()), json_quote(&message)), "}".to_string())
    }
    ExprData::ExprVar { name, binding_kind, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprVar\", \"name\": ".to_string(), json_quote(&name)), ", \"binding_kind\": ".to_string()), match binding_kind.as_ref().map(|__rc| __rc.as_ref()) {
    Some(inner) => {
        let inner = Rc::new(inner.clone());
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": ".to_string(), json_quote(&var_binding_kind_name(inner.clone()))), match inner.as_ref() {
    VarBindingKind::VariantValueBinding { parent_enum, .. } => {
        v2_rt::concat(", \"parent_enum\": ".to_string(), json_quote(&parent_enum))
    }
    _ => {
        "".to_string()
    }
}), "}".to_string())
    }
    None => {
        "null".to_string()
    }
}), "}".to_string())
    }
    ExprData::ExprFieldAccess { field, summary, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprFieldAccess\", \"field\": ".to_string(), json_quote(&field)), ", \"summary\": ".to_string()), match summary.as_ref().map(|__rc| __rc.as_ref()) {
    Some(inner) => {
        let inner = Rc::new(inner.clone());
        serialize_field_summary(inner.clone())
    }
    None => {
        "null".to_string()
    }
}), ", \"children\": ".to_string()), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in ch.iter().cloned() {
        __mapped_2.push(serialize_node(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), "}".to_string())
    }
    ExprData::ExprCall { func, call_semantics, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprCall\", \"func\": ".to_string(), json_quote(&func)), ", \"call_semantics\": ".to_string()), serialize_call_semantics(call_semantics.clone())), ", \"children\": ".to_string()), json_list({
    let mut __mapped_6 = Vec::new();
    for __elem_7 in ch.iter().cloned() {
        __mapped_6.push(serialize_node(__elem_7.clone()));
    }
    Rc::new(__mapped_6)
})), "}".to_string())
    }
    ExprData::ExprMethodCall { method, method_semantics, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprMethodCall\", \"method\": ".to_string(), json_quote(&method)), ", \"method_semantics\": ".to_string()), serialize_method_semantics(method_semantics.clone())), ", \"children\": ".to_string()), json_list({
    let mut __mapped_10 = Vec::new();
    for __elem_11 in ch.iter().cloned() {
        __mapped_10.push(serialize_node(__elem_11.clone()));
    }
    Rc::new(__mapped_10)
})), "}".to_string())
    }
    ExprData::ExprBinOp { op, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprBinOp\", \"op\": ".to_string(), json_quote(&bin_op_name(op.clone()))), ", \"children\": ".to_string()), json_list({
    let mut __mapped_14 = Vec::new();
    for __elem_15 in ch.iter().cloned() {
        __mapped_14.push(serialize_node(__elem_15.clone()));
    }
    Rc::new(__mapped_14)
})), "}".to_string())
    }
    ExprData::ExprUnaryOp { op, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprUnaryOp\", \"op\": ".to_string(), json_quote(&unary_op_name(op.clone()))), ", \"children\": ".to_string()), json_list({
    let mut __mapped_18 = Vec::new();
    for __elem_19 in ch.iter().cloned() {
        __mapped_18.push(serialize_node(__elem_19.clone()));
    }
    Rc::new(__mapped_18)
})), "}".to_string())
    }
    ExprData::ExprLambda { params, semantics, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprLambda\", \"params\": ".to_string(), json_list({
    let mut __mapped_22 = Vec::new();
    for __elem_23 in params.iter().cloned() {
        __mapped_22.push(json_quote(&__elem_23));
    }
    Rc::new(__mapped_22)
})), ", \"semantics\": ".to_string()), serialize_lambda_semantics(semantics.clone())), ", \"children\": ".to_string()), json_list({
    let mut __mapped_26 = Vec::new();
    for __elem_27 in ch.iter().cloned() {
        __mapped_26.push(serialize_node(__elem_27.clone()));
    }
    Rc::new(__mapped_26)
})), "}".to_string())
    }
    ExprData::ExprLet { name, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprLet\", \"name\": ".to_string(), json_quote(&name)), ", \"children\": ".to_string()), json_list({
    let mut __mapped_30 = Vec::new();
    for __elem_31 in ch.iter().cloned() {
        __mapped_30.push(serialize_node(__elem_31.clone()));
    }
    Rc::new(__mapped_30)
})), "}".to_string())
    }
    ExprData::ExprRecordLit { type_name, parent_enum, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprRecordLit\", \"type_name\": ".to_string(), json_optional_string(type_name.clone())), ", \"parent_enum\": ".to_string()), json_optional_string(parent_enum.clone())), ", \"children\": ".to_string()), json_list({
    let mut __mapped_34 = Vec::new();
    for __elem_35 in ch.iter().cloned() {
        __mapped_34.push(serialize_node(__elem_35.clone()));
    }
    Rc::new(__mapped_34)
})), "}".to_string())
    }
    ExprData::ExprForEach { variable, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprForEach\", \"variable\": ".to_string(), json_quote(&variable)), ", \"children\": ".to_string()), json_list({
    let mut __mapped_38 = Vec::new();
    for __elem_39 in ch.iter().cloned() {
        __mapped_38.push(serialize_node(__elem_39.clone()));
    }
    Rc::new(__mapped_38)
})), "}".to_string())
    }
    ExprData::ExprMatch => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprMatch\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_42 = Vec::new();
    for __elem_43 in ch.iter().cloned() {
        __mapped_42.push(serialize_node(__elem_43.clone()));
    }
    Rc::new(__mapped_42)
})), "}".to_string())
    }
    ExprData::ExprIf => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprIf\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_46 = Vec::new();
    for __elem_47 in ch.iter().cloned() {
        __mapped_46.push(serialize_node(__elem_47.clone()));
    }
    Rc::new(__mapped_46)
})), "}".to_string())
    }
    ExprData::ExprListLit => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprListLit\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_50 = Vec::new();
    for __elem_51 in ch.iter().cloned() {
        __mapped_50.push(serialize_node(__elem_51.clone()));
    }
    Rc::new(__mapped_50)
})), "}".to_string())
    }
    ExprData::ExprStringInterp => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprStringInterp\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_54 = Vec::new();
    for __elem_55 in ch.iter().cloned() {
        __mapped_54.push(serialize_node(__elem_55.clone()));
    }
    Rc::new(__mapped_54)
})), "}".to_string())
    }
    ExprData::ExprBlock => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprBlock\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_58 = Vec::new();
    for __elem_59 in ch.iter().cloned() {
        __mapped_58.push(serialize_node(__elem_59.clone()));
    }
    Rc::new(__mapped_58)
})), "}".to_string())
    }
    ExprData::ExprCast => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprCast\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_62 = Vec::new();
    for __elem_63 in ch.iter().cloned() {
        __mapped_62.push(serialize_node(__elem_63.clone()));
    }
    Rc::new(__mapped_62)
})), "}".to_string())
    }
    ExprData::ExprIndex => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprIndex\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_66 = Vec::new();
    for __elem_67 in ch.iter().cloned() {
        __mapped_66.push(serialize_node(__elem_67.clone()));
    }
    Rc::new(__mapped_66)
})), "}".to_string())
    }
    ExprData::ExprSlice => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprSlice\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_70 = Vec::new();
    for __elem_71 in ch.iter().cloned() {
        __mapped_70.push(serialize_node(__elem_71.clone()));
    }
    Rc::new(__mapped_70)
})), "}".to_string())
    }
    ExprData::ExprReturn => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"ExprReturn\"".to_string(), ", \"children\": ".to_string()), json_list({
    let mut __mapped_74 = Vec::new();
    for __elem_75 in ch.iter().cloned() {
        __mapped_74.push(serialize_node(__elem_75.clone()));
    }
    Rc::new(__mapped_74)
})), "}".to_string())
    }
}
    })
}

pub fn serialize_inferred_node(inferred: Rc<InferredNode>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match inferred.as_ref() {
    InferredNode::Resolved { node, .. } => {
        v2_rt::concat(v2_rt::concat("{\"kind\": \"Resolved\", \"node\": ".to_string(), serialize_node(node.clone())), "}".to_string())
    }
    InferredNode::CompilerError { message, span, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"kind\": \"CompilerError\", \"message\": ".to_string(), json_quote(&message)), ", \"span\": ".to_string()), serialize_span(span.clone())), "}".to_string())
    }
}
    })
}

pub fn serialize_resource_use(resource_use: Rc<ResourceUse>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_quote(&resource_use.name)), ", \"resource\": ".to_string()), serialize_node(resource_use.resource.clone())), ", \"span\": ".to_string()), serialize_span(resource_use.span.clone())), "}".to_string())
    })
}

pub fn serialize_field(field: Rc<Field>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_quote(&field.name)), ", \"type_expr\": ".to_string()), serialize_node(field.type_expr.clone())), ", \"cardinality\": ".to_string()), json_quote(&cardinality_name(field.cardinality.clone()))), ", \"default_value\": ".to_string()), json_optional_node(field.default_value.clone())), ", \"from_key\": ".to_string()), json_optional_string(field.from_key.clone())), ", \"span\": ".to_string()), serialize_span(field.span.clone())), "}".to_string())
}

pub fn serialize_param(param: Rc<Param>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_quote(&param.name)), ", \"type_expr\": ".to_string()), serialize_node(param.type_expr.clone())), ", \"default_value\": ".to_string()), json_optional_node(param.default_value.clone())), ", \"span\": ".to_string()), serialize_span(param.span.clone())), "}".to_string())
    })
}

pub fn serialize_node(node: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_quote(&node.name)), ", \"span\": ".to_string()), serialize_span(node.span.clone())), ", \"children\": ".to_string()), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in node.children.iter().cloned() {
        __mapped_2.push(serialize_node(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), ", \"connective\": ".to_string()), match node.connective.clone() {
    Some(inner) => {
        json_quote(&connective_name(inner))
    }
    None => {
        "null".to_string()
    }
}), ", \"params\": ".to_string()), json_list({
    let mut __mapped_6 = Vec::new();
    for __elem_7 in node.params.iter().cloned() {
        __mapped_6.push(serialize_param(__elem_7.clone()));
    }
    Rc::new(__mapped_6)
})), ", \"inferred\": ".to_string()), json_optional_inferred_node(node.inferred.clone())), ", \"return_cardinality\": ".to_string()), json_quote(&cardinality_name(node.return_cardinality.clone()))), ", \"uses\": ".to_string()), json_list({
    let mut __mapped_10 = Vec::new();
    for __elem_11 in node.uses.iter().cloned() {
        __mapped_10.push(serialize_resource_use(__elem_11.clone()));
    }
    Rc::new(__mapped_10)
})), ", \"body\": ".to_string()), json_optional_node(node.body.clone())), ", \"transport\": ".to_string()), json_optional_node(node.transport.clone())), ", \"properties\": ".to_string()), json_list({
    let mut __mapped_14 = Vec::new();
    for __elem_15 in node.properties.iter().cloned() {
        __mapped_14.push(serialize_field_init(__elem_15.clone()));
    }
    Rc::new(__mapped_14)
})), ", \"type_annotation\": ".to_string()), json_optional_node(node.type_annotation.clone())), ", \"is_self_recursive\": ".to_string()), json_bool(node.is_self_recursive.clone())), ", \"has_non_tail_self_call\": ".to_string()), json_bool(node.has_non_tail_self_call.clone())), ", \"expr_data\": ".to_string()), serialize_expr_data(node.clone())), "}".to_string())
    })
}

pub fn serialize_module(module: Rc<Node>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"name\": ".to_string(), json_quote(&module.name)), ", \"imports\": ".to_string()), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in module_imports(module.clone()).iter().cloned() {
        __mapped_2.push(serialize_import_node(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), ", \"span\": ".to_string()), serialize_span(module.span.clone())), "}".to_string())
}

pub fn serialize_typed_module(module: Rc<TypedModule>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"module\": ".to_string(), serialize_module(module.module.clone())), ", \"items\": ".to_string()), json_list({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in module.items.iter().cloned() {
        __mapped_2.push(serialize_node(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
})), ", \"item_registry_keys\": ".to_string()), json_list({
    let mut __mapped_12 = Vec::new();
    for __elem_13 in ({
    let __rc_9 = module.item_registry.clone();
    let __map_unwrapped_10 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    let mut __keys_11 = __map_unwrapped_10.into_keys().collect::<Vec<_>>();
    __keys_11.sort();
    Rc::new(__keys_11)
}).iter().cloned() {
        __mapped_12.push(json_quote(&__elem_13));
    }
    Rc::new(__mapped_12)
})), "}".to_string())
}

pub fn serialize_diagnostic(diagnostic: Rc<Node>) -> String {
    let cat = diagnostic_category(diagnostic.clone());
    let cat_json = match cat {
    Some(inner) => {
        json_quote(&inner)
    }
    None => {
        "null".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\"severity\": ".to_string(), json_quote(&diagnostic_severity(diagnostic.clone()))), ", \"message\": ".to_string()), json_quote(&diagnostic_message(diagnostic.clone()))), ", \"span\": ".to_string()), serialize_span(diagnostic_span(diagnostic.clone()))), ", \"module_name\": ".to_string()), json_optional_string(diagnostic_module_name(diagnostic.clone()))), ", \"category\": ".to_string()), cat_json.clone()), "}".to_string())
}

pub fn emit_dag_artifact(typed: Rc<ResolvedGraph>) -> Rc<EmitResult> {
    let modules_json = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __mapped_0.push(serialize_typed_module(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let diagnostics_json = {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in ({
    let mut __mapped_5 = Vec::new();
    for __elem_6 in typed.diagnostics.iter().cloned() {
        __mapped_5.push(serialize_diagnostic(__elem_6.clone()));
    }
    Rc::new(__mapped_5)
}).iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&", ".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
};
    let item_registry_json = {
    let mut __joined_15 = String::new();
    let mut __first_17 = true;
    for __elem_16 in ({
    let mut __mapped_13 = Vec::new();
    for __elem_14 in ({
    let __rc_10 = typed.item_registry.clone();
    let __map_unwrapped_11 = Rc::try_unwrap(__rc_10).unwrap_or_else(|rc| (*rc).clone());
    let mut __keys_12 = __map_unwrapped_11.into_keys().collect::<Vec<_>>();
    __keys_12.sort();
    Rc::new(__keys_12)
}).iter().cloned() {
        __mapped_13.push(json_quote(&__elem_14));
    }
    Rc::new(__mapped_13)
}).iter().cloned() {
        if !__first_17 {
    __joined_15.push_str(&", ".to_string());
};
        __first_17 = false;
        __joined_15.push_str(&__elem_16);
    }
    __joined_15
};
    let json = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\n  \"version\": \"0.1.0\",\n  \"modules\": [".to_string(), modules_json.clone()), "],\n  \"item_registry_keys\": [".to_string()), item_registry_json.clone()), "],\n  \"diagnostics\": [".to_string()), diagnostics_json.clone()), "],\n  \"files\": []\n}".to_string());
    Rc::new(EmitResult { files: Rc::new(vec!(Rc::new(TextFile { path: "dag-artifact.json".to_string(), content: json.clone() }))), diagnostics: Rc::new(Vec::new()) })
}

pub fn boundary_ref_error(names: Rc<Vec<String>>, ref_name: &str) -> Rc<Vec<Rc<Node>>> {
    {
let __cond = {
    let mut __any_0 = false;
    for __elem_1 in names.iter().cloned() {
        if __elem_1.clone() == ref_name {
    __any_0 = true;
    break;
};
    }
    __any_0
};
if __cond {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(compile_bundle_error(&v2_rt::concat(v2_rt::concat("boundary references unknown artifact '".to_string(), ref_name.to_string()), "'".to_string()))))
}
}
}

pub fn validate_boundaries(plan: Rc<ArtifactPlan>) -> Rc<Vec<Rc<Node>>> {
    let names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in plan.artifacts.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
};
    {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in plan.boundaries.iter().cloned() {
        __flat_mapped_2.extend(v2_rt::concat(boundary_ref_error(names.clone(), &__elem_3.from_artifact), boundary_ref_error(names.clone(), &__elem_3.to_artifact)).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
}
}

pub fn emit_from_artifact_plan(typed: Rc<ResolvedGraph>, artifact_plan: Rc<ArtifactPlan>) -> Rc<EmitResult> {
    if ({
    let __len_0 = artifact_plan.artifacts.clone().len();
    __len_0 as i64
}) == 0_i64 {
    return Rc::new(EmitResult { files: Rc::new(Vec::new()), diagnostics: Rc::new(vec!(compile_bundle_error("compile_sources planned no artifacts"))) });
};
    let boundary_diags = validate_boundaries(artifact_plan.clone());
    if ({
    let __len_1 = boundary_diags.clone().len();
    __len_1 as i64
}) > 0_i64 {
    return Rc::new(EmitResult { files: Rc::new(Vec::new()), diagnostics: boundary_diags.clone() });
};
    let results = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in artifact_plan.artifacts.iter().cloned() {
        __mapped_2.push(emit_artifact(typed.clone(), __elem_3.clone()));
    }
    Rc::new(__mapped_2)
};
    let all_files = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in results.iter().cloned() {
        __flat_mapped_4.extend(__elem_5.files.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    let all_diags = {
    let mut __flat_mapped_6 = Vec::new();
    for __elem_7 in results.iter().cloned() {
        __flat_mapped_6.extend(__elem_7.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_6)
};
    Rc::new(EmitResult { files: all_files.clone(), diagnostics: all_diags.clone() })
}

pub fn collect_diagnostics(parse_results: Rc<Vec<Rc<ParseResult>>>) -> Rc<Vec<Rc<Node>>> {
    {
    let mut __acc_0: Rc<Vec<Rc<Node>>> = Rc::new(Vec::new());
    for __elem_1 in parse_results.iter().cloned() {
        __acc_0 = match __elem_1.error.as_ref().map(|__rc| __rc.as_ref()) {
    Some(diag) => {
        let diag = Rc::new(diag.clone());
        {
    let __rc_3 = __acc_0;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(diag.clone());
    Rc::new(__appended_2)
}
    }
    None => {
        __acc_0
    }
};
    }
    __acc_0
}
}

pub fn front_end_sources(sources: Rc<Vec<Rc<SourceFile>>>) -> Rc<FrontendResult> {
    let tokenized = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in sources.iter().cloned() {
        __mapped_0.push(tokenize(&__elem_1.content));
    }
    Rc::new(__mapped_0)
};
    let parse_results = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in tokenized.iter().cloned() {
        __mapped_2.push(parse(__elem_3.clone()));
    }
    Rc::new(__mapped_2)
};
    let parse_diagnostics = collect_diagnostics(parse_results.clone());
    let has_parse_errors = {
    let mut __any_4 = false;
    for __elem_5 in parse_results.iter().cloned() {
        if __elem_5.error.clone().is_some() {
    __any_4 = true;
    break;
};
    }
    __any_4
};
    if has_parse_errors.clone() {
    Rc::new(FrontendResult { graph: None, diagnostics: parse_diagnostics.clone() })
} else {
    let modules = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in parse_results.iter().cloned() {
        __mapped_6.push(__elem_7.module.clone().unwrap());
    }
    Rc::new(__mapped_6)
};
    let graph = resolve_modules(modules.clone());
    Rc::new(FrontendResult { graph: Some(graph.clone()), diagnostics: v2_rt::concat(parse_diagnostics.clone(), graph.diagnostics.clone()) })
}
}

pub fn resolve_sources(sources: Rc<Vec<Rc<SourceFile>>>) -> Rc<CompileResult> {
    let frontend = front_end_sources(sources.clone());
    Rc::new(CompileResult { files: Rc::new(Vec::new()), diagnostics: frontend.diagnostics.clone() })
}

pub fn compile_sources(sources: Rc<Vec<Rc<SourceFile>>>, target: RenderTarget) -> Rc<PipelineResult> {
    let frontend = front_end_sources(sources.clone());
    match frontend.graph.as_ref().map(|__rc| __rc.as_ref()) {
    None => {
        Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: frontend.diagnostics.clone(), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() })
    }
    Some(graph) => {
        let graph = Rc::new(graph.clone());
        {
    let graph_diags = graph.diagnostics.clone();
    let resolve_errors = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in graph_diags.iter().cloned() {
        if diagnostic_is_error(__elem_1.clone()) {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    if ({
    let __len_2 = resolve_errors.clone().len();
    __len_2 as i64
}) > 0_i64 {
    return Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: frontend.diagnostics.clone(), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() });
};
    let norm = normalize_graph(graph.clone());
    let norm_diags = norm.diagnostics.clone();
    let norm_errors = {
    let mut __filtered_3 = Vec::new();
    for __elem_4 in norm_diags.iter().cloned() {
        if diagnostic_is_error(__elem_4.clone()) {
    __filtered_3.push(__elem_4);
};
    }
    Rc::new(__filtered_3)
};
    if ({
    let __len_5 = norm_errors.clone().len();
    __len_5 as i64
}) > 0_i64 {
    return Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: v2_rt::concat(frontend.diagnostics.clone(), norm_diags.clone()), complexity: empty_complexity_report(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() });
};
    let typed = reconcile(norm.graph.clone());
    let typed_diags = typed.diagnostics.clone();
    let func_entries = extract_func_entries(typed.clone());
    let complexity = build_complexity_report(func_entries.clone());
    let typecheck_errors = {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in typed_diags.iter().cloned() {
        if diagnostic_is_error(__elem_7.clone()) {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
};
    if ({
    let __len_8 = typecheck_errors.clone().len();
    __len_8 as i64
}) > 0_i64 {
    return Rc::new(PipelineResult { files: Rc::new(Vec::new()), diagnostics: v2_rt::concat(v2_rt::concat(frontend.diagnostics.clone(), norm_diags.clone()), typed_diags.clone()), complexity: complexity.clone(), ownership: Rc::new(Vec::new()), artifact_plan: empty_artifact_plan() });
};
    let ownership = extract_ownership_proofs(typed.clone());
    let ownership_diags = ownership_diagnostics(ownership.clone());
    let artifact_plan = default_artifact_plan({
    let mut __mapped_11 = Vec::new();
    for __elem_12 in typed.modules.iter().cloned() {
        __mapped_11.push(__elem_12.module.name.clone());
    }
    Rc::new(__mapped_11)
}, target);
    let emit_result = emit_from_artifact_plan(typed.clone(), artifact_plan.clone());
    let emit_files = emit_result.files.clone();
    let emit_diags = emit_result.diagnostics.clone();
    let emit_errors = {
    let mut __filtered_13 = Vec::new();
    for __elem_14 in emit_diags.iter().cloned() {
        if diagnostic_is_error(__elem_14.clone()) {
    __filtered_13.push(__elem_14);
};
    }
    Rc::new(__filtered_13)
};
    let final_files = if ({
    let __len_15 = emit_errors.clone().len();
    __len_15 as i64
}) > 0_i64 {
    Rc::new(Vec::new())
} else {
    emit_files.clone()
};
    Rc::new(PipelineResult { files: final_files.clone(), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(frontend.diagnostics.clone(), norm_diags.clone()), typed_diags.clone()), ownership_diags.clone()), emit_diags.clone()), complexity: complexity.clone(), ownership: ownership.clone(), artifact_plan: artifact_plan.clone() })
}
    }
}
}

