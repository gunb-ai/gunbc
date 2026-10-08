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
    has_child_named, inferred_to_node, intern, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, let_binding_name_at, let_value, make_error_node,
    match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver, module_items, no_span,
    param_node_name_at, param_node_type_expr, Cardinality, CompilerDiagnostic, Connective,
    ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable, MatchPattern, NewlineIndex,
    Node,
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
        record_fixture_closure_memo_hit(&memo_key);
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
    begin_fixture_closure_fill();
    let census = compile_dag_diagnostic_census_uncached(source);
    finish_fixture_closure_fill(&memo_key);
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
            let sources =
                resolve_virtual_source_with_imports(FIXTURE_SOURCE_PATH, source, &module_index)
                    .map_err(|cause| FixtureRenderRefusal::ClosureUnresolvable { cause })?;
            record_fixture_closure(&sources);
            compile_fixture_rendering_only_what_is_read(sources, None)
        })
    }));
    let result = match compiled {
        Ok(Ok(r)) => r,
        Ok(Err(refusal)) => {
            return CompileDiagnosticCensus::NotRunnable(format!(
                "compile_dag_diagnostic_census: {refusal}"
            ));
        }
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
    let compiler_digest = crate::closure_identity::transform_content_digest();
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
                    for r in info.resource_requirements.iter() {
                        ordered.push(emit_ident(r.binding_name.clone(), RenderTarget::Rust));
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
                if let CalleeEdge::ResolvedCallee { identity, .. } = &**edge {
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
/// `std.primitives` `builtin_registry_surface_names` has been a HAND ROSTER beside it (the census
/// receipt's `hand_roster_missing_registry_rows` / `hand_roster_rows_absent_from_registry` lines
/// are the instrument for its drift). This query reads the compiled registry's key set directly so
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

/// Record the callee of ONE node when the resolver did NOT establish it as a source declaration
/// or a local binding -- the only two callee kinds that are not the census subject; everything
/// else is a primitive identity the resolver minted or a site where it minted nothing, and both
/// are rows. A local binding reaches this arm in two spellings: as a `LocallyBoundCall` target
/// and as `FunctionValueCallSemantics`, which the inferer mints when the callee is a body binding
/// (`let f = ...; f(x)`) and so carries no target at all. Both are one callee kind, the local,
/// and neither is a row: the census subject is the CALL edge to a primitive, and a primitive can
/// only be named in call position (there is no value-position primitive reference for a local to
/// capture), so the value a body binding holds was already walked at its own call sites.
/// The descent is `collect_primitive_call_edges`.
fn primitive_call_edge_at(
    texpr: &Rc<crate::v1_std_core::Node>,
    caller_module: &str,
    caller_decl: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    out: &mut Vec<crate::cli_run::PrimitiveCallEdgeRow>,
) {
    use crate::cli_run::{PrimitiveCallEdgeRow, PrimitiveCalleeIdentity};
    use crate::v1_std_core::{CallSemantics, ExprData, MethodSemantics};
    // The authored spelling is read ONCE per node and shared by the callee arm and the row;
    // reading it twice was a copied producer on the innermost loop of the corpus walk.
    let (authored_spelling, callee): (String, Option<PrimitiveCalleeIdentity>) =
        match &*texpr.expr_data {
            ExprData::ExprCall { call_semantics, .. } => {
                let spelling =
                    crate::v1_std_core::expr_call_func_at(texpr.clone(), source_indices.clone());
                let callee = match call_semantics.as_deref() {
                    Some(CallSemantics::PlainCallSemantics { target })
                    | Some(CallSemantics::ResolvedDirectCallSemantics { target, .. })
                    | Some(CallSemantics::LookupCallSemantics { target }) => {
                        primitive_call_callee_of_target(target, &spelling)
                    }
                    Some(CallSemantics::FunctionValueCallSemantics) => None,
                    None => Some(PrimitiveCalleeIdentity::UndeterminedFreeCall {
                        spelling: spelling.clone(),
                    }),
                };
                (spelling, callee)
            }
            ExprData::ExprMethodCall { method_semantics } => {
                let spelling =
                    crate::v1_std_core::expr_method_name_at(texpr.clone(), source_indices.clone());
                let callee = match method_semantics.as_deref() {
                    Some(MethodSemantics::AlgebraMethodSemantics {
                        algebra_template, ..
                    }) => match algebra_template {
                        Some(t) => Some(PrimitiveCalleeIdentity::AlgebraMethod {
                            template_name: t.name.clone(),
                        }),
                        None => Some(PrimitiveCalleeIdentity::PlainMethod {
                            spelling: spelling.clone(),
                        }),
                    },
                    Some(MethodSemantics::ServiceMethodSemantics { service_name, .. }) => {
                        Some(PrimitiveCalleeIdentity::ServiceOperation {
                            service_name: service_name.clone(),
                            operation: spelling.clone(),
                        })
                    }
                    Some(MethodSemantics::PlainMethodSemantics) | None => {
                        Some(PrimitiveCalleeIdentity::PlainMethod {
                            spelling: spelling.clone(),
                        })
                    }
                };
                (spelling, callee)
            }
            _ => (String::new(), None),
        };
    if let Some(callee) = callee {
        out.push(PrimitiveCallEdgeRow {
            caller_module: caller_module.to_string(),
            caller_decl: caller_decl.to_string(),
            authored_spelling,
            callee,
            span_file: texpr.span.file.clone(),
            span_start: texpr.span.start,
        });
    }
}

/// Walk one typed expression tree and push every call site whose callee the resolver did NOT
/// establish as a source declaration or a local binding. Explicit worklist: a deeply nested
/// expression overflowed the main thread's stack on the first corpus walk (2026-09-18), so the
/// descent is iterative; visit order does not matter because the edges are sorted afterwards.
fn collect_primitive_call_edges(
    texpr: &Rc<crate::v1_std_core::Node>,
    caller_module: &str,
    caller_decl: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    out: &mut Vec<crate::cli_run::PrimitiveCallEdgeRow>,
) {
    let mut pending: Vec<Rc<crate::v1_std_core::Node>> = vec![texpr.clone()];
    while let Some(node) = pending.pop() {
        primitive_call_edge_at(&node, caller_module, caller_decl, source_indices, out);
        // Every field that can carry an expression node: `children` (arguments, a service's
        // operations, a type's variants), `body`, `params` (a default argument), `properties`
        // (a fn's admit props), `transport` (a service transport expression).
        for child in node.children.iter() {
            pending.push(child.clone());
        }
        if let Some(body) = &node.body {
            pending.push(body.clone());
        }
        for param in node.params.iter() {
            pending.push(param.clone());
        }
        for property in node.properties.iter() {
            pending.push(property.clone());
        }
        if let Some(transport) = &node.transport {
            pending.push(transport.clone());
        }
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
        // The ITEM NODE is the walk root, so every expression-bearing field of a declaration --
        // body, params (default arguments), children (a service's operations, a type's
        // variants), properties (admit props), transport -- is inside the denominator.
        collect_primitive_call_edges(item, &module_name, &decl_name, &si, out);
    }
}

/// THE PRIMITIVE-CONSUMER EDGE CENSUS: every call site in every `.dag` module under
/// `pool_roots` (walked when its path starts with one of `entry_prefixes`, all when none are
/// given, and never when it matches an `exclude_substrings`) whose resolver-established callee
/// is a runtime primitive, an algebra method template, a service operation, or NOTHING -- with
/// the caller's declaration identity and the site's span.
///
/// ONE RESOLUTION, NOT ONE PER ENTRY (DESIGN section 6b, the #11401 R1 specimen). The first cut
/// of this query resolved every entry as its own closure through `resolve_entry_with_index`:
/// that assembled a graph per entry, re-typed the v2 compiler modules whenever the typed cache's
/// cap evicted them, and pinned every assembled graph in `resolved_graph_memo` (the
/// `primitive-call-edges.*` trace marks and the process RSS are the instrument for that cost).
/// The population is a walk over ONE resolved corpus, so the query now takes the same route
/// `gunbc compile --source-root` takes for a primary root:
/// `primary_root_subject_closure` for each pool root, one `compile_to_resolved_with_options`
/// over the union under the floor's compile-clean admission (modules outside that closure enter
/// the name census only), and a walk over every module of that single `ResolvedGraph`.
///
/// A module carrying an interpreter-blocking diagnostic is carried as a typed refusal ROW keyed
/// by its module path and its body is not walked, because a partially typed tree can carry a
/// callee identity the resolver never established. The consumer
/// (`gunbc.primitive_egress.census`) refuses when any admitted module is refused.
pub fn compile_dag_primitive_call_edges(
    exclude_substrings: &[String],
    pool_roots: &[String],
    entry_prefixes: &[String],
) -> crate::cli_run::PrimitiveCallEdgeCensus {
    // ON A THREAD WITH A LARGE STACK. The one-resolution graph over the corpus is dropped when
    // this query returns, and dropping a deeply nested `Rc<Node>` chain recurses once per level:
    // on the default main stack that overflowed AFTER the walk had finished (2026-09-19). The
    // stack is reserved virtual memory, committed only as touched.
    let outcome = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("primitive-call-edges".to_string())
            .stack_size(1 << 30)
            .spawn_scoped(scope, || {
                compile_dag_primitive_call_edges_on_this_thread(
                    exclude_substrings,
                    pool_roots,
                    entry_prefixes,
                )
            })
            .map(|handle| handle.join())
    });
    match outcome {
        Ok(Ok(census)) => census,
        Ok(Err(_)) => crate::cli_run::PrimitiveCallEdgeCensus::Refused {
            cause: "primitive call edges: the walk thread panicked".to_string(),
        },
        Err(e) => crate::cli_run::PrimitiveCallEdgeCensus::Refused {
            cause: format!("primitive call edges: could not spawn the walk thread: {e}"),
        },
    }
}

fn compile_dag_primitive_call_edges_on_this_thread(
    exclude_substrings: &[String],
    pool_roots: &[String],
    entry_prefixes: &[String],
) -> crate::cli_run::PrimitiveCallEdgeCensus {
    use crate::cli_run::{PrimitiveCallEdgeCensus, PrimitiveCallEntryRefusal};
    if pool_roots.is_empty() || pool_roots.iter().any(|r| r.trim().is_empty()) {
        return PrimitiveCallEdgeCensus::Refused {
            cause: "primitive call edges: every pool root must be nonempty and at least one is required"
                .to_string(),
        };
    }
    let abs_roots = pool_roots_abs(pool_roots);
    for (authored, abs) in pool_roots.iter().zip(abs_roots.iter()) {
        if !Path::new(abs).is_dir() {
            return PrimitiveCallEdgeCensus::Refused {
                cause: format!(
                    "primitive call edges: declared root '{authored}' is not an inspectable directory"
                ),
            };
        }
    }
    // Progress goes through the trace-mark seam (`v1_rt::trace_mark`, the one phase printer this
    // crate admits) so a killed run still shows which phase it died in.
    v1_rt::trace_mark("primitive-call-edges.close.begin".to_string());
    let index = crate::cli_run::build_multi_entry_index(&abs_roots);
    let mut by_path: BTreeMap<String, Rc<v1_compiler_compile::SourceFile>> = BTreeMap::new();
    for root in pool_roots {
        match crate::cli_run::primary_root_subject_closure(&index, root) {
            Ok(closure) => {
                for source in closure {
                    by_path.insert(source.path.clone(), source);
                }
            }
            Err(refusal) => {
                return PrimitiveCallEdgeCensus::Refused {
                    cause: format!(
                        "primitive call edges: closure of root '{root}' refused at {}: {}",
                        refusal.phase, refusal.cause
                    ),
                }
            }
        }
    }
    let closure: Vec<Rc<v1_compiler_compile::SourceFile>> = by_path.into_values().collect();
    v1_rt::trace_mark(format!(
        "primitive-call-edges.close ({} sources).done",
        closure.len()
    ));
    v1_rt::trace_mark("primitive-call-edges.resolve.begin".to_string());
    let options = compile_clean_pipeline_options_for_sources(Some(&index), &closure);
    let resolved =
        v1_compiler_compile::compile_to_resolved_with_options(Rc::new(closure.into()), options);
    v1_rt::trace_mark("primitive-call-edges.resolve.done".to_string());
    v1_rt::trace_mark("primitive-call-edges.walk.begin".to_string());
    let Some(graph) = resolved.graph.clone() else {
        let blocking = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
            resolved.diagnostics.clone(),
        );
        return PrimitiveCallEdgeCensus::Refused {
            cause: format!(
                "primitive call edges: the corpus did not resolve to a graph: {}",
                blocking.iter().cloned().collect::<Vec<_>>().join("; ")
            ),
        };
    };
    // Blocking diagnostics, attributed to the module that carries them.
    let mut blocked_modules: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for d in resolved.diagnostics.iter() {
        if crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone()) {
            let message = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
                Rc::new(vec![d.clone()].into()),
            )
            .iter()
            .next()
            .cloned()
            .unwrap_or_else(|| "blocking diagnostic".to_string());
            blocked_modules
                .entry(d.module_name.clone())
                .or_default()
                .push(message);
        }
    }
    let admitted = |rel: &str| -> bool {
        !is_excluded_import_path(rel, exclude_substrings)
            && (entry_prefixes.is_empty()
                || entry_prefixes.iter().any(|p| rel.starts_with(p.as_str())))
    };
    let mut entries_resolved = Vec::new();
    let mut entries_refused = Vec::new();
    let mut edges = Vec::new();
    let mut walked_modules: HashSet<String> = HashSet::new();
    for module in graph.modules.iter() {
        let module_file = rel_path_for_layer_import(Path::new(&module.module.span.file));
        if !admitted(&module_file) {
            continue;
        }
        let module_name = crate::v1_std_core::authored_name_at(
            module.type_env.source_indices.clone(),
            module.module.clone(),
        );
        if !walked_modules.insert(module_name.clone()) {
            // Two modules in one resolved graph carrying one authored name is an ambiguity
            // the census must SEE: a typed refusal row, never a shortened population.
            entries_refused.push(PrimitiveCallEntryRefusal {
                entry: module_file,
                cause: format!(
                    "module name `{module_name}` is declared by a second file in the resolved graph"
                ),
            });
            continue;
        }
        if let Some(messages) = blocked_modules.get(&module_name) {
            entries_refused.push(PrimitiveCallEntryRefusal {
                entry: module_file,
                cause: messages.join("; "),
            });
            continue;
        }
        primitive_call_edges_from_module(module, &mut edges);
        entries_resolved.push(module_file);
    }
    v1_rt::trace_mark(format!(
        "primitive-call-edges.walk ({} modules, {} refused, {} edges).done",
        entries_resolved.len(),
        entries_refused.len(),
        edges.len()
    ));
    edges.sort_by(|a, b| {
        (&a.caller_module, &a.caller_decl, &a.span_file, a.span_start).cmp(&(
            &b.caller_module,
            &b.caller_decl,
            &b.span_file,
            b.span_start,
        ))
    });
    entries_resolved.sort();
    entries_refused.sort_by(|a, b| a.entry.cmp(&b.entry));
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
    let compiler_digest = crate::closure_identity::transform_content_digest();
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

