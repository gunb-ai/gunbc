// Seed realization for v2.compiler.program_partition (Wave 2 parallel flip).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.program_partition
// is emitted-only and the behavioral harness is modeled (spp_scaffold_dissolution_trigger).

/// Mirrors `data partition_structural_value_carriers_dissolution_trigger` in src/v2/compiler/program_partition.dag.
pub fn partition_structural_value_carriers_dissolution_trigger() -> String {
    "🟡 dissolve-on: the whole-program emit applies use-site ownership to the module-surface wrapper node (carrier ^dag_surface_module), which is always by-value (a module is never reference-wrapped) — this returns the fixed set of such structural surface carriers. Currently one entry (module surface); expand as further synthetic surface carriers surface. DISSOLVES WHEN the emit stops running use-site ownership over non-value structural surfaces (06_translate scopes the ownership fold to genuine type carriers), at which point no structural-surface value-carrier enrollment is needed.".to_string()
}
