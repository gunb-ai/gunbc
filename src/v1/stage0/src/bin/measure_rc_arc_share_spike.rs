#![allow(clippy::disallowed_macros)]

//! Rc→Arc share spike — measurement harness (NOT a migration).
//!
//! Measures the **floor worker object**: `build_multi_entry_index` over both roots (`dag`, `src/v2`),
//! then discovery-corpus warm — not per-entry import closures.

use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use v1_compiler::cli_run::{
    build_worker_index_shell, build_worker_index_with_warm_store, new_shared_typecheck_caches,
    peak_rss_vhwm_bytes, populate_multi_entry_index_whole_tree_discovery,
    populate_worker_discovery_index, resolve_entry_with_index_for_discovery_corpus,
    spike_measurement_host_metadata, warm_discovery_on_index,
    whole_tree_resolve_exclusion_substrings, workspace_root,
};
use v1_compiler::index_memory_receipt::{
    emit_host_metadata, emit_im_rc_census, emit_index_memory_receipt, emit_timing_point,
    emit_width_scaling_point, multi_entry_index_memory_receipt,
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

fn run_worker_shell_build(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    eprintln!(
        "measure_rc_arc_share_spike: worker-shell-build roots={}",
        source_roots.len()
    );
    let start = Instant::now();
    let _index = build_worker_index_shell(source_roots);
    emit_timing_point(
        "worker-shell-build",
        start.elapsed().as_millis(),
        peak_rss_vhwm_bytes(),
    );
    Ok(ExitCode::SUCCESS)
}

fn run_worker_discovery_warm(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    eprintln!(
        "measure_rc_arc_share_spike: worker-discovery-warm roots={}",
        source_roots.len()
    );
    let shell_start = Instant::now();
    let _shell = build_worker_index_shell(source_roots);
    emit_timing_point(
        "worker-shell-only",
        shell_start.elapsed().as_millis(),
        peak_rss_vhwm_bytes(),
    );

    let warm_start = Instant::now();
    match populate_worker_discovery_index(source_roots) {
        Ok((index, entries, rows)) => {
            let warm_ms = warm_start.elapsed().as_millis();
            let peak = peak_rss_vhwm_bytes();
            emit_timing_point("worker-discovery-warm", warm_ms, peak);
            eprintln!(
                "[rc-arc-spike] kind=worker-object discovery_entries={entries} discovery_rows={rows} peak_rss_bytes={}",
                peak.map(|b| b.to_string()).unwrap_or_else(|| "unavailable".into())
            );
            emit_index_memory_receipt(&multi_entry_index_memory_receipt(&index, peak));
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!(
                "[rc-arc-spike] kind=blocked measurement=worker-discovery-warm elapsed_ms={} error={e}",
                warm_start.elapsed().as_millis()
            );
            Err(ExitCode::from(2))
        }
    }
}

fn run_whole_tree_fields(
    source_roots: &[String],
    exclude_subpaths: &[String],
) -> Result<ExitCode, ExitCode> {
    eprintln!(
        "measure_rc_arc_share_spike: whole-tree-fields roots={}",
        source_roots.len()
    );
    let start = Instant::now();
    match populate_multi_entry_index_whole_tree_discovery(source_roots, exclude_subpaths) {
        Ok((index, modules_resolved, modules_excluded)) => {
            let peak = peak_rss_vhwm_bytes();
            emit_timing_point(
                "whole-tree-discovery-resolve",
                start.elapsed().as_millis(),
                peak,
            );
            eprintln!(
                "[rc-arc-spike] kind=whole-tree modules_resolved={modules_resolved} modules_excluded={modules_excluded}"
            );
            emit_index_memory_receipt(&multi_entry_index_memory_receipt(&index, peak));
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!(
                "[rc-arc-spike] kind=blocked measurement=whole-tree-fields elapsed_ms={} error={e}",
                start.elapsed().as_millis()
            );
            Err(ExitCode::from(2))
        }
    }
}

fn run_build_vs_attach(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    eprintln!("measure_rc_arc_share_spike: build-vs-attach");

    let store = new_shared_typecheck_caches();
    let warm_start = Instant::now();
    let warm_index = build_worker_index_with_warm_store(source_roots, store.clone());
    let (entries, rows) = warm_discovery_on_index(&warm_index, source_roots).map_err(|e| {
        eprintln!("[rc-arc-spike] kind=blocked measurement=build-vs-attach-warm error={e}");
        ExitCode::from(2)
    })?;
    let warm_ms = warm_start.elapsed().as_millis();
    let warm_peak = peak_rss_vhwm_bytes();
    emit_timing_point("build-vs-attach-warm-shared-index", warm_ms, warm_peak);
    eprintln!(
        "[rc-arc-spike] kind=shared-store-warm discovery_entries={entries} discovery_rows={rows}"
    );

    let cold_shell_start = Instant::now();
    let _cold = build_worker_index_shell(source_roots);
    let cold_shell_ms = cold_shell_start.elapsed().as_millis();
    emit_timing_point(
        "build-vs-attach-cold-shell",
        cold_shell_ms,
        peak_rss_vhwm_bytes(),
    );

    let attach_start = Instant::now();
    let attached = build_worker_index_with_warm_store(source_roots, store);
    let attach_ms = attach_start.elapsed().as_millis();
    emit_timing_point(
        "build-vs-attach-shared-shell",
        attach_ms,
        peak_rss_vhwm_bytes(),
    );

    let sample_entry = v1_compiler::cli_run::discovery_corpus_entry_roster(source_roots)
        .map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: discovery roster failed: {e}");
            ExitCode::from(2)
        })?
        .0
        .into_iter()
        .next()
        .ok_or_else(|| {
            eprintln!("measure_rc_arc_share_spike: discovery roster empty");
            ExitCode::from(2)
        })?;
    let resolve_start = Instant::now();
    resolve_entry_with_index_for_discovery_corpus(&attached, &sample_entry).map_err(|e| {
        eprintln!("measure_rc_arc_share_spike: attached resolve failed: {e}");
        ExitCode::from(2)
    })?;
    let resolve_ms = resolve_start.elapsed().as_millis();
    emit_timing_point(
        "build-vs-attach-shared-resolve-one",
        resolve_ms,
        peak_rss_vhwm_bytes(),
    );

    eprintln!(
        "[rc-arc-spike] kind=build-vs-attach-summary warm_shared_ms={warm_ms} cold_shell_ms={cold_shell_ms} shared_attach_ms={attach_ms} shared_resolve_one_ms={resolve_ms}"
    );
    Ok(ExitCode::SUCCESS)
}

