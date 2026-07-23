#![allow(clippy::disallowed_macros)]

//! SCAFFOLD (§7 seed-retained HAND-RUST — authority: docs/plans/namespace-resolution-design.md §13):
//! Host-boundary transport for `tools.walk_target_alias_apply.walk_target_alias_apply_module_fold`.
//! Apply authority lives in `.dag`; this bin is argv → plan rows → path/text projection →
//! in-process fold call → stdout/splice realization only (DESIGN §3 transport-is-a-handler).
//!
//! 🟡 dissolve-on: self-emit of this bin (or v2 bash-emit of the driver capability) from
//! `tools.walk_target_alias_apply_transport` (mirror of `cli_run_walk_target_alias_plan`).
//! Receipt carrier: `CLI_RUN_WALK_TARGET_ALIAS_APPLY_SCAFFOLD_MARKER` in `cli_run.rs`.
//!
//! §13 walk-target alias codemod — APPLY driver (namespace-resolution-design §13).
//!
//! SCAFFOLD (§7 seed-retained HAND-RUST — authority: docs/plans/namespace-resolution-design.md §13):
//! host transport for the `.dag` apply fold (`dag/tools/walk_target_alias_apply.dag`).
//! 🟡 dissolve-on: retires with its sibling `walk_target_alias_plan` when the §13
//! import→alias transmutation + refusal flip complete (the codemod is one-shot: once the
//! corpus is transmuted and the namespace terminal lands, both plan and apply drivers are
//! dead code and delete together — see `CLI_RUN_WALK_TARGET_ALIAS_PLAN_SCAFFOLD_MARKER`,
//! cli_run.rs). Receipt: `rg walk_target_alias_apply src/v1/stage0` until that lane closes;
//! ROADMAP namespace-only lane (docs/plans/namespace-resolution-design.md).
//!
//! Thin host transport for `tools.walk_target_alias_apply.walk_target_alias_apply_module_fold`:
//! argv parse -> plan rows (Rust #6984 planner, seed-retained) -> host-boundary projection
//! (declaring module -> storage rel path -> source text) -> in-process `.dag` interpreter
//! call -> stdout/exit-code projection and splice realization. No apply logic lives here:
//! anchors, splice offsets, alias surface text, skip/conflict/refuse decisions are all
//! decided by the `.dag` fold (DESIGN §3 transport-is-a-handler; precedent: `gunbc converge`).
//!
//! ```text
//! cargo run -p v1-compiler --bin walk_target_alias_apply
//! cargo run -p v1-compiler --bin walk_target_alias_apply -- --source-root dag
//! cargo run -p v1-compiler --bin walk_target_alias_apply -- --write   # refused until the
//!     v1 alias parse arm lands (typed refusal from the .dag corpus write gate)
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::cli_run::{
    build_module_path_index, make_eval_context, resolution_divergence_census_source_roots,
    resolve_entry_graph_shared, walk_target_alias_plan_live, whole_tree_probe_exclusion_substrings,
    WalkTargetAliasPlanClass, WalkTargetAliasPlanRow,
};
use v1_compiler::v1_interpreter::{run_in_context_with_args, ExecutionMode, InterpContext, Value};

const APPLY_ENTRY: &str = "dag/tools/walk_target_alias_apply.dag";
const APPLY_FN: &str = "walk_target_alias_apply_module_fold";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("workspace root")
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("walk_target_alias_apply: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn class_variant_name(class: WalkTargetAliasPlanClass) -> &'static str {
    match class {
        WalkTargetAliasPlanClass::GlobalBareLcp => "GlobalBareLcp",
        WalkTargetAliasPlanClass::GlobalBareLcpTie => "GlobalBareLcpTie",
        WalkTargetAliasPlanClass::FnParentFirstHit => "FnParentFirstHit",
    }
}

fn record(ctx: &InterpContext, type_name: &str, fields: Vec<(&str, Value)>) -> Value {
    Value::Record {
        type_name: ctx.sym(type_name),
        fields: Rc::new(
            fields
                .into_iter()
                .map(|(name, value)| (ctx.sym(name), value))
                .collect(),
        ),
    }
}

fn list(values: Vec<Value>) -> Value {
    Value::List(Rc::new(values.into_iter().collect()))
}

