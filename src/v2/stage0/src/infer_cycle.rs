use crate::infer_env::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn set_has(m: Rc<HashMap<String, bool>>, key: &str) -> bool {
    let result = match m.clone().get(&key.to_string()).cloned() {
    Some(_) => {
        true
    }
    None => {
        false
    }
};
    result.clone()
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
    result.clone()
}

pub fn kahn_count_removed_deps(name: &str, local_deps: Rc<HashMap<String, Rc<Vec<String>>>>, removed_set: Rc<HashMap<String, bool>>) -> i64 {
    let result = match local_deps.clone().get(&name.to_string()).cloned() {
    Some(deps) => {
        {
    let mut __count_2 = 0i64;
    for __elem_3 in deps.iter().cloned() {
        if set_has(removed_set.clone(), &__elem_3) {
    __count_2 += 1i64;
};
    }
    __count_2
}
    }
    None => {
        0_i64
    }
};
    result.clone()
}

pub fn kahn_is_ready(name: &str, local_deps: Rc<HashMap<String, Rc<Vec<String>>>>, remaining_set: Rc<HashMap<String, bool>>) -> bool {
    kahn_count_removed_deps(&name, local_deps.clone(), remaining_set.clone()) == 0_i64
}

pub fn kahn_remove_loop(remaining: Rc<Vec<String>>, local_deps: Rc<HashMap<String, Rc<Vec<String>>>>) -> Rc<Vec<String>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_remaining = remaining;
        let mut __tco_p_local_deps = local_deps;
        loop {
            let remaining = __tco_p_remaining;
            let local_deps = __tco_p_local_deps;
            let remaining_set = {
    let mut __acc_0 = Rc::new(std::collections::HashMap::new());
    for __elem_1 in remaining.iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1, true);
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
            let ready = {
    let mut __filtered_4 = Vec::new();
    for __elem_5 in remaining.iter().cloned() {
        if kahn_is_ready(&__elem_5, local_deps.clone(), remaining_set.clone()) {
    __filtered_4.push(__elem_5);
};
    }
    Rc::new(__filtered_4)
};
            if ({
    let __len_6 = ready.clone().len();
    __len_6 as i64
}) == 0_i64 {
    break remaining.clone();
};
            let ready_set = {
    let mut __acc_7 = Rc::new(std::collections::HashMap::new());
    for __elem_8 in ready.iter().cloned() {
        __acc_7 = {
    let __rc_10 = __acc_7;
    let mut __map_ins_9 = Rc::try_unwrap(__rc_10).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_9.insert(__elem_8.clone(), true);
    Rc::new(__map_ins_9)
};
    }
    __acc_7
};
            let next_remaining = {
    let mut __filtered_11 = Vec::new();
    for __elem_12 in remaining.iter().cloned() {
        if set_has(ready_set.clone(), &__elem_12) == false {
    __filtered_11.push(__elem_12);
};
    }
    Rc::new(__filtered_11)
};
             {
                let __tco_0 = next_remaining.clone();
                let __tco_1 = local_deps.clone();
                __tco_p_remaining = __tco_0;
                __tco_p_local_deps = __tco_1;
                continue;
            }

        }
    })
}

pub fn detect_type_cycles_kahn(deps_map: Rc<HashMap<String, Rc<Vec<String>>>>, bindings: Rc<HashMap<String, Rc<TypeBinding>>>) -> Rc<Vec<String>> {
    let all_names = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let __rc_0 = bindings.clone();
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
    __map_ins_8.insert(__elem_7.clone(), true);
    Rc::new(__map_ins_8)
};
    }
    __acc_6
};
    let local_deps = compute_in_graph_deps(all_names.clone(), deps_map.clone(), name_set.clone());
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
    let cycle_members = kahn_remove_loop(all_names.clone(), local_deps.clone());
    let sr_set = {
    let mut __acc_14 = Rc::new(std::collections::HashMap::new());
    for __elem_15 in self_refs.iter().cloned() {
        __acc_14 = {
    let __rc_17 = __acc_14;
    let mut __map_ins_16 = Rc::try_unwrap(__rc_17).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_16.insert(__elem_15.clone(), true);
    Rc::new(__map_ins_16)
};
    }
    __acc_14
};
    let cm_set = {
    let mut __acc_18 = sr_set.clone();
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
    result.clone()
}

