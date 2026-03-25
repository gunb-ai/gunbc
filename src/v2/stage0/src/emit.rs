use crate::v2_core::*;
use crate::infer_env::*;
use crate::infer_method::*;
use crate::infer_types::*;
use crate::infer_sigs::*;
use crate::infer::*;
use crate::infer_emit_info::*;
use crate::artifact::*;
use crate::rust_emit::*;
use crate::python_emit::*;
use crate::go_emit::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmitResult {
    pub files: Rc<Vec<Rc<TextFile>>>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockEmitState {
    pub text: Rc<Vec<String>>,
    pub scope: Rc<InferScope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum TypedItemKind {
    #[default]
    TypedItemTypeDef,
    TypedItemTypeAlias,
    TypedItemTypeDecl,
    TypedItemFunction,
    TypedItemDataDef,
    TypedItemServiceDef,
    TypedItemResourceDef,
    TypedItemExternFunc,
    TypedItemUnhandled,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum TypeStructureKind {
    #[default]
    TypeLeaf,
    TypeConj,
    TypeDisj,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum BackendCapability {
    #[default]
    CapServiceEmit,
    CapAsyncTransport,
    CapTestGeneration,
    CapDryRunMode,
    CapRcOwnership,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackendInfo {
    pub target_name: String,
    pub capabilities: Rc<Vec<BackendCapability>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestProjection {
    pub service_name: String,
    pub operation_name: String,
    pub mock_field_inits: Rc<Vec<Rc<FieldInit>>>,
}

pub fn has_mock_prefix(name: &str) -> bool {
    if v2_rt::string_length(&name) < 5_i64 {
    false
} else {
    v2_rt::substring(&name, 0_i64, 5_i64) == "mock_"
}
}

pub fn extract_test_projections(typed: Rc<ResolvedGraph>) -> Rc<Vec<Rc<TestProjection>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in __elem_1.items.iter().cloned() {
        if is_service_item(__elem_3.clone()) {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __flat_mapped_4.extend(({
    let mut __mapped_10 = Vec::new();
    for __elem_11 in ({
    let mut __filtered_6 = Vec::new();
    for __elem_7 in __elem_5.children.iter().cloned() {
        {
let __cond = {
    let mut __any_8 = false;
    for __elem_9 in __elem_7.properties.iter().cloned() {
        if has_mock_prefix(&__elem_9.name) {
    __any_8 = true;
    break;
};
    }
    __any_8
};
if __cond {
    __filtered_6.push(__elem_7);
}
};
    }
    Rc::new(__filtered_6)
}).iter().cloned() {
        __mapped_10.push(Rc::new(TestProjection { service_name: __elem_5.name.clone(), operation_name: __elem_11.name.clone(), mock_field_inits: {
    let mut __filtered_12 = Vec::new();
    for __elem_13 in __elem_11.properties.iter().cloned() {
        if has_mock_prefix(&__elem_13.name) {
    __filtered_12.push(__elem_13);
};
    }
    Rc::new(__filtered_12)
} }));
    }
    Rc::new(__mapped_10)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InterpPart {
    pub format_segment: String,
    pub arg_expr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmitContext {
    pub field_access_index: Rc<HashMap<String, Rc<FieldSummary>>>,
    pub intrinsic_index: Rc<HashMap<String, IntrinsicMethod>>,
    pub bridge_index: Rc<HashMap<String, bool>>,
    pub vtoe_index: Rc<HashMap<String, String>>,
    pub primitive_set: Rc<HashMap<String, bool>>,
}

pub fn empty_emit_scope() -> Rc<InferScope> {
    Rc::new(InferScope { type_env: Rc::new(TypeEnv { bindings: Rc::new(std::collections::HashMap::new()), recursive_types: Rc::new(Vec::new()), recursive_type_set: Rc::new(std::collections::HashMap::new()) }), func_env: Rc::new(ResolvedFuncEnv { signatures: Rc::new(std::collections::HashMap::new()) }), locals: Rc::new(std::collections::HashMap::new()), module_name: "".to_string(), service_registry: Rc::new(std::collections::HashMap::new()), item_registry: Rc::new(std::collections::HashMap::new()) })
}

pub fn module_emit_scope(typed_module: Rc<TypedModule>) -> Rc<InferScope> {
    Rc::new(InferScope { type_env: typed_module.type_env.clone(), func_env: typed_module.func_env.clone(), locals: Rc::new(std::collections::HashMap::new()), module_name: typed_module.module.name.clone(), service_registry: Rc::new(std::collections::HashMap::new()), item_registry: typed_module.item_registry.clone() })
}

pub fn scope_after_expr(texpr: Rc<Node>, scope: Rc<InferScope>) -> Rc<InferScope> {
    match texpr.expr_data.as_ref() {
    ExprData::ExprLet { name, value, body, .. } => {
        if body.clone().is_none() {
    extend_scope(scope.clone(), &name, rt_type(value.clone()))
} else {
    scope.clone()
}
    }
    _ => {
        scope.clone()
    }
}
}

pub fn lookup_item(registry: Rc<HashMap<String, Rc<ItemInfo>>>, name: &str) -> Option<Rc<ItemInfo>> {
    registry.clone().get(&name.to_string()).cloned()
}

pub fn lookup_func_sig_in_scope(scope: Rc<InferScope>, name: &str) -> Option<Rc<ResolvedFuncSig>> {
    scope.func_env.signatures.clone().get(&name.to_string()).cloned()
}

pub fn typed_named_arg_matches(arg: Rc<NamedArg>, name: &str) -> bool {
    if arg.name.clone().is_none() {
    false
} else {
    arg.name.clone().unwrap() == name
}
}

pub fn order_typed_call_args(args: Rc<Vec<Rc<NamedArg>>>, func: &str, scope: Rc<InferScope>) -> Rc<Vec<Rc<NamedArg>>> {
    let has_unnamed = {
    let mut __any_0 = false;
    for __elem_1 in args.iter().cloned() {
        if __elem_1.name.clone().is_none() {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    if has_unnamed.clone() {
    args.clone()
} else {
    match lookup_func_sig_in_scope(scope.clone(), &func) {
    None => {
        args.clone()
    }
    Some(sig) => {
        {
    let arg_map = {
    let mut __acc_2: Rc<std::collections::HashMap<String, Rc<NamedArg>>> = Rc::new(std::collections::HashMap::new());
    for __elem_3 in args.iter().cloned() {
        __acc_2 = if __elem_3.name.clone().is_some() {
    {
    let __rc_5 = __acc_2;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert(__elem_3.name.clone().unwrap(), __elem_3.clone());
    Rc::new(__map_ins_4)
}
} else {
    __acc_2.clone()
};
    }
    __acc_2
};
    let param_name_set = {
    let mut __acc_6 = Rc::new(std::collections::HashMap::new());
    for __elem_7 in sig.params.iter().cloned() {
        __acc_6 = {
    let __rc_9 = __acc_6;
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert(__elem_7.name.clone(), true);
    Rc::new(__map_ins_8)
};
    }
    __acc_6
};
    let ordered = {
    let mut __flat_mapped_10 = Vec::new();
    for __elem_11 in sig.params.iter().cloned() {
        __flat_mapped_10.extend((match arg_map.clone().get(&__elem_11.name.clone()).cloned() {
    Some(arg) => {
        Rc::new(vec!(arg.clone()))
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_10)
};
    let leftovers = {
    let mut __filtered_12 = Vec::new();
    for __elem_13 in args.iter().cloned() {
        if if __elem_13.name.clone().is_none() {
    true
} else {
    emit_map_has(param_name_set.clone(), &__elem_13.name.clone().unwrap()) == false
} {
    __filtered_12.push(__elem_13);
};
    }
    Rc::new(__filtered_12)
};
    v2_rt::concat(ordered.clone(), leftovers.clone())
}
    }
}
}
}

pub fn unique_strings(items: Rc<Vec<String>>) -> Rc<Vec<String>> {
    let result = {
    let mut __acc_0 = Rc::new(UniqueAccum { seen: Rc::new(std::collections::HashMap::new()), result: Rc::new(Vec::new()) });
    for __elem_1 in items.iter().cloned() {
        __acc_0 = if emit_map_has(__acc_0.seen.clone(), &__elem_1) {
    __acc_0.clone()
} else {
    Rc::new(UniqueAccum { seen: {
    let __rc_3 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).seen);
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.clone(), true);
    Rc::new(__map_ins_2)
}, result: {
    let __rc_5 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).result);
    let mut __appended_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __appended_4.push(__elem_1.clone());
    Rc::new(__appended_4)
} })
};
    }
    __acc_0
};
    result.result.clone()
}

pub fn build_emit_context(graph: Rc<ResolvedGraph>) -> Rc<EmitContext> {
    let info = build_emit_graph_info(graph.modules.clone());
    let field_idx = {
    let mut __acc_4 = Rc::new(std::collections::HashMap::new());
    for __elem_5 in ({
    let __rc_0 = info.type_summaries.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = {
    let mut __acc_9 = __acc_4.clone();
    for __elem_10 in ({
    let __rc_6 = __elem_5.field_summaries.clone();
    let __map_unwrapped_7 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| (*rc).clone());
    let mut __keys_8 = __map_unwrapped_7.into_keys().collect::<Vec<_>>();
    __keys_8.sort();
    Rc::new(__keys_8)
}).iter().cloned() {
        __acc_9 = match __elem_5.field_summaries.clone().get(&__elem_10.clone()).cloned() {
    Some(fs) => {
        {
    let __rc_12 = __acc_9;
    let mut __map_ins_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_11.insert(v2_rt::concat(v2_rt::concat(__elem_5.name.clone(), "::".to_string()), __elem_10.clone()), fs.clone());
    Rc::new(__map_ins_11)
}
    }
    None => {
        __acc_9.clone()
    }
};
    }
    __acc_9
};
    }
    __acc_4
};
    let intrinsic_idx = intrinsic_method_index();
    let bridge_idx = Rc::new(std::collections::HashMap::new());
    let vtoe_idx = info.variant_to_enum.clone();
    let prim_set = {
    let mut __acc_13 = Rc::new(std::collections::HashMap::new());
    for __elem_14 in Rc::new(KERNEL_TYPES.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        __acc_13 = {
    let __rc_16 = __acc_13;
    let mut __map_ins_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_15.insert(__elem_14.clone(), true);
    Rc::new(__map_ins_15)
};
    }
    __acc_13
};
    Rc::new(EmitContext { field_access_index: field_idx.clone(), intrinsic_index: intrinsic_idx.clone(), bridge_index: bridge_idx.clone(), vtoe_index: vtoe_idx.clone(), primitive_set: prim_set.clone() })
}

pub fn has_nested_records_node(n: Rc<Node>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_n = n;
        loop {
            let n = __tco_p_n;
            let n_kind = classify_type_structure(n.clone());
            if n_kind.clone() == TypeStructureKind::TypeConj {
    break true;
} else {
    if n_kind.clone() == TypeStructureKind::TypeDisj {
    if node_is_optional(n.clone()) {
     {
        let __tco_0 = with_required_cardinality(n.clone());
        __tco_p_n = __tco_0;
        continue;
    }

} else {
    break false;
};
} else {
    if node_is_map(n.clone()) {
    match n.children.clone().get((1_i64) as usize).cloned() {
    Some(val_child) => {
         {
            let __tco_0 = val_child.clone();
            __tco_p_n = __tco_0;
            continue;
        }

    }
    None => {
        break false;
    }
};
} else {
    if ({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 1_i64 {
    match n.children.clone().first().cloned() {
    Some(el) => {
         {
            let __tco_0 = el.clone();
            __tco_p_n = __tco_0;
            continue;
        }

    }
    None => {
        break false;
    }
};
} else {
    break false;
};
};
};
};
        }
    })
}

pub fn emit_data_value_json(value: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match value.expr_data.as_ref() {
    ExprData::ExprLiteral { value: v, .. } => {
        match v.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        v2_rt::concat(v2_rt::concat("\"".to_string(), escape_json_string(&s)), "\"".to_string())
    }
    LiteralValue::LitInt { value: i, .. } => {
        v2_rt::to_string(i.clone())
    }
    LiteralValue::LitFloat { value: f, .. } => {
        f.clone()
    }
    LiteralValue::LitBool { value: b, .. } => {
        if b.clone() {
    "true".to_string()
} else {
    "false".to_string()
}
    }
    LiteralValue::LitNull => {
        "null".to_string()
    }
}
    }
    ExprData::ExprListLit { elements: els, .. } => {
        {
    let el_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in els.iter().cloned() {
        __mapped_0.push(emit_data_value_json(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    v2_rt::concat(v2_rt::concat("[".to_string(), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in el_strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), "]".to_string())
}
    }
    ExprData::ExprRecordLit { type_name: _, fields: fs, parent_enum: _, .. } => {
        {
    let field_strs = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in fs.iter().cloned() {
        __mapped_5.push(v2_rt::concat(v2_rt::concat(v2_rt::concat("\"".to_string(), escape_json_string(&__elem_6.name)), "\": ".to_string()), emit_data_value_json(__elem_6.value.clone())));
    }
    Rc::new(__mapped_5)
};
    v2_rt::concat(v2_rt::concat("{".to_string(), {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in field_strs.iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&", ".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
}), "}".to_string())
}
    }
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        v2_rt::concat(v2_rt::concat("\"".to_string(), escape_json_string(&n)), "\"".to_string())
    }
    _ => {
        "<<<UNSUPPORTED_MOCK_EXPR>>>".to_string()
    }
}
    })
}

pub fn escape_json_string(s: &str) -> String {
    {
    let mut __joined_17 = String::new();
    let mut __first_19 = true;
    for __elem_18 in ({
    let mut __split_parts_15 = Vec::new();
    for __part_16 in ({
    let mut __joined_12 = String::new();
    let mut __first_14 = true;
    for __elem_13 in ({
    let mut __split_parts_10 = Vec::new();
    for __part_11 in ({
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in ({
    let mut __split_parts_5 = Vec::new();
    for __part_6 in ({
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in s.to_string().split("\\".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\\\\".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}).split("\"".to_string().as_str()) {
        __split_parts_5.push(__part_6.to_string());
    }
    __split_parts_5
}).iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&"\\\"".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
}).split("\n".to_string().as_str()) {
        __split_parts_10.push(__part_11.to_string());
    }
    __split_parts_10
}).iter().cloned() {
        if !__first_14 {
    __joined_12.push_str(&"\\n".to_string());
};
        __first_14 = false;
        __joined_12.push_str(&__elem_13);
    }
    __joined_12
}).split("	".to_string().as_str()) {
        __split_parts_15.push(__part_16.to_string());
    }
    __split_parts_15
}).iter().cloned() {
        if !__first_19 {
    __joined_17.push_str(&"\\t".to_string());
};
        __first_19 = false;
        __joined_17.push_str(&__elem_18);
    }
    __joined_17
}
}

pub fn module_to_filename(name: &str) -> String {
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in name.to_string().split(".".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"_".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
}

pub fn make_indent(level: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if level.clone() <= 0_i64 {
    "".to_string()
} else {
    v2_rt::concat("    ".to_string(), make_indent(level.clone() - 1_i64))
}
    })
}

pub fn to_string(value: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if value.clone() < 0_i64 {
    v2_rt::concat("-".to_string(), v2_rt::to_string(0_i64 - value.clone()))
} else {
    if value.clone() == 0_i64 {
    "0".to_string()
} else {
    let digit_chars = Rc::new(vec!("0".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string()));
    {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in to_string_helper(value.clone(), Rc::new(Vec::new())).iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&"".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}
}
}
    })
}

pub fn to_string_helper(value: i64, acc: Rc<Vec<String>>) -> Rc<Vec<String>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_value = value;
        let mut __tco_p_acc = acc;
        loop {
            let value = __tco_p_value;
            let acc = __tco_p_acc;
            if value.clone() == 0_i64 {
    break acc.clone();
} else {
    let digit = value.clone() % 10_i64;
    let rest = (value.clone() - digit.clone()) / 10_i64;
    let digit_chars = Rc::new(vec!("0".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string()));
    let ch = match {
    let mut __found_8 = None;
    for __elem_9 in ({
    let mut __enumerated_5 = Vec::new();
    for (__idx_6, __elem_7) in digit_chars.clone().iter().enumerate() {
        __enumerated_5.push((__idx_6 as i64, __elem_7.clone()));
    }
    Rc::new(__enumerated_5)
}).iter().cloned() {
        if __elem_9.0.clone() == digit.clone() {
    __found_8 = Some(__elem_9);
    break;
};
    }
    __found_8
} {
    Some(p) => {
        p.1.clone()
    }
    None => {
        "?".to_string()
    }
};
     {
        let __tco_0 = rest.clone();
        let __tco_1 = v2_rt::concat(Rc::new(vec!(ch.clone())), acc.clone());
        __tco_p_value = __tco_0;
        __tco_p_acc = __tco_1;
        continue;
    }

};
        }
    })
}

pub fn to_snake(name: &str) -> String {
    let chars_list = {
    let mut __chars_0 = Vec::new();
    for __ch_1 in name.to_string().chars() {
        __chars_0.push(__ch_1.to_string());
    }
    Rc::new(__chars_0)
};
    let result = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in ({
    let mut __enumerated_2 = Vec::new();
    for (__idx_3, __elem_4) in chars_list.clone().iter().enumerate() {
        __enumerated_2.push((__idx_3 as i64, __elem_4.clone()));
    }
    Rc::new(__enumerated_2)
}).iter().cloned() {
        __mapped_5.push({
    let idx = __elem_6.0.clone();
    let ch = __elem_6.1.clone();
    if is_upper(&ch) {
    if idx.clone() == 0_i64 {
    to_lower_char(&ch)
} else {
    v2_rt::concat("_".to_string(), to_lower_char(&ch))
}
} else {
    ch.clone()
}
});
    }
    Rc::new(__mapped_5)
};
    {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in result.iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&"".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
}
}

pub fn to_screaming_snake(name: &str) -> String {
    let snake = to_snake(&name);
    {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in ({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in ({
    let mut __chars_0 = Vec::new();
    for __ch_1 in snake.chars() {
        __chars_0.push(__ch_1.to_string());
    }
    Rc::new(__chars_0)
}).iter().cloned() {
        __mapped_2.push(to_upper_char(&__elem_3));
    }
    Rc::new(__mapped_2)
}).iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
}
}

pub fn is_upper(ch: &str) -> bool {
    (ch >= "A") && (ch <= "Z")
}

pub fn to_lower_char(ch: &str) -> String {
    let cp = v2_rt::code_point(&ch);
    if (cp.clone() >= 65_i64) && (cp.clone() <= 90_i64) {
    let lower_cp = cp.clone() + 32_i64;
    v2_rt::from_code_point(lower_cp.clone())
} else {
    ch.to_string()
}
}

pub fn to_upper_char(ch: &str) -> String {
    let cp = v2_rt::code_point(&ch);
    if (cp.clone() >= 97_i64) && (cp.clone() <= 122_i64) {
    let upper_cp = cp.clone() - 32_i64;
    v2_rt::from_code_point(upper_cp.clone())
} else {
    ch.to_string()
}
}

pub fn sanitize_service_name(name: &str) -> String {
    let parts = {
    let mut __split_parts_0 = Vec::new();
    for __part_1 in name.to_string().split(".".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
};
    let pascal_parts = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in parts.iter().cloned() {
        __mapped_2.push(capitalize_first(&__elem_3));
    }
    Rc::new(__mapped_2)
};
    {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in pascal_parts.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
}
}

pub fn capitalize_first(s: &str) -> String {
    let chars_list = {
    let mut __chars_0 = Vec::new();
    for __ch_1 in s.to_string().chars() {
        __chars_0.push(__ch_1.to_string());
    }
    Rc::new(__chars_0)
};
    if ({
    let __len_10 = chars_list.clone().len();
    __len_10 as i64
}) == 0_i64 {
    "".to_string()
} else {
    {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in ({
    let mut __mapped_5 = Vec::new();
    for __elem_6 in ({
    let mut __enumerated_2 = Vec::new();
    for (__idx_3, __elem_4) in chars_list.clone().iter().enumerate() {
        __enumerated_2.push((__idx_3 as i64, __elem_4.clone()));
    }
    Rc::new(__enumerated_2)
}).iter().cloned() {
        __mapped_5.push(if __elem_6.0.clone() == 0_i64 {
    to_upper_char(&__elem_6.1)
} else {
    __elem_6.1.clone()
});
    }
    Rc::new(__mapped_5)
}).iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&"".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
}
}
}

pub fn service_var_name(service_name: &str) -> String {
    to_snake(&sanitize_service_name(&service_name))
}

pub fn apply_type_template1(template: &str, arg0: &str) -> String {
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in template.to_string().split("{0}".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&arg0.to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
}

pub fn apply_type_template2(template: &str, arg0: &str, arg1: &str) -> String {
    {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in ({
    let mut __split_parts_5 = Vec::new();
    for __part_6 in ({
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in template.to_string().split("{0}".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&arg0.to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}).split("{1}".to_string().as_str()) {
        __split_parts_5.push(__part_6.to_string());
    }
    __split_parts_5
}).iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&arg1.to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
}
}

pub fn emit_string_literal(s: &str, suffix: &str) -> String {
    let escaped = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in s.to_string().split("\\".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\\\\".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let escaped2 = {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in ({
    let mut __split_parts_5 = Vec::new();
    for __part_6 in escaped.clone().split("\"".to_string().as_str()) {
        __split_parts_5.push(__part_6.to_string());
    }
    __split_parts_5
}).iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&"\\\"".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat("\"".to_string(), escaped2.clone()), "\"".to_string()), suffix.to_string())
}

pub fn is_null_coalesce(op: BinOpKind) -> bool {
    match op {
    BinOpKind::NullCoalesce => {
        true
    }
    _ => {
        false
    }
}
}

pub fn rust_keyword(key: &str) -> String {
    match v2_rt::lookup(&RUST_KEYWORDS, key.to_string()) {
    Some(kw) => {
        kw.clone()
    }
    None => {
        key.to_string()
    }
}
}

pub fn python_keyword(key: &str) -> String {
    match v2_rt::lookup(&PYTHON_KEYWORDS, key.to_string()) {
    Some(kw) => {
        kw.clone()
    }
    None => {
        key.to_string()
    }
}
}

pub fn go_keyword(key: &str) -> String {
    match v2_rt::lookup(&GO_KEYWORDS, key.to_string()) {
    Some(kw) => {
        kw.clone()
    }
    None => {
        key.to_string()
    }
}
}

pub fn rust_container(kind: &str, inner: &str) -> String {
    match v2_rt::lookup(&RUST_CONTAINER_TEMPLATES, kind.to_string()) {
    Some(tmpl) => {
        apply_type_template1(&tmpl, &inner)
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(kind.to_string(), "<".to_string()), inner.to_string()), ">".to_string())
    }
}
}

pub fn python_container(kind: &str, inner: &str) -> String {
    match v2_rt::lookup(&PYTHON_CONTAINER_TEMPLATES, kind.to_string()) {
    Some(tmpl) => {
        apply_type_template1(&tmpl, &inner)
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(kind.to_string(), "[".to_string()), inner.to_string()), "]".to_string())
    }
}
}

pub fn go_container(kind: &str, inner: &str) -> String {
    match v2_rt::lookup(&GO_CONTAINER_TEMPLATES, kind.to_string()) {
    Some(tmpl) => {
        apply_type_template1(&tmpl, &inner)
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(kind.to_string(), "[".to_string()), inner.to_string()), "]".to_string())
    }
}
}

