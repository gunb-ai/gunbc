// Seed realization for v2.compiler.compile (Wave 2 Gate-A flip).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.compile
// is emitted-only and the behavioral harness is modeled (sc_scaffold_dissolution_trigger).

/// Mirrors `data required_lens_grain_note` in src/v2/compiler/00_compile.dag.
/// The Gate-A behavioral witness (dag/gunbc/instruments/self_host_00_compile_shims)
/// asserts EQUALITY between the emitted artifact and this oracle, so this string is not
/// documentation of the authority — it is the compared value, and it cuts over with it.
pub fn required_lens_grain_note() -> String {
    "Two registered grains, one roster, one door (validate_then_compile is the single gate authority; per sharp-bee-290 contract 2026-07-13 no second gate surface exists). A CompileLensSubtree registration runs PER SUBTREE NODE (node-local predicates: fact_density, unit_modeling, lifecycle_carrier -- each gate reads only the node's direct children; a nested subtree walk inside the gate would stack with run_required_lens_gates_on_subtree's fold_node and be O(n^2)). A CompileLensRoot registration runs ONCE at the tree root -- the grain for rooted whole-tree walks (accumulator-copy carrier threading), where per-node invocation would be O(n^2): the enforcement of the complexity lens must not itself violate the complexity lens (DESIGN 7 self-application). The grain is a FIELD on the registration and required_lenses_at_grain derives each execution's list from the one roster, so this cost argument is carried by the row rather than by which of two lists a lens was written into. Witness accumulation on the subtree fold uses prepend-then-reverse, not append-in-step. Required registrations at both grains fire on every validate_then_compile call regardless of caller-supplied lenses, and a caller-supplied lens now runs at ITS OWN declared grain rather than at root whatever it declared.".to_string()
}
