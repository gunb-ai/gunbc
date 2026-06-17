//! `rust-gate-runner` — SCAFFOLD CI executor for the Rust-conditional gates.
//!
//! `src/v2/workflow/rust_stage0_gates.dag` is the *selection authority*: it decides which
//! changed paths belong to the Rust monolith (`path_is_rust_monolith`). This binary REALIZES
//! that model's clippy/fmt dependents — it asks the selector (per changed path, String in /
//! Bool out, evaluated in-interpreter so the matching logic is never re-implemented here) and,
//! if any path is Rust, runs the two host gates:
//!
//!   - `cargo fmt --all --check`
//!   - `cargo clippy --all-targets -- -D warnings`
//!
//! The in-process witness gate (`ci-claim-gate`) is the ALWAYS-ON CI floor and is NOT run
//! here — this binary only adds the two gates that are conditional on Rust changing.
//!
//! FAIL-CLOSED BIAS (§5): if the diff cannot be determined (git error or empty result), the
//! gates run anyway — under-firing ships broken Rust silently; over-firing only costs CI time.
//!
//! Dissolution trigger: delete when the Rust seed reaches zero (DESIGN §7).
//!
//! Exit codes: 0 = gates passed or skipped (no Rust in diff); 1 = a gate failed;
//! 2 = usage / selector-transport error.

#![allow(clippy::disallowed_macros)]

use std::process::{Command, ExitCode};

use v1_compiler::cli_run::{build_multi_entry_index, make_eval_context, resolve_entry_with_index};
use v1_compiler::v1_interpreter::{run_in_context_with_args, Value};

struct Config {
    source_roots: Vec<String>,
    gate_entry: String,
    predicate_fn: String,
    changed_paths: Vec<String>,
    from_git: bool,
    base: String,
    plan_only: bool,
}

fn usage() -> ! {
    eprintln!(
        "usage: rust-gate-runner [--source-root <dir> ...] [--gate-entry <file>] \\\n\
         \x20       [--predicate-fn <name>] [--plan-only] \\\n\
         \x20       ( --changed-path <p> [--changed-path <p> ...] \\\n\
         \x20       | --changed-paths-from-git [--base <ref>] )"
    );
    std::process::exit(2);
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut gate_entry: Option<String> = None;
    let mut predicate_fn: Option<String> = None;
    let mut changed_paths: Vec<String> = Vec::new();
    let mut from_git = false;
    let mut base: Option<String> = None;
    let mut plan_only = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--gate-entry" => {
                i += 1;
                gate_entry = Some(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--predicate-fn" => {
                i += 1;
                predicate_fn = Some(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--changed-path" => {
                i += 1;
                changed_paths.push(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--changed-paths-from-git" => from_git = true,
            "--base" => {
                i += 1;
                base = Some(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--plan-only" => plan_only = true,
            other => {
                eprintln!("rust-gate-runner: unknown argument: {other}");
                usage();
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        source_roots.push("src/v2".to_string());
    }

    Config {
        source_roots,
        gate_entry: gate_entry
            .unwrap_or_else(|| "src/v2/workflow/rust_stage0_gates.dag".to_string()),
        predicate_fn: predicate_fn.unwrap_or_else(|| "path_is_rust_monolith".to_string()),
        changed_paths,
        from_git,
        base: base.unwrap_or_else(|| "origin/main".to_string()),
        plan_only,
    }
}

/// `Some(paths)` if the diff was determined, `None` if it could not be (spawn error,
/// non-zero git status, or empty output). The caller treats `None` as fail-closed.
fn changed_paths_from_git(base: &str) -> Option<Vec<String>> {
    let out = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let paths: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

/// Ask the `.dag` selector whether any changed path belongs to the Rust monolith. The
/// selector is the authority — this never re-implements the matching. `Err(code)` on a
/// selector-transport error (exit 2).
fn rust_touched_via_selector(cfg: &Config, paths: &[String]) -> Result<bool, ExitCode> {
    let index = build_multi_entry_index(&cfg.source_roots);
    let (graph, si) = match resolve_entry_with_index(&index, &cfg.gate_entry) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("rust-gate-runner: resolve {}: {e}", cfg.gate_entry);
            return Err(ExitCode::from(2));
        }
    };
    let ctx = make_eval_context(&graph, si);
    for p in paths {
        match run_in_context_with_args(
            &ctx,
            &cfg.predicate_fn,
            &[(Some("path".to_string()), Value::Str(p.clone()))],
            false,
        ) {
            Ok(Value::Bool(true)) => return Ok(true),
            Ok(Value::Bool(false)) => {}
            Ok(other) => {
                eprintln!(
                    "rust-gate-runner: {} returned `{other}`, expected Bool",
                    cfg.predicate_fn
                );
                return Err(ExitCode::from(2));
            }
            Err(e) => {
                eprintln!("rust-gate-runner: {e}");
                return Err(ExitCode::from(2));
            }
        }
    }
    Ok(false)
}

/// Run both Rust gates, fail-closed, without short-circuiting — so a single CI run reports
/// every failing gate, not just the first.
fn run_gates() -> ExitCode {
    let gates: [(&str, &[&str]); 2] = [
        ("cargo fmt --all --check", &["fmt", "--all", "--check"]),
        (
            "cargo clippy --all-targets -- -D warnings",
            &["clippy", "--all-targets", "--", "-D", "warnings"],
        ),
    ];
    let mut failures: Vec<&str> = Vec::new();
    for (name, args) in gates {
        println!("rust-gate-runner: running {name}");
        match Command::new("cargo").args(args).status() {
            Ok(s) if s.success() => {}
            Ok(_) => failures.push(name),
            Err(e) => {
                eprintln!("rust-gate-runner: failed to spawn `{name}`: {e}");
                failures.push(name);
            }
        }
    }
    if failures.is_empty() {
        println!("rust-gate-runner: all Rust gates passed (fmt, clippy)");
        ExitCode::SUCCESS
    } else {
        for name in &failures {
            eprintln!("::error::rust gate failed: {name}");
        }
        ExitCode::from(1)
    }
}

fn main() -> ExitCode {
    let cfg = parse_args();

    let (paths, fail_closed): (Vec<String>, bool) = if !cfg.changed_paths.is_empty() {
        (cfg.changed_paths.clone(), false)
    } else if cfg.from_git {
        match changed_paths_from_git(&cfg.base) {
            Some(p) => (p, false),
            None => {
                println!(
                    "rust-gate-runner: could not determine changed paths (git error or empty) — running gates fail-closed"
                );
                (Vec::new(), true)
            }
        }
    } else {
        eprintln!("rust-gate-runner: provide --changed-path <p> or --changed-paths-from-git");
        usage();
    };

    let rust_touched = if fail_closed {
        true
    } else {
        match rust_touched_via_selector(&cfg, &paths) {
            Ok(t) => t,
            Err(code) => return code,
        }
    };

    if !rust_touched {
        println!(
            "rust-gate-runner: no Rust-monolith path in diff — clippy/fmt skipped (in-process witness gate is the always-on floor)"
        );
        return ExitCode::SUCCESS;
    }

    if cfg.plan_only {
        println!(
            "rust-gate-runner: PLAN — cargo fmt --all --check; cargo clippy --all-targets -- -D warnings"
        );
        return ExitCode::SUCCESS;
    }

    run_gates()
}
