#![allow(clippy::disallowed_macros)]

//! Rc→Arc share spike — measurement harness (NOT a migration).
//!
//! Modes:
//!   index-fields   — per-field bytes on a strict whole-tree resolve through MultiEntryIndex
//!   width-point    — peak RSS with W parallel worker-index simulations (one process)
//!   width-scaling  — width-point for widths 1..=max-width (subprocess isolation per point)
//!   union-sample   — shareable union across sampled witness entry closures
//!   im-rc-census   — im_rc blocker file census (report-only)
//!
//! Every line is `[rc-arc-spike] kind=...` for grep/oracle consumption. Reports numbers,
//! never asserts migration success.

use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

use v1_compiler::cli_run::{
    peak_rss_vhwm_bytes, populate_multi_entry_index_entry, populate_multi_entry_index_whole_tree,
    ResolveTypecheckGate, whole_tree_resolve_exclusion_substrings, workspace_root,
};
use v1_compiler::index_memory_receipt::{
    emit_im_rc_census, emit_index_memory_receipt, emit_net_win_point, emit_width_scaling_point,
    multi_entry_index_memory_receipt, net_win_curve,
};

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("measure_rc_arc_share_spike: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn parse_usize(flag: &str, raw: &str) -> Result<usize, ExitCode> {
    raw.parse::<usize>().map_err(|_| {
        eprintln!("measure_rc_arc_share_spike: {flag} must be a positive integer, got `{raw}`");
        ExitCode::from(2)
    })
}

fn default_source_roots() -> Vec<String> {
    let ws = workspace_root();
    ["dag", "src/v2"]
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect()
}

fn run_index_fields(source_roots: &[String], exclude: &[String]) -> Result<ExitCode, ExitCode> {
    eprintln!(
        "measure_rc_arc_share_spike: index-fields over {} root(s)",
        source_roots.len()
    );
    let (index, modules_resolved, modules_excluded) =
        populate_multi_entry_index_whole_tree(
            source_roots,
            exclude,
            ResolveTypecheckGate::DiscoveryCorpusAdvisory,
        )
        .map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: whole-tree populate failed:\n{e}");
            ExitCode::from(2)
        })?;
    let peak = peak_rss_vhwm_bytes();
    let receipt = multi_entry_index_memory_receipt(&index, peak);
    eprintln!(
        "[rc-arc-spike] kind=corpus modules_resolved={modules_resolved} modules_excluded={modules_excluded} typecheck_gate=DiscoveryCorpusAdvisory"
    );
    emit_index_memory_receipt(&receipt);
    Ok(ExitCode::SUCCESS)
}

fn run_width_point(source_roots: &[String], exclude: &[String], width: usize) -> Result<ExitCode, ExitCode> {
    if width == 0 {
        eprintln!("measure_rc_arc_share_spike: --width must be >= 1");
        return Err(ExitCode::from(2));
    }
    eprintln!("measure_rc_arc_share_spike: width-point width={width}");
    let roots = Arc::new(source_roots.to_vec());
    let exclude = Arc::new(exclude.to_vec());
    let errors: Arc<std::sync::Mutex<Vec<String>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let barrier = Arc::new(Barrier::new(width));
    let mut handles = Vec::new();
    for worker in 0..width {
        let roots = Arc::clone(&roots);
        let exclude = Arc::clone(&exclude);
        let errors = Arc::clone(&errors);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let result = populate_multi_entry_index_whole_tree(
                roots.as_ref(),
                exclude.as_ref(),
                ResolveTypecheckGate::DiscoveryCorpusAdvisory,
            );
            if let Err(e) = result {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("worker {worker}: {e}"));
            }
        }));
    }
    for handle in handles {
        handle.join().map_err(|_| ExitCode::from(2))?;
    }
    let errs = errors.lock().unwrap();
    if !errs.is_empty() {
        for e in errs.iter() {
            eprintln!("measure_rc_arc_share_spike: {e}");
        }
        return Err(ExitCode::from(2));
    }
    emit_width_scaling_point(width, peak_rss_vhwm_bytes());
    Ok(ExitCode::SUCCESS)
}

