// Hand-maintained mirror of src/v2/compile.dag pipeline fold (v2_compile_pipeline).
// Survives stage0 regeneration — compile.dag is source of truth; this module tracks it.
// See src/v2/tests/src/source_audit.rs::compile_stage0_mirror_uses_pipeline_fold_authority.

use crate::v2_compiler_artifact::{default_artifact_plan, empty_artifact_plan, ArtifactPlan, RenderTarget};
use crate::v2_compiler_complexity::{build_complexity_report, empty_complexity_report, ComplexityReport};
use crate::v2_compiler_infer::reconcile;
use crate::v2_compiler_infer_items::ResolvedGraph;
use crate::v2_compiler_normalize::normalize_graph;
use crate::v2_compiler_ownership::OwnershipProof;
use crate::v2_compiler_resolve::ModuleGraph;
use crate::v2_rt;
use crate::v2_std_core::{
    authored_name_at, empty_intern_table, is_error_diagnostic, InternTable, NewlineIndex, TextFile,
};
use crate::v2_std_core::{CompileResult, ErrorNode};
use std::collections::HashMap;
use std::rc::Rc;

use super::v2_compiler_compile::{
    complexity_diagnostics, emit_from_artifact_plan, extract_func_entries, extract_ownership_proofs,
    front_end_sources, ownership_diagnostics, EmitResult, FrontendResult, PipelineResult, SourceFile,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStageKind {
    Frontend,
    Normalize,
    Infer,
    Complexity,
    Ownership,
    ArtifactPlan,
    Emit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineGatePolicy {
    BlockingOnErrors,
    SurfaceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineStageSpec {
    pub stage: PipelineStageKind,
    pub gate: PipelineGatePolicy,
}

const V2_COMPILE_PIPELINE: [PipelineStageSpec; 7] = [
    PipelineStageSpec {
        stage: PipelineStageKind::Frontend,
        gate: PipelineGatePolicy::BlockingOnErrors,
    },
    PipelineStageSpec {
        stage: PipelineStageKind::Normalize,
        gate: PipelineGatePolicy::BlockingOnErrors,
    },
    PipelineStageSpec {
        stage: PipelineStageKind::Infer,
        gate: PipelineGatePolicy::BlockingOnErrors,
    },
    PipelineStageSpec {
        stage: PipelineStageKind::Complexity,
        gate: PipelineGatePolicy::SurfaceOnly,
    },
    PipelineStageSpec {
        stage: PipelineStageKind::Ownership,
        gate: PipelineGatePolicy::BlockingOnErrors,
    },
    PipelineStageSpec {
        stage: PipelineStageKind::ArtifactPlan,
        gate: PipelineGatePolicy::BlockingOnErrors,
    },
    PipelineStageSpec {
        stage: PipelineStageKind::Emit,
        gate: PipelineGatePolicy::BlockingOnErrors,
    },
];

fn pipeline_gate_blocks(
    policy: PipelineGatePolicy,
    diagnostics: Rc<Vec<Rc<ErrorNode>>>,
) -> bool {
    match policy {
        PipelineGatePolicy::BlockingOnErrors => diagnostics
            .iter()
            .any(|d| is_error_diagnostic(d.diagnostic.clone())),
        PipelineGatePolicy::SurfaceOnly => false,
    }
}

#[derive(Debug, Clone)]
pub struct CompilePipelineState {
    pub sources: Rc<Vec<Rc<SourceFile>>>,
    pub halted: bool,
    pub emit_blocked: bool,
    pub run_emit: bool,
    pub frontend: Rc<FrontendResult>,
    pub newline_indices: Rc<Vec<Rc<NewlineIndex>>>,
    pub graph: Option<Rc<ModuleGraph>>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    pub norm_diags: Rc<Vec<Rc<ErrorNode>>>,
    pub typed: Option<Rc<ResolvedGraph>>,
    pub complexity: Rc<ComplexityReport>,
    pub ownership: Rc<Vec<Rc<OwnershipProof>>>,
    pub artifact_plan: Rc<ArtifactPlan>,
    pub stage_diags: Rc<Vec<Rc<ErrorNode>>>,
    pub emit_diags: Rc<Vec<Rc<ErrorNode>>>,
    pub emit_files: Rc<Vec<Rc<TextFile>>>,
    pub target: RenderTarget,
}

fn pipeline_state_mark_halted(state: CompilePipelineState) -> CompilePipelineState {
    CompilePipelineState {
        halted: true,
        ..state
    }
}

fn apply_pipeline_stage(
    state: CompilePipelineState,
    spec: PipelineStageSpec,
) -> CompilePipelineState {
    if state.halted {
        return state;
    }
    match spec.stage {
        PipelineStageKind::Frontend => {
            let frontend = front_end_sources(state.sources.clone());
            match frontend.graph.clone() {
                None => CompilePipelineState {
                    halted: true,
                    frontend: frontend.clone(),
                    newline_indices: frontend.newline_indices.clone(),
                    graph: None,
                    source_indices: v2_rt::rc_empty_map(),
                    ..state
                },
                Some(graph) => {
                    if pipeline_gate_blocks(spec.gate, graph.diagnostics.clone()) {
                        CompilePipelineState {
                            halted: true,
                            frontend: frontend.clone(),
                            newline_indices: frontend.newline_indices.clone(),
                            graph: Some(graph),
                            source_indices: v2_rt::rc_empty_map(),
                            ..state
                        }
                    } else {
                        let source_indices = frontend.newline_indices.clone().iter().cloned().fold(
                            v2_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
                            |acc, index| v2_rt::rc_map_insert(acc, index.file.clone(), index.clone()),
                        );
                        CompilePipelineState {
                            halted: false,
                            frontend: frontend.clone(),
                            newline_indices: frontend.newline_indices.clone(),
                            graph: Some(graph),
                            source_indices,
                            ..state
                        }
                    }
                }
            }
        }
        PipelineStageKind::Normalize => match state.graph.clone() {
            None => pipeline_state_mark_halted(state),
            Some(graph) => {
                let norm = normalize_graph(&graph, state.source_indices.clone());
                let norm_diags = norm.diagnostics.clone();
                if pipeline_gate_blocks(spec.gate, norm_diags.clone()) {
                    CompilePipelineState {
                        halted: true,
                        norm_diags: norm_diags.clone(),
                        ..state
                    }
                } else {
                    CompilePipelineState {
                        graph: Some(norm.graph.clone()),
                        norm_diags,
                        ..state
                    }
                }
            }
        },
        PipelineStageKind::Infer => match state.graph.clone() {
            None => pipeline_state_mark_halted(state),
            Some(graph) => {
                let typed = reconcile(
                    graph,
                    state.source_indices.clone(),
                    state.frontend.intern_table.clone(),
                );
                let typed_diags = typed.diagnostics.clone();
                let infer_blocks_emit = pipeline_gate_blocks(spec.gate, typed_diags.clone());
                CompilePipelineState {
                    emit_blocked: state.emit_blocked || infer_blocks_emit,
                    typed: Some(typed),
                    stage_diags: v2_rt::concat(state.stage_diags.clone(), typed_diags),
                    ..state
                }
            }
        },
        PipelineStageKind::Complexity => match state.typed.clone() {
            None => pipeline_state_mark_halted(state),
            Some(typed) => {
                let func_entries = extract_func_entries(typed.clone());
                let recursion_ctx = super::v2_compiler_compile::build_recursion_context(typed.clone());
                let complexity = build_complexity_report(
                    &func_entries,
                    recursion_ctx,
                    state.source_indices.clone(),
                );
                let complexity_diags = complexity_diagnostics(complexity.clone());
                let complexity_blocks_emit =
                    pipeline_gate_blocks(spec.gate, complexity_diags.clone());
                CompilePipelineState {
                    emit_blocked: state.emit_blocked || complexity_blocks_emit,
                    complexity,
                    stage_diags: v2_rt::concat(state.stage_diags.clone(), complexity_diags),
                    ..state
                }
            }
        },
        PipelineStageKind::Ownership => {
            if state.emit_blocked {
                state
            } else {
                match state.typed.clone() {
                    None => pipeline_state_mark_halted(state),
                    Some(typed) => {
                        let ownership = extract_ownership_proofs(typed);
                        let ownership_diags = ownership_diagnostics(ownership.clone());
                        let ownership_blocks_emit =
                            pipeline_gate_blocks(spec.gate, ownership_diags.clone());
                        CompilePipelineState {
                            emit_blocked: state.emit_blocked || ownership_blocks_emit,
                            ownership,
                            stage_diags: v2_rt::concat(state.stage_diags.clone(), ownership_diags),
                            ..state
                        }
                    }
                }
            }
        }
        PipelineStageKind::ArtifactPlan => {
            if state.emit_blocked {
                state
            } else {
                match state.typed.clone() {
                    None => pipeline_state_mark_halted(state),
                    Some(typed) => {
                        let root_modules = Rc::new({
                            let mut names = Vec::new();
                            for m in typed.modules.iter() {
                                names.push(authored_name_at(
                                    state.source_indices.clone(),
                                    &m.module,
                                ));
                            }
                            names
                        });
                        let artifact_plan =
                            default_artifact_plan(root_modules, state.target);
                        CompilePipelineState {
                            artifact_plan,
                            ..state
                        }
                    }
                }
            }
        }
        PipelineStageKind::Emit => {
            if !state.run_emit || state.emit_blocked {
                state
            } else {
                match state.typed.clone() {
                    None => pipeline_state_mark_halted(state),
                    Some(typed) => {
                        let emit_result =
                            emit_from_artifact_plan(typed, &state.artifact_plan);
                        let emit_blocks =
                            pipeline_gate_blocks(spec.gate, emit_result.diagnostics.clone());
                        CompilePipelineState {
                            emit_blocked: state.emit_blocked || emit_blocks,
                            emit_diags: emit_result.diagnostics.clone(),
                            emit_files: if emit_blocks {
                                Rc::new(vec![])
                            } else {
                                emit_result.files.clone()
                            },
                            ..state
                        }
                    }
                }
            }
        }
    }
}

pub fn run_compile_pipeline_fold(
    sources: Rc<Vec<Rc<SourceFile>>>,
    target: RenderTarget,
    run_emit: bool,
) -> CompilePipelineState {
    let empty_frontend = Rc::new(FrontendResult {
        graph: None,
        diagnostics: Rc::new(vec![]),
        newline_indices: Rc::new(vec![]),
        intern_table: empty_intern_table(),
    });
    let mut state = CompilePipelineState {
        sources: sources.clone(),
        halted: false,
        emit_blocked: false,
        run_emit,
        frontend: empty_frontend,
        newline_indices: Rc::new(vec![]),
        graph: None,
        source_indices: v2_rt::rc_empty_map(),
        norm_diags: Rc::new(vec![]),
        typed: None,
        complexity: empty_complexity_report(),
        ownership: Rc::new(vec![]),
        artifact_plan: empty_artifact_plan(),
        stage_diags: Rc::new(vec![]),
        emit_diags: Rc::new(vec![]),
        emit_files: Rc::new(vec![]),
        target,
    };
    for spec in V2_COMPILE_PIPELINE {
        state = apply_pipeline_stage(state, spec);
    }
    state
}

pub fn compile_pipeline_state_to_result(state: CompilePipelineState) -> Rc<PipelineResult> {
    Rc::new(PipelineResult {
        files: if state.emit_blocked || state.halted {
            Rc::new(vec![])
        } else {
            state.emit_files.clone()
        },
        diagnostics: v2_rt::concat(
            v2_rt::concat(
                state.frontend.diagnostics.clone(),
                state.norm_diags.clone(),
            ),
            v2_rt::concat(state.stage_diags.clone(), state.emit_diags.clone()),
        ),
        complexity: state.complexity.clone(),
        ownership: state.ownership.clone(),
        artifact_plan: state.artifact_plan.clone(),
        newline_indices: state.newline_indices.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResolvedPipelineResult {
    pub graph: Option<Rc<ResolvedGraph>>,
    pub diagnostics: Rc<Vec<Rc<ErrorNode>>>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    pub complexity: Rc<ComplexityReport>,
    pub ownership: Rc<Vec<Rc<OwnershipProof>>>,
    pub newline_indices: Rc<Vec<Rc<NewlineIndex>>>,
}

pub fn compile_sources_via_fold(
    sources: Rc<Vec<Rc<SourceFile>>>,
    target: RenderTarget,
) -> Rc<PipelineResult> {
    let state = run_compile_pipeline_fold(sources, target, true);
    compile_pipeline_state_to_result(state)
}

pub fn compile_to_resolved_via_fold(
    sources: Rc<Vec<Rc<SourceFile>>>,
) -> Rc<ResolvedPipelineResult> {
    let state = run_compile_pipeline_fold(sources, RenderTarget::Rust, false);
    Rc::new(ResolvedPipelineResult {
        graph: if state.emit_blocked || state.halted {
            None
        } else {
            state.typed.clone()
        },
        diagnostics: v2_rt::concat(
            v2_rt::concat(
                state.frontend.diagnostics.clone(),
                state.norm_diags.clone(),
            ),
            state.stage_diags.clone(),
        ),
        source_indices: state.source_indices.clone(),
        complexity: state.complexity.clone(),
        ownership: state.ownership.clone(),
        newline_indices: state.newline_indices.clone(),
    })
}