fn plan_row_value(ctx: &InterpContext, row: &WalkTargetAliasPlanRow) -> Value {
    let class = Value::Variant {
        type_name: ctx.sym("WalkTargetAliasPlanClass"),
        variant_name: ctx.sym(class_variant_name(row.key.class)),
        fields: Rc::new(vec![]),
    };
    let key = record(
        ctx,
        "WalkTargetAliasPlanRowKey",
        vec![
            ("class", class),
            (
                "declaring_module",
                Value::Str(row.key.declaring_module.clone()),
            ),
            ("binding", Value::Str(row.key.binding.clone())),
            (
                "walk_target_qualified_path",
                Value::Str(row.key.walk_target_qualified_path.clone()),
            ),
        ],
    );
    record(
        ctx,
        "WalkTargetAliasPlanRow",
        vec![
            ("key", key),
            (
                "pre_flip_qualified_path",
                Value::Str(row.pre_flip_qualified_path.clone()),
            ),
            ("walk_verified", Value::Bool(row.walk_verified)),
            ("lookup_events", Value::Int(row.lookup_events as i64)),
        ],
    )
}

fn module_input_value(
    ctx: &InterpContext,
    declaring_module: &str,
    rel_source_path: &str,
    source_text: &str,
    rows: &[&WalkTargetAliasPlanRow],
) -> Value {
    record(
        ctx,
        "WalkTargetAliasModuleInput",
        vec![
            ("declaring_module", Value::Str(declaring_module.to_string())),
            ("rel_source_path", Value::Str(rel_source_path.to_string())),
            ("source_text", Value::Str(source_text.to_string())),
            (
                "rows",
                list(rows.iter().map(|r| plan_row_value(ctx, r)).collect()),
            ),
        ],
    )
}

fn value_str(ctx: &InterpContext, value: &Value) -> String {
    match value {
        Value::Str(s) => s.clone(),
        other => ctx.format_value(other),
    }
}

fn value_int(value: &Value) -> i64 {
    match value {
        Value::Int(i) => *i,
        _ => -1,
    }
}

fn value_bool(value: &Value) -> bool {
    matches!(value, Value::Bool(true))
}

