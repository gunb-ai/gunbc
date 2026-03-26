use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeExpr {
    SizeConst { value: i64 },
    SizeVar { name: String },
    SizeLen { collection: String },
    SizeAdd { left: Rc<SizeExpr>, right: Rc<SizeExpr> },
    SizeMax { left: Rc<SizeExpr>, right: Rc<SizeExpr> },
}

impl Default for SizeExpr {
    fn default() -> Self {
        SizeExpr::SizeConst { value: Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostExpr {
    CostConst { value: i64 },
    CostAdd { left: Rc<CostExpr>, right: Rc<CostExpr> },
    CostMul { left: Rc<CostExpr>, right: Rc<CostExpr> },
    CostMax { left: Rc<CostExpr>, right: Rc<CostExpr> },
    CostSum { binder: String, upper: Rc<SizeExpr>, body: Rc<CostExpr> },
    CostUnknown { reason: String },
}

impl Default for CostExpr {
    fn default() -> Self {
        CostExpr::CostConst { value: Default::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum Certainty {
    #[default]
    Proven,
    Conservative,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CostShape {
    #[default]
    ShapeConstant,
    ShapeLinearScan { produces_collection: bool },
    ShapeIterateBody { produces_collection: bool },
    ShapeSortBody,
}

pub fn intrinsic_method_cost_shape(method: IntrinsicMethod) -> Rc<CostShape> {
    match method {
    IntrinsicMethod::MethodMap => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: true })
    }
    IntrinsicMethod::MethodFlatMap => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: true })
    }
    IntrinsicMethod::MethodFilter => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: true })
    }
    IntrinsicMethod::MethodEnumerate => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: true })
    }
    IntrinsicMethod::MethodSkip => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: true })
    }
    IntrinsicMethod::MethodTake => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: true })
    }
    IntrinsicMethod::MethodFold => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: false })
    }
    IntrinsicMethod::MethodAny => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: false })
    }
    IntrinsicMethod::MethodAll => {
        Rc::new(CostShape::ShapeIterateBody { produces_collection: false })
    }
    IntrinsicMethod::MethodSortBy => {
        Rc::new(CostShape::ShapeSortBody)
    }
    IntrinsicMethod::MethodCount => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: false })
    }
    IntrinsicMethod::MethodFirst => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: false })
    }
    IntrinsicMethod::MethodLast => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: false })
    }
    IntrinsicMethod::MethodJoin => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: false })
    }
    IntrinsicMethod::MethodStringContains => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: false })
    }
    IntrinsicMethod::MethodConcat => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: false })
    }
    IntrinsicMethod::MethodChars => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: true })
    }
    IntrinsicMethod::MethodSplit => {
        Rc::new(CostShape::ShapeLinearScan { produces_collection: true })
    }
    IntrinsicMethod::MethodAppend => {
        Rc::new(CostShape::ShapeConstant)
    }
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComplexitySummary {
    pub work: Rc<CostExpr>,
    pub span: Rc<CostExpr>,
    pub output_size: Rc<HashMap<String, Rc<CostExpr>>>,
    pub certainty: Certainty,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CostInternTable {
    pub summaries: Rc<HashMap<String, Rc<ComplexitySummary>>>,
}

pub fn empty_intern_table() -> Rc<CostInternTable> {
    Rc::new(CostInternTable { summaries: Rc::new(std::collections::HashMap::new()) })
}

pub fn cache_summary(table: Rc<CostInternTable>, func_name: &str, summary: Rc<ComplexitySummary>) -> Rc<CostInternTable> {
    Rc::new(CostInternTable { summaries: {
    let __rc_1 = table.summaries.clone();
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert(func_name.to_string(), summary.clone());
    Rc::new(__map_ins_0)
} })
}

pub fn lookup_summary(table: Rc<CostInternTable>, func_name: &str) -> Option<Rc<ComplexitySummary>> {
    table.summaries.clone().get(&func_name.to_string()).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecursionPattern {
    LinearRecursion { iteration_var: String },
    DivideAndConquer { split_factor: i64 },
    UnresolvableRecursion { reason: String },
}

impl Default for RecursionPattern {
    fn default() -> Self {
        RecursionPattern::LinearRecursion { iteration_var: Default::default() }
    }
}

pub fn cost_seq(a: Rc<CostExpr>, b: Rc<CostExpr>) -> Rc<CostExpr> {
    Rc::new(CostExpr::CostAdd { left: a.clone(), right: b.clone() })
}

pub fn cost_par(a: Rc<CostExpr>, b: Rc<CostExpr>) -> Rc<CostExpr> {
    Rc::new(CostExpr::CostMax { left: a.clone(), right: b.clone() })
}

pub fn cost_loop(binder: &str, iterations: Rc<SizeExpr>, body: Rc<CostExpr>) -> Rc<CostExpr> {
    Rc::new(CostExpr::CostSum { binder: binder.to_string(), upper: iterations.clone(), body: body.clone() })
}

pub fn cost_conditional(condition: Rc<CostExpr>, branches: Rc<Vec<Rc<CostExpr>>>) -> Rc<CostExpr> {
    let max_branch = {
    let mut __acc_0 = Rc::new(CostExpr::CostConst { value: 0_i64 });
    for __elem_1 in branches.iter().cloned() {
        __acc_0 = Rc::new(CostExpr::CostMax { left: __acc_0.clone(), right: __elem_1.clone() });
    }
    __acc_0
};
    Rc::new(CostExpr::CostAdd { left: condition.clone(), right: max_branch.clone() })
}

pub fn collection_output(binder: &str, size: Rc<SizeExpr>) -> Rc<HashMap<String, Rc<CostExpr>>> {
    let result = {
    let __rc_1 = Rc::new(std::collections::HashMap::new());
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert("result".to_string(), cost_loop(&binder, size.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })));
    Rc::new(__map_ins_0)
};
    result.clone()
}

pub fn scalar_output() -> Rc<HashMap<String, Rc<CostExpr>>> {
    let result = Rc::new(std::collections::HashMap::new());
    result.clone()
}

pub fn is_size_preserving_intrinsic_method(method: IntrinsicMethod) -> bool {
    ((((((method.clone() == IntrinsicMethod::MethodMap) || (method.clone() == IntrinsicMethod::MethodFilter)) || (method.clone() == IntrinsicMethod::MethodFlatMap)) || (method.clone() == IntrinsicMethod::MethodSkip)) || (method.clone() == IntrinsicMethod::MethodEnumerate)) || (method.clone() == IntrinsicMethod::MethodSortBy)) || (method.clone() == IntrinsicMethod::MethodConcat)
}

