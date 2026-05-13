// Binary entrypoint may use `eprintln!` for operator-facing diagnostics (disallowed
// in library code per clippy config).
#![allow(clippy::disallowed_macros)]

//! `gunbc-ci` — BinaryShim workflow runner entrypoint (T-WAD Slice 5) plus host
//! utilities used before full dispatch lands.
//!
//! Invocation (GitHub Actions thin shim per `dsl/gunbc/ci_emission.dag`):
//! `gunbc-ci --workflow ci --event <GITHUB_EVENT_PATH>`
//!
//! Subcommands:
//! - `wall-clock-warn-manifest` — print JSONL warn-policy lines projected from
//!   `dsl/gunbc/test_node_wall_clock_ratchet.dag` (R3 gate #102 substrate authority).

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
            "gunbc-ci: BinaryShim dispatch stub (workflow={wf}); gate matrix execution not wired yet."
        );
        return ExitCode::SUCCESS;
    }

    usage();
}
