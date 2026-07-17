// Seed realization for v2.compiler.compile (Wave 2 Gate-A flip).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.compile
// is emitted-only and the behavioral harness is modeled (sc_scaffold_dissolution_trigger).

/// Mirrors `data required_lens_grain_note` in src/v2/compiler/00_compile.dag.
pub fn required_lens_grain_note() -> String {
    "Two required-roster grains, one door (validate_then_compile is the single gate authority; per sharp-bee-290 contract 2026-07-13 no second gate surface exists). always_required_lenses runs PER SUBTREE NODE (node-local predicates: fact_density, unit_modeling — each gate reads only the node's direct children; a nested subtree walk inside the gate would stack with run_required_lens_gates_on_subtree's fold_node and be O(n^2)). always_required_root_lenses runs ONCE at the tree root — the grain for rooted whole-tree walks (accumulator-copy carrier threading), where per-node invocation would be O(n^2): the enforcement of the complexity lens must not itself violate the complexity lens (DESIGN 7 self-application). Witness accumulation on the subtree fold uses prepend-then-reverse, not append-in-step. Root-required lenses fire on every validate_then_compile call regardless of caller-supplied lenses, exactly like the subtree roster.".to_string()
}
