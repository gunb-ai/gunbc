// Split from cli_run.rs (pure code motion; no semantic change).
#![allow(unused_imports)]
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
    clippy::unneeded_struct_pattern,
    clippy::useless_vec,
    dead_code,
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
    has_child_named, inferred_to_node, intern, is_error_diagnostic,
    is_interpreter_blocking_diagnostic, let_binding_name_at, let_value, make_error_node,
    match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver, module_items, no_span,
    param_node_name_at, param_node_type_expr, Cardinality, CompilerDiagnostic, Connective,
    ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable, MatchPattern, NewlineIndex,
    Node,
};
use serde::Serialize;

/// Project the path site keys out of the typed `non_fold_residue_frontier` rows of the
/// `gunbc.non_fold_residue` authority SOURCE TEXT via the real front-end — the roster's
/// re-home off this file's former `NON_FOLD_RESIDUE_ROSTER` const (group-of-units ruling,
/// enrolled in `gunbc.roster_registry`). Per-row reasons and dissolution triggers are
/// `.dag`-side facts the host does not consume. Fail-closed: a parse error, a missing data
/// def, a non-record element, a missing/non-literal `subject.path` field, a duplicate path,
/// or an empty roster is a loud panic, never a silent fallback.
// 🟡 dissolve-on: hand-Rust reader over the `.dag` authority — dissolves with
// `witness_exclusion_rows_from_module_source` when the host consumes an emitted manifest of
// the rows (module-binding supply-carrier pattern), and with the scan below into a pure
// `.dag` Node-tree lens at gunbc#5364.
pub(crate) fn non_fold_residue_units_from_module_source(
    module_rel_path: &str,
    content: &str,
) -> Vec<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let filename = module_rel_path.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(
        content.to_string(),
        filename.clone(),
        crate::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = std::rc::Rc::new(source_indices);
    let result = crate::v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "nfr frontier reader: parse error in {module_rel_path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result
        .module
        .as_ref()
        .unwrap_or_else(|| panic!("nfr frontier reader: {module_rel_path} parsed to no module"));
    let data_name = NON_FOLD_RESIDUE_FRONTIER_DATA_NAME;
    for item in module.children.iter() {
        if item.name != data_name
            || item.module_item_kind
                != crate::v1_std_core::ParsedModuleItemKind::ModuleItemDataValue
        {
            continue;
        }
        let body = item.body.as_ref().unwrap_or_else(|| {
            panic!("nfr frontier reader: `data {data_name}` in {module_rel_path} has no value body")
        });
        if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
            panic!(
                "nfr frontier reader: `data {data_name}` in {module_rel_path} is not a list \
                 literal"
            );
        }
        let mut units = Vec::new();
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for el in body.children.iter() {
            if !matches!(el.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
                panic!(
                    "nfr frontier reader: an element of `{data_name}` in {module_rel_path} is \
                     not a record literal (refusing — rows must stay directly host-readable)"
                );
            }
            let mut path: Option<String> = None;
            for field in el.children.iter() {
                let fname = crate::v1_std_core::field_init_node_name_at(
                    field.clone(),
                    source_indices.clone(),
                );
                if fname != "subject" {
                    continue;
                }
                let value = crate::v1_std_core::field_init_node_value(field.clone());
                match value.expr_data.as_ref() {
                    ExprData::ExprRecordLit { .. } => {
                        let variant_name = crate::v1_std_core::authored_name_at(
                            source_indices.clone(),
                            value.clone(),
                        );
                        if variant_name != "PathSubject" {
                            panic!(
                                "nfr frontier reader: `subject` in a `{data_name}` row of \
                                 {module_rel_path} is not PathSubject {{ ... }}"
                            );
                        }
                        let mut row_path: Option<String> = None;
                        for subfield in value.children.iter() {
                            let subname = crate::v1_std_core::field_init_node_name_at(
                                subfield.clone(),
                                source_indices.clone(),
                            );
                            if subname != "path" {
                                continue;
                            }
                            let path_value =
                                crate::v1_std_core::field_init_node_value(subfield.clone());
                            match path_value.expr_data.as_ref() {
                                ExprData::ExprLiteral { value: lit } => match lit.as_ref() {
                                    LiteralValue::LitStr { value: s } => row_path = Some(s.clone()),
                                    _ => panic!(
                                        "nfr frontier reader: `path` in a `{data_name}` row \
                                         of {module_rel_path} is not a string literal"
                                    ),
                                },
                                _ => panic!(
                                    "nfr frontier reader: `path` in a `{data_name}` row of \
                                     {module_rel_path} is not a literal"
                                ),
                            }
                        }
                        path = Some(row_path.unwrap_or_else(|| {
                            panic!(
                                "nfr frontier reader: `subject` in a `{data_name}` row of \
                                 {module_rel_path} is `PathSubject` but carries no `path` field"
                            )
                        }));
                    }
                    _ => panic!(
                        "nfr frontier reader: `subject` in a `{data_name}` row of \
                         {module_rel_path} is not a record literal"
                    ),
                }
            }
            let path = path.unwrap_or_else(|| {
                panic!(
                    "nfr frontier reader: a `{data_name}` row in {module_rel_path} has no \
                     `subject` field"
                )
            });
            if !seen.insert(path.clone()) {
                panic!(
                    "nfr frontier reader: duplicate path {path:?} in `{data_name}` of \
                     {module_rel_path} (the const this replaced tolerated duplicates; the \
                     typed roster refuses them)"
                );
            }
            units.push(path);
        }
        if units.is_empty() {
            panic!(
                "nfr frontier reader: `{data_name}` in {module_rel_path} is empty (fail-closed)"
            );
        }
        return units;
    }
    panic!("nfr frontier reader: no `data {data_name}` def in {module_rel_path}")
}

