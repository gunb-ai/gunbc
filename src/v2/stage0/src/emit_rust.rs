use crate::v2_core::*;
use crate::artifact::*;
use crate::rust_emit::*;
use crate::languages::*;
use crate::runtime_rust::*;
use crate::infer_env::*;
use crate::infer_types::*;
use crate::infer_sigs::*;
use crate::infer_items::*;
use crate::infer_service::*;
use crate::infer_lookup::*;
use crate::infer::*;
use crate::infer_emit_info::*;
use crate::emit::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn rust_source_root() -> String {
    match scaffold_for_target(RenderTarget::Rust).source_dir.clone() {
    Some(dir) => {
        dir
    }
    None => {
        "".to_string()
    }
}
}

pub fn rust_source_ext() -> String {
    scaffold_for_target(RenderTarget::Rust).source_file_extension.clone()
}

pub fn rust_visibility_prefix() -> String {
    top_level_visibility_for_target(RenderTarget::Rust)
}

pub fn rust_struct_derives_text() -> String {
    match serialization_for_target(RenderTarget::Rust).struct_derives.clone() {
    Some(derives) => {
        derives
    }
    None => {
        "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]".to_string()
    }
}
}

pub fn rust_struct_derives_copy_text() -> String {
    match serialization_for_target(RenderTarget::Rust).struct_derives_copy.clone() {
    Some(derives) => {
        derives
    }
    None => {
        "#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]".to_string()
    }
}
}

pub fn rust_enum_derives_text() -> String {
    match serialization_for_target(RenderTarget::Rust).enum_derives.clone() {
    Some(derives) => {
        derives
    }
    None => {
        "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]".to_string()
    }
}
}

pub fn rust_enum_derives_copy_text() -> String {
    match serialization_for_target(RenderTarget::Rust).enum_derives_copy.clone() {
    Some(derives) => {
        derives
    }
    None => {
        "#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]".to_string()
    }
}
}

pub fn rust_serde_tag_attr() -> String {
    match serialization_for_target(RenderTarget::Rust).tag_attribute.clone() {
    Some(attr) => {
        attr
    }
    None => {
        "#[serde(tag = \"_variant\")]".to_string()
    }
}
}

pub fn rust_serde_rename_template_text() -> String {
    match serialization_for_target(RenderTarget::Rust).rename_attribute_template.clone() {
    Some(template) => {
        template
    }
    None => {
        "#[serde(rename = \"{0}\")]".to_string()
    }
}
}

pub fn rust_test_file_path(module_name: &str) -> String {
    let conventions = test_conventions_for_target(RenderTarget::Rust);
    let file_dir = match conventions.file_dir.clone() {
    Some(dir) => {
        dir
    }
    None => {
        "".to_string()
    }
};
    let filename = module_to_filename(&module_name);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(file_dir.clone(), conventions.file_prefix.clone()), filename), conventions.file_suffix.clone()), rust_source_ext())
}

pub fn rust_test_name(projection: Rc<TestProjection>) -> String {
    test_function_name(projection.clone(), RenderTarget::Rust)
}

pub fn rust_async_test_decorator() -> String {
    match test_conventions_for_target(RenderTarget::Rust).async_decorator.clone() {
    Some(decorator) => {
        decorator
    }
    None => {
        "#[tokio::test]".to_string()
    }
}
}

pub fn emit_rust_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> Rc<BlockEmitState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_text = text;
        let mut __tco_p_scope = scope;
        let mut __tco_p_registry = registry;
        let mut __tco_p_depth = depth;
        let mut __tco_p_vtoe = vtoe;
        let mut __tco_p_rc_types = rc_types;
        let mut __tco_p_emit_info = emit_info;
        loop {
            let remaining = __tco_p_remaining;
            let text = __tco_p_text;
            let scope = __tco_p_scope;
            let registry = __tco_p_registry;
            let depth = __tco_p_depth;
            let vtoe = __tco_p_vtoe;
            let rc_types = __tco_p_rc_types;
            let emit_info = __tco_p_emit_info;
            match remaining.clone().first().cloned() {
    None => {
        break Rc::new(BlockEmitState { text: text.clone(), scope: scope.clone() });
    }
    Some(stmt) => {
        {
    let line = emit_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
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
        let __tco_5 = vtoe.clone();
        let __tco_6 = rc_types.clone();
        let __tco_7 = emit_info.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_text = __tco_1;
        __tco_p_scope = __tco_2;
        __tco_p_registry = __tco_3;
        __tco_p_depth = __tco_4;
        __tco_p_vtoe = __tco_5;
        __tco_p_rc_types = __tco_6;
        __tco_p_emit_info = __tco_7;
        continue;
    }

};
    }
};
        }
    })
}

pub fn emit_rust_init_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> Rc<BlockEmitState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_text = text;
        let mut __tco_p_scope = scope;
        let mut __tco_p_registry = registry;
        let mut __tco_p_depth = depth;
        let mut __tco_p_vtoe = vtoe;
        let mut __tco_p_rc_types = rc_types;
        let mut __tco_p_emit_info = emit_info;
        loop {
            let remaining = __tco_p_remaining;
            let text = __tco_p_text;
            let scope = __tco_p_scope;
            let registry = __tco_p_registry;
            let depth = __tco_p_depth;
            let vtoe = __tco_p_vtoe;
            let rc_types = __tco_p_rc_types;
            let emit_info = __tco_p_emit_info;
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
    let line = emit_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
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
        let __tco_5 = vtoe.clone();
        let __tco_6 = rc_types.clone();
        let __tco_7 = emit_info.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_text = __tco_1;
        __tco_p_scope = __tco_2;
        __tco_p_registry = __tco_3;
        __tco_p_depth = __tco_4;
        __tco_p_vtoe = __tco_5;
        __tco_p_rc_types = __tco_6;
        __tco_p_emit_info = __tco_7;
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

pub fn has_complex_variants(item: Rc<Node>) -> bool {
    if ({
    let __len_2 = item.children.clone().len();
    __len_2 as i64
}) == 0_i64 {
    false
} else {
    {
    let mut __any_0 = false;
    for __elem_1 in item.children.iter().cloned() {
        if node_has_structure(__elem_1.clone()) {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}
}

pub fn type_summary_needs_rc(summary: Rc<TypeSummary>) -> bool {
    match summary.repr.as_ref() {
    TypeRepr::StructRepr => {
        true
    }
    TypeRepr::EnumRepr { unit_only, .. } => {
        unit_only.clone() == false
    }
}
}

pub fn is_simple_disj(item: Rc<Node>) -> bool {
    let complex = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in item.children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.children.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    ({
    let __len_3 = complex.clone().len();
    __len_3 as i64
}) == 0_i64
}

pub fn emit_rust(typed: Rc<ResolvedGraph>) -> Rc<EmitResult> {
    let emit_info = build_emit_graph_info(typed.modules.clone());
    let vtoe = emit_info.variant_to_enum.clone();
    let rc_types = {
    let mut __acc_4 = Rc::new(std::collections::HashMap::new());
    for __elem_5 in ({
    let __rc_0 = emit_info.type_summaries.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = if type_summary_needs_rc(__elem_5.clone()) {
    {
    let __rc_7 = __acc_4;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(__elem_5.name.clone(), true);
    Rc::new(__map_ins_6)
}
} else {
    __acc_4.clone()
};
    }
    __acc_4
};
    let registry = typed.item_registry.clone();
    let workflow_funcs = collect_workflow_funcs(typed.modules.clone(), registry.clone());
    let workflow_default_diags = validate_workflow_param_defaults(workflow_funcs.clone());
    if ({
    let __len_8 = workflow_default_diags.clone().len();
    __len_8 as i64
}) > 0_i64 {
    return Rc::new(EmitResult { files: Rc::new(Vec::new()), diagnostics: workflow_default_diags.clone() });
};
    let svc_module_map = {
    let mut __acc_9 = Rc::new(std::collections::HashMap::new());
    for __elem_10 in typed.modules.iter().cloned() {
        __acc_9 = {
    let svc_items = {
    let mut __filtered_11 = Vec::new();
    for __elem_12 in __elem_10.items.iter().cloned() {
        if is_service_item(__elem_12.clone()) {
    __filtered_11.push(__elem_12);
};
    }
    Rc::new(__filtered_11)
};
    let mod_filename = module_to_filename(&__elem_10.module.name);
    {
    let mut __acc_13 = __acc_9.clone();
    for __elem_14 in svc_items.iter().cloned() {
        __acc_13 = {
    let __rc_16 = __acc_13;
    let mut __map_ins_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_15.insert(__elem_14.name.clone(), mod_filename.clone());
    Rc::new(__map_ins_15)
};
    }
    __acc_13
}
};
    }
    __acc_9
};
    let test_projections = extract_test_projections(typed.clone());
    let module_files = {
    let mut __mapped_17 = Vec::new();
    for __elem_18 in typed.modules.iter().cloned() {
        __mapped_17.push(emit_module_full(__elem_18.clone(), registry.clone(), emit_info.clone(), vtoe.clone(), rc_types.clone(), svc_module_map.clone()));
    }
    Rc::new(__mapped_17)
};
    let test_files = {
    let mut __filtered_23 = Vec::new();
    for __elem_24 in ({
    let mut __mapped_19 = Vec::new();
    for __elem_20 in typed.modules.iter().cloned() {
        __mapped_19.push(emit_test_file(&__elem_20.module.name, {
    let mut __filtered_21 = Vec::new();
    for __elem_22 in test_projections.iter().cloned() {
        if __elem_22.module_name.clone() == __elem_20.module.name.clone() {
    __filtered_21.push(__elem_22);
};
    }
    Rc::new(__filtered_21)
}));
    }
    Rc::new(__mapped_19)
}).iter().cloned() {
        if __elem_24.content.clone() != "" {
    __filtered_23.push(__elem_24);
};
    }
    Rc::new(__filtered_23)
};
    let has_services = has_service_items(typed.clone());
    let lib_file = emit_lib_rs(typed.modules.clone(), has_services.clone());
    let cargo = emit_cargo_toml("v2_compiled", has_services.clone());
    let main_file = emit_main_rs(workflow_funcs.clone(), typed.modules.clone(), has_services.clone());
    let rt_file = emit_v2_rt_module();
    let dry_run_file = if has_services.clone() {
    Rc::new(vec!(emit_dry_run_module()))
} else {
    Rc::new(Vec::new())
};
    let files = v2_rt::concat(v2_rt::concat(v2_rt::concat(Rc::new(vec!(cargo.clone(), lib_file.clone(), main_file.clone(), rt_file.clone())), dry_run_file.clone()), module_files.clone()), test_files.clone());
    Rc::new(EmitResult { files: files.clone(), diagnostics: Rc::new(Vec::new()) })
}

pub fn emit_v2_rt_module() -> Rc<TextFile> {
    Rc::new(TextFile { path: v2_rt::concat(v2_rt::concat(rust_source_root(), "v2_rt".to_string()), rust_source_ext()), content: rust_runtime_source() })
}

pub fn emit_lib_rs(modules: Rc<Vec<Rc<TypedModule>>>, has_services: bool) -> Rc<TextFile> {
    let mod_decls = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in modules.iter().cloned() {
        __mapped_0.push({
    let raw_name = module_to_filename(&__elem_1.module.name);
    let mod_name = if raw_name.clone() == "main" {
    "main_mod".to_string()
} else {
    raw_name.clone()
};
    v2_rt::concat(v2_rt::concat("pub mod ".to_string(), mod_name.clone()), ";".to_string())
});
    }
    Rc::new(__mapped_0)
};
    let dry_run_mod = if has_services {
    "\npub mod dry_run;".to_string()
} else {
    "".to_string()
};
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Generated by v2 compiler -- do not edit.\n\n".to_string(), "#![allow(unused_imports, unused_variables, unused_mut, dead_code, unreachable_patterns)]\n\n".to_string()), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in mod_decls.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), "\npub mod v2_rt;".to_string()), dry_run_mod.clone());
    Rc::new(TextFile { path: v2_rt::concat(v2_rt::concat(rust_source_root(), "lib".to_string()), rust_source_ext()), content: v2_rt::concat(content.clone(), "\n".to_string()) })
}

pub fn emit_module(typed_module: Rc<TypedModule>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> Rc<TextFile> {
    let emit_info = build_emit_graph_info(Rc::new(vec!(typed_module.clone())));
    let rc_types = {
    let mut __acc_4 = Rc::new(std::collections::HashMap::new());
    for __elem_5 in ({
    let __rc_0 = emit_info.type_summaries.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = if type_summary_needs_rc(__elem_5.clone()) {
    {
    let __rc_7 = __acc_4;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(__elem_5.name.clone(), true);
    Rc::new(__map_ins_6)
}
} else {
    __acc_4.clone()
};
    }
    __acc_4
};
    emit_module_full(typed_module.clone(), registry.clone(), emit_info.clone(), emit_info.variant_to_enum.clone(), rc_types.clone(), Rc::new(std::collections::HashMap::new()))
}

pub fn emit_module_full(typed_module: Rc<TypedModule>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, emit_info: Rc<EmitGraphInfo>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, svc_module_map: Rc<HashMap<String, String>>) -> Rc<TextFile> {
    let m = typed_module.module.clone();
    let __all_imported = {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in m.imports.iter().cloned() {
        __flat_mapped_0.extend((match __elem_1.names.as_ref() {
    ImportNames::ImportSpecific { names: ns, .. } => {
        ns.clone()
    }
    _ => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
};
    let __local_type_names = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in typed_module.items.iter().cloned() {
        __mapped_2.push(__elem_3.name.clone());
    }
    Rc::new(__mapped_2)
};
    let __import_enums = {
    let mut __filtered_4 = Vec::new();
    for __elem_5 in __all_imported.iter().cloned() {
        if is_enum_type_name(&__elem_5, vtoe.clone()) {
    __filtered_4.push(__elem_5);
};
    }
    Rc::new(__filtered_4)
};
    let __local_enums = {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in __local_type_names.iter().cloned() {
        if is_enum_type_name(&__elem_7, vtoe.clone()) {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
};
    let __local_variant_names = {
    let mut __flat_mapped_8 = Vec::new();
    for __elem_9 in typed_module.items.iter().cloned() {
        __flat_mapped_8.extend(({
let __cond = {
    let mut __any_12 = false;
    for __elem_13 in __local_enums.iter().cloned() {
        if __elem_13.clone() == __elem_9.name.clone() {
    __any_12 = true;
    break;
};
    }
    __any_12
};
if __cond {
    {
    let mut __mapped_10 = Vec::new();
    for __elem_11 in __elem_9.children.iter().cloned() {
        __mapped_10.push(__elem_11.name.clone());
    }
    Rc::new(__mapped_10)
}
} else {
    Rc::new(Vec::new())
}
}).iter().cloned());
    }
    Rc::new(__flat_mapped_8)
};
    let __module_enums = v2_rt::concat(__local_enums, __import_enums);
    let __candidate_variant_names = unique_strings(v2_rt::concat(__all_imported, __local_variant_names));
    let module_vtoe = {
    let mut __acc_14 = vtoe.clone();
    for __elem_15 in __candidate_variant_names.iter().cloned() {
        __acc_14 = match __acc_14.clone().get(&__elem_15.clone()).cloned() {
    Some(current_parent) => {
        {
    let parent_ok = {
    let mut __any_16 = false;
    for __elem_17 in __module_enums.iter().cloned() {
        if __elem_17.clone() == current_parent.clone() {
    __any_16 = true;
    break;
};
    }
    __any_16
};
    if parent_ok.clone() {
    __acc_14.clone()
} else {
    let correct = {
    let mut __found_20 = None;
    for __elem_21 in __module_enums.iter().cloned() {
        if emit_map_has(emit_info.enum_variant_membership.clone(), &v2_rt::concat(v2_rt::concat(__elem_21.clone(), "|".to_string()), __elem_15.clone())) {
    __found_20 = Some(__elem_21);
    break;
};
    }
    __found_20
};
    match correct.clone() {
    Some(p) => {
        {
    let __rc_23 = __acc_14;
    let mut __map_ins_22 = Rc::try_unwrap(__rc_23).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_22.insert(__elem_15.clone(), p.clone());
    Rc::new(__map_ins_22)
}
    }
    None => {
        __acc_14.clone()
    }
}
}
}
    }
    None => {
        __acc_14.clone()
    }
};
    }
    __acc_14
};
    let scope = module_emit_scope(typed_module.clone());
    let prelude = emit_prelude();
    let local_type_names = {
    let mut __mapped_26 = Vec::new();
    for __elem_27 in ({
    let mut __filtered_24 = Vec::new();
    for __elem_25 in typed_module.items.iter().cloned() {
        if classify_typed_item(__elem_25.clone()) == TypedItemKind::TypedItemTypeDef {
    __filtered_24.push(__elem_25);
};
    }
    Rc::new(__filtered_24)
}).iter().cloned() {
        __mapped_26.push(__elem_27.name.clone());
    }
    Rc::new(__mapped_26)
};
    let imports_str = emit_imports(m.imports.clone(), module_vtoe.clone(), registry.clone(), local_type_names.clone());
    let imports_section = if imports_str.clone() == "" {
    "".to_string()
} else {
    v2_rt::concat("\n".to_string(), imports_str.clone())
};
    let this_mod_filename = module_to_filename(&m.name);
    let all_svc_names = {
    let mut __flat_mapped_28 = Vec::new();
    for __elem_29 in typed_module.items.iter().cloned() {
        __flat_mapped_28.extend((match registry.clone().get(&__elem_29.name.clone()).cloned() {
    Some(info) => {
        {
    let is_func = info.kind.clone() == ItemKind::FuncItem;
    if is_func.clone() {
    info.service_names.clone()
} else {
    Rc::new(Vec::new())
}
}
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_28)
};
    let extern_svc_imports = {
    let mut __flat_mapped_30 = Vec::new();
    for __elem_31 in unique_strings(all_svc_names.clone()).iter().cloned() {
        __flat_mapped_30.extend((match svc_module_map.clone().get(&__elem_31.clone()).cloned() {
    Some(mod_file) => {
        if mod_file.clone() == this_mod_filename.clone() {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("pub use crate::".to_string(), mod_file.clone()), "::".to_string()), sanitize_service_name(&__elem_31)), ";".to_string())))
}
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_30)
};
    let svc_imports_str = if ({
    let __len_35 = extern_svc_imports.clone().len();
    __len_35 as i64
}) == 0_i64 {
    "".to_string()
} else {
    v2_rt::concat("\n".to_string(), {
    let mut __joined_32 = String::new();
    let mut __first_34 = true;
    for __elem_33 in extern_svc_imports.iter().cloned() {
        if !__first_34 {
    __joined_32.push_str(&"\n".to_string());
};
        __first_34 = false;
        __joined_32.push_str(&__elem_33);
    }
    __joined_32
})
};
    let local_enum_uses = {
    let mut __mapped_38 = Vec::new();
    for __elem_39 in ({
    let mut __filtered_36 = Vec::new();
    for __elem_37 in typed_module.items.iter().cloned() {
        if (classify_typed_item(__elem_37.clone()) == TypedItemKind::TypedItemTypeDef) && (classify_type_structure(__elem_37.clone()) == TypeStructureKind::TypeDisj) {
    __filtered_36.push(__elem_37);
};
    }
    Rc::new(__filtered_36)
}).iter().cloned() {
        __mapped_38.push(v2_rt::concat(v2_rt::concat("use ".to_string(), __elem_39.name.clone()), "::*;".to_string()));
    }
    Rc::new(__mapped_38)
};
    let local_uses_str = if ({
    let __len_43 = local_enum_uses.clone().len();
    __len_43 as i64
}) == 0_i64 {
    "".to_string()
} else {
    v2_rt::concat("\n".to_string(), {
    let mut __joined_40 = String::new();
    let mut __first_42 = true;
    for __elem_41 in local_enum_uses.iter().cloned() {
        if !__first_42 {
    __joined_40.push_str(&"\n".to_string());
};
        __first_42 = false;
        __joined_40.push_str(&__elem_41);
    }
    __joined_40
})
};
    let items_str = {
    let mut __joined_46 = String::new();
    let mut __first_48 = true;
    for __elem_47 in ({
    let mut __mapped_44 = Vec::new();
    for __elem_45 in typed_module.items.iter().cloned() {
        __mapped_44.push(emit_typed_item(__elem_45.clone(), registry.clone(), scope.clone(), module_vtoe.clone(), rc_types.clone(), emit_info.clone()));
    }
    Rc::new(__mapped_44)
}).iter().cloned() {
        if !__first_48 {
    __joined_46.push_str(&"\n\n".to_string());
};
        __first_48 = false;
        __joined_46.push_str(&__elem_47);
    }
    __joined_46
};
    let raw_filename = module_to_filename(&m.name);
    let filename = if raw_filename.clone() == "main" {
    "main_mod".to_string()
} else {
    raw_filename.clone()
};
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Generated by v2 compiler -- do not edit.\n".to_string(), "// Source module: ".to_string()), m.name.clone()), "\n\n".to_string()), prelude), imports_section.clone()), svc_imports_str.clone()), local_uses_str.clone()), "\n\n".to_string()), items_str.clone()), "\n".to_string());
    Rc::new(TextFile { path: v2_rt::concat(v2_rt::concat(rust_source_root(), filename.clone()), rust_source_ext()), content: content.clone() })
}

pub fn emit_imports(imports: Rc<Vec<Rc<Import>>>, vtoe: Rc<HashMap<String, String>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, local_names: Rc<Vec<String>>) -> String {
    if ({
    let __len_48 = imports.clone().len();
    __len_48 as i64
}) == 0_i64 {
    "".to_string()
} else {
    let import_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in imports.iter().cloned() {
        __mapped_0.push({
    let mod_name = module_to_filename(&__elem_1.module_path);
    match __elem_1.names.as_ref() {
    ImportNames::ImportAll => {
        v2_rt::concat(v2_rt::concat("use crate::".to_string(), mod_name.clone()), "::*;".to_string())
    }
    ImportNames::ImportSpecific { names: specific_names, .. } => {
        {
    let filtered_names = {
    let mut __filtered_2 = Vec::new();
    for __elem_3 in specific_names.iter().cloned() {
        {
let __cond = {
    let mut __all_4 = true;
    for __elem_5 in local_names.iter().cloned() {
        if !(__elem_5.clone() != __elem_3.clone()) {
    __all_4 = false;
    break;
};
    }
    __all_4
};
if __cond {
    __filtered_2.push(__elem_3);
}
};
    }
    Rc::new(__filtered_2)
};
    if ({
    let __len_42 = filtered_names.clone().len();
    __len_42 as i64
}) == 0_i64 {
    "".to_string()
} else {
    let deduped_names = unique_strings(filtered_names.clone());
    let top_level = {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in deduped_names.iter().cloned() {
        if match vtoe.clone().get(&__elem_7.clone()).cloned() {
    Some(_) => {
        is_enum_type_name(&__elem_7, vtoe.clone())
    }
    None => {
        true
    }
} {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
};
    let all_parents = {
    let mut __filtered_10 = Vec::new();
    for __elem_11 in ({
    let mut __mapped_8 = Vec::new();
    for __elem_9 in deduped_names.iter().cloned() {
        __mapped_8.push(if is_enum_type_name(&__elem_9, vtoe.clone()) {
    "".to_string()
} else {
    match vtoe.clone().get(&__elem_9.clone()).cloned() {
    Some(parent) => {
        parent.clone()
    }
    None => {
        "".to_string()
    }
}
});
    }
    Rc::new(__mapped_8)
}).iter().cloned() {
        if __elem_11.clone() != "" {
    __filtered_10.push(__elem_11);
};
    }
    Rc::new(__filtered_10)
};
    let parent_list = unique_strings(all_parents.clone());
    let top_with_parents = unique_strings(v2_rt::concat(top_level.clone(), parent_list.clone()));
    let main_line = if ({
    let __len_17 = top_with_parents.clone().len();
    __len_17 as i64
}) > 0_i64 {
    let names_str = {
    let mut __joined_14 = String::new();
    let mut __first_16 = true;
    for __elem_15 in ({
    let mut __mapped_12 = Vec::new();
    for __elem_13 in top_with_parents.iter().cloned() {
        __mapped_12.push(match registry.clone().get(&__elem_13.clone()).cloned() {
    Some(info) => {
        {
    let is_data = info.kind.clone() == ItemKind::DataItem;
    if is_data.clone() {
    to_screaming_snake(&__elem_13)
} else {
    __elem_13.clone()
}
}
    }
    None => {
        __elem_13.clone()
    }
});
    }
    Rc::new(__mapped_12)
}).iter().cloned() {
        if !__first_16 {
    __joined_14.push_str(&", ".to_string());
};
        __first_16 = false;
        __joined_14.push_str(&__elem_15);
    }
    __joined_14
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("pub use crate::".to_string(), mod_name.clone()), "::{".to_string()), names_str.clone()), "};".to_string())
} else {
    "".to_string()
};
    let variant_lines = {
    let mut __mapped_18 = Vec::new();
    for __elem_19 in parent_list.iter().cloned() {
        __mapped_18.push({
    let variants = {
    let mut __filtered_20 = Vec::new();
    for __elem_21 in deduped_names.iter().cloned() {
        if if is_enum_type_name(&__elem_21, vtoe.clone()) {
    false
} else {
    match vtoe.clone().get(&__elem_21.clone()).cloned() {
    Some(p) => {
        p.clone() == __elem_19.clone()
    }
    None => {
        false
    }
}
} {
    __filtered_20.push(__elem_21);
};
    }
    Rc::new(__filtered_20)
};
    let vars_str = {
    let mut __joined_22 = String::new();
    let mut __first_24 = true;
    for __elem_23 in variants.iter().cloned() {
        if !__first_24 {
    __joined_22.push_str(&", ".to_string());
};
        __first_24 = false;
        __joined_22.push_str(&__elem_23);
    }
    __joined_22
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("use crate::".to_string(), mod_name.clone()), "::".to_string()), __elem_19.clone()), "::{".to_string()), vars_str.clone()), "};".to_string())
});
    }
    Rc::new(__mapped_18)
};
    let imported_enums = {
    let mut __filtered_25 = Vec::new();
    for __elem_26 in top_level.iter().cloned() {
        if is_enum_type_name(&__elem_26, vtoe.clone()) {
    __filtered_25.push(__elem_26);
};
    }
    Rc::new(__filtered_25)
};
    let wildcard_enum_lines = {
    let mut __mapped_31 = Vec::new();
    for __elem_32 in ({
    let mut __filtered_27 = Vec::new();
    for __elem_28 in imported_enums.iter().cloned() {
        if ({
    let mut __any_29 = false;
    for __elem_30 in parent_list.iter().cloned() {
        if __elem_30.clone() == __elem_28.clone() {
    __any_29 = true;
    break;
};
    }
    __any_29
}) == false {
    __filtered_27.push(__elem_28);
};
    }
    Rc::new(__filtered_27)
}).iter().cloned() {
        __mapped_31.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("use crate::".to_string(), mod_name.clone()), "::".to_string()), __elem_32.clone()), "::*;".to_string()));
    }
    Rc::new(__mapped_31)
};
    let all_lines = if main_line.clone() != "" {
    v2_rt::concat(v2_rt::concat(Rc::new(vec!(main_line.clone())), {
    let mut __filtered_33 = Vec::new();
    for __elem_34 in variant_lines.iter().cloned() {
        if __elem_34.clone() != "" {
    __filtered_33.push(__elem_34);
};
    }
    Rc::new(__filtered_33)
}), wildcard_enum_lines.clone())
} else {
    v2_rt::concat({
    let mut __filtered_37 = Vec::new();
    for __elem_38 in variant_lines.iter().cloned() {
        if __elem_38.clone() != "" {
    __filtered_37.push(__elem_38);
};
    }
    Rc::new(__filtered_37)
}, wildcard_enum_lines.clone())
};
    {
    let mut __joined_39 = String::new();
    let mut __first_41 = true;
    for __elem_40 in all_lines.iter().cloned() {
        if !__first_41 {
    __joined_39.push_str(&"\n".to_string());
};
        __first_41 = false;
        __joined_39.push_str(&__elem_40);
    }
    __joined_39
}
}
}
    }
}
});
    }
    Rc::new(__mapped_0)
};
    {
    let mut __joined_45 = String::new();
    let mut __first_47 = true;
    for __elem_46 in ({
    let mut __filtered_43 = Vec::new();
    for __elem_44 in import_lines.iter().cloned() {
        if __elem_44.clone() != "" {
    __filtered_43.push(__elem_44);
};
    }
    Rc::new(__filtered_43)
}).iter().cloned() {
        if !__first_47 {
    __joined_45.push_str(&"\n".to_string());
};
        __first_47 = false;
        __joined_45.push_str(&__elem_46);
    }
    __joined_45
}
}
}

pub fn emit_prelude() -> String {
    let serde_import = v2_rt::concat("use serde::{".to_string(), "Serialize, Deserialize};".to_string());
    let imports = v2_rt::concat(v2_rt::concat("use std::collections::BTreeMap;\nuse std::rc::Rc;\n".to_string(), serde_import.clone()), "\nuse crate::v2_rt;".to_string());
    v2_rt::concat(v2_rt::concat(imports.clone(), "\n\n".to_string()), emit_non_empty_wrappers())
}

pub fn emit_non_empty_wrappers() -> String {
    let vec_wrapper = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("#[derive(Debug, Clone, PartialEq, Serialize)]\n".to_string(), "pub struct NonEmptyVec<T>(Vec<T>);\n\n".to_string()), "impl<T> NonEmptyVec<T> {\n".to_string()), "    pub fn new(items: Vec<T>) -> Result<Self, &'static str> {\n".to_string()), "        if items.is_empty() {\n".to_string()), "            Err(\"NonEmptyVec requires at least one element\")\n".to_string()), "        } else {\n".to_string()), "            Ok(Self(items))\n".to_string()), "        }\n".to_string()), "    }\n\n".to_string()), "    pub fn as_slice(&self) -> &[T] {\n".to_string()), "        &self.0\n".to_string()), "    }\n\n".to_string()), "    pub fn into_vec(self) -> Vec<T> {\n".to_string()), "        self.0\n".to_string()), "    }\n".to_string()), "}\n\n".to_string()), "impl<'de, T> Deserialize<'de> for NonEmptyVec<T>\n".to_string()), "where\n".to_string()), "    T: Deserialize<'de>,\n".to_string()), "{\n".to_string()), "    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n".to_string()), "    where\n".to_string()), "        D: serde::Deserializer<'de>,\n".to_string()), "    {\n".to_string()), "        let items = Vec::<T>::deserialize(deserializer)?;\n".to_string()), "        NonEmptyVec::new(items).map_err(serde::de::Error::custom)\n".to_string()), "    }\n".to_string()), "}".to_string());
    let set_wrapper = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("#[derive(Debug, Clone, PartialEq, Serialize)]\n".to_string(), "pub struct NonEmptyBTreeSet<T: Ord>(std::collections::BTreeSet<T>);\n\n".to_string()), "impl<T: Ord> NonEmptyBTreeSet<T> {\n".to_string()), "    pub fn new(items: std::collections::BTreeSet<T>) -> Result<Self, &'static str> {\n".to_string()), "        if items.is_empty() {\n".to_string()), "            Err(\"NonEmptyBTreeSet requires at least one element\")\n".to_string()), "        } else {\n".to_string()), "            Ok(Self(items))\n".to_string()), "        }\n".to_string()), "    }\n\n".to_string()), "    pub fn as_set(&self) -> &std::collections::BTreeSet<T> {\n".to_string()), "        &self.0\n".to_string()), "    }\n\n".to_string()), "    pub fn into_set(self) -> std::collections::BTreeSet<T> {\n".to_string()), "        self.0\n".to_string()), "    }\n".to_string()), "}\n\n".to_string()), "impl<'de, T> Deserialize<'de> for NonEmptyBTreeSet<T>\n".to_string()), "where\n".to_string()), "    T: Ord + Deserialize<'de>,\n".to_string()), "{\n".to_string()), "    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n".to_string()), "    where\n".to_string()), "        D: serde::Deserializer<'de>,\n".to_string()), "    {\n".to_string()), "        let items = std::collections::BTreeSet::<T>::deserialize(deserializer)?;\n".to_string()), "        NonEmptyBTreeSet::new(items).map_err(serde::de::Error::custom)\n".to_string()), "    }\n".to_string()), "}".to_string());
    v2_rt::concat(v2_rt::concat(vec_wrapper.clone(), "\n\n".to_string()), set_wrapper.clone())
}

