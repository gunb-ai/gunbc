use crate::v2_core::*;
use crate::infer_types::*;
use crate::infer_env::*;
use crate::infer_sigs::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum ItemKind {
    #[default]
    FnItem,
    FuncItem,
    TypeItem,
    DataItem,
    ServiceItem,
    OtherItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemInfo {
    pub name: String,
    pub kind: ItemKind,
    pub service_names: Rc<Vec<String>>,
    pub resource_names: Rc<Vec<String>>,
    pub params: Rc<Vec<Rc<Param>>>,
    pub is_self_recursive: bool,
    pub has_non_tail_self_call: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedModule {
    pub module: Rc<Module>,
    pub items: Rc<Vec<Rc<Node>>>,
    pub type_env: Rc<TypeEnv>,
    pub func_env: Rc<ResolvedFuncEnv>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypedGraph {
    pub modules: Rc<Vec<Rc<TypedModule>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedGraph {
    pub modules: Rc<Vec<Rc<TypedModule>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn inferred_to_outputs(inferred: Option<Rc<InferredNode>>, span: SourceSpan) -> Rc<Vec<Rc<Field>>> {
    if inferred.clone().is_none() {
    Rc::new(Vec::new())
} else {
    match inferred.clone().unwrap().as_ref() {
    InferredNode::CompilerError { message: _, span: _, .. } => {
        Rc::new(Vec::new())
    }
    InferredNode::Resolved { node: rt, .. } => {
        if rt.connective != Connective::NoConnective {
    if rt.connective == Connective::Conj {
    if rt.name.clone() == "" {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in rt.children.iter().cloned() {
        __mapped_0.push({
    let child_type = if __elem_1.inferred.clone().is_none() {
    leaf_node(&__elem_1.name)
} else {
    rt_type(__elem_1.clone())
};
    Rc::new(Field { name: __elem_1.name.clone(), type_expr: child_type.clone(), cardinality: Cardinality::Required, default_value: None, from_key: None, span: span.clone() })
});
    }
    Rc::new(__mapped_0)
}
} else {
    Rc::new(vec!(Rc::new(Field { name: "value".to_string(), type_expr: rt.clone(), cardinality: Cardinality::Required, default_value: None, from_key: None, span: span.clone() })))
}
} else {
    Rc::new(vec!(Rc::new(Field { name: "value".to_string(), type_expr: rt.clone(), cardinality: Cardinality::Required, default_value: None, from_key: None, span: span.clone() })))
}
} else {
    if (rt.name.clone() == "Unit") && (({
    let __len_2 = rt.children.clone().len();
    __len_2 as i64
}) == 0_i64) {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(Rc::new(Field { name: "value".to_string(), type_expr: rt.clone(), cardinality: Cardinality::Required, default_value: None, from_key: None, span: span.clone() })))
}
}
    }
}
}
}

pub fn item_kind(item: Rc<Node>) -> ItemKind {
    let kind = if (item.connective != Connective::NoConnective) && (item.transport.clone().is_none()) {
    ItemKind::TypeItem
} else {
    if item.transport.clone().is_some() {
    ItemKind::ServiceItem
} else {
    if (item.body.clone().is_some()) && (({
    let __len_1 = item.uses.clone().len();
    __len_1 as i64
}) > 0_i64) {
    ItemKind::FuncItem
} else {
    if (item.body.clone().is_some()) && (({
    let __len_0 = item.params.clone().len();
    __len_0 as i64
}) > 0_i64) {
    ItemKind::FnItem
} else {
    if (item.body.clone().is_some()) && (item.type_annotation.clone().is_some()) {
    ItemKind::DataItem
} else {
    if item.body.clone().is_some() {
    ItemKind::FnItem
} else {
    ItemKind::OtherItem
}
}
}
}
}
};
    kind.clone()
}

pub fn variant_locals_from_items(items: Rc<Vec<Rc<Node>>>, init: Rc<HashMap<String, Rc<TypeBinding>>>) -> Rc<HashMap<String, Rc<TypeBinding>>> {
    {
    let mut __acc_0 = init.clone();
    for __elem_1 in items.iter().cloned() {
        __acc_0 = if __elem_1.connective == Connective::Disj {
    {
    let mut __acc_2 = __acc_0.clone();
    for __elem_3 in __elem_1.children.iter().cloned() {
        __acc_2 = {
    let __rc_5 = __acc_2;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert(__elem_3.name.clone(), Rc::new(TypeBinding { name: __elem_3.name.clone(), resolved: __elem_1.clone() }));
    Rc::new(__map_ins_4)
};
    }
    __acc_2
}
} else {
    __acc_0.clone()
};
    }
    __acc_0
}
}

