use crate::v2_core::*;
use crate::resolve::*;
use crate::infer_types::*;
use crate::infer_method::*;
use crate::infer_cycle::*;
use crate::infer_env::*;
use crate::infer_resolve::*;
use crate::infer_sigs::*;
use crate::infer_emit_info::*;
use crate::infer_items::*;
use crate::infer_service::*;
use crate::infer_patterns::*;
use crate::infer_lookup::*;
use crate::infer_access::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemContribution {
    pub resolved_item: Rc<Node>,
    pub resolve_diagnostics: Rc<Vec<Rc<Diagnostic>>>,
    pub func_sig: Option<Rc<DeclaredFuncSig>>,
    pub svc_entries: Rc<Vec<Rc<OpEntry>>>,
    pub svc_local: Option<Rc<TypeBinding>>,
    pub item_info: Rc<ItemInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleContext {
    pub resolved_items: Rc<Vec<Rc<Node>>>,
    pub func_env: Rc<ResolvedFuncEnv>,
    pub svc_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>,
    pub locals: Rc<HashMap<String, Rc<TypeBinding>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InferScope {
    pub type_env: Rc<TypeEnv>,
    pub func_env: Rc<ResolvedFuncEnv>,
    pub locals: Rc<HashMap<String, Rc<TypeBinding>>>,
    pub module_name: String,
    pub service_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InferResult {
    pub typed: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockInferState {
    pub scope: Rc<InferScope>,
    pub diag_chunks: Rc<Vec<Rc<Vec<Rc<Diagnostic>>>>>,
    pub last_type: Rc<Node>,
    pub typed_stmts: Rc<Vec<Rc<Node>>>,
}

pub fn infer_block_stmts(remaining: Rc<Vec<Rc<Node>>>, scope: Rc<InferScope>, typed_stmts: Rc<Vec<Rc<Node>>>, diag_chunks: Rc<Vec<Rc<Vec<Rc<Diagnostic>>>>>, last_type: Rc<Node>) -> Rc<BlockInferState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_scope = scope;
        let mut __tco_p_typed_stmts = typed_stmts;
        let mut __tco_p_diag_chunks = diag_chunks;
        let mut __tco_p_last_type = last_type;
        loop {
            let remaining = __tco_p_remaining;
            let scope = __tco_p_scope;
            let typed_stmts = __tco_p_typed_stmts;
            let diag_chunks = __tco_p_diag_chunks;
            let last_type = __tco_p_last_type;
            match remaining.clone().first().cloned() {
    None => {
        break Rc::new(BlockInferState { scope: scope.clone(), diag_chunks: diag_chunks.clone(), last_type: last_type.clone(), typed_stmts: typed_stmts.clone() });
    }
    Some(stmt) => {
        {
    let stmt_result = infer_expr(stmt.clone(), scope.clone());
    let stmt_typed = stmt_result.typed.clone();
    let stmt_diags = stmt_result.diagnostics.clone();
    let stmt_rt = rt_type(stmt_typed.clone());
    let next_scope = scope_after_stmt_node(stmt.clone(), stmt_rt.clone(), scope.clone());
     {
        let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
        let __tco_1 = next_scope.clone();
        let __tco_2 = {
    let __rc_1 = typed_stmts;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(stmt_typed.clone());
    Rc::new(__appended_0)
};
        let __tco_3 = {
    let __rc_3 = diag_chunks;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(stmt_diags.clone());
    Rc::new(__appended_2)
};
        let __tco_4 = stmt_rt.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_scope = __tco_1;
        __tco_p_typed_stmts = __tco_2;
        __tco_p_diag_chunks = __tco_3;
        __tco_p_last_type = __tco_4;
        continue;
    }

};
    }
};
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedItemResult {
    pub item: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArmInferResult {
    pub typed_arm: Rc<MatchArm>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
    pub body_type: Rc<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatternScopeResult {
    pub scope: Rc<InferScope>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringPartInferResult {
    pub typed_part: Rc<StringPart>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArgInferResult {
    pub typed_arg: Rc<NamedArg>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldInferResult {
    pub typed_field: Rc<FieldInit>,
    pub infer_result: Rc<InferResult>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuildTypeEnvResult {
    pub env: Rc<TypeEnv>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParentModulesResult {
    pub modules: Rc<Vec<Rc<TypedModule>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VariantResult {
    pub variant: Rc<Variant>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CycleDetectState {
    pub recursive_names: Rc<HashMap<String, bool>>,
    pub global_visited: Rc<HashMap<String, bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InferScopeComponents {
    pub func_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>,
    pub svc_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>,
    pub svc_locals: Rc<HashMap<String, Rc<TypeBinding>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocalContributionState {
    pub resolved_items: Rc<Vec<Rc<Node>>>,
    pub func_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>,
    pub svc_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>,
    pub svc_locals: Rc<HashMap<String, Rc<TypeBinding>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub diag_chunks: Rc<Vec<Rc<Vec<Rc<Diagnostic>>>>>,
}

pub fn merge_scope_from_imports(remaining: Rc<Vec<Rc<ResolvedImport>>>, parent_index: Rc<HashMap<String, Rc<TypedModule>>>, env: Rc<TypeEnv>, func_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>, svc_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>, svc_locals: Rc<HashMap<String, Rc<TypeBinding>>>) -> Rc<InferScopeComponents> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_parent_index = parent_index;
        let mut __tco_p_env = env;
        let mut __tco_p_func_sigs = func_sigs;
        let mut __tco_p_svc_registry = svc_registry;
        let mut __tco_p_svc_locals = svc_locals;
        loop {
            let remaining = __tco_p_remaining;
            let parent_index = __tco_p_parent_index;
            let env = __tco_p_env;
            let func_sigs = __tco_p_func_sigs;
            let svc_registry = __tco_p_svc_registry;
            let svc_locals = __tco_p_svc_locals;
            match remaining.clone().first().cloned() {
    None => {
        break Rc::new(InferScopeComponents { func_sigs: func_sigs.clone(), svc_registry: svc_registry.clone(), svc_locals: svc_locals.clone() });
    }
    Some(imp) => {
        match parent_index.clone().get(&imp.module_path.clone()).cloned() {
    Some(typed_parent) => {
        {
    let next_func_sigs = {
    let mut __acc_4 = func_sigs.clone();
    for __elem_5 in ({
    let __rc_0 = typed_parent.func_env.signatures.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = {
    let sig_key = __elem_5.name.clone();
    let sig_params = __elem_5.params.clone();
    let sig_rt = __elem_5.return_type.clone();
    let sig_async = __elem_5.is_async.clone();
    {
    let __rc_7 = __acc_4;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(sig_key.clone(), Rc::new(DeclaredFuncSig { name: sig_key.clone(), params: sig_params.clone(), return_type: Some(sig_rt.clone()), is_async: sig_async.clone() }));
    Rc::new(__map_ins_6)
}
};
    }
    __acc_4
};
    let parent_result = {
    let mut __acc_8 = Rc::new(InferScopeComponents { func_sigs: next_func_sigs.clone(), svc_registry: svc_registry.clone(), svc_locals: svc_locals.clone() });
    for __elem_9 in typed_parent.items.iter().cloned() {
        __acc_8 = if (__elem_9.transport.clone().is_some()) && (({
    let __len_19 = __elem_9.children.clone().len();
    __len_19 as i64
}) > 0_i64) {
    let entries = {
    let mut __mapped_10 = Vec::new();
    for __elem_11 in __elem_9.children.iter().cloned() {
        __mapped_10.push(Rc::new(OpEntry { name: __elem_11.name.clone(), outputs: return_type_to_outputs(__elem_11.return_type.clone(), __elem_11.span.clone()), params: __elem_11.params.clone() }));
    }
    Rc::new(__mapped_10)
};
    let root = namespace_root_from_properties(__elem_9.properties.clone(), &__elem_9.name);
    Rc::new(InferScopeComponents { func_sigs: __acc_8.func_sigs.clone(), svc_registry: {
    let __rc_13 = std::mem::take(&mut Rc::make_mut(&mut __acc_8).svc_registry);
    let mut __map_ins_12 = Rc::try_unwrap(__rc_13).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_12.insert(__elem_9.name.clone(), entries.clone());
    Rc::new(__map_ins_12)
}, svc_locals: {
    let __rc_15 = std::mem::take(&mut Rc::make_mut(&mut __acc_8).svc_locals);
    let mut __map_ins_14 = Rc::try_unwrap(__rc_15).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_14.insert(root.clone(), Rc::new(TypeBinding { name: root.clone(), resolved: leaf_node(&root) }));
    Rc::new(__map_ins_14)
} })
} else {
    if ((((__elem_9.body.clone().is_some()) && (({
    let __len_18 = __elem_9.params.clone().len();
    __len_18 as i64
}) == 0_i64)) && (__elem_9.transport.clone().is_none())) && (node_has_structure(__elem_9.clone()) == false)) && (__elem_9.return_type.clone().is_some()) {
    Rc::new(InferScopeComponents { func_sigs: __acc_8.func_sigs.clone(), svc_registry: __acc_8.svc_registry.clone(), svc_locals: {
    let __rc_17 = std::mem::take(&mut Rc::make_mut(&mut __acc_8).svc_locals);
    let mut __map_ins_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_16.insert(__elem_9.name.clone(), Rc::new(TypeBinding { name: __elem_9.name.clone(), resolved: rt_type(__elem_9.clone()) }));
    Rc::new(__map_ins_16)
} })
} else {
    __acc_8.clone()
}
};
    }
    __acc_8
};
     {
        let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
        let __tco_1 = parent_index.clone();
        let __tco_2 = env.clone();
        let __tco_3 = parent_result.func_sigs.clone();
        let __tco_4 = parent_result.svc_registry.clone();
        let __tco_5 = parent_result.svc_locals.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_parent_index = __tco_1;
        __tco_p_env = __tco_2;
        __tco_p_func_sigs = __tco_3;
        __tco_p_svc_registry = __tco_4;
        __tco_p_svc_locals = __tco_5;
        continue;
    }

};
    }
    None => {
         {
            let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
            let __tco_1 = parent_index.clone();
            let __tco_2 = env.clone();
            let __tco_3 = func_sigs.clone();
            let __tco_4 = svc_registry.clone();
            let __tco_5 = svc_locals.clone();
            __tco_p_remaining = __tco_0;
            __tco_p_parent_index = __tco_1;
            __tco_p_env = __tco_2;
            __tco_p_func_sigs = __tco_3;
            __tco_p_svc_registry = __tco_4;
            __tco_p_svc_locals = __tco_5;
            continue;
        }

    }
};
    }
};
        }
    })
}

pub fn namespace_root_from_properties(properties: Rc<Vec<Rc<FieldInit>>>, name: &str) -> String {
    match {
    let mut __found_2 = None;
    for __elem_3 in properties.iter().cloned() {
        if __elem_3.name.clone() == "namespace_root" {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(ns_prop) => {
        match ns_prop.value.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitStr { value: root, .. } = value.as_ref() else { unreachable!() };
        root.clone()
    }
    _ => {
        name.to_string()
    }
}
    }
    None => {
        name.to_string()
    }
}
}

pub fn expr_span(texpr: Rc<Node>) -> SourceSpan {
    texpr.span.clone()
}

pub fn ok_infer(texpr: Rc<Node>) -> Rc<InferResult> {
    Rc::new(InferResult { typed: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
}

pub fn semantic_expr_error_node(message: &str, span: SourceSpan) -> Rc<Node> {
    make_expr_error_node(ExprErrorKind::SemanticExprError, &message, span)
}

pub fn internal_expr_error_node(message: &str, span: SourceSpan) -> Rc<Node> {
    make_expr_error_node(ExprErrorKind::InternalExprError, &message, span)
}

pub fn lookup_variant_parent_enum(scope: Rc<InferScope>, name: &str) -> Option<String> {
    match scope.locals.clone().get(&name.to_string()).cloned() {
    Some(binding) => {
        match lookup_type(scope.type_env.clone(), &binding.resolved.name) {
    Some(parent) => {
        if node_is_coproduct(parent.clone()) {
    {
let __cond = {
    let mut __any_0 = false;
    for __elem_1 in parent.children.iter().cloned() {
        if __elem_1.name.clone() == name {
    __any_0 = true;
    break;
};
    }
    __any_0
};
if __cond {
    Some(binding.resolved.name.clone())
} else {
    None
}
}
} else {
    None
}
    }
    None => {
        None
    }
}
    }
    None => {
        None
    }
}
}

pub fn infer_var_binding_kind(scope: Rc<InferScope>, name: &str) -> Rc<VarBindingKind> {
    match lookup_variant_parent_enum(scope.clone(), &name) {
    Some(parent_enum) => {
        Rc::new(VarBindingKind::VariantValueBinding { parent_enum })
    }
    None => {
        match scope.locals.clone().get(&name.to_string()).cloned() {
    Some(_) => {
        Rc::new(VarBindingKind::LocalValueBinding)
    }
    None => {
        Rc::new(VarBindingKind::FunctionValueBinding)
    }
}
    }
}
}

pub fn inference_error(message: &str, span: SourceSpan, module_name: &str) -> Rc<Diagnostic> {
    Rc::new(Diagnostic { severity: Severity::Error, message: message.to_string(), span: Some(span), module_name: Some(module_name.to_string()), category: None })
}

pub fn categorized_error(message: &str, span: SourceSpan, module_name: &str, category: ErrorCategory) -> Rc<Diagnostic> {
    Rc::new(Diagnostic { severity: Severity::Error, message: message.to_string(), span: Some(span), module_name: Some(module_name.to_string()), category: Some(category) })
}

pub fn inferred_from_node_type(result: Rc<NodeType>, fallback_message: &str, fallback_span: SourceSpan) -> Rc<InferredNode> {
    match result.as_ref() {
    NodeType::Typed { node: resolved, .. } => {
        Rc::new(InferredNode::Resolved { node: resolved.clone() })
    }
    NodeType::InferError { message, span, .. } => {
        Rc::new(InferredNode::CompilerError { message: message.clone(), span: span.clone() })
    }
    NodeType::Untyped => {
        Rc::new(InferredNode::CompilerError { message: fallback_message.to_string(), span: fallback_span })
    }
}
}

pub fn lambda_semantics_from_param_types(param_types: Rc<Vec<Rc<Node>>>) -> Rc<LambdaSemantics> {
    Rc::new(LambdaSemantics { param_types: param_types.clone() })
}

pub fn lambda_param_types_from_scope(scope: Rc<InferScope>, params: Rc<Vec<String>>) -> Rc<Vec<Rc<Node>>> {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(match scope.locals.clone().get(&__elem_1.clone()).cloned() {
    Some(binding) => {
        binding.resolved.clone()
    }
    None => {
        error_type_node()
    }
});
    }
    Rc::new(__mapped_0)
}
}

pub fn resolve_pattern_subject(scope: Rc<InferScope>, scrutinee_subject: Rc<PatternSubject>) -> Rc<PatternSubject> {
    match scrutinee_subject.as_ref() {
    PatternSubject::PatternResolved { node: scrutinee_type, .. } => {
        pattern_subject_from_node(resolve_scrutinee_type_node(scope.type_env.clone(), scrutinee_type.clone()))
    }
    PatternSubject::PatternDynamic { span: dynamic_span, .. } => {
        Rc::new(PatternSubject::PatternDynamic { span: dynamic_span.clone() })
    }
    PatternSubject::PatternLookupBlocked => {
        Rc::new(PatternSubject::PatternLookupBlocked)
    }
}
}

pub fn annotate_pattern_parent_enums(pattern: Rc<MatchPattern>, scrutinee_subject: Rc<PatternSubject>, scope: Rc<InferScope>) -> Rc<MatchPattern> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::VariantPattern { name: variant_name, parent_enum: _, field_bindings: bindings, .. } => {
        {
    let resolved_scrut = resolve_pattern_subject(scope.clone(), scrutinee_subject.clone());
    let inferred_parent = match resolved_scrut.as_ref() {
    PatternSubject::PatternResolved { node: resolved_scrut_node, .. } => {
        if node_is_optional(resolved_scrut_node.clone()) && ((variant_name.clone() == "Some") || (variant_name.clone() == "None")) {
    Some("Optional".to_string())
} else {
    if node_is_coproduct(resolved_scrut_node.clone()) {
    Some(resolved_scrut_node.name.clone())
} else {
    None
}
}
    }
    PatternSubject::PatternDynamic { span: _, .. } => {
        None
    }
    PatternSubject::PatternLookupBlocked => {
        None
    }
};
    let variant_lookup = lookup_variant_in_type(resolved_scrut.clone(), &variant_name, &scope.module_name);
    let variant_subject = lookup_result_subject(variant_lookup.clone());
    let annotated_bindings = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in bindings.iter().cloned() {
        __mapped_0.push({
    let field_lookup = lookup_field_in_variant(variant_subject.clone(), &__elem_1.field_name, &scope.module_name);
    let field_subject = lookup_result_subject(field_lookup.clone());
    Rc::new(FieldBinding { field_name: __elem_1.field_name.clone(), binding: annotate_pattern_parent_enums(__elem_1.binding.clone(), field_subject.clone(), scope.clone()) })
});
    }
    Rc::new(__mapped_0)
};
    match inferred_parent.clone() {
    Some(parent_name) => {
        Rc::new(MatchPattern::VariantPattern { name: variant_name.clone(), parent_enum: Some(parent_name.clone()), field_bindings: annotated_bindings.clone() })
    }
    None => {
        Rc::new(MatchPattern::VariantPattern { name: variant_name.clone(), parent_enum: None, field_bindings: annotated_bindings.clone() })
    }
}
}
    }
    _ => {
        pattern.clone()
    }
}
    })
}

pub fn build_params_scope(scope: Rc<InferScope>, params: Rc<Vec<Rc<Param>>>) -> Rc<InferScope> {
    let new_locals = {
    let mut __acc_0 = scope.locals.clone();
    for __elem_1 in params.iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.name.clone(), Rc::new(TypeBinding { name: __elem_1.name.clone(), resolved: __elem_1.type_expr.clone() }));
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
    Rc::new(InferScope { type_env: scope.type_env.clone(), func_env: scope.func_env.clone(), locals: new_locals.clone(), module_name: scope.module_name.clone(), service_registry: scope.service_registry.clone(), item_registry: scope.item_registry.clone() })
}

pub fn extend_scope(scope: Rc<InferScope>, name: &str, resolved: Rc<Node>) -> Rc<InferScope> {
    {
    let __rc_1 = scope;
    let mut __owned_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| { debug_assert!(false, "V5: expected sole ownership of `scope`"); (*rc).clone() });
    let __taken_2 = std::mem::take(&mut __owned_0.locals);
    __owned_0.locals = {
    let __rc_4 = __taken_2;
    let mut __map_ins_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_3.insert(name.to_string(), Rc::new(TypeBinding { name: name.to_string(), resolved: resolved.clone() }));
    Rc::new(__map_ins_3)
};
    Rc::new(__owned_0)
}
}

pub fn extend_scope_with_params(scope: Rc<InferScope>, params: Rc<Vec<String>>) -> Rc<InferScope> {
    let new_locals = {
    let mut __acc_0 = scope.locals.clone();
    for __elem_1 in params.iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.clone(), Rc::new(TypeBinding { name: __elem_1.clone(), resolved: leaf_node("Dynamic") }));
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
    Rc::new(InferScope { type_env: scope.type_env.clone(), func_env: scope.func_env.clone(), locals: new_locals.clone(), module_name: scope.module_name.clone(), service_registry: scope.service_registry.clone(), item_registry: scope.item_registry.clone() })
}

pub fn scope_after_stmt_node(stmt: Rc<Node>, stmt_type: Rc<Node>, scope: Rc<InferScope>) -> Rc<InferScope> {
    match stmt.expr_data.as_ref() {
    ExprData::ExprLet { name, value: _, body, .. } => {
        if body.clone().is_none() {
    extend_scope(scope.clone(), &name, stmt_type.clone())
} else {
    scope.clone()
}
    }
    _ => {
        scope.clone()
    }
}
}

pub fn extend_scope_with_pattern_node(scope: Rc<InferScope>, pattern: Rc<MatchPattern>, scrutinee_subject: Rc<PatternSubject>) -> Rc<PatternScopeResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::Bind { name: n, .. } => {
        Rc::new(PatternScopeResult { scope: extend_scope(scope.clone(), &n, pattern_binding_type(scrutinee_subject.clone())), diagnostics: Rc::new(Vec::new()) })
    }
    MatchPattern::Wildcard => {
        Rc::new(PatternScopeResult { scope: scope.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    MatchPattern::LitPattern { value: _, .. } => {
        Rc::new(PatternScopeResult { scope: scope.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    MatchPattern::VariantPattern { name: vname, parent_enum: _, field_bindings: bindings, .. } => {
        {
    let resolved_scrut = resolve_pattern_subject(scope.clone(), scrutinee_subject.clone());
    let variant_lookup = lookup_variant_in_type(resolved_scrut.clone(), &vname, &scope.module_name);
    let variant_subject = lookup_result_subject(variant_lookup.clone());
    let variant_diags = variant_lookup.diagnostics.clone();
    {
    let mut __acc_0 = Rc::new(PatternScopeResult { scope: scope.clone(), diagnostics: variant_diags.clone() });
    for __elem_1 in bindings.iter().cloned() {
        __acc_0 = {
    let field_lookup = lookup_field_in_variant(variant_subject.clone(), &__elem_1.field_name, &scope.module_name);
    let field_subject = lookup_result_subject(field_lookup.clone());
    let field_type = pattern_binding_type(field_subject.clone());
    match __elem_1.binding.as_ref() {
    MatchPattern::Bind { name: n, .. } => {
        Rc::new(PatternScopeResult { scope: extend_scope(__acc_0.scope.clone(), &n, field_type.clone()), diagnostics: v2_rt::concat(__acc_0.diagnostics.clone(), field_lookup.diagnostics.clone()) })
    }
    _ => {
        {
    let nested = extend_scope_with_pattern_node(__acc_0.scope.clone(), __elem_1.binding.clone(), field_subject.clone());
    Rc::new(PatternScopeResult { scope: nested.scope.clone(), diagnostics: v2_rt::concat(__acc_0.diagnostics.clone(), v2_rt::concat(field_lookup.diagnostics.clone(), nested.diagnostics.clone())) })
}
    }
}
};
    }
    __acc_0
}
}
    }
}
    })
}

pub fn arg_has_name(arg: Rc<NamedArg>, name: &str) -> bool {
    if arg.name.clone().is_none() {
    false
} else {
    arg.name.clone().unwrap() == name
}
}

pub fn extract_fold_init_info(method_name: &str, method_args: Rc<Vec<Rc<NamedArg>>>, min_args: i64, scope: Rc<InferScope>) -> Option<Rc<ArgInferResult>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if (method_name == "fold") && (({
    let __len_4 = method_args.clone().len();
    __len_4 as i64
}) >= min_args) {
    let init_arg = match {
    let mut __found_2 = None;
    for __elem_3 in method_args.iter().cloned() {
        if arg_has_name(__elem_3.clone(), "init") {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(a) => {
        Some(a.clone())
    }
    None => {
        method_args.clone().first().cloned()
    }
};
    match init_arg.clone() {
    Some(ia) => {
        {
    let init_result = infer_expr(ia.value.clone(), scope.clone());
    Some(Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: ia.name.clone(), value: init_result.typed.clone() }), diagnostics: init_result.diagnostics.clone() }))
}
    }
    None => {
        None
    }
}
} else {
    None
}
    })
}

pub fn infer_method_args_with_fold(method_name: &str, method_args: Rc<Vec<Rc<NamedArg>>>, fold_info: Option<Rc<ArgInferResult>>, fold_acc_type: Rc<Node>, element_type: Rc<Node>, scope: Rc<InferScope>) -> Rc<Vec<Rc<ArgInferResult>>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in method_args.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __mapped_3.push({
    let a = __elem_4.1.clone();
    let idx = __elem_4.0.clone();
    let is_init_arg = arg_has_name(a.clone(), "init") || ((a.name.clone().is_none()) && (idx.clone() == 0_i64));
    if ((method_name == "fold") && (fold_info.clone().is_some())) && is_init_arg.clone() {
    match fold_info.as_ref().map(|__rc| __rc.as_ref()) {
    Some(fi) => {
        let fi = Rc::new(fi.clone());
        fi.clone()
    }
    None => {
        infer_arg_with_element_type(a.clone(), element_type.clone(), scope.clone())
    }
}
} else {
    if (method_name == "fold") && (fold_info.clone().is_some()) {
    infer_fold_lambda_arg(a.clone(), scope.clone(), fold_acc_type.clone(), element_type.clone())
} else {
    infer_arg_with_element_type(a.clone(), element_type.clone(), scope.clone())
}
}
});
    }
    Rc::new(__mapped_3)
}
    })
}

