// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 2 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::unneeded_struct_pattern,  // 1
    dead_code,  // 1
    unused_imports,  // 0 -- pre-existing
)]
// cli_run.rs is this module's PARENT, and an `#![allow]` there reaches every module
// under it -- the same cascade this commit removed at the crate root, one level down.
// These are the names its roster carries that this module does not trip, restored to
// warn so `-D warnings` still judges them here. A name moves from this list to the
// allow list above only with a counted site, never silently.
#![warn(
    clippy::assertions_on_constants,
    clippy::clone_on_copy,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_str_replace,
    clippy::disallowed_macros,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_borrow,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_to_owned,
    clippy::useless_vec,
    unused_mut
)]

use super::*;
use im::HashMap;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::coproduct_reflection::{decl_facts_corpus_walk, DeclFactRaw};
use crate::module_path_index::{
    parse_module_binding, ModuleBindingOutcome, ModuleBindingRefusal, ParsedModuleBinding,
};
use crate::shared_typecheck_store::{self, SharedTypecheckCaches};
use crate::std_node::compiler_recursive_types;
use crate::std_syntax::LiteralValue;
use crate::std_types::{kernel_type_set, SourceSpan};
use crate::v1_compiler_compile;
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_env::{
    lookup_binding_by_name, lookup_type_by_name, qualified_all_but_last, symbol_index_insert,
    symbol_index_lookup, GlobalBareLookupState, SymbolIndex, TypeEnv,
};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_lookup::global_bare_callable_node;
use crate::v1_compiler_infer_method::infer_builtin_call_type;
use crate::v1_compiler_infer_sigs::{lookup_resolved_sig, ResolvedFuncEnv, ResolvedFuncSig};
use crate::v1_compiler_normalize;
use crate::v1_compiler_parse;
use crate::v1_compiler_resolve;
use crate::v1_compiler_tokenize;
use crate::v1_interpreter;
use crate::v1_interpreter::str_value;
use crate::v1_interpreter::Value;
use crate::v1_rt;
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_body, arm_pattern, authored_name_at, block_stmts,
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    empty_intern_table, empty_node_list, expr_call_func_at, expr_method_name_at, expr_var_name_at,
    field_access_base, field_access_field_at, field_init_node_name_at, field_init_node_value,
    has_child_named, inferred_to_node, intern, is_discovery_corpus_blocking_diagnostic,
    is_error_diagnostic, is_interpreter_blocking_diagnostic, let_binding_name_at, let_value,
    make_error_node, match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver,
    module_items, no_span, param_node_name_at, param_node_type_expr, Cardinality,
    CompilerDiagnostic, Connective, ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable,
    MatchPattern, NewlineIndex, Node,
};
use serde::Serialize;

/// `(hits, misses)` for the `compile_dag_diagnostic_census` memo across the whole process.
/// Report-only; no consumer branches on it. Same accounting shape, and the same reason for it,
/// as [`compile_dag_rust_emit_check_memo_counts`]: a hit count with no denominator is not a
/// measurement.
pub fn compile_dag_diagnostic_census_memo_counts() -> (u64, u64) {
    (
        COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO_HITS.load(std::sync::atomic::Ordering::Relaxed),
        COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO_MISSES.load(std::sync::atomic::Ordering::Relaxed),
    )
}

/// Memoized entry point for the census, mirroring [`compile_dag_rust_emit_check`]'s memo rather
/// than inventing a second caching discipline beside it (DESIGN §3 — the two builtins compile the
/// same program through the same pipeline and differ only in what they report, so they must not
/// disagree about when a compile may be reused).
///
/// WHY THIS EXISTS, stated as the measurement that produced it rather than as a general
/// preference: `compile_dag_rust_emit_check` was memoized and this sibling was not, so a witness
/// asking two QUESTIONS about one source paid for two full compiles of it. `neither_green_source_
/// refuses_and_neither_mis_resolves` (`test.claim.callable_candidate_ambiguity_witness`) is the
/// specimen — four census calls over two distinct sources, so half of its compiles recomputed a
/// pure function of an input already compiled in the same run. That is the DESIGN §6
/// bare-minimum-cost class ("a proven cost-shape defect is ALWAYS fixed, regardless of the
/// realized n"), and its n stopped being small: the row reached 5437ms CPU against the 5000ms
/// `required_floor_claim_cpu_safety_limit_ms`, which is a FAIL-STOP protecting the executor and
/// explicitly "never a budget, tolerance, or target" — so the admissible repair is to stop
/// recomputing, never to raise the line.
///
/// PURITY, and it is the whole reason for the guard: the memo is armed ONLY under the floor's
/// prepared-inventory snapshot and keyed on the source TOGETHER WITH that inventory's content
/// digest, because `build_module_path_index_from_witness_roots` reads those bytes and the census
/// is therefore a function of the corpus as well as of the source. Outside the guard there is no
/// snapshot, so a hit would be a claim about disk that nothing established — DESIGN's
/// cache-impurity rule (key on declared-input content), and the reason this is not simply a
/// `HashMap` on the source string.
pub fn compile_dag_diagnostic_census(source: &str) -> CompileDiagnosticCensus {
    let Some(inventory_digest) = floor_prepared_inventory_digest() else {
        return compile_dag_diagnostic_census_uncached(source);
    };
    let memo_key = {
        use crate::v1_rt::{atom_identity_hash, hash_combine};
        let h = atom_identity_hash(source.to_string());
        hash_combine(h, atom_identity_hash(inventory_digest))
    };
    if let Some(hit) =
        COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO.with(|m| m.borrow().get(&memo_key).cloned())
    {
        COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return hit;
    }
    COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO_MISSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // A MISS FILLS A SHARED ARTIFACT, and this memo's fill is the same quantity its sibling's is.
    // The memo above was landed without this bracket, so the census half of the attribution went
    // unrecorded: `record_shared_artifact_fill_cpu` was wired into `compile_dag_rust_emit_check`
    // only, and a claim that filled a CENSUS entry was still charged the whole compile. That is
    // one accounting rule with two homes, one of which does not apply it (DESIGN §3) — and it is
    // load-bearing rather than cosmetic, because the ceiling it feeds is a fail-stop: measured on
    // main run 33131296988, `test.claim.callable_candidate_ambiguity_witness
    // .neither_green_source_refuses_and_neither_mis_resolves` was the first claim to reach BOTH
    // green sources, paid both compiles, and refused the floor at 5812ms against 5000ms, while the
    // two later claims naming those same sources read them free. The number was a fact about
    // discovery order, not about the row.
    //
    // NOTHING IS EXEMPTED. As at the sibling seam, the fill is measured on the same thread clock
    // the claim loop enforces against, recorded rather than subtracted here, and reported by
    // `run_claim_measured` as its own `[floor-shared-fill]` column — the claim's marginal and fill
    // halves still sum to what it actually spent.
    let fill_started = v1_interpreter::thread_cpu_nanos();
    let fill_wall_started = std::time::Instant::now();
    let census = compile_dag_diagnostic_census_uncached(source);
    record_shared_artifact_fill_cpu(
        v1_interpreter::thread_cpu_nanos().saturating_sub(fill_started),
    );
    record_shared_artifact_fill_wall(fill_wall_started.elapsed().as_nanos());
    COMPILE_DAG_DIAGNOSTIC_CENSUS_MEMO.with(|m| m.borrow_mut().insert(memo_key, census.clone()));
    census
}