pub fn method_preserves_collection_size(method_semantics: Option<Rc<MethodSemantics>>) -> bool {
    if method_semantics.clone().is_none() {
    false
} else {
    match method_semantics.clone().unwrap().as_ref() {
    MethodSemantics::IntrinsicMethodSemantics { intrinsic: method, fold_accumulator_type: _, .. } => {
        is_size_preserving_intrinsic_method(method.clone())
    }
    MethodSemantics::RuntimeBridgeSemantics { method: bridge_method, .. } => {
        bridge_method.clone() == RuntimeBridgeMethod::BridgeReverse
    }
    _ => {
        false
    }
}
}
}

pub fn receiver_size_var(recv: Rc<Node>) -> Rc<SizeExpr> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        let mut __tco_p_recv = recv;
        loop {
            let recv = __tco_p_recv;
            match recv.expr_data.as_ref() {
    ExprData::ExprVar { name: vname, .. } => {
        break Rc::new(SizeExpr::SizeLen { collection: vname.clone() });
    }
    ExprData::ExprFieldAccess { base: _, field: fname, .. } => {
        break Rc::new(SizeExpr::SizeLen { collection: fname.clone() });
    }
    ExprData::ExprMethodCall { receiver: inner_recv, method: _, args: _, method_semantics, .. } => {
        if method_preserves_collection_size(method_semantics.clone()) {
     {
        let __tco_0 = inner_recv.clone();
        __tco_p_recv = __tco_0;
        continue;
    }

} else {
    break Rc::new(SizeExpr::SizeLen { collection: "__expr".to_string() });
};
    }
    _ => {
        break Rc::new(SizeExpr::SizeLen { collection: "__expr".to_string() });
    }
};
        }
    })
}

pub fn size_binder_name(size: Rc<SizeExpr>) -> String {
    match size.as_ref() {
    SizeExpr::SizeLen { collection: c, .. } => {
        v2_rt::concat("_".to_string(), c.clone())
    }
    SizeExpr::SizeVar { name: n, .. } => {
        n.clone()
    }
    _ => {
        "_i".to_string()
    }
}
}

pub fn resolve_lambda_arg(mc_args: Rc<Vec<Rc<NamedArg>>>) -> Option<Rc<Node>> {
    let f_arg = {
    let mut __found_2 = None;
    for __elem_3 in mc_args.iter().cloned() {
        if __elem_3.name.clone() == Some("f".to_string()) {
    __found_2 = Some(__elem_3);
    break;
};
    }
    __found_2
};
    match f_arg.clone() {
    Some(fa) => {
        Some(fa.value.clone())
    }
    None => {
        ({
    let mut __mapped_4 = Vec::new();
    for __elem_5 in mc_args.iter().cloned() {
        __mapped_4.push(__elem_5.value.clone());
    }
    Rc::new(__mapped_4)
}).first().cloned()
    }
}
}

pub fn resolve_callback_cost(lambda_arg: Option<Rc<Node>>, recv_r: Rc<SummaryResult>, func_index: Rc<HashMap<String, Rc<FuncEntry>>>) -> Rc<SummaryResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match lambda_arg.as_ref().map(|__rc| __rc.as_ref()) {
    Some(la) => {
        let la = Rc::new(la.clone());
        match la.expr_data.as_ref() {
    ExprData::ExprVar { name: fn_ref, .. } => {
        match func_index.clone().get(&fn_ref.clone()).cloned() {
    Some(_) => {
        get_or_compute_summary(&fn_ref, func_index.clone(), recv_r.table.clone())
    }
    None => {
        cost_of_expr(la.clone(), func_index.clone(), recv_r.table.clone())
    }
}
    }
    _ => {
        cost_of_expr(la.clone(), func_index.clone(), recv_r.table.clone())
    }
}
    }
    None => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Conservative }), table: recv_r.table.clone() })
    }
}
    })
}