pub fn infer_fold_lambda_arg(arg: Rc<NamedArg>, scope: Rc<InferScope>, acc_type: Rc<Node>, elem_type: Rc<Node>) -> Rc<ArgInferResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match arg.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: lam_params, body: lam_body, semantics: _, .. } => {
        {
    let lam_span = arg.value.span.clone();
    let param_count = {
    let __len_0 = lam_params.clone().len();
    __len_0 as i64
};
    let param_types = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __enumerated_1 = Vec::new();
    for (__idx_2, __elem_3) in lam_params.clone().iter().enumerate() {
        __enumerated_1.push((__idx_2 as i64, __elem_3.clone()));
    }
    Rc::new(__enumerated_1)
}).iter().cloned() {
        __mapped_4.push(if __elem_5.0.clone() == 0_i64 {
    acc_type.clone()
} else {
    if __elem_5.0.clone() == (param_count.clone() - 1_i64) {
    elem_type.clone()
} else {
    leaf_node("Dynamic")
}
});
    }
    Rc::new(__mapped_4)
};
    let lam_locals = {
    let mut __acc_9 = scope.locals.clone();
    for __elem_10 in ({
    let mut __enumerated_6 = Vec::new();
    for (__idx_7, __elem_8) in lam_params.clone().iter().enumerate() {
        __enumerated_6.push((__idx_7 as i64, __elem_8.clone()));
    }
    Rc::new(__enumerated_6)
}).iter().cloned() {
        __acc_9 = if __elem_10.0.clone() == 0_i64 {
    {
    let __rc_12 = __acc_9;
    let mut __map_ins_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_11.insert(__elem_10.1.clone(), Rc::new(TypeBinding { name: __elem_10.1.clone(), resolved: acc_type.clone() }));
    Rc::new(__map_ins_11)
}
} else {
    if __elem_10.0.clone() == (param_count.clone() - 1_i64) {
    {
    let __rc_14 = __acc_9;
    let mut __map_ins_13 = Rc::try_unwrap(__rc_14).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_13.insert(__elem_10.1.clone(), Rc::new(TypeBinding { name: __elem_10.1.clone(), resolved: elem_type.clone() }));
    Rc::new(__map_ins_13)
}
} else {
    {
    let __rc_16 = __acc_9;
    let mut __map_ins_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_15.insert(__elem_10.1.clone(), Rc::new(TypeBinding { name: __elem_10.1.clone(), resolved: leaf_node("Dynamic") }));
    Rc::new(__map_ins_15)
}
}
};
    }
    __acc_9
};
    let typed_lam_scope = Rc::new(InferScope { type_env: scope.type_env.clone(), func_env: scope.func_env.clone(), locals: lam_locals.clone(), module_name: scope.module_name.clone(), service_registry: scope.service_registry.clone(), item_registry: scope.item_registry.clone() });
    let body_result = infer_expr(lam_body.clone(), typed_lam_scope.clone());
    let typed_lam = make_expr_node(Rc::new(ExprData::ExprLambda { params: lam_params.clone(), body: body_result.typed.clone(), semantics: Some(lambda_semantics_from_param_types(param_types.clone())) }), Some(Rc::new(InferredNode::Resolved { node: rt_type(body_result.typed.clone()) })), lam_span);
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: arg.name.clone(), value: typed_lam.clone() }), diagnostics: body_result.diagnostics.clone() })
}
    }
    _ => {
        {
    let ar = infer_expr(arg.value.clone(), scope.clone());
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: arg.name.clone(), value: ar.typed.clone() }), diagnostics: ar.diagnostics.clone() })
}
    }
}
    })
}

pub fn infer_lambda_with_element_type(lambda_expr: Rc<Node>, element_type: Rc<Node>, scope: Rc<InferScope>) -> Rc<InferResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match lambda_expr.expr_data.as_ref() {
    ExprData::ExprLambda { params: lam_params, body: lam_body, semantics: _, .. } => {
        {
    let span = lambda_expr.span.clone();
    let param_types = if ({
    let __len_6 = lam_params.clone().len();
    __len_6 as i64
}) == 1_i64 {
    Rc::new(vec!(element_type.clone()))
} else {
    {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in lam_params.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __mapped_3.push(if __elem_4.0.clone() == (({
    let __len_5 = lam_params.clone().len();
    __len_5 as i64
}) - 1_i64) {
    element_type.clone()
} else {
    leaf_node("Dynamic")
});
    }
    Rc::new(__mapped_3)
}
};
    let typed_scope = if ({
    let __len_13 = lam_params.clone().len();
    __len_13 as i64
}) == 1_i64 {
    match lam_params.clone().first().cloned() {
    Some(p) => {
        extend_scope(scope.clone(), &p, element_type.clone())
    }
    None => {
        scope.clone()
    }
}
} else {
    {
    let mut __acc_10 = scope.clone();
    for __elem_11 in ({
    let mut __enumerated_7 = Vec::new();
    for (__idx_8, __elem_9) in lam_params.clone().iter().enumerate() {
        __enumerated_7.push((__idx_8 as i64, __elem_9.clone()));
    }
    Rc::new(__enumerated_7)
}).iter().cloned() {
        __acc_10 = if __elem_11.0.clone() == (({
    let __len_12 = lam_params.clone().len();
    __len_12 as i64
}) - 1_i64) {
    extend_scope(__acc_10.clone(), &__elem_11.1, element_type.clone())
} else {
    extend_scope(__acc_10.clone(), &__elem_11.1, leaf_node("Dynamic"))
};
    }
    __acc_10
}
};
    let body_result = infer_expr(lam_body.clone(), typed_scope.clone());
    let body_typed = body_result.typed.clone();
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprLambda { params: lam_params.clone(), body: body_typed.clone(), semantics: Some(lambda_semantics_from_param_types(param_types.clone())) }), Some(Rc::new(InferredNode::Resolved { node: rt_type(body_typed.clone()) })), span), diagnostics: body_result.diagnostics.clone() })
}
    }
    _ => {
        infer_expr(lambda_expr.clone(), scope.clone())
    }
}
    })
}

pub fn is_lambda_expr(e: Rc<Node>) -> bool {
    match e.expr_data.as_ref() {
    ExprData::ExprLambda { params: _, body: _, semantics: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn infer_lambda_with_callable_type(lambda_expr: Rc<Node>, callable_type: Rc<Node>, arg_name: Option<String>, scope: Rc<InferScope>) -> Rc<ArgInferResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match lambda_expr.expr_data.as_ref() {
    ExprData::ExprLambda { params: lam_params, body: lam_body, semantics: _, .. } => {
        {
    let span = lambda_expr.span.clone();
    let callable_params = callable_type.params.clone();
    let param_types = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in lam_params.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __mapped_3.push(match callable_params.clone().get((__elem_4.0.clone()) as usize).cloned() {
    Some(cp) => {
        cp.type_expr.clone()
    }
    None => {
        leaf_node("Dynamic")
    }
});
    }
    Rc::new(__mapped_3)
};
    let typed_scope = {
    let mut __acc_8 = scope.clone();
    for __elem_9 in ({
    let mut __enumerated_5 = Vec::new();
    for (__idx_6, __elem_7) in lam_params.clone().iter().enumerate() {
        __enumerated_5.push((__idx_6 as i64, __elem_7.clone()));
    }
    Rc::new(__enumerated_5)
}).iter().cloned() {
        __acc_8 = {
    let pt = match callable_params.clone().get((__elem_9.0.clone()) as usize).cloned() {
    Some(cp) => {
        cp.type_expr.clone()
    }
    None => {
        leaf_node("Dynamic")
    }
};
    extend_scope(__acc_8.clone(), &__elem_9.1, pt.clone())
};
    }
    __acc_8
};
    let body_result = infer_expr(lam_body.clone(), typed_scope.clone());
    let body_typed = body_result.typed.clone();
    let typed_lam = make_expr_node(Rc::new(ExprData::ExprLambda { params: lam_params.clone(), body: body_typed.clone(), semantics: Some(lambda_semantics_from_param_types(param_types.clone())) }), Some(Rc::new(InferredNode::Resolved { node: rt_type(body_typed.clone()) })), span);
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: arg_name, value: typed_lam.clone() }), diagnostics: body_result.diagnostics.clone() })
}
    }
    _ => {
        {
    let ar = infer_expr(lambda_expr.clone(), scope.clone());
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: arg_name, value: ar.typed.clone() }), diagnostics: ar.diagnostics.clone() })
}
    }
}
    })
}

