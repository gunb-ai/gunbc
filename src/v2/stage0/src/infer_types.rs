use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn child_return_type_or_name(ch: Rc<Node>) -> Rc<Node> {
    if ch.return_type.clone().is_none() {
    leaf_node(&ch.name)
} else {
    rt_type(ch.clone())
}
}

pub fn leaf_node(name: &str) -> Rc<Node> {
    Rc::new(Node { name: name.to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn rt_type(n: Rc<Node>) -> Rc<Node> {
    match rt_node(n.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        rt.clone()
    }
    NodeType::InferError { message: _, span: _, .. } => {
        leaf_node("Unit")
    }
    NodeType::Untyped => {
        leaf_node("Unit")
    }
}
}

pub fn product_property() -> Rc<FieldInit> {
    Rc::new(FieldInit { name: "is_product".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) })
}

pub fn coproduct_property() -> Rc<FieldInit> {
    Rc::new(FieldInit { name: "is_coproduct".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) })
}

pub fn with_optional_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: Cardinality::CardOptional, uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), expr_data: n.expr_data.clone() })
}

pub fn with_required_cardinality(n: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: n.name.clone(), span: n.span.clone(), children: n.children.clone(), connective: n.connective.clone(), params: n.params.clone(), return_type: n.return_type.clone(), return_cardinality: Cardinality::Required, uses: n.uses.clone(), body: n.body.clone(), transport: n.transport.clone(), properties: n.properties.clone(), type_annotation: n.type_annotation.clone(), config: n.config.clone(), is_self_recursive: n.is_self_recursive.clone(), has_non_tail_self_call: n.has_non_tail_self_call.clone(), expr_data: n.expr_data.clone() })
}

pub fn container_property(kind_name: &str) -> Rc<FieldInit> {
    Rc::new(FieldInit { name: "container_kind".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitStr { value: kind_name.to_string() }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) })
}