pub fn rust_primitive_type(name: &str) -> String {
    match v2_rt::lookup(&RUST_TYPE_MAP, name.to_string()) {
    Some(m) => {
        m.clone()
    }
    None => {
        name.to_string()
    }
}
}

pub fn python_primitive_type(name: &str) -> String {
    match v2_rt::lookup(&PYTHON_TYPE_MAP, name.to_string()) {
    Some(m) => {
        m.clone()
    }
    None => {
        name.to_string()
    }
}
}

pub fn go_primitive_type(name: &str) -> String {
    match v2_rt::lookup(&GO_TYPE_MAP, name.to_string()) {
    Some(m) => {
        m.clone()
    }
    None => {
        name.to_string()
    }
}
}

pub fn rust_map_template() -> String {
    match v2_rt::lookup(&RUST_CONTAINER_TEMPLATES, "map".to_string()) {
    Some(tmpl) => {
        tmpl.clone()
    }
    None => {
        "BTreeMap<{0}, {1}>".to_string()
    }
}
}

pub fn python_map_template() -> String {
    match v2_rt::lookup(&PYTHON_CONTAINER_TEMPLATES, "map".to_string()) {
    Some(tmpl) => {
        tmpl.clone()
    }
    None => {
        "dict[{0}, {1}]".to_string()
    }
}
}

