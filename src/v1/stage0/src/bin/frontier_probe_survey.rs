#![allow(clippy::disallowed_macros)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use v1_compiler::cli_run::{
    discover_source_root_reads_for_entry, emit_source_root_ingest_manifest,
    parse_source_root_entry_admission, resolve_entry_graph,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

const PROBE_ENTRY: &str = "src/v2/test/claim/long/compiler_frontier_probe_entry_test.dag";
const PROBE_RECEIPT_FN: &str = "frontier_probe_entry_receipt";
const WITNESS_LAYER_ROOTS: &[&str] = &["dag", "src/v2"];
const FRONTIER_SWEEP_ORDER_ENTRY: &str = "src/v2/compiler/self_host/frontier.dag";
const SWEEP_ORDER_DATA: &str = "compiler_frontier_sweep_order";
const HOST_REASON_SYMBOLS_ENTRY: &str = "src/v2/compiler/self_host/frontier_probe_types.dag";
const HOST_RUNTIME_ERROR_REASON_DATA: &str = "frontier_probe_survey_host_runtime_error_reason";
const HOST_OOM_OR_BUDGET_REASON_DATA: &str = "frontier_probe_survey_host_oom_or_budget_reason";

fn free_monoid_elems<'a>(value: &'a Value, ctx: &InterpContext) -> Result<Vec<&'a Value>, String> {
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .ok_or_else(|| "Cons without `head` field".to_string())?;
                out.push(head);
                cur = ctx
                    .field(fields, "tail")
                    .ok_or_else(|| "Cons without `tail` field".to_string())?;
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => {
                return Ok(out);
            }
            Value::List(items) => {
                out.extend(items.iter());
                return Ok(out);
            }
            other => {
                return Err(format!(
                    "expected a List (Cons/Empty), got {}",
                    other.type_label_public()
                ));
            }
        }
    }
}

fn str_list_from_value(value: &Value, ctx: &InterpContext) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for elem in free_monoid_elems(value, ctx)? {
        match elem {
            Value::Str(s) => out.push(s.clone()),
            other => {
                return Err(format!(
                    "expected a List<String> element, got {}",
                    other.type_label_public()
                ));
            }
        }
    }
    Ok(out)
}

fn load_compiler_frontier_sweep_order(source_roots: &[String]) -> Result<Vec<String>, String> {
    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.extend(source_roots.iter().cloned());
    let (graph, source_indices) =
        resolve_entry_graph(&roots, FRONTIER_SWEEP_ORDER_ENTRY).map_err(|e| format!("{e}"))?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value = v1_interpreter::run_in_context(&ctx, SWEEP_ORDER_DATA, true)
        .map_err(|e| format!("read {SWEEP_ORDER_DATA}: {e}"))?;
    str_list_from_value(&value, &ctx)
}

struct ProbeReceiptRow {
    module_path: String,
    blocker_variant: String,
    located_stage: String,
    located_reason: String,
    assemble_detail_manifest: String,
    rejection_chain: String,
    overlap_roster_detail: String,
    probe_error: Option<String>,
}