pub fn cost_of_method_by_shape(shape: Rc<CostShape>, recv_r: Rc<SummaryResult>, mc_args: Rc<Vec<Rc<NamedArg>>>, size: Rc<SizeExpr>, binder: &str, func_index: Rc<HashMap<String, Rc<FuncEntry>>>) -> Rc<SummaryResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match shape.as_ref() {
    CostShape::ShapeIterateBody { produces_collection: pc, .. } => {
        {
    let lambda_arg = resolve_lambda_arg(mc_args.clone());
    let body_result = resolve_callback_cost(lambda_arg.clone(), recv_r.clone(), func_index.clone());
    let loop_work = cost_loop(&binder, size.clone(), body_result.summary.work.clone());
    let os = if pc.clone() == false {
    body_result.summary.output_size.clone()
} else {
    let r = {
    let __rc_1 = Rc::new(std::collections::HashMap::new());
    let mut __map_ins_0 = Rc::try_unwrap(__rc_1).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_0.insert("result".to_string(), cost_loop(&binder, size.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })));
    Rc::new(__map_ins_0)
};
    r.clone()
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(recv_r.summary.work.clone(), loop_work.clone()), span: cost_seq(recv_r.summary.span.clone(), loop_work.clone()), output_size: os.clone(), certainty: body_result.summary.certainty.clone() }), table: body_result.table.clone() })
}
    }
    CostShape::ShapeSortBody => {
        {
    let lambda_arg = resolve_lambda_arg(mc_args.clone());
    let key_result = match lambda_arg.as_ref().map(|__rc| __rc.as_ref()) {
    Some(la) => {
        let la = Rc::new(la.clone());
        cost_of_expr(la.clone(), func_index.clone(), recv_r.table.clone())
    }
    None => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Conservative }), table: recv_r.table.clone() })
    }
};
    let sort_work = cost_loop(&binder, size.clone(), key_result.summary.work.clone());
    let sort_os = {
    let __rc_3 = Rc::new(std::collections::HashMap::new());
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert("result".to_string(), cost_loop(&binder, size.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })));
    Rc::new(__map_ins_2)
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(recv_r.summary.work.clone(), sort_work.clone()), span: cost_seq(recv_r.summary.span.clone(), sort_work.clone()), output_size: sort_os.clone(), certainty: Certainty::Conservative }), table: key_result.table.clone() })
}
    }
    CostShape::ShapeLinearScan { produces_collection: pc, .. } => {
        {
    let loop_work = cost_loop(&binder, size.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 }));
    let scan_os = if pc.clone() {
    let r = {
    let __rc_5 = Rc::new(std::collections::HashMap::new());
    let mut __map_ins_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_4.insert("result".to_string(), cost_loop(&binder, size.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })));
    Rc::new(__map_ins_4)
};
    r.clone()
} else {
    let r = Rc::new(std::collections::HashMap::new());
    r.clone()
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(recv_r.summary.work.clone(), loop_work.clone()), span: cost_seq(recv_r.summary.span.clone(), loop_work.clone()), output_size: scan_os.clone(), certainty: recv_r.summary.certainty.clone() }), table: recv_r.table.clone() })
}
    }
    CostShape::ShapeConstant => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(recv_r.summary.work.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })), span: cost_seq(recv_r.summary.span.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })), output_size: Rc::new(std::collections::HashMap::new()), certainty: recv_r.summary.certainty.clone() }), table: recv_r.table.clone() })
    }
}
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComplexityViolation {
    pub func_name: String,
    pub reason: String,
    pub summary: Option<Rc<ComplexitySummary>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComplexityReport {
    pub function_summaries: Rc<HashMap<String, Rc<ComplexitySummary>>>,
    pub violations: Rc<Vec<Rc<ComplexityViolation>>>,
    pub intern_table: Rc<CostInternTable>,
    pub formatted: String,
}

pub fn empty_complexity_report() -> Rc<ComplexityReport> {
    Rc::new(ComplexityReport { function_summaries: Rc::new(std::collections::HashMap::new()), violations: Rc::new(Vec::new()), intern_table: empty_intern_table(), formatted: "".to_string() })
}

pub fn estimate_cost_expr_size(expr: Rc<CostExpr>, budget: i64) -> i64 {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if budget.clone() <= 0_i64 {
    0_i64
} else {
    match expr.as_ref() {
    CostExpr::CostConst { value: _, .. } => {
        budget.clone() - 1_i64
    }
    CostExpr::CostUnknown { reason: _, .. } => {
        budget.clone() - 1_i64
    }
    CostExpr::CostAdd { left: l, right: r, .. } => {
        {
    let next = estimate_cost_expr_size(l.clone(), budget.clone() - 1_i64);
    estimate_cost_expr_size(r.clone(), next)
}
    }
    CostExpr::CostMul { left: l, right: r, .. } => {
        {
    let next = estimate_cost_expr_size(l.clone(), budget.clone() - 1_i64);
    estimate_cost_expr_size(r.clone(), next)
}
    }
    CostExpr::CostMax { left: l, right: r, .. } => {
        {
    let next = estimate_cost_expr_size(l.clone(), budget.clone() - 1_i64);
    estimate_cost_expr_size(r.clone(), next)
}
    }
    CostExpr::CostSum { binder: _, upper: _, body: bd, .. } => {
        estimate_cost_expr_size(bd.clone(), budget.clone() - 1_i64)
    }
}
}
    })
}

pub fn simplify_cost(expr: Rc<CostExpr>) -> Rc<CostExpr> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.as_ref() {
    CostExpr::CostConst { value: _, .. } => {
        expr.clone()
    }
    CostExpr::CostUnknown { reason: _, .. } => {
        expr.clone()
    }
    CostExpr::CostAdd { left: l, right: r, .. } => {
        {
    let sl = simplify_cost(l.clone());
    let sr = simplify_cost(r.clone());
    let simplified = match sl.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        sr.clone()
    }
    CostExpr::CostConst { value: a, .. } => {
        match sr.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        sl.clone()
    }
    CostExpr::CostConst { value: b, .. } => {
        Rc::new(CostExpr::CostConst { value: a.clone() + b.clone() })
    }
    _ => {
        Rc::new(CostExpr::CostAdd { left: sl.clone(), right: sr.clone() })
    }
}
    }
    _ => {
        match sr.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        sl.clone()
    }
    _ => {
        Rc::new(CostExpr::CostAdd { left: sl.clone(), right: sr.clone() })
    }
}
    }
};
    simplified.clone()
}
    }
    CostExpr::CostMul { left: l, right: r, .. } => {
        {
    let sl = simplify_cost(l.clone());
    let sr = simplify_cost(r.clone());
    let simplified = match sl.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        Rc::new(CostExpr::CostConst { value: 0_i64 })
    }
    CostExpr::CostConst { value: 1, .. } => {
        sr.clone()
    }
    _ => {
        match sr.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        Rc::new(CostExpr::CostConst { value: 0_i64 })
    }
    CostExpr::CostConst { value: 1, .. } => {
        sl.clone()
    }
    _ => {
        Rc::new(CostExpr::CostMul { left: sl.clone(), right: sr.clone() })
    }
}
    }
};
    simplified.clone()
}
    }
    CostExpr::CostMax { left: l, right: r, .. } => {
        {
    let sl = simplify_cost(l.clone());
    let sr = simplify_cost(r.clone());
    let simplified = match sl.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        sr.clone()
    }
    _ => {
        match sr.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        sl.clone()
    }
    _ => {
        Rc::new(CostExpr::CostMax { left: sl.clone(), right: sr.clone() })
    }
}
    }
};
    simplified.clone()
}
    }
    CostExpr::CostSum { binder: b, upper: u, body: bd, .. } => {
        {
    let sbd = simplify_cost(bd.clone());
    match sbd.as_ref() {
    CostExpr::CostConst { value: 0, .. } => {
        Rc::new(CostExpr::CostConst { value: 0_i64 })
    }
    _ => {
        Rc::new(CostExpr::CostSum { binder: b.clone(), upper: u.clone(), body: sbd.clone() })
    }
}
}
    }
}
    })
}

pub fn maybe_simplify_cost(expr: Rc<CostExpr>) -> Rc<CostExpr> {
    if estimate_cost_expr_size(expr.clone(), 1024_i64) <= 0_i64 {
    expr.clone()
} else {
    simplify_cost(expr.clone())
}
}

