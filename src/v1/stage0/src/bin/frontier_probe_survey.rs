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

const COMPILER_FRONTIER_MODULE_PATHS: &[&str] = &[
    "src/v2/compiler/parse_engine_hooks.dag",
    "src/v2/compiler/use_site_verdict.dag",
    "src/v2/compiler/discovery_enumeration.dag",
    "src/v2/compiler/self_host.dag",
    "src/v2/compiler/materialization_carriers.dag",
    "src/v2/compiler/07_target_carriers.dag",
    "src/v2/compiler/fold_lowering.dag",
    "src/v2/compiler/03_normalize.dag",
    "src/v2/compiler/03_resolve.dag",
    "src/v2/compiler/03_name_resolve.dag",
    "src/v2/compiler/04_infer.dag",
    "src/v2/compiler/05_eval.dag",
    "src/v2/compiler/06_translate.dag",
    "src/v2/compiler/05_emit.dag",
    "src/v2/compiler/emit_module.dag",
    "src/v2/compiler/05_emit_orchestration.dag",
    "src/v2/compiler/program_partition.dag",
    "src/v2/compiler/emit_semantic_decl.dag",
    "src/v2/compiler/emit_host.dag",
    "src/v2/compiler/emit_produced.dag",
    "src/v2/compiler/02_parse.dag",
    "src/v2/compiler/source_authority.dag",
    "src/v2/compiler/program_assembly.dag",
    "src/v2/compiler/03_ingest.dag",
    "src/v2/compiler/01_tokenize.dag",
    "src/v2/compiler/03_body_producer.dag",
    "src/v2/compiler/00_compile.dag",
];

struct ProbeReceiptRow {
    module_path: String,
    blocker_variant: String,
    located_stage: String,
    located_reason: String,
    probe_error: Option<String>,
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

fn value_symbol_name(ctx: &InterpContext, value: &Value) -> Result<String, String> {
    match value {
        Value::Str(s) => Ok(symbol_to_manifest(s)),
        other => Err(format!(
            "expected symbol String, got {}",
            other.type_label_public()
        )),
    }
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
                let reason_sym = value_symbol_name(ctx, reason)?;
                Ok(format!("{name} {{ reason: {reason_sym} }}"))
            } else {
                Ok(name)
            }
        }
        _ => Err("blocker_class not a variant".to_string()),
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
    Ok(ProbeReceiptRow {
        module_path,
        blocker_variant: blocker_variant_emit(blocker, ctx)?,
        located_stage: variant_name(ctx, stage)?,
        located_reason: value_symbol_name(ctx, reason)?,
        probe_error: None,
    })
}

fn unknown_probe_row(module_path: &str, cause: &str) -> ProbeReceiptRow {
    ProbeReceiptRow {
        module_path: module_path.replace('\\', "/"),
        blocker_variant: format!("UnknownProbeCause {{ reason: ^{cause} }}"),
        located_stage: "ProbeStageAssemble".to_string(),
        located_reason: format!("^{cause}"),
        probe_error: Some(cause.to_string()),
    }
}

fn write_probe_overlay_manifest(module_path: &str, ingest_manifest: &Path) -> Result<(), String> {
    let escaped = module_path.replace('\\', "/");
    let append = format!(
        "\nimport v2.std.text {{ String }}\n\ndata frontier_probe_entry_module_path: String = \"{escaped}\"\n"
    );
    let mut body = fs::read_to_string(ingest_manifest)
        .map_err(|e| format!("read ingest manifest: {e}"))?;
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
    let records =
        discover_source_root_reads_for_entry(&[discover_root], module_path, &exclude)?;
    let entry_source = fs::read_to_string(module_path)
        .map_err(|e| format!("read entry {module_path}: {e}"))?;
    let admission = parse_source_root_entry_admission(&entry_source)?;
    let ingest_manifest = overlay_dir.join("host_source_root_ingest_manifest.dag");
    emit_source_root_ingest_manifest(&ingest_manifest, &records, Some(&admission))?;
    write_probe_overlay_manifest(module_path, &ingest_manifest)?;

    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS
        .iter()
        .map(|r| r.to_string())
        .collect();
    roots.push(overlay_dir.to_string_lossy().into_owned());

    let (graph, source_indices) = resolve_entry_graph(&roots, PROBE_ENTRY)?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value = v1_interpreter::run_in_context(&ctx, PROBE_RECEIPT_FN, true)
        .map_err(|e| format!("{e}"))?;
    extract_probe_receipt(&value, &ctx)
}

fn emit_receipt_row(row: &ProbeReceiptRow) -> String {
    format!(
        "FrontierProbeReceipt {{\n  module_path: \"{}\",\n  blocker_class: {},\n  located_stage: {},\n  located_reason: {}\n}}",
        row.module_path.replace('\\', "/"),
        row.blocker_variant,
        row.located_stage,
        row.located_reason,
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
    writeln!(out, "module\tself_emit_ready\tblocker_class\tlocated_stage\tlocated_reason\tprobe_error")
        .map_err(|e| format!("write tsv header: {e}"))?;
    for row in rows {
        let self_emit = if row.blocker_variant == "SelfEmitReady" {
            "yes"
        } else {
            "no"
        };
        writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.module_path,
            self_emit,
            row.blocker_variant,
            row.located_stage,
            row.located_reason,
            row.probe_error.as_deref().unwrap_or("")
        )
        .map_err(|e| format!("write tsv row: {e}"))?;
    }
    Ok(())
}

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
    let distinct_reasons = by_reason.len();
    let dominant = by_reason.values().max().copied().unwrap_or(0);
    let verdict = if distinct_reasons <= 3 && dominant >= rows.len().saturating_sub(3) {
        "COMMON_ROOT_LIKELY — few distinct located_reason clusters dominate"
    } else if distinct_reasons >= rows.len() / 2 {
        "HETEROGENEOUS_GAPS — many distinct located roots"
    } else {
        "MIXED — moderate clustering; inspect located_reason histogram"
    };
    lines.push(format!("distinct located_reasons: {distinct_reasons}"));
    lines.push(format!("verdict: {verdict}"));
    lines.join("\n")
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut survey_manifest: Option<PathBuf> = None;
    let mut survey_tsv: Option<PathBuf> = None;
    let mut survey_dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
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

    let mut rows: Vec<ProbeReceiptRow> = Vec::new();
    for module_path in COMPILER_FRONTIER_MODULE_PATHS {
        eprintln!("frontier_probe_survey: probing {module_path}");
        match run_probe_for_module(&source_roots, &survey_dir, module_path) {
            Ok(row) => {
                eprintln!(
                    "  blocker={} stage={} reason={}",
                    row.blocker_variant, row.located_stage, row.located_reason
                );
                rows.push(row);
            }
            Err(msg) => {
                let cause = if msg.contains("EvalBudgetExceeded") || msg.contains("OOM") {
                    "frontier_probe_oom_or_budget"
                } else {
                    "frontier_probe_runtime_error"
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