/// WHAT ONE FIXTURE COMPILE ESTABLISHES, independent of what a caller asserts about it.
/// The memo key is (source, read path, prepared inventory): the inputs the compile and render
/// depend on. `includes`/`excludes` are assertions evaluated AFTER compilation over the emitted
/// text, so they are not inputs of the compile and key nothing; each call evaluates its own
/// against the shared read (`emit_check_verdict`).
#[derive(Clone)]
pub(crate) enum EmitCheckRead {
    /// At least one compile-clean hard diagnostic: every assertion set reads `false`.
    HardDiagnostics,
    /// The emitted text at the read path.
    Content(Rc<str>),
}

fn emit_check_verdict(read: &EmitCheckRead, includes: &[String], excludes: &[String]) -> bool {
    match read {
        EmitCheckRead::HardDiagnostics => false,
        EmitCheckRead::Content(content) => {
            includes.iter().all(|n| content.contains(n.as_str()))
                && excludes.iter().all(|n| !content.contains(n.as_str()))
        }
    }
}

pub(crate) fn compile_dag_rust_emit_check_memo_key(
    source: &str,
    file_path: &str,
    inventory_digest: &str,
) -> String {
    use crate::v1_rt::{atom_identity_hash, hash_combine};
    let mut h = atom_identity_hash(source.to_string());
    h = hash_combine(h, atom_identity_hash(file_path.to_string()));
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
/// THE PARSED `requires` MEMBERS OF ONE OPERATION, as the seed parser carried them (D13 step b0).
/// Parses the supplied source with the seed's own tokenizer and parser, finds the named service
/// and operation, and reads the members through the one .dag reader,
/// `v1.compiler.parse` `operation_requires_declaration` -- whose production consumer is D13 step (c) --
/// returning the declared form first (`undeclared`, `none`, `opaque` or `resources`) and, for
/// `resources`, each member's type spelling in authored order. A parse error, a missing service or a
/// missing operation REFUSES with a located message; it never answers an empty list for a source
/// it could not read.
pub fn compile_dag_operation_requires(
    source: &str,
    service: &str,
    operation: &str,
) -> Result<Vec<String>, String> {
    let filename = "test.dag".to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(
        source.to_string(),
        filename.clone(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = Rc::new(source_indices);
    let result = crate::v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        return Err(format!(
            "compile_dag_operation_requires: parse error: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        ));
    }
    let module = result
        .module
        .as_ref()
        .ok_or_else(|| "compile_dag_operation_requires: source parsed to no module".to_string())?;
    let service_item = module
        .children
        .iter()
        .find(|item| {
            item.name == service
                && item.module_item_kind
                    == crate::v1_std_core::ParsedModuleItemKind::ModuleItemService
        })
        .ok_or_else(|| format!("compile_dag_operation_requires: no service `{service}`"))?;
    let op = service_item
        .children
        .iter()
        .find(|op| op.name == operation)
        .ok_or_else(|| {
            format!("compile_dag_operation_requires: no operation `{operation}` in `{service}`")
        })?;
    use crate::v1_compiler_parse::OperationRequiresDeclaration::*;
    Ok(
        match crate::v1_compiler_parse::operation_requires_declaration(op.clone(), source_indices)
            .as_ref()
        {
            RequiresUndeclared => vec!["undeclared".to_string()],
            RequiresNone => vec!["none".to_string()],
            RequiresOpaque => vec!["opaque".to_string()],
            RequiresResources { members } => std::iter::once("resources".to_string())
                .chain(members.iter().map(|member| member.name.to_string()))
                .collect(),
        },
    )
}

pub fn compile_dag_rust_emit_check(
    source: &str,
    file_path: &str,
    includes: &[String],
    excludes: &[String],
) -> Result<bool, FixtureRenderRefusal> {
    // Memo only under the floor guard, keyed on declared inputs AND the prepared
    // inventory digest (`build_module_path_index_from_witness_roots` reads those bytes).
    // Outside the guard there is no snapshot, so a hit would lie about disk.
    let Some(inventory_digest) = floor_prepared_inventory_digest() else {
        return compile_dag_rust_emit_check_uncached(source, file_path)
            .map(|read| emit_check_verdict(&read, includes, excludes));
    };
    let memo_key = compile_dag_rust_emit_check_memo_key(source, file_path, &inventory_digest);
    if let Some(hit) = COMPILE_DAG_RUST_EMIT_CHECK_MEMO.with(|m| m.borrow().get(&memo_key).cloned())
    {
        COMPILE_DAG_RUST_EMIT_CHECK_MEMO_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        record_fixture_closure_memo_hit(&memo_key);
        return hit.map(|read| emit_check_verdict(&read, includes, excludes));
    }
    COMPILE_DAG_RUST_EMIT_CHECK_MEMO_MISSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // A MISS FILLS A SHARED ARTIFACT. Measured on the same thread clock the claim loop enforces
    // against, so the two quantities cannot drift apart, and recorded rather than subtracted here
    // — the claim loop does the split, this only says how much of the cost was a fill.
    let fill_started = v1_interpreter::thread_cpu_nanos();
    let fill_wall_started = std::time::Instant::now();
    begin_fixture_closure_fill();
    let verdict = compile_dag_rust_emit_check_uncached(source, file_path);
    finish_fixture_closure_fill(&memo_key);
    record_shared_artifact_fill_cpu(
        v1_interpreter::thread_cpu_nanos().saturating_sub(fill_started),
    );
    record_shared_artifact_fill_wall(fill_wall_started.elapsed().as_nanos());
    COMPILE_DAG_RUST_EMIT_CHECK_MEMO.with(|m| m.borrow_mut().insert(memo_key, verdict.clone()));
    verdict.map(|read| emit_check_verdict(&read, includes, excludes))
}

pub(crate) fn compile_dag_rust_emit_check_uncached(
    source: &str,
    file_path: &str,
) -> Result<EmitCheckRead, FixtureRenderRefusal> {
    let module_index = build_module_path_index_from_witness_roots();
    let sources = resolve_virtual_source_with_imports(FIXTURE_SOURCE_PATH, source, &module_index)
        .map_err(|cause| FixtureRenderRefusal::ClosureUnresolvable { cause })?;
    record_fixture_closure(&sources);
    let result = compile_fixture_rendering_only_what_is_read(sources, Some(file_path))?;
    let hard_diagnostics = result
        .diagnostics
        .iter()
        .filter(|d| compile_clean_diagnostic_is_hard(d))
        .count();
    if hard_diagnostics != 0 {
        return Ok(EmitCheckRead::HardDiagnostics);
    }
    match result.files.iter().find(|f| f.path == file_path) {
        Some(f) => Ok(EmitCheckRead::Content(Rc::from(f.content.as_str()))),
        // A CLEAN COMPILE THAT DID NOT PRODUCE THE READ PATH IS NOT A RED VERDICT ABOUT ITS TEXT.
        // The path named no module of the compiled closure and no crate-level file the emitter
        // writes, so there was nothing to read; answering `false` would let a misspelled path pass
        // as a discriminating red. Under the full render this was the same absence, read as false.
        None => Err(FixtureRenderRefusal::ReadPathNotEmitted {
            read_path: file_path.to_string(),
            emitted: result.files.iter().map(|f| f.path.clone()).collect(),
        }),
    }
}

/// The path the census and the emit check give the caller's fixture source in the source vector.
pub(crate) const FIXTURE_SOURCE_PATH: &str = "test.dag";

/// WHY A FIXTURE COMPILE NO LONGER RENDERS EVERY MODULE (floor repair C1, plan
/// `docs/plans/floor-time-attribution-2026-10-02.md`).
///
/// Both fixture instruments compile the fixture's whole live import closure. That compile is
/// their subject and is unchanged. Up to this commit they then rendered Rust for EVERY module of
/// the closure (`RenderEveryModule`). The emit check reads ONE emitted file, and the census reads
/// diagnostics, among them the per-module emit refusals of the modules it renders. So every
/// closure module rendered beyond the read set was demanded by no consumer. That is §2 redundant
/// demand, not a cost to cache: rendering is a pure function of (graph, module), and a module
/// nobody reads owes no rendering at all.
///
/// The render set is DERIVED from the compiled graph, never authored by the caller:
///   * the fixture's own modules, i.e. those whose declaration is spanned in
///     [`FIXTURE_SOURCE_PATH`] (their emit refusals are the census's subject);
///   * for the emit check, the closure module whose emitted path IS the read path, if any.
///
/// So the selection cannot name a module outside the compiled closure. It is a subset of the
/// closure's own module names, so "names an outside module" has no constructor here.
/// Crate-level files (`Cargo.toml`, `src/lib.rs`, `src/main.rs`, ...) are emitted under every
/// selection, so a read of one needs no module selected.
///
/// WHAT NARROWS, stated so no receipt claims more than this reads (DESIGN §4b): a corpus module
/// in the fixture's closure is no longer rendered, so ITS per-module emit refusal no longer
/// reaches the census rows or the emit check's hard-diagnostic gate. Whole-graph emit checks
/// (anonymous records, effectful recursion, file-name and symbol collisions) run before
/// selection and are unchanged. A corpus module's own emission is not a fixture claim's subject:
/// the required floor renders every recorded closure module once per run in
/// [`fixture_closure_union_emit_receipt`], and an emit refusal there refuses the floor.
///
/// REFUSES, NEVER WIDENS. If the graph holds no module spanned in the fixture source, there is
/// no subject to render, and the answer is a typed refusal. It is never a fall back to
/// `RenderEveryModule`, which would re-buy the work and hide the deficit (DESIGN §5, absorbing
/// fallback).
///
/// The control that the selected bytes equal the full render's is
/// [`compile_dag_render_selection_agreement`], which runs the full render for real.
pub(crate) fn compile_fixture_rendering_only_what_is_read(
    sources: Vec<Rc<v1_compiler_compile::SourceFile>>,
    read_path: Option<&str>,
) -> Result<Rc<v1_compiler_compile::PipelineResult>, FixtureRenderRefusal> {
    let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
    let selection = fixture_render_selection(&resolved, read_path)?;
    Ok(v1_compiler_compile::emit_resolved_for_target_selected(
        resolved,
        crate::v1_compiler_artifact::RenderTarget::Rust,
        selection,
    ))
}

/// One pinned control fixture of [`render_selection_agreement_receipt`].
pub(crate) struct RenderSelectionAgreementFixture {
    pub name: &'static str,
    pub source: &'static str,
    pub read_path: &'static str,
    /// Whether the derived selection must render strictly fewer files than the full render.
    /// `false` is the import-free RED: a closure that is only the fixture has nothing to narrow,
    /// so a receipt that reported a narrowing there would be reading something other than the
    /// render it ran.
    pub narrows: bool,
}

/// THE PINNED CONTROL FIXTURES for floor repair C1, kept minimal so the receipt costs seconds.
/// The two importing fixtures import two leaf `std` modules that have no imports of their own
/// (`std.logic`, `std.error_primitives`). Reading one of them leaves the other unrendered,
/// which is the narrowing.
///
/// THESE SOURCES ARE CORPUS CONSUMERS THAT NO IMPORT SCAN SEES. They are strings handed to the
/// live module index, so a change that retires a module they import breaks the receipt and is
/// told nothing. That happened once: gunbc#12846 retired `std.magnitude`, the receipt's first
/// choice, and the floor refused on main+PR with `unresolved import`. That refusal is the
/// receipt working: a fixture whose world moved stops the line rather than passing on some other
/// mechanism (DESIGN §3). The two modules are chosen so that retiring them is a visible
/// cascade: `std.logic` is imported across the corpus, and `std.error_primitives` by
/// `std.algebra`. They are not chosen for being small. One reads the fixture's own module, one reads a module the fixture
/// IMPORTS (the read-path arm of [`fixture_render_selection`]), and one is import-free (the red).
pub(crate) const RENDER_SELECTION_AGREEMENT_FIXTURES: &[RenderSelectionAgreementFixture] = &[
    RenderSelectionAgreementFixture {
        name: "reads_fixture_module",
        source: "module rsa_own\n\nimport std.logic { Classical }\nimport std.error_primitives { DivError }\n\nfn rsa_pick(c: Classical, e: DivError) -> DivError {\n  e\n}\n",
        read_path: "src/rsa_own.rs",
        narrows: true,
    },
    RenderSelectionAgreementFixture {
        name: "reads_imported_module",
        source: "module rsa_importer\n\nimport std.logic { Classical }\nimport std.error_primitives { DivError }\n\nfn rsa_pick(c: Classical, e: DivError) -> DivError {\n  e\n}\n",
        read_path: "src/std_logic.rs",
        narrows: true,
    },
    RenderSelectionAgreementFixture {
        name: "import_free_has_nothing_to_narrow",
        source: "module rsa_alone\n\nfn rsa_one() -> Int {\n  1\n}\n",
        read_path: "src/rsa_alone.rs",
        narrows: false,
    },
];

/// THE CONTROL FOR [`compile_fixture_rendering_only_what_is_read`], run by the required floor on
/// every run (`required_floor_runner`, `[floor-receipt] receipt=render-selection-agreement`). It
/// is also the one execution of the FULL render that the fixture instruments no longer perform:
/// DESIGN §3's pairing obligation keeps one execution of the real path, and DESIGN §4b(4) keeps
/// the discriminating red and the positive control enrolled.
///
/// For each of [`RENDER_SELECTION_AGREEMENT_FIXTURES`] it compiles once at this revision, renders
/// the resolved graph with `RenderEveryModule` and with the derived selection, and requires:
///   * the bytes at the read path are present in the full render and equal in both;
///   * the two diagnostic lists are equal, element for element;
///   * the selected render emitted strictly fewer files exactly when the fixture says it narrows.
///
/// The third condition checks the ROUTE, not the answer: a selection that silently widened to
/// every module would agree on bytes and diagnostics trivially.
///
/// Any mismatch is returned as a located refusal naming the fixture, the read path and the first
/// differing byte offset or diagnostic index. The caller refuses the floor with it and never
/// warns. On success it returns, per fixture, (name, wall ms, full files, selected files) for the
/// caller's log line.
pub(crate) fn render_selection_agreement_receipt(
) -> Result<Vec<(&'static str, u128, usize, usize)>, String> {
    let mut observed = Vec::with_capacity(RENDER_SELECTION_AGREEMENT_FIXTURES.len());
    for fx in RENDER_SELECTION_AGREEMENT_FIXTURES {
        let fixture_started = std::time::Instant::now();
        let refuse = |what: String| {
            format!(
                "REQUIRED-FLOOR REFUSAL cause=RenderSelectionDisagreement \
                 receipt=render_selection_agreement_receipt fixture={} read_path={} -- {what}",
                fx.name, fx.read_path
            )
        };
        let module_index = build_module_path_index_from_witness_roots();
        let sources =
            resolve_virtual_source_with_imports(FIXTURE_SOURCE_PATH, fx.source, &module_index)
                .map_err(&refuse)?;
        let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
        let selection = fixture_render_selection(&resolved, Some(fx.read_path))
            .map_err(|r| refuse(r.to_string()))?;
        let full = v1_compiler_compile::emit_resolved_for_target(
            resolved.clone(),
            crate::v1_compiler_artifact::RenderTarget::Rust,
        );
        let selected = v1_compiler_compile::emit_resolved_for_target_selected(
            resolved,
            crate::v1_compiler_artifact::RenderTarget::Rust,
            selection,
        );
        let read = |r: &v1_compiler_compile::PipelineResult| {
            r.files
                .iter()
                .find(|f| f.path == fx.read_path)
                .map(|f| f.content.clone())
        };
        let Some(full_bytes) = read(&full) else {
            let diagnostics: Vec<String> = full
                .diagnostics
                .iter()
                .map(|d| diagnostic_to_message(d.diagnostic.clone()))
                .collect();
            return Err(refuse(format!(
                "the full render did not emit the read path ({} files, {} diagnostics: {})",
                full.files.len(),
                full.diagnostics.len(),
                diagnostics.join(" | ")
            )));
        };
        let Some(selected_bytes) = read(&selected) else {
            return Err(refuse(
                "the selected render did not emit the read path".to_string(),
            ));
        };
        if full_bytes != selected_bytes {
            let offset = full_bytes
                .bytes()
                .zip(selected_bytes.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(full_bytes.len().min(selected_bytes.len()));
            return Err(refuse(format!(
                "read-path bytes differ at byte {offset} (full {} bytes, selected {} bytes)",
                full_bytes.len(),
                selected_bytes.len()
            )));
        }
        if full.diagnostics != selected.diagnostics {
            let index = full
                .diagnostics
                .iter()
                .zip(selected.diagnostics.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(full.diagnostics.len().min(selected.diagnostics.len()));
            let at = |d: &v1_compiler_compile::PipelineResult| {
                d.diagnostics
                    .get(index)
                    .map(|e| diagnostic_to_message(e.diagnostic.clone()))
                    .unwrap_or_else(|| "<absent>".to_string())
            };
            return Err(refuse(format!(
                "diagnostics differ at index {index} (full {}, selected {}): full=`{}` selected=`{}`",
                full.diagnostics.len(),
                selected.diagnostics.len(),
                at(&full),
                at(&selected)
            )));
        }
        let narrowed = selected.files.len() < full.files.len();
        if narrowed != fx.narrows {
            return Err(refuse(format!(
                "route mismatch: expected narrows={} but full render emitted {} files and the \
                 selected render {}",
                fx.narrows,
                full.files.len(),
                selected.files.len()
            )));
        }
        observed.push((
            fx.name,
            fixture_started.elapsed().as_millis(),
            full.files.len(),
            selected.files.len(),
        ));
    }
    Ok(observed)
}

/// Why a fixture compile could not be answered. Typed, so a caller states which arm fired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureRenderRefusal {
    /// The compiled graph holds no module whose declaration is spanned in the fixture source.
    NoFixtureModuleInClosure {
        fixture_path: String,
        closure_modules: usize,
    },
    /// A clean compile emitted no file at the read path.
    ReadPathNotEmitted {
        read_path: String,
        emitted: Vec<String>,
    },
    /// The closure authority refused to close the fixture's imported modules
    /// (`resolve_virtual_source_with_imports`), so there is no program to compile.
    ClosureUnresolvable { cause: String },
}

impl std::fmt::Display for FixtureRenderRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureRenderRefusal::NoFixtureModuleInClosure {
                fixture_path,
                closure_modules,
            } => write!(
                f,
                "render selection refused: none of the {closure_modules} compiled module(s) is \
                 declared in fixture source '{fixture_path}', so there is no fixture module to \
                 render (refused rather than rendering every module)"
            ),
            FixtureRenderRefusal::ReadPathNotEmitted { read_path, emitted } => write!(
                f,
                "read path '{read_path}' names no module of the compiled closure and no \
                 crate-level file; the compile was clean and emitted {} file(s): {}",
                emitted.len(),
                emitted.join(", ")
            ),
            FixtureRenderRefusal::ClosureUnresolvable { cause } => {
                write!(
                    f,
                    "fixture closure refused by the closure authority: {cause}"
                )
            }
        }
    }
}

/// The render selection for a fixture compile, derived from the compiled graph (see
/// [`compile_fixture_rendering_only_what_is_read`]). A graph that did not survive the front end
/// is not emitted at all, so no selection is needed and an empty one is returned.
pub(crate) fn fixture_render_selection(
    resolved: &v1_compiler_compile::ResolvedPipelineResult,
    read_path: Option<&str>,
) -> Result<Rc<crate::v1_compiler_artifact::RustModuleRenderSelection>, FixtureRenderRefusal> {
    use crate::v1_compiler_artifact::RustModuleRenderSelection;
    let selected = |basenames: Vec<String>| {
        Rc::new(RustModuleRenderSelection::RenderSelectedMirrors {
            basenames: Rc::new(basenames.into()),
        })
    };
    let Some(graph) = resolved.graph.as_ref() else {
        return Ok(selected(Vec::new()));
    };
    let root = crate::v1_compiler_emit_rust::rust_source_root();
    let mut fixture: Vec<String> = Vec::new();
    let mut read: Vec<String> = Vec::new();
    for tm in graph.modules.iter() {
        let name = authored_name_at(tm.type_env.source_indices.clone(), tm.module.clone());
        let path = crate::v1_compiler_emit_rust::rust_module_emit_path(name);
        let basename = path
            .strip_prefix(root.as_str())
            .unwrap_or(&path)
            .to_string();
        if tm.module.span.file == FIXTURE_SOURCE_PATH {
            fixture.push(basename.clone());
        }
        if read_path == Some(path.as_str()) {
            read.push(basename);
        }
    }
    if fixture.is_empty() {
        return Err(FixtureRenderRefusal::NoFixtureModuleInClosure {
            fixture_path: FIXTURE_SOURCE_PATH.to_string(),
            closure_modules: graph.modules.len(),
        });
    }
    fixture.extend(read);
    fixture.sort();
    fixture.dedup();
    Ok(selected(fixture))
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
    out.push_str("import std.types { List }\n");
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
        out.push_str("import std.types { List }\n");
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
    let index = match super::entry_resolve::try_index_for_run_or_owned_pool(source_roots) {
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

/// THE FIXTURE-CLOSURE UNION (retires `gunbc.rung_drop.fixture_closure_corpus_emit_refusals_lost_as_passenger`).
///
/// Floor C1 (gunbc#13037) stopped rendering every closure module per fixture compile, so a corpus
/// module's own emit refusal no longer reached any required gate unless the v1 seed mirrors
/// happened to cover it. The restoration is the deduplicated form of what was removed: each fixture
/// instrument RECORDS the closure it resolved (every source except the fixture itself, by path and
/// bytes), and the required floor renders the union of those closures ONCE, after the claims ran
/// ([`fixture_closure_union_emit_receipt`]).
///
/// THE POPULATION IS THE INSTRUMENT'S OUTPUT, NOT A LIST. It is exactly the closures the fixture
/// instruments resolved on this run, recorded at the one seam both of them pass through
/// (`resolve_virtual_source_with_imports` over the floor's module index), so it cannot name a module
/// no fixture reached and cannot omit one a fixture did. A memo hit replays the closure its fill
/// recorded ([`record_fixture_closure_memo_hit`]), so the union stays complete on a later run in the
/// same process, where every fixture call may be a hit.
///
/// One path recorded with two different byte contents is not unioned by picking one: it is kept as
/// a conflict, and the receipt refuses on it (DESIGN §5, refuse rather than widen or choose).
#[derive(Default)]
pub(crate) struct FixtureClosureUnion {
    pub members: BTreeMap<String, String>,
    pub conflicts: BTreeSet<String>,
    pub fixture_compiles: usize,
    /// Fixture calls answered by a memo hit, whose closures were replayed rather than resolved.
    pub memo_hits: usize,
}

static FIXTURE_CLOSURE_UNION: Mutex<Option<FixtureClosureUnion>> = Mutex::new(None);

/// WHAT A MEMO HIT REPLAYS. The two fixture memos are process-lived and thread-local, while the
/// union is per run (`run_required_floor` resets it). A later run in the same process whose fixture
/// compiles all hit the memo would otherwise record nothing and render an empty union green. So a
/// fill stores its closure's paths under its memo key, and every hit replays them into the union
/// ([`record_fixture_closure_memo_hit`]). A replay returns exactly the bytes ITS OWN fill read: the
/// closure is stored as (path, bytes) under the memo key, never looked up by path alone, so a file
/// that changed between runs cannot be replayed with another fill's bytes (review 76336). Identical
/// bytes for one path are shared, not copied, through `interned`.
/// One fill's closure: every recorded path with the exact bytes that fill read.
type RecordedFixtureClosure = Vec<(String, Arc<String>)>;

#[derive(Default)]
struct FixtureClosureMemoReplay {
    closure_by_memo_key: std::collections::HashMap<String, RecordedFixtureClosure>,
    interned: std::collections::HashMap<String, Vec<Arc<String>>>,
}

impl FixtureClosureMemoReplay {
    fn intern(&mut self, path: &str, content: &str) -> Arc<String> {
        let variants = self.interned.entry(path.to_string()).or_default();
        if let Some(existing) = variants.iter().find(|v| v.as_str() == content) {
            return existing.clone();
        }
        let fresh = Arc::new(content.to_string());
        variants.push(fresh.clone());
        fresh
    }
}

static FIXTURE_CLOSURE_MEMO_REPLAY: Mutex<Option<FixtureClosureMemoReplay>> = Mutex::new(None);

thread_local! {
    /// The paths the last [`record_fixture_closure`] on this thread recorded, taken by the memo
    /// wrapper that triggered the fill. Cleared before each fill so a fill that panicked before
    /// resolving cannot inherit an earlier fill's closure.
    static LAST_RECORDED_FIXTURE_CLOSURE: RefCell<Option<RecordedFixtureClosure>> = const { RefCell::new(None) };
}

/// Called by a memo wrapper just before it fills.
pub(crate) fn begin_fixture_closure_fill() {
    LAST_RECORDED_FIXTURE_CLOSURE.with(|l| *l.borrow_mut() = None);
}

/// Called by a memo wrapper after it filled `memo_key`: the closure the fill recorded becomes what
/// a later hit on that key replays.
pub(crate) fn finish_fixture_closure_fill(memo_key: &str) {
    let Some(closure) = LAST_RECORDED_FIXTURE_CLOSURE.with(|l| l.borrow_mut().take()) else {
        return;
    };
    let mut guard = FIXTURE_CLOSURE_MEMO_REPLAY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard
        .get_or_insert_with(FixtureClosureMemoReplay::default)
        .closure_by_memo_key
        .insert(memo_key.to_string(), closure);
}

/// A memo hit on `memo_key`: replay the closure its fill recorded into this run's union.
pub(crate) fn record_fixture_closure_memo_hit(memo_key: &str) {
    let guard = FIXTURE_CLOSURE_MEMO_REPLAY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(replay) = guard.as_ref() else {
        return;
    };
    let Some(closure) = replay.closure_by_memo_key.get(memo_key) else {
        return;
    };
    let sources: Vec<Rc<v1_compiler_compile::SourceFile>> = closure
        .iter()
        .map(|(path, content)| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.as_str().to_string(),
            })
        })
        .collect();
    drop(guard);
    record_into_union(&sources, true);
}

/// Record one fixture compile's resolved closure into the run's union. Called by the two fixture
/// instruments' uncached paths with the exact source vector they compile.
pub(crate) fn record_fixture_closure(sources: &[Rc<v1_compiler_compile::SourceFile>]) {
    let closure: RecordedFixtureClosure = {
        let mut replay = FIXTURE_CLOSURE_MEMO_REPLAY
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let replay = replay.get_or_insert_with(FixtureClosureMemoReplay::default);
        sources
            .iter()
            .filter(|source| source.path != FIXTURE_SOURCE_PATH)
            .map(|source| {
                (
                    source.path.clone(),
                    replay.intern(&source.path, &source.content),
                )
            })
            .collect()
    };
    LAST_RECORDED_FIXTURE_CLOSURE.with(|l| *l.borrow_mut() = Some(closure));
    record_into_union(sources, false);
}

fn record_into_union(sources: &[Rc<v1_compiler_compile::SourceFile>], memo_hit: bool) {
    let mut guard = FIXTURE_CLOSURE_UNION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let union = guard.get_or_insert_with(FixtureClosureUnion::default);
    if memo_hit {
        union.memo_hits += 1;
    } else {
        union.fixture_compiles += 1;
    }
    for source in sources {
        if source.path == FIXTURE_SOURCE_PATH {
            continue;
        }
        match union.members.get(&source.path) {
            Some(existing) if existing != &source.content => {
                union.conflicts.insert(source.path.clone());
            }
            Some(_) => {}
            None => {
                union
                    .members
                    .insert(source.path.clone(), source.content.clone());
            }
        }
    }
}

/// Take the union recorded so far, leaving it empty for the next run in this process.
pub(crate) fn take_fixture_closure_union() -> FixtureClosureUnion {
    FIXTURE_CLOSURE_UNION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
        .unwrap_or_default()
}

/// What a held union render observed, for the caller's `[floor-phase]` line.
#[derive(Debug)]
pub(crate) struct FixtureClosureUnionObserved {
    pub members: usize,
    pub digest: String,
    pub files: usize,
    pub emit_diagnostics: usize,
    /// Services not rendered, each naming module, service, the capture-policy fact, and the drop.
    /// Empty when the union rendered every member. Never a skip without a row here.
    pub excluded: Vec<String>,
}

/// The digest of a union: path and bytes of every member, in path order. Two runs print the same
/// digest exactly when they rendered the same population.
pub(crate) fn fixture_closure_union_digest(members: &BTreeMap<String, String>) -> String {
    use crate::v1_rt::{atom_identity_hash, hash_combine};
    let mut h = atom_identity_hash(format!("fixture-closure-union:{}", members.len()));
    for (path, content) in members {
        h = hash_combine(h, atom_identity_hash(path.clone()));
        h = hash_combine(h, atom_identity_hash(content.clone()));
    }
    h
}

/// RENDER THE UNION ONCE THROUGH THE RUST EMITTER AND REFUSE ON ANY PER-MODULE EMIT REFUSAL.
///
/// The members are compiled together as one source vector and rendered with `RenderEveryModule`,
/// so every member owes its own rendering, which is the coverage #13037 removed. The compile is
/// fresh: the floor's prepared graph is the gate closure with its typecheck caches stripped
/// (`prepared_graph_without_typecheck_caches`), a different carrier from the `compile_to_resolved`
/// output the fixture instruments render, and the caller prints how many members lie outside it,
/// so a reader can see the union is not that closure rather than assume it.
///
/// Every failure is a typed, located refusal, never a skip:
///   * `FixtureClosureUnionConflict`: one path recorded with two contents;
///   * `FixtureClosureUnionUncompilable`: a blocking compile diagnostic, so no member could be
///     rendered, named by module;
///   * `FixtureClosureUnionEmitRefused`: an error diagnostic produced by the render, named by
///     module and emit reason. These are the render's own diagnostics: a render's diagnostic list
///     is the compile's list followed by the emitter's (`emit_resolved_for_target_selected`), so
///     the suffix past the compile's length is exactly what the emitter added.
///
/// Unmodeled rust stderr-capture channels (`gunbc.rung_drop`
/// `fixture_closure_union_unmodeled_stderr_capture`) are a typed exclusion, not this refusal:
/// those services are stripped from the emit graph and listed on the observation. A sibling
/// unmodeled key or any other emit error still refuses.
pub(crate) fn fixture_closure_union_emit_receipt(
    union: &FixtureClosureUnion,
) -> Result<FixtureClosureUnionObserved, String> {
    let digest = fixture_closure_union_digest(&union.members);
    let refuse = |cause: &str, what: String| {
        format!(
            "REQUIRED-FLOOR REFUSAL cause={cause} receipt=fixture_closure_union_emit_receipt \
             members={} digest={digest} -- {what}",
            union.members.len()
        )
    };
    if !union.conflicts.is_empty() {
        return Err(refuse(
            "FixtureClosureUnionConflict",
            format!(
                "paths recorded with two different contents in one run: {:?}",
                union.conflicts
            ),
        ));
    }
    // AN EMPTY UNION IS A BROKEN SEAM, NOT A CLEAN ONE (review 76399). The required floor always
    // runs fixture claims, so a union with no members means the recording seam was bypassed (an
    // instrument that does not record, a hit that did not replay), and holding here would let the
    // restored capability die green.
    if union.members.is_empty() {
        return Err(refuse(
            "FixtureClosureUnionEmpty",
            format!(
                "no fixture closure was recorded on this run (fixture_compiles={} memo_hits={}); \
                 the recording seam (record_fixture_closure / record_fixture_closure_memo_hit) was \
                 bypassed",
                union.fixture_compiles, union.memo_hits
            ),
        ));
    }
    let sources: Vec<Rc<v1_compiler_compile::SourceFile>> = union
        .members
        .iter()
        .map(|(path, content)| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
    let located = |d: &Rc<ErrorNode>| {
        format!(
            "module={} reason=`{}`",
            d.module_name,
            diagnostic_to_message(d.diagnostic.clone())
        )
    };
    if v1_compiler_compile::emittable_graph(resolved.clone()).is_none() {
        let blocking: Vec<String> = resolved
            .diagnostics
            .iter()
            .filter(|d| {
                crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())
            })
            .map(located)
            .collect();
        return Err(refuse(
            "FixtureClosureUnionUncompilable",
            format!(
                "the union compile produced {} blocking diagnostics, so no member was rendered: {}",
                blocking.len(),
                blocking.join(" | ")
            ),
        ));
    }
    let (resolved, excluded) = union_emit_graph_excluding_unmodeled_stderr_capture(resolved);
    let compile_diagnostics = resolved.diagnostics.len();
    let rendered = v1_compiler_compile::emit_resolved_for_target(
        resolved,
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    let emitted: Vec<&Rc<ErrorNode>> = rendered
        .diagnostics
        .iter()
        .skip(compile_diagnostics)
        .collect();
    let refusals: Vec<String> = emitted
        .iter()
        .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| located(d))
        .collect();
    if !refusals.is_empty() {
        return Err(refuse(
            "FixtureClosureUnionEmitRefused",
            format!(
                "{} per-module emit refusals: {}",
                refusals.len(),
                refusals.join(" | ")
            ),
        ));
    }
    Ok(FixtureClosureUnionObserved {
        members: union.members.len(),
        digest,
        files: rendered.files.len(),
        emit_diagnostics: emitted.len(),
        excluded,
    })
}

/// Deleting either the drop row or its typed carrier fails this compile. The union compiles
/// the carrier through `compile_to_resolved` and evaluates `exclusion` (review 77125).
const STDERR_CAPTURE_GAP_CARRIER_SOURCE: &str =
    include_str!("../../../../../dag/gunbc/stderr_capture_gap_exclusion.dag");
const STDERR_CAPTURE_POLICY_DROP_SOURCE: &str = include_str!(
    "../../../../../dag/gunbc/rung_drop/fixture_closure_union_unmodeled_stderr_capture.dag"
);

struct CaptureGapExclusion {
    drop_identity: String,
    declaring_module: String,
    service: String,
    operation_qualified: String,
    operation_bare: String,
}

fn record_field_string(
    ctx: &crate::v1_interpreter::InterpContext,
    fields: &[(crate::v1_interpreter::Symbol, crate::v1_interpreter::Value)],
    name: &str,
) -> Result<String, String> {
    use crate::v1_interpreter::Value;
    match fields
        .iter()
        .find(|(sym, _)| ctx.sym_eq(*sym, name))
        .map(|(_, v)| v)
    {
        Some(Value::Str(s)) => Ok(s.to_string()),
        Some(other) => Err(format!(
            "cause=CaptureGapExclusionFieldNotString field={name} value={other:?}"
        )),
        None => Err(format!(
            "cause=CaptureGapExclusionFieldMissing field={name}"
        )),
    }
}

fn load_capture_gap_exclusion() -> Result<CaptureGapExclusion, String> {
    let _drop_row_tether = STDERR_CAPTURE_POLICY_DROP_SOURCE;
    let files = vec![Rc::new(v1_compiler_compile::SourceFile {
        path: "dag/gunbc/stderr_capture_gap_exclusion.dag".to_string(),
        content: STDERR_CAPTURE_GAP_CARRIER_SOURCE.to_string(),
    })];
    let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
    let located = |d: &Rc<ErrorNode>| {
        format!(
            "module={} reason=`{}`",
            d.module_name,
            crate::v1_std_core::diagnostic_to_message(d.diagnostic.clone())
        )
    };
    if v1_compiler_compile::emittable_graph(resolved.clone()).is_none() {
        let blocking: Vec<String> = resolved
            .diagnostics
            .iter()
            .filter(|d| {
                crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())
            })
            .map(located)
            .collect();
        return Err(format!(
            "cause=CaptureGapExclusionUncompilable drop_row_bytes={} diagnostics={}",
            _drop_row_tether.len(),
            blocking.join(" | ")
        ));
    }
    let Some(graph) = resolved.graph.clone() else {
        return Err(format!(
            "cause=CaptureGapExclusionUncompilable drop_row_bytes={} -- resolved graph is none",
            _drop_row_tether.len()
        ));
    };
    let ctx = make_eval_context(
        &graph,
        resolved.source_indices.clone(),
        crate::v1_interpreter::ExecutionMode::Hermetic,
    );
    let val = crate::v1_interpreter::with_active_context(&ctx, || {
        match crate::v1_interpreter::eval_data_item_value(&ctx, "exclusion") {
            Ok(Some(v)) => Ok(Some(v)),
            Ok(None) => crate::v1_interpreter::eval_data_item_value(
                &ctx,
                "gunbc.stderr_capture_gap_exclusion.exclusion",
            ),
            Err(e) => Err(e),
        }
    })
    .map_err(|e| format!("cause=CaptureGapExclusionEvalFailed error={e}"))?
    .ok_or_else(|| "cause=CaptureGapExclusionDataMissing name=exclusion".to_string())?;
    let crate::v1_interpreter::Value::Record {
        ref fields,
        type_name,
    } = val
    else {
        return Err(format!("cause=CaptureGapExclusionNotRecord value={val:?}"));
    };
    if !ctx.sym_eq(type_name, "StderrCaptureGapExclusion")
        && !ctx.sym_eq(
            type_name,
            "gunbc.stderr_capture_gap_exclusion.StderrCaptureGapExclusion",
        )
    {
        return Err(format!(
            "cause=CaptureGapExclusionUnexpectedType type={}",
            ctx.resolve(type_name)
        ));
    }
    Ok(CaptureGapExclusion {
        drop_identity: record_field_string(&ctx, fields.as_slice(), "drop_identity")?,
        declaring_module: record_field_string(&ctx, fields.as_slice(), "declaring_module")?,
        service: record_field_string(&ctx, fields.as_slice(), "service")?,
        operation_qualified: record_field_string(&ctx, fields.as_slice(), "operation_qualified")?,
        operation_bare: record_field_string(&ctx, fields.as_slice(), "operation_bare")?,
    })
}