struct HostFailureReasons {
    runtime_error: String,
    oom_or_budget: String,
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("frontier_probe_survey: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn symbol_to_manifest(sym: &str) -> String {
    if sym.starts_with('^') {
        sym.to_string()
    } else {
        format!("^{sym}")
    }
}

fn value_symbol_name(value: &Value) -> Result<String, String> {
    match value {
        Value::Str(s) => Ok(symbol_to_manifest(s)),
        other => Err(format!(
            "expected symbol String, got {}",
            other.type_label_public()
        )),
    }
}

fn load_symbol_data_from_entry(
    source_roots: &[String],
    entry: &str,
    data_name: &str,
) -> Result<String, String> {
    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.extend(source_roots.iter().cloned());
    let (graph, source_indices) = resolve_entry_graph(&roots, entry).map_err(|e| format!("{e}"))?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value = v1_interpreter::run_in_context(&ctx, data_name, true)
        .map_err(|e| format!("read {data_name}: {e}"))?;
    let sym = value_symbol_name(&value)?;
    Ok(sym.trim_start_matches('^').to_string())
}

fn load_host_failure_reasons(source_roots: &[String]) -> Result<HostFailureReasons, String> {
    Ok(HostFailureReasons {
        runtime_error: load_symbol_data_from_entry(
            source_roots,
            HOST_REASON_SYMBOLS_ENTRY,
            HOST_RUNTIME_ERROR_REASON_DATA,
        )?,
        oom_or_budget: load_symbol_data_from_entry(
            source_roots,
            HOST_REASON_SYMBOLS_ENTRY,
            HOST_OOM_OR_BUDGET_REASON_DATA,
        )?,
    })
}

fn variant_name(ctx: &InterpContext, value: &Value) -> Result<String, String> {
    match value {
        Value::Variant { variant_name, .. } => Ok(ctx.resolve(*variant_name)),
        _ => Err(format!(
            "expected variant, got {}",
            value.type_label_public()
        )),
    }
}

fn record_field<'a>(
    ctx: &InterpContext,
    value: &'a Value,
    name: &str,
) -> Result<&'a Value, String> {
    match value {
        Value::Record { fields, .. } => ctx
            .field(fields, name)
            .ok_or_else(|| format!("record missing field {name}")),
        _ => Err(format!(
            "expected record, got {}",
            value.type_label_public()
        )),
    }
}

fn blocker_variant_emit(blocker: &Value, ctx: &InterpContext) -> Result<String, String> {
    match blocker {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "UnknownProbeCause" {
                let reason = ctx
                    .field(fields, "reason")
                    .or_else(|| ctx.field(fields, "cause"))
                    .ok_or_else(|| format!("{name} missing reason/cause"))?;
                let reason_sym = value_symbol_name(reason)?;
                Ok(format!("{name} {{ reason: {reason_sym} }}"))
            } else if name == "UpstreamSemanticRefusal" {
                // HAND-RUST GATE (+12 LOC): mechanical serialize arm for new coproduct variant —
                // see v2.compiler.self_host.frontier_probe_survey::frontier_probe_survey_blocker_variant_emit_rust_gate_note
                let diagnostic_class = ctx
                    .field(fields, "diagnostic_class")
                    .ok_or_else(|| format!("{name} missing diagnostic_class"))?;
                let diagnostic_name = ctx
                    .field(fields, "diagnostic_name")
                    .ok_or_else(|| format!("{name} missing diagnostic_name"))?;
                let class_sym = value_symbol_name(diagnostic_class)?;
                let name_sym = value_symbol_name(diagnostic_name)?;
                Ok(format!(
                    "{name} {{ diagnostic_class: {class_sym}, diagnostic_name: {name_sym} }}"
                ))
            } else {
                Ok(name)
            }
        }
        _ => Err("blocker_class not a variant".to_string()),
    }
}

