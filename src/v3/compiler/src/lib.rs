// v3 compiler — M0 substrate skeleton.
//
// Pipeline (target end state for M0):
//   source text -> tokenize -> parse -> lower to L1 behaviors -> infer -> Dag
//
// Fail-closed compile boundary (invariant C-8):
//   compile_to_dag returns Ok(Dag) ONLY when the diagnostic table
//   is empty. Any semantic errors (type mismatches, unresolved
//   names, arity errors, etc.) surface as Err(CompileError::Semantic(dag))
//   — the dag is still handed back so the caller can inspect the
//   diagnostics, but the Result variant is Err.
//
//   Structural errors (tokenize/parse) surface as their own variants
//   because they occur before a Dag exists. G5: no TypeError variant
//   on CompileError — type errors live on the Dag, not in the Err
//   payload.

pub mod dag;
pub mod diagnostics;
mod regen_bootstrap_emit;

/// SG-0 producer-owned generated-file manifest.
///
/// `GENERATED_FILES` is the workspace-relative path list of every
/// `.rs` file under `src/v3/compiler` that is produced by a codegen
/// authority. The list is emitted by `build.rs` at build time; the
/// literal is reviewed there. Two consumers today: the `regen_*`
/// binaries (they assert their output path is in the list before
/// writing) and the SG-0 census test (it uses the list as the sole
/// generated/hand-authored partition — no content-marker scanning).
pub mod generated_files {
    include!(concat!(env!("OUT_DIR"), "/v3_generated_files.rs"));
}

pub mod emit;
pub mod emit_rust;
pub mod lens_depth;
pub mod lens_testgen;
pub mod lens_unused_parameters;
pub mod post_emit_verifier;
pub mod serialize {
    use crate::dag::{Behavior, Dag};
    use crate::diagnostics::Diagnostic;

    include!("serialize_generated.rs");
}
pub mod types {
    use crate::dag::DeclarationId;

    include!("types_generated.rs");
}
pub mod parse_surface {
    use crate::diagnostics::SourceSpan;
    use crate::operators::OperatorKind;

    include!("parse_surface_generated.rs");
}

pub use regen_bootstrap_emit::{render_bootstrap_generated_rs, render_bootstrap_std_generated_rs};

pub mod operators {
    pub use crate::dag::{ArithmeticOp, ComparisonOp, LogicalOp, OperatorKind};

    mod generated {
        #![allow(
            dead_code,
            unused_imports,
            unused_parens,
            unused_variables,
            clippy::clone_on_copy,
            clippy::collapsible_else_if
        )]

        use crate::dag::{ArithmeticOp, ComparisonOp, LogicalOp, OperatorKind};

        include!("operators_generated.rs");
    }

    pub use generated::{algebra_field_name, from_symbol, symbol};
}