pub fn refine_collection_result_type(method_name: &str, typed_args: Rc<Vec<Rc<NamedArg>>>, receiver_type: Rc<Node>, fallback: Rc<Node>) -> Rc<Node> {
    let receiver_type_name = receiver_type.name.clone();
    let receiver_is_map = node_is_map(receiver_type.clone());
    if method_name == "map" {
    match {
    let mut __found_2 = None;
    for __elem_3 in typed_args.iter().cloned() {
        if is_lambda_expr(__elem_3.value.clone()) {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(lambda_arg) => {
        {
    let lambda_ret = rt_type(lambda_arg.value.clone());
    container_node(&receiver_type_name, lambda_ret.clone())
}
    }
    None => {
        fallback.clone()
    }
}
} else {
    if method_name == "flat_map" {
    match {
    let mut __found_6 = None;
    for __elem_7 in typed_args.iter().cloned() {
        if is_lambda_expr(__elem_7.value.clone()) {
    __found_6 = Some(__elem_7);
    break;
};
    }
    __found_6
} {
    Some(lambda_arg) => {
        match rt_node(lambda_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        fallback.clone()
    }
    NodeType::Untyped => {
        fallback.clone()
    }
}
    }
    None => {
        fallback.clone()
    }
}
} else {
    if method_name == "fold" {
    match {
    let mut __found_10 = None;
    for __elem_11 in typed_args.iter().cloned() {
        if is_lambda_expr(__elem_11.value.clone()) {
    __found_10 = Some(__elem_11);
    break;
};
    }
    __found_10
} {
    Some(lambda_arg) => {
        match rt_node(lambda_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        fallback.clone()
    }
    NodeType::Untyped => {
        fallback.clone()
    }
}
    }
    None => {
        fallback.clone()
    }
}
} else {
    if ((method_name == "list_push") && (({
    let __len_18 = typed_args.clone().len();
    __len_18 as i64
}) >= 1_i64)) && (({
    let __len_19 = receiver_type.children.clone().len();
    __len_19 as i64
}) == 0_i64) {
    match typed_args.clone().first().cloned() {
    Some(item_arg) => {
        match rt_node(item_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        container_node("List", rt.clone())
    }
    NodeType::InferError { message: _, span: _, .. } => {
        fallback.clone()
    }
    NodeType::Untyped => {
        fallback.clone()
    }
}
    }
    None => {
        fallback.clone()
    }
}
} else {
    if (((method_name == "map_insert") && (({
    let __len_16 = receiver_type.children.clone().len();
    __len_16 as i64
}) == 0_i64)) && receiver_is_map.clone()) && (({
    let __len_17 = typed_args.clone().len();
    __len_17 as i64
}) >= 2_i64) {
    let key_type = match typed_args.clone().first().cloned() {
    Some(key_arg) => {
        match rt_node(key_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        fallback.clone()
    }
    NodeType::Untyped => {
        fallback.clone()
    }
}
    }
    None => {
        fallback.clone()
    }
};
    match typed_args.clone().get((1_i64) as usize).cloned() {
    Some(val_arg) => {
        map_node(key_type.clone(), match rt_node(val_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        fallback.clone()
    }
    NodeType::Untyped => {
        fallback.clone()
    }
})
    }
    None => {
        fallback.clone()
    }
}
} else {
    if ((method_name == "map_merge") && receiver_is_map.clone()) && (({
    let __len_15 = typed_args.clone().len();
    __len_15 as i64
}) >= 1_i64) {
    let overlay_type = match typed_args.clone().first().cloned() {
    Some(overlay_arg) => {
        match rt_node(overlay_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        receiver_type.clone()
    }
    NodeType::Untyped => {
        receiver_type.clone()
    }
}
    }
    None => {
        receiver_type.clone()
    }
};
    if (({
    let __len_13 = receiver_type.children.clone().len();
    __len_13 as i64
}) == 0_i64) && (({
    let __len_14 = overlay_type.children.clone().len();
    __len_14 as i64
}) > 0_i64) {
    overlay_type.clone()
} else {
    if ({
    let __len_12 = receiver_type.children.clone().len();
    __len_12 as i64
}) > 0_i64 {
    receiver_type.clone()
} else {
    fallback.clone()
}
}
} else {
    fallback.clone()
}
}
}
}
}
}
}

pub fn infer_arg_with_element_type(arg: Rc<NamedArg>, element_type: Rc<Node>, scope: Rc<InferScope>) -> Rc<ArgInferResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if is_lambda_expr(arg.value.clone()) {
    let result = infer_lambda_with_element_type(arg.value.clone(), element_type.clone(), scope.clone());
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: arg.name.clone(), value: result.typed.clone() }), diagnostics: result.diagnostics.clone() })
} else {
    let ar = infer_expr(arg.value.clone(), scope.clone());
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: arg.name.clone(), value: ar.typed.clone() }), diagnostics: ar.diagnostics.clone() })
}
    })
}