pub(crate) fn non_fold_residue_roster_entries() -> &'static [String] {
    static ENTRIES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    ENTRIES.get_or_init(|| {
        let path = process_workspace_root().join(NON_FOLD_RESIDUE_AUTHORITY_REL);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "nfr frontier reader: failed to read {}: {e}",
                path.display()
            )
        });
        non_fold_residue_units_from_module_source(NON_FOLD_RESIDUE_AUTHORITY_REL, &content)
    })
}

pub(crate) fn non_fold_residue_roster_set() -> &'static std::collections::BTreeSet<&'static str> {
    static SET: std::sync::OnceLock<std::collections::BTreeSet<&'static str>> =
        std::sync::OnceLock::new();
    SET.get_or_init(|| {
        non_fold_residue_roster_entries()
            .iter()
            .map(|s| s.as_str())
            .collect()
    })
}

pub fn non_fold_residue_closed_coproduct_type_names() -> &'static std::collections::BTreeSet<String>
{
    nfr_closed_coproduct_name_set()
}

pub fn non_fold_residue_site_is_rostered(site: &str) -> bool {
    non_fold_residue_roster_set().contains(site)
}

// ---------------------------------------------------------------------------
// TYPED fallback-arm walk: the non-fold-residue population keyed on the scrutinee's INFERRED type.
//
// Closedness is read from the checker's EXISTING result, never re-derived from syntax: every typed
// `match` keeps its scrutinee's inferred type (`InferredNode::Resolved`), and
// `v1.compiler.infer_patterns` `constructor_roster_for` is the classification
// `check_match_exhaustiveness` itself consults, over the module's own `type_env`. So a local
// binding, a field projection or a call is classified exactly like a parameter -- the population
// the deleted parameter-keyed text scan could not see by construction. This walk is the one site
// authority: the floor runs it per diff, and the whole-corpus census below derives the roster.
//
// The required floor runs it DIFF-SCOPED over the graph its strict preparation already typed
// (`non_fold_residue_diff_verdict`): no second compile. What that scope cannot see is declared,
// not implied -- gunbc.recurring_failure_mode non_fold_residue_diff_scope_misses_an_untouched_flip.
// v1 standing (`gunbc.v1_maintenance_standing` v1_seed_standing): admitted under the purpose test,
// it adds no analysis to the frozen 04_* stages and only reads their output. It dissolves into
// v2.lens.fallback_arm_census when v2.compiler.infer types coproduct match scrutinees.
// ---------------------------------------------------------------------------

/// Closed-total verdict of a typed scrutinee. `Undetermined` is its own arm (a scrutinee whose
/// inferred type is not a resolved node), counted by the caller, never folded into `Open`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TypedScrutineeClosedness {
    Closed,
    Open,
    Undetermined,
}

pub(crate) fn typed_scrutinee_closedness(
    scrutinee: &Rc<Node>,
    env: &Rc<TypeEnv>,
) -> TypedScrutineeClosedness {
    match scrutinee.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => {
            match crate::v1_compiler_infer_patterns::constructor_roster_for(
                node.clone(),
                env.clone(),
            )
            .as_ref()
            {
                crate::v1_compiler_infer_patterns::ConstructorRoster::ConstructorsClosed {
                    ..
                } => TypedScrutineeClosedness::Closed,
                crate::v1_compiler_infer_patterns::ConstructorRoster::ConstructorsOpen => {
                    TypedScrutineeClosedness::Open
                }
            }
        }
        _ => TypedScrutineeClosedness::Undetermined,
    }
}

/// Every wildcard arm in a typed graph, one fact per arm, keyed `rel::decl#armN`.
#[derive(Debug, Default)]
pub(crate) struct TypedFallbackArmWalk {
    pub facts: Vec<FallbackArmCensusFactRaw>,
    /// `rel::decl` sites whose wildcard arm's scrutinee type was not a resolved node.
    pub undetermined_sites: Vec<String>,
    /// Repo-relative paths of every non-test module the walk covered.
    pub covered_paths: BTreeSet<String>,
    /// Module paths of every non-test module the walk covered.
    pub covered_modules: BTreeSet<String>,
    /// Repo-relative path -> module path, for every covered module.
    pub module_of_path: BTreeMap<String, String>,
}

impl TypedFallbackArmWalk {
    /// The non-fold-residue population: `rel::decl` of every wildcard over a closed coproduct.
    pub(crate) fn non_fold_residue_sites(&self) -> BTreeSet<String> {
        self.facts
            .iter()
            .filter(|f| f.closed_coproduct_scrutinee)
            .map(|f| format!("{}::{}", f.rel_path, f.fn_name))
            .collect()
    }
}