pub fn emit_typed_item(item: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let kind = classify_typed_item(item.clone());
    if kind.clone() == TypedItemKind::TypedItemTypeDef {
    emit_type_def_from_connective(item.clone(), scope.type_env.recursive_type_set.clone(), rc_types.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemTypeAlias {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "type ".to_string()), item.name.clone()), " = ".to_string()), emit_node_type_rc(rt_type(item.clone()), RenderTarget::Rust, rc_types.clone())), ";".to_string())
} else {
    if kind.clone() == TypedItemKind::TypedItemTypeDecl {
    "".to_string()
} else {
    if kind.clone() == TypedItemKind::TypedItemFunction {
    if ({
    let __len_0 = item.uses.clone().len();
    __len_0 as i64
}) > 0_i64 {
    emit_func_def(&item.name, item.params.clone(), rt_type(item.clone()), item.uses.clone(), item.body.clone().unwrap(), registry.clone(), scope.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
} else {
    emit_fn_def(&item.name, item.params.clone(), rt_type(item.clone()), item.body.clone().unwrap(), registry.clone(), scope.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
}
} else {
    if kind.clone() == TypedItemKind::TypedItemDataDef {
    emit_data_def(&item.name, item.type_annotation.clone().unwrap(), item.body.clone().unwrap(), registry.clone(), scope.clone(), 0_i64, vtoe.clone(), rc_types.clone(), emit_info.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemServiceDef {
    emit_service_def(item.clone(), registry.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemResourceDef {
    emit_resource_def(item.clone())
} else {
    if kind.clone() == TypedItemKind::TypedItemExternFunc {
    let params_str = emit_params(item.params.clone(), rc_types.clone());
    let ret_str = emit_return_type(rt_type(item.clone()), rc_types.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "fn ".to_string()), emit_ident(&item.name, RenderTarget::Rust)), "(".to_string()), params_str), ")".to_string()), ret_str), " { todo!(\"extern func\") }".to_string())
} else {
    v2_rt::concat(v2_rt::concat("compile_error!(\"unhandled item: ".to_string(), item.name.clone()), "\");".to_string())
}
}
}
}
}
}
}
}
}

pub fn needs_box_wrapping(n: Rc<Node>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_n = n;
        let mut __tco_p_recursive_types = recursive_types;
        let mut __tco_p_rc_types = rc_types;
        loop {
            let n = __tco_p_n;
            let recursive_types = __tco_p_recursive_types;
            let rc_types = __tco_p_rc_types;
            if ({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64 {
    if emit_map_has(rc_types.clone(), &n.name) {
    break false;
} else {
    break emit_map_has(recursive_types.clone(), &n.name);
};
} else {
    if node_is_optional(n.clone()) {
     {
        let __tco_0 = with_required_cardinality(n.clone());
        let __tco_1 = recursive_types.clone();
        let __tco_2 = rc_types.clone();
        __tco_p_n = __tco_0;
        __tco_p_recursive_types = __tco_1;
        __tco_p_rc_types = __tco_2;
        continue;
    }

} else {
    break false;
};
};
        }
    })
}

pub fn emit_type_def_from_connective(item: Rc<Node>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let kind = classify_type_structure(item.clone());
    if kind == TypeStructureKind::TypeConj {
    emit_struct_from_children(&item.name, item.children.clone(), recursive_types.clone(), rc_types.clone())
} else {
    emit_enum_from_children(&item.name, item.children.clone(), recursive_types.clone(), rc_types.clone())
}
}

pub fn emit_struct_from_children(name: &str, children: Rc<Vec<Rc<Node>>>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let derives = if emit_map_has(rc_types.clone(), &name) {
    rust_struct_derives_text()
} else {
    rust_struct_derives_copy_text()
};
    if ({
    let __len_5 = children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(derives.clone(), "\n".to_string()), rust_visibility_prefix()), "struct ".to_string()), name.to_string()), ";".to_string())
} else {
    let field_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in children.iter().cloned() {
        __mapped_0.push(emit_struct_field_from_child(__elem_1.clone(), recursive_types.clone(), rc_types.clone()));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(derives.clone(), "\n".to_string()), rust_visibility_prefix()), "struct ".to_string()), name.to_string()), " {\n".to_string()), fields_str.clone()), "\n}".to_string())
}
}

pub fn emit_struct_field_from_child(child: Rc<Node>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let rt_child = rt_type(child.clone());
    let ty = emit_node_type_rc(rt_child.clone(), RenderTarget::Rust, rc_types.clone());
    let final_ty = if needs_box_wrapping(rt_child.clone(), recursive_types.clone(), rc_types.clone()) {
    v2_rt::concat(v2_rt::concat("Box<".to_string(), ty), ">".to_string())
} else {
    ty
};
    let rename_attr = match {
    let mut __found_2 = None;
    for __elem_3 in child.properties.iter().cloned() {
        if __elem_3.name.clone() == "from_key" {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(prop) => {
        match prop.value.expr_data.as_ref() {
    ExprData::ExprLiteral { value: lv, .. } => {
        match lv.as_ref() {
    LiteralValue::LitStr { value: key, .. } => {
        v2_rt::concat(v2_rt::concat("    ".to_string(), apply_type_template1(&rust_serde_rename_template_text(), &key)), "\n".to_string())
    }
    _ => {
        "".to_string()
    }
}
    }
    _ => {
        "".to_string()
    }
}
    }
    None => {
        "".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rename_attr.clone(), "    ".to_string()), rust_visibility_prefix()), emit_ident(&child.name, RenderTarget::Rust)), ": ".to_string()), final_ty.clone()), ",".to_string())
}

pub fn enum_derives(children: Rc<Vec<Rc<Node>>>) -> String {
    let complex = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.children.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    match ({
    let __len_3 = complex.clone().len();
    __len_3 as i64
}) == 0_i64 {
    true => {
        rust_enum_derives_copy_text()
    }
    false => {
        rust_enum_derives_text()
    }
}
}

pub fn emit_enum_from_children(name: &str, children: Rc<Vec<Rc<Node>>>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let derives = enum_derives(children.clone());
    let variant_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in children.iter().cloned() {
        __mapped_0.push(emit_variant_from_child(__elem_1.clone(), recursive_types.clone(), rc_types.clone()));
    }
    Rc::new(__mapped_0)
};
    let variants_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in variant_lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let enum_def = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(derives, "\n".to_string()), rust_serde_tag_attr()), "\n".to_string()), rust_visibility_prefix()), "enum ".to_string()), name.to_string()), " {\n".to_string()), variants_str.clone()), "\n}".to_string());
    let accessor_impl = emit_enum_shared_accessors(&name, children.clone(), recursive_types.clone(), rc_types.clone());
    if accessor_impl.clone() == "" {
    enum_def.clone()
} else {
    v2_rt::concat(v2_rt::concat(enum_def.clone(), "\n".to_string()), accessor_impl.clone())
}
}

pub fn find_shared_enum_fields(children: Rc<Vec<Rc<Node>>>) -> Rc<Vec<String>> {
    let fielded = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.children.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    if ({
    let __len_3 = fielded.clone().len();
    __len_3 as i64
}) == 0_i64 {
    return Rc::new(Vec::new());
};
    let first_fields = match fielded.clone().first().cloned() {
    Some(v) => {
        {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in v.children.iter().cloned() {
        __mapped_4.push(__elem_5.name.clone());
    }
    Rc::new(__mapped_4)
}
    }
    None => {
        Rc::new(Vec::new())
    }
};
    {
    let mut __filtered_6 = Vec::new();
    for __elem_7 in first_fields.iter().cloned() {
        {
let __cond = {
    let mut __all_8 = true;
    for __elem_9 in fielded.iter().cloned() {
        if !({
    let mut __any_10 = false;
    for __elem_11 in __elem_9.children.iter().cloned() {
        if __elem_11.name.clone() == __elem_7.clone() {
    __any_10 = true;
    break;
};
    }
    __any_10
}) {
    __all_8 = false;
    break;
};
    }
    __all_8
};
if __cond {
    __filtered_6.push(__elem_7);
}
};
    }
    Rc::new(__filtered_6)
}
}

pub fn emit_enum_shared_accessors(name: &str, children: Rc<Vec<Rc<Node>>>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let all_shared = find_shared_enum_fields(children.clone());
    if ({
    let __len_0 = all_shared.clone().len();
    __len_0 as i64
}) == 0_i64 {
    return "".to_string();
};
    let fielded = {
    let mut __filtered_1 = Vec::new();
    for __elem_2 in children.iter().cloned() {
        if ({
    let __len_3 = __elem_2.children.clone().len();
    __len_3 as i64
}) > 0_i64 {
    __filtered_1.push(__elem_2);
};
    }
    Rc::new(__filtered_1)
};
    if ({
    let __len_4 = fielded.clone().len();
    __len_4 as i64
}) == 0_i64 {
    return "compile_error!(\"enum shared accessor missing fielded variant\")".to_string();
};
    let first_fielded = fielded.clone().first().cloned().unwrap();
    let shared = {
    let mut __filtered_5 = Vec::new();
    for __elem_6 in all_shared.iter().cloned() {
        {
let __cond = {
    let types = {
    let mut __mapped_7 = Vec::new();
    for __elem_8 in fielded.iter().cloned() {
        __mapped_7.push(match {
    let mut __found_11 = None;
    for __elem_12 in __elem_8.children.iter().cloned() {
        if __elem_12.name.clone() == __elem_6.clone() {
    __found_11 = Some(__elem_12);
    break;
};
    }
    __found_11
} {
    Some(f) => {
        emit_node_type_rc(rt_type(f.clone()), RenderTarget::Rust, rc_types.clone())
    }
    None => {
        "".to_string()
    }
});
    }
    Rc::new(__mapped_7)
};
    let first_type = match types.clone().first().cloned() {
    Some(t) => {
        t.clone()
    }
    None => {
        "".to_string()
    }
};
    {
    let mut __all_13 = true;
    for __elem_14 in types.iter().cloned() {
        if !(__elem_14.clone() == first_type.clone()) {
    __all_13 = false;
    break;
};
    }
    __all_13
}
};
if __cond {
    __filtered_5.push(__elem_6);
}
};
    }
    Rc::new(__filtered_5)
};
    if ({
    let __len_15 = shared.clone().len();
    __len_15 as i64
}) == 0_i64 {
    return "".to_string();
};
    let accessor_fns = {
    let mut __mapped_16 = Vec::new();
    for __elem_17 in shared.iter().cloned() {
        __mapped_16.push({
    let ty = match {
    let mut __found_20 = None;
    for __elem_21 in first_fielded.children.iter().cloned() {
        if __elem_21.name.clone() == __elem_17.clone() {
    __found_20 = Some(__elem_21);
    break;
};
    }
    __found_20
} {
    Some(f) => {
        emit_node_type_rc(rt_type(f.clone()), RenderTarget::Rust, rc_types.clone())
    }
    None => {
        "compile_error!(\"enum shared accessor missing field metadata\")".to_string()
    }
};
    let arms = {
    let mut __mapped_22 = Vec::new();
    for __elem_23 in children.iter().cloned() {
        __mapped_22.push(if ({
    let __len_24 = __elem_23.children.clone().len();
    __len_24 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("            ".to_string(), name.to_string()), "::".to_string()), __elem_23.name.clone()), " => panic!(\"no ".to_string()), __elem_17.clone()), " on unit variant\"),".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("            ".to_string(), name.to_string()), "::".to_string()), __elem_23.name.clone()), " { ".to_string()), emit_ident(&__elem_17, RenderTarget::Rust)), ": __val, .. } => __val.clone(),".to_string())
});
    }
    Rc::new(__mapped_22)
};
    let arms_str = {
    let mut __joined_25 = String::new();
    let mut __first_27 = true;
    for __elem_26 in arms.iter().cloned() {
        if !__first_27 {
    __joined_25.push_str(&"\n".to_string());
};
        __first_27 = false;
        __joined_25.push_str(&__elem_26);
    }
    __joined_25
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    pub fn ".to_string(), emit_ident(&__elem_17, RenderTarget::Rust)), "(&self) -> ".to_string()), ty.clone()), " {\n        match self {\n".to_string()), arms_str.clone()), "\n        }\n    }".to_string())
});
    }
    Rc::new(__mapped_16)
};
    let fns_str = {
    let mut __joined_28 = String::new();
    let mut __first_30 = true;
    for __elem_29 in accessor_fns.iter().cloned() {
        if !__first_30 {
    __joined_28.push_str(&"\n".to_string());
};
        __first_30 = false;
        __joined_28.push_str(&__elem_29);
    }
    __joined_28
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("impl ".to_string(), name.to_string()), " {\n".to_string()), fns_str.clone()), "\n}".to_string())
}

pub fn emit_variant_from_child(child: Rc<Node>, recursive_types: Rc<HashMap<String, bool>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    if ({
    let __len_5 = child.children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat("    ".to_string(), child.name.clone()), ",".to_string())
} else {
    let field_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in child.children.iter().cloned() {
        __mapped_0.push({
    let rt_f = rt_type(__elem_1.clone());
    let ty = emit_node_type_rc(rt_f.clone(), RenderTarget::Rust, rc_types.clone());
    let final_ty = if needs_box_wrapping(rt_f.clone(), recursive_types.clone(), rc_types.clone()) {
    v2_rt::concat(v2_rt::concat("Box<".to_string(), ty.clone()), ">".to_string())
} else {
    ty.clone()
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("        ".to_string(), emit_ident(&__elem_1.name, RenderTarget::Rust)), ": ".to_string()), final_ty.clone()), ",".to_string())
});
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), child.name.clone()), " {\n".to_string()), fields_str.clone()), "\n    },".to_string())
}
}

pub fn emit_fn_def(name: &str, params: Rc<Vec<Rc<Param>>>, return_type: Rc<Node>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let params_str = emit_params(params.clone(), rc_types.clone());
    let ret_str = emit_return_type(return_type.clone(), rc_types.clone());
    let body_scope = build_params_scope(scope.clone(), params.clone());
    let depth = 0_i64;
    let use_tco = is_tco_eligible(&name, body.clone(), registry.clone());
    let needs_stacker = is_self_recursive(&name, body.clone(), registry.clone()) && (use_tco.clone() == false);
    if use_tco.clone() {
    let tco_params_str = emit_tco_params(params.clone(), rc_types.clone());
    let body_str = emit_typed_tco_body(body.clone(), &name, params.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "fn ".to_string()), emit_ident(&name, RenderTarget::Rust)), "(".to_string()), tco_params_str), ")".to_string()), ret_str), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), body_str), "\n}".to_string())
} else {
    emit_fn_def_non_tco(&name, &params_str, &ret_str, body.clone(), registry.clone(), body_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone(), needs_stacker.clone())
}
}

pub fn emit_fn_def_non_tco(name: &str, params_str: &str, ret_str: &str, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>, needs_stacker: bool) -> String {
    if needs_stacker {
    let body_str = emit_typed_expr(body.clone(), registry.clone(), scope.clone(), depth.clone() + 2_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "fn ".to_string()), emit_ident(&name, RenderTarget::Rust)), "(".to_string()), params_str.to_string()), ")".to_string()), ret_str.to_string()), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), "stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {\n".to_string()), make_indent(depth.clone() + 2_i64)), body_str), "\n".to_string()), make_indent(depth.clone() + 1_i64)), "})\n}".to_string())
} else {
    let body_str = emit_typed_expr(body.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "fn ".to_string()), emit_ident(&name, RenderTarget::Rust)), "(".to_string()), params_str.to_string()), ")".to_string()), ret_str.to_string()), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), body_str), "\n}".to_string())
}
}

pub fn emit_func_def(name: &str, params: Rc<Vec<Rc<Param>>>, return_type: Rc<Node>, uses: Rc<Vec<Rc<ResourceUse>>>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let depth = 0_i64;
    let service_names = match lookup_item(registry.clone(), &name) {
    Some(info) => {
        info.service_names.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let params_str = emit_func_params(params.clone(), uses.clone(), service_names.clone(), rc_types.clone());
    let ret_str = emit_func_return_type(return_type.clone(), rc_types.clone());
    let body_scope = build_params_scope(scope.clone(), params.clone());
    let body_str = emit_func_body(body.clone(), registry.clone(), body_scope.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "async fn ".to_string()), emit_ident(&name, RenderTarget::Rust)), "(".to_string()), params_str), ")".to_string()), ret_str), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), body_str), "\n}".to_string())
}

pub fn emit_func_body(body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match body.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: inner, .. } => {
        {
    let val_str = emit_typed_expr(v.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
    let let_line = emit_let_binding(&n, &val_str, RenderTarget::Rust);
    let next_scope = extend_scope(scope.clone(), &n, rt_type(v.clone()));
    match inner.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        v2_rt::concat(v2_rt::concat(let_line, "\n".to_string()), emit_func_body(bd.clone(), registry.clone(), next_scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone()))
    }
    None => {
        let_line
    }
}
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        if ({
    let __len_7 = ss.clone().len();
    __len_7 as i64
}) == 0_i64 {
    "Ok(())".to_string()
} else {
    let init_state = emit_rust_init_block_stmts(ss.clone(), Rc::new(Vec::new()), scope.clone(), registry.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
    let last_stmt = ss.clone().last().cloned();
    let last_str = match last_stmt.clone() {
    Some(s) => {
        match s.expr_data.as_ref() {
    ExprData::ExprReturn { value: v, .. } => {
        v2_rt::concat(v2_rt::concat("return Ok(".to_string(), emit_typed_expr(v.clone(), registry.clone(), init_state.scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())), ")".to_string())
    }
    _ => {
        v2_rt::concat(v2_rt::concat("Ok(".to_string(), emit_typed_expr(s.clone(), registry.clone(), init_state.scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())), ")".to_string())
    }
}
    }
    None => {
        "Ok(())".to_string()
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
    ExprData::ExprReturn { value: v, .. } => {
        v2_rt::concat(v2_rt::concat("return Ok(".to_string(), emit_typed_expr(v.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())), ")".to_string())
    }
    _ => {
        v2_rt::concat(v2_rt::concat("Ok(".to_string(), emit_typed_expr(body.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())), ")".to_string())
    }
}
    })
}

pub fn emit_tco_params(params: Rc<Vec<Rc<Param>>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_tco_param(__elem_1.clone(), rc_types.clone()));
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

pub fn emit_tco_param(param: Rc<Param>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let n = param.type_expr.clone();
    let ty = emit_rust_param_type(n.clone(), rc_types.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat("mut ".to_string(), emit_ident(&param.name, RenderTarget::Rust)), ": ".to_string()), ty)
}

