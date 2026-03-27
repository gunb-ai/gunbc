use crate::v2_core::*;
use crate::artifact::*;
use crate::languages::*;
use crate::infer_env::*;
use crate::infer_types::*;
use crate::infer_sigs::*;
use crate::infer_items::*;
use crate::infer_service::*;
use crate::infer::*;
use crate::emit::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn emit_py_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> Rc<BlockEmitState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_text = text;
        let mut __tco_p_scope = scope;
        let mut __tco_p_registry = registry;
        let mut __tco_p_depth = depth;
        loop {
            let remaining = __tco_p_remaining;
            let text = __tco_p_text;
            let scope = __tco_p_scope;
            let registry = __tco_p_registry;
            let depth = __tco_p_depth;
            match remaining.clone().first().cloned() {
    None => {
        break Rc::new(BlockEmitState { text: text.clone(), scope: scope.clone() });
    }
    Some(stmt) => {
        {
    let line = emit_py_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth.clone());
    let next_scope = scope_after_expr(stmt.clone(), scope.clone());
     {
        let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
        let __tco_1 = {
    let __rc_1 = text;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(line);
    Rc::new(__appended_0)
};
        let __tco_2 = next_scope.clone();
        let __tco_3 = registry.clone();
        let __tco_4 = depth.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_text = __tco_1;
        __tco_p_scope = __tco_2;
        __tco_p_registry = __tco_3;
        __tco_p_depth = __tco_4;
        continue;
    }

};
    }
};
        }
    })
}

pub fn emit_py_init_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> Rc<BlockEmitState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_text = text;
        let mut __tco_p_scope = scope;
        let mut __tco_p_registry = registry;
        let mut __tco_p_depth = depth;
        loop {
            let remaining = __tco_p_remaining;
            let text = __tco_p_text;
            let scope = __tco_p_scope;
            let registry = __tco_p_registry;
            let depth = __tco_p_depth;
            match remaining.clone().first().cloned() {
    None => {
        break Rc::new(BlockEmitState { text: text.clone(), scope: scope.clone() });
    }
    Some(stmt) => {
        {
    let rest = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
    match rest.clone().first().cloned() {
    None => {
        break Rc::new(BlockEmitState { text: text.clone(), scope: scope.clone() });
    }
    Some(_) => {
        {
    let line = emit_py_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth.clone());
    let next_scope = scope_after_expr(stmt.clone(), scope.clone());
     {
        let __tco_0 = rest.clone();
        let __tco_1 = {
    let __rc_1 = text;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(line);
    Rc::new(__appended_0)
};
        let __tco_2 = next_scope.clone();
        let __tco_3 = registry.clone();
        let __tco_4 = depth.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_text = __tco_1;
        __tco_p_scope = __tco_2;
        __tco_p_registry = __tco_3;
        __tco_p_depth = __tco_4;
        continue;
    }

};
    }
};
};
    }
};
        }
    })
}

pub fn emit_python(typed: Rc<ResolvedGraph>) -> Rc<EmitResult> {
    let registry = typed.item_registry.clone();
    let test_projections = extract_test_projections(typed.clone());
    let module_files = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __mapped_0.push(emit_py_module(__elem_1.clone(), registry.clone()));
    }
    Rc::new(__mapped_0)
};
    let test_files = {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in ({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in typed.modules.iter().cloned() {
        __mapped_2.push(emit_py_test_file(&__elem_3.module.name, {
    let mut __filtered_4 = Vec::new();
    for __elem_5 in test_projections.iter().cloned() {
        if __elem_5.module_name.clone() == __elem_3.module.name.clone() {
    __filtered_4.push(__elem_5);
};
    }
    Rc::new(__filtered_4)
}));
    }
    Rc::new(__mapped_2)
}).iter().cloned() {
        if __elem_7.content.clone() != "" {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
};
    let init_file = emit_init_py(typed.modules.clone());
    let requirements = emit_requirements_txt(has_service_items(typed.clone()));
    let files = v2_rt::concat(v2_rt::concat(Rc::new(vec!(requirements.clone(), init_file.clone())), module_files.clone()), test_files.clone());
    Rc::new(EmitResult { files: files.clone(), diagnostics: Rc::new(Vec::new()) })
}

pub fn py_source_extension() -> String {
    scaffold_for_target(RenderTarget::Python).source_file_extension.clone()
}

pub fn py_module_init_path() -> String {
    match scaffold_for_target(RenderTarget::Python).module_init_file.clone() {
    Some(path) => {
        path
    }
    None => {
        "__init__.py".to_string()
    }
}
}

pub fn py_derive_attribute() -> String {
    match serialization_for_target(RenderTarget::Python).derive_attribute.clone() {
    Some(attr) => {
        attr
    }
    None => {
        "@dataclass".to_string()
    }
}
}

pub fn py_default_value() -> String {
    match serialization_for_target(RenderTarget::Python).default_value.clone() {
    Some(value) => {
        value
    }
    None => {
        "None".to_string()
    }
}
}

pub fn py_test_file_path(module_name: &str) -> String {
    let conventions = test_conventions_for_target(RenderTarget::Python);
    let file_dir = match conventions.file_dir.clone() {
    Some(dir) => {
        dir
    }
    None => {
        "".to_string()
    }
};
    let filename = module_to_filename(&module_name);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(file_dir.clone(), conventions.file_prefix.clone()), filename), conventions.file_suffix.clone()), py_source_extension())
}

pub fn py_test_name(projection: Rc<TestProjection>) -> String {
    test_function_name(projection.clone(), RenderTarget::Python)
}

pub fn emit_init_py(modules: Rc<Vec<Rc<TypedModule>>>) -> Rc<TextFile> {
    let import_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in modules.iter().cloned() {
        __mapped_0.push({
    let mod_name = module_to_filename(&__elem_1.module.name);
    v2_rt::concat("from . import ".to_string(), mod_name.clone())
});
    }
    Rc::new(__mapped_0)
};
    let content = v2_rt::concat(v2_rt::concat("# Generated by v2 compiler -- do not edit.\n\n".to_string(), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in import_lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), "\n".to_string());
    Rc::new(TextFile { path: py_module_init_path(), content: content.clone() })
}

pub fn python_test_signature_comment(projection: Rc<TestProjection>) -> String {
    let params_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projection.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(__elem_1.name.clone(), ": ".to_string()), emit_node_type(__elem_1.type_expr.clone(), RenderTarget::Python)));
    }
    Rc::new(__mapped_0)
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("# Signature: ".to_string(), sanitize_service_name(&projection.service_name)), ".".to_string()), projection.operation_name.clone()), "(".to_string()), params_str.clone()), ") -> ".to_string()), emit_node_type(projection.inferred.clone(), RenderTarget::Python))
}

pub fn emit_py_test_file(module_name: &str, projections: Rc<Vec<Rc<TestProjection>>>) -> Rc<TextFile> {
    if ({
    let __len_5 = projections.clone().len();
    __len_5 as i64
}) == 0_i64 {
    Rc::new(TextFile { path: "".to_string(), content: "".to_string() })
} else {
    let tests_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projections.iter().cloned() {
        __mapped_0.push(emit_py_operation_test(__elem_1.clone(), 0_i64));
    }
    Rc::new(__mapped_0)
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("# Generated tests -- do not edit.\n".to_string(), "# Source module: ".to_string()), module_name.to_string()), "\n\n".to_string()), tests_str.clone()), "\n".to_string());
    Rc::new(TextFile { path: py_test_file_path(&module_name), content: content.clone() })
}
}

pub fn emit_py_operation_test(projection: Rc<TestProjection>, depth: i64) -> String {
    let test_name = py_test_name(projection.clone());
    let mock_setup = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projection.mock_field_inits.iter().cloned() {
        __mapped_0.push(emit_py_mock_prop_setup(__elem_1.clone(), depth.clone() + 1_i64));
    }
    Rc::new(__mapped_0)
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("def ".to_string(), test_name), "() -> None:\n".to_string()), make_indent(depth.clone() + 1_i64)), python_test_signature_comment(projection.clone())), "\n".to_string()), make_indent(depth.clone() + 1_i64)), mock_setup.clone()), "\n".to_string()), make_indent(depth.clone() + 1_i64)), "assert True\n".to_string())
}

