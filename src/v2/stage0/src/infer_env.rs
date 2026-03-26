use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeEnv {
    pub bindings: Rc<HashMap<String, Rc<TypeBinding>>>,
    pub recursive_types: Rc<Vec<String>>,
    pub recursive_type_set: Rc<HashMap<String, bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeBinding {
    pub name: String,
    pub resolved: Rc<Node>,
}

pub fn is_recursive_type(env: Rc<TypeEnv>, name: &str) -> bool {
    match env.recursive_type_set.clone().get(&name.to_string()).cloned() {
    Some(_) => {
        true
    }
    _ => {
        false
    }
}
}

pub fn lookup_type(env: Rc<TypeEnv>, name: &str) -> Option<Rc<Node>> {
    let canonical = name.to_string();
    match env.bindings.clone().get(&canonical).cloned() {
    Some(binding) => {
        Some(binding.resolved.clone())
    }
    None => {
        None
    }
}
}

pub fn merge_envs(envs: Rc<Vec<Rc<TypeEnv>>>) -> Rc<TypeEnv> {
    let merged_bindings = {
    let mut __acc_0 = Rc::new(std::collections::HashMap::new());
    for __elem_1 in envs.iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_merged_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_2.extend(Rc::try_unwrap(__elem_1.bindings.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_2)
};
    }
    __acc_0
};
    let merged_recursive = {
    let mut __acc_4: Rc<Vec<String>> = Rc::new(Vec::new());
    for __elem_5 in envs.iter().cloned() {
        __acc_4 = v2_rt::concat(__acc_4, __elem_5.recursive_types.clone());
    }
    __acc_4
};
    let merged_recursive_set = {
    let mut __acc_6 = Rc::new(std::collections::HashMap::new());
    for __elem_7 in envs.iter().cloned() {
        __acc_6 = {
    let __rc_9 = __acc_6;
    let mut __map_merged_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_merged_8.extend(Rc::try_unwrap(__elem_7.recursive_type_set.clone()).unwrap_or_else(|rc| (*rc).clone()));
    Rc::new(__map_merged_8)
};
    }
    __acc_6
};
    Rc::new(TypeEnv { bindings: merged_bindings.clone(), recursive_types: merged_recursive.clone(), recursive_type_set: merged_recursive_set.clone() })
}

