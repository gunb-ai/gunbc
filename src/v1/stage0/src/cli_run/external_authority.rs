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
    has_child_named, inferred_to_node, intern, is_discovery_corpus_blocking_diagnostic,
    is_error_diagnostic, is_interpreter_blocking_diagnostic, let_binding_name_at, let_value,
    make_error_node, match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver,
    module_items, no_span, param_node_name_at, param_node_type_expr, Cardinality,
    CompilerDiagnostic, Connective, ErrorNode, ExprData, ExprErrorKind, InferredNode, InternTable,
    MatchPattern, NewlineIndex, Node,
};
use serde::Serialize;

pub(crate) fn external_authority_uri_record_from_anchor_body(
    body: &Rc<crate::v1_std_core::Node>,
    variant: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<Rc<crate::v1_std_core::Node>> {
    match variant {
        "ExternalAuthority" | "StableAuthority" | "ExternalUri" => {
            extdeps_record_field_value(body, "uri", source_indices)
        }
        _ => None,
    }
}

pub(crate) fn external_authority_scheme_identity_from_value_node(
    node: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> String {
    use crate::v1_std_core::authored_name_at;
    authored_name_at(source_indices.clone(), node.clone())
}

pub(crate) fn external_authority_alias_symbol_name(
    body: &Rc<crate::v1_std_core::Node>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Option<String> {
    use crate::v1_std_core::{authored_name_at, expr_var_name_at, ExprData};
    match body.expr_data.as_ref() {
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(body.clone(), source_indices.clone());
            if !name.is_empty() {
                Some(name)
            } else if !body.name.is_empty() {
                Some(body.name.clone())
            } else {
                None
            }
        }
        _ => {
            let variant = authored_name_at(source_indices.clone(), body.clone());
            if variant.is_empty() {
                None
            } else {
                Some(variant)
            }
        }
    }
}

pub(crate) fn external_authority_anchor_from_data_body(
    body: &Rc<crate::v1_std_core::Node>,
    module: &Rc<crate::v1_std_core::Node>,
    module_path: &str,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    visited: &mut std::collections::HashSet<String>,
) -> ExternalAuthorityAnchorProjection {
    use crate::v1_std_core::authored_name_at;
    let variant = authored_name_at(source_indices.clone(), body.clone());
    if let Some(uri_node) =
        external_authority_uri_record_from_anchor_body(body, variant.as_str(), source_indices)
    {
        let Some(scheme) = extdeps_record_field_value(&uri_node, "scheme", source_indices)
            .map(|n| external_authority_scheme_identity_from_value_node(&n, source_indices))
        else {
            return ExternalAuthorityAnchorProjection::Refused {
                cause: format!("anchor uri record has no `scheme` field in {variant}"),
            };
        };
        let Some(locator) = extdeps_record_field_value(&uri_node, "locator", source_indices)
            .and_then(|n| extdeps_literal_string_value(&n))
        else {
            return ExternalAuthorityAnchorProjection::Refused {
                cause: format!("anchor uri record has no `locator` field in {variant}"),
            };
        };
        if scheme.is_empty() {
            return ExternalAuthorityAnchorProjection::Refused {
                cause: format!("anchor uri record has an empty `scheme` in {variant}"),
            };
        }
        return ExternalAuthorityAnchorProjection::Present {
            scheme_identity: scheme,
            locator,
        };
    }
    let Some(symbol) = external_authority_alias_symbol_name(body, source_indices) else {
        return ExternalAuthorityAnchorProjection::Absent;
    };
    if let Some(home) = extdeps_import_home_for_symbol(module, symbol.as_str(), source_indices) {
        if !visited.insert(home.clone()) {
            return ExternalAuthorityAnchorProjection::Refused {
                cause: format!("alias cycle revisiting {home} for symbol {symbol}"),
            };
        }
        return project_external_authority_named_data(&home, symbol.as_str(), visited);
    }
    // The alias names no IMPORTED symbol, so the only home left for it is this module itself. A
    // `.dag` declaration may reference a sibling declaration in its own file without importing it,
    // and reading that as Absent conflates "this module declares no anchor" with "this module
    // declares one I did not follow" -- two states with opposite remedies, one an authoring defect
    // and one a defect in this reader. Resolving the local home removes the second state rather
    // than reporting it (DESIGN section 5).
    let local_key = format!("{module_path}::{symbol}");
    if !visited.insert(local_key) {
        return ExternalAuthorityAnchorProjection::Refused {
            cause: format!("alias cycle revisiting {module_path} for local symbol {symbol}"),
        };
    }
    project_external_authority_named_data(module_path, symbol.as_str(), visited)
}

pub(crate) fn external_authority_machinery_exempt_module_paths() -> &'static [&'static str] {
    &[
        "extdeps.uri",
        "extdeps.external_authority",
        "extdeps.publication",
    ]
}

pub(crate) fn external_authority_is_machinery_exempt_for_module_path(module_path: &str) -> bool {
    external_authority_machinery_exempt_module_paths().contains(&module_path)
}

pub(crate) fn external_authority_is_clean_tree_roster_excluded_for_module_path(
    module_path: &str,
) -> bool {
    module_path.starts_with("extdeps.fixture.") || module_path.ends_with(".mock_corpus")
}

pub(crate) fn external_authority_live_violation_module_paths() -> Vec<String> {
    let mut violations = Vec::new();
    for path in extdeps_derived_extdeps_module_paths() {
        if external_authority_is_clean_tree_roster_excluded_for_module_path(&path) {
            continue;
        }
        if external_authority_is_machinery_exempt_for_module_path(&path) {
            continue;
        }
        match project_external_authority_anchor(&path) {
            ExternalAuthorityAnchorProjection::Absent => violations.push(format!("missing:{path}")),
            ExternalAuthorityAnchorProjection::Refused { cause } => {
                violations.push(format!("refused:{path}:{cause}"))
            }
            ExternalAuthorityAnchorProjection::Present {
                scheme_identity, ..
            } if scheme_identity != "Http" && scheme_identity != "Https" => {
                violations.push(format!("non_external:{path}:{scheme_identity}"))
            }
            _ => {}
        }
    }
    violations
}
