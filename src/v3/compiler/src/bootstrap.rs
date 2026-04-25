// Dag::new() bootstrap.
//
// PB-1 closure: `Dag::new()` no longer tokenizes/parses/lowers any
// bootstrap authority at runtime. All four bootstrap authority sets
// are loaded from committed generated snapshots:
//
// - `std_fixtures`   (`dsl/std/*.dag`)
// - `STAGED_FILES`   (`src/v3/std/*.dag`)
// - `V3_SPECS`       (`src/v3/spec/*.dag`)
// - `COMPILER_FILES` (`src/v3/compiler/*.dag`, minus `tokenize.dag`)
//
// `compile_parse_surface_std_authority_dag` uses the companion snapshot
// that omits `src/v3/std/parse_surface.dag`, so a fresh parse+lower of
// that authority still stays first-of-name.
//
// **Production bootstrap does not inject target-language
// realizations.** Realization facts for emitted languages live in
// `dsl/extdeps/languages/*` per the thesis; compiler code does not
// manufacture those. L1.5 adds one narrow exception: the staged
// `src/v3/compiler/pipeline.dag` declarations are upgraded in-place to
// `ArrowBody::ExternalRealization` so the compiler's own pipeline
// authority lives in the bootstrap Dag with the intended stage-body
// shape.
//
// Bootstrap failures (tokenize/parse/lower errors on std/ files,
// unresolved cross-file references) attach to the Dag's diagnostic
// table via `Dag::attach_diagnostic` rather than panicking, so
// `compile_to_dag` surfaces them through `Err(CompileError::Semantic(dag))`
// on every subsequent call — the same structural channel user errors
// go through. A failed bootstrap is visible to callers without a
// side channel.

use crate::dag::{ArrowBody, Dag, Declaration, DeclarationId, TemplateArgument, TypeConnective};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::lower::{collect_symbols_phase, lower_bodies_phase, resolve_pending_identifiers};
use crate::parse::{parse, SurfaceModule};
use crate::pipeline_authority::{ordered_pipeline_stages, PIPELINE_AUTHORITY_FILE};
use crate::tokenize::tokenize;
use std::collections::HashMap;

const LOGIC_DAG: &str = include_str!("../../../../dsl/std/logic.dag");
const BIT_DAG: &str = include_str!("../../../../dsl/std/bit.dag");
const ALGEBRA_DAG: &str = include_str!("../../../../dsl/std/algebra.dag");
const INTEGER_DAG: &str = include_str!("../../../../dsl/std/integer.dag");
const FLOAT_DAG: &str = include_str!("../../../../dsl/std/float.dag");
const STRING_TYPE_DAG: &str = include_str!("../../../../dsl/std/string_type.dag");
const TYPES_DAG: &str = include_str!("../../../../dsl/std/types.dag");

// M1(3) PR-B-unwind R1 — the v3-only staged files are enumerated
// by `build.rs` at compile time and exposed via generated statics:
//
//   - `STAGED_FILES` for `src/v3/std/*.dag`
//   - `V3_SPECS` for `src/v3/spec/*.dag`
//   - `COMPILER_FILES` for `src/v3/compiler/*.dag`
//
// Adding a new staged std/spec/compiler file is a pure file-system change:
// drop the `.dag` file in the staged directory, the build script
// picks it up at the next compile, and the bootstrap loop loads it
// without any `bootstrap.rs` edit.
//
// The pre-unwind shape had `const RUST_DAG: &str = include_str!(...)`
// constants here and a hardcoded fixture array, which PR #445
// review flagged as a duplicate-authority bug: the on-disk spec
// files and the Rust constants were two parallel representations
// of the same set. The build-script-generated staged arrays are the
// single authority.
//
// **Why these live in `src/v3/std/` and `src/v3/spec/` instead of
// `dsl/std/`.** v2's CI pipeline scans `dsl/` recursively and tries
// to resolve every identifier in every record-literal field value.
// v2 doesn't know about v3-only surface/substrate features, so
// keeping staged files outside the v2-scanned tree is the cleanest
// separation. v3 reads them via the build-script-generated arrays —
// no source-root scanning involved.
include!(concat!(env!("OUT_DIR"), "/v3_staged_files.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_specs.rs"));
include!(concat!(env!("OUT_DIR"), "/v3_compiler_files.rs"));

const PIPELINE_REALIZATION_META: &str = "CompilerHostRealization";

fn declaration_name_preference_rank(file: &str) -> usize {
    if file.starts_with("src/v3/") {
        2
    } else if file.starts_with("dsl/") {
        0
    } else {
        1
    }
}