pub fn emit_func_params(params: Rc<Vec<Rc<Param>>>, uses: Rc<Vec<Rc<ResourceUse>>>, service_names: Rc<Vec<String>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let param_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_param(__elem_1.clone(), rc_types.clone()));
    }
    Rc::new(__mapped_0)
};
    let resource_strs = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in uses.iter().cloned() {
        __mapped_2.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.name, RenderTarget::Rust), ": &".to_string()), emit_node_type_rc(__elem_3.resource.clone(), RenderTarget::Rust, rc_types.clone())));
    }
    Rc::new(__mapped_2)
};
    let service_strs = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in service_names.iter().cloned() {
        __mapped_4.push(v2_rt::concat(v2_rt::concat(service_var_name(&__elem_5), ": &".to_string()), sanitize_service_name(&__elem_5)));
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

pub fn emit_func_return_type(return_type: Rc<Node>, rc_types: Rc<HashMap<String, bool>>) -> String {
    v2_rt::concat(v2_rt::concat(" -> Result<".to_string(), emit_node_type_rc(return_type.clone(), RenderTarget::Rust, rc_types.clone())), ", Box<dyn std::error::Error>>".to_string())
}

pub fn emit_params(params: Rc<Vec<Rc<Param>>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in params.iter().cloned() {
        __mapped_0.push(emit_param(__elem_1.clone(), rc_types.clone()));
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

pub fn emit_rust_param_type(n: Rc<Node>, rc_types: Rc<HashMap<String, bool>>) -> String {
    if (n.name.clone() == "Callable") && (({
    let __len_5 = n.params.clone().len();
    __len_5 as i64
}) > 0_i64) {
    let param_types = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in n.params.iter().cloned() {
        __mapped_0.push(emit_node_type_rc(__elem_1.type_expr.clone(), RenderTarget::Rust, rc_types.clone()));
    }
    Rc::new(__mapped_0)
};
    let param_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in param_types.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let ret_str = match n.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        emit_node_type_rc(rt.clone(), RenderTarget::Rust, rc_types.clone())
    }
    _ => {
        "()".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat("impl Fn(".to_string(), param_str.clone()), ") -> ".to_string()), ret_str.clone())
} else {
    emit_node_type_rc(n.clone(), RenderTarget::Rust, rc_types.clone())
}
}

pub fn emit_param(param: Rc<Param>, rc_types: Rc<HashMap<String, bool>>) -> String {
    let n = param.type_expr.clone();
    let ty = emit_rust_param_type(n.clone(), rc_types.clone());
    v2_rt::concat(v2_rt::concat(emit_ident(&param.name, RenderTarget::Rust), ": ".to_string()), ty)
}

pub fn emit_return_type(return_type: Rc<Node>, rc_types: Rc<HashMap<String, bool>>) -> String {
    v2_rt::concat(" -> ".to_string(), emit_node_type_rc(return_type.clone(), RenderTarget::Rust, rc_types.clone()))
}

pub fn needs_reference_node(n: Rc<Node>) -> bool {
    if node_has_structure(n.clone()) || (({
    let __len_1 = n.children.clone().len();
    __len_1 as i64
}) > 0_i64) {
    true
} else {
    let is_copy = ((is_int_type_node(n.clone()) || is_bool_type_node(n.clone())) || is_float_type_node(n.clone())) || ((n.name.clone() == "Unit") && (({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64));
    is_copy.clone() == false
}
}

pub fn is_string_lit_pattern(p: Rc<MatchPattern>) -> bool {
    match p.as_ref() {
    MatchPattern::LitPattern { value: v, .. } => {
        match v.as_ref() {
    LiteralValue::LitStr { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
    }
    _ => {
        false
    }
}
}

pub fn collect_pattern_string_guards(pattern: Rc<MatchPattern>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_pattern = pattern;
        loop {
            let pattern = __tco_p_pattern;
            match pattern.as_ref() {
    MatchPattern::VariantPattern { name: n, parent_enum: _, field_bindings: fbs, .. } => {
        if (n.clone() == "Some") && (({
    let __len_7 = fbs.clone().len();
    __len_7 as i64
}) == 1_i64) {
    match fbs.clone().first().cloned() {
    Some(fb) => {
        if is_string_lit_pattern(fb.binding.clone()) {
    match fb.binding.as_ref() {
    MatchPattern::LitPattern { ref value, .. } => {
        let LiteralValue::LitStr { value: s, .. } = value.as_ref() else { unreachable!() };
        break v2_rt::concat("__some_val == ".to_string(), emit_string_literal(&s, ""));
    }
    _ => {
        break "".to_string();
    }
};
} else {
     {
        let __tco_0 = fb.binding.clone();
        __tco_p_pattern = __tco_0;
        continue;
    }

};
    }
    None => {
        break "".to_string();
    }
};
} else {
    let str_bindings = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in fbs.iter().cloned() {
        if is_string_lit_pattern(__elem_1.binding.clone()) {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    let guards = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in str_bindings.iter().cloned() {
        __mapped_2.push(match __elem_3.binding.as_ref() {
    MatchPattern::LitPattern { ref value, .. } => {
        let LiteralValue::LitStr { value: s, .. } = value.as_ref() else { unreachable!() };
        v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.field_name, RenderTarget::Rust), " == ".to_string()), emit_string_literal(&s, ""))
    }
    _ => {
        "".to_string()
    }
});
    }
    Rc::new(__mapped_2)
};
    {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in guards.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&" && ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    break __joined_4;
};
};
    }
    _ => {
        break "".to_string();
    }
};
        }
    })
}

pub fn all_arms_are_string_lit(arms: Rc<Vec<Rc<MatchArm>>>) -> bool {
    {
    let mut __all_0 = true;
    for __elem_1 in arms.iter().cloned() {
        if !(match __elem_1.pattern.as_ref() {
    MatchPattern::LitPattern { value: v, .. } => {
        match v.as_ref() {
    LiteralValue::LitStr { value: _, .. } => {
        true
    }
    _ => {
        false
    }
}
    }
    MatchPattern::Wildcard => {
        true
    }
    _ => {
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

pub fn emit_pattern(pattern: Rc<MatchPattern>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, scrut_type: &str) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::Bind { name: n, .. } => {
        emit_ident(&n, RenderTarget::Rust)
    }
    MatchPattern::LitPattern { value: v, .. } => {
        rust_literal_for_pattern(v.clone())
    }
    MatchPattern::VariantPattern { name: n, parent_enum, field_bindings: fbs, .. } => {
        emit_variant_pattern(&n, parent_enum.clone(), fbs.clone(), vtoe.clone(), rc_types.clone(), &scrut_type)
    }
    MatchPattern::Wildcard => {
        "_".to_string()
    }
}
    })
}

pub fn pattern_parent_enum(name: &str, parent_enum: Option<String>, scrut_type: &str, vtoe: Rc<HashMap<String, String>>) -> Option<String> {
    let scrut_is_known_enum = (scrut_type != "") && is_enum_type_name(&scrut_type, vtoe.clone());
    if (name == "Some") || (name == "None") {
    None
} else {
    if scrut_is_known_enum.clone() {
    Some(scrut_type.to_string())
} else {
    parent_enum
}
}
}

pub fn emit_variant_pattern(name: &str, parent_enum: Option<String>, field_bindings: Rc<Vec<Rc<FieldBinding>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, scrut_type: &str) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let qualified = if (name == "Some") || (name == "None") {
    name.to_string()
} else {
    match pattern_parent_enum(&name, parent_enum, &scrut_type, vtoe.clone()) {
    Some(parent) => {
        v2_rt::concat(v2_rt::concat(parent, "::".to_string()), name.to_string())
    }
    None => {
        name.to_string()
    }
}
};
        if (name == "Some") && (({
    let __len_9 = field_bindings.clone().len();
    __len_9 as i64
}) == 1_i64) {
    match field_bindings.clone().first().cloned() {
    Some(fb) => {
        if is_string_lit_pattern(fb.binding.clone()) {
    "Some(ref __some_val)".to_string()
} else {
    let inner_pat = emit_pattern(fb.binding.clone(), vtoe.clone(), rc_types.clone(), &scrut_type);
    v2_rt::concat(v2_rt::concat("Some(".to_string(), inner_pat), ")".to_string())
}
    }
    None => {
        qualified.clone()
    }
}
} else {
    if ({
    let __len_8 = field_bindings.clone().len();
    __len_8 as i64
}) == 0_i64 {
    qualified.clone()
} else {
    let effective_bindings = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in field_bindings.iter().cloned() {
        if match __elem_1.binding.as_ref() {
    MatchPattern::Wildcard => {
        false
    }
    _ => {
        true
    }
} {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    if ({
    let __len_7 = effective_bindings.clone().len();
    __len_7 as i64
}) == 0_i64 {
    v2_rt::concat(qualified.clone(), " { .. }".to_string())
} else {
    let binding_strs = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in effective_bindings.iter().cloned() {
        __mapped_2.push(if is_string_lit_pattern(__elem_3.binding.clone()) {
    v2_rt::concat("ref ".to_string(), emit_ident(&__elem_3.field_name, RenderTarget::Rust))
} else {
    let pat_str = emit_pattern(__elem_3.binding.clone(), vtoe.clone(), rc_types.clone(), "");
    v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.field_name, RenderTarget::Rust), ": ".to_string()), pat_str.clone())
});
    }
    Rc::new(__mapped_2)
};
    let bindings_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in binding_strs.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(qualified.clone(), " { ".to_string()), bindings_str.clone()), ", .. }".to_string())
}
}
}
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RcPatternAnalysis {
    pub matches_rc_variant: bool,
    pub matches_option_rc_variant: bool,
    pub needs_rc_pattern: bool,
    pub ref_bound_fields: Rc<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RcMatchAnalysis {
    pub needs_option_deref: bool,
    pub needs_deref: bool,
}

pub fn empty_rc_pattern_analysis() -> Rc<RcPatternAnalysis> {
    Rc::new(RcPatternAnalysis { matches_rc_variant: false, matches_option_rc_variant: false, needs_rc_pattern: false, ref_bound_fields: Rc::new(Vec::new()) })
}

pub fn field_needs_rc_ref(field_name: &str, rc_analysis: Rc<RcPatternAnalysis>) -> bool {
    {
    let mut __any_0 = false;
    for __elem_1 in rc_analysis.ref_bound_fields.iter().cloned() {
        if __elem_1.clone() == field_name {
    __any_0 = true;
    break;
};
    }
    __any_0
}
}

pub fn analyze_rc_pattern(pattern: Rc<MatchPattern>, scrut_type: &str, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>) -> Rc<RcPatternAnalysis> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::VariantPattern { name: n, parent_enum, field_bindings: fbs, .. } => {
        if (n.clone() == "Some") || (n.clone() == "None") {
    if ({
    let __len_0 = fbs.clone().len();
    __len_0 as i64
}) == 1_i64 {
    match fbs.clone().first().cloned() {
    Some(fb) => {
        {
    let inner = analyze_rc_pattern(fb.binding.clone(), "", vtoe.clone(), rc_types.clone());
    Rc::new(RcPatternAnalysis { matches_rc_variant: false, matches_option_rc_variant: (n.clone() == "Some") && inner.matches_rc_variant.clone(), needs_rc_pattern: inner.needs_rc_pattern.clone(), ref_bound_fields: Rc::new(Vec::new()) })
}
    }
    None => {
        empty_rc_pattern_analysis()
    }
}
} else {
    empty_rc_pattern_analysis()
}
} else {
    let matches_rc_variant = match pattern_parent_enum(&n, parent_enum.clone(), &scrut_type, vtoe.clone()) {
    Some(enum_name) => {
        emit_map_has(rc_types.clone(), &enum_name)
    }
    None => {
        false
    }
};
    let ref_bound_fields = {
    let mut __flat_mapped_1 = Vec::new();
    for __elem_2 in fbs.iter().cloned() {
        __flat_mapped_1.extend((if analyze_rc_pattern(__elem_2.binding.clone(), "", vtoe.clone(), rc_types.clone()).matches_rc_variant.clone() {
    Rc::new(vec!(__elem_2.field_name.clone()))
} else {
    Rc::new(Vec::new())
}).iter().cloned());
    }
    Rc::new(__flat_mapped_1)
};
    Rc::new(RcPatternAnalysis { matches_rc_variant: matches_rc_variant.clone(), matches_option_rc_variant: false, needs_rc_pattern: ({
    let __len_3 = ref_bound_fields.clone().len();
    __len_3 as i64
}) > 0_i64, ref_bound_fields: ref_bound_fields.clone() })
}
    }
    _ => {
        empty_rc_pattern_analysis()
    }
}
    })
}

pub fn analyze_rc_match(scrutinee: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>>, scrut_type: &str, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>) -> Rc<RcMatchAnalysis> {
    let arm_analyses = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in arms.iter().cloned() {
        __mapped_0.push(analyze_rc_pattern(__elem_1.pattern.clone(), &scrut_type, vtoe.clone(), rc_types.clone()));
    }
    Rc::new(__mapped_0)
};
    let scrutinee_is_optional = match scrutinee.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(rt.clone())
    }
    _ => {
        false
    }
};
    let scrutinee_is_rc_wrapped = match scrutinee.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        if (({
    let __len_2 = rt.children.clone().len();
    __len_2 as i64
}) == 0_i64) && (rt.name.clone() != "") {
    emit_map_has(rc_types.clone(), &rt.name)
} else {
    false
}
    }
    _ => {
        false
    }
};
    let arms_want_option = {
    let mut __any_3 = false;
    for __elem_4 in arm_analyses.iter().cloned() {
        if __elem_4.matches_option_rc_variant.clone() {
    __any_3 = true;
    break;
};
    }
    __any_3
};
    let arms_want_deref = {
    let mut __any_5 = false;
    for __elem_6 in arm_analyses.iter().cloned() {
        if __elem_6.matches_rc_variant.clone() {
    __any_5 = true;
    break;
};
    }
    __any_5
};
    Rc::new(RcMatchAnalysis { needs_option_deref: arms_want_option.clone(), needs_deref: if scrutinee_is_optional.clone() {
    false
} else {
    scrutinee_is_rc_wrapped.clone() || arms_want_deref.clone()
} })
}

pub fn emit_pattern_rc_aware(pattern: Rc<MatchPattern>, rc_analysis: Rc<RcPatternAnalysis>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, scrut_type: &str) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match pattern.as_ref() {
    MatchPattern::Bind { name: n, .. } => {
        emit_ident(&n, RenderTarget::Rust)
    }
    MatchPattern::LitPattern { value: v, .. } => {
        rust_literal_for_pattern(v.clone())
    }
    MatchPattern::VariantPattern { name: n, parent_enum, field_bindings: fbs, .. } => {
        emit_variant_pattern_rc_aware(&n, parent_enum.clone(), fbs.clone(), rc_analysis.clone(), vtoe.clone(), rc_types.clone(), &scrut_type)
    }
    MatchPattern::Wildcard => {
        "_".to_string()
    }
}
    })
}

pub fn emit_variant_pattern_rc_aware(name: &str, parent_enum: Option<String>, field_bindings: Rc<Vec<Rc<FieldBinding>>>, rc_analysis: Rc<RcPatternAnalysis>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, scrut_type: &str) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let qualified = if (name == "Some") || (name == "None") {
    name.to_string()
} else {
    match pattern_parent_enum(&name, parent_enum, &scrut_type, vtoe.clone()) {
    Some(parent) => {
        v2_rt::concat(v2_rt::concat(parent, "::".to_string()), name.to_string())
    }
    None => {
        name.to_string()
    }
}
};
        if (name == "Some") && (({
    let __len_9 = field_bindings.clone().len();
    __len_9 as i64
}) == 1_i64) {
    match field_bindings.clone().first().cloned() {
    Some(fb) => {
        if is_string_lit_pattern(fb.binding.clone()) {
    "Some(ref __some_val)".to_string()
} else {
    let inner_analysis = analyze_rc_pattern(fb.binding.clone(), &scrut_type, vtoe.clone(), rc_types.clone());
    let inner_pat = emit_pattern_rc_aware(fb.binding.clone(), inner_analysis.clone(), vtoe.clone(), rc_types.clone(), &scrut_type);
    v2_rt::concat(v2_rt::concat("Some(".to_string(), inner_pat), ")".to_string())
}
    }
    None => {
        qualified.clone()
    }
}
} else {
    if ({
    let __len_8 = field_bindings.clone().len();
    __len_8 as i64
}) == 0_i64 {
    qualified.clone()
} else {
    let effective_bindings = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in field_bindings.iter().cloned() {
        if match __elem_1.binding.as_ref() {
    MatchPattern::Wildcard => {
        false
    }
    _ => {
        true
    }
} {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    if ({
    let __len_7 = effective_bindings.clone().len();
    __len_7 as i64
}) == 0_i64 {
    v2_rt::concat(qualified.clone(), " { .. }".to_string())
} else {
    let binding_strs = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in effective_bindings.iter().cloned() {
        __mapped_2.push(if is_string_lit_pattern(__elem_3.binding.clone()) {
    v2_rt::concat("ref ".to_string(), emit_ident(&__elem_3.field_name, RenderTarget::Rust))
} else {
    if field_needs_rc_ref(&__elem_3.field_name, rc_analysis.clone()) {
    v2_rt::concat("ref ".to_string(), emit_ident(&__elem_3.field_name, RenderTarget::Rust))
} else {
    let inner_analysis = analyze_rc_pattern(__elem_3.binding.clone(), "", vtoe.clone(), rc_types.clone());
    let pat_str = emit_pattern_rc_aware(__elem_3.binding.clone(), inner_analysis.clone(), vtoe.clone(), rc_types.clone(), "");
    v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.field_name, RenderTarget::Rust), ": ".to_string()), pat_str.clone())
}
});
    }
    Rc::new(__mapped_2)
};
    let bindings_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in binding_strs.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(qualified.clone(), " { ".to_string()), bindings_str.clone()), ", .. }".to_string())
}
}
}
    })
}

pub fn rc_pattern_preludes(pattern: Rc<MatchPattern>, rc_analysis: Rc<RcPatternAnalysis>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_pattern = pattern;
        let mut __tco_p_rc_analysis = rc_analysis;
        let mut __tco_p_vtoe = vtoe;
        let mut __tco_p_rc_types = rc_types;
        loop {
            let pattern = __tco_p_pattern;
            let rc_analysis = __tco_p_rc_analysis;
            let vtoe = __tco_p_vtoe;
            let rc_types = __tco_p_rc_types;
            match pattern.as_ref() {
    MatchPattern::VariantPattern { name: n, parent_enum: _, field_bindings: fbs, .. } => {
        if (n.clone() == "Some") || (n.clone() == "None") {
    if ({
    let __len_0 = fbs.clone().len();
    __len_0 as i64
}) == 1_i64 {
    match fbs.clone().first().cloned() {
    Some(fb) => {
        {
    let inner_analysis = analyze_rc_pattern(fb.binding.clone(), "", vtoe.clone(), rc_types.clone());
     {
        let __tco_0 = fb.binding.clone();
        let __tco_1 = inner_analysis.clone();
        let __tco_2 = vtoe.clone();
        let __tco_3 = rc_types.clone();
        __tco_p_pattern = __tco_0;
        __tco_p_rc_analysis = __tco_1;
        __tco_p_vtoe = __tco_2;
        __tco_p_rc_types = __tco_3;
        continue;
    }

};
    }
    None => {
        break "".to_string();
    }
};
} else {
    break "".to_string();
};
} else {
    let preludes = {
    let mut __flat_mapped_1 = Vec::new();
    for __elem_2 in fbs.iter().cloned() {
        __flat_mapped_1.extend((if field_needs_rc_ref(&__elem_2.field_name, rc_analysis.clone()) {
    Rc::new(vec!(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let ".to_string(), emit_pattern(__elem_2.binding.clone(), vtoe.clone(), rc_types.clone(), "")), " = ".to_string()), emit_ident(&__elem_2.field_name, RenderTarget::Rust)), ".as_ref() else { unreachable!() };".to_string())))
} else {
    Rc::new(Vec::new())
}).iter().cloned());
    }
    Rc::new(__flat_mapped_1)
};
    {
    let mut __joined_3 = String::new();
    let mut __first_5 = true;
    for __elem_4 in preludes.iter().cloned() {
        if !__first_5 {
    __joined_3.push_str(&" ".to_string());
};
        __first_5 = false;
        __joined_3.push_str(&__elem_4);
    }
    break __joined_3;
};
};
    }
    _ => {
        break "".to_string();
    }
};
        }
    })
}

pub fn explicit_record_struct_name(type_name: Option<String>, inferred_node: Rc<Node>, rc_types: Rc<HashMap<String, bool>>) -> Option<String> {
    if node_is_optional(inferred_node.clone()) {
    let result = type_name.clone();
    return result;
};
    let n = if (inferred_node.name.clone() == "Refined") && node_has_structure(inferred_node.clone()) {
    match inferred_node.children.clone().first().cloned() {
    Some(base) => {
        base.clone()
    }
    None => {
        inferred_node.clone()
    }
}
} else {
    inferred_node.clone()
};
    let n_kind = classify_type_structure(n.clone());
    if n.name.clone() == "__EmitTypeCacheMiss" {
    type_name.clone()
} else {
    if n.name.clone() == "Error" {
    type_name.clone()
} else {
    if n_kind.clone() == TypeStructureKind::TypeConj {
    if n.name.clone() == "" {
    type_name.clone()
} else {
    Some(n.name.clone())
}
} else {
    if n_kind.clone() == TypeStructureKind::TypeDisj {
    type_name.clone()
} else {
    if (({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64) && (n.name.clone() != "") {
    Some(n.name.clone())
} else {
    type_name.clone()
}
}
}
}
}
}

pub fn is_primitive_numeric_node(n: Rc<Node>) -> bool {
    (is_int_type_node(n.clone()) || is_float_type_node(n.clone())) || is_bool_type_node(n.clone())
}

pub fn variant_parent_from_binding_kind(binding_kind: Option<Rc<VarBindingKind>>) -> Option<String> {
    match binding_kind.as_ref().map(|__rc| __rc.as_ref()) {
    Some(VarBindingKind::VariantValueBinding { parent_enum, .. }) => {
        Some(parent_enum.clone())
    }
    _ => {
        None
    }
}
}

pub fn effective_variant_parent(name: &str, binding_kind: Option<Rc<VarBindingKind>>, resolved_type: Option<Rc<InferredNode>>, emit_info: Rc<EmitGraphInfo>) -> Option<String> {
    match resolved_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        if ((rt.name.clone() != "") && (rt.name.clone() != name)) && variant_belongs_to_enum(&name, &rt.name, emit_info.clone()) {
    Some(rt.name.clone())
} else {
    variant_parent_from_binding_kind(binding_kind.clone())
}
    }
    _ => {
        variant_parent_from_binding_kind(binding_kind.clone())
    }
}
}

pub fn emit_var_ref(name: &str, binding_kind: Option<Rc<VarBindingKind>>, resolved_type: Option<Rc<InferredNode>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, emit_info: Rc<EmitGraphInfo>) -> String {
    if (name == "none") || (name == "None") {
    "None".to_string()
} else {
    if (name == "true") || (name == "false") {
    name.to_string()
} else {
    let variant_parent = effective_variant_parent(&name, binding_kind.clone(), resolved_type.clone(), emit_info.clone());
    let ref_str = match variant_parent {
    Some(enum_name) => {
        {
    let qualified = v2_rt::concat(v2_rt::concat(enum_name.clone(), "::".to_string()), name.to_string());
    if emit_map_has(rc_types.clone(), &enum_name) {
    v2_rt::concat(v2_rt::concat("Rc::new(".to_string(), qualified.clone()), ")".to_string())
} else {
    qualified.clone()
}
}
    }
    None => {
        match registry.clone().get(&name.to_string()).cloned() {
    Some(info) => {
        {
    let is_data = info.kind.clone() == ItemKind::DataItem;
    if is_data.clone() {
    v2_rt::concat(to_screaming_snake(&name), ".clone()".to_string())
} else {
    let is_function_value = match binding_kind.as_ref().map(|__rc| __rc.as_ref()) {
    Some(VarBindingKind::FunctionValueBinding) => {
        true
    }
    _ => {
        false
    }
};
    let ident = emit_ident(&name, RenderTarget::Rust);
    let ident_str = if is_function_value.clone() {
    ident
} else {
    match resolved_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(_) => {
        v2_rt::concat(ident, ".clone()".to_string())
    }
    _ => {
        ident
    }
}
};
    ident_str.clone()
}
}
    }
    None => {
        {
    let ident = emit_ident(&name, RenderTarget::Rust);
    let ident_str = match resolved_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(_) => {
        v2_rt::concat(ident, ".clone()".to_string())
    }
    _ => {
        ident
    }
};
    ident_str.clone()
}
    }
}
    }
};
    ref_str.clone()
}
}
}

pub fn emit_typed_expr_base(texpr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprVar { name: n, binding_kind, .. } => {
        if (n.clone() == "none") || (n.clone() == "None") {
    "None".to_string()
} else {
    if (n.clone() == "true") || (n.clone() == "false") {
    n.clone()
} else {
    let variant_parent = effective_variant_parent(&n, binding_kind.clone(), texpr.return_type.clone(), emit_info.clone());
    match variant_parent {
    Some(enum_name) => {
        {
    let qualified = v2_rt::concat(v2_rt::concat(enum_name.clone(), "::".to_string()), n.clone());
    if emit_map_has(rc_types.clone(), &enum_name) {
    v2_rt::concat(v2_rt::concat("Rc::new(".to_string(), qualified.clone()), ")".to_string())
} else {
    qualified.clone()
}
}
    }
    None => {
        match registry.clone().get(&n.clone()).cloned() {
    Some(info) => {
        {
    let is_data = info.kind.clone() == ItemKind::DataItem;
    if is_data.clone() {
    to_screaming_snake(&n)
} else {
    emit_ident(&n, RenderTarget::Rust)
}
}
    }
    None => {
        emit_ident(&n, RenderTarget::Rust)
    }
}
    }
}
}
}
    }
    _ => {
        emit_typed_expr(texpr.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
    })
}

