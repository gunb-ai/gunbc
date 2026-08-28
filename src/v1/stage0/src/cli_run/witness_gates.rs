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

pub(crate) fn witness_layer_root_spelling(root: &str) -> String {
    let p = Path::new(root);
    if p.is_absolute() {
        repo_relative_path_normalized(p)
    } else {
        root.trim_start_matches("./")
            .trim_end_matches('/')
            .to_string()
    }
}

/// Project the `witness_layer_roots` `List<String>` literal out of the ci_layer_roots authority.
pub(crate) fn witness_layer_roots_from_source(content: &str) -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(content, WITNESS_LAYER_ROOTS_DATA_NAME)
}

/// Project the typed `witness_exclusion_frontier` rows out of a `.dag` module's SOURCE TEXT
/// via the real front-end (`tokenize` + `parse`) — the record-literal sibling of
/// `string_list_data_from_module_source`. Fail-closed: a parse error, a missing data def,
/// a non-record element, a missing/non-literal `pattern` field, or an unrecognized
/// classification name is a loud panic, never a silent fallback.
// 🟡 dissolve-on: hand-Rust reader over the `.dag` authority — dissolves when the host
// consumes an emitted manifest of the frontier rows (rows supplied at the boundary, no
// host-side source parse), together with WITNESS_EXCLUSION_CLASSIFICATIONS above; the
// scaffold classification is carried at the model authority in
// `gunbc.roster_registry.roster_registry_visibility_note`.
pub(crate) fn witness_exclusion_rows_from_module_source(
    module_rel_path: &str,
    content: &str,
) -> Vec<WitnessExclusionRowRaw> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let filename = module_rel_path.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), filename.clone());
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let source_indices = std::rc::Rc::new(source_indices);
    let result = crate::v1_compiler_parse::parse(tokens, source_indices.clone());
    if let Some(err) = result.error.as_ref() {
        panic!(
            "exclusion frontier reader: parse error in {module_rel_path}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result.module.as_ref().unwrap_or_else(|| {
        panic!("exclusion frontier reader: {module_rel_path} parsed to no module")
    });
    let data_name = WITNESS_EXCLUSION_FRONTIER_DATA_NAME;
    for item in module.children.iter() {
        if item.name != data_name
            || !crate::v1_compiler_emit_core_support::is_data_def_item(item.clone())
        {
            continue;
        }
        let body = item.body.as_ref().unwrap_or_else(|| {
            panic!("exclusion frontier reader: `data {data_name}` in {module_rel_path} has no value body")
        });
        if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
            panic!(
                "exclusion frontier reader: `data {data_name}` in {module_rel_path} is not a \
                 list literal"
            );
        }
        let mut rows = Vec::new();
        for el in body.children.iter() {
            if !matches!(el.expr_data.as_ref(), ExprData::ExprRecordLit { .. }) {
                panic!(
                    "exclusion frontier reader: an element of `{data_name}` in \
                     {module_rel_path} is not a record literal (refusing — rows must stay \
                     directly host-readable)"
                );
            }
            let mut pattern: Option<String> = None;
            let mut classification: Option<String> = None;
            for field in el.children.iter() {
                let fname = crate::v1_std_core::field_init_node_name_at(
                    field.clone(),
                    source_indices.clone(),
                );
                let value = crate::v1_std_core::field_init_node_value(field.clone());
                match fname.as_str() {
                    "pattern" => match value.expr_data.as_ref() {
                        ExprData::ExprLiteral { value: lit } => match lit.as_ref() {
                            LiteralValue::LitStr { value: s } => pattern = Some(s.clone()),
                            _ => panic!(
                                "exclusion frontier reader: `pattern` in a `{data_name}` row \
                                 of {module_rel_path} is not a string literal"
                            ),
                        },
                        _ => panic!(
                            "exclusion frontier reader: `pattern` in a `{data_name}` row of \
                             {module_rel_path} is not a literal"
                        ),
                    },
                    "classification" => {
                        let name = crate::v1_std_core::field_init_node_name_at(
                            value.clone(),
                            source_indices.clone(),
                        );
                        if !WITNESS_EXCLUSION_CLASSIFICATIONS.contains(&name.as_str()) {
                            panic!(
                                "exclusion frontier reader: unrecognized classification \
                                 {name:?} in a `{data_name}` row of {module_rel_path} \
                                 (recognized: {WITNESS_EXCLUSION_CLASSIFICATIONS:?})"
                            );
                        }
                        classification = Some(name);
                    }
                    _ => {}
                }
            }
            let pattern = pattern.unwrap_or_else(|| {
                panic!(
                    "exclusion frontier reader: a `{data_name}` row in {module_rel_path} has \
                     no `pattern` field"
                )
            });
            let classification = classification.unwrap_or_else(|| {
                panic!(
                    "exclusion frontier reader: row {pattern:?} in {module_rel_path} has no \
                     `classification` field"
                )
            });
            rows.push(WitnessExclusionRowRaw {
                pattern,
                classification,
            });
        }
        if rows.is_empty() {
            panic!(
                "exclusion frontier reader: `{data_name}` in {module_rel_path} is empty \
                 (fail-closed)"
            );
        }
        return rows;
    }
    panic!("exclusion frontier reader: no `data {data_name}` def in {module_rel_path}")
}