/// Cost lens. The authority lives in `src/v3/lenses/complexity.dag`;
/// the Rust projection is auto-emitted into
/// `src/v3/compiler/src/lens_cost_generated.rs` and re-exported here
/// so callers use `v3_compiler::lens_cost::{cost_of, CostLookup}`.
/// Editing the lens means editing the `.dag` — there is no
/// hand-written implementation on this crate side.
///
/// L-8 compliance: `cost_of` returns the typed `CostLookup` carrier
/// (`MissingCost | FoundCost(Int)`). Callers pattern-match on the
/// variant rather than receiving a panicked-collapsed `usize`.
pub mod lens_cost {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if,
        clippy::large_enum_variant
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_cost_generated.rs");
    }

    pub use generated::{cost_of, CostLookup};

    #[cfg(test)]
    mod tests {
        use super::{cost_of, CostLookup};
        use crate::dag::{
            ArithmeticOp, BranchPattern, Dag, LiteralBits, LoopBound, OperatorKind, Path, PortId,
            TransformTarget,
        };
        use crate::diagnostics::SourceSpan;

        fn span() -> SourceSpan {
            SourceSpan::new("<lens-cost-test>", 0, 0)
        }

        fn expect_found(lookup: CostLookup) -> i64 {
            match lookup {
                CostLookup::FoundCost { _0: c } => c,
                CostLookup::MissingCost => panic!("expected FoundCost, got MissingCost"),
            }
        }

        fn assert_cost(dag: &Dag, port: PortId, expected: i64) {
            assert_eq!(expect_found(cost_of(dag, &port)), expected);
        }

        fn int_value(dag: &mut Dag, value: i64) -> PortId {
            dag.push_value(LiteralBits::Int(value), span())
        }

        fn add(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
            dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![lhs, rhs],
                span(),
            )
        }

        fn bind_arm(dag: &mut Dag, name: &str, output: PortId) -> Path {
            Path {
                body: dag.push_bind(name, output, Vec::new(), span()),
                output,
                pattern: BranchPattern::UnresolvedVariant {
                    name: name.to_string(),
                    span: span(),
                },
                binding: None,
            }
        }

        #[test]
        fn value_port_has_zero_cost() {
            let mut dag = Dag::new();
            let port = dag.push_value(LiteralBits::Int(7), span());
            assert_cost(&dag, port, 0);
        }

        #[test]
        fn transform_adds_one_to_sum_of_input_costs() {
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let sum = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            assert_cost(&dag, sum, 1);
        }

        #[test]
        fn chained_transforms_accumulate_through_input_edges() {
            // (1 + 2) + 3: outer transform = 1 + (inner=1) + (literal=0) = 2.
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let c = dag.push_value(LiteralBits::Int(3), span());
            let inner = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            let outer = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![inner, c],
                span(),
            );
            assert_cost(&dag, outer, 2);
        }

        #[test]
        fn branch_cost_is_one_plus_input_plus_max_of_path_outputs() {
            // cond=Bool(true) [0], arm=Int(1) [0] → branch = 1 + 0 + 0 = 1.
            let mut dag = Dag::new();
            let cond = dag.push_value(LiteralBits::Bool(true), span());
            let arm_output = int_value(&mut dag, 1);
            let arm_body = dag.push_bind("arm", arm_output, Vec::new(), span());
            let branch = dag.push_branch(
                cond,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Only".to_string(),
                        span: span(),
                    },
                    binding: None,
                }],
                span(),
            );
            assert_cost(&dag, branch, 1);
        }

        #[test]
        fn branch_cost_uses_max_not_sum_across_paths() {
            let mut dag = Dag::new();
            let cond = dag.push_value(LiteralBits::Bool(true), span());
            let cheap = int_value(&mut dag, 20);
            let forty = int_value(&mut dag, 40);
            let fifty = int_value(&mut dag, 50);
            let pricey = add(&mut dag, forty, fifty);
            let sixty = int_value(&mut dag, 60);
            let pricier = add(&mut dag, pricey, sixty);
            let paths = vec![
                bind_arm(&mut dag, "cheap_arm", cheap),
                bind_arm(&mut dag, "pricier_arm", pricier),
            ];
            let branch = dag.push_branch(cond, paths, span());

            // branch = 1 + cond(0) + max(cheap=0, pricier=((40+50)+60)=2)
            assert_cost(&dag, branch, 3);
        }

        #[test]
        fn loop_cost_is_one_plus_source_plus_init() {
            let mut dag = Dag::new();
            let source = dag.push_value(LiteralBits::Int(4), span());
            let init = dag.push_value(LiteralBits::Int(0), span());
            let body_output = dag.push_value(LiteralBits::Int(0), span());
            let body = dag.push_bind("loop_body", body_output, Vec::new(), span());
            let loop_port = dag.push_loop(
                source,
                init,
                body,
                LoopBound::Cardinality { count: source },
                span(),
            );
            assert_cost(&dag, loop_port, 1);
        }

        #[test]
        fn bind_cost_tracks_body_value_cost() {
            // let x = 1 + 2: bind.value is the Add transform (cost 1).
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            let _ = dag.push_bind("x", body, Vec::new(), span());
            assert_cost(&dag, body, 1);
        }

        #[test]
        fn bind_params_seed_as_zero_cost_and_body_costs_accumulate() {
            // fn double(x) = x + x: body transform reads x twice. Each param
            // port is seeded to cost 0, so body cost = 1 + 0 + 0 = 1.
            let mut dag = Dag::new();
            let int_shape = dag.int_shape().expect("bootstrap Int");
            let x = dag.alloc_port_with_shape(int_shape);
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![x, x],
                span(),
            );
            let _ = dag.push_bind("double", body, vec![x], span());

            // Parameter ports look up against the seeded entries.
            assert_cost(&dag, x, 0);
            assert_cost(&dag, body, 1);
        }
    }
}

/// Symbolic-cost lens (Lane 2 Stage 2d / DB-7). Authority lives in
/// `src/v3/lenses/cost.dag`; the Rust projection is auto-emitted
/// into `src/v3/compiler/src/lens_cost_symbolic_generated.rs` and
/// re-exported so callers use `v3_compiler::lens_cost_symbolic::*`.
///
/// The `SymbolicCost` + `SizeVariable` carriers live in
/// `src/v3/compiler/src/dag.rs` rather than the generated module
/// because they're declared in `src/v3/std/algebra.dag`, which
/// `emit_rust_module`'s `is_bootstrap_file` filter excludes from
/// type emission. The hand-maintained Rust mirror adjacent to
/// `Behavior` / `LoopBound` follows the same substrate-ownership
/// pattern the other bootstrap-resident types use.
pub mod lens_cost_symbolic {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_cost_symbolic_generated.rs");
    }

    pub use generated::{symbolic_cost_of, SymbolicCostEntry, SymbolicCostLookup};
}