pub fn format_size(size: Rc<SizeExpr>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match size.as_ref() {
    SizeExpr::SizeLen { collection: c, .. } => {
        v2_rt::concat(v2_rt::concat("|".to_string(), c.clone()), "|".to_string())
    }
    SizeExpr::SizeVar { name: n, .. } => {
        n.clone()
    }
    SizeExpr::SizeConst { value: v, .. } => {
        v2_rt::to_string(v.clone())
    }
    SizeExpr::SizeAdd { left: l, right: r, .. } => {
        v2_rt::concat(v2_rt::concat(format_size(l.clone()), " + ".to_string()), format_size(r.clone()))
    }
    SizeExpr::SizeMax { left: l, right: r, .. } => {
        v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("max(".to_string(), format_size(l.clone())), ", ".to_string()), format_size(r.clone())), ")".to_string())
    }
}
    })
}

pub fn strip_big_o(s: &str) -> String {
    if (v2_rt::string_length(&s) > 3_i64) && (v2_rt::substring(&s, 0_i64, 2_i64) == "O(") {
    v2_rt::substring(&s, 2_i64, v2_rt::string_length(&s) - 1_i64)
} else {
    s.to_string()
}
}

pub fn has_cost_sum(expr: Rc<CostExpr>) -> bool {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.as_ref() {
    CostExpr::CostSum { binder: _, upper: _, body: _, .. } => {
        true
    }
    CostExpr::CostAdd { left: l, right: r, .. } => {
        has_cost_sum(l.clone()) || has_cost_sum(r.clone())
    }
    CostExpr::CostMul { left: l, right: r, .. } => {
        has_cost_sum(l.clone()) || has_cost_sum(r.clone())
    }
    CostExpr::CostMax { left: l, right: r, .. } => {
        has_cost_sum(l.clone()) || has_cost_sum(r.clone())
    }
    _ => {
        false
    }
}
    })
}

pub fn cost_sum_depth(expr: Rc<CostExpr>) -> i64 {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.as_ref() {
    CostExpr::CostSum { binder: _, upper: _, body: b, .. } => {
        1_i64 + cost_sum_depth(b.clone())
    }
    CostExpr::CostAdd { left: l, right: r, .. } => {
        {
    let ld = cost_sum_depth(l.clone());
    let rd = cost_sum_depth(r.clone());
    if rd.clone() > ld.clone() {
    rd.clone()
} else {
    ld.clone()
}
}
    }
    CostExpr::CostMul { left: l, right: r, .. } => {
        {
    let ld = cost_sum_depth(l.clone());
    let rd = cost_sum_depth(r.clone());
    if rd.clone() > ld.clone() {
    rd.clone()
} else {
    ld.clone()
}
}
    }
    CostExpr::CostMax { left: l, right: r, .. } => {
        {
    let ld = cost_sum_depth(l.clone());
    let rd = cost_sum_depth(r.clone());
    if rd.clone() > ld.clone() {
    rd.clone()
} else {
    ld.clone()
}
}
    }
    _ => {
        0_i64
    }
}
    })
}

pub fn classify_complexity(expr: Rc<CostExpr>) -> String {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.as_ref() {
    CostExpr::CostConst { value: _, .. } => {
        "O(1)".to_string()
    }
    CostExpr::CostUnknown { reason: _, .. } => {
        "O(?)".to_string()
    }
    CostExpr::CostSum { binder: _, upper: u, body: bd, .. } => {
        if has_cost_sum(bd.clone()) {
    let inner_class = classify_complexity(bd.clone());
    let inner_bare = strip_big_o(&inner_class);
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat("O(".to_string(), format_size(u.clone())), " * ".to_string()), inner_bare), ")".to_string())
} else {
    v2_rt::concat(v2_rt::concat("O(".to_string(), format_size(u.clone())), ")".to_string())
}
    }
    CostExpr::CostAdd { left: l, right: r, .. } => {
        {
    let lc = classify_complexity(l.clone());
    let rc = classify_complexity(r.clone());
    let dominant = if has_cost_sum(l.clone()) && has_cost_sum(r.clone()) {
    if cost_sum_depth(r.clone()) > cost_sum_depth(l.clone()) {
    rc.clone()
} else {
    lc.clone()
}
} else {
    if has_cost_sum(l.clone()) {
    lc.clone()
} else {
    if has_cost_sum(r.clone()) {
    rc.clone()
} else {
    "O(1)".to_string()
}
}
};
    dominant.clone()
}
    }
    CostExpr::CostMul { left: l, right: r, .. } => {
        {
    let lc = classify_complexity(l.clone());
    let rc = classify_complexity(r.clone());
    let combined = if lc.clone() == "O(1)" {
    rc.clone()
} else {
    if rc.clone() == "O(1)" {
    lc.clone()
} else {
    v2_rt::concat(v2_rt::concat(lc.clone(), " * ".to_string()), rc.clone())
}
};
    combined.clone()
}
    }
    CostExpr::CostMax { left: l, right: r, .. } => {
        {
    let lc = classify_complexity(l.clone());
    let rc = classify_complexity(r.clone());
    let dominant = if has_cost_sum(l.clone()) {
    lc.clone()
} else {
    if has_cost_sum(r.clone()) {
    rc.clone()
} else {
    lc.clone()
}
};
    dominant.clone()
}
    }
}
    })
}

pub fn collect_size_vars_from_size(size: Rc<SizeExpr>) -> Rc<Vec<String>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match size.as_ref() {
    SizeExpr::SizeLen { collection: c, .. } => {
        if c.clone() == "__expr" {
    Rc::new(Vec::new())
} else {
    Rc::new(vec!(c.clone()))
}
    }
    SizeExpr::SizeAdd { left: l, right: r, .. } => {
        v2_rt::concat(collect_size_vars_from_size(l.clone()), collect_size_vars_from_size(r.clone()))
    }
    SizeExpr::SizeMax { left: l, right: r, .. } => {
        v2_rt::concat(collect_size_vars_from_size(l.clone()), collect_size_vars_from_size(r.clone()))
    }
    _ => {
        Rc::new(Vec::new())
    }
}
    })
}

pub fn collect_size_vars(expr: Rc<CostExpr>) -> Rc<Vec<String>> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match expr.as_ref() {
    CostExpr::CostSum { binder: _, upper: u, body: bd, .. } => {
        v2_rt::concat(collect_size_vars_from_size(u.clone()), collect_size_vars(bd.clone()))
    }
    CostExpr::CostAdd { left: l, right: r, .. } => {
        v2_rt::concat(collect_size_vars(l.clone()), collect_size_vars(r.clone()))
    }
    CostExpr::CostMul { left: l, right: r, .. } => {
        v2_rt::concat(collect_size_vars(l.clone()), collect_size_vars(r.clone()))
    }
    CostExpr::CostMax { left: l, right: r, .. } => {
        v2_rt::concat(collect_size_vars(l.clone()), collect_size_vars(r.clone()))
    }
    _ => {
        Rc::new(Vec::new())
    }
}
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeduplicateAcc {
    pub seen: Rc<HashMap<String, bool>>,
    pub out: Rc<Vec<String>>,
}

