#![allow(clippy::disallowed_macros)]

// CI2-0 Commit B sweep driver (operator mandate 2026-08-08; convergence mandate 2026-08-09):
// per roster entry, discover the import closure with the executor's OWN
// discover_source_root_reads_for_entry (never a second discovery), write the same
// host_source_root_ingest_manifest overlay the frontier probe survey rides, and evaluate the
// .dag-side sweep probe (src/v2/test/claim/long/realization_sweep_probe_entry.dag —
// realization_sweep_entry_receipt) so the canonical assembly, exactly-one identity join,
// infer, and translate all run IN the substrate. The host's role is transport + aggregation
// only: per-identity rows come back as TSV, and the phase-and-cause histogram is counted
// across entries. A host-side failure is a typed row (phase=host, located cause), never a
// skipped entry — the roster in equals the roster accounted.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use v1_compiler::cli_run::{
    discover_floor_witness_roster, discover_source_root_reads_for_entry,
    emit_source_ref_dag_for_path, emit_source_root_ingest_manifest,
    parse_source_root_entry_admission, resolve_entry_graph,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

const PROBE_ENTRY: &str = "src/v2/test/claim/long/realization_sweep_probe_entry.dag";
const PROBE_RECEIPT_FN: &str = "realization_sweep_entry_receipt";
const MEMBER_SCAN_FN: &str = "realization_sweep_member_scan_receipt";
const WITNESS_LAYER_ROOTS: &[&str] = &["dag", "src/v2"];
const DEFAULT_SURVEY_DIR: &str = "target/realization-sweep";
// Cites gunbc.ci_layer_roots `witness_discovery_scan_dirs` — the enrolled discovery homes the
// floor's corpus batch scans. A drift here surfaces as a canonical/output mismatch refusal in
// the survey verdict, never a silent narrowing.
const DISCOVERY_SCAN_DIRS: &[&str] = &[
    "dag/test/claim",
    "src/v2/test/claim/manual",
    "src/v2/test/claim/emit",
];

// Canonical enrolled roster (operator verdict msg_6305b900 correction 1): identities come from
// the executor's OWN enrolled floor population — entry identity x declaration identity — never a
// source-text line scan beside the execution roster. Enrollment is the UNION of executing
// cadences (discovery exclusion is a cadence fact, not an enrollment fact): the per-PR discovery
// walk PLUS the explicit-consumer keys (ci_layer_roots long-lane rows, wet enrollment, exact
// admissions, commit-roster claims — one authority, `witness_admission_explicit_consumer_keys`),
// mirroring `v2.workflow.realization_sweep` `entry_canonical_identities`.
fn canonical_identities_for(entry_path: &str) -> Result<Vec<String>, String> {
    let roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    let scan_dirs: Vec<String> = DISCOVERY_SCAN_DIRS.iter().map(|d| d.to_string()).collect();
    let excludes = v1_compiler::cli_run::witness_exclusion_substrings();
    let rows = discover_floor_witness_roster(&roots, &scan_dirs, &excludes, &[])?;
    let mut identities: Vec<String> = rows
        .into_iter()
        .filter(|r| r.entry == entry_path)
        .map(|r| r.function)
        .collect();
    for key in v1_compiler::cli_run::witness_admission_explicit_consumer_keys() {
        if let Some((entry, function)) = key.split_once("::") {
            if entry == entry_path && !identities.iter().any(|f| f == function) {
                identities.push(function.to_string());
            }
        }
    }
    Ok(identities)
}

fn identities_dag_list(identities: &[String]) -> String {
    let mut out = String::from("[");
    for (i, f) in identities.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\"", f.replace('\\', "/")));
    }
    out.push(']');
    out
}

fn write_probe_overlay(
    entry_path: &str,
    ingest_manifest: &Path,
    entry_source_ref: &str,
    identities: &[String],
) -> Result<(), String> {
    let escaped = entry_path.replace('\\', "/");
    let ids = identities_dag_list(identities);
    let append = format!(
        "\ndata frontier_probe_entry_module_path: String = \"{escaped}\"\n\
         data frontier_probe_entry_source_ref: SourceRef = {entry_source_ref}\n\
         data realization_sweep_entry_identities: List<String> = {ids}\n"
    );
    let mut body =
        fs::read_to_string(ingest_manifest).map_err(|e| format!("read ingest manifest: {e}"))?;
    if body.contains("frontier_probe_entry_module_path") {
        return Ok(());
    }
    body.push_str(&append);
    fs::write(ingest_manifest, body).map_err(|e| format!("append entry overlay: {e}"))
}