pub fn go_map_template() -> String {
    match v2_rt::lookup(&GO_CONTAINER_TEMPLATES, "map".to_string()) {
    Some(tmpl) => {
        tmpl.clone()
    }
    None => {
        "map[{0}]{1}".to_string()
    }
}
}

pub fn rust_literal(value: Rc<LiteralValue>) -> String {
    match value.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        emit_string_literal(&s, ".to_string()")
    }
    LiteralValue::LitInt { value: i, .. } => {
        v2_rt::to_string(i.clone())
    }
    LiteralValue::LitFloat { value: f, .. } => {
        f.clone()
    }
    LiteralValue::LitBool { value: b, .. } => {
        if b.clone() {
    rust_keyword("true")
} else {
    rust_keyword("false")
}
    }
    LiteralValue::LitNull => {
        rust_keyword("null")
    }
}
}

pub fn rust_literal_for_pattern(value: Rc<LiteralValue>) -> String {
    match value.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        emit_string_literal(&s, "")
    }
    LiteralValue::LitInt { value: i, .. } => {
        v2_rt::to_string(i.clone())
    }
    LiteralValue::LitFloat { value: f, .. } => {
        f.clone()
    }
    LiteralValue::LitBool { value: b, .. } => {
        if b.clone() {
    rust_keyword("true")
} else {
    rust_keyword("false")
}
    }
    LiteralValue::LitNull => {
        rust_keyword("null")
    }
}
}