pub fn emit_py_mock_prop_setup(mock_prop: Rc<FieldInit>, depth: i64) -> String {
    v2_rt::concat(v2_rt::concat(emit_ident(&mock_prop.name, RenderTarget::Python), " = ".to_string()), emit_simple_expr(mock_prop.value.clone(), RenderTarget::Python))
}

pub fn emit_py_module(typed_module: Rc<TypedModule>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> Rc<TextFile> {
    let m = typed_module.module.clone();
    let scope = module_emit_scope(typed_module.clone());
    let prelude = emit_py_prelude(typed_module.clone());
    let imports_str = emit_py_imports(module_imports(m.clone()));
    let imports_section = if imports_str.clone() == "" {
    "".to_string()
} else {
    v2_rt::concat("\n".to_string(), imports_str.clone())
};
    let items_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in typed_module.items.iter().cloned() {
        __mapped_0.push(emit_py_typed_item(__elem_1.clone(), registry.clone(), scope.clone()));
    }
    Rc::new(__mapped_0)
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let filename = module_to_filename(&m.name);
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("# Generated by v2 compiler -- do not edit.\n".to_string(), "# Source module: ".to_string()), m.name.clone()), "\n\n".to_string()), prelude), imports_section.clone()), "\n\n\n".to_string()), items_str.clone()), "\n".to_string());
    Rc::new(TextFile { path: v2_rt::concat(filename, py_source_extension()), content: content.clone() })
}

pub fn emit_py_imports(imports: Rc<Vec<Rc<Node>>>) -> String {
    if ({
    let __len_11 = imports.clone().len();
    __len_11 as i64
}) == 0_i64 {
    "".to_string()
} else {
    let import_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in imports.iter().cloned() {
        __mapped_0.push({
    let mod_name = module_to_filename(&__elem_1.name);
    if import_is_all(__elem_1.clone()) {
    v2_rt::concat(v2_rt::concat("from ".to_string(), mod_name.clone()), " import *".to_string())
} else {
    let specific_names = import_specific_names(__elem_1.clone());
    if ({
    let __len_5 = specific_names.clone().len();
    __len_5 as i64
}) == 0_i64 {
    "".to_string()
} else {
    let names_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in specific_names.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat("from ".to_string(), mod_name.clone()), " import ".to_string()), names_str.clone())
}
}
});
    }
    Rc::new(__mapped_0)
};
    {
    let mut __joined_8 = String::new();
    let mut __first_10 = true;
    for __elem_9 in ({
    let mut __filtered_6 = Vec::new();
    for __elem_7 in import_lines.iter().cloned() {
        if __elem_7.clone() != "" {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
}).iter().cloned() {
        if !__first_10 {
    __joined_8.push_str(&"\n".to_string());
};
        __first_10 = false;
        __joined_8.push_str(&__elem_9);
    }
    __joined_8
}
}
}

pub fn emit_py_prelude(typed_module: Rc<TypedModule>) -> String {
    let base_imports = v2_rt::concat(v2_rt::concat(v2_rt::concat("from __future__ import annotations\n".to_string(), "from dataclasses import dataclass, field\n".to_string()), "from enum import Enum, auto\n".to_string()), "from typing import Optional, Union\n".to_string());
    let has_services = {
    let mut __any_0 = false;
    for __elem_1 in typed_module.items.iter().cloned() {
        if is_service_item(__elem_1.clone()) {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    let async_imports = if has_services.clone() {
    "import aiohttp\n".to_string()
} else {
    "".to_string()
};
    v2_rt::concat(base_imports.clone(), async_imports.clone())
}

pub fn emit_py_typed_item(item: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let kind = classify_typed_item(item.clone());
    if kind.clone() == TypedItemKind::TypedItemTypeDef {
    emit_py_type_def_from_connective(item.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemTypeAlias {
    v2_rt::concat(v2_rt::concat(item.name.clone(), " = ".to_string()), emit_node_type(rt_type(item.clone()), RenderTarget::Python))
} else {
    if kind.clone() == TypedItemKind::TypedItemTypeDecl {
    "".to_string()
} else {
    if kind.clone() == TypedItemKind::TypedItemFunction {
    if ({
    let __len_0 = item.uses.clone().len();
    __len_0 as i64
}) > 0_i64 {
    emit_py_func_def(&item.name, item.params.clone(), rt_type(item.clone()), item.uses.clone(), item.body.clone().unwrap(), registry.clone(), scope.clone())
} else {
    emit_py_fn_def(&item.name, item.params.clone(), rt_type(item.clone()), item.body.clone().unwrap(), registry.clone(), scope.clone())
}
} else {
    if kind.clone() == TypedItemKind::TypedItemDataDef {
    emit_py_data_def(&item.name, item.type_annotation.clone().unwrap(), item.body.clone().unwrap(), registry.clone(), scope.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemServiceDef {
    emit_py_service_def(item.clone(), registry.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemResourceDef {
    emit_py_resource_def(item.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemExternFunc {
    let params_str = emit_py_params(item.params.clone());
    let ret_str = emit_py_inferred(rt_type(item.clone()));
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("def ".to_string(), emit_ident(&item.name, RenderTarget::Python)), "(".to_string()), params_str), ")".to_string()), ret_str), ":\n".to_string()), "    raise NotImplementedError(\"extern func\")".to_string())
} else {
    v2_rt::concat("# unhandled node: ".to_string(), item.name.clone())
}
}
}
}
}
}
}
}
}

pub fn emit_py_type_def_from_connective(item: Rc<Node>) -> String {
    let is_product = node_is_product(item.clone());
    if is_product {
    emit_py_dataclass_from_children(&item.name, item.children.clone())
} else {
    emit_py_enum_from_children(&item.name, item.children.clone())
}
}

pub fn emit_py_dataclass_from_children(name: &str, children: Rc<Vec<Rc<Node>>>) -> String {
    if ({
    let __len_5 = children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(py_derive_attribute(), "\nclass ".to_string()), name.to_string()), ":\n    pass".to_string())
} else {
    let field_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in children.iter().cloned() {
        __mapped_0.push(emit_py_dataclass_field_from_child(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let fields_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in field_lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(py_derive_attribute(), "\nclass ".to_string()), name.to_string()), ":\n".to_string()), fields_str.clone())
}
}

pub fn emit_py_dataclass_field_from_child(child: Rc<Node>) -> String {
    let ty = emit_node_type(rt_type(child.clone()), RenderTarget::Python);
    let is_optional = node_is_optional(rt_type(child.clone()));
    let default_str = if is_optional {
    v2_rt::concat(" = ".to_string(), py_default_value())
} else {
    "".to_string()
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), emit_ident(&child.name, RenderTarget::Python)), ": ".to_string()), ty), default_str.clone())
}

