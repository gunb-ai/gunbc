use crate::v2_core::*;
use crate::artifact::*;
use crate::go_emit::*;
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

pub fn emit_go_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> Rc<BlockEmitState> {
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
    let line = emit_go_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth.clone());
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

pub fn emit_go_init_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> Rc<BlockEmitState> {
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
    let line = emit_go_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth.clone());
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

pub fn emit_go(typed: Rc<ResolvedGraph>) -> Rc<EmitResult> {
    let registry = typed.item_registry.clone();
    let test_projections = extract_test_projections(typed.clone());
    let module_files = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in typed.modules.iter().cloned() {
        __mapped_0.push(emit_go_module(__elem_1.clone(), registry.clone()));
    }
    Rc::new(__mapped_0)
};
    let test_files = {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in ({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in typed.modules.iter().cloned() {
        __mapped_2.push(emit_go_test_file(&__elem_3.module.name, {
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
    let go_mod = emit_go_mod("generated");
    let files = v2_rt::concat(v2_rt::concat(Rc::new(vec!(go_mod.clone())), module_files.clone()), test_files.clone());
    Rc::new(EmitResult { files: files.clone(), diagnostics: Rc::new(Vec::new()) })
}

pub fn emit_go_mod(module_name: &str) -> Rc<TextFile> {
    let manifest_path = match scaffold_for_target(RenderTarget::Go).manifest_file.clone() {
    Some(path) => {
        path
    }
    None => {
        "go.mod".to_string()
    }
};
    let content = v2_rt::concat(v2_rt::concat("module ".to_string(), module_name.to_string()), "\n\ngo 1.21\n".to_string());
    Rc::new(TextFile { path: manifest_path.clone(), content: content.clone() })
}

pub fn go_source_extension() -> String {
    scaffold_for_target(RenderTarget::Go).source_file_extension.clone()
}

pub fn go_test_file_path(module_name: &str) -> String {
    let conventions = test_conventions_for_target(RenderTarget::Go);
    let file_dir = match conventions.file_dir.clone() {
    Some(dir) => {
        dir
    }
    None => {
        "".to_string()
    }
};
    let filename = module_to_filename(&module_name);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(file_dir.clone(), conventions.file_prefix.clone()), filename), conventions.file_suffix.clone()), go_source_extension())
}

pub fn go_test_name(projection: Rc<TestProjection>) -> String {
    test_function_name(projection.clone(), RenderTarget::Go)
}

pub fn go_mock_expr_uses_fmt(expr: Rc<Node>) -> bool {
    let rendered = emit_simple_expr(expr.clone(), RenderTarget::Go);
    ({
    let __len_2 = ({
    let mut __split_parts_0 = Vec::new();
    for __part_1 in rendered.split("fmt.Sprintf(".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
}).len();
    __len_2 as i64
}) > 1_i64
}

pub fn go_test_import_block(projections: Rc<Vec<Rc<TestProjection>>>) -> String {
    let needs_fmt = {
    let mut __any_0 = false;
    for __elem_1 in projections.iter().cloned() {
        {
let __cond = {
    let mut __any_2 = false;
    for __elem_3 in __elem_1.mock_field_inits.iter().cloned() {
        if go_mock_expr_uses_fmt(__elem_3.value.clone()) {
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
};
    if needs_fmt.clone() {
    "import (\n	\"fmt\"\n	\"testing\"\n)\n\n".to_string()
} else {
    "import \"testing\"\n\n".to_string()
}
}

pub fn go_test_signature_comment(projection: Rc<TestProjection>) -> String {
    let params_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projection.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(__elem_1.name.clone(), " ".to_string()), emit_node_type(__elem_1.type_expr.clone(), RenderTarget::Go)));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Signature: ".to_string(), sanitize_service_name(&projection.service_name)), ".".to_string()), projection.operation_name.clone()), "(".to_string()), params_str.clone()), ") ".to_string()), emit_node_type(projection.inferred.clone(), RenderTarget::Go))
}

pub fn emit_go_test_file(module_name: &str, projections: Rc<Vec<Rc<TestProjection>>>) -> Rc<TextFile> {
    if ({
    let __len_5 = projections.clone().len();
    __len_5 as i64
}) == 0_i64 {
    Rc::new(TextFile { path: "".to_string(), content: "".to_string() })
} else {
    let package_name = go_package_name(&module_name);
    let tests_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projections.iter().cloned() {
        __mapped_0.push(emit_go_operation_test(__elem_1.clone(), 0_i64));
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
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Generated tests -- do not edit.\n".to_string(), "// Source module: ".to_string()), module_name.to_string()), "\n\n".to_string()), "package ".to_string()), package_name), "\n\n".to_string()), go_test_import_block(projections.clone())), tests_str.clone()), "\n".to_string());
    Rc::new(TextFile { path: go_test_file_path(&module_name), content: content.clone() })
}
}

pub fn emit_go_operation_test(projection: Rc<TestProjection>, depth: i64) -> String {
    let test_name = go_test_name(projection.clone());
    let mock_setup = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projection.mock_field_inits.iter().cloned() {
        __mapped_0.push(emit_go_mock_prop_setup(__elem_1.clone(), depth.clone() + 1_i64));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func ".to_string(), test_name), "(t *testing.T) {\n".to_string()), make_indent(depth.clone() + 1_i64)), go_test_signature_comment(projection.clone())), "\n".to_string()), make_indent(depth.clone() + 1_i64)), mock_setup.clone()), "\n".to_string()), make_indent(depth.clone() + 1_i64)), "t.Helper()\n".to_string()), "}".to_string())
}

pub fn emit_go_mock_prop_setup(mock_prop: Rc<FieldInit>, depth: i64) -> String {
    v2_rt::concat(v2_rt::concat(emit_ident(&mock_prop.name, RenderTarget::Go), " := ".to_string()), emit_simple_expr(mock_prop.value.clone(), RenderTarget::Go))
}

pub fn emit_go_module(typed_module: Rc<TypedModule>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> Rc<TextFile> {
    let m = typed_module.module.clone();
    let scope = module_emit_scope(typed_module.clone());
    let pkg_name = go_package_name(&m.name);
    let pkg_decl = v2_rt::concat("package ".to_string(), pkg_name);
    let imports_str = emit_go_imports(typed_module.items.clone(), m.imports.clone());
    let imports_section = if imports_str.clone() == "" {
    "".to_string()
} else {
    v2_rt::concat("\n\n".to_string(), imports_str.clone())
};
    let items_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in typed_module.items.iter().cloned() {
        __mapped_0.push(emit_go_typed_item(__elem_1.clone(), registry.clone(), scope.clone()));
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
    let filename = module_to_filename(&m.name);
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Generated by v2 compiler -- do not edit.\n".to_string(), "// Source module: ".to_string()), m.name.clone()), "\n\n".to_string()), pkg_decl.clone()), imports_section.clone()), "\n\n".to_string()), items_str.clone()), "\n".to_string());
    Rc::new(TextFile { path: v2_rt::concat(filename, go_source_extension()), content: content.clone() })
}

pub fn go_package_name(module_name: &str) -> String {
    let parts = {
    let mut __split_parts_0 = Vec::new();
    for __part_1 in module_name.to_string().split(".".to_string().as_str()) {
        __split_parts_0.push(__part_1.to_string());
    }
    __split_parts_0
};
    match parts.clone().last().cloned() {
    Some(last_part) => {
        {
    let mut __joined_6 = String::new();
    let mut __first_8 = true;
    for __elem_7 in ({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __chars_2 = Vec::new();
    for __ch_3 in last_part.clone().chars() {
        __chars_2.push(__ch_3.to_string());
    }
    Rc::new(__chars_2)
}).iter().cloned() {
        __mapped_4.push(to_lower_char(&__elem_5));
    }
    Rc::new(__mapped_4)
}).iter().cloned() {
        if !__first_8 {
    __joined_6.push_str(&"".to_string());
};
        __first_8 = false;
        __joined_6.push_str(&__elem_7);
    }
    __joined_6
}
    }
    None => {
        "main".to_string()
    }
}
}

pub fn emit_go_imports(items: Rc<Vec<Rc<Node>>>, imports: Rc<Vec<Rc<Import>>>) -> String {
    let has_services = {
    let mut __any_0 = false;
    for __elem_1 in items.iter().cloned() {
        if is_service_item(__elem_1.clone()) {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    let has_errors = {
    let mut __any_2 = false;
    for __elem_3 in items.iter().cloned() {
        if (__elem_3.body.clone().is_some()) && (({
    let __len_4 = __elem_3.uses.clone().len();
    __len_4 as i64
}) > 0_i64) {
    __any_2 = true;
    break;
};
    }
    __any_2
};
    let std_imports = collect_go_std_imports(has_services.clone(), has_errors.clone());
    let module_imports = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in imports.iter().cloned() {
        __mapped_5.push({
    let mod_name = module_to_filename(&__elem_6.module_path);
    v2_rt::concat(v2_rt::concat("	\"generated/".to_string(), mod_name.clone()), "\"".to_string())
});
    }
    Rc::new(__mapped_5)
};
    let all_imports = v2_rt::concat(std_imports.clone(), module_imports.clone());
    if ({
    let __len_10 = all_imports.clone().len();
    __len_10 as i64
}) == 0_i64 {
    "".to_string()
} else {
    let imports_str = {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in all_imports.iter().cloned() {
        if !__first_9 {
    __joined_7.push_str(&"\n".to_string());
};
        __first_9 = false;
        __joined_7.push_str(&__elem_8);
    }
    __joined_7
};
    v2_rt::concat(v2_rt::concat("import (\n".to_string(), imports_str.clone()), "\n)".to_string())
}
}

pub fn collect_go_std_imports(has_services: bool, has_errors: bool) -> Rc<Vec<String>> {
    let base = Rc::new(vec!("	\"fmt\"".to_string()));
    let net_imports = if has_services {
    Rc::new(vec!("	\"net/http\"".to_string(), "	\"encoding/json\"".to_string(), "	\"bytes\"".to_string(), "	\"io\"".to_string()))
} else {
    Rc::new(Vec::new())
};
    v2_rt::concat(base.clone(), net_imports.clone())
}

pub fn emit_go_typed_item(item: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let kind = classify_typed_item(item.clone());
    if kind.clone() == TypedItemKind::TypedItemTypeDef {
    emit_go_type_def_from_connective(item.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemTypeAlias {
    emit_go_type_alias(&item.name, rt_type(item.clone()))
} else {
    if kind.clone() == TypedItemKind::TypedItemTypeDecl {
    "".to_string()
} else {
    if kind.clone() == TypedItemKind::TypedItemFunction {
    if ({
    let __len_0 = item.uses.clone().len();
    __len_0 as i64
}) > 0_i64 {
    emit_go_func_def(&item.name, item.params.clone(), rt_type(item.clone()), item.uses.clone(), item.body.clone().unwrap(), registry.clone(), scope.clone())
} else {
    emit_go_fn_def(&item.name, item.params.clone(), rt_type(item.clone()), item.body.clone().unwrap(), registry.clone(), scope.clone())
}
} else {
    if kind.clone() == TypedItemKind::TypedItemDataDef {
    emit_go_data_def(&item.name, item.type_annotation.clone().unwrap(), item.body.clone().unwrap(), registry.clone(), scope.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemServiceDef {
    emit_go_service_def(item.clone(), registry.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemResourceDef {
    emit_go_resource_def(item.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemExternFunc {
    let params_str = emit_go_params(item.params.clone());
    let ret_str = emit_go_inferred(rt_type(item.clone()));
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func ".to_string(), go_export_ident(&item.name)), "(".to_string()), params_str), ")".to_string()), ret_str), " {\n".to_string()), "	panic(\"extern func not implemented\")\n}".to_string())
} else {
    v2_rt::concat("// unhandled node: ".to_string(), item.name.clone())
}
}
}
}
}
}
}
}
}

pub fn emit_go_type_def_from_connective(item: Rc<Node>) -> String {
    let is_product = item.connective == Connective::Conj;
    if is_product {
    emit_go_struct_from_children(&item.name, item.children.clone())
} else {
    emit_go_sum_from_children(&item.name, item.children.clone())
}
}

pub fn emit_go_struct_from_children(name: &str, children: Rc<Vec<Rc<Node>>>) -> String {
    if ({
    let __len_5 = children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " struct{}".to_string())
} else {
    let field_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in children.iter().cloned() {
        __mapped_0.push(emit_go_struct_field_from_child(__elem_1.clone()));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " struct {\n".to_string()), fields_str.clone()), "\n}".to_string())
}
}

pub fn emit_go_struct_field_from_child(child: Rc<Node>) -> String {
    let ty = emit_node_type(rt_type(child.clone()), RenderTarget::Go);
    let json_tag = v2_rt::concat(v2_rt::concat(" `json:\"".to_string(), to_snake(&child.name)), "\"`".to_string());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("	".to_string(), go_export_ident(&child.name)), " ".to_string()), ty), json_tag.clone())
}

pub fn emit_go_sum_from_children(name: &str, children: Rc<Vec<Rc<Node>>>) -> String {
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
    let iface = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " interface {\n	is".to_string()), name.to_string()), "()\n}".to_string());
    let variant_structs = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in children.iter().cloned() {
        __mapped_3.push(emit_go_variant_struct(&name, __elem_4.clone()));
    }
    Rc::new(__mapped_3)
};
    let structs_str = {
    let mut __joined_5 = String::new();
    let mut __first_7 = true;
    for __elem_6 in variant_structs.iter().cloned() {
        if !__first_7 {
    __joined_5.push_str(&"\n\n".to_string());
};
        __first_7 = false;
        __joined_5.push_str(&__elem_6);
    }
    __joined_5
};
    v2_rt::concat(v2_rt::concat(iface.clone(), "\n\n".to_string()), structs_str.clone())
} else {
    let type_decl = v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " int".to_string());
    let variant_consts = {
    let mut __mapped_11 = Vec::new();
    for __elem_12 in ({
    let mut __enumerated_8 = Vec::new();
    for (__idx_9, __elem_10) in children.clone().iter().enumerate() {
        __enumerated_8.push((__idx_9 as i64, __elem_10.clone()));
    }
    Rc::new(__enumerated_8)
}).iter().cloned() {
        __mapped_11.push({
    let child = __elem_12.1.clone();
    if __elem_12.0.clone() == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("	".to_string(), name.to_string()), child.name.clone()), " ".to_string()), name.to_string()), " = iota".to_string())
} else {
    v2_rt::concat(v2_rt::concat("	".to_string(), name.to_string()), child.name.clone())
}
});
    }
    Rc::new(__mapped_11)
};
    let consts_str = {
    let mut __joined_13 = String::new();
    let mut __first_15 = true;
    for __elem_14 in variant_consts.iter().cloned() {
        if !__first_15 {
    __joined_13.push_str(&"\n".to_string());
};
        __first_15 = false;
        __joined_13.push_str(&__elem_14);
    }
    __joined_13
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(type_decl.clone(), "\n\nconst (\n".to_string()), consts_str.clone()), "\n)".to_string())
}
}