/// Host realization backing the `compile_dag_diagnostic_census` builtin: compile an in-memory
/// `.dag` program through the v1 pipeline to the Rust render target (the same pipeline
/// [`compile_dag_rust_emit_check`] uses), and report the full per-class diagnostic census the
/// compile produced.
///
/// MEASUREMENT ONLY. Nothing here judges acceptance and nothing is filtered: every diagnostic the
/// compile emitted appears, advisories included, with `blocking` carried **as data** read through
/// the existing [`compile_clean_diagnostic_is_hard`] delegation so the severity policy keeps one
/// home. Callers filter. The sibling builtin collapses this same information into a `bool`, which
/// discards class identity, severity, and every advisory — the three facts a guarantee probe needs
/// in order to state which judgment fired rather than merely that something refused.
///
/// Scope, stated so a receipt cannot claim coverage it does not have (DESIGN §4b): this is the v1
/// pipeline to the Rust render target over a synthetic single-module source — `SyntheticProgram` ×
/// `CompileAccept` × `V1Pipeline` in `GuaranteePath` axes — **with**
/// [`crate::v1_rt::with_type_ref_hit_ne_bind_measure`] armed for the nested compile (N1a). That
/// bracket is census-only: for masked, pool-present, non-authority type refs it can emit blocking
/// `UnresolvedType` while [`compile_dag_rust_emit_check`] (measure off) stays on the production
/// fail-open / `UnlistedImportUse` advisory path. Census receipts therefore must not be read as
/// production compile-clean behavior for those type positions. It observes nothing about the
/// interpreter's disposition of the same program and nothing about other emission targets.
pub(crate) fn compile_dag_diagnostic_census_uncached(source: &str) -> CompileDiagnosticCensus {
    let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::v1_rt::with_type_ref_hit_ne_bind_measure(|| {
            let module_index = build_module_path_index_from_witness_roots();
            let sources = resolve_virtual_source_with_imports("test.dag", source, &module_index);
            v1_compiler_compile::compile_sources(
                Rc::new(sources.into()),
                crate::v1_compiler_artifact::RenderTarget::Rust,
            )
        })
    }));
    let result = match compiled {
        Ok(r) => r,
        Err(_) => {
            return CompileDiagnosticCensus::NotRunnable(
                "compile_dag_diagnostic_census: the compile panicked before producing diagnostics"
                    .to_string(),
            );
        }
    };
    CompileDiagnosticCensus::Observed(compile_diagnostic_census_rows(&result.diagnostics))
}

