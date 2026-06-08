// Binary entrypoint may use `eprintln!` for operator-facing diagnostics (disallowed
// in library code per clippy config).
#![allow(clippy::disallowed_macros)]

//! `gunbc-ci` — BinaryShim workflow runner entrypoint (T-WAD Slice 6) plus host
//! utilities used before full dispatch lands.
//!
//! Invocation (GitHub Actions thin shim per `dsl/gunbc/ci_emission.dag`):
//! `gunbc-ci --workflow ci --event <GITHUB_EVENT_PATH>`
//!
//! Subcommands:
//! - `wall-clock-warn-manifest` — print JSONL warn-policy lines projected from
//!   `dsl/gunbc/test_node_wall_clock_ratchet.dag` (**interim bridge** toward gate
//!   **#102**; canonical pass target is policy from `TestNodeCostDimension` timing
//!   facts — not a parallel warn-token table).
//!
//! **`--workflow` / `--event` dispatch:** not implemented yet (BinaryShim gate-matrix
//! wiring is pending). That path exits **2** fail-closed unless
//! `GUNBC_CI_ALLOW_DISPATCH_STUB=1` (or `true`) is set so callers can smoke the
//! binary without implying dispatch succeeded.

use std::path::PathBuf;
use std::process::ExitCode;

use v3_compiler::compile_to_dag;
use v3_compiler::wall_clock_ratchet_manifest::{
    emit_warn_policy_jsonl_lines, RATCHET_DAG_REL_PATH,
};

fn usage() -> ! {
    eprintln!(
        "usage:\n  gunbc-ci wall-clock-warn-manifest\n  gunbc-ci --workflow <name> --event <github_event.json>"
    );
    std::process::exit(2);
}

fn repo_root() -> PathBuf {
    if let Ok(p) = std::env::var("GITHUB_WORKSPACE") {
        return PathBuf::from(p);
    }
    std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("cwd: {e}");
        std::process::exit(2);
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }

    if args[0] == "wall-clock-warn-manifest" {
        if args.len() != 1 {
            eprintln!("wall-clock-warn-manifest: unexpected arguments");
            return ExitCode::from(2);
        }
        let root = repo_root();
        let path = root.join(RATCHET_DAG_REL_PATH);
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        };
        let dag = match compile_to_dag(&source, path.to_string_lossy().as_ref()) {
            Ok(d) => d,
            Err(err) => {
                eprintln!("compile: {err:?}");
                return ExitCode::FAILURE;
            }
        };
        return match emit_warn_policy_jsonl_lines(&dag) {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
                ExitCode::SUCCESS
            }
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::FAILURE
            }
        };
    }

    if args.len() == 4 && args[0] == "--workflow" && args[2] == "--event" {
        let wf = args[1].as_str();
        let event_path = args[3].as_str();
        if wf != "ci" {
            eprintln!("unsupported workflow: {wf}");
            return ExitCode::from(2);
        }
        if !std::path::Path::new(event_path).is_file() {
            eprintln!("--event path is not a readable file: {event_path}");
            return ExitCode::from(2);
        }
        eprintln!(
            "gunbc-ci: BinaryShim dispatch stub (workflow={wf}); run-all (Phase A3) not wired yet."
        );
        let allow_stub = std::env::var("GUNBC_CI_ALLOW_DISPATCH_STUB")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if allow_stub {
            return ExitCode::SUCCESS;
        }
        // A3 run-all is blocked on two substrate gaps — see
        // docs/planning/thin-shim-ci-a3-runall-blocker-report-2026-06-08.md.
        // Both are Mgr-C/operator-gated (model edits + loader); not fabricated here.
        eprintln!(
            "gunbc-ci: refusing success for unimplemented dispatch (exit 2). Phase A3 run-all needs:\n\
             \x20 (B1) a cross-module v4 runtime loader reachable by this runner — frozen-v3 \
             single-source `compile_to_dag` cannot load `src/v4/workflow/ci.dag` + its 28-import \
             closure (v4 load is the v2 `dag run` path, not linked here);\n\
             \x20 (B2) per-job runnable command authority IN the model — `CiCommand` arms are \
             abstract tags; the shell lives in ci.yml/ci-floor, and a Rust dispatch table would be \
             a banned dual-authority. See the A3 blocker report.\n\
             Set GUNBC_CI_ALLOW_DISPATCH_STUB=1 to smoke this stub until dispatch lands."
        );
        return ExitCode::from(2);
    }

    usage();
}