pub fn container_node(kind_name: &str, element: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: kind_name.to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(vec!(element.clone())), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(vec!(container_property(&kind_name))), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn tuple_node(first: Rc<Node>, second: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: "Tuple".to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(vec!(Rc::new(Node { name: "first".to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: first.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }), Rc::new(Node { name: "second".to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: Some(Rc::new(InferredNode::Resolved { node: second.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) }))), connective: Some(Connective::Conj), params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(vec!(product_property())), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn map_node(key: Rc<Node>, value: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: "Map".to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(vec!(key.clone(), value.clone())), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(vec!(Rc::new(FieldInit { name: "map_type".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) }))), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn bare_map_node() -> Rc<Node> {
    Rc::new(Node { name: "Map".to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, params: Rc::new(Vec::new()), return_type: None, return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(vec!(Rc::new(FieldInit { name: "map_type".to_string(), value: make_expr_node(Rc::new(ExprData::ExprLiteral { value: Rc::new(LiteralValue::LitBool { value: true }) }), None, SourceSpan { start: 0_i64, end: 0_i64 }) }))), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn callable_node(func_params: Rc<Vec<Rc<Param>>>, ret: Rc<Node>) -> Rc<Node> {
    Rc::new(Node { name: "Callable".to_string(), span: SourceSpan { start: 0_i64, end: 0_i64 }, children: Rc::new(Vec::new()), connective: None, params: func_params.clone(), return_type: Some(Rc::new(InferredNode::Resolved { node: ret.clone() })), return_cardinality: Cardinality::Required, uses: Rc::new(Vec::new()), body: None, transport: None, properties: Rc::new(Vec::new()), type_annotation: None, config: None, is_self_recursive: false, has_non_tail_self_call: false, expr_data: Rc::new(ExprData::NoExprData) })
}

pub fn callable_return_type(n: Rc<Node>) -> Rc<Node> {
    if n.name.clone() == "Callable" {
    match n.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: ret, .. }) => {
        ret.clone()
    }
    None => {
        leaf_node("Dynamic")
    }
    _ => {
        leaf_node("Dynamic")
    }
}
} else {
    n.clone()
}
}

pub fn error_type_node() -> Rc<Node> {
    leaf_node("Error")
}

pub fn node_is_product(n: Rc<Node>) -> bool {
    ({
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "is_product" {
    __any_0 = true;
    break;
};
    }
    __any_0
}) || ((n.connective.clone().is_some()) && (n.connective.clone() == Some(Connective::Conj)))
}

pub fn node_is_coproduct(n: Rc<Node>) -> bool {
    ({
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "is_coproduct" {
    __any_0 = true;
    break;
};
    }
    __any_0
}) || ((n.connective.clone().is_some()) && (n.connective.clone() == Some(Connective::Disj)))
}

pub fn node_has_structure(n: Rc<Node>) -> bool {
    node_is_product(n.clone()) || node_is_coproduct(n.clone())
}

pub fn node_is_container(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "container_kind" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn node_is_optional(n: Rc<Node>) -> bool {
    match n.return_cardinality.clone() {
    Cardinality::CardOptional => {
        true
    }
    Cardinality::Required => {
        false
    }
}
}

pub fn node_is_map(n: Rc<Node>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in n.properties.iter().cloned() {
        if __elem_1.name.clone() == "map_type" {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn node_is_leaf(n: Rc<Node>) -> bool {
    (((n.connective.clone().is_none()) && (node_has_structure(n.clone()) == false)) && (({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64)) && (({
    let __len_1 = n.properties.clone().len();
    __len_1 as i64
}) == 0_i64)
}

pub fn node_is_named_ref(n: Rc<Node>) -> bool {
    if n.return_type.clone().is_none() {
    false
} else {
    match rt_node(n.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        ((((node_has_structure(rt.clone()) == false) && (({
    let __len_0 = rt.children.clone().len();
    __len_0 as i64
}) == 0_i64)) && (rt.name.clone() != "")) && (rt.name.clone() != "None")) && (is_kernel_type(&rt.name) == false)
    }
    NodeType::InferError { message: _, span: _, .. } => {
        false
    }
    NodeType::Untyped => {
        false
    }
}
}
}

pub fn normalize_access_type_node(n: Rc<Node>) -> Rc<Node> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_n = n;
        loop {
            let n = __tco_p_n;
            let unwrapped = if (n.name.clone() == "Refined") && node_has_structure(n.clone()) {
    n.children.clone().first().cloned()
} else {
    None
};
            match unwrapped.clone() {
    Some(base) => {
         {
            let __tco_0 = base.clone();
            __tco_p_n = __tco_0;
            continue;
        }

    }
    None => {
        break n.clone();
    }
};
        }
    })
}

pub fn normalize_type_name(name: &str) -> String {
    if (v2_rt::string_contains(&name, &"<") == false) && (v2_rt::string_contains(&name, &".") == false) {
    name.to_string()
} else {
    let base = match ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in name.to_string().split("<".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).first().cloned() {
    Some(part) => {
        part.clone()
    }
    None => {
        name.to_string()
    }
};
    match ({
    let mut __split_parts_2 = Vec::new();
    for __part_3 in base.clone().split(".".to_string().as_str()) {
        __split_parts_2.push(__part_3.to_string());
    }
    __split_parts_2
}).last().cloned() {
    Some(short) => {
        short.clone()
    }
    None => {
        base.clone()
    }
}
}
}

pub fn node_type_shape(n: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if node_is_leaf(n.clone()) {
    if node_is_named_ref(n.clone()) {
    v2_rt::concat(v2_rt::concat("Named(".to_string(), normalize_type_name(&n.name)), ")".to_string())
} else {
    v2_rt::concat(v2_rt::concat("Primitive(".to_string(), n.name.clone()), ")".to_string())
}
} else {
    if node_is_product(n.clone()) {
    if n.name.clone() == "" {
    "Product(<anon>)".to_string()
} else {
    v2_rt::concat(v2_rt::concat("Product(".to_string(), normalize_type_name(&n.name)), ")".to_string())
}
} else {
    if node_is_coproduct(n.clone()) {
    if n.name.clone() == "" {
    "Coproduct(<anon>)".to_string()
} else {
    v2_rt::concat(v2_rt::concat("Coproduct(".to_string(), normalize_type_name(&n.name)), ")".to_string())
}
} else {
    if node_is_container(n.clone()) {
    let elem_shape = match n.children.clone().first().cloned() {
    Some(el) => {
        node_type_shape(el.clone())
    }
    None => {
        "?".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Container(".to_string(), n.name.clone()), ",".to_string()), elem_shape.clone()), ")".to_string())
} else {
    if node_is_optional(n.clone()) {
    let inner_shape = node_type_shape(with_required_cardinality(n.clone()));
    v2_rt::concat(v2_rt::concat("Optional(".to_string(), inner_shape), ")".to_string())
} else {
    if node_is_map(n.clone()) {
    "Map(...)".to_string()
} else {
    v2_rt::concat(v2_rt::concat("Node(".to_string(), n.name.clone()), ")".to_string())
}
}
}
}
}
}
    })
}

pub fn node_type_compatible(left: Rc<Node>, right: Rc<Node>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_left = left;
        let mut __tco_p_right = right;
        loop {
            let left = __tco_p_left;
            let right = __tco_p_right;
            if (left.name.clone() == "Error") || (right.name.clone() == "Error") {
    break true;
} else {
    if (left.name.clone() == "Dynamic") || (right.name.clone() == "Dynamic") {
    break true;
} else {
    if node_is_optional(left.clone()) && (right.name.clone() == "Unit") {
    break true;
} else {
    if (left.name.clone() == "Unit") && node_is_optional(right.clone()) {
    break true;
} else {
    if node_is_container(left.clone()) && node_is_container(right.clone()) {
    if normalize_type_name(&left.name) != normalize_type_name(&right.name) {
    break false;
} else {
    match left.children.clone().first().cloned() {
    Some(left_el) => {
        match right.children.clone().first().cloned() {
    Some(right_el) => {
        if (left_el.name.clone() == "Unit") || (right_el.name.clone() == "Unit") {
    break true;
} else {
     {
        let __tco_0 = left_el.clone();
        let __tco_1 = right_el.clone();
        __tco_p_left = __tco_0;
        __tco_p_right = __tco_1;
        continue;
    }

};
    }
    None => {
        break true;
    }
};
    }
    None => {
        break true;
    }
};
};
} else {
    if node_is_optional(left.clone()) && node_is_optional(right.clone()) {
    let left_inner = with_required_cardinality(left.clone());
    let right_inner = with_required_cardinality(right.clone());
    if (left_inner.name.clone() == "Unit") || (right_inner.name.clone() == "Unit") {
    break true;
} else {
     {
        let __tco_0 = left_inner.clone();
        let __tco_1 = right_inner.clone();
        __tco_p_left = __tco_0;
        __tco_p_right = __tco_1;
        continue;
    }

};
} else {
    if node_is_optional(left.clone()) || node_is_optional(right.clone()) {
    break false;
} else {
    break normalize_type_name(&left.name) == normalize_type_name(&right.name);
};
};
};
};
};
};
};
        }
    })
}

pub fn prefer_specific_type(left: Rc<Node>, right: Rc<Node>) -> Rc<Node> {
    let left_is_container = node_is_container(left.clone());
    let left_is_optional = node_is_optional(left.clone());
    let left_first_child = left.children.clone().first().cloned();
    let left_norm_name = left.name.clone();
    let left_is_unit_inner = if left_is_container.clone() {
    match left_first_child.clone() {
    Some(el) => {
        el.name.clone() == "Unit"
    }
    None => {
        false
    }
}
} else {
    if left_is_optional.clone() {
    left.name.clone() == "Unit"
} else {
    false
}
};
    let same_kind = if left_is_container.clone() && node_is_container(right.clone()) {
    normalize_type_name(&left_norm_name) == normalize_type_name(&right.name)
} else {
    if left_is_optional.clone() && node_is_optional(right.clone()) {
    true
} else {
    false
}
};
    if same_kind.clone() && left_is_unit_inner.clone() {
    right.clone()
} else {
    left.clone()
}
}

pub fn node_type_equals(left: Rc<Node>, right: Rc<Node>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if (left.name.clone() == "Error") || (right.name.clone() == "Error") {
    true
} else {
    if (left.name.clone() == "Dynamic") && (right.name.clone() == "Dynamic") {
    true
} else {
    if (left.name.clone() == "Dynamic") || (right.name.clone() == "Dynamic") {
    false
} else {
    if node_is_optional(left.clone()) && (right.name.clone() == "Unit") {
    true
} else {
    if (left.name.clone() == "Unit") && node_is_optional(right.clone()) {
    true
} else {
    if node_is_leaf(left.clone()) && node_is_leaf(right.clone()) {
    normalize_type_name(&left.name) == normalize_type_name(&right.name)
} else {
    if node_has_structure(left.clone()) && node_has_structure(right.clone()) {
    if normalize_type_name(&left.name) != normalize_type_name(&right.name) {
    false
} else {
    if node_is_product(left.clone()) != node_is_product(right.clone()) {
    false
} else {
    if ({
    let __len_5 = left.children.clone().len();
    __len_5 as i64
}) != ({
    let __len_6 = right.children.clone().len();
    __len_6 as i64
}) {
    false
} else {
    {
    let mut __all_3 = true;
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in left.children.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        if !(match right.children.clone().get((__elem_4.0.clone()) as usize).cloned() {
    Some(right_child) => {
        node_type_equals(__elem_4.1.clone(), right_child.clone())
    }
    None => {
        false
    }
}) {
    __all_3 = false;
    break;
};
    }
    __all_3
}
}
}
}
} else {
    if node_is_leaf(left.clone()) && node_has_structure(right.clone()) {
    normalize_type_name(&left.name) == normalize_type_name(&right.name)
} else {
    if node_has_structure(left.clone()) && node_is_leaf(right.clone()) {
    normalize_type_name(&left.name) == normalize_type_name(&right.name)
} else {
    if node_is_container(left.clone()) && node_is_container(right.clone()) {
    if left.name.clone() != right.name.clone() {
    false
} else {
    match left.children.clone().first().cloned() {
    Some(left_el) => {
        match right.children.clone().first().cloned() {
    Some(right_el) => {
        node_type_equals(left_el.clone(), right_el.clone())
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
} else {
    if node_is_optional(left.clone()) && node_is_optional(right.clone()) {
    node_type_equals(with_required_cardinality(left.clone()), with_required_cardinality(right.clone()))
} else {
    if node_is_map(left.clone()) && node_is_map(right.clone()) {
    if (({
    let __len_7 = left.children.clone().len();
    __len_7 as i64
}) == 2_i64) && (({
    let __len_8 = right.children.clone().len();
    __len_8 as i64
}) == 2_i64) {
    let left_first = match left.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        left.clone()
    }
};
    let right_first = match right.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        right.clone()
    }
};
    let left_second = match left.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        left.clone()
    }
};
    let right_second = match right.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        right.clone()
    }
};
    node_type_equals(left_first.clone(), right_first.clone()) && node_type_equals(left_second.clone(), right_second.clone())
} else {
    false
}
} else {
    if (left.name.clone() == "Callable") && (right.name.clone() == "Callable") {
    if ({
    let __len_14 = left.params.clone().len();
    __len_14 as i64
}) != ({
    let __len_15 = right.params.clone().len();
    __len_15 as i64
}) {
    false
} else {
    let params_eq = {
    let mut __all_12 = true;
    for __elem_13 in ({
    let mut __enumerated_9 = Vec::new();
    for (__idx_10, __elem_11) in left.params.clone().iter().enumerate() {
        __enumerated_9.push((__idx_10 as i64, __elem_11.clone()));
    }
    Rc::new(__enumerated_9)
}).iter().cloned() {
        if !(match right.params.clone().get((__elem_13.0.clone()) as usize).cloned() {
    Some(right_param) => {
        node_type_equals(__elem_13.1.type_expr.clone(), right_param.type_expr.clone())
    }
    None => {
        false
    }
}) {
    __all_12 = false;
    break;
};
    }
    __all_12
};
    if params_eq.clone() == false {
    false
} else {
    match left.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: left_ret, .. }) => {
        match right.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: right_ret, .. }) => {
        node_type_equals(left_ret.clone(), right_ret.clone())
    }
    None => {
        false
    }
    _ => {
        false
    }
}
    }
    None => {
        right.return_type.clone().is_none()
    }
    _ => {
        false
    }
}
}
}
} else {
    false
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
    })
}

pub fn node_type_deps(n: Rc<Node>) -> Rc<Vec<String>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if node_is_named_ref(n.clone()) {
    match rt_node(n.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        Rc::new(vec!(rt.name.clone()))
    }
    NodeType::InferError { message: _, span: _, .. } => {
        Rc::new(Vec::new())
    }
    NodeType::Untyped => {
        Rc::new(Vec::new())
    }
}
} else {
    if node_has_structure(n.clone()) {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in n.children.iter().cloned() {
        __flat_mapped_0.extend((if __elem_1.return_type.clone().is_some() {
    match rt_node(__elem_1.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        node_type_deps(rt.clone())
    }
    NodeType::InferError { message: _, span: _, .. } => {
        Rc::new(Vec::new())
    }
    NodeType::Untyped => {
        Rc::new(Vec::new())
    }
}
} else {
    node_type_deps(__elem_1.clone())
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
} else {
    if n.return_type.clone().is_some() {
    match rt_node(n.clone()).as_ref() {
    NodeType::Typed { node: rt, .. } => {
        node_type_deps(rt.clone())
    }
    NodeType::InferError { message: _, span: _, .. } => {
        Rc::new(Vec::new())
    }
    NodeType::Untyped => {
        Rc::new(Vec::new())
    }
}
} else {
    if ({
    let __len_4 = n.children.clone().len();
    __len_4 as i64
}) > 0_i64 {
    let child_deps = {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in n.children.iter().cloned() {
        __flat_mapped_2.extend(node_type_deps(__elem_3.clone()).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
};
    if (n.name.clone() != "") && (is_kernel_type(&n.name) == false) {
    v2_rt::concat(Rc::new(vec!(n.name.clone())), child_deps.clone())
} else {
    child_deps.clone()
}
} else {
    if (is_kernel_type(&n.name) || (n.name.clone() == "None")) || (n.name.clone() == "") {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(n.name.clone()))
}
}
}
}
}
    })
}

pub fn is_int_type_node(n: Rc<Node>) -> bool {
    let normed = normalize_access_type_node(n.clone());
    (normed.name.clone() == "Int") && node_is_leaf(normed.clone())
}

pub fn is_string_type_node(n: Rc<Node>) -> bool {
    let normed = normalize_access_type_node(n.clone());
    (normed.name.clone() == "String") && node_is_leaf(normed.clone())
}

pub fn is_bool_type_node(n: Rc<Node>) -> bool {
    let normed = normalize_access_type_node(n.clone());
    (normed.name.clone() == "Bool") && node_is_leaf(normed.clone())
}

pub fn is_float_type_node(n: Rc<Node>) -> bool {
    let normed = normalize_access_type_node(n.clone());
    (normed.name.clone() == "Float") && node_is_leaf(normed.clone())
}

pub fn infer_literal_node(lit: Rc<LiteralValue>) -> Rc<Node> {
    match lit.as_ref() {
    LiteralValue::LitStr { value: _, .. } => {
        leaf_node("String")
    }
    LiteralValue::LitInt { value: _, .. } => {
        leaf_node("Int")
    }
    LiteralValue::LitFloat { value: _, .. } => {
        leaf_node("Float")
    }
    LiteralValue::LitBool { value: _, .. } => {
        leaf_node("Bool")
    }
    LiteralValue::LitNull => {
        with_optional_cardinality(leaf_node("Unit"))
    }
}
}

pub fn method_receiver_element_node(receiver_type: Rc<Node>) -> Rc<Node> {
    let normed = normalize_access_type_node(receiver_type.clone());
    if (node_has_structure(normed.clone()) == false) && (({
    let __len_0 = normed.children.clone().len();
    __len_0 as i64
}) == 1_i64) {
    match normed.children.clone().first().cloned() {
    Some(el) => {
        el.clone()
    }
    None => {
        receiver_type.clone()
    }
}
} else {
    if node_is_map(normed.clone()) {
    match normed.children.clone().get((1_i64) as usize).cloned() {
    Some(val_type) => {
        val_type.clone()
    }
    None => {
        receiver_type.clone()
    }
}
} else {
    receiver_type.clone()
}
}
}

pub fn extract_optional_inner_node(n: Rc<Node>) -> Rc<Node> {
    if node_is_optional(n.clone()) {
    with_required_cardinality(n.clone())
} else {
    n.clone()
}
}

pub fn infer_binop_type_node(op: BinOpKind, left_type: Rc<Node>) -> Rc<Node> {
    match op {
    BinOpKind::BinEq => {
        leaf_node("Bool")
    }
    BinOpKind::BinNe => {
        leaf_node("Bool")
    }
    BinOpKind::BinLt => {
        leaf_node("Bool")
    }
    BinOpKind::BinGt => {
        leaf_node("Bool")
    }
    BinOpKind::BinLe => {
        leaf_node("Bool")
    }
    BinOpKind::BinGe => {
        leaf_node("Bool")
    }
    BinOpKind::BinAnd => {
        leaf_node("Bool")
    }
    BinOpKind::BinOr => {
        leaf_node("Bool")
    }
    BinOpKind::NullCoalesce => {
        extract_optional_inner_node(left_type.clone())
    }
    _ => {
        left_type.clone()
    }
}
}

pub fn for_each_element_type_node(n: Rc<Node>) -> Rc<Node> {
    let normed = normalize_access_type_node(n.clone());
    let is_single_child = (node_has_structure(normed.clone()) == false) && (({
    let __len_0 = normed.children.clone().len();
    __len_0 as i64
}) == 1_i64);
    let extracted = if is_single_child.clone() {
    normed.children.clone().first().cloned()
} else {
    None
};
    match extracted.clone() {
    Some(el) => {
        el.clone()
    }
    None => {
        if node_is_leaf(normed.clone()) && (normed.name.clone() == "String") {
    leaf_node("String")
} else {
    normed.clone()
}
    }
}
}