pub(crate) fn witness_exclusion_frontier_rows() -> &'static [WitnessExclusionRowRaw] {
    static ROWS: OnceLock<Vec<WitnessExclusionRowRaw>> = OnceLock::new();
    ROWS.get_or_init(|| {
        witness_exclusion_rows_from_module_source(
            CI_LAYER_ROOTS_AUTHORITY_REL,
            ci_layer_roots_authority_content(),
        )
    })
}

/// Project the discovery-exclusion patterns out of the typed frontier rows.
pub(crate) fn witness_exclusion_substrings_from_source(content: &str) -> Vec<String> {
    witness_exclusion_rows_from_module_source(CI_LAYER_ROOTS_AUTHORITY_REL, content)
        .into_iter()
        .map(|r| r.pattern)
        .collect()
}

/// The witness layer roots, read live from the single .dag authority and memoized.
///
/// VISIBILITY WIDENED `pub(crate)` -> `pub` by the cli-run cut, and the reason is a §3 one
/// rather than a convenience. The driver seam in `main.rs` is a separate crate, so it could
/// not see this; the alternative was to spell `["dag", "src/v2"]` as a literal beside the
/// `ci` verb, which forks a live authority that is READ FROM `.dag` at runtime into a
/// hand-written constant that cannot track it. Exposing the existing single authority is
/// not new investment in the frozen engine — nothing here is re-homed, reshaped, or
/// reimplemented, and the body is untouched.
pub fn witness_layer_roots() -> Vec<String> {
    static ROOTS: OnceLock<Vec<String>> = OnceLock::new();
    ROOTS
        .get_or_init(|| witness_layer_roots_from_source(ci_layer_roots_authority_content()))
        .clone()
}

/// Floor discovery path exclusions — single authority `gunbc.ci_layer_roots.witness_exclusion_substrings`.
pub fn witness_exclusion_substrings() -> Vec<String> {
    static EXCLUDES: OnceLock<Vec<String>> = OnceLock::new();
    EXCLUDES
        .get_or_init(
            || witness_exclusion_substrings_from_source(ci_layer_roots_authority_content()),
        )
        .clone()
}

