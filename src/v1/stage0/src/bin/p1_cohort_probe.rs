#![allow(clippy::disallowed_macros)]

// P1 retention cohort + width-2 crossover harness. Calls the SAME production entrypoint
// `run_discovery_corpus_with_options` that `claim_executor` uses.
//
// Default (no width override): `DiscoveryWidthPolicy::DerivedSchedule` — fleet-equivalent
// path; schedule-retention arms on the width-1 special case inside discovery.
//
// Width-2 crossover overrides (one run per invocation — interleave by alternating env):
//   GUNBC_P1_COHORT_WIDTH=1  → DiscoveryWidthPolicy::Serial (width-1 baseline)
//   GUNBC_P1_COHORT_WIDTH=2  → DiscoveryWidthPolicy::ControlledWidthTwo
//   GUNBC_P1_MATRIX_CELL=A|B|C|D → 2×2 matrix (overrides width + shared-store arm)
//
// Shared typed JSON store (experiment only — see `p1_experimental_arm_shared_typed_store`):
//   GUNBC_P1_SHARED_TYPED_STORE=auto|0|1|private|shared
//
// Roster:
//   GUNBC_P1_COHORT_ROSTER=relative path under workspace (default p1_cohort_roster.txt)
//   GUNBC_P1_COHORT_LIMIT=N trims the roster to first N entries after load
//
// Instrumentation:
//   GUNBC_P1_COHORT_RECEIPT=1 — per-entry lines + periodic heartbeat (auto-on for matrix runs)
//
// Terminal-run wall clock: PASS/REFUSED lines emit `cohort_complete_wall_ms` at the moment
// `run_discovery_corpus_with_options` returns (before heartbeat teardown). Do not compare
// against external `timeout` wrappers or heartbeat `elapsed_ms` — those are probe windows.

use std::io::Write;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use v1_compiler::cli_run::{
    heartbeat_feed_snapshot, p1_cohort_cgroup_memory, p1_experimental_arm_shared_typed_store,
    run_discovery_corpus_with_options, shared_typecheck_store_counters_snapshot,
    typecheck_compute_count, DiscoveryCorpusOptions, DiscoveryWidthPolicy,
    NodeFrontierSelectionMode,
};
use v1_compiler::memory_governor::{leaf_cgroup_dir, read_cgroup_raw, read_cgroup_u64};
use v1_compiler::v1_interpreter::ExecutionMode;

fn cohort_roster_relative_path() -> String {
    std::env::var("GUNBC_P1_COHORT_ROSTER")
        .unwrap_or_else(|_| "src/v1/stage0/src/bin/p1_cohort_roster.txt".to_string())
}

fn load_cohort_paths() -> Result<Vec<String>, String> {
    let rel = cohort_roster_relative_path();
    let text = match std::fs::read_to_string(&rel) {
        Ok(t) => t,
        Err(e) => return Err(format!("read cohort roster {rel}: {e}")),
    };
    let limit = std::env::var("GUNBC_P1_COHORT_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());
    let mut paths: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    if let Some(n) = limit {
        paths.truncate(n);
    }
    if paths.is_empty() {
        return Err(format!("cohort roster {rel} produced zero entries"));
    }
    Ok(paths)
}

fn matrix_cell_label() -> Option<String> {
    let raw = std::env::var("GUNBC_P1_MATRIX_CELL")
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    if raw.is_empty() {
        return None;
    }
    let cell = raw.chars().next()?;
    if matches!(cell, 'A' | 'B' | 'C' | 'D') {
        Some(cell.to_string())
    } else {
        None
    }
}

fn cohort_width_policy() -> Result<DiscoveryWidthPolicy, String> {
    if let Some(cell) = matrix_cell_label() {
        match cell.as_str() {
            "A" | "B" => Ok(DiscoveryWidthPolicy::Serial),
            "C" | "D" => Ok(DiscoveryWidthPolicy::ControlledWidthTwo),
            _ => Err(format!("invalid GUNBC_P1_MATRIX_CELL {cell:?}")),
        }
    } else if let Ok(raw) = std::env::var("GUNBC_P1_COHORT_WIDTH") {
        match raw.trim() {
            "1" => Ok(DiscoveryWidthPolicy::Serial),
            "2" => Ok(DiscoveryWidthPolicy::ControlledWidthTwo),
            other => Err(format!(
                "GUNBC_P1_COHORT_WIDTH must be 1 (serial) or 2 (controlled-width-two); got {other:?}"
            )),
        }
    } else {
        Ok(DiscoveryWidthPolicy::DerivedSchedule)
    }
}

fn cohort_receipt_enabled() -> bool {
    std::env::var("GUNBC_P1_COHORT_RECEIPT")
        .ok()
        .as_deref()
        .map(|v| matches!(v, "1" | "true" | "TRUE"))
        .unwrap_or(false)
        || matrix_cell_label().is_some()
}

fn width_label(width_policy: &DiscoveryWidthPolicy) -> &'static str {
    match width_policy {
        DiscoveryWidthPolicy::Serial => "serial-width-1",
        DiscoveryWidthPolicy::ControlledWidthTwo => "controlled-width-2",
        DiscoveryWidthPolicy::DerivedSchedule => "derived-schedule",
        DiscoveryWidthPolicy::FixedWidth(w) => {
            if *w <= 1 {
                "fixed-width-1"
            } else {
                "fixed-width-n"
            }
        }
    }
}

