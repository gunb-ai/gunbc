use crate::v2_core::*;
use crate::infer_types::*;
use crate::infer_env::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeResolveResult {
    pub resolved: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemResult {
    pub item: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldResult {
    pub field: Rc<Field>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExprResolveResult {
    pub expr: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamedArgResolveResult {
    pub arg: Rc<NamedArg>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchArmResolveResult {
    pub arm: Rc<MatchArm>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldInitResolveResult {
    pub field_init: Rc<FieldInit>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StringPartResolveResult {
    pub part: Rc<StringPart>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransportResolveResult {
    pub transport: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParamResult {
    pub param: Rc<Param>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceUseResult {
    pub resource_use: Rc<ResourceUse>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn resolve_node(n: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<NodeResolveResult> {
    resolve_node_bounded(n, env, &module_name, 0_i64)
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
    match lookup_type(env, &n.name) {
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
}) == 0_i64) && (n.connective.clone().is_none())) && (n.body.clone().is_none())) && (n.inferred.clone().is_none());
        if is_slot {
    match slot_bindings.clone().get(&n.name.clone()).cloned() {
    Some(concrete) => {
        concrete
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
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: new_children, connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), match_pattern: n.match_pattern.clone(), expr_data: n.expr_data.clone() })
}
    })
}

pub fn resolve_node_bounded(n: Rc<Node>, env: Rc<TypeEnv>, module_name: &str, depth: i64) -> Rc<NodeResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if depth.clone() > 100_i64 {
    return Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(vec!(diagnostic_node("error", &v2_rt::concat(v2_rt::concat("internal: type resolution exceeded depth 100 for '".to_string(), n.name.clone()), "'".to_string()), n.span.clone(), Some(module_name.to_string()), Some("invalid_operation".to_string())))) });
};
        let n = if n.collection_kind.clone().is_none() {
    let ck = collection_kind_for_name(&n.name);
    match ck.clone() {
    Some(_) => {
        Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), collection_kind: ck.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), match_pattern: n.match_pattern.clone(), expr_data: n.expr_data.clone() })
    }
    None => {
        n.clone()
    }
}
} else {
    n.clone()
};
        if node_has_structure(n.clone()) {
    if node_is_product(n.clone()) {
    if n.name.clone() == "Refined" {
    match n.children.clone().first().cloned() {
    Some(base) => {
        {
    let base_result = resolve_node_bounded(base, env.clone(), &module_name, depth.clone() + 1_i64);
    let base_resolved = base_result.resolved.clone();
    let base_diags = base_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: Rc::new(vec!(base_resolved)), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: base_diags })
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
        __mapped_0.push(if __elem_1.inferred.clone().is_none() {
    Rc::new(NodeResolveResult { resolved: __elem_1.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    let child_rt = rt_type(__elem_1.clone());
    let rt_result = resolve_node_bounded(child_rt.clone(), env.clone(), &module_name, depth.clone() + 1_i64);
    let rt_resolved = rt_result.resolved.clone();
    let rt_diags = rt_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: __elem_1.name.clone(), span: __elem_1.span.clone(), children: __elem_1.children.clone(), connective: __elem_1.connective.clone(), collection_kind: __elem_1.collection_kind.clone(), params: __elem_1.params.clone(), inferred: Some(Rc::new(InferredNode::Resolved { node: rt_resolved.clone() })), return_cardinality: __elem_1.return_cardinality.clone(), uses: __elem_1.uses.clone(), body: __elem_1.body.clone(), transport: __elem_1.transport.clone(), properties: __elem_1.properties.clone(), type_annotation: __elem_1.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: rt_diags.clone() })
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
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: resolved_children, connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: all_diags })
}
} else {
    if node_is_optional(n.clone()) {
    let inner = with_required_cardinality(n.clone());
    let inner_result = resolve_node_bounded(inner, env.clone(), &module_name, depth.clone() + 1_i64);
    let inner_resolved = inner_result.resolved.clone();
    let inner_diags = inner_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: with_optional_cardinality(inner_resolved), diagnostics: inner_diags })
} else {
    let variant_results = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in n.children.iter().cloned() {
        __mapped_6.push({
    let field_results = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in __elem_7.children.iter().cloned() {
        __mapped_8.push(if __elem_9.inferred.clone().is_none() {
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
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: __elem_9.name.clone(), span: __elem_9.span.clone(), children: __elem_9.children.clone(), connective: __elem_9.connective.clone(), collection_kind: __elem_9.collection_kind.clone(), params: __elem_9.params.clone(), inferred: Some(Rc::new(InferredNode::Resolved { node: rt_resolved.clone() })), return_cardinality: __elem_9.return_cardinality.clone(), uses: __elem_9.uses.clone(), body: __elem_9.body.clone(), transport: __elem_9.transport.clone(), properties: __elem_9.properties.clone(), type_annotation: __elem_9.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: rt_diags.clone() })
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
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: __elem_7.name.clone(), span: __elem_7.span.clone(), children: resolved_fields.clone(), connective: __elem_7.connective.clone(), collection_kind: __elem_7.collection_kind.clone(), params: __elem_7.params.clone(), inferred: __elem_7.inferred.clone(), return_cardinality: __elem_7.return_cardinality.clone(), uses: __elem_7.uses.clone(), body: __elem_7.body.clone(), transport: __elem_7.transport.clone(), properties: __elem_7.properties.clone(), type_annotation: __elem_7.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: field_diags.clone() })
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
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: resolved_variants, connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: all_diags })
}
}
} else {
    if (({
    let __len_46 = n.children.clone().len();
    __len_46 as i64
}) > 0_i64) && is_user_generic_use_site(n.clone(), env.clone()) {
    let decl = match lookup_type(env.clone(), &n.name) {
    Some(d) => {
        d
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
    Rc::new(vec!(diagnostic_node("error", &v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), n.name.clone()), " expects ".to_string()), v2_rt::to_string(expected_arity.clone())), " type arguments, got ".to_string()), v2_rt::to_string(actual_arity.clone())), n.span.clone(), Some(module_name.to_string()), Some("type_mismatch".to_string()))))
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
    let result = Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: substituted_children, connective: decl.connective.clone(), collection_kind: collection_kind_for_name(&n.name), params: Rc::new(Vec::new()), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: decl.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: is_recursive, has_non_tail_self_call: n.has_non_tail_self_call.clone(), match_pattern: n.match_pattern.clone(), expr_data: n.expr_data.clone() }), diagnostics: v2_rt::concat(arity_diags, arg_diags) });
    result
} else {
    if node_is_map(n.clone()) {
    let key_child = match n.children.clone().first().cloned() {
    Some(k) => {
        k
    }
    None => {
        leaf_node("String")
    }
};
    let val_child = match n.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v
    }
    None => {
        leaf_node("Unit")
    }
};
    let key_result = resolve_node_bounded(key_child, env.clone(), &module_name, depth.clone() + 1_i64);
    let key_resolved = key_result.resolved.clone();
    let key_diags = key_result.diagnostics.clone();
    let val_result = resolve_node_bounded(val_child, env.clone(), &module_name, depth.clone() + 1_i64);
    let val_resolved = val_result.resolved.clone();
    let val_diags = val_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: Rc::new(vec!(key_resolved, val_resolved)), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: v2_rt::concat(key_diags, val_diags) })
} else {
    if ({
    let __len_45 = n.children.clone().len();
    __len_45 as i64
}) == 1_i64 {
    match n.children.clone().first().cloned() {
    Some(el) => {
        {
    let el_result = resolve_node_bounded(el, env.clone(), &module_name, depth.clone() + 1_i64);
    let el_resolved = el_result.resolved.clone();
    let el_diags = el_result.diagnostics.clone();
    Rc::new(NodeResolveResult { resolved: Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: Rc::new(vec!(el_resolved)), connective: n.connective.clone(), collection_kind: n.collection_kind.clone(), params: n.params.clone(), inferred: n.inferred.clone(), return_cardinality: n.return_cardinality.clone(), uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: el_diags })
}
    }
    None => {
        Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
    }
}
} else {
    if ({
    let __len_44 = n.children.clone().len();
    __len_44 as i64
}) == 0_i64 {
    if is_recursive_type(env.clone(), &n.name) {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    match lookup_type(env.clone(), &n.name) {
    Some(resolved) => {
        {
    let structurally_resolved = if ((node_has_structure(resolved.clone()) == false) && (({
    let __len_43 = resolved.children.clone().len();
    __len_43 as i64
}) == 0_i64)) && (resolved.inferred.clone().is_some()) {
    match rt_node(resolved.clone()).as_ref() {
    NodeType::Typed { node: target, .. } => {
        target.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        resolved.clone()
    }
    NodeType::Untyped => {
        resolved.clone()
    }
}
} else {
    resolved.clone()
};
    let final_resolved = if node_is_optional(n.clone()) {
    with_optional_cardinality(structurally_resolved)
} else {
    structurally_resolved
};
    Rc::new(NodeResolveResult { resolved: final_resolved, diagnostics: Rc::new(Vec::new()) })
}
    }
    None => {
        if ((is_kernel_type(&n.name) || (n.name.clone() == "Dynamic")) || (n.name.clone() == "Error")) || (n.name.clone() == "Callable") {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(Vec::new()) })
} else {
    Rc::new(NodeResolveResult { resolved: n.clone(), diagnostics: Rc::new(vec!(diagnostic_node("error", &v2_rt::concat(v2_rt::concat("unresolved type '".to_string(), n.name.clone()), "'".to_string()), n.span.clone(), Some(module_name.to_string()), Some("unresolved_name".to_string())))) })
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
        resolve_node(inner.clone(), env, &module_name)
    }
    InferredNode::CompilerError { message: msg, span: sp, .. } => {
        Rc::new(NodeResolveResult { resolved: leaf_node("Error"), diagnostics: Rc::new(vec!(diagnostic_node("error", &msg, sp.clone(), Some(module_name.to_string()), None))) })
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
    Rc::new(FieldResult { field: Rc::new(Field { name: field.name.clone(), type_expr: type_resolved, cardinality: field.cardinality.clone(), default_value: match default_resolved.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, from_key: field.from_key.clone(), span: field.span.clone() }), diagnostics: v2_rt::concat(type_diags, default_diags) })
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
    Rc::new(ParamResult { param: Rc::new(Param { name: param.name.clone(), type_expr: type_resolved, default_value: match default_resolved.clone() {
    Some(result) => {
        Some(result.expr.clone())
    }
    None => {
        None
    }
}, span: param.span.clone() }), diagnostics: v2_rt::concat(type_diags, default_diags) })
}

pub fn resolve_resource_use(ru: Rc<ResourceUse>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ResourceUseResult> {
    let type_result = resolve_node(ru.resource.clone(), env, &module_name);
    let type_resolved = type_result.resolved.clone();
    let type_diags = type_result.diagnostics.clone();
    Rc::new(ResourceUseResult { resource_use: Rc::new(ResourceUse { name: ru.name.clone(), resource: type_resolved, span: ru.span.clone() }), diagnostics: type_diags })
}

pub fn resolve_named_arg(arg: Rc<NamedArg>, env: Rc<TypeEnv>, module_name: &str) -> Rc<NamedArgResolveResult> {
    let value_result = resolve_expr_types(arg.value.clone(), env, &module_name);
    let value_expr = value_result.expr.clone();
    let value_diags = value_result.diagnostics.clone();
    Rc::new(NamedArgResolveResult { arg: Rc::new(NamedArg { name: arg.name.clone(), value: value_expr }), diagnostics: value_diags })
}

pub fn resolve_field_init(field_init: Rc<FieldInit>, env: Rc<TypeEnv>, module_name: &str) -> Rc<FieldInitResolveResult> {
    let value_result = resolve_expr_types(field_init.value.clone(), env, &module_name);
    let value_expr = value_result.expr.clone();
    let value_diags = value_result.diagnostics.clone();
    Rc::new(FieldInitResolveResult { field_init: Rc::new(FieldInit { name: field_init.name.clone(), value: value_expr }), diagnostics: value_diags })
}

pub fn resolve_match_arm(arm: Rc<MatchArm>, env: Rc<TypeEnv>, module_name: &str) -> Rc<MatchArmResolveResult> {
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
}, body: body_expr }), diagnostics: v2_rt::concat(guard_diags, body_diags) })
}