pub(crate) fn witness_layer_roots_compile_clean_sources_for_plan(
    plan: &CompileCleanScopePlan,
) -> Result<Option<Vec<Rc<v1_compiler_compile::SourceFile>>>, String> {
    match plan {
        CompileCleanScopePlan::Refused { reason } => {
            eprintln!("compile-clean scope: refused ({reason})");
            Err(reason.clone())
        }
        CompileCleanScopePlan::SkipNoAffected { reason } => {
            eprintln!("compile-clean scope: skipped ({reason})");
            Ok(None)
        }
        CompileCleanScopePlan::WholeTree => {
            eprintln!("compile-clean scope: whole-tree entry closure (witness_layer_roots)");
            let roots = witness_layer_roots();
            let mei = build_multi_entry_index_primary_precedence(&roots);
            load_compile_clean_entry_sources(&roots, &mei, None).map(|mut sources| {
                append_test_floor_compile_clean_inject(&mut sources);
                Some(sources)
            })
        }
        CompileCleanScopePlan::Scoped { entry_paths } => {
            eprintln!(
                "compile-clean scope: {} affected entr{} (of whole-tree gate)",
                entry_paths.len(),
                if entry_paths.len() == 1 { "y" } else { "ies" }
            );
            let filter: std::collections::HashSet<String> = entry_paths
                .iter()
                .map(|p| workspace_relative_repo_path(p))
                .collect();
            let roots = witness_layer_roots();
            let mei = build_multi_entry_index_primary_precedence(&roots);
            load_compile_clean_entry_sources(&roots, &mei, Some(&filter)).map(Some)
        }
    }
}

/// Resolve/typecheck leg of compile-clean over `witness_layer_roots` (`dag` + `src/v2` only).
/// Uses `primary-precedence` pool indexing like shell compile, but a narrower root set than
/// `compile_clean_source_roots()` (which adds `src/v1` for cross-tree perturb receipts).
/// In CI (`GITHUB_ACTIONS=true`) or when `GUNBC_CI_DIFF_BASE` is set, scopes to affected
/// shard entries from `tools.dag_compile_clean_scope` (lever a) using `gunbc_ci_spec.diff_policy`
/// defaults via `floor_diff_observe`; diff/disposition failure refuses (never widens).
/// Skip/whole-tree/skip-vs-run authority lives in `tools.dag_compile_clean_scope` (including
/// `RequireWholeTree` for non-docs infra/Rust touches with no shard intersection).
pub fn witness_layer_roots_compile_clean_check() -> bool {
    match witness_layer_roots_compile_clean_sources_for_plan(&compile_clean_scope_plan_for_ci()) {
        Ok(None) => true,
        Ok(Some(sources)) => {
            let roots = witness_layer_roots();
            let index = build_multi_entry_index_primary_precedence(&roots);
            let options = compile_clean_pipeline_options_for_sources(Some(&index), &sources);
            let result = v1_compiler_compile::compile_to_resolved_with_options(
                Rc::new(sources.into()),
                options,
            );
            if compile_clean_resolve_has_hard_errors(&result) {
                eprint_compile_clean_hard_diagnostics(result.diagnostics.as_ref());
                false
            } else {
                true
            }
        }
        Err(msg) => {
            eprintln!("compile-clean: source load failed ({msg})");
            false
        }
    }
}

/// Emit leg: `--target dag` compile over witness layer roots without shell or disk write.
/// Direct-run oracle for non-floor contexts (cargo tests, enrolled witnesses). The CI
/// floor gate consumes `install_or_consume_floor_compile_clean_gate_receipt` instead (Lever A).
pub fn witness_layer_roots_compile_clean_emit_check() -> bool {
    match witness_layer_roots_compile_clean_sources_for_plan(&compile_clean_scope_plan_for_ci()) {
        Ok(None) => true,
        Ok(Some(sources)) => {
            let roots = witness_layer_roots();
            let index = build_multi_entry_index_primary_precedence(&roots);
            floor_compile_clean_emit_ok(sources, Some(&index))
        }
        Err(msg) => {
            eprintln!("compile-clean emit: source load failed ({msg})");
            false
        }
    }
}