fn symbol_list_csv(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let mut parts = Vec::new();
    for item in free_monoid_elems(value, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    Ok(parts.join(","))
}

fn non_empty_symbol_chain_csv_flat(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let head = record_field(ctx, value, "head")?;
    let tail = record_field(ctx, value, "tail")?;
    let mut parts = vec![value_symbol_name(head)?];
    for item in free_monoid_elems(tail, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    Ok(parts.join(","))
}

fn grammar_choice_ambiguity_row_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let production = value_symbol_name(record_field(ctx, value, "production")?)?;
    let left_list = symbol_list_dag_emit(record_field(ctx, value, "left_first_terminals")?, ctx)?;
    let right_list = symbol_list_dag_emit(record_field(ctx, value, "right_first_terminals")?, ctx)?;
    let overlap_list = symbol_list_dag_emit(record_field(ctx, value, "overlap_terminals")?, ctx)?;
    let both_nullable = match record_field(ctx, value, "both_nullable")? {
        Value::Bool(b) => {
            if *b {
                "true"
            } else {
                "false"
            }
        }
        other => {
            return Err(format!(
                "both_nullable not Bool: {}",
                other.type_label_public()
            ));
        }
    };
    Ok(format!(
        "GrammarChoiceAmbiguityRow {{\n  production: {production},\n  left_first_terminals: {left_list},\n  right_first_terminals: {right_list},\n  overlap_terminals: {overlap_list},\n  both_nullable: {both_nullable}\n}}"
    ))
}

fn symbol_list_dag_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let mut parts = Vec::new();
    for item in free_monoid_elems(value, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    let mut list = String::from("Empty");
    while let Some(sym) = parts.pop() {
        list = format!("Cons {{\n  head: {sym},\n  tail: {list}\n}}");
    }
    Ok(list)
}

fn non_empty_grammar_rows_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let head = grammar_choice_ambiguity_row_emit(record_field(ctx, value, "head")?, ctx)?;
    let tail = record_field(ctx, value, "tail")?;
    let mut tail_nodes: Vec<String> = Vec::new();
    for item in free_monoid_elems(tail, ctx)? {
        tail_nodes.push(grammar_choice_ambiguity_row_emit(item, ctx)?);
    }
    let mut list = String::from("Empty");
    while let Some(node) = tail_nodes.pop() {
        list = format!("Cons {{\n  head: {node},\n  tail: {list}\n}}");
    }
    Ok(format!(
        "NonEmptyGrammarChoiceAmbiguityRows {{\n  head: {head},\n  tail: {list}\n}}"
    ))
}

fn assemble_rejection_cause_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "GrammarChoiceOverlap" {
                let rows = ctx
                    .field(fields, "rows")
                    .ok_or_else(|| format!("{name} missing rows"))?;
                Ok(format!(
                    "GrammarChoiceOverlap {{ rows: {} }}",
                    non_empty_grammar_rows_emit(rows, ctx)?
                ))
            } else if name == "MaskedAssembleRefusal" {
                let overlap_evidence = ctx
                    .field(fields, "overlap_evidence")
                    .ok_or_else(|| format!("{name} missing overlap_evidence"))?;
                let remaining_chain = ctx
                    .field(fields, "remaining_chain")
                    .ok_or_else(|| format!("{name} missing remaining_chain"))?;
                Ok(format!(
                    "MaskedAssembleRefusal {{\n  overlap_evidence: {},\n  remaining_chain: {}\n}}",
                    non_empty_grammar_rows_emit(overlap_evidence, ctx)?,
                    non_empty_symbol_chain_emit(remaining_chain, ctx)?
                ))
            } else if name == "MaskedAssembleRefusalWithoutEvidence" {
                let remaining_chain = ctx
                    .field(fields, "remaining_chain")
                    .ok_or_else(|| format!("{name} missing remaining_chain"))?;
                Ok(format!(
                    "MaskedAssembleRefusalWithoutEvidence {{\n  remaining_chain: {}\n}}",
                    non_empty_symbol_chain_emit(remaining_chain, ctx)?
                ))
            } else if name == "MissingModule" {
                let requested = ctx
                    .field(fields, "requested")
                    .ok_or_else(|| format!("{name} missing requested"))?;
                Ok(format!(
                    "MissingModule {{ requested: {} }}",
                    qualified_name_emit(requested, ctx)?
                ))
            } else if name == "MissingModuleIdentityUnavailable" {
                Ok("MissingModuleIdentityUnavailable".to_string())
            } else if name == "OtherAssembleRefusal" {
                let reason = ctx
                    .field(fields, "reason")
                    .ok_or_else(|| format!("{name} missing reason"))?;
                Ok(format!(
                    "OtherAssembleRefusal {{ reason: {} }}",
                    value_symbol_name(reason)?
                ))
            } else {
                Err(format!("unknown AssembleRejectionCause variant {name}"))
            }
        }
        _ => Err("assemble cause not a variant".to_string()),
    }
}

fn qualified_name_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let mut parts = Vec::new();
    for item in free_monoid_elems(value, ctx)? {
        parts.push(value_symbol_name(item)?);
    }
    let mut list = String::from("Empty");
    while let Some(sym) = parts.pop() {
        list = format!("Cons {{\n  head: {sym},\n  tail: {list}\n}}");
    }
    Ok(list)
}