fn overlay_dir_for(survey_dir: &Path, entry_path: &str) -> PathBuf {
    let slug = entry_path.replace('\\', "/").replace('/', "__");
    survey_dir.join(slug)
}

fn sweep_rows_for_entry(
    survey_dir: &Path,
    entry_path: &str,
    receipt_fn: &str,
) -> Result<String, String> {
    let exclude = vec!["host_source_root_ingest_manifest.dag".to_string()];
    let overlay_dir = overlay_dir_for(survey_dir, entry_path);
    fs::create_dir_all(&overlay_dir).map_err(|e| format!("mkdir overlay {overlay_dir:?}: {e}"))?;

    let discover_roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    let records = discover_source_root_reads_for_entry(&discover_roots, entry_path, &exclude)?;
    let entry_source =
        fs::read_to_string(entry_path).map_err(|e| format!("read entry {entry_path}: {e}"))?;
    let admission = parse_source_root_entry_admission(&entry_source)?;
    let ingest_manifest = overlay_dir.join("host_source_root_ingest_manifest.dag");
    emit_source_root_ingest_manifest(&ingest_manifest, &records, Some(&admission))?;
    let entry_source_ref = emit_source_ref_dag_for_path(&records, entry_path)?;
    let identities = canonical_identities_for(entry_path)?;
    write_probe_overlay(entry_path, &ingest_manifest, &entry_source_ref, &identities)?;

    let mut roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    roots.push(overlay_dir.to_string_lossy().into_owned());

    let (graph, source_indices) = resolve_entry_graph(&roots, PROBE_ENTRY)?;
    let ctx = InterpContext::new(&graph, source_indices, ExecutionMode::Hermetic);
    let value =
        v1_interpreter::run_in_context(&ctx, receipt_fn, true).map_err(|e| format!("{e}"))?;
    match value {
        Value::Str(tsv) => Ok(tsv),
        other => Err(format!(
            "receipt fn returned {}, expected String",
            other.type_label_public()
        )),
    }
}