pub fn infer_expr(texpr: Rc<Node>, scope: Rc<InferScope>) -> Rc<InferResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: lit, .. } => {
        {
    let span = texpr.span.clone();
    ok_infer(make_expr_node(Rc::new(ExprData::ExprLiteral { value: lit.clone() }), Some(Rc::new(InferredNode::Resolved { node: infer_literal_node(lit.clone()) })), span.clone()))
}
    }
    ExprData::ExprError { kind, message, .. } => {
        {
    let span = texpr.span.clone();
    Rc::new(InferResult { typed: make_expr_error_node(kind.clone(), &message, span.clone()), diagnostics: Rc::new(Vec::new()) })
}
    }
    ExprData::ExprVar { name, binding_kind: _, .. } => {
        {
    let span = texpr.span.clone();
    match scope.locals.clone().get(&name.clone()).cloned() {
    Some(binding) => {
        {
    let binding_kind = infer_var_binding_kind(scope.clone(), &name);
    ok_infer(make_expr_node(Rc::new(ExprData::ExprVar { name: name.clone(), binding_kind: Some(binding_kind.clone()) }), Some(Rc::new(InferredNode::Resolved { node: binding.resolved.clone() })), span.clone()))
}
    }
    None => {
        match lookup_func_sig(scope.func_env.clone(), &name) {
    Some(fsig) => {
        ok_infer(make_expr_node(Rc::new(ExprData::ExprVar { name: name.clone(), binding_kind: Some(Rc::new(VarBindingKind::FunctionValueBinding)) }), Some(Rc::new(InferredNode::Resolved { node: callable_node(fsig.params.clone(), fsig.return_type.clone()) })), span.clone()))
    }
    None => {
        {
    let err_texpr = make_expr_node(Rc::new(ExprData::ExprVar { name: name.clone(), binding_kind: None }), Some(Rc::new(InferredNode::Resolved { node: error_type_node() })), span.clone());
    Rc::new(InferResult { typed: err_texpr.clone(), diagnostics: Rc::new(vec!(inference_error(&v2_rt::concat(v2_rt::concat("undefined variable '".to_string(), name.clone()), "'".to_string()), span.clone(), &scope.module_name))) })
}
    }
}
    }
}
}
    }
    ExprData::ExprFieldAccess { base: base_expr, field: field_name, summary: _, .. } => {
        {
    let span = texpr.span.clone();
    let base_result = infer_expr(base_expr.clone(), scope.clone());
    let base_typed = base_result.typed.clone();
    let base_diags = base_result.diagnostics.clone();
    let base_rt = match rt_node(base_typed.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        error_type_node()
    }
    NodeType::Untyped => {
        leaf_node("Unit")
    }
};
    let resolved_base = resolve_scrutinee_type_node(scope.type_env.clone(), base_rt.clone());
    if resolved_base.name.clone() == "Error" {
    Rc::new(InferResult { typed: make_expr_error_node(ExprErrorKind::SemanticExprError, "error type cascade", span.clone()), diagnostics: base_diags.clone() })
} else {
    match lookup_field_type_node(resolved_base.clone(), &field_name) {
    Some(field_type) => {
        {
    let field_summary = field_summary_for_type(base_rt.clone(), scope.type_env.clone(), &field_name);
    let fa_texpr = make_expr_node(Rc::new(ExprData::ExprFieldAccess { base: base_typed.clone(), field: field_name.clone(), summary: field_summary.clone() }), Some(Rc::new(InferredNode::Resolved { node: field_type.clone() })), span.clone());
    Rc::new(InferResult { typed: fa_texpr.clone(), diagnostics: base_diags.clone() })
}
    }
    None => {
        if resolved_base.name.clone() == "Dynamic" {
    let fa_texpr = make_expr_node(Rc::new(ExprData::ExprFieldAccess { base: base_typed.clone(), field: field_name.clone(), summary: Some(Rc::new(FieldSummary { access_style: FieldAccessStyle::StoredField, value_shape: FieldValueShape::PlainValue })) }), Some(Rc::new(InferredNode::Resolved { node: leaf_node("Dynamic") })), span.clone());
    Rc::new(InferResult { typed: fa_texpr.clone(), diagnostics: base_diags.clone() })
} else {
    match check_service_field_access_node(base_rt.clone(), &field_name, scope.service_registry.clone()) {
    Some(svc_type) => {
        {
    let fa_texpr = make_expr_node(Rc::new(ExprData::ExprFieldAccess { base: base_typed.clone(), field: field_name.clone(), summary: Some(Rc::new(FieldSummary { access_style: FieldAccessStyle::StoredField, value_shape: FieldValueShape::PlainValue })) }), Some(Rc::new(InferredNode::Resolved { node: svc_type.clone() })), span.clone());
    Rc::new(InferResult { typed: fa_texpr.clone(), diagnostics: base_diags.clone() })
}
    }
    None => {
        {
    let error_message = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("no field '".to_string(), field_name.clone()), "' on type '".to_string()), resolved_base.name.clone()), "'".to_string());
    let fa_texpr = make_expr_error_node(ExprErrorKind::SemanticExprError, &error_message, span.clone());
    Rc::new(InferResult { typed: fa_texpr.clone(), diagnostics: {
    let __rc_1 = base_diags;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(inference_error(&error_message, span.clone(), &scope.module_name));
    Rc::new(__appended_0)
} })
}
    }
}
}
    }
}
}
}
    }
    ExprData::ExprCall { func: func_name, args: call_args, call_semantics: _, .. } => {
        {
    let span = texpr.span.clone();
    let sig = lookup_func_sig(scope.func_env.clone(), &func_name);
    let sig_params = match sig.as_ref().map(|__rc| __rc.as_ref()) {
    Some(s) => {
        let s = Rc::new(s.clone());
        s.params.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let has_lambda = {
    let mut __any_2 = false;
    for __elem_3 in call_args.iter().cloned() {
        if is_lambda_expr(__elem_3.value.clone()) {
    __any_2 = true;
    break;
};
    }
    __any_2
};
    let call_method_args = { let __s = call_args.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
    let call_fold_info = extract_fold_init_info(&func_name, call_method_args.clone(), 2_i64, scope.clone());
    let call_fold_acc_type = match call_fold_info.as_ref().map(|__rc| __rc.as_ref()) {
    Some(fi) => {
        let fi = Rc::new(fi.clone());
        match rt_node(fi.typed_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        error_type_node()
    }
    NodeType::Untyped => {
        error_type_node()
    }
}
    }
    None => {
        error_type_node()
    }
};
    let arg_infer_results = if (has_lambda.clone() && (({
    let __len_9 = call_args.clone().len();
    __len_9 as i64
}) >= 2_i64)) && (sig.clone().is_none()) {
    match call_args.clone().first().cloned() {
    Some(first_arg) => {
        {
    let first_result = infer_expr(first_arg.value.clone(), scope.clone());
    let first_type = rt_type(first_result.typed.clone());
    let elem_type = for_each_element_type_node(first_type.clone());
    let remaining_results = infer_method_args_with_fold(&func_name, call_method_args.clone(), call_fold_info.clone(), call_fold_acc_type.clone(), elem_type.clone(), scope.clone());
    v2_rt::concat(Rc::new(vec!(Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: first_arg.name.clone(), value: first_result.typed.clone() }), diagnostics: first_result.diagnostics.clone() }))), remaining_results.clone())
}
    }
    None => {
        Rc::new(Vec::new())
    }
}
} else {
    {
    let mut __mapped_7 = Vec::new();
    for __elem_8 in ({
    let mut __enumerated_4 = Vec::new();
    for (__idx_5, __elem_6) in call_args.clone().iter().enumerate() {
        __enumerated_4.push((__idx_5 as i64, __elem_6.clone()));
    }
    Rc::new(__enumerated_4)
}).iter().cloned() {
        __mapped_7.push({
    let a = __elem_8.1.clone();
    let formal_param_type = match sig_params.clone().get((__elem_8.0.clone()) as usize).cloned() {
    Some(p) => {
        p.type_expr.clone()
    }
    None => {
        leaf_node("Dynamic")
    }
};
    let is_callable_formal = formal_param_type.name.clone() == "Callable";
    if is_lambda_expr(a.value.clone()) && is_callable_formal.clone() {
    infer_lambda_with_callable_type(a.value.clone(), formal_param_type.clone(), a.name.clone(), scope.clone())
} else {
    let ar = infer_expr(a.value.clone(), scope.clone());
    Rc::new(ArgInferResult { typed_arg: Rc::new(NamedArg { name: a.name.clone(), value: ar.typed.clone() }), diagnostics: ar.diagnostics.clone() })
}
});
    }
    Rc::new(__mapped_7)
}
};
    let typed_args = {
    let mut __mapped_10 = Vec::new();
    for __elem_11 in arg_infer_results.iter().cloned() {
        __mapped_10.push(__elem_11.typed_arg.clone());
    }
    Rc::new(__mapped_10)
};
    let arg_diags = {
    let mut __flat_mapped_12 = Vec::new();
    for __elem_13 in arg_infer_results.iter().cloned() {
        __flat_mapped_12.extend(__elem_13.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_12)
};
    if sig.clone().is_some() {
    let resolved_type = match sig.as_ref().map(|__rc| __rc.as_ref()) {
    Some(s) => {
        let s = Rc::new(s.clone());
        s.return_type.clone()
    }
    None => {
        error_type_node()
    }
};
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprCall { func: func_name.clone(), args: typed_args.clone(), call_semantics: Some(CallSemantics::PlainCallSemantics) }), Some(Rc::new(InferredNode::Resolved { node: resolved_type.clone() })), span.clone()), diagnostics: arg_diags.clone() })
} else {
    let first_arg_type = match typed_args.clone().first().cloned() {
    Some(ta) => {
        rt_type(ta.value.clone())
    }
    None => {
        leaf_node("Unit")
    }
};
    let method_receiver = match typed_args.clone().first().cloned() {
    Some(ta) => {
        ta.value.clone()
    }
    None => {
        internal_expr_error_node("method bridge missing receiver", span.clone())
    }
};
    let method_resolution = resolve_known_method_node(method_receiver.clone(), first_arg_type.clone(), &func_name, if call_fold_info.clone().is_some() {
    Some(call_fold_acc_type.clone())
} else {
    None
}, scope.service_registry.clone());
    let is_known_method = method_resolution.result_type.clone().is_some();
    if is_known_method.clone() && (({
    let __len_14 = typed_args.clone().len();
    __len_14 as i64
}) > 0_i64) {
    let receiver = method_receiver.clone();
    let remaining = { let __s = typed_args.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
    let base_result_type = match method_resolution.result_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(mt) => {
        let mt = Rc::new(mt.clone());
        mt.clone()
    }
    None => {
        error_type_node()
    }
};
    let bridge_result_type = refine_collection_result_type(&func_name, remaining.clone(), first_arg_type.clone(), base_result_type.clone());
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprMethodCall { receiver: receiver.clone(), method: func_name.clone(), args: remaining.clone(), method_semantics: method_resolution.semantics.clone() }), Some(Rc::new(InferredNode::Resolved { node: bridge_result_type.clone() })), span.clone()), diagnostics: arg_diags.clone() })
} else {
    if func_name.clone() == "empty_map" {
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprCall { func: func_name.clone(), args: typed_args.clone(), call_semantics: Some(CallSemantics::PlainCallSemantics) }), Some(Rc::new(InferredNode::Resolved { node: bare_map_node() })), span.clone()), diagnostics: arg_diags.clone() })
} else {
    if infer_builtin_call_type(&func_name).is_some() {
    let bt = if (func_name.clone() == "lookup") || (func_name.clone() == "map_get") {
    match typed_args.clone().first().cloned() {
    Some(receiver_arg) => {
        match rt_node(receiver_arg.value.clone()).as_ref() {
    NodeType::Typed { node: receiver_type, .. } => {
        match map_value_type_in_env(receiver_type.clone(), scope.type_env.clone()) {
    Some(value_type) => {
        with_optional_cardinality(value_type.clone())
    }
    None => {
        resolve_builtin_call_type(&func_name)
    }
}
    }
    _ => {
        resolve_builtin_call_type(&func_name)
    }
}
    }
    None => {
        resolve_builtin_call_type(&func_name)
    }
}
} else {
    resolve_builtin_call_type(&func_name)
};
    let call_semantics = if func_name.clone() == "lookup" {
    Some(CallSemantics::LookupCallSemantics)
} else {
    Some(CallSemantics::PlainCallSemantics)
};
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprCall { func: func_name.clone(), args: typed_args.clone(), call_semantics: call_semantics.clone() }), Some(Rc::new(InferredNode::Resolved { node: bt.clone() })), span.clone()), diagnostics: arg_diags.clone() })
} else {
    let callable_local = match scope.locals.clone().get(&func_name.clone()).cloned() {
    Some(binding) => {
        if binding.resolved.name.clone() == "Callable" {
    Some(binding.resolved.clone())
} else {
    None
}
    }
    None => {
        None
    }
};
    if callable_local.clone().is_some() {
    let callable_type = match callable_local.clone() {
    Some(ct) => {
        ct.clone()
    }
    None => {
        error_type_node()
    }
};
    let resolved_type = callable_return_type(callable_type.clone());
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprCall { func: func_name.clone(), args: typed_args.clone(), call_semantics: Some(CallSemantics::PlainCallSemantics) }), Some(Rc::new(InferredNode::Resolved { node: resolved_type.clone() })), span.clone()), diagnostics: arg_diags.clone() })
} else {
    let type_match = lookup_type(scope.type_env.clone(), &func_name);
    let resolved_type = match type_match.as_ref().map(|__rc| __rc.as_ref()) {
    Some(tn) => {
        let tn = Rc::new(tn.clone());
        tn.clone()
    }
    None => {
        error_type_node()
    }
};
    let call_diags = match type_match.as_ref().map(|__rc| __rc.as_ref()) {
    Some(_) => {
        Rc::new(Vec::new())
    }
    None => {
        Rc::new(vec!(inference_error(&v2_rt::concat(v2_rt::concat("function '".to_string(), func_name.clone()), "' not found in scope".to_string()), span.clone(), &scope.module_name)))
    }
};
    Rc::new(InferResult { typed: make_expr_node(Rc::new(ExprData::ExprCall { func: func_name.clone(), args: typed_args.clone(), call_semantics: Some(CallSemantics::PlainCallSemantics) }), Some(Rc::new(InferredNode::Resolved { node: resolved_type.clone() })), span.clone()), diagnostics: v2_rt::concat(arg_diags.clone(), call_diags.clone()) })
}
}
}
}
}
}
    }
    ExprData::ExprMethodCall { receiver: recv, method: method_name, args: mc_args, method_semantics: _, .. } => {
        {
    let span = texpr.span.clone();
    let recv_result = infer_expr(recv.clone(), scope.clone());
    let recv_typed = recv_result.typed.clone();
    let recv_diags = recv_result.diagnostics.clone();
    let recv_rt = rt_type(recv_typed.clone());
    let recv_elem_type = for_each_element_type_node(recv_rt.clone());
    let fold_info = extract_fold_init_info(&method_name, mc_args.clone(), 2_i64, scope.clone());
    let fold_acc_type = match fold_info.as_ref().map(|__rc| __rc.as_ref()) {
    Some(fi) => {
        let fi = Rc::new(fi.clone());
        match rt_node(fi.typed_arg.value.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        error_type_node()
    }
    NodeType::Untyped => {
        error_type_node()
    }
}
    }
    None => {
        error_type_node()
    }
};
    let mc_arg_infer_results = infer_method_args_with_fold(&method_name, mc_args.clone(), fold_info.clone(), fold_acc_type.clone(), recv_elem_type.clone(), scope.clone());
    let typed_mc_args = {
    let mut __mapped_15 = Vec::new();
    for __elem_16 in mc_arg_infer_results.iter().cloned() {
        __mapped_15.push(__elem_16.typed_arg.clone());
    }
    Rc::new(__mapped_15)
};
    let mc_arg_diags = {
    let mut __flat_mapped_17 = Vec::new();
    for __elem_18 in mc_arg_infer_results.iter().cloned() {
        __flat_mapped_17.extend(__elem_18.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_17)
};
    let method_resolution = resolve_known_method_node(recv_typed.clone(), recv_rt.clone(), &method_name, if fold_info.clone().is_some() {
    Some(fold_acc_type.clone())
} else {
    None
}, scope.service_registry.clone());
    let base_result_type = match method_resolution.result_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(rt) => {
        let rt = Rc::new(rt.clone());
        rt.clone()
    }
    None => {
        recv_rt.clone()
    }
};
    let result_type = refine_collection_result_type(&method_name, typed_mc_args.clone(), recv_rt.clone(), base_result_type.clone());
    let method_semantics = if method_resolution.semantics.clone().is_some() {
    method_resolution.semantics.clone()
} else {
    Some(Rc::new(MethodSemantics::PlainMethodSemantics))
};
    let mc_texpr = make_expr_node(Rc::new(ExprData::ExprMethodCall { receiver: recv_typed.clone(), method: method_name.clone(), args: typed_mc_args.clone(), method_semantics: method_semantics.clone() }), Some(Rc::new(InferredNode::Resolved { node: result_type.clone() })), span.clone());
    Rc::new(InferResult { typed: mc_texpr.clone(), diagnostics: v2_rt::concat(recv_diags.clone(), mc_arg_diags.clone()) })
}
    }
    ExprData::ExprMatch { scrutinee: scrut, arms, .. } => {
        {
    let span = texpr.span.clone();
    let scrut_result = infer_expr(scrut.clone(), scope.clone());
    let scrut_typed = scrut_result.typed.clone();
    let scrut_diags = scrut_result.diagnostics.clone();
    let scrut_rt = rt_type(scrut_typed.clone());
    let scrut_subject = pattern_subject_from_node_type(rt_node(scrut_typed.clone()));
    let arm_infer_results = {
    let mut __mapped_19 = Vec::new();
    for __elem_20 in arms.iter().cloned() {
        __mapped_19.push({
    let typed_pattern = annotate_pattern_parent_enums(__elem_20.pattern.clone(), scrut_subject.clone(), scope.clone());
    let pattern_result = extend_scope_with_pattern_node(scope.clone(), typed_pattern.clone(), scrut_subject.clone());
    let arm_scope = pattern_result.scope.clone();
    let pattern_diags = pattern_result.diagnostics.clone();
    let guard_result = if __elem_20.guard.clone().is_some() {
    Some(infer_expr(__elem_20.guard.clone().unwrap(), arm_scope.clone()))
} else {
    None
};
    let body_result = infer_expr(__elem_20.body.clone(), arm_scope.clone());
    let body_typed = body_result.typed.clone();
    let body_diags = body_result.diagnostics.clone();
    let guard_unwrapped = match guard_result.clone() {
    Some(gr) => {
        gr.clone()
    }
    None => {
        Rc::new(InferResult { typed: __elem_20.body.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let guard_diags = if guard_result.clone().is_some() {
    guard_unwrapped.diagnostics.clone()
} else {
    Rc::new(Vec::new())
};
    Rc::new(ArmInferResult { typed_arm: Rc::new(MatchArm { pattern: typed_pattern.clone(), guard: if guard_result.clone().is_some() {
    Some(guard_unwrapped.typed.clone())
} else {
    None
}, body: body_typed.clone() }), diagnostics: v2_rt::concat(pattern_diags.clone(), v2_rt::concat(guard_diags.clone(), body_diags.clone())), body_type: rt_type(body_typed.clone()) })
});
    }
    Rc::new(__mapped_19)
};
    let typed_arms = {
    let mut __mapped_21 = Vec::new();
    for __elem_22 in arm_infer_results.iter().cloned() {
        __mapped_21.push(__elem_22.typed_arm.clone());
    }
    Rc::new(__mapped_21)
};
    let arm_diags = {
    let mut __flat_mapped_23 = Vec::new();
    for __elem_24 in arm_infer_results.iter().cloned() {
        __flat_mapped_23.extend(__elem_24.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_23)
};
    let result_type = match arm_infer_results.clone().first().cloned() {
    Some(ar) => {
        ar.body_type.clone()
    }
    None => {
        scrut_rt.clone()
    }
};
    let empty_arms_diags = if ({
    let __len_25 = arm_infer_results.clone().len();
    __len_25 as i64
}) == 0_i64 {
    Rc::new(vec!(inference_error("match expression has no arms", span.clone(), &scope.module_name)))
} else {
    Rc::new(Vec::new())
};
    let exhaustiveness_diags = match resolve_pattern_subject(scope.clone(), scrut_subject.clone()).as_ref() {
    PatternSubject::PatternResolved { node: resolved_scrutinee, .. } => {
        check_match_exhaustiveness(resolved_scrutinee.clone(), typed_arms.clone(), scope.type_env.clone(), span.clone(), &scope.module_name)
    }
    PatternSubject::PatternDynamic { span: _, .. } => {
        Rc::new(Vec::new())
    }
    PatternSubject::PatternLookupBlocked => {
        Rc::new(Vec::new())
    }
};
    let match_texpr = make_expr_node(Rc::new(ExprData::ExprMatch { scrutinee: scrut_typed.clone(), arms: typed_arms.clone() }), Some(Rc::new(InferredNode::Resolved { node: result_type.clone() })), span.clone());
    Rc::new(InferResult { typed: match_texpr.clone(), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(scrut_diags.clone(), arm_diags.clone()), empty_arms_diags.clone()), exhaustiveness_diags.clone()) })
}
    }
    ExprData::ExprIf { condition: cond, then_branch: then_expr, else_branch: else_expr, .. } => {
        {
    let span = texpr.span.clone();
    let cond_result = infer_expr(cond.clone(), scope.clone());
    let cond_typed = cond_result.typed.clone();
    let cond_diags = cond_result.diagnostics.clone();
    let then_result = infer_expr(then_expr.clone(), scope.clone());
    let then_typed = then_result.typed.clone();
    let then_diags = then_result.diagnostics.clone();
    match else_expr.as_ref().map(|__rc| __rc.as_ref()) {
    Some(else_branch) => {
        let else_branch = Rc::new(else_branch.clone());
        {
    let else_result = infer_expr(else_branch.clone(), scope.clone());
    let else_typed = else_result.typed.clone();
    let else_diags = else_result.diagnostics.clone();
    let then_rt = rt_type(then_typed.clone());
    let else_rt = rt_type(else_typed.clone());
    let branch_diags = if node_type_compatible(then_rt.clone(), else_rt.clone()) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(inference_error(&v2_rt::concat(v2_rt::concat(v2_rt::concat("if branches resolve to incompatible types: ".to_string(), node_type_shape(then_rt.clone())), " vs ".to_string()), node_type_shape(else_rt.clone())), span.clone(), &scope.module_name)))
};
    let resolved_type = prefer_specific_type(then_rt.clone(), else_rt.clone());
    let if_texpr = make_expr_node(Rc::new(ExprData::ExprIf { condition: cond_typed.clone(), then_branch: then_typed.clone(), else_branch: Some(else_typed.clone()) }), Some(Rc::new(InferredNode::Resolved { node: resolved_type.clone() })), span.clone());
    Rc::new(InferResult { typed: if_texpr.clone(), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(cond_diags.clone(), then_diags.clone()), else_diags.clone()), branch_diags.clone()) })
}
    }
    None => {
        {
    let if_texpr2 = make_expr_node(Rc::new(ExprData::ExprIf { condition: cond_typed.clone(), then_branch: then_typed.clone(), else_branch: None }), Some(Rc::new(InferredNode::Resolved { node: leaf_node("Unit") })), span.clone());
    Rc::new(InferResult { typed: if_texpr2.clone(), diagnostics: v2_rt::concat(cond_diags.clone(), then_diags.clone()) })
}
    }
}
}
    }
    ExprData::ExprLet { name: let_name, value: let_value, body: let_body, .. } => {
        {
    let span = texpr.span.clone();
    let val_result = infer_expr(let_value.clone(), scope.clone());
    let val_typed = val_result.typed.clone();
    let val_diags = val_result.diagnostics.clone();
    let val_type = rt_type(val_typed.clone());
    if let_body.clone().is_none() {
    let let_texpr = make_expr_node(Rc::new(ExprData::ExprLet { name: let_name.clone(), value: val_typed.clone(), body: None }), Some(Rc::new(InferredNode::Resolved { node: val_type.clone() })), span.clone());
    Rc::new(InferResult { typed: let_texpr.clone(), diagnostics: val_diags.clone() })
} else {
    let extended = extend_scope(scope.clone(), &let_name, val_type.clone());
    let body_result = infer_expr(let_body.clone().unwrap(), extended.clone());
    let body_typed = body_result.typed.clone();
    let body_diags = body_result.diagnostics.clone();
    let let_texpr2 = make_expr_node(Rc::new(ExprData::ExprLet { name: let_name.clone(), value: val_typed.clone(), body: Some(body_typed.clone()) }), Some(Rc::new(InferredNode::Resolved { node: rt_type(body_typed.clone()) })), span.clone());
    Rc::new(InferResult { typed: let_texpr2.clone(), diagnostics: v2_rt::concat(val_diags.clone(), body_diags.clone()) })
}
}
    }
    ExprData::ExprRecordLit { type_name: tn, fields: field_inits, parent_enum: _, .. } => {
        {
    let span = texpr.span.clone();
    infer_record_lit(tn.clone(), field_inits.clone(), span.clone(), scope.clone())
}
    }
    ExprData::ExprListLit { elements, .. } => {
        {
    let span = texpr.span.clone();
    let elem_results = {
    let mut __mapped_26 = Vec::new();
    for __elem_27 in elements.iter().cloned() {
        __mapped_26.push(infer_expr(__elem_27.clone(), scope.clone()));
    }
    Rc::new(__mapped_26)
};
    let typed_elements = {
    let mut __mapped_28 = Vec::new();
    for __elem_29 in elem_results.iter().cloned() {
        __mapped_28.push(__elem_29.typed.clone());
    }
    Rc::new(__mapped_28)
};
    let elem_diags = {
    let mut __flat_mapped_30 = Vec::new();
    for __elem_31 in elem_results.iter().cloned() {
        __flat_mapped_30.extend(__elem_31.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_30)
};
    let elem_type_node = if ({
    let __len_32 = elem_results.clone().len();
    __len_32 as i64
}) > 0_i64 {
    match elem_results.clone().first().cloned() {
    Some(r) => {
        rt_type(r.typed.clone())
    }
    None => {
        leaf_node("Unit")
    }
}
} else {
    leaf_node("Unit")
};
    let ll_texpr = make_expr_node(Rc::new(ExprData::ExprListLit { elements: typed_elements.clone() }), Some(Rc::new(InferredNode::Resolved { node: container_node("List", elem_type_node.clone()) })), span.clone());
    Rc::new(InferResult { typed: ll_texpr.clone(), diagnostics: elem_diags.clone() })
}
    }
    ExprData::ExprBinOp { op, left: left_expr, right: right_expr, .. } => {
        {
    let span = texpr.span.clone();
    let left_result = infer_expr(left_expr.clone(), scope.clone());
    let left_typed = left_result.typed.clone();
    let left_diags = left_result.diagnostics.clone();
    let right_result = infer_expr(right_expr.clone(), scope.clone());
    let right_typed = right_result.typed.clone();
    let right_diags = right_result.diagnostics.clone();
    let result_type = infer_binop_type_node(op.clone(), rt_type(left_typed.clone()));
    let bo_texpr = make_expr_node(Rc::new(ExprData::ExprBinOp { op: op.clone(), left: left_typed.clone(), right: right_typed.clone() }), Some(Rc::new(InferredNode::Resolved { node: result_type.clone() })), span.clone());
    Rc::new(InferResult { typed: bo_texpr.clone(), diagnostics: v2_rt::concat(left_diags.clone(), right_diags.clone()) })
}
    }
    ExprData::ExprUnaryOp { op, operand: operand_expr, .. } => {
        {
    let span = texpr.span.clone();
    let operand_result = infer_expr(operand_expr.clone(), scope.clone());
    let operand_typed = operand_result.typed.clone();
    let operand_diags = operand_result.diagnostics.clone();
    let result_type = match op.clone() {
    UnaryOpKind::Not => {
        leaf_node("Bool")
    }
    UnaryOpKind::Neg => {
        rt_type(operand_typed.clone())
    }
};
    let uo_texpr = make_expr_node(Rc::new(ExprData::ExprUnaryOp { op: op.clone(), operand: operand_typed.clone() }), Some(Rc::new(InferredNode::Resolved { node: result_type.clone() })), span.clone());
    Rc::new(InferResult { typed: uo_texpr.clone(), diagnostics: operand_diags.clone() })
}
    }
    ExprData::ExprLambda { params: lam_params, body: lam_body, semantics: _, .. } => {
        {
    let span = texpr.span.clone();
    let lam_scope = extend_scope_with_params(scope.clone(), lam_params.clone());
    let body_result = infer_expr(lam_body.clone(), lam_scope.clone());
    let body_typed = body_result.typed.clone();
    let body_diags = body_result.diagnostics.clone();
    let lam_texpr = make_expr_node(Rc::new(ExprData::ExprLambda { params: lam_params.clone(), body: body_typed.clone(), semantics: Some(lambda_semantics_from_param_types(lambda_param_types_from_scope(lam_scope.clone(), lam_params.clone()))) }), Some(Rc::new(InferredNode::Resolved { node: rt_type(body_typed.clone()) })), span.clone());
    Rc::new(InferResult { typed: lam_texpr.clone(), diagnostics: body_diags.clone() })
}
    }
    ExprData::ExprStringInterp { parts: str_parts, .. } => {
        {
    let span = texpr.span.clone();
    let part_results = {
    let mut __mapped_33 = Vec::new();
    for __elem_34 in str_parts.iter().cloned() {
        __mapped_33.push(match __elem_34.as_ref() {
    StringPart::Text { value: v, .. } => {
        Rc::new(StringPartInferResult { typed_part: Rc::new(StringPart::Text { value: v.clone() }), diagnostics: Rc::new(Vec::new()) })
    }
    StringPart::Interpolation { expr: e, .. } => {
        {
    let r = infer_expr(e.clone(), scope.clone());
    let r_typed = r.typed.clone();
    let r_diags = r.diagnostics.clone();
    Rc::new(StringPartInferResult { typed_part: Rc::new(StringPart::Interpolation { expr: r_typed.clone() }), diagnostics: r_diags.clone() })
}
    }
});
    }
    Rc::new(__mapped_33)
};
    let typed_parts = {
    let mut __mapped_35 = Vec::new();
    for __elem_36 in part_results.iter().cloned() {
        __mapped_35.push(__elem_36.typed_part.clone());
    }
    Rc::new(__mapped_35)
};
    let interp_diags = {
    let mut __flat_mapped_37 = Vec::new();
    for __elem_38 in part_results.iter().cloned() {
        __flat_mapped_37.extend(__elem_38.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_37)
};
    let si_texpr = make_expr_node(Rc::new(ExprData::ExprStringInterp { parts: typed_parts.clone() }), Some(Rc::new(InferredNode::Resolved { node: leaf_node("String") })), span.clone());
    Rc::new(InferResult { typed: si_texpr.clone(), diagnostics: interp_diags.clone() })
}
    }
    ExprData::ExprBlock { stmts, .. } => {
        {
    let span = texpr.span.clone();
    if ({
    let __len_41 = stmts.clone().len();
    __len_41 as i64
}) > 0_i64 {
    let state = infer_block_stmts(stmts.clone(), scope.clone(), Rc::new(Vec::new()), Rc::new(Vec::new()), leaf_node("Unit"));
    let blk_texpr = make_expr_node(Rc::new(ExprData::ExprBlock { stmts: state.typed_stmts.clone() }), Some(Rc::new(InferredNode::Resolved { node: state.last_type.clone() })), span.clone());
    Rc::new(InferResult { typed: blk_texpr.clone(), diagnostics: {
    let mut __flat_mapped_39 = Vec::new();
    for __elem_40 in state.diag_chunks.iter().cloned() {
        __flat_mapped_39.extend(__elem_40.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_39)
} })
} else {
    ok_infer(make_expr_node(Rc::new(ExprData::ExprBlock { stmts: Rc::new(Vec::new()) }), Some(Rc::new(InferredNode::Resolved { node: leaf_node("Unit") })), span.clone()))
}
}
    }
    ExprData::ExprCast { expr: cast_inner, target: target_type, .. } => {
        {
    let span = texpr.span.clone();
    let inner_result = infer_expr(cast_inner.clone(), scope.clone());
    let inner_typed = inner_result.typed.clone();
    let inner_diags = inner_result.diagnostics.clone();
    let cast_texpr = make_expr_node(Rc::new(ExprData::ExprCast { expr: inner_typed.clone(), target: target_type.clone() }), Some(Rc::new(InferredNode::Resolved { node: target_type.clone() })), span.clone());
    Rc::new(InferResult { typed: cast_texpr.clone(), diagnostics: inner_diags.clone() })
}
    }
    ExprData::ExprForEach { variable, collection: coll, body: body_expr, .. } => {
        {
    let span = texpr.span.clone();
    let coll_result = infer_expr(coll.clone(), scope.clone());
    let coll_typed = coll_result.typed.clone();
    let coll_diags = coll_result.diagnostics.clone();
    let elem_type_node = for_each_element_type_node(rt_type(coll_typed.clone()));
    let body_scope = extend_scope(scope.clone(), &variable, elem_type_node.clone());
    let body_result = infer_expr(body_expr.clone(), body_scope.clone());
    let body_typed = body_result.typed.clone();
    let body_diags = body_result.diagnostics.clone();
    let fe_texpr = make_expr_node(Rc::new(ExprData::ExprForEach { variable: variable.clone(), collection: coll_typed.clone(), body: body_typed.clone() }), Some(Rc::new(InferredNode::Resolved { node: rt_type(body_typed.clone()) })), span.clone());
    Rc::new(InferResult { typed: fe_texpr.clone(), diagnostics: v2_rt::concat(coll_diags.clone(), body_diags.clone()) })
}
    }
    ExprData::ExprIndex { base: base_expr, index: index_expr, .. } => {
        {
    let span = texpr.span.clone();
    let base_result = infer_expr(base_expr.clone(), scope.clone());
    let base_typed = base_result.typed.clone();
    let base_diags = base_result.diagnostics.clone();
    let index_result = infer_expr(index_expr.clone(), scope.clone());
    let index_typed = index_result.typed.clone();
    let index_diags = index_result.diagnostics.clone();
    let base_type_result = rt_node(base_typed.clone());
    let index_type_result = rt_node(index_typed.clone());
    let index_check = match base_type_result.as_ref() {
    NodeType::Typed { node: base_type, .. } => {
        match index_type_result.as_ref() {
    NodeType::Typed { node: index_type, .. } => {
        Some(check_index_access_node(base_type.clone(), index_type.clone(), span.clone(), &scope.module_name))
    }
    NodeType::InferError { message: _, span: _, .. } => {
        None
    }
    NodeType::Untyped => {
        None
    }
}
    }
    NodeType::InferError { message: _, span: _, .. } => {
        None
    }
    NodeType::Untyped => {
        None
    }
};
    let access_failure_source = match base_type_result.as_ref() {
    NodeType::Typed { node: _, .. } => {
        index_type_result.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        base_type_result.clone()
    }
    NodeType::Untyped => {
        base_type_result.clone()
    }
};
    let idx_return_type = match index_check.clone() {
    Some(checked) => {
        checked.return_type.clone()
    }
    None => {
        Some(inferred_from_node_type(access_failure_source.clone(), "invalid index access", span.clone()))
    }
};
    let index_access_diags = match index_check.clone() {
    Some(checked) => {
        checked.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let idx_texpr = make_expr_node(Rc::new(ExprData::ExprIndex { base: base_typed.clone(), index: index_typed.clone() }), idx_return_type.clone(), span.clone());
    Rc::new(InferResult { typed: idx_texpr.clone(), diagnostics: v2_rt::concat(v2_rt::concat(base_diags.clone(), index_diags.clone()), index_access_diags.clone()) })
}
    }
    ExprData::ExprSlice { base: base_expr, start: start_expr, end: end_expr, .. } => {
        {
    let span = texpr.span.clone();
    let base_result = infer_expr(base_expr.clone(), scope.clone());
    let base_typed = base_result.typed.clone();
    let base_diags = base_result.diagnostics.clone();
    let start_result = infer_expr(start_expr.clone(), scope.clone());
    let start_typed = start_result.typed.clone();
    let start_diags = start_result.diagnostics.clone();
    let end_result = infer_expr(end_expr.clone(), scope.clone());
    let end_typed = end_result.typed.clone();
    let end_diags = end_result.diagnostics.clone();
    let base_type_result = rt_node(base_typed.clone());
    let start_type_result = rt_node(start_typed.clone());
    let end_type_result = rt_node(end_typed.clone());
    let slice_check = match base_type_result.as_ref() {
    NodeType::Typed { node: base_type, .. } => {
        match start_type_result.as_ref() {
    NodeType::Typed { node: start_type, .. } => {
        match end_type_result.as_ref() {
    NodeType::Typed { node: end_type, .. } => {
        Some(check_slice_access_node(base_type.clone(), start_type.clone(), end_type.clone(), span.clone(), &scope.module_name))
    }
    NodeType::InferError { message: _, span: _, .. } => {
        None
    }
    NodeType::Untyped => {
        None
    }
}
    }
    NodeType::InferError { message: _, span: _, .. } => {
        None
    }
    NodeType::Untyped => {
        None
    }
}
    }
    NodeType::InferError { message: _, span: _, .. } => {
        None
    }
    NodeType::Untyped => {
        None
    }
};
    let access_failure_source = match base_type_result.as_ref() {
    NodeType::Typed { node: _, .. } => {
        match start_type_result.as_ref() {
    NodeType::Typed { node: _, .. } => {
        end_type_result.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        start_type_result.clone()
    }
    NodeType::Untyped => {
        start_type_result.clone()
    }
}
    }
    NodeType::InferError { message: _, span: _, .. } => {
        base_type_result.clone()
    }
    NodeType::Untyped => {
        base_type_result.clone()
    }
};
    let slc_return_type = match slice_check.clone() {
    Some(checked) => {
        checked.return_type.clone()
    }
    None => {
        Some(inferred_from_node_type(access_failure_source.clone(), "invalid slice access", span.clone()))
    }
};
    let slice_access_diags = match slice_check.clone() {
    Some(checked) => {
        checked.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let slc_texpr = make_expr_node(Rc::new(ExprData::ExprSlice { base: base_typed.clone(), start: start_typed.clone(), end: end_typed.clone() }), slc_return_type.clone(), span.clone());
    Rc::new(InferResult { typed: slc_texpr.clone(), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(base_diags.clone(), start_diags.clone()), end_diags.clone()), slice_access_diags.clone()) })
}
    }
    ExprData::ExprReturn { value: inner_expr, .. } => {
        {
    let span = texpr.span.clone();
    let inner_result = infer_expr(inner_expr.clone(), scope.clone());
    let ret_texpr = make_expr_node(Rc::new(ExprData::ExprReturn { value: inner_result.typed.clone() }), Some(Rc::new(InferredNode::Resolved { node: rt_type(inner_result.typed.clone()) })), span.clone());
    Rc::new(InferResult { typed: ret_texpr.clone(), diagnostics: inner_result.diagnostics.clone() })
}
    }
    _ => {
        {
    let span = texpr.span.clone();
    Rc::new(InferResult { typed: make_expr_error_node(ExprErrorKind::InternalExprError, "unhandled expression variant in infer_expr", span.clone()), diagnostics: Rc::new(vec!(inference_error("unhandled expression variant in infer_expr", span.clone(), &scope.module_name))) })
}
    }
}
    })
}

pub fn infer_record_lit(type_name: Option<String>, field_inits: Rc<Vec<Rc<FieldInit>>>, span: SourceSpan, scope: Rc<InferScope>) -> Rc<InferResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let fi_infer_results = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in field_inits.iter().cloned() {
        __mapped_0.push({
    let ar = infer_expr(__elem_1.value.clone(), scope.clone());
    let ar_typed = ar.typed.clone();
    let ar_diags = ar.diagnostics.clone();
    Rc::new(FieldInferResult { typed_field: Rc::new(FieldInit { name: __elem_1.name.clone(), value: ar_typed.clone() }), infer_result: ar.clone(), diagnostics: ar_diags.clone() })
});
    }
    Rc::new(__mapped_0)
};
        let typed_fields = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in fi_infer_results.iter().cloned() {
        __mapped_2.push(__elem_3.typed_field.clone());
    }
    Rc::new(__mapped_2)
};
        let fi_diags = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in fi_infer_results.iter().cloned() {
        __flat_mapped_4.extend(__elem_5.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
        if type_name.clone().is_none() {
    let child_nodes = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in fi_infer_results.iter().cloned() {
        __mapped_6.push(Rc::new(Node { name: __elem_7.typed_field.name.clone(), span: no_span(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: rt_type(__elem_7.infer_result.typed.clone()) })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }));
    }
    Rc::new(__mapped_6)
};
    let anon_node = Rc::new(Node { name: "".to_string(), span: no_span(), children: child_nodes.clone(), connective: Some(Connective::Conj), collection_kind: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let texpr = make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: None, fields: typed_fields.clone(), parent_enum: None }), Some(Rc::new(InferredNode::Resolved { node: anon_node.clone() })), span.clone());
    Rc::new(InferResult { typed: texpr.clone(), diagnostics: fi_diags.clone() })
} else {
    let type_lookup = lookup_type(scope.type_env.clone(), &type_name.clone().unwrap());
    let local_variant_parent = lookup_variant_parent_enum(scope.clone(), &type_name.clone().unwrap());
    let effective_lookup = match type_lookup.as_ref().map(|__rc| __rc.as_ref()) {
    Some(_) => {
        type_lookup.clone()
    }
    None => {
        {
    let local_lookup = lookup_in_scope(scope.locals.clone(), &type_name.clone().unwrap());
    match local_lookup.as_ref().map(|__rc| __rc.as_ref()) {
    Some(local_node) => {
        let local_node = Rc::new(local_node.clone());
        lookup_type(scope.type_env.clone(), &local_node.name)
    }
    None => {
        None
    }
}
}
    }
};
    let raw_resolved = match effective_lookup.clone() {
    Some(tn) => {
        tn.clone()
    }
    None => {
        error_type_node()
    }
};
    let expected_optional_parent = Some("Optional".to_string());
    let is_some_ctor = (type_name.clone().unwrap() == "Some") && (local_variant_parent.clone() == expected_optional_parent.clone());
    let resolved_node = if is_some_ctor.clone() {
    let val_field = {
    let mut __found_10 = None;
    for __elem_11 in fi_infer_results.iter().cloned() {
        if __elem_11.typed_field.name.clone() == "value" {
    __found_10 = Some(__elem_11);
    break;
};
    }
    __found_10
};
    match val_field.clone() {
    Some(val_fir) => {
        with_optional_cardinality(rt_type(val_fir.infer_result.typed.clone()))
    }
    None => {
        raw_resolved.clone()
    }
}
} else {
    raw_resolved.clone()
};
    let type_diags = match effective_lookup.clone() {
    Some(_) => {
        Rc::new(Vec::new())
    }
    None => {
        Rc::new(vec!(inference_error(&v2_rt::concat(v2_rt::concat("type '".to_string(), type_name.clone().unwrap()), "' not found in scope".to_string()), span.clone(), &scope.module_name)))
    }
};
    let texpr = make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: type_name.clone(), fields: typed_fields.clone(), parent_enum: local_variant_parent.clone() }), Some(Rc::new(InferredNode::Resolved { node: resolved_node.clone() })), span.clone());
    Rc::new(InferResult { typed: texpr.clone(), diagnostics: v2_rt::concat(fi_diags.clone(), type_diags.clone()) })
}
    })
}