pub fn emit_typed_field_access(base: Rc<Node>, field: &str, summary: Option<Rc<FieldSummary>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_typed_expr_base(base.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let base_is_anon_record = match base.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: bt, .. }) => {
        {
    let kind = classify_type_structure(bt.clone());
    if (kind == TypeStructureKind::TypeConj) && (bt.name.clone() == "") {
    true
} else {
    false
}
}
    }
    _ => {
        false
    }
};
        if base_is_anon_record.clone() {
    let bt = rt_type(base.clone());
    if ({
    let __len_5 = bt.children.clone().len();
    __len_5 as i64
}) == 1_i64 {
    emit_typed_expr(base.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
} else {
    let matches = {
    let mut __filtered_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in bt.children.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        if __elem_4.1.name.clone() == field {
    __filtered_3.push(__elem_4);
};
    }
    Rc::new(__filtered_3)
};
    match matches.clone().first().cloned() {
    Some(m) => {
        if m.0.clone() == 0_i64 {
    v2_rt::concat(base_str, ".0".to_string())
} else {
    if m.0.clone() == 1_i64 {
    v2_rt::concat(base_str, ".1".to_string())
} else {
    if m.0.clone() == 2_i64 {
    v2_rt::concat(base_str, ".2".to_string())
} else {
    if m.0.clone() == 3_i64 {
    v2_rt::concat(base_str, ".3".to_string())
} else {
    v2_rt::concat(v2_rt::concat("compile_error!(\"anonymous record tuple index >= 4 for field: ".to_string(), field.to_string()), "\")".to_string())
}
}
}
}
    }
    None => {
        v2_rt::concat(v2_rt::concat("compile_error!(\"anonymous record field not found: ".to_string(), field.to_string()), "\")".to_string())
    }
}
}
} else {
    match summary.as_ref().map(|__rc| __rc.as_ref()) {
    Some(field_summary) => {
        let field_summary = Rc::new(field_summary.clone());
        match field_summary.access_style.clone() {
    FieldAccessStyle::TupleFirst => {
        v2_rt::concat(base_str, ".0.clone()".to_string())
    }
    FieldAccessStyle::TupleSecond => {
        v2_rt::concat(base_str, ".1.clone()".to_string())
    }
    FieldAccessStyle::OptionalUnwrap => {
        v2_rt::concat(base_str, ".clone().unwrap()".to_string())
    }
    FieldAccessStyle::EnumAccessor => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(base_str, ".".to_string()), emit_ident(&field, RenderTarget::Rust)), "()".to_string())
    }
    FieldAccessStyle::StoredField => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(base_str, ".".to_string()), emit_ident(&field, RenderTarget::Rust)), ".clone()".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat("compile_error!(\"field access missing reconcile summary for '".to_string(), field.to_string()), "'\")".to_string())
    }
}
}
    })
}

pub fn type_needs_rc(env: Rc<TypeEnv>, type_node: Rc<Node>) -> bool {
    type_needs_rc_seen(env.clone(), type_node.clone(), Rc::new(std::collections::HashMap::new()))
}

pub fn type_needs_rc_seen(env: Rc<TypeEnv>, type_node: Rc<Node>, seen: Rc<HashMap<String, bool>>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_env = env;
        let mut __tco_p_type_node = type_node;
        let mut __tco_p_seen = seen;
        loop {
            let env = __tco_p_env;
            let type_node = __tco_p_type_node;
            let seen = __tco_p_seen;
            let normed = normalize_access_type_node(type_node);
            let normed_kind = classify_type_structure(normed.clone());
            if normed_kind.clone() == TypeStructureKind::TypeConj {
    break true;
} else {
    if normed_kind.clone() == TypeStructureKind::TypeDisj {
    {
    let mut __any_0 = false;
    for __elem_1 in normed.children.iter().cloned() {
        if ({
    let __len_2 = __elem_1.children.clone().len();
    __len_2 as i64
}) > 0_i64 {
    __any_0 = true;
    break;
};
    }
    break __any_0;
};
} else {
    let canonical = normed.name.clone();
    if normed.return_type.clone().is_some() {
    let next_seen = if canonical.clone() == "" {
    seen.clone()
} else {
    {
    let __rc_4 = seen;
    let mut __map_ins_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_3.insert(canonical.clone(), true);
    Rc::new(__map_ins_3)
}
};
     {
        let __tco_0 = env.clone();
        let __tco_1 = rt_type(normed.clone());
        let __tco_2 = next_seen.clone();
        __tco_p_env = __tco_0;
        __tco_p_type_node = __tco_1;
        __tco_p_seen = __tco_2;
        continue;
    }

} else {
    if (canonical.clone() != "") && emit_map_has(seen.clone(), &canonical) {
    break false;
} else {
    let next_seen = if canonical.clone() == "" {
    seen.clone()
} else {
    {
    let __rc_6 = seen;
    let mut __map_ins_5 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_5.insert(canonical.clone(), true);
    Rc::new(__map_ins_5)
}
};
    let resolved = resolve_scrutinee_type_node(env.clone(), normed.clone());
    if (((node_has_structure(resolved.clone()) == false) && (resolved.return_type.clone().is_none())) && (resolved.name.clone() == normed.name.clone())) && (({
    let __len_7 = resolved.children.clone().len();
    __len_7 as i64
}) == 0_i64) {
    break false;
} else {
     {
        let __tco_0 = env.clone();
        let __tco_1 = resolved.clone();
        let __tco_2 = next_seen.clone();
        __tco_p_env = __tco_0;
        __tco_p_type_node = __tco_1;
        __tco_p_seen = __tco_2;
        continue;
    }

};
};
};
};
};
        }
    })
}

pub fn rust_map_value_type(receiver_type: Rc<Node>, scope: Rc<InferScope>) -> Option<Rc<Node>> {
    let resolved = resolve_scrutinee_type_node(scope.type_env.clone(), normalize_access_type_node(receiver_type.clone()));
    let map_type = normalize_access_type_node(resolved.clone());
    if node_is_map(map_type.clone()) {
    match map_type.children.clone().get((1_i64) as usize).cloned() {
    Some(value_type) => {
        Some(value_type.clone())
    }
    None => {
        None
    }
}
} else {
    None
}
}

pub fn rust_lookup_receiver_needs_rc_wrap(receiver: Rc<Node>, scope: Rc<InferScope>) -> bool {
    let is_data_binding = match receiver.expr_data.as_ref() {
    ExprData::ExprVar { name: binding_name, binding_kind: _, .. } => {
        match scope.item_registry.clone().get(&binding_name.clone()).cloned() {
    Some(info) => {
        info.kind.clone() == ItemKind::DataItem
    }
    None => {
        false
    }
}
    }
    _ => {
        false
    }
};
    if is_data_binding.clone() == false {
    return false;
};
    match receiver.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: receiver_type, .. }) => {
        match rust_map_value_type(receiver_type.clone(), scope.clone()) {
    Some(value_type) => {
        type_needs_rc(scope.type_env.clone(), value_type.clone())
    }
    None => {
        false
    }
}
    }
    _ => {
        false
    }
}
}

pub fn rust_runtime_bridge_passes_receiver_by_ref(function_name: &str) -> bool {
    (&RT_REF_MAP_FUNCTIONS).contains_key(&function_name.to_string())
}

pub fn rust_runtime_bridge_wraps_result_in_rc(function_name: &str, receiver: Rc<Node>, scope: Rc<InferScope>) -> bool {
    ((function_name == "map_get") || (function_name == "lookup")) && rust_lookup_receiver_needs_rc_wrap(receiver.clone(), scope.clone())
}

pub fn emit_rust_expr_var(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    match expr.expr_data.as_ref() {
    ExprData::ExprVar { name: n, binding_kind, .. } => {
        emit_var_ref(&n, binding_kind.clone(), expr.return_type.clone(), vtoe.clone(), rc_types.clone(), registry.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_var expected ExprVar", RenderTarget::Rust)
    }
}
}

pub fn emit_rust_expr_field_access(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprFieldAccess { base: b, field: f, summary, .. } => {
        if is_typed_service_call_receiver(expr.clone()) {
    match extract_typed_service_name(expr.clone()) {
    Some(svc_name) => {
        service_var_name(&svc_name)
    }
    None => {
        emit_typed_field_access(b.clone(), &f, summary.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
} else {
    emit_typed_field_access(b.clone(), &f, summary.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
}
    }
    _ => {
        emit_error_expr("emit_rust_expr_field_access expected ExprFieldAccess", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_call(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: a, call_semantics: _, .. } => {
        emit_typed_call_expr(&f, a.clone(), expr.return_type.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_call expected ExprCall", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_method_call(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprMethodCall { receiver: r, method: m, args: a, method_semantics, .. } => {
        emit_typed_method_call(r.clone(), &m, a.clone(), expr.return_type.clone(), method_semantics.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_method_call expected ExprMethodCall", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_match(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        emit_typed_match(s.clone(), arm_list.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_match expected ExprMatch", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_if(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        emit_typed_if(c.clone(), t.clone(), e.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_if expected ExprIf", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_let(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: bd, .. } => {
        emit_typed_let(&n, v.clone(), bd.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_let expected ExprLet", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_record_lit(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprRecordLit { type_name: tn, fields: fs, parent_enum, .. } => {
        emit_typed_record_lit(tn.clone(), fs.clone(), parent_enum.clone(), rt_type(expr.clone()), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_record_lit expected ExprRecordLit", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_string_interp(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprStringInterp { parts: ps, .. } => {
        emit_typed_string_interp(ps.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_string_interp expected ExprStringInterp", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_block(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprBlock { stmts: ss, .. } => {
        emit_typed_block(ss.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_block expected ExprBlock", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_cast(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprCast { expr: e, target: t, .. } => {
        emit_typed_cast(e.clone(), t.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_cast expected ExprCast", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_for_each(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprForEach { variable: v, collection: c, body: bd, .. } => {
        emit_typed_for_each(&v, c.clone(), bd.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_for_each expected ExprForEach", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_index(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprIndex { base: b, index: i, .. } => {
        emit_typed_index(b.clone(), i.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_index expected ExprIndex", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_expr_slice(expr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.expr_data.as_ref() {
    ExprData::ExprSlice { base: b, start: s, end: e, .. } => {
        emit_typed_slice(b.clone(), s.clone(), e.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    _ => {
        emit_error_expr("emit_rust_expr_slice expected ExprSlice", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_typed_expr(texpr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        emit_shared_expr(texpr.clone(), RenderTarget::Rust, |result| result.clone(), |child| emit_typed_expr(child.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_var(expr.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_field_access(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_call(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_method_call(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_match(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_if(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_let(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_record_lit(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_string_interp(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_block(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_cast(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_for_each(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_index(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |expr| emit_rust_expr_slice(expr.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()))
    })
}

pub fn emit_cloned_arg(texpr: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        emit_typed_expr(texpr.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    })
}

pub fn is_enum_type_name(type_name: &str, vtoe: Rc<HashMap<String, String>>) -> bool {
    {
    let mut __any_4 = false;
    for __elem_5 in ({
    let __rc_0 = vtoe.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        if __elem_5.clone() == type_name {
    __any_4 = true;
    break;
};
    }
    __any_4
}
}

pub fn variant_belongs_to_enum(variant_name: &str, enum_name: &str, emit_info: Rc<EmitGraphInfo>) -> bool {
    emit_map_has(emit_info.enum_variant_membership.clone(), &v2_rt::concat(v2_rt::concat(enum_name.to_string(), "|".to_string()), variant_name.to_string()))
}

pub fn contextual_variant_parent(variant_name: &str, parent_enum: Option<String>, resolved_type: Rc<Node>, emit_info: Rc<EmitGraphInfo>) -> Option<String> {
    match parent_enum {
    Some(explicit_parent) => {
        if variant_belongs_to_enum(&variant_name, &explicit_parent, emit_info.clone()) {
    Some(explicit_parent.clone())
} else {
    None
}
    }
    None => {
        if ((resolved_type.name.clone() != "") && (resolved_type.name.clone() != variant_name)) && variant_belongs_to_enum(&variant_name, &resolved_type.name, emit_info.clone()) {
    Some(resolved_type.name.clone())
} else {
    None
}
    }
}
}

pub fn is_map_typed_expr(texpr: Rc<Node>) -> bool {
    match texpr.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: n, .. }) => {
        node_is_map(n.clone())
    }
    _ => {
        false
    }
}
}

pub fn emit_typed_call_expr(func: &str, args: Rc<Vec<Rc<NamedArg>>>, return_type: Option<Rc<InferredNode>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let call_str = if func == "empty_map" {
    match return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: ret_type, .. }) => {
        {
    let resolved_ret = resolve_scrutinee_type_node(scope.type_env.clone(), ret_type.clone());
    let type_str = emit_node_type_rc(resolved_ret.clone(), RenderTarget::Rust, rc_types.clone());
    if (type_str.clone() != "") && (type_str.clone() != "Dynamic") {
    v2_rt::concat(v2_rt::concat("<".to_string(), type_str.clone()), ">::new()".to_string())
} else {
    "compile_error!(\"empty_map requires a concrete result type\")".to_string()
}
}
    }
    _ => {
        "compile_error!(\"empty_map missing resolved return type\")".to_string()
    }
}
} else {
    emit_typed_call(&func, args.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
};
        let fallback_lookup_wrap = if func == "lookup" {
    let base_check = match args.clone().first().cloned() {
    Some(receiver_arg) => {
        rust_lookup_receiver_needs_rc_wrap(receiver_arg.value.clone(), scope.clone())
    }
    None => {
        false
    }
};
    if base_check.clone() {
    true
} else {
    match return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: ret_type, .. }) => {
        {
    let resolved_ret = resolve_scrutinee_type_node(scope.type_env.clone(), ret_type.clone());
    if node_is_optional(resolved_ret.clone()) {
    type_needs_rc(scope.type_env.clone(), with_required_cardinality(resolved_ret.clone()))
} else {
    false
}
}
    }
    _ => {
        false
    }
}
}
} else {
    false
};
        if fallback_lookup_wrap.clone() {
    v2_rt::concat(call_str.clone(), ".map(Rc::new)".to_string())
} else {
    call_str.clone()
}
    })
}

pub fn emit_typed_call(func: &str, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if func == "get" {
    let get_args = order_typed_call_args(args.clone(), &func, scope.clone());
    return match get_args.clone().first().cloned() {
    Some(list_arg) => {
        match get_args.clone().get((1_i64) as usize).cloned() {
    Some(idx_arg) => {
        {
    let list_str = emit_typed_expr(list_arg.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let index_str = emit_typed_expr(idx_arg.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(list_str, ".get((".to_string()), index_str), ") as usize).cloned()".to_string())
}
    }
    None => {
        {
    "compile_error!(\"get call missing index argument\")".to_string()
}
    }
}
    }
    None => {
        {
    "compile_error!(\"get call missing list argument\")".to_string()
}
    }
};
};
        if func == "with" {
    let with_args = order_typed_call_args(args.clone(), &func, scope.clone());
    if ({
    let __len_0 = with_args.clone().len();
    __len_0 as i64
}) == 0_i64 {
    return "compile_error!(\"with call missing base record\")".to_string();
};
    let base_arg = with_args.clone().first().cloned().unwrap().value.clone();
    if ({
    let __len_1 = with_args.clone().len();
    __len_1 as i64
}) < 2_i64 {
    return "compile_error!(\"with call missing update record\")".to_string();
};
    let update_arg = with_args.clone().get((1_i64) as usize).cloned().unwrap().value.clone();
    let base_str = emit_typed_expr(base_arg.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let type_name = if base_arg.return_type.clone().is_some() {
    rt_type(base_arg.clone()).name.clone()
} else {
    "compile_error!(\"with call missing resolved record type\")".to_string()
};
    let field_strs = match update_arg.expr_data.as_ref() {
    ExprData::ExprRecordLit { fields: fs, type_name: _, parent_enum: _, .. } => {
        {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in fs.iter().cloned() {
        __mapped_2.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_3.name, RenderTarget::Rust), ": ".to_string()), emit_typed_expr(__elem_3.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())));
    }
    Rc::new(__mapped_2)
}
    }
    _ => {
        Rc::new(vec!(emit_typed_expr(update_arg.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())))
    }
};
    let fields_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in field_strs.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&", ".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    let needs_rc = emit_map_has(rc_types.clone(), &type_name);
    let spread = if needs_rc.clone() {
    v2_rt::concat(v2_rt::concat("(*".to_string(), base_str), ").clone()".to_string())
} else {
    v2_rt::concat(base_str, ".clone()".to_string())
};
    let raw = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(type_name.clone(), " { ".to_string()), fields_str.clone()), ", ..".to_string()), spread.clone()), " }".to_string());
    return if needs_rc.clone() {
    v2_rt::concat(v2_rt::concat("Rc::new(".to_string(), raw.clone()), ")".to_string())
} else {
    raw.clone()
};
};
        if func == "to_string" {
    let to_string_args = order_typed_call_args(args.clone(), &func, scope.clone());
    return match to_string_args.clone().first().cloned() {
    Some(value_arg) => {
        {
    v2_rt::concat(v2_rt::concat("(".to_string(), emit_typed_expr(value_arg.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())), ").to_string()".to_string())
}
    }
    None => {
        {
    "compile_error!(\"to_string call missing value argument\")".to_string()
}
    }
};
};
        let collection_scope = if (((func == "map") || (func == "filter")) || (func == "flat_map")) || (func == "fold") {
    let call_args = order_typed_call_args(args.clone(), &func, scope.clone());
    match call_args.clone().last().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: lps, body: _, semantics, .. } => {
        lambda_scope_from_semantics(scope.clone(), lps.clone(), semantics.clone())
    }
    _ => {
        scope.clone()
    }
}
    }
    None => {
        scope.clone()
    }
}
} else {
    scope.clone()
};
        let ordered_args = order_typed_call_args(args.clone(), &func, collection_scope.clone());
        let callee = lookup_item(registry.clone(), &func);
        let filled_args = fill_default_args(ordered_args.clone(), callee.clone(), registry.clone(), collection_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let is_rt = (&RT_FUNCTIONS).contains_key(&func.to_string());
        let is_rt_ref_map = (&RT_REF_MAP_FUNCTIONS).contains_key(&func.to_string());
        let arg_strs = {
    let mut __mapped_10 = Vec::new();
    for __elem_11 in ({
    let mut __enumerated_7 = Vec::new();
    for (__idx_8, __elem_9) in filled_args.clone().iter().enumerate() {
        __enumerated_7.push((__idx_8 as i64, __elem_9.clone()));
    }
    Rc::new(__enumerated_7)
}).iter().cloned() {
        __mapped_10.push({
    let idx = __elem_11.0.clone();
    let a = __elem_11.1.clone();
    if is_rt_ref_map.clone() && (idx.clone() == 0_i64) {
    let base = emit_typed_expr_base(a.value.clone(), registry.clone(), collection_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat("&".to_string(), base.clone())
} else {
    emit_cloned_arg(a.value.clone(), registry.clone(), collection_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
}
});
    }
    Rc::new(__mapped_10)
};
        let extra_args = match callee.as_ref().map(|__rc| __rc.as_ref()) {
    Some(info) => {
        let info = Rc::new(info.clone());
        {
    let is_func = info.kind.clone() == ItemKind::FuncItem;
    if is_func.clone() {
    let resource_args = {
    let mut __mapped_12 = Vec::new();
    for __elem_13 in info.resource_names.iter().cloned() {
        __mapped_12.push(v2_rt::concat("&".to_string(), emit_ident(&__elem_13, RenderTarget::Rust)));
    }
    Rc::new(__mapped_12)
};
    let service_args = {
    let mut __mapped_14 = Vec::new();
    for __elem_15 in info.service_names.iter().cloned() {
        __mapped_14.push(service_var_name(&__elem_15));
    }
    Rc::new(__mapped_14)
};
    v2_rt::concat(resource_args.clone(), service_args.clone())
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
    let mut __joined_16 = String::new();
    let mut __first_18 = true;
    for __elem_17 in all_args.iter().cloned() {
        if !__first_18 {
    __joined_16.push_str(&", ".to_string());
};
        __first_18 = false;
        __joined_16.push_str(&__elem_17);
    }
    __joined_16
};
        let func_name = if is_rt.clone() {
    v2_rt::concat("v2_rt::".to_string(), emit_ident(&func, RenderTarget::Rust))
} else {
    emit_ident(&func, RenderTarget::Rust)
};
        let call_str = if (is_rt.clone() && (func == "concat")) && (({
    let __len_19 = all_args.clone().len();
    __len_19 as i64
}) > 2_i64) {
    emit_nested_rt_concat(all_args.clone(), "", rc_types.clone())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(func_name.clone(), "(".to_string()), args_str.clone()), ")".to_string())
};
        match callee.as_ref().map(|__rc| __rc.as_ref()) {
    Some(info) => {
        let info = Rc::new(info.clone());
        {
    let is_func = info.kind.clone() == ItemKind::FuncItem;
    if is_func.clone() {
    v2_rt::concat(call_str.clone(), ".await?".to_string())
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

pub fn fill_default_args(ordered: Rc<Vec<Rc<NamedArg>>>, callee: Option<Rc<ItemInfo>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> Rc<Vec<Rc<NamedArg>>> {
    match callee.as_ref().map(|__rc| __rc.as_ref()) {
    Some(info) => {
        let info = Rc::new(info.clone());
        {
    let provided_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in ordered.iter().cloned() {
        __mapped_0.push(match __elem_1.name.clone() {
    Some(n) => {
        n.clone()
    }
    None => {
        "".to_string()
    }
});
    }
    Rc::new(__mapped_0)
};
    let missing_with_defaults = {
    let mut __filtered_2 = Vec::new();
    for __elem_3 in info.params.iter().cloned() {
        {
let __cond = {
    let is_provided = {
    let mut __any_4 = false;
    for __elem_5 in provided_names.iter().cloned() {
        if __elem_5.clone() == __elem_3.name.clone() {
    __any_4 = true;
    break;
};
    }
    __any_4
};
    if is_provided.clone() {
    false
} else {
    __elem_3.default_value.clone().is_some()
}
};
if __cond {
    __filtered_2.push(__elem_3);
}
};
    }
    Rc::new(__filtered_2)
};
    let default_args = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in missing_with_defaults.iter().cloned() {
        __mapped_6.push(Rc::new(NamedArg { name: Some(__elem_7.name.clone()), value: __elem_7.default_value.clone().unwrap() }));
    }
    Rc::new(__mapped_6)
};
    v2_rt::concat(ordered.clone(), default_args.clone())
}
    }
    None => {
        ordered.clone()
    }
}
}

pub fn fill_op_default_args(ordered: Rc<Vec<Rc<NamedArg>>>, op_params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> Rc<Vec<Rc<NamedArg>>> {
    let provided_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in ordered.iter().cloned() {
        __mapped_0.push(match __elem_1.name.clone() {
    Some(n) => {
        n.clone()
    }
    None => {
        "".to_string()
    }
});
    }
    Rc::new(__mapped_0)
};
    let missing_with_defaults = {
    let mut __filtered_2 = Vec::new();
    for __elem_3 in op_params.iter().cloned() {
        {
let __cond = {
    let is_provided = {
    let mut __any_4 = false;
    for __elem_5 in provided_names.iter().cloned() {
        if __elem_5.clone() == __elem_3.name.clone() {
    __any_4 = true;
    break;
};
    }
    __any_4
};
    if is_provided.clone() {
    false
} else {
    __elem_3.default_value.clone().is_some()
}
};
if __cond {
    __filtered_2.push(__elem_3);
}
};
    }
    Rc::new(__filtered_2)
};
    let default_args = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in missing_with_defaults.iter().cloned() {
        __mapped_6.push(Rc::new(NamedArg { name: Some(__elem_7.name.clone()), value: __elem_7.default_value.clone().unwrap() }));
    }
    Rc::new(__mapped_6)
};
    v2_rt::concat(ordered.clone(), default_args.clone())
}

pub fn emit_nested_rt_concat(remaining: Rc<Vec<String>>, acc: &str, rc_types: Rc<HashMap<String, bool>>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_acc = acc.to_string();
        let mut __tco_p_rc_types = rc_types;
        loop {
            let remaining = __tco_p_remaining;
            let acc = __tco_p_acc;
            let rc_types = __tco_p_rc_types;
            match remaining.clone().first().cloned() {
    None => {
        break acc.to_string();
    }
    Some(arg) => {
        {
    let next_acc = if acc == "" {
    arg.clone()
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2_rt::concat(".to_string(), acc.to_string()), ", ".to_string()), arg.clone()), ")".to_string())
};
     {
        let __tco_0 = { let __s = remaining.clone(); let __n = (1_i64) as usize; Rc::new(__s[__n.min(__s.len())..].to_vec()) };
        let __tco_1 = next_acc;
        let __tco_2 = rc_types.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_acc = __tco_1;
        __tco_p_rc_types = __tco_2;
        continue;
    }

};
    }
};
        }
    })
}

pub fn emit_typed_for_each(variable: &str, collection: Rc<Node>, body: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let coll_str = emit_typed_expr(collection.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let elem_type = for_each_element_type_node(rt_type(collection.clone()));
        let body_scope = extend_scope(scope.clone(), &variable, elem_type.clone());
        let body_str = emit_typed_expr(body.clone(), registry.clone(), body_scope.clone(), depth.clone() + 2_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
        let ind1 = make_indent(depth.clone() + 1_i64);
        let ind2 = make_indent(depth.clone() + 2_i64);
        let ind0 = make_indent(depth.clone());
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{\n".to_string(), ind1.clone()), "let mut __collect = Vec::new();\n".to_string()), ind1.clone()), "for ".to_string()), emit_ident(&variable, RenderTarget::Rust)), " in ".to_string()), coll_str), ".iter().cloned() {\n".to_string()), ind2), "__collect.push(".to_string()), body_str), ");\n".to_string()), ind1.clone()), "}\n".to_string()), ind1.clone()), "__collect\n".to_string()), ind0), "}".to_string())
    })
}

pub fn emit_typed_index(base: Rc<Node>, index: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_typed_expr(base.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let index_str = emit_typed_expr(index.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let base_node = normalize_access_type_node(rt_type(base.clone()));
        if is_string_type_node(base_node.clone()) {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2_rt::char_at(&".to_string(), base_str), ", ".to_string()), index_str), ")".to_string())
} else {
    if node_is_map(base_node.clone()) {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), base_str), ").get(&".to_string()), index_str), ").cloned()".to_string())
} else {
    "panic!(\"internal error: unsupported index base in emitter\")".to_string()
}
}
    })
}

pub fn emit_typed_slice(base: Rc<Node>, start: Rc<Node>, end: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let base_str = emit_typed_expr(base.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let start_str = emit_typed_expr(start.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let end_str = emit_typed_expr(end.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let base_node = normalize_access_type_node(rt_type(base.clone()));
        if is_string_type_node(base_node.clone()) {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2_rt::substring(&".to_string(), base_str), ", ".to_string()), start_str), ", ".to_string()), end_str), ")".to_string())
} else {
    "panic!(\"internal error: unsupported slice base in emitter\")".to_string()
}
    })
}

pub fn collection_element_type(receiver_type: Option<Rc<InferredNode>>, rc_types: Rc<HashMap<String, bool>>) -> String {
    match receiver_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        if node_is_container(rt.clone()) {
    match rt.children.clone().first().cloned() {
    Some(elem_node) => {
        emit_node_type_rc(elem_node.clone(), RenderTarget::Rust, rc_types.clone())
    }
    None => {
        "_".to_string()
    }
}
} else {
    "_".to_string()
}
    }
    _ => {
        "_".to_string()
    }
}
}

pub fn lambda_scope_from_semantics(scope: Rc<InferScope>, params: Rc<Vec<String>>, semantics: Option<Rc<LambdaSemantics>>) -> Rc<InferScope> {
    match semantics.as_ref().map(|__rc| __rc.as_ref()) {
    Some(lambda_semantics) => {
        let lambda_semantics = Rc::new(lambda_semantics.clone());
        {
    let mut __acc_3 = scope.clone();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in params.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __acc_3 = {
    let idx = __elem_4.0.clone();
    let param_name = __elem_4.1.clone();
    let param_type = match lambda_semantics.param_types.clone().get((idx.clone()) as usize).cloned() {
    Some(resolved_type) => {
        resolved_type.clone()
    }
    None => {
        leaf_node("Dynamic")
    }
};
    extend_scope(__acc_3.clone(), &param_name, param_type.clone())
};
    }
    __acc_3
}
    }
    None => {
        scope.clone()
    }
}
}

pub fn lambda_param_type_strs(params: Rc<Vec<String>>, semantics: Option<Rc<LambdaSemantics>>, fallback_types: Rc<Vec<String>>, rc_types: Rc<HashMap<String, bool>>) -> Rc<Vec<String>> {
    {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in params.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __mapped_3.push({
    let idx = __elem_4.0.clone();
    let param_name = __elem_4.1.clone();
    let inferred_type = match semantics.as_ref().map(|__rc| __rc.as_ref()) {
    Some(lambda_semantics) => {
        let lambda_semantics = Rc::new(lambda_semantics.clone());
        match lambda_semantics.param_types.clone().get((idx.clone()) as usize).cloned() {
    Some(param_type) => {
        if (param_type.name.clone() == "Dynamic") || (param_type.name.clone() == "Error") {
    None
} else {
    Some(emit_node_type_rc(param_type.clone(), RenderTarget::Rust, rc_types.clone()))
}
    }
    None => {
        None
    }
}
    }
    None => {
        None
    }
};
    let fallback_type = match fallback_types.clone().get((idx.clone()) as usize).cloned() {
    Some(ty) => {
        ty.clone()
    }
    None => {
        "_".to_string()
    }
};
    match inferred_type.clone() {
    Some(ty) => {
        v2_rt::concat(v2_rt::concat(emit_ident(&param_name, RenderTarget::Rust), ": ".to_string()), ty.clone())
    }
    None => {
        v2_rt::concat(v2_rt::concat(emit_ident(&param_name, RenderTarget::Rust), ": ".to_string()), fallback_type.clone())
    }
}
});
    }
    Rc::new(__mapped_3)
}
}

pub fn emit_typed_collection_lambda(lambda_expr: Rc<Node>, elem_type_str: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match lambda_expr.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let param_strs = lambda_param_type_strs(ps.clone(), semantics.clone(), Rc::new(vec!(elem_type_str.to_string())), rc_types.clone());
    let params_str = {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in param_strs.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&", ".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
};
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat("|".to_string(), params_str.clone()), "| ".to_string()), body_str)
}
    }
    _ => {
        emit_typed_expr(lambda_expr.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
    })
}

pub fn emit_typed_fold_lambda(lambda_expr: Rc<Node>, acc_type_str: &str, elem_type_str: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match lambda_expr.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let fallback_types = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __enumerated_0 = Vec::new();
    for (__idx_1, __elem_2) in ps.clone().iter().enumerate() {
        __enumerated_0.push((__idx_1 as i64, __elem_2.clone()));
    }
    Rc::new(__enumerated_0)
}).iter().cloned() {
        __mapped_3.push(if __elem_4.0.clone() == 0_i64 {
    "_".to_string()
} else {
    elem_type_str.to_string()
});
    }
    Rc::new(__mapped_3)
};
    let param_strs = lambda_param_type_strs(ps.clone(), None, fallback_types.clone(), rc_types.clone());
    let params_str = {
    let mut __joined_5 = String::new();
    let mut __first_7 = true;
    for __elem_6 in param_strs.iter().cloned() {
        if !__first_7 {
    __joined_5.push_str(&", ".to_string());
};
        __first_7 = false;
        __joined_5.push_str(&__elem_6);
    }
    __joined_5
};
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat("|".to_string(), params_str.clone()), "| ".to_string()), body_str)
}
    }
    _ => {
        emit_typed_expr(lambda_expr.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
    })
}

pub fn emit_intrinsic_typed_method_call(intrinsic: IntrinsicMethod, fold_accumulator_type: Option<Rc<Node>>, result_type: Option<Rc<InferredNode>>, receiver: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let recv_str = emit_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let first_arg_str = emit_typed_first_arg(args.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let elem_type_str = collection_element_type(receiver.return_type.clone(), rc_types.clone());
        match intrinsic {
    IntrinsicMethod::MethodCount => {
        v2_rt::concat(v2_rt::concat("(".to_string(), recv_str), ".len() as i64)".to_string())
    }
    IntrinsicMethod::MethodJoin => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".join(&".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodSplit => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".split(&".to_string()), first_arg_str), ").map(|s| s.to_string()).collect::<Vec<_>>()".to_string())
    }
    IntrinsicMethod::MethodLast => {
        v2_rt::concat(recv_str, ".last().cloned()".to_string())
    }
    IntrinsicMethod::MethodFirst => {
        v2_rt::concat(recv_str, ".first().cloned()".to_string())
    }
    IntrinsicMethod::MethodEnumerate => {
        v2_rt::concat(recv_str, ".iter().cloned().enumerate().map(|(i, v)| (i as i64, v)).collect::<Vec<_>>()".to_string())
    }
    IntrinsicMethod::MethodChars => {
        v2_rt::concat(recv_str, ".chars().map(|c| c.to_string()).collect::<Vec<_>>()".to_string())
    }
    IntrinsicMethod::MethodStringContains => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2_rt::string_contains(".to_string(), recv_str), ", ".to_string()), first_arg_str), ")".to_string())
    }
    IntrinsicMethod::MethodConcat => {
        {
    let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
    }
    Rc::new(__mapped_0)
};
    let all_arg_strs = v2_rt::concat(Rc::new(vec!(recv_str)), arg_strs.clone());
    emit_nested_rt_concat(all_arg_strs.clone(), "", rc_types.clone())
}
    }
    IntrinsicMethod::MethodMap => {
        {
    let recv_is_optional = match receiver.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(resolve_scrutinee_type_node(scope.type_env.clone(), rt.clone()))
    }
    _ => {
        false
    }
};
    if recv_is_optional.clone() {
    match args.clone().first().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let dag_name = match ps.clone().first().cloned() {
    Some(n) => {
        n.clone()
    }
    None => {
        "__x".to_string()
    }
};
    let p = emit_ident(&dag_name, RenderTarget::Rust);
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".map(|".to_string()), p.clone()), "| ".to_string()), body_str), ")".to_string())
}
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".map(".to_string()), first_arg_str), ")".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".map(".to_string()), first_arg_str), ")".to_string())
    }
}
} else {
    match args.clone().first().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let dag_name = match ps.clone().first().cloned() {
    Some(n) => {
        n.clone()
    }
    None => {
        "__x".to_string()
    }
};
    let p = emit_ident(&dag_name, RenderTarget::Rust);
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{ let mut __result = Vec::new(); for ".to_string(), p.clone()), " in ".to_string()), recv_str), ".iter().cloned() { __result.push(".to_string()), body_str), "); } __result }".to_string())
}
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().map(".to_string()), first_arg_str), ").collect::<Vec<_>>()".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().map(".to_string()), first_arg_str), ").collect::<Vec<_>>()".to_string())
    }
}
}
}
    }
    IntrinsicMethod::MethodFilter => {
        match args.clone().first().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let dag_name = match ps.clone().first().cloned() {
    Some(n) => {
        n.clone()
    }
    None => {
        "__x".to_string()
    }
};
    let p = emit_ident(&dag_name, RenderTarget::Rust);
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{ let mut __result = Vec::new(); for ".to_string(), p.clone()), " in ".to_string()), recv_str), ".iter().cloned() { if ".to_string()), body_str), " { __result.push(".to_string()), p.clone()), "); } } __result }".to_string())
}
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().filter(".to_string()), first_arg_str), ").collect::<Vec<_>>()".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().filter(".to_string()), first_arg_str), ").collect::<Vec<_>>()".to_string())
    }
}
    }
    IntrinsicMethod::MethodAny => {
        match args.clone().first().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let dag_name = match ps.clone().first().cloned() {
    Some(n) => {
        n.clone()
    }
    None => {
        "__x".to_string()
    }
};
    let p = emit_ident(&dag_name, RenderTarget::Rust);
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{ let mut __found = false; for ".to_string(), p.clone()), " in ".to_string()), recv_str), ".iter().cloned() { if ".to_string()), body_str), " { __found = true; break; } } __found }".to_string())
}
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().any(".to_string()), first_arg_str), ")".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().any(".to_string()), first_arg_str), ")".to_string())
    }
}
    }
    IntrinsicMethod::MethodAll => {
        match args.clone().first().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let dag_name = match ps.clone().first().cloned() {
    Some(n) => {
        n.clone()
    }
    None => {
        "__x".to_string()
    }
};
    let p = emit_ident(&dag_name, RenderTarget::Rust);
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{ let mut __all = true; for ".to_string(), p.clone()), " in ".to_string()), recv_str), ".iter().cloned() { if !(".to_string()), body_str), ") { __all = false; break; } } __all }".to_string())
}
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().all(".to_string()), first_arg_str), ")".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().all(".to_string()), first_arg_str), ")".to_string())
    }
}
    }
    IntrinsicMethod::MethodFlatMap => {
        match args.clone().first().cloned() {
    Some(a) => {
        match a.value.expr_data.as_ref() {
    ExprData::ExprLambda { params: ps, body: bd, semantics, .. } => {
        {
    let dag_name = match ps.clone().first().cloned() {
    Some(n) => {
        n.clone()
    }
    None => {
        "__x".to_string()
    }
};
    let p = emit_ident(&dag_name, RenderTarget::Rust);
    let lambda_scope = lambda_scope_from_semantics(scope.clone(), ps.clone(), semantics.clone());
    let body_str = emit_typed_expr(bd.clone(), registry.clone(), lambda_scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{ let mut __result = Vec::new(); for ".to_string(), p.clone()), " in ".to_string()), recv_str), ".iter().cloned() { __result.extend(".to_string()), body_str), "); } __result }".to_string())
}
    }
    _ => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().flat_map(".to_string()), first_arg_str), ").collect::<Vec<_>>()".to_string())
    }
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().flat_map(".to_string()), first_arg_str), ").collect::<Vec<_>>()".to_string())
    }
}
    }
    IntrinsicMethod::MethodSkip => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().skip(".to_string()), first_arg_str), " as usize).collect::<Vec<_>>()".to_string())
    }
    IntrinsicMethod::MethodTake => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().take(".to_string()), first_arg_str), " as usize).collect::<Vec<_>>()".to_string())
    }
    IntrinsicMethod::MethodFold => {
        {
    let contextual_acc_type = match result_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: method_result_type, .. }) => {
        Some(method_result_type.clone())
    }
    _ => {
        None
    }
};
    let acc_type_node = match fold_accumulator_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(acc_type) => {
        let acc_type = Rc::new(acc_type.clone());
        if node_is_map(acc_type.clone()) && (({
    let __len_2 = acc_type.children.clone().len();
    __len_2 as i64
}) == 0_i64) {
    match contextual_acc_type.clone() {
    Some(concrete_type) => {
        concrete_type.clone()
    }
    None => {
        acc_type.clone()
    }
}
} else {
    acc_type.clone()
}
    }
    None => {
        match contextual_acc_type.clone() {
    Some(concrete_type) => {
        concrete_type.clone()
    }
    None => {
        match args.clone().first().cloned() {
    Some(init_arg) => {
        if init_arg.value.return_type.clone().is_some() {
    rt_type(init_arg.value.clone())
} else {
    leaf_node("Dynamic")
}
    }
    None => {
        leaf_node("Dynamic")
    }
}
    }
}
    }
};
    let acc_type_str = emit_node_type_rc(acc_type_node.clone(), RenderTarget::Rust, rc_types.clone());
    let init_str = match args.clone().first().cloned() {
    Some(init_arg) => {
        match init_arg.value.expr_data.as_ref() {
    ExprData::ExprCall { func: init_func, args: _, call_semantics: _, .. } => {
        if ((init_func.clone() == "empty_map") && (acc_type_str.clone() != "_")) && (acc_type_str.clone() != "Dynamic") {
    v2_rt::concat(v2_rt::concat("<".to_string(), acc_type_str.clone()), ">::new()".to_string())
} else {
    emit_typed_expr(init_arg.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
}
    }
    _ => {
        emit_typed_expr(init_arg.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
    }
    None => {
        "compile_error!(\"missing fold init argument\")".to_string()
    }
};
    let fold_fn = match args.clone().get((1_i64) as usize).cloned() {
    Some(a) => {
        emit_typed_fold_lambda(a.value.clone(), &acc_type_str, &elem_type_str, registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "compile_error!(\"missing fold function argument\")".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".iter().cloned().fold(".to_string()), init_str.clone()), ", ".to_string()), fold_fn.clone()), ")".to_string())
}
    }
    IntrinsicMethod::MethodSortBy => {
        {
    let elem_type_str = match receiver.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        {
    let resolved = resolve_scrutinee_type_node(scope.type_env.clone(), rt.clone());
    let elem = for_each_element_type_node(resolved.clone());
    if (elem.name.clone() != "") && (elem.name.clone() != "Dynamic") {
    emit_node_type_rc(elem.clone(), RenderTarget::Rust, rc_types.clone())
} else {
    "_".to_string()
}
}
    }
    _ => {
        "_".to_string()
    }
};
    let sort_key_fn = match args.clone().first().cloned() {
    Some(a) => {
        emit_typed_collection_lambda(a.value.clone(), &elem_type_str, registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "compile_error!(\"missing sort_by key function argument\")".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("{ let mut __sorted = ".to_string(), recv_str), ".clone(); __sorted.sort_by(|a: &".to_string()), elem_type_str.clone()), ", b: &".to_string()), elem_type_str.clone()), "| { let __ka = (".to_string()), sort_key_fn.clone()), ")(a.clone()); let __kb = (".to_string()), sort_key_fn.clone()), ")(b.clone()); __ka.partial_cmp(&__kb).unwrap_or(std::cmp::Ordering::Equal) }); __sorted }".to_string())
}
    }
    IntrinsicMethod::MethodAppend => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2_rt::list_push(".to_string(), recv_str), ", ".to_string()), first_arg_str), ")".to_string())
    }
}
    })
}