/// Provenance lens. The authority lives in
/// `src/v3/lenses/provenance.dag`; the Rust projection is auto-emitted
/// into `src/v3/compiler/src/lens_provenance_generated.rs` and wrapped
/// here as a module so callers use `v3_compiler::lens_provenance`.
/// Editing the lens means editing the `.dag` — there is no hand-written
/// implementation on this crate side.
///
/// Only `Origin` and `origin_of` are re-exported. The generated module
/// also declares internal helper carriers (`PortLookup`,
/// `BehaviorLookup`) and their `find_*` / `behavior_id` walkers, which
/// exist solely because the substrate still exposes `Dag.ports` /
/// `Dag.nodes` as linear lists. Those helpers are bounded scaffolding
/// that dissolves when the substrate grows total keyed `port(id)` /
/// `node(id)` accessors — keeping them crate-private now prevents the
/// tracked-scaffold from leaking into `v3_compiler::lens_provenance`'s
/// public surface and attracting downstream consumers.
pub mod lens_provenance {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_provenance_generated.rs");
    }

    pub use generated::{origin_of, Origin};

    #[cfg(test)]
    mod tests {
        use super::{origin_of, Origin};
        use crate::dag::{
            ArithmeticOp, BranchPattern, Dag, LiteralBits, LoopBound, OperatorKind, Path,
            TransformTarget,
        };
        use crate::diagnostics::SourceSpan;

        fn span() -> SourceSpan {
            SourceSpan::new("<lens-provenance-test>", 0, 0)
        }

        fn label(origin: &Origin) -> &'static str {
            match origin {
                Origin::NoProducer => "NoProducer",
                Origin::MissingPort => "MissingPort",
                Origin::MissingBehavior => "MissingBehavior",
                Origin::Source { .. } => "Source",
                Origin::Computed { .. } => "Computed",
                Origin::Selected { .. } => "Selected",
                Origin::Accumulated { .. } => "Accumulated",
            }
        }

        #[test]
        fn unproduced_parameter_port_reports_no_producer() {
            let mut dag = Dag::new();
            let int_shape = dag.int_shape().expect("bootstrap Int");
            let param = dag.alloc_port_with_shape(int_shape);
            assert_eq!(label(&origin_of(&dag, &param)), "NoProducer");
        }

        #[test]
        fn value_port_reports_source_origin() {
            let mut dag = Dag::new();
            let port = dag.push_value(LiteralBits::Int(1), span());
            assert_eq!(label(&origin_of(&dag, &port)), "Source");
        }

        #[test]
        fn transform_port_reports_computed_origin() {
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let sum = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            assert_eq!(label(&origin_of(&dag, &sum)), "Computed");
        }

        #[test]
        fn branch_port_reports_selected_origin() {
            let mut dag = Dag::new();
            let cond = dag.push_value(LiteralBits::Bool(true), span());
            let arm_output = dag.push_value(LiteralBits::Int(1), span());
            let arm_body = dag.push_bind("arm", arm_output, Vec::new(), span());
            let branch = dag.push_branch(
                cond,
                vec![Path {
                    body: arm_body,
                    output: arm_output,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Only".to_string(),
                        span: span(),
                    },
                    binding: None,
                }],
                span(),
            );
            assert_eq!(label(&origin_of(&dag, &branch)), "Selected");
        }

        #[test]
        fn loop_port_reports_accumulated_origin() {
            let mut dag = Dag::new();
            let source = dag.push_value(LiteralBits::Int(4), span());
            let init = dag.push_value(LiteralBits::Int(0), span());
            let body_output = dag.push_value(LiteralBits::Int(0), span());
            let body = dag.push_bind("loop_body", body_output, Vec::new(), span());
            let loop_port = dag.push_loop(
                source,
                init,
                body,
                LoopBound::Cardinality { count: source },
                span(),
            );
            assert_eq!(label(&origin_of(&dag, &loop_port)), "Accumulated");
        }

        #[test]
        fn bind_value_origin_is_its_producer_not_the_bind_itself() {
            // `let x = 1 + 2` — bind.value is the transform output, so
            // origin_of(bind.value) walks through to the Transform producer
            // and reports Computed. A Bind's own output is only reached
            // when something references the Bind node directly.
            let mut dag = Dag::new();
            let a = dag.push_value(LiteralBits::Int(1), span());
            let b = dag.push_value(LiteralBits::Int(2), span());
            let body = dag.push_transform(
                TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
                vec![a, b],
                span(),
            );
            let _ = dag.push_bind("x", body, Vec::new(), span());
            assert_eq!(label(&origin_of(&dag, &body)), "Computed");
        }
    }
}

