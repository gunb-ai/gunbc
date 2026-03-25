use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum EdgeKind {
    #[default]
    Consumed,
    Read,
    Threaded,
    Projected,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EdgeClassification {
    pub kind: EdgeKind,
    pub site: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingUsage {
    pub name: String,
    pub consumers: Rc<Vec<Rc<EdgeClassification>>>,
}

pub fn semantic_consumer_count(usage: Rc<BindingUsage>) -> i64 {
    {
    let mut __count_2 = 0i64;
    for __elem_3 in usage.consumers.iter().cloned() {
        if match __elem_3.kind.clone() {
    EdgeKind::Consumed => {
        true
    }
    _ => {
        false
    }
} {
    __count_2 += 1i64;
};
    }
    __count_2
}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipDecision {
    SoleOwner { binding: String, site: String },
    SharedError { binding: String, consumer_count: i64, sites: Rc<Vec<String>> },
    Unclassified { binding: String, reason: String },
}

impl Default for OwnershipDecision {
    fn default() -> Self {
        OwnershipDecision::SoleOwner { binding: Default::default(), site: Default::default() }
    }
}

impl OwnershipDecision {
    pub fn binding(&self) -> String {
        match self {
            OwnershipDecision::SoleOwner { binding, .. } => binding.clone(),
            OwnershipDecision::SharedError { binding, .. } => binding.clone(),
            OwnershipDecision::Unclassified { binding, .. } => binding.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnershipProof {
    pub func_name: String,
    pub bindings: Rc<HashMap<String, Rc<BindingUsage>>>,
    pub decisions: Rc<Vec<Rc<OwnershipDecision>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UsageAccum {
    pub bindings: Rc<HashMap<String, Rc<BindingUsage>>>,
}

pub fn empty_usage_accum() -> Rc<UsageAccum> {
    Rc::new(UsageAccum { bindings: Rc::new(std::collections::HashMap::new()) })
}

pub fn record_use(accum: Rc<UsageAccum>, name: &str, kind: EdgeKind, site: &str) -> Rc<UsageAccum> {
    let edge = Rc::new(EdgeClassification { kind, site: site.to_string() });
    let existing = match accum.bindings.clone().get(&name.to_string()).cloned() {
    Some(usage) => {
        usage.clone()
    }
    None => {
        Rc::new(BindingUsage { name: name.to_string(), consumers: Rc::new(Vec::new()) })
    }
};
    let updated = Rc::new(BindingUsage { name: name.to_string(), consumers: {
    let __rc_1 = existing.consumers.clone();
    let mut __appended_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __appended_0.push(edge.clone());
    Rc::new(__appended_0)
} });
    Rc::new(UsageAccum { bindings: {
    let __rc_3 = accum.bindings.clone();
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(name.to_string(), updated.clone());
    Rc::new(__map_ins_2)
} })
}

pub fn merge_branch_usages(base: Rc<UsageAccum>, branches: Rc<Vec<Rc<UsageAccum>>>) -> Rc<UsageAccum> {
    {
    let mut __acc_0 = base.clone();
    for __elem_1 in branches.iter().cloned() {
        __acc_0 = {
    let mut __acc_6 = __acc_0.clone();
    for __elem_7 in ({
    let __rc_2 = __elem_1.bindings.clone();
    let __map_unwrapped_3 = Rc::try_unwrap(__rc_2).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_4 = __map_unwrapped_3.into_iter().collect::<Vec<_>>();
    __entries_4.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_5 = __entries_4.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_5)
}).iter().cloned() {
        __acc_6 = {
    let current_count = match __acc_6.bindings.clone().get(&__elem_7.name.clone()).cloned() {
    Some(existing) => {
        semantic_consumer_count(existing.clone())
    }
    None => {
        0_i64
    }
};
    let branch_count = semantic_consumer_count(__elem_7.clone());
    if branch_count.clone() > current_count.clone() {
    Rc::new(UsageAccum { bindings: {
    let __rc_9 = std::mem::take(&mut Rc::make_mut(&mut __acc_6).bindings);
    let mut __map_ins_8 = Rc::try_unwrap(__rc_9).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_8.insert(__elem_7.name.clone(), __elem_7.clone());
    Rc::new(__map_ins_8)
} })
} else {
    __acc_6.clone()
}
};
    }
    __acc_6
};
    }
    __acc_0
}
}

pub fn walk_expr(accum: Rc<UsageAccum>, texpr: Rc<Node>, in_tail: bool) -> Rc<UsageAccum> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprVar { name: n, .. } => {
        if in_tail.clone() {
    record_use(accum.clone(), &n, EdgeKind::Consumed, "return")
} else {
    record_use(accum.clone(), &n, EdgeKind::Read, "read")
}
    }
    ExprData::ExprLiteral { value: _, .. } => {
        accum.clone()
    }
    ExprData::ExprFieldAccess { base: b, field: f, .. } => {
        match b.expr_data.as_ref() {
    ExprData::ExprVar { name: vn, .. } => {
        record_use(accum.clone(), &vn, EdgeKind::Projected, &v2_rt::concat(".".to_string(), f.clone()))
    }
    _ => {
        walk_expr(accum.clone(), b.clone(), false)
    }
}
    }
    ExprData::ExprCall { func: fname, args: call_args, .. } => {
        if fname.clone() == "fold" {
    let init_arg = {
    let mut __found_2 = None;
    for __elem_3 in call_args.iter().cloned() {
        if __elem_3.name.clone() == Some("init".to_string()) {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
};
    let threaded_accum = match init_arg.clone() {
    Some(ia) => {
        match ia.value.expr_data.as_ref() {
    ExprData::ExprVar { name: vn, .. } => {
        record_use(accum.clone(), &vn, EdgeKind::Threaded, "fold_init")
    }
    _ => {
        walk_expr(accum.clone(), ia.value.clone(), false)
    }
}
    }
    None => {
        accum.clone()
    }
};
    let non_init = {
    let mut __filtered_4 = Vec::new();
    for __elem_5 in call_args.iter().cloned() {
        if __elem_5.name.clone() != Some("init".to_string()) {
    __filtered_4.push(__elem_5);
};
    }
    Rc::new(__filtered_4)
};
    {
    let mut __acc_6 = threaded_accum.clone();
    for __elem_7 in non_init.iter().cloned() {
        __acc_6 = walk_expr(__acc_6.clone(), __elem_7.value.clone(), false);
    }
    __acc_6
}
} else {
    {
    let mut __acc_8 = accum.clone();
    for __elem_9 in call_args.iter().cloned() {
        __acc_8 = walk_expr(__acc_8.clone(), __elem_9.value.clone(), false);
    }
    __acc_8
}
}
    }
    ExprData::ExprMethodCall { receiver: recv, method: mname, args: mc_args, .. } => {
        if mname.clone() == "fold" {
    let recv_accum = walk_expr(accum.clone(), recv.clone(), false);
    let init_arg = {
    let mut __found_12 = None;
    for __elem_13 in mc_args.iter().cloned() {
        if __elem_13.name.clone() == Some("init".to_string()) {
    __found_12 = Some(__elem_13);
    break;
};
    }
    __found_12
};
    let threaded_accum = match init_arg.clone() {
    Some(ia) => {
        match ia.value.expr_data.as_ref() {
    ExprData::ExprVar { name: vn, .. } => {
        record_use(recv_accum.clone(), &vn, EdgeKind::Threaded, "fold_init")
    }
    _ => {
        walk_expr(recv_accum.clone(), ia.value.clone(), false)
    }
}
    }
    None => {
        recv_accum.clone()
    }
};
    let non_init = {
    let mut __filtered_14 = Vec::new();
    for __elem_15 in mc_args.iter().cloned() {
        if __elem_15.name.clone() != Some("init".to_string()) {
    __filtered_14.push(__elem_15);
};
    }
    Rc::new(__filtered_14)
};
    {
    let mut __acc_16 = threaded_accum.clone();
    for __elem_17 in non_init.iter().cloned() {
        __acc_16 = walk_expr(__acc_16.clone(), __elem_17.value.clone(), false);
    }
    __acc_16
}
} else {
    let recv_accum = walk_expr(accum.clone(), recv.clone(), false);
    {
    let mut __acc_18 = recv_accum.clone();
    for __elem_19 in mc_args.iter().cloned() {
        __acc_18 = walk_expr(__acc_18.clone(), __elem_19.value.clone(), false);
    }
    __acc_18
}
}
    }
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        {
    let s_accum = walk_expr(accum.clone(), s.clone(), false);
    let branch_accums = {
    let mut __mapped_20 = Vec::new();
    for __elem_21 in arm_list.iter().cloned() {
        __mapped_20.push(walk_expr(s_accum.clone(), __elem_21.body.clone(), in_tail.clone()));
    }
    Rc::new(__mapped_20)
};
    merge_branch_usages(s_accum.clone(), branch_accums.clone())
}
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        {
    let c_accum = walk_expr(accum.clone(), c.clone(), false);
    let t_accum = walk_expr(c_accum.clone(), t.clone(), in_tail.clone());
    let e_accum = match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        walk_expr(c_accum.clone(), eb.clone(), in_tail.clone())
    }
    None => {
        c_accum.clone()
    }
};
    merge_branch_usages(c_accum.clone(), Rc::new(vec!(t_accum.clone(), e_accum.clone())))
}
    }
    ExprData::ExprLet { name: _, value: v, body: bd, .. } => {
        {
    let v_accum = walk_expr(accum.clone(), v.clone(), false);
    match bd.as_ref().map(|__rc| __rc.as_ref()) {
    Some(b) => {
        let b = Rc::new(b.clone());
        walk_expr(v_accum.clone(), b.clone(), in_tail.clone())
    }
    None => {
        v_accum.clone()
    }
}
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        {
    let ss_count = {
    let __len_22 = ss.clone().len();
    __len_22 as i64
};
    if ss_count.clone() == 0_i64 {
    accum.clone()
} else {
    let init_accum = {
    let mut __acc_28 = accum.clone();
    for __elem_29 in ({
    let mut __filtered_26 = Vec::new();
    for __elem_27 in ({
    let mut __enumerated_23 = Vec::new();
    for (__idx_24, __elem_25) in ss.clone().iter().enumerate() {
        __enumerated_23.push((__idx_24 as i64, __elem_25.clone()));
    }
    Rc::new(__enumerated_23)
}).iter().cloned() {
        if __elem_27.0.clone() < (ss_count.clone() - 1_i64) {
    __filtered_26.push(__elem_27);
};
    }
    Rc::new(__filtered_26)
}).iter().cloned() {
        __acc_28 = walk_expr(__acc_28.clone(), __elem_29.1.clone(), false);
    }
    __acc_28
};
    match ss.clone().last().cloned() {
    Some(last_expr) => {
        walk_expr(init_accum.clone(), last_expr.clone(), in_tail.clone())
    }
    None => {
        init_accum.clone()
    }
}
}
}
    }
    ExprData::ExprBinOp { op: _, left: l, right: r, .. } => {
        {
    let l_accum = walk_expr(accum.clone(), l.clone(), false);
    walk_expr(l_accum.clone(), r.clone(), false)
}
    }
    ExprData::ExprUnaryOp { op: _, operand: e, .. } => {
        walk_expr(accum.clone(), e.clone(), false)
    }
    ExprData::ExprLambda { params: _, body: bd, .. } => {
        walk_expr(accum.clone(), bd.clone(), false)
    }
    ExprData::ExprRecordLit { type_name: _, fields: fs, .. } => {
        {
    let mut __acc_30 = accum.clone();
    for __elem_31 in fs.iter().cloned() {
        __acc_30 = walk_expr(__acc_30.clone(), __elem_31.value.clone(), false);
    }
    __acc_30
}
    }
    ExprData::ExprListLit { elements: els, .. } => {
        {
    let mut __acc_32 = accum.clone();
    for __elem_33 in els.iter().cloned() {
        __acc_32 = walk_expr(__acc_32.clone(), __elem_33.clone(), false);
    }
    __acc_32
}
    }
    ExprData::ExprStringInterp { parts: ps, .. } => {
        {
    let mut __acc_34 = accum.clone();
    for __elem_35 in ps.iter().cloned() {
        __acc_34 = match __elem_35.as_ref() {
    StringPart::Interpolation { expr: e, .. } => {
        walk_expr(__acc_34.clone(), e.clone(), false)
    }
    StringPart::Text { value: _, .. } => {
        __acc_34.clone()
    }
};
    }
    __acc_34
}
    }
    ExprData::ExprCast { expr: e, target: _, .. } => {
        walk_expr(accum.clone(), e.clone(), false)
    }
    ExprData::ExprForEach { variable: _, collection: c, body: bd, .. } => {
        {
    let c_accum = walk_expr(accum.clone(), c.clone(), false);
    walk_expr(c_accum.clone(), bd.clone(), false)
}
    }
    ExprData::ExprIndex { base: b, index: i, .. } => {
        {
    let b_accum = walk_expr(accum.clone(), b.clone(), false);
    walk_expr(b_accum.clone(), i.clone(), false)
}
    }
    ExprData::ExprSlice { base: b, start: s, end: e, .. } => {
        {
    let b_accum = walk_expr(accum.clone(), b.clone(), false);
    let s_accum = walk_expr(b_accum.clone(), s.clone(), false);
    walk_expr(s_accum.clone(), e.clone(), false)
}
    }
    ExprData::ExprReturn { value: v, .. } => {
        walk_expr(accum.clone(), v.clone(), true)
    }
    ExprData::ExprError { kind: _, message: _, .. } => {
        accum.clone()
    }
    ExprData::NoExprData => {
        accum.clone()
    }
}
    })
}

pub fn make_decision(usage: Rc<BindingUsage>) -> Rc<OwnershipDecision> {
    let sc = semantic_consumer_count(usage.clone());
    if sc.clone() == 1_i64 {
    let consumed_sites = {
    let mut __filtered_0 = Vec::new();
    for __elem_1 in usage.consumers.iter().cloned() {
        if match __elem_1.kind.clone() {
    EdgeKind::Consumed => {
        true
    }
    _ => {
        false
    }
} {
    __filtered_0.push(__elem_1);
};
    }
    Rc::new(__filtered_0)
};
    let site = match consumed_sites.clone().first().cloned() {
    Some(c) => {
        c.site.clone()
    }
    None => {
        "unknown".to_string()
    }
};
    Rc::new(OwnershipDecision::SoleOwner { binding: usage.name.clone(), site: site.clone() })
} else {
    if sc.clone() > 1_i64 {
    let sites = {
    let mut __mapped_4 = Vec::new();
    for __elem_5 in ({
    let mut __filtered_2 = Vec::new();
    for __elem_3 in usage.consumers.iter().cloned() {
        if match __elem_3.kind.clone() {
    EdgeKind::Consumed => {
        true
    }
    _ => {
        false
    }
} {
    __filtered_2.push(__elem_3);
};
    }
    Rc::new(__filtered_2)
}).iter().cloned() {
        __mapped_4.push(__elem_5.site.clone());
    }
    Rc::new(__mapped_4)
};
    Rc::new(OwnershipDecision::SharedError { binding: usage.name.clone(), consumer_count: sc.clone(), sites: sites.clone() })
} else {
    Rc::new(OwnershipDecision::Unclassified { binding: usage.name.clone(), reason: "no consumers found".to_string() })
}
}
}

pub fn analyze_ownership(func_name: &str, params: Rc<Vec<Rc<Param>>>, body: Rc<Node>) -> Rc<OwnershipProof> {
    let initial = {
    let mut __acc_0 = empty_usage_accum();
    for __elem_1 in params.iter().cloned() {
        __acc_0 = Rc::new(UsageAccum { bindings: {
    let __rc_3 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).bindings);
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.name.clone(), Rc::new(BindingUsage { name: __elem_1.name.clone(), consumers: Rc::new(Vec::new()) }));
    Rc::new(__map_ins_2)
} });
    }
    __acc_0
};
    let result = walk_expr(initial.clone(), body.clone(), true);
    let binding_list = {
    let __rc_4 = result.bindings.clone();
    let __map_unwrapped_5 = Rc::try_unwrap(__rc_4).unwrap_or_else(|rc| (*rc).clone());
    let mut __entries_6 = __map_unwrapped_5.into_iter().collect::<Vec<_>>();
    __entries_6.sort_by(|a, b| a.0.cmp(&b.0));
    let __values_7 = __entries_6.into_iter().map(|(_, value)| value).collect::<Vec<_>>();
    Rc::new(__values_7)
};
    let decisions = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in binding_list.iter().cloned() {
        __mapped_8.push(make_decision(__elem_9.clone()));
    }
    Rc::new(__mapped_8)
};
    Rc::new(OwnershipProof { func_name: func_name.to_string(), bindings: result.bindings.clone(), decisions: decisions.clone() })
}