fn assemble_detail_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let name = ctx.resolve(*variant_name);
            if name == "NoFrontierProbeDetail" {
                Ok("NoFrontierProbeDetail".to_string())
            } else if name == "AssembleRejectionDetail" {
                let chain = ctx
                    .field(fields, "rejection_chain")
                    .ok_or_else(|| format!("{name} missing rejection_chain"))?;
                let cause = ctx
                    .field(fields, "cause")
                    .ok_or_else(|| format!("{name} missing cause"))?;
                Ok(format!(
                    "AssembleRejectionDetail {{\n  rejection_chain: {},\n  cause: {}\n}}",
                    non_empty_symbol_chain_emit(chain, ctx)?,
                    assemble_rejection_cause_emit(cause, ctx)?
                ))
            } else {
                Err(format!("unknown assemble_detail variant {name}"))
            }
        }
        _ => Err("assemble_detail not a variant".to_string()),
    }
}

fn non_empty_symbol_chain_emit(value: &Value, ctx: &InterpContext) -> Result<String, String> {
    let head = value_symbol_name(record_field(ctx, value, "head")?)?;
    let tail = symbol_list_dag_emit(record_field(ctx, value, "tail")?, ctx)?;
    Ok(format!(
        "NonEmptySymbolChain {{\n  head: {head},\n  tail: {tail}\n}}"
    ))
}

fn overlap_roster_detail_from_assemble_detail(
    value: &Value,
    ctx: &InterpContext,
) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "AssembleRejectionDetail") => {
            let cause = ctx
                .field(fields, "cause")
                .ok_or_else(|| "AssembleRejectionDetail missing cause".to_string())?;
            match cause {
                Value::Variant {
                    variant_name: cause_name,
                    fields: cause_fields,
                    ..
                } if ctx.sym_eq(*cause_name, "GrammarChoiceOverlap") => {
                    let rows = ctx
                        .field(cause_fields, "rows")
                        .ok_or_else(|| "GrammarChoiceOverlap missing rows".to_string())?;
                    overlap_rows_detail_from_rows_value(rows, ctx)
                }
                _ => Ok(String::new()),
            }
        }
        _ => Ok(String::new()),
    }
}

fn overlap_rows_detail_from_rows_value(
    value: &Value,
    ctx: &InterpContext,
) -> Result<String, String> {
    let mut rows = Vec::new();
    let head = record_field(ctx, value, "head")?;
    rows.push(head);
    for item in free_monoid_elems(record_field(ctx, value, "tail")?, ctx)? {
        rows.push(item);
    }
    let mut out = Vec::new();
    for row in rows {
        let production = value_symbol_name(record_field(ctx, row, "production")?)?;
        let left = symbol_list_csv(record_field(ctx, row, "left_first_terminals")?, ctx)?;
        let right = symbol_list_csv(record_field(ctx, row, "right_first_terminals")?, ctx)?;
        let overlap = symbol_list_csv(record_field(ctx, row, "overlap_terminals")?, ctx)?;
        out.push(format!(
            "{production}|left={left}|right={right}|overlap={overlap}"
        ));
    }
    Ok(out.join(";"))
}

fn rejection_chain_from_assemble_detail(
    value: &Value,
    ctx: &InterpContext,
) -> Result<String, String> {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "AssembleRejectionDetail") => {
            let chain = ctx
                .field(fields, "rejection_chain")
                .ok_or_else(|| "AssembleRejectionDetail missing rejection_chain".to_string())?;
            non_empty_symbol_chain_csv_flat(chain, ctx)
        }
        _ => Ok(String::new()),
    }
}