/// Structural-resolution lens. The authority lives in
/// `src/v3/lenses/structural_resolution.dag`; the Rust projection is
/// auto-emitted into `src/v3/compiler/src/lens_structural_resolution_generated.rs`
/// and wrapped here as a module so callers use
/// `v3_compiler::lens_structural_resolution`. Editing the lens means
/// editing the `.dag` — there is no hand-written implementation on
/// this crate side.
///
/// Detects leaked `ArrowBody::Pending` in the final Dag.
/// Defense-in-depth regression pin for the R13 fix (see the `.dag`
/// source for the full detection rule and disposal trigger).
pub mod lens_structural_resolution {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;
        use crate::diagnostics::*;

        include!("lens_structural_resolution_generated.rs");
    }

    pub use generated::{check, name_keyed_references, NameKeyedReference, UnresolvedArrowBody};

    #[cfg(test)]
    mod tests {
        use super::{check, name_keyed_references, NameKeyedReference, UnresolvedArrowBody};
        use crate::dag::{ArrowBody, AtomPayload, Declaration, DeclarationId, TypeConnective};
        use crate::diagnostics::SourceSpan;
        use crate::{compile_to_dag, Dag};

        fn span() -> SourceSpan {
            SourceSpan::new("<lens-structural-resolution-test>", 0, 0)
        }

        fn inject_named_pending_arrow(
            dag: &mut Dag,
            name: &str,
            output_type: DeclarationId,
        ) -> DeclarationId {
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: Some(name.to_string()),
                connective: TypeConnective::Arrow {
                    inputs: Vec::new(),
                    output: output_type,
                    body: ArrowBody::Pending,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: span(),
            });
            id
        }

        fn inject_anonymous_pending_arrow(
            dag: &mut Dag,
            output_type: DeclarationId,
        ) -> DeclarationId {
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Arrow {
                    inputs: Vec::new(),
                    output: output_type,
                    body: ArrowBody::Pending,
                },
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: span(),
            });
            id
        }

        fn inject_name_keyed_reference(dag: &mut Dag, target: DeclarationId) -> DeclarationId {
            let id = dag.alloc_declaration_id();
            dag.push_declaration(Declaration {
                id,
                name: None,
                connective: TypeConnective::Atom(AtomPayload::ResolvedByName(target)),
                type_params: Vec::new(),
                meta_tag: None,
                inhabits: None,
                value_body: None,
                refinement: None,
                span: span(),
            });
            id
        }

        fn violations(dag: &Dag) -> Vec<UnresolvedArrowBody> {
            check(dag)
        }

        fn name_keyed(dag: &Dag) -> Vec<NameKeyedReference> {
            name_keyed_references(dag)
        }

        #[test]
        fn lens_flags_named_arrow_pending_injected_into_dag() {
            let mut dag = Dag::new();
            let int_output = dag.int_shape().expect("bootstrap Dag has Int").declaration;
            let decl_id = inject_named_pending_arrow(&mut dag, "leaked_fn", int_output);

            let found = violations(&dag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one violation, got: {found:?}"
            );
            assert_eq!(found[0].declaration, decl_id);
            assert_eq!(found[0].name, "leaked_fn");
        }

        #[test]
        fn lens_silent_on_empty_bootstrap_dag() {
            let dag = Dag::new();
            let found = violations(&dag);
            assert!(
                found.is_empty(),
                "bootstrap Dag must produce zero violations (algebra arrows are anonymous), got: {found:?}"
            );
        }

        #[test]
        fn lens_flags_anonymous_arrow_pending_injected_into_dag() {
            let mut dag = Dag::new();
            let int_output = dag.int_shape().expect("bootstrap Dag has Int").declaration;
            let decl_id = inject_anonymous_pending_arrow(&mut dag, int_output);

            let found = violations(&dag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one anonymous violation, got: {found:?}"
            );
            assert_eq!(found[0].declaration, decl_id);
            assert_eq!(found[0].name, "<anonymous>");
        }

        #[test]
        fn lens_survives_co_existing_injected_and_compiled_declarations() {
            let mut dag =
                compile_to_dag("fn good(x: Int) -> Int = x + 1", "user.v3").expect("compiles");
            let int_output = dag.int_shape().expect("Int shape").declaration;
            let leak_id = inject_named_pending_arrow(&mut dag, "leaked", int_output);
            let found = violations(&dag);
            assert_eq!(
                found.len(),
                1,
                "expected exactly one violation amid real declarations, got: {found:?}"
            );
            assert_eq!(found[0].declaration, leak_id);
            assert_eq!(found[0].name, "leaked");
        }

        #[test]
        fn lens_flags_injected_name_keyed_reference() {
            let mut dag = Dag::new();
            let int_id = dag.int_shape().expect("bootstrap Dag has Int").declaration;
            let site_id = inject_name_keyed_reference(&mut dag, int_id);

            let found = name_keyed(&dag);
            let injected = found
                .iter()
                .find(|entry| entry.declaration == site_id)
                .unwrap_or_else(|| {
                    panic!("expected injected site in name-keyed references, got: {found:?}")
                });
            assert_eq!(injected.resolved_to, int_id);
        }
    }
}