pub fn emit_go_variant_struct(parent_name: &str, child: Rc<Node>) -> String {
    let struct_name = v2_rt::concat(parent_name.to_string(), child.name.clone());
    let marker_method = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func (".to_string(), struct_name.clone()), ") is".to_string()), parent_name.to_string()), "() {}".to_string());
    if ({
    let __len_5 = child.children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), struct_name.clone()), " struct{}\n\n".to_string()), marker_method.clone())
} else {
    let field_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in child.children.iter().cloned() {
        __mapped_0.push(emit_go_struct_field_from_child(__elem_1.clone()));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), struct_name.clone()), " struct {\n".to_string()), fields_str.clone()), "\n}\n\n".to_string()), marker_method.clone())
}
}

pub fn emit_go_type_alias(name: &str, base: Rc<Node>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " = ".to_string()), emit_node_type(base.clone(), RenderTarget::Go))
}

pub fn emit_go_fn_def(name: &str, params: Rc<Vec<Rc<Param>>>, inferred: Rc<Node>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let params_str = emit_go_params(params.clone());
    let ret_str = emit_go_inferred(inferred.clone());
    let body_scope = build_params_scope(scope.clone(), params.clone());
    let use_tco = is_tco_eligible(&name, body.clone(), registry.clone());
    if use_tco {
    let body_str = emit_go_typed_tco_body(body.clone(), &name, params.clone(), registry.clone(), body_scope.clone(), 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func ".to_string(), go_export_ident(&name)), "(".to_string()), params_str), ")".to_string()), ret_str), " {\n".to_string()), body_str), "\n}".to_string())
} else {
    let body_str = emit_go_typed_expr(body.clone(), registry.clone(), body_scope.clone(), 0_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func ".to_string(), go_export_ident(&name)), "(".to_string()), params_str), ")".to_string()), ret_str), " {\n".to_string()), make_indent(1_i64)), "return ".to_string()), body_str), "\n}".to_string())
}
}

pub fn emit_go_func_def(name: &str, params: Rc<Vec<Rc<Param>>>, inferred: Rc<Node>, uses: Rc<Vec<Rc<ResourceUse>>>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let service_names = match lookup_item(registry.clone(), &name) {
    Some(info) => {
        info.service_names.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let params_str = emit_go_func_params(params.clone(), uses.clone(), service_names.clone());
    let ret_type = emit_node_type(inferred.clone(), RenderTarget::Go);
    let body_scope = build_params_scope(scope.clone(), params.clone());
    let body_str = emit_go_typed_func_body(body.clone(), registry.clone(), body_scope.clone(), 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func ".to_string(), go_export_ident(&name)), "(".to_string()), params_str), ") (".to_string()), ret_type), ", error) {\n".to_string()), body_str), "\n}".to_string())
}

pub fn emit_go_func_params(params: Rc<Vec<Rc<Param>>>, uses: Rc<Vec<Rc<ResourceUse>>>, service_names: Rc<Vec<String>>) -> String {
    let param_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_go_param(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let resource_strs = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in uses.iter().cloned() {
        __mapped_2.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.name, RenderTarget::Go), " ".to_string()), emit_node_type(__elem_3.resource.clone(), RenderTarget::Go)));
    }
    Rc::new(__mapped_2)
};
    let service_strs = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in service_names.iter().cloned() {
        __mapped_4.push(v2_rt::concat(v2_rt::concat(service_var_name(&__elem_5), " *".to_string()), sanitize_service_name(&__elem_5)));
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

pub fn emit_go_params(params: Rc<Vec<Rc<Param>>>) -> String {
    let strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_go_param(__elem_1.clone()));
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

pub fn emit_go_param(param: Rc<Param>) -> String {
    let ty = emit_node_type(param.type_expr.clone(), RenderTarget::Go);
    v2_rt::concat(v2_rt::concat(emit_ident(&param.name, RenderTarget::Go), " ".to_string()), ty)
}

pub fn emit_go_inferred(inferred: Rc<Node>) -> String {
    let ty = emit_node_type(inferred.clone(), RenderTarget::Go);
    if ty.clone() == "struct{}" {
    "".to_string()
} else {
    v2_rt::concat(" ".to_string(), ty.clone())
}
}

pub fn emit_go_pattern(pattern: Rc<MatchPattern>) -> String {
    match pattern.as_ref() {
    MatchPattern::Bind { name: n, .. } => {
        emit_ident(&n, RenderTarget::Go)
    }
    MatchPattern::LitPattern { value: v, .. } => {
        emit_literal(v.clone(), RenderTarget::Go)
    }
    MatchPattern::VariantPattern { name: n, parent_enum: _, field_bindings: fbs, .. } => {
        emit_go_variant_pattern(&n, fbs.clone())
    }
    MatchPattern::Wildcard => {
        "_".to_string()
    }
}
}

pub fn emit_go_variant_pattern(name: &str, field_bindings: Rc<Vec<Rc<FieldBinding>>>) -> String {
    name.to_string()
}

pub fn emit_go_field_access(base: Rc<Node>, field: &str, summary: Option<Rc<FieldSummary>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_go_typed_expr(base.clone(), registry.clone(), scope.clone(), 0_i64);
        match summary.as_ref().map(|__rc| __rc.as_ref()) {
    Some(fs) => {
        let fs = Rc::new(fs.clone());
        match fs.access_style.clone() {
    FieldAccessStyle::TupleFirst => {
        v2_rt::concat(base_str, ".First".to_string())
    }
    FieldAccessStyle::TupleSecond => {
        v2_rt::concat(base_str, ".Second".to_string())
    }
    _ => {
        v2_rt::concat(v2_rt::concat(base_str, ".".to_string()), go_export_ident(&field))
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(base_str, ".".to_string()), go_export_ident(&field))
    }
}
    })
}

pub fn emit_go_expr_var(expr: Rc<Node>, depth: i64) -> String {
    let prefix = make_indent(depth);
    match expr.expr_data.as_ref() {
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        v2_rt::concat(prefix, emit_ident(&n, RenderTarget::Go))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_var expected ExprVar", RenderTarget::Go))
    }
}
}

pub fn emit_go_expr_field_access(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprFieldAccess { base: b, field: f, summary, .. } => {
        if is_typed_service_call_receiver(expr.clone()) {
    match extract_typed_service_name(expr.clone()) {
    Some(svc_name) => {
        v2_rt::concat(prefix, service_var_name(&svc_name))
    }
    None => {
        v2_rt::concat(prefix, emit_go_field_access(b.clone(), &f, summary.clone(), registry.clone(), scope.clone()))
    }
}
} else {
    v2_rt::concat(prefix, emit_go_field_access(b.clone(), &f, summary.clone(), registry.clone(), scope.clone()))
}
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_field_access expected ExprFieldAccess", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_call(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: a, call_semantics: _, .. } => {
        v2_rt::concat(prefix, emit_go_typed_call(&f, a.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_call expected ExprCall", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_method_call(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprMethodCall { receiver: r, method: m, args: a, method_semantics, .. } => {
        v2_rt::concat(prefix, emit_go_typed_method_call(r.clone(), &m, a.clone(), method_semantics.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_method_call expected ExprMethodCall", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_match(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        emit_go_typed_match(s.clone(), arm_list.clone(), registry.clone(), scope.clone(), depth)
    }
    _ => {
        v2_rt::concat(make_indent(depth), emit_error_expr("emit_go_expr_match expected ExprMatch", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_if(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        emit_go_typed_if(c.clone(), t.clone(), e.clone(), registry.clone(), scope.clone(), depth)
    }
    _ => {
        v2_rt::concat(make_indent(depth), emit_error_expr("emit_go_expr_if expected ExprIf", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_let(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: bd, .. } => {
        emit_go_typed_let(&n, v.clone(), bd.clone(), registry.clone(), scope.clone(), depth)
    }
    _ => {
        v2_rt::concat(make_indent(depth), emit_error_expr("emit_go_expr_let expected ExprLet", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_record_lit(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprRecordLit { type_name: tn, fields: fs, parent_enum: _, .. } => {
        v2_rt::concat(prefix, emit_go_typed_record_lit(tn.clone(), fs.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_record_lit expected ExprRecordLit", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_string_interp(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprStringInterp { parts: ps, .. } => {
        v2_rt::concat(prefix, emit_go_typed_string_interp(ps.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_string_interp expected ExprStringInterp", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_block(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprBlock { stmts: ss, .. } => {
        emit_go_typed_block(ss.clone(), registry.clone(), scope.clone(), depth)
    }
    _ => {
        v2_rt::concat(make_indent(depth), emit_error_expr("emit_go_expr_block expected ExprBlock", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_cast(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprCast { expr: e, target: t, .. } => {
        v2_rt::concat(prefix, emit_go_typed_cast(e.clone(), t.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_cast expected ExprCast", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_for_each(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprForEach { variable: v, collection: c, body: bd, .. } => {
        emit_go_typed_for_each(&v, c.clone(), bd.clone(), registry.clone(), scope.clone(), depth)
    }
    _ => {
        v2_rt::concat(make_indent(depth), emit_error_expr("emit_go_expr_for_each expected ExprForEach", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_index(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprIndex { base: b, index: i, .. } => {
        v2_rt::concat(prefix, emit_go_typed_index(b.clone(), i.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_index expected ExprIndex", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_expr_slice(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match expr.expr_data.as_ref() {
    ExprData::ExprSlice { base: b, start: s, end: e, .. } => {
        v2_rt::concat(prefix, emit_go_typed_slice(b.clone(), s.clone(), e.clone(), registry.clone(), scope.clone()))
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_expr_slice expected ExprSlice", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_typed_expr(texpr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth.clone());
        emit_shared_expr(texpr.clone(), RenderTarget::Go, |result| if result.clone() == "" {
    "".to_string()
} else {
    v2_rt::concat(prefix.clone(), result.clone())
}, |child| emit_go_typed_expr(child.clone(), registry.clone(), scope.clone(), 0_i64), |expr| emit_go_expr_var(expr.clone(), depth.clone()), |expr| emit_go_expr_field_access(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_call(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_method_call(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_match(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_if(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_let(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_record_lit(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_string_interp(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_block(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_cast(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_for_each(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_index(expr.clone(), registry.clone(), scope.clone(), depth.clone()), |expr| emit_go_expr_slice(expr.clone(), registry.clone(), scope.clone(), depth.clone()))
    })
}

pub fn emit_go_typed_call(func: &str, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let ordered_args = order_typed_call_args(args.clone(), &func, scope.clone());
        let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in ordered_args.iter().cloned() {
        __mapped_0.push(emit_go_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), 0_i64));
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
        v2_rt::concat(v2_rt::concat(v2_rt::concat(go_export_ident(&func), "(".to_string()), args_str.clone()), ")".to_string())
    })
}

pub fn emit_go_typed_for_each(variable: &str, collection: Rc<Node>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let coll_str = emit_go_typed_expr(collection.clone(), registry.clone(), scope.clone(), 0_i64);
        let elem_type = for_each_element_type_node(rt_type(collection.clone()));
        let body_scope = extend_scope(scope.clone(), &variable, elem_type.clone());
        let body_str = emit_go_typed_expr(body.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone()), "for _, ".to_string()), emit_ident(&variable, RenderTarget::Go)), " := range ".to_string()), coll_str), " {\n".to_string()), body_str), "\n".to_string()), make_indent(depth.clone())), "}".to_string())
    })
}

pub fn emit_go_typed_index(base: Rc<Node>, index: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_go_typed_expr(base.clone(), registry.clone(), scope.clone(), 0_i64);
        let index_str = emit_go_typed_expr(index.clone(), registry.clone(), scope.clone(), 0_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(base_str, "[".to_string()), index_str), "]".to_string())
    })
}

pub fn emit_go_typed_slice(base: Rc<Node>, start: Rc<Node>, end: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_go_typed_expr(base.clone(), registry.clone(), scope.clone(), 0_i64);
        let start_str = emit_go_typed_expr(start.clone(), registry.clone(), scope.clone(), 0_i64);
        let end_str = emit_go_typed_expr(end.clone(), registry.clone(), scope.clone(), 0_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(base_str, "[".to_string()), start_str), ":".to_string()), end_str), "]".to_string())
    })
}

pub fn emit_go_intrinsic_method_call(intrinsic: IntrinsicMethod, receiver: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let recv_str = emit_go_typed_expr(receiver.clone(), registry.clone(), scope.clone(), 0_i64);
        let first_arg_str = emit_go_typed_first_arg(args.clone(), registry.clone(), scope.clone());
        match intrinsic {
    IntrinsicMethod::MethodCount => {
        v2_rt::concat(v2_rt::concat("len(".to_string(), recv_str.clone()), ")".to_string())
    }
    IntrinsicMethod::MethodJoin => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("strings.Join(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodSplit => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("strings.Split(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodLast => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[len(".to_string()), recv_str.clone()), ")-1]".to_string())
    }
    IntrinsicMethod::MethodFirst => {
        v2_rt::concat(recv_str.clone(), "[0]".to_string())
    }
    IntrinsicMethod::MethodEnumerate => {
        recv_str.clone()
    }
    IntrinsicMethod::MethodChars => {
        v2_rt::concat(v2_rt::concat("strings.Split(".to_string(), recv_str.clone()), ", \"\")".to_string())
    }
    IntrinsicMethod::MethodStringContains => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("strings.Contains(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodConcat => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("append(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), "...)".to_string())
    }
    IntrinsicMethod::MethodMap => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.Map(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodFilter => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.Filter(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodAny => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.Any(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodAll => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.All(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodFlatMap => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.FlatMap(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodSkip => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[".to_string()), first_arg_str), ":]".to_string())
    }
    IntrinsicMethod::MethodTake => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str.clone(), "[:".to_string()), first_arg_str), "]".to_string())
    }
    IntrinsicMethod::MethodFold => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.Fold(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodSortBy => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2rt.SortBy(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodAppend => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("append(".to_string(), recv_str.clone()), ", ".to_string()), first_arg_str), ")".to_string())
    }
}
    })
}

pub fn go_bridge_method_name(method: RuntimeBridgeMethod) -> String {
    let base = bridge_method_base_name(method);
    let parts = {
    let mut __split_parts_0 = Vec::new();
    for __part_1 in base.split("_".to_string().as_str()) {
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

pub fn emit_go_runtime_bridge_method_call(method: RuntimeBridgeMethod, receiver: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let method_name = go_bridge_method_name(method);
        let recv_expr = emit_go_typed_expr(receiver.clone(), registry.clone(), scope.clone(), 0_i64);
        let recv_str = recv_expr;
        let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_go_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), 0_i64));
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
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".".to_string()), method_name), "(".to_string()), args_str.clone()), ")".to_string())
    })
}

pub fn emit_go_plain_method_call(receiver: Rc<Node>, method: &str, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let recv_str = emit_go_typed_expr(receiver.clone(), registry.clone(), scope.clone(), 0_i64);
        let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_go_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), 0_i64));
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
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".".to_string()), go_export_ident(&method)), "(".to_string()), args_str.clone()), ")".to_string())
    })
}