pub fn deduplicate(items: Rc<Vec<String>>) -> Rc<Vec<String>> {
    let result = {
    let mut __acc_0 = Rc::new(DeduplicateAcc { seen: Rc::new(std::collections::HashMap::new()), out: Rc::new(Vec::new()) });
    for __elem_1 in items.iter().cloned() {
        __acc_0 = match __acc_0.seen.clone().get(&__elem_1.clone()).cloned() {
    Some(_) => {
        __acc_0.clone()
    }
    None => {
        Rc::new(DeduplicateAcc { seen: {
    let __rc_3 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).seen);
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.clone(), true);
    Rc::new(__map_ins_2)
}, out: {
    let __rc_5 = std::mem::take(&mut Rc::make_mut(&mut __acc_0).out);
    let mut __appended_4 = Rc::try_unwrap(__rc_5).unwrap_or_else(|rc| (*rc).clone());
    __appended_4.push(__elem_1.clone());
    Rc::new(__appended_4)
} })
    }
};
    }
    __acc_0
};
    result.out.clone()
}

pub fn format_space_class(expr: Rc<CostExpr>) -> String {
    let space_class = classify_complexity(expr.clone());
    if space_class.clone() == "O(1)" {
    "".to_string()
} else {
    v2_rt::concat(", space ".to_string(), space_class.clone())
}
}

pub fn format_func_complexity(name: &str, summary: Rc<ComplexitySummary>) -> String {
    let class = classify_complexity(summary.work.clone());
    let marker = match summary.certainty.clone() {
    Certainty::Proven => {
        "".to_string()
    }
    Certainty::Conservative => {
        "~".to_string()
    }
    Certainty::Unknown => {
        "?".to_string()
    }
};
    let space_str = match summary.output_size.clone().get(&"result".to_string()).cloned() {
    Some(size_expr) => {
        format_space_class(size_expr.clone())
    }
    _ => {
        "".to_string()
    }
};
    v2_rt::concat(v2_rt::concat(v2_rt::concat(v2_rt::concat(name.to_string(), ": ".to_string()), marker.clone()), class), space_str.clone())
}