mod bootstrap;
mod dimension;
mod infer;

/// SG-4 prep: first .dag-authority slice of `infer.rs`. Authority
/// lives in `src/v3/lenses/infer_helpers.dag`; the Rust projection is
/// auto-emitted into `src/v3/compiler/src/infer_helpers_generated.rs`
/// and consumed by `infer.rs` via
/// `crate::infer_helpers::behavior_output_port`. Editing the helper
/// means editing the `.dag` — there is no hand-written implementation
/// on this crate side. SG-6 owns folding the standalone regen driver
/// and relocating extracted helper modules out of `lenses/` once the
/// consolidated regen target lands.
pub(crate) mod infer_helpers {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]
    mod generated {
        use crate::dag::*;

        include!("infer_helpers_generated.rs");
    }

    pub(crate) use generated::{
        behavior_output_port, resolve_template_argument_value, template_argument_value,
        TemplateArgumentLookup,
    };
}

pub mod lens_idempotency;
pub mod lens_parallelism;
mod lower;
#[path = "parse_generated.rs"]
mod parse;
mod pipeline_authority;
mod regen_parse_emit;
mod tokenize;

pub use regen_parse_emit::{render_parse_generated_rs, RenderParseGeneratedError};
pub(crate) mod variant_payload {
    #[allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if,
        clippy::cmp_owned,
        clippy::large_enum_variant
    )]
    mod generated {
        use crate::dag::*;

        include!("variant_payload_generated.rs");
    }

    pub(crate) use generated::{
        variant_payload_shape, VariantPayloadShape, VariantPayloadShapeLookup,
    };
}
pub(crate) mod workflow_idempotency;
pub(crate) mod workflow_parallelism;

pub use dag::{Dag, NodeId};
pub use diagnostics::{Diagnostic, SourceSpan};
pub use emit::{EmitDispatchError, EmitMode, EmitTarget, EmittedSource};
pub use emit_rust::EmitError;
/// Lane 2 Stage 2b — supported public surface: [`analyze_workflow`] is the
/// primary entry; [`report_unsupported_workflow_variant`] and
/// [`lane2_workflow_idempotency_report`] are additionally exported so
/// `emit_rust_module(idempotency.dag)` output can link in rustc round-trip
/// tests. Composition helpers such as `compose_operation_effects` /
/// `operation_to_breaker` are **not** re-exported: naming and algebra authority
/// live in `src/v3/std/effects.dag`, and the Rust bridge must not become a
/// parallel public implementation surface beyond these std.effects mirrors.
pub use lens_idempotency::analyze_workflow;
/// Lane 2 Stage 2e — parallel composition safety (`ParallelEffect`); see DB-20.
pub use lens_parallelism::analyze_parallelism;
pub use workflow_idempotency::{
    lane2_workflow_idempotency_report, report_unsupported_workflow_variant,
};

