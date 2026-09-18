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
/// realized n"), and its n stopped being small: the row reached 5437ms CPU against the 5000ms CPU
/// safety deadline standing at the time, a FAIL-STOP protecting the executor and explicitly "never
/// a budget, tolerance, or target" — so the admissible repair is to stop recomputing, never to
/// raise the line. That deadline is gone (the claim ceiling gates on
/// `claim_eval_step_budget_for_identity` since 2026-09-12 and CPU is observed-only), which does
/// NOT retire this memo: the defect it repairs is a recomputation, and a recomputation costs the
/// same whether or not a clock refuses on it.
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

/// BUILD A CLAIM SCOPE over a caller-authored fixture manifest, and report what the scope
/// builder said.
///
/// The manifest validation is deliberately the SAME shape as
/// `compile_dag_multi_module_fixture`'s, because the failure modes it rejects are properties of
/// a manifest and not of what one later does with it.
///
/// `entry_module_path` is a MODULE PATH (`amb.user`), not a file path: `claim_scope_for` is
/// keyed on the module whose scope is being built, which is what the floor passes it.
pub fn claim_scope_dag_multi_module_fixture(
    paths: &[String],
    contents: &[String],
    entry_module_path: &str,
) -> crate::cli_run::ClaimScopeFixtureOutcome {
    use crate::cli_run::ClaimScopeFixtureOutcome as Outcome;
    if paths.len() != contents.len() {
        return Outcome::InstrumentRefused {
            cause: format!(
                "claim_scope_dag_multi_module_fixture: manifest is {} paths against {} contents; \
                 a source is a (path, content) pair and a ragged manifest names no subject",
                paths.len(),
                contents.len()
            ),
        };
    }
    if paths.is_empty() {
        return Outcome::InstrumentRefused {
            cause: "claim_scope_dag_multi_module_fixture: empty manifest — an empty subject \
                    builds a scope over nothing, which is could-not-measure wearing the \
                    subject's verdict"
                .to_string(),
        };
    }
    let mut seen: HashSet<&str> = HashSet::new();
    for path in paths.iter() {
        if path.trim().is_empty() {
            return Outcome::InstrumentRefused {
                cause: "claim_scope_dag_multi_module_fixture: a supplied source has an empty path"
                    .to_string(),
            };
        }
        if !seen.insert(path.as_str()) {
            return Outcome::InstrumentRefused {
                cause: format!(
                    "claim_scope_dag_multi_module_fixture: path '{path}' supplied twice; which \
                     bytes are at that path is then undecidable"
                ),
            };
        }
    }
    let files: Vec<Rc<v1_compiler_compile::SourceFile>> = paths
        .iter()
        .zip(contents.iter())
        .map(|(path, content)| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        v1_compiler_compile::compile_to_resolved(Rc::new(files.into()))
    }));
    let resolved = match compiled {
        Ok(resolved) => resolved,
        Err(_) => {
            return Outcome::InstrumentRefused {
                cause: "claim_scope_dag_multi_module_fixture: the compile panicked before \
                        producing a graph"
                    .to_string(),
            };
        }
    };
    let rows = compile_diagnostic_census_rows(&resolved.diagnostics);
    let Some(graph) = resolved.graph.clone() else {
        return Outcome::CompileRefused { diagnostics: rows };
    };
    if rows.iter().any(|row| row.blocking) {
        return Outcome::CompileRefused { diagnostics: rows };
    }
    let module_count = graph.modules.len() as i64;
    let source_indices = v1_compiler_compile::dag_graph_source_indices(graph.clone());
    // THE SAME CARRIER THE FLOOR PASSES. Every field is supplied from this manifest rather than
    // from a corpus: `full_inventory` and `discovery_exclusions` are empty because a fixture has
    // no discovery step, and the counts describe this manifest. No corpus root is read, which is
    // what keeps the subject under the calling test's control.
    // A REAL DIGEST, NOT AN EMPTY STRING. The per-subject memos are keyed by this value, so an
    // empty digest is not merely uninformative -- every fixture would collide with every other
    // fixture and with anything else that left the field blank, and a scope would consume an
    // index built from a different graph.
    let subject_digest = multi_module_fixture_source_digest(
        &paths
            .iter()
            .zip(contents.iter())
            .map(|(path, content)| MultiModuleFixtureSource {
                path: path.clone(),
                content: content.clone(),
            })
            .collect::<Vec<MultiModuleFixtureSource>>(),
        entry_module_path,
    );
    let prepared = crate::cli_run::PreparedRepository {
        graph,
        source_indices,
        subject_digest,
        modules_resolved: module_count as usize,
        modules_excluded: 0,
        full_inventory: Vec::new(),
        discovery_exclusions: HashMap::new(),
    };
    // THE FIXTURE BUILDS ITS OWN REFERENCE-CLOSURE INDEX AND REGISTERS NOTHING.
    //
    // `reference_closure_index` memoizes in a `thread_local!` keyed by `subject_digest` and
    // bounded at `FLOOR_PREPARED_SUBJECTS_PER_PROCESS`; a subject beyond that population is
    // refused. The required floor already holds both slots -- the corpus subject and the
    // `policy_prepared` subject built for `REQUIRED_FLOOR_POLICY_MODULE` -- so a fixture going
    // through the cache is the THIRD subject and is refused, for a reason that says nothing about
    // the manifest. This subject is one module and is discarded immediately, so it has no business
    // in a cache sized for the floor's own long-lived subjects; it builds its index directly and
    // hands it to scope construction, occupying no slot and evicting nothing.
    //
    // RAISING THE BOUND IS NOT THE REMEDY. It is a stated production cost wall, and widening it so
    // a test instrument fits is the instrument dictating production limits.
    //
    // WHY THIS IS NOT A TUNING DETAIL. The memo is thread-LOCAL and the floor evaluates claims
    // across several workers, so an instrument that reaches it is order- and thread-dependent BY
    // CONSTRUCTION: which worker picks a claim up can decide its verdict, and a local run holding
    // only one subject cannot see that at all. Anyone building another floor-resident instrument
    // should read that before trusting a green. The receipt is the required floor's own
    // `required-witnesses-floor` job on this branch, whose control rows re-derive it; it is named
    // rather than transcribed, because a copied observation rots without anyone touching either
    // end (DESIGN §6).
    //
    // The Err that survives here is `ExprVarReconciliationMismatch`, which IS about the supplied
    // graph, so it is reported as a scope refusal rather than an instrument one.
    let reference_index = match crate::cli_run::build_reference_closure_index(&prepared) {
        Ok(index) => index,
        Err(cause) => return Outcome::ScopeRefused { cause },
    };
    // WITHOUT MEMOS, deliberately: `claim_scope_for` reaches per-subject memo caches bounded at
    // `FLOOR_PREPARED_SUBJECTS_PER_PROCESS`, and a fixture subject -- synthesized here, one
    // module wide, discarded immediately -- must neither read from nor write to them, because
    // occupying one of that bounded population would refuse the next real subject. So this calls the SAME
    // `claim_scope_for_with_memos` the floor's entry point calls, passing `None` for the fragment
    // cache and a scope-private order index. That is a caching decision rather than a semantic
    // one: it is one function computing one scope, which is what keeps this instrument and the
    // corpus floor on a single detector.
    //
    // `claim_scope_for_without_memos` is the obvious spelling and is NOT used: it is gated behind
    // `cfg(any(test, feature = "interp_test_witness"))`, so a release build has no such function
    // and this instrument has to run in one.
    let built = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::cli_run::claim_scope_for_with_memos(
            &prepared,
            entry_module_path,
            None,
            crate::cli_run::build_scope_order_index(&prepared),
            Some(reference_index),
        )
    }));
    match built {
        Err(_) => Outcome::InstrumentRefused {
            cause: "claim_scope_dag_multi_module_fixture: scope construction panicked".to_string(),
        },
        Ok(Err(cause)) => Outcome::ScopeRefused { cause },
        Ok(Ok(_scope)) => Outcome::ScopeAccepted { module_count },
    }
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
        // Resolved-registry projection BEFORE emit consumes the graph. Subject grain: ItemInfo
        // parameter binding (with emit_ident / service_var_name transforms), keyed by source
        // identity — not emitted file bytes. Emit-path fidelity is the declared next-rung climb.
        // Walk per TypedModule registry. Bare Fn/Func names that collide across modules are
        // omitted (Absent) — never InstrumentRefused, never an un-expanded loser row.
        let resolved_rust_functions = project_resolved_rust_fn_signatures(resolved.as_ref());
        let result = v1_compiler_compile::emit_resolved_for_target(
            resolved,
            crate::v1_compiler_artifact::RenderTarget::Rust,
        );
        (module_count, resolved_rust_functions, result)
    }));
    let (module_count, resolved_rust_functions, result) = match compiled {
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
        resolved_rust_functions,
        diagnostics: rows,
        source_digest,
        compiler_digest,
    }
}

