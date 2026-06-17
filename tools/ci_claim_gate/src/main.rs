//! `ci-claim-gate` — uniform `.dag`-driven CI Bool-witness gate host.
//!
//! The CI floor invokes this binary directly (no bash gate script, no `.dag`
//! shell-out). The roster is sourced one of two ways:
//!
//! - `--roster-from-discovery --scan-dir <dir>`: reflection over the discovered
//!   `unified_claim_*` BoolWitness corpus — the dissolution target, no hand-list.
//! - `--gate-entry <dag> --rows-fn <fn>`: legacy modeled `*_rows_tsv()` projection
//!   (retained for per-gate rosters not yet migrated to discovery).
//!
//! Either way it then runs:
//!
//! 1. GREEN pass — one `claim_batch`-style multi-entry resolve (module index once)
//! 2. PERTURB pass (optional) — per-row temp-tree witness body → `false`, must fail
//!
//! Exit codes: 0 = all witnesses passed (+ perturb receipts when requested);
//! 1 = witness failure or perturb did not go red; 2 = usage / transport error.

#![allow(clippy::disallowed_macros)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use v1_compiler::cli_run::{
    build_multi_entry_index, discover_owned_data_decls, make_eval_context,
    resolve_entry_with_index, run_claim, ClaimOutcome, OwnedDataDeclInitializer,
};
use v1_compiler::v1_interpreter::{run_in_context_with_args, Value};

struct GateRow {
    label: String,
    entry: String,
    function: String,
}

/// Roster source: either a modeled `*_rows_tsv()` fn (legacy per-gate hand-list)
/// or reflection over the discovered `unified_claim_*` BoolWitness decls. The
/// discovery path is the dissolution target — no hand-typed roster, no rows-fn.
enum RosterSource {
    RowsFn { gate_entry: String, rows_fn: String },
    Discovery { scan_dirs: Vec<String> },
}

struct Config {
    source_roots: Vec<String>,
    roster: RosterSource,
    perturb: bool,
    print_tsv_only: bool,
    notice_title: String,
    /// Opt-in (`--rust-gates`): after the witness floor passes, run the conditional
    /// Rust-monolith clippy/fmt gates. Off by default (backward compatible).
    rust_gates: bool,
    /// Explicit changed paths for the Rust-gate selector (`--changed-path`, repeatable).
    /// Empty → fall back to `git diff` against `rust_gates_base`.
    rust_gates_changed_paths: Vec<String>,
    /// Merge base for the Rust-gate `git diff` fallback (`--base`, default `origin/main`).
    rust_gates_base: String,
}

// Mirror of discover_owned_data's default exclude set: the manifest/law files that
// import the ephemeral discovery output would otherwise re-enter discovery acyclically.
// Plus the manual lane: `src/v2/test/manual/` claims carry their own
// ExpectPass|ExpectFail expected-outcome (some are pinned-red to tracking anchors,
// e.g. witness_sg2_arrow ExpectFail{#4801}). A universal-green floor must NOT run them
// as must-pass — excluded until expected-outcome is modeled at the claim level.
const DISCOVERY_EXCLUDES: &[&str] = &[
    "impossible_bug",
    "test/manual/",
    "glob_discovery.dag",
    "glob_discovery_law.dag",
    "host_discovered_owned_data_manifest.dag",
    "unified_test_claim_substrate_equivalence.dag",
];