/// Lane 2 Stage 2f — DB-3 dimension abstraction (`std/dimensions.dag` types;
/// `analyze_symbolic_cost_dimension` is the first migrated lens path).
pub use dimension::{
    analyze_symbolic_cost_dimension, behavior_spine_in_node_order, DimensionReport, Witness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageSnapshotKind {
    Surface,
    Text,
    Dag,
}

#[derive(Debug, Clone)]
pub struct StageSnapshot {
    pub stage: String,
    pub kind: StageSnapshotKind,
    pub bytes: Vec<u8>,
    pub dag: Option<Dag>,
}

#[derive(Debug)]
pub enum StageSnapshotError {
    Compile(Box<CompileError>),
    Emit(Box<emit_rust::EmitError>),
    Pipeline(String),
}

#[derive(Debug)]
pub struct FixedPointMismatch {
    pub stage: String,
    pub detail: String,
}

/// Test-only hook: tokenize a source string. Used by the
/// `real_stdlib_parse_smoke` integration test to verify the parser
/// accepts production `dsl/std/*.dag` files before bootstrap migration.
#[doc(hidden)]
pub fn tokenize_for_test(source: &str, file: &str) -> Result<Vec<tokenize::Token>, Diagnostic> {
    tokenize::tokenize(source, file)
}

/// Test-only hook: parse a token stream into a surface module.
#[doc(hidden)]
pub fn parse_for_test(
    tokens: &[tokenize::Token],
    file: &str,
) -> Result<parse::SurfaceModule, Diagnostic> {
    parse::parse(tokens, file)
}

/// Test-only hook: top-level `let` binding names in source order.
#[doc(hidden)]
pub fn surface_top_level_let_names_for_test(module: &parse::SurfaceModule) -> Vec<String> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            parse::SurfaceItem::Let { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// Test hook: pipeline stage identifiers in `compile { ... }` order in
/// `pipeline.dag` — the same ordering as `materialize_pipeline_realizations`.
#[doc(hidden)]
pub fn pipeline_compile_order_stage_names() -> Result<Vec<String>, String> {
    pipeline_authority::pipeline_compile_order_names()
}

/// Top-level compile failure. Distinguishes three structural
/// categories of failure by phase of the pipeline where they occurred.
///
/// **Dissolution receipt: TERMINAL.** Three variants, each with a
/// structurally distinct payload:
/// - `Tokenize(Diagnostic)`: tokenization produced a single diagnostic;
///   no Dag exists yet, so no Dag payload.
/// - `Parse(Diagnostic)`: parsing produced a single diagnostic; no Dag
///   exists yet.
/// - `Semantic(Dag)`: lowering/inference produced one or more
///   diagnostics; the Dag exists and carries them in its diagnostic
///   table, so it's handed back as the payload for caller inspection.
///
/// The three variants correspond to three structurally different
/// failure states (no-Dag-yet with a diagnostic vs Dag-with-
/// diagnostic-table). Pattern 2 (variant-is-data) fails because the
/// payloads are different types. Pattern 3 (algebraic-form) doesn't
/// apply — these are failure phases, not algebraic operations.
///
/// Guardrail G5: there is no `TypeError` variant. Type errors are
/// data on the Dag via the diagnostic table, not fields on the
/// error type. `Semantic(Dag)` is a handoff, not a classification of
/// what went wrong — the caller reads `dag.diagnostics()` for
/// specifics. This is what "fail-closed at the boundary" means in
/// practice: a successful compile returns `Ok(Dag)` with an empty
/// diagnostic table; a failed compile returns `Err(Semantic(Dag))`
/// with a non-empty one. There is no third outcome.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum CompileError {
    Tokenize(diagnostics::Diagnostic),
    Parse(diagnostics::Diagnostic),
    /// Semantic errors. The Dag is included so callers can inspect
    /// `dag.diagnostics()` to see what went wrong. `Err(Semantic(_))`
    /// means: the compile reached infer, some (>=1) diagnostics were
    /// produced, and the result is not usable.
    Semantic(Dag),
}

// `result_large_err`: clippy flags `Result<Dag, CompileError>`
// because `CompileError::Semantic(Dag)` carries a `Dag` payload
// (~264 bytes after the M1(3) PR-B-unwind R1 added the realization
// meta cache). Boxing the Dag would touch every pattern-match
// against `CompileError::Semantic` in the test suite, and the
// payload is on the cold failure path where the indirection would
// matter less than the API churn. Targeted `allow` on the function
// signature only — the rest of the crate keeps the lint enforced.
#[allow(clippy::result_large_err)]
pub fn compile_to_dag(source: &str, file: &str) -> Result<Dag, CompileError> {
    let tokens = tokenize::tokenize(source, file).map_err(CompileError::Tokenize)?;
    let surface = parse::parse(&tokens, file).map_err(CompileError::Parse)?;
    let mut dag = lower::lower(&surface);
    infer::infer(&mut dag);
    if dag.diagnostics().is_empty() {
        Ok(dag)
    } else {
        Err(CompileError::Semantic(dag))
    }
}

/// Lower `runtime_mirrors.dag` for codegen (`regen_parse`, SG-2 staging tests).
///
/// Unlike [`compile_to_dag`], this starts from a bootstrap Dag that omits the embedded
/// `runtime_mirrors.dag` compiler fixture so the fresh parse is first-of-name and can be
/// lowered without duplicate-declaration diagnostics.
#[allow(clippy::result_large_err)]
pub fn compile_runtime_mirrors_authority_dag(
    source: &str,
    file: &str,
) -> Result<Dag, CompileError> {
    let tokens = tokenize::tokenize(source, file).map_err(CompileError::Tokenize)?;
    let surface = parse::parse(&tokens, file).map_err(CompileError::Parse)?;
    let mut dag = Dag::new_without_runtime_mirrors_compiler_fixture_bootstrap();
    let user_start = dag.declarations().len();
    lower::lower_into(&mut dag, &surface);
    lower::finalize_strict_user_lower_range(&mut dag, user_start);
    infer::infer(&mut dag);
    if dag.diagnostics().is_empty() {
        Ok(dag)
    } else {
        Err(CompileError::Semantic(dag))
    }
}

/// PB-1 scaffold helper: re-run the pre-snapshot std bootstrap path for
/// `regen_bootstrap` and the PB-1 drift tests only. This is NOT a second
/// production bootstrap authority; `Dag::new()` seeds from the committed
/// generated snapshot. Dissolution trigger: same as
/// `bootstrap::bootstrap_std_fixtures_only`.
pub fn compile_std_bootstrap_dag() -> Dag {
    let mut dag = Dag::empty();
    bootstrap::bootstrap_std_fixtures_only(&mut dag);
    dag
}

/// PB-1-a generated snapshot helper: load the committed std-fixture
/// bootstrap snapshot without re-running tokenize/parse/lower.
pub fn generated_std_bootstrap_dag() -> Dag {
    Dag::std_fixture_bootstrap_snapshot()
}

/// PB-1 closure scaffold helper for `regen_bootstrap`: layer the staged/spec/
/// compiler bootstrap authorities onto an explicitly supplied std seed so all
/// generated outputs in one regen pass derive from the same `dsl/std/*.dag`
/// authority. This is not a production bootstrap entry point.
pub fn compile_full_bootstrap_dag_from_std_seed(std_seed: Dag) -> Dag {
    let mut dag = std_seed;
    bootstrap::bootstrap_runtime_authorities_on(&mut dag, &[]);
    dag
}

/// PB-1 closure scaffold helper for `regen_bootstrap`: same as
/// `compile_full_bootstrap_dag_from_std_seed`, but excludes
/// `runtime_mirrors.dag` so regen/tests can keep that authority first-of-name.
pub fn compile_full_bootstrap_without_runtime_mirrors_dag_from_std_seed(std_seed: Dag) -> Dag {
    let mut dag = std_seed;
    bootstrap::bootstrap_runtime_authorities_on(&mut dag, &["src/v3/compiler/runtime_mirrors.dag"]);
    dag
}

pub fn compile_full_bootstrap_dag() -> Dag {
    let mut dag = Dag::empty();
    bootstrap::bootstrap_all_runtime(&mut dag, &[]);
    dag
}

pub fn compile_full_bootstrap_without_runtime_mirrors_dag() -> Dag {
    let mut dag = Dag::empty();
    bootstrap::bootstrap_all_runtime(&mut dag, &["src/v3/compiler/runtime_mirrors.dag"]);
    dag
}

pub fn generated_full_bootstrap_dag() -> Dag {
    Dag::new()
}

pub fn generated_full_bootstrap_without_runtime_mirrors_dag() -> Dag {
    Dag::new_without_runtime_mirrors_compiler_fixture_bootstrap()
}

pub fn default_fixed_point_source() -> &'static str {
    "let x: Int = 1 + 2\nlet y: Int = x + 3\n"
}