fn scheduled_width(width_policy: &DiscoveryWidthPolicy) -> usize {
    match width_policy {
        DiscoveryWidthPolicy::Serial => 1,
        DiscoveryWidthPolicy::ControlledWidthTwo => 2,
        DiscoveryWidthPolicy::DerivedSchedule => {
            let hardware_max = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            v1_compiler::derived_realization_schedule::RealizationConcurrency::for_walk(
                hardware_max,
            )
            .map(|s| s.current_target_width())
            .unwrap_or(1)
        }
        DiscoveryWidthPolicy::FixedWidth(w) => *w,
    }
}

fn emit_periodic_heartbeat(elapsed_ms: u64) {
    let dir = leaf_cgroup_dir();
    let counters = shared_typecheck_store_counters_snapshot();
    let typecheck_computes = typecheck_compute_count();
    let feed = heartbeat_feed_snapshot();
    let (cgroup_current, cgroup_peak) = p1_cohort_cgroup_memory();
    let swap = dir
        .as_ref()
        .and_then(|d| read_cgroup_u64(d, "memory.swap.current"));
    let events = dir
        .as_ref()
        .and_then(|d| read_cgroup_raw(d, "memory.events"));
    let progress = match feed {
        Some(f) => format!(
            "batch={} entry={}/{}",
            f.batch_label, f.entry_done, f.entry_total
        ),
        None => "batch=none".to_string(),
    };
    eprintln!(
        "[p1-cohort-heartbeat] elapsed_ms={} {} width_policy shared_store hit={} miss={} encode={} decode={} encode_bytes={} decode_bytes={} lock_wait_ns={} compute_held_ns={} private_fallback={} typecheck_compute={} cgroup_current={} cgroup_peak={} swap_current={} memory.events={}",
        elapsed_ms,
        progress,
        counters.shared_store_hit,
        counters.shared_store_miss,
        counters.shared_store_encode,
        counters.shared_store_decode,
        counters.shared_store_encode_bytes,
        counters.shared_store_decode_bytes,
        counters.shared_store_lock_wait_nanos,
        counters.shared_store_compute_held_nanos,
        counters.private_store_fallback,
        typecheck_computes,
        cgroup_current
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".into()),
        cgroup_peak
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".into()),
        swap.map(|b| b.to_string()).unwrap_or_else(|| "unreadable".into()),
        events.unwrap_or_else(|| "unreadable".into()),
    );
    let _ = std::io::stderr().flush();
}

#[cfg(unix)]
static INTERRUPT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
extern "C" fn on_interrupt_signal(_: libc::c_int) {
    INTERRUPT_REQUESTED.store(true, Ordering::SeqCst);
}

#[cfg(unix)]
fn install_interrupt_flush() {
    static INSTALLED: std::sync::Once = std::sync::Once::new();
    INSTALLED.call_once(|| unsafe {
        libc::signal(libc::SIGINT, on_interrupt_signal as *const () as usize);
        libc::signal(libc::SIGTERM, on_interrupt_signal as *const () as usize);
    });
}

#[cfg(not(unix))]
fn install_interrupt_flush() {}

fn cgroup_terminal_snapshot() -> String {
    let dir = leaf_cgroup_dir();
    let (current, peak) = p1_cohort_cgroup_memory();
    let max = dir.as_ref().and_then(|d| read_cgroup_u64(d, "memory.max"));
    let high = dir.as_ref().and_then(|d| read_cgroup_raw(d, "memory.high"));
    let swap_max = dir
        .as_ref()
        .and_then(|d| read_cgroup_u64(d, "memory.swap.max"));
    let swap_current = dir
        .as_ref()
        .and_then(|d| read_cgroup_u64(d, "memory.swap.current"));
    let events = dir
        .as_ref()
        .and_then(|d| read_cgroup_raw(d, "memory.events"));
    format!(
        "memory.max={} memory.high={} memory.peak={} memory.current={} memory.swap.max={} memory.swap.current={} memory.events={}",
        max.map(|b| b.to_string()).unwrap_or_else(|| "unreadable".into()),
        high.unwrap_or_else(|| "unreadable".into()),
        peak
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".into()),
        current
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".into()),
        swap_max
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".into()),
        swap_current
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unreadable".into()),
        events.unwrap_or_else(|| "unreadable".into()),
    )
}