fn extract_probe_receipt(value: &Value, ctx: &InterpContext) -> Result<ProbeReceiptRow, String> {
    let module_path = match record_field(ctx, value, "module_path")? {
        Value::Str(s) => s.clone(),
        other => {
            return Err(format!(
                "module_path not String: {}",
                other.type_label_public()
            ));
        }
    };
    let blocker = record_field(ctx, value, "blocker_class")?;
    let stage = record_field(ctx, value, "located_stage")?;
    let reason = record_field(ctx, value, "located_reason")?;
    let assemble_detail = record_field(ctx, value, "assemble_detail")?;
    let assemble_detail_manifest = assemble_detail_emit(assemble_detail, ctx)?;
    let rejection_chain = rejection_chain_from_assemble_detail(assemble_detail, ctx)?;
    let overlap_roster_detail = overlap_roster_detail_from_assemble_detail(assemble_detail, ctx)?;
    Ok(ProbeReceiptRow {
        module_path,
        blocker_variant: blocker_variant_emit(blocker, ctx)?,
        located_stage: variant_name(ctx, stage)?,
        located_reason: value_symbol_name(reason)?,
        assemble_detail_manifest,
        rejection_chain,
        overlap_roster_detail,
        probe_error: None,
    })
}

fn unknown_probe_row(module_path: &str, cause: &str) -> ProbeReceiptRow {
    ProbeReceiptRow {
        module_path: module_path.replace('\\', "/"),
        blocker_variant: format!("UnknownProbeCause {{ reason: ^{cause} }}"),
        located_stage: "ProbeStageAssemble".to_string(),
        located_reason: format!("^{cause}"),
        assemble_detail_manifest: "NoFrontierProbeDetail".to_string(),
        rejection_chain: String::new(),
        overlap_roster_detail: String::new(),
        probe_error: Some(cause.to_string()),
    }
}

fn write_probe_overlay_manifest(module_path: &str, ingest_manifest: &Path) -> Result<(), String> {
    let escaped = module_path.replace('\\', "/");
    // The emitted ingest manifest already imports `String` in its header block
    // (`host_source_root_ingest_content_hash: String`), and post-namespace-wave the
    // grammar accepts imports ONLY in the header — a trailing `import` at EOF is now a
    // parse error ("expected item declaration"). Append the `data` decl alone; String
    // is already in scope.
    let append = format!("\ndata frontier_probe_entry_module_path: String = \"{escaped}\"\n");
    let mut body =
        fs::read_to_string(ingest_manifest).map_err(|e| format!("read ingest manifest: {e}"))?;
    if body.contains("frontier_probe_entry_module_path") {
        return Ok(());
    }
    body.push_str(&append);
    fs::write(ingest_manifest, body).map_err(|e| format!("append module_path overlay: {e}"))
}

fn per_module_overlay_dir(survey_dir: &Path, module_path: &str) -> PathBuf {
    let slug = module_path
        .replace('\\', "/")
        .trim_start_matches("src/v2/")
        .replace('/', "__");
    survey_dir.join(slug)
}

fn run_probe_for_module(
    source_roots: &[String],
    survey_dir: &Path,
    module_path: &str,
) -> Result<ProbeReceiptRow, String> {
    let exclude = vec!["host_source_root_ingest_manifest.dag".to_string()];
    let overlay_dir = per_module_overlay_dir(survey_dir, module_path);
    fs::create_dir_all(&overlay_dir)
        .map_err(|e| format!("mkdir overlay {:?}: {e}", overlay_dir))?;

    let discover_root = source_roots
        .first()
        .cloned()
        .unwrap_or_else(|| "src/v2".to_string());
    let records = discover_source_root_reads_for_entry(&[discover_root], module_path, &exclude)?;
    let entry_source =
        fs::read_to_string(module_path).map_err(|e| format!("read entry {module_path}: {e}"))?;
    let admission = parse_source_root_entry_admission(&entry_source)?;
    let ingest_manifest = overlay_dir.join("host_source_root_ingest_manifest.dag");
    emit_source_root_ingest_manifest(&ingest_manifest, &records, Some(&admission))?;
    write_probe_overlay_manifest(module_path, &ingest_manifest)?;

    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.push(overlay_dir.to_string_lossy().into_owned());

    let (graph, source_indices) = resolve_entry_graph(&roots, PROBE_ENTRY)?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value =
        v1_interpreter::run_in_context(&ctx, PROBE_RECEIPT_FN, true).map_err(|e| format!("{e}"))?;
    extract_probe_receipt(&value, &ctx)
}

