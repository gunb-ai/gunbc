// Dag::new() bootstrap.
//
// PB-1 closure: `Dag::new()` no longer tokenizes/parses/lowers any
// bootstrap authority at runtime. Five bootstrap authority sets
// are loaded from committed generated snapshots:
//
// - `std_fixtures`                 (`dsl/std/*.dag`)
// - `STAGED_FILES` / `V3_SPECS` / `COMPILER_FILES` / extdeps primitives — fresh
//   tokenize/parse/lower lives in `bootstrap_regen_fresh.rs` behind feature
//   `bootstrap-regen-fresh` (`regen_bootstrap` only). `Dag::new()` does not load
//   them from disk at runtime.
//
// **Single authority at runtime:** `Dag::new()` (and `std_fixture_bootstrap_snapshot`)
// materialize **only** from the committed `bootstrap_*_generated.rs` snapshots
// (`include!` / generated Rust). They never re-tokenize or re-parse the `OUT_DIR`
// fixture string tables — those arrays exist solely for the feature-gated regen host.
//
// `compile_parse_surface_std_authority_dag` uses the companion snapshot
// that omits `src/v3/std/parse_surface.dag`, so a fresh parse+lower of
// that authority still stays first-of-name.
//
// **Target-language realization facts stay out of bootstrap.** Realization
// facts for emitted languages live in `dsl/extdeps/languages/*` per the
// thesis and are consumed by the per-target emitters at emission time via
// `SubstrateAccessorBinding` records — compiler code does not manufacture
// those. The L1.5 exception remains: `src/v3/compiler/pipeline.dag` stage
// declarations are upgraded in-place to `ArrowBody::ExternalRealization`
// so the compiler's own pipeline authority lives in the bootstrap Dag with
// the intended stage-body shape.
//
// `EXTDEPS_BOOTSTRAP_FIXTURES` is a narrow, bounded extension of that
// boundary: only extdeps authorities whose content is pure structural
// **data** (target-primitive declarations consumed symbolically by the
// target-grounding engine as `Declaration`-shaped values; see
// `dsl/extdeps/languages/rust/primitives.dag`) are loaded. Arrow/realization
// files (`rust/emit.dag`, `rust/types.dag`, etc.) are deliberately excluded
// — their bodies stay per-target emitter-side, not in the bootstrap Dag.
// Coverage is currently `rust/primitives.dag` only (T-Ground-Engine pilot
// unblock, Director-dispatched); expansion to python/go primitives is a
// file-system extension once those targets reach the same pilot stage.
//
// **Type-structure-only load (Path 2 scoping).** The top-level
// `rust_pilot_primitives: List<RustPrimitive> = [...]` data declaration
// lowers with `value_body = ValueBody::Unparsed(span)` because v3's
// `ValueBody` enum does not yet carry a top-level list/aggregate variant
// (`dag.rs:258-287`). Type-structure walking of `RustPrimitive =
// IntegerPrimitive | NonIntegerPrimitive {...}` is fully available via the
// loaded declarations; the 10-element pilot enumeration becomes walkable
// only when R2 T-Substrate's 4th sub-lane lands the top-level
// `ValueBody::List`/aggregate extension. Same substrate gap as
// `kernel_algebra_profile`'s hand-Rust mirror (`dag.rs:1530`) and
// tokenizer `sub_charclass_in_std_unicode` phase-2.
//
// Transitional shape: PB-Bootstrap-Process lane (Zero-Floor program;
// tracked in `docs/design-pure-bootstrap-zero.md` §"New lanes") absorbs
// `bootstrap.rs` entirely when it lands `bootstrap.dag` declaring the
// workflow as data — at that point this loader logic becomes part of the
// to-be-generated content with no hand-Rust edit.
//
// Bootstrap failures (tokenize/parse/lower errors on std/ files,
// unresolved cross-file references) attach to the Dag's diagnostic
// table via `Dag::attach_diagnostic` rather than panicking, so
// `compile_to_dag` surfaces them through `Err(CompileError::Semantic(dag))`
// on every subsequent call — the same structural channel user errors
// go through. A failed bootstrap is visible to callers without a
// side channel.
//
// **PB-1-e split — what stays here:** `patch_kernel_bool_boolean_algebra_inhabits`,
// `materialize_pipeline_realizations`, and `report_pipeline_authority_error` remain
// in this file because the `#[cfg(test)]` module below exercises them on
// `Dag::new()` snapshots. The regen-only fresh tokenize/parse/lower loop lives in
// `bootstrap_regen_fresh.rs` (feature `bootstrap-regen-fresh`); do not duplicate
// pipeline materialization there without relocating or rewriting these unit tests.

