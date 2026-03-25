use crate::v2_core::*;
use crate::infer_types::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessCheckResultNode {
    pub resolved_type: Rc<Node>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn access_error(message: &str, span: SourceSpan, module_name: &str) -> Rc<Diagnostic> {
    Rc::new(Diagnostic { severity: Severity::Error, message: message.to_string(), span: Some(span), module_name: Some(module_name.to_string()), category: None })
}

pub fn check_index_access_node(base_type: Rc<Node>, index_type: Rc<Node>, span: SourceSpan, module_name: &str) -> Rc<AccessCheckResultNode> {
    let normed = normalize_access_type_node(base_type.clone());
    if ((node_has_structure(normed.clone()) == false) && (({
    let __len_1 = normed.children.clone().len();
    __len_1 as i64
}) == 0_i64)) && (normed.name.clone() == "String") {
    let diags = if is_int_type_node(index_type.clone()) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(access_error("string index requires an Int index", span, &module_name)))
};
    Rc::new(AccessCheckResultNode { resolved_type: leaf_node("String"), diagnostics: diags.clone() })
} else {
    if node_is_map(normed.clone()) && (({
    let __len_0 = normed.children.clone().len();
    __len_0 as i64
}) >= 2_i64) {
    let key_node = match normed.children.clone().first().cloned() {
    Some(k) => {
        k.clone()
    }
    None => {
        leaf_node("String")
    }
};
    let val_node = match normed.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        leaf_node("Unit")
    }
};
    let key_diags = if node_type_equals(key_node.clone(), index_type.clone()) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(access_error("map index key type does not match the map key type", span, &module_name)))
};
    Rc::new(AccessCheckResultNode { resolved_type: with_optional_cardinality(val_node.clone()), diagnostics: key_diags.clone() })
} else {
    Rc::new(AccessCheckResultNode { resolved_type: leaf_node("Unit"), diagnostics: Rc::new(vec!(access_error("indexing is only supported for String and Map values", span, &module_name))) })
}
}
}

pub fn check_slice_access_node(base_type: Rc<Node>, start_type: Rc<Node>, end_type: Rc<Node>, span: SourceSpan, module_name: &str) -> Rc<AccessCheckResultNode> {
    let base_is_string = is_string_type_node(base_type.clone());
    let base_diags = if base_is_string.clone() {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(access_error("slice is only supported for String values", span.clone(), &module_name)))
};
    let start_diags = if is_int_type_node(start_type.clone()) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(access_error("slice start requires an Int index", span.clone(), &module_name)))
};
    let end_diags = if is_int_type_node(end_type.clone()) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(access_error("slice end requires an Int index", span.clone(), &module_name)))
};
    Rc::new(AccessCheckResultNode { resolved_type: if base_is_string.clone() {
    leaf_node("String")
} else {
    leaf_node("Unit")
}, diagnostics: v2_rt::concat(v2_rt::concat(base_diags.clone(), start_diags.clone()), end_diags.clone()) })
}