fn emit_receipt_row(row: &ProbeReceiptRow) -> String {
    format!(
        "FrontierProbeReceipt {{\n  module_path: \"{}\",\n  blocker_class: {},\n  located_stage: {},\n  located_reason: {},\n  assemble_detail: {}\n}}",
        row.module_path.replace('\\', "/"),
        row.blocker_variant,
        row.located_stage,
        row.located_reason,
        row.assemble_detail_manifest,
    )
}

fn emit_survey_manifest(path: &Path, rows: &[ProbeReceiptRow]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {:?}: {e}", parent))?;
    }
    let mut receipt_nodes: Vec<String> = rows.iter().map(emit_receipt_row).collect();
    let mut list = String::from("Empty");
    while let Some(head) = receipt_nodes.pop() {
        list = format!("Cons {{\n  head: {head},\n  tail: {list}\n}}");
    }
    let body = format!(
        "// GENERATED by frontier_probe_survey — execution receipt table. Regenerate via frontier_probe_survey bin.\n\
         module v2.test.workflow.host_frontier_probe_survey_manifest\n\n\
         import v2.compiler.self_host.frontier_probe_types {{ FrontierProbeReceipt }}\n\
         import v2.std.grammar_choice_ambiguity {{ GrammarChoiceAmbiguityRow, NonEmptyGrammarChoiceAmbiguityRows }}\n\
         import v2.std.algebra {{ Cons, Empty }}\n\
         import v2.std.collection {{ List }}\n\
         import v2.std.logic {{ Int }}\n\n\
         data host_frontier_probe_survey_module_count: Int = {}\n\n\
         data host_frontier_probe_survey: List<FrontierProbeReceipt> = {list}\n",
        rows.len()
    );
    fs::write(path, body).map_err(|e| format!("write survey manifest: {e}"))
}

fn emit_tsv(path: &Path, rows: &[ProbeReceiptRow]) -> Result<(), String> {
    let mut out = fs::File::create(path).map_err(|e| format!("create tsv: {e}"))?;
    writeln!(
        out,
        "module\tself_emit_ready\tblocker_class\tlocated_stage\tlocated_reason\trejection_chain\toverlap_roster_detail\tprobe_error"
    )
    .map_err(|e| format!("write tsv header: {e}"))?;
    for row in rows {
        let self_emit = if row.blocker_variant == "SelfEmitReady" {
            "yes"
        } else {
            "no"
        };
        writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.module_path,
            self_emit,
            row.blocker_variant,
            row.located_stage,
            row.located_reason,
            row.rejection_chain,
            row.overlap_roster_detail,
            row.probe_error.as_deref().unwrap_or("")
        )
        .map_err(|e| format!("write tsv row: {e}"))?;
    }
    Ok(())
}

