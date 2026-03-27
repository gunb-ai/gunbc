use crate::v2_core::*;
use crate::infer_types::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn classify_reconciled_intrinsic_method(method: &str) -> Option<IntrinsicMethod> {
    if method == "count" {
    Some(IntrinsicMethod::MethodCount)
} else {
    if method == "join" {
    Some(IntrinsicMethod::MethodJoin)
} else {
    if method == "split" {
    Some(IntrinsicMethod::MethodSplit)
} else {
    if method == "last" {
    Some(IntrinsicMethod::MethodLast)
} else {
    if method == "first" {
    Some(IntrinsicMethod::MethodFirst)
} else {
    if method == "enumerate" {
    Some(IntrinsicMethod::MethodEnumerate)
} else {
    if method == "chars" {
    Some(IntrinsicMethod::MethodChars)
} else {
    if method == "string_contains" {
    Some(IntrinsicMethod::MethodStringContains)
} else {
    if method == "concat" {
    Some(IntrinsicMethod::MethodConcat)
} else {
    if method == "map" {
    Some(IntrinsicMethod::MethodMap)
} else {
    if method == "filter" {
    Some(IntrinsicMethod::MethodFilter)
} else {
    if method == "any" {
    Some(IntrinsicMethod::MethodAny)
} else {
    if method == "all" {
    Some(IntrinsicMethod::MethodAll)
} else {
    if method == "flat_map" {
    Some(IntrinsicMethod::MethodFlatMap)
} else {
    if method == "skip" {
    Some(IntrinsicMethod::MethodSkip)
} else {
    if method == "take" {
    Some(IntrinsicMethod::MethodTake)
} else {
    if method == "fold" {
    Some(IntrinsicMethod::MethodFold)
} else {
    if method == "sort_by" {
    Some(IntrinsicMethod::MethodSortBy)
} else {
    if method == "append" {
    Some(IntrinsicMethod::MethodAppend)
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

pub fn classify_runtime_bridge_method(method: &str) -> Option<RuntimeBridgeMethod> {
    if method == "get" {
    Some(RuntimeBridgeMethod::BridgeGet)
} else {
    if method == "with" {
    Some(RuntimeBridgeMethod::BridgeWith)
} else {
    if method == "list_push" {
    Some(RuntimeBridgeMethod::BridgeListPush)
} else {
    if method == "map_insert" {
    Some(RuntimeBridgeMethod::BridgeMapInsert)
} else {
    if method == "map_merge" {
    Some(RuntimeBridgeMethod::BridgeMapMerge)
} else {
    if method == "map_get" {
    Some(RuntimeBridgeMethod::BridgeMapGet)
} else {
    if method == "map_has" {
    Some(RuntimeBridgeMethod::BridgeMapHas)
} else {
    if method == "emit_map_has" {
    Some(RuntimeBridgeMethod::BridgeEmitMapHas)
} else {
    if method == "map_values" {
    Some(RuntimeBridgeMethod::BridgeMapValues)
} else {
    if method == "map_keys" {
    Some(RuntimeBridgeMethod::BridgeMapKeys)
} else {
    if method == "map_contains_key" {
    Some(RuntimeBridgeMethod::BridgeMapContainsKey)
} else {
    if method == "char_at" {
    Some(RuntimeBridgeMethod::BridgeCharAt)
} else {
    if method == "string_at" {
    Some(RuntimeBridgeMethod::BridgeStringAt)
} else {
    if method == "string_length" {
    Some(RuntimeBridgeMethod::BridgeStringLength)
} else {
    if method == "length" {
    Some(RuntimeBridgeMethod::BridgeLength)
} else {
    if method == "starts_with" {
    Some(RuntimeBridgeMethod::BridgeStartsWith)
} else {
    if method == "ends_with" {
    Some(RuntimeBridgeMethod::BridgeEndsWith)
} else {
    if method == "to_string" {
    Some(RuntimeBridgeMethod::BridgeToString)
} else {
    if method == "trim" {
    Some(RuntimeBridgeMethod::BridgeTrim)
} else {
    if method == "to_lower" {
    Some(RuntimeBridgeMethod::BridgeToLower)
} else {
    if method == "to_upper" {
    Some(RuntimeBridgeMethod::BridgeToUpper)
} else {
    if method == "replace" {
    Some(RuntimeBridgeMethod::BridgeReplace)
} else {
    if method == "substring" {
    Some(RuntimeBridgeMethod::BridgeSubstring)
} else {
    if method == "to_int" {
    Some(RuntimeBridgeMethod::BridgeToInt)
} else {
    if method == "empty_map" {
    Some(RuntimeBridgeMethod::BridgeEmptyMap)
} else {
    if method == "contains" {
    Some(RuntimeBridgeMethod::BridgeContains)
} else {
    if method == "reverse" {
    Some(RuntimeBridgeMethod::BridgeReverse)
} else {
    if method == "lookup" {
    Some(RuntimeBridgeMethod::BridgeLookup)
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

pub fn infer_intrinsic_method_type_node(receiver_type: Rc<Node>, intrinsic: IntrinsicMethod, fold_accumulator_type: Option<Rc<Node>>) -> Option<Rc<Node>> {
    let elem_node = method_receiver_element_node(receiver_type.clone());
    match intrinsic {
    IntrinsicMethod::MethodMap => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodFilter => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodEnumerate => {
        Some(container_node("List", tuple_node(leaf_node("Int"), elem_node.clone())))
    }
    IntrinsicMethod::MethodFirst => {
        Some(with_optional_cardinality(elem_node.clone()))
    }
    IntrinsicMethod::MethodLast => {
        Some(with_optional_cardinality(elem_node.clone()))
    }
    IntrinsicMethod::MethodCount => {
        Some(leaf_node("Int"))
    }
    IntrinsicMethod::MethodJoin => {
        Some(leaf_node("String"))
    }
    IntrinsicMethod::MethodAny => {
        Some(leaf_node("Bool"))
    }
    IntrinsicMethod::MethodAll => {
        Some(leaf_node("Bool"))
    }
    IntrinsicMethod::MethodSplit => {
        Some(container_node("List", leaf_node("String")))
    }
    IntrinsicMethod::MethodChars => {
        Some(container_node("List", leaf_node("String")))
    }
    IntrinsicMethod::MethodFlatMap => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodSkip => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodTake => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodFold => {
        fold_accumulator_type
    }
    IntrinsicMethod::MethodSortBy => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodAppend => {
        Some(receiver_type.clone())
    }
    IntrinsicMethod::MethodStringContains => {
        Some(leaf_node("Bool"))
    }
    IntrinsicMethod::MethodConcat => {
        Some(receiver_type.clone())
    }
}
}

pub fn infer_runtime_bridge_method_type_node(receiver_type: Rc<Node>, method: RuntimeBridgeMethod) -> Option<Rc<Node>> {
    let elem_node = method_receiver_element_node(receiver_type.clone());
    match method {
    RuntimeBridgeMethod::BridgeGet => {
        Some(with_optional_cardinality(elem_node.clone()))
    }
    RuntimeBridgeMethod::BridgeWith => {
        Some(receiver_type.clone())
    }
    RuntimeBridgeMethod::BridgeListPush => {
        Some(receiver_type.clone())
    }
    RuntimeBridgeMethod::BridgeMapInsert => {
        Some(receiver_type.clone())
    }
    RuntimeBridgeMethod::BridgeMapMerge => {
        Some(receiver_type.clone())
    }
    RuntimeBridgeMethod::BridgeMapGet => {
        Some(with_optional_cardinality(elem_node.clone()))
    }
    RuntimeBridgeMethod::BridgeMapHas => {
        Some(leaf_node("Bool"))
    }
    RuntimeBridgeMethod::BridgeEmitMapHas => {
        Some(leaf_node("Bool"))
    }
    RuntimeBridgeMethod::BridgeMapContainsKey => {
        Some(leaf_node("Bool"))
    }
    RuntimeBridgeMethod::BridgeContains => {
        Some(leaf_node("Bool"))
    }
    RuntimeBridgeMethod::BridgeMapValues => {
        Some(container_node("List", elem_node.clone()))
    }
    RuntimeBridgeMethod::BridgeMapKeys => {
        Some(container_node("List", leaf_node("String")))
    }
    RuntimeBridgeMethod::BridgeCharAt => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeStringAt => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeStringLength => {
        Some(leaf_node("Int"))
    }
    RuntimeBridgeMethod::BridgeLength => {
        Some(leaf_node("Int"))
    }
    RuntimeBridgeMethod::BridgeStartsWith => {
        Some(leaf_node("Bool"))
    }
    RuntimeBridgeMethod::BridgeEndsWith => {
        Some(leaf_node("Bool"))
    }
    RuntimeBridgeMethod::BridgeToString => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeTrim => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeToLower => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeToUpper => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeReplace => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeSubstring => {
        Some(leaf_node("String"))
    }
    RuntimeBridgeMethod::BridgeToInt => {
        Some(leaf_node("Int"))
    }
    RuntimeBridgeMethod::BridgeEmptyMap => {
        Some(bare_map_node())
    }
    RuntimeBridgeMethod::BridgeLookup => {
        Some(with_optional_cardinality(elem_node.clone()))
    }
    RuntimeBridgeMethod::BridgeReverse => {
        Some(receiver_type.clone())
    }
}
}

pub fn infer_builtin_call_type(name: &str) -> Option<Rc<Node>> {
    if ((name == "string_length") || (name == "code_point")) || (name == "to_int") {
    Some(leaf_node("Int"))
} else {
    if name == "parse_int" {
    Some(with_optional_cardinality(leaf_node("Int")))
} else {
    if ((((name == "char_at") || (name == "substring")) || (name == "from_code_point")) || (name == "to_string")) || (name == "concat") {
    Some(leaf_node("String"))
} else {
    if (((name == "scan_while") || (name == "scan_string_end")) || (name == "scan_to_eol")) || (name == "skip_horizontal_ws") {
    Some(leaf_node("Int"))
} else {
    if (name == "lookup") || (name == "map_get") {
    Some(with_optional_cardinality(leaf_node("Dynamic")))
} else {
    if ((name == "map_insert") || (name == "map_merge")) || (name == "with") {
    Some(bare_map_node())
} else {
    if ((name == "map_contains_key") || (name == "map_has")) || (name == "emit_map_has") {
    Some(leaf_node("Bool"))
} else {
    if (((name == "map_keys") || (name == "map_values")) || (name == "reverse")) || (name == "list_push") {
    Some(container_node("List", leaf_node("Dynamic")))
} else {
    if name == "Some" {
    Some(with_optional_cardinality(leaf_node("Dynamic")))
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

pub fn resolve_builtin_call_type(name: &str) -> Rc<Node> {
    match infer_builtin_call_type(&name) {
    Some(v) => {
        v
    }
    None => {
        leaf_node("Unit")
    }
}
}

pub fn intrinsic_method_index() -> Rc<HashMap<String, IntrinsicMethod>> {
    let m = Rc::new(std::collections::HashMap::new());
    let m = {
    let __rc_1 = m;
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert("count".to_string(), IntrinsicMethod::MethodCount);
    Rc::new(__map_ins_0)
};
    let m = {
    let __rc_3 = m;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert("join".to_string(), IntrinsicMethod::MethodJoin);
    Rc::new(__map_ins_2)
};
    let m = {
    let __rc_5 = m;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert("split".to_string(), IntrinsicMethod::MethodSplit);
    Rc::new(__map_ins_4)
};
    let m = {
    let __rc_7 = m;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert("last".to_string(), IntrinsicMethod::MethodLast);
    Rc::new(__map_ins_6)
};
    let m = {
    let __rc_9 = m;
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert("first".to_string(), IntrinsicMethod::MethodFirst);
    Rc::new(__map_ins_8)
};
    let m = {
    let __rc_11 = m;
    let mut __map_ins_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_10.insert("enumerate".to_string(), IntrinsicMethod::MethodEnumerate);
    Rc::new(__map_ins_10)
};
    let m = {
    let __rc_13 = m;
    let mut __map_ins_12 = Rc::try_unwrap(__rc_13).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_12.insert("chars".to_string(), IntrinsicMethod::MethodChars);
    Rc::new(__map_ins_12)
};
    let m = {
    let __rc_15 = m;
    let mut __map_ins_14 = Rc::try_unwrap(__rc_15).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_14.insert("string_contains".to_string(), IntrinsicMethod::MethodStringContains);
    Rc::new(__map_ins_14)
};
    let m = {
    let __rc_17 = m;
    let mut __map_ins_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_16.insert("concat".to_string(), IntrinsicMethod::MethodConcat);
    Rc::new(__map_ins_16)
};
    let m = {
    let __rc_19 = m;
    let mut __map_ins_18 = Rc::try_unwrap(__rc_19).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_18.insert("map".to_string(), IntrinsicMethod::MethodMap);
    Rc::new(__map_ins_18)
};
    let m = {
    let __rc_21 = m;
    let mut __map_ins_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_20.insert("filter".to_string(), IntrinsicMethod::MethodFilter);
    Rc::new(__map_ins_20)
};
    let m = {
    let __rc_23 = m;
    let mut __map_ins_22 = Rc::try_unwrap(__rc_23).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_22.insert("any".to_string(), IntrinsicMethod::MethodAny);
    Rc::new(__map_ins_22)
};
    let m = {
    let __rc_25 = m;
    let mut __map_ins_24 = Rc::try_unwrap(__rc_25).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_24.insert("all".to_string(), IntrinsicMethod::MethodAll);
    Rc::new(__map_ins_24)
};
    let m = {
    let __rc_27 = m;
    let mut __map_ins_26 = Rc::try_unwrap(__rc_27).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_26.insert("flat_map".to_string(), IntrinsicMethod::MethodFlatMap);
    Rc::new(__map_ins_26)
};
    let m = {
    let __rc_29 = m;
    let mut __map_ins_28 = Rc::try_unwrap(__rc_29).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_28.insert("skip".to_string(), IntrinsicMethod::MethodSkip);
    Rc::new(__map_ins_28)
};
    let m = {
    let __rc_31 = m;
    let mut __map_ins_30 = Rc::try_unwrap(__rc_31).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_30.insert("take".to_string(), IntrinsicMethod::MethodTake);
    Rc::new(__map_ins_30)
};
    let m = {
    let __rc_33 = m;
    let mut __map_ins_32 = Rc::try_unwrap(__rc_33).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_32.insert("fold".to_string(), IntrinsicMethod::MethodFold);
    Rc::new(__map_ins_32)
};
    let m = {
    let __rc_35 = m;
    let mut __map_ins_34 = Rc::try_unwrap(__rc_35).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_34.insert("sort_by".to_string(), IntrinsicMethod::MethodSortBy);
    Rc::new(__map_ins_34)
};
    let m = {
    let __rc_37 = m;
    let mut __map_ins_36 = Rc::try_unwrap(__rc_37).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_36.insert("append".to_string(), IntrinsicMethod::MethodAppend);
    Rc::new(__map_ins_36)
};
    m.clone()
}

