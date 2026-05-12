// AUTO-GENERATED from `src/v3/lenses/parallelism.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub fn analyze_parallelism(_p0: &Dag, _p1: NodeId) -> WorkflowParallelismReport {
    WorkflowParallelismReport::ParallelismUnsupported(ParallelismUnsupportedDetail {
        kind: ParallelismUnsupportedKind::LensSurfacePending,
        downstream_stage: "lane2_stage2e_parallelism_lens".to_string(),
        reason: "temporary bootstrap stub; run regen_lens --lens parallelism".to_string(),
    })
}

pub fn loop_iteration_parallel_emission_indicator(_p0: &Dag, _p1: NodeId) -> i64 {
    0
}
