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
        // THE BASE COULD NOT BE NAMED: the model refused, and the refusal is carried to the
        // caller verbatim rather than folded into the catch-all below. The round stops here --
        // this arm is reached before the scope has selected a mirror, so nothing has been
        // written. gunbc.design_argument::fail_closed: a missing input is a located refusal, never a scope.
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "UnifiedBaseRefUnresolvable") => {
            match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => Err(r.to_string()),
                _ => Err(
                    "the changed-mirror scope's base ref could not be resolved in this clone; \
                     the round refuses rather than computing a scope from a base it cannot name"
                        .to_string(),
                ),
            }
        }
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
        // THE BASE COULD NOT BE NAMED (see the unified sibling above): the located refusal is
        // carried through, and the round stops before it selects or writes anything.
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "NameStatusBaseRefUnresolvable") => {
            match ctx.field(fields, "reason") {
                Some(Value::Str(r)) => Err(r.to_string()),
                _ => Err(
                    "the changed-mirror scope's base ref could not be resolved in this clone; \
                     the round refuses rather than computing a scope from a base it cannot name"
                        .to_string(),
                ),
            }
        }
        other => Err(format!(
            "floor_observe_git_diff_name_status_for_ci returned `{}`, expected FloorNameStatusDiffResult",
            ctx.format_value(other)
        )),
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
        // projection (gunbc.design_argument::single_authority — a path is a discriminator, the authored module name is the fact).
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
        // text handed to the compiler by a fixture (gunbc.design_argument::guarantee_ladder), never a module in a closure.
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