fn run_width_scaling_worker(
    source_roots: &[String],
    max_width: usize,
) -> Result<ExitCode, ExitCode> {
    eprintln!("measure_rc_arc_share_spike: width-scaling-worker 1..={max_width}");
    for width in 1..=max_width {
        eprintln!(
            "measure_rc_arc_share_spike: width-point width={width} object=worker-discovery-warm"
        );
        let roots = Arc::new(source_roots.to_vec());
        let errors = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let barrier = Arc::new(Barrier::new(width));
        let mut handles = Vec::new();
        for worker in 0..width {
            let roots = Arc::clone(&roots);
            let errors = Arc::clone(&errors);
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                barrier.wait();
                if let Err(e) = populate_worker_discovery_index(roots.as_ref()) {
                    errors.lock().unwrap().push(format!("worker {worker}: {e}"));
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
            eprintln!("[rc-arc-spike] kind=blocked measurement=width-scaling-worker width={width}");
            return Err(ExitCode::from(2));
        }
        emit_width_scaling_point(width, peak_rss_vhwm_bytes());
    }
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut mode = String::from("worker-discovery-warm");
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths = whole_tree_resolve_exclusion_substrings();
    let mut max_width = 3usize;

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

    let (hostname, mem_available_kb, cgroup_max_bytes) = spike_measurement_host_metadata();
    emit_host_metadata(&hostname, mem_available_kb, cgroup_max_bytes);

    match mode.as_str() {
        "worker-shell-build" => run_worker_shell_build(&source_roots),
        "worker-discovery-warm" => run_worker_discovery_warm(&source_roots),
        "whole-tree-fields" => run_whole_tree_fields(&source_roots, &exclude_subpaths),
        "build-vs-attach" => run_build_vs_attach(&source_roots),
        "width-scaling-worker" => run_width_scaling_worker(&source_roots, max_width),
        "im-rc-census" => {
            emit_im_rc_census();
            Ok(ExitCode::SUCCESS)
        }
        other => {
            eprintln!(
                "measure_rc_arc_share_spike: unknown --mode {other} (expected worker-shell-build|worker-discovery-warm|whole-tree-fields|build-vs-attach|width-scaling-worker|im-rc-census)"
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