pub fn emit_py_enum_from_children(name: &str, children: Rc<Vec<Rc<Node>>>) -> String {
    let has_data = {
    let mut __any_0 = false;
    for __elem_1 in children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.children.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    if has_data.clone() {
    let variant_classes = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in children.iter().cloned() {
        __mapped_3.push(emit_py_variant_class_from_child(&name, __elem_4.clone()));
    }
    Rc::new(__mapped_3)
};
    let variant_names = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in children.iter().cloned() {
        __mapped_5.push(v2_rt::concat(name.to_string(), __elem_6.name.clone()));
    }
    Rc::new(__mapped_5)
};
    let union_str = {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in variant_names.iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&", ".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
};
    let classes_str = {
    let mut __joined_10 = String::new();
    let mut __first_12 = true;
    for __elem_11 in variant_classes.iter().cloned() {
        if !__first_12 {
    __joined_10.push_str(&"\n\n\n".to_string());
};
        __first_12 = false;
        __joined_10.push_str(&__elem_11);
    }
    __joined_10
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(classes_str.clone(), "\n\n\n".to_string()), name.to_string()), " = Union[".to_string()), union_str.clone()), "]".to_string())
} else {
    let variant_lines = {
    let mut __mapped_13 = Vec::new();
    for __elem_14 in children.iter().cloned() {
        __mapped_13.push(v2_rt::concat(v2_rt::concat("    ".to_string(), __elem_14.name.clone()), " = auto()".to_string()));
    }
    Rc::new(__mapped_13)
};
    let variants_str = {
    let mut __joined_15 = String::new();
    let mut __first_17 = true;
    for __elem_16 in variant_lines.iter().cloned() {
        if !__first_17 {
    __joined_15.push_str(&"\n".to_string());
};
        __first_17 = false;
        __joined_15.push_str(&__elem_16);
    }
    __joined_15
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat("class ".to_string(), name.to_string()), "(Enum):\n".to_string()), variants_str.clone())
}
}

pub fn emit_py_variant_class_from_child(parent_name: &str, child: Rc<Node>) -> String {
    let class_name = v2_rt::concat(parent_name.to_string(), child.name.clone());
    if ({
    let __len_5 = child.children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(py_derive_attribute(), "\nclass ".to_string()), class_name.clone()), ":\n    pass".to_string())
} else {
    let field_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in child.children.iter().cloned() {
        __mapped_0.push(emit_py_dataclass_field_from_child(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let fields_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in field_lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(py_derive_attribute(), "\nclass ".to_string()), class_name.clone()), ":\n".to_string()), fields_str.clone())
}
}

pub fn emit_py_fn_def(name: &str, params: Rc<Vec<Rc<Param>>>, inferred: Rc<Node>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let depth = 0_i64;
    let params_str = emit_py_params(params.clone());
    let ret_str = emit_py_inferred(inferred.clone());
    let body_scope = build_params_scope(scope.clone(), params.clone());
    let use_tco = is_tco_eligible(&name, body.clone(), registry.clone());
    if use_tco {
    let body_str = emit_py_typed_tco_body(body.clone(), &name, params.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("def ".to_string(), emit_ident(&name, RenderTarget::Python)), "(".to_string()), params_str), ")".to_string()), ret_str), ":\n".to_string()), make_indent(depth.clone() + 1_i64)), body_str)
} else {
    let body_str = emit_py_typed_expr(body.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("def ".to_string(), emit_ident(&name, RenderTarget::Python)), "(".to_string()), params_str), ")".to_string()), ret_str), ":\n".to_string()), make_indent(depth.clone() + 1_i64)), "return ".to_string()), body_str)
}
}

pub fn emit_py_func_def(name: &str, params: Rc<Vec<Rc<Param>>>, inferred: Rc<Node>, uses: Rc<Vec<Rc<ResourceUse>>>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let depth = 0_i64;
    let service_names = match lookup_item(registry.clone(), &name) {
    Some(info) => {
        info.service_names.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let params_str = emit_py_func_params(params.clone(), uses.clone(), service_names.clone());
    let ret_str = emit_py_inferred(inferred.clone());
    let body_scope = build_params_scope(scope.clone(), params.clone());
    let body_str = emit_py_typed_func_body(body.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("async def ".to_string(), emit_ident(&name, RenderTarget::Python)), "(".to_string()), params_str), ")".to_string()), ret_str), ":\n".to_string()), make_indent(depth.clone() + 1_i64)), body_str)
}

pub fn emit_py_func_params(params: Rc<Vec<Rc<Param>>>, uses: Rc<Vec<Rc<ResourceUse>>>, service_names: Rc<Vec<String>>) -> String {
    let param_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_py_param(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let resource_strs = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in uses.iter().cloned() {
        __mapped_2.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.name, RenderTarget::Python), ": ".to_string()), emit_node_type(__elem_3.resource.clone(), RenderTarget::Python)));
    }
    Rc::new(__mapped_2)
};
    let service_strs = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in service_names.iter().cloned() {
        __mapped_4.push(v2_rt::concat(v2_rt::concat(service_var_name(&__elem_5), ": ".to_string()), sanitize_service_name(&__elem_5)));
    }
    Rc::new(__mapped_4)
};
    let all_params = v2_rt::concat(v2_rt::concat(param_strs.clone(), resource_strs.clone()), service_strs.clone());
    {
    let mut __joined_6 = String::new();
    let mut __first_8 = true;
    for __elem_7 in all_params.iter().cloned() {
        if !__first_8 {
    __joined_6.push_str(&", ".to_string());
};
        __first_8 = false;
        __joined_6.push_str(&__elem_7);
    }
    __joined_6
}
}

pub fn emit_py_params(params: Rc<Vec<Rc<Param>>>) -> String {
    let strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_py_param(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
}

pub fn emit_py_param(param: Rc<Param>) -> String {
    let ty = emit_node_type(param.type_expr.clone(), RenderTarget::Python);
    v2_rt::concat(v2_rt::concat(emit_ident(&param.name, RenderTarget::Python), ": ".to_string()), ty)
}

pub fn emit_py_inferred(inferred: Rc<Node>) -> String {
    v2_rt::concat(" -> ".to_string(), emit_node_type(inferred.clone(), RenderTarget::Python))
}

pub fn emit_py_pattern(pattern: Rc<MatchPattern>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::Bind { name: n, .. } => {
        emit_ident(&n, RenderTarget::Python)
    }
    MatchPattern::LitPattern { value: v, .. } => {
        emit_literal(v.clone(), RenderTarget::Python)
    }
    MatchPattern::VariantPattern { name: n, parent_enum: _, field_bindings: fbs, .. } => {
        emit_py_variant_pattern(&n, fbs.clone())
    }
    MatchPattern::Wildcard => {
        "_".to_string()
    }
}
    })
}

pub fn emit_py_variant_pattern(name: &str, field_bindings: Rc<Vec<Rc<FieldBinding>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if ({
    let __len_5 = field_bindings.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(name.to_string(), "()".to_string())
} else {
    let binding_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in field_bindings.iter().cloned() {
        __mapped_0.push({
    let pat_str = emit_py_pattern(__elem_1.binding.clone());
    v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.field_name, RenderTarget::Python), "=".to_string()), pat_str.clone())
});
    }
    Rc::new(__mapped_0)
};
    let bindings_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in binding_strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(name.to_string(), "(".to_string()), bindings_str.clone()), ")".to_string())
}
    })
}

pub fn emit_py_field_access(base: Rc<Node>, field: &str, summary: Option<Rc<FieldSummary>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_py_typed_expr(base.clone(), registry.clone(), scope.clone(), depth);
        match summary.as_ref().map(|__rc| __rc.as_ref()) {
    Some(fs) => {
        let fs = Rc::new(fs.clone());
        match fs.access_style.clone() {
    FieldAccessStyle::TupleFirst => {
        v2_rt::concat(base_str, "[0]".to_string())
    }
    FieldAccessStyle::TupleSecond => {
        v2_rt::concat(base_str, "[1]".to_string())
    }
    _ => {
        v2_rt::concat(v2_rt::concat(base_str, ".".to_string()), emit_ident(&field, RenderTarget::Python))
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(base_str, ".".to_string()), emit_ident(&field, RenderTarget::Python))
    }
}
    })
}

pub fn emit_py_expr_var(expr: Rc<Node>) -> String {
    match expr.expr_data.as_ref() {
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        emit_ident(&n, RenderTarget::Python)
    }
    _ => {
        emit_error_expr("emit_py_expr_var expected ExprVar", RenderTarget::Python)
    }
}
}