/// Walk every typed module of `graph`, or only those whose module path is in `module_filter`.
pub(crate) fn typed_fallback_arm_walk(
    graph: &ResolvedGraph,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    module_filter: Option<&BTreeSet<String>>,
) -> TypedFallbackArmWalk {
    let mut walk = TypedFallbackArmWalk::default();
    for tm in graph.modules.iter() {
        let module_path = authored_name_at(si.clone(), tm.module.clone());
        if module_filter.is_some_and(|keep| !keep.contains(&module_path)) {
            continue;
        }
        let rel = rel_path_for_layer_import(Path::new(&tm.module.span.file));
        if is_test_dag(&rel) {
            continue;
        }
        walk.covered_paths.insert(rel.clone());
        walk.module_of_path.insert(rel.clone(), module_path.clone());
        walk.covered_modules.insert(module_path);
        for item in tm.items.iter() {
            let Some(body) = item.body.as_ref() else {
                continue;
            };
            let name = authored_name_at(si.clone(), item.clone());
            if name.is_empty() {
                continue;
            }
            typed_collect_wildcard_arms(body, si, &tm.type_env, &name, &rel, &mut walk);
        }
    }
    walk.facts.sort();
    walk.undetermined_sites.sort();
    walk.undetermined_sites.dedup();
    walk
}

fn typed_collect_wildcard_arms(
    node: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    env: &Rc<TypeEnv>,
    decl: &str,
    rel: &str,
    walk: &mut TypedFallbackArmWalk,
) {
    // An explicit worklist, not native recursion: a typed body's depth is corpus-shaped, and the
    // floor runs this walk on the main thread (gunbc.recurring_failure_mode class of #10610).
    // Visit order is immaterial: the caller sorts `facts` and `undetermined_sites`.
    let mut pending: Vec<&Rc<Node>> = vec![node];
    while let Some(node) = pending.pop() {
        if let ExprData::ExprMatch = node.expr_data.as_ref() {
            let arms = match_arm_nodes(node.clone());
            if arms.iter().any(cla_is_wildcard_arm) {
                let closedness = typed_scrutinee_closedness(&match_scrutinee(node.clone()), env);
                if closedness == TypedScrutineeClosedness::Undetermined {
                    walk.undetermined_sites.push(format!("{rel}::{decl}"));
                }
                let closed = closedness == TypedScrutineeClosedness::Closed;
                for (arm_idx, arm) in arms.iter().enumerate() {
                    if !cla_is_wildcard_arm(arm) {
                        continue;
                    }
                    // DeclaredInterim needs a typed arm-to-FrontierRow join the host does not have;
                    // it stays false here exactly as the parse-level fac walk leaves it.
                    let class = fac_classify_arm(&arm_body(arm.clone()), si, closed, false);
                    walk.facts.push(FallbackArmCensusFactRaw {
                        site: format!("{rel}::{decl}#arm{arm_idx}"),
                        fn_name: decl.to_string(),
                        rel_path: rel.to_string(),
                        class: class.to_string(),
                        owning_lane: fac_owning_lane(rel).to_string(),
                        closed_coproduct_scrutinee: closed,
                    });
                }
            }
        }
        pending.extend(node.children.iter());
    }
}

/// (unrostered live sites, stale roster rows) over one typed walk, against the live roster.
pub(crate) fn non_fold_residue_roster_diff(
    walk: &TypedFallbackArmWalk,
) -> (Vec<String>, Vec<String>) {
    let root = process_workspace_root();
    non_fold_residue_roster_diff_with(walk, non_fold_residue_roster_entries(), &|path: &str| {
        root.join(path).is_file()
    })
}

/// The pure core. A roster row is STALE when its site's module was typed by this walk and no
/// longer carries the site, OR when its subject file does not exist at all -- a deleted module
/// never enters `covered_paths`, so without the existence arm its row would outlive it silently.
pub(crate) fn non_fold_residue_roster_diff_with(
    walk: &TypedFallbackArmWalk,
    roster_rows: &[String],
    path_exists: &dyn Fn(&str) -> bool,
) -> (Vec<String>, Vec<String>) {
    let live = walk.non_fold_residue_sites();
    // A ROSTERED UNDETERMINED SITE IS NOT STALE. The diff verdict refuses an undetermined
    // scrutinee unless a row names it (`undetermined_unrostered`), so such a row is one the walk
    // itself demands; judging it stale because it is not a closed-coproduct site made both
    // dispositions refuse, and any change touching the module could never pass.
    let undetermined: BTreeSet<&str> = walk.undetermined_sites.iter().map(|s| s.as_str()).collect();
    let rostered: BTreeSet<&str> = roster_rows.iter().map(|r| r.as_str()).collect();
    let unrostered = live
        .iter()
        .filter(|s| !rostered.contains(s.as_str()))
        .cloned()
        .collect();
    let stale = roster_rows
        .iter()
        .filter(|e| {
            let path = e.split("::").next().unwrap_or("");
            !path_exists(path)
                || (walk.covered_paths.contains(path)
                    && !live.contains(e.as_str())
                    && !undetermined.contains(e.as_str()))
        })
        .cloned()
        .collect();
    (unrostered, stale)
}

/// Subject paths of every roster row the diff ADDED or DELETED (base vs head). A row-only edit
/// touches the roster module and not the subject, so without these the typed scope never reaches
/// the site the row claims about.
pub(crate) fn non_fold_residue_changed_row_paths(
    base_rows: &[String],
    head_rows: &[String],
) -> BTreeSet<String> {
    let base: BTreeSet<&String> = base_rows.iter().collect();
    let head: BTreeSet<&String> = head_rows.iter().collect();
    base.symmetric_difference(&head)
        .map(|row| row.split("::").next().unwrap_or("").to_string())
        .collect()
}