/// The floor receipt's leg label, read from the memo primed by
/// `prime_witness_execution_legs`. The label is a pure function of the entry path, derived
/// from the `.dag` classification authority — there is no census carrier to read and none
/// to keep in sync (§2/§3).
///
/// Fail-closed (§5): an unprimed entry refuses. It does not quietly derive on the spot,
/// because doing so off the floor's own index costs a second whole-corpus index — measured
/// at ~6GB of extra demand, which pushed the floor into swap and inflated batch 3 by 44%.
/// A miss is a wiring bug in the caller, and it says so rather than paying that silently.
pub fn witness_execution_leg_label(entry: &str) -> String {
    let rel = repo_relative_dag_path(entry);
    match witness_execution_leg_cached(&rel) {
        Some(hit) => hit,
        None => panic!(
            "witness execution leg: entry {rel:?} was not primed (refuse — call \
             prime_witness_execution_legs with the floor's index before running rows)"
        ),
    }
}

pub(crate) fn witness_execution_leg_cache() -> &'static std::sync::RwLock<HashMap<String, String>> {
    WITNESS_EXECUTION_LEG_CACHE.get_or_init(|| std::sync::RwLock::new(HashMap::new()))
}

pub(crate) fn witness_execution_leg_cached(rel: &str) -> Option<String> {
    match witness_execution_leg_cache().read() {
        Ok(map) => map.get(rel).cloned(),
        Err(e) => panic!("witness execution leg cache poisoned: {e} (refuse)"),
    }
}

pub(crate) fn witness_execution_leg_cache_put(rel: &str, leg: &str) {
    match witness_execution_leg_cache().write() {
        Ok(mut map) => {
            map.insert(rel.to_string(), leg.to_string());
        }
        Err(e) => panic!("witness execution leg cache poisoned: {e} (refuse)"),
    }
}

pub(crate) fn witness_cost_nanosecond_value(
    ctx: &v1_interpreter::InterpContext,
    nanos: u128,
) -> Result<v1_interpreter::Value, String> {
    let count = i64::try_from(nanos).map_err(|_| {
        format!("[witness-row-cost] REFUSED: measured wall {nanos}ns exceeds the seed Int boundary")
    })?;
    v1_interpreter::run_in_context_with_args(
        ctx,
        "nanosecond",
        &[(Some("count".to_string()), v1_interpreter::Value::Int(count))],
        false,
    )
    .map_err(|e| format!("[witness-row-cost] REFUSED: nanosecond constructor failed: {e}"))
}

/// Hand the occurrence's two clocks over as ONE basis-tagged list, built by the authored
/// constructor rather than assembled here.
///
/// The seed constructs no carrier: it calls `observation_durations_cpu_and_wall` across the
/// interpreter boundary and passes the returned value straight through, exactly as it already
/// does for `nanosecond` and `millisecond`. A `Value::List` of hand-built records would fork
/// `std.observation`'s shape into Rust, which is the thing every note on this seam forbids.
///
/// Both clocks come from ONE run — `run_claim_measured` reads `thread_cpu_nanos` and
/// `Instant::elapsed` around the same evaluation — so a row can never carry a CPU figure from
/// one occurrence beside a wall figure from another. Order is fixed by the constructor's
/// parameter names, not by position here, so a swap is a compile error rather than a silent
/// exchange of the two magnitudes.
pub(crate) fn witness_cost_eval_durations(
    ctx: &v1_interpreter::InterpContext,
    cpu_nanos: u128,
    wall_nanos: u128,
) -> Result<v1_interpreter::Value, String> {
    let cpu = witness_cost_nanosecond_value(ctx, cpu_nanos)?;
    let wall = witness_cost_nanosecond_value(ctx, wall_nanos)?;
    v1_interpreter::run_in_context_with_args(
        ctx,
        "observation_durations_cpu_and_wall",
        &[
            (Some("cpu".to_string()), cpu),
            (Some("wall".to_string()), wall),
        ],
        false,
    )
    .map_err(|e| format!("[witness-row-cost] REFUSED: eval duration constructor failed: {e}"))
}

pub(crate) fn witness_cost_string_field(
    ctx: &v1_interpreter::InterpContext,
    fields: &[(v1_interpreter::Symbol, v1_interpreter::Value)],
    name: &str,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(value)) if !value.is_empty() => Ok(value.to_string()),
        Some(other) => Err(format!(
            "[witness-row-cost] REFUSED: projected {name} is {}, expected NonEmptyStr",
            other.type_label_public()
        )),
        None => Err(format!(
            "[witness-row-cost] REFUSED: projected receipt row lacks {name}"
        )),
    }
}

