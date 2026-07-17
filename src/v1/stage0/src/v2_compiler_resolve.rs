// Seed realization for v2.compiler.resolve (Wave 2 parallel flip).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.resolve
// is emitted-only and the behavioral harness is modeled (sr_scaffold_dissolution_trigger).

/// Mirrors `data arrow_domain_param_scoping_note` in src/v2/compiler/03_resolve.dag.
pub fn arrow_domain_param_scoping_note() -> String {
    "Wave 1 Gate 1 A1: param scoping is add_arrow_domain_named_params over lowered Arrow domain Named edges (general-body-producer-design Stage A). dag_fn_decl_param_binding_atoms and scope_with_fn_decl_params dissolved. fn_decl ident now routes through StampLexeme and SymbolIndex fill carries module_qn.fn.param (parity proven by execution); domain construction remains interim param-list-walk (body_lower_domain_from_param_list) until domain is built from SymbolIndex lookup rather than walked from the param list.".to_string()
}