pub fn resolve_string_part(part: Rc<StringPart>, env: Rc<TypeEnv>, module_name: &str) -> Rc<StringPartResolveResult> {
    match part.as_ref() {
    StringPart::Text { value, .. } => {
        Rc::new(StringPartResolveResult { part: Rc::new(StringPart::Text { value: value.clone() }), diagnostics: Rc::new(Vec::new()) })
    }
    StringPart::Interpolation { expr, .. } => {
        {
    let expr_result = resolve_expr_types(expr.clone(), env, &module_name);
    let resolved_expr = expr_result.expr.clone();
    let expr_diags = expr_result.diagnostics.clone();
    Rc::new(StringPartResolveResult { part: Rc::new(StringPart::Interpolation { expr: resolved_expr }), diagnostics: expr_diags })
}
    }
}
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
    Rc::new(TransportResolveResult { transport: make_transport_node(&transport.name, resolved_props, resolved_children, transport.span.clone()), diagnostics: v2_rt::concat(prop_diags, child_diags) })
}
}

pub fn resolve_expr_types(texpr: Rc<Node>, env: Rc<TypeEnv>, module_name: &str) -> Rc<ExprResolveResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: _, .. } => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    ExprData::ExprError { kind, message, .. } => {
        Rc::new(ExprResolveResult { expr: make_expr_error_node(kind.clone(), &message, texpr.span.clone()), diagnostics: Rc::new(vec!(diagnostic_node("error", &message, texpr.span.clone(), Some(module_name.to_string()), Some("cascade_error".to_string())))) })
    }
    ExprData::ExprVar { name: _, binding_kind: _, .. } => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    ExprData::ExprFieldAccess { field: _, summary: _, .. } => {
        {
    let r = match texpr.children.clone().first().cloned() {
    Some(base) => {
        resolve_expr_types(base.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: map_children(texpr.clone(), |child| r.expr.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::ExprCall { func, call_semantics: cs, .. } => {
        {
    let resolved_children = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in texpr.children.iter().cloned() {
        __mapped_0.push({
    let val = match __elem_1.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        __elem_1.clone()
    }
};
    let vr = resolve_expr_types(val.clone(), env.clone(), &module_name);
    make_arg_node(if __elem_1.name.clone() == "" {
    None
} else {
    Some(__elem_1.name.clone())
}, vr.expr.clone(), __elem_1.span.clone())
});
    }
    Rc::new(__mapped_0)
};
    let all_diags = {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in texpr.children.iter().cloned() {
        __flat_mapped_2.extend(({
    let val = match __elem_3.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        __elem_3.clone()
    }
};
    let vr = resolve_expr_types(val.clone(), env.clone(), &module_name);
    vr.diagnostics.clone()
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprCall { func: func.clone(), call_semantics: cs.clone() }), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprMethodCall { method, method_semantics: ms, .. } => {
        {
    let resolved_children = {
    let mut __mapped_7 = Vec::new();
    for __elem_8 in ({
    let mut __enumerated_4 = Vec::new();
    for (__idx_5, __elem_6) in texpr.children.clone().iter().enumerate() {
        __enumerated_4.push((__idx_5 as i64, __elem_6.clone()));
    }
    Rc::new(__enumerated_4)
}).iter().cloned() {
        __mapped_7.push({
    let idx = __elem_8.0.clone();
    let child = __elem_8.1.clone();
    if idx.clone() == 0_i64 {
    let rr = resolve_expr_types(child.clone(), env.clone(), &module_name);
    rr.expr.clone()
} else {
    let val = match child.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        child.clone()
    }
};
    let vr = resolve_expr_types(val.clone(), env.clone(), &module_name);
    make_arg_node(if child.name.clone() == "" {
    None
} else {
    Some(child.name.clone())
}, vr.expr.clone(), child.span.clone())
}
});
    }
    Rc::new(__mapped_7)
};
    let all_diags = {
    let mut __flat_mapped_12 = Vec::new();
    for __elem_13 in ({
    let mut __enumerated_9 = Vec::new();
    for (__idx_10, __elem_11) in texpr.children.clone().iter().enumerate() {
        __enumerated_9.push((__idx_10 as i64, __elem_11.clone()));
    }
    Rc::new(__enumerated_9)
}).iter().cloned() {
        __flat_mapped_12.extend(({
    let idx = __elem_13.0.clone();
    let child = __elem_13.1.clone();
    if idx.clone() == 0_i64 {
    let rr = resolve_expr_types(child.clone(), env.clone(), &module_name);
    rr.diagnostics.clone()
} else {
    let val = match child.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        child.clone()
    }
};
    let vr = resolve_expr_types(val.clone(), env.clone(), &module_name);
    vr.diagnostics.clone()
}
}).iter().cloned());
    }
    Rc::new(__flat_mapped_12)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprMethodCall { method: method.clone(), method_semantics: ms.clone() }), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprMatch => {
        {
    let resolved_children = {
    let mut __mapped_17 = Vec::new();
    for __elem_18 in ({
    let mut __enumerated_14 = Vec::new();
    for (__idx_15, __elem_16) in texpr.children.clone().iter().enumerate() {
        __enumerated_14.push((__idx_15 as i64, __elem_16.clone()));
    }
    Rc::new(__enumerated_14)
}).iter().cloned() {
        __mapped_17.push({
    let idx = __elem_18.0.clone();
    let child = __elem_18.1.clone();
    if idx.clone() == 0_i64 {
    let sr = resolve_expr_types(child.clone(), env.clone(), &module_name);
    sr.expr.clone()
} else {
    let arm_ch = child.children.clone();
    let has_guard = ({
    let __len_19 = arm_ch.clone().len();
    __len_19 as i64
}) == 2_i64;
    if has_guard.clone() {
    let guard_r = match arm_ch.clone().first().cloned() {
    Some(g) => {
        resolve_expr_types(g.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: child.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let body_r = match arm_ch.clone().get((1_i64) as usize).cloned() {
    Some(b) => {
        resolve_expr_types(b.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: child.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    make_arm_node(match child.match_pattern.clone() {
    Some(p) => {
        p.clone()
    }
    None => {
        Rc::new(MatchPattern::Wildcard)
    }
}, Some(guard_r.expr.clone()), body_r.expr.clone(), child.span.clone())
} else {
    let body_r = match arm_ch.clone().first().cloned() {
    Some(b) => {
        resolve_expr_types(b.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: child.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    make_arm_node(match child.match_pattern.clone() {
    Some(p) => {
        p.clone()
    }
    None => {
        Rc::new(MatchPattern::Wildcard)
    }
}, None, body_r.expr.clone(), child.span.clone())
}
}
});
    }
    Rc::new(__mapped_17)
};
    let all_diags = {
    let mut __flat_mapped_23 = Vec::new();
    for __elem_24 in ({
    let mut __enumerated_20 = Vec::new();
    for (__idx_21, __elem_22) in texpr.children.clone().iter().enumerate() {
        __enumerated_20.push((__idx_21 as i64, __elem_22.clone()));
    }
    Rc::new(__enumerated_20)
}).iter().cloned() {
        __flat_mapped_23.extend(({
    let idx = __elem_24.0.clone();
    let child = __elem_24.1.clone();
    if idx.clone() == 0_i64 {
    let sr = resolve_expr_types(child.clone(), env.clone(), &module_name);
    sr.diagnostics.clone()
} else {
    let arm_ch = child.children.clone();
    let has_guard = ({
    let __len_25 = arm_ch.clone().len();
    __len_25 as i64
}) == 2_i64;
    if has_guard.clone() {
    let guard_r = match arm_ch.clone().first().cloned() {
    Some(g) => {
        resolve_expr_types(g.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: child.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let body_r = match arm_ch.clone().get((1_i64) as usize).cloned() {
    Some(b) => {
        resolve_expr_types(b.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: child.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    v2_rt::concat(guard_r.diagnostics.clone(), body_r.diagnostics.clone())
} else {
    let body_r = match arm_ch.clone().first().cloned() {
    Some(b) => {
        resolve_expr_types(b.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: child.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    body_r.diagnostics.clone()
}
}
}).iter().cloned());
    }
    Rc::new(__flat_mapped_23)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprMatch), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprIf => {
        {
    let ch = texpr.children.clone();
    let cr = match ch.clone().first().cloned() {
    Some(c) => {
        resolve_expr_types(c.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let tr = match ch.clone().get((1_i64) as usize).cloned() {
    Some(t) => {
        resolve_expr_types(t, env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let er = match ch.clone().get((2_i64) as usize).cloned() {
    Some(e) => {
        Some(resolve_expr_types(e, env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let resolved_children = match er.clone() {
    Some(r) => {
        Rc::new(vec!(cr.expr.clone(), tr.expr.clone(), r.expr.clone()))
    }
    None => {
        Rc::new(vec!(cr.expr.clone(), tr.expr.clone()))
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprIf), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(v2_rt::concat(cr.diagnostics.clone(), tr.diagnostics.clone()), match er.clone() {
    Some(r) => {
        r.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
}) })
}
    }
    ExprData::ExprLet { name, .. } => {
        {
    let ch = texpr.children.clone();
    let vr = match ch.clone().first().cloned() {
    Some(v) => {
        resolve_expr_types(v.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let br = match ch.clone().get((1_i64) as usize).cloned() {
    Some(bd) => {
        Some(resolve_expr_types(bd, env.clone(), &module_name))
    }
    None => {
        None
    }
};
    let resolved_children = match br.clone() {
    Some(r) => {
        Rc::new(vec!(vr.expr.clone(), r.expr.clone()))
    }
    None => {
        Rc::new(vec!(vr.expr.clone()))
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprLet { name: name.clone() }), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(vr.diagnostics.clone(), match br.clone() {
    Some(r) => {
        r.diagnostics.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
}) })
}
    }
    ExprData::ExprRecordLit { type_name: tn, parent_enum: pe, .. } => {
        {
    let resolved_children = {
    let mut __mapped_26 = Vec::new();
    for __elem_27 in texpr.children.iter().cloned() {
        __mapped_26.push({
    let val = match __elem_27.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        __elem_27.clone()
    }
};
    let vr = resolve_expr_types(val.clone(), env.clone(), &module_name);
    make_field_init_node(&__elem_27.name, vr.expr.clone(), __elem_27.span.clone())
});
    }
    Rc::new(__mapped_26)
};
    let all_diags = {
    let mut __flat_mapped_28 = Vec::new();
    for __elem_29 in texpr.children.iter().cloned() {
        __flat_mapped_28.extend(({
    let val = match __elem_29.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        __elem_29.clone()
    }
};
    let vr = resolve_expr_types(val.clone(), env.clone(), &module_name);
    vr.diagnostics.clone()
}).iter().cloned());
    }
    Rc::new(__flat_mapped_28)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprRecordLit { type_name: tn.clone(), parent_enum: pe.clone() }), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprListLit => {
        {
    let el_results = {
    let mut __mapped_30 = Vec::new();
    for __elem_31 in texpr.children.iter().cloned() {
        __mapped_30.push(resolve_expr_types(__elem_31.clone(), env.clone(), &module_name));
    }
    Rc::new(__mapped_30)
};
    let resolved_children = {
    let mut __mapped_32 = Vec::new();
    for __elem_33 in el_results.iter().cloned() {
        __mapped_32.push(__elem_33.expr.clone());
    }
    Rc::new(__mapped_32)
};
    let all_diags = {
    let mut __flat_mapped_34 = Vec::new();
    for __elem_35 in el_results.iter().cloned() {
        __flat_mapped_34.extend(__elem_35.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_34)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprListLit), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprBinOp { op, .. } => {
        {
    let ch = texpr.children.clone();
    let lr = match ch.clone().first().cloned() {
    Some(l) => {
        resolve_expr_types(l, env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let rr = match ch.clone().get((1_i64) as usize).cloned() {
    Some(r) => {
        resolve_expr_types(r.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprBinOp { op: op.clone() }), Rc::new(vec!(lr.expr.clone(), rr.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(lr.diagnostics.clone(), rr.diagnostics.clone()) })
}
    }
    ExprData::ExprUnaryOp { op, .. } => {
        {
    let r = match texpr.children.clone().first().cloned() {
    Some(o) => {
        resolve_expr_types(o, env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprUnaryOp { op: op.clone() }), Rc::new(vec!(r.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::ExprLambda { params: p, semantics: s, .. } => {
        {
    let r = match texpr.children.clone().first().cloned() {
    Some(b) => {
        resolve_expr_types(b.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprLambda { params: p.clone(), semantics: s.clone() }), Rc::new(vec!(r.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: r.diagnostics.clone() })
}
    }
    ExprData::ExprStringInterp => {
        {
    let resolved_children = {
    let mut __mapped_36 = Vec::new();
    for __elem_37 in texpr.children.iter().cloned() {
        __mapped_36.push(match __elem_37.expr_data.as_ref() {
    ExprData::ExprLiteral { value: _, .. } => {
        __elem_37.clone()
    }
    _ => {
        match __elem_37.children.clone().first().cloned() {
    Some(inner) => {
        {
    let r = resolve_expr_types(inner.clone(), env.clone(), &module_name);
    make_interp_part_node(r.expr.clone(), __elem_37.span.clone())
}
    }
    None => {
        __elem_37.clone()
    }
}
    }
});
    }
    Rc::new(__mapped_36)
};
    let all_diags = {
    let mut __flat_mapped_38 = Vec::new();
    for __elem_39 in texpr.children.iter().cloned() {
        __flat_mapped_38.extend((match __elem_39.expr_data.as_ref() {
    ExprData::ExprLiteral { value: _, .. } => {
        Rc::new(Vec::new())
    }
    _ => {
        match __elem_39.children.clone().first().cloned() {
    Some(inner) => {
        {
    let r = resolve_expr_types(inner.clone(), env.clone(), &module_name);
    r.diagnostics.clone()
}
    }
    None => {
        Rc::new(Vec::new())
    }
}
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_38)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprStringInterp), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprBlock => {
        {
    let stmt_results = {
    let mut __mapped_40 = Vec::new();
    for __elem_41 in texpr.children.iter().cloned() {
        __mapped_40.push(resolve_expr_types(__elem_41, env.clone(), &module_name));
    }
    Rc::new(__mapped_40)
};
    let resolved_children = {
    let mut __mapped_42 = Vec::new();
    for __elem_43 in stmt_results.iter().cloned() {
        __mapped_42.push(__elem_43.expr.clone());
    }
    Rc::new(__mapped_42)
};
    let all_diags = {
    let mut __flat_mapped_44 = Vec::new();
    for __elem_45 in stmt_results.iter().cloned() {
        __flat_mapped_44.extend(__elem_45.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_44)
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprBlock), resolved_children.clone(), texpr.inferred.clone(), texpr.span.clone()), diagnostics: all_diags.clone() })
}
    }
    ExprData::ExprCast => {
        {
    let ch = texpr.children.clone();
    let r = match ch.clone().first().cloned() {
    Some(inner) => {
        resolve_expr_types(inner.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let tr = match ch.clone().get((1_i64) as usize).cloned() {
    Some(target) => {
        resolve_node(target, env.clone(), &module_name)
    }
    None => {
        Rc::new(NodeResolveResult { resolved: leaf_node("Unit"), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprCast), Rc::new(vec!(r.expr.clone(), tr.resolved.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(r.diagnostics.clone(), tr.diagnostics.clone()) })
}
    }
    ExprData::ExprForEach { variable, .. } => {
        {
    let ch = texpr.children.clone();
    let cr = match ch.clone().first().cloned() {
    Some(c) => {
        resolve_expr_types(c.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let br = match ch.clone().get((1_i64) as usize).cloned() {
    Some(b) => {
        resolve_expr_types(b.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprForEach { variable: variable.clone() }), Rc::new(vec!(cr.expr.clone(), br.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(cr.diagnostics.clone(), br.diagnostics.clone()) })
}
    }
    ExprData::ExprIndex => {
        {
    let ch = texpr.children.clone();
    let br = match ch.clone().first().cloned() {
    Some(base) => {
        resolve_expr_types(base.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let ir = match ch.clone().get((1_i64) as usize).cloned() {
    Some(index) => {
        resolve_expr_types(index, env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprIndex), Rc::new(vec!(br.expr.clone(), ir.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(br.diagnostics.clone(), ir.diagnostics.clone()) })
}
    }
    ExprData::ExprSlice => {
        {
    let ch = texpr.children.clone();
    let br = match ch.clone().first().cloned() {
    Some(base) => {
        resolve_expr_types(base.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let sr = match ch.clone().get((1_i64) as usize).cloned() {
    Some(start) => {
        resolve_expr_types(start, env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    let er = match ch.clone().get((2_i64) as usize).cloned() {
    Some(end_e) => {
        resolve_expr_types(end_e, env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprSlice), Rc::new(vec!(br.expr.clone(), sr.expr.clone(), er.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: v2_rt::concat(v2_rt::concat(br.diagnostics.clone(), sr.diagnostics.clone()), er.diagnostics.clone()) })
}
    }
    ExprData::ExprReturn => {
        {
    let r = match texpr.children.clone().first().cloned() {
    Some(inner) => {
        resolve_expr_types(inner.clone(), env.clone(), &module_name)
    }
    None => {
        Rc::new(ExprResolveResult { expr: texpr.clone(), diagnostics: Rc::new(Vec::new()) })
    }
};
    Rc::new(ExprResolveResult { expr: make_expr_node(Rc::new(ExprData::ExprReturn), Rc::new(vec!(r.expr.clone())), texpr.inferred.clone(), texpr.span.clone()), diagnostics: r.diagnostics.clone() })
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
        let ret_result = resolve_optional_node(item.inferred.clone(), env.clone(), &module_name);
        let ret_resolved = ret_result.resolved.clone();
        let ret_diags = ret_result.diagnostics.clone();
        let resolved_ret = if item.inferred.clone().is_none() {
    None
} else {
    Some(Rc::new(InferredNode::Resolved { node: ret_resolved }))
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
        Rc::new(ItemResult { item: Rc::new(Node { name: item.name.clone(), span: item.span.clone(), children: resolved_children, params: resolved_params, inferred: resolved_ret, return_cardinality: item.return_cardinality.clone(), uses: resolved_uses, body: resolved_body, connective: item.connective.clone(), transport: resolved_transport, collection_kind: item.collection_kind.clone(), properties: resolved_props, type_annotation: resolved_anno, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), diagnostics: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(param_diags, ret_diags), use_diags), body_diags), anno_diags), transport_diags), prop_diags), child_diags) })
    })
}

