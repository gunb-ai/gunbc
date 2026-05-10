//! R3 T-Free-Consequences: optional `lane2_workflow` on the workflow root from an **authored
//! comment** in the claim `source` string.
//!
//! **Modeling tension (explicit):** `docs/design-db18-workflow-effect-carrier.md` targets workflow /
//! loop shape as **lowering-only** substrate facts. This module is a **harness side channel**: it
//! fabricates `WorkflowEffect::LoopEffect` from a magic comment after parse/lower, so downstream
//! lenses can read the same carrier lowering will eventually own. That is intentional **Pattern A
//! author-now debt**, not the end-state authority model.
//!
//! **Program budget (P5 receipt):** `ROADMAP.md` §"Reflective integration patterns" — bullet
//! **"R3 second-batch auto-loop scaffold (T-Free-Consequences gates #46–#48)"** authorizes this path
//! and names the single dissolution trigger. SG-0 net-add PRs must pair with Director-budget class
//! **(b)** citing that ROADMAP URL, not research-only briefs as sole authority. Shape discussion (not
//! authorization) remains in `docs/briefs/r3-v-auto-loop-parallelism-cross-target-witness-shapes.md`.
//!
//! **Paired scaffold:** [`crate::lens_apply::apply_lens_declaration`] special-cases lens
//! `auto_loop_parallelism_pending_lens` when `program_under_test` is `Some`; dissolve **together**
//! with this scanner when lowering installs `lane2_workflow` from surface syntax.
//!
//! **Scope — global `compile_to_dag`, not `cfg(test)`:** [`crate::compile_to_dag`] and
//! `compile_onto_parse_surface_free_bootstrap` always call into this module after `infer`. Any
//! `.v3` source that includes the magic directive therefore stages `lane2_workflow` on the workflow
//! **Bind shell** ([`Dag::workflow_lane2_subject`]) in that compile. This is deliberate: claim programs and ordinary programs share one
//! pipeline, and activation is **opt-in** via the `gunbc::r3_free_consequences::…` namespaced prefix
//! (not a test-only hook). Reading that staged fact through `auto_loop_parallelism_pending_lens`
//! still requires the test-runner / lens path to pass `program_under_test: Some(_)`.
//!
//! **Dissolution:** delete this scan and the magic-comment contract when lowering authors loop
//! `lane2_workflow` facts; remove the lens name-key branch in the same change set.

use crate::dag::{Dag, EffectShape, IdempotentShape, KeySource, OperationEffect, WorkflowEffect};
use crate::diagnostics::{Diagnostic, SourceSpan};

const DIRECTIVE_PREFIX: &str = "// gunbc::r3_free_consequences::lane2_loop_witness:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WitnessKind {
    ReadOnlyLoop,
    UpsertLoop,
    /// Present in source: do not register `lane2_workflow` (unproven / sequential default).
    Unproven,
}

enum WitnessScan {
    Absent,
    Ok {
        kind: WitnessKind,
        directive_span: SourceSpan,
    },
    Malformed {
        span: SourceSpan,
        message: String,
    },
}