/// Host realization backing the `compile_dag_multi_module_fixture` builtin: compile a
/// CALLER-AUTHORED SET of `.dag` modules through the v1 pipeline to the Rust render target, with
/// NO corpus roots, NO module index, and no filesystem read of any kind.
///
/// WHY THIS EXISTS BESIDE [`compile_dag_diagnostic_census`] RATHER THAN INSIDE IT. The census
/// takes ONE synthetic module and resolves its imports against
/// [`build_module_path_index_from_witness_roots`], which walks the live checkout. That makes it
/// unable to answer any question whose subject is the RELATIONSHIP BETWEEN TWO MODULES — a
/// cross-module name collision, an import that must not bind, a spelling held by both a local
/// declaration and a corpus one — because the corpus is always in the pool and the second module
/// can never be authored. Three lanes hit that wall independently before this instrument existed:
/// invalid fixtures had to be relocated out of the corpus for want of it, an ambiguity arm was
/// measured and deleted for want of it, and this repository's own DESIGN names the missing
/// multi-module compile fixture as the next-rung trigger for the acyclicity class.
///
/// CORPUS ISOLATION IS BY CONSTRUCTION, not by a flag. The supplied manifest IS the source vector
/// handed to `compile_to_resolved_with_options`; there is no code path here that consults a module
/// index, so a fixture module may reuse a corpus module's spelling and bind its OWN declaration.
/// That is the property control 5 of the witness pins, and it is what makes the instrument usable
/// for resolution questions at all.
///
/// SCOPE, stated so a receipt cannot claim coverage it does not have (DESIGN §4b): the v1 pipeline
/// to the Rust render target over an authored multi-module subject. It observes nothing about the
/// interpreter's disposition of the same program, nothing about other emission targets, and
/// nothing about corpus-grain prevalence. Unlike the census it does NOT arm
/// `with_type_ref_hit_ne_bind_measure`: the census arms it to sharpen masked type refs against a
/// corpus pool this instrument does not have, so arming it here would be a knob with no subject.
pub fn compile_dag_multi_module_fixture(
    paths: &[String],
    contents: &[String],
    entry: &str,
) -> MultiModuleCompileFixtureOutcome {
    if paths.len() != contents.len() {
        return MultiModuleCompileFixtureOutcome::InstrumentRefused {
            cause: format!(
                "compile_dag_multi_module_fixture: manifest is {} paths against {} contents; \
                 a source is a (path, content) pair and a ragged manifest names no subject",
                paths.len(),
                contents.len()
            ),
        };
    }
    if paths.is_empty() {
        return MultiModuleCompileFixtureOutcome::InstrumentRefused {
            cause: "compile_dag_multi_module_fixture: empty manifest — an empty subject compiles \
                    clean vacuously, which is could-not-measure wearing the subject's verdict"
                .to_string(),
        };
    }
    let sources: Vec<MultiModuleFixtureSource> = paths
        .iter()
        .zip(contents.iter())
        .map(|(p, c)| MultiModuleFixtureSource {
            path: p.clone(),
            content: c.clone(),
        })
        .collect();
    let mut seen: HashSet<&str> = HashSet::new();
    for s in sources.iter() {
        if s.path.trim().is_empty() {
            return MultiModuleCompileFixtureOutcome::InstrumentRefused {
                cause: "compile_dag_multi_module_fixture: a supplied source has an empty path"
                    .to_string(),
            };
        }
        if !seen.insert(s.path.as_str()) {
            return MultiModuleCompileFixtureOutcome::InstrumentRefused {
                cause: format!(
                    "compile_dag_multi_module_fixture: path '{}' supplied twice; which bytes are \
                     at that path is then undecidable and the digest would name neither",
                    s.path
                ),
            };
        }
    }
    if !sources.iter().any(|s| s.path == entry) {
        return MultiModuleCompileFixtureOutcome::InstrumentRefused {
            cause: format!(
                "compile_dag_multi_module_fixture: entry '{entry}' names no supplied source"
            ),
        };
    }
    let source_digest = multi_module_fixture_source_digest(&sources, entry);
    let compiler_digest = crate::resolved_graph_cache::transform_content_digest();
    let files: Vec<Rc<v1_compiler_compile::SourceFile>> = sources
        .iter()
        .map(|s| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: s.path.clone(),
                content: s.content.clone(),
            })
        })
        .collect();
    let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
        let module_count = match resolved.graph.clone() {
            Some(g) => g.modules.len() as i64,
            None => 0,
        };
        // Project BEFORE emit consumes the resolved graph: the ordered parameter list is exactly
        // the emit-binding (authored + resource uses + service vars) that the Rust emitter writes
        // into each function signature. Keyed by source identity so a witness never encodes
        // module_to_filename mangling.
        let emitted_rust_functions = project_emitted_rust_fn_signatures(resolved.as_ref());
        let result = v1_compiler_compile::emit_resolved_for_target(
            resolved,
            crate::v1_compiler_artifact::RenderTarget::Rust,
        );
        (module_count, emitted_rust_functions, result)
    }));
    let (module_count, emitted_rust_functions, result) = match compiled {
        Ok(r) => r,
        Err(_) => {
            return MultiModuleCompileFixtureOutcome::InstrumentRefused {
                cause: "compile_dag_multi_module_fixture: the compile panicked before producing \
                        diagnostics"
                    .to_string(),
            };
        }
    };
    let rows = compile_diagnostic_census_rows(&result.diagnostics);
    if rows.iter().any(|r| r.blocking) {
        return MultiModuleCompileFixtureOutcome::CompileRefused {
            module_count,
            diagnostics: rows,
            source_digest,
            compiler_digest,
        };
    }
    MultiModuleCompileFixtureOutcome::CompileCompleted {
        module_count,
        emitted_files: result.files.iter().map(|f| f.path.clone()).collect(),
        emitted_rust_functions,
        diagnostics: rows,
        source_digest,
        compiler_digest,
    }
}

/// Emit-binding projection for the Rust target: one row per `FnItem` / `FuncItem` in the resolved
/// registry, with `ordered_parameter_names` matching `emit_func_params` (authored params, then
/// resource-use names, then `service_var_name` for each service). Not a text parse of emitted
/// bytes — the parameter list is the one the emitter binds into the artifact.
fn project_emitted_rust_fn_signatures(
    resolved: &v1_compiler_compile::ResolvedPipelineResult,
) -> Vec<crate::cli_run::EmittedRustFnSignature> {
    use crate::v1_compiler_infer_items::ItemKind;
    use crate::v1_std_core::param_node_name_at;
    let Some(graph) = resolved.graph.as_ref() else {
        return Vec::new();
    };
    let source_indices = resolved.source_indices.clone();
    let mut rows: Vec<crate::cli_run::EmittedRustFnSignature> = Vec::new();
    for info in graph.item_registry.values() {
        match info.kind {
            ItemKind::FnItem | ItemKind::FuncItem => {
                let mut ordered: Vec<String> = Vec::new();
                for p in info.params.iter() {
                    ordered.push(param_node_name_at(p.clone(), source_indices.clone()));
                }
                for r in info.resource_names.iter() {
                    ordered.push(r.clone());
                }
                for sn in info.service_names.iter() {
                    ordered.push(crate::v1_compiler_emit_core_support::service_var_name(
                        sn.clone(),
                    ));
                }
                rows.push(crate::cli_run::EmittedRustFnSignature {
                    owner_module: info.module_name.clone(),
                    declaration_name: info.name.clone(),
                    ordered_parameter_names: ordered,
                });
            }
            ItemKind::TypeItem
            | ItemKind::DataItem
            | ItemKind::ServiceItem
            | ItemKind::OtherItem => {}
        }
    }
    rows.sort_by(|a, b| {
        (&a.owner_module, &a.declaration_name).cmp(&(&b.owner_module, &b.declaration_name))
    });
    rows
}