pub fn python_literal(value: Rc<LiteralValue>) -> String {
    match value.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        emit_string_literal(&s, "")
    }
    LiteralValue::LitInt { value: i, .. } => {
        v2_rt::to_string(i.clone())
    }
    LiteralValue::LitFloat { value: f, .. } => {
        f.clone()
    }
    LiteralValue::LitBool { value: b, .. } => {
        if b.clone() {
    python_keyword("true")
} else {
    python_keyword("false")
}
    }
    LiteralValue::LitNull => {
        python_keyword("null")
    }
}
}

pub fn go_literal(value: Rc<LiteralValue>) -> String {
    match value.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        emit_string_literal(&s, "")
    }
    LiteralValue::LitInt { value: i, .. } => {
        v2_rt::to_string(i.clone())
    }
    LiteralValue::LitFloat { value: f, .. } => {
        f.clone()
    }
    LiteralValue::LitBool { value: b, .. } => {
        if b.clone() {
    go_keyword("true")
} else {
    go_keyword("false")
}
    }
    LiteralValue::LitNull => {
        go_keyword("null")
    }
}
}

pub fn rust_bin_op_symbol(op: BinOpKind) -> String {
    match op {
    BinOpKind::Add => {
        "+".to_string()
    }
    BinOpKind::Sub => {
        "-".to_string()
    }
    BinOpKind::Mul => {
        "*".to_string()
    }
    BinOpKind::Div => {
        rust_keyword("div")
    }
    BinOpKind::Mod => {
        "%".to_string()
    }
    BinOpKind::BinEq => {
        "==".to_string()
    }
    BinOpKind::BinNe => {
        "!=".to_string()
    }
    BinOpKind::BinLt => {
        "<".to_string()
    }
    BinOpKind::BinGt => {
        ">".to_string()
    }
    BinOpKind::BinLe => {
        "<=".to_string()
    }
    BinOpKind::BinGe => {
        ">=".to_string()
    }
    BinOpKind::BinAnd => {
        rust_keyword("and")
    }
    BinOpKind::BinOr => {
        rust_keyword("or")
    }
    BinOpKind::NullCoalesce => {
        "??".to_string()
    }
}
}

pub fn python_bin_op_symbol(op: BinOpKind) -> String {
    match op {
    BinOpKind::Add => {
        "+".to_string()
    }
    BinOpKind::Sub => {
        "-".to_string()
    }
    BinOpKind::Mul => {
        "*".to_string()
    }
    BinOpKind::Div => {
        python_keyword("div")
    }
    BinOpKind::Mod => {
        "%".to_string()
    }
    BinOpKind::BinEq => {
        "==".to_string()
    }
    BinOpKind::BinNe => {
        "!=".to_string()
    }
    BinOpKind::BinLt => {
        "<".to_string()
    }
    BinOpKind::BinGt => {
        ">".to_string()
    }
    BinOpKind::BinLe => {
        "<=".to_string()
    }
    BinOpKind::BinGe => {
        ">=".to_string()
    }
    BinOpKind::BinAnd => {
        python_keyword("and")
    }
    BinOpKind::BinOr => {
        python_keyword("or")
    }
    BinOpKind::NullCoalesce => {
        "??".to_string()
    }
}
}

pub fn go_bin_op_symbol(op: BinOpKind) -> String {
    match op {
    BinOpKind::Add => {
        "+".to_string()
    }
    BinOpKind::Sub => {
        "-".to_string()
    }
    BinOpKind::Mul => {
        "*".to_string()
    }
    BinOpKind::Div => {
        go_keyword("div")
    }
    BinOpKind::Mod => {
        "%".to_string()
    }
    BinOpKind::BinEq => {
        "==".to_string()
    }
    BinOpKind::BinNe => {
        "!=".to_string()
    }
    BinOpKind::BinLt => {
        "<".to_string()
    }
    BinOpKind::BinGt => {
        ">".to_string()
    }
    BinOpKind::BinLe => {
        "<=".to_string()
    }
    BinOpKind::BinGe => {
        ">=".to_string()
    }
    BinOpKind::BinAnd => {
        go_keyword("and")
    }
    BinOpKind::BinOr => {
        go_keyword("or")
    }
    BinOpKind::NullCoalesce => {
        "??".to_string()
    }
}
}

pub fn emit_keyword(key: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        rust_keyword(&key)
    }
    RenderTarget::Go => {
        go_keyword(&key)
    }
    RenderTarget::Python => {
        python_keyword(&key)
    }
    RenderTarget::Dag => {
        key.to_string()
    }
}
}

pub fn emit_literal(value: Rc<LiteralValue>, target: RenderTarget) -> String {
    match value.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        {
    let suffix = match target {
    RenderTarget::Rust => {
        ".to_string()".to_string()
    }
    _ => {
        "".to_string()
    }
};
    emit_string_literal(&s, &suffix)
}
    }
    LiteralValue::LitInt { value: i, .. } => {
        v2_rt::to_string(i.clone())
    }
    LiteralValue::LitFloat { value: f, .. } => {
        f.clone()
    }
    LiteralValue::LitBool { value: b, .. } => {
        if b.clone() {
    emit_keyword("true", target)
} else {
    emit_keyword("false", target)
}
    }
    LiteralValue::LitNull => {
        emit_keyword("null", target)
    }
}
}

pub fn emit_bin_op_symbol(op: BinOpKind, target: RenderTarget) -> String {
    match op {
    BinOpKind::Add => {
        "+".to_string()
    }
    BinOpKind::Sub => {
        "-".to_string()
    }
    BinOpKind::Mul => {
        "*".to_string()
    }
    BinOpKind::Div => {
        emit_keyword("div", target)
    }
    BinOpKind::Mod => {
        "%".to_string()
    }
    BinOpKind::BinEq => {
        "==".to_string()
    }
    BinOpKind::BinNe => {
        "!=".to_string()
    }
    BinOpKind::BinLt => {
        "<".to_string()
    }
    BinOpKind::BinGt => {
        ">".to_string()
    }
    BinOpKind::BinLe => {
        "<=".to_string()
    }
    BinOpKind::BinGe => {
        ">=".to_string()
    }
    BinOpKind::BinAnd => {
        emit_keyword("and", target)
    }
    BinOpKind::BinOr => {
        emit_keyword("or", target)
    }
    BinOpKind::NullCoalesce => {
        "??".to_string()
    }
}
}

pub fn emit_primitive_type(name: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        rust_primitive_type(&name)
    }
    RenderTarget::Go => {
        go_primitive_type(&name)
    }
    RenderTarget::Python => {
        python_primitive_type(&name)
    }
    RenderTarget::Dag => {
        name.to_string()
    }
}
}

pub fn emit_container(kind: &str, inner: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        rust_container(&kind, &inner)
    }
    RenderTarget::Go => {
        go_container(&kind, &inner)
    }
    RenderTarget::Python => {
        python_container(&kind, &inner)
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(kind.to_string(), "<".to_string()), inner.to_string()), ">".to_string())
    }
}
}

pub fn emit_map_type(key_type: &str, val_type: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        apply_type_template2(&rust_map_template(), &key_type, &val_type)
    }
    RenderTarget::Go => {
        apply_type_template2(&go_map_template(), &key_type, &val_type)
    }
    RenderTarget::Python => {
        apply_type_template2(&python_map_template(), &key_type, &val_type)
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Map<".to_string(), key_type.to_string()), ", ".to_string()), val_type.to_string()), ">".to_string())
    }
}
}

pub fn emit_node_type(n: Rc<Node>, target: RenderTarget) -> String {
    emit_node_type_rc(n.clone(), target, Rc::new(std::collections::HashMap::new()))
}

