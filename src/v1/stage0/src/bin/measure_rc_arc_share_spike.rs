#![allow(clippy::disallowed_macros)]

//! Rc→Arc share spike — measurement harness (NOT a migration).
//!
//! Measures the **floor worker object**: `build_multi_entry_index` over both roots (`dag`, `src/v2`),
//! then discovery-corpus warm — not per-entry import closures.

use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use v1_compiler::cli_run::{
    build_worker_index_shell, build_worker_index_with_warm_store, export_typed_cache_for_spike,
    new_shared_typecheck_caches, peak_rss_vhwm_bytes,
    populate_multi_entry_index_whole_tree_discovery, populate_worker_discovery_index,
    resolve_entry_with_index_for_discovery_corpus, spike_measurement_host_metadata,
    spike_set_module_typecheck_recording, spike_take_module_typecheck_records,
    warm_discovery_on_index, whole_tree_resolve_exclusion_substrings, workspace_root,
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

use v1_compiler::shared_typecheck_store::SharedTypecheckCaches;

fn module_grain_out_dir() -> std::path::PathBuf {
    workspace_root().join("target/rc-arc-spike-module-grain")
}

fn run_module_grain_produce(
    source_roots: &[String],
    sample_size: usize,
) -> Result<ExitCode, ExitCode> {
    eprintln!("measure_rc_arc_share_spike: module-grain-produce sample_size={sample_size}");
    spike_set_module_typecheck_recording(true);
    let warm_start = Instant::now();
    let (index, entries, rows) = populate_worker_discovery_index(source_roots).map_err(|e| {
        eprintln!("[rc-arc-spike] kind=blocked measurement=module-grain-produce error={e}");
        ExitCode::from(2)
    })?;
    let warm_ms = warm_start.elapsed().as_millis();
    emit_timing_point("module-grain-warm", warm_ms, peak_rss_vhwm_bytes());

    let records = spike_take_module_typecheck_records();
    let cache = export_typed_cache_for_spike(&index);
    drop(index);

    const MAX_SNAPSHOT_BYTES: usize = 50_000_000;
    const MAX_TYPECHECK_NS: u64 = 500_000_000; // 500ms — skip outlier modules whose serde payloads OOM
    let mut tc_by_key: std::collections::HashMap<String, (String, u64)> =
        std::collections::HashMap::new();
    for (mod_name, typed_key, ns) in records {
        if ns > 0 && ns <= MAX_TYPECHECK_NS {
            tc_by_key.insert(typed_key, (mod_name, ns));
        }
    }

    let mut keys: Vec<(String, String, u64)> = tc_by_key
        .into_iter()
        .map(|(k, (m, ns))| (k, m, ns))
        .collect();
    keys.sort_by(|a, b| a.2.cmp(&b.2)); // ascending — typical modules first

    let out_dir = module_grain_out_dir();
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).map_err(|e| {
        eprintln!("measure_rc_arc_share_spike: cannot create {out_dir:?}: {e}");
        ExitCode::from(2)
    })?;

    let manifest_path = out_dir.join("manifest.jsonl");
    let mut manifest = std::fs::File::create(&manifest_path).map_err(|e| {
        eprintln!("measure_rc_arc_share_spike: create manifest: {e}");
        ExitCode::from(2)
    })?;
    use std::io::Write;

    let mut written = 0usize;
    let mut skipped_large = 0usize;
    let mut skipped_missing = 0usize;
    for (typed_key, mod_name, typecheck_ns) in keys {
        if written >= sample_size {
            break;
        }
        let Some(result) = cache.get(&typed_key) else {
            skipped_missing += 1;
            continue;
        };
        let Some(bytes) =
            SharedTypecheckCaches::try_encode_typed_snapshot(result.as_ref(), MAX_SNAPSHOT_BYTES)
                .map_err(|e| {
                eprintln!("measure_rc_arc_share_spike: encode failed for {mod_name}: {e}");
                ExitCode::from(2)
            })?
        else {
            skipped_large += 1;
            eprintln!(
                "[rc-arc-spike] kind=module-grain-skip mod_name={mod_name} reason=snapshot_bytes>{MAX_SNAPSHOT_BYTES}"
            );
            continue;
        };
        let snap_path = out_dir.join(format!("{written}.bin"));
        std::fs::write(&snap_path, bytes.as_slice()).map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: write {:?}: {e}", snap_path);
            ExitCode::from(2)
        })?;
        let line = format!(
            "{{\"typed_key\":{},\"mod_name\":{},\"typecheck_ns\":{typecheck_ns},\"snapshot_path\":{}}}\n",
            serde_json::to_string(&typed_key).unwrap(),
            serde_json::to_string(&mod_name).unwrap(),
            serde_json::to_string(snap_path.to_string_lossy().as_ref()).unwrap(),
        );
        manifest
            .write_all(line.as_bytes())
            .map_err(|_| ExitCode::from(2))?;
        written += 1;
    }
    drop(cache);

    if written == 0 {
        eprintln!("[rc-arc-spike] kind=blocked measurement=module-grain-produce encoded=0");
        return Err(ExitCode::from(2));
    }

    eprintln!(
        "[rc-arc-spike] kind=module-grain-produce discovery_entries={entries} discovery_rows={rows} sampled_modules={written} skipped_large={skipped_large} skipped_missing={skipped_missing} manifest={}",
        manifest_path.display()
    );

    let exe = std::env::current_exe().map_err(|_| ExitCode::from(2))?;
    let decode_out = Command::new(exe)
        .args([
            "--mode",
            "module-grain-decode",
            "--manifest",
            &manifest_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: decode subprocess failed: {e}");
            ExitCode::from(2)
        })?;
    let stderr = String::from_utf8_lossy(&decode_out.stderr);
    print!("{stderr}");
    if !decode_out.status.success() {
        eprintln!(
            "[rc-arc-spike] kind=blocked measurement=module-grain-decode exit={}",
            decode_out.status
        );
        return Err(ExitCode::from(2));
    }
    Ok(ExitCode::SUCCESS)
}