use crate::dag::{ArrowBody, Dag, Declaration, TemplateArgument, TypeConnective};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::pipeline_authority::{ordered_pipeline_stages, PIPELINE_AUTHORITY_FILE};

// Used by `materialize_pipeline_realizations` (regen + unit tests below). When
// `bootstrap-regen-fresh` is off, that path is cfg-dead in non-test lib builds.
#[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
const PIPELINE_REALIZATION_META: &str = "CompilerHostRealization";

/// v3-only inhabitance for kernel `Bool` (Class 5 / Lane 1e-2b Path A).
///
/// `dsl/std/types.dag` must stay free of `inhabits` so v2 can parse every
/// `dsl/` file. After the std fixtures lower, wire `Bool` to
/// `BooleanAlgebra<Bool>` the same way surface `inhabits` lowering would,
/// without shadowing `Bool` (which would reallocate sum variants and break
/// `src/v3/std/algebra.dag` pattern wiring).
///
/// Preconditions are checked: any failure attaches a bootstrap
/// `Diagnostic::ResolveError` via `Dag::attach_diagnostic` so compilation
/// fails closed instead of silently omitting `inhabits`.
///
/// **Dissolution:** remove this patch once the v2 compiler surface accepts
/// `type … inhabits … =` in `dsl/` (then express
/// `type Bool inhabits BooleanAlgebra<Bool> = True | False` in
/// `dsl/std/types.dag` and delete `patch_kernel_bool_boolean_algebra_inhabits`).
#[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
pub(crate) fn patch_kernel_bool_boolean_algebra_inhabits(dag: &mut Dag) {
    const BOOL_TYPES_FILE: &str = "dsl/std/types.dag";
    let Some(bool_decl) = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("Bool") && d.span.file == BOOL_TYPES_FILE)
    else {
        dag.attach_diagnostic(Diagnostic::ResolveError {
            name: format!(
                "bootstrap: Lane 1e-2b Path A — kernel `Bool` not found in `{BOOL_TYPES_FILE}`; \
                 cannot set `Declaration.inhabits` for `BooleanAlgebra<Bool>`"
            ),
            span: SourceSpan::new(BOOL_TYPES_FILE, 0, 0),
            fixes: Vec::new(),
        });
        return;
    };
    let bool_id = bool_decl.id;
    let span_for_inst = bool_decl.span.clone();
    if dag.declaration(bool_id).inhabits.is_some() {
        return;
    }
    let (ba_template, param_id) = {
        let Some(ba) = dag.declaration_by_name("BooleanAlgebra") else {
            dag.attach_diagnostic(Diagnostic::ResolveError {
                name: "bootstrap: Lane 1e-2b Path A — `BooleanAlgebra` not present in the \
                       bootstrap Dag; cannot wire kernel `Bool` `inhabits`"
                    .to_string(),
                span: span_for_inst.clone(),
                fixes: Vec::new(),
            });
            return;
        };
        let Some(&param_id) = ba.type_params.first() else {
            dag.attach_diagnostic(Diagnostic::ResolveError {
                name: "bootstrap: Lane 1e-2b Path A — `BooleanAlgebra` has no type parameters; \
                       cannot instantiate `BooleanAlgebra<Bool>` for kernel `Bool` `inhabits`"
                    .to_string(),
                span: span_for_inst.clone(),
                fixes: Vec::new(),
            });
            return;
        };
        (ba.id, param_id)
    };
    let inst_id = dag.alloc_declaration_id();
    dag.push_declaration(Declaration {
        id: inst_id,
        name: None,
        connective: TypeConnective::Instantiation {
            template: ba_template,
            arguments: vec![TemplateArgument {
                parameter: param_id,
                value: bool_id,
            }],
        },
        type_params: Vec::new(),
        phantom_params: Vec::new(),
        meta_tag: None,
        specialization_parent: None,
        inhabits: None,
        value_body: None,
        refinement: None,
        span: span_for_inst,
    });
    dag.declaration_mut(bool_id).inhabits = Some(inst_id);
}

// DB-14 substrate accessors (`port` / `node` / `resolve_producer`) are
// deliberately NOT materialized at bootstrap. Their Arrow bodies stay
// `Unparsed` (the `{ host <name> }` stub) because the accessor → realization
// mapping is TARGET-specific — upgrading the Arrow body once at bootstrap
// to a single realization would silently overwrite when a second emitter
// registers its own binding. Each emitter reads `SubstrateAccessorBinding`
// records at emission time, filters by its own `language: LanguageSpec`
// id, and dispatches to the target-matching realization's carrier
// template. See `emit_rust::RealizationIndexes::substrate_accessors`.
//
// This diverges from `materialize_pipeline_realizations` (above), which
// DOES upgrade Arrow bodies — pipeline stages are target-invariant
// (same runtime for every target), so "one realization per stage" is
// the correct authority there.

