#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use v1_compiler::cli_run::{
    discover_source_root_reads_for_entry, emit_source_root_ingest_manifest,
    parse_source_root_entry_admission, resolve_entry_graph, run_in_context,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

const PROBE_ENTRY: &str = "src/v2/test/claim/long/compiler_frontier_probe_entry_test.dag";
const PROBE_RECEIPT_FN: &str = "frontier_probe_entry_receipt";

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

fn value_symbol_name(_ctx: &InterpContext, value: &Value) -> Result<String, String> {
    match value {
        Value::Str(s) => Ok(symbol_to_manifest(s)),
        other => Err(format!(
            "expected symbol String, got {}",
            other.type_label_public()
        )),
    }
}

fn variant_name(value: &Value) -> Result<String, String> {
    match value {
        Value::Variant { variant_name, .. } => Ok(variant_name.clone()),
        _ => Err(format!(
            "expected variant, got {}",
            value.type_label_public()
        )),
    }
}

fn record_field<'a>(value: &'a Value, name: &str) -> Result<&'a Value, String> {
    match value {
        Value::Record { fields, .. } => fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, v)| v)
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
            if variant_name == "UnclassifiedProbeReason" {
                let reason = fields
                    .iter()
                    .find(|(n, _)| n == "reason")
                    .map(|(_, v)| v)
                    .ok_or_else(|| "UnclassifiedProbeReason missing reason".to_string())?;
                let reason_sym = value_symbol_name(ctx, reason)?;
                Ok(format!(
                    "UnclassifiedProbeReason {{ reason: {reason_sym} }}"
                ))
            } else {
                Ok(variant_name.clone())
            }
        }
        _ => Err("blocker_class not a variant".to_string()),
    }
}

fn extract_probe_receipt(value: &Value, ctx: &InterpContext) -> Result<ProbeReceiptRow, String> {
    let module_path = match record_field(value, "module_path")? {
        Value::Str(s) => s.clone(),
        other => {
            return Err(format!(
                "module_path not String: {}",
                other.type_label_public()
            ))
        }
    };
    let blocker = record_field(value, "blocker_class")?;
    let stage = record_field(value, "located_stage")?;
    let reason = record_field(value, "located_reason")?;
    Ok(ProbeReceiptRow {
        module_path,
        blocker_variant: blocker_variant_emit(blocker, ctx)?,
        located_stage: variant_name(stage)?,
        located_reason: value_symbol_name(ctx, reason)?,
    })
}

fn write_probe_overlay_manifest(
    survey_dir: &Path,
    module_path: &str,
    ingest_manifest: &Path,
) -> Result<(), String> {
    let escaped = module_path.replace('\\', "/");
    let append = format!(
        "\nimport v2.std.text {{ String }}\n\ndata frontier_probe_entry_module_path: String = \"{escaped}\"\n"
    );
    let mut body = fs::read_to_string(ingest_manifest)
        .map_err(|e| format!("read ingest manifest: {e}"))?;
    body.push_str(&append);
    fs::write(ingest_manifest, body).map_err(|e| format!("append module_path overlay: {e}"))
}

fn run_probe_for_module(
    source_roots: &[String],
    survey_dir: &Path,
    module_path: &str,
) -> Result<ProbeReceiptRow, String> {
    let exclude = vec!["host_source_root_ingest_manifest.dag".to_string()];
    let records = discover_source_root_reads_for_entry(source_roots, module_path, &exclude)?;
    let entry_source = fs::read_to_string(module_path)
        .map_err(|e| format!("read entry {module_path}: {e}"))?;
    let admission = parse_source_root_entry_admission(&entry_source)?;
    let ingest_manifest = survey_dir.join("host_source_root_ingest_manifest.dag");
    emit_source_root_ingest_manifest(&ingest_manifest, &records, Some(&admission))?;
    write_probe_overlay_manifest(survey_dir, module_path, &ingest_manifest)?;

    let mut roots = vec![
        source_roots[0].clone(),
        "dag".to_string(),
        survey_dir.to_string_lossy().into_owned(),
    ];
    if source_roots.len() > 1 {
        roots.extend_from_slice(&source_roots[1..]);
    }

    let (graph, source_indices) = resolve_entry_graph(&roots, PROBE_ENTRY)?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value = run_in_context(&ctx, PROBE_RECEIPT_FN, true).map_err(|e| format!("{e}"))?;
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
        "// GENERATED by frontier_probe_survey — ephemeral host transport. DO NOT COMMIT.\n\
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

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut survey_manifest: Option<PathBuf> = None;
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
                eprintln!("frontier_probe_survey: {module_path}: {msg}");
                return Err(ExitCode::from(1));
            }
        }
    }

    emit_survey_manifest(&survey_manifest, &rows).map_err(|msg| {
        eprintln!("frontier_probe_survey: {msg}");
        ExitCode::from(1)
    })?;
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