/// Resolved-registry projection for the Rust emit target: one row per `FnItem` in
/// each `TypedModule.item_registry` (not the bare-name-merged `ResolvedGraph.item_registry`).
/// `ordered_parameter_names` applies `emit_ident(..., Rust)` on authored params and resource-use
/// names, then `service_var_name` per service, concatenated in that order — the same per-arm
/// transforms `emit_func_params` uses today. Not a text parse of emitted bytes.
///
/// Rows are keyed `(owner_module, declaration_name)`. When the same bare Fn/Func name appears in
/// two or more modules, those rows are **omitted** (lookup → `ResolvedRustFnAbsent`) — not an
/// instrument refusal, and not a fallback to the un-expanded per-module `ItemInfo`.
///
/// Why omit: `expand_transitive_services` writes only the bare-name-merged `graph.item_registry`,
/// and `emit_func_def` resolves services via `lookup_item` on that same merged map (no module
/// check). Publishing the loser's un-expanded local row would report missing `service_names`
/// while the emitter still binds the survivor's services — a silent wrong answer (§5). Both
/// present/absent consumers already treat `Absent` as false on both polarities, so omission is
/// fail-closed. Service-name overlay for non-colliding names: use the expanded graph row when it
/// still names this module under that bare name.
///
/// Below ceiling on ORDER and MEMBERSHIP (§3b middle value — deliberate divergence with stated
/// reason on `ResolvedRustFnSignature`): second walk over ItemInfo, not a consumption of
/// `emit_func_params`. Resource arm reads `info.resource_names`; emit folds `uses` via
/// `resource_use_name_at` — nothing refuses on disagreement. Next-rung trigger: derive from the
/// same source `emit_func_params` reads (or from its emit result). Why unbuilt: emit_rust seed
/// regen would couple this instrument to #10688's live surface. Name spelling via `emit_ident` /
/// `service_var_name` is required at this grain so membership cannot miss a reserved-word or
/// camelCase name the registry arms will hand to emit.
fn project_resolved_rust_fn_signatures(
    resolved: &v1_compiler_compile::ResolvedPipelineResult,
) -> Vec<crate::cli_run::ResolvedRustFnSignature> {
    use crate::v1_compiler_artifact::RenderTarget;
    use crate::v1_compiler_emit::emit_ident;
    use crate::v1_compiler_infer_items::ItemKind;
    use crate::v1_std_core::param_node_name_at;
    use std::collections::HashMap;
    let Some(graph) = resolved.graph.as_ref() else {
        return Vec::new();
    };
    let source_indices = resolved.source_indices.clone();
    // Bare Fn/Func name → how many TypedModules publish it. Count > 1 ⇒ omit every row for
    // that name (see doc above); never publish the un-expanded loser.
    let mut bare_fn_module_count: HashMap<String, usize> = HashMap::new();
    for typed in graph.modules.iter() {
        for local in typed.item_registry.values() {
            if matches!(local.kind, ItemKind::FnItem) {
                *bare_fn_module_count.entry(local.name.clone()).or_insert(0) += 1;
            }
        }
    }
    let mut rows: Vec<crate::cli_run::ResolvedRustFnSignature> = Vec::new();
    for typed in graph.modules.iter() {
        for local in typed.item_registry.values() {
            match local.kind {
                ItemKind::FnItem => {
                    if bare_fn_module_count.get(&local.name).copied().unwrap_or(0) > 1 {
                        continue;
                    }
                    // Prefer the expanded graph registry row when it still names this module —
                    // transitive service expansion is applied there, not on TypedModule.item_registry.
                    // The graph registry is keyed on the DECLARATION IDENTITY (owner.decl), not on
                    // the bare leaf: a leaf spelled the same in two modules occupies two rows, so a
                    // bare-name read here missed every time and silently took the unexpanded local
                    // row, dropping transitive service_names at this seam.
                    let identity = crate::v1_std_core::callable_identity(std::rc::Rc::new(
                        crate::v1_std_core::DeclaredCallableIdentity {
                            owner_module_path: local.module_name.clone(),
                            decl_name: local.name.clone(),
                        },
                    ));
                    let info = match graph.item_registry.get(&identity) {
                        Some(expanded) if expanded.module_name == local.module_name => expanded,
                        _ => local,
                    };
                    let mut ordered: Vec<String> = Vec::new();
                    for p in info.params.iter() {
                        ordered.push(emit_ident(
                            param_node_name_at(p.clone(), source_indices.clone()),
                            RenderTarget::Rust,
                        ));
                    }
                    for r in info.resource_names.iter() {
                        ordered.push(emit_ident(r.clone(), RenderTarget::Rust));
                    }
                    for sn in info.service_names.iter() {
                        ordered.push(crate::v1_compiler_emit_core_support::service_var_name(
                            sn.clone(),
                        ));
                    }
                    rows.push(crate::cli_run::ResolvedRustFnSignature {
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
    }
    rows.sort_by(|a, b| {
        (&a.owner_module, &a.declaration_name).cmp(&(&b.owner_module, &b.declaration_name))
    });
    rows
}

fn resolved_call_edges_from_graph(
    graph: &v1_compiler_compile::ResolvedGraph,
) -> Vec<crate::cli_run::ResolvedCallEdgeRow> {
    use crate::cli_run::ResolvedCallEdgeRow;
    use crate::v1_compiler_infer_service::{build_module_callees, CalleeEdge};
    let mut edges = Vec::new();
    for callees in build_module_callees(graph.modules.clone()).iter() {
        for item in callees.items.iter() {
            for edge in item.called.iter() {
                if let CalleeEdge::ResolvedCallee { identity } = &**edge {
                    edges.push(ResolvedCallEdgeRow {
                        caller_module: item.item_identity.owner_module_path.clone(),
                        caller_decl: item.item_identity.decl_name.clone(),
                        callee_module: identity.owner_module_path.clone(),
                        callee_decl: identity.decl_name.clone(),
                    });
                }
            }
        }
    }
    edges
}

pub fn compile_dag_importer_resolved_call_edges(
    import_modules: &[String],
    exclude_substrings: &[String],
    pool_roots: &[String],
    target_leaves: &[String],
) -> crate::cli_run::ResolvedCallEdgeCensus {
    compile_dag_candidate_resolved_call_edges(
        import_modules,
        exclude_substrings,
        pool_roots,
        target_leaves,
        false,
    )
}

pub fn compile_dag_callsite_resolved_call_edges(
    import_modules: &[String],
    exclude_substrings: &[String],
    pool_roots: &[String],
    target_leaves: &[String],
) -> crate::cli_run::ResolvedCallEdgeCensus {
    compile_dag_candidate_resolved_call_edges(
        import_modules,
        exclude_substrings,
        pool_roots,
        target_leaves,
        true,
    )
}

fn token_is_trivia(token: &crate::v1_std_core::Token) -> bool {
    matches!(
        token.shape,
        crate::v1_std_core::TokenShape::ShNewline | crate::v1_std_core::TokenShape::ShEof
    )
}

fn dag_ident_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Broad raw prefilter only: exact target-leaf bytes at an identifier boundary.
/// Not a call-form detector. `//` annotations sit between a leaf and `(` in source
/// bytes while the tokenizer elides them and treats newline as trivia, so a `(`
/// proximity test here is a false-negative class. `first_call_form_leaf` remains
/// the sole call-form authority after this filter admits a file to tokenize.
fn content_has_target_leaf_at_ident_boundary(content: &str, leaves: &HashSet<String>) -> bool {
    let bytes = content.as_bytes();
    for leaf in leaves {
        let needle = leaf.as_bytes();
        if needle.is_empty() {
            continue;
        }
        let mut i = 0;
        while i + needle.len() <= bytes.len() {
            if &bytes[i..i + needle.len()] != needle {
                i += 1;
                continue;
            }
            let before_ok = i == 0 || !dag_ident_continue(bytes[i - 1]);
            let after = i + needle.len();
            let after_ok = after == bytes.len() || !dag_ident_continue(bytes[after]);
            if before_ok && after_ok {
                return true;
            }
            i += 1;
        }
    }
    false
}

/// Production horizon roots are witness-layer names (`dag`, `src/v2`): callers may import
/// across the admitted universe, so resolve uses `default_source_roots()`. A fixture pool
/// is not a layer root; resolving it through the live-tree universe rebuilds a corpus
/// index for a closed handful of files.
fn resolve_roots_for_call_edge_pool(pool_roots: &[String]) -> Vec<String> {
    if pool_roots.is_empty() {
        return default_source_roots();
    }
    let layers = crate::cli_run::witness_layer_roots();
    let horizon = pool_roots
        .iter()
        .all(|root| layers.iter().any(|layer| layer == root));
    if horizon {
        default_source_roots()
    } else {
        pool_roots_abs(pool_roots)
    }
}

fn first_call_form_leaf(
    content: &str,
    rel: &str,
    leaves: &HashSet<String>,
) -> Result<Option<String>, String> {
    let tokens = crate::v1_compiler_tokenize::tokenize(
        content.to_string(),
        rel.to_string(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    if tokens
        .iter()
        .any(|token| token.shape == crate::v1_std_core::TokenShape::ShUnknown)
    {
        return Err(format!(
            "call-form leaf guard: {rel}: tokenizer produced an unknown token"
        ));
    }
    let tokens: Vec<Rc<crate::v1_std_core::Token>> = tokens.iter().cloned().collect();
    let mut i = 0;
    while i < tokens.len() {
        if token_is_trivia(&tokens[i]) {
            i += 1;
            continue;
        }
        if let Some(ident) = token_ident(&tokens[i]) {
            if leaves.contains(ident) {
                let mut j = i + 1;
                while j < tokens.len() && token_is_trivia(&tokens[j]) {
                    j += 1;
                }
                if tokens
                    .get(j)
                    .map(|token| token.shape == crate::v1_std_core::TokenShape::ShLParen)
                    .unwrap_or(false)
                {
                    return Ok(Some(ident.to_string()));
                }
            }
        }
        i += 1;
    }
    Ok(None)
}

/// Fail-closed call-form candidate guard over a declared pool. Tokenizes every admitted `.dag`
/// under `pool_roots`; a leaf identifier followed by `(` after trivia is a candidate. Does not
/// resolve. Empty candidate set is `ProductionCoverageQualified`; a hit is
/// `CandidateOutsideExactResolution`; unreadable or untokenizable input is `ProductionCoverageRefused`.
pub fn compile_dag_call_form_leaf_guard(
    exclude_substrings: &[String],
    pool_roots: &[String],
    target_leaves: &[String],
    exact_resolved_roots: &[String],
) -> crate::cli_run::EvaluationStoreAddressProductionCoverage {
    use crate::cli_run::EvaluationStoreAddressProductionCoverage;
    if pool_roots.iter().any(|r| r.trim().is_empty()) || pool_roots.is_empty() {
        return EvaluationStoreAddressProductionCoverage::Refused {
            root: pool_roots.first().cloned().unwrap_or_default(),
            path: String::new(),
            cause: "call-form leaf guard: every scan root must be nonempty".to_string(),
        };
    }
    if target_leaves.iter().any(|l| l.trim().is_empty()) || target_leaves.is_empty() {
        return EvaluationStoreAddressProductionCoverage::Refused {
            root: pool_roots[0].clone(),
            path: String::new(),
            cause: "call-form leaf guard: every target leaf must be nonempty".to_string(),
        };
    }
    let leaves: HashSet<String> = target_leaves.iter().cloned().collect();
    let abs_roots = pool_roots_abs(pool_roots);
    let mut files = Vec::new();
    for (authored, abs) in pool_roots.iter().zip(abs_roots.iter()) {
        let root_path = Path::new(abs);
        if !root_path.is_dir() {
            return EvaluationStoreAddressProductionCoverage::Refused {
                root: authored.clone(),
                path: authored.clone(),
                cause: format!(
                    "call-form leaf guard: declared root '{authored}' is not an inspectable directory"
                ),
            };
        }
        if let Err(cause) = collect_dag_files_complete(root_path, &mut files) {
            return EvaluationStoreAddressProductionCoverage::Refused {
                root: authored.clone(),
                path: authored.clone(),
                cause,
            };
        }
    }
    files.sort();
    for file in files {
        let rel = rel_path_for_layer_import(&file);
        if is_excluded_import_path(&rel, exclude_substrings) {
            continue;
        }
        let owning_root = pool_roots
            .iter()
            .zip(abs_roots.iter())
            .find(|(_, abs)| file.starts_with(abs))
            .map(|(authored, _)| authored.clone())
            .unwrap_or_else(|| pool_roots[0].clone());
        let content = match std::fs::read_to_string(&file) {
            Ok(content) => content,
            Err(e) => {
                return EvaluationStoreAddressProductionCoverage::Refused {
                    root: owning_root,
                    path: rel,
                    cause: format!("call-form leaf guard: cannot read: {e}"),
                };
            }
        };
        if !content_has_target_leaf_at_ident_boundary(&content, &leaves) {
            continue;
        }
        match first_call_form_leaf(&content, &rel, &leaves) {
            Ok(Some(target_leaf)) => {
                return EvaluationStoreAddressProductionCoverage::CandidateOutsideExactResolution {
                    root: owning_root,
                    path: rel,
                    target_leaf,
                };
            }
            Ok(None) => {}
            Err(cause) => {
                return EvaluationStoreAddressProductionCoverage::Refused {
                    root: owning_root,
                    path: rel,
                    cause,
                };
            }
        }
    }
    EvaluationStoreAddressProductionCoverage::Qualified {
        exact_resolved_roots: exact_resolved_roots.to_vec(),
        zero_candidate_roots: pool_roots.to_vec(),
    }
}

fn compile_dag_candidate_resolved_call_edges(
    import_modules: &[String],
    exclude_substrings: &[String],
    pool_roots: &[String],
    target_leaves: &[String],
    include_reference_forms: bool,
) -> crate::cli_run::ResolvedCallEdgeCensus {
    use crate::cli_run::ResolvedCallEdgeCensus;
    if import_modules.iter().any(|m| m.trim().is_empty()) {
        return ResolvedCallEdgeCensus::Refused {
            cause: "candidate resolved call edges: every home module must be nonempty".to_string(),
        };
    }
    compile_dag_candidate_resolved_call_edges_uncached(
        import_modules,
        exclude_substrings,
        pool_roots,
        target_leaves,
        include_reference_forms,
    )
}

fn collect_dag_files_complete(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        format!(
            "candidate resolved call edges: cannot inspect directory {}: {e}",
            dir.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "candidate resolved call edges: cannot read directory entry under {}: {e}",
                dir.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            if is_cargo_target_output_dir(dir, &path) {
                continue;
            }
            collect_dag_files_complete(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
    Ok(())
}

fn significant_token_shapes(
    tokens: &[Rc<crate::v1_std_core::Token>],
) -> Vec<Rc<crate::v1_std_core::Token>> {
    tokens
        .iter()
        .filter(|token| {
            !matches!(
                token.shape,
                crate::v1_std_core::TokenShape::ShNewline | crate::v1_std_core::TokenShape::ShEof
            )
        })
        .cloned()
        .collect()
}

fn token_keyword(token: &crate::v1_std_core::Token, keyword: &str) -> bool {
    token.shape == crate::v1_std_core::TokenShape::ShKeyword && token.text == keyword
}

fn token_ident(token: &crate::v1_std_core::Token) -> Option<&str> {
    match token.shape {
        crate::v1_std_core::TokenShape::ShIdent => Some(token.text.as_str()),
        _ => None,
    }
}

fn dotted_path_from(
    tokens: &[Rc<crate::v1_std_core::Token>],
    start: usize,
) -> Option<(String, usize)> {
    let first = token_ident(tokens.get(start)?)?;
    let mut path = first.to_string();
    let mut i = start + 1;
    while i + 1 < tokens.len()
        && tokens[i].shape == crate::v1_std_core::TokenShape::ShDot
        && token_ident(&tokens[i + 1]).is_some()
    {
        path.push('.');
        path.push_str(token_ident(&tokens[i + 1]).unwrap());
        i += 2;
    }
    Some((path, i))
}

struct ReferenceFormFile {
    rel: String,
    module_path: String,
    imports: Vec<String>,
    called_leaves: HashSet<String>,
    declared_names: HashSet<String>,
}

fn parse_reference_form_file(rel: &str, content: &str) -> Result<ReferenceFormFile, String> {
    let Some(module_path) = extract_module_path(content) else {
        return Err(format!(
            "candidate resolved call edges: {rel}: no module declaration"
        ));
    };
    let tokens = crate::v1_compiler_tokenize::tokenize(
        content.to_string(),
        rel.to_string(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    if tokens
        .iter()
        .any(|token| token.shape == crate::v1_std_core::TokenShape::ShUnknown)
    {
        return Err(format!(
            "candidate resolved call edges: {rel}: tokenizer produced an unknown token"
        ));
    }
    let tokens = significant_token_shapes(&tokens.iter().cloned().collect::<Vec<_>>());
    let mut imports = Vec::new();
    let mut called_leaves = HashSet::new();
    let mut declared_names = HashSet::new();
    let mut i = 0;
    while i < tokens.len() {
        if token_keyword(&tokens[i], "import") {
            if let Some((path, next)) = dotted_path_from(&tokens, i + 1) {
                imports.push(path);
                i = next;
                continue;
            }
        }
        if token_keyword(&tokens[i], "fn")
            || token_keyword(&tokens[i], "data")
            || token_keyword(&tokens[i], "type")
        {
            if let Some(name) = tokens.get(i + 1).and_then(|t| token_ident(t)) {
                declared_names.insert(name.to_string());
            }
        }
        if let Some(ident) = token_ident(&tokens[i]) {
            if tokens.get(i + 1).map(|t| t.shape) == Some(crate::v1_std_core::TokenShape::ShLParen)
            {
                called_leaves.insert(ident.to_string());
            }
        }
        i += 1;
    }
    Ok(ReferenceFormFile {
        rel: rel.to_string(),
        module_path,
        imports,
        called_leaves,
        declared_names,
    })
}

fn load_reference_form_pool(
    pool_roots: &[String],
    exclude_substrings: &[String],
    leaf_needles: &[String],
    require_leaf_bytes: bool,
) -> Result<Vec<ReferenceFormFile>, String> {
    let scan_roots = if pool_roots.is_empty() {
        default_source_roots()
    } else {
        pool_roots.to_vec()
    };
    let abs_roots = pool_roots_abs(&scan_roots);
    let mut files = Vec::new();
    for (authored, abs) in scan_roots.iter().zip(abs_roots.iter()) {
        let root_path = Path::new(abs);
        if !root_path.is_dir() {
            return Err(format!(
                "candidate resolved call edges: declared root '{authored}' is not an inspectable directory"
            ));
        }
        collect_dag_files_complete(root_path, &mut files)?;
    }
    files.sort();
    let call_form_leaves: HashSet<String> = if require_leaf_bytes {
        leaf_needles.iter().cloned().collect()
    } else {
        HashSet::new()
    };
    let mut loaded = Vec::new();
    for file in files {
        let rel = rel_path_for_layer_import(&file);
        if is_excluded_import_path(&rel, exclude_substrings) {
            continue;
        }
        let content = std::fs::read_to_string(&file).map_err(|e| {
            format!(
                "candidate resolved call edges: cannot read {}: {e}",
                file.display()
            )
        })?;
        if require_leaf_bytes
            && !call_form_leaves.is_empty()
            && !content_has_target_leaf_at_ident_boundary(&content, &call_form_leaves)
        {
            continue;
        }
        loaded.push(parse_reference_form_file(&rel, &content)?);
    }
    Ok(loaded)
}

fn compile_dag_candidate_resolved_call_edges_uncached(
    import_modules: &[String],
    exclude_substrings: &[String],
    pool_roots: &[String],
    target_leaves: &[String],
    include_reference_forms: bool,
) -> crate::cli_run::ResolvedCallEdgeCensus {
    use crate::cli_run::ResolvedCallEdgeCensus;
    let homes: HashSet<String> = import_modules.iter().cloned().collect();
    let requested_leaves: HashSet<String> = target_leaves
        .iter()
        .filter(|leaf| !leaf.is_empty())
        .cloned()
        .collect();
    let files = match load_reference_form_pool(
        pool_roots,
        exclude_substrings,
        target_leaves,
        include_reference_forms && !requested_leaves.is_empty(),
    ) {
        Ok(files) => files,
        Err(cause) => return ResolvedCallEdgeCensus::Refused { cause },
    };
    let home_leaves: HashSet<String> = if requested_leaves.is_empty() {
        files
            .iter()
            .filter(|file| homes.contains(&file.module_path))
            .flat_map(|file| file.declared_names.iter().cloned())
            .collect()
    } else {
        requested_leaves
    };
    let mut entries: HashSet<String> = HashSet::new();
    for file in &files {
        let import_hit = file.imports.iter().any(|import| homes.contains(import));
        let home_hit = homes.contains(&file.module_path);
        let call_hit = include_reference_forms
            && file
                .called_leaves
                .intersection(&home_leaves)
                .next()
                .is_some();
        let selected = if include_reference_forms {
            call_hit
        } else {
            import_hit || home_hit
        };
        if selected {
            entries.insert(file.rel.clone());
        }
    }
    if entries.is_empty() {
        return ResolvedCallEdgeCensus::Refused {
            cause: "candidate resolved call edges: no importer, home, or reference-form path"
                .to_string(),
        };
    }
    // Caller horizon (`pool_roots`) selects candidate files only. When that horizon is a
    // witness-layer root, resolution runs against the admitted live-tree universe
    // (`witness_layer_roots` → dag ∪ src/v2) through the process-shared index. Using the
    // horizon `dag` as resolve_roots under-resolved std.materialization_provider (it
    // imports v2.compiler.self_host.generation). A fixture pool is not a layer root:
    // resolve against a PRIVATE index of that subtree only. Routing fixtures through
    // process_shared_index either rebuilds a whole-corpus slot (two-slot TLS replace) or
    // typechecks the live-tree universe — the cost that made the alias census an
    // enrolment_bound_without_ceiling measurement rather than a bounded inhabitance.
    let dependency_universe = resolve_roots_for_call_edge_pool(pool_roots);
    let layers = crate::cli_run::witness_layer_roots();
    let live_tree_horizon = pool_roots.is_empty()
        || pool_roots
            .iter()
            .all(|root| layers.iter().any(|layer| layer == root));
    let mut entries: Vec<String> = entries.into_iter().collect();
    entries.sort();
    let mut edges = Vec::new();
    let mut seen: HashSet<(String, String, String, String)> = HashSet::new();
    let take_graph = |resolved: Result<(Rc<v1_compiler_compile::ResolvedGraph>, _), String>,
                      entry: &str|
     -> Result<
        Rc<v1_compiler_compile::ResolvedGraph>,
        crate::cli_run::ResolvedCallEdgeCensus,
    > {
        match resolved {
            Ok((graph, _)) => {
                let blocking = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
                    graph.diagnostics.clone(),
                );
                if !blocking.is_empty() {
                    let detail = blocking.iter().cloned().collect::<Vec<_>>().join("; ");
                    return Err(ResolvedCallEdgeCensus::Refused {
                        cause: format!(
                            "candidate resolved call edges: {entry}: blocking acquisition/import/resolution/type diagnostics: {detail}"
                        ),
                    });
                }
                Ok(graph)
            }
            Err(cause) => Err(ResolvedCallEdgeCensus::Refused {
                cause: format!("candidate resolved call edges: {entry}: {cause}"),
            }),
        }
    };
    let mut push_edges = |graph: &v1_compiler_compile::ResolvedGraph| {
        for edge in resolved_call_edges_from_graph(graph) {
            let key = (
                edge.caller_module.clone(),
                edge.caller_decl.clone(),
                edge.callee_module.clone(),
                edge.callee_decl.clone(),
            );
            if seen.insert(key) {
                edges.push(edge);
            }
        }
    };
    if live_tree_horizon {
        for entry in entries {
            match take_graph(
                resolve_entry_graph_shared(&dependency_universe, &entry),
                &entry,
            ) {
                Ok(graph) => push_edges(&graph),
                Err(refused) => return refused,
            }
        }
    } else {
        let index = crate::cli_run::build_multi_entry_index(&dependency_universe);
        for entry in entries {
            match take_graph(
                crate::cli_run::resolve_entry_with_index(&index, &entry),
                &entry,
            ) {
                Ok(graph) => push_edges(&graph),
                Err(refused) => return refused,
            }
        }
    }
    ResolvedCallEdgeCensus::Observed { edges }
}

/// THE KEYS OF THE BUILTIN REGISTRY, READ OFF THE AUTHORITY.
///
/// `v1.compiler.infer_method` `builtin_function_registry` is a `Map<String, BuiltinSignature>`
/// whose values carry `Node`s, so the interpreter cannot evaluate the `data` row itself
/// (`NoSuchField { type_name: "Node", field: "ident" }` when it tries), which is why
/// `std.primitives` `builtin_registry_surface_names` has been a HAND ROSTER beside it -- measured
/// 17 rows behind on 2026-09-18. This query reads the compiled registry's key set directly so
/// `gunbc.primitive_egress.census` derives the registry population from the one authority and
/// reports the roster's drift as a typed finding rather than inheriting it.
pub fn builtin_function_registry_keys() -> Vec<String> {
    let registry = crate::v1_compiler_infer_method::builtin_function_registry();
    let mut keys: Vec<String> = registry.keys().cloned().collect();
    keys.sort();
    keys
}

fn primitive_call_callee_of_target(
    target: &crate::v1_std_core::CallTargetIdentity,
    spelling: &str,
) -> Option<crate::cli_run::PrimitiveCalleeIdentity> {
    use crate::cli_run::PrimitiveCalleeIdentity;
    use crate::v1_std_core::CallTargetIdentity;
    match target {
        CallTargetIdentity::RuntimePrimitiveCall {
            primitive_name,
            projected_from,
        } => Some(PrimitiveCalleeIdentity::RuntimePrimitive {
            primitive_name: primitive_name.clone(),
            projected_from_module: projected_from.as_ref().map(|d| d.owner_module_path.clone()),
            projected_from_decl: projected_from.as_ref().map(|d| d.decl_name.clone()),
        }),
        CallTargetIdentity::CallableTargetUndetermined => {
            Some(PrimitiveCalleeIdentity::UndeterminedFreeCall {
                spelling: spelling.to_string(),
            })
        }
        CallTargetIdentity::SourceDeclarationCall { .. }
        | CallTargetIdentity::LocallyBoundCall { .. } => None,
    }
}

/// Walk one typed expression tree and push every call site whose callee the resolver did NOT
/// establish as a source declaration or a local binding. Those two are the only callee kinds
/// that are not the census's subject; everything else is either a primitive identity the
/// resolver minted or a site where it minted nothing, and both are rows.
fn collect_primitive_call_edges(
    texpr: &Rc<crate::v1_std_core::Node>,
    caller_module: &str,
    caller_decl: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    out: &mut Vec<crate::cli_run::PrimitiveCallEdgeRow>,
) {
    use crate::cli_run::{PrimitiveCallEdgeRow, PrimitiveCalleeIdentity};
    use crate::v1_std_core::{CallSemantics, ExprData, MethodSemantics};
    let callee: Option<PrimitiveCalleeIdentity> = match &*texpr.expr_data {
        ExprData::ExprCall { call_semantics, .. } => {
            let spelling =
                crate::v1_std_core::expr_call_func_at(texpr.clone(), source_indices.clone());
            match call_semantics.as_deref() {
                Some(CallSemantics::PlainCallSemantics { target })
                | Some(CallSemantics::ResolvedDirectCallSemantics { target, .. })
                | Some(CallSemantics::LookupCallSemantics { target }) => {
                    primitive_call_callee_of_target(target, &spelling)
                }
                Some(CallSemantics::FunctionValueCallSemantics) => None,
                None => Some(PrimitiveCalleeIdentity::UndeterminedFreeCall { spelling }),
            }
        }
        ExprData::ExprMethodCall { method_semantics } => {
            let spelling =
                crate::v1_std_core::expr_method_name_at(texpr.clone(), source_indices.clone());
            match method_semantics.as_deref() {
                Some(MethodSemantics::AlgebraMethodSemantics {
                    algebra_template, ..
                }) => match algebra_template {
                    Some(t) => Some(PrimitiveCalleeIdentity::AlgebraMethod {
                        template_name: t.name.clone(),
                    }),
                    None => Some(PrimitiveCalleeIdentity::PlainMethod { spelling }),
                },
                Some(MethodSemantics::ServiceMethodSemantics { service_name, .. }) => {
                    Some(PrimitiveCalleeIdentity::ServiceOperation {
                        service_name: service_name.clone(),
                        operation: spelling,
                    })
                }
                Some(MethodSemantics::PlainMethodSemantics) | None => {
                    Some(PrimitiveCalleeIdentity::PlainMethod { spelling })
                }
            }
        }
        _ => None,
    };
    if let Some(callee) = callee {
        let authored_spelling = match &*texpr.expr_data {
            ExprData::ExprCall { .. } => {
                crate::v1_std_core::expr_call_func_at(texpr.clone(), source_indices.clone())
            }
            _ => crate::v1_std_core::expr_method_name_at(texpr.clone(), source_indices.clone()),
        };
        out.push(PrimitiveCallEdgeRow {
            caller_module: caller_module.to_string(),
            caller_decl: caller_decl.to_string(),
            authored_spelling,
            callee,
            span_file: texpr.span.file.clone(),
            span_start: texpr.span.start,
        });
    }
    for child in texpr.children.iter() {
        collect_primitive_call_edges(child, caller_module, caller_decl, source_indices, out);
    }
    if let Some(body) = &texpr.body {
        collect_primitive_call_edges(body, caller_module, caller_decl, source_indices, out);
    }
}

fn primitive_call_edges_from_module(
    module: &TypedModule,
    out: &mut Vec<crate::cli_run::PrimitiveCallEdgeRow>,
) {
    let si = module.type_env.source_indices.clone();
    let module_name = crate::v1_std_core::authored_name_at(si.clone(), module.module.clone());
    for item in module.items.iter() {
        let decl_name = crate::v1_std_core::authored_name_at(si.clone(), item.clone());
        if let Some(body) = &item.body {
            collect_primitive_call_edges(body, &module_name, &decl_name, &si, out);
        }
        // A `data` row's initializer is a body too, but a declaration-level default argument
        // or where-clause reaches the same walk through `params`/`children`, so both are
        // covered from the item node itself rather than from a second entry point.
        for param in item.params.iter() {
            collect_primitive_call_edges(param, &module_name, &decl_name, &si, out);
        }
    }
}

/// THE PRIMITIVE-CONSUMER EDGE CENSUS: every call site in every `.dag` under `pool_roots` whose
/// resolver-established callee is a runtime primitive, an algebra method template, a service
/// operation, or NOTHING -- with the caller's declaration identity and the site's span.
///
/// Every file under the pool (minus `exclude_substrings`) is resolved as its own entry against a
/// PRIVATE index of `pool_roots`, so a census over `dag ∪ src/v2 ∪ src/v1` never replaces the
/// process-shared slot the running interpreter resolved its own program through. Each entry's
/// OWN module is walked once; modules reached only as dependencies are walked when their own
/// file is the entry, so no module is counted twice and no module in the pool is skipped. An
/// entry with blocking diagnostics is carried as a typed refusal ROW rather than aborting the
/// whole census, because the consumer (`gunbc.primitive_egress.census`) decides whether a
/// refused entry is admissible -- and it refuses when any production entry is.
pub fn compile_dag_primitive_call_edges(
    exclude_substrings: &[String],
    pool_roots: &[String],
) -> crate::cli_run::PrimitiveCallEdgeCensus {
    use crate::cli_run::{PrimitiveCallEdgeCensus, PrimitiveCallEntryRefusal};
    if pool_roots.is_empty() || pool_roots.iter().any(|r| r.trim().is_empty()) {
        return PrimitiveCallEdgeCensus::Refused {
            cause: "primitive call edges: every pool root must be nonempty and at least one is required"
                .to_string(),
        };
    }
    let abs_roots = pool_roots_abs(pool_roots);
    let mut files = Vec::new();
    for (authored, abs) in pool_roots.iter().zip(abs_roots.iter()) {
        let root_path = Path::new(abs);
        if !root_path.is_dir() {
            return PrimitiveCallEdgeCensus::Refused {
                cause: format!(
                    "primitive call edges: declared root '{authored}' is not an inspectable directory"
                ),
            };
        }
        if let Err(cause) = collect_dag_files_complete(root_path, &mut files) {
            return PrimitiveCallEdgeCensus::Refused { cause };
        }
    }
    files.sort();
    let mut entries: Vec<String> = files
        .iter()
        .map(|file| rel_path_for_layer_import(file))
        .filter(|rel| !is_excluded_import_path(rel, exclude_substrings))
        .collect();
    entries.sort();
    entries.dedup();
    if entries.is_empty() {
        return PrimitiveCallEdgeCensus::Refused {
            cause:
                "primitive call edges: no .dag entry under the pool roots survives the exclusions"
                    .to_string(),
        };
    }
    let index = crate::cli_run::build_multi_entry_index(&abs_roots);
    let mut entries_resolved = Vec::new();
    let mut entries_refused = Vec::new();
    let mut edges = Vec::new();
    let mut walked_modules: HashSet<String> = HashSet::new();
    for entry in entries {
        match crate::cli_run::resolve_entry_with_index(&index, &entry) {
            Ok((graph, _)) => {
                let blocking = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
                    graph.diagnostics.clone(),
                );
                if !blocking.is_empty() {
                    entries_refused.push(PrimitiveCallEntryRefusal {
                        entry: entry.clone(),
                        cause: blocking.iter().cloned().collect::<Vec<_>>().join("; "),
                    });
                    continue;
                }
                for module in graph.modules.iter() {
                    let module_file = module.module.span.file.clone();
                    let is_entry_module = module_file == entry
                        || rel_path_for_layer_import(Path::new(&module_file)) == entry;
                    if !is_entry_module {
                        continue;
                    }
                    let module_name = crate::v1_std_core::authored_name_at(
                        module.type_env.source_indices.clone(),
                        module.module.clone(),
                    );
                    if walked_modules.insert(module_name) {
                        primitive_call_edges_from_module(module, &mut edges);
                    }
                }
                entries_resolved.push(entry);
                // RELEASE THE ENTRY GRAPH. `resolve_entry_with_index` memoizes every
                // assembled ResolvedGraph by closure subject, and a graph strong-pins
                // every module of its closure; over a whole-corpus walk that memo alone
                // held 25 GiB (measured 2026-09-18) before the first receipt. The walk
                // reads each entry once, so the memo buys nothing here; the typed-module
                // cache (capped) is what makes the next entry cheap.
                index.resolved_graph_memo.borrow_mut().clear();
                index
                    .resolved_graph_memo_cross_process_subjects
                    .borrow_mut()
                    .clear();
            }
            Err(cause) => entries_refused.push(PrimitiveCallEntryRefusal { entry, cause }),
        }
    }
    edges.sort_by(|a, b| {
        (&a.caller_module, &a.caller_decl, &a.span_file, a.span_start).cmp(&(
            &b.caller_module,
            &b.caller_decl,
            &b.span_file,
            b.span_start,
        ))
    });
    PrimitiveCallEdgeCensus::Observed {
        entries_resolved,
        entries_refused,
        edges,
    }
}

/// Reference-occurrence-grain binding observation over exactly one supplied source vector.
/// Occurrence discovery and resolution remain separate products in the returned carrier: callers
/// anti-join them and must not use the resolver's emissions as their own denominator.
pub fn compile_dag_reference_occurrence_binding_census(
    paths: &[String],
    contents: &[String],
    entry: &str,
) -> ReferenceOccurrenceBindingCensus {
    let compiler_digest = crate::resolved_graph_cache::transform_content_digest();
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

    match observed_occurrence_transport_and_bindings(graph.as_ref()) {
        ObservedOccurrenceBindingHalves::Refused { cause } => {
            ReferenceOccurrenceBindingCensus::Refused { cause }
        }
        ObservedOccurrenceBindingHalves::Ready {
            denominator,
            observations,
            ..
        } => ReferenceOccurrenceBindingCensus::Observed {
            source_digest,
            compiler_digest,
            denominator,
            observations,
        },
    }
}

pub(crate) enum ObservedOccurrenceBindingHalves {
    Ready {
        transport: Rc<crate::std_occurrence_identity::OccurrenceTransport>,
        denominator: Vec<ReferenceOccurrenceDenominatorRow>,
        observations: Vec<ReferenceOccurrenceBindingRow>,
    },
    Refused {
        cause: String,
    },
}

/// Seed-observed occurrence transport (per-module sidecars concatenated as stored) plus the
/// occurrence-grain binding walk over that same transport. Callers must not rebuild containment.
pub(crate) fn observed_occurrence_transport_and_bindings(
    graph: &v1_compiler_compile::ResolvedGraph,
) -> ObservedOccurrenceBindingHalves {
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
            return ObservedOccurrenceBindingHalves::Refused {
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
            return ObservedOccurrenceBindingHalves::Refused {
                    cause: format!(
                    "reference binding census: occurrence {} is in the references view with no \
                     recorded consumer module; the walk that fills both changed under this instrument",
                    reference.occurrence.value
                ),
            };
        };
        let Some(authored_name) = names.get(&reference.occurrence.value).cloned() else {
            return ObservedOccurrenceBindingHalves::Refused {
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
                        return ObservedOccurrenceBindingHalves::Refused {
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
    ObservedOccurrenceBindingHalves::Ready {
        transport,
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
        if !matches!(kind, ItemKind::FnItem) {
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

/// Host body of the one XL-1 builtin `emit_rust_reference_derived_rows_bridge`.
/// Not a builtin spelling, CLI flag, or second compile entry: only that interpreter arm calls it.
#[derive(Debug, Clone)]
pub(crate) enum Xl1PrimaryRootTap {
    Refused {
        cause: String,
    },
    Observed {
        primary_root: String,
        repair_rows: Rc<im::Vector<Rc<crate::v1_compiler_emit_rust::ReferenceDerivedCandidateRow>>>,
        occurrence_transport: Rc<crate::std_occurrence_identity::OccurrenceTransport>,
        binding_rows: Vec<ReferenceOccurrenceBindingRow>,
    },
}

pub(crate) fn compile_xl1_primary_root_tap(
    source_roots: &[String],
    repository: &str,
    measured_root_demands: &str,
) -> Xl1PrimaryRootTap {
    if source_roots.is_empty() {
        return Xl1PrimaryRootTap::Refused {
            cause: "xl1 primary-root tap: source_roots is empty".to_string(),
        };
    }
    let root = source_roots[0].clone();
    // THE ROOT DEMAND IS DECLARED BY THE CALLER, exactly as `gunbc compile` receives it on
    // argv: the repository identity and the projection path are repository facts
    // (gunbc.whole_corpus_compile_admission whole_corpus_compile_repository,
    // gunbc.whole_corpus_compile_demand_projection), so the .dag caller names them and the
    // tap borrows no identity. Admission is then the SAME whole-root admission the compile
    // transaction asks, on the root's own measured row (#11265).
    let request = CompileRequest {
        subject: CompileSubject::PrimaryRoot(root.clone()),
        source_roots: source_roots.to_vec(),
        primary_precedence: false,
        render_targets: vec![crate::v1_compiler_artifact::RenderTarget::Rust],
        root_demand: RootDemandDeclaration {
            repository: Some(repository.to_string()),
            measured_root_demands: Some(measured_root_demands.to_string()),
        },
    };
    if let Err(cause) = request.primary_root_agrees_with_precedence() {
        return Xl1PrimaryRootTap::Refused { cause };
    }
    let identity = crate::memory_governor::WholeCorpusCompileRootIdentity {
        repository: repository.to_string(),
        primary_root: root.clone(),
        dependency_pools: source_roots.iter().skip(1).cloned().collect(),
    };
    let read =
        crate::memory_governor::read_whole_corpus_compile_demands(Some(measured_root_demands));
    let budget = crate::memory_governor::read_host_budget_resolution();
    let admission =
        crate::memory_governor::whole_corpus_compile_admission(&budget, &identity, &read);
    if let Some(diagnostic) =
        crate::memory_governor::whole_corpus_compile_refusal_diagnostic(&admission)
    {
        return Xl1PrimaryRootTap::Refused { cause: diagnostic };
    }
    let root_abs = if std::path::Path::new(&root).is_absolute() {
        std::path::PathBuf::from(&root)
    } else {
        process_workspace_root().join(&root)
    };
    if repo_relative_path(&root_abs).is_err() {
        return Xl1PrimaryRootTap::Refused {
            cause: format!(
                "primary source root is outside the workspace root: {} is not under {}",
                root_abs.display(),
                process_workspace_root().display()
            ),
        };
    }
    let index = match try_process_shared_index(source_roots) {
        Ok(idx) => idx,
        Err(cause) => {
            return Xl1PrimaryRootTap::Refused {
                cause: format!("xl1 primary-root tap: source-discovery: {cause}"),
            }
        }
    };
    // ONE DERIVATION OF THE PRIMARY-ROOT SUBJECT, shared with the compile transaction's
    // `CompileSubject::PrimaryRoot` arm (`cli_run.rs` `primary_root_subject_closure`). This
    // tap used to carry its own copy and the copy had drifted in the fail-open direction: it
    // had no module-less-`.dag` visibility step, so a `.dag` under the root that lost its
    // `module` header would have left the closure silently and the census would have
    // under-counted while reporting rows (review 66847 on gunbc#11461).
    let closure = match primary_root_subject_closure(&index, &root) {
        Ok(c) => c,
        Err(PrimaryRootSubjectRefusal { phase, cause }) => {
            return Xl1PrimaryRootTap::Refused {
                cause: format!("xl1 primary-root tap: {phase}: {cause}"),
            }
        }
    };
    // The pool outside the closure enters the name census only -- the SAME derivation the
    // compile transaction's PrimaryRoot arm and the required floor consume, never a copy
    // (review 67039 on gunbc#11461).
    let options = compile_clean_pipeline_options_for_sources(Some(&index), &closure);
    let resolved = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        v1_compiler_compile::compile_to_resolved_with_options(
            Rc::new(closure.clone().into()),
            options,
        )
    })) {
        Ok(value) => value,
        Err(_) => {
            return Xl1PrimaryRootTap::Refused {
                cause: "xl1 primary-root tap: frontend panicked before producing a graph"
                    .to_string(),
            }
        }
    };
    let Some(graph) = resolved.graph.clone() else {
        return Xl1PrimaryRootTap::Refused {
            cause: "xl1 primary-root tap: frontend produced no resolved graph".to_string(),
        };
    };
    let repair_rows = crate::v1_compiler_emit_rust::emit_rust_reference_derived_rows(graph.clone());
    match observed_occurrence_transport_and_bindings(graph.as_ref()) {
        ObservedOccurrenceBindingHalves::Refused { cause } => Xl1PrimaryRootTap::Refused { cause },
        ObservedOccurrenceBindingHalves::Ready {
            transport,
            observations,
            ..
        } => Xl1PrimaryRootTap::Observed {
            primary_root: root,
            repair_rows,
            occurrence_transport: transport,
            binding_rows: observations,
        },
    }
}