fn usage() -> ! {
    eprintln!(
        "usage: ci-claim-gate --source-root <dir> [--source-root <dir> ...] \\\n\
         \x20       ( --gate-entry <file.dag> --rows-fn <function>          \\\n\
         \x20       | --roster-from-discovery --scan-dir <dir> [--scan-dir <dir> ...] ) \\\n\
         \x20       [--perturb-check] [--print-tsv-only] [--notice-title <title>] \\\n\
         \x20       [--rust-gates [--changed-path <p> ...] [--base <ref>]]"
    );
    std::process::exit(2);
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut source_roots = Vec::new();
    let mut gate_entry = None;
    let mut rows_fn = None;
    let mut roster_from_discovery = false;
    let mut scan_dirs: Vec<String> = Vec::new();
    let mut perturb = false;
    let mut print_tsv_only = false;
    let mut notice_title = "v2 CI claim gate".to_string();
    let mut rust_gates = false;
    let mut rust_gates_changed_paths: Vec<String> = Vec::new();
    let mut rust_gates_base = "origin/main".to_string();

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
            "--rows-fn" => {
                i += 1;
                rows_fn = Some(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--roster-from-discovery" => roster_from_discovery = true,
            "--scan-dir" => {
                i += 1;
                scan_dirs.push(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--perturb-check" => perturb = true,
            "--print-tsv-only" => print_tsv_only = true,
            "--notice-title" => {
                i += 1;
                notice_title = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            "--rust-gates" => rust_gates = true,
            "--changed-path" => {
                i += 1;
                rust_gates_changed_paths.push(args.get(i).cloned().unwrap_or_else(|| usage()));
            }
            "--base" => {
                i += 1;
                rust_gates_base = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            other => {
                eprintln!("ci-claim-gate: unknown argument: {other}");
                usage();
            }
        }
        i += 1;
    }

    let roster = if roster_from_discovery {
        if rows_fn.is_some() || gate_entry.is_some() {
            eprintln!(
                "ci-claim-gate: --roster-from-discovery is exclusive with --gate-entry/--rows-fn"
            );
            usage();
        }
        if scan_dirs.is_empty() {
            eprintln!("ci-claim-gate: --roster-from-discovery requires at least one --scan-dir");
            usage();
        }
        RosterSource::Discovery { scan_dirs }
    } else {
        RosterSource::RowsFn {
            gate_entry: gate_entry.unwrap_or_else(|| {
                eprintln!(
                    "ci-claim-gate: --gate-entry is required (or use --roster-from-discovery)"
                );
                usage();
            }),
            rows_fn: rows_fn.unwrap_or_else(|| {
                eprintln!("ci-claim-gate: --rows-fn is required (or use --roster-from-discovery)");
                usage();
            }),
        }
    };

    Config {
        source_roots,
        roster,
        perturb,
        print_tsv_only,
        notice_title,
        rust_gates,
        rust_gates_changed_paths,
        rust_gates_base,
    }
}

/// Reflection roster: every discovered `unified_claim_*` BoolWitness decl across
/// the scan dirs becomes a gate row (label = decl name minus the `unified_claim_`
/// prefix, entry/function = the modeled witness). No hand-typed roster, no rows-fn.
fn discover_roster(source_roots: &[String], scan_dirs: &[String]) -> Result<Vec<GateRow>, String> {
    let excludes: Vec<String> = DISCOVERY_EXCLUDES.iter().map(|s| s.to_string()).collect();
    let mut rows: Vec<GateRow> = Vec::new();
    let mut seen: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    for scan_dir in scan_dirs {
        let discovery = discover_owned_data_decls(source_roots, scan_dir, &excludes)?;
        for rec in discovery.records {
            if let OwnedDataDeclInitializer::BoolWitnessClaim {
                witness_entry,
                witness_function,
            } = rec.initializer
            {
                if witness_entry.is_empty() || witness_function.is_empty() {
                    return Err(format!(
                        "discovered decl '{}' has malformed BoolWitness transport (entry/function)",
                        rec.decl_name
                    ));
                }
                if seen.insert((witness_entry.clone(), witness_function.clone())) {
                    let label = rec
                        .decl_name
                        .strip_prefix("unified_claim_")
                        .unwrap_or(&rec.decl_name)
                        .to_string();
                    rows.push(GateRow {
                        label,
                        entry: witness_entry,
                        function: witness_function,
                    });
                }
            }
        }
    }

    // Single-representation `test fn NAME()` / `test data NAME` tests. The v2
    // parser drops the contextual `test` keyword, so the gate detects the marker
    // in source text (same posture as the `data unified_claim_` scan) and runs
    // NAME. Dual-mode with the loop above so claims migrate off `unified_claim_*`
    // one at a time.
    //
    // Convention, enforced fail-closed: a `test` declaration may live ONLY in
    // `*_test.dag` files. The whole source root is scanned (so manual/ and
    // implementation files are covered), and a `test` decl found anywhere else is
    // a hard error — tests do not live in implementation files.
    let mut test_fn_violations: Vec<String> = Vec::new();
    for root in source_roots {
        let mut dag_files: Vec<PathBuf> = Vec::new();
        collect_dag_files(Path::new(root), &mut dag_files);
        dag_files.sort();
        for path in dag_files {
            let entry = path.to_string_lossy().into_owned();
            let content =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            let names = scan_test_decl_names(&content);
            if names.is_empty() {
                continue;
            }
            if !entry.ends_with("_test.dag") {
                test_fn_violations.push(entry);
                continue;
            }
            for name in names {
                if seen.insert((entry.clone(), name.clone())) {
                    rows.push(GateRow {
                        label: name.clone(),
                        entry: entry.clone(),
                        function: name,
                    });
                }
            }
        }
    }
    if !test_fn_violations.is_empty() {
        test_fn_violations.sort();
        return Err(format!(
            "`test`-marked tests must live in `*_test.dag` files; found a `test` decl in: {}",
            test_fn_violations.join(", ")
        ));
    }
    rows.sort_by(|a, b| {
        a.entry
            .cmp(&b.entry)
            .then_with(|| a.function.cmp(&b.function))
    });
    Ok(rows)
}

fn evaluate_rows_tsv(
    gate_entry: &str,
    rows_fn: &str,
    source_roots: &[String],
) -> Result<String, String> {
    let index = build_multi_entry_index(source_roots);
    let (graph, si) = resolve_entry_with_index(&index, gate_entry)?;
    let ctx = make_eval_context(&graph, si);
    match run_in_context_with_args(&ctx, rows_fn, &[], false) {
        Ok(Value::Str(s)) => Ok(s),
        Ok(other) => Err(format!(
            "{rows_fn} returned `{other}`, expected a String TSV of label\\tentry\\tfunction rows"
        )),
        Err(e) => Err(format!("{e}")),
    }
}

fn parse_rows_tsv(tsv: &str) -> Vec<GateRow> {
    tsv.lines()
        .filter_map(|line| {
            let mut it = line.split('\t');
            let label = it.next()?.trim();
            let entry = it.next()?.trim();
            let function = it.next()?.trim();
            if label.is_empty() || entry.is_empty() || function.is_empty() {
                return None;
            }
            Some(GateRow {
                label: label.to_string(),
                entry: entry.to_string(),
                function: function.to_string(),
            })
        })
        .collect()
}

fn run_green_pass(source_roots: &[String], rows: &[GateRow]) -> Result<bool, String> {
    let index = build_multi_entry_index(source_roots);
    let mut by_entry: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for row in rows {
        by_entry
            .entry(row.entry.as_str())
            .or_default()
            .push(row.function.as_str());
    }

    let mut any_failed = false;
    for (entry, functions) in &by_entry {
        let (graph, si) = resolve_entry_with_index(&index, entry)?;
        let ctx = make_eval_context(&graph, si);
        for function in functions {
            match run_claim(&ctx, function) {
                ClaimOutcome::Pass => println!("PASS {function}"),
                ClaimOutcome::Fail => {
                    println!("FAIL {function}");
                    any_failed = true;
                }
                ClaimOutcome::NotBool { got } => {
                    println!("FAIL {function} (returned `{got}`, not Bool)");
                    any_failed = true;
                }
                ClaimOutcome::RuntimeError { message } => {
                    println!("FAIL {function} (runtime error: {message})");
                    any_failed = true;
                }
            }
        }
    }
    Ok(!any_failed)
}

fn perturb_function_to_false(path: &Path, function: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let needle = format!("fn {function}(");
    let start = text
        .find(&needle)
        .ok_or_else(|| format!("{}: missing function {function}", path.display()))?;
    let brace = text[start..]
        .find('{')
        .ok_or_else(|| format!("{}: missing body for {function}", path.display()))?;
    let brace = start + brace;
    let mut depth = 0;
    let mut end = None;
    for (i, ch) in text[brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(brace + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.ok_or_else(|| format!("{}: unterminated body for {function}", path.display()))?;
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..brace]);
    out.push_str("{\n  false\n}");
    out.push_str(&text[end..]);
    fs::write(path, out).map_err(|e| format!("write {}: {e}", path.display()))
}

fn remap_entry_for_temp(source_root: &str, entry: &str) -> PathBuf {
    let prefix = format!("{source_root}/");
    if let Some(suffix) = entry.strip_prefix(&prefix) {
        PathBuf::from("src").join(suffix)
    } else if let Some(suffix) = entry.strip_prefix("src/v2/") {
        PathBuf::from("src").join(suffix)
    } else {
        PathBuf::from(entry)
    }
}

fn run_perturb_pass(
    _source_roots: &[String],
    rows: &[GateRow],
    primary_root: &str,
) -> Result<bool, String> {
    let mut all_ok = true;
    for (idx, row) in rows.iter().enumerate() {
        let tmp = std::env::temp_dir().join(format!(
            "ci-claim-gate-perturb-{}-{}",
            std::process::id(),
            idx
        ));
        if tmp.exists() {
            fs::remove_dir_all(&tmp).map_err(|e| format!("rm {}: {e}", tmp.display()))?;
        }
        let src_v4 = tmp.join("src");
        let from = Path::new(primary_root);
        fs::create_dir_all(&src_v4).map_err(|e| format!("mkdir {}: {e}", src_v4.display()))?;
        copy_dir_all(from, &src_v4)?;

        let perturbed_entry = tmp.join(remap_entry_for_temp(primary_root, &row.entry));
        perturb_function_to_false(&perturbed_entry, &row.function)?;

        let temp_source_root = src_v4.to_string_lossy().into_owned();
        let perturbed_entry_str = perturbed_entry.to_string_lossy().into_owned();
        let index = build_multi_entry_index(&[temp_source_root]);
        let (graph, si) = resolve_entry_with_index(&index, &perturbed_entry_str)?;
        let ctx = make_eval_context(&graph, si);
        println!("::group::perturb: {}", row.label);
        let failed = matches!(run_claim(&ctx, &row.function), ClaimOutcome::Fail);
        println!("::endgroup::");
        if !failed {
            eprintln!("::error::perturbed witness still passed: {}", row.label);
            all_ok = false;
        }
        let _ = fs::remove_dir_all(&tmp);
    }
    Ok(all_ok)
}

/// Recursively collect `.dag` files under `dir`, skipping any path containing an
/// excluded substring (mirrors the `unified_claim_` discovery exclude set).
fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

/// Extract `NAME` from every `test fn NAME(...)` / `test data NAME: ...`
/// declaration in source text.
fn scan_test_decl_names(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let rest = trimmed
            .strip_prefix("test fn ")
            .or_else(|| trimmed.strip_prefix("test data "));
        if let Some(rest) = rest {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                names.push(name);
            }
        }
    }
    names
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Err(format!("{} is not a directory", from.display()));
    }
    fs::create_dir_all(to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("read_dir {}: {e}", from.display()))? {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let ft = entry.file_type().map_err(|e| format!("file_type: {e}"))?;
        let dest = to.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest).map_err(|e| {
                format!("copy {} -> {}: {e}", entry.path().display(), dest.display())
            })?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Rust-monolith conditional gates (folded from the former `rust_gate_runner`).
//
// `src/v2/workflow/rust_stage0_gates.dag` is the *selection authority*: it decides
// which changed paths belong to the Rust monolith (`path_is_rust_monolith`). This
// phase REALIZES that model's clippy/fmt dependents — it asks the selector (per
// changed path, String in / Bool out, evaluated in-interpreter so the matching
// logic is never re-implemented here) and, if any path is Rust, runs the two host
// gates: `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`.
//
// The in-process witness gate above is the ALWAYS-ON CI floor; this only adds the
// two gates conditional on the Rust seed changing. FAIL-CLOSED BIAS (§5): if the
// diff cannot be determined (git error or empty result), the gates run anyway —
// under-firing ships broken Rust silently, over-firing only costs CI time.
//
// SCAFFOLD — dissolves when the Rust seed reaches zero (DESIGN §7).
// ---------------------------------------------------------------------------

/// Selection authority: the `.dag` model that decides Rust-monolith membership.
const RUST_GATES_ENTRY: &str = "src/v2/workflow/rust_stage0_gates.dag";
/// The selector predicate (`path: String -> Bool`) consulted per changed path.
const RUST_GATES_PREDICATE: &str = "path_is_rust_monolith";

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
fn rust_touched_via_selector(source_roots: &[String], paths: &[String]) -> Result<bool, ExitCode> {
    let index = build_multi_entry_index(source_roots);
    let (graph, si) = match resolve_entry_with_index(&index, RUST_GATES_ENTRY) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ci-claim-gate (rust gates): resolve {RUST_GATES_ENTRY}: {e}");
            return Err(ExitCode::from(2));
        }
    };
    let ctx = make_eval_context(&graph, si);
    for p in paths {
        match run_in_context_with_args(
            &ctx,
            RUST_GATES_PREDICATE,
            &[(Some("path".to_string()), Value::Str(p.clone()))],
            false,
        ) {
            Ok(Value::Bool(true)) => return Ok(true),
            Ok(Value::Bool(false)) => {}
            Ok(other) => {
                eprintln!(
                    "ci-claim-gate (rust gates): {RUST_GATES_PREDICATE} returned `{other}`, expected Bool"
                );
                return Err(ExitCode::from(2));
            }
            Err(e) => {
                eprintln!("ci-claim-gate (rust gates): {e}");
                return Err(ExitCode::from(2));
            }
        }
    }
    Ok(false)
}