pub fn rust_bridge_fn_name(method: RuntimeBridgeMethod) -> String {
    bridge_method_base_name(method)
}

pub fn emit_runtime_bridge_method_call(method: RuntimeBridgeMethod, receiver: Rc<Node>, args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let function_name = rust_bridge_fn_name(method.clone());
        let lowered = if method.clone() == RuntimeBridgeMethod::BridgeGet {
    let list_str = emit_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let index_str = match args.clone().first().cloned() {
    Some(a) => {
        emit_typed_expr(a.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "compile_error!(\"get method missing index\")".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(list_str, ".get((".to_string()), index_str.clone()), ") as usize).cloned()".to_string())
} else {
    if method.clone() == RuntimeBridgeMethod::BridgeWith {
    let base_str = emit_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let type_name = match receiver.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        if (rt.name.clone() == "Error") || (rt.name.clone() == "") {
    "compile_error!(\"with method missing resolved record type\")".to_string()
} else {
    rt.name.clone()
}
    }
    _ => {
        "compile_error!(\"with method missing resolved record type\")".to_string()
    }
};
    if ({
    let __len_0 = args.clone().len();
    __len_0 as i64
}) == 0_i64 {
    return "compile_error!(\"with method missing update record\")".to_string();
};
    let update_arg = args.clone().first().cloned().unwrap().value.clone();
    let field_strs = match update_arg.expr_data.as_ref() {
    ExprData::ExprRecordLit { fields: fs, type_name: _, parent_enum: _, .. } => {
        {
    let mut __mapped_1 = Vec::new();
    for __elem_2 in fs.iter().cloned() {
        __mapped_1.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_2.name, RenderTarget::Rust), ": ".to_string()), emit_typed_expr(__elem_2.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())));
    }
    Rc::new(__mapped_1)
}
    }
    _ => {
        Rc::new(vec!(emit_typed_expr(update_arg.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())))
    }
};
    let fields_str = {
    let mut __joined_3 = String::new();
    let mut __first_5 = true;
    for __elem_4 in field_strs.iter().cloned() {
        if !__first_5 {
    __joined_3.push_str(&", ".to_string());
};
        __first_5 = false;
        __joined_3.push_str(&__elem_4);
    }
    __joined_3
};
    let needs_rc = emit_map_has(rc_types.clone(), &type_name);
    let spread = if needs_rc.clone() {
    v2_rt::concat(v2_rt::concat("(*".to_string(), base_str), ").clone()".to_string())
} else {
    v2_rt::concat(base_str, ".clone()".to_string())
};
    let raw = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(type_name.clone(), " { ".to_string()), fields_str.clone()), ", ..".to_string()), spread.clone()), " }".to_string());
    if needs_rc.clone() {
    v2_rt::concat(v2_rt::concat("Rc::new(".to_string(), raw.clone()), ")".to_string())
} else {
    raw.clone()
}
} else {
    let recv_str = if rust_runtime_bridge_passes_receiver_by_ref(&function_name) {
    v2_rt::concat("&".to_string(), emit_typed_expr_base(receiver.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()))
} else {
    emit_cloned_arg(receiver.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
};
    let arg_strs = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in args.iter().cloned() {
        __mapped_6.push(emit_cloned_arg(__elem_7.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
    }
    Rc::new(__mapped_6)
};
    let all_strs = v2_rt::concat(Rc::new(vec!(recv_str.clone())), arg_strs.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("v2_rt::".to_string(), emit_ident(&function_name, RenderTarget::Rust)), "(".to_string()), {
    let mut __joined_8 = String::new();
    let mut __first_10 = true;
    for __elem_9 in all_strs.iter().cloned() {
        if !__first_10 {
    __joined_8.push_str(&", ".to_string());
};
        __first_10 = false;
        __joined_8.push_str(&__elem_9);
    }
    __joined_8
}), ")".to_string())
}
};
        if rust_runtime_bridge_wraps_result_in_rc(&function_name, receiver.clone(), scope.clone()) {
    v2_rt::concat(lowered.clone(), ".map(Rc::new)".to_string())
} else {
    lowered.clone()
}
    })
}

pub fn emit_typed_method_call(receiver: Rc<Node>, method: &str, args: Rc<Vec<Rc<NamedArg>>>, result_type: Option<Rc<InferredNode>>, method_semantics: Option<Rc<MethodSemantics>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if method_semantics.clone().is_some() {
    match method_semantics.clone().unwrap().as_ref() {
    MethodSemantics::ServiceMethodSemantics { service_name: svc_name, op_params, .. } => {
        {
    let var_name = service_var_name(&svc_name);
    let filled_args = fill_op_default_args(args.clone(), op_params.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let arg_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in filled_args.iter().cloned() {
        __mapped_0.push(emit_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(var_name, ".".to_string()), emit_ident(&method, RenderTarget::Rust)), "(".to_string()), args_str.clone()), ").await?".to_string())
}
    }
    MethodSemantics::IntrinsicMethodSemantics { intrinsic, fold_accumulator_type, .. } => {
        {
    let lowered = emit_intrinsic_typed_method_call(intrinsic.clone(), fold_accumulator_type.clone(), result_type.clone(), receiver.clone(), args.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    lowered
}
    }
    MethodSemantics::RuntimeBridgeSemantics { method: bridge_method, .. } => {
        emit_runtime_bridge_method_call(bridge_method.clone(), receiver.clone(), args.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    MethodSemantics::PlainMethodSemantics => {
        {
    let recv_str = emit_typed_expr(receiver.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let arg_strs = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in args.iter().cloned() {
        __mapped_5.push(emit_typed_expr(__elem_6.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(recv_str, ".".to_string()), emit_ident(&method, RenderTarget::Rust)), "(".to_string()), args_str.clone()), ")".to_string())
}
    }
}
} else {
    "compile_error!(\"method call missing reconcile semantics\")".to_string()
}
    })
}

pub fn emit_typed_first_arg(args: Rc<Vec<Rc<NamedArg>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match args.clone().first().cloned() {
    Some(a) => {
        emit_typed_expr(a.value.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "compile_error!(\"missing method argument\")".to_string()
    }
}
    })
}

pub fn emit_typed_match(scrutinee: Rc<Node>, arms: Rc<Vec<Rc<MatchArm>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let scrut_str = emit_typed_expr(scrutinee.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let scrut_type = match scrutinee.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        if node_is_optional(rt.clone()) {
    with_required_cardinality(rt.clone()).name.clone()
} else {
    rt.name.clone()
}
    }
    _ => {
        "".to_string()
    }
};
        let rc_match = analyze_rc_match(scrutinee.clone(), arms.clone(), &scrut_type, vtoe.clone(), rc_types.clone());
        let match_result_type = match arms.clone().first().cloned() {
    Some(first_arm) => {
        rt_type(first_arm.body.clone())
    }
    None => {
        leaf_node("Dynamic")
    }
};
        let arm_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in arms.iter().cloned() {
        __mapped_0.push(emit_typed_match_arm(__elem_1.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone(), &scrut_type, match_result_type.clone()));
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
        let needs_as_str = all_arms_are_string_lit(arms.clone()) && (({
    let __len_5 = arms.clone().len();
    __len_5 as i64
}) > 0_i64);
        if rc_match.needs_option_deref.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), ".as_deref().cloned() {\n".to_string()), arms_str.clone()), "\n}".to_string())
} else {
    if rc_match.needs_deref.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match (*".to_string(), scrut_str), ").clone() {\n".to_string()), arms_str.clone()), "\n}".to_string())
} else {
    if needs_as_str.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), ".as_str() {\n".to_string()), arms_str.clone()), "\n}".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), " {\n".to_string()), arms_str.clone()), "\n}".to_string())
}
}
}
    })
}

pub fn emit_typed_match_arm(arm: Rc<MatchArm>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>, scrut_type: &str, match_result_type: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let rc_analysis = analyze_rc_pattern(arm.pattern.clone(), &scrut_type, vtoe.clone(), rc_types.clone());
        let pat_str = if rc_analysis.needs_rc_pattern.clone() {
    emit_pattern_rc_aware(arm.pattern.clone(), rc_analysis.clone(), vtoe.clone(), rc_types.clone(), &scrut_type)
} else {
    emit_pattern(arm.pattern.clone(), vtoe.clone(), rc_types.clone(), &scrut_type)
};
        let field_guards = collect_pattern_string_guards(arm.pattern.clone());
        let arm_guard = match arm.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(g) => {
        let g = Rc::new(g.clone());
        emit_typed_expr(g.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "".to_string()
    }
};
        let guard_str = if (field_guards.clone() != "") && (arm_guard.clone() != "") {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(" if ".to_string(), field_guards.clone()), " && ".to_string()), arm_guard.clone())
} else {
    if field_guards.clone() != "" {
    v2_rt::concat(" if ".to_string(), field_guards.clone())
} else {
    if arm_guard.clone() != "" {
    v2_rt::concat(" if ".to_string(), arm_guard.clone())
} else {
    "".to_string()
}
}
};
        let body_str = match arm.body.expr_data.as_ref() {
    ExprData::ExprVar { name: body_name, binding_kind: body_binding_kind, .. } => {
        if variant_parent_from_binding_kind(body_binding_kind.clone()).is_some() {
    emit_var_ref(&body_name, body_binding_kind.clone(), Some(Rc::new(InferredNode::Resolved { node: match_result_type.clone() })), vtoe.clone(), rc_types.clone(), registry.clone(), emit_info.clone())
} else {
    emit_typed_expr(arm.body.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
}
    }
    _ => {
        emit_typed_expr(arm.body.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
};
        if rc_analysis.needs_rc_pattern.clone() {
    let prelude = rc_pattern_preludes(arm.pattern.clone(), rc_analysis.clone(), vtoe.clone(), rc_types.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), pat_str.clone()), guard_str.clone()), " => { ".to_string()), prelude), " ".to_string()), body_str.clone()), " },".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), pat_str.clone()), guard_str.clone()), " => ".to_string()), body_str.clone()), ",".to_string())
}
    })
}

