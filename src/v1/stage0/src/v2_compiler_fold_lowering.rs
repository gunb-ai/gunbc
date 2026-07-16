// Seed realization for v2.compiler.fold_lowering (Wave 2 Band B).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.fold_lowering
// is emitted-only and the behavioral harness is modeled (sfl_scaffold_dissolution_trigger).

pub type Symbol = String;

pub fn fold_family_head(sym: Symbol) -> bool {
    sym == "fold"
        || sym == "fold_list"
        || sym == "fold_list_right"
        || sym == "fold_node"
}