pub fn emit_node_type_rc(n: Rc<Node>, target: RenderTarget, rc_types: Rc<HashMap<String, bool>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let is_rust = match target.clone() {
    RenderTarget::Rust => {
        true
    }
    _ => {
        false
    }
};
        if ((n.name.clone() == "Dynamic") || (n.name.clone() == "Error")) && (({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64) {
    return if is_rust.clone() {
    v2_rt::concat(v2_rt::concat("compile_error!(\"unresolved ".to_string(), n.name.clone()), " type reached emit\")".to_string())
} else {
    v2_rt::concat(v2_rt::concat("__EMIT_BUG_UNRESOLVED_".to_string(), n.name.clone()), "__".to_string())
};
};
        if (n.name.clone() == "Callable") && (({
    let __len_6 = n.params.clone().len();
    __len_6 as i64
}) > 0_i64) {
    let param_types = {
    let mut __mapped_1 = Vec::new();
    for __elem_2 in n.params.iter().cloned() {
        __mapped_1.push(emit_node_type_rc(__elem_2.type_expr.clone(), target.clone(), rc_types.clone()));
    }
    Rc::new(__mapped_1)
};
    let param_str = {
    let mut __joined_3 = String::new();
    let mut __first_5 = true;
    for __elem_4 in param_types.iter().cloned() {
        if !__first_5 {
    __joined_3.push_str(&", ".to_string());
};
        __first_5 = false;
        __joined_3.push_str(&__elem_4);
    }
    __joined_3
};
    let ret_str = match n.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        emit_node_type_rc(rt.clone(), target.clone(), rc_types.clone())
    }
    _ => {
        match target.clone() {
    RenderTarget::Go => {
        "".to_string()
    }
    RenderTarget::Python => {
        "None".to_string()
    }
    _ => {
        "()".to_string()
    }
}
    }
};
    match target.clone() {
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("func(".to_string(), param_str.clone()), ")".to_string()), if ret_str.clone() != "" {
    v2_rt::concat(" ".to_string(), ret_str.clone())
} else {
    "".to_string()
})
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Callable[[".to_string(), param_str.clone()), "], ".to_string()), ret_str.clone()), "]".to_string())
    }
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("impl Fn(".to_string(), param_str.clone()), ") -> ".to_string()), ret_str.clone())
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("Fn(".to_string(), param_str.clone()), ") -> ".to_string()), ret_str.clone())
    }
}
} else {
    if node_is_optional(n.clone()) {
    emit_container("optional", &emit_node_type_rc(with_required_cardinality(n.clone()), target.clone(), rc_types.clone()), target.clone())
} else {
    let kind = classify_type_structure(n.clone());
    let is_leaf = kind.clone() == TypeStructureKind::TypeLeaf;
    let is_conj = kind.clone() == TypeStructureKind::TypeConj;
    if is_leaf.clone() {
    emit_node_type_leaf_rc(n.clone(), target.clone(), rc_types.clone())
} else {
    if is_conj.clone() {
    emit_node_type_conj_rc(n.clone(), target.clone(), rc_types.clone())
} else {
    emit_node_type_disj_rc(n.clone(), target.clone(), rc_types.clone())
}
}
}
}
    })
}

pub fn emit_node_type_leaf_rc(n: Rc<Node>, target: RenderTarget, rc_types: Rc<HashMap<String, bool>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if ({
    let __len_1 = n.children.clone().len();
    __len_1 as i64
}) == 0_i64 {
    let base = if n.name.clone() == "Map" {
    emit_map_type("_", "_", target.clone())
} else {
    if (((n.name.clone() == "List") || (n.name.clone() == "Set")) || (n.name.clone() == "NonEmptyList")) || (n.name.clone() == "NonEmptySet") {
    emit_container(&to_snake(&n.name), "_", target.clone())
} else {
    emit_primitive_type(&n.name, target.clone())
}
};
    if emit_map_has(rc_types.clone(), &n.name) {
    v2_rt::concat(v2_rt::concat("Rc<".to_string(), base.clone()), ">".to_string())
} else {
    base.clone()
}
} else {
    if node_is_map(n.clone()) {
    let k = match n.children.clone().first().cloned() {
    Some(kn) => {
        emit_node_type_rc(kn.clone(), target.clone(), rc_types.clone())
    }
    None => {
        "__EMIT_BUG_MISSING_MAP_KEY__".to_string()
    }
};
    let v = match n.children.clone().get((1_i64) as usize).cloned() {
    Some(vn) => {
        emit_node_type_rc(vn.clone(), target.clone(), rc_types.clone())
    }
    None => {
        "__EMIT_BUG_MISSING_MAP_VALUE__".to_string()
    }
};
    emit_map_type(&k, &v, target.clone())
} else {
    if ({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 1_i64 {
    let inner = match n.children.clone().first().cloned() {
    Some(child) => {
        emit_node_type_rc(child.clone(), target.clone(), rc_types.clone())
    }
    None => {
        "__EMIT_BUG_MISSING_CONTAINER_ELEMENT__".to_string()
    }
};
    let is_rust = match target.clone() {
    RenderTarget::Rust => {
        true
    }
    _ => {
        false
    }
};
    let container_kind = if n.name.clone() == "NonEmptyList" {
    if is_rust.clone() {
    "non_empty_list".to_string()
} else {
    "list".to_string()
}
} else {
    if n.name.clone() == "NonEmptySet" {
    if is_rust.clone() {
    "non_empty_set".to_string()
} else {
    "set".to_string()
}
} else {
    to_snake(&n.name)
}
};
    emit_container(&container_kind, &inner, target.clone())
} else {
    n.name.clone()
}
}
}
    })
}

pub fn emit_node_type_conj_rc(n: Rc<Node>, target: RenderTarget, rc_types: Rc<HashMap<String, bool>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if n.name.clone() == "Refined" {
    match n.children.clone().first().cloned() {
    Some(base) => {
        emit_node_type_rc(base.clone(), target.clone(), rc_types.clone())
    }
    None => {
        n.name.clone()
    }
}
} else {
    if n.name.clone() == "Tuple" {
    let first_str = match n.children.clone().first().cloned() {
    Some(c) => {
        if c.return_type.clone().is_some() {
    emit_node_type_rc(rt_type(c.clone()), target.clone(), rc_types.clone())
} else {
    emit_node_type_rc(c.clone(), target.clone(), rc_types.clone())
}
    }
    None => {
        "__EMIT_BUG_MISSING_TUPLE_FIRST__".to_string()
    }
};
    let second_str = match n.children.clone().get((1_i64) as usize).cloned() {
    Some(c) => {
        if c.return_type.clone().is_some() {
    emit_node_type_rc(rt_type(c.clone()), target.clone(), rc_types.clone())
} else {
    emit_node_type_rc(c.clone(), target.clone(), rc_types.clone())
}
    }
    None => {
        "__EMIT_BUG_MISSING_TUPLE_SECOND__".to_string()
    }
};
    match target.clone() {
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("struct{ First ".to_string(), first_str.clone()), "; Second ".to_string()), second_str.clone()), " }".to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Tuple[".to_string(), first_str.clone()), ", ".to_string()), second_str.clone()), "]".to_string())
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), first_str.clone()), ", ".to_string()), second_str.clone()), ")".to_string())
    }
}
} else {
    if n.name.clone() != "" {
    if emit_map_has(rc_types.clone(), &n.name) {
    v2_rt::concat(v2_rt::concat("Rc<".to_string(), n.name.clone()), ">".to_string())
} else {
    n.name.clone()
}
} else {
    let is_rust = match target.clone() {
    RenderTarget::Rust => {
        true
    }
    _ => {
        false
    }
};
    if is_rust.clone() {
    if ({
    let __len_5 = n.children.clone().len();
    __len_5 as i64
}) == 1_i64 {
    match n.children.clone().first().cloned() {
    Some(field_node) => {
        if field_node.return_type.clone().is_some() {
    emit_node_type_rc(rt_type(field_node.clone()), target.clone(), rc_types.clone())
} else {
    "compile_error!(\"anonymous product field missing return_type\")".to_string()
}
    }
    None => {
        "compile_error!(\"anonymous product reached Node emitter\")".to_string()
    }
}
} else {
    let field_types = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in n.children.iter().cloned() {
        __mapped_0.push(if __elem_1.return_type.clone().is_some() {
    emit_node_type_rc(rt_type(__elem_1.clone()), target.clone(), rc_types.clone())
} else {
    "compile_error!(\"anonymous product field missing return_type\")".to_string()
});
    }
    Rc::new(__mapped_0)
};
    let result = v2_rt::concat(v2_rt::concat("(".to_string(), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in field_types.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), ")".to_string());
    result.clone()
}
} else {
    "__EMIT_BUG_ANONYMOUS_CONJ__".to_string()
}
}
}
}
    })
}

pub fn emit_node_type_disj_rc(n: Rc<Node>, target: RenderTarget, rc_types: Rc<HashMap<String, bool>>) -> String {
    if n.name.clone() != "" {
    if emit_map_has(rc_types.clone(), &n.name) {
    v2_rt::concat(v2_rt::concat("Rc<".to_string(), n.name.clone()), ">".to_string())
} else {
    n.name.clone()
}
} else {
    let is_rust = match target {
    RenderTarget::Rust => {
        true
    }
    _ => {
        false
    }
};
    if is_rust.clone() {
    "compile_error!(\"anonymous coproduct reached Node emitter\")".to_string()
} else {
    "__EMIT_BUG_ANONYMOUS_DISJ__".to_string()
}
}
}

pub fn is_type_alias_return_node(n: Rc<Node>) -> bool {
    n.name.clone() != "Unit"
}

pub fn is_service_item(item: Rc<Node>) -> bool {
    (item.transport.clone().is_some()) && (({
    let __len_0 = item.children.clone().len();
    __len_0 as i64
}) > 0_i64)
}