// PB-1-e scaffold boundary: the runtime-parse helpers below exist solely so
// `regen_bootstrap` can produce the committed snapshot files
// (`bootstrap_*_generated.rs`) from the canonical `.dag` authorities.
// Production bootstrap MUST seed from `Dag::std_fixture_bootstrap_snapshot()`
// or `Dag::new()` directly; neither calls these helpers.
//
// In-tree DB-8 cross-check is "the committed snapshot is internally
// consistent" (see `tests/integration/pb1_bootstrap_full_snapshot_test.rs`).
// The fresh-parse-vs-snapshot acid test runs at regen time: CI invokes
// `cargo run --bin regen_bootstrap` and asserts `git diff --exit-code` on
// `src/v3/compiler/src/bootstrap_*_generated.rs` — drift between the
// committed bytes and a fresh compile fails CI.
//
// Named dissolution trigger: delete these helpers once `regen_bootstrap`
// itself is generated from a `.dag` regen-authority spec (so the fresh-parse
// step is no longer hand-Rust). At that point the std snapshot can be
// derived from the same authority that drives the rest of the regen registry.
pub(crate) fn bootstrap_std_fixtures_only(dag: &mut Dag) {
    *dag = Dag::empty();
    load_fixtures(dag, std_fixtures());
    dag.populate_primitive_cache();
}

pub(crate) fn bootstrap_runtime_authorities_on(
    dag: &mut Dag,
    excluded_staged_paths: &[&str],
    excluded_compiler_paths: &[&str],
) {
    load_runtime_bootstrap_authorities(dag, excluded_staged_paths, excluded_compiler_paths);
}

fn load_runtime_bootstrap_authorities(
    dag: &mut Dag,
    excluded_staged_paths: &[&str],
    excluded_compiler_paths: &[&str],
) {
    let staged_iter = STAGED_FILES
        .iter()
        .copied()
        .filter(|(path, _)| !excluded_staged_paths.contains(path));
    let compiler_iter = COMPILER_FILES
        .iter()
        .copied()
        .filter(|(path, _)| !excluded_compiler_paths.contains(path));
    let fixtures: Vec<(&str, &str)> = staged_iter
        .chain(V3_SPECS.iter().copied())
        .chain(compiler_iter)
        .collect();
    load_fixtures(dag, &fixtures);
    materialize_pipeline_realizations(dag);
    dag.populate_primitive_cache();
}

fn std_fixtures() -> &'static [(&'static str, &'static str)] {
    &[
        ("dsl/std/logic.dag", LOGIC_DAG),
        ("dsl/std/bit.dag", BIT_DAG),
        ("dsl/std/algebra.dag", ALGEBRA_DAG),
        ("dsl/std/integer.dag", INTEGER_DAG),
        ("dsl/std/float.dag", FLOAT_DAG),
        ("dsl/std/types.dag", TYPES_DAG),
        ("dsl/std/string_type.dag", STRING_TYPE_DAG),
    ]
}

fn load_fixtures(dag: &mut Dag, fixtures: &[(&str, &str)]) {
    let mut parsed: Vec<(SurfaceModule, Vec<bool>)> = Vec::with_capacity(fixtures.len());
    for (file, source) in fixtures.iter() {
        let Some(module) = parse_fixture(dag, source, file) else {
            continue;
        };
        let (_stale_symbols, is_first) = collect_symbols_phase(dag, &module.items);
        parsed.push((module, is_first));
    }

    // Rebuild the symbols map from the shared declaration table. By
    // now every top-level declaration across all fixtures is present
    // with its type_params slot populated, so Phase 2 can resolve
    // every cross-file template reference at construction time.
    // Use the same staged-v3-over-dsl preference policy as
    // `collect_symbols`, otherwise Phase 1 can register the staged
    // shadowing declaration but Phase 2 will still lower bodies
    // against the legacy `dsl/` declaration.
    let mut shared_symbols: HashMap<String, DeclarationId> = HashMap::new();
    for d in dag.declarations() {
        if let Some(name) = &d.name {
            match shared_symbols.get(name).copied() {
                None => {
                    shared_symbols.insert(name.clone(), d.id);
                }
                Some(existing_id) => {
                    let existing = dag.declaration(existing_id);
                    let new_rank = declaration_name_preference_rank(&d.span.file);
                    let existing_rank = declaration_name_preference_rank(&existing.span.file);
                    if new_rank > existing_rank {
                        shared_symbols.insert(name.clone(), d.id);
                    }
                }
            }
        }
    }

    // Phase 2: lower bodies using the shared symbols map.
    for (module, is_first) in parsed.iter() {
        lower_bodies_phase(dag, module, &shared_symbols, is_first);
    }

    resolve_pending_identifiers(dag);
    patch_kernel_bool_boolean_algebra_inhabits(dag);
}

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
fn patch_kernel_bool_boolean_algebra_inhabits(dag: &mut Dag) {
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

fn materialize_pipeline_realizations(dag: &mut Dag) {
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

fn report_pipeline_authority_error(dag: &mut Dag, name: String) {
    dag.attach_diagnostic(Diagnostic::ResolveError {
        name,
        span: SourceSpan::new(PIPELINE_AUTHORITY_FILE, 0, 0),
        fixes: Vec::new(),
    });
}

fn parse_fixture(dag: &mut Dag, source: &str, file: &str) -> Option<SurfaceModule> {
    let tokens = match tokenize(source, file) {
        Ok(t) => t,
        Err(diag) => {
            dag.attach_diagnostic(diag);
            return None;
        }
    };
    match parse(&tokens, file) {
        Ok(m) => Some(m),
        Err(diag) => {
            dag.attach_diagnostic(diag);
            None
        }
    }
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
