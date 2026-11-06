// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 97 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::disallowed_macros,  // 75
    clippy::needless_borrow,  // 7
    clippy::needless_return,  // 1
    clippy::nonminimal_bool,  // 1
    clippy::type_complexity,  // 1
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
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::enum_variant_names,
    clippy::iter_kv_map,
    clippy::manual_is_multiple_of,
    clippy::manual_strip,
    clippy::map_identity,
    clippy::missing_const_for_thread_local,
    clippy::needless_lifetimes,
    clippy::only_used_in_recursion,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_char_add_str,
    clippy::too_many_arguments,
    clippy::unnecessary_to_owned,
    clippy::unneeded_struct_pattern,
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

/// The two grounds on which the floor removes an enrolled identity from a roster BEFORE the
/// fold. The authority is `v1.compiler.expected_red_roster_join`; this alias only shortens the
/// path at the suppression site, which is the one place both grounds are decided.
use crate::v1_compiler_expected_red_roster_join::ExpectedRedSuppressionGround as SuppressionGround;
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

/// Why the discovery authority's per-file fold produced no roster. Three causes, three repairs: a
/// source's test-marked declarations could not be READ (it does not parse, so which of its
/// declarations are tests is undeterminable), the authority could not be EVALUATED (a frame or
/// interpreter defect, located at the source when one was being folded), or it evaluated and
/// REFUSED the corpus (a placement or live_tree_disposition violation, carried as the producer's
/// own reason).
pub(crate) enum FloorDiscoveryFoldRefusal {
    Unparseable {
        source: String,
        cause: String,
    },
    Unevaluable {
        source: Option<String>,
        function: &'static str,
        error: String,
    },
    Refused {
        reason: String,
    },
}

impl FloorDiscoveryFoldRefusal {
    pub(crate) fn rendered(&self) -> String {
        match self {
            FloorDiscoveryFoldRefusal::Unparseable { source, cause } => {
                format!("cause=FloorDiscoveryTestMarkersUnreadable source={source} — {cause}")
            }
            FloorDiscoveryFoldRefusal::Unevaluable {
                source: Some(source),
                function,
                error,
            } => format!(
                "cause=FloorDiscoveryAuthorityUnevaluable source={source} — {function}: {error}"
            ),
            FloorDiscoveryFoldRefusal::Unevaluable {
                source: None,
                function,
                error,
            } => format!("cause=FloorDiscoveryAuthorityUnevaluable — {function}: {error}"),
            FloorDiscoveryFoldRefusal::Refused { reason } => {
                format!("cause=FloorDiscoveryRefused — {reason}")
            }
        }
    }
}

/// THE FLOOR'S DISCOVERY, AS ONE FOLD OVER WHATEVER SOURCES THE CALLER HOLDS.
///
/// `v2.workflow.floor_discovery_source_authority` `discover_floor_rows_for_source` per source, then
/// `floor_discovery_finalize_source_outcomes` over the outcomes -- the one authority for which
/// `test` declarations a source enrolls. The required floor folds it over its full inventory;
/// `gunbc test`'s claim route folds it over the modules its operand names. Same authority, same
/// calls, narrower subject: the subject narrows, the decision does not move (review 44641's rule).
/// `frame` must have that module in scope by its qualified name.
///
/// THE MARKER FACT IS SUPPLIED, NOT SCANNED FOR. The authority neither parses nor resolves the
/// files it enumerates, so each source's `marked` argument is the names of the declarations
/// carrying the authored `test` marker, read once by `test_marker::test_marked_decl_names` (the
/// seed's single consumer of the grammar's mark). A source that does not parse refuses here
/// (`Unparseable`); no text scan stands in for the missing fact.
pub(crate) fn floor_discovery_rows_over_sources<'a>(
    frame: &v1_interpreter::InterpContext,
    sources: impl IntoIterator<Item = &'a v1_compiler_compile::SourceFile>,
) -> Result<Vec<DiscoveryRow>, FloorDiscoveryFoldRefusal> {
    let mut outcomes: Vec<v1_interpreter::Value> = Vec::new();
    for source in sources {
        let repo_path = source.path.replace('\\', "/");
        let marked =
            test_marker::test_marked_decl_names(&source.content, &repo_path).map_err(|cause| {
                FloorDiscoveryFoldRefusal::Unparseable {
                    source: repo_path.clone(),
                    cause,
                }
            })?;
        let args = [
            (Some("repo_path".to_string()), str_value(repo_path.clone())),
            (
                Some("marked".to_string()),
                list_value_from_vec(marked.into_iter().map(str_value).collect()),
            ),
            (
                Some("content".to_string()),
                str_value(source.content.clone()),
            ),
        ];
        let outcome = v1_interpreter::run_in_context_with_args(
            frame,
            "v2.workflow.floor_discovery_source_authority.discover_floor_rows_for_source",
            &args,
            false,
        )
        .map_err(|e| FloorDiscoveryFoldRefusal::Unevaluable {
            source: Some(repo_path),
            function: "discover_floor_rows_for_source",
            error: e.to_string(),
        })?;
        outcomes.push(outcome);
    }
    let finalized = v1_interpreter::run_in_context_with_args(
        frame,
        "v2.workflow.floor_discovery_source_authority.floor_discovery_finalize_source_outcomes",
        &[(Some("outcomes".to_string()), list_value_from_vec(outcomes))],
        false,
    )
    .map_err(|e| FloorDiscoveryFoldRefusal::Unevaluable {
        source: None,
        function: "floor_discovery_finalize_source_outcomes",
        error: e.to_string(),
    })?;
    parse_floor_discovery_producer_result(frame, &finalized)
        .map_err(|reason| FloorDiscoveryFoldRefusal::Refused { reason })
}

pub(crate) fn floor_git_diff_range() -> Result<String, String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_unified_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_unified_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffOk") => match ctx.field(fields, "text") {
            Some(Value::Str(s)) => Ok(s.to_string()),
            _ => Err("UnifiedDiffOk missing `text` field".to_string()),
        },
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.to_string()),
            _ => Err("git diff observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_unified_for_ci returned `{}`, expected FloorUnifiedDiffResult",
            ctx.format_value(other)
        )),
    }
}