pub fn format_complexity_report(entries: Rc<Vec<Rc<FuncEntry>>>, summaries: Rc<HashMap<String, Rc<ComplexitySummary>>>) -> String {
    let lines = {
    let mut __flat_mapped_0 = Vec::new();
    for __elem_1 in entries.iter().cloned() {
        __flat_mapped_0.extend((match summaries.clone().get(&__elem_1.name.clone()).cloned() {
    Some(summary) => {
        Rc::new(vec!(format_func_complexity(&__elem_1.name, summary.clone())))
    }
    None => {
        Rc::new(Vec::new())
    }
}).iter().cloned());
    }
    Rc::new(__flat_mapped_0)
};
    {
    let mut __joined_2 = String::new();
    let mut __first_4 = true;
    for __elem_3 in lines.iter().cloned() {
        if !__first_4 {
    __joined_2.push_str(&"\n".to_string());
};
        __first_4 = false;
        __joined_2.push_str(&__elem_3);
    }
    __joined_2
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FuncEntry {
    pub name: String,
    pub body: Rc<Node>,
    pub params: Rc<Vec<Rc<Param>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SummaryResult {
    pub summary: Rc<ComplexitySummary>,
    pub table: Rc<CostInternTable>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchCostAccum {
    pub result: Rc<SummaryResult>,
    pub branch_costs: Rc<Vec<Rc<CostExpr>>>,
}

pub fn cost_of_expr(texpr: Rc<Node>, func_index: Rc<HashMap<String, Rc<FuncEntry>>>, table: Rc<CostInternTable>) -> Rc<SummaryResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match texpr.expr_data.as_ref() {
    ExprData::ExprLiteral { value: _, .. } => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() })
    }
    ExprData::ExprError { kind: _, message: _, .. } => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() })
    }
    ExprData::ExprVar { name: _, .. } => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() })
    }
    ExprData::ExprBinOp { op: _, left: l, right: r, .. } => {
        {
    let lr = cost_of_expr(l.clone(), func_index.clone(), table.clone());
    let rr = cost_of_expr(r.clone(), func_index.clone(), lr.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(lr.summary.work.clone(), cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), rr.summary.work.clone())), span: cost_seq(lr.summary.span.clone(), cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), rr.summary.span.clone())), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: rr.table.clone() })
}
    }
    ExprData::ExprCall { func: fname, args: call_args, .. } => {
        {
    let callee_result = match func_index.clone().get(&fname.clone()).cloned() {
    Some(entry) => {
        get_or_compute_summary(&fname, func_index.clone(), table.clone())
    }
    None => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() })
    }
};
    let args_result = {
    let mut __acc_0 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: callee_result.table.clone() });
    for __elem_1 in call_args.iter().cloned() {
        __acc_0 = {
    let ar = cost_of_expr(__elem_1.value.clone(), func_index.clone(), __acc_0.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(__acc_0.summary.work.clone(), ar.summary.work.clone()), span: cost_seq(__acc_0.summary.span.clone(), ar.summary.span.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: ar.table.clone() })
};
    }
    __acc_0
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(args_result.summary.work.clone(), callee_result.summary.work.clone()), span: cost_seq(args_result.summary.span.clone(), callee_result.summary.span.clone()), output_size: callee_result.summary.output_size.clone(), certainty: callee_result.summary.certainty.clone() }), table: args_result.table.clone() })
}
    }
    ExprData::ExprMethodCall { receiver: recv, method: mname, args: mc_args, method_semantics: ms, .. } => {
        {
    let recv_r = cost_of_expr(recv.clone(), func_index.clone(), table.clone());
    let size = receiver_size_var(recv.clone());
    let binder = size_binder_name(size.clone());
    let method_cost_result = if ms.clone().is_none() {
    None
} else {
    match ms.clone().unwrap().as_ref() {
    MethodSemantics::IntrinsicMethodSemantics { intrinsic: im, fold_accumulator_type: _, .. } => {
        {
    let shape = intrinsic_method_cost_shape(im.clone());
    Some(cost_of_method_by_shape(shape.clone(), recv_r.clone(), mc_args.clone(), size.clone(), &binder, func_index.clone()))
}
    }
    MethodSemantics::RuntimeBridgeSemantics { method: _, .. } => {
        Some(Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(recv_r.summary.work.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })), span: cost_seq(recv_r.summary.span.clone(), Rc::new(CostExpr::CostConst { value: 1_i64 })), output_size: Rc::new(std::collections::HashMap::new()), certainty: recv_r.summary.certainty.clone() }), table: recv_r.table.clone() }))
    }
    _ => {
        None
    }
}
};
    match method_cost_result.clone() {
    Some(result) => {
        result.clone()
    }
    _ => {
        {
    let args_result = {
    let mut __acc_2 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: recv_r.table.clone() });
    for __elem_3 in mc_args.iter().cloned() {
        __acc_2 = {
    let ar = cost_of_expr(__elem_3.value.clone(), func_index.clone(), __acc_2.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(__acc_2.summary.work.clone(), ar.summary.work.clone()), span: cost_seq(__acc_2.summary.span.clone(), ar.summary.span.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: ar.table.clone() })
};
    }
    __acc_2
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(recv_r.summary.work.clone(), cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), args_result.summary.work.clone())), span: cost_seq(recv_r.summary.span.clone(), cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), args_result.summary.span.clone())), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: args_result.table.clone() })
}
    }
}
}
    }
    ExprData::ExprMatch { scrutinee: s, arms: arm_list, .. } => {
        {
    let s_r = cost_of_expr(s.clone(), func_index.clone(), table.clone());
    let arms_accum = {
    let mut __acc_4 = Rc::new(MatchCostAccum { result: Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: s_r.table.clone() }), branch_costs: Rc::new(Vec::new()) });
    for __elem_5 in arm_list.iter().cloned() {
        __acc_4 = {
    let ar = cost_of_expr(__elem_5.body.clone(), func_index.clone(), __acc_4.result.table.clone());
    Rc::new(MatchCostAccum { result: Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_par(__acc_4.result.summary.work.clone(), ar.summary.work.clone()), span: cost_par(__acc_4.result.summary.span.clone(), ar.summary.span.clone()), output_size: ar.summary.output_size.clone(), certainty: ar.summary.certainty.clone() }), table: ar.table.clone() }), branch_costs: {
    let __rc_7 = std::mem::take(&mut Rc::make_mut(&mut __acc_4).branch_costs);
    let mut __appended_6 = Rc::try_unwrap(__rc_7).unwrap_or_else(|rc| (*rc).clone());
    __appended_6.push(ar.summary.work.clone());
    Rc::new(__appended_6)
} })
};
    }
    __acc_4
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_conditional(s_r.summary.work.clone(), arms_accum.branch_costs.clone()), span: cost_conditional(s_r.summary.span.clone(), Rc::new(vec!(arms_accum.result.summary.span.clone()))), output_size: arms_accum.result.summary.output_size.clone(), certainty: arms_accum.result.summary.certainty.clone() }), table: arms_accum.result.table.clone() })
}
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        {
    let c_r = cost_of_expr(c.clone(), func_index.clone(), table.clone());
    let t_r = cost_of_expr(t.clone(), func_index.clone(), c_r.table.clone());
    let e_result = match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        {
    let er = cost_of_expr(eb.clone(), func_index.clone(), t_r.table.clone());
    Rc::new(SummaryResult { summary: er.summary.clone(), table: er.table.clone() })
}
    }
    None => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: t_r.table.clone() })
    }
};
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_conditional(c_r.summary.work.clone(), Rc::new(vec!(t_r.summary.work.clone(), e_result.summary.work.clone()))), span: cost_conditional(c_r.summary.span.clone(), Rc::new(vec!(t_r.summary.span.clone(), e_result.summary.span.clone()))), output_size: t_r.summary.output_size.clone(), certainty: Certainty::Proven }), table: e_result.table.clone() })
}
    }
    ExprData::ExprLet { name: _, value: v, body: bd, .. } => {
        {
    let v_r = cost_of_expr(v.clone(), func_index.clone(), table.clone());
    match bd.as_ref().map(|__rc| __rc.as_ref()) {
    Some(b) => {
        let b = Rc::new(b.clone());
        {
    let b_r = cost_of_expr(b.clone(), func_index.clone(), v_r.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(v_r.summary.work.clone(), b_r.summary.work.clone()), span: cost_seq(v_r.summary.span.clone(), b_r.summary.span.clone()), output_size: b_r.summary.output_size.clone(), certainty: b_r.summary.certainty.clone() }), table: b_r.table.clone() })
}
    }
    None => {
        v_r.clone()
    }
}
}
    }
    ExprData::ExprBlock { stmts: ss, .. } => {
        {
    let mut __acc_8 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() });
    for __elem_9 in ss.iter().cloned() {
        __acc_8 = {
    let sr = cost_of_expr(__elem_9.clone(), func_index.clone(), __acc_8.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(__acc_8.summary.work.clone(), sr.summary.work.clone()), span: cost_seq(__acc_8.summary.span.clone(), sr.summary.span.clone()), output_size: sr.summary.output_size.clone(), certainty: sr.summary.certainty.clone() }), table: sr.table.clone() })
};
    }
    __acc_8
}
    }
    ExprData::ExprForEach { variable: _, collection: c, body: bd, .. } => {
        {
    let c_r = cost_of_expr(c.clone(), func_index.clone(), table.clone());
    let bd_r = cost_of_expr(bd.clone(), func_index.clone(), c_r.table.clone());
    let size = receiver_size_var(c.clone());
    let binder = size_binder_name(size.clone());
    let loop_work = cost_loop(&binder, size.clone(), bd_r.summary.work.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(c_r.summary.work.clone(), loop_work.clone()), span: cost_seq(c_r.summary.span.clone(), loop_work.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: bd_r.summary.certainty.clone() }), table: bd_r.table.clone() })
}
    }
    ExprData::ExprReturn { value: v, .. } => {
        cost_of_expr(v.clone(), func_index.clone(), table.clone())
    }
    ExprData::ExprFieldAccess { base: b, field: _, .. } => {
        cost_of_expr(b.clone(), func_index.clone(), table.clone())
    }
    ExprData::ExprUnaryOp { op: _, operand: e, .. } => {
        {
    let er = cost_of_expr(e.clone(), func_index.clone(), table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), er.summary.work.clone()), span: cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), er.summary.span.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: er.summary.certainty.clone() }), table: er.table.clone() })
}
    }
    ExprData::ExprLambda { params: _, body: bd, .. } => {
        cost_of_expr(bd.clone(), func_index.clone(), table.clone())
    }
    ExprData::ExprRecordLit { type_name: _, fields: fs, .. } => {
        {
    let mut __acc_10 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() });
    for __elem_11 in fs.iter().cloned() {
        __acc_10 = {
    let fr = cost_of_expr(__elem_11.value.clone(), func_index.clone(), __acc_10.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(__acc_10.summary.work.clone(), fr.summary.work.clone()), span: cost_seq(__acc_10.summary.span.clone(), fr.summary.span.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: fr.summary.certainty.clone() }), table: fr.table.clone() })
};
    }
    __acc_10
}
    }
    ExprData::ExprListLit { elements: els, .. } => {
        {
    let mut __acc_12 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() });
    for __elem_13 in els.iter().cloned() {
        __acc_12 = {
    let er = cost_of_expr(__elem_13.clone(), func_index.clone(), __acc_12.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(__acc_12.summary.work.clone(), er.summary.work.clone()), span: cost_seq(__acc_12.summary.span.clone(), er.summary.span.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: er.summary.certainty.clone() }), table: er.table.clone() })
};
    }
    __acc_12
}
    }
    ExprData::ExprStringInterp { parts: ps, .. } => {
        {
    let mut __acc_14 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 1_i64 }), span: Rc::new(CostExpr::CostConst { value: 1_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() });
    for __elem_15 in ps.iter().cloned() {
        __acc_14 = match __elem_15.as_ref() {
    StringPart::Interpolation { expr: e, .. } => {
        {
    let er = cost_of_expr(e.clone(), func_index.clone(), __acc_14.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(__acc_14.summary.work.clone(), er.summary.work.clone()), span: cost_seq(__acc_14.summary.span.clone(), er.summary.span.clone()), output_size: Rc::new(std::collections::HashMap::new()), certainty: er.summary.certainty.clone() }), table: er.table.clone() })
}
    }
    StringPart::Text { value: _, .. } => {
        __acc_14.clone()
    }
};
    }
    __acc_14
}
    }
    ExprData::ExprCast { expr: e, target: _, .. } => {
        cost_of_expr(e.clone(), func_index.clone(), table.clone())
    }
    ExprData::ExprIndex { base: b, index: i, .. } => {
        {
    let br = cost_of_expr(b.clone(), func_index.clone(), table.clone());
    let ir = cost_of_expr(i.clone(), func_index.clone(), br.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(br.summary.work.clone(), cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), ir.summary.work.clone())), span: cost_seq(br.summary.span.clone(), cost_seq(Rc::new(CostExpr::CostConst { value: 1_i64 }), ir.summary.span.clone())), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: ir.table.clone() })
}
    }
    ExprData::ExprSlice { base: b, start: s, end: e, .. } => {
        {
    let br = cost_of_expr(b.clone(), func_index.clone(), table.clone());
    let sr = cost_of_expr(s.clone(), func_index.clone(), br.table.clone());
    let er = cost_of_expr(e.clone(), func_index.clone(), sr.table.clone());
    Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: cost_seq(br.summary.work.clone(), cost_seq(sr.summary.work.clone(), er.summary.work.clone())), span: cost_seq(br.summary.span.clone(), cost_seq(sr.summary.span.clone(), er.summary.span.clone())), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: er.table.clone() })
}
    }
    ExprData::NoExprData => {
        Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: table.clone() })
    }
}
    })
}

