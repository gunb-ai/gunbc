// Seed realization for v2.compiler.source_authority (Wave 2 Gate-A flip lane).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.source_authority
// is emitted-only and the behavioral harness is modeled (ssa_scaffold_dissolution_trigger).

/// Mirrors `data source_authority_module_note` in src/v2/compiler/source_authority.dag.
pub fn source_authority_module_note() -> String {
    "Wave 2 Gate-A flip lane: v2.compiler.source_authority is the cross-tree ingest and round-trip authority (parse→normalize→resolve→serialize laws over DagSourceReadWitness / SourceRootIngest). Curated seed-linked scaffold defers full tree behavioral oracle until emitter Rc/Optional coherence (#6775) greens rustc on the closure.".to_string()
}