/// Reference-occurrence-grain binding observation over exactly one supplied source vector.
/// Occurrence discovery and resolution remain separate products in the returned carrier: callers
/// anti-join them and must not use the resolver's emissions as their own denominator.
pub fn compile_dag_reference_occurrence_binding_census(
    paths: &[String],
    contents: &[String],
    entry: &str,
) -> ReferenceOccurrenceBindingCensus {
    if paths.len() != contents.len() || paths.is_empty() {
        return ReferenceOccurrenceBindingCensus::Refused {
            cause: "reference binding census: manifest is empty or ragged".to_string(),
        };
    }
    let sources: Vec<MultiModuleFixtureSource> = paths
        .iter()
        .zip(contents.iter())
        .map(|(path, content)| MultiModuleFixtureSource {
            path: path.clone(),
            content: content.clone(),
        })
        .collect();
    let mut seen = HashSet::new();
    if sources
        .iter()
        .any(|source| source.path.trim().is_empty() || !seen.insert(source.path.clone()))
    {
        return ReferenceOccurrenceBindingCensus::Refused {
            cause: "reference binding census: every source path must be nonempty and unique"
                .to_string(),
        };
    }
    if !sources.iter().any(|source| source.path == entry) {
        return ReferenceOccurrenceBindingCensus::Refused {
            cause: format!("reference binding census: entry '{entry}' names no supplied source"),
        };
    }
    let source_digest = multi_module_fixture_source_digest(&sources, entry);
    let compiler_digest = crate::resolved_graph_cache::transform_content_digest();
    let files: Vec<Rc<v1_compiler_compile::SourceFile>> = sources
        .iter()
        .map(|source| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: source.path.clone(),
                content: source.content.clone(),
            })
        })
        .collect();
    let resolved = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        v1_compiler_compile::compile_to_resolved(Rc::new(files.into()))
    })) {
        Ok(value) => value,
        Err(_) => {
            return ReferenceOccurrenceBindingCensus::Refused {
                cause: "reference binding census: frontend panicked before producing a graph"
                    .to_string(),
            }
        }
    };
    let Some(graph) = resolved.graph.clone() else {
        return ReferenceOccurrenceBindingCensus::Refused {
            cause: "reference binding census: frontend produced no resolved graph".to_string(),
        };
    };

    use crate::std_occurrence_binding_candidates as candidates;
    use crate::std_occurrence_identity as identity;
    let mut entries = Vec::new();
    let mut declarations = Vec::new();
    let mut references = Vec::new();
    let mut module_paths = Vec::new();
    let mut exposure_rows = Vec::new();
    let mut authored_order_rows = Vec::new();
    let mut consumer_by_occurrence = std::collections::HashMap::new();
    for module in graph.modules.iter() {
        let module_path = module.type_env.module_path.clone();
        let Some(transport) = module.occurrence_transport.clone() else {
            continue;
        };
        for entry in transport.index.entries.iter() {
            entries.push(entry.clone());
            module_paths.push(Rc::new(candidates::OccurrenceModulePathRow {
                occurrence: entry.projection.occurrence,
                module_path: module_path.clone(),
            }));
            authored_order_rows.push(Rc::new(candidates::AuthoredOrderRow {
                occurrence: entry.projection.occurrence,
                ordinal: identity::AuthoredTokenOrdinal {
                    value: entry.projection.diagnostic_span.start,
                },
            }));
        }
        for declaration in transport.declarations.iter() {
            declarations.push(declaration.clone());
            exposure_rows.push(Rc::new(candidates::DeclarationExposureRow {
                occurrence: declaration.occurrence,
                exposure: candidates::declaration_exposure_from_containment(
                    module_path.clone(),
                    declaration.containment.clone(),
                    candidates::DeclarationExposureGrounding::NamespaceStructuralRootExposure,
                ),
            }));
        }
        for reference in transport.references.iter() {
            references.push(reference.clone());
            consumer_by_occurrence.insert(reference.occurrence.value, module_path.clone());
        }
    }
    let transport = Rc::new(identity::OccurrenceTransport {
        index: Rc::new(identity::OccurrenceIndex {
            entries: Rc::new(entries.into()),
        }),
        declarations: Rc::new(declarations.into()),
        references: Rc::new(references.clone().into()),
    });
    let inputs = Rc::new(candidates::OccurrenceBindingCandidateInputs {
        module_paths: Rc::new(module_paths.into()),
        exposure_rows: Rc::new(exposure_rows.into()),
        authored_order_rows: Rc::new(authored_order_rows.into()),
    });
    let index = match &*candidates::occurrence_candidate_index_build(transport.clone(), inputs) {
        candidates::OccurrenceCandidateIndexBuild::OccurrenceCandidateIndexReady { index } => {
            index.clone()
        }
        other => {
            return ReferenceOccurrenceBindingCensus::Refused {
                cause: format!("reference binding census: candidate index refused: {other:?}"),
            }
        }
    };
    let names: std::collections::HashMap<i64, String> = transport
        .index
        .entries
        .iter()
        .map(|entry| {
            (
                entry.projection.occurrence.value,
                entry.projection.authored_name.clone(),
            )
        })
        .collect();
    let mut denominator = Vec::new();
    let mut observations = Vec::new();
    let mut ordinals_by_file: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for reference in references {
        // BOTH LOOKUPS ARE TOTAL BY CONSTRUCTION -- every reference was pushed from a module whose
        // path was recorded in the same loop, and every occurrence in the transport carries a
        // projection with an authored name. A miss therefore means the walk changed under this
        // instrument, and a sentinel module or an empty spelling would enter the denominator as a
        // FABRICATED row: exactly the class this census refuses to publish (DESIGN section 5 -- a
        // failure arm must refuse, never widen). So the miss stops the line and says which
        // occurrence it was, rather than being absorbed into a row that reads as an observation.
        let Some(consumer_module) = consumer_by_occurrence
            .get(&reference.occurrence.value)
            .cloned()
        else {
            return ReferenceOccurrenceBindingCensus::Refused {
                cause: format!(
                    "reference binding census: occurrence {} is in the references view with no \
                     recorded consumer module; the walk that fills both changed under this instrument",
                    reference.occurrence.value
                ),
            };
        };
        let Some(authored_name) = names.get(&reference.occurrence.value).cloned() else {
            return ReferenceOccurrenceBindingCensus::Refused {
                cause: format!(
                    "reference binding census: occurrence {} is in the references view with no \
                     entry in the occurrence index, so it has no authored spelling",
                    reference.occurrence.value
                ),
            };
        };
        let file_reference_ordinal = *ordinals_by_file
            .entry(reference.diagnostic_span.file.clone())
            .and_modify(|n| *n += 1)
            .or_insert(0);
        let base = ReferenceOccurrenceDenominatorRow {
            occurrence: reference.occurrence.value,
            consumer_file: reference.diagnostic_span.file.clone(),
            consumer_module: consumer_module.clone(),
            authored_name: authored_name.clone(),
            category: reference.category,
            file_reference_ordinal,
            span_start: reference.diagnostic_span.start,
        };
        denominator.push(base.clone());
        let candidate_ids =
            candidates::candidate_occurrence_ids_for_reference(index.clone(), reference.clone());
        let disposition = match &*candidates::resolve_reference_via_structural_candidates(
            index.clone(),
            reference.clone(),
        ) {
            candidates::ReferenceBindingProjection::ReferenceBindingProjectionBound {
                provider,
            } => {
                let binding_source = if authored_name.contains('.') {
                    UnlistedImportBindingSource::DefinerResolvable
                } else {
                    // A MISSING CONSUMER MODULE MUST NOT DECIDE THIS. `unwrap_or_default()` here
                    // yielded an EMPTY import list, which makes `listed` false, which stamps the row
                    // PoolCoincidence -- a fabricated semantic disposition produced by a failed
                    // lookup and indistinguishable in the output from an observed one. That is the
                    // absorbing fallback of DESIGN section 5 at its most expensive, because this
                    // exact field is what the census reports about. The module is in the same graph
                    // the reference was walked from, so a miss is a broken invariant, not a case.
                    let Some(module) = graph
                        .modules
                        .iter()
                        .find(|module| module.type_env.module_path == consumer_module)
                    else {
                        return ReferenceOccurrenceBindingCensus::Refused {
                            cause: format!(
                                "reference binding census: consumer module '{consumer_module}' \
                                 carries occurrence {} but is absent from the resolved graph, so \
                                 its import list cannot decide ListedImport against PoolCoincidence",
                                reference.occurrence.value
                            ),
                        };
                    };
                    let listed = import_module_paths_for_typed_module(module)
                        .contains(&provider.provider_module);
                    if listed {
                        UnlistedImportBindingSource::ListedImport
                    } else {
                        UnlistedImportBindingSource::PoolCoincidence
                    }
                };
                ReferenceOccurrenceBindingDisposition::Bound {
                    declaration_occurrence: provider.declaration_occurrence.value,
                    provider_module: provider.provider_module.clone(),
                    binding_source,
                }
            }
            candidates::ReferenceBindingProjection::ReferenceBindingProjectionUnbound {
                ..
            } => ReferenceOccurrenceBindingDisposition::Unresolved,
            candidates::ReferenceBindingProjection::ReferenceBindingProjectionAmbiguous {
                ..
            } => ReferenceOccurrenceBindingDisposition::Ambiguous {
                candidates: candidate_ids
                    .iter()
                    .map(|candidate| candidate.value)
                    .collect(),
            },
            other => ReferenceOccurrenceBindingDisposition::Refused {
                cause: format!("{other:?}"),
            },
        };
        observations.push(ReferenceOccurrenceBindingRow {
            denominator: base,
            disposition,
        });
    }
    ReferenceOccurrenceBindingCensus::Observed {
        source_digest,
        compiler_digest,
        denominator,
        observations,
    }
}

