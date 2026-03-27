use crate::v2_core::*;
use crate::infer_types::*;
use crate::infer_items::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UniqueAccum {
    pub seen: Rc<HashMap<String, bool>>,
    pub result: Rc<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpEntry {
    pub name: String,
    pub outputs: Rc<Vec<Rc<Field>>>,
    pub params: Rc<Vec<Rc<Param>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceMethodResult {
    pub result_type: Rc<Node>,
    pub op_params: Rc<Vec<Rc<Param>>>,
}

pub fn is_typed_service_call_receiver(receiver: Rc<Node>) -> bool {
    match receiver.expr_data.as_ref() {
    ExprData::ExprFieldAccess { field: f, summary: _, .. } => {
        {
    let b = field_access_base(receiver.clone());
    match b.expr_data.as_ref() {
    ExprData::ExprVar { name: _, binding_kind: _, .. } => {
        match ({
    let mut __chars_0 = Vec::new();
    for __ch_1 in f.clone().chars() {
        __chars_0.push(__ch_1.to_string());
    }
    Rc::new(__chars_0)
}).first().cloned() {
    Some(ch) => {
        (ch.clone() >= "A".to_string()) && (ch.clone() <= "Z".to_string())
    }
    None => {
        false
    }
}
    }
    _ => {
        false
    }
}
}
    }
    _ => {
        false
    }
}
}

pub fn extract_typed_service_name(receiver: Rc<Node>) -> Option<String> {
    match receiver.expr_data.as_ref() {
    ExprData::ExprFieldAccess { field: f, summary: _, .. } => {
        {
    let b = field_access_base(receiver.clone());
    match b.expr_data.as_ref() {
    ExprData::ExprVar { name: ns, binding_kind: _, .. } => {
        Some(v2_rt::concat(v2_rt::concat(ns.clone(), ".".to_string()), f.clone()))
    }
    _ => {
        None
    }
}
}
    }
    _ => {
        None
    }
}
}

pub fn collect_typed_service_calls(texpr: Rc<Node>) -> Rc<Vec<String>> {
    let result = collect_typed_service_calls_into(texpr, Rc::new(UniqueAccum { seen: Rc::new(std::collections::HashMap::new()), result: Rc::new(Vec::new()) }));
    result.result.clone()
}

pub fn collect_typed_service_calls_into(texpr: Rc<Node>, acc: Rc<UniqueAccum>) -> Rc<UniqueAccum> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let this_acc = match texpr.expr_data.as_ref() {
    ExprData::ExprMethodCall { method: _, method_semantics: _, .. } => {
        {
    let r = method_receiver(texpr.clone());
    if is_typed_service_call_receiver(r.clone()) {
    match extract_typed_service_name(r.clone()) {
    Some(service_name) => {
        if emit_map_has(acc.seen.clone(), &service_name) {
    acc.clone()
} else {
    Rc::new(UniqueAccum { seen: {
    let __rc_1 = acc.seen.clone();
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert(service_name.clone(), true);
    Rc::new(__map_ins_0)
}, result: {
    let __rc_3 = acc.result.clone();
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(service_name.clone());
    Rc::new(__appended_2)
} })
}
    }
    None => {
        acc.clone()
    }
}
} else {
    acc.clone()
}
}
    }
    _ => {
        acc.clone()
    }
};
        let result = {
    let mut __acc_4 = this_acc;
    for __elem_5 in texpr.children.iter().cloned() {
        __acc_4 = collect_typed_service_calls_into(__elem_5, __acc_4);
    }
    __acc_4
};
        result
    })
}

pub fn collect_called_func_names_into(texpr: Rc<Node>, acc: Rc<UniqueAccum>) -> Rc<UniqueAccum> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let this_acc = match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, call_semantics: _, .. } => {
        if emit_map_has(acc.seen.clone(), &f) {
    acc.clone()
} else {
    Rc::new(UniqueAccum { seen: {
    let __rc_1 = acc.seen.clone();
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert(f.clone(), true);
    Rc::new(__map_ins_0)
}, result: {
    let __rc_3 = acc.result.clone();
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(f.clone());
    Rc::new(__appended_2)
} })
}
    }
    _ => {
        acc.clone()
    }
};
        let result = {
    let mut __acc_4 = this_acc;
    for __elem_5 in texpr.children.iter().cloned() {
        __acc_4 = collect_called_func_names_into(__elem_5, __acc_4);
    }
    __acc_4
};
        result
    })
}

pub fn collect_called_func_names(texpr: Rc<Node>) -> Rc<Vec<String>> {
    let result = collect_called_func_names_into(texpr, Rc::new(UniqueAccum { seen: Rc::new(std::collections::HashMap::new()), result: Rc::new(Vec::new()) }));
    result.result.clone()
}

