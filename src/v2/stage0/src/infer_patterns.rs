use crate::v2_core::*;
use crate::infer_types::*;
use crate::infer_env::*;
use crate::infer_emit_info::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeLookupResult {
    pub resolved: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn synthesize_optional_some_variant(scrut: Rc<Node>) -> Rc<NodeLookupResult> {
    let inner = extract_optional_inner_node(scrut.clone());
    let value_field = Rc::new(Node { name: "value".to_string(), span: scrut.span.clone(), children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: inner.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    let some_node = Rc::new(Node { name: "Some".to_string(), span: scrut.span.clone(), children: Rc::new(vec!(value_field.clone())), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) });
    Rc::new(NodeLookupResult { resolved: some_node.clone(), diagnostics: Rc::new(Vec::new()) })
}

pub fn variant_not_found_result(scrut: Rc<Node>, variant_name: &str, module_name: &str) -> Rc<NodeLookupResult> {
    Rc::new(NodeLookupResult { resolved: scrut.clone(), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat("variant '".to_string(), v2_rt::concat(variant_name.to_string(), v2_rt::concat("' not found in type '".to_string(), v2_rt::concat(scrut.name.clone(), "'".to_string())))), span: Some(scrut.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::VariantNotFound) }))) })
}

pub fn lookup_variant_in_type(scrut: Rc<Node>, variant_name: &str, module_name: &str) -> Rc<NodeLookupResult> {
    let scrut_opt = node_is_optional(scrut.clone());
    if scrut.name.clone() == "Error" {
    Rc::new(NodeLookupResult { resolved: error_type_node(), diagnostics: Rc::new(Vec::new()) })
} else {
    if scrut.name.clone() == "Dynamic" {
    Rc::new(NodeLookupResult { resolved: error_type_node(), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat("cannot resolve variant '".to_string(), v2_rt::concat(variant_name.to_string(), "' on Dynamic scrutinee".to_string())), span: Some(scrut.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::VariantNotFound) }))) })
} else {
    if ((node_has_structure(scrut.clone()) == false) && (({
    let __len_4 = scrut.children.clone().len();
    __len_4 as i64
}) == 0_i64)) && (scrut_opt.clone() == false) {
    Rc::new(NodeLookupResult { resolved: error_type_node(), diagnostics: Rc::new(Vec::new()) })
} else {
    let direct_match = {
    let mut __found_2 = None;
    for __elem_3 in scrut.children.iter().cloned() {
        if __elem_3.name.clone() == variant_name {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
};
    let fallback = if scrut_opt.clone() && (variant_name == "Some") {
    synthesize_optional_some_variant(scrut.clone())
} else {
    if scrut_opt.clone() && (variant_name == "None") {
    Rc::new(NodeLookupResult { resolved: leaf_node("None"), diagnostics: Rc::new(Vec::new()) })
} else {
    variant_not_found_result(scrut.clone(), &variant_name, &module_name)
}
};
    match direct_match.clone() {
    Some(v) => {
        Rc::new(NodeLookupResult { resolved: v.clone(), diagnostics: Rc::new(Vec::new()) })
    }
    None => {
        fallback.clone()
    }
}
}
}
}
}

pub fn lookup_field_in_variant(variant: Rc<Node>, field_name: &str, module_name: &str) -> Rc<NodeLookupResult> {
    if variant.name.clone() == "Error" {
    Rc::new(NodeLookupResult { resolved: error_type_node(), diagnostics: Rc::new(Vec::new()) })
} else {
    if variant.name.clone() == "Dynamic" {
    Rc::new(NodeLookupResult { resolved: error_type_node(), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat("cannot resolve field '".to_string(), v2_rt::concat(field_name.to_string(), "' on Dynamic variant".to_string())), span: Some(variant.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::FieldNotFound) }))) })
} else {
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
        {
    let resolved = child_return_type_or_name(field_child.clone());
    Rc::new(NodeLookupResult { resolved: resolved.clone(), diagnostics: Rc::new(Vec::new()) })
}
    }
    None => {
        Rc::new(NodeLookupResult { resolved: error_type_node(), diagnostics: Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat("field '".to_string(), v2_rt::concat(field_name.to_string(), v2_rt::concat("' not found in variant '".to_string(), v2_rt::concat(variant.name.clone(), "'".to_string())))), span: Some(variant.span.clone()), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::FieldNotFound) }))) })
    }
}
}
}
}

pub fn check_match_exhaustiveness(scrutinee_type: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>>, env: Rc<TypeEnv>, span: SourceSpan, module_name: &str) -> Rc<Vec<Rc<Diagnostic>>> {
    let resolved = if node_has_structure(scrutinee_type.clone()) {
    scrutinee_type.clone()
} else {
    match lookup_type(env.clone(), &scrutinee_type.name) {
    Some(def) => {
        def.clone()
    }
    None => {
        scrutinee_type.clone()
    }
}
};
    if node_is_coproduct(resolved.clone()) {
    let variant_names = if node_is_optional(resolved.clone()) {
    Rc::new(vec!("Some".to_string(), "None".to_string()))
} else {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in resolved.children.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
}
};
    let has_catch_all = {
    let mut __any_2 = false;
    for __elem_3 in arms.iter().cloned() {
        if match __elem_3.pattern.as_ref() {
    MatchPattern::Wildcard => {
        true
    }
    MatchPattern::Bind { name: _, .. } => {
        true
    }
    _ => {
        false
    }
} {
    __any_2 = true;
    break;
};
    }
    __any_2
};
    if has_catch_all.clone() {
    Rc::new(Vec::new())
} else {
    let covered_set = {
    let mut __acc_4 = Rc::new(std::collections::HashMap::new());
    for __elem_5 in arms.iter().cloned() {
        __acc_4 = match __elem_5.pattern.as_ref() {
    MatchPattern::VariantPattern { name: n, parent_enum: _, field_bindings: _, .. } => {
        {
    let __rc_7 = __acc_4;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(n.clone(), true);
    Rc::new(__map_ins_6)
}
    }
    _ => {
        __acc_4.clone()
    }
};
    }
    __acc_4
};
    let uncovered = {
    let mut __filtered_8 = Vec::new();
    for __elem_9 in variant_names.iter().cloned() {
        if emit_map_has(covered_set.clone(), &__elem_9) == false {
    __filtered_8.push(__elem_9);
};
    }
    Rc::new(__filtered_8)
};
    if ({
    let __len_13 = uncovered.clone().len();
    __len_13 as i64
}) > 0_i64 {
    Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat("non-exhaustive match: missing variant(s) ".to_string(), {
    let mut __joined_10 = String::new();
    let mut __first_12 = true;
    for __elem_11 in uncovered.iter().cloned() {
        if !__first_12 {
    __joined_10.push_str(&", ".to_string());
};
        __first_12 = false;
        __joined_10.push_str(&__elem_11);
    }
    __joined_10
}), span: Some(span), module_name: Some(module_name.to_string()), category: Some(ErrorCategory::InvalidOperation) })))
} else {
    Rc::new(Vec::new())
}
}
} else {
    Rc::new(Vec::new())
}
}