/// `(hits, misses)` for the `compile_dag_rust_emit_check` memo across the whole process.
/// Report-only; no consumer branches on it.
pub fn compile_dag_rust_emit_check_memo_counts() -> (u64, u64) {
    (
        COMPILE_DAG_RUST_EMIT_CHECK_MEMO_HITS.load(std::sync::atomic::Ordering::Relaxed),
        COMPILE_DAG_RUST_EMIT_CHECK_MEMO_MISSES.load(std::sync::atomic::Ordering::Relaxed),
    )
}

pub(crate) fn compile_dag_rust_emit_check_memo_key(
    source: &str,
    file_path: &str,
    includes: &[String],
    excludes: &[String],
    inventory_digest: &str,
) -> String {
    use crate::v1_rt::{atom_identity_hash, hash_combine};
    let mut h = atom_identity_hash(source.to_string());
    h = hash_combine(h, atom_identity_hash(file_path.to_string()));
    for s in includes {
        h = hash_combine(h, atom_identity_hash(s.clone()));
    }
    for s in excludes {
        h = hash_combine(h, atom_identity_hash(s.clone()));
    }
    h = hash_combine(h, atom_identity_hash(inventory_digest.to_string()));
    h
}

/// Host realization backing the `compile_dag_rust_emit_check` builtin: compile an in-memory
/// `.dag` program to Rust and check that the named emitted file contains every string in
/// `includes` and none of `excludes`, with zero **compile-clean hard** diagnostics
/// (`compile_clean_diagnostic_is_hard` — the same authority as the CI compile-clean gate).
/// Advisory diagnostics (including `WhereRefinementUnenforced` deferrals) do not fail this
/// check. A real, green-by-execution consumer of the v1 Rust emitter (DESIGN §5) — not a
/// re-derivation of the emitter's own formula, so it can go red on a real emission regression.
pub fn compile_dag_rust_emit_check(
    source: &str,
    file_path: &str,
    includes: &[String],
    excludes: &[String],
) -> bool {
    // Memo only under the floor guard, keyed on declared inputs AND the prepared
    // inventory digest (`build_module_path_index_from_witness_roots` reads those bytes).
    // Outside the guard there is no snapshot, so a hit would lie about disk.
    let Some(inventory_digest) = floor_prepared_inventory_digest() else {
        return compile_dag_rust_emit_check_uncached(source, file_path, includes, excludes);
    };
    let memo_key = compile_dag_rust_emit_check_memo_key(
        source,
        file_path,
        includes,
        excludes,
        &inventory_digest,
    );
    if let Some(hit) = COMPILE_DAG_RUST_EMIT_CHECK_MEMO.with(|m| m.borrow().get(&memo_key).copied())
    {
        COMPILE_DAG_RUST_EMIT_CHECK_MEMO_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return hit;
    }
    COMPILE_DAG_RUST_EMIT_CHECK_MEMO_MISSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // A MISS FILLS A SHARED ARTIFACT. Measured on the same thread clock the claim loop enforces
    // against, so the two quantities cannot drift apart, and recorded rather than subtracted here
    // — the claim loop does the split, this only says how much of the cost was a fill.
    let fill_started = v1_interpreter::thread_cpu_nanos();
    let fill_wall_started = std::time::Instant::now();
    let verdict = compile_dag_rust_emit_check_uncached(source, file_path, includes, excludes);
    record_shared_artifact_fill_cpu(
        v1_interpreter::thread_cpu_nanos().saturating_sub(fill_started),
    );
    record_shared_artifact_fill_wall(fill_wall_started.elapsed().as_nanos());
    COMPILE_DAG_RUST_EMIT_CHECK_MEMO.with(|m| m.borrow_mut().insert(memo_key, verdict));
    verdict
}

