use crate::v2_core::*;
use crate::infer_types::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessCheckResultNode {
    pub inferred: Option<Rc<InferredNode>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn access_error(message: &str, span: SourceSpan, module_name: &str) -> Rc<Node> {
    diagnostic_node("error", &message, span, Some(module_name.to_string()), None)
}

pub fn access_result(inferred: Rc<Node>, diagnostics: Rc<Vec<Rc<Node>>>, span: SourceSpan, fallback_message: &str) -> Rc<AccessCheckResultNode> {
    if ({
    let __len_0 = diagnostics.clone().len();
    __len_0 as i64
}) == 0_i64 {
    Rc::new(AccessCheckResultNode { inferred: Some(Rc::new(InferredNode::Resolved { node: inferred.clone() })), diagnostics: diagnostics.clone() })
} else {
    let message = match diagnostics.clone().first().cloned() {
    Some(diag) => {
        diagnostic_message(diag.clone())
    }
    None => {
        fallback_message.to_string()
    }
};
    Rc::new(AccessCheckResultNode { inferred: Some(Rc::new(InferredNode::CompilerError { message: message.clone(), span })), diagnostics: diagnostics.clone() })
}
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
    Rc::new(vec!(access_error("string index requires an Int index", span.clone(), &module_name)))
};
    access_result(leaf_node("String"), diags.clone(), span.clone(), "invalid string index access")
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
        leaf_node("")
    }
};
    let val_node = match normed.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        leaf_node("")
    }
};
    let key_diags = if node_type_equals(key_node.clone(), index_type.clone()) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(access_error("map index key type does not match the map key type", span.clone(), &module_name)))
};
    access_result(with_optional_cardinality(val_node.clone()), key_diags.clone(), span.clone(), "invalid map index access")
} else {
    if node_is_map(normed.clone()) {
    let malformed_diags = Rc::new(vec!(access_error("malformed Map type in index access", span.clone(), &module_name)));
    access_result(leaf_node("Unit"), malformed_diags.clone(), span.clone(), "malformed Map type in index access")
} else {
    let diags = Rc::new(vec!(access_error("indexing is only supported for String and Map values", span.clone(), &module_name)));
    access_result(leaf_node("Unit"), diags.clone(), span.clone(), "invalid index access")
}
}
}
}

pub fn check_slice_access_node(base_type: Rc<Node>, start_type: Rc<Node>, end_type: Rc<Node>, span: SourceSpan, module_name: &str) -> Rc<AccessCheckResultNode> {
    let base_is_string = is_string_type_node(base_type.clone());
    let base_diags = if base_is_string {
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
    let all_diags = v2_rt::concat(v2_rt::concat(base_diags.clone(), start_diags.clone()), end_diags.clone());
    access_result(leaf_node("String"), all_diags.clone(), span.clone(), "invalid slice access")
}