pub fn emit_py_expr_field_access(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprFieldAccess { field: f, summary, .. } => {
        {
    let b = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    if is_typed_service_call_receiver(expr.clone()) {
    match extract_typed_service_name(expr.clone()) {
    Some(svc_name) => {
        service_var_name(&svc_name)
    }
    None => {
        emit_py_field_access(b.clone(), &f, summary.clone(), registry.clone(), scope.clone(), depth)
    }
}
} else {
    emit_py_field_access(b.clone(), &f, summary.clone(), registry.clone(), scope.clone(), depth)
}
}
    }
    _ => {
        emit_error_expr("emit_py_expr_field_access expected ExprFieldAccess", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_call(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, call_semantics: _, .. } => {
        {
    let a = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in expr.children.iter().cloned() {
        __mapped_0.push(Rc::new(NamedArg { name: arg_name(__elem_1.clone()), value: arg_value(__elem_1.clone()) }));
    }
    Rc::new(__mapped_0)
};
    emit_py_typed_call(&f, a.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_call expected ExprCall", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_method_call(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprMethodCall { method: m, method_semantics, .. } => {
        {
    let r = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let a = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in { let __s = expr.children.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) }.iter().cloned() {
        __mapped_0.push(Rc::new(NamedArg { name: arg_name(__elem_1.clone()), value: arg_value(__elem_1.clone()) }));
    }
    Rc::new(__mapped_0)
};
    emit_py_typed_method_call(r.clone(), &m, a.clone(), method_semantics.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_method_call expected ExprMethodCall", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_match(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprMatch => {
        {
    let s = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let arm_list = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in { let __s = expr.children.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) }.iter().cloned() {
        __mapped_0.push(Rc::new(MatchArm { pattern: arm_pattern(__elem_1.clone()), guard: arm_guard(__elem_1.clone()), body: arm_body(__elem_1.clone()) }));
    }
    Rc::new(__mapped_0)
};
    emit_py_typed_match(s.clone(), arm_list.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_match expected ExprMatch", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_if(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprIf => {
        {
    let c = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let t = match expr.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let e = expr.children.clone().get((2_i64) as usize).cloned();
    emit_py_typed_if(c.clone(), t.clone(), e.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_if expected ExprIf", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_let(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprLet { name: n, .. } => {
        {
    let v = match expr.children.clone().first().cloned() {
    Some(val) => {
        val.clone()
    }
    None => {
        expr.clone()
    }
};
    let bd = expr.children.clone().get((1_i64) as usize).cloned();
    emit_py_typed_let(&n, v.clone(), bd.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_let expected ExprLet", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_record_lit(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprRecordLit { type_name: tn, parent_enum: _, .. } => {
        {
    let fs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in expr.children.iter().cloned() {
        __mapped_0.push(Rc::new(FieldInit { name: field_init_node_name(__elem_1.clone()), value: field_init_node_value(__elem_1.clone()) }));
    }
    Rc::new(__mapped_0)
};
    emit_py_typed_record_lit(tn.clone(), fs.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_record_lit expected ExprRecordLit", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_string_interp(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprStringInterp => {
        {
    let ps = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in expr.children.iter().cloned() {
        __mapped_0.push(match __elem_1.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitStr { value: text, .. } = value.as_ref() else { unreachable!() };
        Rc::new(StringPart::Text { value: text.clone() })
    }
    _ => {
        Rc::new(StringPart::Interpolation { expr: match __elem_1.children.clone().first().cloned() {
    Some(e) => {
        e.clone()
    }
    None => {
        __elem_1.clone()
    }
} })
    }
});
    }
    Rc::new(__mapped_0)
};
    emit_py_typed_string_interp(ps.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_string_interp expected ExprStringInterp", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_block(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprBlock => {
        emit_py_typed_block(expr.children.clone(), registry.clone(), scope.clone(), depth)
    }
    _ => {
        emit_error_expr("emit_py_expr_block expected ExprBlock", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_cast(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprCast => {
        {
    let e = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let t = match expr.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    emit_py_typed_cast(e.clone(), t.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_cast expected ExprCast", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_for_each(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprForEach { variable: v, .. } => {
        {
    let c = match expr.children.clone().first().cloned() {
    Some(val) => {
        val.clone()
    }
    None => {
        expr.clone()
    }
};
    let bd = match expr.children.clone().get((1_i64) as usize).cloned() {
    Some(val) => {
        val.clone()
    }
    None => {
        expr.clone()
    }
};
    emit_py_typed_for_each(&v, c.clone(), bd.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_for_each expected ExprForEach", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_index(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprIndex => {
        {
    let b = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let i = match expr.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    emit_py_typed_index(b.clone(), i.clone(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_index expected ExprIndex", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_expr_slice(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprSlice => {
        {
    let b = match expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let s = match expr.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        expr.clone()
    }
};
    let e = expr.children.clone().get((2_i64) as usize).cloned();
    emit_py_typed_slice(b.clone(), s.clone(), e.unwrap(), registry.clone(), scope.clone(), depth)
}
    }
    _ => {
        emit_error_expr("emit_py_expr_slice expected ExprSlice", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_typed_expr(texpr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        emit_shared_expr(texpr.clone(), RenderTarget::Python, |result| result.clone(), |child| emit_py_typed_expr(child.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_var(expr.clone()), |expr| emit_py_expr_field_access(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_call(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_method_call(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_match(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_if(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_let(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_record_lit(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_string_interp(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_block(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_cast(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_for_each(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_index(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_py_expr_slice(expr.clone(), registry.clone(), scope.clone(), depth.clone()))
    })
}

pub fn emit_py_typed_call(func: &str, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let ordered_args = order_typed_call_args(args.clone(), &func, scope.clone());
        let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in ordered_args.iter().cloned() {
        __mapped_0.push(emit_py_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
        let callee = lookup_item(registry.clone(), &func);
        let extra_args = match callee.as_ref().map(|__rc| __rc.as_ref()) {
    Some(info) => {
        let info = Rc::new(info.clone());
        {
    let is_func = info.kind.clone() == ItemKind::FuncItem;
    if is_func.clone() {
    {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in info.service_names.iter().cloned() {
        __mapped_2.push(service_var_name(&__elem_3));
    }
    Rc::new(__mapped_2)
}
} else {
    Rc::new(Vec::new())
}
}
    }
    None => {
        Rc::new(Vec::new())
    }
};
        let all_args = v2_rt::concat(arg_strs.clone(), extra_args.clone());
        let args_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in all_args.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
        let call_str = v2_rt::concat(v2_rt::concat(v2_rt::concat(emit_ident(&func, RenderTarget::Python), "(".to_string()), args_str.clone()), ")".to_string());
        match callee.as_ref().map(|__rc| __rc.as_ref()) {
    Some(info) => {
        let info = Rc::new(info.clone());
        {
    let is_func = info.kind.clone() == ItemKind::FuncItem;
    if is_func.clone() {
    v2_rt::concat("await ".to_string(), call_str.clone())
} else {
    call_str.clone()
}
}
    }
    None => {
        call_str.clone()
    }
}
    })
}

pub fn emit_py_typed_for_each(variable: &str, collection: Rc<Node>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let coll_str = emit_py_typed_expr(collection.clone(), registry.clone(), scope.clone(), depth.clone());
        let elem_type = for_each_element_type_node(rt_type(collection.clone()));
        let body_scope = extend_scope(scope.clone(), &variable, elem_type.clone());
        let body_str = emit_py_typed_expr(body.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("for ".to_string(), emit_ident(&variable, RenderTarget::Python)), " in ".to_string()), coll_str), ":\n".to_string()), make_indent(depth.clone() + 1_i64)), body_str)
    })
}

pub fn emit_py_typed_index(base: Rc<Node>, index: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_py_typed_expr(base.clone(), registry.clone(), scope.clone(), depth.clone());
        let index_str = emit_py_typed_expr(index.clone(), registry.clone(), scope.clone(), depth.clone());
        v2_rt::concat(v2_rt::concat(v2_rt::concat(base_str, "[".to_string()), index_str), "]".to_string())
    })
}

pub fn emit_py_typed_slice(base: Rc<Node>, start: Rc<Node>, end: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_py_typed_expr(base.clone(), registry.clone(), scope.clone(), depth.clone());
        let start_str = emit_py_typed_expr(start.clone(), registry.clone(), scope.clone(), depth.clone());
        let end_str = emit_py_typed_expr(end.clone(), registry.clone(), scope.clone(), depth.clone());
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(base_str, "[".to_string()), start_str), ":".to_string()), end_str), "]".to_string())
    })
}

pub fn emit_py_intrinsic_method_call(intrinsic: IntrinsicMethod, receiver: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let recv_str = emit_py_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone());
        let first_arg_str = emit_py_typed_first_arg(args.clone(), registry.clone(), scope.clone(), depth.clone());
        match intrinsic {
    IntrinsicMethod::MethodCount => {
        v2_rt::concat(v2_rt::concat("len(".to_string(), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodJoin => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(first_arg_str, ".join(".to_string()), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodSplit => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), ".split(".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodLast => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[-1] if ".to_string()), recv_str.clone()), " else None".to_string())
    }
    IntrinsicMethod::MethodFirst => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[0] if ".to_string()), recv_str.clone()), " else None".to_string())
    }
    IntrinsicMethod::MethodEnumerate => {
        v2_rt::concat(v2_rt::concat("list(enumerate(".to_string(), recv_str.clone()), "))".to_string())
    }
    IntrinsicMethod::MethodChars => {
        v2_rt::concat(v2_rt::concat("list(".to_string(), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodStringContains => {
        v2_rt::concat(v2_rt::concat(first_arg_str, " in ".to_string()), recv_str.clone())
    }
    IntrinsicMethod::MethodConcat => {
        v2_rt::concat(v2_rt::concat(recv_str.clone(), " + ".to_string()), first_arg_str)
    }
    IntrinsicMethod::MethodMap => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("[".to_string(), first_arg_str), "(x) for x in ".to_string()), recv_str.clone()), "]".to_string())
    }
    IntrinsicMethod::MethodFilter => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("[x for x in ".to_string(), recv_str.clone()), " if ".to_string()), first_arg_str), "(x)]".to_string())
    }
    IntrinsicMethod::MethodAny => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("any(".to_string(), first_arg_str), "(x) for x in ".to_string()), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodAll => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("all(".to_string(), first_arg_str), "(x) for x in ".to_string()), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodFlatMap => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("[y for x in ".to_string(), recv_str.clone()), " for y in ".to_string()), first_arg_str), "(x)]".to_string())
    }
    IntrinsicMethod::MethodSkip => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[".to_string()), first_arg_str), ":]".to_string())
    }
    IntrinsicMethod::MethodTake => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[:".to_string()), first_arg_str), "]".to_string())
    }
    IntrinsicMethod::MethodFold => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("functools.reduce(".to_string(), first_arg_str), ", ".to_string()), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodSortBy => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("sorted(".to_string(), recv_str.clone()), ", key=".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodAppend => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), " + [".to_string()), first_arg_str), "]".to_string())
    }
}
    })
}