/// A changed roster row's subject, by its EXACT path, with a typed disposition. Nothing is
/// dropped: a path that exists but cannot be read or declares no module refuses, and a readable
/// module is seeded AND must then appear by that same path among the walk's covered paths.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NonFoldResidueRowSubject {
    /// Absent at head: the row is judged by the existence arm of the roster diff (stale).
    Missing {
        path: String,
    },
    Unreadable {
        path: String,
        detail: String,
    },
    NoModuleDeclaration {
        path: String,
    },
    Module {
        path: String,
        module: String,
    },
}

pub(crate) fn non_fold_residue_classify_row_subject(
    path: &str,
    exists: bool,
    read: Result<String, String>,
) -> NonFoldResidueRowSubject {
    if !exists {
        return NonFoldResidueRowSubject::Missing {
            path: path.to_string(),
        };
    }
    match read {
        Err(detail) => NonFoldResidueRowSubject::Unreadable {
            path: path.to_string(),
            detail,
        },
        Ok(content) => match extract_module_path(&content) {
            Some(module) => NonFoldResidueRowSubject::Module {
                path: path.to_string(),
                module,
            },
            None => NonFoldResidueRowSubject::NoModuleDeclaration {
                path: path.to_string(),
            },
        },
    }
}

/// The subjects of every roster row the diff added or deleted, read at the floor's resolved diff
/// base and classified by exact path. Refuses when the base roster cannot be read.
pub(crate) fn non_fold_residue_changed_row_subjects(
) -> Result<Vec<NonFoldResidueRowSubject>, String> {
    let base_rows = match floor_base_file_read(NON_FOLD_RESIDUE_AUTHORITY_REL)? {
        None => Vec::new(),
        Some(content) => {
            non_fold_residue_units_from_module_source(NON_FOLD_RESIDUE_AUTHORITY_REL, &content)
        }
    };
    let root = process_workspace_root();
    Ok(
        non_fold_residue_changed_row_paths(&base_rows, non_fold_residue_roster_entries())
            .into_iter()
            .map(|path| {
                let full = root.join(&path);
                let read = std::fs::read_to_string(&full).map_err(|e| e.to_string());
                non_fold_residue_classify_row_subject(&path, full.exists(), read)
            })
            .collect(),
    )
}

/// The seeds a subject set contributes: the module of every readable, module-declaring subject.
pub(crate) fn non_fold_residue_row_subject_seeds(
    subjects: &[NonFoldResidueRowSubject],
) -> BTreeSet<String> {
    subjects
        .iter()
        .filter_map(|s| match s {
            NonFoldResidueRowSubject::Module { module, .. } => Some(module.clone()),
            NonFoldResidueRowSubject::Missing { .. }
            | NonFoldResidueRowSubject::Unreadable { .. }
            | NonFoldResidueRowSubject::NoModuleDeclaration { .. } => None,
        })
        .collect()
}

/// Every changed-row subject this run could not judge by its exact path: unreadable, module-less,
/// or a module path the walk did not cover (not typed, excluded as a test file, or a same-named
/// module at another path). Missing subjects are absent here -- the roster diff judges them stale.
pub(crate) fn non_fold_residue_row_subjects_unjudged(
    subjects: &[NonFoldResidueRowSubject],
    walk: &TypedFallbackArmWalk,
) -> Vec<String> {
    subjects
        .iter()
        .filter_map(|s| match s {
            NonFoldResidueRowSubject::Missing { .. } => None,
            NonFoldResidueRowSubject::Unreadable { path, detail } => {
                Some(format!("{path} (unreadable: {detail})"))
            }
            NonFoldResidueRowSubject::NoModuleDeclaration { path } => {
                Some(format!("{path} (declares no module)"))
            }
            NonFoldResidueRowSubject::Module { path, module } => {
                if walk.covered_paths.contains(path)
                    && walk.module_of_path.get(path) == Some(module)
                {
                    None
                } else {
                    Some(format!(
                        "{path} (module {module} not covered by the typed walk at this path)"
                    ))
                }
            }
        })
        .collect()
}

/// The required floor's DIFF-SCOPED non-fold-residue verdict over the graph its strict preparation
/// already typed: every module the diff touched plus every interface consumer it planned. A
/// wildcard is added or removed only by editing its module, and a scrutinee's closedness moves
/// with a changed interface only in a planned consumer, so within this scope a new unrostered site
/// and a newly stale row are both caught. A scoped module the graph does not carry is reported by
/// name, never counted as clean.
pub(crate) struct NonFoldResidueDiffVerdict {
    pub scoped_modules: usize,
    pub walk: TypedFallbackArmWalk,
    pub scoped_but_untyped: Vec<String>,
    pub unrostered: Vec<String>,
    pub stale: Vec<String>,
    /// Scoped sites whose wildcard scrutinee carries no resolved inferred type and no roster row
    /// of their own: closedness is undecided, so they refuse rather than pass as open.
    pub undetermined_unrostered: Vec<String>,
    /// Changed roster rows whose subject this run could not judge at its exact path: a row edit
    /// that cannot be judged REFUSES rather than passing unexamined.
    pub row_subjects_untyped: Vec<String>,
}