/// The `.dag` fold hands lists back in either carrier: the native `Value::List`
/// or the modeled `std.algebra` `Cons`/`Empty` variant chain (the tracked
/// model-vs-realization dual, DESIGN open thread). Decode both; anything else
/// decodes as empty and is caught by the probe's count check.
fn value_list_items(ctx: &InterpContext, value: &Value) -> Vec<Value> {
    match value {
        Value::List(items) => items.iter().cloned().collect(),
        Value::Variant { .. } => {
            let mut out = Vec::new();
            let mut cursor = value.clone();
            loop {
                let Value::Variant {
                    variant_name,
                    fields,
                    ..
                } = &cursor
                else {
                    break;
                };
                match ctx.resolve(*variant_name).as_str() {
                    "Cons" => {
                        if let Some(head) = ctx.field(fields, "head") {
                            out.push(head.clone());
                        }
                        let Some(tail) = ctx.field(fields, "tail").cloned() else {
                            break;
                        };
                        cursor = tail;
                    }
                    _ => break,
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

fn diagnostics_head_reason(ctx: &InterpContext, diagnostics: &Value) -> String {
    if let Value::Record { fields, .. } = diagnostics {
        if let Some(head) = ctx.field(fields, "head") {
            if let Value::Record {
                fields: head_fields,
                ..
            } = head
            {
                if let Some(reason) = ctx.field(head_fields, "reason") {
                    return value_str(ctx, reason);
                }
            }
        }
    }
    ctx.format_value(diagnostics)
}

/// Insert `insert` before the char at char-offset `offset` (tokenize counts chars,
/// not bytes; identical for ASCII sources). Fail-closed on out-of-range offsets.
fn apply_splice_char_offset(source: &str, offset: usize, insert: &str) -> Result<String, String> {
    let char_count = source.chars().count();
    if offset > char_count {
        return Err(format!(
            "splice offset {offset} out of range (source has {char_count} chars)"
        ));
    }
    let byte = source
        .char_indices()
        .nth(offset)
        .map(|(b, _)| b)
        .unwrap_or(source.len());
    let mut out = String::with_capacity(source.len() + insert.len());
    out.push_str(&source[..byte]);
    out.push_str(insert);
    out.push_str(&source[byte..]);
    Ok(out)
}

struct SpliceRow {
    rel_source_path: String,
    splice_offset: i64,
    insert_text: String,
}

/// Transport self-test: prove the argv -> Value marshaling -> `.dag` fold ->
/// decode -> stdout path on a synthetic module input, without the (slow)
/// whole-corpus planner. The fixture mirrors the long-lane
/// `walk_target_alias_apply_module_fold_end_to_end_holds` witness; the .dag
/// fold is the shared authority for what the right answer is.
fn run_marshal_probe(ws: &PathBuf) -> Result<ExitCode, ExitCode> {
    let dag_roots: Vec<String> = resolution_divergence_census_source_roots(ws);
    let (graph, indices) =
        resolve_entry_graph_shared(&dag_roots, &ws.join(APPLY_ENTRY).to_string_lossy()).map_err(
            |e| {
                eprintln!("walk_target_alias_apply: resolve failed for {APPLY_ENTRY}:\n{e}");
                ExitCode::from(2)
            },
        )?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let fixture_source =
        "module v2.test.walk_target_alias_apply.probe\nimport v2.std.algebra { Cons, Empty }\n";
    let row = WalkTargetAliasPlanRow {
        key: v1_compiler::cli_run::WalkTargetAliasPlanKey {
            class: WalkTargetAliasPlanClass::GlobalBareLcp,
            declaring_module: "v2.test.walk_target_alias_apply.probe".to_string(),
            binding: "ProbeAlias".to_string(),
            walk_target_qualified_path: "test.aliasplan.near.AmbigType".to_string(),
        },
        lookup_events: 1,
        walk_verified: true,
        pre_flip_qualified_path: "test.aliasplan.near.AmbigType".to_string(),
    };
    let module_input = module_input_value(
        &ctx,
        "v2.test.walk_target_alias_apply.probe",
        "v2/test/walk_target_alias_apply/probe.dag",
        fixture_source,
        &[&row],
    );
    let call_args = [
        (Some("inputs".to_string()), list(vec![module_input])),
        (
            Some("source_scope_label".to_string()),
            Value::Str("marshal-probe".to_string()),
        ),
        (Some("write_requested".to_string()), Value::Bool(false)),
    ];
    let result = run_in_context_with_args(&ctx, APPLY_FN, &call_args, false).map_err(|e| {
        eprintln!("walk_target_alias_apply: marshal probe runtime error: {e}");
        ExitCode::from(1)
    })?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &result
    else {
        eprintln!("walk_target_alias_apply: marshal probe returned a non-variant");
        return Err(ExitCode::from(1));
    };
    if ctx.resolve(*variant_name) != "Accepted" {
        eprintln!(
            "walk_target_alias_apply: marshal probe REFUSED: {}",
            ctx.field(fields, "diagnostics")
                .map(|d| diagnostics_head_reason(&ctx, d))
                .unwrap_or_else(|| ctx.format_value(&result))
        );
        return Err(ExitCode::from(1));
    }
    let Some(Value::Record {
        fields: report_fields,
        ..
    }) = ctx.field(fields, "value").cloned()
    else {
        eprintln!("walk_target_alias_apply: marshal probe Accepted arm carried no report");
        return Err(ExitCode::from(1));
    };
    let edits = ctx
        .field(&report_fields, "edits")
        .map(|v| value_list_items(&ctx, v))
        .unwrap_or_default();
    let splices = ctx
        .field(&report_fields, "splices")
        .map(|v| value_list_items(&ctx, v))
        .unwrap_or_default();
    let refused = ctx
        .field(&report_fields, "refused")
        .map(|v| value_list_items(&ctx, v))
        .unwrap_or_default();
    if edits.len() != 1 || splices.len() != 1 || !refused.is_empty() {
        eprintln!(
            "walk_target_alias_apply: marshal probe expected 1 edit / 1 splice / 0 refusals, got {}/{}/{}",
            edits.len(),
            splices.len(),
            refused.len()
        );
        eprintln!(
            "walk_target_alias_apply: marshal probe report value: {}",
            ctx.format_value(&Value::Record {
                type_name: ctx.sym("WalkTargetAliasApplyReport"),
                fields: Rc::new(report_fields.to_vec()),
            })
        );
        return Err(ExitCode::from(1));
    }
    let alias_line = edits
        .first()
        .and_then(|e| match e {
            Value::Record { fields, .. } => {
                ctx.field(fields, "alias_line").map(|v| value_str(&ctx, v))
            }
            _ => None,
        })
        .unwrap_or_default();
    let offset = splices
        .first()
        .and_then(|s| match s {
            Value::Record { fields, .. } => ctx.field(fields, "splice_offset").map(value_int),
            _ => None,
        })
        .unwrap_or(-1);
    println!("[walk-target-alias-apply] marshal-probe ok: alias_line={alias_line} splice_offset={offset}");
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, ExitCode> {
    let ws = workspace_root();
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude = whole_tree_probe_exclusion_substrings();
    let mut write = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                let rel = require_value(&args, i, "--source-root")?;
                source_roots.push(ws.join(&rel).to_string_lossy().into_owned());
            }
            "--exclude-subpath" => {
                i += 1;
                exclude.push(require_value(&args, i, "--exclude-subpath")?);
            }
            "--write" => write = true,
            "--marshal-probe" => return run_marshal_probe(&ws),
            other => {
                eprintln!("walk_target_alias_apply: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        source_roots = resolution_divergence_census_source_roots(&ws);
    }

    let plan = walk_target_alias_plan_live(&source_roots, &exclude).map_err(|e| {
        eprintln!("walk_target_alias_apply: plan resolve failed:\n{e}");
        ExitCode::from(2)
    })?;
    println!(
        "[walk-target-alias-apply] scope={} plan_rows={} plan_refused={}",
        plan.source_scope_label,
        plan.rows.len(),
        plan.refused.len()
    );
    if plan.rows.is_empty() {
        println!("[walk-target-alias-apply] no plan rows — nothing to apply");
        return Ok(ExitCode::SUCCESS);
    }

    // Host-boundary projection: declaring module -> storage rel path -> source text.
    // Missing projections are typed, counted refusals (never silently dropped rows).
    let module_index = build_module_path_index(&source_roots);
    let mut by_module: BTreeMap<String, Vec<&WalkTargetAliasPlanRow>> = BTreeMap::new();
    for row in &plan.rows {
        by_module
            .entry(row.key.declaring_module.clone())
            .or_default()
            .push(row);
    }
    let mut host_refused: Vec<String> = Vec::new();
    let mut module_sources: Vec<(String, String, String, Vec<&WalkTargetAliasPlanRow>)> =
        Vec::new();
    for (module, rows) in by_module {
        let Some(rel) = module_index.get(&module).cloned() else {
            host_refused.push(format!(
                "HOST_PROJECTION_REFUSED\tmodule={module}\treason=not in module_path_index"
            ));
            continue;
        };
        match std::fs::read_to_string(ws.join(&rel)) {
            Ok(text) => module_sources.push((module, rel, text, rows)),
            Err(e) => host_refused.push(format!(
                "HOST_PROJECTION_REFUSED\tmodule={module}\treason=read {rel}: {e}"
            )),
        }
    }

    // In-process `.dag` call: the fold decides everything.
    let dag_roots: Vec<String> = resolution_divergence_census_source_roots(&ws);
    let (graph, indices) =
        resolve_entry_graph_shared(&dag_roots, &ws.join(APPLY_ENTRY).to_string_lossy()).map_err(
            |e| {
                eprintln!("walk_target_alias_apply: resolve failed for {APPLY_ENTRY}:\n{e}");
                ExitCode::from(2)
            },
        )?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let inputs: Vec<Value> = module_sources
        .iter()
        .map(|(module, rel, text, rows)| module_input_value(&ctx, module, rel, text, rows))
        .collect();
    let call_args = [
        (Some("inputs".to_string()), list(inputs)),
        (
            Some("source_scope_label".to_string()),
            Value::Str(plan.source_scope_label.clone()),
        ),
        (Some("write_requested".to_string()), Value::Bool(write)),
    ];
    let result = run_in_context_with_args(&ctx, APPLY_FN, &call_args, false).map_err(|e| {
        eprintln!("walk_target_alias_apply: runtime error: {e}");
        ExitCode::from(1)
    })?;

    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &result
    else {
        eprintln!(
            "walk_target_alias_apply: {APPLY_FN} returned an unexpected shape: {}",
            ctx.format_value(&result)
        );
        return Err(ExitCode::from(1));
    };
    if ctx.resolve(*variant_name) == "Rejected" {
        let reason = ctx
            .field(fields, "diagnostics")
            .map(|d| diagnostics_head_reason(&ctx, d))
            .unwrap_or_else(|| ctx.format_value(&result));
        eprintln!("walk_target_alias_apply: REFUSED: {reason}");
        return Err(ExitCode::from(1));
    }
    let Some(Value::Record {
        fields: report_fields,
        ..
    }) = ctx.field(fields, "value").cloned()
    else {
        eprintln!("walk_target_alias_apply: Accepted arm carried no report record");
        return Err(ExitCode::from(1));
    };

    let edits = ctx
        .field(&report_fields, "edits")
        .map(|v| value_list_items(&ctx, v))
        .unwrap_or_default();
    let splices = ctx
        .field(&report_fields, "splices")
        .map(|v| value_list_items(&ctx, v))
        .unwrap_or_default();
    let dag_refused = ctx
        .field(&report_fields, "refused")
        .map(|v| value_list_items(&ctx, v))
        .unwrap_or_default();
    let skipped_existing = ctx
        .field(&report_fields, "skipped_existing")
        .map(value_int)
        .unwrap_or(-1);

    for edit in &edits {
        if let Value::Record { fields, .. } = edit {
            let get = |name: &str| {
                ctx.field(fields, name)
                    .map(|v| value_str(&ctx, v))
                    .unwrap_or_default()
            };
            println!(
                "ALIAS_APPLY_EDIT\tmodule={}\tpath={}\tbinding={}\talias_line={}",
                get("declaring_module"),
                get("rel_source_path"),
                get("binding"),
                get("alias_line"),
            );
        }
    }
    let mut splice_rows: Vec<SpliceRow> = Vec::new();
    for splice in &splices {
        if let Value::Record { fields, .. } = splice {
            let path = ctx
                .field(fields, "rel_source_path")
                .map(|v| value_str(&ctx, v))
                .unwrap_or_default();
            let offset = ctx
                .field(fields, "splice_offset")
                .map(value_int)
                .unwrap_or(-1);
            let insert_text = ctx
                .field(fields, "insert_text")
                .map(|v| value_str(&ctx, v))
                .unwrap_or_default();
            println!(
                "ALIAS_APPLY_SPLICE\tpath={path}\toffset={offset}\tinsert_chars={}",
                insert_text.chars().count()
            );
            splice_rows.push(SpliceRow {
                rel_source_path: path,
                splice_offset: offset,
                insert_text,
            });
        }
    }
    for refusal in &dag_refused {
        println!("ALIAS_APPLY_REFUSED\t{}", value_str(&ctx, refusal));
    }
    for refusal in &host_refused {
        println!("{refusal}");
    }

    let write_mode = if write { "write" } else { "dry-run" };
    println!(
        "[walk-target-alias-apply] mode={write_mode} edits={} splices={} skipped_existing={skipped_existing} dag_refused={} host_refused={}",
        edits.len(),
        splices.len(),
        dag_refused.len(),
        host_refused.len()
    );

    // Splice realization: only reachable once the .dag corpus write gate lifts
    // (today write_requested=true is refused above with a typed reason).
    if write {
        for row in &splice_rows {
            let abs = ws.join(&row.rel_source_path);
            let source = std::fs::read_to_string(&abs).map_err(|e| {
                eprintln!(
                    "walk_target_alias_apply: write read-back {}: {e}",
                    row.rel_source_path
                );
                ExitCode::from(1)
            })?;
            let offset = usize::try_from(row.splice_offset).map_err(|_| {
                eprintln!(
                    "walk_target_alias_apply: negative splice offset for {}",
                    row.rel_source_path
                );
                ExitCode::from(1)
            })?;
            let next =
                apply_splice_char_offset(&source, offset, &row.insert_text).map_err(|e| {
                    eprintln!("walk_target_alias_apply: {}: {e}", row.rel_source_path);
                    ExitCode::from(1)
                })?;
            std::fs::write(&abs, next).map_err(|e| {
                eprintln!(
                    "walk_target_alias_apply: write {}: {e}",
                    row.rel_source_path
                );
                ExitCode::from(1)
            })?;
            println!("ALIAS_APPLY_WRITTEN\tpath={}", row.rel_source_path);
        }
    }

    if !host_refused.is_empty() || !dag_refused.is_empty() {
        return Err(ExitCode::from(1));
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::apply_splice_char_offset;

    #[test]
    fn splice_mid_source_inserts_before_offset_char() {
        let out = apply_splice_char_offset("abcdef", 3, "-X-").expect("splice");
        assert_eq!(out, "abc-X-def");
    }

    #[test]
    fn splice_at_end_appends() {
        let out = apply_splice_char_offset("abc", 3, "-X").expect("splice");
        assert_eq!(out, "abc-X");
    }

    #[test]
    fn splice_after_import_block_shape() {
        let src = "module m.p\nimport a.b { C }\n\nfn f() {}\n";
        // char offset of the closing `}` of the import block, exclusive end
        let offset = src.find("}\n\nfn").expect("anchor") + 1;
        let out = apply_splice_char_offset(src, offset, "\n\nalias X = a.b.C").expect("splice");
        assert_eq!(
            out,
            "module m.p\nimport a.b { C }\n\nalias X = a.b.C\n\nfn f() {}\n"
        );
    }

    #[test]
    fn splice_out_of_range_refuses() {
        let err = apply_splice_char_offset("abc", 4, "-X").expect_err("must refuse");
        assert!(err.contains("out of range"), "got: {err}");
    }
}