pub fn py_bridge_method_name(method: RuntimeBridgeMethod) -> String {
    if method.clone() == RuntimeBridgeMethod::BridgeWith {
    "with_update".to_string()
} else {
    bridge_method_base_name(method.clone())
}
}

pub fn emit_py_runtime_bridge_method_call(method: RuntimeBridgeMethod, receiver: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let function_name = py_bridge_method_name(method);
        let recv_str = emit_py_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone());
        let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_py_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
        let all_args = v2_rt::concat(Rc::new(vec!(recv_str)), arg_strs.clone());
        v2_rt::concat(v2_rt::concat(v2_rt::concat(emit_ident(&function_name, RenderTarget::Python), "(".to_string()), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in all_args.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), ")".to_string())
    })
}

pub fn emit_py_plain_method_call(receiver: Rc<Node>, method: &str, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let recv_str = emit_py_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone());
        let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_py_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
        let args_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in arg_strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".".to_string()), emit_ident(&method, RenderTarget::Python)), "(".to_string()), args_str.clone()), ")".to_string())
    })
}

pub fn emit_py_typed_method_call(receiver: Rc<Node>, method: &str, args: Rc<Vec<Rc<NamedArg>>>, method_semantics: Option<Rc<MethodSemantics>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if method_semantics.clone().is_some() {
    match method_semantics.clone().unwrap().as_ref() {
    MethodSemantics::ServiceMethodSemantics { service_name: svc_name, .. } => {
        {
    let var_name = service_var_name(&svc_name);
    let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_py_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
    let args_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in arg_strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("await ".to_string(), var_name), ".".to_string()), emit_ident(&method, RenderTarget::Python)), "(".to_string()), args_str.clone()), ")".to_string())
}
    }
    MethodSemantics::IntrinsicMethodSemantics { intrinsic, fold_accumulator_type: _, .. } => {
        emit_py_intrinsic_method_call(intrinsic.clone(), receiver.clone(), args.clone(), registry.clone(), scope.clone(), depth.clone())
    }
    MethodSemantics::RuntimeBridgeSemantics { method: bridge_method, .. } => {
        emit_py_runtime_bridge_method_call(bridge_method.clone(), receiver.clone(), args.clone(), registry.clone(), scope.clone(), depth.clone())
    }
    MethodSemantics::PlainMethodSemantics => {
        emit_py_plain_method_call(receiver.clone(), &method, args.clone(), registry.clone(), scope.clone(), depth.clone())
    }
}
} else {
    if is_typed_service_call_receiver(receiver.clone()) {
    match extract_typed_service_name(receiver.clone()) {
    Some(svc_name) => {
        {
    let var_name = service_var_name(&svc_name);
    let arg_strs = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in args.iter().cloned() {
        __mapped_5.push(emit_py_typed_expr(__elem_6.value.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_5)
};
    let args_str = {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in arg_strs.iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&", ".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("await ".to_string(), var_name), ".".to_string()), emit_ident(&method, RenderTarget::Python)), "(".to_string()), args_str.clone()), ")".to_string())
}
    }
    None => {
        "raise NotImplementedError(\"unsupported service receiver\")".to_string()
    }
}
} else {
    emit_py_plain_method_call(receiver.clone(), &method, args.clone(), registry.clone(), scope.clone(), depth.clone())
}
}
    })
}

pub fn emit_py_typed_first_arg(args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match args.clone().first().cloned() {
    Some(a) => {
        emit_py_typed_expr(a.value.clone(), registry.clone(), scope.clone(), depth)
    }
    None => {
        "raise NotImplementedError(\"missing method argument\")".to_string()
    }
}
    })
}

pub fn emit_py_typed_match(scrutinee: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let scrut_str = emit_py_typed_expr(scrutinee.clone(), registry.clone(), scope.clone(), depth.clone());
        let arm_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in arms.iter().cloned() {
        __mapped_0.push(emit_py_typed_match_arm(__elem_1.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
        let arms_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in arm_strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
        v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), ":\n".to_string()), arms_str.clone())
    })
}

pub fn emit_py_typed_match_arm(arm: Rc<MatchArm>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let pat_str = emit_py_pattern(arm.pattern.clone());
        let guard_str = match arm.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(g) => {
        let g = Rc::new(g.clone());
        v2_rt::concat(" if ".to_string(), emit_py_typed_expr(g.clone(), registry.clone(), scope.clone(), depth.clone()))
    }
    None => {
        "".to_string()
    }
};
        let body_str = emit_py_typed_expr(arm.body.clone(), registry.clone(), scope.clone(), depth.clone() + 2_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone() + 1_i64), "case ".to_string()), pat_str), guard_str.clone()), ":\n".to_string()), make_indent(depth.clone() + 2_i64)), body_str)
    })
}