pub fn expand_transitive_services_once(modules: Rc<Vec<Rc<TypedModule>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> Rc<HashMap<String, Rc<ItemInfo>>> {
    let all_items = {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in modules.iter().cloned() {
        __flat_mapped_0.extend(__elem_1.items.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_0)
};
    {
    let mut __acc_2 = registry;
    for __elem_3 in all_items.iter().cloned() {
        __acc_2 = match __acc_2.clone().get(&__elem_3.name.clone()).cloned() {
    Some(info) => {
        {
    let is_not_func = info.kind.clone() != ItemKind::FuncItem;
    let has_no_body = __elem_3.body.clone().is_none();
    if is_not_func.clone() {
    __acc_2.clone()
} else {
    if has_no_body.clone() {
    __acc_2.clone()
} else {
    let called = collect_called_func_names(__elem_3.body.clone().unwrap());
    let extra = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in called.iter().cloned() {
        __flat_mapped_4.extend((match __acc_2.clone().get(&__elem_5.clone()).cloned() {
    Some(callee_info) => {
        callee_info.service_names.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    let merged = {
    let mut __acc_6 = info.service_names.clone();
    for __elem_7 in extra.iter().cloned() {
        __acc_6 = {
let __cond = {
    let mut __any_10 = false;
    for __elem_11 in __acc_6.iter().cloned() {
        if __elem_11.clone() == __elem_7.clone() {
    __any_10 = true;
    break;
};
    }
    __any_10
};
if __cond {
    __acc_6.clone()
} else {
    {
    let __rc_9 = __acc_6;
    let mut __appended_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __appended_8.push(__elem_7.clone());
    Rc::new(__appended_8)
}
}
};
    }
    __acc_6
};
    let same_count = ({
    let __len_12 = merged.clone().len();
    __len_12 as i64
}) == ({
    let __len_13 = info.service_names.clone().len();
    __len_13 as i64
});
    if same_count.clone() {
    __acc_2.clone()
} else {
    {
    let __rc_15 = __acc_2;
    let mut __map_ins_14 = Rc::try_unwrap(__rc_15).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_14.insert(__elem_3.name.clone(), Rc::new(ItemInfo { name: info.name.clone(), kind: info.kind.clone(), service_names: merged.clone(), resource_names: info.resource_names.clone(), params: info.params.clone(), is_self_recursive: info.is_self_recursive.clone(), has_non_tail_self_call: info.has_non_tail_self_call.clone() }));
    Rc::new(__map_ins_14)
}
}
}
}
}
    }
    None => {
        __acc_2.clone()
    }
};
    }
    __acc_2
}
}

pub fn total_service_count(registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> i64 {
    {
    let mut __acc_4 = 0_i64;
    for __elem_5 in ({
    let __rc_0 = registry;
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 += {
    let __len_6 = __elem_5.service_names.clone().len();
    __len_6 as i64
};
    }
    __acc_4
}
}

pub fn expand_transitive_services(modules: Rc<Vec<Rc<TypedModule>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, remaining_passes: i64) -> Rc<HashMap<String, Rc<ItemInfo>>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_modules = modules;
        let mut __tco_p_registry = registry;
        let mut __tco_p_remaining_passes = remaining_passes;
        loop {
            let modules = __tco_p_modules;
            let registry = __tco_p_registry;
            let remaining_passes = __tco_p_remaining_passes;
            if remaining_passes.clone() <= 0_i64 {
    break registry.clone();
} else {
    let before = total_service_count(registry.clone());
    let next = expand_transitive_services_once(modules.clone(), registry.clone());
    let after = total_service_count(next.clone());
    if before == after {
    break registry.clone();
} else {
     {
        let __tco_0 = modules.clone();
        let __tco_1 = next.clone();
        let __tco_2 = remaining_passes.clone() - 1_i64;
        __tco_p_modules = __tco_0;
        __tco_p_registry = __tco_1;
        __tco_p_remaining_passes = __tco_2;
        continue;
    }

};
};
        }
    })
}

pub fn check_service_field_access_node(base_type: Rc<Node>, field: &str, service_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>) -> Option<Rc<Node>> {
    if (node_has_structure(base_type.clone()) == false) && (({
    let __len_0 = base_type.children.clone().len();
    __len_0 as i64
}) == 0_i64) {
    let path = v2_rt::concat(v2_rt::concat(base_type.name.clone(), ".".to_string()), field.to_string());
    match service_registry.get(&path.clone()).cloned() {
    Some(_) => {
        Some(leaf_node(&path))
    }
    None => {
        None
    }
}
} else {
    None
}
}

pub fn check_service_method_call_node(receiver_type: Rc<Node>, method: &str, service_registry: Rc<HashMap<String, Rc<Vec<Rc<OpEntry>>>>>) -> Option<Rc<ServiceMethodResult>> {
    if (node_has_structure(receiver_type.clone()) == false) && (({
    let __len_5 = receiver_type.children.clone().len();
    __len_5 as i64
}) == 0_i64) {
    match service_registry.get(&receiver_type.name.clone()).cloned() {
    Some(ops) => {
        {
    let matching = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in ops.iter().cloned() {
        if __elem_1.name.clone() == method {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    match matching.first().cloned() {
    Some(op) => {
        if ({
    let __len_4 = op.outputs.clone().len();
    __len_4 as i64
}) == 0_i64 {
    Some(Rc::new(ServiceMethodResult { result_type: leaf_node("Unit"), op_params: op.params.clone() }))
} else {
    Some(Rc::new(ServiceMethodResult { result_type: Rc::new(Node { name: "".to_string(), span: no_span(), children: {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in op.outputs.iter().cloned() {
        __mapped_2.push(Rc::new(Node { name: __elem_3.name.clone(), span: __elem_3.span.clone(), children: Rc::new(Vec::new()), connective: None, collection_kind: None, params: Rc::new(Vec::new()), inferred: Some(Rc::new(InferredNode::Resolved { node: __elem_3.type_expr.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }));
    }
    Rc::new(__mapped_2)
}, connective: Some(Connective::Conj), collection_kind: None, params: Rc::new(Vec::new()), inferred: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, is_self_recursive: false, has_non_tail_self_call: false, match_pattern: None, expr_data: Rc::new(ExprData::NoExprData) }), op_params: op.params.clone() }))
}
    }
    None => {
        None
    }
}
}
    }
    None => {
        None
    }
}
} else {
    None
}
}

pub fn service_op_entry(child: Rc<Node>) -> Rc<OpEntry> {
    Rc::new(OpEntry { name: child.name.clone(), outputs: inferred_to_outputs(child.inferred.clone(), child.span.clone()), params: child.params.clone() })
}