/// Run both Rust gates, fail-closed, without short-circuiting — so a single CI run reports
/// every failing gate, not just the first. Returns `true` iff all gates passed.
fn run_rust_gates() -> bool {
    let gates: [(&str, &[&str]); 2] = [
        ("cargo fmt --all --check", &["fmt", "--all", "--check"]),
        (
            "cargo clippy --all-targets -- -D warnings",
            &["clippy", "--all-targets", "--", "-D", "warnings"],
        ),
    ];
    let mut failures: Vec<&str> = Vec::new();
    for (name, args) in gates {
        println!("ci-claim-gate (rust gates): running {name}");
        match Command::new("cargo").args(args).status() {
            Ok(s) if s.success() => {}
            Ok(_) => failures.push(name),
            Err(e) => {
                eprintln!("ci-claim-gate (rust gates): failed to spawn `{name}`: {e}");
                failures.push(name);
            }
        }
    }
    if failures.is_empty() {
        println!("ci-claim-gate (rust gates): all Rust gates passed (fmt, clippy)");
        true
    } else {
        for name in &failures {
            eprintln!("::error::rust gate failed: {name}");
        }
        false
    }
}

/// The conditional Rust-gate phase: determine the changed paths (explicit `--changed-path`
/// or `git diff` fallback, fail-closed), ask the `.dag` selector whether the Rust monolith
/// was touched, and run clippy/fmt iff so. Returns `None` when the phase passed or skipped
/// (caller continues), or `Some(code)` to short-circuit `main` with that exit code.
fn run_rust_gates_phase(cfg: &Config) -> Option<ExitCode> {
    let (paths, fail_closed): (Vec<String>, bool) = if !cfg.rust_gates_changed_paths.is_empty() {
        (cfg.rust_gates_changed_paths.clone(), false)
    } else {
        match changed_paths_from_git(&cfg.rust_gates_base) {
            Some(p) => (p, false),
            None => {
                println!(
                    "ci-claim-gate (rust gates): could not determine changed paths (git error or empty) — running gates fail-closed"
                );
                (Vec::new(), true)
            }
        }
    };

    let rust_touched = if fail_closed {
        true
    } else {
        match rust_touched_via_selector(&cfg.source_roots, &paths) {
            Ok(t) => t,
            Err(code) => return Some(code),
        }
    };

    if !rust_touched {
        println!(
            "ci-claim-gate (rust gates): no Rust-monolith path in diff — clippy/fmt skipped (in-process witness gate is the always-on floor)"
        );
        return None;
    }

    if run_rust_gates() {
        None
    } else {
        Some(ExitCode::from(1))
    }
}