pub fn infer_item(item: Rc<Node>, scope: Rc<InferScope>) -> Rc<TypedItemResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let typed_anno = item.type_annotation.clone();
        if node_has_structure(item.clone()) && (item.transport.clone().is_none()) {
    Rc::new(TypedItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in item.children.iter().cloned() {
        __mapped_0.push(infer_item(__elem_1.clone(), scope.clone()).item.clone());
    }
    Rc::new(__mapped_0)
}, params: item.params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: leaf_node("Unit") })), return_cardinality: item.return_cardinality.clone(), uses: item.uses.clone(), body: None, connective: item.connective.clone(), collection_kind: None, transport: item.transport.clone(), properties: item.properties.clone(), type_annotation: typed_anno.clone(), config: item.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: Rc::new(Vec::new()) })
} else {
    if (item.body.clone().is_some()) && (({
    let __len_11 = item.params.clone().len();
    __len_11 as i64
}) > 0_i64) {
    let fn_scope = build_params_scope(scope.clone(), item.params.clone());
    let body_result = infer_expr(item.body.clone().unwrap(), fn_scope.clone());
    let body_typed = body_result.typed.clone();
    let body_diags = body_result.diagnostics.clone();
    let resolved_ret = if item.return_type.clone().is_none() {
    rt_type(body_typed.clone())
} else {
    rt_type(item.clone())
};
    Rc::new(TypedItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in item.children.iter().cloned() {
        __mapped_2.push(infer_item(__elem_3.clone(), scope.clone()).item.clone());
    }
    Rc::new(__mapped_2)
}, params: item.params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: resolved_ret.clone() })), return_cardinality: item.return_cardinality.clone(), uses: item.uses.clone(), body: Some(body_typed.clone()), connective: None, collection_kind: None, transport: item.transport.clone(), properties: item.properties.clone(), type_annotation: typed_anno.clone(), config: item.config.clone(), is_self_recursive: expr_has_self_call(body_typed.clone(), &item.name), has_non_tail_self_call: expr_has_non_tail_self_call(body_typed.clone(), &item.name, true), expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: body_diags.clone() })
} else {
    if ((item.body.clone().is_some()) && (({
    let __len_10 = item.params.clone().len();
    __len_10 as i64
}) == 0_i64)) && (item.return_type.clone().is_some()) {
    let fn_scope = scope.clone();
    let body_result = infer_expr(item.body.clone().unwrap(), fn_scope.clone());
    let body_typed = body_result.typed.clone();
    let body_diags = body_result.diagnostics.clone();
    let resolved_ret = rt_type(item.clone());
    Rc::new(TypedItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: resolved_ret.clone() })), return_cardinality: item.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: Some(body_typed.clone()), connective: None, collection_kind: None, transport: item.transport.clone(), properties: item.properties.clone(), type_annotation: typed_anno.clone(), config: item.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: body_diags.clone() })
} else {
    if (item.body.clone().is_some()) && (({
    let __len_9 = item.params.clone().len();
    __len_9 as i64
}) == 0_i64) {
    let val_result = infer_expr(item.body.clone().unwrap(), scope.clone());
    let val_typed = val_result.typed.clone();
    let val_diags = val_result.diagnostics.clone();
    let resolved_ret = if item.type_annotation.clone().is_none() {
    rt_type(val_typed.clone())
} else {
    item.type_annotation.clone().unwrap()
};
    Rc::new(TypedItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: Rc::new(Vec::new()), params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: resolved_ret.clone() })), return_cardinality: item.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: Some(val_typed.clone()), connective: None, collection_kind: None, transport: item.transport.clone(), properties: item.properties.clone(), type_annotation: typed_anno.clone(), config: item.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: val_diags.clone() })
} else {
    if (({
    let __len_8 = item.params.clone().len();
    __len_8 as i64
}) > 0_i64) && (item.body.clone().is_none()) {
    let resolved_ret = if item.return_type.clone().is_none() {
    leaf_node("Unit")
} else {
    rt_type(item.clone())
};
    Rc::new(TypedItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in item.children.iter().cloned() {
        __mapped_4.push(infer_item(__elem_5.clone(), scope.clone()).item.clone());
    }
    Rc::new(__mapped_4)
}, params: item.params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: resolved_ret.clone() })), return_cardinality: item.return_cardinality.clone(), uses: item.uses.clone(), body: None, connective: None, collection_kind: None, transport: item.transport.clone(), properties: item.properties.clone(), type_annotation: typed_anno.clone(), config: item.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: Rc::new(Vec::new()) })
} else {
    let resolved_ret = if item.return_type.clone().is_none() {
    leaf_node("Unit")
} else {
    rt_type(item.clone())
};
    Rc::new(TypedItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in item.children.iter().cloned() {
        __mapped_6.push(infer_item(__elem_7.clone(), scope.clone()).item.clone());
    }
    Rc::new(__mapped_6)
}, params: item.params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: resolved_ret.clone() })), return_cardinality: item.return_cardinality.clone(), uses: item.uses.clone(), body: None, connective: item.connective.clone(), collection_kind: None, transport: item.transport.clone(), properties: item.properties.clone(), type_annotation: typed_anno.clone(), config: item.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: Rc::new(Vec::new()) })
}
}
}
}
}
    })
}