pub fn emit_typed_if(condition: Rc<Node>, then_branch: Rc<Node>, else_branch: Option<Rc<Node>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let cond_str = emit_typed_expr(condition.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        let then_str = emit_typed_expr(then_branch.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
        match else_branch.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        {
    let else_str = emit_typed_expr(eb.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), then_str), "\n} else {\n".to_string()), make_indent(depth.clone() + 1_i64)), else_str), "\n}".to_string())
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), then_str), "\n}".to_string())
    }
}
    })
}

pub fn emit_typed_let(name: &str, value: Rc<Node>, body: Option<Rc<Node>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let val_str = emit_typed_expr(value.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
        let let_line = emit_let_binding(&name, &val_str, RenderTarget::Rust);
        match body.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        {
    let next_scope = extend_scope(scope.clone(), &name, rt_type(value.clone()));
    v2_rt::concat(v2_rt::concat(let_line, "\n".to_string()), emit_typed_expr(bd.clone(), registry.clone(), next_scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone()))
}
    }
    None => {
        let_line
    }
}
    })
}

pub fn emit_record_lit_full(type_name: Option<String>, fields: Rc<Vec<Rc<FieldInit>>>, span: SourceSpan, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let inferred = infer_record_lit(type_name, fields.clone(), span, scope.clone());
    match inferred.typed.expr_data.as_ref() {
    ExprData::ExprRecordLit { type_name: tn, fields: fs, parent_enum, .. } => {
        emit_typed_record_lit(tn.clone(), fs.clone(), parent_enum.clone(), rt_type(inferred.typed.clone()), registry.clone(), scope.clone(), 0_i64, Rc::new(std::collections::HashMap::new()), rc_types.clone(), emit_info.clone())
    }
    _ => {
        "compile_error!(\"internal error: infer_record_lit did not produce ExprRecordLit\")".to_string()
    }
}
}

pub fn is_optional_struct_field(emit_info: Rc<EmitGraphInfo>, struct_name: &str, field_name: &str) -> bool {
    match lookup_emit_type_summary(emit_info.clone(), &struct_name) {
    Some(summary) => {
        match summary.field_summaries.clone().get(&field_name.to_string()).cloned() {
    Some(field_summary) => {
        match field_summary.value_shape.clone() {
    FieldValueShape::OptionalValue => {
        true
    }
    _ => {
        false
    }
}
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

pub fn is_already_optional(texpr: Rc<Node>, emit_info: Rc<EmitGraphInfo>, scope: Rc<InferScope>) -> bool {
    match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: v, .. } => {
        match v.as_ref() {
    LiteralValue::LitNull => {
        true
    }
    _ => {
        false
    }
}
    }
    ExprData::ExprVar { name: n, binding_kind: _, .. } => {
        if (n.clone() == "none") || (n.clone() == "None") {
    true
} else {
    match texpr.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(rt.clone()) || node_is_optional(resolve_scrutinee_type_node(scope.type_env.clone(), rt.clone()))
    }
    _ => {
        match scope.locals.clone().get(&n.clone()).cloned() {
    Some(binding) => {
        node_is_optional(binding.resolved.clone()) || node_is_optional(resolve_scrutinee_type_node(scope.type_env.clone(), binding.resolved.clone()))
    }
    None => {
        false
    }
}
    }
}
}
    }
    ExprData::ExprRecordLit { type_name: tn, fields: _, parent_enum: _, .. } => {
        match tn.clone() {
    Some(name) => {
        (name.clone() == "Some") || (name.clone() == "None")
    }
    None => {
        false
    }
}
    }
    ExprData::ExprFieldAccess { base: b, field: f, summary: fa_summary, .. } => {
        {
    let summary_says_optional = match fa_summary.as_ref().map(|__rc| __rc.as_ref()) {
    Some(fs) => {
        let fs = Rc::new(fs.clone());
        match fs.value_shape.clone() {
    FieldValueShape::OptionalValue => {
        true
    }
    _ => {
        false
    }
}
    }
    None => {
        false
    }
};
    if summary_says_optional.clone() {
    true
} else {
    match b.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: base_type, .. }) => {
        {
    let resolved_base = resolve_scrutinee_type_node(scope.type_env.clone(), base_type.clone());
    if is_optional_struct_field(emit_info.clone(), &resolved_base.name, &f) {
    true
} else {
    match texpr.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(resolve_scrutinee_type_node(scope.type_env.clone(), rt.clone()))
    }
    _ => {
        false
    }
}
}
}
    }
    _ => {
        match texpr.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(resolve_scrutinee_type_node(scope.type_env.clone(), rt.clone()))
    }
    _ => {
        false
    }
}
    }
}
}
}
    }
    _ => {
        match texpr.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(rt.clone()) || node_is_optional(resolve_scrutinee_type_node(scope.type_env.clone(), rt.clone()))
    }
    _ => {
        false
    }
}
    }
}
}

pub fn lookup_struct_field_type_name(struct_node: Rc<Node>, field_name: &str, variant_name: Option<String>) -> Option<String> {
    let direct = match {
    let mut __found_2 = None;
    for __elem_3 in struct_node.children.iter().cloned() {
        if __elem_3.name.clone() == field_name {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
} {
    Some(field_child) => {
        if field_child.return_type.clone().is_some() {
    let field_type = rt_type(field_child.clone());
    if (field_type.name.clone() != "") && (field_type.name.clone() != "Dynamic") {
    Some(field_type.name.clone())
} else {
    None
}
} else {
    None
}
    }
    None => {
        None
    }
};
    match direct.clone() {
    Some(_) => {
        direct.clone()
    }
    None => {
        match variant_name {
    Some(vn) => {
        match {
    let mut __found_6 = None;
    for __elem_7 in struct_node.children.iter().cloned() {
        if __elem_7.name.clone() == vn.clone() {
    __found_6 = Some(__elem_7);
    break;
};
    }
    __found_6
} {
    Some(variant_child) => {
        match {
    let mut __found_10 = None;
    for __elem_11 in variant_child.children.iter().cloned() {
        if __elem_11.name.clone() == field_name {
    __found_10 = Some(__elem_11);
    break;
};
    }
    __found_10
} {
    Some(field_child) => {
        if field_child.return_type.clone().is_some() {
    let field_type = rt_type(field_child.clone());
    if (field_type.name.clone() != "") && (field_type.name.clone() != "Dynamic") {
    Some(field_type.name.clone())
} else {
    None
}
} else {
    None
}
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
    None => {
        None
    }
}
    }
}
}

pub fn emit_field_value_with_context(field_value: Rc<Node>, struct_node: Rc<Node>, outer_type_name: Option<String>, field_name: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match field_value.expr_data.as_ref() {
    ExprData::ExprRecordLit { type_name: tn, fields: inner_fields, parent_enum: pe, .. } => {
        match tn.clone() {
    Some(variant_name) => {
        {
    let node_lookup = lookup_struct_field_type_name(struct_node.clone(), &field_name, outer_type_name);
    let expected_type = match node_lookup {
    Some(_) => {
        node_lookup
    }
    None => {
        if struct_node.name.clone() != "" {
    let ftn_key = v2_rt::concat(v2_rt::concat(struct_node.name.clone(), "|".to_string()), field_name.to_string());
    emit_info.field_type_names.clone().get(&ftn_key.clone()).cloned()
} else {
    None
}
    }
};
    let corrected_parent = match expected_type.clone() {
    Some(et) => {
        {
    let key = v2_rt::concat(v2_rt::concat(et.clone(), "|".to_string()), variant_name);
    if emit_map_has(emit_info.enum_variant_membership.clone(), &key) {
    Some(et.clone())
} else {
    pe.clone()
}
}
    }
    None => {
        pe.clone()
    }
};
    emit_typed_record_lit(tn.clone(), inner_fields.clone(), corrected_parent.clone(), rt_type(field_value.clone()), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
}
    }
    None => {
        emit_typed_expr(field_value.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
    }
    _ => {
        emit_typed_expr(field_value.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
    })
}

pub fn emit_typed_record_lit(type_name: Option<String>, fields: Rc<Vec<Rc<FieldInit>>>, parent_enum: Option<String>, resolved_type: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let struct_name = explicit_record_struct_name(type_name.clone(), resolved_type.clone(), rc_types.clone());
        let qualified_name = match struct_name {
    Some(sn) => {
        Some(sn)
    }
    None => {
        None
    }
};
        match qualified_name.clone() {
    None => {
        {
    let kind = classify_type_structure(resolved_type.clone());
    if (kind == TypeStructureKind::TypeConj) && (resolved_type.name.clone() == "") {
    if ({
    let __len_5 = fields.clone().len();
    __len_5 as i64
}) == 1_i64 {
    match fields.clone().first().cloned() {
    Some(f) => {
        emit_typed_expr(f.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "compile_error!(\"empty anonymous record literal\")".to_string()
    }
}
} else {
    let vals = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in fields.iter().cloned() {
        __mapped_0.push(emit_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
    }
    Rc::new(__mapped_0)
};
    v2_rt::concat(v2_rt::concat("(".to_string(), {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in vals.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}), ")".to_string())
}
} else {
    "compile_error!(\"cannot resolve anonymous record type in emitter\")".to_string()
}
}
    }
    Some(tn) => {
        {
    let context_lookup = contextual_variant_parent(&tn, parent_enum.clone(), resolved_type.clone(), emit_info.clone());
    let vtoe_lookup = match vtoe.clone().get(&tn.clone()).cloned() {
    Some(vp) => {
        Some(vp.clone())
    }
    None => {
        None
    }
};
    let effective_parent = match context_lookup {
    Some(context_parent) => {
        Some(context_parent)
    }
    None => {
        match vtoe_lookup.clone() {
    Some(vtoe_parent) => {
        Some(vtoe_parent.clone())
    }
    None => {
        if (((resolved_type.name.clone() != "") && (resolved_type.name.clone() != tn.clone())) && (resolved_type.name.clone() != "Dynamic")) && (resolved_type.name.clone() != "Error") {
    Some(resolved_type.name.clone())
} else {
    parent_enum.clone()
}
    }
}
    }
};
    let display_tn = if (tn.clone() == "Some") || (tn.clone() == "None") {
    tn.clone()
} else {
    match effective_parent.clone() {
    Some(resolved_parent_enum) => {
        v2_rt::concat(v2_rt::concat(resolved_parent_enum.clone(), "::".to_string()), tn.clone())
    }
    None => {
        tn.clone()
    }
}
};
    if (tn.clone() == "Some") && (({
    let __len_12 = fields.clone().len();
    __len_12 as i64
}) == 1_i64) {
    match fields.clone().first().cloned() {
    Some(f) => {
        {
    let inner = emit_typed_expr(f.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat("Some(".to_string(), inner), ")".to_string())
}
    }
    None => {
        v2_rt::concat(display_tn.clone(), " {}".to_string())
    }
}
} else {
    if ({
    let __len_11 = fields.clone().len();
    __len_11 as i64
}) == 0_i64 {
    let empty_raw = v2_rt::concat(display_tn.clone(), " {}".to_string());
    let record_parent_enum = match parent_enum.clone() {
    Some(en) => {
        en
    }
    None => {
        "".to_string()
    }
};
    let needs_rc = (emit_map_has(rc_types.clone(), &tn) || emit_map_has(rc_types.clone(), &resolved_type.name)) || emit_map_has(rc_types.clone(), &record_parent_enum);
    if needs_rc.clone() {
    v2_rt::concat(v2_rt::concat("Rc::new(".to_string(), empty_raw.clone()), ")".to_string())
} else {
    empty_raw.clone()
}
} else {
    let field_strs = {
    let mut __mapped_6 = Vec::new();
    for __elem_7 in fields.iter().cloned() {
        __mapped_6.push({
    let val_str = emit_field_value_with_context(__elem_7.value.clone(), resolved_type.clone(), type_name.clone(), &__elem_7.name, registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let needs_wrap = is_optional_struct_field(emit_info.clone(), &tn, &__elem_7.name) && (is_already_optional(__elem_7.value.clone(), emit_info.clone(), scope.clone()) == false);
    let field_val = if needs_wrap.clone() {
    v2_rt::concat(v2_rt::concat("Some(".to_string(), val_str.clone()), ")".to_string())
} else {
    val_str.clone()
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), emit_ident(&__elem_7.name, RenderTarget::Rust)), ": ".to_string()), field_val.clone()), ",".to_string())
});
    }
    Rc::new(__mapped_6)
};
    let all_field_strs = field_strs.clone();
    let fields_str = {
    let mut __joined_8 = String::new();
    let mut __first_10 = true;
    for __elem_9 in all_field_strs.iter().cloned() {
        if !__first_10 {
    __joined_8.push_str(&"\n".to_string());
};
        __first_10 = false;
        __joined_8.push_str(&__elem_9);
    }
    __joined_8
};
    let raw = v2_rt::concat(v2_rt::concat(v2_rt::concat(display_tn.clone(), " {\n".to_string()), fields_str.clone()), "\n}".to_string());
    let record_parent_enum = match parent_enum.clone() {
    Some(en) => {
        en
    }
    None => {
        "".to_string()
    }
};
    let needs_rc = (emit_map_has(rc_types.clone(), &tn) || emit_map_has(rc_types.clone(), &resolved_type.name)) || emit_map_has(rc_types.clone(), &record_parent_enum);
    if needs_rc.clone() {
    v2_rt::concat(v2_rt::concat("Rc::new(".to_string(), raw.clone()), ")".to_string())
} else {
    raw.clone()
}
}
}
}
    }
}
    })
}

pub fn emit_typed_bin_op(op: BinOpKind, left: Rc<Node>, right: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let l_str = emit_typed_expr(left.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let r_str = emit_typed_expr(right.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    if is_null_coalesce(op.clone()) {
    emit_null_coalesce(&l_str, &r_str, RenderTarget::Rust)
} else {
    let op_str = emit_bin_op_symbol(op.clone(), RenderTarget::Rust);
    if is_string_comparison(op.clone(), left.clone(), right.clone()) {
    let l_optional = is_optional_typed_expr(left.clone());
    let r_optional = is_optional_typed_expr(right.clone());
    if l_optional.clone() || r_optional.clone() {
    let l_cmp = if l_optional.clone() {
    v2_rt::concat(l_str, ".as_deref()".to_string())
} else {
    v2_rt::concat(v2_rt::concat("Some(".to_string(), l_str), ".as_str())".to_string())
};
    let r_cmp = if r_optional.clone() {
    v2_rt::concat(r_str, ".as_deref()".to_string())
} else {
    v2_rt::concat(v2_rt::concat("Some(".to_string(), r_str), ".as_str())".to_string())
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), l_cmp.clone()), " ".to_string()), op_str), " ".to_string()), r_cmp.clone()), ")".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), l_str), ".as_str() ".to_string()), op_str), " ".to_string()), r_str), ".as_str())".to_string())
}
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("(".to_string(), l_str), " ".to_string()), op_str), " ".to_string()), r_str), ")".to_string())
}
}
}

pub fn is_optional_typed_expr(e: Rc<Node>) -> bool {
    match e.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        node_is_optional(rt.clone())
    }
    _ => {
        false
    }
}
}

pub fn is_string_comparison(op: BinOpKind, left: Rc<Node>, right: Rc<Node>) -> bool {
    match op {
    BinOpKind::BinEq => {
        is_string_typed_expr(left.clone()) && is_string_typed_expr(right.clone())
    }
    BinOpKind::BinNe => {
        is_string_typed_expr(left.clone()) && is_string_typed_expr(right.clone())
    }
    _ => {
        false
    }
}
}

pub fn is_string_typed_expr(e: Rc<Node>) -> bool {
    match e.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        {
    let inner = if node_is_optional(rt.clone()) {
    with_required_cardinality(rt.clone())
} else {
    rt.clone()
};
    is_string_type_node(inner.clone())
}
    }
    _ => {
        false
    }
}
}

pub fn emit_typed_string_interp(parts: Rc<Vec<Rc<StringPart>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let fmt_parts = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in parts.iter().cloned() {
        __mapped_0.push(typed_interp_format_part(__elem_1.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
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
    v2_rt::concat(v2_rt::concat("\"".to_string(), fmt_str.clone()), "\".to_string()".to_string())
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("format!(\"".to_string(), fmt_str.clone()), "\", ".to_string()), args_str.clone()), ")".to_string())
}
    })
}

pub fn typed_interp_format_part(part: Rc<StringPart>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> Rc<InterpPart> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match part.as_ref() {
    StringPart::Text { value: v, .. } => {
        {
    let escaped = {
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
};
    let escaped2 = {
    let mut __joined_7 = String::new();
    let mut __first_9 = true;
    for __elem_8 in ({
    let mut __split_parts_5 = Vec::new();
    for __part_6 in escaped.clone().split("}".to_string().as_str()) {
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
};
    Rc::new(InterpPart { format_segment: escaped2.clone(), arg_expr: "".to_string() })
}
    }
    StringPart::Interpolation { expr: e, .. } => {
        Rc::new(InterpPart { format_segment: "{}".to_string(), arg_expr: emit_typed_expr(e.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone()) })
    }
}
    })
}

pub fn emit_typed_block(stmts: Rc<Vec<Rc<Node>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let state = emit_rust_block_stmts(stmts.clone(), Rc::new(Vec::new()), scope.clone(), registry.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
        v2_rt::concat(v2_rt::concat(v2_rt::concat("{\n".to_string(), make_indent(depth.clone() + 1_i64)), {
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
}), "\n}".to_string())
    })
}

pub fn emit_typed_cast(expr: Rc<Node>, target: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let expr_str = emit_typed_expr(expr.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
        let ty_str = emit_node_type_rc(target.clone(), RenderTarget::Rust, rc_types.clone());
        if is_primitive_numeric_node(target.clone()) {
    v2_rt::concat(v2_rt::concat(expr_str, " as ".to_string()), ty_str.clone())
} else {
    let src_ty = match expr.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: n, .. }) => {
        emit_node_type_rc(n.clone(), RenderTarget::Rust, rc_types.clone())
    }
    _ => {
        "".to_string()
    }
};
    if (src_ty.clone() != "") && (src_ty.clone() == ty_str.clone()) {
    expr_str
} else {
    v2_rt::concat(v2_rt::concat("compile_error!(\"unsupported cast to ".to_string(), ty_str.clone()), "\")".to_string())
}
}
    })
}

pub fn emit_tco_init_block_stmts(remaining: Rc<Vec<Rc<Node>>>, text: Rc<Vec<String>>, scope: Rc<InferScope>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>, params: Rc<Vec<Rc<Param>>>) -> Rc<BlockEmitState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_text = text;
        let mut __tco_p_scope = scope;
        let mut __tco_p_registry = registry;
        let mut __tco_p_depth = depth;
        let mut __tco_p_vtoe = vtoe;
        let mut __tco_p_rc_types = rc_types;
        let mut __tco_p_emit_info = emit_info;
        let mut __tco_p_params = params;
        loop {
            let remaining = __tco_p_remaining;
            let text = __tco_p_text;
            let scope = __tco_p_scope;
            let registry = __tco_p_registry;
            let depth = __tco_p_depth;
            let vtoe = __tco_p_vtoe;
            let rc_types = __tco_p_rc_types;
            let emit_info = __tco_p_emit_info;
            let params = __tco_p_params;
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
    let line = emit_tco_init_stmt(stmt.clone(), params.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
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
        let __tco_5 = vtoe.clone();
        let __tco_6 = rc_types.clone();
        let __tco_7 = emit_info.clone();
        let __tco_8 = params.clone();
        __tco_p_remaining = __tco_0;
        __tco_p_text = __tco_1;
        __tco_p_scope = __tco_2;
        __tco_p_registry = __tco_3;
        __tco_p_depth = __tco_4;
        __tco_p_vtoe = __tco_5;
        __tco_p_rc_types = __tco_6;
        __tco_p_emit_info = __tco_7;
        __tco_p_params = __tco_8;
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

pub fn emit_tco_init_stmt(stmt: Rc<Node>, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    match stmt.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: _, .. } => {
        {
    let val_str = emit_typed_expr(v.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone());
    let is_param = {
    let mut __any_0 = false;
    for __elem_1 in params.iter().cloned() {
        if __elem_1.name.clone() == n.clone() {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    if is_param.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(emit_ident(&n, RenderTarget::Rust), " = ".to_string()), val_str), ";".to_string())
} else {
    emit_let_binding(&n, &val_str, RenderTarget::Rust)
}
}
    }
    _ => {
        emit_typed_expr(stmt.clone(), registry.clone(), scope.clone(), depth, vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
}
}

pub fn emit_typed_tco_body(texpr: Rc<Node>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let inner = emit_typed_tco_expr(texpr.clone(), &fn_name, params.clone(), registry.clone(), scope.clone(), depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat("loop {\n".to_string(), make_indent(depth.clone() + 1_i64)), inner), "\n}".to_string())
}

pub fn emit_rust_tco_non_self_call(frame: Rc<TcoFrame>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    match frame.expr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, args: a, call_semantics: _, .. } => {
        {
    let call_str = emit_typed_call(&f, a.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat("break ".to_string(), call_str), ";".to_string())
}
    }
    _ => {
        emit_error_expr("emit_rust_tco_non_self_call expected ExprCall", RenderTarget::Rust)
    }
}
}

pub fn emit_rust_tco_if(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        {
    let cond_str = emit_typed_expr(c.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let then_str = emit_typed_tco_expr(t.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        {
    let else_str = emit_typed_tco_expr(eb.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone() + 1_i64, vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), " {\n".to_string()), make_indent(frame.depth.clone() + 1_i64)), then_str), "\n} else {\n".to_string()), make_indent(frame.depth.clone() + 1_i64)), else_str), "\n}".to_string())
}
    }
    None => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if ".to_string(), cond_str), " {\n".to_string()), make_indent(frame.depth.clone() + 1_i64)), then_str), "\n}".to_string())
    }
}
}
    }
    _ => {
        emit_error_expr("emit_rust_tco_if expected ExprIf", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_tco_match(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        {
    let scrut_str = emit_typed_expr(s.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let tco_scrut_type = match s.return_type.as_ref().map(|__rc| __rc.as_ref()) {
    Some(InferredNode::Resolved { node: rt, .. }) => {
        if node_is_optional(rt.clone()) {
    with_required_cardinality(rt.clone()).name.clone()
} else {
    rt.name.clone()
}
    }
    _ => {
        "".to_string()
    }
};
    let rc_match = analyze_rc_match(s.clone(), arm_list.clone(), &tco_scrut_type, vtoe.clone(), rc_types.clone());
    let arm_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in arm_list.iter().cloned() {
        __mapped_0.push(emit_typed_tco_match_arm(__elem_1.clone(), &fn_name, params.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone(), &tco_scrut_type));
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
    let tco_needs_as_str = all_arms_are_string_lit(arm_list.clone()) && (({
    let __len_5 = arm_list.clone().len();
    __len_5 as i64
}) > 0_i64);
    if rc_match.needs_option_deref.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), ".as_deref().cloned() {\n".to_string()), arms_str.clone()), "\n}".to_string())
} else {
    if rc_match.needs_deref.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match (*".to_string(), scrut_str), ").clone() {\n".to_string()), arms_str.clone()), "\n}".to_string())
} else {
    if tco_needs_as_str.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), ".as_str() {\n".to_string()), arms_str.clone()), "\n}".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("match ".to_string(), scrut_str), " {\n".to_string()), arms_str.clone()), "\n}".to_string())
}
}
}
}
    }
    _ => {
        emit_error_expr("emit_rust_tco_match expected ExprMatch", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_tco_let(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprLet { name: n, value: v, body: bd, .. } => {
        {
    let val_str = emit_typed_expr(v.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    let is_param = {
    let mut __any_0 = false;
    for __elem_1 in params.iter().cloned() {
        if __elem_1.name.clone() == n.clone() {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    let let_line = if is_param.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(emit_ident(&n, RenderTarget::Rust), " = ".to_string()), val_str), ";".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let ".to_string(), emit_ident(&n, RenderTarget::Rust)), " = ".to_string()), val_str), ";".to_string())
};
    let next_scope = extend_scope(frame.scope.clone(), &n, rt_type(v.clone()));
    match bd.as_ref().map(|__rc| __rc.as_ref()) {
    Some(b) => {
        let b = Rc::new(b.clone());
        v2_rt::concat(v2_rt::concat(let_line.clone(), "\n".to_string()), emit_typed_tco_expr(b.clone(), &fn_name, params.clone(), registry.clone(), next_scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()))
    }
    None => {
        let_line.clone()
    }
}
}
    }
    _ => {
        emit_error_expr("emit_rust_tco_let expected ExprLet", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_tco_block(frame: Rc<TcoFrame>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match frame.expr.expr_data.as_ref() {
    ExprData::ExprBlock { stmts: ss, .. } => {
        if ({
    let __len_7 = ss.clone().len();
    __len_7 as i64
}) == 0_i64 {
    "break;".to_string()
} else {
    let init_state = emit_tco_init_block_stmts(ss.clone(), Rc::new(Vec::new()), frame.scope.clone(), registry.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone(), params.clone());
    let last_str = match ss.clone().last().cloned() {
    Some(last_expr) => {
        emit_typed_tco_expr(last_expr.clone(), &fn_name, params.clone(), registry.clone(), init_state.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "break;".to_string()
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
        emit_error_expr("emit_rust_tco_block expected ExprBlock", RenderTarget::Rust)
    }
}
    })
}

pub fn emit_rust_tco_default_return(frame: Rc<TcoFrame>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let val_str = emit_typed_expr(frame.expr.clone(), registry.clone(), frame.scope.clone(), frame.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
    v2_rt::concat(v2_rt::concat("break ".to_string(), val_str), ";".to_string())
}

pub fn emit_typed_tco_expr(texpr: Rc<Node>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        emit_shared_tco_expr(Rc::new(TcoFrame { expr: texpr.clone(), scope: scope.clone(), depth }), &fn_name, |input| emit_typed_tco_reassign(input.args.clone(), params.clone(), registry.clone(), input.scope.clone(), input.depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |frame| emit_rust_tco_non_self_call(frame.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |frame| emit_rust_tco_if(frame.clone(), &fn_name, params.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |frame| emit_rust_tco_match(frame.clone(), &fn_name, params.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |frame| emit_rust_tco_let(frame.clone(), &fn_name, params.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |frame| emit_rust_tco_block(frame.clone(), &fn_name, params.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()), |frame| emit_rust_tco_default_return(frame.clone(), registry.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()))
    })
}

pub fn emit_typed_tco_match_arm(arm: Rc<MatchArm>, fn_name: &str, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>, scrut_type: &str) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let rc_analysis = analyze_rc_pattern(arm.pattern.clone(), &scrut_type, vtoe.clone(), rc_types.clone());
        let pat_str = if rc_analysis.needs_rc_pattern.clone() {
    emit_pattern_rc_aware(arm.pattern.clone(), rc_analysis.clone(), vtoe.clone(), rc_types.clone(), &scrut_type)
} else {
    emit_pattern(arm.pattern.clone(), vtoe.clone(), rc_types.clone(), &scrut_type)
};
        let field_guards = collect_pattern_string_guards(arm.pattern.clone());
        let arm_guard = match arm.guard.as_ref().map(|__rc| __rc.as_ref()) {
    Some(g) => {
        let g = Rc::new(g.clone());
        emit_typed_expr(g.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone())
    }
    None => {
        "".to_string()
    }
};
        let guard_str = if (field_guards.clone() != "") && (arm_guard.clone() != "") {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(" if ".to_string(), field_guards.clone()), " && ".to_string()), arm_guard.clone())
} else {
    if field_guards.clone() != "" {
    v2_rt::concat(" if ".to_string(), field_guards.clone())
} else {
    if arm_guard.clone() != "" {
    v2_rt::concat(" if ".to_string(), arm_guard.clone())
} else {
    "".to_string()
}
}
};
        let body_str = emit_typed_tco_expr(arm.body.clone(), &fn_name, params.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone());
        if rc_analysis.needs_rc_pattern.clone() {
    let prelude = rc_pattern_preludes(arm.pattern.clone(), rc_analysis.clone(), vtoe.clone(), rc_types.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), pat_str.clone()), guard_str.clone()), " => { ".to_string()), prelude), " ".to_string()), body_str), " },".to_string())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    ".to_string(), pat_str.clone()), guard_str.clone()), " => { ".to_string()), body_str), " },".to_string())
}
    })
}

pub fn emit_typed_tco_reassign(args: Rc<Vec<Rc<NamedArg>>>, params: Rc<Vec<Rc<Param>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let ordered_args = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in args.iter().cloned() {
        __mapped_0.push(emit_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), rc_types.clone(), emit_info.clone()));
    }
    Rc::new(__mapped_0)
};
    let param_names = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in params.iter().cloned() {
        __mapped_2.push(emit_ident(&__elem_3.name, RenderTarget::Rust));
    }
    Rc::new(__mapped_2)
};
    let all_lines = tco_reassign_core(ordered_args.clone(), param_names.clone(), "__tco_", "let ", " = ", ";", "continue;", "");
    v2_rt::concat(v2_rt::concat(v2_rt::concat("{\n".to_string(), make_indent(depth.clone() + 1_i64)), {
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
}), "\n}".to_string())
}