fn run_width_scaling(
    source_roots: &[String],
    exclude: &[String],
    max_width: usize,
) -> Result<ExitCode, ExitCode> {
    eprintln!("measure_rc_arc_share_spike: width-scaling 1..={max_width}");
    for width in 1..=max_width {
        run_width_point(source_roots, exclude, width)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn sample_witness_entries(source_roots: &[String], sample_count: usize) -> Vec<String> {
    use v1_compiler::cli_run::discover_floor_corpus_rows;
    let scan_dirs = vec!["dag".to_string(), "src/v2".to_string()];
    let excludes = whole_tree_resolve_exclusion_substrings();
    let rows = discover_floor_corpus_rows(source_roots, &scan_dirs, &excludes)
        .unwrap_or_default();
    let mut entries: Vec<String> = rows
        .iter()
        .map(|r| r.entry.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    entries.sort();
    if entries.len() > sample_count {
        let stride = entries.len() / sample_count;
        entries = entries
            .into_iter()
            .enumerate()
            .filter_map(|(i, e)| if i % stride.max(1) == 0 { Some(e) } else { None })
            .take(sample_count)
            .collect();
    }
    entries
}

fn run_union_sample(
    source_roots: &[String],
    sample_count: usize,
    max_width: usize,
) -> Result<ExitCode, ExitCode> {
    let entries = sample_witness_entries(source_roots, sample_count);
    eprintln!(
        "measure_rc_arc_share_spike: union-sample {} entr(y/ies)",
        entries.len()
    );
    let mut receipts = Vec::new();
    for entry in &entries {
        let index = populate_multi_entry_index_entry(source_roots, entry).map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: entry populate failed for {entry}:\n{e}");
            ExitCode::from(2)
        })?;
        receipts.push(multi_entry_index_memory_receipt(&index, None));
    }
    if receipts.is_empty() {
        eprintln!("measure_rc_arc_share_spike: union-sample found zero entries");
        return Err(ExitCode::from(2));
    }
    let shareable_per_worker = receipts.iter().map(|r| r.shareable_bytes).sum::<u64>()
        / receipts.len() as u64;
    let residue_per_worker = receipts.iter().map(|r| r.residue_bytes).sum::<u64>()
        / receipts.len() as u64;
    // Union shareable: walk is expensive; upper bound = sum of per-entry shareable (private W×),
    // lower bound = max (identical prefix). Report both + max for crossover model.
    let union_shareable_sum: u64 = receipts.iter().map(|r| r.shareable_bytes).sum();
    let union_shareable_max = receipts.iter().map(|r| r.shareable_bytes).max().unwrap_or(0);
    eprintln!(
        "[rc-arc-spike] kind=union-sample entries={} shareable_per_worker_avg={} residue_per_worker_avg={} union_shareable_sum={} union_shareable_max={}",
        receipts.len(),
        shareable_per_worker,
        residue_per_worker,
        union_shareable_sum,
        union_shareable_max,
    );
    for point in net_win_curve(
        shareable_per_worker,
        residue_per_worker,
        union_shareable_max,
        max_width,
    ) {
        emit_net_win_point(&point);
    }
    for point in net_win_curve(
        shareable_per_worker,
        residue_per_worker,
        union_shareable_sum,
        max_width,
    ) {
        eprintln!(
            "[rc-arc-spike] kind=net-win-pessimistic width={} private_total_bytes={} shared_model_bytes={} net_win_bytes={}",
            point.width,
            point.private_total_bytes,
            point.shared_model_bytes,
            point.net_win_bytes,
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut mode = String::from("index-fields");
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths = whole_tree_resolve_exclusion_substrings();
    let mut max_width = 4usize;
    let mut width = 1usize;
    let mut sample_count = 9usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                i += 1;
                mode = require_value(&args, i, "--mode")?;
            }
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--exclude-subpath" => {
                i += 1;
                exclude_subpaths.push(require_value(&args, i, "--exclude-subpath")?);
            }
            "--max-width" => {
                i += 1;
                max_width = parse_usize("--max-width", &require_value(&args, i, "--max-width")?)?;
            }
            "--width" => {
                i += 1;
                width = parse_usize("--width", &require_value(&args, i, "--width")?)?;
            }
            "--sample-entries" => {
                i += 1;
                sample_count =
                    parse_usize("--sample-entries", &require_value(&args, i, "--sample-entries")?)?;
            }
            other => {
                eprintln!("measure_rc_arc_share_spike: unknown argument: {other}");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        source_roots = default_source_roots();
    }

    match mode.as_str() {
        "index-fields" => run_index_fields(&source_roots, &exclude_subpaths),
        "width-point" => run_width_point(&source_roots, &exclude_subpaths, width),
        "width-scaling" => run_width_scaling(&source_roots, &exclude_subpaths, max_width),
        "union-sample" => run_union_sample(&source_roots, sample_count, max_width),
        "im-rc-census" => {
            emit_im_rc_census();
            Ok(ExitCode::SUCCESS)
        }
        other => {
            eprintln!(
                "measure_rc_arc_share_spike: unknown --mode {other} (expected index-fields|width-point|width-scaling|union-sample|im-rc-census)"
            );
            Err(ExitCode::from(2))
        }
    }
}

fn main() -> ExitCode {
    static STARTED: AtomicUsize = AtomicUsize::new(0);
    if STARTED.fetch_add(1, Ordering::SeqCst) == 0 {
        eprintln!("measure_rc_arc_share_spike: Rc→Arc share spike harness (measurement only)");
    }
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
