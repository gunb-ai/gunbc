use crate::v2_core::*;
use crate::infer_types::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedFuncSig {
    pub name: String,
    pub params: Rc<Vec<Rc<Param>>>,
    pub inferred: Rc<Node>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedFuncEnv {
    pub signatures: Rc<HashMap<String, Rc<ResolvedFuncSig>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolveFuncSigsResult {
    pub func_env: Rc<ResolvedFuncEnv>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SigsAccum {
    pub signatures: Rc<HashMap<String, Rc<ResolvedFuncSig>>>,
    pub diagnostics: Rc<Vec<Rc<Node>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CallEdge {
    pub caller: String,
    pub callee: String,
}

pub fn collect_func_call_edges(items: Rc<Vec<Rc<Node>>>, local_func_set: Rc<HashMap<String, bool>>) -> Rc<Vec<Rc<CallEdge>>> {
    {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in items.iter().cloned() {
        __flat_mapped_0.extend((if (({
    let __len_2 = __elem_1.params.clone().len();
    __len_2 as i64
}) > 0_i64) && (__elem_1.body.clone().is_some()) {
    collect_calls_in_expr(&__elem_1.name, __elem_1.body.clone().unwrap(), local_func_set.clone())
} else {
    Rc::new(Vec::new())
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
}
}

pub fn collect_calls_in_expr(caller: &str, texpr: Rc<Node>, local_func_set: Rc<HashMap<String, bool>>) -> Rc<Vec<Rc<CallEdge>>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let this_edges = match texpr.expr_data.as_ref() {
    ExprData::ExprCall { func: f, call_semantics: _, .. } => {
        if emit_map_has(local_func_set.clone(), &f) {
    Rc::new(vec!(Rc::new(CallEdge { caller: caller.to_string(), callee: f.clone() })))
} else {
    Rc::new(Vec::new())
}
    }
    _ => {
        Rc::new(Vec::new())
    }
};
        let child_edges = {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in texpr.children.iter().cloned() {
        __flat_mapped_0.extend(collect_calls_in_expr(&caller, __elem_1.clone(), local_func_set.clone()).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
};
        let result = v2_rt::concat(this_edges, child_edges);
        result
    })
}

pub fn func_reaches_self(root: &str, current: &str, call_edges: Rc<Vec<Rc<CallEdge>>>, visited: Rc<HashMap<String, bool>>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if emit_map_has(visited.clone(), &current) {
    false
} else {
    let next_visited = {
    let __rc_1 = visited;
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert(current.to_string(), true);
    Rc::new(__map_ins_0)
};
    let callees = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in call_edges.iter().cloned() {
        if __elem_3.caller.clone() == current {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_4.push(__elem_5.callee.clone());
    }
    Rc::new(__mapped_4)
};
    {
    let mut __any_6 = false;
    for __elem_7 in callees.iter().cloned() {
        if if __elem_7.clone() == root {
    true
} else {
    func_reaches_self(&root, &__elem_7, call_edges.clone(), next_visited.clone())
} {
    __any_6 = true;
    break;
};
    }
    __any_6
}
}
    })
}

pub fn declared_to_resolved(dsig: Rc<DeclaredFuncSig>) -> Rc<ResolvedFuncSig> {
    Rc::new(ResolvedFuncSig { name: dsig.name.clone(), params: dsig.params.clone(), inferred: dsig.inferred.clone().unwrap(), is_async: dsig.is_async.clone() })
}

pub fn merge_remaining_declared(declared_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>, resolved: Rc<HashMap<String, Rc<ResolvedFuncSig>>>) -> Rc<HashMap<String, Rc<ResolvedFuncSig>>> {
    {
    let mut __acc_4 = resolved;
    for __elem_5 in ({
    let __rc_0 = declared_sigs;
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = if __elem_5.inferred.clone().is_some() {
    {
    let __rc_7 = __acc_4;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(__elem_5.name.clone(), declared_to_resolved(__elem_5.clone()));
    Rc::new(__map_ins_6)
}
} else {
    __acc_4
};
    }
    __acc_4
}
}

pub fn topo_resolve_loop(remaining: Rc<Vec<String>>, resolved: Rc<HashMap<String, Rc<ResolvedFuncSig>>>, declared_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>, call_edges: Rc<Vec<Rc<CallEdge>>>, local_func_set: Rc<HashMap<String, bool>>, module_name: &str, diagnostics: Rc<Vec<Rc<Node>>>) -> Rc<ResolveFuncSigsResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_resolved = resolved;
        let mut __tco_p_declared_sigs = declared_sigs;
        let mut __tco_p_call_edges = call_edges;
        let mut __tco_p_local_func_set = local_func_set;
        let mut __tco_p_module_name = module_name.to_string();
        let mut __tco_p_diagnostics = diagnostics;
        loop {
            let remaining = __tco_p_remaining;
            let resolved = __tco_p_resolved;
            let declared_sigs = __tco_p_declared_sigs;
            let call_edges = __tco_p_call_edges;
            let local_func_set = __tco_p_local_func_set;
            let module_name = __tco_p_module_name;
            let diagnostics = __tco_p_diagnostics;
            if ({
    let __len_8 = remaining.clone().len();
    __len_8 as i64
}) == 0_i64 {
    let all_resolved = {
    let mut __acc_4 = resolved.clone();
    for __elem_5 in ({
    let __rc_0 = declared_sigs.clone();
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __acc_4 = if __elem_5.inferred.clone().is_some() {
    {
    let __rc_7 = __acc_4;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(__elem_5.name.clone(), declared_to_resolved(__elem_5.clone()));
    Rc::new(__map_ins_6)
}
} else {
    __acc_4
};
    }
    __acc_4
};
    break Rc::new(ResolveFuncSigsResult { func_env: Rc::new(ResolvedFuncEnv { signatures: all_resolved.clone() }), diagnostics: diagnostics.clone() });
};
            let ready = {
    let mut __filtered_9 = Vec::new();
    for __elem_10 in remaining.iter().cloned() {
        {
let __cond = {
    let local_callees = {
    let mut __filtered_15 = Vec::new();
    for __elem_16 in ({
    let mut __mapped_13 = Vec::new();
    for __elem_14 in ({
    let mut __filtered_11 = Vec::new();
    for __elem_12 in call_edges.iter().cloned() {
        if __elem_12.caller.clone() == __elem_10.clone() {
    __filtered_11.push(__elem_12);
};
    }
    Rc::new(__filtered_11)
}).iter().cloned() {
        __mapped_13.push(__elem_14.callee.clone());
    }
    Rc::new(__mapped_13)
}).iter().cloned() {
        if emit_map_has(local_func_set.clone(), &__elem_16) {
    __filtered_15.push(__elem_16);
};
    }
    Rc::new(__filtered_15)
};
    {
    let mut __all_17 = true;
    for __elem_18 in local_callees.iter().cloned() {
        if !(resolved.clone().get(&__elem_18.clone()).cloned().is_some()) {
    __all_17 = false;
    break;
};
    }
    __all_17
}
};
if __cond {
    __filtered_9.push(__elem_10);
}
};
    }
    Rc::new(__filtered_9)
};
            if ({
    let __len_33 = ready.clone().len();
    __len_33 as i64
}) == 0_i64 {
    let cycle_accum = {
    let mut __acc_19 = Rc::new(SigsAccum { signatures: resolved.clone(), diagnostics: Rc::new(Vec::new()) });
    for __elem_20 in remaining.iter().cloned() {
        __acc_19 = match declared_sigs.clone().get(&__elem_20.clone()).cloned() {
    Some(dsig) => {
        if dsig.inferred.clone().is_some() {
    Rc::new(SigsAccum { signatures: {
    let __rc_22 = std::mem::take(&mut Rc::make_mut(&mut __acc_19).signatures);
    let mut __map_ins_21 = Rc::try_unwrap(__rc_22).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_21.insert(__elem_20.clone(), declared_to_resolved(dsig.clone()));
    Rc::new(__map_ins_21)
}, diagnostics: __acc_19.diagnostics.clone() })
} else {
    Rc::new(SigsAccum { signatures: __acc_19.signatures.clone(), diagnostics: {
    let __rc_24 = std::mem::take(&mut Rc::make_mut(&mut __acc_19).diagnostics);
    let mut __appended_23 = Rc::try_unwrap(__rc_24).unwrap_or_else(|rc| (*rc).clone());
    __appended_23.push(diagnostic_node("error", &v2_rt::concat(v2_rt::concat("recursive function '".to_string(), __elem_20.clone()), "' requires return type annotation".to_string()), no_span(), Some(module_name.to_string()), Some("type_mismatch".to_string())));
    Rc::new(__appended_23)
} })
}
    }
    None => {
        __acc_19.clone()
    }
};
    }
    __acc_19
};
    let all_resolved = {
    let mut __acc_29 = cycle_accum.signatures.clone();
    for __elem_30 in ({
    let __rc_25 = declared_sigs.clone();
    let __map_unwrapped_26 = Rc::try_unwrap(__rc_25).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_27 = __map_unwrapped_26.into_iter().collect::<Vec<_>>();
    __entries_27.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_28 = __entries_27.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_28)
}).iter().cloned() {
        __acc_29 = if __elem_30.inferred.clone().is_some() {
    {
    let __rc_32 = __acc_29;
    let mut __map_ins_31 = Rc::try_unwrap(__rc_32).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_31.insert(__elem_30.name.clone(), declared_to_resolved(__elem_30.clone()));
    Rc::new(__map_ins_31)
}
} else {
    __acc_29
};
    }
    __acc_29
};
    break Rc::new(ResolveFuncSigsResult { func_env: Rc::new(ResolvedFuncEnv { signatures: all_resolved.clone() }), diagnostics: v2_rt::concat(diagnostics.clone(), cycle_accum.diagnostics.clone()) });
};
            let ready_accum = {
    let mut __acc_34 = Rc::new(SigsAccum { signatures: resolved.clone(), diagnostics: diagnostics.clone() });
    for __elem_35 in ready.iter().cloned() {
        __acc_34 = match declared_sigs.clone().get(&__elem_35.clone()).cloned() {
    Some(dsig) => {
        if dsig.inferred.clone().is_some() {
    Rc::new(SigsAccum { signatures: {
    let __rc_37 = std::mem::take(&mut Rc::make_mut(&mut __acc_34).signatures);
    let mut __map_ins_36 = Rc::try_unwrap(__rc_37).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_36.insert(__elem_35.clone(), declared_to_resolved(dsig.clone()));
    Rc::new(__map_ins_36)
}, diagnostics: __acc_34.diagnostics.clone() })
} else {
    Rc::new(SigsAccum { signatures: __acc_34.signatures.clone(), diagnostics: {
    let __rc_39 = std::mem::take(&mut Rc::make_mut(&mut __acc_34).diagnostics);
    let mut __appended_38 = Rc::try_unwrap(__rc_39).unwrap_or_else(|rc| (*rc).clone());
    __appended_38.push(diagnostic_node("error", &v2_rt::concat(v2_rt::concat("function '".to_string(), __elem_35.clone()), "' requires return type annotation".to_string()), no_span(), Some(module_name.to_string()), Some("type_mismatch".to_string())));
    Rc::new(__appended_38)
} })
}
    }
    None => {
        __acc_34.clone()
    }
};
    }
    __acc_34
};
            let ready_set = {
    let mut __acc_40 = Rc::new(std::collections::HashMap::new());
    for __elem_41 in ready.iter().cloned() {
        __acc_40 = {
    let __rc_43 = __acc_40;
    let mut __map_ins_42 = Rc::try_unwrap(__rc_43).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_42.insert(__elem_41, true);
    Rc::new(__map_ins_42)
};
    }
    __acc_40
};
            let next_remaining = {
    let mut __filtered_44 = Vec::new();
    for __elem_45 in remaining.iter().cloned() {
        if emit_map_has(ready_set.clone(), &__elem_45) == false {
    __filtered_44.push(__elem_45);
};
    }
    Rc::new(__filtered_44)
};
             {
                let __tco_0 = next_remaining;
                let __tco_1 = ready_accum.signatures.clone();
                let __tco_2 = declared_sigs.clone();
                let __tco_3 = call_edges.clone();
                let __tco_4 = local_func_set.clone();
                let __tco_5 = module_name;
                let __tco_6 = ready_accum.diagnostics.clone();
                __tco_p_remaining = __tco_0;
                __tco_p_resolved = __tco_1;
                __tco_p_declared_sigs = __tco_2;
                __tco_p_call_edges = __tco_3;
                __tco_p_local_func_set = __tco_4;
                __tco_p_module_name = __tco_5;
                __tco_p_diagnostics = __tco_6;
                continue;
            }

        }
    })
}

pub fn resolve_func_sigs(declared_sigs: Rc<HashMap<String, Rc<DeclaredFuncSig>>>, items: Rc<Vec<Rc<Node>>>, module_name: &str) -> Rc<ResolveFuncSigsResult> {
    let local_func_names = {
    let mut __mapped_3 = Vec::new();
    for __elem_4 in ({
    let mut __filtered_0 = Vec::new();
    for __elem_1 in items.iter().cloned() {
        if (({
    let __len_2 = __elem_1.params.clone().len();
    __len_2 as i64
}) > 0_i64) && (__elem_1.body.clone().is_some()) {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
}).iter().cloned() {
        __mapped_3.push(__elem_4.name.clone());
    }
    Rc::new(__mapped_3)
};
    let local_func_set = {
    let mut __acc_5 = Rc::new(std::collections::HashMap::new());
    for __elem_6 in local_func_names.iter().cloned() {
        __acc_5 = {
    let __rc_8 = __acc_5;
    let mut __map_ins_7 = Rc::try_unwrap(__rc_8).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_7.insert(__elem_6, true);
    Rc::new(__map_ins_7)
};
    }
    __acc_5
};
    let call_edges = collect_func_call_edges(items.clone(), local_func_set.clone());
    let parent_resolved = {
    let mut __acc_13: Rc<std::collections::HashMap<String, Rc<ResolvedFuncSig>>> = Rc::new(std::collections::HashMap::new());
    for __elem_14 in ({
    let __rc_9 = declared_sigs.clone();
    let __map_unwrapped_10 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_11 = __map_unwrapped_10.into_iter().collect::<Vec<_>>();
    __entries_11.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_12 = __entries_11.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_12)
}).iter().cloned() {
        __acc_13 = if emit_map_has(local_func_set.clone(), &__elem_14.name) {
    __acc_13
} else {
    if __elem_14.inferred.clone().is_some() {
    {
    let __rc_16 = __acc_13;
    let mut __map_ins_15 = Rc::try_unwrap(__rc_16).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_15.insert(__elem_14.name.clone(), declared_to_resolved(__elem_14.clone()));
    Rc::new(__map_ins_15)
}
} else {
    __acc_13
}
};
    }
    __acc_13
};
    topo_resolve_loop(local_func_names.clone(), parent_resolved, declared_sigs.clone(), call_edges, local_func_set.clone(), &module_name, Rc::new(Vec::new()))
}