fn scan_witness(source: &str, file: &str) -> WitnessScan {
    let mut i = 0usize;
    while i < source.len() {
        let line_end = source[i..]
            .find('\n')
            .map(|p| i + p)
            .unwrap_or(source.len());
        let line_no_nl = &source[i..line_end];
        let ws_leading = line_no_nl.len() - line_no_nl.trim_start().len();
        let trimmed = line_no_nl.trim();
        let trimmed_start = i + ws_leading;
        let trimmed_end = trimmed_start + trimmed.len();

        if let Some(rest) = trimmed.strip_prefix(DIRECTIVE_PREFIX) {
            let token = rest.trim();
            if token.is_empty() {
                return WitnessScan::Malformed {
                    span: SourceSpan::new(file, trimmed_start as u32, trimmed_end as u32),
                    message: "`lane2_loop_witness` directive requires a token: read_only, upsert_dependent, or unproven".to_string(),
                };
            }
            let kind = match token {
                "read_only" => WitnessKind::ReadOnlyLoop,
                "upsert_dependent" => WitnessKind::UpsertLoop,
                "unproven" => WitnessKind::Unproven,
                _ => {
                    return WitnessScan::Malformed {
                        span: SourceSpan::new(file, trimmed_start as u32, trimmed_end as u32),
                        message: format!(
                            "unknown `lane2_loop_witness` directive token `{token}`; expected read_only | upsert_dependent | unproven"
                        ),
                    };
                }
            };
            return WitnessScan::Ok {
                kind,
                directive_span: SourceSpan::new(file, trimmed_start as u32, trimmed_end as u32),
            };
        }

        i = if line_end < source.len() {
            line_end + 1
        } else {
            line_end
        };
    }
    WitnessScan::Absent
}