fn capture_gap_exclusion() -> Result<&'static CaptureGapExclusion, &'static str> {
    static GAP: OnceLock<Result<CaptureGapExclusion, String>> = OnceLock::new();
    match GAP.get_or_init(load_capture_gap_exclusion) {
        Ok(gap) => Ok(gap),
        Err(e) => Err(e.as_str()),
    }
}

/// Facts `v1.compiler.emit` `shell_emission_refusal_fact` renders for
/// `ShellChannelNotRealizedByTarget` on the three stderr-accounting channels the drop names.
/// Not every channel `shell_channel_realized_by_target` currently returns false for: a later
/// unrealized stdout-side channel must still refuse the union (review 77034).
fn rust_stderr_capture_channel_not_realized_facts() -> &'static BTreeSet<String> {
    static FACTS: OnceLock<BTreeSet<String>> = OnceLock::new();
    FACTS.get_or_init(|| {
        use crate::v1_compiler_artifact::RenderTarget;
        use crate::v1_compiler_emit::{
            render_target_name, shell_channel_realized_by_target, shell_emission_refusal_fact,
            shell_result_channel_key, ShellEmissionRefusal, ShellResultChannel,
        };
        let target = RenderTarget::Rust;
        let target_name = render_target_name(target);
        [
            ShellResultChannel::ShellChanStderrTruncated,
            ShellResultChannel::ShellChanStderrTotalBytes,
            ShellResultChannel::ShellChanStderrRetainedBytes,
        ]
        .into_iter()
        .filter(|channel| !shell_channel_realized_by_target(*channel, target))
        .map(|channel| {
            shell_emission_refusal_fact(Rc::new(
                ShellEmissionRefusal::ShellChannelNotRealizedByTarget {
                    key: shell_result_channel_key(channel),
                    target_name: target_name.clone(),
                },
            ))
        })
        .collect()
    })
}

