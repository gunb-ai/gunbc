use crate::infer_env::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn set_has(m: Rc<HashMap<String, bool>>, key: &str) -> bool {
    let result = match m.get(&key.to_string()).cloned() {
    Some(_) => {
        true
    }
    None => {
        false
    }
};
    result
}

pub fn compute_in_graph_deps(all_names: Rc<Vec<String>>, deps_map: Rc<HashMap<String, Rc<Vec<String>>>>, name_set: Rc<HashMap<String, bool>>) -> Rc<HashMap<String, Rc<Vec<String>>>> {
    let result = {
    let mut __acc_0 = Rc::new(std::collections::HashMap::new());
    for __elem_1 in all_names.iter().cloned() {
        __acc_0 = match deps_map.clone().get(&__elem_1.clone()).cloned() {
    Some(deps) => {
        {
    let local = {
    let mut __filtered_2 = Vec::new();
    for __elem_3 in deps.iter().cloned() {
        if (__elem_3.clone() != __elem_1.clone()) && set_has(name_set.clone(), &__elem_3) {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
};
    {
    let __rc_5 = __acc_0;
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert(__elem_1.clone(), local.clone());
    Rc::new(__map_ins_4)
}
}
    }
    None => {
        {
    let __rc_7 = __acc_0;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(__elem_1.clone(), Rc::new(Vec::new()));
    Rc::new(__map_ins_6)
}
    }
};
    }
    __acc_0
};
    result
}

pub fn build_reverse_adj(all_names: Rc<Vec<String>>, local_deps: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<HashMap<String, Rc<Vec<String>>>> {
    {
    let mut __acc_0 = Rc::new(std::collections::HashMap::<String, Rc<Vec<String>>>::new());
    for __elem_1 in all_names.iter().cloned() {
        __acc_0 = match local_deps.clone().get(&__elem_1.clone()).cloned() {
    Some(deps) => {
        {
    let mut __acc_2 = __acc_0.clone();
    for __elem_3 in deps.iter().cloned() {
        __acc_2 = {
    let existing = match __acc_2.clone().get(&__elem_3.clone()).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    {
    let __rc_7 = __acc_2;
    let mut __map_ins_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_6.insert(__elem_3.clone(), {
    let __rc_5 = existing;
    let mut __appended_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __appended_4.push(__elem_1.clone());
    Rc::new(__appended_4)
});
    Rc::new(__map_ins_6)
}
};
    }
    __acc_2
}
    }
    None => {
        __acc_0.clone()
    }
};
    }
    __acc_0
}
}

pub fn build_in_degree(all_names: Rc<Vec<String>>, local_deps: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<HashMap<String, i64>> {
    {
    let mut __acc_0 = Rc::new(std::collections::HashMap::new());
    for __elem_1 in all_names.iter().cloned() {
        __acc_0 = {
    let deg = match local_deps.clone().get(&__elem_1.clone()).cloned() {
    Some(deps) => {
        {
    let __len_2 = deps.clone().len();
    __len_2 as i64
}
    }
    None => {
        0_i64
    }
};
    {
    let __rc_4 = __acc_0;
    let mut __map_ins_3 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_3.insert(__elem_1.clone(), deg.clone());
    Rc::new(__map_ins_3)
}
};
    }
    __acc_0
}
}

pub fn kahn_remove_loop(remaining: Rc<Vec<String>>, local_deps: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<Vec<String>> {
    let reverse_adj = build_reverse_adj(remaining.clone(), local_deps.clone());
    let in_degree = build_in_degree(remaining.clone(), local_deps.clone());
    let initial_queue = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in remaining.iter().cloned() {
        if match in_degree.clone().get(&__elem_1.clone()).cloned() {
    Some(d) => {
        d.clone() == 0_i64
    }
    None => {
        true
    }
} {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    let final_state = kahn_cycle_drain(initial_queue, in_degree.clone(), reverse_adj, 0_i64);
    if final_state.removed_count.clone() == ({
    let __len_4 = remaining.clone().len();
    __len_4 as i64
}) {
    Rc::new(Vec::new())
} else {
    {
    let mut __filtered_2 = Vec::new();
    for __elem_3 in remaining.iter().cloned() {
        if match final_state.in_degree.clone().get(&__elem_3.clone()).cloned() {
    Some(d) => {
        d.clone() > 0_i64
    }
    None => {
        false
    }
} {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KahnState {
    pub in_degree: Rc<HashMap<String, i64>>,
    pub removed_count: i64,
}

pub fn kahn_cycle_drain(queue: Rc<Vec<String>>, in_degree: Rc<HashMap<String, i64>>, reverse_adj: Rc<HashMap<String, Rc<Vec<String>>>>, removed_count: i64) -> Rc<KahnState> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_queue = queue;
        let mut __tco_p_in_degree = in_degree;
        let mut __tco_p_reverse_adj = reverse_adj;
        let mut __tco_p_removed_count = removed_count;
        loop {
            let queue = __tco_p_queue;
            let in_degree = __tco_p_in_degree;
            let reverse_adj = __tco_p_reverse_adj;
            let removed_count = __tco_p_removed_count;
            if ({
    let __len_0 = queue.clone().len();
    __len_0 as i64
}) == 0_i64 {
    break Rc::new(KahnState { in_degree: in_degree.clone(), removed_count: removed_count.clone() });
};
            let result = {
    let mut __acc_1 = Rc::new(KahnState { in_degree: in_degree.clone(), removed_count: removed_count.clone() });
    for __elem_2 in queue.iter().cloned() {
        __acc_1 = {
    let dependents = match reverse_adj.clone().get(&__elem_2).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    let new_deg = {
    let mut __acc_3 = __acc_1.in_degree.clone();
    for __elem_4 in dependents.iter().cloned() {
        __acc_3 = {
    let old = match __acc_3.clone().get(&__elem_4.clone()).cloned() {
    Some(d) => {
        d.clone()
    }
    None => {
        0_i64
    }
};
    {
    let __rc_6 = __acc_3;
    let mut __map_ins_5 = Rc::try_unwrap(__rc_6).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_5.insert(__elem_4.clone(), old.clone() - 1_i64);
    Rc::new(__map_ins_5)
}
};
    }
    __acc_3
};
    Rc::new(KahnState { in_degree: new_deg.clone(), removed_count: __acc_1.removed_count.clone() + 1_i64 })
};
    }
    __acc_1
};
            let next_queue = {
    let mut __acc_7 = Rc::new(Vec::new());
    for __elem_8 in queue.iter().cloned() {
        __acc_7 = {
    let dependents = match reverse_adj.clone().get(&__elem_8).cloned() {
    Some(v) => {
        v.clone()
    }
    None => {
        Rc::new(Vec::new())
    }
};
    {
    let mut __acc_9 = __acc_7;
    for __elem_10 in dependents.iter().cloned() {
        __acc_9 = {
    let deg = match result.in_degree.clone().get(&__elem_10.clone()).cloned() {
    Some(d) => {
        d.clone()
    }
    None => {
        0_i64
    }
};
    if deg.clone() == 0_i64 {
    {
    let __rc_12 = __acc_9;
    let mut __appended_11 = Rc::try_unwrap(__rc_12).unwrap_or_else(|rc| (*rc).clone());
    __appended_11.push(__elem_10.clone());
    Rc::new(__appended_11)
}
} else {
    __acc_9
}
};
    }
    __acc_9
}
};
    }
    __acc_7
};
             {
                let __tco_0 = next_queue;
                let __tco_1 = result.in_degree.clone();
                let __tco_2 = reverse_adj.clone();
                let __tco_3 = result.removed_count.clone();
                __tco_p_queue = __tco_0;
                __tco_p_in_degree = __tco_1;
                __tco_p_reverse_adj = __tco_2;
                __tco_p_removed_count = __tco_3;
                continue;
            }

        }
    })
}

pub fn detect_type_cycles_kahn(deps_map: Rc<HashMap<String, Rc<Vec<String>>>>, bindings: Rc<HashMap<String, Rc<TypeBinding>>>) -> Rc<Vec<String>> {
    let all_names = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let __rc_0 = bindings;
    let __map_unwrapped_1 = Rc::try_unwrap(__rc_0).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_2 = __map_unwrapped_1.into_iter().collect::<Vec<_>>();
    __entries_2.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_3 = __entries_2.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_3)
}).iter().cloned() {
        __mapped_4.push(__elem_5.name.clone());
    }
    Rc::new(__mapped_4)
};
    let name_set = {
    let mut __acc_6 = Rc::new(std::collections::HashMap::new());
    for __elem_7 in all_names.iter().cloned() {
        __acc_6 = {
    let __rc_9 = __acc_6;
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert(__elem_7, true);
    Rc::new(__map_ins_8)
};
    }
    __acc_6
};
    let local_deps = compute_in_graph_deps(all_names.clone(), deps_map.clone(), name_set);
    let self_refs = {
    let mut __filtered_10 = Vec::new();
    for __elem_11 in all_names.iter().cloned() {
        if match deps_map.clone().get(&__elem_11.clone()).cloned() {
    Some(deps) => {
        {
    let mut __any_12 = false;
    for __elem_13 in deps.iter().cloned() {
        if __elem_13.clone() == __elem_11.clone() {
    __any_12 = true;
    break;
};
    }
    __any_12
}
    }
    None => {
        false
    }
} {
    __filtered_10.push(__elem_11);
};
    }
    Rc::new(__filtered_10)
};
    let cycle_members = kahn_remove_loop(all_names.clone(), local_deps);
    let sr_set = {
    let mut __acc_14 = Rc::new(std::collections::HashMap::new());
    for __elem_15 in self_refs.iter().cloned() {
        __acc_14 = {
    let __rc_17 = __acc_14;
    let mut __map_ins_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_16.insert(__elem_15, true);
    Rc::new(__map_ins_16)
};
    }
    __acc_14
};
    let cm_set = {
    let mut __acc_18 = sr_set;
    for __elem_19 in cycle_members.iter().cloned() {
        __acc_18 = {
    let __rc_21 = __acc_18;
    let mut __map_ins_20 = Rc::try_unwrap(__rc_21).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_20.insert(__elem_19, true);
    Rc::new(__map_ins_20)
};
    }
    __acc_18
};
    let result = {
    let mut __filtered_22 = Vec::new();
    for __elem_23 in all_names.iter().cloned() {
        if set_has(cm_set.clone(), &__elem_23) {
    __filtered_22.push(__elem_23);
};
    }
    Rc::new(__filtered_22)
};
    result
}