pub fn emit_go_typed_method_call(receiver: Rc<Node>, method: &str, args: Rc<Vec<Rc<NamedArg>>>, method_semantics: Option<Rc<MethodSemantics>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if method_semantics.clone().is_some() {
    match method_semantics.clone().unwrap().as_ref() {
    MethodSemantics::ServiceMethodSemantics { service_name: svc_name, .. } => {
        {
    let var_name = service_var_name(&svc_name);
    let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_go_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), 0_i64));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(var_name, ".".to_string()), go_export_ident(&method)), "(".to_string()), args_str.clone()), ")".to_string())
}
    }
    MethodSemantics::IntrinsicMethodSemantics { intrinsic, fold_accumulator_type: _, .. } => {
        emit_go_intrinsic_method_call(intrinsic.clone(), receiver.clone(), args.clone(), registry.clone(), scope.clone())
    }
    MethodSemantics::RuntimeBridgeSemantics { method: bridge_method, .. } => {
        emit_go_runtime_bridge_method_call(bridge_method.clone(), receiver.clone(), args.clone(), registry.clone(), scope.clone())
    }
    MethodSemantics::PlainMethodSemantics => {
        emit_go_plain_method_call(receiver.clone(), &method, args.clone(), registry.clone(), scope.clone())
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
        __mapped_5.push(emit_go_typed_expr(__elem_6.value.clone(), registry.clone(), scope.clone(), 0_i64));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(var_name, ".".to_string()), go_export_ident(&method)), "(".to_string()), args_str.clone()), ")".to_string())
}
    }
    None => {
        emit_error_expr("unsupported service receiver", RenderTarget::Go)
    }
}
} else {
    emit_go_plain_method_call(receiver.clone(), &method, args.clone(), registry.clone(), scope.clone())
}
}
    })
}