/// The drop population: rust shell TransportEmissionNotModeled on exactly
/// `extdeps.gunbc` `WitnessBin.Run` whose fact equals a ShellChannelNotRealizedByTarget
/// fact for an unrealized rust channel. Any other module, service, or operation stays rendered.
fn stderr_capture_policy_gap_service(d: &Rc<ErrorNode>) -> Option<(String, String)> {
    let Ok(gap) = capture_gap_exclusion() else {
        return None;
    };
    match &*d.diagnostic {
        crate::v1_std_core::CompilerDiagnostic::TransportEmissionNotModeled {
            transport_kind,
            service,
            operation,
            declaring_module,
            target,
            missing_realization_fact,
            ..
        } if transport_kind == "shell"
            && target == "rust"
            && declaring_module.as_str() == gap.declaring_module
            && service.as_str() == gap.service
            && is_drop_run_operation(operation)
            && rust_stderr_capture_channel_not_realized_facts()
                .contains(missing_realization_fact) =>
        {
            Some((declaring_module.clone(), service.clone()))
        }
        _ => None,
    }
}

fn is_drop_run_operation(operation: &str) -> bool {
    let Ok(gap) = capture_gap_exclusion() else {
        return false;
    };
    operation == gap.operation_bare || operation == gap.operation_qualified
}

