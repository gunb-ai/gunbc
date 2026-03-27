use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleGraph {
    pub modules: Rc<Vec<Rc<ResolvedModule>>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedModule {
    pub module: Rc<Node>,
    pub resolved_imports: Rc<Vec<Rc<ResolvedImport>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedImport {
    pub module_path: String,
    pub target_span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DepEdge {
    pub from_module: String,
    pub to_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolveAccum {
    pub imports_by_name: Rc<HashMap<String, Rc<Vec<Rc<ResolvedImport>>>>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn map_has(m: Rc<HashMap<String, bool>>, key: &str) -> bool {
    match m.get(&key.to_string()).cloned() {
    Some(_) => {
        true
    }
    None => {
        false
    }
}
}

pub fn resolve_modules(modules: Rc<Vec<Rc<Node>>>) -> Rc<ModuleGraph> {
    let dup_diags = check_duplicate_modules(modules.clone());
    let module_index = {
    let mut __acc_0: Rc<std::collections::HashMap<String, Rc<Node>>> = Rc::new(std::collections::HashMap::new());
    for __elem_1 in modules.iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.name.clone(), __elem_1.clone());
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
    let export_sets = {
    let mut __acc_4 = Rc::new(std::collections::HashMap::new());
    for __elem_5 in modules.iter().cloned() {
        __acc_4 = {
    let exported = get_exported_names(__elem_5.clone());
    let exported_set = {
    let mut __acc_6 = Rc::new(std::collections::HashMap::new());
    for __elem_7 in exported.iter().cloned() {
        __acc_6 = {
    let __rc_9 = __acc_6;
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert(__elem_7, true);
    Rc::new(__map_ins_8)
};
    }
    __acc_6
};
    {
    let __rc_11 = __acc_4;
    let mut __map_ins_10 = Rc::try_unwrap(__rc_11).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_10.insert(__elem_5.name.clone(), exported_set.clone());
    Rc::new(__map_ins_10)
}
};
    }
    __acc_4
};
    let resolve_accum = {
    let mut __acc_12 = Rc::new(ResolveAccum { imports_by_name: Rc::new(std::collections::HashMap::new()), diagnostics: Rc::new(Vec::new()) });
    for __elem_13 in modules.iter().cloned() {
        __acc_12 = {
    let result = resolve_module_imports(__elem_13.clone(), module_index.clone(), export_sets.clone());
    Rc::new(ResolveAccum { imports_by_name: {
    let __rc_15 = std::mem::take(&mut Rc::make_mut(&mut __acc_12).imports_by_name);
    let mut __map_ins_14 = Rc::try_unwrap(__rc_15).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_14.insert(__elem_13.name.clone(), result.resolved_imports.clone());
    Rc::new(__map_ins_14)
}, diagnostics: v2_rt::concat(__acc_12.diagnostics.clone(), result.diagnostics.clone()) })
};
    }
    __acc_12
};
    let imports_by_name = resolve_accum.imports_by_name.clone();
    let import_diags = resolve_accum.diagnostics.clone();
    let topo_result = topological_sort(modules.clone());
    let topo_diags = match topo_result.cycle_error.as_ref().map(|__rc| __rc.as_ref()) {
    Some(diag) => {
        let diag = Rc::new(diag.clone());
        Rc::new(vec!(diag.clone()))
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let sorted_names = topo_result.sorted.clone();
    let acyclic_resolved = {
    let mut __flat_mapped_16 = Vec::new();
    for __elem_17 in sorted_names.iter().cloned() {
        __flat_mapped_16.extend((match module_index.clone().get(&__elem_17.clone()).cloned() {
    Some(m) => {
        match imports_by_name.clone().get(&__elem_17.clone()).cloned() {
    Some(imps) => {
        Rc::new(vec!(Rc::new(ResolvedModule { module: m.clone(), resolved_imports: imps.clone() })))
    }
    None => {
        Rc::new(Vec::new())
    }
}
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_16)
};
    Rc::new(ModuleGraph { modules: acyclic_resolved, diagnostics: v2_rt::concat(v2_rt::concat(dup_diags, import_diags), topo_diags) })
}

pub fn find_module(module_index: Rc<HashMap<String, Rc<Node>>>, path: &str) -> Option<Rc<Node>> {
    module_index.get(&path.to_string()).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleResolveResult {
    pub resolved_imports: Rc<Vec<Rc<ResolvedImport>>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn resolve_module_imports(module: Rc<Node>, module_index: Rc<HashMap<String, Rc<Node>>>, export_sets: Rc<HashMap<String, Rc<HashMap<String, bool>>>>) -> Rc<ModuleResolveResult> {
    let results = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in module_imports(module.clone()).iter().cloned() {
        __mapped_0.push(resolve_import(__elem_1.clone(), module_index.clone(), &module.name, export_sets.clone()));
    }
    Rc::new(__mapped_0)
};
    let resolved = {
    let mut __mapped_5 = Vec::new();
    for __elem_6 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in results.iter().cloned() {
        if (__elem_3.resolved.target_span.clone().is_some()) && (({
    let __len_4 = __elem_3.diagnostics.clone().len();
    __len_4 as i64
}) == 0_i64) {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_5.push(__elem_6.resolved.clone());
    }
    Rc::new(__mapped_5)
};
    let diags = {
    let mut __flat_mapped_7 = Vec::new();
    for __elem_8 in results.iter().cloned() {
        __flat_mapped_7.extend(__elem_8.diagnostics.clone().iter().cloned());
    }
    Rc::new(__flat_mapped_7)
};
    Rc::new(ModuleResolveResult { resolved_imports: resolved, diagnostics: diags })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportResolveResult {
    pub resolved: Rc<ResolvedImport>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn resolve_import(import: Rc<Node>, module_index: Rc<HashMap<String, Rc<Node>>>, importing_module: &str, export_sets: Rc<HashMap<String, Rc<HashMap<String, bool>>>>) -> Rc<ImportResolveResult> {
    let target = find_module(module_index, &import.name);
    match target.as_ref().map(|__rc| __rc.as_ref()) {
    None => {
        {
    let diag = diagnostic_node("error", &v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("unresolved import: module '".to_string(), import.name.clone()), "' not found (imported by '".to_string()), importing_module.to_string()), "')".to_string()), import.span.clone(), Some(importing_module.to_string()), Some("unresolved_name".to_string()));
    Rc::new(ImportResolveResult { resolved: Rc::new(ResolvedImport { module_path: import.name.clone(), target_span: None }), diagnostics: Rc::new(vec!(diag)) })
}
    }
    Some(target_mod) => {
        let target_mod = Rc::new(target_mod.clone());
        {
    let exported_set = match export_sets.get(&import.name.clone()).cloned() {
    Some(set) => {
        set
    }
    None => {
        Rc::new(std::collections::HashMap::new())
    }
};
    let name_diags = if import_is_all(import.clone()) {
    Rc::new(Vec::new())
} else {
    {
    let mut __mapped_2 = Vec::new();
    for __elem_3 in ({
    let mut __filtered_0 = Vec::new();
    for __elem_1 in import_specific_names(import.clone()).iter().cloned() {
        if map_has(exported_set.clone(), &__elem_1) == false {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
}).iter().cloned() {
        __mapped_2.push(diagnostic_node("error", &v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("name '".to_string(), __elem_3.clone()), "' not found in module '".to_string()), import.name.clone()), "' (imported by '".to_string()), importing_module.to_string()), "')".to_string()), import.span.clone(), Some(importing_module.to_string()), Some("unresolved_name".to_string())));
    }
    Rc::new(__mapped_2)
}
};
    Rc::new(ImportResolveResult { resolved: Rc::new(ResolvedImport { module_path: import.name.clone(), target_span: Some(target_mod.span.clone()) }), diagnostics: name_diags })
}
    }
}
}

pub fn get_exported_names(module: Rc<Node>) -> Rc<Vec<String>> {
    let item_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in module_items(module.clone()).iter().cloned() {
        __mapped_0.push(get_item_name(__elem_1.clone()));
    }
    Rc::new(__mapped_0)
};
    let variant_names = {
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in module_items(module.clone()).iter().cloned() {
        __flat_mapped_2.extend(get_variant_names(__elem_3.clone()).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
};
    let imported_names = {
    let mut __flat_mapped_4 = Vec::new();
    for __elem_5 in module_imports(module.clone()).iter().cloned() {
        __flat_mapped_4.extend((if import_is_all(__elem_5.clone()) {
    Rc::new(Vec::new())
} else {
    import_specific_names(__elem_5.clone())
}).iter().cloned());
    }
    Rc::new(__flat_mapped_4)
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(item_names, variant_names), imported_names), Rc::new(KERNEL_TYPES.iter().map(|s| s.to_string()).collect::<Vec<_>>()))
}

pub fn get_item_name(item: Rc<Node>) -> String {
    item.name.clone()
}

pub fn get_variant_names(item: Rc<Node>) -> Rc<Vec<String>> {
    let is_coproduct = (item.connective.clone().is_some()) && (item.connective.clone() == Some(Connective::Disj));
    if is_coproduct {
    {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in item.children.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
}
} else {
    Rc::new(Vec::new())
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DuplicateCheckState {
    pub seen_names: Rc<HashMap<String, bool>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

pub fn check_duplicate_modules(modules: Rc<Vec<Rc<Node>>>) -> Rc<Vec<Rc<Node>>> {
    let result = {
    let mut __acc_0 = Rc::new(DuplicateCheckState { seen_names: Rc::new(std::collections::HashMap::new()), diagnostics: Rc::new(Vec::new()) });
    for __elem_1 in modules.iter().cloned() {
        __acc_0 = {
    let is_dup = map_has(__acc_0.seen_names.clone(), &__elem_1.name);
    if is_dup.clone() {
    Rc::new(DuplicateCheckState { seen_names: __acc_0.seen_names.clone(), diagnostics: {
    let __rc_3 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).diagnostics);
    let mut __appended_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __appended_2.push(diagnostic_node("error", &v2_rt::concat(v2_rt::concat("duplicate module declaration: '".to_string(), __elem_1.name.clone()), "'".to_string()), __elem_1.span.clone(), Some(__elem_1.name.clone()), Some("invalid_operation".to_string())));
    Rc::new(__appended_2)
} })
} else {
    Rc::new(DuplicateCheckState { seen_names: {
    let __rc_5 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).seen_names);
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert(__elem_1.name.clone(), true);
    Rc::new(__map_ins_4)
}, diagnostics: __acc_0.diagnostics.clone() })
}
};
    }
    __acc_0
};
    result.diagnostics.clone()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TopoResult {
    pub sorted: Rc<Vec<String>>,
    pub cycle_error: Option<Rc<Node>>,
}

pub fn adjacency_add_edge(adjacency: Rc<HashMap<String, Rc<Vec<String>>>>, from_module: &str, to_module: &str) -> Rc<HashMap<String, Rc<Vec<String>>>> {
    let existing = match adjacency.clone().get(&from_module.to_string()).cloned() {
    Some(lst) => {
        lst
    }
    None => {
        Rc::new(Vec::new())
    }
};
    {
    let __rc_3 = adjacency;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(from_module.to_string(), {
    let __rc_1 = existing;
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(to_module.to_string());
    Rc::new(__appended_0)
});
    Rc::new(__map_ins_2)
}
}

pub fn topological_sort(modules: Rc<Vec<Rc<Node>>>) -> Rc<TopoResult> {
    let module_names = {
    let mut __mapped_0 = Vec::new();
    for __elem_1 in modules.iter().cloned() {
        __mapped_0.push(__elem_1.name.clone());
    }
    Rc::new(__mapped_0)
};
    let adjacency = {
    let mut __acc_6: Rc<std::collections::HashMap<String, Rc<Vec<String>>>> = Rc::new(std::collections::HashMap::new());
    for __elem_7 in ({
    let mut __flat_mapped_2 = Vec::new();
    for __elem_3 in modules.iter().cloned() {
        __flat_mapped_2.extend(({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in module_imports(__elem_3.clone()).iter().cloned() {
        __mapped_4.push(Rc::new(DepEdge { from_module: __elem_5.name.clone(), to_module: __elem_3.name.clone() }));
    }
    Rc::new(__mapped_4)
}).iter().cloned());
    }
    Rc::new(__flat_mapped_2)
}).iter().cloned() {
        __acc_6 = adjacency_add_edge(__acc_6, &__elem_7.from_module, &__elem_7.to_module);
    }
    __acc_6
};
    let in_degree_map = {
    let mut __acc_8 = Rc::new(std::collections::HashMap::new());
    for __elem_9 in modules.iter().cloned() {
        __acc_8 = {
    let __rc_12 = __acc_8;
    let mut __map_ins_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_11.insert(__elem_9.name.clone(), {
    let __len_10 = module_imports(__elem_9.clone()).len();
    __len_10 as i64
});
    Rc::new(__map_ins_11)
};
    }
    __acc_8
};
    let initial_queue = {
    let __rc_16 = {
    let mut __filtered_13 = Vec::new();
    for __elem_14 in module_names.iter().cloned() {
        if match in_degree_map.clone().get(&__elem_14.clone()).cloned() {
    Some(0) => {
        true
    }
    _ => {
        false
    }
} {
    __filtered_13.push(__elem_14);
};
    }
    Rc::new(__filtered_13)
};
    let mut __sorted_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __sorted_15.sort_by_key(|name| name.clone());
    Rc::new(__sorted_15)
};
    let result = kahn_drain(initial_queue, Rc::new(Vec::new()), in_degree_map.clone(), adjacency);
    let module_count = {
    let __len_17 = modules.clone().len();
    __len_17 as i64
};
    if ({
    let __len_27 = result.sorted.clone().len();
    __len_27 as i64
}) == module_count {
    Rc::new(TopoResult { sorted: result.sorted.clone(), cycle_error: None })
} else {
    let sorted_set = {
    let mut __acc_18 = Rc::new(std::collections::HashMap::new());
    for __elem_19 in result.sorted.iter().cloned() {
        __acc_18 = {
    let __rc_21 = __acc_18;
    let mut __map_ins_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_20.insert(__elem_19, true);
    Rc::new(__map_ins_20)
};
    }
    __acc_18
};
    let cycle_members = {
    let mut __filtered_22 = Vec::new();
    for __elem_23 in module_names.iter().cloned() {
        if map_has(sorted_set.clone(), &__elem_23) == false {
    __filtered_22.push(__elem_23);
};
    }
    Rc::new(__filtered_22)
};
    let cycle_desc = {
    let mut __joined_24 = String::new();
    let mut __first_26 = true;
    for __elem_25 in cycle_members.iter().cloned() {
        if !__first_26 {
    __joined_24.push_str(&" -> ".to_string());
};
        __first_26 = false;
        __joined_24.push_str(&__elem_25);
    }
    __joined_24
};
    Rc::new(TopoResult { sorted: result.sorted.clone(), cycle_error: Some(diagnostic_node("error", &v2_rt::concat("circular dependency detected: ".to_string(), cycle_desc), no_span(), None, Some("invalid_operation".to_string()))) })
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KahnDrainState {
    pub sorted: Rc<Vec<String>>,
    pub in_degree_map: Rc<HashMap<String, i64>>,
}

pub fn kahn_drain(queue: Rc<Vec<String>>, sorted: Rc<Vec<String>>, in_degree_map: Rc<HashMap<String, i64>>, adjacency: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<KahnDrainState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_queue = queue;
        let mut __tco_p_sorted = sorted;
        let mut __tco_p_in_degree_map = in_degree_map;
        let mut __tco_p_adjacency = adjacency;
        loop {
            let queue = __tco_p_queue;
            let sorted = __tco_p_sorted;
            let in_degree_map = __tco_p_in_degree_map;
            let adjacency = __tco_p_adjacency;
            if ({
    let __len_0 = queue.clone().len();
    __len_0 as i64
}) == 0_i64 {
    break Rc::new(KahnDrainState { sorted: sorted.clone(), in_degree_map: in_degree_map.clone() });
};
            let batch_result = {
    let mut __acc_1 = Rc::new(KahnDrainState { sorted: sorted.clone(), in_degree_map: in_degree_map.clone() });
    for __elem_2 in queue.iter().cloned() {
        __acc_1 = {
    let new_sorted = {
    let __rc_4 = std::mem::take(&mut Rc::make_mut(&mut __acc_1).sorted);
    let mut __appended_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __appended_3.push(__elem_2.clone());
    Rc::new(__appended_3)
};
    let neighbors = match adjacency.clone().get(&__elem_2.clone()).cloned() {
    Some(ns) => {
        ns.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let new_degrees = {
    let mut __acc_5 = __acc_1.in_degree_map.clone();
    for __elem_6 in neighbors.iter().cloned() {
        __acc_5 = {
    let current = match __acc_5.clone().get(&__elem_6.clone()).cloned() {
    Some(d) => {
        d.clone()
    }
    None => {
        0_i64
    }
};
    {
    let __rc_8 = __acc_5;
    let mut __map_ins_7 = Rc::try_unwrap(__rc_8).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_7.insert(__elem_6.clone(), current.clone() - 1_i64);
    Rc::new(__map_ins_7)
}
};
    }
    __acc_5
};
    Rc::new(KahnDrainState { sorted: new_sorted.clone(), in_degree_map: new_degrees.clone() })
};
    }
    __acc_1
};
            let new_zero_set = {
    let mut __acc_13 = Rc::new(std::collections::HashMap::new());
    for __elem_14 in ({
    let mut __filtered_11 = Vec::new();
    for __elem_12 in ({
    let mut __flat_mapped_9 = Vec::new();
    for __elem_10 in queue.iter().cloned() {
        __flat_mapped_9.extend((match adjacency.clone().get(&__elem_10.clone()).cloned() {
    Some(ns) => {
        ns.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_9)
}).iter().cloned() {
        if match batch_result.in_degree_map.clone().get(&__elem_12.clone()).cloned() {
    Some(0) => {
        true
    }
    _ => {
        false
    }
} {
    __filtered_11.push(__elem_12);
};
    }
    Rc::new(__filtered_11)
}).iter().cloned() {
        __acc_13 = {
    let __rc_16 = __acc_13;
    let mut __map_ins_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_15.insert(__elem_14, true);
    Rc::new(__map_ins_15)
};
    }
    __acc_13
};
            let new_zero = {
    let __rc_21 = {
    let __rc_17 = new_zero_set;
    let __map_unwrapped_18 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    let mut __keys_19 = __map_unwrapped_18.into_keys().collect::<Vec<_>>();
    __keys_19.sort();
    Rc::new(__keys_19)
};
    let mut __sorted_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __sorted_20.sort_by_key(|name| name.clone());
    Rc::new(__sorted_20)
};
             {
                let __tco_0 = new_zero;
                let __tco_1 = batch_result.sorted.clone();
                let __tco_2 = batch_result.in_degree_map.clone();
                let __tco_3 = adjacency.clone();
                __tco_p_queue = __tco_0;
                __tco_p_sorted = __tco_1;
                __tco_p_in_degree_map = __tco_2;
                __tco_p_adjacency = __tco_3;
                continue;
            }

        }
    })
}