fn spawn_periodic_heartbeat(stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("p1-cohort-heartbeat".into())
        .spawn(move || {
            let started = Instant::now();
            while !stop.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(30));
                #[cfg(unix)]
                if INTERRUPT_REQUESTED.load(Ordering::Relaxed) {
                    stop.store(true, Ordering::Relaxed);
                    emit_periodic_heartbeat(started.elapsed().as_millis() as u64);
                    break;
                }
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                emit_periodic_heartbeat(started.elapsed().as_millis() as u64);
            }
        })
        .expect("spawn p1-cohort heartbeat")
}

fn main() -> ExitCode {
    let ws = workspace_root();
    std::env::set_current_dir(&ws).expect("chdir to workspace root");

    if cohort_receipt_enabled() {
        std::env::set_var("GUNBC_P1_COHORT_RECEIPT", "1");
    }

    let width_policy = match cohort_width_policy() {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("p1_cohort_probe: refused: {msg}");
            return ExitCode::from(1);
        }
    };
    let width_label = width_label(&width_policy);
    let scheduled = scheduled_width(&width_policy);
    let shared_armed = p1_experimental_arm_shared_typed_store(scheduled);
    let matrix = matrix_cell_label().unwrap_or_else(|| "-".to_string());

    let cohort_paths = match load_cohort_paths() {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("p1_cohort_probe: refused: {msg}");
            return ExitCode::from(1);
        }
    };

    let source_roots = vec![
        ws.join("dag").to_string_lossy().into_owned(),
        ws.join("src/v2").to_string_lossy().into_owned(),
    ];
    let explicit_entries: Vec<(String, String)> = cohort_paths
        .into_iter()
        .map(|rel| (ws.join(&rel).to_string_lossy().into_owned(), String::new()))
        .collect();

    eprintln!(
        "p1_cohort_probe: matrix_cell={} {} explicit cohort entr(y/ies), shared_typed_store={}, roster={} limit={:?}, source_roots={:?}",
        matrix,
        explicit_entries.len(),
        shared_armed,
        cohort_roster_relative_path(),
        std::env::var("GUNBC_P1_COHORT_LIMIT").ok(),
        source_roots,
    );

    let options = DiscoveryCorpusOptions {
        node_frontier_selection: NodeFrontierSelectionMode::Off,
        execution_authority_source_roots: source_roots.clone(),
        explicit_roster_only: true,
        ..Default::default()
    };

    let stop = Arc::new(AtomicBool::new(false));
    install_interrupt_flush();
    let heartbeat = spawn_periodic_heartbeat(Arc::clone(&stop));
    let wall_start = Instant::now();

    let run_result = run_discovery_corpus_with_options(
        &source_roots,
        &[],
        &explicit_entries,
        ExecutionMode::Wet,
        width_policy,
        options,
    );
    let cohort_complete_wall_ms = wall_start.elapsed().as_millis();

    stop.store(true, Ordering::Relaxed);
    let _ = heartbeat.join();
    emit_periodic_heartbeat(wall_start.elapsed().as_millis() as u64);

    match run_result {
        Ok(summary) => {
            let counters = shared_typecheck_store_counters_snapshot();
            eprintln!(
                "p1_cohort_probe: PASS matrix_cell={} width={} shared_typed_store={} cohort_complete_wall_ms={} total={} skipped={} failures={} resolve_ms={:.3} eval_ms={:.3} typecheck_compute={}",
                matrix,
                width_label,
                shared_armed,
                cohort_complete_wall_ms,
                summary.total,
                summary.skipped,
                summary.failures.len(),
                summary.total_resolve_nanos as f64 / 1.0e6,
                summary.total_measured_nanos as f64 / 1.0e6,
                typecheck_compute_count(),
            );
            eprintln!(
                "p1_cohort_probe: shared_store hit={} miss={} encode={} decode={} encode_bytes={} decode_bytes={} lock_wait_ns={} compute_held_ns={} private_fallback={}",
                counters.shared_store_hit,
                counters.shared_store_miss,
                counters.shared_store_encode,
                counters.shared_store_decode,
                counters.shared_store_encode_bytes,
                counters.shared_store_decode_bytes,
                counters.shared_store_lock_wait_nanos,
                counters.shared_store_compute_held_nanos,
                counters.private_store_fallback,
            );
            eprintln!("p1_cohort_probe: cgroup {}", cgroup_terminal_snapshot());
            if summary.failures.is_empty() {
                ExitCode::SUCCESS
            } else {
                for f in &summary.failures {
                    eprintln!("p1_cohort_probe: FAIL {f}");
                }
                ExitCode::from(1)
            }
        }
        Err(msg) => {
            eprintln!(
                "p1_cohort_probe: refused matrix_cell={} width={} cohort_complete_wall_ms={}: {msg}",
                matrix,
                width_label,
                cohort_complete_wall_ms,
            );
            eprintln!("p1_cohort_probe: cgroup {}", cgroup_terminal_snapshot());
            ExitCode::from(1)
        }
    }
}

fn workspace_root() -> std::path::PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .expect("CARGO_MANIFEST_DIR")
        .ancestors()
        .nth(3)
        .expect("workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}