#[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
pub(crate) fn materialize_pipeline_realizations(dag: &mut Dag) {
    let stages = match ordered_pipeline_stages(dag) {
        Ok(stages) => stages,
        Err(error) => {
            report_pipeline_authority_error(dag, error);
            return;
        }
    };
    let Some(meta_decl_id) = dag
        .declaration_by_name(PIPELINE_REALIZATION_META)
        .map(|decl| decl.id)
    else {
        report_pipeline_authority_error(
            dag,
            format!("missing pipeline realization meta `{PIPELINE_REALIZATION_META}`"),
        );
        return;
    };
    let meta_connective = match dag.declaration(meta_decl_id).connective.clone() {
        connective @ TypeConnective::Conj { .. } => connective,
        _ => {
            report_pipeline_authority_error(
                dag,
                format!(
                    "pipeline realization meta `{PIPELINE_REALIZATION_META}` must lower to a record"
                ),
            );
            return;
        }
    };

    for stage in &stages {
        let realization = dag.declaration_mut(stage.realization);
        if realization.meta_tag != Some(meta_decl_id) {
            report_pipeline_authority_error(
                dag,
                format!(
                    "pipeline realization `{}` is not tagged with `{PIPELINE_REALIZATION_META}`",
                    stage.realization_name
                ),
            );
            continue;
        }
        realization.connective = meta_connective.clone();
    }

    for stage in stages {
        let stage_decl = dag.declaration_mut(stage.stage);
        match &mut stage_decl.connective {
            TypeConnective::Arrow { body, .. } => {
                *body = ArrowBody::ExternalRealization(stage.realization);
            }
            _ => report_pipeline_authority_error(
                dag,
                format!(
                    "pipeline stage `{}` must lower to an arrow",
                    stage.stage_name
                ),
            ),
        }
    }
}

#[cfg_attr(not(feature = "bootstrap-regen-fresh"), allow(dead_code))]
fn report_pipeline_authority_error(dag: &mut Dag, name: String) {
    dag.attach_diagnostic(Diagnostic::ResolveError {
        name,
        span: SourceSpan::new(PIPELINE_AUTHORITY_FILE, 0, 0),
        fixes: Vec::new(),
    });
}

#[cfg(test)]
mod tests {
    //! §6.5 realization smoke test. The stub chain (Realization meta-
    //! type, realization instance, realization Arrow) is constructed
    //! entirely inside this test module — no production bootstrap code
    //! is involved. The test exercises the
    //! `ArrowBody::ExternalRealization` substrate path end-to-end
    //! (construction + typed-edge validation + inference dispatch)
    //! without manufacturing realization facts at `Dag::new()` time.

    use super::*;
    use crate::dag::{ArrowBody, AtomPayload, Declaration, DeclarationId, TypeConnective};

    /// Build a Realization → instance → Arrow chain inside a fresh Dag.
    /// Returns the Arrow's DeclarationId so callers can walk it.
    fn inject_test_realization(dag: &mut Dag) -> DeclarationId {
        let span = SourceSpan::new("<test:realization>", 0, 0);

        let meta_type_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: meta_type_id,
            name: Some("TestRealization".to_string()),
            connective: TypeConnective::Conj {
                children: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,

            value_body: None,
            refinement: None,
            span: span.clone(),
        });