pub fn emit_go_typed_first_arg(args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match args.clone().first().cloned() {
    Some(a) => {
        emit_go_typed_expr(a.value.clone(), registry.clone(), scope.clone(), 0_i64)
    }
    None => {
        "panic(\"missing method argument\")".to_string()
    }
}
    })
}

pub fn emit_go_typed_match(scrutinee: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let scrut_str = emit_go_typed_expr(scrutinee.clone(), registry.clone(), scope.clone(), 0_i64);
        let arm_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in arms.iter().cloned() {
        __mapped_0.push(emit_go_typed_switch_case(__elem_1.clone(), registry.clone(), scope.clone(), depth.clone()));
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
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone()), "switch ".to_string()), scrut_str), " {\n".to_string()), arms_str.clone()), "\n".to_string()), make_indent(depth.clone())), "}".to_string())
    })
}

pub fn emit_go_typed_switch_case(arm: Rc<MatchArm>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let pat_str = emit_go_pattern(arm.pattern.clone());
        let body_str = emit_go_typed_expr(arm.body.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64);
        let case_keyword = if pat_str.clone() == "_" {
    "default".to_string()
} else {
    v2_rt::concat("case ".to_string(), pat_str.clone())
};
        v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone()), case_keyword.clone()), ":\n".to_string()), body_str)
    })
}