pub fn emit_service_def(item: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let safe_name = sanitize_service_name(&item.name);
    let fallback_transport = service_fallback_transport(item.clone());
    let op_children = item.children.clone();
    let struct_def = emit_service_struct(&safe_name, fallback_transport.clone(), op_children.clone(), item.config.clone());
    let impl_block = emit_service_impl(&safe_name, fallback_transport.clone(), op_children.clone(), registry.clone());
    v2_rt::concat(v2_rt::concat(struct_def, "\n\n".to_string()), impl_block)
}

pub fn emit_service_struct(name: &str, fallback_transport: Rc<Node>, op_children: Rc<Vec<Rc<Node>>>, config: Option<Rc<ServiceConfig>>) -> String {
    let derives = "#[derive(Debug, Clone)]".to_string();
    let config_fields = emit_service_config_fields(fallback_transport.clone(), op_children.clone(), config.clone());
    let dry_run_field = "\n    pub dry_run: crate::dry_run::DryRunMode,".to_string();
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(derives.clone(), "\npub struct ".to_string()), name.to_string()), " {\n".to_string()), config_fields), dry_run_field.clone()), "\n}".to_string())
}

pub fn emit_service_config_fields(fallback_transport: Rc<Node>, op_children: Rc<Vec<Rc<Node>>>, config: Option<Rc<ServiceConfig>>) -> String {
    let has_shell = service_uses_transport(fallback_transport.clone(), op_children.clone(), Rc::new(TransportKind::ShellTransport));
    let has_rest = service_uses_transport(fallback_transport.clone(), op_children.clone(), Rc::new(TransportKind::RestTransport));
    let has_file = service_uses_transport(fallback_transport.clone(), op_children.clone(), Rc::new(TransportKind::FileTransport));
    let has_auth = service_has_rest_auth(fallback_transport.clone(), op_children.clone());
    let shell_field = if has_shell {
    "    pub working_dir: Option<String>,\n".to_string()
} else {
    "".to_string()
};
    let rest_field = if has_rest {
    "    pub base_url: String,\n".to_string()
} else {
    "".to_string()
};
    let auth_field = if has_auth {
    "    pub auth_token: String,\n".to_string()
} else {
    "".to_string()
};
    let file_field = if has_file {
    "    pub base_path: String,\n".to_string()
} else {
    "".to_string()
};
    let fields = v2_rt::concat(v2_rt::concat(v2_rt::concat(shell_field.clone(), rest_field.clone()), auth_field.clone()), file_field.clone());
    if fields.clone() == "" {
    "    // No configuration needed for local binding.\n".to_string()
} else {
    fields.clone()
}
}

pub fn emit_service_impl(name: &str, transport: Rc<Node>, op_children: Rc<Vec<Rc<Node>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let depth = 0_i64;
    let new_method = emit_service_new_method(&name, transport.clone(), op_children.clone());
    let method_strs = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in op_children.iter().cloned() {
        __mapped_0.push(emit_operation_method(&name, transport.clone(), __elem_1.clone(), registry.clone(), depth.clone() + 1_i64));
    }
    Rc::new(__mapped_0)
};
    let all_methods = v2_rt::concat(Rc::new(vec!(new_method)), method_strs.clone());
    let methods_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in all_methods.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("impl ".to_string(), name.to_string()), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), methods_str.clone()), "\n}".to_string())
}

pub fn emit_service_new_method(name: &str, fallback_transport: Rc<Node>, op_children: Rc<Vec<Rc<Node>>>) -> String {
    let has_shell = service_uses_transport(fallback_transport.clone(), op_children.clone(), Rc::new(TransportKind::ShellTransport));
    let has_rest = service_uses_transport(fallback_transport.clone(), op_children.clone(), Rc::new(TransportKind::RestTransport));
    let has_file = service_uses_transport(fallback_transport.clone(), op_children.clone(), Rc::new(TransportKind::FileTransport));
    let has_auth = service_has_rest_auth(fallback_transport.clone(), op_children.clone());
    let base_url_default = if has_rest.clone() {
    let from_fallback = if is_transport_kind(fallback_transport.clone(), Rc::new(TransportKind::RestTransport)) {
    match transport_base_url(fallback_transport.clone()) {
    Some(bu) => {
        match bu.expr_data.as_ref() {
    ExprData::ExprLiteral { ref value, .. } => {
        let LiteralValue::LitStr { value: s, .. } = value.as_ref() else { unreachable!() };
        s.clone()
    }
    _ => {
        "".to_string()
    }
}
    }
    None => {
        "".to_string()
    }
}
} else {
    "".to_string()
};
    if from_fallback.clone() != "" {
    from_fallback.clone()
} else {
    "http://localhost".to_string()
}
} else {
    "".to_string()
};
    let shell_init = if has_shell {
    "        working_dir: None,\n".to_string()
} else {
    "".to_string()
};
    let rest_init = if has_rest.clone() {
    v2_rt::concat(v2_rt::concat("        base_url: \"".to_string(), base_url_default.clone()), "\".to_string(),\n".to_string())
} else {
    "".to_string()
};
    let auth_init = if has_auth {
    "        auth_token: String::new(),\n".to_string()
} else {
    "".to_string()
};
    let file_init = if has_file {
    "        base_path: \".\".to_string(),\n".to_string()
} else {
    "".to_string()
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("pub fn new(dry_run: crate::dry_run::DryRunMode) -> Self {\n".to_string(), "    ".to_string()), name.to_string()), " {\n".to_string()), shell_init.clone()), rest_init.clone()), auth_init.clone()), file_init.clone()), "        dry_run,\n".to_string()), "    }\n".to_string()), "}".to_string())
}

pub fn emit_modifier_doc_from_props(properties: Rc<Vec<Rc<FieldInit>>>) -> String {
    let names = extract_modifier_names(properties.clone());
    if ({
    let __len_3 = names.clone().len();
    __len_3 as i64
}) == 0_i64 {
    "".to_string()
} else {
    v2_rt::concat(v2_rt::concat("/// Modifiers: ".to_string(), {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in names.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&", ".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}), "\n".to_string())
}
}

pub fn emit_operation_method(service_name: &str, transport: Rc<Node>, op_node: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> String {
    let input_params = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in op_node.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.name, RenderTarget::Rust), ": ".to_string()), emit_rust_param_type(__elem_1.type_expr.clone(), Rc::new(std::collections::HashMap::new()))));
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
    "&self".to_string()
} else {
    v2_rt::concat("&self, ".to_string(), params_str.clone())
};
    let ret_type = emit_node_type_rc(rt_type(op_node.clone()), RenderTarget::Rust, Rc::new(std::collections::HashMap::new()));
    let eff_transport = effective_operation_transport(op_node.clone(), transport.clone());
    let op_return_type = rt_type(op_node.clone());
    let real_body = emit_transport_call(eff_transport.clone(), &op_node.name, registry.clone(), depth.clone() + 2_i64, op_return_type.clone());
    let mock_props = {
    let mut __filtered_5 = Vec::new();
    for __elem_6 in op_node.properties.iter().cloned() {
        if has_mock_prefix(&__elem_6.name) {
    __filtered_5.push(__elem_6);
};
    }
    Rc::new(__filtered_5)
};
    let dry_run_body = emit_dry_run_branch_from_props(&op_node.name, rt_type(op_node.clone()), mock_props.clone(), registry.clone());
    let body = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("if self.dry_run.is_dry_run() {\n".to_string(), make_indent(depth.clone() + 2_i64)), dry_run_body), "\n".to_string()), "} else {\n".to_string()), make_indent(depth.clone() + 2_i64)), real_body), "\n".to_string()), "}".to_string());
    let modifier_doc = emit_modifier_doc_from_props(op_node.properties.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(modifier_doc, rust_visibility_prefix()), "async fn ".to_string()), emit_ident(&op_node.name, RenderTarget::Rust)), "(".to_string()), all_params.clone()), ") -> Result<".to_string()), ret_type), ", Box<dyn std::error::Error>> {\n".to_string()), make_indent(depth.clone() + 1_i64)), body.clone()), "\n}".to_string())
}

pub fn emit_dry_run_branch_from_props(op_name: &str, return_type: Rc<Node>, mock_props: Rc<Vec<Rc<FieldInit>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> String {
    let log_line = v2_rt::concat(v2_rt::concat("eprintln!(\"[dry-run] ".to_string(), op_name.to_string()), "\");".to_string());
    if ({
    let __len_0 = mock_props.clone().len();
    __len_0 as i64
}) > 0_i64 {
    let first_mock = mock_props.clone().first().cloned();
    match first_mock.clone() {
    Some(mp) => {
        {
    let mock_json = emit_data_value_json(mp.value.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(log_line.clone(), "\n".to_string()), "let mock_value: serde_json::Value = serde_json::from_str(r#\"".to_string()), mock_json), "\"#)?;\n".to_string()), "Ok(serde_json::from_value(mock_value)?)".to_string())
}
    }
    None => {
        v2_rt::concat(log_line.clone(), "\ncompile_error!(\"mock property list was non-empty but first() returned None\")".to_string())
    }
}
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(log_line.clone(), "\ncompile_error!(\"no mock data available for dry-run operation: ".to_string()), op_name.to_string()), "\")".to_string())
}
}

pub fn emit_transport_call(transport: Rc<Node>, op_name: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64, return_type: Rc<Node>) -> String {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::RestTransport)) {
    emit_rest_call(&op_name, transport.clone(), registry.clone(), depth)
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::ShellTransport)) {
    emit_shell_call(&op_name, transport.clone(), registry.clone(), depth, return_type.clone())
} else {
    if is_transport_kind(transport.clone(), Rc::new(TransportKind::FileTransport)) {
    emit_file_call(&op_name, return_type.clone())
} else {
    emit_local_call(&op_name)
}
}
}
}

pub fn emit_rest_call(op_name: &str, transport: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64) -> String {
    let client_init = "let client = reqwest::Client::new();".to_string();
    let url_line = v2_rt::concat(v2_rt::concat("let url = format!(\"{}/{}\", self.base_url, \"".to_string(), emit_ident(&op_name, RenderTarget::Rust)), "\");".to_string());
    let header_name = match transport_auth_header_name(transport.clone()) {
    Some(h) => {
        h
    }
    None => {
        "Authorization".to_string()
    }
};
    let token_node = match transport_auth_token(transport.clone()) {
    Some(tn) => {
        emit_simple_expr(tn.clone(), RenderTarget::Rust)
    }
    None => {
        "\"\"".to_string()
    }
};
    let auth_line = if transport_has_auth(transport.clone()) {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let request = client.post(&url)\n    .header(\"".to_string(), header_name.clone()), "\", ".to_string()), token_node.clone()), ");".to_string())
} else {
    "let request = client.post(&url);".to_string()
};
    let headers = transport_headers(transport.clone());
    let header_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in headers.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let request = request.header(\"".to_string(), __elem_1.name.clone()), "\", ".to_string()), emit_simple_expr(__elem_1.value.clone(), RenderTarget::Rust)), ");".to_string()));
    }
    Rc::new(__mapped_0)
};
    let send_line = "let response = request.send().await?;".to_string();
    let parse_line = "let result = response.json().await?;".to_string();
    let return_line = "Ok(result)".to_string();
    let all_lines = v2_rt::concat(v2_rt::concat(Rc::new(vec!(client_init.clone(), url_line.clone(), auth_line.clone())), header_lines.clone()), Rc::new(vec!(send_line.clone(), parse_line.clone(), return_line.clone())));
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

pub fn emit_shell_call(op_name: &str, transport: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, depth: i64, return_type: Rc<Node>) -> String {
    let cmd_line = v2_rt::concat(v2_rt::concat("let output = std::process::Command::new(\"".to_string(), emit_ident(&op_name, RenderTarget::Rust)), "\")".to_string());
    let env_entries = transport_env(transport.clone());
    let env_lines = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in env_entries.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("    .env(\"".to_string(), __elem_1.name.clone()), "\", ".to_string()), emit_simple_expr(__elem_1.value.clone(), RenderTarget::Rust)), ")".to_string()));
    }
    Rc::new(__mapped_0)
};
    let wd_line = "    .current_dir(self.working_dir.as_deref().unwrap_or(\".\"))".to_string();
    let output_line = "    .output()?;".to_string();
    let check_line = "let stdout = String::from_utf8_lossy(&output.stdout).to_string();".to_string();
    let return_line = emit_shell_return(return_type.clone());
    let all_lines = v2_rt::concat(v2_rt::concat(Rc::new(vec!(cmd_line.clone())), env_lines.clone()), Rc::new(vec!(wd_line.clone(), output_line.clone(), check_line.clone(), return_line)));
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

pub fn emit_shell_return(return_type: Rc<Node>) -> String {
    let effective = unwrap_single_field_product(return_type.clone());
    let kind = classify_type_structure(effective.clone());
    if (effective.name.clone() == "Bool") || (effective.name.clone() == "bool") {
    "Ok(output.status.success())".to_string()
} else {
    if ((effective.name.clone() == "List") || (effective.name.clone() == "Vec")) || node_is_container(effective.clone()) {
    "Ok(stdout.lines().filter(|l| !l.is_empty()).map(|l| l.trim().to_string()).collect())".to_string()
} else {
    if (kind == TypeStructureKind::TypeConj) && (({
    let __len_0 = effective.children.clone().len();
    __len_0 as i64
}) > 1_i64) {
    "let parsed: serde_json::Value = serde_json::from_str(&stdout)?;\nOk(serde_json::from_value(parsed)?)".to_string()
} else {
    "Ok(stdout)".to_string()
}
}
}
}

pub fn unwrap_single_field_product(n: Rc<Node>) -> Rc<Node> {
    let kind = classify_type_structure(n.clone());
    if ((kind == TypeStructureKind::TypeConj) && (n.name.clone() == "")) && (({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 1_i64) {
    match n.children.clone().first().cloned() {
    Some(field_node) => {
        rt_type(field_node.clone())
    }
    None => {
        n.clone()
    }
}
} else {
    n.clone()
}
}

pub fn emit_file_call(op_name: &str, return_type: Rc<Node>) -> String {
    let effective = unwrap_single_field_product(return_type.clone());
    let kind = classify_type_structure(effective.clone());
    let parse_line = if (kind == TypeStructureKind::TypeConj) && (({
    let __len_0 = effective.children.clone().len();
    __len_0 as i64
}) > 1_i64) {
    "let parsed: serde_json::Value = serde_json::from_str(&content)?;\nOk(serde_json::from_value(parsed)?)".to_string()
} else {
    "Ok(content)".to_string()
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let path = format!(\"{}/{}\", self.base_path, \"".to_string(), emit_ident(&op_name, RenderTarget::Rust)), "\");\n".to_string()), "let content = std::fs::read_to_string(&path)?;\n".to_string()), parse_line.clone())
}

pub fn emit_local_call(op_name: &str) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat("// Local binding -- direct function call\n".to_string(), "Ok(".to_string()), emit_ident(&op_name, RenderTarget::Rust)), "())".to_string())
}

pub fn emit_resource_def(item: Rc<Node>) -> String {
    let cap_children = item.children.clone();
    if ({
    let __len_5 = cap_children.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat("#[derive(Debug, Clone)]\npub struct ".to_string(), item.name.clone()), ";".to_string())
} else {
    let depth = 0_i64;
    let cap_methods = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in cap_children.iter().cloned() {
        __mapped_0.push(emit_capability_method(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let methods_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in cap_methods.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("#[async_trait::async_trait]\npub trait ".to_string(), item.name.clone()), " {\n".to_string()), make_indent(depth.clone() + 1_i64)), methods_str.clone()), "\n}".to_string())
}
}

pub fn emit_capability_method(cap_node: Rc<Node>) -> String {
    let input_params = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in cap_node.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(emit_ident(&__elem_1.name, RenderTarget::Rust), ": ".to_string()), emit_rust_param_type(__elem_1.type_expr.clone(), Rc::new(std::collections::HashMap::new()))));
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
    "&self".to_string()
} else {
    v2_rt::concat("&self, ".to_string(), params_str.clone())
};
    let ret = emit_node_type_rc(rt_type(cap_node.clone()), RenderTarget::Rust, Rc::new(std::collections::HashMap::new()));
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("async fn ".to_string(), emit_ident(&cap_node.name, RenderTarget::Rust)), "(".to_string()), all_params.clone()), ") -> Result<".to_string()), ret), ", Box<dyn std::error::Error>>;".to_string())
}

pub fn emit_data_def(name: &str, type_node: Rc<Node>, value: Rc<Node>, registry: Rc<HashMap<String, Rc<ItemInfo>>>, scope: Rc<InferScope>, depth: i64, vtoe: Rc<HashMap<String, String>>, rc_types: Rc<HashMap<String, bool>>, emit_info: Rc<EmitGraphInfo>) -> String {
    let ty_str = emit_node_type_rc(type_node.clone(), RenderTarget::Rust, Rc::new(std::collections::HashMap::new()));
    let upper_name = to_screaming_snake(&name);
    if is_simple_type_node(type_node.clone()) {
    let val_str = emit_typed_expr(value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), Rc::new(std::collections::HashMap::new()), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "const ".to_string()), upper_name), ": ".to_string()), ty_str), " = ".to_string()), val_str.clone()), ";".to_string())
} else {
    if has_nested_records_node(type_node.clone()) {
    let json_str = emit_data_value_json(value.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_visibility_prefix(), "fn ".to_string()), to_snake(&name)), "() -> ".to_string()), ty_str), " {\n".to_string()), "    serde_json::from_value(serde_json::json!(".to_string()), json_str), "))\n".to_string()), "        .expect(\"valid data definition\")\n".to_string()), "}".to_string())
} else {
    if node_is_map(type_node.clone()) {
    match value.expr_data.as_ref() {
    ExprData::ExprRecordLit { fields: fs, type_name: _, parent_enum: _, .. } => {
        {
    let inserts = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in fs.iter().cloned() {
        __mapped_0.push({
    let val_str = emit_typed_expr(__elem_1.value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), Rc::new(std::collections::HashMap::new()), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("        __m.insert(\"".to_string(), __elem_1.name.clone()), "\".to_string(), ".to_string()), val_str.clone()), ");".to_string())
});
    }
    Rc::new(__mapped_0)
};
    let inserts_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in inserts.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("lazy_static::lazy_static! {\n".to_string(), "    ".to_string()), rust_visibility_prefix()), "static ref ".to_string()), upper_name), ": ".to_string()), ty_str), " = {\n".to_string()), "        let mut __m = BTreeMap::new();\n".to_string()), inserts_str.clone()), "\n".to_string()), "        __m\n".to_string()), "    };\n".to_string()), "}".to_string())
}
    }
    _ => {
        {
    let val_str = emit_typed_expr(value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), Rc::new(std::collections::HashMap::new()), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("lazy_static::lazy_static! {\n".to_string(), "    ".to_string()), rust_visibility_prefix()), "static ref ".to_string()), upper_name), ": ".to_string()), ty_str), " = ".to_string()), val_str.clone()), ";\n".to_string()), "}".to_string())
}
    }
}
} else {
    let val_str = emit_typed_expr(value.clone(), registry.clone(), scope.clone(), depth.clone(), vtoe.clone(), Rc::new(std::collections::HashMap::new()), emit_info.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("lazy_static::lazy_static! {\n".to_string(), "    ".to_string()), rust_visibility_prefix()), "static ref ".to_string()), upper_name), ": ".to_string()), ty_str), " = ".to_string()), val_str.clone()), ";\n".to_string()), "}".to_string())
}
}
}
}

pub fn is_simple_type_node(n: Rc<Node>) -> bool {
    (is_int_type_node(n.clone()) || is_bool_type_node(n.clone())) || is_float_type_node(n.clone())
}

pub fn emit_test_file(module_name: &str, projections: Rc<Vec<Rc<TestProjection>>>) -> Rc<TextFile> {
    let depth = 0_i64;
    let test_fns = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projections.iter().cloned() {
        __mapped_0.push(emit_operation_test(__elem_1.clone(), depth.clone() + 1_i64));
    }
    Rc::new(__mapped_0)
};
    if ({
    let __len_5 = test_fns.clone().len();
    __len_5 as i64
}) == 0_i64 {
    Rc::new(TextFile { path: "".to_string(), content: "".to_string() })
} else {
    let filename = module_to_filename(&module_name);
    let tests_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in test_fns.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Generated tests -- do not edit.\n".to_string(), "// Source module: ".to_string()), module_name.to_string()), "\n\n".to_string()), "#[cfg(test)]\nmod tests {\n".to_string()), "    use super::*;\n\n".to_string()), make_indent(depth.clone() + 1_i64)), tests_str.clone()), "\n".to_string()), "}\n".to_string());
    Rc::new(TextFile { path: rust_test_file_path(&module_name), content: content.clone() })
}
}

pub fn rust_test_signature_comment(projection: Rc<TestProjection>) -> String {
    let params_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in ({
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projection.params.iter().cloned() {
        __mapped_0.push(v2_rt::concat(v2_rt::concat(__elem_1.name.clone(), ": ".to_string()), emit_rust_param_type(__elem_1.type_expr.clone(), Rc::new(std::collections::HashMap::new()))));
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
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Signature: ".to_string(), sanitize_service_name(&projection.service_name)), ".".to_string()), projection.operation_name.clone()), "(".to_string()), params_str.clone()), ") -> ".to_string()), emit_node_type(projection.return_type.clone(), RenderTarget::Rust))
}

pub fn emit_operation_test(projection: Rc<TestProjection>, depth: i64) -> String {
    let test_name = rust_test_name(projection.clone());
    let mock_setup = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in projection.mock_field_inits.iter().cloned() {
        __mapped_0.push(emit_mock_prop_setup(__elem_1.clone(), depth.clone() + 1_i64));
    }
    Rc::new(__mapped_0)
};
    let mock_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in mock_setup.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(rust_async_test_decorator(), "\nasync fn ".to_string()), test_name), "() {\n".to_string()), make_indent(depth.clone() + 1_i64)), rust_test_signature_comment(projection.clone())), "\n".to_string()), make_indent(depth.clone() + 1_i64)), mock_str.clone()), "\n".to_string()), make_indent(depth.clone() + 1_i64)), "// Mock-based test: verifies operation contract with fixture data.".to_string()), "\n".to_string()), "}".to_string())
}