pub(crate) fn compile_dag_rust_emit_check_uncached(
    source: &str,
    file_path: &str,
    includes: &[String],
    excludes: &[String],
) -> bool {
    let module_index = build_module_path_index_from_witness_roots();
    let sources = resolve_virtual_source_with_imports("test.dag", source, &module_index);
    let result = v1_compiler_compile::compile_sources(
        Rc::new(sources.into()),
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    let hard_diagnostics = result
        .diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
        .count();
    if hard_diagnostics != 0 {
        false
    } else {
        match result.files.iter().find(|f| f.path == file_path) {
            Some(f) => {
                includes.iter().all(|n| f.content.contains(n.as_str()))
                    && excludes.iter().all(|n| !f.content.contains(n.as_str()))
            }
            None => false,
        }
    }
}

pub(crate) fn emit_source_root_entry_admission_data(
    admission: &SourceRootEntryAdmission,
) -> String {
    format!(
        "data host_compiler_closure_admission: Admission = Admission {{\n  subject: ResolutionSubject {{\n    name: {}\n  }},\n  imports: {}\n}}\n\n\n",
        free_monoid_symbol_emit_dag(&admission.subject),
        emit_import_admission_list(&admission.imports)
    )
}

pub(crate) fn emit_source_content_hash_dag_for_text(source: &str) -> String {
    let digest = crate::v1_rt::atom_identity_hash(source.to_string());
    format!("Fnv1a64(Fnv1a64Structural {{ digest: \"{digest}\" }})")
}

pub(crate) fn emit_source_ref_dag(rec: &SourceRootReadRecord) -> Result<String, String> {
    let path = dag_manifest_scalar_escape(&rec.file_path)?;
    let hash = emit_source_content_hash_dag_for_text(&rec.source);
    Ok(format!(
        "SourceRef {{ path: \"{path}\", source_root: {}, content_hash: {hash} }}",
        rec.source_root
    ))
}

pub(crate) fn emit_source_ref_list_dag(records: &[SourceRootReadRecord]) -> Result<String, String> {
    let mut nodes: Vec<String> = records
        .iter()
        .map(emit_source_ref_dag)
        .collect::<Result<_, _>>()?;
    let mut out = String::from("Empty");
    while let Some(head) = nodes.pop() {
        out = format!("Cons {{\n  head: {head},\n  tail: {out}\n}}");
    }
    Ok(out)
}

pub fn emit_source_ref_dag_for_path(
    records: &[SourceRootReadRecord],
    file_path: &str,
) -> Result<String, String> {
    let rec = records
        .iter()
        .find(|r| r.file_path.replace('\\', "/") == file_path.replace('\\', "/"))
        .ok_or_else(|| format!("emit_source_ref_dag_for_path: no record for {file_path}"))?;
    emit_source_ref_dag(rec)
}

pub(crate) fn emit_source_root_ref_import(records: &[SourceRootReadRecord]) -> String {
    let mut variants: Vec<&str> = records.iter().map(|r| r.source_root.as_str()).collect();
    variants.sort_unstable();
    variants.dedup();
    if variants.is_empty() {
        return String::new();
    }
    format!(
        "import v2.std.cross_tree.import_model {{ {} }}\n",
        variants.join(", ")
    )
}

/// Emit the module-binding manifest: the host handler for the `.dag`-modeled op
/// `v2.compiler.source_authority.module_storage_bindings_for_source_roots`.
///
/// This is a TRANSPORT of that modeled op, not a rival authority. It carries zero
/// independent policy: it serializes the same parse-derived rows as `build_module_path_index`
/// via `collect_module_binding_manifest_rows` (shared `for_each_parsed_module_binding` walk),
/// which is the one host producer the module-identity design says must be repointed —
/// so supplying the rows and repointing the producer are the same motion.
///
/// Rows are `ParsedFromSource`: `build_module_path_index` routes through
/// `v1_compiler_parse::parse` (src/v1/stage0/src/module_path_index), the bootstrap
/// parse path — not `extract_module_path` substring scan (task 4 repoint).
///
/// Unlike the source-root ingest manifest this carries NO source text — the binding needs
/// module <-> path only. That is what lets it scale past `MANIFEST_INLINE_LIST_MAX`, which
/// exists to stop the ingest manifest from inlining the corpus.
///
/// Dissolve-on: host-effect emission (witness-realization lane), at which point this
/// handler is emitted from the `.dag` model instead of hand-written here.
pub fn emit_module_storage_binding_manifest(
    path: &Path,
    source_roots: &[String],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let mut rows = collect_module_binding_manifest_rows(source_roots);
    rows.sort_by(|a, b| a.module_path.cmp(&b.module_path));

    let mut out = String::new();
    out.push_str("module v2.test.workflow.host_module_binding_manifest\n\n\n");
    out.push_str("import v2.compiler.source_authority {\n");
    out.push_str("  ModuleStorageIndex,\n");
    out.push_str("  module_storage_parsed_binding\n");
    out.push_str("}\n");
    out.push_str("import v2.std.artifact { Artifact, SourceFile }\n");
    out.push_str("import std.algebra { Cons, Empty }\n");
    out.push_str("import v2.std.integer { Int }\n");
    out.push_str("import v2.std.provenance { span_index_empty }\n");
    out.push_str("import v2.std.qualified_name { qualified_name_from_string_segments }\n");
    out.push_str(&emit_module_binding_source_root_import(&rows));
    out.push('\n');
    out.push_str(&format!(
        "data host_module_binding_count: Int = {}\n\n\n",
        rows.len()
    ));
    out.push_str("data host_module_bindings: ModuleStorageIndex = ");
    out.push_str(&emit_module_binding_monoid(&rows)?);
    out.push('\n');

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

/// Import exactly the `SourceRootRef` constructors the rows reference (mirrors
/// `emit_source_root_ref_import`; an unreferenced constructor import is an unlisted-import
/// error, and a referenced-but-unimported one fails to resolve).
pub(crate) fn emit_module_binding_source_root_import(rows: &[ModuleBindingManifestRow]) -> String {
    let mut names: Vec<&str> = rows.iter().map(|r| r.root_variant.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    if names.is_empty() {
        return String::new();
    }
    format!(
        "import v2.std.cross_tree.import_model {{ {} }}\n",
        names.join(", ")
    )
}

/// Render a dotted module path as a `QualifiedName`, via the std construction authority
/// `qualified_name_from_string_segments`.
///
/// Deliberately NOT `^segment` symbol literals: module segments may collide with `.dag`
/// keywords (`v2.test.claim.compiler.pipeline.corpus` emits `^pipeline`, which is a parse
/// error), and the `^(...)` form is discriminant sugar with different semantics, not an
/// escape hatch. Going through the std helper takes segments as STRINGS, so keywords are
/// inert, and it reuses the one construction authority instead of hand-rolling a second
/// spelling of the same value (DESIGN.md §3).
pub(crate) fn emit_module_binding_qualified_name(module_path: &str) -> Result<String, String> {
    let segments: Vec<&str> = module_path.split('.').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err(
            "module-binding manifest: empty module path (cannot render QualifiedName)".to_string(),
        );
    }
    let rendered: Vec<String> = segments
        .iter()
        .map(|s| dag_manifest_scalar_escape(s).map(|e| format!("\"{e}\"")))
        .collect::<Result<_, _>>()?;
    Ok(format!(
        "qualified_name_from_string_segments(segments: [{}])",
        rendered.join(", ")
    ))
}

/// THE HOST TRANSPORT HAS NO OCCURRENCE IDENTITY TO OFFER, SO IT OFFERS NONE.
///
/// This emitted a one-entry `SpanIndex` keyed on an `OccurrenceId` derived from the ident
/// span's byte offset (`start.max(1)`). `std.occurrence_identity` `occurrence_identity_scope_law`
/// names `SourceSpan` a FORBIDDEN identity input, and the derivation additionally collided:
/// offsets 0 and 1 both produced 1.
///
/// Traced to its consumers, the fabricated id was WRITE-ONLY, which is why it stood. Each row
/// built its own `span_index_empty()` and recorded exactly one entry, so the collision could not
/// manifest within a row; `span_index_merge`'s only production caller is `v2.compiler.02_parse`
/// over parser-ALLOCATED ids, never over this manifest's; `span_index_lookup` has only test
/// callers; and the one executing consumer, the module-binding supply gate, compares
/// `(file_path -> module)` and discards the field by pattern in
/// `v2.compiler.source_authority` `module_storage_binding_file_path`.
///
/// So this was not a defect with a victim. It was a forbidden identity input written into a
/// COMMITTED artifact whose innocence rested entirely on no consumer ever reading it -- the
/// inverse of correctness by construction (DESIGN §5), and one consumer away from becoming real.
///
/// THE LOCUS GOES WITH IT, AND THAT IS THE HONEST TRADE RATHER THAN A LOSS. `SpanIndex.entries`
/// is a `Map<OccurrenceId, OriginEvent>`, so the `ByteRange` this used to carry is reachable
/// ONLY under a key. A producer holding a locus but no allocator identity therefore cannot
/// record the locus without inventing the key -- the carrier makes the honest state
/// representable only as EMPTY. Emitting empty is that honest state; inventing a key to keep a
/// byte range no consumer reads is fabricated plausible output. The carrier gap is named here
/// rather than papered over, and it is what a future locus-carrying transport must close first.
pub(crate) fn emit_module_binding_span_index() -> String {
    String::from("span_index_empty()")
}

pub(crate) fn emit_module_binding_row(row: &ModuleBindingManifestRow) -> Result<String, String> {
    let qn = emit_module_binding_qualified_name(&row.module_path)?;
    let artifact_id = source_root_ingest_artifact_id_for_path(&row.rel_path);
    let span_index = emit_module_binding_span_index();
    Ok(format!(
        "module_storage_parsed_binding(\n  module: {qn},\n  artifact: Artifact {{\n    kind: SourceFile,\n    id: {artifact_id},\n    file_path: \"{}\"\n  }},\n  span_index: {span_index},\n  source_root: {}\n)",
        dag_manifest_scalar_escape(&row.rel_path)?,
        row.root_variant
    ))
}

pub(crate) fn emit_module_binding_monoid(
    rows: &[ModuleBindingManifestRow],
) -> Result<String, String> {
    let mut nodes: Vec<String> = rows
        .iter()
        .map(emit_module_binding_row)
        .collect::<Result<_, _>>()?;
    let mut out = String::from("Empty");
    while let Some(head) = nodes.pop() {
        out = format!("Cons {{\n  head: {head},\n  tail: {out}\n}}");
    }
    Ok(out)
}

pub fn emit_source_root_ingest_manifest(
    path: &Path,
    records: &[SourceRootReadRecord],
    entry_admission: Option<&SourceRootEntryAdmission>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create manifest parent {:?}: {}", parent, e))?;
    }

    let content_hash = source_root_ingest_content_hash_fnv1a64(records);
    let read_count = records.len();

    let mut out = String::new();
    out.push_str("module v2.test.workflow.host_source_root_ingest_manifest\n\n\n");
    out.push_str("import v2.compiler.source_authority {\n");
    out.push_str("  DagSourceReadWitness,\n");
    out.push_str("  DiscoveredSourceRefsDigestFromList,\n");
    out.push_str("  SourceRef,\n");
    out.push_str("  SourceRootIngest,\n");
    out.push_str("  SourceRootCoverageComplete,\n");
    out.push_str("  SourceRootManifestAbsent,\n");
    out.push_str("  SourceRootManifestElided,\n");
    out.push_str("  SourceRootProvenanceCoverageReceipt\n");
    out.push_str("}\n");
    out.push_str("import extdeps.communication.medium { Lossless, Medium }\n");
    out.push_str("import std.content_hash { ContentHash, Fnv1a64, Fnv1a64Structural }\n");
    out.push_str("import v2.std.algebra { Cons, Empty }\n");
    out.push_str("import v2.std.artifact { Artifact, SourceFile }\n");
    out.push_str("import v2.std.collection { List }\n");
    out.push_str("import v2.std.text { String }\n");
    // Each DagSourceReadWitness carries a grounded `source_root: SourceRootRef` (V2Tree/DagTree,
    // #5473/#5486), so the manifest must import the constructors it references or every witness
    // fails with `undefined variable 'V2Tree'` (the source_root ingest gate's persistent RED).
    // #6269's emit_source_root_ref_import derives exactly the referenced constructors from the
    // records (supersedes the earlier hardcoded-both-constructors form).
    if !records.is_empty() {
        out.push_str(&emit_source_root_ref_import(records));
    }
    if entry_admission.is_some() {
        out.push_str("import v2.compiler.name_resolve {\n");
        out.push_str("  Admission,\n");
        out.push_str("  Import,\n");
        out.push_str("  ImportVisible,\n");
        out.push_str("  ResolutionSubject\n");
        out.push_str("}\n");
        out.push_str("import v2.std.algebra { Cons, Empty }\n");
        out.push_str("import v2.std.collection { List }\n");
    }
    out.push('\n');
    out.push_str(&format!(
        "data host_source_root_ingest_content_hash: String = \"{}\"\n\n\n",
        dag_manifest_scalar_escape(&content_hash)?
    ));
    out.push_str("data host_source_root_ingest_coverage_receipt: SourceRootProvenanceCoverageReceipt = SourceRootProvenanceCoverageReceipt {\n");
    // Capless closure transport: closure-ref rows are always uncapped. Past
    // MANIFEST_INLINE_LIST_MAX the inline Lossless carrier is refused via
    // SourceRootManifestElided (typed expected/observed/capacity) — never zero
    // produced rows with a positive read count (empty-observation narrow).
    let produced_row_count = read_count;
    out.push_str(&format!("  ingest_read_count: {read_count},\n"));
    out.push_str(&format!("  produced_row_count: {produced_row_count},\n"));
    out.push_str(&format!(
        "  discovered_source_refs_digest: DiscoveredSourceRefsDigestFromList {{ digest: Fnv1a64Structural {{ digest: \"{}\" }} }},\n",
        source_ref_list_structural_digest_hex(records)
    ));
    if read_count > MANIFEST_INLINE_LIST_MAX {
        out.push_str(&format!(
            "  coverage: SourceRootManifestElided {{ read_count: {read_count}, cap: {MANIFEST_INLINE_LIST_MAX} }}\n"
        ));
    } else if read_count > 0 {
        out.push_str("  coverage: SourceRootCoverageComplete\n");
    } else {
        out.push_str("  coverage: SourceRootManifestAbsent\n");
    }
    out.push_str("}\n\n\n");
    out.push_str("data host_source_root_ingest: SourceRootIngest = Empty\n");
    if !records.is_empty() {
        out.push('\n');
        out.push_str("data host_source_root_closure_refs: List<SourceRef> = ");
        out.push_str(&emit_source_ref_list_dag(records)?);
        out.push('\n');
    }
    if let Some(admission) = entry_admission {
        out.push('\n');
        out.push_str(&emit_source_root_entry_admission_data(admission));
    }

    std::fs::write(path, out).map_err(|e| format!("failed to write manifest {:?}: {}", path, e))
}

pub(crate) fn transport_script_body_arg_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    let args = crate::v1_compiler_infer::call_args_by_name(node.clone(), source_indices.clone());
    v1_rt::map_get(&args, "body".to_string())
}