pub fn has_service_items(typed: Rc<ResolvedGraph>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in typed.modules.iter().cloned() {
        {
let __cond = {
    let mut __any_2 = false;
    for __elem_3 in __elem_1.items.iter().cloned() {
        if is_service_item(__elem_3.clone()) {
    __any_2 = true;
    break;
};
    }
    __any_2
};
if __cond {
    __any_0 = true;
    break;
}
};
    }
    __any_0
}
}

pub fn service_fallback_transport(item: Rc<Node>) -> Rc<Node> {
    if item.transport.clone().is_none() {
    local_transport_node(item.span.clone())
} else {
    item.transport.clone().unwrap()
}
}

pub fn effective_operation_transport(op_node: Rc<Node>, fallback: Rc<Node>) -> Rc<Node> {
    match op_node.transport.as_ref().map(|__rc| __rc.as_ref()) {
    Some(op_transport) => {
        let op_transport = Rc::new(op_transport.clone());
        op_transport.clone()
    }
    None => {
        fallback.clone()
    }
}
}

pub fn service_uses_transport(fallback_transport: Rc<Node>, op_children: Rc<Vec<Rc<Node>>>, kind: Rc<TransportKind>) -> bool {
    let from_fallback = is_transport_kind(fallback_transport.clone(), kind.clone());
    let from_ops = {
    let mut __any_0 = false;
    for __elem_1 in op_children.iter().cloned() {
        if if __elem_1.transport.clone().is_some() {
    is_transport_kind(__elem_1.transport.clone().unwrap(), kind.clone())
} else {
    false
} {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    from_fallback || from_ops.clone()
}

pub fn service_has_rest_auth(fallback_transport: Rc<Node>, op_children: Rc<Vec<Rc<Node>>>) -> bool {
    let from_fallback = if is_transport_kind(fallback_transport.clone(), Rc::new(TransportKind::RestTransport)) {
    transport_has_auth(fallback_transport.clone())
} else {
    false
};
    let from_ops = {
    let mut __any_0 = false;
    for __elem_1 in op_children.iter().cloned() {
        if if __elem_1.transport.clone().is_some() {
    let t = __elem_1.transport.clone().unwrap();
    if is_transport_kind(t.clone(), Rc::new(TransportKind::RestTransport)) {
    transport_has_auth(t.clone())
} else {
    false
}
} else {
    false
} {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    from_fallback.clone() || from_ops.clone()
}

pub fn extract_modifier_names(properties: Rc<Vec<Rc<FieldInit>>>) -> Rc<Vec<String>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in properties.iter().cloned() {
        __flat_mapped_0.extend((match field_init_operation_modifier(__elem_1.clone()) {
    Some(modifier) => {
        Rc::new(vec!(operation_modifier_name(modifier.clone())))
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn classify_typed_item(item: Rc<Node>) -> TypedItemKind {
    let item_has_structure = node_has_structure(item.clone());
    let kind = if item_has_structure.clone() && (item.transport.clone().is_none()) {
    TypedItemKind::TypedItemTypeDef
} else {
    if (((((item_has_structure.clone() == false) && (item.body.clone().is_none())) && (({
    let __len_7 = item.params.clone().len();
    __len_7 as i64
}) == 0_i64)) && (item.transport.clone().is_none())) && (({
    let __len_8 = item.children.clone().len();
    __len_8 as i64
}) == 0_i64)) && is_type_alias_return_node(rt_type(item.clone())) {
    TypedItemKind::TypedItemTypeAlias
} else {
    if (item.body.clone().is_some()) && (item.type_annotation.clone().is_none()) {
    TypedItemKind::TypedItemFunction
} else {
    if (item.body.clone().is_some()) && (item.type_annotation.clone().is_some()) {
    TypedItemKind::TypedItemDataDef
} else {
    if (item.transport.clone().is_some()) && (({
    let __len_6 = item.children.clone().len();
    __len_6 as i64
}) > 0_i64) {
    TypedItemKind::TypedItemServiceDef
} else {
    if ((item.transport.clone().is_none()) && (({
    let __len_3 = item.children.clone().len();
    __len_3 as i64
}) > 0_i64)) || ((((item.transport.clone().is_none()) && (({
    let __len_4 = item.children.clone().len();
    __len_4 as i64
}) == 0_i64)) && (({
    let __len_5 = item.properties.clone().len();
    __len_5 as i64
}) > 0_i64)) && (item.body.clone().is_none())) {
    TypedItemKind::TypedItemResourceDef
} else {
    if (((({
    let __len_1 = item.params.clone().len();
    __len_1 as i64
}) > 0_i64) && (item.body.clone().is_none())) && (item.transport.clone().is_none())) && (({
    let __len_2 = item.children.clone().len();
    __len_2 as i64
}) == 0_i64) {
    TypedItemKind::TypedItemTypeDecl
} else {
    if ({
    let __len_0 = item.params.clone().len();
    __len_0 as i64
}) > 0_i64 {
    TypedItemKind::TypedItemExternFunc
} else {
    TypedItemKind::TypedItemUnhandled
}
}
}
}
}
}
}
};
    kind.clone()
}

pub fn classify_type_structure(n: Rc<Node>) -> TypeStructureKind {
    if node_is_product(n.clone()) {
    TypeStructureKind::TypeConj
} else {
    if node_is_coproduct(n.clone()) {
    TypeStructureKind::TypeDisj
} else {
    TypeStructureKind::TypeLeaf
}
}
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum ExprCategory {
    #[default]
    ExprCatLeaf,
    ExprCatCompound,
    ExprCatControlFlow,
    ExprCatBinding,
    ExprCatService,
    ExprCatNone,
}

pub fn classify_expr(texpr: Rc<Node>) -> ExprCategory {
    match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: _, .. } => {
        ExprCategory::ExprCatLeaf
    }
    ExprData::ExprError { kind: _, message: _, .. } => {
        ExprCategory::ExprCatLeaf
    }
    ExprData::ExprVar { name: _, binding_kind: _, .. } => {
        ExprCategory::ExprCatLeaf
    }
    ExprData::ExprFieldAccess { base: _, field: _, summary: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprCall { func: _, args: _, call_semantics: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprMethodCall { receiver: _, method: _, args: _, method_semantics: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprRecordLit { type_name: _, fields: _, parent_enum: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprListLit { elements: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprBinOp { op: _, left: _, right: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprUnaryOp { op: _, operand: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprLambda { params: _, body: _, semantics: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprStringInterp { parts: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprCast { expr: _, target: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprIndex { base: _, index: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprSlice { base: _, start: _, end: _, .. } => {
        ExprCategory::ExprCatCompound
    }
    ExprData::ExprMatch { scrutinee: _, arms: _, .. } => {
        ExprCategory::ExprCatControlFlow
    }
    ExprData::ExprIf { condition: _, then_branch: _, else_branch: _, .. } => {
        ExprCategory::ExprCatControlFlow
    }
    ExprData::ExprForEach { variable: _, collection: _, body: _, .. } => {
        ExprCategory::ExprCatControlFlow
    }
    ExprData::ExprBlock { stmts: _, .. } => {
        ExprCategory::ExprCatControlFlow
    }
    ExprData::ExprLet { name: _, value: _, body: _, .. } => {
        ExprCategory::ExprCatBinding
    }
    ExprData::ExprReturn { value: _, .. } => {
        ExprCategory::ExprCatControlFlow
    }
    ExprData::NoExprData => {
        ExprCategory::ExprCatNone
    }
    _ => {
        ExprCategory::ExprCatNone
    }
}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncBodyShape {
    FuncBodyLet { name: String, value: Rc<Node>, rest: Option<Rc<Node>> },
    FuncBodyBlock { stmts: Rc<Vec<Rc<Node>>> },
    FuncBodyExpr { expr: Rc<Node> },
}

impl Default for FuncBodyShape {
    fn default() -> Self {
        FuncBodyShape::FuncBodyLet { name: Default::default(), value: Default::default(), rest: Default::default() }
    }
}

pub fn classify_func_body(body: Rc<Node>) -> Rc<FuncBodyShape> {
    match body.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: rest, .. } => {
        Rc::new(FuncBodyShape::FuncBodyLet { name: n.clone(), value: v.clone(), rest: rest.clone() })
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        Rc::new(FuncBodyShape::FuncBodyBlock { stmts: ss.clone() })
    }
    _ => {
        Rc::new(FuncBodyShape::FuncBodyExpr { expr: body.clone() })
    }
}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TcoExprShape {
    TcoCall { func: String, args: Rc<Vec<Rc<NamedArg>>> },
    TcoIf { condition: Rc<Node>, then_branch: Rc<Node>, else_branch: Option<Rc<Node>> },
    TcoMatch { scrutinee: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>> },
    TcoLet { name: String, value: Rc<Node>, body: Option<Rc<Node>> },
    TcoBlock { stmts: Rc<Vec<Rc<Node>>> },
    TcoOther { expr: Rc<Node> },
}

impl Default for TcoExprShape {
    fn default() -> Self {
        TcoExprShape::TcoCall { func: Default::default(), args: Default::default() }
    }
}

pub fn classify_tco_expr(texpr: Rc<Node>) -> Rc<TcoExprShape> {
    match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: a, call_semantics: _, .. } => {
        Rc::new(TcoExprShape::TcoCall { func: f.clone(), args: a.clone() })
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        Rc::new(TcoExprShape::TcoIf { condition: c.clone(), then_branch: t.clone(), else_branch: e.clone() })
    }
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        Rc::new(TcoExprShape::TcoMatch { scrutinee: s.clone(), arms: arm_list.clone() })
    }
    ExprData::ExprLet { name: n, value: v, body: bd, .. } => {
        Rc::new(TcoExprShape::TcoLet { name: n.clone(), value: v.clone(), body: bd.clone() })
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        Rc::new(TcoExprShape::TcoBlock { stmts: ss.clone() })
    }
    _ => {
        Rc::new(TcoExprShape::TcoOther { expr: texpr.clone() })
    }
}
}

pub fn block_stmts_init(stmts: Rc<Vec<Rc<Node>>>) -> Rc<Vec<Rc<Node>>> {
    if ({
    let __len_1 = stmts.clone().len();
    __len_1 as i64
}) <= 1_i64 {
    Rc::new(Vec::new())
} else {
    { let __t = stmts.clone(); let __n = (({
    let __len_0 = stmts.clone().len();
    __len_0 as i64
}) - 1_i64) as usize; Rc::new(__t[..__n.min(__t.len())].to_vec()) }
}
}

pub fn is_tco_eligible(name: &str, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> bool {
    match lookup_item(registry.clone(), &name) {
    Some(info) => {
        info.is_self_recursive.clone() && (info.has_non_tail_self_call.clone() == false)
    }
    None => {
        expr_has_self_call(body.clone(), &name) && (expr_has_non_tail_self_call(body.clone(), &name, true) == false)
    }
}
}

pub fn is_self_recursive(name: &str, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> bool {
    match lookup_item(registry.clone(), &name) {
    Some(info) => {
        info.is_self_recursive.clone()
    }
    None => {
        expr_has_self_call(body.clone(), &name)
    }
}
}

pub fn tco_reassign_core(ordered_args: Rc<Vec<String>>, param_names: Rc<Vec<String>>, temp_var_prefix: &str, temp_decl_prefix: &str, temp_assign_op: &str, stmt_terminator: &str, continue_str: &str, line_prefix: &str) -> Rc<Vec<String>> {
    let temp_lets = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in ordered_args.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __mapped_3.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(line_prefix.to_string(), temp_decl_prefix.to_string()), temp_var_prefix.to_string()), v2_rt::to_string(__elem_4.0.clone())), temp_assign_op.to_string()), __elem_4.1.clone()), stmt_terminator.to_string()));
    }
    Rc::new(__mapped_3)
};
    let assigns = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in ({
    let mut __enumerated_5 = Vec::new();
    for (__idx_6, __elem_7) in param_names.clone().iter().enumerate() {
        __enumerated_5.push((__idx_6 as i64, __elem_7.clone()));
    }
    Rc::new(__enumerated_5)
}).iter().cloned() {
        __mapped_8.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(line_prefix.to_string(), __elem_9.1.clone()), " = ".to_string()), temp_var_prefix.to_string()), v2_rt::to_string(__elem_9.0.clone())), stmt_terminator.to_string()));
    }
    Rc::new(__mapped_8)
};
    v2_rt::concat(v2_rt::concat(temp_lets.clone(), assigns.clone()), Rc::new(vec!(v2_rt::concat(line_prefix.to_string(), continue_str.to_string()))))
}

pub fn is_tco_candidate(texpr: Rc<Node>, func_name: &str) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: _, call_semantics: _, .. } => {
        f.clone() == func_name
    }
    ExprData::ExprIf { condition: _, then_branch: t, else_branch: e, .. } => {
        is_tco_candidate(t.clone(), &func_name) || (match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        is_tco_candidate(eb.clone(), &func_name)
    }
    None => {
        false
    }
})
    }
    ExprData::ExprMatch { scrutinee: _, arms: arm_list, .. } => {
        {
    let mut __any_0 = false;
    for __elem_1 in arm_list.iter().cloned() {
        if is_tco_candidate(__elem_1.body.clone(), &func_name) {
    __any_0 = true;
    break;
};
    }
    __any_0
}
    }
    ExprData::ExprLet { name: _, value: _, body: bd, .. } => {
        match bd.as_ref().map(|__rc| __rc.as_ref()) {
    Some(b) => {
        let b = Rc::new(b.clone());
        is_tco_candidate(b.clone(), &func_name)
    }
    None => {
        false
    }
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        if ({
    let __len_4 = ss.clone().len();
    __len_4 as i64
}) == 0_i64 {
    false
} else {
    {
    let mut __any_2 = false;
    for __elem_3 in ss.iter().cloned() {
        if is_tco_candidate(__elem_3.clone(), &func_name) {
    __any_2 = true;
    break;
};
    }
    __any_2
}
}
    }
    _ => {
        false
    }
}
    })
}

