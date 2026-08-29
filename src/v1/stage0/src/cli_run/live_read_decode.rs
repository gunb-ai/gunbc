// Split from cli_run.rs (pure code motion; no semantic change).
#![allow(unused_imports)]
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

/// Declaration-grain key. Entry path plus function name, which is the identity the floor already
/// enrolls rows under.
pub fn live_read_manifest_key(entry_path: &str, fn_name: &str) -> String {
    format!("{}::{}", workspace_relative_repo_path(entry_path), fn_name)
}

/// Decode one `v2.std.live_read.LiveReadSelectionProjection` value.
///
/// `LiveReadSelectionRefused` decodes to `Refused`, NOT to a decode error: it is a legitimate,
/// modelled state of the authority (the classification could not be established), and the seed's
/// job is to carry it through so the consumer forbids a skip. Collapsing it into an error here
/// would lose the located cause the authority already computed.
///
/// Everything else refuses. An earlier revision of this decoder accepted carrier list items only
/// as `Value::Str` or a record with a `module` field and fell through to an empty carrier vector
/// on anything else — but `LiveReadCarrier` is a coproduct, so every real carrier arrives as
/// `Value::Variant` and took that fallthrough. The result was a `RuntimeRead` whose carriers were
/// silently empty, which reads to a consumer as "runtime read, intersecting nothing" — the
/// under-selection direction, produced by an absorbing arm sitting directly beneath a comment
/// claiming fail-closed behaviour. There is no such arm now: an unrecognised shape at any depth
/// refuses.
pub(crate) fn decode_live_read_selection_row(
    ctx: &v1_interpreter::InterpContext,
    projection: &v1_interpreter::Value,
) -> Result<LiveReadSelectionRow, String> {
    let (variant_name, fields) = variant_parts(projection).ok_or_else(|| {
        format!(
            "LIVE-READ MANIFEST REFUSAL cause=ProjectionShapeUnexpected got={} — expected a \
             LiveReadSelectionProjection variant.",
            projection.type_label_public()
        )
    })?;
    if ctx.sym_eq(variant_name, "LiveReadSelectionRefused") {
        let cause = match ctx.field(fields, "cause") {
            Some(Value::Str(c)) => c.to_string(),
            _ => "typed refusal without a String cause".to_string(),
        };
        return Ok(LiveReadSelectionRow::Refused { cause });
    }
    if !ctx.sym_eq(variant_name, "LiveReadSelectionClassified") {
        return Err(
            "LIVE-READ MANIFEST REFUSAL cause=ProjectionVariantUnknown — the projection carried a \
             variant this seed does not model, so its classification cannot be attributed."
                .to_string(),
        );
    }
    let classification = ctx.field(fields, "classification").ok_or_else(|| {
        "LIVE-READ MANIFEST REFUSAL cause=ClassificationAbsent — a Classified projection with no \
         classification."
            .to_string()
    })?;
    Ok(LiveReadSelectionRow::Classified(
        decode_live_read_classification(ctx, classification)?,
    ))
}

pub(crate) fn decode_live_read_classification(
    ctx: &v1_interpreter::InterpContext,
    value: &v1_interpreter::Value,
) -> Result<LiveReadClassification, String> {
    let (name, fields) = variant_parts(value).ok_or_else(|| {
        format!(
            "LIVE-READ MANIFEST REFUSAL cause=ClassificationShapeUnexpected got={} — expected a \
             LiveReadClassification variant.",
            value.type_label_public()
        )
    })?;
    if ctx.sym_eq(name, "LocalRead") {
        return Ok(LiveReadClassification::LocalRead);
    }
    if !ctx.sym_eq(name, "RuntimeRead") {
        return Err(
            "LIVE-READ MANIFEST REFUSAL cause=ClassificationVariantUnknown — neither LocalRead nor \
             RuntimeRead; this seed cannot decide whether a skip is licensed."
                .to_string(),
        );
    }
    let Some(v1_interpreter::Value::List(items)) = ctx.field(fields, "carriers") else {
        return Err(
            "LIVE-READ MANIFEST REFUSAL cause=CarrierListAbsent — a RuntimeRead whose carrier list \
             is missing or not a list. An empty carrier set intersects no diff, so accepting this \
             shape would license exactly the skip the runtime read forbids."
                .to_string(),
        );
    };
    let carriers = items
        .iter()
        .map(|item| decode_live_read_carrier(ctx, item))
        .collect::<Result<Vec<LiveReadCarrier>, String>>()?;
    Ok(LiveReadClassification::RuntimeRead { carriers })
}