/// Read ONE clock out of a row's tagged eval durations, through the authored projection.
///
/// The row's `eval` term is a `List<TimedMeasurement>` — every clock the occurrence was
/// measured on, each tagged — so the seed cannot reach for "the duration" any more, and that
/// is the point: which clock it wants is now a thing it has to say. It says it by calling
/// `std.observation.observation_duration_of`, the same projection every `.dag` consumer uses,
/// rather than walking the list here. A local walk would be a second copy of the lookup,
/// including its ambiguity refusal, and the two would drift.
///
/// Returns `None` for a clock the producer did not sample, distinguished from a clock it
/// sampled as zero. `MeasuredUnavailable` carries its cause and this returns `None` rather
/// than `0`, which is the same `observation_measured_note` discipline the TSV's `unmeasured`
/// cell keeps: a number that does not exist must not render as a small one.
pub(crate) fn witness_cost_clock_nanos(
    ctx: &v1_interpreter::InterpContext,
    durations: &v1_interpreter::Value,
    basis_constructor: &str,
) -> Result<Option<u128>, String> {
    use v1_interpreter::Value;
    let basis = v1_interpreter::run_in_context_with_args(ctx, basis_constructor, &[], false)
        .map_err(|e| format!("[witness-row-cost] REFUSED: {basis_constructor} failed: {e}"))?;
    let measured = v1_interpreter::run_in_context_with_args(
        ctx,
        "observation_duration_of",
        &[
            (Some("durations".to_string()), durations.clone()),
            (Some("basis".to_string()), basis),
        ],
        false,
    )
    .map_err(|e| {
        format!(
            "[witness-row-cost] REFUSED: observation_duration_of({basis_constructor}) failed: {e}"
        )
    })?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = measured
    else {
        return Err(
            "[witness-row-cost] REFUSED: observation_duration_of returned a non-Measured value"
                .to_string(),
        );
    };
    if ctx.sym_eq(variant_name, "MeasuredUnavailable") {
        return Ok(None);
    }
    if !ctx.sym_eq(variant_name, "MeasuredValue") {
        return Err(format!(
            "[witness-row-cost] REFUSED: observation_duration_of returned unknown variant {}",
            ctx.resolve(variant_name)
        ));
    }
    let value = ctx.field(&fields, "value").cloned().ok_or_else(|| {
        "[witness-row-cost] REFUSED: MeasuredValue lacks its Nanosecond".to_string()
    })?;
    witness_cost_nanosecond_count(ctx, value, basis_constructor).map(Some)
}

pub(crate) fn witness_cost_nanosecond_count(
    ctx: &v1_interpreter::InterpContext,
    value: v1_interpreter::Value,
    field: &str,
) -> Result<u128, String> {
    match v1_interpreter::run_in_context_with_args(
        ctx,
        "nanosecond_count",
        &[(Some("n".to_string()), value)],
        false,
    ) {
        Ok(v1_interpreter::Value::Int(n)) if n >= 0 => Ok(n as u128),
        Ok(other) => Err(format!(
            "[witness-row-cost] REFUSED: nanosecond_count({field}) returned {}, expected Nat",
            other.type_label_public()
        )),
        Err(e) => Err(format!(
            "[witness-row-cost] REFUSED: nanosecond_count({field}) failed: {e}"
        )),
    }
}

pub(crate) fn witness_exclusion_patterns_with_classification(classification: &str) -> Vec<String> {
    witness_exclusion_frontier_rows()
        .iter()
        .filter(|r| r.classification == classification)
        .map(|r| r.pattern.clone())
        .collect()
}

pub(crate) fn witness_admission_offline_exclusion_substrings() -> Vec<String> {
    witness_exclusion_patterns_with_classification("OfflineLocalRecipe")
}

pub(crate) fn witness_admission_fixture_exclusion_substrings() -> Vec<String> {
    witness_exclusion_patterns_with_classification("FixtureExplicitRoster")
}