pub fn estimate_expr_size(texpr: Rc<Node>, budget: i64) -> i64 {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        if budget.clone() <= 0_i64 {
    0_i64
} else {
    match texpr.expr_data.as_ref() {
    ExprData::ExprBlock { stmts: ss, .. } => {
        {
    let mut __acc_0 = budget.clone();
    for __elem_1 in ss.iter().cloned() {
        __acc_0 = estimate_expr_size(__elem_1.clone(), __acc_0.clone());
    }
    __acc_0
}
    }
    ExprData::ExprBinOp { op: _, left: l, right: r, .. } => {
        {
    let s1 = estimate_expr_size(l.clone(), budget.clone() - 1_i64);
    estimate_expr_size(r.clone(), s1)
}
    }
    ExprData::ExprIf { condition: c, then_branch: t, else_branch: e, .. } => {
        {
    let s1 = estimate_expr_size(c.clone(), budget.clone() - 1_i64);
    let s2 = estimate_expr_size(t.clone(), s1);
    match e.as_ref().map(|__rc| __rc.as_ref()) {
    Some(eb) => {
        let eb = Rc::new(eb.clone());
        estimate_expr_size(eb.clone(), s2)
    }
    None => {
        s2
    }
}
}
    }
    ExprData::ExprLet { name: _, value: v, body: b, .. } => {
        {
    let s1 = estimate_expr_size(v.clone(), budget.clone() - 1_i64);
    match b.as_ref().map(|__rc| __rc.as_ref()) {
    Some(bd) => {
        let bd = Rc::new(bd.clone());
        estimate_expr_size(bd.clone(), s1)
    }
    None => {
        s1
    }
}
}
    }
    ExprData::ExprMatch { scrutinee: s, arms, .. } => {
        {
    let s1 = estimate_expr_size(s.clone(), budget.clone() - 1_i64);
    {
    let mut __acc_2 = s1;
    for __elem_3 in arms.iter().cloned() {
        __acc_2 = estimate_expr_size(__elem_3.body.clone(), __acc_2.clone());
    }
    __acc_2
}
}
    }
    ExprData::ExprForEach { variable: _, collection: c, body: b, .. } => {
        {
    let s1 = estimate_expr_size(c.clone(), budget.clone() - 1_i64);
    estimate_expr_size(b.clone(), s1)
}
    }
    ExprData::ExprCall { func: _, args: a, .. } => {
        {
    let mut __acc_4 = budget.clone() - 1_i64;
    for __elem_5 in a.iter().cloned() {
        __acc_4 = estimate_expr_size(__elem_5.value.clone(), __acc_4.clone());
    }
    __acc_4
}
    }
    ExprData::ExprMethodCall { receiver: r, method: _, args: a, .. } => {
        {
    let s1 = estimate_expr_size(r.clone(), budget.clone() - 1_i64);
    {
    let mut __acc_6 = s1;
    for __elem_7 in a.iter().cloned() {
        __acc_6 = estimate_expr_size(__elem_7.value.clone(), __acc_6.clone());
    }
    __acc_6
}
}
    }
    ExprData::ExprLambda { params: _, body: b, .. } => {
        estimate_expr_size(b.clone(), budget.clone() - 1_i64)
    }
    ExprData::ExprReturn { value: v, .. } => {
        estimate_expr_size(v.clone(), budget.clone() - 1_i64)
    }
    _ => {
        budget.clone() - 1_i64
    }
}
}
    })
}