pub fn emit_py_typed_if(condition: Rc<Node>, then_branch: Rc<Node>, else_branch: Option<Rc<Node>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let cond_str = emit_py_typed_expr(condition.clone(), registry.clone(), scope.clone(), depth.clone());
        let then_str = emit_py_typed_expr(then_branch.clone(), registry.clone(), scope.clone(), depth.clone());
        match else_branch.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        {
    let else_str = emit_py_typed_expr(eb.clone(), registry.clone(), scope.clone(), depth.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), then_str), ") if (".to_string()), cond_str), ") else (".to_string()), else_str), ")".to_string())
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), ":\n".to_string()), make_indent(depth.clone() + 1_i64)), then_str)
    }
}
    })
}

pub fn emit_py_typed_let(name: &str, value: Rc<Node>, body: Option<Rc<Node>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let val_str = emit_py_typed_expr(value.clone(), registry.clone(), scope.clone(), depth);
        let let_line = emit_let_binding(&name, &val_str, RenderTarget::Python);
        match body.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        {
    let next_scope = extend_scope(scope.clone(), &name, rt_type(value.clone()));
    v2_rt::concat(v2_rt::concat(let_line, "\n".to_string()), emit_py_typed_expr(bd.clone(), registry.clone(), next_scope.clone(), depth))
}
    }
    None => {
        let_line
    }
}
    })
}

pub fn emit_py_typed_record_lit(type_name: Option<String>, fields: Rc<Vec<Rc<FieldInit>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match type_name {
    None => {
        if ({
    let __len_5 = fields.clone().len();
    __len_5 as i64
}) == 0_i64 {
    "{}".to_string()
} else {
    let field_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in fields.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    \"".to_string(), __elem_1.name.clone()), "\": ".to_string()), emit_py_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone())), ",".to_string()));
    }
    Rc::new(__mapped_0)
};
    let fields_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in field_strs.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat("{\n".to_string(), fields_str.clone()), "\n}".to_string())
}
    }
    Some(tn) => {
        if ({
    let __len_11 = fields.clone().len();
    __len_11 as i64
}) == 0_i64 {
    v2_rt::concat(tn, "()".to_string())
} else {
    let field_strs = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in fields.iter().cloned() {
        __mapped_6.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_7.name, RenderTarget::Python), "=".to_string()), emit_py_typed_expr(__elem_7.value.clone(), registry.clone(), scope.clone(), depth.clone())));
    }
    Rc::new(__mapped_6)
};
    let fields_str = {
    let mut __joined_8 = String::new();
    let mut __first_10 = true;
    for __elem_9 in field_strs.iter().cloned() {
        if !__first_10 {
    __joined_8.push_str(&", ".to_string());
};
        __first_10 = false;
        __joined_8.push_str(&__elem_9);
    }
    __joined_8
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(tn, "(".to_string()), fields_str.clone()), ")".to_string())
}
    }
}
    })
}

pub fn emit_py_typed_bin_op(op: BinOpKind, left: Rc<Node>, right: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    let l_str = emit_py_typed_expr(left.clone(), registry.clone(), scope.clone(), depth.clone());
    let r_str = emit_py_typed_expr(right.clone(), registry.clone(), scope.clone(), depth.clone());
    if is_null_coalesce(op.clone()) {
    emit_null_coalesce(&l_str, &r_str, RenderTarget::Python)
} else {
    let op_str = emit_bin_op_symbol(op.clone(), RenderTarget::Python);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), l_str), " ".to_string()), op_str), " ".to_string()), r_str), ")".to_string())
}
}

pub fn emit_py_typed_string_interp(parts: Rc<Vec<Rc<StringPart>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let segments = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in parts.iter().cloned() {
        __mapped_0.push(py_typed_interp_segment(__elem_1.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
        let fstr = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in segments.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
        v2_rt::concat(v2_rt::concat("f\"".to_string(), fstr.clone()), "\"".to_string())
    })
}

pub fn py_typed_interp_segment(part: Rc<StringPart>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match part.as_ref() {
    StringPart::Text { value: v, .. } => {
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
    for __part_1 in v.clone().split("{".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"{{".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}).split("}".to_string().as_str()) {
        __split_parts_5.push(__part_6.to_string());
    }
    __split_parts_5
}).iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&"}}".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
}
    }
    StringPart::Interpolation { expr: e, .. } => {
        v2_rt::concat(v2_rt::concat("{".to_string(), emit_py_typed_expr(e.clone(), registry.clone(), scope.clone(), depth)), "}".to_string())
    }
}
    })
}

pub fn emit_py_typed_block(stmts: Rc<Vec<Rc<Node>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let state = emit_py_block_stmts(stmts.clone(), Rc::new(Vec::new()), scope.clone(), registry.clone(), depth);
        {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in state.text.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&"\n".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}
    })
}

pub fn emit_py_typed_cast(expr: Rc<Node>, target: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let expr_str = emit_py_typed_expr(expr.clone(), registry.clone(), scope.clone(), depth);
        let ty_str = emit_node_type(target.clone(), RenderTarget::Python);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(ty_str, "(".to_string()), expr_str), ")".to_string())
    })
}

pub fn emit_py_typed_func_body(body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match body.expr_data.as_ref() {
    ExprData::ExprLet { name: n, .. } => {
        {
    let ch = body.children.clone();
    let v = match ch.clone().first().cloned() {
    Some(val) => {
        val.clone()
    }
    None => {
        body.clone()
    }
};
    let inner = ch.clone().get((1_i64) as usize).cloned();
    let val_str = emit_py_typed_expr(v.clone(), registry.clone(), scope.clone(), depth);
    let let_line = emit_let_binding(&n, &val_str, RenderTarget::Python);
    let next_scope = extend_scope(scope.clone(), &n, rt_type(v.clone()));
    match inner.clone() {
    Some(bd) => {
        v2_rt::concat(v2_rt::concat(let_line, "\n".to_string()), emit_py_typed_func_body(bd.clone(), registry.clone(), next_scope.clone(), depth))
    }
    None => {
        let_line
    }
}
}
    }
    ExprData::ExprBlock => {
        {
    let ss = body.children.clone();
    if ({
    let __len_7 = ss.clone().len();
    __len_7 as i64
}) == 0_i64 {
    "return None".to_string()
} else {
    let init_state = emit_py_init_block_stmts(ss.clone(), Rc::new(Vec::new()), scope.clone(), registry.clone(), depth);
    let last_stmt = ss.clone().last().cloned();
    let last_str = match last_stmt.clone() {
    Some(s) => {
        v2_rt::concat("return ".to_string(), emit_py_typed_expr(s.clone(), registry.clone(), init_state.scope.clone(), depth))
    }
    None => {
        "return None".to_string()
    }
};
    if ({
    let __len_6 = init_state.text.clone().len();
    __len_6 as i64
}) == 0_i64 {
    last_str.clone()
} else {
    v2_rt::concat(v2_rt::concat({
    let mut __joined_3 = String::new();
    let mut __first_5 = true;
    for __elem_4 in init_state.text.iter().cloned() {
        if !__first_5 {
    __joined_3.push_str(&"\n".to_string());
};
        __first_5 = false;
        __joined_3.push_str(&__elem_4);
    }
    __joined_3
}, "\n".to_string()), last_str.clone())
}
}
}
    }
    _ => {
        v2_rt::concat("return ".to_string(), emit_py_typed_expr(body.clone(), registry.clone(), scope.clone(), depth))
    }
}
    })
}

pub fn emit_py_typed_tco_body(texpr: Rc<Node>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    let inner = emit_py_typed_tco_expr(texpr.clone(), &fn_name, params.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat("while True:\n".to_string(), make_indent(depth.clone() + 1_i64)), inner)
}