// --scan-members: per closure member of --entry, re-exec this bin as a child process
// (`--entry <member>`), so one member's interpreted-assembly OOM is a counted
// [member-row] host_error line instead of killing the whole scan. The member's own
// module source supplies its admission (parse_source_root_entry_admission), and the
// member's OWN import closure is discovered — the same canonical route entries ride.
fn scan_members(entry_path: &str) -> Result<(), String> {
    let exclude = vec!["host_source_root_ingest_manifest.dag".to_string()];
    let discover_roots: Vec<String> = WITNESS_LAYER_ROOTS.iter().map(|r| r.to_string()).collect();
    let records = discover_source_root_reads_for_entry(&discover_roots, entry_path, &exclude)?;
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    for rec in &records {
        let out = std::process::Command::new(&exe)
            .args(["--entry", &rec.file_path])
            .output()
            .map_err(|e| format!("spawn member {}: {e}", rec.file_path))?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut reported = false;
        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("[sweep-row] ") {
                println!("[member-row] {rest}");
                reported = true;
            }
        }
        if !reported {
            let cause = if out.status.success() {
                "member_scan_no_rows"
            } else {
                "member_scan_child_died"
            };
            println!(
                "[member-row] {}	(member-scan)	host	{}	{}",
                rec.file_path, cause, rec.file_path
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut entries: Vec<String> = Vec::new();
    let mut survey_dir = PathBuf::from(DEFAULT_SURVEY_DIR);
    let mut receipt_fn = PROBE_RECEIPT_FN;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--entry" => {
                if i + 1 >= args.len() {
                    eprintln!("--entry requires a path");
                    return ExitCode::FAILURE;
                }
                entries.push(args[i + 1].clone());
                i += 2;
            }
            "--entries-file" => {
                if i + 1 >= args.len() {
                    eprintln!("--entries-file requires a path");
                    return ExitCode::FAILURE;
                }
                match fs::read_to_string(&args[i + 1]) {
                    Ok(body) => entries.extend(
                        body.lines()
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty() && !l.starts_with('#')),
                    ),
                    Err(e) => {
                        eprintln!("read entries file {}: {e}", args[i + 1]);
                        return ExitCode::FAILURE;
                    }
                }
                i += 2;
            }
            "--member-scan" => {
                receipt_fn = MEMBER_SCAN_FN;
                i += 1;
            }
            "--scan-members" => {
                if i + 1 >= args.len() {
                    eprintln!("--scan-members requires an entry path");
                    return ExitCode::FAILURE;
                }
                return match scan_members(&args[i + 1]) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("scan-members: {e}");
                        ExitCode::FAILURE
                    }
                };
            }
            "--survey-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("--survey-dir requires a path");
                    return ExitCode::FAILURE;
                }
                survey_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    if entries.is_empty() {
        eprintln!(
            "usage: realization_sweep_survey --entry <path> [--entry <path> ...] \
             [--entries-file <path>] [--survey-dir <dir>]"
        );
        return ExitCode::FAILURE;
    }

    let mut histogram: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut located: BTreeMap<String, u64> = BTreeMap::new();
    let mut seen: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut identity_rows: u64 = 0;
    let mut host_error_rows: u64 = 0;
    let mut canonical_total: u64 = 0;
    let mut missing_rows: u64 = 0;
    for entry in &entries {
        let canonical: Vec<String> = if receipt_fn == PROBE_RECEIPT_FN {
            match canonical_identities_for(entry) {
                Ok(ids) => ids,
                Err(e) => {
                    eprintln!("[sweep-host-error] {entry}: canonical roster: {e}");
                    host_error_rows += 1;
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        canonical_total += canonical.len() as u64;
        match sweep_rows_for_entry(&survey_dir, entry, receipt_fn) {
            Ok(tsv) => {
                for line in tsv.lines().filter(|l| !l.is_empty()) {
                    println!("[sweep-row] {line}");
                    let cols: Vec<&str> = line.split('\t').collect();
                    let phase = cols.get(2).unwrap_or(&"").to_string();
                    let cause = cols.get(3).unwrap_or(&"").to_string();
                    let at = cols.get(4).unwrap_or(&"").to_string();
                    if !at.is_empty() {
                        *located.entry(at).or_insert(0) += 1;
                    }
                    let fn_name = cols.get(1).unwrap_or(&"").to_string();
                    if cause == "realization_sweep_attempt_row_missing" {
                        missing_rows += 1;
                    }
                    *seen.entry((entry.clone(), fn_name)).or_insert(0) += 1;
                    *histogram.entry((phase, cause)).or_insert(0) += 1;
                    identity_rows += 1;
                }
            }
            Err(e) => {
                // A host-side failure is a counted row, never a silently skipped entry.
                println!("[sweep-row] {entry}\t\thost\trealization_sweep_host_error");
                eprintln!("[sweep-host-error] {entry}: {e}");
                *histogram
                    .entry((
                        "host".to_string(),
                        "realization_sweep_host_error".to_string(),
                    ))
                    .or_insert(0) += 1;
                host_error_rows += 1;
            }
        }
    }

    let duplicate_rows: u64 = seen.values().filter(|c| **c > 1).map(|c| *c - 1).sum();
    println!(
        "[sweep-summary] entries {} identity_rows {identity_rows} host_error_rows {host_error_rows} canonical_total {canonical_total} duplicate_rows {duplicate_rows} missing_rows {missing_rows}",
        entries.len()
    );
    for ((phase, cause), count) in &histogram {
        println!("[sweep-histogram] {phase}\t{cause}\t{count}");
    }
    for (at, count) in &located {
        println!("[sweep-located] {at}\t{count}");
    }
    // Survey verdict (operator verdict msg_6305b900 correction 3): the receipt above stays for
    // diagnosis, but the VERDICT refuses unless the canonical enrolled population is accounted
    // exactly — a host error absorbed as one row per entry must not green a run whose entry
    // held twenty tests. Member-scan mode is diagnosis-only and carries no accounting verdict.
    if receipt_fn == PROBE_RECEIPT_FN {
        let holds = host_error_rows == 0
            && duplicate_rows == 0
            && missing_rows == 0
            && identity_rows == canonical_total;
        if holds {
            println!("[sweep-verdict] accepted");
            ExitCode::SUCCESS
        } else {
            println!(
                "[sweep-verdict] refused canonical {canonical_total} accounted {identity_rows} host_errors {host_error_rows} duplicates {duplicate_rows} missing {missing_rows}"
            );
            ExitCode::FAILURE
        }
    } else {
        println!("[sweep-verdict] diagnosis-only");
        ExitCode::SUCCESS
    }
}
