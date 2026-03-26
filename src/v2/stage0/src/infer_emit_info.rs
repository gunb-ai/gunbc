use crate::v2_core::*;
use crate::infer_types::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TypeRepr {
    #[default]
    StructRepr,
    EnumRepr { unit_only: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeSummary {
    pub name: String,
    pub repr: Rc<TypeRepr>,
    pub field_summaries: Rc<HashMap<String, Rc<FieldSummary>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmitGraphInfo {
    pub type_summaries: Rc<HashMap<String, Rc<TypeSummary>>>,
    pub variant_to_enum: Rc<HashMap<String, String>>,
    pub enum_variant_membership: Rc<HashMap<String, bool>>,
    pub field_type_names: Rc<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmitInfoBuildState {
    pub type_summaries: Rc<HashMap<String, Rc<TypeSummary>>>,
    pub variant_to_enum: Rc<HashMap<String, String>>,
    pub enum_variant_membership: Rc<HashMap<String, bool>>,
    pub field_type_names: Rc<HashMap<String, String>>,
}

pub fn empty_emit_graph_info() -> Rc<EmitGraphInfo> {
    Rc::new(EmitGraphInfo { type_summaries: Rc::new(std::collections::HashMap::new()), variant_to_enum: Rc::new(std::collections::HashMap::new()), enum_variant_membership: Rc::new(std::collections::HashMap::new()), field_type_names: Rc::new(std::collections::HashMap::new()) })
}

pub fn lookup_emit_type_summary(emit_info: Rc<EmitGraphInfo>, type_name: &str) -> Option<Rc<TypeSummary>> {
    emit_info.type_summaries.clone().get(&type_name.to_string()).cloned()
}

pub fn field_value_shape_from_type_node(type_node: Rc<Node>) -> FieldValueShape {
    let normed = normalize_access_type_node(type_node.clone());
    if node_is_optional(normed.clone()) {
    FieldValueShape::OptionalValue
} else {
    FieldValueShape::PlainValue
}
}

pub fn is_pair_children(children: Rc<Vec<Rc<Node>>>) -> bool {
    if ({
    let __len_0 = children.clone().len();
    __len_0 as i64
}) != 2_i64 {
    false
} else {
    match children.clone().first().cloned() {
    Some(c0) => {
        match children.clone().get((1_i64) as usize).cloned() {
    Some(c1) => {
        (c0.name.clone() == "first") && (c1.name.clone() == "second")
    }
    None => {
        false
    }
}
    }
    None => {
        false
    }
}
}
}

pub fn pair_access_style(field_name: &str) -> FieldAccessStyle {
    if field_name == "first" {
    FieldAccessStyle::TupleFirst
} else {
    FieldAccessStyle::TupleSecond
}
}

pub fn build_struct_field_summaries(children: Rc<Vec<Rc<Node>>>) -> Rc<HashMap<String, Rc<FieldSummary>>> {
    let is_pair = is_pair_children(children.clone());
    {
    let mut __acc_0: Rc<std::collections::HashMap<String, Rc<FieldSummary>>> = Rc::new(std::collections::HashMap::new());
    for __elem_1 in children.iter().cloned() {
        __acc_0 = if __elem_1.return_type.clone().is_none() {
    __acc_0.clone()
} else {
    let style = if is_pair.clone() {
    pair_access_style(&__elem_1.name)
} else {
    FieldAccessStyle::StoredField
};
    {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.name.clone(), Rc::new(FieldSummary { access_style: style.clone(), value_shape: field_value_shape_from_type_node(rt_type(__elem_1.clone())) }));
    Rc::new(__map_ins_2)
}
};
    }
    __acc_0
}
}

pub fn find_first_enum_field_node(variants: Rc<Vec<Rc<Node>>>, field_name: &str) -> Option<Rc<Node>> {
    match variants.clone().first().cloned() {
    Some(variant) => {
        match {
    let mut __found_2 = None;
    for __elem_3 in variant.children.iter().cloned() {
        if __elem_3.name.clone() == field_name {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(field_child) => {
        Some(field_child.clone())
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

pub fn enum_field_present_in_all_variants(variants: Rc<Vec<Rc<Node>>>, field_name: &str) -> bool {
    {
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
}
}

pub fn enum_field_type_consistent(variants: Rc<Vec<Rc<Node>>>, field_name: &str, expected: Rc<Node>) -> bool {
    {
    let mut __all_0 = true;
    for __elem_1 in variants.iter().cloned() {
        if !(match {
    let mut __found_4 = None;
    for __elem_5 in __elem_1.children.iter().cloned() {
        if __elem_5.name.clone() == field_name {
    __found_4 = Some(__elem_5);
    break;
};
    }
    __found_4
} {
    Some(field_child) => {
        node_type_equals(child_return_type_or_name(field_child.clone()), expected.clone())
    }
    None => {
        false
    }
}) {
    __all_0 = false;
    break;
};
    }
    __all_0
}
}

pub fn build_enum_field_summaries(variants: Rc<Vec<Rc<Node>>>) -> Rc<HashMap<String, Rc<FieldSummary>>> {
    let first_field_names = match variants.clone().first().cloned() {
    Some(first_variant) => {
        {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in first_variant.children.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
}
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let shared = {
    let mut __filtered_2 = Vec::new();
    for __elem_3 in first_field_names.iter().cloned() {
        if enum_field_present_in_all_variants(variants.clone(), &__elem_3) {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
};
    let consistent = {
    let mut __filtered_4 = Vec::new();
    for __elem_5 in shared.iter().cloned() {
        if match find_first_enum_field_node(variants.clone(), &__elem_5) {
    Some(first_field) => {
        enum_field_type_consistent(variants.clone(), &__elem_5, child_return_type_or_name(first_field.clone()))
    }
    None => {
        false
    }
} {
    __filtered_4.push(__elem_5);
};
    }
    Rc::new(__filtered_4)
};
    {
    let mut __acc_6: Rc<std::collections::HashMap<String, Rc<FieldSummary>>> = Rc::new(std::collections::HashMap::new());
    for __elem_7 in consistent.iter().cloned() {
        __acc_6 = match find_first_enum_field_node(variants.clone(), &__elem_7) {
    Some(first_field) => {
        {
    let __rc_9 = __acc_6;
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert(__elem_7.clone(), Rc::new(FieldSummary { access_style: FieldAccessStyle::EnumAccessor, value_shape: field_value_shape_from_type_node(child_return_type_or_name(first_field.clone())) }));
    Rc::new(__map_ins_8)
}
    }
    None => {
        __acc_6.clone()
    }
};
    }
    __acc_6
}
}

pub fn build_type_summary(item: Rc<Node>) -> Option<Rc<TypeSummary>> {
    if (node_has_structure(item.clone()) == false) || (item.transport.clone().is_some()) {
    return None;
};
    if node_is_product(item.clone()) {
    Some(Rc::new(TypeSummary { name: item.name.clone(), repr: Rc::new(TypeRepr::StructRepr), field_summaries: build_struct_field_summaries(item.children.clone()) }))
} else {
    let unit_only = {
    let mut __all_0 = true;
    for __elem_1 in item.children.iter().cloned() {
        if !(({
    let __len_2 = __elem_1.children.clone().len();
    __len_2 as i64
}) == 0_i64) {
    __all_0 = false;
    break;
};
    }
    __all_0
};
    Some(Rc::new(TypeSummary { name: item.name.clone(), repr: Rc::new(TypeRepr::EnumRepr { unit_only: unit_only.clone() }), field_summaries: build_enum_field_summaries(item.children.clone()) }))
}
}

pub fn add_emit_item_summary(state: Rc<EmitInfoBuildState>, item: Rc<Node>) -> Rc<EmitInfoBuildState> {
    match build_type_summary(item.clone()) {
    Some(summary) => {
        {
    let with_variants = match summary.repr.as_ref() {
    TypeRepr::EnumRepr { unit_only: _, .. } => {
        {
    let mut __acc_0 = state.type_summaries.clone();
    for __elem_1 in item.children.iter().cloned() {
        __acc_0 = if ({
    let __len_4 = __elem_1.children.clone().len();
    __len_4 as i64
}) > 0_i64 {
    {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.name.clone(), Rc::new(TypeSummary { name: __elem_1.name.clone(), repr: Rc::new(TypeRepr::StructRepr), field_summaries: build_struct_field_summaries(__elem_1.children.clone()) }));
    Rc::new(__map_ins_2)
}
} else {
    __acc_0.clone()
};
    }
    __acc_0
}
    }
    _ => {
        state.type_summaries.clone()
    }
};
    let next_summaries = {
    let __rc_6 = with_variants;
    let mut __map_ins_5 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_5.insert(summary.name.clone(), summary.clone());
    Rc::new(__map_ins_5)
};
    let next_variants = match summary.repr.as_ref() {
    TypeRepr::EnumRepr { unit_only: _, .. } => {
        {
    let mut __acc_7 = state.variant_to_enum.clone();
    for __elem_8 in item.children.iter().cloned() {
        __acc_7 = match __acc_7.clone().get(&__elem_8.name.clone()).cloned() {
    Some(existing) => {
        if existing.clone() == summary.name.clone() {
    __acc_7.clone()
} else {
    {
    let __rc_10 = __acc_7;
    let mut __map_ins_9 = Rc::try_unwrap(__rc_10).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_9.insert(__elem_8.name.clone(), "__ambiguous__".to_string());
    Rc::new(__map_ins_9)
}
}
    }
    None => {
        {
    let __rc_12 = __acc_7;
    let mut __map_ins_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_11.insert(__elem_8.name.clone(), summary.name.clone());
    Rc::new(__map_ins_11)
}
    }
};
    }
    __acc_7
}
    }
    _ => {
        state.variant_to_enum.clone()
    }
};
    let next_evm = match summary.repr.as_ref() {
    TypeRepr::EnumRepr { unit_only: _, .. } => {
        {
    let mut __acc_13 = state.enum_variant_membership.clone();
    for __elem_14 in item.children.iter().cloned() {
        __acc_13 = {
    let __rc_16 = __acc_13;
    let mut __map_ins_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_15.insert(v2_rt::concat(v2_rt::concat(summary.name.clone(), "|".to_string()), __elem_14.name.clone()), true);
    Rc::new(__map_ins_15)
};
    }
    __acc_13
}
    }
    _ => {
        state.enum_variant_membership.clone()
    }
};
    let next_ftn = match summary.repr.as_ref() {
    TypeRepr::StructRepr => {
        {
    let mut __acc_17 = state.field_type_names.clone();
    for __elem_18 in item.children.iter().cloned() {
        __acc_17 = match __elem_18.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: ft, .. }) => {
        {
    let resolved_name = normalize_access_type_node(ft.clone()).name.clone();
    if (resolved_name.clone() != "") && (resolved_name.clone() != "Dynamic") {
    {
    let __rc_20 = __acc_17;
    let mut __map_ins_19 = Rc::try_unwrap(__rc_20).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_19.insert(v2_rt::concat(v2_rt::concat(summary.name.clone(), "|".to_string()), __elem_18.name.clone()), resolved_name.clone());
    Rc::new(__map_ins_19)
}
} else {
    __acc_17.clone()
}
}
    }
    _ => {
        __acc_17.clone()
    }
};
    }
    __acc_17
}
    }
    TypeRepr::EnumRepr { unit_only: _, .. } => {
        {
    let mut __acc_21 = state.field_type_names.clone();
    for __elem_22 in item.children.iter().cloned() {
        __acc_21 = {
    let mut __acc_23 = __acc_21.clone();
    for __elem_24 in __elem_22.children.iter().cloned() {
        __acc_23 = match __elem_24.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: ft, .. }) => {
        {
    let resolved_name = normalize_access_type_node(ft.clone()).name.clone();
    if (resolved_name.clone() != "") && (resolved_name.clone() != "Dynamic") {
    {
    let __rc_26 = __acc_23;
    let mut __map_ins_25 = Rc::try_unwrap(__rc_26).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_25.insert(v2_rt::concat(v2_rt::concat(__elem_22.name.clone(), "|".to_string()), __elem_24.name.clone()), resolved_name.clone());
    Rc::new(__map_ins_25)
}
} else {
    __acc_23.clone()
}
}
    }
    _ => {
        __acc_23.clone()
    }
};
    }
    __acc_23
};
    }
    __acc_21
}
    }
};
    Rc::new(EmitInfoBuildState { type_summaries: next_summaries.clone(), variant_to_enum: next_variants.clone(), enum_variant_membership: next_evm.clone(), field_type_names: next_ftn.clone() })
}
    }
    None => {
        state.clone()
    }
}
}