fn transport_emission_run(d: &Rc<ErrorNode>) -> Option<(String, String)> {
    match &*d.diagnostic {
        crate::v1_std_core::CompilerDiagnostic::TransportEmissionNotModeled {
            service,
            operation,
            declaring_module,
            ..
        } if is_drop_run_operation(operation) => Some((declaring_module.clone(), service.clone())),
        _ => None,
    }
}

/// The selection fold: among supplied unmodeled-transport rows, keep `(module, service)`
/// only when every Run row for that pair is a capture-gap fact. Production feeds it the
/// three emit diagnostic streams; tests supply rows.
fn select_run_operations_excluded_for_stderr_capture_gap(
    unmodeled: &[Rc<ErrorNode>],
) -> BTreeSet<(String, String)> {
    let mut gap_runs: BTreeSet<(String, String)> = BTreeSet::new();
    for d in unmodeled {
        if let Some(key) = stderr_capture_policy_gap_service(d) {
            gap_runs.insert(key);
        }
    }
    gap_runs
        .into_iter()
        .filter(|key| {
            unmodeled
                .iter()
                .filter(|d| transport_emission_run(d).as_ref() == Some(key))
                .all(|d| stderr_capture_policy_gap_service(d).is_some())
        })
        .collect()
}

/// `extdeps.gunbc` `gunbc.WitnessBin.Run` when that operation's unmodeled-transport refusals
/// are solely the rust stderr-capture gap. Other operations on the same service are not members.
fn run_operations_excluded_for_stderr_capture_gap(
    typed: &Rc<crate::v1_compiler_infer_items::ResolvedGraph>,
) -> BTreeSet<(String, String)> {
    let target = crate::v1_compiler_artifact::RenderTarget::Rust;
    let mut unmodeled = Vec::new();
    unmodeled.extend(
        crate::v1_compiler_emit::unmodeled_file_transport_diagnostics(typed.clone(), target)
            .iter()
            .cloned(),
    );
    unmodeled.extend(
        crate::v1_compiler_emit::unmodeled_shell_transport_diagnostics(typed.clone(), target)
            .iter()
            .cloned(),
    );
    unmodeled.extend(
        crate::v1_compiler_emit::unmodeled_rest_transport_diagnostics(typed.clone(), target)
            .iter()
            .cloned(),
    );
    select_run_operations_excluded_for_stderr_capture_gap(&unmodeled)
}

fn strip_excluded_run_operations(
    typed: Rc<crate::v1_compiler_infer_items::ResolvedGraph>,
    excluded: &BTreeSet<(String, String)>,
) -> Rc<crate::v1_compiler_infer_items::ResolvedGraph> {
    if excluded.is_empty() {
        return typed;
    }
    let modules = Rc::new(
        typed
            .modules
            .iter()
            .map(|tm| {
                let module_name = crate::v1_compiler_infer_env::authored_name(
                    tm.type_env.clone(),
                    tm.module.clone(),
                );
                let items = Rc::new(
                    tm.items
                        .iter()
                        .filter_map(|item| {
                            if item.module_item_kind
                                != crate::v1_std_core::ParsedModuleItemKind::ModuleItemService
                            {
                                return Some((*item).clone());
                            }
                            let service = crate::v1_compiler_infer_env::authored_name(
                                tm.type_env.clone(),
                                (*item).clone(),
                            );
                            if !excluded.contains(&(module_name.clone(), service)) {
                                return Some((*item).clone());
                            }
                            let kept: im::Vector<_> = item
                                .children
                                .iter()
                                .filter(|op| {
                                    !is_drop_run_operation(
                                        &crate::v1_compiler_infer_env::authored_name(
                                            tm.type_env.clone(),
                                            (*op).clone(),
                                        ),
                                    )
                                })
                                .cloned()
                                .collect();
                            if kept.is_empty() {
                                return None;
                            }
                            Some(Rc::new({
                                let mut node = (**item).clone();
                                node.children = Rc::new(kept);
                                node
                            }))
                        })
                        .collect::<im::Vector<_>>(),
                );
                Rc::new(crate::v1_compiler_infer_items::TypedModule {
                    items,
                    ..(**tm).clone()
                })
            })
            .collect::<im::Vector<_>>(),
    );
    Rc::new(crate::v1_compiler_infer_items::ResolvedGraph {
        modules,
        ..(*typed).clone()
    })
}