pub(crate) fn non_fold_residue_diff_verdict(
    graph: &ResolvedGraph,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    scoped_modules: &BTreeSet<String>,
    row_subjects: &[NonFoldResidueRowSubject],
) -> NonFoldResidueDiffVerdict {
    let walk = typed_fallback_arm_walk(graph, si, Some(scoped_modules));
    let typed: BTreeSet<String> = graph
        .modules
        .iter()
        .map(|tm| authored_name_at(si.clone(), tm.module.clone()))
        .collect();
    let scoped_but_untyped = scoped_modules
        .iter()
        .filter(|m| !typed.contains(*m))
        .cloned()
        .collect();

    let (unrostered, stale) = non_fold_residue_roster_diff(&walk);
    let undetermined_unrostered = walk
        .undetermined_sites
        .iter()
        .filter(|s| !non_fold_residue_site_is_rostered(s))
        .cloned()
        .collect();
    let row_subjects_untyped = non_fold_residue_row_subjects_unjudged(row_subjects, &walk);
    NonFoldResidueDiffVerdict {
        scoped_modules: scoped_modules.len(),
        walk,
        scoped_but_untyped,
        unrostered,
        stale,
        undetermined_unrostered,
        row_subjects_untyped,
    }
}

/// Type a fixture through the REAL checker and walk it: the controls exercise the same route the
/// floor reads (compile, then `typed_fallback_arm_walk`), never a copy of it.
#[cfg(test)]
pub(crate) fn typed_fallback_arm_walk_for_fixture(
    files: &[(&str, &str)],
) -> Result<TypedFallbackArmWalk, String> {
    let sources: Vec<Rc<v1_compiler_compile::SourceFile>> = files
        .iter()
        .map(|(path, content)| {
            Rc::new(v1_compiler_compile::SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        })
        .collect();
    let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
    let blocking =
        v1_compiler_compile::interpreter_blocking_diagnostic_messages(resolved.diagnostics.clone());
    if !blocking.is_empty() {
        return Err(format!(
            "typed fixture did not type: {}",
            blocking.iter().cloned().collect::<Vec<_>>().join("; ")
        ));
    }
    let graph = resolved
        .graph
        .clone()
        .ok_or_else(|| "typed fixture produced no graph".to_string())?;
    Ok(typed_fallback_arm_walk(
        &graph,
        &resolved.source_indices,
        None,
    ))
}

#[cfg(test)]
mod nfr_typed_tests {
    use super::*;

    fn sites(src: &str) -> BTreeSet<String> {
        typed_fallback_arm_walk_for_fixture(&[("m.dag", src)])
            .unwrap_or_else(|cause| panic!("{cause}"))
            .non_fold_residue_sites()
    }

    #[test]
    fn red_control_wildcard_over_closed_coproduct_param_is_residue() {
        let got = sites("module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n");
        assert!(
            got.contains("m.dag::f"),
            "param scrutinee must be flagged; got {got:?}"
        );
    }

    // THE WIDENING'S DISCRIMINATING REDS: the deleted parameter-keyed text scan could see neither
    // (receipts: gunbc.live_deploy.fleet_request replacement_is_foreign's local `from`,
    // release_member_changed).
    #[test]
    fn red_control_wildcard_over_closed_coproduct_local_binding_is_residue() {
        let got = sites("module m\ntype Mode = A | B | C\nfn pick() -> Mode { B }\nfn f() -> Bool {\n  let from = pick()\n  match from {\n    A => true\n    _ => false\n  }\n}\n");
        assert!(
            got.contains("m.dag::f"),
            "local-binding scrutinee must be flagged; got {got:?}"
        );
    }

    #[test]
    fn red_control_wildcard_over_closed_coproduct_field_is_residue() {
        let got = sites("module m\ntype Mode = A | B | C\ntype Holder { mode: Mode }\nfn f(h: Holder) -> Bool {\n  match h.mode {\n    A => true\n    _ => false\n  }\n}\n");
        assert!(
            got.contains("m.dag::f"),
            "field scrutinee must be flagged; got {got:?}"
        );
    }

    #[test]
    fn green_control_total_fold_is_not_residue() {
        let got = sites("module m\ntype Mode = A | B | C\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    B => false\n    C => false\n  }\n}\n");
        assert!(
            !got.contains("m.dag::f"),
            "an exhaustive match must NOT be flagged; got {got:?}"
        );
    }

    #[test]
    fn green_control_wildcard_over_open_domain_is_not_residue() {
        let got = sites("module m\nfn g(s: String) -> Bool {\n  match s {\n    \"y\" => true\n    _ => false\n  }\n}\n");
        assert!(
            !got.contains("m.dag::g"),
            "an open/primitive domain must NOT be flagged; got {got:?}"
        );
    }

    #[test]
    fn green_control_wildcard_over_open_local_is_not_residue() {
        let got = sites("module m\nfn g(n: Int) -> Bool {\n  let k = n\n  match k {\n    0 => true\n    _ => false\n  }\n}\n");
        assert!(
            !got.contains("m.dag::g"),
            "an open local must NOT be flagged; got {got:?}"
        );
    }

    const WILDCARD_MODULE: &str = "module a\ntype Mode = A | B\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n";
    const TOTAL_MODULE: &str = "module a\ntype Mode = A | B\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    B => false\n  }\n}\n";

    fn rows(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| x.to_string()).collect()
    }

    // RED CONTROL 1 -- row-only ADDITION of a stale site: the diff adds a row for a site the
    // module does not carry. The changed-row set names the module, and once it is typed the row
    // is stale.
    #[test]
    fn red_control_row_only_addition_of_a_stale_site_is_judged() {
        let changed = non_fold_residue_changed_row_paths(&rows(&[]), &rows(&["a.dag::f"]));
        assert_eq!(changed, ["a.dag".to_string()].into_iter().collect());
        let walk = typed_fallback_arm_walk_for_fixture(&[("a.dag", TOTAL_MODULE)])
            .unwrap_or_else(|c| panic!("{c}"));
        let (unrostered, stale) =
            non_fold_residue_roster_diff_with(&walk, &rows(&["a.dag::f"]), &|_| true);
        assert!(unrostered.is_empty());
        assert_eq!(
            stale,
            rows(&["a.dag::f"]),
            "an added row for a non-site must be stale"
        );
    }

    // RED CONTROL 2 -- row-only DELETION of a live site: the changed-row set names the module, and
    // the live wildcard is then unrostered.
    #[test]
    fn red_control_row_only_deletion_of_a_live_site_is_judged() {
        let changed = non_fold_residue_changed_row_paths(&rows(&["a.dag::f"]), &rows(&[]));
        assert_eq!(changed, ["a.dag".to_string()].into_iter().collect());
        let walk = typed_fallback_arm_walk_for_fixture(&[("a.dag", WILDCARD_MODULE)])
            .unwrap_or_else(|c| panic!("{c}"));
        let (unrostered, stale) = non_fold_residue_roster_diff_with(&walk, &rows(&[]), &|_| true);
        assert_eq!(
            unrostered,
            rows(&["a.dag::f"]),
            "deleting a live site's row must refuse"
        );
        assert!(stale.is_empty());
    }

    // RED CONTROL 3 -- subject MODULE deleted, row retained: the deleted path never enters
    // covered_paths, so only the existence arm can judge it.
    #[test]
    fn red_control_deleted_subject_module_with_retained_row_is_stale() {
        let walk = TypedFallbackArmWalk::default();
        let (_, stale) =
            non_fold_residue_roster_diff_with(&walk, &rows(&["gone.dag::f"]), &|p| p != "gone.dag");
        assert_eq!(stale, rows(&["gone.dag::f"]));
        let (_, kept) =
            non_fold_residue_roster_diff_with(&walk, &rows(&["here.dag::f"]), &|_| true);
        assert!(
            kept.is_empty(),
            "an existing, unscoped row is not judged stale (positive control)"
        );
    }

    // CONTROL 3b -- a row naming an UNDETERMINED site is the disposition the verdict demands, so it
    // is not stale; the same row over a covered path with no wildcard at all still is.
    #[test]
    fn rostered_undetermined_site_is_not_stale() {
        let mut walk = TypedFallbackArmWalk::default();
        walk.covered_paths.insert("u.dag".to_string());
        walk.undetermined_sites.push("u.dag::f".to_string());
        let (unrostered, stale) =
            non_fold_residue_roster_diff_with(&walk, &rows(&["u.dag::f"]), &|_| true);
        assert!(unrostered.is_empty());
        assert!(
            stale.is_empty(),
            "a rostered undetermined site must not be stale; got {stale:?}"
        );
        let (_, stale) = non_fold_residue_roster_diff_with(&walk, &rows(&["u.dag::g"]), &|_| true);
        assert_eq!(
            stale,
            rows(&["u.dag::g"]),
            "a row for a non-site in the same covered module stays stale (red control)"
        );
    }

    // RED CONTROLS 4-6 -- a changed row's subject that cannot be judged at its EXACT path refuses.
    // They drive the real route: classify the path by reading it, then judge against the walk.
    #[test]
    fn red_control_row_subject_existing_non_module_path_refuses() {
        let subject = non_fold_residue_classify_row_subject(
            "Cargo.toml",
            true,
            Ok("[workspace]\nmembers = []\n".to_string()),
        );
        assert_eq!(
            subject,
            NonFoldResidueRowSubject::NoModuleDeclaration {
                path: "Cargo.toml".to_string()
            }
        );
        let unjudged =
            non_fold_residue_row_subjects_unjudged(&[subject], &TypedFallbackArmWalk::default());
        assert_eq!(
            unjudged.len(),
            1,
            "a module-less existing path must refuse, not drop"
        );
    }

    #[test]
    fn red_control_row_subject_unreadable_path_refuses() {
        let subject = non_fold_residue_classify_row_subject(
            "x.dag",
            true,
            Err("permission denied".to_string()),
        );
        let unjudged =
            non_fold_residue_row_subjects_unjudged(&[subject], &TypedFallbackArmWalk::default());
        assert_eq!(unjudged.len(), 1);
    }

    #[test]
    fn red_control_row_subject_excluded_test_path_refuses() {
        // The walk types the module but excludes `_test.dag` paths, so the row's exact path is
        // never covered: the row must refuse rather than pass unjudged.
        let src = "module t\ntype Mode = A | B\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n";
        let subject =
            non_fold_residue_classify_row_subject("dag/test/x_test.dag", true, Ok(src.to_string()));
        assert_eq!(
            subject,
            NonFoldResidueRowSubject::Module {
                path: "dag/test/x_test.dag".to_string(),
                module: "t".to_string()
            }
        );
        let walk = typed_fallback_arm_walk_for_fixture(&[("dag/test/x_test.dag", src)])
            .unwrap_or_else(|c| panic!("{c}"));
        assert!(!walk.covered_paths.contains("dag/test/x_test.dag"));
        assert_eq!(
            non_fold_residue_row_subjects_unjudged(&[subject], &walk).len(),
            1
        );
    }

    #[test]
    fn green_control_row_subject_covered_at_its_path_is_judged() {
        let subject =
            non_fold_residue_classify_row_subject("a.dag", true, Ok(WILDCARD_MODULE.to_string()));
        let walk = typed_fallback_arm_walk_for_fixture(&[("a.dag", WILDCARD_MODULE)])
            .unwrap_or_else(|c| panic!("{c}"));
        assert!(non_fold_residue_row_subjects_unjudged(&[subject], &walk).is_empty());
        let missing = non_fold_residue_classify_row_subject("gone.dag", false, Err(String::new()));
        assert!(
            non_fold_residue_row_subjects_unjudged(&[missing], &walk).is_empty(),
            "a missing subject is judged stale by the roster diff, not refused here"
        );
    }

    #[test]
    fn diff_scope_judges_only_scoped_modules() {
        let walk = typed_fallback_arm_walk_for_fixture(&[
            ("a.dag", "module a\ntype Mode = A | B\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n"),
            ("b.dag", "module b\nfn g(n: Int) -> Int { n }\n"),
        ])
        .unwrap_or_else(|cause| panic!("{cause}"));
        assert!(walk.non_fold_residue_sites().contains("a.dag::f"));
        let sources: Vec<Rc<v1_compiler_compile::SourceFile>> = [
            ("a.dag", "module a\ntype Mode = A | B\nfn f(x: Mode) -> Bool {\n  match x {\n    A => true\n    _ => false\n  }\n}\n"),
            ("b.dag", "module b\nfn g(n: Int) -> Int { n }\n"),
        ]
        .iter()
        .map(|(p, c)| Rc::new(v1_compiler_compile::SourceFile { path: p.to_string(), content: c.to_string() }))
        .collect();
        let resolved = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
        let graph = resolved.graph.clone().expect("graph");
        let scope: BTreeSet<String> = ["b".to_string(), "absent.module".to_string()]
            .into_iter()
            .collect();
        let verdict = non_fold_residue_diff_verdict(&graph, &resolved.source_indices, &scope, &[]);
        assert!(
            verdict.unrostered.is_empty(),
            "module a is out of scope; got {:?}",
            verdict.unrostered
        );
        assert_eq!(
            verdict.scoped_but_untyped,
            vec!["absent.module".to_string()]
        );
        let scope_a: BTreeSet<String> = ["a".to_string()].into_iter().collect();
        let verdict_a =
            non_fold_residue_diff_verdict(&graph, &resolved.source_indices, &scope_a, &[]);
        assert_eq!(
            verdict_a.unrostered,
            vec!["a.dag::f".to_string()],
            "in scope and unrostered must refuse"
        );
    }

    /// Discriminating control for the stack class of #10610 on the floor's own call: a body that
    /// is a `NFR_DEEP_BODY_DEPTH`-term `+` chain (a left-nested binary tree in the typed graph),
    /// typed on a large-stack thread (typing is not the subject), then judged by
    /// `non_fold_residue_diff_verdict` on an 8 MiB thread -- the Linux main-thread size the
    /// required floor runs it on. Measured on the natively recursive walk: 12,000 terms complete,
    /// 20,000 abort with a stack overflow (the red gunbc#12526's floor hit); the worklist
    /// completes at both and still reports the module's residue.
    #[test]
    fn deep_body_is_judged_on_a_main_thread_sized_stack() {
        const NFR_DEEP_BODY_DEPTH: usize = 20_000;
        let mut src = String::from(
            "module m\ntype Mode = A | B | C\nfn g(x: Mode) -> Int {\n  match x {\n    A => 1\n    _ => 0\n  }\n}\nfn f(x: Mode) -> Int {\n  g(x: x)",
        );
        for _ in 1..NFR_DEEP_BODY_DEPTH {
            src.push_str(" + 1");
        }
        src.push_str("\n}\n");
        std::thread::scope(|scope| {
            let typed = std::thread::Builder::new()
                .stack_size(1 << 30)
                .spawn_scoped(scope, || {
                    let sources: Vec<Rc<v1_compiler_compile::SourceFile>> =
                        vec![Rc::new(v1_compiler_compile::SourceFile {
                            path: "m.dag".to_string(),
                            content: src.clone(),
                        })];
                    let resolved =
                        v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
                    let blocking = v1_compiler_compile::interpreter_blocking_diagnostic_messages(
                        resolved.diagnostics.clone(),
                    );
                    assert!(
                        blocking.is_empty(),
                        "deep fixture did not type: {blocking:?}"
                    );
                    let graph = resolved
                        .graph
                        .clone()
                        .expect("deep fixture produced no graph");
                    let scoped: BTreeSet<String> = ["m".to_string()].into_iter().collect();
                    // The typed graph is `Rc`-shared and so not `Send`; this thread blocks on the
                    // join below and touches none of it meanwhile, so exactly one thread uses it
                    // at a time.
                    struct HandOff<T>(T);
                    unsafe impl<T> Send for HandOff<T> {}
                    let handed = HandOff((&graph, &resolved.source_indices, &scoped));
                    let judged = std::thread::scope(|inner| {
                        std::thread::Builder::new()
                            .stack_size(8 * 1024 * 1024)
                            .spawn_scoped(inner, move || {
                                let handed = handed;
                                let (graph, indices, scoped) = handed.0;
                                non_fold_residue_diff_verdict(graph, indices, scoped, &[])
                                    .walk
                                    .non_fold_residue_sites()
                            })
                            .expect("spawn main-sized thread")
                            .join()
                            .expect("verdict thread panicked")
                    });
                    judged
                })
                .expect("spawn typing thread")
                .join()
                .expect("typing thread panicked");
            assert!(
                typed.contains("m.dag::g"),
                "the walk must complete the deep module and report its residue; got {typed:?}"
            );
        });
    }
}