pub fn emit_ident(name: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        {
    let snake = to_snake(&name);
    {
let __cond = {
    let mut __any_0 = false;
    for __elem_1 in Rc::new(RUST_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        if __elem_1.clone() == snake.clone() {
    __any_0 = true;
    break;
};
    }
    __any_0
};
if __cond {
    v2_rt::concat(RUST_RESERVED_ESCAPE_PREFIX.to_string(), snake.clone())
} else {
    snake.clone()
}
}
}
    }
    RenderTarget::Go => {
        {
    let parts = {
    let mut __split_parts_2 = Vec::new();
    for __part_3 in name.to_string().split("_".to_string().as_str()) {
        __split_parts_2.push(__part_3.to_string());
    }
    __split_parts_2
};
    if ({
    let __len_23 = parts.clone().len();
    __len_23 as i64
}) == 0_i64 {
    name.to_string()
} else {
    let first_part = match parts.clone().first().cloned() {
    Some(p) => {
        {
    let mut __joined_8 = String::new();
    let mut __first_10 = true;
    for __elem_9 in ({
    let mut __mapped_6 = Vec::new();
    for __elem_7 in ({
    let mut __chars_4 = Vec::new();
    for __ch_5 in p.clone().chars() {
        __chars_4.push(__ch_5.to_string());
    }
    Rc::new(__chars_4)
}).iter().cloned() {
        __mapped_6.push(to_lower_char(&__elem_7));
    }
    Rc::new(__mapped_6)
}).iter().cloned() {
        if !__first_10 {
    __joined_8.push_str(&"".to_string());
};
        __first_10 = false;
        __joined_8.push_str(&__elem_9);
    }
    __joined_8
}
    }
    None => {
        "".to_string()
    }
};
    let rest_parts = {
    let mut __mapped_16 = Vec::new();
    for __elem_17 in ({
    let mut __filtered_14 = Vec::new();
    for __elem_15 in ({
    let mut __enumerated_11 = Vec::new();
    for (__idx_12, __elem_13) in parts.clone().iter().enumerate() {
        __enumerated_11.push((__idx_12 as i64, __elem_13.clone()));
    }
    Rc::new(__enumerated_11)
}).iter().cloned() {
        if __elem_15.0.clone() > 0_i64 {
    __filtered_14.push(__elem_15);
};
    }
    Rc::new(__filtered_14)
}).iter().cloned() {
        __mapped_16.push(capitalize_first(&__elem_17.1));
    }
    Rc::new(__mapped_16)
};
    let camel = v2_rt::concat(first_part.clone(), {
    let mut __joined_18 = String::new();
    let mut __first_20 = true;
    for __elem_19 in rest_parts.iter().cloned() {
        if !__first_20 {
    __joined_18.push_str(&"".to_string());
};
        __first_20 = false;
        __joined_18.push_str(&__elem_19);
    }
    __joined_18
});
    {
let __cond = {
    let mut __any_21 = false;
    for __elem_22 in Rc::new(GO_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        if __elem_22.clone() == camel.clone() {
    __any_21 = true;
    break;
};
    }
    __any_21
};
if __cond {
    v2_rt::concat(camel.clone(), GO_RESERVED_ESCAPE_SUFFIX.to_string())
} else {
    camel.clone()
}
}
}
}
    }
    RenderTarget::Python => {
        {
    let snake = to_snake(&name);
    {
let __cond = {
    let mut __any_24 = false;
    for __elem_25 in Rc::new(PYTHON_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        if __elem_25.clone() == snake.clone() {
    __any_24 = true;
    break;
};
    }
    __any_24
};
if __cond {
    v2_rt::concat(snake.clone(), PYTHON_RESERVED_ESCAPE_SUFFIX.to_string())
} else {
    snake.clone()
}
}
}
    }
    RenderTarget::Dag => {
        name.to_string()
    }
}
}

