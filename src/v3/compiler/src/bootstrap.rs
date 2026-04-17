// Dag::new() bootstrap.
//
// Parses the production `dsl/std/*.dag` files in dependency order and
// lowers them into the freshly-created Dag so that the declaration table
// is primed with primitive types and algebraic structures before any
// user code runs. The seven files are embedded via `include_str!` so
// bootstrap is hermetic at runtime and the declaration table stays in
// sync with the `.dag` source at build time.
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

use crate::dag::{ArrowBody, Dag, DeclarationId, FieldValue, TypeConnective, ValueBody};
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

pub(crate) fn bootstrap(dag: &mut Dag) {
    // Two-phase loading across all seven std/ files. Phase 1 parses and
    // `collect_symbols_phase`s every file, allocating top-level
    // declarations + their TypeParam children in one batch. Phase 2
    // fills in each file's bodies, at which point every cross-file
    // template reference (e.g., `bit.dag`'s `Word64 { bytes: List<Byte> }`
    // where `List` is declared in `types.dag`) finds its template's
    // `type_params` slot already populated — no half-valid template
    // arguments, no post-sweep fixup pass.
    //
    // Load order within each phase: `logic` → `bit` (needs Classical)
    // → `algebra` (no deps) → `integer`/`float` (need algebra + bit)
    // → `types` (needs integer for Int64) → `string_type` (needs
    // Char from types; the sweep resolves the cross-file forward ref).
    // Standard library fixtures — shared with v2. The set is
    // hardcoded because dsl/std/ is the v3 substrate's sibling
    // tree and adding new std/ files is a coordinated change
    // that goes through both compilers.
    let std_fixtures: &[(&str, &str)] = &[
        ("dsl/std/logic.dag", LOGIC_DAG),
        ("dsl/std/bit.dag", BIT_DAG),
        ("dsl/std/algebra.dag", ALGEBRA_DAG),
        ("dsl/std/integer.dag", INTEGER_DAG),
        ("dsl/std/float.dag", FLOAT_DAG),
        ("dsl/std/types.dag", TYPES_DAG),
        ("dsl/std/string_type.dag", STRING_TYPE_DAG),
    ];

    // v3-only staged fixtures — enumerated by build.rs at compile
    // time from `src/v3/std/*.dag`, `src/v3/spec/*.dag`, and
    // `src/v3/compiler/*.dag`. Adding a new staged file is a pure
    // file-system change.
    let fixtures: Vec<(&str, &str)> = std_fixtures
        .iter()
        .copied()
        .chain(STAGED_FILES.iter().copied())
        .chain(V3_SPECS.iter().copied())
        .chain(COMPILER_FILES.iter().copied())
        .collect();

    // Phase 0: parse every fixture. Tokenize/parse errors attach to
    // `dag.diagnostics()` and the corresponding module is omitted
    // from later phases.
    //
    // Phase 1: per-file `collect_symbols_phase` runs inline with the
    // parse loop so every file's declarations + type_params land in
    // the shared `dag` before ANY body lowering runs. The per-file
    // symbols map is captured but discarded — it's stale by the end
    // of Phase 1 because later files' declarations aren't in it.
    // Phase 2 uses a REBUILT shared symbols map below.
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

    // Batch-final resolution for cross-file forward references. In
    // bootstrap mode the sweep tolerates dangling stubs — the canonical
    // std/ files reference types that live in modules outside the
    // M1(2.6) load set (e.g., `Tuple`), and those are not bootstrap
    // errors. User-code compilation uses the strict variant.
    resolve_pending_identifiers(dag);
    materialize_pipeline_realizations(dag);
    materialize_substrate_accessors(dag);

    // Cache the canonical role declarations (Int, Bool, String,
    // Realization) now that every std/ module has been lowered and the
    // resolution sweep has linked cross-file references. Downstream
    // consumers ask `dag.int_shape()` / `dag.realization_meta_id()`
    // etc. instead of running a name scan per call.
    dag.populate_primitive_cache();
}

const SUBSTRATE_ACCESSOR_BINDING_TYPE: &str = "SubstrateAccessorBinding";
const SUBSTRATE_ACCESSOR_REALIZATION_META: &str = "SubstrateAccessorRealization";