// Diagnostic stderr histogram only — NOT the Wave-2 sizing verdict authority.
// Canonical clustering: `frontier_gap_clustering_from_receipts` in
// `v2.compiler.self_host.frontier_probe_survey` (witness:
// `compiler_frontier_gap_clustering_common_root_at_blocker_class_holds`).
// Deletes with the seed `frontier_probe_survey` bin (P5 scaffold).
fn cluster_summary(rows: &[ProbeReceiptRow]) -> String {
    use std::collections::BTreeMap;
    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_blocker: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        *by_reason.entry(row.located_reason.clone()).or_insert(0) += 1;
        *by_blocker.entry(row.blocker_variant.clone()).or_insert(0) += 1;
    }
    let mut lines = vec![
        "=== frontier_probe_survey cluster summary ===".to_string(),
        format!("modules: {}", rows.len()),
        "--- by blocker_class ---".to_string(),
    ];
    for (k, v) in &by_blocker {
        lines.push(format!("  {v}\t{k}"));
    }
    lines.push("--- by located_reason (root clustering) ---".to_string());
    for (k, v) in &by_reason {
        lines.push(format!("  {v}\t{k}"));
    }
    lines.push(format!("distinct located_reasons: {}", by_reason.len()));
    lines.push(
        "verdict: (see frontier_gap_clustering_from_receipts on emitted manifest)".to_string(),
    );
    lines.join("\n")
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut survey_manifest: Option<PathBuf> = None;
    let mut survey_tsv: Option<PathBuf> = None;
    let mut survey_dir: Option<PathBuf> = None;
    let mut only_modules: Vec<String> = Vec::new();
    let mut used_module_filter = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--module" => {
                i += 1;
                only_modules.push(require_value(&args, i, "--module")?);
                used_module_filter = true;
            }
            "--emit-survey-manifest" => {
                i += 1;
                survey_manifest = Some(PathBuf::from(require_value(
                    &args,
                    i,
                    "--emit-survey-manifest",
                )?));
            }
            "--emit-tsv" => {
                i += 1;
                survey_tsv = Some(PathBuf::from(require_value(&args, i, "--emit-tsv")?));
            }
            "--survey-dir" => {
                i += 1;
                survey_dir = Some(PathBuf::from(require_value(&args, i, "--survey-dir")?));
            }
            other => {
                eprintln!("frontier_probe_survey: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("frontier_probe_survey: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let survey_manifest = survey_manifest.ok_or_else(|| {
        eprintln!("frontier_probe_survey: --emit-survey-manifest required");
        ExitCode::from(2)
    })?;
    let survey_dir = survey_dir.unwrap_or_else(|| {
        survey_manifest
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("target/frontier-probe-survey"))
    });
    fs::create_dir_all(&survey_dir).map_err(|e| {
        eprintln!("frontier_probe_survey: mkdir {:?}: {e}", survey_dir);
        ExitCode::from(1)
    })?;

    let mut module_paths = load_compiler_frontier_sweep_order(&source_roots).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;
    if !only_modules.is_empty() {
        module_paths = only_modules;
    }
    if module_paths.is_empty() {
        eprintln!("frontier_probe_survey: no module paths to probe");
        return Err(ExitCode::from(1));
    }
    if used_module_filter {
        eprintln!(
            "frontier_probe_survey: probing {} module path(s) (--module filter)",
            module_paths.len()
        );
    } else {
        eprintln!(
            "frontier_probe_survey: probing {} module path(s) from {SWEEP_ORDER_DATA}",
            module_paths.len()
        );
    }

    let host_failure_reasons = load_host_failure_reasons(&source_roots).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;

    let mut rows: Vec<ProbeReceiptRow> = Vec::new();
    for module_path in &module_paths {
        eprintln!("frontier_probe_survey: probing {module_path}");
        match run_probe_for_module(&source_roots, &survey_dir, module_path) {
            Ok(row) => {
                eprintln!(
                    "  blocker={} stage={} reason={} chain={} overlap={}",
                    row.blocker_variant,
                    row.located_stage,
                    row.located_reason,
                    row.rejection_chain,
                    row.overlap_roster_detail
                );
                rows.push(row);
            }
            Err(msg) => {
                let cause = if msg.contains("EvalBudgetExceeded") || msg.contains("OOM") {
                    host_failure_reasons.oom_or_budget.as_str()
                } else {
                    host_failure_reasons.runtime_error.as_str()
                };
                eprintln!("frontier_probe_survey: {module_path}: {msg} -> Unknown({cause})");
                rows.push(unknown_probe_row(module_path, cause));
            }
        }
    }

    emit_survey_manifest(&survey_manifest, &rows).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;
    if let Some(tsv_path) = survey_tsv {
        emit_tsv(&tsv_path, &rows).map_err(|msg| {
            eprintln!("frontier_probe_survey: {msg}");
            ExitCode::from(1)
        })?;
        eprintln!("frontier_probe_survey: wrote {}", tsv_path.display());
    }
    let summary = cluster_summary(&rows);
    eprintln!("{summary}");
    if let Some(parent) = survey_manifest.parent() {
        let summary_path = parent.join("frontier_probe_survey_summary.txt");
        let _ = fs::write(&summary_path, &summary);
    }
    eprintln!(
        "frontier_probe_survey: wrote {} ({} receipt(s))",
        survey_manifest.display(),
        rows.len()
    );
    Ok(ExitCode::from(0))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