pub(crate) fn transport_script_arg_node(
    node: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<Rc<Node>> {
    for arg in method_arg_nodes(node.clone()).iter() {
        if arg_name_at(arg.clone(), source_indices.clone()).as_deref() == Some("script") {
            return Some(arg_value(arg.clone()));
        }
    }
    None
}

pub(crate) fn transport_script_facts_for_function_body(
    rel_path: &str,
    function: &str,
    body: &Rc<Node>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Vec<TransportScriptPositionFactRaw> {
    let mut bindings = HashMap::new();
    if let ExprData::ExprBlock { .. } = body.expr_data.as_ref() {
        collect_let_bindings_in_block_transport_script(body, &mut bindings, source_indices);
    }
    let mut facts = Vec::new();
    walk_transport_script_expr(body, &bindings, source_indices, &mut |shape| {
        facts.push(TransportScriptPositionFactRaw {
            path: rel_path.to_string(),
            function: function.to_string(),
            shape: shape.as_symbol(),
        });
    });
    facts
}

pub fn transport_script_position_facts_for_path(
    path: String,
) -> Vec<TransportScriptPositionFactRaw> {
    let (items, source_indices) = parse_module_items_for_transport_script(&path);
    let mut facts = Vec::new();
    for item in items.iter() {
        let kind = item_kind(item.clone());
        if !matches!(kind, ItemKind::FuncItem | ItemKind::FnItem) {
            continue;
        }
        let Some(body) = item.body.as_ref() else {
            continue;
        };
        facts.extend(transport_script_facts_for_function_body(
            &path,
            &item.name,
            body,
            &source_indices,
        ));
    }
    facts
}