/// Compile stays the full closure. Emit strips only `gunbc.WitnessBin.Run` when its rust
/// refusals are solely the capture-policy gap. Other operations on that service stay.
fn union_emit_graph_excluding_unmodeled_stderr_capture(
    resolved: Rc<v1_compiler_compile::ResolvedPipelineResult>,
) -> (Rc<v1_compiler_compile::ResolvedPipelineResult>, Vec<String>) {
    let Some(typed) = resolved.graph.clone() else {
        return (resolved, Vec::new());
    };
    let excluded_keys = run_operations_excluded_for_stderr_capture_gap(&typed);
    if excluded_keys.is_empty() {
        return (resolved, Vec::new());
    }
    let Ok(gap) = capture_gap_exclusion() else {
        return (resolved, Vec::new());
    };
    let excluded: Vec<String> = excluded_keys
        .iter()
        .map(|(module, service)| {
            format!(
                "module={module} service={service} operation={} \
                 cause=ShellChannelNotRealizedByTarget \
                 fact=stderr_capture_policy_unrealized drop={}",
                gap.operation_qualified, gap.drop_identity,
            )
        })
        .collect();
    let graph = strip_excluded_run_operations(typed, &excluded_keys);
    (
        Rc::new(v1_compiler_compile::ResolvedPipelineResult {
            graph: Some(graph),
            ..(*resolved).clone()
        }),
        excluded,
    )
}

/// The red control's member: a non-tail effectful self-call, which the rust emitter refuses
/// (`EffectfulSelfRecursionUnrealized`, the derived form exercised by
/// `test.claim.effectful_item_kind_collapse_witness_test`).
const FIXTURE_CLOSURE_UNION_RED_MEMBER: &str = "module efr_member\nimport extdeps.filesystem.filesystem_io { Filesystem }\nfn walk(n: Int) -> Int {\n  let listed = Filesystem.List(path: \".\")\n  if n == 0 { 0 } else if listed.success { n + walk(n: n - 1) } else { 0 }\n}\n";

/// The positive control's member: the same closure with the self-call in tail position, which is
/// lowered to a loop and renders clean.
const FIXTURE_CLOSURE_UNION_CLEAN_MEMBER: &str = "module efr_member\nimport extdeps.filesystem.filesystem_io { Filesystem }\nfn walk(n: Int) -> Int {\n  let listed = Filesystem.List(path: \".\")\n  if n == 0 { 0 } else if listed.success { walk(n: n - 1) } else { 0 }\n}\n";

const FIXTURE_CLOSURE_UNION_CONTROL_PATH: &str = "dag/fixture_closure_union_control/efr_member.dag";

/// A union built the way a fixture compile builds its closure: the member's imports resolved over
/// the live module index, the member itself given a corpus path so it is a union MEMBER.
pub(crate) fn fixture_closure_union_control_union(
    content: &str,
) -> Result<FixtureClosureUnion, String> {
    let module_index = build_module_path_index_from_witness_roots();
    let mut union = FixtureClosureUnion::default();
    for source in resolve_virtual_source_with_imports(
        FIXTURE_CLOSURE_UNION_CONTROL_PATH,
        content,
        &module_index,
    )? {
        union
            .members
            .insert(source.path.clone(), source.content.clone());
    }
    Ok(union)
}

/// THE ENROLLED CONTROLS FOR [`fixture_closure_union_emit_receipt`], run by the required floor on
/// every run before the union renders (review 76399; DESIGN §4b(4): the discriminating red and the
/// positive control stay enrolled on the acceptance path, not only in cargo unit tests, which run on
/// no CI path). The red member must refuse as `FixtureClosureUnionEmitRefused` located at
/// `module=efr_member`; the clean member must hold. Either failing refuses the floor, so a later
/// change that disables the refusal arm or breaks the clean render is a required red.
/// Returns (red wall ms, clean wall ms) for the caller's log line.
pub(crate) fn fixture_closure_union_controls() -> Result<(u128, u128), String> {
    let refuse = |what: String| {
        format!(
            "REQUIRED-FLOOR REFUSAL cause=FixtureClosureUnionControlFailed \
             receipt=fixture_closure_union_controls -- {what}"
        )
    };
    let red_started = std::time::Instant::now();
    match fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_UNION_RED_MEMBER).map_err(&refuse)?,
    ) {
        Err(refusal)
            if refusal.contains("cause=FixtureClosureUnionEmitRefused")
                && refusal.contains("module=efr_member") => {}
        Err(other) => {
            return Err(refuse(format!(
                "the red member refused for the wrong reason: {other}"
            )))
        }
        Ok(observed) => {
            return Err(refuse(format!(
                "the red member's emit refusal did not refuse the union: {observed:?}"
            )))
        }
    }
    let red_ms = red_started.elapsed().as_millis();
    let clean_started = std::time::Instant::now();
    fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_UNION_CLEAN_MEMBER)
            .map_err(&refuse)?,
    )
    .map_err(|refusal| refuse(format!("the clean member did not hold: {refusal}")))?;
    // THE CLOSURE CONTROLS (#13437). The fixture closure is the corpus closure of what the
    // fixture imports, not its import edges alone. `std.syllogism` imports nothing and reaches
    // `std.graph` by the bare names `GraphEdge` / `CallGraph`, so an import-only walk compiles
    // it without its provider and the union refuses at `module=std.syllogism` (#13420's refusal).
    fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_REFERENCE_REACH_MEMBER)
            .map_err(&refuse)?,
    )
    .map_err(|refusal| {
        refuse(format!(
            "a provider reached only by reference is missing from the fixture closure: {refusal}"
        ))
    })?;
    // THE QUALIFIED-REFERENCE EDGE. The same walker must follow a dotted module-path
    // reference, not only a bare name: `test.fixture.reference_derived_graph.consumer_reference_only`
    // imports `std.types` and names `test.fixture.reference_derived_graph.provider.provided_value`
    // qualified, so an import-only walk compiles it without its provider. The pair is small
    // (no `v2.std.node`); the large corpus specimens (`v2.std.artifact` → `v2.std.refinement`)
    // are the same edge at a cost the floor control must not pay.
    fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_QUALIFIED_REFERENCE_REACH_MEMBER)
            .map_err(&refuse)?,
    )
    .map_err(|refusal| {
        refuse(format!(
            "a provider reached only by qualified reference is missing from the fixture closure: {refusal}"
        ))
    })?;
    // And the closure fix closes providers without narrowing what refuses: a real error in a
    // member of the fixture's own closure still refuses, located at that member.
    match fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_REAL_ERROR_MEMBER).map_err(&refuse)?,
    ) {
        Err(refusal)
            if refusal.contains("cause=FixtureClosureUnionUncompilable")
                && refusal.contains("module=efr_member") => {}
        Err(other) => {
            return Err(refuse(format!(
                "the real-error member refused for the wrong reason: {other}"
            )))
        }
        Ok(observed) => {
            return Err(refuse(format!(
                "a real error in the fixture closure did not refuse the union: {observed:?}"
            )))
        }
    }
    // A fixture whose closure reaches extdeps.gunbc is admitted with the typed capture-policy
    // exclusion, not as FixtureClosureUnionEmitRefused (the #13420 floor after #13437).
    let gunbc_observed = fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_GUNBC_REACH_MEMBER)
            .map_err(&refuse)?,
    )
    .map_err(|refusal| {
        refuse(format!(
            "a fixture whose closure reaches extdeps.gunbc refused: {refusal}"
        ))
    })?;
    if !gunbc_observed.excluded.iter().any(|row| {
        let Ok(gap) = capture_gap_exclusion() else {
            return false;
        };
        row.contains(&format!("module={}", gap.declaring_module))
            && row.contains(&format!("operation={}", gap.operation_qualified))
            && row.contains("cause=ShellChannelNotRealizedByTarget")
            && row.contains(&gap.drop_identity)
    }) {
        return Err(refuse(format!(
            "a fixture whose closure reaches extdeps.gunbc was admitted without the typed exclusion: {gunbc_observed:?}"
        )));
    }
    // A real emit error in a member the union still renders still refuses, even when the same
    // closure also reaches the excluded service.
    match fixture_closure_union_emit_receipt(
        &fixture_closure_union_control_union(FIXTURE_CLOSURE_GUNBC_AND_REAL_EMIT_ERROR)
            .map_err(&refuse)?,
    ) {
        Err(refusal)
            if refusal.contains("cause=FixtureClosureUnionEmitRefused")
                && refusal.contains("module=efr_member") => {}
        Err(other) => {
            return Err(refuse(format!(
                "the gunbc-plus-real-emit-error member refused for the wrong reason: {other}"
            )))
        }
        Ok(observed) => {
            return Err(refuse(format!(
                "a real emit error beside the excluded gunbc service did not refuse the union: {observed:?}"
            )))
        }
    }
    Ok((red_ms, clean_started.elapsed().as_millis()))
}

/// A member reaching `std.graph` only through `std.syllogism`'s bare references.
const FIXTURE_CLOSURE_REFERENCE_REACH_MEMBER: &str = "module efr_member\nimport std.syllogism { Argument, argument_is_acyclic }\nfn acyclic(a: Argument) -> Bool {\n  argument_is_acyclic(a)\n}\n";

/// A member reaching `test.fixture.reference_derived_graph.provider` only through that
/// consumer's dotted qualified reference (no import of the provider, no bare name).
const FIXTURE_CLOSURE_QUALIFIED_REFERENCE_REACH_MEMBER: &str = "module efr_member\nimport test.fixture.reference_derived_graph.consumer_reference_only { uses_provider }\nfn probe() -> Int {\n  uses_provider()\n}\n";

/// The same closure with a call to a function nothing declares.
const FIXTURE_CLOSURE_REAL_ERROR_MEMBER: &str = "module efr_member\nimport std.syllogism { Argument }\nfn broken(a: Argument) -> Bool {\n  no_such_function_anywhere(a)\n}\n";

/// A member that imports a type from extdeps.gunbc, so the fixture closure contains WitnessBin.Run.
const FIXTURE_CLOSURE_GUNBC_REACH_MEMBER: &str = "module efr_member\nimport extdeps.gunbc { CrateRole }\nfn keep(r: CrateRole) -> CrateRole {\n  r\n}\n";

/// The gunbc-reaching closure with a non-tail effectful self-call the rust emitter still refuses.
const FIXTURE_CLOSURE_GUNBC_AND_REAL_EMIT_ERROR: &str = "module efr_member\nimport extdeps.gunbc { CrateRole }\nimport extdeps.filesystem.filesystem_io { Filesystem }\nfn walk(n: Int) -> Int {\n  let listed = Filesystem.List(path: \".\")\n  if n == 0 { 0 } else if listed.success { n + walk(n: n - 1) } else { 0 }\n}\n";

#[cfg(test)]
mod fixture_closure_union_tests {
    use super::*;