pub(crate) fn witness_admission_manifest_key(entry: &str, function: &str) -> String {
    format!("{entry}::{function}")
}

/// INTERIM hand-Rust scaffold (`CLI_RUN_WITNESS_ADMISSION_SOURCE_SCAN_SCAFFOLD_MARKER` / §7).
///
/// Fail-closed in BOTH directions (§5): every occurrence of a recognized row head either
/// parses to a key, is a verified definition or non-literal pass-through site, or PANICS with
/// its location — a mis-parse stops the line and never silently excuses an orphan; and a
/// consumer expressed in an unrecognized form yields NO key, so its deferred row surfaces as
/// a loud orphan rather than being absorbed.
pub(crate) fn witness_admission_entry_function_keys_from_source(
    source_label: &str,
    content: &str,
) -> Vec<String> {
    const WINDOW: usize = 400;
    let mut keys: Vec<String> = Vec::new();
    fn push_pair(keys: &mut Vec<String>, entry: &str, function: &str) {
        // U3 file-grain: empty `f: ""` means every test decl in the entry. Admission
        // keys must expand the same way `expand_explicit_entries` does for execution —
        // otherwise deferred leaf rows (entry::leaf) never match the unexpanded
        // consumer key (entry::) and Phase 0(b) falsely refuses (roadmap_belt #7273).
        if test_module_hygiene_bridge::is_file_grain_function(function) {
            let path = workspace_root().join(entry);
            let names = test_module_hygiene_bridge::enumerate_entry_test_fns(
                path.to_str().unwrap_or(entry),
            )
            .unwrap_or_else(|e| {
                panic!(
                    "witness admission: file-grain `{entry}` failed to enumerate test decls: {e}"
                )
            });
            for name in names {
                let key = witness_admission_manifest_key(entry, &name);
                if !keys.iter().any(|k| k == &key) {
                    keys.push(key);
                }
            }
            return;
        }
        let key = witness_admission_manifest_key(entry, function);
        if !keys.iter().any(|k| k == &key) {
            keys.push(key);
        }
    }
    // `known_red_probe(` replaced `probe_red(` and the seed-emitter wet row constructor when the
    // quarantine moved to one function-grain authority (2026-08-03): both cadences now author the
    // same row shape in `gunbc.explicit_witness_admission`, so one head reads both.
    let heads: [(&str, &str); 7] = [
        ("bin_wet(", "entry: String"),
        ("known_red_probe(", "entry: NonEmptyStr"),
        (
            "source_root_ingest_gate_admitted_witness(",
            "entry: NonEmptyStr",
        ),
        ("self_host_wet_entry(", "entry: String"),
        ("SelfHostWetReceiptBinding {", ""),
        ("RehomedBinWetRow {", ""),
        ("SubstrateLongLaneRow {", ""),
    ];
    for (head, def_sig) in heads {
        let mut search_from = 0;
        while let Some(rel) = content[search_from..].find(head) {
            let occ = search_from + rel;
            search_from = occ + head.len();
            // Word-boundary guard: `bin_wet(` must not match inside a longer identifier
            // (`witness_exclusion_row_is_rehomed_bin_wet(row: …)` — the class that panicked
            // run 30033250697). A head preceded by an identifier char is a different name.
            if occ > 0 {
                let prev = content.as_bytes()[occ - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' {
                    continue;
                }
            }
            let after = &content[search_from..];
            // WINDOW is a BYTE budget, but `after` is UTF-8: clamping with `min` alone
            // slices mid-character whenever byte `WINDOW` lands inside a multi-byte char,
            // and the admission rows this reads are prose that routinely contains one
            // (em dash, arrow). That panicked the whole floor worker — exit 101, no
            // terminal receipt — on a row whose reason string simply happened to put an
            // em dash across byte 400. Every other slice in this function takes its
            // offset from `find`/`split_once`, which return char boundaries; this fixed
            // offset was the only unguarded one. Walk down to the nearest boundary so a
            // long row is truncated rather than fatal.
            let window_end = {
                let mut end = after.len().min(WINDOW);
                while end > 0 && !after.is_char_boundary(end) {
                    end -= 1;
                }
                end
            };
            let window = &after[..window_end];
            let trimmed = window.trim_start();
            if !def_sig.is_empty() && trimmed.starts_with(def_sig) {
                continue;
            }
            if def_sig.is_empty() && content[..occ].ends_with("type ") {
                continue;
            }
            let Some(entry_rel) = window.find("entry: \"") else {
                if window.contains("entry: ") {
                    continue;
                }
                panic!(
                    "witness admission: {source_label}: `{head}` at byte {occ} has no \
                     recognizable `entry:` argument in range — refusing; a mis-parse must \
                     stop the line, never excuse an orphan"
                );
            };
            let entry_start = entry_rel + "entry: \"".len();
            let Some((entry, after_entry)) = window[entry_start..].split_once('"') else {
                panic!(
                    "witness admission: {source_label}: unterminated entry literal after \
                     `{head}` at byte {occ} — refusing"
                );
            };
            let marker = after_entry.find("f: \"").map(|p| (p, "f: \"")).or_else(|| {
                after_entry
                    .find("function: \"")
                    .map(|p| (p, "function: \""))
            });
            let Some((fn_pos, fn_marker)) = marker else {
                panic!(
                    "witness admission: {source_label}: `{head}` row for entry {entry:?} at \
                     byte {occ} has no `f:`/`function:` literal in range — refusing"
                );
            };
            let fn_start = fn_pos + fn_marker.len();
            let Some((function, _)) = after_entry[fn_start..].split_once('"') else {
                panic!(
                    "witness admission: {source_label}: unterminated function literal after \
                     `{head}` row for entry {entry:?} at byte {occ} — refusing"
                );
            };
            push_pair(&mut keys, entry, function);
        }
    }
    keys.sort();
    keys
}