pub fn emit_go_typed_if(condition: Rc<Node>, then_branch: Rc<Node>, else_branch: Option<Rc<Node>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth.clone());
        let cond_str = emit_go_typed_expr(condition.clone(), registry.clone(), scope.clone(), 0_i64);
        match else_branch.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        {
    let then_str = emit_go_typed_expr(then_branch.clone(), registry.clone(), scope.clone(), 0_i64);
    let else_str = emit_go_typed_expr(eb.clone(), registry.clone(), scope.clone(), 0_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "func() interface{} { if ".to_string()), cond_str), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), "return ".to_string()), then_str), "\n".to_string()), prefix.clone()), "} else {\n".to_string()), make_indent(depth.clone() + 1_i64)), "return ".to_string()), else_str), "\n".to_string()), prefix.clone()), "} }()".to_string())
}
    }
    None => {
        {
    let then_str = emit_go_typed_expr(then_branch.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "if ".to_string()), cond_str), " {\n".to_string()), then_str), "\n".to_string()), prefix.clone()), "}".to_string())
}
    }
}
    })
}

pub fn emit_go_typed_let(name: &str, value: Rc<Node>, body: Option<Rc<Node>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let val_str = emit_go_typed_expr(value.clone(), registry.clone(), scope.clone(), 0_i64);
        let let_line = v2_rt::concat(make_indent(depth), emit_let_binding(&name, &val_str, RenderTarget::Go));
        match body.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        {
    let next_scope = extend_scope(scope.clone(), &name, rt_type(value.clone()));
    v2_rt::concat(v2_rt::concat(let_line.clone(), "\n".to_string()), emit_go_typed_expr(bd.clone(), registry.clone(), next_scope.clone(), depth))
}
    }
    None => {
        let_line.clone()
    }
}
    })
}

