use crate::v2_core::*;
use crate::infer_types::*;
use crate::infer_env::*;
use crate::infer_emit_info::*;
use crate::infer_method::*;
use crate::infer_sigs::*;
use crate::infer_service::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KnownMethodResolution {
    pub semantics: Option<Rc<MethodSemantics>>,
    pub result_type: Option<Rc<Node>>,
}

pub fn lookup_in_scope(locals: Rc<HashMap<String, Rc<TypeBinding>>>, name: &str) -> Option<Rc<Node>> {
    match locals.clone().get(&name.to_string()).cloned() {
    Some(binding) => {
        Some(binding.resolved.clone())
    }
    None => {
        None
    }
}
}

pub fn lookup_func_sig(func_env: Rc<ResolvedFuncEnv>, name: &str) -> Option<Rc<ResolvedFuncSig>> {
    func_env.signatures.clone().get(&name.to_string()).cloned()
}

pub fn lookup_field_type_node(n: Rc<Node>, field_name: &str) -> Option<Rc<Node>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if n.return_cardinality == Cardinality::CardOptional {
    let inner = with_required_cardinality(n.clone());
    if field_name == "value" {
    Some(inner.clone())
} else {
    match lookup_field_type_node(inner.clone(), &field_name) {
    Some(inner_result) => {
        Some(with_optional_cardinality(inner_result.clone()))
    }
    None => {
        None
    }
}
}
} else {
    if n.connective != Connective::NoConnective {
    if n.connective == Connective::Conj {
    match {
    let mut __found_2 = None;
    for __elem_3 in n.children.iter().cloned() {
        if __elem_3.name.clone() == field_name {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(field_child) => {
        Some(child_inferred_or_name(field_child.clone()))
    }
    None => {
        None
    }
}
} else {
    lookup_coproduct_common_field_node(n.children.clone(), &field_name)
}
} else {
    None
}
}
    })
}

pub fn lookup_coproduct_common_field_node(variants: Rc<Vec<Rc<Node>>>, field_name: &str) -> Option<Rc<Node>> {
    let found_in_all = {
    let mut __all_0 = true;
    for __elem_1 in variants.iter().cloned() {
        if !({
    let mut __any_2 = false;
    for __elem_3 in __elem_1.children.iter().cloned() {
        if __elem_3.name.clone() == field_name {
    __any_2 = true;
    break;
};
    }
    __any_2
}) {
    __all_0 = false;
    break;
};
    }
    __all_0
};
    let first_field = if found_in_all.clone() {
    match variants.clone().first().cloned() {
    Some(first_variant) => {
        {
    let mut __found_6 = None;
    for __elem_7 in first_variant.children.iter().cloned() {
        if __elem_7.name.clone() == field_name {
    __found_6 = Some(__elem_7);
    break;
};
    }
    __found_6
}
    }
    None => {
        None
    }
}
} else {
    None
};
    match first_field.clone() {
    Some(field_child) => {
        Some(child_inferred_or_name(field_child.clone()))
    }
    None => {
        None
    }
}
}

pub fn resolve_scrutinee_type_node(env: Rc<TypeEnv>, n: Rc<Node>) -> Rc<Node> {
    resolve_scrutinee_type_node_seen(env.clone(), n.clone(), Rc::new(std::collections::HashMap::new()))
}

pub fn resolve_scrutinee_type_node_seen(env: Rc<TypeEnv>, n: Rc<Node>, seen: Rc<HashMap<String, bool>>) -> Rc<Node> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let normed = normalize_access_type_node(n.clone());
        if ((normed.connective != Connective::NoConnective) == false) && (({
    let __len_6 = normed.children.clone().len();
    __len_6 as i64
}) == 0_i64) {
    let canonical = normed.name.clone();
    if normed.inferred.clone().is_some() {
    let next_seen = if canonical.clone() == "" {
    seen.clone()
} else {
    {
    let __rc_1 = seen;
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert(canonical.clone(), true);
    Rc::new(__map_ins_0)
}
};
    match rt_node(normed.clone()).as_ref() {
    NodeType::Typed { node: target, .. } => {
        if (((target.name.clone() == normed.name.clone()) && (target.inferred.clone().is_none())) && ((target.connective != Connective::NoConnective) == false)) && (({
    let __len_2 = target.children.clone().len();
    __len_2 as i64
}) == 0_i64) {
    normed.clone()
} else {
    resolve_scrutinee_type_node_seen(env.clone(), target.clone(), next_seen.clone())
}
    }
    NodeType::InferError { message: _, span: _, .. } => {
        normed.clone()
    }
    NodeType::Untyped => {
        normed.clone()
    }
}
} else {
    if (canonical.clone() != "") && emit_map_has(seen.clone(), &canonical) {
    leaf_node(&normed.name)
} else {
    let next_seen = if canonical.clone() == "" {
    seen.clone()
} else {
    {
    let __rc_4 = seen;
    let mut __map_ins_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_3.insert(canonical.clone(), true);
    Rc::new(__map_ins_3)
}
};
    match lookup_type(env.clone(), &normed.name) {
    Some(resolved) => {
        if (((resolved.name.clone() == normed.name.clone()) && (resolved.inferred.clone().is_none())) && ((resolved.connective != Connective::NoConnective) == false)) && (({
    let __len_5 = resolved.children.clone().len();
    __len_5 as i64
}) == 0_i64) {
    normed.clone()
} else {
    let result = resolve_scrutinee_type_node_seen(env.clone(), resolved.clone(), next_seen.clone());
    if normed.return_cardinality == Cardinality::CardOptional {
    with_optional_cardinality(result.clone())
} else {
    result.clone()
}
}
    }
    None => {
        normed.clone()
    }
}
}
}
} else {
    normed.clone()
}
    })
}