    /// The recorder and the union are process-wide; tests that touch them run one at a time.
    static UNION_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn gap_excl() -> &'static CaptureGapExclusion {
        capture_gap_exclusion().unwrap_or_else(|e| panic!("{e}"))
    }

    /// An empty union is a bypassed recording seam and refuses (review 76399).
    #[test]
    fn an_empty_union_refuses() {
        let refusal = fixture_closure_union_emit_receipt(&FixtureClosureUnion::default())
            .expect_err("an empty union must refuse");
        assert!(
            refusal.contains("cause=FixtureClosureUnionEmpty"),
            "{refusal}"
        );
    }

    /// One path recorded with two contents is refused, never resolved by picking one.
    #[test]
    fn a_path_recorded_with_two_contents_refuses() {
        let mut union = FixtureClosureUnion::default();
        union.conflicts.insert("dag/x.dag".to_string());
        let refusal = fixture_closure_union_emit_receipt(&union).expect_err("conflict refuses");
        assert!(
            refusal.contains("cause=FixtureClosureUnionConflict"),
            "{refusal}"
        );
    }

    /// A LATER RUN IN THE SAME PROCESS WHOSE FIXTURE CALLS ALL HIT THE MEMO still reaches the
    /// union: the hit replays the closure its fill recorded (review 76318). Without the replay the
    /// second union is empty and the receipt would hold having rendered nothing.
    #[test]
    fn a_memo_hit_on_a_later_run_replays_its_closure() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let file = |path: &str, content: &str| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        };
        drop(take_fixture_closure_union());
        begin_fixture_closure_fill();
        record_fixture_closure(&[
            file("dag/replay_a.dag", "module replay_a\n"),
            file(FIXTURE_SOURCE_PATH, "x"),
        ]);
        finish_fixture_closure_fill("replay-test-key");
        drop(take_fixture_closure_union());
        record_fixture_closure_memo_hit("replay-test-key");
        let second_run = take_fixture_closure_union();
        assert_eq!(second_run.fixture_compiles, 0);
        assert_eq!(second_run.memo_hits, 1);
        assert_eq!(
            second_run.members.keys().collect::<Vec<_>>(),
            vec!["dag/replay_a.dag"]
        );
    }

    /// A FILE THAT CHANGED BETWEEN RUNS: a hit on run 1's key replays run 1's bytes and a fresh fill
    /// on run 2 records the new bytes, each under its own key. Path-keyed replay would hand the
    /// second key the first fill's bytes (review 76336).
    #[test]
    fn a_replay_returns_its_own_fills_bytes() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let file = |path: &str, content: &str| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        };
        drop(take_fixture_closure_union());
        begin_fixture_closure_fill();
        record_fixture_closure(&[file("dag/replay_b.dag", "module replay_b\n// v1\n")]);
        finish_fixture_closure_fill("replay-bytes-key-1");
        begin_fixture_closure_fill();
        record_fixture_closure(&[file("dag/replay_b.dag", "module replay_b\n// v2\n")]);
        finish_fixture_closure_fill("replay-bytes-key-2");
        drop(take_fixture_closure_union());
        record_fixture_closure_memo_hit("replay-bytes-key-2");
        let run = take_fixture_closure_union();
        assert!(run.conflicts.is_empty(), "{:?}", run.conflicts);
        assert_eq!(
            run.members.get("dag/replay_b.dag").map(String::as_str),
            Some("module replay_b\n// v2\n")
        );
    }

    /// The recorder skips the fixture source itself and keeps a content conflict.
    #[test]
    fn the_recorder_excludes_the_fixture_and_keeps_conflicts() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        drop(take_fixture_closure_union());
        let file = |path: &str, content: &str| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        };
        record_fixture_closure(&[
            file("dag/a.dag", "module a\n"),
            file(FIXTURE_SOURCE_PATH, "x"),
        ]);
        record_fixture_closure(&[file("dag/a.dag", "module a2\n")]);
        let union = take_fixture_closure_union();
        assert_eq!(union.fixture_compiles, 2);
        assert_eq!(union.members.keys().collect::<Vec<_>>(), vec!["dag/a.dag"]);
        assert!(union.conflicts.contains("dag/a.dag"));
    }

    /// RED: extending a fixture through the process-shared index left that fixture's
    /// both-closure in the caches the claim fold reads. GREEN: the same walk still
    /// closes (syllogism reaches its reference provider) and the shared typed cache
    /// and both-closure edge map do not grow.
    #[test]
    fn fixture_closure_extension_does_not_populate_the_process_shared_index() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let layers = crate::cli_run::witness_layer_roots();
        let shared = super::entry_resolve::try_process_shared_index(&layers)
            .unwrap_or_else(|e| panic!("process-shared index is the subject of this control: {e}"));
        let typed_before = shared.typed_module_cache.borrow().len();
        let edges_before = shared
            .both_closure_edges
            .borrow()
            .as_ref()
            .map(|e| e.ref_out.len())
            .unwrap_or(0);
        let admissions_before = shared.bare_reference_admission.borrow().len();
        let module_index = crate::cli_run::build_module_path_index_from_witness_roots();
        let sources = crate::cli_run::resolve_virtual_source_with_imports(
            FIXTURE_SOURCE_PATH,
            FIXTURE_CLOSURE_REFERENCE_REACH_MEMBER,
            &module_index,
        )
        .unwrap_or_else(|e| panic!("fixture closure must still close: {e}"));
        assert!(
            sources.len() > 1,
            "the syllogism specimen must pull its provider, got {}",
            sources.len()
        );
        let typed_after = shared.typed_module_cache.borrow().len();
        let edges_after = shared
            .both_closure_edges
            .borrow()
            .as_ref()
            .map(|e| e.ref_out.len())
            .unwrap_or(0);
        let admissions_after = shared.bare_reference_admission.borrow().len();
        assert_eq!(
            typed_before, typed_after,
            "fixture closure must not admit typed-cache rows on the process-shared index"
        );
        assert_eq!(
            edges_before, edges_after,
            "fixture closure must not grow both_closure_edges on the process-shared index"
        );
        assert_eq!(
            admissions_before, admissions_after,
            "fixture closure must not grow bare-reference admission on the process-shared index"
        );
    }

    /// A second fixture extend of the same specimen must reuse the fixture-only slot
    /// (both_closure_edges already filled), not cold-extend again.
    #[test]
    fn second_fixture_extend_reuses_the_scratch_slot() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let module_index = crate::cli_run::build_module_path_index_from_witness_roots();
        crate::cli_run::resolve_virtual_source_with_imports(
            FIXTURE_SOURCE_PATH,
            FIXTURE_CLOSURE_REFERENCE_REACH_MEMBER,
            &module_index,
        )
        .unwrap_or_else(|e| panic!("first fixture extend must close: {e}"));
        let scratch = crate::cli_run::fixture_extension_index_for_test()
            .expect("the production loader must install the fixture-only slot");
        let edges_after_first = scratch
            .both_closure_edges
            .borrow()
            .as_ref()
            .map(|e| e.ref_out.len())
            .unwrap_or(0);
        assert!(
            edges_after_first > 0,
            "the first extend must populate the fixture-only edge map"
        );
        crate::cli_run::resolve_virtual_source_with_imports(
            FIXTURE_SOURCE_PATH,
            FIXTURE_CLOSURE_REFERENCE_REACH_MEMBER,
            &module_index,
        )
        .unwrap_or_else(|e| panic!("second fixture extend must close: {e}"));
        let same = crate::cli_run::fixture_extension_index_for_test()
            .expect("the fixture-only slot must survive the second extend");
        let edges_after_second = same
            .both_closure_edges
            .borrow()
            .as_ref()
            .map(|e| e.ref_out.len())
            .unwrap_or(0);
        assert_eq!(
            edges_after_first, edges_after_second,
            "a second extend of the same specimen must hit the fixture-only caches, not rebuild them"
        );
        assert!(
            Rc::ptr_eq(&scratch, &same),
            "the fixture-only slot must be one index, not a fresh shell per compile"
        );
    }

    /// THE DISCRIMINATING RED of the class: the pre-fix loader
    /// (`try_index_for_run_or_owned_pool` over the layer roots) grows `both_closure_edges`
    /// on the process-shared index. If this greens, the green control above has no red.
    #[test]
    fn fixture_closure_extension_via_shared_index_grows_both_closure_edges() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let layers = crate::cli_run::witness_layer_roots();
        let shared = super::entry_resolve::try_process_shared_index(&layers)
            .unwrap_or_else(|e| panic!("process-shared index is the subject of this control: {e}"));
        let edges_before = shared
            .both_closure_edges
            .borrow()
            .as_ref()
            .map(|e| e.ref_out.len())
            .unwrap_or(0);
        let module_index = crate::cli_run::build_module_path_index_from_witness_roots();
        let sources = crate::cli_run::extend_fixture_imports_on_process_shared_index(
            FIXTURE_CLOSURE_REFERENCE_REACH_MEMBER,
            &module_index,
        )
        .unwrap_or_else(|e| panic!("pre-fix shared-index extension must still close: {e}"));
        assert!(
            sources.len() > 1,
            "the syllogism specimen must pull its provider, got {}",
            sources.len()
        );
        let edges_after = shared
            .both_closure_edges
            .borrow()
            .as_ref()
            .map(|e| e.ref_out.len())
            .unwrap_or(0);
        assert!(
            edges_after > edges_before,
            "the pre-fix route must grow both_closure_edges (before={edges_before} after={edges_after})"
        );
    }

    /// PLAN-TIME CLOSURE of the forged-probe witness module (the changed-witness seed), not
    /// the in-memory probe string. If MegaRAC production paths appear, planning that witness
    /// compiled the string's imports as both-closure edges and the pin at fold-start is that
    /// increment. If they do not, the increment is the typed graph of this module itself.
    #[test]
    fn forged_probe_witness_module_both_closure_excludes_string_literal_imports() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let layers = crate::cli_run::witness_layer_roots();
        let shared = super::entry_resolve::try_process_shared_index(&layers)
            .unwrap_or_else(|e| panic!("process-shared index is the subject of this control: {e}"));
        let scratch = super::entry_resolve::new_multi_entry_index_scratch_over(
            shared.source_files.clone(),
            &shared.source_roots,
        );
        let rel = "dag/test/claim/host/megarac_managed_host_forged_probe_witness_test.dag";
        let content = std::fs::read_to_string(process_workspace_root().join(rel))
            .unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let source = Rc::new(v1_compiler_compile::SourceFile {
            path: rel.to_string(),
            content,
        });
        let closed =
            crate::cli_run::extend_sources_to_both_closure_fixpoint(vec![source], &scratch)
                .unwrap_or_else(|e| panic!("witness-module closure must close: {e}"));
        let megarac: Vec<String> = closed
            .iter()
            .map(|s| s.path.replace('\\', "/"))
            .filter(|p| p.contains("megarac") && p != rel)
            .collect();
        assert!(
            megarac.is_empty(),
            "planning the witness must not both-close MegaRAC production named only inside \
             forged_probe_source; pulled {megarac:?} (closure_len={})",
            closed.len()
        );
    }

    /// Scratch over an already-indexed name set must not register as a second
    /// `MultiEntryIndex` of that set (`MultiEntryIndexBuiltTwiceForOneNameSet`).
    #[test]
    fn fixture_scratch_shell_is_not_a_second_index_of_the_name_set() {
        let _serial = UNION_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let layers = crate::cli_run::witness_layer_roots();
        let shared = super::entry_resolve::try_process_shared_index(&layers)
            .unwrap_or_else(|e| panic!("process-shared index is the subject of this control: {e}"));
        let before = crate::cli_run::multi_entry_index_builds().len();
        let _ = super::entry_resolve::new_multi_entry_index_scratch_over(
            shared.source_files.clone(),
            &shared.source_roots,
        );
        let _ = super::entry_resolve::new_multi_entry_index_scratch_over(
            shared.source_files.clone(),
            &shared.source_roots,
        );
        assert_eq!(
            crate::cli_run::multi_entry_index_builds().len(),
            before,
            "a scratch shell is empty caches over an indexed name set, not another index"
        );
    }

    #[test]
    fn capture_gap_keys_on_shell_channel_not_realized_fact_equality() {
        use crate::v1_compiler_emit::{shell_emission_refusal_fact, ShellEmissionRefusal};
        use crate::v1_std_core::{make_error_node, CompilerDiagnostic};
        let span = crate::v1_std_core::kernel_span("probe".to_string());
        let mk = |fact: String| {
            let gap = gap_excl();
            make_error_node(
                Rc::new(CompilerDiagnostic::TransportEmissionNotModeled {
                    transport_kind: "shell".to_string(),
                    service: gap.service.to_string(),
                    operation: gap.operation_qualified.to_string(),
                    declaring_module: gap.declaring_module.to_string(),
                    target: "rust".to_string(),
                    missing_realization_fact: fact,
                    span: span.clone(),
                }),
                gap.declaring_module.to_string(),
            )
        };
        let gap = shell_emission_refusal_fact(Rc::new(
            ShellEmissionRefusal::ShellChannelNotRealizedByTarget {
                key: "stderr_truncated".to_string(),
                target_name: "rust".to_string(),
            },
        ));
        let unmodeled_key =
            shell_emission_refusal_fact(Rc::new(ShellEmissionRefusal::ShellOutputKeyNotModeled {
                key: "not_a_channel".to_string(),
            }));
        let substring_poison =
            "unmodeled key 'not_a_channel' implements no stderr capture policy".to_string();
        let stdout_unrealized = shell_emission_refusal_fact(Rc::new(
            ShellEmissionRefusal::ShellChannelNotRealizedByTarget {
                key: "stdout".to_string(),
                target_name: "rust".to_string(),
            },
        ));
        assert!(stderr_capture_policy_gap_service(&mk(gap.clone())).is_some());
        assert!(stderr_capture_policy_gap_service(&mk(unmodeled_key)).is_none());
        assert!(stderr_capture_policy_gap_service(&mk(substring_poison)).is_none());
        assert!(stderr_capture_policy_gap_service(&mk(stdout_unrealized)).is_none());
        let other_module = make_error_node(
            Rc::new(CompilerDiagnostic::TransportEmissionNotModeled {
                transport_kind: "shell".to_string(),
                service: gap_excl().service.to_string(),
                operation: gap_excl().operation_qualified.to_string(),
                declaring_module: "extdeps.other".to_string(),
                target: "rust".to_string(),
                missing_realization_fact: gap,
                span: span.clone(),
            }),
            "extdeps.other".to_string(),
        );
        assert!(stderr_capture_policy_gap_service(&other_module).is_none());
    }

    fn gap_fact() -> String {
        use crate::v1_compiler_emit::{shell_emission_refusal_fact, ShellEmissionRefusal};
        shell_emission_refusal_fact(Rc::new(
            ShellEmissionRefusal::ShellChannelNotRealizedByTarget {
                key: "stderr_truncated".to_string(),
                target_name: "rust".to_string(),
            },
        ))
    }

    fn transport_row(
        module: &str,
        service: &str,
        operation: &str,
        fact: String,
    ) -> Rc<crate::v1_std_core::ErrorNode> {
        crate::v1_std_core::make_error_node(
            Rc::new(
                crate::v1_std_core::CompilerDiagnostic::TransportEmissionNotModeled {
                    transport_kind: "shell".to_string(),
                    service: service.to_string(),
                    operation: operation.to_string(),
                    declaring_module: module.to_string(),
                    target: "rust".to_string(),
                    missing_realization_fact: fact,
                    span: crate::v1_std_core::kernel_span("probe".to_string()),
                },
            ),
            module.to_string(),
        )
    }

    #[test]
    fn selection_fold_allows_only_run_capture_rows_and_vetoes_mixed_refusal() {
        let gap = gap_fact();
        let gap_row = gap_excl();
        let allowed = transport_row(
            &gap_row.declaring_module,
            &gap_row.service,
            &gap_row.operation_qualified,
            gap.clone(),
        );
        let selected = select_run_operations_excluded_for_stderr_capture_gap(&[allowed]);
        assert_eq!(
            selected.iter().cloned().collect::<Vec<_>>(),
            vec![(
                gap_row.declaring_module.to_string(),
                gap_row.service.to_string()
            )]
        );

        let mixed_key = crate::v1_compiler_emit::shell_emission_refusal_fact(Rc::new(
            crate::v1_compiler_emit::ShellEmissionRefusal::ShellOutputKeyNotModeled {
                key: "not_a_channel".to_string(),
            },
        ));
        let mixed = vec![
            transport_row(
                &gap_row.declaring_module,
                &gap_row.service,
                &gap_row.operation_qualified,
                gap.clone(),
            ),
            transport_row(
                &gap_row.declaring_module,
                &gap_row.service,
                &gap_row.operation_qualified,
                mixed_key,
            ),
        ];
        assert!(select_run_operations_excluded_for_stderr_capture_gap(&mixed).is_empty());

        let wrong_module = transport_row(
            "extdeps.other",
            &gap_row.service,
            &gap_row.operation_qualified,
            gap.clone(),
        );
        assert!(select_run_operations_excluded_for_stderr_capture_gap(&[wrong_module]).is_empty());

        let nonmember = transport_row(&gap_row.declaring_module, &gap_row.service, "Sibling", gap);
        assert!(select_run_operations_excluded_for_stderr_capture_gap(&[nonmember]).is_empty());
    }

    fn kernel_named(
        name: &str,
        kind: crate::v1_std_core::ParsedModuleItemKind,
        children: im::Vector<Rc<crate::v1_std_core::Node>>,
    ) -> Rc<crate::v1_std_core::Node> {
        let mut node = (*crate::v1_std_core::leaf_node_with_span(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            name.to_string(),
            crate::v1_std_core::kernel_span(name.to_string()),
        ))
        .clone();
        node.module_item_kind = kind;
        node.children = Rc::new(children);
        Rc::new(node)
    }

    fn supplied_graph_with_run_and_sibling() -> Rc<crate::v1_compiler_infer_items::ResolvedGraph> {
        use crate::v1_compiler_infer_items::{
            ModuleInterface, ModuleTypecheckProgress, ResolvedGraph, TypedModule,
        };
        use crate::v1_std_core::ParsedModuleItemKind;
        let env = crate::v1_compiler_infer_env::empty_type_env();
        let cache = crate::v1_compiler_infer_env::empty_type_env_cache();
        let gap = gap_excl();
        let run = kernel_named(
            &gap.operation_bare,
            ParsedModuleItemKind::NotAModuleItem,
            im::vector![],
        );
        let sibling = kernel_named(
            "Sibling",
            ParsedModuleItemKind::NotAModuleItem,
            im::vector![],
        );
        let other_fn = kernel_named(
            "unrelated_fn",
            ParsedModuleItemKind::ModuleItemFunction,
            im::vector![],
        );
        let service = kernel_named(
            &gap.service,
            ParsedModuleItemKind::ModuleItemService,
            im::vector![run, sibling],
        );
        let module = kernel_named(
            &gap.declaring_module,
            ParsedModuleItemKind::NotAModuleItem,
            im::vector![],
        );
        let other_module_node = kernel_named(
            "other.mod",
            ParsedModuleItemKind::NotAModuleItem,
            im::vector![],
        );
        let other_item = kernel_named(
            "KeepMe",
            ParsedModuleItemKind::ModuleItemFunction,
            im::vector![],
        );
        let interface = |e: Rc<crate::v1_compiler_infer_env::TypeEnv>,
                         c: Rc<crate::v1_compiler_infer_env::TypeEnvCache>,
                         path: &str| {
            Rc::new(ModuleInterface {
                summary: Rc::new(crate::std_interface_summary::InterfaceSummary {
                    module_path: path.to_string(),
                    exports: Rc::new(im::vector![]),
                    interface_hash: crate::std_interface_summary::interface_summary_rollup(
                        Rc::new(im::vector![]),
                    ),
                }),
                env: e,
                cache: c,
            })
        };
        let tm = Rc::new(TypedModule {
            module,
            items: Rc::new(im::vector![service, other_fn]),
            progress: ModuleTypecheckProgress::ItemsChecked,
            type_env: env.clone(),
            type_env_cache: cache.clone(),
            interface: interface(env.clone(), cache.clone(), &gap.declaring_module),
            func_env: Rc::new(crate::v1_compiler_infer_sigs::ResolvedFuncEnv {
                name: gap.declaring_module.to_string(),
                local: crate::v1_rt::rc_empty_map(),
                parents: Rc::new(im::vector![]),
            }),
            item_registry: crate::v1_rt::rc_empty_map(),
            occurrence_transport: None,
        });
        let other = Rc::new(TypedModule {
            module: other_module_node,
            items: Rc::new(im::vector![other_item]),
            progress: ModuleTypecheckProgress::ItemsChecked,
            type_env: env.clone(),
            type_env_cache: cache.clone(),
            interface: interface(env, cache, "other.mod"),
            func_env: Rc::new(crate::v1_compiler_infer_sigs::ResolvedFuncEnv {
                name: "other.mod".to_string(),
                local: crate::v1_rt::rc_empty_map(),
                parents: Rc::new(im::vector![]),
            }),
            item_registry: crate::v1_rt::rc_empty_map(),
            occurrence_transport: None,
        });
        Rc::new(ResolvedGraph {
            modules: Rc::new(im::vector![tm, other]),
            item_registry: crate::v1_rt::rc_empty_map(),
            item_leaf_owner_modules: crate::v1_rt::rc_empty_map(),
            diagnostics: Rc::new(im::vector![]),
        })
    }

    fn service_op_names(
        graph: &crate::v1_compiler_infer_items::ResolvedGraph,
        module: &str,
        service: &str,
    ) -> Vec<String> {
        let env = crate::v1_compiler_infer_env::empty_type_env();
        graph
            .modules
            .iter()
            .find(|tm| {
                crate::v1_compiler_infer_env::authored_name(tm.type_env.clone(), tm.module.clone())
                    == module
            })
            .into_iter()
            .flat_map(|tm| tm.items.iter())
            .filter(|item| {
                item.module_item_kind == crate::v1_std_core::ParsedModuleItemKind::ModuleItemService
                    && crate::v1_compiler_infer_env::authored_name(env.clone(), (*item).clone())
                        == service
            })
            .flat_map(|item| {
                item.children
                    .iter()
                    .map(|op| crate::v1_compiler_infer_env::authored_name(env.clone(), op.clone()))
            })
            .collect()
    }

    #[test]
    fn strip_removes_only_run_and_keeps_sibling_and_unrelated_items() {
        let gap = gap_excl();
        let graph = supplied_graph_with_run_and_sibling();
        let mut excluded = BTreeSet::new();
        excluded.insert((gap.declaring_module.to_string(), gap.service.to_string()));
        let stripped = strip_excluded_run_operations(graph.clone(), &excluded);
        assert_eq!(
            service_op_names(&stripped, &gap.declaring_module, &gap.service),
            vec!["Sibling".to_string()]
        );
        let env = crate::v1_compiler_infer_env::empty_type_env();
        let gunbc_item_names: Vec<String> = stripped
            .modules
            .iter()
            .find(|tm| {
                crate::v1_compiler_infer_env::authored_name(tm.type_env.clone(), tm.module.clone())
                    == gap.declaring_module
            })
            .unwrap()
            .items
            .iter()
            .map(|item| crate::v1_compiler_infer_env::authored_name(env.clone(), item.clone()))
            .collect();
        assert!(
            gunbc_item_names.contains(&"unrelated_fn".to_string()),
            "{gunbc_item_names:?}"
        );
        assert_eq!(stripped.modules.len(), 2, "unrelated module must remain");
        let empty = strip_excluded_run_operations(graph, &BTreeSet::new());
        assert_eq!(
            service_op_names(&empty, &gap.declaring_module, &gap.service),
            vec![gap.operation_bare.to_string(), "Sibling".to_string()]
        );
    }

    #[test]
    fn capture_gap_exclusion_reads_typed_fields_from_the_drop_module() {
        let gap = gap_excl();
        assert!(!gap.drop_identity.is_empty());
        assert!(!gap.declaring_module.is_empty());
        assert!(!gap.service.is_empty());
        assert!(!gap.operation_bare.is_empty());
        assert!(
            gap.operation_qualified == gap.operation_bare
                || gap
                    .operation_qualified
                    .ends_with(&format!(".{}", gap.operation_bare)),
            "qualified={} bare={}",
            gap.operation_qualified,
            gap.operation_bare
        );
    }
}