pub fn emit_py_tco_non_self_call(frame: Rc<TcoFrame>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    match frame.expr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, call_semantics: _, .. } => {
        {
    let a = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in frame.expr.children.iter().cloned() {
        __mapped_0.push(Rc::new(NamedArg { name: arg_name(__elem_1.clone()), value: arg_value(__elem_1.clone()) }));
    }
    Rc::new(__mapped_0)
};
    let call_str = emit_py_typed_call(&f, a.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone());
    v2_rt::concat("return ".to_string(), call_str)
}
    }
    _ => {
        emit_error_expr("emit_py_tco_non_self_call expected ExprCall", RenderTarget::Python)
    }
}
}

pub fn emit_py_tco_if(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprIf => {
        {
    let c = match frame.expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        frame.expr.clone()
    }
};
    let t = match frame.expr.children.clone().get((1_i64) as usize).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        frame.expr.clone()
    }
};
    let e = frame.expr.children.clone().get((2_i64) as usize).cloned();
    let cond_str = emit_py_typed_expr(c.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone());
    let then_str = emit_py_typed_tco_expr(t.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone() + 1_i64);
    match e.clone() {
    Some(eb) => {
        {
    let else_str = emit_py_typed_tco_expr(eb.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), ":\n".to_string()), make_indent(frame.depth.clone() + 1_i64)), then_str), "\nelse:\n".to_string()), make_indent(frame.depth.clone() + 1_i64)), else_str)
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), ":\n".to_string()), make_indent(frame.depth.clone() + 1_i64)), then_str)
    }
}
}
    }
    _ => {
        emit_error_expr("emit_py_tco_if expected ExprIf", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_tco_match(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprMatch => {
        {
    let s = match frame.expr.children.clone().first().cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        frame.expr.clone()
    }
};
    let arm_list = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in { let __s = frame.expr.children.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) }.iter().cloned() {
        __mapped_0.push(Rc::new(MatchArm { pattern: arm_pattern(__elem_1.clone()), guard: arm_guard(__elem_1.clone()), body: arm_body(__elem_1.clone()) }));
    }
    Rc::new(__mapped_0)
};
    let scrut_str = emit_py_typed_expr(s.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone());
    let arm_strs = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in arm_list.iter().cloned() {
        __mapped_2.push(emit_py_typed_tco_match_arm(__elem_3.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone()));
    }
    Rc::new(__mapped_2)
};
    let arms_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in arm_strs.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"\n".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), ":\n".to_string()), arms_str.clone())
}
    }
    _ => {
        emit_error_expr("emit_py_tco_match expected ExprMatch", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_tco_let(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprLet { name: n, .. } => {
        {
    let v = match frame.expr.children.clone().first().cloned() {
    Some(val) => {
        val.clone()
    }
    None => {
        frame.expr.clone()
    }
};
    let bd = frame.expr.children.clone().get((1_i64) as usize).cloned();
    let val_str = emit_py_typed_expr(v.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone());
    let let_line = emit_let_binding(&n, &val_str, RenderTarget::Python);
    let next_scope = extend_scope(frame.scope.clone(), &n, rt_type(v.clone()));
    match bd.clone() {
    Some(b) => {
        v2_rt::concat(v2_rt::concat(let_line, "\n".to_string()), emit_py_typed_tco_expr(b.clone(), &fn_name, params.clone(), registry.clone(), next_scope.clone(), frame.depth.clone()))
    }
    None => {
        let_line
    }
}
}
    }
    _ => {
        emit_error_expr("emit_py_tco_let expected ExprLet", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_tco_block(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprBlock => {
        {
    let ss = frame.expr.children.clone();
    if ({
    let __len_7 = ss.clone().len();
    __len_7 as i64
}) == 0_i64 {
    "return None".to_string()
} else {
    let init_state = emit_py_init_block_stmts(ss.clone(), Rc::new(Vec::new()), frame.scope.clone(), registry.clone(), frame.depth.clone());
    let last_str = match ss.clone().last().cloned() {
    Some(last_expr) => {
        emit_py_typed_tco_expr(last_expr.clone(), &fn_name, params.clone(), registry.clone(), init_state.scope.clone(), frame.depth.clone())
    }
    None => {
        "return None".to_string()
    }
};
    if ({
    let __len_6 = init_state.text.clone().len();
    __len_6 as i64
}) == 0_i64 {
    last_str.clone()
} else {
    v2_rt::concat(v2_rt::concat({
    let mut __joined_3 = String::new();
    let mut __first_5 = true;
    for __elem_4 in init_state.text.iter().cloned() {
        if !__first_5 {
    __joined_3.push_str(&"\n".to_string());
};
        __first_5 = false;
        __joined_3.push_str(&__elem_4);
    }
    __joined_3
}, "\n".to_string()), last_str.clone())
}
}
}
    }
    _ => {
        emit_error_expr("emit_py_tco_block expected ExprBlock", RenderTarget::Python)
    }
}
    })
}

pub fn emit_py_tco_default_return(frame: Rc<TcoFrame>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let val_str = emit_py_typed_expr(frame.expr.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone());
    v2_rt::concat("return ".to_string(), val_str)
}

pub fn emit_py_typed_tco_expr(texpr: Rc<Node>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        emit_shared_tco_expr(Rc::new(TcoFrame { expr: texpr.clone(), scope: scope.clone(), depth }), &fn_name, |input| emit_py_typed_tco_reassign(input.args.clone(), params.clone(), registry.clone(), input.scope.clone(), input.depth.clone()), |frame| emit_py_tco_non_self_call(frame.clone(), registry.clone()), |frame| emit_py_tco_if(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_py_tco_match(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_py_tco_let(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_py_tco_block(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_py_tco_default_return(frame.clone(), registry.clone()))
    })
}

pub fn emit_py_typed_tco_match_arm(arm: Rc<MatchArm>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let pat_str = emit_py_pattern(arm.pattern.clone());
        let guard_str = match arm.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(g) => {
        let g = Rc::new(g.clone());
        v2_rt::concat(" if ".to_string(), emit_py_typed_expr(g.clone(), registry.clone(), scope.clone(), depth.clone()))
    }
    None => {
        "".to_string()
    }
};
        let body_str = emit_py_typed_tco_expr(arm.body.clone(), &fn_name, params.clone(), registry.clone(), scope.clone(), depth.clone() + 2_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone() + 1_i64), "case ".to_string()), pat_str), guard_str.clone()), ":\n".to_string()), make_indent(depth.clone() + 2_i64)), body_str)
    })
}

pub fn emit_py_typed_tco_reassign(args: Rc<Vec<Rc<NamedArg>>>, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    let ordered_args = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_py_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone()));
    }
    Rc::new(__mapped_0)
};
    let param_names = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in params.iter().cloned() {
        __mapped_2.push(emit_ident(&__elem_3.name, RenderTarget::Python));
    }
    Rc::new(__mapped_2)
};
    let all_lines = tco_reassign_core(ordered_args.clone(), param_names.clone(), "__tco_", "", " = ", "", "continue", "");
    {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in all_lines.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"\n".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
}
}

pub fn emit_py_service_def(item: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let depth = 0_i64;
    let safe_name = sanitize_service_name(&item.name);
    let transport = service_fallback_transport(item.clone());
    let init_method = emit_py_service_init(transport.clone());
    let op_children = item.children.clone();
    let methods = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in op_children.iter().cloned() {
        __mapped_0.push(emit_py_operation_method(&safe_name, transport.clone(), __elem_1.clone(), registry.clone(), depth.clone() + 1_i64));
    }
    Rc::new(__mapped_0)
};
    let methods_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in methods.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("class ".to_string(), safe_name.clone()), ":\n".to_string()), make_indent(depth.clone() + 1_i64)), init_method), "\n\n".to_string()), make_indent(depth.clone() + 1_i64)), methods_str.clone())
}

