#![allow(clippy::disallowed_macros)]

//! Owned CI control plane v0 host transport.
//!
//! Poll-first subject discovery, durable queue, one local executor, Checks projection.
//! See `docs/plans/owned-ci-control-plane-design.md`.

use std::process::ExitCode;

use clap::Parser;
use v1_compiler::ci_control_plane::{CiControlPlane, Config};
use v1_compiler::cli_run::workspace_root;

#[derive(Parser, Debug)]
#[command(name = "ci_control_plane", about = "Owned CI control plane v0")]
struct Args {
    /// Run one poll/queue/execute cycle and exit.
    #[arg(long)]
    once: bool,

    /// Discover/enqueue from mirror; skip GitHub REST (PR poll, Checks write) and executor stages.
    #[arg(long)]
    dry_run: bool,

    /// Enqueue an operator rerun for this commit SHA (v0 explicit rerun surface).
    #[arg(long)]
    enqueue_rerun_sha: Option<String>,

    /// Workspace root (defaults to gunbc workspace_root()).
    #[arg(long)]
    workspace_root: Option<String>,

    /// Bare git mirror path.
    #[arg(long)]
    mirror: Option<String>,

    /// Base URL for /ci/run/{id} details_url (Checks projection).
    #[arg(long)]
    serve_base_url: Option<String>,

    /// Poll interval seconds between cycles.
    #[arg(long, default_value_t = 60)]
    poll_interval_secs: u64,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mut config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ci_control_plane: config refused: {e}");
            return ExitCode::from(2);
        }
    };
    if let Some(root) = args.workspace_root {
        config.workspace_root = root.into();
    } else {
        config.workspace_root = workspace_root();
    }
    if let Some(mirror) = args.mirror {
        config.mirror_path = mirror.into();
    }
    if let Some(url) = args.serve_base_url {
        config.serve_base_url = url;
    }
    config.once = config.once || args.once;
    config.dry_run = config.dry_run || args.dry_run;
    if let Some(rerun_sha) = args.enqueue_rerun_sha {
        config.enqueue_rerun_sha = Some(rerun_sha);
    }
    config.poll_interval_secs = args.poll_interval_secs;

    if let Err(err) = std::env::set_current_dir(&config.workspace_root) {
        eprintln!("ci_control_plane: chdir refused: {err}");
        return ExitCode::from(2);
    }

    let mut plane = match CiControlPlane::new(config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ci_control_plane: authority load refused: {e}");
            return ExitCode::from(2);
        }
    };
    match plane.run_loop() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ci_control_plane: {e}");
            ExitCode::from(1)
        }
    }
}