pub fn compile_stage_snapshots(
    source: &str,
    file: &str,
) -> Result<Vec<StageSnapshot>, StageSnapshotError> {
    let pipeline_dag = Dag::new();
    if !pipeline_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(Box::new(
            CompileError::Semantic(pipeline_dag),
        )));
    }
    let pipeline = pipeline_authority::ordered_pipeline_stages(&pipeline_dag)
        .map_err(StageSnapshotError::Pipeline)?;

    let tokens = tokenize::tokenize(source, file)
        .map_err(CompileError::Tokenize)
        .map_err(|error| StageSnapshotError::Compile(Box::new(error)))?;
    let surface = parse::parse(&tokens, file)
        .map_err(CompileError::Parse)
        .map_err(|error| StageSnapshotError::Compile(Box::new(error)))?;
    let parse_bytes = format!("{surface:#?}").into_bytes();

    let mut lower_dag = lower::lower(&surface);
    let lower_snapshot = lower_dag.clone();
    let lower_bytes = serialize::serialize_dag(&lower_snapshot);

    infer::infer(&mut lower_dag);
    if !lower_dag.diagnostics().is_empty() {
        return Err(StageSnapshotError::Compile(Box::new(
            CompileError::Semantic(lower_dag.clone()),
        )));
    }

    let infer_snapshot = lower_dag.clone();
    let infer_bytes = serialize::serialize_dag(&infer_snapshot);
    let emitted = emit::emit(&lower_dag, EmitTarget::Rust)
        .map(|source| source.text)
        .map_err(|error| match error {
            emit::EmitDispatchError::Core(error) => StageSnapshotError::Emit(Box::new(error)),
            emit::EmitDispatchError::Python(_) => {
                unreachable!("EmitTarget::Rust cannot yield a Python emission error")
            }
        })?;

    let mut snapshots = Vec::with_capacity(pipeline.len());
    for stage in pipeline {
        let (kind, bytes, dag) = match stage.stage_name.as_str() {
            "parse" => (StageSnapshotKind::Surface, parse_bytes.clone(), None),
            "lower" => (
                StageSnapshotKind::Dag,
                lower_bytes.clone(),
                Some(lower_snapshot.clone()),
            ),
            "infer" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "compute_ownership" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "lens_complexity" => (
                StageSnapshotKind::Dag,
                infer_bytes.clone(),
                Some(infer_snapshot.clone()),
            ),
            "emit" => (StageSnapshotKind::Text, emitted.clone().into_bytes(), None),
            other => {
                return Err(StageSnapshotError::Pipeline(format!(
                    "pipeline stage `{other}` has no Rust snapshot implementation"
                )));
            }
        };

        if !snapshot_kind_matches(stage.snapshot_kind, kind) {
            return Err(StageSnapshotError::Pipeline(format!(
                "pipeline stage `{}` declares snapshot kind {:?} but Rust produced {:?}",
                stage.stage_name, stage.snapshot_kind, kind
            )));
        }

        snapshots.push(StageSnapshot {
            stage: stage.stage_name,
            kind,
            bytes,
            dag,
        });
    }

    Ok(snapshots)
}