        let instance_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: instance_id,
            name: None,
            connective: TypeConnective::Conj {
                children: Vec::new(),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: Some(meta_type_id),
            specialization_parent: None,
            inhabits: None,

            value_body: None,
            refinement: None,
            span: span.clone(),
        });

        // Typed-edge check: verify the instance is realization-shaped
        // before encoding it in `ArrowBody::ExternalRealization`. This
        // is the same invariant `infer::is_realization_shape` enforces
        // at dispatch time; the test asserts it at construction time
        // as well, so both sides of the invariant are exercised.
        let instance_decl = dag.declaration(instance_id);
        assert!(
            matches!(instance_decl.connective, TypeConnective::Conj { .. }),
            "realization instance must be a Conj"
        );
        assert_eq!(
            instance_decl.meta_tag,
            Some(meta_type_id),
            "realization instance's meta_tag must point at the TestRealization meta-type"
        );

        // Use an anonymous Int primitive reference for the Arrow
        // inputs/output. At runtime, the smoke test walks through the
        // real Int declaration via `declaration_by_name`.
        let int_id = dag
            .declaration_by_name("Int")
            .expect("Int is populated by bootstrap before the test runs")
            .id;
        let arrow_id = dag.alloc_declaration_id();
        dag.push_declaration(Declaration {
            id: arrow_id,
            name: None,
            connective: TypeConnective::Arrow {
                inputs: vec![int_id, int_id],
                output: int_id,
                body: ArrowBody::ExternalRealization(instance_id),
            },
            type_params: Vec::new(),
            phantom_params: Vec::new(),
            meta_tag: None,
            specialization_parent: None,
            inhabits: None,

            value_body: None,
            refinement: None,
            span,
        });

        arrow_id
    }

    #[test]
    fn smoke_int_add_external_realization() {
        let mut dag = Dag::new();
        let arrow_id = inject_test_realization(&mut dag);

        let arrow_decl = dag.declaration(arrow_id);
        let (inputs, output, body) = match &arrow_decl.connective {
            TypeConnective::Arrow {
                inputs,
                output,
                body,
            } => (inputs.clone(), *output, body.clone()),
            other => panic!("expected realization arrow, got {other:?}"),
        };
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0], output);
        assert!(
            arrow_decl.name.is_none(),
            "realization arrow is anonymous so it stays out of declaration_by_name"
        );

        let realization_id = match body {
            ArrowBody::ExternalRealization(id) => id,
            other => panic!("expected ExternalRealization body, got {other:?}"),
        };
        let realization_decl = dag.declaration(realization_id);
        assert!(
            realization_decl.name.is_none(),
            "realization instance is anonymous"
        );
        assert!(
            matches!(realization_decl.connective, TypeConnective::Conj { .. }),
            "realization instance must be a Conj"
        );

        let meta_type_id = realization_decl
            .meta_tag
            .expect("realization instance must carry a meta_tag");
        let meta_type_decl = dag.declaration(meta_type_id);
        assert_eq!(
            meta_type_decl.name.as_deref(),
            Some("TestRealization"),
            "meta_tag points at the test-local meta-type"
        );
        assert!(
            realization_decl.inhabits.is_none(),
            "realization instance uses meta_tag only, not inhabits"
        );

        // Self-check on the AtomPayload enum so the test depends on
        // its shape (otherwise an unused import warning fires).
        let _probe: Option<&AtomPayload> = None;
    }

    #[test]
    fn malformed_pipeline_stage_attaches_diagnostic() {
        let mut dag = Dag::new();
        assert!(dag.diagnostics().is_empty(), "bootstrap should start clean");

        let parse_stage = dag
            .declaration_by_name("parse")
            .expect("parse stage present")
            .id;
        dag.declaration_mut(parse_stage).connective = TypeConnective::Conj {
            children: Vec::new(),
        };

        materialize_pipeline_realizations(&mut dag);

        assert!(
            dag.diagnostics().iter().any(|(_, diag)| matches!(
                diag,
                Diagnostic::ResolveError { name, span, .. }
                    if name.contains("pipeline stage `parse`")
                        && span.file == PIPELINE_AUTHORITY_FILE
            )),
            "malformed pipeline authority should fail closed with a diagnostic"
        );
    }

    #[test]
    fn kernel_bool_path_a_attaches_diagnostic_when_boolean_algebra_unresolvable() {
        let mut dag = Dag::new();
        assert!(
            dag.diagnostics().is_empty(),
            "production bootstrap should satisfy Path A preconditions: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        );

        let bool_id = dag
            .declaration_by_name("Bool")
            .expect("kernel Bool from std")
            .id;
        dag.declaration_mut(bool_id).inhabits = None;

        let ba_id = dag
            .declaration_by_name("BooleanAlgebra")
            .expect("BooleanAlgebra from std")
            .id;
        dag.declaration_mut(ba_id).name = Some("__test_hidden_BooleanAlgebra".to_string());

        super::patch_kernel_bool_boolean_algebra_inhabits(&mut dag);

        assert!(
            dag.declaration(bool_id).inhabits.is_none(),
            "inhabits must stay unset when the patch cannot complete"
        );
        assert!(
            dag.diagnostics().iter().any(|(_, diag)| {
                matches!(
                    diag,
                    Diagnostic::ResolveError { name, .. }
                        if name.contains("Lane 1e-2b Path A") && name.contains("BooleanAlgebra")
                )
            }),
            "expected bootstrap Path A diagnostic, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        );
    }
}