pub fn infer_items(items: Rc<Vec<Rc<Node>>>, scope: Rc<InferScope>) -> Rc<Vec<Rc<TypedItemResult>>> {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in items.iter().cloned() {
        __mapped_0.push(infer_item(__elem_1.clone(), scope.clone()));
    }
    Rc::new(__mapped_0)
}
}

pub fn build_type_env(module: Rc<ResolvedModule>, parent_index: Rc<HashMap<String, Rc<TypedModule>>>) -> Rc<BuildTypeEnvResult> {
    let zero_span = SourceSpan { start: 0_i64, end: 0_i64 };
    let kernel_bindings = {
    let mut __acc_0: Rc<std::collections::HashMap<String, Rc<TypeBinding>>> = Rc::new(std::collections::HashMap::new());
    for __elem_1 in Rc::new(KERNEL_TYPES.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.clone(), Rc::new(TypeBinding { name: __elem_1.clone(), resolved: leaf_node(&__elem_1) }));
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
    let some_value_field = Rc::new(Node { name: "value".to_string(), span: zero_span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: leaf_node("Dynamic") })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let some_variant = Rc::new(Node { name: "Some".to_string(), span: zero_span.clone(), children: Rc::new(vec!(some_value_field.clone())), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let kernel_optional = Rc::new(Node { name: "Optional".to_string(), span: zero_span.clone(), children: Rc::new(vec!(some_variant.clone(), leaf_node("None"))), connective: Some(Connective::Disj), collection_kind: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let kernel_bindings = {
    let __rc_5 = kernel_bindings;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert("Optional".to_string(), Rc::new(TypeBinding { name: "Optional".to_string(), resolved: kernel_optional.clone() }));
    Rc::new(__map_ins_4)
};
    let kernel = Rc::new(TypeEnv { bindings: kernel_bindings.clone(), recursive_types: Rc::new(Vec::new()), recursive_type_set: Rc::new(std::collections::HashMap::new()) });
    let parent_envs = {
    let mut __flat_mapped_6 = Vec::new();
    for __elem_7 in module.resolved_imports.iter().cloned() {
        __flat_mapped_6.extend((match parent_index.clone().get(&__elem_7.module_path.clone()).cloned() {
    Some(typed_parent) => {
        Rc::new(vec!(typed_parent.type_env.clone()))
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_6)
};
    let import_bindings = {
    let mut __acc_8 = Rc::new(std::collections::HashMap::new());
    for __elem_9 in parent_envs.iter().cloned() {
        __acc_8 = {
    let __rc_11 = __acc_8;
    let mut __map_merged_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_10.extend(Rc::try_unwrap(__elem_9.bindings.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_10)
};
    }
    __acc_8
};
    let import_recursive = {
    let mut __acc_12 = Rc::new(Vec::new());
    for __elem_13 in parent_envs.iter().cloned() {
        __acc_12 = v2_rt::concat(__acc_12, __elem_13.recursive_types.clone());
    }
    __acc_12
};
    let import_recursive_set = {
    let mut __acc_14 = Rc::new(std::collections::HashMap::new());
    for __elem_15 in parent_envs.iter().cloned() {
        __acc_14 = {
    let __rc_17 = __acc_14;
    let mut __map_merged_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_16.extend(Rc::try_unwrap(__elem_15.recursive_type_set.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_16)
};
    }
    __acc_14
};
    let import_env = Rc::new(TypeEnv { bindings: import_bindings.clone(), recursive_types: import_recursive.clone(), recursive_type_set: import_recursive_set.clone() });
    let import_diags = {
    let mut __flat_mapped_18 = Vec::new();
    for __elem_19 in module.resolved_imports.iter().cloned() {
        __flat_mapped_18.extend((match parent_index.clone().get(&__elem_19.module_path.clone()).cloned() {
    Some(_) => {
        Rc::new(Vec::new())
    }
    None => {
        Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("missing parent environment for imported module '".to_string(), __elem_19.module_path.clone()), "' while typechecking '".to_string()), module.module.name.clone()), "'".to_string()), span: Some(__elem_19.target_module.clone().unwrap().span.clone()), module_name: Some(module.module.name.clone()), category: Some(ErrorCategory::UnresolvedName) })))
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_18)
};
    let local_bindings = {
    let mut __acc_20: Rc<std::collections::HashMap<String, Rc<TypeBinding>>> = Rc::new(std::collections::HashMap::new());
    for __elem_21 in module.module.items.iter().cloned() {
        __acc_20 = if node_has_structure(__elem_21.clone()) {
    let type_node = Rc::new(Node { name: __elem_21.name.clone(), span: __elem_21.span.clone(), children: __elem_21.children.clone(), connective: __elem_21.connective.clone(), collection_kind: None, params: __elem_21.params.clone(), return_type: None, return_cardinality: __elem_21.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_23 = __acc_20;
    let mut __map_ins_22 = Rc::try_unwrap(__rc_23).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_22.insert(__elem_21.name.clone(), Rc::new(TypeBinding { name: __elem_21.name.clone(), resolved: type_node.clone() }));
    Rc::new(__map_ins_22)
}
} else {
    if ((__elem_21.return_type.clone().is_some()) && (({
    let __len_36 = __elem_21.params.clone().len();
    __len_36 as i64
}) == 0_i64)) && (__elem_21.body.clone().is_none()) {
    let alias_node = Rc::new(Node { name: __elem_21.name.clone(), span: __elem_21.span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: __elem_21.return_type.clone(), return_cardinality: __elem_21.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_25 = __acc_20;
    let mut __map_ins_24 = Rc::try_unwrap(__rc_25).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_24.insert(__elem_21.name.clone(), Rc::new(TypeBinding { name: __elem_21.name.clone(), resolved: alias_node.clone() }));
    Rc::new(__map_ins_24)
}
} else {
    if (__elem_21.transport.clone().is_none()) && (({
    let __len_35 = __elem_21.children.clone().len();
    __len_35 as i64
}) > 0_i64) {
    let ref_node = Rc::new(Node { name: __elem_21.name.clone(), span: __elem_21.span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: leaf_node(&__elem_21.name) })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_27 = __acc_20;
    let mut __map_ins_26 = Rc::try_unwrap(__rc_27).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_26.insert(__elem_21.name.clone(), Rc::new(TypeBinding { name: __elem_21.name.clone(), resolved: ref_node.clone() }));
    Rc::new(__map_ins_26)
}
} else {
    if (((({
    let __len_34 = __elem_21.params.clone().len();
    __len_34 as i64
}) > 0_i64) && (node_has_structure(__elem_21.clone()) == false)) && (__elem_21.body.clone().is_none())) && (__elem_21.transport.clone().is_none()) {
    let bare_node = Rc::new(Node { name: __elem_21.name.clone(), span: __elem_21.span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: __elem_21.params.clone(), return_type: None, return_cardinality: __elem_21.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_29 = __acc_20;
    let mut __map_ins_28 = Rc::try_unwrap(__rc_29).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_28.insert(__elem_21.name.clone(), Rc::new(TypeBinding { name: __elem_21.name.clone(), resolved: bare_node.clone() }));
    Rc::new(__map_ins_28)
}
} else {
    if ((((({
    let __len_32 = __elem_21.properties.clone().len();
    __len_32 as i64
}) > 0_i64) && (node_has_structure(__elem_21.clone()) == false)) && (__elem_21.transport.clone().is_none())) && (__elem_21.return_type.clone().is_none())) && (({
    let __len_33 = __elem_21.params.clone().len();
    __len_33 as i64
}) == 0_i64) {
    {
    let __rc_31 = __acc_20;
    let mut __map_ins_30 = Rc::try_unwrap(__rc_31).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_30.insert(__elem_21.name.clone(), Rc::new(TypeBinding { name: __elem_21.name.clone(), resolved: leaf_node(&__elem_21.name) }));
    Rc::new(__map_ins_30)
}
} else {
    __acc_20.clone()
}
}
}
}
};
    }
    __acc_20
};
    let param_bindings = {
    let mut __acc_37 = Rc::new(std::collections::HashMap::new());
    for __elem_38 in module.module.items.iter().cloned() {
        __acc_37 = {
    let is_type_decl = match local_bindings.clone().get(&__elem_38.name.clone()).cloned() {
    Some(_) => {
        true
    }
    None => {
        false
    }
};
    if (({
    let __len_43 = __elem_38.params.clone().len();
    __len_43 as i64
}) > 0_i64) && is_type_decl.clone() {
    let result = {
    let mut __acc_39 = __acc_37.clone();
    for __elem_40 in __elem_38.params.iter().cloned() {
        __acc_39 = {
    let __rc_42 = __acc_39;
    let mut __map_ins_41 = Rc::try_unwrap(__rc_42).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_41.insert(__elem_40.name.clone(), Rc::new(TypeBinding { name: __elem_40.name.clone(), resolved: leaf_node(&__elem_40.name) }));
    Rc::new(__map_ins_41)
};
    }
    __acc_39
};
    result.clone()
} else {
    __acc_37.clone()
}
};
    }
    __acc_37
};
    let local_name_set = {
    let mut __acc_48 = Rc::new(std::collections::HashMap::new());
    for __elem_49 in ({
    let __rc_44 = local_bindings.clone();
    let __map_unwrapped_45 = Rc::try_unwrap(__rc_44).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_46 = __map_unwrapped_45.into_iter().collect::<Vec<_>>();
    __entries_46.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_47 = __entries_46.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_47)
}).iter().cloned() {
        __acc_48 = {
    let __rc_51 = __acc_48;
    let mut __map_ins_50 = Rc::try_unwrap(__rc_51).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_50.insert(__elem_49.name.clone(), true);
    Rc::new(__map_ins_50)
};
    }
    __acc_48
};
    let all_local_bindings = {
    let __rc_53 = local_bindings;
    let mut __map_merged_52 = Rc::try_unwrap(__rc_53).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_52.extend(Rc::try_unwrap(param_bindings.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_52)
};
    let local_env = Rc::new(TypeEnv { bindings: all_local_bindings.clone(), recursive_types: Rc::new(Vec::new()), recursive_type_set: Rc::new(std::collections::HashMap::new()) });
    let merged = merge_envs(Rc::new(vec!(kernel.clone(), import_env.clone(), local_env.clone())));
    let all_deps_map = {
    let mut __acc_58: Rc<std::collections::HashMap<String, Rc<Vec<String>>>> = Rc::new(std::collections::HashMap::new());
    for __elem_59 in ({
    let __rc_54 = merged.bindings.clone();
    let __map_unwrapped_55 = Rc::try_unwrap(__rc_54).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_56 = __map_unwrapped_55.into_iter().collect::<Vec<_>>();
    __entries_56.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_57 = __entries_56.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_57)
}).iter().cloned() {
        __acc_58 = {
    let __rc_61 = __acc_58;
    let mut __map_ins_60 = Rc::try_unwrap(__rc_61).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_60.insert(__elem_59.name.clone(), node_type_deps(__elem_59.resolved.clone()));
    Rc::new(__map_ins_60)
};
    }
    __acc_58
};
    let cycle_set = detect_type_cycles_kahn(all_deps_map.clone(), merged.bindings.clone());
    let cycle_map = {
    let mut __acc_62 = Rc::new(std::collections::HashMap::new());
    for __elem_63 in cycle_set.iter().cloned() {
        __acc_62 = {
    let __rc_65 = __acc_62;
    let mut __map_ins_64 = Rc::try_unwrap(__rc_65).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_64.insert(__elem_63, true);
    Rc::new(__map_ins_64)
};
    }
    __acc_62
};
    let unresolved_env = Rc::new(TypeEnv { bindings: merged.bindings.clone(), recursive_types: cycle_set.clone(), recursive_type_set: cycle_map.clone() });
    let resolved = resolve_env_bindings(unresolved_env.clone(), &module.module.name, local_name_set.clone(), all_deps_map.clone());
    let resolved_env_out = resolved.env.clone();
    let resolved_diags = resolved.diagnostics.clone();
    Rc::new(BuildTypeEnvResult { env: resolved_env_out.clone(), diagnostics: v2_rt::concat(import_diags.clone(), resolved_diags.clone()) })
}

pub fn build_type_env_unresolved(module: Rc<ResolvedModule>, parent_index: Rc<HashMap<String, Rc<TypedModule>>>) -> Rc<BuildTypeEnvResult> {
    let zero_span = SourceSpan { start: 0_i64, end: 0_i64 };
    let kernel_bindings = {
    let mut __acc_0: Rc<std::collections::HashMap<String, Rc<TypeBinding>>> = Rc::new(std::collections::HashMap::new());
    for __elem_1 in Rc::new(KERNEL_TYPES.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.clone(), Rc::new(TypeBinding { name: __elem_1.clone(), resolved: leaf_node(&__elem_1) }));
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
    let some_value_field = Rc::new(Node { name: "value".to_string(), span: zero_span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: leaf_node("Dynamic") })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let some_variant = Rc::new(Node { name: "Some".to_string(), span: zero_span.clone(), children: Rc::new(vec!(some_value_field.clone())), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let kernel_optional = Rc::new(Node { name: "Optional".to_string(), span: zero_span.clone(), children: Rc::new(vec!(some_variant.clone(), leaf_node("None"))), connective: Some(Connective::Disj), collection_kind: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let kernel_bindings = {
    let __rc_5 = kernel_bindings;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert("Optional".to_string(), Rc::new(TypeBinding { name: "Optional".to_string(), resolved: kernel_optional.clone() }));
    Rc::new(__map_ins_4)
};
    let kernel = Rc::new(TypeEnv { bindings: kernel_bindings.clone(), recursive_types: Rc::new(Vec::new()), recursive_type_set: Rc::new(std::collections::HashMap::new()) });
    let parent_envs = {
    let mut __flat_mapped_6 = Vec::new();
    for __elem_7 in module.resolved_imports.iter().cloned() {
        __flat_mapped_6.extend((match parent_index.clone().get(&__elem_7.module_path.clone()).cloned() {
    Some(typed_parent) => {
        Rc::new(vec!(typed_parent.type_env.clone()))
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_6)
};
    let import_bindings = {
    let mut __acc_8 = Rc::new(std::collections::HashMap::new());
    for __elem_9 in parent_envs.iter().cloned() {
        __acc_8 = {
    let __rc_11 = __acc_8;
    let mut __map_merged_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_10.extend(Rc::try_unwrap(__elem_9.bindings.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_10)
};
    }
    __acc_8
};
    let import_recursive = {
    let mut __acc_12 = Rc::new(Vec::new());
    for __elem_13 in parent_envs.iter().cloned() {
        __acc_12 = v2_rt::concat(__acc_12, __elem_13.recursive_types.clone());
    }
    __acc_12
};
    let import_recursive_set = {
    let mut __acc_14 = Rc::new(std::collections::HashMap::new());
    for __elem_15 in parent_envs.iter().cloned() {
        __acc_14 = {
    let __rc_17 = __acc_14;
    let mut __map_merged_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_16.extend(Rc::try_unwrap(__elem_15.recursive_type_set.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_16)
};
    }
    __acc_14
};
    let import_env = Rc::new(TypeEnv { bindings: import_bindings.clone(), recursive_types: import_recursive.clone(), recursive_type_set: import_recursive_set.clone() });
    let local_bindings = {
    let mut __acc_18: Rc<std::collections::HashMap<String, Rc<TypeBinding>>> = Rc::new(std::collections::HashMap::new());
    for __elem_19 in module.module.items.iter().cloned() {
        __acc_18 = if node_has_structure(__elem_19.clone()) {
    let type_node = Rc::new(Node { name: __elem_19.name.clone(), span: __elem_19.span.clone(), children: __elem_19.children.clone(), connective: __elem_19.connective.clone(), collection_kind: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: __elem_19.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_21 = __acc_18;
    let mut __map_ins_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_20.insert(__elem_19.name.clone(), Rc::new(TypeBinding { name: __elem_19.name.clone(), resolved: type_node.clone() }));
    Rc::new(__map_ins_20)
}
} else {
    if ((__elem_19.return_type.clone().is_some()) && (({
    let __len_31 = __elem_19.params.clone().len();
    __len_31 as i64
}) == 0_i64)) && (__elem_19.body.clone().is_none()) {
    let alias_node = Rc::new(Node { name: __elem_19.name.clone(), span: __elem_19.span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: __elem_19.return_type.clone(), return_cardinality: __elem_19.return_cardinality.clone(), uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_23 = __acc_18;
    let mut __map_ins_22 = Rc::try_unwrap(__rc_23).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_22.insert(__elem_19.name.clone(), Rc::new(TypeBinding { name: __elem_19.name.clone(), resolved: alias_node.clone() }));
    Rc::new(__map_ins_22)
}
} else {
    if (__elem_19.transport.clone().is_none()) && (({
    let __len_30 = __elem_19.children.clone().len();
    __len_30 as i64
}) > 0_i64) {
    let ref_node = Rc::new(Node { name: __elem_19.name.clone(), span: __elem_19.span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: leaf_node(&__elem_19.name) })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    {
    let __rc_25 = __acc_18;
    let mut __map_ins_24 = Rc::try_unwrap(__rc_25).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_24.insert(__elem_19.name.clone(), Rc::new(TypeBinding { name: __elem_19.name.clone(), resolved: ref_node.clone() }));
    Rc::new(__map_ins_24)
}
} else {
    if ((((({
    let __len_28 = __elem_19.properties.clone().len();
    __len_28 as i64
}) > 0_i64) && (node_has_structure(__elem_19.clone()) == false)) && (__elem_19.transport.clone().is_none())) && (__elem_19.return_type.clone().is_none())) && (({
    let __len_29 = __elem_19.params.clone().len();
    __len_29 as i64
}) == 0_i64) {
    {
    let __rc_27 = __acc_18;
    let mut __map_ins_26 = Rc::try_unwrap(__rc_27).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_26.insert(__elem_19.name.clone(), Rc::new(TypeBinding { name: __elem_19.name.clone(), resolved: leaf_node(&__elem_19.name) }));
    Rc::new(__map_ins_26)
}
} else {
    __acc_18.clone()
}
}
}
};
    }
    __acc_18
};
    let local_env = Rc::new(TypeEnv { bindings: local_bindings.clone(), recursive_types: Rc::new(Vec::new()), recursive_type_set: Rc::new(std::collections::HashMap::new()) });
    let merged = merge_envs(Rc::new(vec!(kernel.clone(), import_env.clone(), local_env.clone())));
    let all_deps_map = {
    let mut __acc_36: Rc<std::collections::HashMap<String, Rc<Vec<String>>>> = Rc::new(std::collections::HashMap::new());
    for __elem_37 in ({
    let __rc_32 = merged.bindings.clone();
    let __map_unwrapped_33 = Rc::try_unwrap(__rc_32).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_34 = __map_unwrapped_33.into_iter().collect::<Vec<_>>();
    __entries_34.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_35 = __entries_34.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_35)
}).iter().cloned() {
        __acc_36 = {
    let __rc_39 = __acc_36;
    let mut __map_ins_38 = Rc::try_unwrap(__rc_39).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_38.insert(__elem_37.name.clone(), node_type_deps(__elem_37.resolved.clone()));
    Rc::new(__map_ins_38)
};
    }
    __acc_36
};
    let cycle_set = detect_type_cycles_kahn(all_deps_map.clone(), merged.bindings.clone());
    let cycle_map = {
    let mut __acc_40 = Rc::new(std::collections::HashMap::new());
    for __elem_41 in cycle_set.iter().cloned() {
        __acc_40 = {
    let __rc_43 = __acc_40;
    let mut __map_ins_42 = Rc::try_unwrap(__rc_43).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_42.insert(__elem_41, true);
    Rc::new(__map_ins_42)
};
    }
    __acc_40
};
    let unresolved_env = Rc::new(TypeEnv { bindings: merged.bindings.clone(), recursive_types: cycle_set.clone(), recursive_type_set: cycle_map.clone() });
    Rc::new(BuildTypeEnvResult { env: unresolved_env.clone(), diagnostics: Rc::new(Vec::new()) })
}

pub fn build_item_info(item: Rc<Node>) -> Rc<ItemInfo> {
    let kind = item_kind(item.clone());
    let res_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in item.uses.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
};
    match kind {
    ItemKind::FuncItem => {
        Rc::new(ItemInfo { name: item.name.clone(), kind, service_names: if item.body.clone().is_none() {
    Rc::new(Vec::new())
} else {
    collect_typed_service_calls(item.body.clone().unwrap())
}, resource_names: res_names.clone(), params: item.params.clone(), is_self_recursive: false, has_non_tail_self_call: false })
    }
    ItemKind::FnItem => {
        Rc::new(ItemInfo { name: item.name.clone(), kind, service_names: Rc::new(Vec::new()), resource_names: res_names.clone(), params: item.params.clone(), is_self_recursive: if item.body.clone().is_none() {
    false
} else {
    expr_has_self_call(item.body.clone().unwrap(), &item.name)
}, has_non_tail_self_call: if item.body.clone().is_none() {
    false
} else {
    expr_has_non_tail_self_call(item.body.clone().unwrap(), &item.name, true)
} })
    }
    _ => {
        Rc::new(ItemInfo { name: item.name.clone(), kind, service_names: Rc::new(Vec::new()), resource_names: res_names.clone(), params: item.params.clone(), is_self_recursive: false, has_non_tail_self_call: false })
    }
}
}

pub fn analyze_item(item: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ItemContribution> {
    let resolved = resolve_item_types(item.clone(), env.clone(), &module_name);
    let ritem = resolved.item.clone();
    let is_func = ({
    let __len_0 = ritem.params.clone().len();
    __len_0 as i64
}) > 0_i64;
    let is_zero_arg_func = ((({
    let __len_1 = ritem.params.clone().len();
    __len_1 as i64
}) == 0_i64) && (ritem.return_type.clone().is_some())) && (ritem.body.clone().is_some());
    let func_sig = if is_func.clone() || is_zero_arg_func.clone() {
    let declared_rt = if ritem.return_type.clone().is_some() {
    Some(rt_type(ritem.clone()))
} else {
    None
};
    Some(Rc::new(DeclaredFuncSig { name: ritem.name.clone(), params: ritem.params.clone(), return_type: declared_rt.clone(), is_async: ({
    let __len_2 = ritem.uses.clone().len();
    __len_2 as i64
}) > 0_i64 }))
} else {
    None
};
    let svc_entries = if (ritem.transport.clone().is_some()) && (({
    let __len_5 = ritem.children.clone().len();
    __len_5 as i64
}) > 0_i64) {
    {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ritem.children.iter().cloned() {
        __mapped_3.push(service_op_entry(__elem_4.clone()));
    }
    Rc::new(__mapped_3)
}
} else {
    Rc::new(Vec::new())
};
    let svc_local = if (ritem.transport.clone().is_some()) && (({
    let __len_6 = ritem.children.clone().len();
    __len_6 as i64
}) > 0_i64) {
    let root = namespace_root_from_properties(ritem.properties.clone(), &ritem.name);
    Some(Rc::new(TypeBinding { name: root.clone(), resolved: leaf_node(&root) }))
} else {
    None
};
    Rc::new(ItemContribution { resolved_item: ritem.clone(), resolve_diagnostics: resolved.diagnostics.clone(), func_sig: func_sig.clone(), svc_entries: svc_entries.clone(), svc_local: svc_local.clone(), item_info: build_item_info(ritem.clone()) })
}

pub fn fold_module_contributions(remaining: Rc<Vec<Rc<ItemContribution>>>, resolved_items: Rc<Vec<Rc<Node>>>, func_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>, svc_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>, svc_locals: Rc<HashMap<String, Rc<TypeBinding>>>, item_registry: Rc<HashMap<String, Rc<ItemInfo>>>, diag_chunks: Rc<Vec<Rc<Vec<Rc<Diagnostic>>>>>) -> Rc<LocalContributionState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_resolved_items = resolved_items;
        let mut __tco_p_func_sigs = func_sigs;
        let mut __tco_p_svc_registry = svc_registry;
        let mut __tco_p_svc_locals = svc_locals;
        let mut __tco_p_item_registry = item_registry;
        let mut __tco_p_diag_chunks = diag_chunks;
        loop {
            let remaining = __tco_p_remaining;
            let resolved_items = __tco_p_resolved_items;
            let func_sigs = __tco_p_func_sigs;
            let svc_registry = __tco_p_svc_registry;
            let svc_locals = __tco_p_svc_locals;
            let item_registry = __tco_p_item_registry;
            let diag_chunks = __tco_p_diag_chunks;
            match remaining.clone().first().cloned() {
    None => {
        break Rc::new(LocalContributionState { resolved_items: resolved_items.clone(), func_sigs: func_sigs.clone(), svc_registry: svc_registry.clone(), svc_locals: svc_locals.clone(), item_registry: item_registry.clone(), diag_chunks: diag_chunks.clone() });
    }
    Some(contribution) => {
        {
    let next_func_sigs = match contribution.func_sig.clone() {
    Some(sig) => {
        {
    let __rc_1 = func_sigs;
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert(sig.name.clone(), sig.clone());
    Rc::new(__map_ins_0)
}
    }
    None => {
        func_sigs.clone()
    }
};
    let next_svc_registry = if ({
    let __len_4 = contribution.svc_entries.clone().len();
    __len_4 as i64
}) > 0_i64 {
    {
    let __rc_3 = svc_registry;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(contribution.resolved_item.name.clone(), contribution.svc_entries.clone());
    Rc::new(__map_ins_2)
}
} else {
    svc_registry.clone()
};
    let next_svc_locals = match contribution.svc_local.clone() {
    Some(binding) => {
        {
    let __rc_6 = svc_locals;
    let mut __map_ins_5 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_5.insert(binding.name.clone(), binding.clone());
    Rc::new(__map_ins_5)
}
    }
    None => {
        svc_locals.clone()
    }
};
     {
        let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
        let __tco_1 = {
    let __rc_8 = resolved_items;
    let mut __appended_7 = Rc::try_unwrap(__rc_8).unwrap_or_else(|rc| (*rc).clone());
    __appended_7.push(contribution.resolved_item.clone());
    Rc::new(__appended_7)
};
        let __tco_2 = next_func_sigs.clone();
        let __tco_3 = next_svc_registry.clone();
        let __tco_4 = next_svc_locals.clone();
        let __tco_5 = {
    let __rc_10 = item_registry;
    let mut __map_ins_9 = Rc::try_unwrap(__rc_10).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_9.insert(contribution.item_info.name.clone(), contribution.item_info.clone());
    Rc::new(__map_ins_9)
};
        let __tco_6 = {
    let __rc_12 = diag_chunks;
    let mut __appended_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __appended_11.push(contribution.resolve_diagnostics.clone());
    Rc::new(__appended_11)
};
        __tco_p_remaining = __tco_0;
        __tco_p_resolved_items = __tco_1;
        __tco_p_func_sigs = __tco_2;
        __tco_p_svc_registry = __tco_3;
        __tco_p_svc_locals = __tco_4;
        __tco_p_item_registry = __tco_5;
        __tco_p_diag_chunks = __tco_6;
        continue;
    }

};
    }
};
        }
    })
}

pub fn build_module_context(contributions: Rc<Vec<Rc<ItemContribution>>>, parent_index: Rc<HashMap<String, Rc<TypedModule>>>, resolved_imports: Rc<Vec<Rc<ResolvedImport>>>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ModuleContext> {
    let local = fold_module_contributions(contributions.clone(), Rc::new(Vec::new()), Rc::new(std::collections::HashMap::new()), Rc::new(std::collections::HashMap::new()), Rc::new(std::collections::HashMap::new()), Rc::new(std::collections::HashMap::new()), Rc::new(Vec::new()));
    let imported_variant_locals = {
    let mut __acc_4 = Rc::new(std::collections::HashMap::new());
    for __elem_5 in ({
    let __rc_0 = env.bindings.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = if node_is_coproduct(__elem_5.resolved.clone()) {
    {
    let mut __acc_6 = __acc_4.clone();
    for __elem_7 in __elem_5.resolved.children.iter().cloned() {
        __acc_6 = {
    let __rc_9 = __acc_6;
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert(__elem_7.name.clone(), Rc::new(TypeBinding { name: __elem_7.name.clone(), resolved: leaf_node(&__elem_5.name) }));
    Rc::new(__map_ins_8)
};
    }
    __acc_6
}
} else {
    __acc_4.clone()
};
    }
    __acc_4
};
    let env_variant_locals = variant_locals_from_items(local.resolved_items.clone(), imported_variant_locals.clone());
    let merged_scope = merge_scope_from_imports(resolved_imports.clone(), parent_index.clone(), env.clone(), Rc::new(std::collections::HashMap::new()), local.svc_registry.clone(), local.svc_locals.clone());
    let all_declared_sigs = {
    let __rc_11 = merged_scope.func_sigs.clone();
    let mut __map_merged_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_10.extend(Rc::try_unwrap(local.func_sigs.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_10)
};
    let resolve_result = resolve_func_sigs(all_declared_sigs.clone(), local.resolved_items.clone(), &module_name);
    let optional_locals = {
    let __rc_13 = env_variant_locals;
    let mut __map_ins_12 = Rc::try_unwrap(__rc_13).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_12.insert("Some".to_string(), Rc::new(TypeBinding { name: "Some".to_string(), resolved: leaf_node("Optional") }));
    Rc::new(__map_ins_12)
};
    let optional_locals = {
    let __rc_15 = optional_locals;
    let mut __map_ins_14 = Rc::try_unwrap(__rc_15).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_14.insert("None".to_string(), Rc::new(TypeBinding { name: "None".to_string(), resolved: leaf_node("Optional") }));
    Rc::new(__map_ins_14)
};
    let all_locals = {
    let mut __acc_20 = optional_locals.clone();
    for __elem_21 in ({
    let __rc_16 = merged_scope.svc_locals.clone();
    let __map_unwrapped_17 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_18 = __map_unwrapped_17.into_iter().collect::<Vec<_>>();
    __entries_18.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_19 = __entries_18.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_19)
}).iter().cloned() {
        __acc_20 = {
    let __rc_23 = __acc_20;
    let mut __map_ins_22 = Rc::try_unwrap(__rc_23).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_22.insert(__elem_21.name.clone(), __elem_21.clone());
    Rc::new(__map_ins_22)
};
    }
    __acc_20
};
    Rc::new(ModuleContext { resolved_items: local.resolved_items.clone(), func_env: resolve_result.func_env.clone(), svc_registry: merged_scope.svc_registry.clone(), locals: all_locals.clone(), item_registry: local.item_registry.clone(), diagnostics: v2_rt::concat({
    let mut __flat_mapped_26 = Vec::new();
    for __elem_27 in local.diag_chunks.iter().cloned() {
        __flat_mapped_26.extend(__elem_27.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_26)
}, resolve_result.diagnostics.clone()) })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypecheckModuleResult {
    pub typed: Rc<TypedModule>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn typecheck_module(resolved: Rc<ResolvedModule>, parent_index: Rc<HashMap<String, Rc<TypedModule>>>) -> Rc<TypecheckModuleResult> {
    let env_result = build_type_env(resolved.clone(), parent_index.clone());
    let env = env_result.env.clone();
    let env_diags = env_result.diagnostics.clone();
    let env_errors = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in env_diags.iter().cloned() {
        if __elem_1.severity.clone() == Severity::Error {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    if ({
    let __len_2 = env_errors.clone().len();
    __len_2 as i64
}) > 0_i64 {
    return Rc::new(TypecheckModuleResult { typed: Rc::new(TypedModule { module: resolved.module.clone(), items: Rc::new(Vec::new()), type_env: env.clone(), func_env: Rc::new(ResolvedFuncEnv { signatures: Rc::new(std::collections::HashMap::new()) }), item_registry: Rc::new(std::collections::HashMap::new()) }), diagnostics: env_diags.clone() });
};
    let contributions = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in resolved.module.items.iter().cloned() {
        __mapped_3.push(analyze_item(__elem_4.clone(), env.clone(), &resolved.module.name));
    }
    Rc::new(__mapped_3)
};
    let ctx = build_module_context(contributions.clone(), parent_index.clone(), resolved.resolved_imports.clone(), env.clone(), &resolved.module.name);
    let data_locals = {
    let mut __acc_5 = ctx.locals.clone();
    for __elem_6 in ctx.resolved_items.iter().cloned() {
        __acc_5 = if (((__elem_6.body.clone().is_some()) && (({
    let __len_9 = __elem_6.params.clone().len();
    __len_9 as i64
}) == 0_i64)) && (__elem_6.return_type.clone().is_none())) && (__elem_6.type_annotation.clone().is_some()) {
    {
    let __rc_8 = __acc_5;
    let mut __map_ins_7 = Rc::try_unwrap(__rc_8).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_7.insert(__elem_6.name.clone(), Rc::new(TypeBinding { name: __elem_6.name.clone(), resolved: __elem_6.type_annotation.clone().unwrap() }));
    Rc::new(__map_ins_7)
}
} else {
    __acc_5.clone()
};
    }
    __acc_5
};
    let infer_scope = Rc::new(InferScope { type_env: env.clone(), func_env: ctx.func_env.clone(), locals: data_locals.clone(), module_name: resolved.module.name.clone(), service_registry: ctx.svc_registry.clone(), item_registry: ctx.item_registry.clone() });
    let typed_item_results = infer_items(ctx.resolved_items.clone(), infer_scope.clone());
    let typed_items = {
    let mut __mapped_10 = Vec::new();
    for __elem_11 in typed_item_results.iter().cloned() {
        __mapped_10.push(__elem_11.item.clone());
    }
    Rc::new(__mapped_10)
};
    let infer_diags = {
    let mut __flat_mapped_12 = Vec::new();
    for __elem_13 in typed_item_results.iter().cloned() {
        __flat_mapped_12.extend(__elem_13.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_12)
};
    let typed_module = Rc::new(Module { name: resolved.module.name.clone(), imports: resolved.module.imports.clone(), items: ctx.resolved_items.clone(), span: resolved.module.span.clone() });
    Rc::new(TypecheckModuleResult { typed: Rc::new(TypedModule { module: typed_module.clone(), items: typed_items.clone(), type_env: env.clone(), func_env: ctx.func_env.clone(), item_registry: ctx.item_registry.clone() }), diagnostics: v2_rt::concat(v2_rt::concat(env_diags.clone(), ctx.diagnostics.clone()), infer_diags.clone()) })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvResolveResult {
    pub env: Rc<TypeEnv>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingResult {
    pub binding: Rc<TypeBinding>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingsAccum {
    pub bindings: Rc<HashMap<String, Rc<TypeBinding>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn resolve_env_bindings(env: Rc<TypeEnv>, module_name: &str, local_names: Rc<HashMap<String, bool>>, deps_map: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<EnvResolveResult> {
    let remaining = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in ({
    let mut __filtered_4 = Vec::new();
    for __elem_5 in ({
    let __rc_0 = env.bindings.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        if emit_map_has(local_names.clone(), &__elem_5.name) {
    __filtered_4.push(__elem_5);
};
    }
    Rc::new(__filtered_4)
}).iter().cloned() {
        __mapped_6.push(__elem_7.name.clone());
    }
    Rc::new(__mapped_6)
};
    topo_resolve_types(remaining.clone(), env.clone(), &module_name, Rc::new(Vec::new()), local_names.clone(), deps_map.clone())
}

pub fn topo_resolve_types(remaining: Rc<Vec<String>>, env: Rc<TypeEnv>, module_name: &str, diagnostics: Rc<Vec<Rc<Diagnostic>>>, local_names: Rc<HashMap<String, bool>>, deps_map: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<EnvResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_env = env;
        let mut __tco_p_module_name = module_name.to_string();
        let mut __tco_p_diagnostics = diagnostics;
        let mut __tco_p_local_names = local_names;
        let mut __tco_p_deps_map = deps_map;
        loop {
            let remaining = __tco_p_remaining;
            let env = __tco_p_env;
            let module_name = __tco_p_module_name;
            let diagnostics = __tco_p_diagnostics;
            let local_names = __tco_p_local_names;
            let deps_map = __tco_p_deps_map;
            if ({
    let __len_0 = remaining.clone().len();
    __len_0 as i64
}) == 0_i64 {
    break Rc::new(EnvResolveResult { env: env.clone(), diagnostics: diagnostics.clone() });
};
            let remaining_set = {
    let mut __acc_1 = Rc::new(std::collections::HashMap::new());
    for __elem_2 in remaining.iter().cloned() {
        __acc_1 = {
    let __rc_4 = __acc_1;
    let mut __map_ins_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_3.insert(__elem_2, true);
    Rc::new(__map_ins_3)
};
    }
    __acc_1
};
            let ready = {
    let mut __filtered_5 = Vec::new();
    for __elem_6 in remaining.iter().cloned() {
        if match deps_map.clone().get(&__elem_6.clone()).cloned() {
    Some(deps) => {
        {
    let mut __all_7 = true;
    for __elem_8 in deps.iter().cloned() {
        if !((((is_kernel_type(&__elem_8) || (__elem_8.clone() == "None")) || (__elem_8.clone() == "")) || is_recursive_type(env.clone(), &__elem_8)) || (emit_map_has(remaining_set.clone(), &__elem_8) == false)) {
    __all_7 = false;
    break;
};
    }
    __all_7
}
    }
    None => {
        true
    }
} {
    __filtered_5.push(__elem_6);
};
    }
    Rc::new(__filtered_5)
};
            if ({
    let __len_13 = ready.clone().len();
    __len_13 as i64
}) == 0_i64 {
    let stuck_accum = {
    let mut __acc_9 = Rc::new(BindingsAccum { bindings: env.bindings.clone(), diagnostics: Rc::new(Vec::new()) });
    for __elem_10 in remaining.iter().cloned() {
        __acc_9 = match env.bindings.clone().get(&__elem_10.clone()).cloned() {
    Some(binding) => {
        {
    let result = resolve_node(binding.resolved.clone(), env.clone(), &module_name);
    Rc::new(BindingsAccum { bindings: {
    let __rc_12 = std::mem::take(&mut Rc::make_mut(&mut __acc_9).bindings);
    let mut __map_ins_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_11.insert(__elem_10.clone(), Rc::new(TypeBinding { name: __elem_10.clone(), resolved: result.resolved.clone() }));
    Rc::new(__map_ins_11)
}, diagnostics: v2_rt::concat(__acc_9.diagnostics.clone(), result.diagnostics.clone()) })
}
    }
    None => {
        __acc_9.clone()
    }
};
    }
    __acc_9
};
    break Rc::new(EnvResolveResult { env: Rc::new(TypeEnv { bindings: stuck_accum.bindings.clone(), recursive_types: env.recursive_types.clone(), recursive_type_set: env.recursive_type_set.clone() }), diagnostics: v2_rt::concat(diagnostics.clone(), stuck_accum.diagnostics.clone()) });
};
            let ready_accum = {
    let mut __acc_14 = Rc::new(BindingsAccum { bindings: env.bindings.clone(), diagnostics: Rc::new(Vec::new()) });
    for __elem_15 in ready.iter().cloned() {
        __acc_14 = match env.bindings.clone().get(&__elem_15.clone()).cloned() {
    Some(binding) => {
        {
    let result = resolve_node(binding.resolved.clone(), env.clone(), &module_name);
    Rc::new(BindingsAccum { bindings: {
    let __rc_17 = std::mem::take(&mut Rc::make_mut(&mut __acc_14).bindings);
    let mut __map_ins_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_16.insert(__elem_15.clone(), Rc::new(TypeBinding { name: __elem_15.clone(), resolved: result.resolved.clone() }));
    Rc::new(__map_ins_16)
}, diagnostics: v2_rt::concat(__acc_14.diagnostics.clone(), result.diagnostics.clone()) })
}
    }
    None => {
        __acc_14.clone()
    }
};
    }
    __acc_14
};
            let ready_set = {
    let mut __acc_18 = Rc::new(std::collections::HashMap::new());
    for __elem_19 in ready.iter().cloned() {
        __acc_18 = {
    let __rc_21 = __acc_18;
    let mut __map_ins_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_20.insert(__elem_19.clone(), true);
    Rc::new(__map_ins_20)
};
    }
    __acc_18
};
            let next_remaining = {
    let mut __filtered_22 = Vec::new();
    for __elem_23 in remaining.iter().cloned() {
        if emit_map_has(ready_set.clone(), &__elem_23) == false {
    __filtered_22.push(__elem_23);
};
    }
    Rc::new(__filtered_22)
};
             {
                let __tco_0 = next_remaining.clone();
                let __tco_1 = Rc::new(TypeEnv { bindings: ready_accum.bindings.clone(), recursive_types: env.recursive_types.clone(), recursive_type_set: env.recursive_type_set.clone() });
                let __tco_2 = module_name;
                let __tco_3 = v2_rt::concat(diagnostics.clone(), ready_accum.diagnostics.clone());
                let __tco_4 = local_names.clone();
                let __tco_5 = deps_map.clone();
                __tco_p_remaining = __tco_0;
                __tco_p_env = __tco_1;
                __tco_p_module_name = __tco_2;
                __tco_p_diagnostics = __tco_3;
                __tco_p_local_names = __tco_4;
                __tco_p_deps_map = __tco_5;
                continue;
            }

        }
    })
}

pub fn collect_parent_envs(resolved: Rc<ResolvedModule>, module_index: Rc<HashMap<String, Rc<TypedModule>>>) -> Rc<ParentModulesResult> {
    let modules = {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in resolved.resolved_imports.iter().cloned() {
        __flat_mapped_0.extend((match module_index.clone().get(&__elem_1.module_path.clone()).cloned() {
    Some(typed) => {
        Rc::new(vec!(typed.clone()))
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
};
    let diagnostics = {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in resolved.resolved_imports.iter().cloned() {
        __flat_mapped_2.extend((match module_index.clone().get(&__elem_3.module_path.clone()).cloned() {
    Some(_) => {
        Rc::new(Vec::new())
    }
    None => {
        Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("missing parent environment for imported module '".to_string(), __elem_3.module_path.clone()), "' while ordering '".to_string()), resolved.module.name.clone()), "'".to_string()), span: Some(__elem_3.target_module.clone().unwrap().span.clone()), module_name: Some(resolved.module.name.clone()), category: Some(ErrorCategory::UnresolvedName) })))
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
};
    Rc::new(ParentModulesResult { modules: modules.clone(), diagnostics: diagnostics.clone() })
}

pub fn build_emit_graph_info(modules: Rc<Vec<Rc<TypedModule>>>) -> Rc<EmitGraphInfo> {
    let init = Rc::new(EmitInfoBuildState { type_summaries: Rc::new(std::collections::HashMap::new()), variant_to_enum: Rc::new(std::collections::HashMap::new()), enum_variant_membership: Rc::new(std::collections::HashMap::new()), field_type_names: Rc::new(std::collections::HashMap::new()) });
    let built = {
    let mut __acc_0 = init.clone();
    for __elem_1 in modules.iter().cloned() {
        __acc_0 = {
    let mut __acc_2 = __acc_0.clone();
    for __elem_3 in __elem_1.items.iter().cloned() {
        __acc_2 = add_emit_item_summary(__acc_2.clone(), __elem_3.clone());
    }
    __acc_2
};
    }
    __acc_0
};
    Rc::new(EmitGraphInfo { type_summaries: built.type_summaries.clone(), variant_to_enum: built.variant_to_enum.clone(), enum_variant_membership: built.enum_variant_membership.clone(), field_type_names: built.field_type_names.clone() })
}

pub fn typecheck(graph: Rc<ModuleGraph>) -> Rc<TypedGraph> {
    typecheck_modules(graph.modules.clone(), Rc::new(Vec::new()), Rc::new(std::collections::HashMap::new()), Rc::new(std::collections::HashMap::new()), Rc::new(Vec::new()))
}

pub fn typecheck_modules(remaining: Rc<Vec<Rc<ResolvedModule>>>, modules: Rc<Vec<Rc<TypedModule>>>, module_index: Rc<HashMap<String, Rc<TypedModule>>>, item_registry: Rc<HashMap<String, Rc<ItemInfo>>>, diag_chunks: Rc<Vec<Rc<Vec<Rc<Diagnostic>>>>>) -> Rc<TypedGraph> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_modules = modules;
        let mut __tco_p_module_index = module_index;
        let mut __tco_p_item_registry = item_registry;
        let mut __tco_p_diag_chunks = diag_chunks;
        loop {
            let remaining = __tco_p_remaining;
            let modules = __tco_p_modules;
            let module_index = __tco_p_module_index;
            let item_registry = __tco_p_item_registry;
            let diag_chunks = __tco_p_diag_chunks;
            match remaining.clone().first().cloned() {
    None => {
        {
    let expanded_registry = expand_transitive_services(modules.clone(), item_registry.clone(), 5_i64);
    break Rc::new(TypedGraph { modules: modules.clone(), item_registry: expanded_registry.clone(), diagnostics: {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in diag_chunks.iter().cloned() {
        __flat_mapped_0.extend(__elem_1.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_0)
} });
};
    }
    Some(resolved) => {
        {
    let parent_result = collect_parent_envs(resolved.clone(), module_index.clone());
    let tc_result = typecheck_module(resolved.clone(), module_index.clone());
    let typed = tc_result.typed.clone();
    let tc_diags = tc_result.diagnostics.clone();
     {
        let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
        let __tco_1 = {
    let __rc_3 = modules;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(typed.clone());
    Rc::new(__appended_2)
};
        let __tco_2 = {
    let __rc_5 = module_index;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert(typed.module.name.clone(), typed.clone());
    Rc::new(__map_ins_4)
};
        let __tco_3 = {
    let __rc_7 = item_registry;
    let mut __map_merged_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_6.extend(Rc::try_unwrap(typed.item_registry.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_6)
};
        let __tco_4 = {
    let __rc_11 = {
    let __rc_9 = diag_chunks;
    let mut __appended_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __appended_8.push(parent_result.diagnostics.clone());
    Rc::new(__appended_8)
};
    let mut __appended_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __appended_10.push(tc_diags.clone());
    Rc::new(__appended_10)
};
        __tco_p_remaining = __tco_0;
        __tco_p_modules = __tco_1;
        __tco_p_module_index = __tco_2;
        __tco_p_item_registry = __tco_3;
        __tco_p_diag_chunks = __tco_4;
        continue;
    }

};
    }
};
        }
    })
}

pub fn reconcile(graph: Rc<ModuleGraph>) -> Rc<ResolvedGraph> {
    let typed = typecheck(graph.clone());
    Rc::new(ResolvedGraph { modules: typed.modules.clone(), item_registry: typed.item_registry.clone(), diagnostics: typed.diagnostics.clone() })
}

