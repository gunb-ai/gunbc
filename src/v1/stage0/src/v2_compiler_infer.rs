// Seed realization for v2.compiler.infer (Gate-A flip prep, curated seed-link scaffold).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.infer
// is emitted-only and the behavioral harness is modeled (i4_scaffold_dissolution_trigger).

/// Mirrors `data infer_arrow_domain_binding_heuristic_note` in src/v2/compiler/04_infer.dag.
pub fn infer_arrow_domain_binding_heuristic_note() -> String {
    "interim: infer_find_arrow_domain_type_in_tree DFS-walks the resolved tree for the first Arrow whose Conj domain names the atom-binding symbol — symbol-name collision across unrelated arrows picks first-visit order, not lexical scope. dissolve-on: namespace-only name resolution consumes containment-tree binding→domain (DESIGN open thread §namespace-only); delete this heuristic walker when SymbolIndex lookup lands.".to_string()
}
