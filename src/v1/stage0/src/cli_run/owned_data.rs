// Split from cli_run.rs (pure code motion; no semantic change).
// CLIPPY ROSTER -- 1 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::collapsible_if,  // 1
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

pub(crate) fn owned_data_initializer_from_body(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_path: &str,
    decl_name: &str,
    body: &Rc<Node>,
    type_annotation: Option<&Rc<Node>>,
) -> Result<OwnedDataDeclInitializer, String> {
    let resolved_initializer =
        resolved_initializer_decl_ref(graph, source_indices, body, type_annotation)
            .map_err(|e| format!("{entry_path}: owned data '{decl_name}': {e}"))?;
    if is_resolved_bool_witness_claim(&resolved_initializer) {
        let (witness_entry, witness_function) =
            extract_bool_witness_transport(body, source_indices);
        if witness_entry.is_empty() || witness_function.is_empty() {
            return Err(format!(
                "{}: owned data '{}' has malformed BoolWitnessClaim witness (missing entry and/or function)",
                entry_path, decl_name
            ));
        }
        return Ok(OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        });
    }
    if is_resolved_node_corpus(&resolved_initializer) {
        return Ok(OwnedDataDeclInitializer::NodeCorpus);
    }
    Ok(OwnedDataDeclInitializer::Other {
        resolved: resolved_initializer,
    })
}

pub fn owned_data_decls_for_entry(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
    entry_path: &str,
    entry_module: &str,
) -> Result<Vec<OwnedDataDeclRecord>, String> {
    let si = Rc::new(source_indices.clone());
    let typed_module = entry_typed_module(graph, source_indices, entry_module)
        .map_err(|e| format!("{entry_path}: {e}"))?;

    let mut records = Vec::new();
    for item in typed_module.items.iter() {
        if item.body.is_none() || item.type_annotation.is_none() {
            continue;
        }
        let decl_name = authored_name_at(si.clone(), item.clone());
        if decl_name.is_empty() {
            return Err(format!(
                "{entry_path}: owned data item in module '{}' missing authored name",
                entry_module
            ));
        }
        let info = typed_module.item_registry.get(&decl_name).ok_or_else(|| {
            format!(
                "{entry_path}: owned data '{}' missing from item_registry",
                decl_name
            )
        })?;
        if info.kind != ItemKind::DataItem {
            continue;
        }
        if info.module_name != entry_module {
            return Err(format!(
                "{entry_path}: item_registry module mismatch for '{}' (expected {}, got {})",
                decl_name, entry_module, info.module_name
            ));
        }
        if info.name != decl_name {
            return Err(format!(
                "{entry_path}: item_registry name mismatch for '{}' (registry name '{}')",
                decl_name, info.name
            ));
        }
        if !decl_name.starts_with("unified_claim_") {
            continue;
        }
        let body = item.body.as_ref().ok_or_else(|| {
            format!(
                "{entry_path}: owned data '{}' missing initializer body",
                decl_name
            )
        })?;
        let initializer = owned_data_initializer_from_body(
            graph,
            source_indices,
            entry_path,
            &decl_name,
            body,
            item.type_annotation.as_ref(),
        )?;
        records.push(OwnedDataDeclRecord {
            entry: entry_path.to_string(),
            module: entry_module.to_string(),
            decl_name,
            initializer,
        });
    }

    let discovered: HashSet<&str> = records.iter().map(|r| r.decl_name.as_str()).collect();
    for (decl_name, info) in typed_module.item_registry.iter() {
        if info.kind == ItemKind::DataItem
            && info.module_name == entry_module
            && decl_name.starts_with("unified_claim_")
        {
            if !discovered.contains(decl_name.as_str()) {
                return Err(format!(
                    "{entry_path}: item_registry data '{}' not found in entry module items",
                    decl_name
                ));
            }
        }
    }

    records.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.decl_name.cmp(&b.decl_name))
    });
    Ok(records)
}

pub fn owned_data_bool_witness_transport_tsv(
    records: &[OwnedDataDeclRecord],
) -> Result<String, String> {
    let mut rows = Vec::new();
    for rec in records {
        let OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } = &rec.initializer
        else {
            continue;
        };
        if witness_entry.is_empty() || witness_function.is_empty() {
            return Err(format!(
                "{}: owned data '{}' has malformed BoolWitnessClaim witness transport (missing entry and/or function)",
                rec.entry, rec.decl_name
            ));
        }
        let label = rec
            .decl_name
            .strip_prefix("unified_claim_")
            .unwrap_or(rec.decl_name.as_str());
        rows.push(format!("{label}\t{witness_entry}\t{witness_function}"));
    }
    rows.sort();
    let mut out = rows.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    Ok(out)
}

pub(crate) fn owned_data_decl_record_to_value(
    rec: &OwnedDataDeclRecord,
    ctx: &v1_interpreter::InterpContext,
) -> v1_interpreter::Value {
    use std::rc::Rc;
    use v1_interpreter::{sorted_fields, Value};
    let initializer = match &rec.initializer {
        OwnedDataDeclInitializer::BoolWitnessClaim {
            witness_entry,
            witness_function,
        } => Value::Variant {
            type_name: ctx.sym("OwnedDataDeclInitializer"),
            variant_name: ctx.sym("OwnedBoolWitnessClaimInit"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("witness_entry"), str_value(witness_entry.clone())),
                (
                    ctx.sym("witness_function"),
                    str_value(witness_function.clone()),
                ),
            ])),
        },
        OwnedDataDeclInitializer::NodeCorpus => Value::Variant {
            type_name: ctx.sym("OwnedDataDeclInitializer"),
            variant_name: ctx.sym("OwnedNodeCorpusInit"),
            fields: Rc::new(vec![]),
        },
        OwnedDataDeclInitializer::Other { resolved } => Value::Variant {
            type_name: ctx.sym("OwnedDataDeclInitializer"),
            variant_name: ctx.sym("OwnedOtherInit"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("resolved"),
                Value::Record {
                    type_name: ctx.sym("ResolvedDeclRef"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("module"), str_value(resolved.module.clone())),
                        (ctx.sym("name"), str_value(resolved.name.clone())),
                    ])),
                },
            )])),
        },
    };
    Value::Record {
        type_name: ctx.sym("OwnedDataDeclRecord"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("entry"), str_value(rec.entry.clone())),
            (ctx.sym("module"), str_value(rec.module.clone())),
            (ctx.sym("decl_name"), str_value(rec.decl_name.clone())),
            (ctx.sym("initializer"), initializer),
        ])),
    }
}
