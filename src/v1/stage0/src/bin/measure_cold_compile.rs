#![allow(clippy::disallowed_macros)]

//! Cold-compile benchmark receipt bin (bold-crane-271).
//!
//! Runs the whole-tree compile-clean kernel (`compile_clean_whole_tree_hard_diagnostics`)
//! and prints a scannable timing line. The frozen unit order in
//! `docs/probes/data_cold_compile_unit_order.txt` pins sha256(path) tier shape for
//! before/after comparisons on the same tree.
//!
//! Usage:
//!   measure_cold_compile
//!   measure_cold_compile --verify-order docs/probes/data_cold_compile_unit_order.txt

use std::process::ExitCode;

use v1_compiler::cli_run::{
    compile_clean_whole_tree_hard_diagnostics, peak_rss_vhwm_bytes, workspace_root,
};

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
}

fn ctrl_build_mode() -> String {
    std::env::var("CTRL_BUILD_MODE").unwrap_or_else(|_| "unset".to_string())
}

fn verify_order(path: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let mut rows = 0usize;
    for line in content.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((hash, rel)) = line.split_once('\t') else {
            return Err(format!("malformed order row: {line}"));
        };
        if hash.len() != 64 || rel.is_empty() {
            return Err(format!("malformed order row: {line}"));
        }
        rows += 1;
    }
    if rows == 0 {
        return Err(format!("order file {path} has zero data rows"));
    }
    eprintln!("measure_cold_compile: verified order rows={rows}");
    Ok(())
}

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    std::env::remove_var("GITHUB_ACTIONS");
    std::env::remove_var("GUNBC_CI_DIFF_BASE");

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--verify-order" {
        match verify_order(&args[2]) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("COLD_COMPILE_STATUS error");
                eprintln!("COLD_COMPILE_ERROR {e}");
                return ExitCode::from(2);
            }
        }
    }

    eprintln!(
        "measure_cold_compile: box={} ctrl_build_mode={} git={}",
        hostname(),
        ctrl_build_mode(),
        option_env!("GIT_COMMIT").unwrap_or("unknown")
    );
    eprintln!("measure_cold_compile: starting whole-tree compile-clean…");
    let started = std::time::Instant::now();

    match compile_clean_whole_tree_hard_diagnostics() {
        Ok(diags) => {
            let elapsed = started.elapsed();
            println!("COLD_COMPILE_STATUS ok");
            println!("COLD_COMPILE_HARD_DIAGS {}", diags.len());
            println!("COLD_COMPILE_ELAPSED_SECS {:.1}", elapsed.as_secs_f64());
            if let Some(rss) = peak_rss_vhwm_bytes() {
                println!("COLD_COMPILE_RSS_MIB {}", rss / (1024 * 1024));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("COLD_COMPILE_STATUS error");
            eprintln!("COLD_COMPILE_ERROR {e}");
            if let Some(rss) = peak_rss_vhwm_bytes() {
                eprintln!("COLD_COMPILE_RSS_MIB {}", rss / (1024 * 1024));
            }
            ExitCode::from(2)
        }
    }
}
