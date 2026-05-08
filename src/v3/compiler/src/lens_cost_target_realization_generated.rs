// AUTO-GENERATED from `src/v3/lenses/cost_target_realization.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub fn type_realization_meta(p0: &Dag) -> Option<Declaration> {
    (p0).declaration_by_name(&&(String::from("TypeRealization")))
        .cloned()
}
pub fn callable_realization_meta(p0: &Dag) -> Option<Declaration> {
    (p0).declaration_by_name(&&(String::from("CallableRealization")))
        .cloned()
}
pub fn operator_realization_meta(p0: &Dag) -> Option<Declaration> {
    (p0).declaration_by_name(&&(String::from("OperatorRealization")))
        .cloned()
}
pub fn behavior_realization_meta(p0: &Dag) -> Option<Declaration> {
    (p0).declaration_by_name(&&(String::from("BehaviorRealization")))
        .cloned()
}
pub fn type_instantiation_realization_meta(p0: &Dag) -> Option<Declaration> {
    (p0).declaration_by_name(&&(String::from("TypeInstantiationRealization")))
        .cloned()
}
pub fn pattern_realization_meta(p0: &Dag) -> Option<Declaration> {
    (p0).declaration_by_name(&&(String::from("PatternRealization")))
        .cloned()
}