fn run_module_grain_decode(manifest_path: &str) -> Result<ExitCode, ExitCode> {
    eprintln!("measure_rc_arc_share_spike: module-grain-decode manifest={manifest_path}");
    let content = std::fs::read_to_string(manifest_path).map_err(|e| {
        eprintln!("measure_rc_arc_share_spike: read manifest: {e}");
        ExitCode::from(2)
    })?;

    let mut typecheck_total_ns = 0u64;
    let mut decode_total_ns = 0u64;
    let mut modules = 0usize;
    let mut missing_typecheck = 0usize;

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: serde_json::Value = serde_json::from_str(line).map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: bad manifest line: {e}");
            ExitCode::from(2)
        })?;
        let mod_name = row["mod_name"].as_str().unwrap_or("?");
        let typecheck_ns = row["typecheck_ns"].as_u64().unwrap_or(0);
        let snap_path = row["snapshot_path"].as_str().ok_or_else(|| {
            eprintln!("measure_rc_arc_share_spike: manifest missing snapshot_path");
            ExitCode::from(2)
        })?;
        let bytes = std::fs::read(snap_path).map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: read snapshot {snap_path}: {e}");
            ExitCode::from(2)
        })?;
        let decode_start = Instant::now();
        SharedTypecheckCaches::decode_typed_snapshot(&bytes).map_err(|e| {
            eprintln!("measure_rc_arc_share_spike: decode {mod_name}: {e}");
            ExitCode::from(2)
        })?;
        let decode_ns = decode_start.elapsed().as_nanos() as u64;
        eprintln!(
            "[rc-arc-spike] kind=module-grain-row mod_name={mod_name} typecheck_ns={typecheck_ns} decode_ns={decode_ns}"
        );
        if typecheck_ns == 0 {
            missing_typecheck += 1;
        }
        typecheck_total_ns += typecheck_ns;
        decode_total_ns += decode_ns;
        modules += 1;
    }

    if modules == 0 {
        eprintln!("[rc-arc-spike] kind=blocked measurement=module-grain-decode modules=0");
        return Err(ExitCode::from(2));
    }

    let typecheck_ms_per_module = (typecheck_total_ns as f64) / (modules as f64) / 1_000_000.0;
    let decode_ms_per_module = (decode_total_ns as f64) / (modules as f64) / 1_000_000.0;
    let ratio = if decode_total_ns > 0 {
        typecheck_total_ns as f64 / decode_total_ns as f64
    } else {
        0.0
    };
    eprintln!(
        "[rc-arc-spike] kind=module-grain-summary modules={modules} missing_typecheck_ns={missing_typecheck} typecheck_ms_per_module={typecheck_ms_per_module:.3} decode_ms_per_module={decode_ms_per_module:.3} typecheck_to_decode_ratio={ratio:.2}"
    );
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut mode = String::from("worker-discovery-warm");
    let mut source_roots: Vec<String> = Vec::new();
    let mut exclude_subpaths = whole_tree_resolve_exclusion_substrings();
    let mut max_width = 3usize;
    let mut sample_size = 200usize;
    let mut manifest_path = String::new();

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
            "--sample-size" => {
                i += 1;
                sample_size =
                    parse_usize("--sample-size", &require_value(&args, i, "--sample-size")?)?;
            }
            "--manifest" => {
                i += 1;
                manifest_path = require_value(&args, i, "--manifest")?;
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
        "module-grain-produce" => run_module_grain_produce(&source_roots, sample_size),
        "module-grain-decode" => {
            if manifest_path.is_empty() {
                eprintln!(
                    "measure_rc_arc_share_spike: --manifest required for module-grain-decode"
                );
                return Err(ExitCode::from(2));
            }
            run_module_grain_decode(&manifest_path)
        }
        "im-rc-census" => {
            emit_im_rc_census();
            Ok(ExitCode::SUCCESS)
        }
        other => {
            eprintln!(
                "measure_rc_arc_share_spike: unknown --mode {other} (expected worker-shell-build|worker-discovery-warm|whole-tree-fields|build-vs-attach|width-scaling-worker|module-grain-produce|module-grain-decode|im-rc-census)"
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
