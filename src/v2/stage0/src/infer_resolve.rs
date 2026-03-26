use crate::v2_core::*;
use crate::infer_types::*;
use crate::infer_env::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeResolveResult {
    pub resolved: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemResult {
    pub item: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldResult {
    pub field: Rc<Field>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExprResolveResult {
    pub expr: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamedArgResolveResult {
    pub arg: Rc<NamedArg>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchArmResolveResult {
    pub arm: Rc<MatchArm>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldInitResolveResult {
    pub field_init: Rc<FieldInit>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringPartResolveResult {
    pub part: Rc<StringPart>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransportResolveResult {
    pub transport: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceConfigResolveResult {
    pub config: Rc<ServiceConfig>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParamResult {
    pub param: Rc<Param>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceUseResult {
    pub resource_use: Rc<ResourceUse>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn resolve_node(n: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<NodeResolveResult> {
    resolve_node_bounded(n.clone(), env.clone(), &module_name, 0_i64)
}

pub fn is_user_generic_use_site(n: Rc<Node>, env: Rc<TypeEnv>) -> bool {
    if node_has_structure(n.clone()) {
    false
} else {
    if node_is_map(n.clone()) {
    false
} else {
    if node_is_container(n.clone()) {
    false
} else {
    match lookup_type(env.clone(), &n.name) {
    Some(decl) => {
        ({
    let __len_0 = decl.params.clone().len();
    __len_0 as i64
}) > 0_i64
    }
    None => {
        false
    }
}
}
}
}
}

pub fn substitute_type_slots(n: Rc<Node>, slot_bindings: Rc<HashMap<String, Rc<Node>>>, decl_name: &str) -> Rc<Node> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let is_slot = (((({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64) && (n.connective.clone().is_none())) && (n.body.clone().is_none())) && (n.return_type.clone().is_none());
        if is_slot.clone() {
    match slot_bindings.clone().get(&n.name.clone()).cloned() {
    Some(concrete) => {
        concrete.clone()
    }
    None => {
        n.clone()
    }
}
} else {
    let new_children = {
    let mut __mapped_1 = Vec::new();
    for __elem_2 in n.children.iter().cloned() {
        __mapped_1.push(if __elem_2.name.clone() == decl_name {
    let substituted_args = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in __elem_2.children.iter().cloned() {
        __mapped_3.push(substitute_type_slots(__elem_4.clone(), slot_bindings.clone(), &decl_name));
    }
    Rc::new(__mapped_3)
};
    {
    let __rc_6 = __elem_2;
    let mut __owned_5 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| { debug_assert!(false, "V5: expected sole ownership of `__elem_2`"); (*rc).clone() });
    let __taken_7 = std::mem::take(&mut __owned_5.children);
    __owned_5.children = substituted_args.clone();
    Rc::new(__owned_5)
}
} else {
    substitute_type_slots(__elem_2.clone(), slot_bindings.clone(), &decl_name)
});
    }
    Rc::new(__mapped_1)
};
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: new_children.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), expr_data: n.expr_data.clone() })
}
    })
}

pub fn resolve_node_bounded(n: Rc<Node>, env: Rc<TypeEnv>, module_name: &str, depth: i64) -> Rc<NodeResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if depth.clone() > 100_i64 {
    return Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat(v2_rt::concat("internal: type resolution exceeded depth 100 for '".to_string(), n.name.clone()), "'".to_string()), span: Some(n.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::InvalidOperation) }))) });
};
        if node_has_structure(n.clone()) {
    if node_is_product(n.clone()) {
    if n.name.clone() == "Refined" {
    match n.children.clone().first().cloned() {
    Some(base) => {
        {
    let base_result = resolve_node_bounded(base.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let base_resolved = base_result.resolved.clone();
    let base_diags = base_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: Rc::new(vec!(base_resolved.clone())), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: base_diags.clone() })
}
    }
    None => {
        Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
    }
}
} else {
    let child_results = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in n.children.iter().cloned() {
        __mapped_0.push(if __elem_1.return_type.clone().is_none() {
    Rc::new(NodeResolveResult { resolved: __elem_1.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    let child_rt = rt_type(__elem_1.clone());
    let rt_result = resolve_node_bounded(child_rt.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let rt_resolved = rt_result.resolved.clone();
    let rt_diags = rt_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: __elem_1.name.clone(), span: __elem_1.span.clone(), children: __elem_1.children.clone(), connective: __elem_1.connective.clone(), collection_kind: __elem_1.collection_kind.clone(), params: __elem_1.params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: rt_resolved.clone() })), return_cardinality: __elem_1.return_cardinality.clone(), uses: __elem_1.uses.clone(), body: __elem_1.body.clone(), transport: __elem_1.transport.clone(), properties: __elem_1.properties.clone(), type_annotation: __elem_1.type_annotation.clone(), config: __elem_1.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: rt_diags.clone() })
});
    }
    Rc::new(__mapped_0)
};
    let resolved_children = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in child_results.iter().cloned() {
        __mapped_2.push(__elem_3.resolved.clone());
    }
    Rc::new(__mapped_2)
};
    let all_diags = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in child_results.iter().cloned() {
        __flat_mapped_4.extend(__elem_5.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: resolved_children.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: all_diags.clone() })
}
} else {
    if node_is_optional(n.clone()) {
    let inner = with_required_cardinality(n.clone());
    let inner_result = resolve_node_bounded(inner.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let inner_resolved = inner_result.resolved.clone();
    let inner_diags = inner_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: with_optional_cardinality(inner_resolved.clone()), diagnostics: inner_diags.clone() })
} else {
    let variant_results = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in n.children.iter().cloned() {
        __mapped_6.push({
    let field_results = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in __elem_7.children.iter().cloned() {
        __mapped_8.push(if __elem_9.return_type.clone().is_none() {
    Rc::new(NodeResolveResult { resolved: __elem_9.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    let field_rt = rt_type(__elem_9.clone());
    let is_self_ref = (field_rt.name.clone() == n.name.clone()) && (({
    let __len_10 = field_rt.children.clone().len();
    __len_10 as i64
}) > 0_i64);
    let rt_result = if is_self_ref.clone() {
    Rc::new(NodeResolveResult { resolved: field_rt.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    resolve_node_bounded(field_rt.clone(), env.clone(), &module_name, depth.clone() + 1_i64)
};
    let rt_resolved = rt_result.resolved.clone();
    let rt_diags = rt_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: __elem_9.name.clone(), span: __elem_9.span.clone(), children: __elem_9.children.clone(), connective: __elem_9.connective.clone(), collection_kind: __elem_9.collection_kind.clone(), params: __elem_9.params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: rt_resolved.clone() })), return_cardinality: __elem_9.return_cardinality.clone(), uses: __elem_9.uses.clone(), body: __elem_9.body.clone(), transport: __elem_9.transport.clone(), properties: __elem_9.properties.clone(), type_annotation: __elem_9.type_annotation.clone(), config: __elem_9.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: rt_diags.clone() })
});
    }
    Rc::new(__mapped_8)
};
    let resolved_fields = {
    let mut __mapped_11 = Vec::new();
    for __elem_12 in field_results.iter().cloned() {
        __mapped_11.push(__elem_12.resolved.clone());
    }
    Rc::new(__mapped_11)
};
    let field_diags = {
    let mut __flat_mapped_13 = Vec::new();
    for __elem_14 in field_results.iter().cloned() {
        __flat_mapped_13.extend(__elem_14.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_13)
};
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: __elem_7.name.clone(), span: __elem_7.span.clone(), children: resolved_fields.clone(), connective: __elem_7.connective.clone(), collection_kind: __elem_7.collection_kind.clone(), params: __elem_7.params.clone(), return_type: __elem_7.return_type.clone(), return_cardinality: __elem_7.return_cardinality.clone(), uses: __elem_7.uses.clone(), body: __elem_7.body.clone(), transport: __elem_7.transport.clone(), properties: __elem_7.properties.clone(), type_annotation: __elem_7.type_annotation.clone(), config: __elem_7.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: field_diags.clone() })
});
    }
    Rc::new(__mapped_6)
};
    let resolved_variants = {
    let mut __mapped_15 = Vec::new();
    for __elem_16 in variant_results.iter().cloned() {
        __mapped_15.push(__elem_16.resolved.clone());
    }
    Rc::new(__mapped_15)
};
    let all_diags = {
    let mut __flat_mapped_17 = Vec::new();
    for __elem_18 in variant_results.iter().cloned() {
        __flat_mapped_17.extend(__elem_18.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_17)
};
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: resolved_variants.clone(), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: all_diags.clone() })
}
}
} else {
    if (({
    let __len_45 = n.children.clone().len();
    __len_45 as i64
}) > 0_i64) && is_user_generic_use_site(n.clone(), env.clone()) {
    let decl = match lookup_type(env.clone(), &n.name) {
    Some(d) => {
        d.clone()
    }
    None => {
        n.clone()
    }
};
    let expected_arity = {
    let __len_19 = decl.params.clone().len();
    __len_19 as i64
};
    let actual_arity = {
    let __len_20 = n.children.clone().len();
    __len_20 as i64
};
    let arity_diags = if expected_arity.clone() != actual_arity.clone() {
    Rc::new(vec!(Rc::new(Diagnostic { message: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), n.name.clone()), " expects ".to_string()), v2_rt::to_string(expected_arity.clone())), " type arguments, got ".to_string()), v2_rt::to_string(actual_arity.clone())), severity: Severity::Error, span: Some(n.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::TypeMismatch) })))
} else {
    Rc::new(Vec::new())
};
    let arg_results = {
    let mut __mapped_21 = Vec::new();
    for __elem_22 in n.children.iter().cloned() {
        __mapped_21.push(resolve_node_bounded(__elem_22.clone(), env.clone(), &module_name, depth.clone() + 1_i64));
    }
    Rc::new(__mapped_21)
};
    let resolved_args = {
    let mut __mapped_23 = Vec::new();
    for __elem_24 in arg_results.iter().cloned() {
        __mapped_23.push(__elem_24.resolved.clone());
    }
    Rc::new(__mapped_23)
};
    let arg_diags = {
    let mut __flat_mapped_25 = Vec::new();
    for __elem_26 in arg_results.iter().cloned() {
        __flat_mapped_25.extend(__elem_26.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_25)
};
    let slot_bindings = {
    let mut __acc_30 = Rc::new(std::collections::HashMap::new());
    for __elem_31 in ({
    let mut __enumerated_27 = Vec::new();
    for (__idx_28, __elem_29) in decl.params.clone().iter().enumerate() {
        __enumerated_27.push((__idx_28 as i64, __elem_29.clone()));
    }
    Rc::new(__enumerated_27)
}).iter().cloned() {
        __acc_30 = {
    let idx = __elem_31.0.clone();
    let slot_name = __elem_31.1.name.clone();
    match ({
    let mut __mapped_37 = Vec::new();
    for __elem_38 in ({
    let mut __filtered_35 = Vec::new();
    for __elem_36 in ({
    let mut __enumerated_32 = Vec::new();
    for (__idx_33, __elem_34) in resolved_args.clone().iter().enumerate() {
        __enumerated_32.push((__idx_33 as i64, __elem_34.clone()));
    }
    Rc::new(__enumerated_32)
}).iter().cloned() {
        if __elem_36.0.clone() == idx.clone() {
    __filtered_35.push(__elem_36);
};
    }
    Rc::new(__filtered_35)
}).iter().cloned() {
        __mapped_37.push(__elem_38.1.clone());
    }
    Rc::new(__mapped_37)
}).first().cloned() {
    Some(arg) => {
        {
    let __rc_40 = __acc_30;
    let mut __map_ins_39 = Rc::try_unwrap(__rc_40).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_39.insert(slot_name.clone(), arg.clone());
    Rc::new(__map_ins_39)
}
    }
    None => {
        __acc_30.clone()
    }
}
};
    }
    __acc_30
};
    let substituted_children = {
    let mut __mapped_41 = Vec::new();
    for __elem_42 in decl.children.iter().cloned() {
        __mapped_41.push(substitute_type_slots(__elem_42.clone(), slot_bindings.clone(), &n.name));
    }
    Rc::new(__mapped_41)
};
    let is_recursive = is_recursive_type(env.clone(), &n.name);
    let result = Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: substituted_children.clone(), connective: decl.connective.clone(), collection_kind: n.collection_kind.clone(), params: Rc::new(Vec::new()), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: decl.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: is_recursive, has_non_tail_self_call: n.has_non_tail_self_call.clone(), expr_data: n.expr_data.clone() }), diagnostics: v2_rt::concat(arity_diags.clone(), arg_diags.clone()) });
    result.clone()
} else {
    if node_is_map(n.clone()) {
    let key_child = match n.children.clone().first().cloned() {
    Some(k) => {
        k.clone()
    }
    None => {
        leaf_node("String")
    }
};
    let val_child = match n.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        leaf_node("Unit")
    }
};
    let key_result = resolve_node_bounded(key_child.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let key_resolved = key_result.resolved.clone();
    let key_diags = key_result.diagnostics.clone();
    let val_result = resolve_node_bounded(val_child.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let val_resolved = val_result.resolved.clone();
    let val_diags = val_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: Rc::new(vec!(key_resolved.clone(), val_resolved.clone())), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: v2_rt::concat(key_diags.clone(), val_diags.clone()) })
} else {
    if ({
    let __len_44 = n.children.clone().len();
    __len_44 as i64
}) == 1_i64 {
    match n.children.clone().first().cloned() {
    Some(el) => {
        {
    let el_result = resolve_node_bounded(el.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let el_resolved = el_result.resolved.clone();
    let el_diags = el_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: Rc::new(vec!(el_resolved.clone())), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: el_diags.clone() })
}
    }
    None => {
        Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
    }
}
} else {
    if ({
    let __len_43 = n.children.clone().len();
    __len_43 as i64
}) == 0_i64 {
    if is_recursive_type(env.clone(), &n.name) {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    match lookup_type(env.clone(), &n.name) {
    Some(resolved) => {
        {
    let final_resolved = if node_is_optional(n.clone()) {
    with_optional_cardinality(resolved.clone())
} else {
    resolved.clone()
};
    Rc::new(NodeResolveResult { resolved: final_resolved.clone(), diagnostics: Rc::new(Vec::new()) })
}
    }
    None => {
        if ((is_kernel_type(&n.name) || (n.name.clone() == "Dynamic")) || (n.name.clone() == "Error")) || (n.name.clone() == "Callable") {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat(v2_rt::concat("unresolved type '".to_string(), n.name.clone()), "'".to_string()), span: Some(n.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::UnresolvedName) }))) })
}
    }
}
}
} else {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
}
}
}
}
}
    })
}