/// THE ONE-TIME WHOLE-CORPUS TYPED CENSUS behind the widened roster: every module under the source
/// roots compiled in ONE resolution (the `gunbc compile --source-root` primary-root route:
/// `primary_root_subject_closure` per root, one `compile_to_resolved_with_options` over the union
/// under the compile-clean admission), then the typed walk over every module of that graph. It is
/// an operator run (measured on srv1 at 2a1dd41955f, 2026-09-29: 36.5 min wall, 39.9 GB peak RSS), not
/// a merge-path check -- the
/// merge path is the floor's diff-scoped verdict. Output: one line per unrostered site, stale row,
/// undetermined scrutinee and blocked module, written to `GUNBC_NFR_CENSUS_OUT`.
#[cfg(test)]
mod nfr_whole_corpus_census {
    use super::*;

    #[test]
    #[ignore = "operator run: whole-corpus typed NFR census (~40 GB peak RSS); see the fn doc"]
    fn nfr_whole_corpus_typed_census() {
        let out_path = std::env::var("GUNBC_NFR_CENSUS_OUT")
            .unwrap_or_else(|_| "target/nfr-typed-census.txt".to_string());
        let roots = vec!["dag".to_string(), "src/v2".to_string()];
        let report = std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("nfr-typed-census".to_string())
                .stack_size(1 << 30)
                .spawn_scoped(scope, || nfr_whole_corpus_typed_census_report(&roots))
                .expect("spawn census thread")
                .join()
                .expect("census thread panicked")
        });
        std::fs::write(&out_path, &report).expect("write census output");
        println!(
            "nfr-typed-census: wrote {out_path}\n{}",
            report.lines().next().unwrap_or("")
        );
    }

    fn nfr_whole_corpus_typed_census_report(roots: &[String]) -> String {
        std::env::set_current_dir(process_workspace_root()).expect("cd workspace root");
        let index = build_multi_entry_index(&pool_roots_abs(roots));
        let mut by_path: BTreeMap<String, Rc<v1_compiler_compile::SourceFile>> = BTreeMap::new();
        for root in roots {
            match primary_root_subject_closure(&index, root) {
                Ok(closure) => {
                    for source in closure {
                        by_path.insert(source.path.clone(), source);
                    }
                }
                Err(refusal) => panic!(
                    "closure of root '{root}' refused at {}: {}",
                    refusal.phase, refusal.cause
                ),
            }
        }
        let closure: Vec<Rc<v1_compiler_compile::SourceFile>> = by_path.into_values().collect();
        let sources = closure.len();
        let options = compile_clean_pipeline_options_for_sources(Some(&index), &closure);
        let resolved =
            v1_compiler_compile::compile_to_resolved_with_options(Rc::new(closure.into()), options);
        let mut blocked: BTreeSet<String> = BTreeSet::new();
        let mut blocked_modules: BTreeSet<String> = BTreeSet::new();
        for d in resolved.diagnostics.iter() {
            if is_interpreter_blocking_diagnostic(d.diagnostic.clone()) {
                blocked_modules.insert(d.module_name.clone());
                blocked.insert(format!(
                    "blocked-module {} {}",
                    d.module_name,
                    diagnostic_to_message(d.diagnostic.clone())
                        .replace('\n', " ")
                        .chars()
                        .take(200)
                        .collect::<String>()
                ));
            }
        }
        let graph = resolved.graph.clone().unwrap_or_else(|| {
            panic!(
                "the corpus did not resolve to a graph ({} blocking diagnostics)",
                blocked.len()
            )
        });
        let walk = typed_fallback_arm_walk(&graph, &resolved.source_indices, None);
        let (unrostered, stale) = non_fold_residue_roster_diff(&walk);
        let mut lines = vec![format!(
            "summary sources={} typed_modules={} wildcard_arms={} residue_sites={} unrostered={} stale={} undetermined={} blocked_diagnostics={}",
            sources,
            walk.covered_modules.len(),
            walk.facts.len(),
            walk.non_fold_residue_sites().len(),
            unrostered.len(),
            stale.len(),
            walk.undetermined_sites.len(),
            blocked.len()
        )];
        // A site in a module carrying a blocking diagnostic was read from a possibly partial
        // typing: it is tagged, never listed as an ordinary site (nor its absence read as clean).
        let in_blocked = |site: &String| {
            let path = site.split("::").next().unwrap_or("");
            walk.module_of_path
                .get(path)
                .is_some_and(|m| blocked_modules.contains(m))
        };
        lines.extend(unrostered.iter().map(|s| {
            if in_blocked(s) {
                format!("unrostered-in-blocked-module {s}")
            } else {
                format!("unrostered {s}")
            }
        }));
        lines.extend(
            blocked_modules
                .iter()
                .filter(|m| walk.covered_modules.contains(*m))
                .map(|m| format!("blocked-module-typed-partially {m}")),
        );
        lines.extend(stale.iter().map(|s| format!("stale {s}")));
        lines.extend(
            walk.undetermined_sites
                .iter()
                .map(|s| format!("undetermined {s}")),
        );
        lines.extend(blocked);
        lines.join("\n") + "\n"
    }
}
