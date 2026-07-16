// Seed realization for v2.compiler.parse_engine_hooks (Wave 2 Band A).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.parse_engine_hooks
// is emitted-only and the behavioral harness is modeled (peh_scaffold_dissolution_trigger).

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(pub String);

pub fn parse_engine_match_arm_body_production() -> Symbol {
    Symbol("dag_production_match_arm_body".to_string())
}

pub fn parse_engine_match_arm_stmt_body_production() -> Symbol {
    Symbol("dag_production_match_arm_stmt_body".to_string())
}

pub fn parse_engine_expr_production() -> Symbol {
    Symbol("dag_production_expr".to_string())
}