/// DB-14 bootstrap upgrade: walk every `SubstrateAccessorBinding`
/// data record and upgrade the bound accessor fn's Arrow body from
/// `Unparsed` (the trivial `{ host X }` stub) to
/// `ExternalRealization(realization_id)`. After this pass, emission
/// dispatches on the upgraded body and renders the realization's
/// `carrier` template. Pattern mirrors `materialize_pipeline_realizations`;
/// see `docs/design-substrate-external-primitives.md`.
fn materialize_substrate_accessors(dag: &mut Dag) {
    let Some(binding_type_id) = dag
        .declaration_by_name(SUBSTRATE_ACCESSOR_BINDING_TYPE)
        .map(|decl| decl.id)
    else {
        return;
    };

    let mut pairs: Vec<(DeclarationId, DeclarationId, String)> = Vec::new();
    for declaration in dag.declarations() {
        if declaration.meta_tag != Some(binding_type_id) {
            continue;
        }
        let binding_name = declaration
            .name
            .as_deref()
            .unwrap_or("<anonymous SubstrateAccessorBinding>")
            .to_string();
        let Some(ValueBody::Structural { fields }) = &declaration.value_body else {
            report_substrate_accessor_error(
                dag,
                format!("substrate accessor binding `{binding_name}` must carry a structural value body"),
            );
            return;
        };
        let accessor = match require_substrate_accessor_ref(fields, "accessor", &binding_name) {
            Ok(id) => id,
            Err(err) => {
                report_substrate_accessor_error(dag, err);
                return;
            }
        };
        let realization = match require_substrate_accessor_ref(fields, "realization", &binding_name)
        {
            Ok(id) => id,
            Err(err) => {
                report_substrate_accessor_error(dag, err);
                return;
            }
        };
        pairs.push((accessor, realization, binding_name));
    }

    let Some(meta_decl_id) = dag
        .declaration_by_name(SUBSTRATE_ACCESSOR_REALIZATION_META)
        .map(|decl| decl.id)
    else {
        if !pairs.is_empty() {
            report_substrate_accessor_error(
                dag,
                format!(
                    "missing substrate accessor realization meta `{SUBSTRATE_ACCESSOR_REALIZATION_META}`"
                ),
            );
        }
        return;
    };

    // The realization meta must lower to a Conj; the data items that
    // instantiate it carry `TypeConnective::Instantiation { template }`.
    // `is_realization_shape` in infer.rs checks for Conj directly, so we
    // replace each realization's connective with the meta's Conj before
    // upgrading the accessor Arrow body. Same shape as
    // `materialize_pipeline_realizations`.
    let meta_connective = match dag.declaration(meta_decl_id).connective.clone() {
        connective @ TypeConnective::Conj { .. } => connective,
        _ => {
            report_substrate_accessor_error(
                dag,
                format!(
                    "substrate accessor realization meta `{SUBSTRATE_ACCESSOR_REALIZATION_META}` must lower to a record"
                ),
            );
            return;
        }
    };

    for (accessor, realization, binding_name) in pairs {
        let realization_decl = dag.declaration_mut(realization);
        if realization_decl.meta_tag != Some(meta_decl_id) {
            report_substrate_accessor_error(
                dag,
                format!(
                    "substrate accessor binding `{binding_name}` realization is not tagged with `{SUBSTRATE_ACCESSOR_REALIZATION_META}`"
                ),
            );
            continue;
        }
        realization_decl.connective = meta_connective.clone();
        let accessor_decl = dag.declaration_mut(accessor);
        match &mut accessor_decl.connective {
            TypeConnective::Arrow { body, .. } => {
                *body = ArrowBody::ExternalRealization(realization);
            }
            _ => report_substrate_accessor_error(
                dag,
                format!(
                    "substrate accessor binding `{binding_name}` target is not an Arrow declaration"
                ),
            ),
        }
    }
}

fn require_substrate_accessor_ref(
    fields: &[(String, FieldValue)],
    label: &str,
    binding_name: &str,
) -> Result<DeclarationId, String> {
    fields
        .iter()
        .find(|(field, _)| field == label)
        .and_then(|(_, value)| match value {
            FieldValue::Reference(id) => Some(*id),
            _ => None,
        })
        .ok_or_else(|| {
            format!(
                "substrate accessor binding `{binding_name}` is missing required DeclarationRef field `{label}`"
            )
        })
}

fn report_substrate_accessor_error(dag: &mut Dag, name: String) {
    dag.attach_diagnostic(Diagnostic::ResolveError {
        name,
        span: SourceSpan::new("src/v3/std/substrate.dag", 0, 0),
    });
}

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
                Diagnostic::ResolveError { name, span }
                    if name.contains("pipeline stage `parse`")
                        && span.file == PIPELINE_AUTHORITY_FILE
            )),
            "malformed pipeline authority should fail closed with a diagnostic"
        );
    }
}