pub fn emit_go_typed_record_lit(type_name: Option<String>, fields: Rc<Vec<Rc<FieldInit>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match type_name {
    None => {
        if ({
    let __len_5 = fields.clone().len();
    __len_5 as i64
}) == 0_i64 {
    "map[string]interface{}{}".to_string()
} else {
    let field_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in fields.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("	\"".to_string(), __elem_1.name.clone()), "\": ".to_string()), emit_go_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), 0_i64)), ",".to_string()));
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
    v2_rt::concat(v2_rt::concat("map[string]interface{}{\n".to_string(), fields_str.clone()), "\n}".to_string())
}
    }
    Some(tn) => {
        if ({
    let __len_11 = fields.clone().len();
    __len_11 as i64
}) == 0_i64 {
    v2_rt::concat(tn, "{}".to_string())
} else {
    let field_strs = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in fields.iter().cloned() {
        __mapped_6.push(v2_rt::concat(v2_rt::concat(go_export_ident(&__elem_7.name), ": ".to_string()), emit_go_typed_expr(__elem_7.value.clone(), registry.clone(), scope.clone(), 0_i64)));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(tn, "{".to_string()), fields_str.clone()), "}".to_string())
}
    }
}
    })
}

pub fn emit_go_typed_bin_op(op: BinOpKind, left: Rc<Node>, right: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let l_str = emit_go_typed_expr(left.clone(), registry.clone(), scope.clone(), 0_i64);
    let r_str = emit_go_typed_expr(right.clone(), registry.clone(), scope.clone(), 0_i64);
    if is_null_coalesce(op.clone()) {
    emit_null_coalesce(&l_str, &r_str, RenderTarget::Go)
} else {
    let op_str = emit_bin_op_symbol(op.clone(), RenderTarget::Go);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), l_str), " ".to_string()), op_str), " ".to_string()), r_str), ")".to_string())
}
}

pub fn emit_go_typed_string_interp(parts: Rc<Vec<Rc<StringPart>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let fmt_parts = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in parts.iter().cloned() {
        __mapped_0.push(go_typed_interp_segment(__elem_1.clone(), registry.clone(), scope.clone()));
    }
    Rc::new(__mapped_0)
};
        let fmt_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in ({
    let mut __mapped_2 = Vec::new();
    for __elem_3 in fmt_parts.iter().cloned() {
        __mapped_2.push(__elem_3.format_segment.clone());
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
};
        let args = {
    let mut __filtered_9 = Vec::new();
    for __elem_10 in ({
    let mut __mapped_7 = Vec::new();
    for __elem_8 in fmt_parts.iter().cloned() {
        __mapped_7.push(__elem_8.arg_expr.clone());
    }
    Rc::new(__mapped_7)
}).iter().cloned() {
        if __elem_10.clone() != "" {
    __filtered_9.push(__elem_10);
};
    }
    Rc::new(__filtered_9)
};
        if ({
    let __len_14 = args.clone().len();
    __len_14 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat("\"".to_string(), fmt_str.clone()), "\"".to_string())
} else {
    let args_str = {
    let mut __joined_11 = String::new();
    let mut __first_13 = true;
    for __elem_12 in args.iter().cloned() {
        if !__first_13 {
    __joined_11.push_str(&", ".to_string());
};
        __first_13 = false;
        __joined_11.push_str(&__elem_12);
    }
    __joined_11
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("fmt.Sprintf(\"".to_string(), fmt_str.clone()), "\", ".to_string()), args_str.clone()), ")".to_string())
}
    })
}

pub fn go_typed_interp_segment(part: Rc<StringPart>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> Rc<InterpPart> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match part.as_ref() {
    StringPart::Text { value: v, .. } => {
        Rc::new(InterpPart { format_segment: v.clone(), arg_expr: "".to_string() })
    }
    StringPart::Interpolation { expr: e, .. } => {
        Rc::new(InterpPart { format_segment: "%v".to_string(), arg_expr: emit_go_typed_expr(e.clone(), registry.clone(), scope.clone(), 0_i64) })
    }
}
    })
}

pub fn emit_go_typed_block(stmts: Rc<Vec<Rc<Node>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let state = emit_go_block_stmts(stmts.clone(), Rc::new(Vec::new()), scope.clone(), registry.clone(), depth);
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

pub fn emit_go_typed_cast(expr: Rc<Node>, target: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let expr_str = emit_go_typed_expr(expr.clone(), registry.clone(), scope.clone(), 0_i64);
        let ty_str = emit_node_type(target.clone(), RenderTarget::Go);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(ty_str, "(".to_string()), expr_str), ")".to_string())
    })
}

pub fn emit_go_typed_func_body(body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(depth);
        match body.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: inner, .. } => {
        {
    let val_str = emit_go_typed_expr(v.clone(), registry.clone(), scope.clone(), 0_i64);
    let let_line = v2_rt::concat(prefix, emit_let_binding(&n, &val_str, RenderTarget::Go));
    let next_scope = extend_scope(scope.clone(), &n, rt_type(v.clone()));
    match inner.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        v2_rt::concat(v2_rt::concat(let_line.clone(), "\n".to_string()), emit_go_typed_func_body(bd.clone(), registry.clone(), next_scope.clone(), depth))
    }
    None => {
        let_line.clone()
    }
}
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        if ({
    let __len_7 = ss.clone().len();
    __len_7 as i64
}) == 0_i64 {
    v2_rt::concat(prefix, "return struct{}{}, nil".to_string())
} else {
    let init_state = emit_go_init_block_stmts(ss.clone(), Rc::new(Vec::new()), scope.clone(), registry.clone(), depth);
    let last_stmt = ss.clone().last().cloned();
    let last_str = match last_stmt.clone() {
    Some(s) => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix, "return ".to_string()), emit_go_typed_expr(s.clone(), registry.clone(), init_state.scope.clone(), 0_i64)), ", nil".to_string())
    }
    None => {
        v2_rt::concat(prefix, "return struct{}{}, nil".to_string())
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
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix, "return ".to_string()), emit_go_typed_expr(body.clone(), registry.clone(), scope.clone(), 0_i64)), ", nil".to_string())
    }
}
    })
}

pub fn emit_go_typed_tco_body(texpr: Rc<Node>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    let inner = emit_go_typed_tco_expr(texpr.clone(), &fn_name, params.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone()), "for {\n".to_string()), inner), "\n".to_string()), make_indent(depth.clone())), "}".to_string())
}

pub fn emit_go_tco_non_self_call(frame: Rc<TcoFrame>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let prefix = make_indent(frame.depth.clone());
    match frame.expr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: a, call_semantics: _, .. } => {
        {
    let call_str = emit_go_typed_call(&f, a.clone(), registry.clone(), frame.scope.clone());
    v2_rt::concat(v2_rt::concat(prefix, "return ".to_string()), call_str)
}
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_tco_non_self_call expected ExprCall", RenderTarget::Go))
    }
}
}