/// Read back the baseline the affected-set diff was taken against — a PROJECTION of
/// `resolve_diff_baseline` (`v2.workflow.floor_diff_observe`
/// `floor_observe_diff_baseline_readout_for_ci`), never a second derivation. Used only
/// to LOCATE the no-observed-change diagnostic: a receipt that says "zero changed paths"
/// without naming the base it compared against cannot be acted on, because the
/// interesting case is precisely a base that names the head commit itself. Failure to
/// read it is not fatal here — the diagnostic degrades to an unnamed baseline and says
/// so, rather than suppressing the state.
/// Read the resolved COMPARISON WINDOW — base, head and relation — from
/// `v2.workflow.floor_diff_observe` `floor_observe_diff_comparison_readout_for_ci`. A projection of
/// `resolve_diff_baseline`, never a second derivation.
///
/// Distinct from `floor_diff_baseline_readout` beside it, and the distinction is the point: that one
/// answers "which ref" for a diagnostic, and a DECIDING consumer that takes it has to invent the
/// missing head and relation. Inventing them is what shipped the merge-base-on-every-arm fail-open,
/// so the deciding consumers read this one and a `ComparisonReadoutRefused` propagates.
pub(crate) fn floor_diff_comparison_readout() -> Result<FreezeBaselineComparison, String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_diff_comparison_readout_for_ci", false)
            .map_err(|e| format!("floor_observe_diff_comparison_readout_for_ci: {e}"))?;
    let str_field = |fields: &_, name: &str| -> Result<String, String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) => Ok(s.to_string()),
            _ => Err(format!("comparison readout missing `{name}`")),
        }
    };
    // The baseline KIND is a closed coproduct in `gunbc.diff_baseline`, so an unrecognized arm is an
    // unmodeled state rather than a formatting question: it refuses instead of rendering a guess.
    let kind_name = |fields: &_| -> Result<String, String> {
        match ctx.field(fields, "kind") {
            Some(Value::Variant { variant_name, .. }) => {
                for name in [
                    "MergeTargetBaseline",
                    "ExactReplayBaseline",
                    "PushBeforeBaseline",
                    "PushParentBaseline",
                    "MergeGroupBaseBaseline",
                    "OperatorOverrideBaseline",
                ] {
                    if ctx.sym_eq(*variant_name, name) {
                        return Ok(name.to_string());
                    }
                }
                Err("comparison readout carries an unmodeled DiffBaselineKind arm".to_string())
            }
            _ => Err("comparison readout missing `kind`".to_string()),
        }
    };
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "DirectComparison") => {
            Ok(FreezeBaselineComparison::Direct {
                base: str_field(fields, "base")?,
                head: str_field(fields, "head")?,
                kind: kind_name(fields)?,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "MergeBaseComparison") => {
            Ok(FreezeBaselineComparison::MergeBase {
                base: str_field(fields, "base")?,
                head: str_field(fields, "head")?,
                kind: kind_name(fields)?,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "ComparisonReadoutRefused") => {
            match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => Err(r.to_string()),
                _ => Err("comparison readout refused (no reason)".to_string()),
            }
        }
        other => Err(format!(
            "floor_observe_diff_comparison_readout_for_ci returned `{}`, expected \
             FloorDiffComparisonReadout",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn floor_diff_baseline_readout() -> Result<(String, String), String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_diff_baseline_readout_for_ci", false)
            .map_err(|e| format!("floor_observe_diff_baseline_readout_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "BaselineReadout") => {
            let base = match ctx.field(fields, "ref") {
                Some(Value::Str(s)) => s.to_string(),
                _ => return Err("BaselineReadout missing `ref`".to_string()),
            };
            let event = match ctx.field(fields, "event_name") {
                Some(Value::Str(s)) => s.to_string(),
                _ => return Err("BaselineReadout missing `event_name`".to_string()),
            };
            Ok((base, event))
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "BaselineReadoutRefused") => match ctx.field(fields, "reason")
        {
            Some(Value::Str(r)) => Err(r.to_string()),
            _ => Err("baseline readout refused (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_diff_baseline_readout_for_ci returned `{}`, expected FloorDiffBaselineReadout",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn floor_git_diff_name_status_range() -> Result<(Vec<String>, HashSet<String>), String> {
    use v1_interpreter::Value;
    let roots = default_source_roots();
    let entry = "src/v2/workflow/floor_diff_observe.dag";
    let (graph, indices) = resolve_entry_graph_shared(&roots, entry)
        .map_err(|e| format!("floor_diff_observe resolve: {e}"))?;
    let ctx = make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let result =
        v1_interpreter::run_in_context(&ctx, "floor_observe_git_diff_name_status_for_ci", false)
            .map_err(|e| format!("floor_observe_git_diff_name_status_for_ci: {e}"))?;
    match &result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusDiffOk") => {
            let changed = match ctx.field(fields, "changed_paths") {
                Some(v) => string_list_from_value(v, "changed_paths")?,
                None => return Err("NameStatusDiffOk missing `changed_paths` field".to_string()),
            };
            let departed = match ctx.field(fields, "departed_paths") {
                Some(v) => string_list_from_value(v, "departed_paths")?,
                None => return Err("NameStatusDiffOk missing `departed_paths` field".to_string()),
            };
            Ok((
                changed.iter().map(|p| normalize_repo_path(p)).collect(),
                departed.iter().map(|p| normalize_repo_path(p)).collect(),
            ))
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusDiffFail") => match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Err(r.to_string()),
            _ => Err("git diff --name-status observation failed (no reason)".to_string()),
        },
        other => Err(format!(
            "floor_observe_git_diff_name_status_for_ci returned `{}`, expected FloorNameStatusDiffResult",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn floor_diff_edits_from_diff_text(
    index: &MultiEntryIndex,
    diff_text: &str,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let departed = parse_unified_diff_departed_paths(diff_text);
    let added = parse_unified_diff_added_paths(diff_text);
    // No census: `enrolled_test_fns` stays empty. Attribution tests read `edited_test_fns` /
    // `touched_entry_files` / data items from this wrapper; enrolment is only answered when
    // `floor_diff_edits_from_diff_text_with_base_names` (or production) supplies the census.
    floor_diff_edits_from_line_ranges(
        index,
        &line_ranges,
        &departed,
        &added,
        None,
        &std::collections::HashMap::new(),
    )
}

pub(crate) fn floor_diff_edits_from_diff_text_with_base_names(
    index: &MultiEntryIndex,
    diff_text: &str,
    base_test_decl_names: &std::collections::HashMap<String, HashSet<String>>,
) -> Result<FloorDiffEdits, String> {
    let line_ranges = parse_unified_diff_line_ranges(diff_text);
    let departed = parse_unified_diff_departed_paths(diff_text);
    let added = parse_unified_diff_added_paths(diff_text);
    let rename_from = parse_unified_diff_rename_sources(diff_text);
    floor_diff_edits_from_line_ranges(
        index,
        &line_ranges,
        &departed,
        &added,
        Some(base_test_decl_names),
        &rename_from,
    )
}

/// `floor_diff_edits_from_diff_text_with_base_names` with each changed path's new-side
/// content SUPPLIED rather than read from the working tree.
#[cfg(test)]
pub(crate) fn floor_diff_edits_from_diff_text_with_base_names_and_sources(
    index: &MultiEntryIndex,
    diff_text: &str,
    base_test_decl_names: &std::collections::HashMap<String, HashSet<String>>,
    sources: &std::collections::HashMap<String, String>,
) -> Result<FloorDiffEdits, String> {
    floor_diff_edits_from_line_ranges_reading(
        index,
        &parse_unified_diff_line_ranges(diff_text),
        &parse_unified_diff_departed_paths(diff_text),
        &parse_unified_diff_added_paths(diff_text),
        Some(base_test_decl_names),
        &parse_unified_diff_rename_sources(diff_text),
        &|path: &str| Ok(sources.get(path).cloned()),
    )
}

// Host realization under a declared scaffold: the governing row is the `SCAFFOLD (DESIGN
// §6–§7)` declaration above `FloorDiffEdits` in `cli_run`, which owns this function's
// reason, dissolve-on trigger and census. Read it there; it is not restated here.
pub(crate) fn floor_diff_edits_from_line_ranges(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    departed_paths: &HashSet<String>,
    added_paths: &HashSet<String>,
    base_test_decl_names: Option<&std::collections::HashMap<String, HashSet<String>>>,
    rename_from: &std::collections::HashMap<String, String>,
) -> Result<FloorDiffEdits, String> {
    floor_diff_edits_from_line_ranges_reading(
        index,
        line_ranges_by_file,
        departed_paths,
        added_paths,
        base_test_decl_names,
        rename_from,
        &read_working_tree_source,
    )
}

/// The production reader: a changed path's content at the working tree. `Ok(None)` is
/// "absent from the tree", which the caller dispositions against the diff's departed set.
fn read_working_tree_source(file_norm: &str) -> Result<Option<String>, String> {
    let disk_path = process_workspace_root().join(file_norm);
    if !disk_path.is_file() {
        return Ok(None);
    }
    std::fs::read_to_string(&disk_path)
        .map(Some)
        .map_err(|e| format!("read failed for {file_norm}: {e}"))
}

/// Attribution over a supplied reader of each changed path's new-side content. The
/// interface is diff + census + content -> edits, so a witness supplies the content as a
/// value (DESIGN §3 witness rule) rather than reading whatever the live tree now holds.
#[allow(clippy::too_many_arguments)]
pub(crate) fn floor_diff_edits_from_line_ranges_reading(
    index: &MultiEntryIndex,
    line_ranges_by_file: &HashMap<String, Vec<FileLineRange>>,
    departed_paths: &HashSet<String>,
    added_paths: &HashSet<String>,
    base_test_decl_names: Option<&std::collections::HashMap<String, HashSet<String>>>,
    rename_from: &std::collections::HashMap<String, String>,
    read_source: &dyn Fn(&str) -> Result<Option<String>, String>,
) -> Result<FloorDiffEdits, String> {
    let mut overlapping_data_items = HashSet::new();
    let mut edited_test_fns = HashSet::new();
    let mut enrolled_test_fns = HashSet::new();
    let mut touched_entry_files = HashSet::new();
    let mut touched_declarations: HashSet<(String, String)> = HashSet::new();
    // #6269 attributes src/v1/ .dag changes through a dedicated index; the structural-∅ fix
    // dropped the saw_non_dag/saw_dag refusal (a non-.dag-only diff is a nominal empty frontier,
    // handled by the `continue` arm below), so neither flag is needed here.
    let v1_attribution_index = if line_ranges_by_file
        .keys()
        .any(|p| normalize_repo_path(p).starts_with("src/v1/"))
    {
        Some(build_v1_attribution_multi_entry_index())
    } else {
        None
    };
    for (file_path, ranges) in line_ranges_by_file {
        if !file_path.ends_with(".dag") {
            // A non-.dag changed path is a structural-∅ for the .dag frontier: it declares no
            // .dag nodes, so there is nothing to attribute, and its coverage lives in the Rust
            // gates (rust_tests), not the .dag witnesses. Skipping it yields an empty .dag
            // frontier -- the SAME nominal outcome as an empty diff. This is NOT ignorance: the
            // only ignorance state is a failed git-diff observation (UnifiedDiffFail upstream,
            // floor_diff_observe.dag; operator ruling 2026-07-05). Structural-∅ and ignorance
            // are different states -- the mirror of the departed-.dag-path arm below.
            continue;
        }
        let file_norm = normalize_repo_path(file_path);
        let Some(content) = read_source(&file_norm)? else {
            if departed_paths.contains(&file_norm) {
                // Departed per the diff (deletion / rename-from): its decl set
                // is empty by construction — the file has no declarations to
                // attribute. The path-grain fact stays in changed_paths;
                // dependents that imported it fail loudly at their own resolve.
                continue;
            }
            // Absent from the tree but NOT marked departed by the diff: the
            // observation is incoherent (stale tree, quoting artifact, bogus
            // path). Structural-∅ and ignorance are different states — refuse.
            return Err(format!(
                "affected-set derivation refused: diff names {file_path} with \
                 content changes but the path is absent from the working tree \
                 and the diff does not mark it departed (deletion/rename)"
            ));
        };
        let resolve_index = if file_norm.starts_with("src/v1/") {
            v1_attribution_index.as_ref().expect("v1 attribution index")
        } else {
            index
        };
        // Attribution is a PARSE-grade fact: it needs each touched file's
        // declaration line map (names + spans + data/fn kind), never its typecheck.
        // The former full entry RESOLVE here made every touched file's typecheck
        // health gate the whole frontier — on a corpus-wide diff, one latent-red
        // module (a batch-2 debt row no gate compiles) dead-ended batch 2 at one
        // entry per CI cycle. Parse errors still refuse (typed, located); typecheck
        // reds surface where they belong — as that entry's own counted discovery row.
        let source = Rc::new(v1_compiler_compile::SourceFile {
            path: file_norm.clone(),
            content: content.clone(),
        });
        let (module_node, nl) = match parse_module_node_from_index_source(resolve_index, source) {
            Ok(pair) => pair,
            Err(e) => return Err(format!("parse failed for {file_path}: {e}")),
        };
        let single_si: Rc<HashMap<String, Rc<NewlineIndex>>> = Rc::new({
            let mut m = HashMap::new();
            m.insert(file_norm.clone(), nl.clone());
            m
        });
        // A test is a declaration carrying the authored `test` marker, read off the item the
        // parse above already produced: no second reading of the file, and no line scan.
        let mut test_fn_names: HashSet<String> = HashSet::new();
        let mut decls: Vec<(i64, String, bool)> = Vec::new();
        for item in crate::v1_std_core::module_items(module_node.clone()).iter() {
            let line = byte_to_line_col(nl.clone(), item.span.start).line;
            let name = authored_name_at(single_si.clone(), item.clone());
            let is_data = item_kind(item.clone()) == ItemKind::DataItem;
            if item.declaration_marker == crate::v1_std_core::DeclarationMarker::TestMarked {
                test_fn_names.insert(name.clone());
            }
            decls.push((line, name, is_data));
        }
        if decls.is_empty() {
            // A declaration-less module (a lone `module` line — e.g. the
            // shadow-masked fixtures) has nothing to attribute at decl grain;
            // its only edit surface IS the file, so the file-grain touched set
            // carries it (dependents rerun via the import closure). Refusing
            // here dead-ended the frontier on a fixture that is legitimately
            // empty — not an incoherent observation.
            touched_entry_files.insert(file_norm.clone());
            continue;
        }
        decls.sort_by_key(|(line, _, _)| *line);
        // A PATH WHOSE DECLARATION SET IS ESTABLISHED FRESH CONTRIBUTES EVERY DECLARATION IT
        // CARRIES, and the line ranges do not establish that set — they only report which lines
        // the diff happened to print.
        //
        // `parse_unified_diff_added_paths` already rules that both wholly-added files and RENAME
        // DESTINATIONS are new-at-path ("its declaration set is established fresh at NEW"), and
        // for a `/dev/null` add the two agree by accident: every line is a `+` line, so line
        // attribution reaches every declaration anyway. For a rename destination they do not.
        // Git detects the rename and prints only the hunks that differ, so a moved file's
        // declarations are attributed by WHICH LINES THE MOVE HAPPENED TO EDIT — while every
        // identity at the new path is a NEW qualified identity (`<authored module>.<function>`)
        // that has never executed under that spelling, because the authored module name moved
        // with the file.
        //
        // MEASURED, NOT ANTICIPATED (2026-08-31, `git rev-list HEAD` = 90 commits): 8 of 8
        // rename-destination `.dag` files carrying test decls were under-enrolled, 87 of their
        // 103 identities missed. gunbc#9823 is the receipt-confirmed instance — it renamed
        // `machine_shape_construction_wall_test.dag` into `v2.test.` with two `test fn`s, git
        // printed the module line and the removed trailing blank line at EOF, the EOF hunk fell
        // inside the LAST declaration, and exactly one of the two siblings was selected. The
        // wall's DISCRIMINATING RED (`gate_red_synthetic_machine_shape_call`) was the one missed.
        // It executed anyway there, via the ordinary `Planned` arm — but changed-witness
        // membership is precisely what OVERRIDES the cost-debt withhold and the outside-gate
        // suppression below, so the same miss on a rostered identity is a silent decline of a
        // witness whose author is present. That is the state
        // `v2.workflow.floor_changed_witness` was written for.
        //
        // THIS NARROWS NOTHING AND WIDENS NOTHING BEYOND THE PATH'S OWN DECLARATIONS: the
        // universe is this file's parsed decl list, not the corpus, so it is the precise answer
        // to "what does this path declare", never an absorbing "rerun everything" (DESIGN §5).
        let charged: HashSet<usize> = if added_paths.contains(&file_norm) {
            (0..decls.len()).collect()
        } else {
            // A DECLARATION IS CHANGED WHEN ITS OWN TEXT DIFFERS AT BASE AND HEAD, joined by
            // name -- never by which lines a hunk names. Git's `-U0` alignment is free to print a
            // deletion between two similar declarations as a hunk that starts or ends inside the
            // neighbour, and any line-ownership rule then charges a declaration nobody touched
            // (`gunbc.recurring_failure_mode` `a_pure_deletion_force_runs_its_unchanged_neighbour`,
            // both the gap form of gunbc#13103 and the leading-comment form of gunbc#13126). The
            // base is reconstructed from the head and the diff's own removed lines, so this is
            // the same single observation, read without its alignment.
            let head_lines: Vec<&str> = content.lines().collect();
            let starts: Vec<(i64, String)> =
                decls.iter().map(|(l, n, _)| (*l, n.clone())).collect();
            let attribution = declaration_text_attribution(&head_lines, &starts, ranges);
            // Module-line edits stay fail-closed for modifies — renaming can change entry
            // identity. Wholly-added files necessarily touch line 1.
            if attribution.module_line_changed {
                return Err(format!("diff before first declaration in {file_path}"));
            }
            if attribution.pre_declaration_changed {
                touched_entry_files.insert(file_norm.clone());
            }
            attribution.changed
        };
        for (i, (_, name, is_data)) in decls.iter().enumerate() {
            if !charged.contains(&i) {
                continue;
            }
            if test_fn_names.contains(name) {
                edited_test_fns.insert((file_norm.clone(), name.clone()));
                // NEWLY ENROLLED = declared now and not declared at the resolved diff base.
                // The census is path-keyed. A rename destination is absent at the NEW path, so
                // looking up the dest would enrol every fn as new — the author's-only-moved
                // case. Enrolment therefore reads the SOURCE path when `rename_from` names one.
                // Without a census there is no enrolment answer: the added-path rule is the
                // superseded rung and must not remain as a silent alternative (review 64246).
                let newly_declared = match base_test_decl_names {
                    Some(at_base) => {
                        let census_path = rename_from.get(&file_norm).unwrap_or(&file_norm);
                        !at_base
                            .get(census_path)
                            .is_some_and(|names| names.contains(name))
                    }
                    None => false,
                };
                if newly_declared {
                    enrolled_test_fns.insert((file_norm.clone(), name.clone()));
                }
            } else if *is_data {
                overlapping_data_items.insert((file_norm.clone(), name.clone()));
                touched_declarations.insert((file_norm.clone(), name.clone()));
            } else {
                touched_entry_files.insert(file_norm.clone());
                touched_declarations.insert((file_norm.clone(), name.clone()));
            }
        }
    }
    // A present diff whose changed paths are all non-.dag lands here with an empty frontier
    // (structural-∅): it flows through as every row's not-affected skip -- nominal and
    // transparent, exactly like an empty diff, never a refusal. Observation failure (the only
    // ignorance state) is refused upstream in floor_git_diff_range; a successful observation
    // with an empty .dag subset is not ignorance.
    Ok(FloorDiffEdits {
        overlapping_data_items,
        edited_test_fns,
        enrolled_test_fns,
        touched_entry_files,
        touched_declarations,
    })
}

/// What one modified file's declarations look like once each is compared with itself at the
/// diff base. `changed` holds indices into the head declaration list the caller supplied.
struct DeclarationTextAttribution {
    module_line_changed: bool,
    pre_declaration_changed: bool,
    changed: HashSet<usize>,
}

/// A declaration's OWN text, ANNOTATION-ERASED: from its first line up to the next declaration,
/// with every `//` annotation line removed and the trailing blank lines dropped. DESIGN §4c:
/// semantic passes receive only the annotation-erased projection, so adding, deleting, editing or
/// moving a `//` block never changes this text and never charges the declaration. Whitespace is
/// otherwise compared as is.
///
/// A `.dag` string literal may span lines (`scan_string_body` does not stop at a newline), so a
/// line that begins with `//` INSIDE a string is program data, not an annotation, and is kept:
/// `annotation_line_mask` reads the lexical state at each line start.
fn declaration_own_text(lines: &[String], start: usize, next: usize) -> String {
    let span = &lines[start..next.min(lines.len()).max(start)];
    let mut kept: Vec<&str> = span
        .iter()
        .zip(annotation_line_mask(span))
        .filter(|(_, annotation)| !annotation)
        .map(|(l, _)| l.as_str())
        .collect();
    while kept.last().is_some_and(|l| l.trim().is_empty()) {
        kept.pop();
    }
    kept.join("\n")
}

/// For each line, whether it is a `//` annotation line: it starts (after indentation) with `//`
/// while the lexer is in code, not inside a string literal. Strings carry `\` escapes and `${ }`
/// interpolation whose code may itself open strings, so the state is a stack: `Str` inside a
/// literal, `Code(depth)` inside an interpolation's braces.
fn annotation_line_mask(lines: &[String]) -> Vec<bool> {
    enum Frame {
        Str,
        Code(usize),
    }
    let mut stack: Vec<Frame> = Vec::new();
    let mut mask = Vec::with_capacity(lines.len());
    for line in lines {
        let in_string = matches!(stack.last(), Some(Frame::Str));
        mask.push(!in_string && line.trim_start().starts_with("//"));
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match stack.last_mut() {
                Some(Frame::Str) => match c {
                    '\\' => i += 1,
                    '"' => {
                        stack.pop();
                    }
                    '$' if chars.get(i + 1) == Some(&'{') => {
                        stack.push(Frame::Code(0));
                        i += 1;
                    }
                    _ => {}
                },
                code => {
                    if c == '/' && chars.get(i + 1) == Some(&'/') {
                        break;
                    }
                    match (c, code) {
                        ('"', _) => stack.push(Frame::Str),
                        ('{', Some(Frame::Code(d))) => *d += 1,
                        ('}', Some(Frame::Code(0))) => {
                            stack.pop();
                        }
                        ('}', Some(Frame::Code(d))) => *d -= 1,
                        _ => {}
                    }
                }
            }
            i += 1;
        }
    }
    mask
}

/// Whether a line REMOVED by the diff opens a declaration at the base. Removed lines are not
/// parsed, so this reads layout: a column-0 line that is not a comment or a closing bracket. It
/// errs toward splitting -- a false split shortens a base declaration and so CHARGES its head
/// counterpart, and a missed one lengthens it and charges too; neither direction un-charges an
/// edited declaration.
fn removed_line_opens_declaration(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if first.is_whitespace() || matches!(first, '}' | ')' | ']' | '"') || line.starts_with("//") {
        return None;
    }
    const MARKERS: [&str; 6] = ["test", "pub", "fn", "data", "type", "probe"];
    line.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty() && !MARKERS.contains(w))
        .map(str::to_string)
        .next()
        .or_else(|| Some(String::new()))
}

/// Compare every head declaration with the base declaration of the same name. The base file is
/// the head with each hunk's new-side lines replaced by its removed lines, which `-U0` carries
/// in full, so the comparison needs no second read and no alignment.
fn declaration_text_attribution(
    head: &[&str],
    decls: &[(i64, String)],
    ranges: &[FileLineRange],
) -> DeclarationTextAttribution {
    let mut hunks: Vec<&FileLineRange> = ranges.iter().collect();
    hunks.sort_by_key(|r| r.new_start);
    let mut base: Vec<String> = Vec::new();
    let mut base_opens: Vec<(usize, String)> = Vec::new();
    let mut head_to_base: Vec<Option<usize>> = vec![None; head.len()];
    let mut h = 0usize;
    for r in hunks {
        let before = if r.new_count > 0 {
            r.new_start - 1
        } else {
            r.new_start
        };
        let before = (before.max(0) as usize).min(head.len());
        while h < before {
            head_to_base[h] = Some(base.len());
            base.push(head[h].to_string());
            h += 1;
        }
        for line in &r.removed {
            if let Some(name) = removed_line_opens_declaration(line) {
                base_opens.push((base.len(), name));
            }
            base.push(line.clone());
        }
        h = (h + r.new_count.max(0) as usize).min(head.len());
    }
    while h < head.len() {
        head_to_base[h] = Some(base.len());
        base.push(head[h].to_string());
        h += 1;
    }
    let head_owned: Vec<String> = head.iter().map(|l| l.to_string()).collect();
    for (line, name) in decls {
        if let Some(Some(b)) = head_to_base.get((*line - 1).max(0) as usize) {
            base_opens.push((*b, name.clone()));
        }
    }
    base_opens.sort();
    base_opens.dedup_by_key(|(b, _)| *b);
    let first_head = (decls[0].0 - 1).max(0) as usize;
    let first_base = base_opens.first().map(|(b, _)| *b).unwrap_or(base.len());
    let pre_declaration_changed = declaration_own_text(&head_owned, 0, first_head)
        != declaration_own_text(&base, 0, first_base);
    let module_line_changed = head_owned.first() != base.first();
    let mut changed = HashSet::new();
    for (i, (line, name)) in decls.iter().enumerate() {
        let start = (*line - 1).max(0) as usize;
        let next = decls
            .get(i + 1)
            .map(|(l, _)| (*l - 1).max(0) as usize)
            .unwrap_or(head_owned.len());
        let text = declaration_own_text(&head_owned, start, next);
        let unchanged = base_opens.iter().enumerate().any(|(j, (b, n))| {
            n == name && {
                let next_b = base_opens.get(j + 1).map(|(x, _)| *x).unwrap_or(base.len());
                declaration_own_text(&base, *b, next_b) == text
            }
        });
        if !unchanged {
            changed.insert(i);
        }
    }
    DeclarationTextAttribution {
        module_line_changed,
        pre_declaration_changed,
        changed,
    }
}

/// ONE SCHEDULED ROW of the local-repo wet lane, decoded from
/// `v2.workflow.local_repo_wet_terminal.local_repo_wet_schedule`. The `.dag` declaration is the
/// authority for membership and for the verdict each member is expected to reach; this struct is
/// its host realization, never its origin.
#[derive(Debug, Clone)]
pub(crate) struct LocalRepoWetScheduledRow {
    pub identity: String,
    /// The AUTHORED source path. Kept and JOINED, not decoration: the lane refuses if the prepared
    /// subject resolved the module from a different file, so a stale entry cannot sit unread beside
    /// a module the executor reached some other way.
    pub entry: String,
    pub entry_module: String,
    pub function: String,
    /// THE DECLARED HOST PREMISE, if any, by its `v2.workflow.wet_evidence` `WetHostPremise` arm
    /// name, joined from `local_repo_wet_premise_roster`. The executor reads it BEFORE invoking
    /// the claim; the readback and its verdict are `.dag` (`gunbc.runner.wet_host_premise_readback`).
    pub premise: Option<String>,
}

/// The module whose `local_repo_wet_premise_readings_wet` answers every declared premise once.
pub(crate) const LOCAL_REPO_WET_PREMISE_READBACK_MODULE: &str =
    "gunbc.runner.wet_host_premise_readback";

/// Decode the lane's schedule from its `.dag` authority.
fn local_repo_wet_schedule(
    hermetic: &v1_interpreter::InterpContext,
) -> Result<Vec<LocalRepoWetScheduledRow>, String> {
    let value = v1_interpreter::run_in_context(
        hermetic,
        "v2.workflow.local_repo_wet_terminal.local_repo_wet_schedule",
        false,
    )
    .map_err(|e| format!("local_repo_wet_schedule: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|e| format!("local_repo_wet_schedule: {e}"))?;
    let mut out: Vec<LocalRepoWetScheduledRow> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for item in items {
        let v1_interpreter::Value::Record { type_name, fields } = item else {
            return Err(format!(
                "local_repo_wet_schedule: expected WetScheduledClaim, got {}",
                floor_value_shape(Some(&item))
            ));
        };
        if !hermetic.sym_eq(*type_name, "WetScheduledClaim") {
            return Err(format!(
                "local_repo_wet_schedule: expected WetScheduledClaim, got record {}",
                hermetic.resolve(*type_name)
            ));
        }
        let str_field = |name: &str| -> Result<String, String> {
            match hermetic.field(&fields, name) {
                Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                other => Err(format!(
                    "local_repo_wet_schedule: {name} must be String, got {}",
                    floor_value_shape(other)
                )),
            }
        };
        let function = str_field("function")?;
        let entry = str_field("entry")?;
        // THE IDENTITY IS A STRUCTURED `WitnessIdentity`, NOT A SPELLED NAME. It carries
        // `module_path` and `function` as separate fields, so the qualified name is DERIVED here
        // rather than parsed back out of a string — which is what the authority does in
        // `witness_identity_qualified_name`, and deriving it twice the same way is the point.
        let (module_path, identity_function) = match hermetic.field(&fields, "identity") {
            Some(v1_interpreter::Value::Record {
                type_name: id_type,
                fields: id_fields,
            }) => {
                if !hermetic.sym_eq(*id_type, "WitnessIdentity") {
                    return Err(format!(
                        "local_repo_wet_schedule: identity must be a WitnessIdentity, got record {}",
                        hermetic.resolve(*id_type)
                    ));
                }
                let id_str = |name: &str| -> Result<String, String> {
                    match hermetic.field(id_fields, name) {
                        Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                        other => Err(format!(
                            "local_repo_wet_schedule: identity.{name} must be String, got {}",
                            floor_value_shape(other)
                        )),
                    }
                };
                (id_str("module_path")?, id_str("function")?)
            }
            other => {
                return Err(format!(
                    "local_repo_wet_schedule: identity must be a WitnessIdentity record, got {}",
                    floor_value_shape(other)
                ));
            }
        };
        let identity = format!("{module_path}.{identity_function}");
        // THE ENTRY MODULE IS READ FROM THE IDENTITY, NOT PARSED OUT OF IT. `WitnessIdentity`
        // carries `module_path` as its own field, so what used to be a suffix-strip is now a
        // projection (DESIGN §3 — a path is a discriminator, the authored module name is the fact).
        //
        // THE AGREEMENT WALL SURVIVES AND GOT STRONGER RATHER THAN WEAKER. The old check inferred
        // disagreement from a failed suffix-strip, which could only notice it when the spelled
        // identity did not END in the function. The row carries the function name TWICE — once in
        // `identity.function`, once as its own `function` field — so the two are compared directly
        // and any disagreement refuses, not only the ones a suffix happens to expose. An empty
        // module_path is refused for the same reason the strip refused an empty prefix: a member
        // whose module is unnamed cannot key a prepared subject.
        if identity_function != function {
            return Err(format!(
                "local_repo_wet_schedule: identity.function {identity_function:?} and the row's \
                 function {function:?} disagree — one member, two names"
            ));
        }
        if module_path.is_empty() {
            return Err(format!(
                "local_repo_wet_schedule: identity {identity:?} carries an empty module_path"
            ));
        }
        let entry_module = module_path;
        // `ClaimExpectation` HAS TWO ARMS WHERE ITS PREDECESSOR HAD ONE, AND THIS LANE STILL
        // REALIZES EXACTLY ONE. That is the whole reason this refusal is kept rather than widened
        // with the type: the shared expectation vocabulary can now SAY `ExpectedRed`, and nothing
        // in this executor RUNS an expected-red member. A wider type is not a wider capability, and
        // defaulting here would let the authority claim a behaviour production cannot perform.
        // The widening that admits an expected-red member must arrive with the executor arm that
        // realizes it, and this line is what forces those two to land together.
        match hermetic.field(&fields, "expectation") {
            Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
                match hermetic.resolve(*variant_name).as_str() {
                    "ExpectedToHold" => {}
                    other => {
                        return Err(format!(
                            "local_repo_wet_schedule: expectation {other} has no executor arm in this \
                             lane; only ExpectedToHold is realizable today"
                        ));
                    }
                }
            }
            other => {
                return Err(format!(
                    "local_repo_wet_schedule: expectation must be a typed variant, got {}",
                    floor_value_shape(other)
                ));
            }
        }
        // A DUPLICATE REFUSES. The roster is joined to the receipts at identity grain, and a
        // repeated member would make one receipt answer for two rows.
        if !seen.insert(identity.clone()) {
            return Err(format!(
                "local_repo_wet_schedule: duplicate scheduled identity: {identity}"
            ));
        }
        out.push(LocalRepoWetScheduledRow {
            identity,
            entry,
            entry_module,
            function,
            premise: None,
        });
    }
    local_repo_wet_join_premises(hermetic, &mut out)?;
    Ok(out)
}

/// JOIN `v2.workflow.local_repo_wet_terminal.local_repo_wet_premise_roster` onto the schedule at
/// identity grain. A premise row naming an identity nobody scheduled, or naming one twice, REFUSES:
/// a premise that gates nothing is a roster defect, and two premises for one claim have no order.
fn local_repo_wet_join_premises(
    hermetic: &v1_interpreter::InterpContext,
    schedule: &mut [LocalRepoWetScheduledRow],
) -> Result<(), String> {
    let value = v1_interpreter::run_in_context(
        hermetic,
        "v2.workflow.local_repo_wet_terminal.local_repo_wet_premise_roster",
        false,
    )
    .map_err(|e| format!("local_repo_wet_premise_roster: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|e| format!("local_repo_wet_premise_roster: {e}"))?;
    for item in items {
        let v1_interpreter::Value::Record { fields, .. } = item else {
            return Err(format!(
                "local_repo_wet_premise_roster: expected WetPremisedClaim, got {}",
                floor_value_shape(Some(item))
            ));
        };
        let identity = match hermetic.field(fields, "identity") {
            Some(v1_interpreter::Value::Record {
                fields: id_fields, ..
            }) => {
                let id_str = |name: &str| -> Result<String, String> {
                    match hermetic.field(id_fields, name) {
                        Some(v1_interpreter::Value::Str(s)) => Ok(s.to_string()),
                        other => Err(format!(
                            "local_repo_wet_premise_roster: identity.{name} must be String, got {}",
                            floor_value_shape(other)
                        )),
                    }
                };
                format!("{}.{}", id_str("module_path")?, id_str("function")?)
            }
            other => {
                return Err(format!(
                    "local_repo_wet_premise_roster: identity must be a WitnessIdentity record, \
                     got {}",
                    floor_value_shape(other)
                ));
            }
        };
        let premise = match hermetic.field(fields, "premise") {
            Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
                hermetic.resolve(*variant_name).to_string()
            }
            other => {
                return Err(format!(
                    "local_repo_wet_premise_roster: premise must be a WetHostPremise variant, \
                     got {}",
                    floor_value_shape(other)
                ));
            }
        };
        let Some(row) = schedule.iter_mut().find(|r| r.identity == identity) else {
            return Err(format!(
                "local_repo_wet_premise_roster: {identity} declares premise {premise} but is not \
                 scheduled"
            ));
        };
        if let Some(prior) = &row.premise {
            return Err(format!(
                "local_repo_wet_premise_roster: {identity} declares two premises ({prior}, \
                 {premise})"
            ));
        }
        row.premise = Some(premise);
    }
    Ok(())
}

/// Per-witness selection detail (the `SKIP`/`SKIP-RESOLVE`/`PREDICT` lines and the
/// per-resolve `[binding-fork-ledger]` census) is opt-in. The default floor output is the
/// upfront `[affected-set]` categorization plus the final `[measurement]` tally — a wide
/// corpus otherwise streams one skip line per unaffected witness (~1.7k lines), drowning the
/// signal. The counts survive on `DiscoverySummary`; only the per-row narration is gated.
pub(crate) fn floor_verbose() -> bool {
    std::env::var("GUNBC_FLOOR_VERBOSE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// Wall-clock stamp `HH:MM:SS.mmm` (UTC) prefixed on the live floor lines so the stream reads as
/// a timeline and correlates with CI's wall-clock log — dependency-free (no chrono): seconds
/// since the epoch reduced to a 24h clock, plus millis.
pub(crate) fn floor_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

/// THE SEEDS OF THE REQUIRED FLOOR'S NOMINAL PREPARED SUBJECT -- what the floor prepares when a
/// run touches nothing: the gate's prefix roster and authored-module roster, both decoded from
/// `v2.workflow.required_floor` in a frame over that module's own closure, and the local-repo wet
/// schedule rows whose entry modules join the module seeds. The diff-derived seeds (changed
/// witnesses, touched entries, changed-interface consumers) are NOT here: they are a fact about one run's
/// diff, and this is the part of the subject that holds for every run.
///
/// A function rather than a block in `run_required_floor`, so the seed list has one authority for
/// what the floor prepares.
pub struct RequiredFloorNominalSubjectSeeds {
    pub required_gate_prefixes: Vec<String>,
    pub required_gate_authored_modules: Vec<String>,
    pub(crate) local_repo_wet_schedule_rows: Vec<LocalRepoWetScheduledRow>,
}

/// THE LANE'S SCHEDULE IS DECODED HERE, IN THE POLICY CLOSURE, AND NOT LATER -- because its
/// modules must become SEEDS of the prepared subject. The first CI run of that lane refused
/// seven times with `EntryModuleOutsidePreparedSubject`: the prepared graph is the required-gate
/// closure plus the changed set, and the lane's members are on the discovery exclusion
/// frontier, so nothing pulled them in. A lane that cannot reach its own members cannot support
/// the route claim `std.witness_admission` makes for its cadence, so the schedule joins the seed
/// list rather than the executor learning to run outside the subject.
pub fn required_floor_nominal_subject_seeds(
    source_roots: &[String],
    gate_entry_index: &MultiEntryIndex,
) -> Result<RequiredFloorNominalSubjectSeeds, String> {
    let corpus = crate::cli_run::read_source_corpus_once(source_roots);
    required_floor_nominal_subject_seeds_from_corpus(&corpus, gate_entry_index)
}

/// The seed fold over a corpus the CALLER read. `run_required_floor` is the least common ancestor
/// of this prepare and the gate prepare below it, so it reads once and lends to both; the wrapper
/// above stays for callers with only one demand (the lane resolution census).
pub fn required_floor_nominal_subject_seeds_from_corpus(
    corpus: &crate::cli_run::SourceCorpusRead,
    gate_entry_index: &MultiEntryIndex,
) -> Result<RequiredFloorNominalSubjectSeeds, String> {
    let policy_seed = [REQUIRED_FLOOR_POLICY_MODULE.to_string()];
    let (policy_prepared, _) = crate::cli_run::prepare_repository_from_corpus(
        corpus,
        &floor_prepared_subject_exclusions(),
        Some((gate_entry_index, &[], &policy_seed)),
    )?;
    let policy_scope = claim_scope_for(&policy_prepared, REQUIRED_FLOOR_POLICY_MODULE)?;
    let policy_frame = evaluation_frame(
        &policy_scope,
        v1_interpreter::ExecutionMode::Hermetic,
        None,
        None,
    );
    let required_gate_prefixes = floor_decode_module_prefix_roster(
        &policy_frame,
        "v2.workflow.required_floor.required_gate_prefixes",
    )?;
    // THE GATE'S SECOND SELECTOR KIND (`v2.workflow.required_floor`
    // `required_gate_authored_modules`, `RequiredGateSelector`): authored module NAMES,
    // matched at segment boundaries -- the module-seed rule, not the prefix rule. They join
    // the module seeds, so preparation and site disposition admit them under one rule by
    // construction.
    let required_gate_authored_modules = floor_decode_module_prefix_roster(
        &policy_frame,
        "v2.workflow.required_floor.required_gate_authored_modules",
    )?;
    let local_repo_wet_schedule_rows = local_repo_wet_schedule(&policy_frame)?;
    Ok(RequiredFloorNominalSubjectSeeds {
        required_gate_prefixes,
        required_gate_authored_modules,
        local_repo_wet_schedule_rows,
    })
}

/// The nominal MODULE seeds (`v2.workflow.floor_subject_seed` `PreparedSubjectSeedGround`,
/// matched at segment boundaries): the floor's own runtime authorities, the gate's authored
/// modules, and the wet schedule's entry modules. The prefix seeds travel separately because
/// they are matched by a different rule.
pub(crate) fn required_floor_nominal_closure_module_seeds(
    required_gate_authored_modules: &[String],
    local_repo_wet_schedule_rows: &[LocalRepoWetScheduledRow],
) -> Vec<String> {
    REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES
        .iter()
        .map(|m| m.to_string())
        .chain(required_gate_authored_modules.iter().cloned())
        .chain(
            local_repo_wet_schedule_rows
                .iter()
                .map(|row| row.entry_module.clone()),
        )
        .chain(
            local_repo_wet_schedule_rows
                .iter()
                .any(|row| row.premise.is_some())
                .then(|| LOCAL_REPO_WET_PREMISE_READBACK_MODULE.to_string()),
        )
        .collect()
}

pub fn floor_prepared_subject_exclusions() -> Vec<String> {
    vec![
        // MUST-NOT-RESOLVE PROBES. Every module under dag/test/probe/ declares in its header that
        // it must not resolve: each is a designed refusal, consumed BY CLASS AND SUBJECT through
        // `gunbc.compile_diagnostic_census` `census_of(source: probe_source(name))` -- the source
        // text handed to the compiler by a fixture (DESIGN 4b), never a module in a closure.
        // They stayed out of the prepared subject before gunbc#11256 only by accident: no
        // importer, no gate prefix. The touched-entry seed (`module_seeds_from_touched_entry_files`)
        // seeds the authored module of every edited non-data, non-test-fn declaration, so a PR
        // that edits a probe (gunbc#11343 at 76923b733, run 34874099243:
        // `[floor-phase] phase=touched-entry-compile-subject seeds=6 modules=[...,
        // "test.probe.bare_string_from_boundary_probe"]` while
        // `phase=interface-changed-consumers ... consumers_added=0`) pulled its designed refusal
        // into Strict preparation as a blocker. This row is applied AFTER the seed walk, so the
        // seed still prints (as `TouchedEntryExcludedFromPreparation`, a typed row rather than a
        // vanished seed), and `ExclusionOrphansImporter` refuses the day anything imports one.
        "test/probe/".to_string(),
        "test/fixture/meta_exec_confinement_scan/".to_string(),
        // THE FLEET HEALTH FORBIDDEN PROGRAM. test.fixture.health_read_only.mutation_probe is the
        // program the no-mutation witness derives its findings from, consumed PARSE-ONLY: its
        // callees are undeclared spellings by design, so it must never resolve. Nothing imports it,
        // so it stayed out of preparation by that accident alone until gunbc#12761's widened
        // subject prepared it and it refused. Its designed refusal is asserted by class and subject
        // in test.claim.fleet.health_read_only_fixture_refusal_witness.
        "test/fixture/health_read_only/".to_string(),
        "test/manual/ownership_movable.dag".to_string(),
        // WET RECEIPT, AND IT HAS NO CI CONSUMER TODAY — stated plainly rather than dressed up
        // as an enrollment. case4_expansion_carrier_splices dispatches a real jq through
        // jq.Process.RunWithStdin, which carries no mock_response, so the hermetic floor refuses
        // it and one refusing member fails the run. Mocking is not the repair: the witness exists
        // to prove that a REAL jq exits 0 only on two argv words rather than one concatenated
        // one, and a mocked dispatch passes it without any process running.
        //
        // The first attempt at this fix added rows to gunbc.ci.ci_layer_roots
        // (witness_exclusion_frontier + bin_witness_wet_entries) and asserted they would take
        // effect. They did not: run_required_floor consults THIS list and nothing else, the CI
        // receipt was an unchanged modules_excluded=2, and the wet batches those rosters feed
        // were deleted with the old floor. Adding a row to a roster with no live consumer is
        // specification-without-execution, so both rows were reverted rather than left standing.
        //
        // What this exclusion buys is a green floor; what it does NOT buy is coverage. The claim
        // sits at UNEXECUTED-IN-CI with a local recipe recorded in the design note (§15), and
        // re-enrolls when a wet lane exists again — which required_floor.dag deliberately defers
        // until it can be "asked against a live consumer".
        "test/manual/process_argv_expansion_receipt.dag".to_string(),
        // Wet receipt for command_runner's transport-agnostic run site (the LocalExec/SshExec
        // match collapsed onto command_over_transport). Excluded for the same reason as the line
        // above and not a new class: hermetic evaluation replays an operation's declared
        // mock_response and never constructs an argv, so "the words reached the process unsplit"
        // is not observable hermetically. Enrolling it would assert the mock rather than the
        // behaviour -- specification without execution.
        "test/manual/command_runner_local_argv_receipt.dag".to_string(),
        // A ROW HERE MAY NOT NAME A MODULE ANYTHING IMPORTS. An exclusion is not "skip this
        // witness": prepare_repository_closure drops the path from the prepared graph, so an
        // importer cannot resolve and the whole subject refuses. That constraint is enforced --
        // `assemble_prepared_subject_closure` refuses with cause=ExclusionOrphansImporter, naming
        // the row beside each importer -- rather than asked for here.
        //
        // A row may also not be the way a CHANGED witness gets out of running. The floor admits a
        // changed witness regardless of any exclusion (cause=ChangedWitnessOutsidePreparedSubject),
        // which is what stops this list being an escape hatch a witness can opt into on the commit
        // that introduces it. A corpus-priced assertion therefore is not authored as a witness at
        // all; v2.lens.complexity_accumulator_copy.corpus_census is the worked example, and it
        // carries its own next-rung trigger.
    ]
}

// THE CONSTRUCTOR A DECODE ACTUALLY OBSERVED, for refusals whose cause is a shape mismatch.
//
// A decode arm that reports only "not the expected shape" identifies its seam and nothing else:
// an absent field, a variant, a record and a scalar all render the same, and they have different
// remedies. This names the constructor and, where the constructor is a container, its arity --
// enough to discriminate "the field is missing" from "the field holds an empty list" from "the
// producer answered a variant" without dumping a value whose size is the reason the run is slow.
//
// It deliberately does NOT recurse or print contents. A shape reporter that renders values
// becomes a second serializer, and on this path the values in question are the ones large enough
// to have made the phase expensive in the first place.
pub(crate) fn floor_value_shape(v: Option<&v1_interpreter::Value>) -> String {
    match v {
        None => "<field absent>".to_string(),
        Some(v1_interpreter::Value::List(xs)) => format!("List(len={})", xs.len()),
        Some(v1_interpreter::Value::Record { fields, .. }) => {
            format!("Record(fields={})", fields.len())
        }
        Some(v1_interpreter::Value::Variant { variant_name, .. }) => {
            format!("Variant({variant_name:?})")
        }
        Some(other) => floor_value_constructor(other).to_string(),
    }
}

// ONE CARRIER, TWO REALIZATIONS -- decoded here rather than assumed to be one of them.
//
// `v2.std.collection` declares `List<T> = std.algebra.FreeMonoid<T>`, an `Empty | Cons` chain.
// The interpreter ALSO represents some sequences natively as `Value::List`. Which one arrives at
// a given seam depends on whether the value was produced by a `.dag` fold or handed over by the
// host, and that is the model/realization fork DESIGN names as systemic and unfinished -- not a
// choice this decode gets to make.
//
// So the decode reads the carrier in either realization and REFUSES anything else, naming what
// it saw. What it must not do is assume one realization: the previous decode matched
// `Value::List` alone and rendered a perfectly well-formed `Cons` chain as "the attempt carries
// no claim list", which reports a representation mismatch as an absent population.
//
// The walk is ITERATIVE. A `Cons` chain of one element per claim is 10,444 deep on the measured
// subject, and the recursive spelling already in this file for dependency edges would recurse
// once per element -- fine for a handful of edges, a stack overflow here. Depth belongs on the
// heap when the depth is the population size.
pub(crate) fn floor_decode_list<'a>(
    ctx: &v1_interpreter::InterpContext,
    v: Option<&'a v1_interpreter::Value>,
) -> Result<Vec<&'a v1_interpreter::Value>, String> {
    let mut out = Vec::new();
    let mut cursor = match v {
        None => return Err("<field absent>".to_string()),
        Some(v1_interpreter::Value::List(xs)) => {
            return Ok(xs.iter().collect());
        }
        Some(other) => other,
    };
    loop {
        let v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } = cursor
        else {
            return Err(format!(
                "expected a FreeMonoid Empty/Cons chain, observed {}",
                floor_value_shape(Some(cursor))
            ));
        };
        if ctx.sym_eq(*variant_name, "Empty") {
            return Ok(out);
        }
        if !ctx.sym_eq(*variant_name, "Cons") {
            return Err(format!(
                "expected a FreeMonoid Empty/Cons chain, observed Variant({variant_name:?})"
            ));
        }
        let Some(head) = ctx.field(fields, "head") else {
            return Err("Cons carries no head".to_string());
        };
        let Some(tail) = ctx.field(fields, "tail") else {
            return Err("Cons carries no tail".to_string());
        };
        out.push(head);
        cursor = tail;
    }
}

pub(crate) fn floor_value_constructor(v: &v1_interpreter::Value) -> &'static str {
    match v {
        v1_interpreter::Value::Int(_) => "Int",
        v1_interpreter::Value::Str(_) => "Str",
        v1_interpreter::Value::Bool(_) => "Bool",
        v1_interpreter::Value::Null => "Null",
        _ => "<other constructor>",
    }
}

/// DECODE ONE AUTHORED-MODULE-NAME PREFIX ROSTER from the `.dag` authority.
///
/// NO EMPTY-ROSTER REFUSAL, and the first version of this decode had one. It was backwards: an
/// empty exception roster is the DESIRED terminal state, so refusing on it would make "at least
/// one exclusion must exist forever" a structural requirement of the floor. Once a home is
/// discharged, admitting its witnesses to the ordinary population is the intended result, not an
/// error. A decode that FAILS still refuses — a failed read and a legitimately empty roster are
/// different states, and only the first is a defect.
pub(crate) fn floor_decode_module_prefix_roster(
    hermetic: &v1_interpreter::InterpContext,
    qualified_name: &str,
) -> Result<Vec<String>, String> {
    let value = v1_interpreter::run_in_context(hermetic, qualified_name, false)
        .map_err(|e| format!("{qualified_name}: {e}"))?;
    let items = floor_decode_list(hermetic, Some(&value))
        .map_err(|why| format!("{qualified_name} decode: {why}"))?;
    let mut out = Vec::new();
    for item in items {
        match item {
            v1_interpreter::Value::Str(s) => out.push(s.to_string()),
            other => {
                return Err(format!(
                    "{qualified_name}: expected String rows, got {}",
                    floor_value_shape(Some(other))
                ))
            }
        }
    }
    Ok(out)
}