pub fn emit_let_binding(name: &str, value: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let ".to_string(), emit_ident(&name, RenderTarget::Rust)), " = ".to_string()), value.to_string()), ";".to_string())
    }
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat(emit_ident(&name, RenderTarget::Go), " := ".to_string()), value.to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat(emit_ident(&name, RenderTarget::Python), " = ".to_string()), value.to_string())
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("let ".to_string(), name.to_string()), " = ".to_string()), value.to_string())
    }
}
}

pub fn emit_return(value: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat("return ".to_string(), value.to_string()), ";".to_string())
    }
    RenderTarget::Go => {
        v2_rt::concat("return ".to_string(), value.to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat("return ".to_string(), value.to_string())
    }
    RenderTarget::Dag => {
        v2_rt::concat("return ".to_string(), value.to_string())
    }
}
}

pub fn emit_unary_op(op: UnaryOpKind, operand_str: &str, target: RenderTarget) -> String {
    match op {
    UnaryOpKind::Not => {
        v2_rt::concat(emit_keyword("not", target), operand_str.to_string())
    }
    UnaryOpKind::Neg => {
        v2_rt::concat("-".to_string(), operand_str.to_string())
    }
}
}

pub fn emit_lambda(params_str: &str, body_str: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("|".to_string(), params_str.to_string()), "| ".to_string()), body_str.to_string())
    }
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func(".to_string(), params_str.to_string()), ") interface{} { return ".to_string()), body_str.to_string()), " }".to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("lambda ".to_string(), params_str.to_string()), ": ".to_string()), body_str.to_string())
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), params_str.to_string()), ") => ".to_string()), body_str.to_string())
    }
}
}

pub fn emit_error_expr(message: &str, target: RenderTarget) -> String {
    let msg = emit_string_literal(&message, "");
    match target {
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat("compile_error!(".to_string(), msg), ")".to_string())
    }
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat("panic(".to_string(), msg), ")".to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat("raise RuntimeError(".to_string(), msg), ")".to_string())
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat("error(".to_string(), msg), ")".to_string())
    }
}
}

pub fn emit_lambda_params(param_names: Rc<Vec<String>>, target: RenderTarget) -> String {
    let param_strs = match target.clone() {
    RenderTarget::Go => {
        {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in param_names.iter().cloned() {
        __mapped_0.push(v2_rt::concat(emit_ident(&__elem_1, RenderTarget::Go), " interface{}".to_string()));
    }
    Rc::new(__mapped_0)
}
    }
    _ => {
        {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in param_names.iter().cloned() {
        __mapped_2.push(emit_ident(&__elem_3, target.clone()));
    }
    Rc::new(__mapped_2)
}
    }
};
    {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in param_strs.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
}
}

pub fn emit_list_lit_expr(element_strs: Rc<Vec<String>>, target: RenderTarget) -> String {
    if ({
    let __len_3 = element_strs.clone().len();
    __len_3 as i64
}) == 0_i64 {
    match target {
    RenderTarget::Rust => {
        "vec![]".to_string()
    }
    RenderTarget::Python => {
        "[]".to_string()
    }
    RenderTarget::Go => {
        "[]interface{}{}".to_string()
    }
    RenderTarget::Dag => {
        "[]".to_string()
    }
}
} else {
    let els_str = {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in element_strs.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&", ".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
};
    match target {
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat("vec![".to_string(), els_str.clone()), "]".to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat("[".to_string(), els_str.clone()), "]".to_string())
    }
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat("[]interface{}{".to_string(), els_str.clone()), "}".to_string())
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat("[".to_string(), els_str.clone()), "]".to_string())
    }
}
}
}

pub fn emit_null_coalesce(l_str: &str, r_str: &str, target: RenderTarget) -> String {
    match target {
    RenderTarget::Rust => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(l_str.to_string(), ".unwrap_or_else(|| ".to_string()), r_str.to_string()), ")".to_string())
    }
    RenderTarget::Python => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), l_str.to_string()), " if ".to_string()), l_str.to_string()), " is not None else ".to_string()), r_str.to_string()), ")".to_string())
    }
    RenderTarget::Go => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func() interface{} { if ".to_string(), l_str.to_string()), " != nil { return ".to_string()), l_str.to_string()), " }; return ".to_string()), r_str.to_string()), " }()".to_string())
    }
    RenderTarget::Dag => {
        v2_rt::concat(v2_rt::concat(l_str.to_string(), " ?? ".to_string()), r_str.to_string())
    }
}
}

pub fn emit_shared_expr(texpr: Rc<Node>, target: RenderTarget, recurse: impl Fn(Rc<Node>, Rc<InferScope>, i64) -> String, scope: Rc<InferScope>, depth: i64) -> Option<String> {
    match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: v, .. } => {
        Some(emit_literal(v.clone(), target.clone()))
    }
    ExprData::ExprError { kind: _, message, .. } => {
        Some(emit_error_expr(&message, target.clone()))
    }
    ExprData::NoExprData => {
        Some("".to_string())
    }
    ExprData::ExprUnaryOp { op: o, operand: e, .. } => {
        Some(emit_unary_op(o.clone(), &recurse(e.clone(), scope.clone(), depth.clone()), target.clone()))
    }
    ExprData::ExprLambda { params: ps, body: bd, semantics: _, .. } => {
        Some(emit_lambda(&emit_lambda_params(ps.clone(), target.clone()), &recurse(bd.clone(), scope.clone(), depth.clone()), target.clone()))
    }
    ExprData::ExprListLit { elements: els, .. } => {
        Some(emit_list_lit_expr({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in els.iter().cloned() {
        __mapped_2.push(recurse(__elem_3.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_2)
}, target.clone()))
    }
    ExprData::ExprReturn { value: v, .. } => {
        Some(emit_return(&recurse(v.clone(), scope.clone(), depth.clone()), target.clone()))
    }
    _ => {
        None
    }
}
}

pub fn bridge_method_base_name(method: RuntimeBridgeMethod) -> String {
    match method {
    RuntimeBridgeMethod::BridgeGet => {
        "get".to_string()
    }
    RuntimeBridgeMethod::BridgeWith => {
        "with".to_string()
    }
    RuntimeBridgeMethod::BridgeListPush => {
        "list_push".to_string()
    }
    RuntimeBridgeMethod::BridgeMapInsert => {
        "map_insert".to_string()
    }
    RuntimeBridgeMethod::BridgeMapMerge => {
        "map_merge".to_string()
    }
    RuntimeBridgeMethod::BridgeMapGet => {
        "map_get".to_string()
    }
    RuntimeBridgeMethod::BridgeMapHas => {
        "map_has".to_string()
    }
    RuntimeBridgeMethod::BridgeEmitMapHas => {
        "emit_map_has".to_string()
    }
    RuntimeBridgeMethod::BridgeMapValues => {
        "map_values".to_string()
    }
    RuntimeBridgeMethod::BridgeMapKeys => {
        "map_keys".to_string()
    }
    RuntimeBridgeMethod::BridgeMapContainsKey => {
        "map_contains_key".to_string()
    }
    RuntimeBridgeMethod::BridgeCharAt => {
        "char_at".to_string()
    }
    RuntimeBridgeMethod::BridgeStringAt => {
        "string_at".to_string()
    }
    RuntimeBridgeMethod::BridgeStringLength => {
        "string_length".to_string()
    }
    RuntimeBridgeMethod::BridgeLength => {
        "length".to_string()
    }
    RuntimeBridgeMethod::BridgeStartsWith => {
        "starts_with".to_string()
    }
    RuntimeBridgeMethod::BridgeEndsWith => {
        "ends_with".to_string()
    }
    RuntimeBridgeMethod::BridgeToString => {
        "to_string".to_string()
    }
    RuntimeBridgeMethod::BridgeTrim => {
        "trim".to_string()
    }
    RuntimeBridgeMethod::BridgeToLower => {
        "to_lower".to_string()
    }
    RuntimeBridgeMethod::BridgeToUpper => {
        "to_upper".to_string()
    }
    RuntimeBridgeMethod::BridgeReplace => {
        "replace".to_string()
    }
    RuntimeBridgeMethod::BridgeSubstring => {
        "substring".to_string()
    }
    RuntimeBridgeMethod::BridgeToInt => {
        "to_int".to_string()
    }
    RuntimeBridgeMethod::BridgeEmptyMap => {
        "empty_map".to_string()
    }
    RuntimeBridgeMethod::BridgeContains => {
        "contains".to_string()
    }
    RuntimeBridgeMethod::BridgeReverse => {
        "reverse".to_string()
    }
    RuntimeBridgeMethod::BridgeLookup => {
        "lookup".to_string()
    }
}
}