pub fn emit_mock_prop_setup(mock_prop: Rc<FieldInit>, depth: i64) -> String {
    let body_str = emit_simple_expr(mock_prop.value.clone(), RenderTarget::Rust);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let ".to_string(), mock_prop.name.clone()), " = ".to_string()), body_str), ";".to_string())
}

pub fn emit_cargo_toml(crate_name: &str, has_services: bool) -> Rc<TextFile> {
    let header = v2_rt::concat(v2_rt::concat("[package]\nname = \"".to_string(), crate_name.to_string()), "\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_string());
    let deps = "\n[dependencies]\nserde = { version = \"1\", features = [\"derive\", \"rc\"] }\nserde_json = \"1\"\nstacker = \"0.1\"\n".to_string();
    let cli_dep = "clap = { version = \"4\", features = [\"derive\"] }\n".to_string();
    let async_deps = if has_services {
    "tokio = { version = \"1\", features = [\"full\"] }\nreqwest = { version = \"0.12\", features = [\"json\"] }\nasync-trait = \"0.1\"\n".to_string()
} else {
    "".to_string()
};
    let lazy_dep = "lazy_static = \"1\"\n".to_string();
    Rc::new(TextFile { path: "Cargo.toml".to_string(), content: v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(header.clone(), deps.clone()), cli_dep.clone()), async_deps.clone()), lazy_dep.clone()) })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowFunc {
    pub name: String,
    pub module_name: String,
    pub params: Rc<Vec<Rc<Param>>>,
    pub return_type: Rc<Node>,
    pub uses: Rc<Vec<Rc<ResourceUse>>>,
    pub service_names: Rc<Vec<String>>,
}

pub fn to_workflow_func(item: Rc<Node>, module_name: &str, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> Rc<WorkflowFunc> {
    let svc_names = match lookup_item(registry.clone(), &item.name) {
    Some(info) => {
        info.service_names.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    Rc::new(WorkflowFunc { name: item.name.clone(), module_name: module_name.to_string(), params: item.params.clone(), return_type: rt_type(item.clone()), uses: item.uses.clone(), service_names: svc_names.clone() })
}

pub fn collect_workflow_funcs(modules: Rc<Vec<Rc<TypedModule>>>, registry: Rc<HashMap<String, Rc<ItemInfo>>>) -> Rc<Vec<Rc<WorkflowFunc>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in modules.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __mapped_5 = Vec::new();
    for __elem_6 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in __elem_1.items.iter().cloned() {
        if (__elem_3.body.clone().is_some()) && (({
    let __len_4 = __elem_3.uses.clone().len();
    __len_4 as i64
}) > 0_i64) {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_5.push(to_workflow_func(__elem_6.clone(), &__elem_1.module.name, registry.clone()));
    }
    Rc::new(__mapped_5)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn cli_default_literal_value(expr: Rc<Node>) -> Option<String> {
    match expr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: v, .. } => {
        match v.as_ref() {
    LiteralValue::LitStr { value: s, .. } => {
        Some(s.clone())
    }
    LiteralValue::LitInt { value: i, .. } => {
        Some(v2_rt::to_string(i.clone()))
    }
    LiteralValue::LitFloat { value: f, .. } => {
        Some(f.clone())
    }
    LiteralValue::LitBool { value: b, .. } => {
        Some(if b.clone() {
    "true".to_string()
} else {
    "false".to_string()
})
    }
    LiteralValue::LitNull => {
        None
    }
}
    }
    _ => {
        None
    }
}
}

pub fn validate_workflow_param_defaults(workflow_funcs: Rc<Vec<Rc<WorkflowFunc>>>) -> Rc<Vec<Rc<Diagnostic>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in workflow_funcs.iter().cloned() {
        __flat_mapped_0.extend(({
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in __elem_1.params.iter().cloned() {
        __flat_mapped_2.extend((match __elem_3.default_value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(dv) => {
        let dv = Rc::new(dv.clone());
        match cli_default_literal_value(dv.clone()) {
    Some(_) => {
        Rc::new(Vec::new())
    }
    None => {
        Rc::new(vec!(Rc::new(Diagnostic { severity: Severity::Error, message: v2_rt::concat(v2_rt::concat("workflow CLI default for parameter `".to_string(), __elem_3.name.clone()), "` must be a string, int, float, or bool literal".to_string()), span: Some(__elem_3.span.clone()), module_name: Some(__elem_1.module_name.clone()), category: None })))
    }
}
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn find_resource_module(resource_name: &str, modules: Rc<Vec<Rc<TypedModule>>>) -> String {
    let matching = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in modules.iter().cloned() {
        {
let __cond = {
    let mut __any_2 = false;
    for __elem_3 in __elem_1.items.iter().cloned() {
        if __elem_3.name.clone() == resource_name {
    __any_2 = true;
    break;
};
    }
    __any_2
};
if __cond {
    __filtered_0.push(__elem_1);
}
};
    }
    Rc::new(__filtered_0)
};
    let result = match matching.clone().first().cloned() {
    Some(tm) => {
        module_to_filename(&tm.module.name)
    }
    None => {
        "".to_string()
    }
};
    result.clone()
}

pub fn emit_main_rs(workflow_funcs: Rc<Vec<Rc<WorkflowFunc>>>, modules: Rc<Vec<Rc<TypedModule>>>, has_services: bool) -> Rc<TextFile> {
    let has_pipeline = {
    let mut __any_0 = false;
    for __elem_1 in modules.iter().cloned() {
        if __elem_1.module.name.clone() == "v2.compiler.compile" {
    __any_0 = true;
    break;
};
    }
    __any_0
};
    let resource_type_names = {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in workflow_funcs.iter().cloned() {
        __flat_mapped_2.extend(({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in __elem_3.uses.iter().cloned() {
        __mapped_4.push(__elem_5.resource.name.clone());
    }
    Rc::new(__mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
};
    let unique_resource_types = unique_strings(resource_type_names.clone());
    let resource_imports = {
    let mut __flat_mapped_6 = Vec::new();
    for __elem_7 in unique_resource_types.iter().cloned() {
        __flat_mapped_6.extend(({
    let mod_name = find_resource_module(&__elem_7, modules.clone());
    if mod_name.clone() != "" {
    Rc::new(vec!(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("use v2_compiled::".to_string(), mod_name.clone()), "::".to_string()), __elem_7.clone()), ";".to_string())))
} else {
    Rc::new(Vec::new())
}
}).iter().cloned());
    }
    Rc::new(__flat_mapped_6)
};
    let resource_imports_str = {
    let mut __joined_8 = String::new();
    let mut __first_10 = true;
    for __elem_9 in resource_imports.iter().cloned() {
        if !__first_10 {
    __joined_8.push_str(&"\n".to_string());
};
        __first_10 = false;
        __joined_8.push_str(&__elem_9);
    }
    __joined_8
};
    let header = v2_rt::concat("// Generated by v2 compiler -- do not edit.\n\n".to_string(), v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("use clap::".to_string(), "{".to_string()), "Parser, Subcommand".to_string()), "}".to_string()), ";\n".to_string()));
    let crate_use = if has_services.clone() {
    "use v2_compiled::dry_run::DryRunMode;\n".to_string()
} else {
    "".to_string()
};
    let resource_use = if resource_imports_str.clone() != "" {
    v2_rt::concat(resource_imports_str.clone(), "\n".to_string())
} else {
    "".to_string()
};
    let mod_uses = emit_main_mod_uses(workflow_funcs.clone(), has_pipeline.clone());
    let cli_struct = emit_cli_struct(workflow_funcs.clone());
    let subcommand_enum = emit_subcommand_enum(workflow_funcs.clone(), has_pipeline.clone());
    let main_fn = emit_main_fn(workflow_funcs.clone(), has_services.clone(), has_pipeline.clone());
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(header.clone(), crate_use.clone()), resource_use.clone()), mod_uses), "\n".to_string()), cli_struct), "\n\n".to_string()), subcommand_enum), "\n\n".to_string()), main_fn), "\n".to_string());
    Rc::new(TextFile { path: v2_rt::concat(v2_rt::concat(rust_source_root(), "main".to_string()), rust_source_ext()), content: content.clone() })
}

pub fn emit_main_mod_uses(workflow_funcs: Rc<Vec<Rc<WorkflowFunc>>>, has_pipeline: bool) -> String {
    let mod_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in workflow_funcs.iter().cloned() {
        __mapped_0.push(module_to_filename(&__elem_1.module_name));
    }
    Rc::new(__mapped_0)
};
    let unique_mods = unique_strings(mod_names.clone());
    let use_lines = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in unique_mods.iter().cloned() {
        __mapped_2.push(v2_rt::concat(v2_rt::concat("use v2_compiled::".to_string(), __elem_3.clone()), ";".to_string()));
    }
    Rc::new(__mapped_2)
};
    let base = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in use_lines.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"\n".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    if has_pipeline {
    let pipeline_mod = module_to_filename("v2.compiler.compile");
    let artifact_mod = module_to_filename("v2.compiler.artifact");
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(base.clone(), "\nuse std::rc::Rc;\nuse v2_compiled::".to_string()), pipeline_mod), ";\nuse v2_compiled::".to_string()), artifact_mod), ";\n".to_string())
} else {
    base.clone()
}
}

pub fn emit_cli_struct(workflow_funcs: Rc<Vec<Rc<WorkflowFunc>>>) -> String {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("#[derive(Parser)]\n".to_string(), "#[command(name = \"v2-compiled\", about = \"Generated CLI from DAG compiler\")]\n".to_string()), "struct Cli {\n".to_string()), "    #[command(subcommand)]\n".to_string()), "    command: Commands,\n".to_string()), "    /// Run in dry-run mode (mock all service calls)\n".to_string()), "    #[arg(long, global = true)]\n".to_string()), "    dry_run: bool,\n".to_string()), "}".to_string())
}

pub fn emit_subcommand_enum(workflow_funcs: Rc<Vec<Rc<WorkflowFunc>>>, has_pipeline: bool) -> String {
    let depth = 0_i64;
    let variants = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in workflow_funcs.iter().cloned() {
        __mapped_0.push(emit_subcommand_variant(__elem_1.clone(), depth.clone() + 1_i64));
    }
    Rc::new(__mapped_0)
};
    let compile_variant = if has_pipeline.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("/// Compile .dag source files to a target language\n".to_string(), make_indent(depth.clone() + 1_i64)), "Compile {\n".to_string()), make_indent(depth.clone() + 2_i64)), "#[arg(long)]\n".to_string()), make_indent(depth.clone() + 2_i64)), "source_dir: String,\n".to_string()), make_indent(depth.clone() + 2_i64)), "#[arg(long)]\n".to_string()), make_indent(depth.clone() + 2_i64)), "output_dir: String,\n".to_string()), make_indent(depth.clone() + 2_i64)), "/// Target language: rust, python, go, dag\n".to_string()), make_indent(depth.clone() + 2_i64)), "#[arg(long, default_value = \"rust\")]\n".to_string()), make_indent(depth.clone() + 2_i64)), "target: String,\n".to_string()), make_indent(depth.clone() + 1_i64)), "},".to_string())
} else {
    "".to_string()
};
    let all_variants = if has_pipeline.clone() {
    {
    let __rc_3 = variants;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(compile_variant.clone());
    Rc::new(__appended_2)
}
} else {
    variants.clone()
};
    let variants_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in all_variants.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"\n".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("#[derive(Subcommand)]\n".to_string(), "enum Commands {\n".to_string()), make_indent(depth.clone() + 1_i64)), variants_str.clone()), "\n".to_string()), "}".to_string())
}

pub fn emit_subcommand_variant(wf: Rc<WorkflowFunc>, depth: i64) -> String {
    let variant_name = capitalize_first(&wf.name);
    let doc_line = v2_rt::concat(v2_rt::concat("/// Run the ".to_string(), wf.name.clone()), " workflow".to_string());
    if ({
    let __len_5 = wf.params.clone().len();
    __len_5 as i64
}) == 0_i64 {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(doc_line.clone(), "\n".to_string()), variant_name), ",".to_string())
} else {
    let fields = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in wf.params.iter().cloned() {
        __mapped_0.push(emit_subcommand_field(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let fields_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in fields.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(doc_line.clone(), "\n".to_string()), variant_name), " {\n".to_string()), make_indent(depth + 1_i64)), fields_str.clone()), "\n".to_string()), "},".to_string())
}
}

pub fn emit_subcommand_field(param: Rc<Param>) -> String {
    let field_name = emit_ident(&param.name, RenderTarget::Rust);
    let field_type = emit_cli_param_type_node(param.type_expr.clone());
    let default_attr = match param.default_value.as_ref().map(|__rc| __rc.as_ref()) {
    Some(dv) => {
        let dv = Rc::new(dv.clone());
        match cli_default_literal_value(dv.clone()) {
    Some(default_value) => {
        v2_rt::concat(v2_rt::concat("#[arg(long, default_value = \"".to_string(), default_value), "\")]\n".to_string())
    }
    None => {
        "#[arg(long)]\n".to_string()
    }
}
    }
    None => {
        "#[arg(long)]\n".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(default_attr.clone(), field_name), ": ".to_string()), field_type), ",".to_string())
}

pub fn emit_cli_param_type_node(n: Rc<Node>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if node_is_optional(n.clone()) {
    v2_rt::concat(v2_rt::concat("Option<".to_string(), emit_cli_param_type_node(with_required_cardinality(n.clone()))), ">".to_string())
} else {
    if node_has_structure(n.clone()) {
    "String".to_string()
} else {
    if ({
    let __len_0 = n.children.clone().len();
    __len_0 as i64
}) == 0_i64 {
    if is_bool_type_node(n.clone()) {
    "bool".to_string()
} else {
    if is_int_type_node(n.clone()) {
    "i64".to_string()
} else {
    if is_float_type_node(n.clone()) {
    "f64".to_string()
} else {
    "String".to_string()
}
}
}
} else {
    "String".to_string()
}
}
}
    })
}

pub fn emit_main_fn(workflow_funcs: Rc<Vec<Rc<WorkflowFunc>>>, has_services: bool, has_pipeline: bool) -> String {
    let depth = 0_i64;
    let async_attr = if has_services.clone() {
    "#[tokio::main]\nasync fn main() ".to_string()
} else {
    "fn main() ".to_string()
};
    let parse_line = "let cli = Cli::parse();".to_string();
    let dry_run_line = if has_services.clone() {
    "let dry_run = DryRunMode(cli.dry_run);".to_string()
} else {
    "".to_string()
};
    let match_arms = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in workflow_funcs.iter().cloned() {
        __mapped_0.push(emit_main_match_arm(__elem_1.clone(), has_services.clone()));
    }
    Rc::new(__mapped_0)
};
    let compile_arm = if has_pipeline.clone() {
    emit_compile_match_arm()
} else {
    "".to_string()
};
    let all_arms = if has_pipeline.clone() {
    {
    let __rc_3 = match_arms;
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(compile_arm.clone());
    Rc::new(__appended_2)
}
} else {
    match_arms.clone()
};
    let arms_str = {
    let mut __joined_4 = String::new();
    let mut __first_6 = true;
    for __elem_5 in all_arms.iter().cloned() {
        if !__first_6 {
    __joined_4.push_str(&"\n".to_string());
};
        __first_6 = false;
        __joined_4.push_str(&__elem_5);
    }
    __joined_4
};
    let match_block = v2_rt::concat(v2_rt::concat(v2_rt::concat("match cli.command {\n".to_string(), make_indent(depth.clone() + 2_i64)), arms_str.clone()), "\n    }".to_string());
    let result_handling = if has_services.clone() {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("let result = ".to_string(), match_block.clone()), ";\n".to_string()), "match result {\n".to_string()), "    Ok(value) => {\n".to_string()), "        let json = serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!(\"{:?}\", value));\n".to_string()), "        println!(\"{}\", json);\n".to_string()), "    }\n".to_string()), "    Err(e) => {\n".to_string()), "        eprintln!(\"Error: {}\", e);\n".to_string()), "        std::process::exit(1);\n".to_string()), "    }\n".to_string()), "}".to_string())
} else {
    v2_rt::concat(v2_rt::concat("let _result = ".to_string(), match_block.clone()), ";".to_string())
};
    let body_lines = if dry_run_line.clone() == "" {
    v2_rt::concat(v2_rt::concat(parse_line.clone(), "\n".to_string()), result_handling.clone())
} else {
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(parse_line.clone(), "\n".to_string()), dry_run_line.clone()), "\n".to_string()), result_handling.clone())
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(async_attr.clone(), "{\n".to_string()), make_indent(depth.clone() + 1_i64)), body_lines.clone()), "\n}".to_string())
}

pub fn emit_compile_match_arm() -> String {
    let pipeline_mod = module_to_filename("v2.compiler.compile");
    let artifact_mod = module_to_filename("v2.compiler.artifact");
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Commands::Compile { source_dir, output_dir, target } => {\n".to_string(), "            let render_target = match target.as_str() {\n".to_string()), "                \"rust\" => ".to_string()), artifact_mod.clone()), "::RenderTarget::Rust,\n".to_string()), "                \"python\" => ".to_string()), artifact_mod.clone()), "::RenderTarget::Python,\n".to_string()), "                \"go\" => ".to_string()), artifact_mod.clone()), "::RenderTarget::Go,\n".to_string()), "                \"dag\" => ".to_string()), artifact_mod.clone()), "::RenderTarget::Dag,\n".to_string()), "                other => {\n".to_string()), "                    eprintln!(\"unknown target: {}. supported: rust, python, go, dag\", other);\n".to_string()), "                    std::process::exit(1);\n".to_string()), "                }\n".to_string()), "            };\n".to_string()), "            let mut sources: Vec<Rc<".to_string()), pipeline_mod.clone()), "::SourceFile>> = Vec::new();\n".to_string()), "            let mut dag_paths: Vec<std::path::PathBuf> = Vec::new();\n".to_string()), "            fn collect_dag_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {\n".to_string()), "                let mut entries: Vec<_> = std::fs::read_dir(dir)\n".to_string()), "                    .unwrap_or_else(|e| panic!(\"failed to read dir {:?}: {}\", dir, e))\n".to_string()), "                    .filter_map(|e| e.ok())\n".to_string()), "                    .collect();\n".to_string()), "                entries.sort_by_key(|e| e.file_name());\n".to_string()), "                for entry in entries {\n".to_string()), "                    let path = entry.path();\n".to_string()), "                    if path.is_dir() {\n".to_string()), "                        collect_dag_files(&path, files);\n".to_string()), "                    } else if path.extension().map(|e| e == \"dag\").unwrap_or(false) {\n".to_string()), "                        files.push(path);\n".to_string()), "                    }\n".to_string()), "                }\n".to_string()), "            }\n".to_string()), "            collect_dag_files(std::path::Path::new(&source_dir), &mut dag_paths);\n".to_string()), "            for path in &dag_paths {\n".to_string()), "                let content = std::fs::read_to_string(path)\n".to_string()), "                    .unwrap_or_else(|e| panic!(\"failed to read {:?}: {}\", path, e));\n".to_string()), "                let filename = path.file_name().unwrap().to_string_lossy().to_string();\n".to_string()), "                sources.push(Rc::new(".to_string()), pipeline_mod.clone()), "::SourceFile {\n".to_string()), "                    path: filename,\n".to_string()), "                    content,\n".to_string()), "                }));\n".to_string()), "            }\n".to_string()), "            eprintln!(\"compiling {} .dag files from {} (target: {})\", sources.len(), source_dir, target);\n".to_string()), "            let result = ".to_string()), pipeline_mod.clone()), "::compile_sources(\n".to_string()), "                sources,\n".to_string()), "                render_target,\n".to_string()), "            );\n".to_string()), "            std::fs::create_dir_all(format!(\"{}/src\", output_dir))\n".to_string()), "                .unwrap_or_else(|e| panic!(\"failed to create output dir: {}\", e));\n".to_string()), "            for file in result.files.iter() {\n".to_string()), "                let out_path = format!(\"{}/{}\", output_dir, file.path);\n".to_string()), "                if let Some(parent) = std::path::Path::new(&out_path).parent() {\n".to_string()), "                    std::fs::create_dir_all(parent).ok();\n".to_string()), "                }\n".to_string()), "                std::fs::write(&out_path, &*file.content)\n".to_string()), "                    .unwrap_or_else(|e| panic!(\"failed to write {}: {}\", file.path, e));\n".to_string()), "            }\n".to_string()), "            eprintln!(\"compiled: {} files emitted, {} diagnostics\",\n".to_string()), "                result.files.len(), result.diagnostics.len());\n".to_string()), "            for (i, d) in result.diagnostics.iter().take(20).enumerate() {\n".to_string()), "                eprintln!(\"  [{}]: {:?}\", i, d);\n".to_string()), "            }\n".to_string()), "            if result.files.is_empty() {\n".to_string()), "                eprintln!(\"error: no files emitted\");\n".to_string()), "                std::process::exit(1);\n".to_string()), "            }\n".to_string()), "        },".to_string())
}

pub fn emit_main_match_arm(wf: Rc<WorkflowFunc>, has_services: bool) -> String {
    let variant_name = capitalize_first(&wf.name);
    let mod_name = module_to_filename(&wf.module_name);
    let fn_name = emit_ident(&wf.name, RenderTarget::Rust);
    let await_suffix = if has_services.clone() {
    ".await".to_string()
} else {
    "".to_string()
};
    if ({
    let __len_5 = wf.params.clone().len();
    __len_5 as i64
}) == 0_i64 {
    let service_args = emit_main_service_args(wf.clone(), has_services.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Commands::".to_string(), variant_name), " => ".to_string()), mod_name), "::".to_string()), fn_name), "(".to_string()), service_args), ")".to_string()), await_suffix.clone()), ",".to_string())
} else {
    let field_binds = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in wf.params.iter().cloned() {
        __mapped_0.push(emit_ident(&__elem_1.name, RenderTarget::Rust));
    }
    Rc::new(__mapped_0)
};
    let binds_str = {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in field_binds.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&", ".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
};
    let call_args = emit_main_call_args(wf.clone(), has_services.clone());
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("Commands::".to_string(), variant_name), " { ".to_string()), binds_str.clone()), " } => ".to_string()), mod_name), "::".to_string()), fn_name), "(".to_string()), call_args), ")".to_string()), await_suffix.clone()), ",".to_string())
}
}

pub fn emit_main_call_args(wf: Rc<WorkflowFunc>, has_services: bool) -> String {
    let param_args = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in wf.params.iter().cloned() {
        __mapped_0.push({
    let arg_name = emit_ident(&__elem_1.name, RenderTarget::Rust);
    if needs_reference_node(__elem_1.type_expr.clone()) {
    v2_rt::concat("&".to_string(), arg_name.clone())
} else {
    arg_name.clone()
}
});
    }
    Rc::new(__mapped_0)
};
    let service_args = emit_main_service_arg_list(wf.clone(), has_services);
    let all_args = v2_rt::concat(param_args.clone(), service_args.clone());
    {
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
}
}

pub fn emit_main_service_args(wf: Rc<WorkflowFunc>, has_services: bool) -> String {
    let service_args = emit_main_service_arg_list(wf.clone(), has_services);
    {
    let mut __joined_0 = String::new();
    let mut __first_2 = true;
    for __elem_1 in service_args.iter().cloned() {
        if !__first_2 {
    __joined_0.push_str(&", ".to_string());
};
        __first_2 = false;
        __joined_0.push_str(&__elem_1);
    }
    __joined_0
}
}

pub fn emit_main_service_arg_list(wf: Rc<WorkflowFunc>, has_services: bool) -> Rc<Vec<String>> {
    let resource_args = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in wf.uses.iter().cloned() {
        __mapped_0.push(v2_rt::concat("&".to_string(), __elem_1.resource.name.clone()));
    }
    Rc::new(__mapped_0)
};
    let svc_args = {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in wf.service_names.iter().cloned() {
        __mapped_2.push({
    let struct_name = sanitize_service_name(&__elem_3);
    if has_services.clone() {
    v2_rt::concat(v2_rt::concat("&".to_string(), struct_name.clone()), "::new(dry_run.clone())".to_string())
} else {
    v2_rt::concat(v2_rt::concat("&".to_string(), struct_name.clone()), "::new()".to_string())
}
});
    }
    Rc::new(__mapped_2)
};
    v2_rt::concat(resource_args.clone(), svc_args.clone())
}

pub fn emit_dry_run_module() -> Rc<TextFile> {
    let content = v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("// Generated by v2 compiler -- do not edit.\n".to_string(), "//\n".to_string()), "// Dry-run support: when DryRunMode(true), service methods return\n".to_string()), "// mock data instead of performing real I/O.\n\n".to_string()), "#[derive(Debug, Clone)]\n".to_string()), "pub struct DryRunMode(pub bool);\n\n".to_string()), "impl DryRunMode {\n".to_string()), "    pub fn is_dry_run(&self) -> bool {\n".to_string()), "        self.0\n".to_string()), "    }\n".to_string()), "}\n\n".to_string()), "impl Default for DryRunMode {\n".to_string()), "    fn default() -> Self {\n".to_string()), "        DryRunMode(false)\n".to_string()), "    }\n".to_string()), "}\n".to_string());
    Rc::new(TextFile { path: v2_rt::concat(v2_rt::concat(rust_source_root(), "dry_run".to_string()), rust_source_ext()), content: content.clone() })
}