pub fn witness_admission_explicit_consumer_keys() -> Vec<String> {
    static KEYS: OnceLock<Vec<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut keys = witness_admission_entry_function_keys_from_source(
            "dag/gunbc/ci/ci_layer_roots.dag",
            ci_layer_roots_authority_content(),
        );
        let wet =
            std::fs::read_to_string(workspace_root().join(WET_RECEIPT_ENROLLMENT_AUTHORITY_REL))
                .unwrap_or_else(|e| {
                    panic!(
                        "witness admission: failed to read {}: {e}",
                        WET_RECEIPT_ENROLLMENT_AUTHORITY_REL
                    )
                });
        for key in witness_admission_entry_function_keys_from_source(
            WET_RECEIPT_ENROLLMENT_AUTHORITY_REL,
            &wet,
        ) {
            if !keys.iter().any(|k| k == &key) {
                keys.push(key);
            }
        }
        for key in explicit_witness_admission_keys().iter().cloned() {
            if !keys.iter().any(|k| k == &key) {
                keys.push(key);
            }
        }
        for key in commit_roster_witness_claim_keys().iter().cloned() {
            if !keys.iter().any(|k| k == &key) {
                keys.push(key);
            }
        }
        keys.sort();
        keys
    })
    .clone()
}

pub(crate) fn commit_witness_field_str(
    ctx: &v1_interpreter::InterpContext,
    fields: &[(v1_interpreter::Symbol, v1_interpreter::Value)],
    name: &str,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        other => Err(format!("{name} not a String: {other:?}")),
    }
}

pub fn commit_witness_claim_pair_resolvable(entry: &str, function: &str) -> bool {
    let roots = witness_layer_roots();
    let index = process_shared_index(&roots);
    let mut entry_cache = std::collections::HashMap::new();
    matches!(
        commit_witness_claim_pair_resolvability_with_index(
            &index,
            &roots,
            entry,
            function,
            &mut entry_cache,
        ),
        CommitWitnessClaimPairResolvability::Resolvable
    )
}