/// If `source` carries a `lane2_loop_witness` directive, register the matching
/// [`WorkflowEffect::LoopEffect`] on [`Dag::workflow_lane2_subject`] (last `Bind` shell).
///
/// Fail-closed: a `read_only` / `upsert_dependent` directive **must** stage `lane2_workflow` or emit
/// a diagnostic (no silent collapse into the same sequential scalar as a missing carrier).
pub fn apply_authored_lane2_loop_witness(dag: &mut Dag, source: &str, file: &str) {
    match scan_witness(source, file) {
        WitnessScan::Absent => {}
        WitnessScan::Malformed { span, message } => {
            dag.attach_diagnostic(Diagnostic::ParseError {
                message,
                span,
                fixes: vec![],
            });
        }
        WitnessScan::Ok {
            kind,
            directive_span,
        } => {
            if matches!(kind, WitnessKind::Unproven) {
                return;
            }
            let wf = match kind {
                WitnessKind::ReadOnlyLoop => WorkflowEffect::LoopEffect {
                    body: Box::new(WorkflowEffect::LinearEffect {
                        ops: vec![OperationEffect {
                            operation_name: "r3_fc_read".to_string(),
                            shape: EffectShape::IsIdempotent(IdempotentShape::ReadEffect),
                        }],
                    }),
                },
                WitnessKind::UpsertLoop => WorkflowEffect::LoopEffect {
                    body: Box::new(WorkflowEffect::LinearEffect {
                        ops: vec![OperationEffect {
                            operation_name: "r3_fc_upsert".to_string(),
                            shape: EffectShape::IsIdempotent(IdempotentShape::UpsertEffect {
                                key_source: KeySource::PathParam {
                                    param: "id".to_string(),
                                },
                            }),
                        }],
                    }),
                },
                WitnessKind::Unproven => unreachable!("filtered above"),
            };
            let Some(subject) = dag.workflow_lane2_subject() else {
                dag.attach_diagnostic(Diagnostic::ParseError {
                    message: "`lane2_loop_witness` directive requires a workflow shell `Bind` to attach `lane2_workflow`; this program has no `Bind`".to_string(),
                    span: directive_span,
                    fixes: vec![],
                });
                return;
            };
            if !dag.try_register_lane2_workflow_effect(subject, wf) {
                dag.attach_diagnostic(Diagnostic::ParseError {
                    message: "`lane2_loop_witness`: cannot attach `lane2_workflow` (substrate supports Value/Bind nodes only)".to_string(),
                    span: directive_span,
                    fixes: vec![],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compile_to_dag;
    use crate::dag::{EffectShape, IdempotentShape, WorkflowEffect};
    use crate::diagnostics::Diagnostic;
    use crate::CompileError;

    #[test]
    fn directive_on_a_later_line_is_found_after_non_matching_comments() {
        let src = "// unrelated preamble comment\n// gunbc::r3_free_consequences::lane2_loop_witness: read_only\nlet _: Int = 0\n";
        let dag = compile_to_dag(src, "witness_after_preamble.v3").expect("compile");
        let subject = dag.workflow_lane2_subject().expect("lane2 subject");
        assert_eq!(
            crate::workflow_parallelism::loop_iteration_parallel_emission_indicator(&dag, subject),
            1
        );
    }

    #[test]
    fn read_only_directive_registers_loop_workflow() {
        let src = "// gunbc::r3_free_consequences::lane2_loop_witness: read_only\nlet _: Int = 0\n";
        let dag = compile_to_dag(src, "witness_read.v3").expect("compile");
        let subject = dag.workflow_lane2_subject().expect("lane2 subject");
        let wf = dag
            .lane2_workflow_effect_at(&subject)
            .expect("lane2_workflow staged");
        assert!(matches!(
            wf,
            WorkflowEffect::LoopEffect { body }
                if matches!(
                    body.as_ref(),
                    WorkflowEffect::LinearEffect { ops }
                        if ops.len() == 1
                            && matches!(
                                ops[0].shape,
                                EffectShape::IsIdempotent(IdempotentShape::ReadEffect)
                            )
                )
        ));
        assert_eq!(
            crate::workflow_parallelism::loop_iteration_parallel_emission_indicator(&dag, subject),
            1
        );
    }

    #[test]
    fn upsert_directive_registers_sequential_body() {
        let src =
            "// gunbc::r3_free_consequences::lane2_loop_witness: upsert_dependent\nlet _: Int = 0\n";
        let dag = compile_to_dag(src, "witness_upsert.v3").expect("compile");
        let subject = dag.workflow_lane2_subject().expect("lane2 subject");
        let wf = dag
            .lane2_workflow_effect_at(&subject)
            .expect("lane2_workflow staged");
        assert!(matches!(
            wf,
            WorkflowEffect::LoopEffect { body }
                if matches!(
                    body.as_ref(),
                    WorkflowEffect::LinearEffect { ops }
                        if ops.len() == 1
                            && matches!(
                                ops[0].shape,
                                EffectShape::IsIdempotent(IdempotentShape::UpsertEffect { .. })
                            )
                )
        ));
        assert_eq!(
            crate::workflow_parallelism::loop_iteration_parallel_emission_indicator(&dag, subject),
            0
        );
    }

    #[test]
    fn unproven_directive_leaves_lane2_absent() {
        let src = "// gunbc::r3_free_consequences::lane2_loop_witness: unproven\nlet _: Int = 0\n";
        let dag = compile_to_dag(src, "witness_none.v3").expect("compile");
        let subject = dag.workflow_lane2_subject().expect("lane2 subject");
        assert!(dag.lane2_workflow_effect_at(&subject).is_none());
        assert_eq!(
            crate::workflow_parallelism::loop_iteration_parallel_emission_indicator(&dag, subject),
            0
        );
    }

    #[test]
    fn unknown_directive_token_is_parse_error() {
        let file = "witness_typo.v3";
        let src = "// gunbc::r3_free_consequences::lane2_loop_witness: typo\nlet _: Int = 0\n";
        let err = compile_to_dag(src, file).expect_err("diagnostic");
        let CompileError::Semantic(dag) = err else {
            panic!("expected semantic failure");
        };
        // Contract: malformed witness token surfaces as `Diagnostic::ParseError` on the claim file.
        // Avoid substring-matching diagnostic prose (TESTING.md / INVARIANTS P3); span + variant only.
        let mut parse_error_spans = 0u32;
        for (_, d) in dag.diagnostics().iter() {
            if let Diagnostic::ParseError { span, .. } = d {
                assert_eq!(span.file, file);
                parse_error_spans += 1;
            }
        }
        assert_eq!(
            parse_error_spans,
            1,
            "expected exactly one ParseError for malformed witness token, diagnostics={:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        );
    }
}