pub fn emit_py_service_init(transport: Rc<Node>) -> String {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::RestTransport)) {
    let auth_param = if transport_has_auth(transport.clone()) {
    ", auth_token: str".to_string()
} else {
    "".to_string()
};
    let auth_assign = if transport_has_auth(transport.clone()) {
    "\n    self.auth_token = auth_token".to_string()
} else {
    "".to_string()
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("def __init__(self, base_url: str".to_string(), auth_param.clone()), "):\n".to_string()), "    self.base_url = base_url".to_string()), auth_assign.clone())
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::ShellTransport)) {
    v2_rt::concat("def __init__(self, working_dir: str | None = None):\n".to_string(), "    self.working_dir = working_dir".to_string())
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::FileTransport)) {
    v2_rt::concat("def __init__(self, base_path: str):\n".to_string(), "    self.base_path = base_path".to_string())
} else {
    "def __init__(self):\n    pass".to_string()
}
}
}
}

pub fn emit_py_operation_method(service_name: &str, transport: Rc<Node>, op_node: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> String {
    let input_params = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in op_node.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.name, RenderTarget::Python), ": ".to_string()), emit_node_type(__elem_1.type_expr.clone(), RenderTarget::Python)));
    }
    Rc::new(__mapped_0)
};
    let params_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in input_params.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let all_params = if params_str.clone() == "" {
    "self".to_string()
} else {
    v2_rt::concat("self, ".to_string(), params_str.clone())
};
    let ret_type = emit_node_type(rt_type(op_node.clone()), RenderTarget::Python);
    let eff_transport = effective_operation_transport(op_node.clone(), transport.clone());
    let body = emit_py_transport_call(eff_transport.clone(), &op_node.name, registry.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("async def ".to_string(), emit_ident(&op_node.name, RenderTarget::Python)), "(".to_string()), all_params.clone()), ") -> ".to_string()), ret_type), ":\n".to_string()), make_indent(depth + 1_i64)), body)
}

pub fn emit_py_transport_call(transport: Rc<Node>, op_name: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::RestTransport)) {
    emit_py_rest_call(&op_name, transport.clone())
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::ShellTransport)) {
    emit_py_shell_call(&op_name, transport.clone())
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::FileTransport)) {
    emit_py_file_call(&op_name)
} else {
    emit_py_local_call(&op_name)
}
}
}
}

pub fn emit_py_rest_call(op_name: &str, transport: Rc<Node>) -> String {
    let self_base_url = v2_rt::concat(v2_rt::concat("{".to_string(), "self.base_url".to_string()), "}".to_string());
    let url_line = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("url = f\"".to_string(), self_base_url.clone()), "/".to_string()), emit_ident(&op_name, RenderTarget::Python)), "\"".to_string());
    let headers_dict = emit_py_headers_dict(transport.clone());
    let session_lines = v2_rt::concat(v2_rt::concat("async with aiohttp.ClientSession() as session:\n".to_string(), "    async with session.post(url, headers=headers) as response:\n".to_string()), "        return await response.json()".to_string());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(url_line.clone(), "\n".to_string()), headers_dict), "\n".to_string()), session_lines.clone())
}

pub fn emit_py_headers_dict(transport: Rc<Node>) -> String {
    let auth_entry = if transport_has_auth(transport.clone()) {
    let header_name = match transport_auth_header_name(transport.clone()) {
    Some(h) => {
        h
    }
    None => {
        "Authorization".to_string()
    }
};
    v2_rt::concat(v2_rt::concat("\"".to_string(), header_name.clone()), "\": self.auth_token, ".to_string())
} else {
    "".to_string()
};
    let hdrs = transport_headers(transport.clone());
    let header_entries = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in hdrs.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("\"".to_string(), __elem_1.name.clone()), "\": _unimplemented(\"header value emission for '".to_string()), __elem_1.name.clone()), "'\")".to_string()));
    }
    Rc::new(__mapped_0)
};
    let all_entries = if auth_entry.clone() == "" {
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in header_entries.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
} else {
    v2_rt::concat(auth_entry.clone(), {
    let mut __joined_5 = String::new();
    let mut __first_7 = true;
    for __elem_6 in header_entries.iter().cloned() {
        if !__first_7 {
    __joined_5.push_str(&", ".to_string());
};
        __first_7 = false;
        __joined_5.push_str(&__elem_6);
    }
    __joined_5
})
};
    v2_rt::concat(v2_rt::concat("headers = {".to_string(), all_entries.clone()), "}".to_string())
}

pub fn emit_py_shell_call(op_name: &str, transport: Rc<Node>) -> String {
    let envs = transport_env(transport.clone());
    let env_dict_entries = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in envs.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("\"".to_string(), __elem_1.name.clone()), "\": _unimplemented(\"env value emission for '".to_string()), __elem_1.name.clone()), "'\")".to_string()));
    }
    Rc::new(__mapped_0)
};
    let env_str = v2_rt::concat(v2_rt::concat("{".to_string(), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in env_dict_entries.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), "}".to_string());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("import subprocess\n".to_string(), "result = subprocess.run(\n".to_string()), "    [\"".to_string()), emit_ident(&op_name, RenderTarget::Python)), "\"],\n".to_string()), "    cwd=self.working_dir or \".\",\n".to_string()), "    env=".to_string()), env_str.clone()), ",\n".to_string()), "    capture_output=True, text=True,\n".to_string()), ")\n".to_string()), "return result.stdout".to_string())
}

pub fn emit_py_file_call(op_name: &str) -> String {
    let self_base_path = v2_rt::concat(v2_rt::concat("{".to_string(), "self.base_path".to_string()), "}".to_string());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("path = f\"".to_string(), self_base_path.clone()), "/".to_string()), emit_ident(&op_name, RenderTarget::Python)), "\"\n".to_string()), "with open(path) as f:\n".to_string()), "    return f.read()".to_string())
}

pub fn emit_py_local_call(op_name: &str) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat("# Local binding -- direct function call\n".to_string(), "return ".to_string()), emit_ident(&op_name, RenderTarget::Python)), "()".to_string())
}

pub fn emit_py_resource_def(item: Rc<Node>) -> String {
    let depth = 0_i64;
    let cap_children = item.children.clone();
    let methods = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in cap_children.iter().cloned() {
        __mapped_0.push(emit_py_capability_method(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let methods_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in methods.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("from abc import ABC, abstractmethod\n\n".to_string(), "class ".to_string()), item.name.clone()), "(ABC):\n".to_string()), make_indent(depth.clone() + 1_i64)), methods_str.clone())
}

pub fn emit_py_capability_method(cap_node: Rc<Node>) -> String {
    let input_params = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in cap_node.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.name, RenderTarget::Python), ": ".to_string()), emit_node_type(__elem_1.type_expr.clone(), RenderTarget::Python)));
    }
    Rc::new(__mapped_0)
};
    let params_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in input_params.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let all_params = if params_str.clone() == "" {
    "self".to_string()
} else {
    v2_rt::concat("self, ".to_string(), params_str.clone())
};
    let ret = emit_node_type(rt_type(cap_node.clone()), RenderTarget::Python);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("@abstractmethod\n".to_string(), "async def ".to_string()), emit_ident(&cap_node.name, RenderTarget::Python)), "(".to_string()), all_params.clone()), ") -> ".to_string()), ret), ":\n".to_string()), "    ...".to_string())
}

pub fn emit_py_data_def(name: &str, type_node: Rc<Node>, value: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let ty_str = emit_node_type(type_node.clone(), RenderTarget::Python);
    let upper_name = to_screaming_snake(&name);
    let val_str = emit_py_typed_expr(value.clone(), registry.clone(), scope.clone(), 0_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(upper_name, ": ".to_string()), ty_str), " = ".to_string()), val_str)
}

pub fn emit_requirements_txt(has_services: bool) -> Rc<TextFile> {
    let base_deps = "".to_string();
    let async_deps = if has_services {
    "aiohttp>=3.9\n".to_string()
} else {
    "".to_string()
};
    Rc::new(TextFile { path: "requirements.txt".to_string(), content: v2_rt::concat(base_deps.clone(), async_deps.clone()) })
}