pub fn compare_stage_snapshots(
    lhs: &[StageSnapshot],
    rhs: &[StageSnapshot],
) -> Result<(), FixedPointMismatch> {
    if lhs.len() != rhs.len() {
        return Err(FixedPointMismatch {
            stage: "pipeline".to_string(),
            detail: format!(
                "stage count mismatch: pass1 has {}, pass2 has {}",
                lhs.len(),
                rhs.len()
            ),
        });
    }

    for (left, right) in lhs.iter().zip(rhs.iter()) {
        if left.stage != right.stage {
            return Err(FixedPointMismatch {
                stage: "pipeline".to_string(),
                detail: format!(
                    "stage order mismatch: pass1 has `{}`, pass2 has `{}`",
                    left.stage, right.stage
                ),
            });
        }
        if left.kind != right.kind {
            return Err(FixedPointMismatch {
                stage: left.stage.clone(),
                detail: format!(
                    "snapshot kind mismatch at stage `{}`: pass1={:?}, pass2={:?}",
                    left.stage, left.kind, right.kind
                ),
            });
        }
        if left.bytes == right.bytes {
            continue;
        }

        let detail = match (&left.dag, &right.dag) {
            (Some(lhs_dag), Some(rhs_dag)) => serialize::first_difference(lhs_dag, rhs_dag)
                .map(|diff| diff.detail)
                .unwrap_or_else(|| first_differing_line(&left.bytes, &right.bytes)),
            _ => first_differing_line(&left.bytes, &right.bytes),
        };
        return Err(FixedPointMismatch {
            stage: left.stage.clone(),
            detail,
        });
    }

    Ok(())
}

fn snapshot_kind_matches(
    declared: pipeline_authority::PipelineSnapshotKind,
    actual: StageSnapshotKind,
) -> bool {
    matches!(
        (declared, actual),
        (
            pipeline_authority::PipelineSnapshotKind::Surface,
            StageSnapshotKind::Surface
        ) | (
            pipeline_authority::PipelineSnapshotKind::Dag,
            StageSnapshotKind::Dag
        ) | (
            pipeline_authority::PipelineSnapshotKind::Text,
            StageSnapshotKind::Text
        )
    )
}

fn first_differing_line(lhs: &[u8], rhs: &[u8]) -> String {
    let lhs = String::from_utf8_lossy(lhs);
    let rhs = String::from_utf8_lossy(rhs);
    for (idx, (left, right)) in lhs.lines().zip(rhs.lines()).enumerate() {
        if left != right {
            return format!(
                "first differing line {}: pass1=`{}`, pass2=`{}`",
                idx + 1,
                left,
                right
            );
        }
    }
    format!(
        "snapshot byte-length mismatch: pass1={} bytes, pass2={} bytes",
        lhs.len(),
        rhs.len()
    )
}