pub(crate) fn decode_live_read_carrier(
    ctx: &v1_interpreter::InterpContext,
    value: &v1_interpreter::Value,
) -> Result<LiveReadCarrier, String> {
    let (name, fields) = variant_parts(value).ok_or_else(|| {
        format!(
            "LIVE-READ MANIFEST REFUSAL cause=CarrierShapeUnexpected got={} — expected a \
             LiveReadCarrier variant.",
            value.type_label_public()
        )
    })?;
    if ctx.sym_eq(name, "ModuleDeclarationFactsScan") {
        return Ok(LiveReadCarrier::ModuleDeclarationFactsScan);
    }
    if ctx.sym_eq(name, "DeclFactsReflection") {
        let home =
            match ctx.field(fields, "home") {
                Some(Value::Str(h)) => h.to_string(),
                _ => return Err(
                    "LIVE-READ MANIFEST REFUSAL cause=CarrierHomeUnexpected — DeclFactsReflection \
                     without a String home."
                        .to_string(),
                ),
            };
        return Ok(LiveReadCarrier::DeclFactsReflection { home });
    }
    if !ctx.sym_eq(name, "FilesystemReadPath") {
        return Err(
            "LIVE-READ MANIFEST REFUSAL cause=CarrierVariantUnknown — a carrier variant this seed \
             does not model. Its read target is therefore unknown, and an unknown target cannot be \
             intersected with a diff."
                .to_string(),
        );
    }
    let pattern = ctx.field(fields, "path_pattern").ok_or_else(|| {
        "LIVE-READ MANIFEST REFUSAL cause=PathPatternAbsent — FilesystemReadPath without a \
         path_pattern."
            .to_string()
    })?;
    Ok(LiveReadCarrier::FilesystemReadPath(
        decode_live_read_path_pattern(ctx, pattern)?,
    ))
}

pub(crate) fn decode_live_read_path_pattern(
    ctx: &v1_interpreter::InterpContext,
    value: &v1_interpreter::Value,
) -> Result<LiveReadPathPattern, String> {
    let (name, fields) = variant_parts(value).ok_or_else(|| {
        format!(
            "LIVE-READ MANIFEST REFUSAL cause=PathPatternShapeUnexpected got={} — expected a \
             PathPattern variant.",
            value.type_label_public()
        )
    })?;
    if ctx.sym_eq(name, "UnknownPath") {
        return Ok(LiveReadPathPattern::UnknownPath);
    }
    if ctx.sym_eq(name, "LiteralPath") {
        return match ctx.field(fields, "path") {
            Some(Value::Str(p)) => Ok(LiveReadPathPattern::LiteralPath(p.to_string())),
            _ => Err(
                "LIVE-READ MANIFEST REFUSAL cause=LiteralPathUnexpected — LiteralPath without a \
                 String path. Treating it as unknown would silently widen; refusing keeps the \
                 defect countable."
                    .to_string(),
            ),
        };
    }
    if ctx.sym_eq(name, "ParamRef") {
        return match ctx.field(fields, "name") {
            Some(Value::Str(n)) => Ok(LiveReadPathPattern::ParamRef(n.to_string())),
            _ => Err(
                "LIVE-READ MANIFEST REFUSAL cause=ParamRefUnexpected — ParamRef without a String \
                 name."
                    .to_string(),
            ),
        };
    }
    Err(
        "LIVE-READ MANIFEST REFUSAL cause=PathPatternVariantUnknown — a path pattern this seed does \
         not model."
            .to_string(),
    )
}