pub(crate) fn commit_witness_claim_pair_resolvability_with_index(
    index: &MultiEntryIndex,
    roots: &[String],
    entry: &str,
    function: &str,
    entry_cache: &mut std::collections::HashMap<String, RosterEntryRegistryCache>,
) -> CommitWitnessClaimPairResolvability {
    let cached = entry_cache
        .entry(entry.to_string())
        .or_insert_with(|| roster_entry_registry_cache(index, roots, entry));
    match cached {
        RosterEntryRegistryCache::EntryMissing { detail } => {
            CommitWitnessClaimPairResolvability::EntryMissing {
                detail: detail.clone(),
            }
        }
        RosterEntryRegistryCache::EntryResolveFailed { detail } => {
            CommitWitnessClaimPairResolvability::EntryResolveFailed {
                detail: detail.clone(),
            }
        }
        RosterEntryRegistryCache::Functions(names) => {
            if names.contains(function) {
                CommitWitnessClaimPairResolvability::Resolvable
            } else {
                CommitWitnessClaimPairResolvability::FunctionNotFound
            }
        }
    }
}

pub fn commit_witness_claim_roster_defects() -> Vec<(String, String, String)> {
    let Ok(pairs) = ci_floor_commit_witness_claim_pairs() else {
        return vec![(
            "dag/gunbc/commit_workflow.dag".to_string(),
            CI_FLOOR_COMMIT_WITNESS_SCHEDULE_FN.to_string(),
            "schedule_projection_failed".to_string(),
        )];
    };
    let roots = witness_layer_roots();
    let index = process_shared_index(&roots);
    let mut entry_cache = std::collections::HashMap::new();
    let mut defects = Vec::new();
    for (entry, function) in pairs {
        let cause = match commit_witness_claim_pair_resolvability_with_index(
            &index,
            &roots,
            &entry,
            &function,
            &mut entry_cache,
        ) {
            CommitWitnessClaimPairResolvability::Resolvable => continue,
            CommitWitnessClaimPairResolvability::EntryMissing { detail } => {
                format!("entry_missing:{detail}")
            }
            CommitWitnessClaimPairResolvability::EntryResolveFailed { detail } => {
                format!("entry_resolve_failed:{detail}")
            }
            CommitWitnessClaimPairResolvability::FunctionNotFound => {
                "function_not_found".to_string()
            }
        };
        defects.push((entry, function, cause));
    }
    defects
}

pub fn commit_witness_claim_roster_unresolvable_count() -> i64 {
    let defects = commit_witness_claim_roster_defects();
    for (entry, function, cause) in &defects {
        eprintln!(
            "COMMIT_WITNESS_CLAIM_ROSTER_REFUSAL count={} entry={entry} function={function} cause={cause}",
            defects.len()
        );
    }
    defects.len() as i64
}

/// The occurrence's terminal outcome as a stable receipt token.
///
/// EXHAUSTIVE ON PURPOSE -- there is no `_` arm. A new `ClaimOutcome` variant must fail to
/// compile here rather than be absorbed into a plausible existing label, which is the
/// silent-widen arm DESIGN section 5 forbids.
pub(crate) fn witness_execution_outcome_label(outcome: &ClaimOutcome) -> &'static str {
    match outcome {
        ClaimOutcome::Pass => "pass",
        ClaimOutcome::Fail => "fail",
        ClaimOutcome::NotBool { .. } => "not_bool",
        ClaimOutcome::RuntimeError { .. } => "runtime_error",
        // SITE 4. "timed_out" over both arms is what put `outcome=timed_out` on the over-cost
        // line of a row the same ledger reported as having completed.
        ClaimOutcome::BudgetInterrupted { .. } => "budget_interrupted",
        ClaimOutcome::CompletedOverBudget { .. } => "completed_over_budget",
        ClaimOutcome::HostToolUnresolved { .. } => "host_tool_unresolved",
        ClaimOutcome::HostEffectRefused { .. } => "host_effect_refused",
        ClaimOutcome::Panicked { .. } => "panicked",
        ClaimOutcome::NotAttempted { .. } => "not_attempted",
    }
}
