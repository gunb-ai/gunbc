//! `ci-claim-gate` — uniform `.dag`-driven CI Bool-witness gate host.
//!
//! Replaces the duplicated awk/grep roster projection in `v4-lens-ci-gate.sh`,
//! `v4-affected-set-node-frontier-gate.sh`, and peers. The gate model owns the row
//! list; this host evaluates a modeled `*_rows_tsv()` function via the v2 interpreter
//! (the #4804 / `claim_batch` Option-B precedent), then runs:
//!   1. GREEN pass — one `claim_batch`-style multi-entry resolve (module index once)
//!   2. PERTURB pass (optional) — per-row temp-tree witness body → `false`, must fail
//!
//! Exit codes: 0 = all witnesses passed (+ perturb receipts when requested);
//! 1 = witness failure or perturb did not go red; 2 = usage / transport error.

#![allow(clippy::disallowed_macros)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use v2_compiler::cli_run::{
    build_multi_entry_index, make_eval_context, resolve_entry_with_index, run_claim, ClaimOutcome,
};
use v2_compiler::v2_interpreter::{run_in_context_with_args, Value};

struct GateRow {
    label: String,
    entry: String,
    function: String,
}

struct Config {
    source_roots: Vec<String>,
    gate_entry: String,
    rows_fn: String,
    perturb: bool,
    print_tsv_only: bool,
    notice_title: String,
}

fn usage() -> ! {
    eprintln!(
        "usage: ci-claim-gate --source-root <dir> [--source-root <dir> ...] \\\n\
         \x20       --gate-entry <file.dag> --rows-fn <function> \\\n\
         \x20       [--perturb-check] [--print-tsv-only] [--notice-title <title>]"
    );
    std::process::exit(2);
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut source_roots = Vec::new();
    let mut gate_entry = None;
    let mut rows_fn = None;
    let mut perturb = false;
    let mut print_tsv_only = false;
    let mut notice_title = "v4 CI claim gate".to_string();

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
            "--perturb-check" => perturb = true,
            "--print-tsv-only" => print_tsv_only = true,
            "--notice-title" => {
                i += 1;
                notice_title = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            other => {
                eprintln!("ci-claim-gate: unknown argument: {other}");
                usage();
            }
        }
        i += 1;
    }

    Config {
        source_roots,
        gate_entry: gate_entry.unwrap_or_else(|| {
            eprintln!("ci-claim-gate: --gate-entry is required");
            usage();
        }),
        rows_fn: rows_fn.unwrap_or_else(|| {
            eprintln!("ci-claim-gate: --rows-fn is required");
            usage();
        }),
        perturb,
        print_tsv_only,
        notice_title,
    }
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
    } else if let Some(suffix) = entry.strip_prefix("src/v4/") {
        PathBuf::from("src").join(suffix)
    } else {
        PathBuf::from(entry)
    }
}

// Born-mark (single-primary-root temp tree): the perturb temp tree mirrors only `primary_root`
// (source_roots[0]) under `tmp/src`, and `remap_entry_for_temp` only knows how to relocate
// entries under that one root. Every perturbed row must therefore live under the primary root;
// a row from another root fails LOUD (absent temp path), it is NOT a fail-open miss. Today all
// consumers keep each shard's *perturbed* rows single-primary-root (the claim-witness-corpus
// gate is root-aligned for exactly this reason). GENERALIZE to a multi-root, repo-relative-layout
// temp tree (copy each source-root preserving its path, remap each entry verbatim) WHEN a
// cost-balanced shard first genuinely needs perturbed rows spanning multiple roots — that is the
// JIT trigger (a second real consumer), not a speculative generalization for one.
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

fn main() -> ExitCode {
    let cfg = parse_args();
    if cfg.source_roots.is_empty() {
        eprintln!("ci-claim-gate: provide at least one --source-root");
        return ExitCode::from(2);
    }

    let tsv = match evaluate_rows_tsv(&cfg.gate_entry, &cfg.rows_fn, &cfg.source_roots) {
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

    let rows = parse_rows_tsv(&tsv);
    if rows.is_empty() {
        eprintln!(
            "ci-claim-gate: {rows_fn} produced no rows",
            rows_fn = cfg.rows_fn
        );
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
        "::notice title={}::{} discriminating lens witness(es) passed",
        cfg.notice_title,
        rows.len()
    );
    ExitCode::SUCCESS
}