pub fn emit_go_tco_if(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(frame.depth.clone());
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        {
    let cond_str = emit_go_typed_expr(c.clone(), registry.clone(), frame.scope.clone(), 0_i64);
    let then_str = emit_go_typed_tco_expr(t.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone() + 1_i64);
    match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        {
    let else_str = emit_go_typed_tco_expr(eb.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone() + 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "if ".to_string()), cond_str), " {\n".to_string()), then_str), "\n".to_string()), prefix.clone()), "} else {\n".to_string()), else_str), "\n".to_string()), prefix.clone()), "}".to_string())
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "if ".to_string()), cond_str), " {\n".to_string()), then_str), "\n".to_string()), prefix.clone()), "}".to_string())
    }
}
}
    }
    _ => {
        v2_rt::concat(prefix.clone(), emit_error_expr("emit_go_tco_if expected ExprIf", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_tco_match(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(frame.depth.clone());
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        {
    let scrut_str = emit_go_typed_expr(s.clone(), registry.clone(), frame.scope.clone(), 0_i64);
    let arm_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in arm_list.iter().cloned() {
        __mapped_0.push(emit_go_typed_tco_switch_case(__elem_1.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone()));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "switch ".to_string()), scrut_str), " {\n".to_string()), arms_str.clone()), "\n".to_string()), prefix.clone()), "}".to_string())
}
    }
    _ => {
        v2_rt::concat(prefix.clone(), emit_error_expr("emit_go_tco_match expected ExprMatch", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_tco_let(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(frame.depth.clone());
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: bd, .. } => {
        {
    let val_str = emit_go_typed_expr(v.clone(), registry.clone(), frame.scope.clone(), 0_i64);
    let let_line = v2_rt::concat(prefix, emit_let_binding(&n, &val_str, RenderTarget::Go));
    let next_scope = extend_scope(frame.scope.clone(), &n, rt_type(v.clone()));
    match bd.as_ref().map(|__rc| __rc.as_ref()) {
    Some(b) => {
        let b = Rc::new(b.clone());
        v2_rt::concat(v2_rt::concat(let_line.clone(), "\n".to_string()), emit_go_typed_tco_expr(b.clone(), &fn_name, params.clone(), registry.clone(), next_scope.clone(), frame.depth.clone()))
    }
    None => {
        let_line.clone()
    }
}
}
    }
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_tco_let expected ExprLet", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_tco_block(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let prefix = make_indent(frame.depth.clone());
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprBlock { stmts: ss, .. } => {
        if ({
    let __len_7 = ss.clone().len();
    __len_7 as i64
}) == 0_i64 {
    v2_rt::concat(prefix, "return".to_string())
} else {
    let init_state = emit_go_init_block_stmts(ss.clone(), Rc::new(Vec::new()), frame.scope.clone(), registry.clone(), frame.depth.clone());
    let last_str = match ss.clone().last().cloned() {
    Some(last_expr) => {
        emit_go_typed_tco_expr(last_expr.clone(), &fn_name, params.clone(), registry.clone(), init_state.scope.clone(), frame.depth.clone())
    }
    None => {
        v2_rt::concat(prefix, "return".to_string())
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
    _ => {
        v2_rt::concat(prefix, emit_error_expr("emit_go_tco_block expected ExprBlock", RenderTarget::Go))
    }
}
    })
}

pub fn emit_go_tco_default_return(frame: Rc<TcoFrame>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let prefix = make_indent(frame.depth.clone());
    let val_str = emit_go_typed_expr(frame.expr.clone(), registry.clone(), frame.scope.clone(), 0_i64);
    v2_rt::concat(v2_rt::concat(prefix, "return ".to_string()), val_str)
}

pub fn emit_go_typed_tco_expr(texpr: Rc<Node>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        emit_shared_tco_expr(Rc::new(TcoFrame { expr: texpr.clone(), scope: scope.clone(), depth }), &fn_name, |input| emit_go_typed_tco_reassign(input.args.clone(), params.clone(), registry.clone(), input.scope.clone(), input.depth.clone()), |frame| emit_go_tco_non_self_call(frame.clone(), registry.clone()), |frame| emit_go_tco_if(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_go_tco_match(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_go_tco_let(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_go_tco_block(frame.clone(), &fn_name, params.clone(), registry.clone()), |frame| emit_go_tco_default_return(frame.clone(), registry.clone()))
    })
}

pub fn emit_go_typed_tco_switch_case(arm: Rc<MatchArm>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let pat_str = emit_go_pattern(arm.pattern.clone());
        let body_str = emit_go_typed_tco_expr(arm.body.clone(), &fn_name, params.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64);
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth.clone()), "case ".to_string()), pat_str), ":\n".to_string()), body_str)
    })
}

pub fn emit_go_typed_tco_reassign(args: Rc<Vec<Rc<NamedArg>>>, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64) -> String {
    let ordered_args = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_go_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), 0_i64));
    }
    Rc::new(__mapped_0)
};
    let param_names = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in params.iter().cloned() {
        __mapped_2.push(emit_ident(&__elem_3.name, RenderTarget::Go));
    }
    Rc::new(__mapped_2)
};
    let all_lines = tco_reassign_core(ordered_args.clone(), param_names.clone(), "tco", "", " := ", "", "continue", &make_indent(depth));
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

pub fn emit_go_service_def(item: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let safe_name = sanitize_service_name(&item.name);
    let transport = service_fallback_transport(item.clone());
    let struct_def = emit_go_service_struct(&safe_name, transport.clone());
    let op_children = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in item.children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.params.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    let methods = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in op_children.iter().cloned() {
        __mapped_3.push(emit_go_operation_method(&safe_name, transport.clone(), __elem_4.clone(), registry.clone()));
    }
    Rc::new(__mapped_3)
};
    let methods_str = {
    let mut __joined_5 = String::new();
    let mut __first_7 = true;
    for __elem_6 in methods.iter().cloned() {
        if !__first_7 {
    __joined_5.push_str(&"\n\n".to_string());
};
        __first_7 = false;
        __joined_5.push_str(&__elem_6);
    }
    __joined_5
};
    v2_rt::concat(v2_rt::concat(struct_def, "\n\n".to_string()), methods_str.clone())
}

pub fn emit_go_service_struct(name: &str, transport: Rc<Node>) -> String {
    let fields = if is_transport_kind(transport.clone(), Rc::new(TransportKind::RestTransport)) {
    let base = "	BaseURL string".to_string();
    let auth_field = if transport_has_auth(transport.clone()) {
    "\n	AuthToken string".to_string()
} else {
    "".to_string()
};
    v2_rt::concat(base.clone(), auth_field.clone())
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::ShellTransport)) {
    "	WorkingDir string".to_string()
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::FileTransport)) {
    "	BasePath string".to_string()
} else {
    "".to_string()
}
}
};
    if fields.clone() == "" {
    v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " struct{}".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), name.to_string()), " struct {\n".to_string()), fields.clone()), "\n}".to_string())
}
}

pub fn emit_go_operation_method(service_name: &str, transport: Rc<Node>, op_node: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let input_params = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in op_node.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.name, RenderTarget::Go), " ".to_string()), emit_node_type(__elem_1.type_expr.clone(), RenderTarget::Go)));
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
    let receiver = v2_rt::concat(v2_rt::concat("(c *".to_string(), service_name.to_string()), ")".to_string());
    let ret_type = emit_node_type(rt_type(op_node.clone()), RenderTarget::Go);
    let eff_transport = effective_operation_transport(op_node.clone(), transport.clone());
    let body = emit_go_transport_call(eff_transport.clone(), &op_node.name, registry.clone(), 1_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("func ".to_string(), receiver.clone()), " ".to_string()), go_export_ident(&op_node.name)), "(".to_string()), params_str.clone()), ") (".to_string()), ret_type), ", error) {\n".to_string()), body), "\n}".to_string())
}