pub fn map_value_type_in_env(type_node: Rc<Node>, env: Rc<TypeEnv>) -> Option<Rc<Node>> {
    let normed = normalize_access_type_node(type_node.clone());
    let resolved = resolve_scrutinee_type_node(env.clone(), normed.clone());
    let map_type = normalize_access_type_node(resolved.clone());
    if (map_type.collection_kind == CollectionKind::MapKind) && (({
    let __len_0 = map_type.children.clone().len();
    __len_0 as i64
}) >= 2_i64) {
    match map_type.children.clone().get((1_i64) as usize).cloned() {
    Some(value_type) => {
        Some(value_type.clone())
    }
    None => {
        None
    }
}
} else {
    None
}
}

pub fn field_summary_for_type(base_type: Rc<Node>, env: Rc<TypeEnv>, field: &str) -> Option<Rc<FieldSummary>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let resolved = resolve_scrutinee_type_node(env.clone(), base_type.clone());
        let normed = normalize_access_type_node(resolved.clone());
        let normed_opt = normed.return_cardinality == Cardinality::CardOptional;
        if (field == "value") && normed_opt.clone() {
    Some(Rc::new(FieldSummary { access_style: FieldAccessStyle::OptionalUnwrap, value_shape: FieldValueShape::PlainValue }))
} else {
    if normed_opt.clone() {
    let inner = with_required_cardinality(normed.clone());
    match field_summary_for_type(inner.clone(), env.clone(), &field) {
    Some(inner_summary) => {
        Some(Rc::new(FieldSummary { access_style: inner_summary.access_style.clone(), value_shape: FieldValueShape::OptionalValue }))
    }
    None => {
        None
    }
}
} else {
    if (resolved.connective != Connective::NoConnective) == false {
    None
} else {
    if resolved.connective == Connective::Conj {
    build_struct_field_summaries(resolved.children.clone()).get(&field.to_string()).cloned()
} else {
    build_enum_field_summaries(resolved.children.clone()).get(&field.to_string()).cloned()
}
}
}
}
    })
}

pub fn resolve_known_method_node(receiver: Rc<Node>, receiver_type: Rc<Node>, method_name: &str, fold_accumulator_type: Option<Rc<Node>>, service_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>) -> Rc<KnownMethodResolution> {
    match check_service_method_call_node(receiver_type.clone(), &method_name, service_registry.clone()) {
    Some(svc_result) => {
        Rc::new(KnownMethodResolution { semantics: Some(Rc::new(MethodSemantics::ServiceMethodSemantics { service_name: receiver_type.name.clone(), op_params: svc_result.op_params.clone() })), result_type: Some(svc_result.result_type.clone()) })
    }
    None => {
        match classify_reconciled_intrinsic_method(&method_name) {
    Some(intrinsic) => {
        Rc::new(KnownMethodResolution { semantics: Some(Rc::new(MethodSemantics::IntrinsicMethodSemantics { intrinsic: intrinsic.clone(), fold_accumulator_type: fold_accumulator_type.clone() })), result_type: infer_intrinsic_method_type_node(receiver_type.clone(), intrinsic.clone(), fold_accumulator_type.clone()) })
    }
    None => {
        match classify_runtime_bridge_method(&method_name) {
    Some(bridge_method) => {
        Rc::new(KnownMethodResolution { semantics: Some(Rc::new(MethodSemantics::RuntimeBridgeSemantics { method: bridge_method.clone() })), result_type: infer_runtime_bridge_method_type_node(receiver_type.clone(), bridge_method.clone()) })
    }
    None => {
        Rc::new(KnownMethodResolution { semantics: None, result_type: None })
    }
}
    }
}
    }
}
}