pub fn get_or_compute_summary(func_name: &str, func_index: Rc<HashMap<String, Rc<FuncEntry>>>, table: Rc<CostInternTable>) -> Rc<SummaryResult> {
    stacker::maybe_grow(512 * 1024, 2 * 1024 * 1024, || {
        match lookup_summary(table.clone(), &func_name) {
    Some(cached) => {
        Rc::new(SummaryResult { summary: cached.clone(), table: table.clone() })
    }
    None => {
        match func_index.clone().get(&func_name.to_string()).cloned() {
    Some(entry) => {
        {
    let body_budget = estimate_expr_size(entry.body.clone(), 500_i64);
    if body_budget <= 0_i64 {
    let too_large = Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostUnknown { reason: v2_rt::concat("body too large: ".to_string(), func_name.to_string()) }), span: Rc::new(CostExpr::CostUnknown { reason: v2_rt::concat("body too large: ".to_string(), func_name.to_string()) }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Unknown });
    let guarded_table = cache_summary(table.clone(), &func_name, too_large.clone());
    Rc::new(SummaryResult { summary: too_large.clone(), table: guarded_table.clone() })
} else {
    let placeholder = Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostUnknown { reason: v2_rt::concat("computing: ".to_string(), func_name.to_string()) }), span: Rc::new(CostExpr::CostUnknown { reason: v2_rt::concat("computing: ".to_string(), func_name.to_string()) }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Unknown });
    let table_with_placeholder = cache_summary(table.clone(), &func_name, placeholder.clone());
    let result = cost_of_expr(entry.body.clone(), func_index.clone(), table_with_placeholder.clone());
    let simplified = Rc::new(ComplexitySummary { work: maybe_simplify_cost(result.summary.work.clone()), span: maybe_simplify_cost(result.summary.span.clone()), output_size: result.summary.output_size.clone(), certainty: result.summary.certainty.clone() });
    let final_table = cache_summary(result.table.clone(), &func_name, simplified.clone());
    Rc::new(SummaryResult { summary: simplified.clone(), table: final_table.clone() })
}
}
    }
    None => {
        {
    let unknown_summary = Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostUnknown { reason: v2_rt::concat("function not found: ".to_string(), func_name.to_string()) }), span: Rc::new(CostExpr::CostUnknown { reason: v2_rt::concat("function not found: ".to_string(), func_name.to_string()) }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Unknown });
    Rc::new(SummaryResult { summary: unknown_summary.clone(), table: table.clone() })
}
    }
}
    }
}
    })
}

pub fn is_unknown_cost(expr: Rc<CostExpr>) -> bool {
    match expr.as_ref() {
    CostExpr::CostUnknown { reason: _, .. } => {
        true
    }
    _ => {
        false
    }
}
}

pub fn build_complexity_report(func_entries: Rc<Vec<Rc<FuncEntry>>>) -> Rc<ComplexityReport> {
    let func_index = {
    let mut __acc_0: Rc<std::collections::HashMap<String, Rc<FuncEntry>>> = Rc::new(std::collections::HashMap::new());
    for __elem_1 in func_entries.iter().cloned() {
        __acc_0 = {
    let __rc_3 = __acc_0;
    let mut __map_ins_2 = Rc::try_unwrap(__rc_3).unwrap_or_else(|rc| (*rc).clone());
    __map_ins_2.insert(__elem_1.name.clone(), __elem_1.clone());
    Rc::new(__map_ins_2)
};
    }
    __acc_0
};
    let result = {
    let mut __acc_4 = Rc::new(SummaryResult { summary: Rc::new(ComplexitySummary { work: Rc::new(CostExpr::CostConst { value: 0_i64 }), span: Rc::new(CostExpr::CostConst { value: 0_i64 }), output_size: Rc::new(std::collections::HashMap::new()), certainty: Certainty::Proven }), table: empty_intern_table() });
    for __elem_5 in func_entries.iter().cloned() {
        __acc_4 = {
    let sr = get_or_compute_summary(&__elem_5.name, func_index.clone(), __acc_4.table.clone());
    Rc::new(SummaryResult { summary: sr.summary.clone(), table: sr.table.clone() })
};
    }
    __acc_4
};
    let summaries_map = result.table.summaries.clone();
    let violations = {
    let mut __mapped_8 = Vec::new();
    for __elem_9 in ({
    let mut __filtered_6 = Vec::new();
    for __elem_7 in func_entries.iter().cloned() {
        if match summaries_map.clone().get(&__elem_7.name.clone()).cloned() {
    Some(summary) => {
        is_unknown_cost(summary.work.clone())
    }
    None => {
        true
    }
} {
    __filtered_6.push(__elem_7);
};
    }
    Rc::new(__filtered_6)
}).iter().cloned() {
        __mapped_8.push(match summaries_map.clone().get(&__elem_9.name.clone()).cloned() {
    Some(summary) => {
        {
    let reason = match summary.work.as_ref() {
    CostExpr::CostUnknown { reason: r, .. } => {
        r.clone()
    }
    _ => {
        "unknown".to_string()
    }
};
    Rc::new(ComplexityViolation { func_name: __elem_9.name.clone(), reason: reason.clone(), summary: Some(summary.clone()) })
}
    }
    None => {
        Rc::new(ComplexityViolation { func_name: __elem_9.name.clone(), reason: "no summary computed".to_string(), summary: None })
    }
});
    }
    Rc::new(__mapped_8)
};
    let formatted = if ({
    let __len_12 = func_entries.clone().len();
    __len_12 as i64
}) > 200_i64 {
    v2_rt::concat(v2_rt::concat("complexity report omitted for ".to_string(), v2_rt::to_string({
    let __len_11 = func_entries.clone().len();
    __len_11 as i64
})), " functions".to_string())
} else {
    format_complexity_report(func_entries.clone(), summaries_map.clone())
};
    Rc::new(ComplexityReport { function_summaries: summaries_map.clone(), violations: violations.clone(), intern_table: result.table.clone(), formatted: formatted.clone() })
}