pub fn resolve_optional_node(n: Option<Rc<InferredNode>>, env: Rc<TypeEnv>, module_name: &str) -> Rc<NodeResolveResult> {
    if n.clone().is_none() {
    Rc::new(NodeResolveResult { resolved: leaf_node("Unit"), diagnostics: Rc::new(Vec::new()) })
} else {
    match n.clone().unwrap().as_ref() {
    InferredNode::Resolved { node: inner, .. } => {
        resolve_node(inner.clone(), env.clone(), &module_name)
    }
    InferredNode::CompilerError { message: msg, span: sp, .. } => {
        Rc::new(NodeResolveResult { resolved: leaf_node("Error"), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: msg.clone(), span: Some(sp.clone()), module_name: Some(module_name.to_string()), category: None }))) })
    }
}
}
}

pub fn resolve_field(field: Rc<Field>, env: Rc<TypeEnv>, module_name: &str) -> Rc<FieldResult> {
    let type_result = resolve_node(field.type_expr.clone(), env.clone(), &module_name);
    let type_resolved = type_result.resolved.clone();
    let type_diags = type_result.diagnostics.clone();
    let default_resolved = match field.default_value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(default_value) => {
        let default_value = Rc::new(default_value.clone());
        Some(resolve_expr_types(default_value.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let default_diags = match default_resolved.clone() {
    Some(result) => {
        result.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(FieldResult { field: Rc::new(Field { name: field.name.clone(), type_expr: type_resolved.clone(), cardinality: field.cardinality.clone(), default_value: match default_resolved.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, from_key: field.from_key.clone(), span: field.span.clone() }), diagnostics: v2_rt::concat(type_diags.clone(), default_diags.clone()) })
}

pub fn resolve_param(param: Rc<Param>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ParamResult> {
    let type_result = resolve_node(param.type_expr.clone(), env.clone(), &module_name);
    let type_resolved = type_result.resolved.clone();
    let type_diags = type_result.diagnostics.clone();
    let default_resolved = match param.default_value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(default_value) => {
        let default_value = Rc::new(default_value.clone());
        Some(resolve_expr_types(default_value.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let default_diags = match default_resolved.clone() {
    Some(result) => {
        result.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(ParamResult { param: Rc::new(Param { name: param.name.clone(), type_expr: type_resolved.clone(), default_value: match default_resolved.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, span: param.span.clone() }), diagnostics: v2_rt::concat(type_diags.clone(), default_diags.clone()) })
}

pub fn resolve_resource_use(ru: Rc<ResourceUse>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ResourceUseResult> {
    let type_result = resolve_node(ru.resource.clone(), env.clone(), &module_name);
    let type_resolved = type_result.resolved.clone();
    let type_diags = type_result.diagnostics.clone();
    Rc::new(ResourceUseResult { resource_use: Rc::new(ResourceUse { name: ru.name.clone(), resource: type_resolved.clone(), span: ru.span.clone() }), diagnostics: type_diags.clone() })
}

pub fn resolve_named_arg(arg: Rc<NamedArg>, env: Rc<TypeEnv>, module_name: &str) -> Rc<NamedArgResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let value_result = resolve_expr_types(arg.value.clone(), env.clone(), &module_name);
        let value_expr = value_result.expr.clone();
        let value_diags = value_result.diagnostics.clone();
        Rc::new(NamedArgResolveResult { arg: Rc::new(NamedArg { name: arg.name.clone(), value: value_expr.clone() }), diagnostics: value_diags.clone() })
    })
}

pub fn resolve_field_init(field_init: Rc<FieldInit>, env: Rc<TypeEnv>, module_name: &str) -> Rc<FieldInitResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let value_result = resolve_expr_types(field_init.value.clone(), env.clone(), &module_name);
        let value_expr = value_result.expr.clone();
        let value_diags = value_result.diagnostics.clone();
        Rc::new(FieldInitResolveResult { field_init: Rc::new(FieldInit { name: field_init.name.clone(), value: value_expr.clone() }), diagnostics: value_diags.clone() })
    })
}

pub fn resolve_match_arm(arm: Rc<MatchArm>, env: Rc<TypeEnv>, module_name: &str) -> Rc<MatchArmResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let guard_result = match arm.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(guard) => {
        let guard = Rc::new(guard.clone());
        Some(resolve_expr_types(guard.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
        let guard_diags = match guard_result.clone() {
    Some(result) => {
        result.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
        let body_result = resolve_expr_types(arm.body.clone(), env.clone(), &module_name);
        let body_expr = body_result.expr.clone();
        let body_diags = body_result.diagnostics.clone();
        Rc::new(MatchArmResolveResult { arm: Rc::new(MatchArm { pattern: arm.pattern.clone(), guard: match guard_result.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, body: body_expr.clone() }), diagnostics: v2_rt::concat(guard_diags.clone(), body_diags.clone()) })
    })
}

pub fn resolve_string_part(part: Rc<StringPart>, env: Rc<TypeEnv>, module_name: &str) -> Rc<StringPartResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match part.as_ref() {
    StringPart::Text { value, .. } => {
        Rc::new(StringPartResolveResult { part: Rc::new(StringPart::Text { value: value.clone() }), diagnostics: Rc::new(Vec::new()) })
    }
    StringPart::Interpolation { expr, .. } => {
        {
    let expr_result = resolve_expr_types(expr.clone(), env.clone(), &module_name);
    let resolved_expr = expr_result.expr.clone();
    let expr_diags = expr_result.diagnostics.clone();
    Rc::new(StringPartResolveResult { part: Rc::new(StringPart::Interpolation { expr: resolved_expr.clone() }), diagnostics: expr_diags.clone() })
}
    }
}
    })
}

pub fn resolve_service_config(config: Rc<ServiceConfig>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ServiceConfigResolveResult> {
    let endpoint_result = resolve_expr_types(config.endpoint.clone(), env.clone(), &module_name);
    let endpoint_expr = endpoint_result.expr.clone();
    let endpoint_diags = endpoint_result.diagnostics.clone();
    let auth_result = match config.auth.as_ref().map(|__rc| __rc.as_ref()) {
    Some(auth) => {
        let auth = Rc::new(auth.clone());
        Some(resolve_expr_types(auth.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let auth_diags = match auth_result.clone() {
    Some(result) => {
        result.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let rate_result = match config.rate_limit.as_ref().map(|__rc| __rc.as_ref()) {
    Some(rate_limit) => {
        let rate_limit = Rc::new(rate_limit.clone());
        Some(resolve_expr_types(rate_limit.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let rate_diags = match rate_result.clone() {
    Some(result) => {
        result.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let retry_result = match config.retry.as_ref().map(|__rc| __rc.as_ref()) {
    Some(retry) => {
        let retry = Rc::new(retry.clone());
        Some(resolve_expr_types(retry.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let retry_diags = match retry_result.clone() {
    Some(result) => {
        result.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(ServiceConfigResolveResult { config: Rc::new(ServiceConfig { endpoint: endpoint_expr.clone(), auth: match auth_result.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, rate_limit: match rate_result.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, retry: match retry_result.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
} }), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(endpoint_diags.clone(), auth_diags.clone()), rate_diags.clone()), retry_diags.clone()) })
}

pub fn resolve_transport_binding(transport: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<TransportResolveResult> {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::LocalTransport)) {
    Rc::new(TransportResolveResult { transport: transport.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    let prop_results = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in transport.properties.iter().cloned() {
        __mapped_0.push({
    let val_result = resolve_expr_types(__elem_1.value.clone(), env.clone(), &module_name);
    Rc::new(FieldInitResolveResult { field_init: Rc::new(FieldInit { name: __elem_1.name.clone(), value: val_result.expr.clone() }), diagnostics: val_result.diagnostics.clone() })
});
    }
    Rc::new(__mapped_0)
};
    let resolved_props = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in prop_results.iter().cloned() {
        __mapped_2.push(__elem_3.field_init.clone());
    }
    Rc::new(__mapped_2)
};
    let prop_diags = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in prop_results.iter().cloned() {
        __flat_mapped_4.extend(__elem_5.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    let child_results = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in transport.children.iter().cloned() {
        __mapped_6.push(resolve_expr_types(__elem_7.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_6)
};
    let resolved_children = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in child_results.iter().cloned() {
        __mapped_8.push(__elem_9.expr.clone());
    }
    Rc::new(__mapped_8)
};
    let child_diags = {
    let mut __flat_mapped_10 = Vec::new();
    for __elem_11 in child_results.iter().cloned() {
        __flat_mapped_10.extend(__elem_11.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_10)
};
    Rc::new(TransportResolveResult { transport: make_transport_node(&transport.name, resolved_props.clone(), resolved_children.clone(), transport.span.clone()), diagnostics: v2_rt::concat(prop_diags.clone(), child_diags.clone()) })
}
}

pub fn resolve_expr_types(texpr: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ExprResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: _, .. } => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    ExprData::ExprError { kind, message, .. } => {
        Rc::new(ExprResolveResult { expr: make_expr_error_node(kind.clone(), &message, texpr.span.clone()), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: message.clone(), span: Some(texpr.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::CascadeError) }))) })
    }
    ExprData::ExprVar { name: _, binding_kind: _, .. } => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    ExprData::ExprFieldAccess { base, field, summary, .. } => {
        {
    let r = resolve_expr_types(base.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: map_expr_children(texpr.clone(), |child| r.expr.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::ExprCall { func, args, call_semantics: cs, .. } => {
        {
    let ar = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(resolve_named_arg(__elem_1.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_0)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprCall { func: func.clone(), args: {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ar.iter().cloned() {
        __mapped_4.push(__elem_5.arg.clone());
    }
    Rc::new(__mapped_4)
}, call_semantics: cs.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: {
    let mut __flat_mapped_6 = Vec::new();
    for __elem_7 in ar.iter().cloned() {
        __flat_mapped_6.extend(__elem_7.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_6)
} })
}
    }
    ExprData::ExprMethodCall { receiver, method, args, method_semantics: ms, .. } => {
        {
    let rr = resolve_expr_types(receiver.clone(), env.clone(), &module_name);
    let ar = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in args.iter().cloned() {
        __mapped_8.push(resolve_named_arg(__elem_9.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_8)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprMethodCall { receiver: rr.expr.clone(), method: method.clone(), args: {
    let mut __mapped_12 = Vec::new();
    for __elem_13 in ar.iter().cloned() {
        __mapped_12.push(__elem_13.arg.clone());
    }
    Rc::new(__mapped_12)
}, method_semantics: ms.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(rr.diagnostics.clone(), {
    let mut __flat_mapped_14 = Vec::new();
    for __elem_15 in ar.iter().cloned() {
        __flat_mapped_14.extend(__elem_15.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_14)
}) })
}
    }
    ExprData::ExprMatch { scrutinee, arms, .. } => {
        {
    let sr = resolve_expr_types(scrutinee.clone(), env.clone(), &module_name);
    let ar = {
    let mut __mapped_16 = Vec::new();
    for __elem_17 in arms.iter().cloned() {
        __mapped_16.push(resolve_match_arm(__elem_17.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_16)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprMatch { scrutinee: sr.expr.clone(), arms: {
    let mut __mapped_20 = Vec::new();
    for __elem_21 in ar.iter().cloned() {
        __mapped_20.push(__elem_21.arm.clone());
    }
    Rc::new(__mapped_20)
} }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(sr.diagnostics.clone(), {
    let mut __flat_mapped_22 = Vec::new();
    for __elem_23 in ar.iter().cloned() {
        __flat_mapped_22.extend(__elem_23.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_22)
}) })
}
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        {
    let cr = resolve_expr_types(c.clone(), env.clone(), &module_name);
    let tr = resolve_expr_types(t.clone(), env.clone(), &module_name);
    let er = match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        Some(resolve_expr_types(eb.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprIf { condition: cr.expr.clone(), then_branch: tr.expr.clone(), else_branch: match er.clone() {
    Some(r) => {
        Some(r.expr.clone())
    }
    None => {
        None
    }
} }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(v2_rt::concat(cr.diagnostics.clone(), tr.diagnostics.clone()), match er.clone() {
    Some(r) => {
        r.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
}) })
}
    }
    ExprData::ExprLet { name, value: v, body: b, .. } => {
        {
    let vr = resolve_expr_types(v.clone(), env.clone(), &module_name);
    let br = match b.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        Some(resolve_expr_types(bd.clone(), env.clone(), &module_name))
    }
    None => {
        None
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprLet { name: name.clone(), value: vr.expr.clone(), body: match br.clone() {
    Some(r) => {
        Some(r.expr.clone())
    }
    None => {
        None
    }
} }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(vr.diagnostics.clone(), match br.clone() {
    Some(r) => {
        r.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
}) })
}
    }
    ExprData::ExprRecordLit { type_name: tn, fields, parent_enum: pe, .. } => {
        {
    let fr = {
    let mut __mapped_24 = Vec::new();
    for __elem_25 in fields.iter().cloned() {
        __mapped_24.push(resolve_field_init(__elem_25.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_24)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: tn.clone(), fields: {
    let mut __mapped_28 = Vec::new();
    for __elem_29 in fr.iter().cloned() {
        __mapped_28.push(__elem_29.field_init.clone());
    }
    Rc::new(__mapped_28)
}, parent_enum: pe.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: {
    let mut __flat_mapped_30 = Vec::new();
    for __elem_31 in fr.iter().cloned() {
        __flat_mapped_30.extend(__elem_31.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_30)
} })
}
    }
    ExprData::ExprListLit { elements: els, .. } => {
        {
    let er = {
    let mut __mapped_32 = Vec::new();
    for __elem_33 in els.iter().cloned() {
        __mapped_32.push(resolve_expr_types(__elem_33.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_32)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprListLit { elements: {
    let mut __mapped_36 = Vec::new();
    for __elem_37 in er.iter().cloned() {
        __mapped_36.push(__elem_37.expr.clone());
    }
    Rc::new(__mapped_36)
} }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: {
    let mut __flat_mapped_38 = Vec::new();
    for __elem_39 in er.iter().cloned() {
        __flat_mapped_38.extend(__elem_39.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_38)
} })
}
    }
    ExprData::ExprBinOp { op, left: l, right: r, .. } => {
        {
    let lr = resolve_expr_types(l.clone(), env.clone(), &module_name);
    let rr = resolve_expr_types(r.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprBinOp { op: op.clone(), left: lr.expr.clone(), right: rr.expr.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(lr.diagnostics.clone(), rr.diagnostics.clone()) })
}
    }
    ExprData::ExprUnaryOp { op, operand: o, .. } => {
        {
    let r = resolve_expr_types(o.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: map_expr_children(texpr.clone(), |child| r.expr.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::ExprLambda { params: p, body: b, semantics: s, .. } => {
        {
    let r = resolve_expr_types(b.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: map_expr_children(texpr.clone(), |child| r.expr.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::ExprStringInterp { parts, .. } => {
        {
    let pr = {
    let mut __mapped_40 = Vec::new();
    for __elem_41 in parts.iter().cloned() {
        __mapped_40.push(resolve_string_part(__elem_41.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_40)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprStringInterp { parts: {
    let mut __mapped_44 = Vec::new();
    for __elem_45 in pr.iter().cloned() {
        __mapped_44.push(__elem_45.part.clone());
    }
    Rc::new(__mapped_44)
} }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: {
    let mut __flat_mapped_46 = Vec::new();
    for __elem_47 in pr.iter().cloned() {
        __flat_mapped_46.extend(__elem_47.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_46)
} })
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        {
    let sr = {
    let mut __mapped_48 = Vec::new();
    for __elem_49 in ss.iter().cloned() {
        __mapped_48.push(resolve_expr_types(__elem_49.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_48)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock { stmts: {
    let mut __mapped_52 = Vec::new();
    for __elem_53 in sr.iter().cloned() {
        __mapped_52.push(__elem_53.expr.clone());
    }
    Rc::new(__mapped_52)
} }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: {
    let mut __flat_mapped_54 = Vec::new();
    for __elem_55 in sr.iter().cloned() {
        __flat_mapped_54.extend(__elem_55.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_54)
} })
}
    }
    ExprData::ExprCast { expr: inner, target, .. } => {
        {
    let r = resolve_expr_types(inner.clone(), env.clone(), &module_name);
    let tr = resolve_node(target.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprCast { expr: r.expr.clone(), target: tr.resolved.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(r.diagnostics.clone(), tr.diagnostics.clone()) })
}
    }
    ExprData::ExprForEach { variable, collection: c, body: b, .. } => {
        {
    let cr = resolve_expr_types(c.clone(), env.clone(), &module_name);
    let br = resolve_expr_types(b.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprForEach { variable: variable.clone(), collection: cr.expr.clone(), body: br.expr.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(cr.diagnostics.clone(), br.diagnostics.clone()) })
}
    }
    ExprData::ExprIndex { base, index, .. } => {
        {
    let br = resolve_expr_types(base.clone(), env.clone(), &module_name);
    let ir = resolve_expr_types(index.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprIndex { base: br.expr.clone(), index: ir.expr.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(br.diagnostics.clone(), ir.diagnostics.clone()) })
}
    }
    ExprData::ExprSlice { base, start, end, .. } => {
        {
    let br = resolve_expr_types(base.clone(), env.clone(), &module_name);
    let sr = resolve_expr_types(start.clone(), env.clone(), &module_name);
    let er = resolve_expr_types(end.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprSlice { base: br.expr.clone(), start: sr.expr.clone(), end: er.expr.clone() }), texpr.return_type.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(v2_rt::concat(br.diagnostics.clone(), sr.diagnostics.clone()), er.diagnostics.clone()) })
}
    }
    ExprData::ExprReturn { value: inner, .. } => {
        {
    let r = resolve_expr_types(inner.clone(), env.clone(), &module_name);
    Rc::new(ExprResolveResult { expr: map_expr_children(texpr.clone(), |child| r.expr.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::NoExprData => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
}
    })
}

pub fn resolve_item_types(item: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ItemResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let param_results = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in item.params.iter().cloned() {
        __mapped_0.push(resolve_param(__elem_1.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_0)
};
        let resolved_params = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in param_results.iter().cloned() {
        __mapped_2.push(__elem_3.param.clone());
    }
    Rc::new(__mapped_2)
};
        let param_diags = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in param_results.iter().cloned() {
        __flat_mapped_4.extend(__elem_5.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
        let ret_result = resolve_optional_node(item.return_type.clone(), env.clone(), &module_name);
        let ret_resolved = ret_result.resolved.clone();
        let ret_diags = ret_result.diagnostics.clone();
        let resolved_ret = if item.return_type.clone().is_none() {
    None
} else {
    Some(Rc::new(InferredNode::Resolved { node: ret_resolved.clone() }))
};
        let use_results = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in item.uses.iter().cloned() {
        __mapped_6.push(resolve_resource_use(__elem_7.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_6)
};
        let resolved_uses = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in use_results.iter().cloned() {
        __mapped_8.push(__elem_9.resource_use.clone());
    }
    Rc::new(__mapped_8)
};
        let use_diags = {
    let mut __flat_mapped_10 = Vec::new();
    for __elem_11 in use_results.iter().cloned() {
        __flat_mapped_10.extend(__elem_11.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_10)
};
        let body_resolved = if item.body.clone().is_none() {
    Rc::new(ExprResolveResult { expr: item.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    resolve_expr_types(item.body.clone().unwrap(), env.clone(), &module_name)
};
        let resolved_body = if item.body.clone().is_none() {
    None
} else {
    Some(body_resolved.expr.clone())
};
        let body_diags = if item.body.clone().is_none() {
    Rc::new(Vec::new())
} else {
    body_resolved.diagnostics.clone()
};
        let anno_resolved = if item.type_annotation.clone().is_none() {
    Rc::new(NodeResolveResult { resolved: leaf_node("Unit"), diagnostics: Rc::new(Vec::new()) })
} else {
    resolve_node(item.type_annotation.clone().unwrap(), env.clone(), &module_name)
};
        let resolved_anno = if item.type_annotation.clone().is_none() {
    None
} else {
    Some(anno_resolved.resolved.clone())
};
        let anno_diags = anno_resolved.diagnostics.clone();
        let transport_resolved = if item.transport.clone().is_none() {
    Rc::new(TransportResolveResult { transport: local_transport_node(no_span()), diagnostics: Rc::new(Vec::new()) })
} else {
    resolve_transport_binding(item.transport.clone().unwrap(), env.clone(), &module_name)
};
        let resolved_transport = if item.transport.clone().is_none() {
    None
} else {
    Some(transport_resolved.transport.clone())
};
        let transport_diags = transport_resolved.diagnostics.clone();
        let config_resolved = if item.config.clone().is_none() {
    Rc::new(ServiceConfigResolveResult { config: Rc::new(ServiceConfig { endpoint: item.clone(), auth: None, rate_limit: None, retry: None }), diagnostics: Rc::new(Vec::new()) })
} else {
    resolve_service_config(item.config.clone().unwrap(), env.clone(), &module_name)
};
        let resolved_config = if item.config.clone().is_none() {
    None
} else {
    Some(config_resolved.config.clone())
};
        let config_diags = if item.config.clone().is_none() {
    Rc::new(Vec::new())
} else {
    config_resolved.diagnostics.clone()
};
        let prop_results = {
    let mut __mapped_12 = Vec::new();
    for __elem_13 in item.properties.iter().cloned() {
        __mapped_12.push(resolve_field_init(__elem_13.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_12)
};
        let resolved_props = {
    let mut __mapped_14 = Vec::new();
    for __elem_15 in prop_results.iter().cloned() {
        __mapped_14.push(__elem_15.field_init.clone());
    }
    Rc::new(__mapped_14)
};
        let prop_diags = {
    let mut __flat_mapped_16 = Vec::new();
    for __elem_17 in prop_results.iter().cloned() {
        __flat_mapped_16.extend(__elem_17.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_16)
};
        let child_results = {
    let mut __mapped_18 = Vec::new();
    for __elem_19 in item.children.iter().cloned() {
        __mapped_18.push(resolve_item_types(__elem_19.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_18)
};
        let resolved_children = {
    let mut __mapped_20 = Vec::new();
    for __elem_21 in child_results.iter().cloned() {
        __mapped_20.push(__elem_21.item.clone());
    }
    Rc::new(__mapped_20)
};
        let child_diags = {
    let mut __flat_mapped_22 = Vec::new();
    for __elem_23 in child_results.iter().cloned() {
        __flat_mapped_22.extend(__elem_23.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_22)
};
        Rc::new(ItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: resolved_children.clone(), params: resolved_params.clone(), return_type: resolved_ret.clone(), return_cardinality: item.return_cardinality.clone(), uses: resolved_uses.clone(), body: resolved_body.clone(), connective: item.connective.clone(), collection_kind: item.collection_kind.clone(), transport: resolved_transport.clone(), properties: resolved_props.clone(), type_annotation: resolved_anno.clone(), config: resolved_config.clone(), is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(param_diags.clone(), ret_diags.clone()), use_diags.clone()), body_diags.clone()), anno_diags.clone()), transport_diags.clone()), config_diags.clone()), prop_diags.clone()), child_diags.clone()) })
    })
}