pub fn emit_go_transport_call(transport: Rc<Node>, op_name: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> String {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::RestTransport)) {
    emit_go_rest_call(&op_name, transport.clone(), depth)
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::ShellTransport)) {
    emit_go_shell_call(&op_name, transport.clone(), depth)
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::FileTransport)) {
    emit_go_file_call(&op_name, depth)
} else {
    emit_go_local_call(&op_name, depth)
}
}
}
}

pub fn emit_go_rest_call(op_name: &str, transport: Rc<Node>, depth: i64) -> String {
    let prefix = make_indent(depth);
    let url_line = v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "url := fmt.Sprintf(\"%s/".to_string()), to_snake(&op_name)), "\", c.BaseURL)".to_string());
    let body_line = v2_rt::concat(prefix.clone(), "reqBody := bytes.NewBuffer(nil)".to_string());
    let req_line = v2_rt::concat(v2_rt::concat(prefix.clone(), "req, err := http.NewRequest(\"POST\", url, reqBody)\n".to_string()), "if err != nil {\n	return nil, fmt.Errorf(\"creating request: %w\", err)\n}".to_string());
    let auth_line = if transport_has_auth(transport.clone()) {
    let header_name = match transport_auth_header_name(transport.clone()) {
    Some(h) => {
        h
    }
    None => {
        "Authorization".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "req.Header.Set(\"".to_string()), header_name.clone()), "\", c.AuthToken)".to_string())
} else {
    "".to_string()
};
    let hdrs = transport_headers(transport.clone());
    let header_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in hdrs.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "panic(\"header value emission not implemented for '".to_string()), __elem_1.name.clone()), "'\")".to_string()));
    }
    Rc::new(__mapped_0)
};
    let send_lines = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "resp, err := http.DefaultClient.Do(req)\n".to_string()), "if err != nil {\n	return nil, fmt.Errorf(\"sending request: %w\", err)\n}\n".to_string()), "defer resp.Body.Close()\n".to_string()), "body, err := io.ReadAll(resp.Body)\n".to_string()), "if err != nil {\n	return nil, fmt.Errorf(\"reading response: %w\", err)\n}\n".to_string()), "var result interface{}\n".to_string()), "if err := json.Unmarshal(body, &result); err != nil {\n".to_string()), "	return nil, fmt.Errorf(\"decoding response: %w\", err)\n}\n".to_string()), "return result, nil".to_string());
    let all_lines = v2_rt::concat(v2_rt::concat(v2_rt::concat(Rc::new(vec!(url_line.clone(), body_line.clone(), req_line.clone())), if auth_line.clone() == "" {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(auth_line.clone()))
}), header_lines.clone()), Rc::new(vec!(send_lines.clone())));
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in all_lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
}

pub fn emit_go_shell_call(op_name: &str, transport: Rc<Node>, depth: i64) -> String {
    let prefix = make_indent(depth);
    let cmd_line = v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "cmd := exec.Command(\"".to_string()), to_snake(&op_name)), "\")".to_string());
    let dir_line = v2_rt::concat(prefix.clone(), "cmd.Dir = c.WorkingDir".to_string());
    let envs = transport_env(transport.clone());
    let env_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in envs.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "panic(\"env value emission not implemented for '".to_string()), __elem_1.name.clone()), "'\")".to_string()));
    }
    Rc::new(__mapped_0)
};
    let run_lines = v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix.clone(), "output, err := cmd.Output()\n".to_string()), "if err != nil {\n	return \"\", fmt.Errorf(\"running command: %w\", err)\n}\n".to_string()), "return string(output), nil".to_string());
    let all_lines = v2_rt::concat(v2_rt::concat(Rc::new(vec!(cmd_line.clone(), dir_line.clone())), env_lines.clone()), Rc::new(vec!(run_lines.clone())));
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in all_lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
}

pub fn emit_go_file_call(op_name: &str, depth: i64) -> String {
    let prefix = make_indent(depth);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix, "path := fmt.Sprintf(\"%s/".to_string()), to_snake(&op_name)), "\", c.BasePath)\n".to_string()), "data, err := os.ReadFile(path)\n".to_string()), "if err != nil {\n	return \"\", fmt.Errorf(\"reading file: %w\", err)\n}\n".to_string()), "return string(data), nil".to_string())
}

pub fn emit_go_local_call(op_name: &str, depth: i64) -> String {
    let prefix = make_indent(depth);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(prefix, "// Local binding -- direct function call\n".to_string()), "return ".to_string()), go_export_ident(&op_name)), "(), nil".to_string())
}

pub fn emit_go_resource_def(item: Rc<Node>) -> String {
    let cap_children = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in item.children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.params.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    let methods = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in cap_children.iter().cloned() {
        __mapped_3.push(emit_go_capability_method(__elem_4.clone(), 1_i64));
    }
    Rc::new(__mapped_3)
};
    let methods_str = {
    let mut __joined_5 = String::new();
    let mut __first_7 = true;
    for __elem_6 in methods.iter().cloned() {
        if !__first_7 {
    __joined_5.push_str(&"\n".to_string());
};
        __first_7 = false;
        __joined_5.push_str(&__elem_6);
    }
    __joined_5
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("type ".to_string(), item.name.clone()), " interface {\n".to_string()), methods_str.clone()), "\n}".to_string())
}

pub fn emit_go_capability_method(cap_node: Rc<Node>, depth: i64) -> String {
    let input_params = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in cap_node.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.name, RenderTarget::Go), " ".to_string()), emit_node_type(__elem_1.type_expr.clone(), RenderTarget::Go)));
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
    let ret = emit_node_type(rt_type(cap_node.clone()), RenderTarget::Go);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(make_indent(depth), go_export_ident(&cap_node.name)), "(".to_string()), params_str.clone()), ") (".to_string()), ret), ", error)".to_string())
}

pub fn emit_go_data_def(name: &str, type_node: Rc<Node>, value: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>) -> String {
    let ty_str = emit_node_type(type_node.clone(), RenderTarget::Go);
    let val_str = emit_go_typed_expr(value.clone(), registry.clone(), scope.clone(), 0_i64);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("var ".to_string(), go_export_ident(&name)), " ".to_string()), ty_str), " = ".to_string()), val_str)
}

pub fn go_export_ident(name: &str) -> String {
    let parts = {
    let mut __split_parts_0 = Vec::new();
    for __part_1 in name.to_string().split("_".to_string().as_str()) {
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
    let result = {
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
};
    {
let __cond = {
    let mut __any_7 = false;
    for __elem_8 in Rc::new(GO_RESERVED.iter().map(|s| s.to_string()).collect::<Vec<_>>()).iter().cloned() {
        if __elem_8.clone() == result.clone() {
    __any_7 = true;
    break;
};
    }
    __any_7
};
if __cond {
    v2_rt::concat(result.clone(), GO_RESERVED_ESCAPE_SUFFIX.to_string())
} else {
    result.clone()
}
}
}