#[cfg(test)]
mod emit_check_read_tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// One shared read answers every assertion set independently: the memo key no longer
    /// carries `includes`/`excludes`, so this is what keeps a hit from lying about them.
    #[test]
    fn one_read_discriminates_each_assertion_set() {
        let read = EmitCheckRead::Content(Rc::from("pub fn alpha() {}"));
        assert!(emit_check_verdict(&read, &strings(&["alpha"]), &[]));
        assert!(!emit_check_verdict(&read, &strings(&["beta"]), &[]));
        assert!(!emit_check_verdict(&read, &[], &strings(&["alpha"])));
        assert!(emit_check_verdict(&read, &[], &strings(&["beta"])));
        assert!(!emit_check_verdict(
            &EmitCheckRead::HardDiagnostics,
            &[],
            &[]
        ));
    }

    #[test]
    fn memo_key_is_over_source_and_render_only() {
        let a = compile_dag_rust_emit_check_memo_key("s", "src/a.rs", "inv");
        assert_eq!(
            a,
            compile_dag_rust_emit_check_memo_key("s", "src/a.rs", "inv")
        );
        assert_ne!(
            a,
            compile_dag_rust_emit_check_memo_key("s", "src/b.rs", "inv")
        );
        assert_ne!(
            a,
            compile_dag_rust_emit_check_memo_key("t", "src/a.rs", "inv")
        );
    }
}