fn main() -> ExitCode {
    let cfg = parse_args();
    if cfg.source_roots.is_empty() {
        eprintln!("ci-claim-gate: provide at least one --source-root");
        return ExitCode::from(2);
    }

    let rows = match &cfg.roster {
        RosterSource::RowsFn {
            gate_entry,
            rows_fn,
        } => {
            let tsv = match evaluate_rows_tsv(gate_entry, rows_fn, &cfg.source_roots) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ci-claim-gate: {e}");
                    return ExitCode::from(2);
                }
            };
            if cfg.print_tsv_only {
                print!("{tsv}");
                return ExitCode::SUCCESS;
            }
            parse_rows_tsv(&tsv)
        }
        RosterSource::Discovery { scan_dirs } => {
            let rows = match discover_roster(&cfg.source_roots, scan_dirs) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("ci-claim-gate: discovery roster failed: {e}");
                    return ExitCode::from(2);
                }
            };
            if cfg.print_tsv_only {
                for r in &rows {
                    println!("{}\t{}\t{}", r.label, r.entry, r.function);
                }
                return ExitCode::SUCCESS;
            }
            rows
        }
    };
    if rows.is_empty() {
        eprintln!("ci-claim-gate: roster produced no rows (empty corpus → fail closed)");
        return ExitCode::from(2);
    }

    println!(
        "::group::{} green pass ({} witness(es))",
        cfg.notice_title,
        rows.len()
    );
    let green_ok = match run_green_pass(&cfg.source_roots, &rows) {
        Ok(ok) => ok,
        Err(e) => {
            eprintln!("ci-claim-gate: green pass failed: {e}");
            return ExitCode::from(1);
        }
    };
    println!("::endgroup::");
    if !green_ok {
        return ExitCode::from(1);
    }

    if cfg.perturb {
        let primary_root = cfg.source_roots[0].clone();
        match run_perturb_pass(&cfg.source_roots, &rows, &primary_root) {
            Ok(true) => {}
            Ok(false) => return ExitCode::from(1),
            Err(e) => {
                eprintln!("ci-claim-gate: perturb pass failed: {e}");
                return ExitCode::from(1);
            }
        }
    }

    println!(
        "{}: {} discriminating witness(es) passed",
        cfg.notice_title,
        rows.len()
    );

    // Conditional Rust-monolith gates (opt-in). The witness floor above is always-on;
    // these only fire when the diff touches the Rust seed (per the `.dag` selector).
    if cfg.rust_gates {
        if let Some(code) = run_rust_gates_phase(&cfg) {
            return code;
        }
    }

    ExitCode::SUCCESS
}
