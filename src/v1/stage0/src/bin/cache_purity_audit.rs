//! cache_purity_audit — the warm==cold shadow audit over the floor discovery corpus (DESIGN §5;
//! ROADMAP §2 P3). Two modes:
//!
//!  - LEAF (`--shard I/K`): audit shard I of the corpus partitioned `idx % K == I`. For every entry,
//!    a WARM resolved-graph cache hit must be canonically `==` the COLD compute it cached (the
//!    write→read CODEC is the only difference under test). A divergence is a located, typed, LOUD
//!    `CachePurityViolation` → exit 1. With `--result-file P` it also writes a parseable summary line
//!    (the orchestrator reads it; a MISSING result file is treated as a fail-closed drop).
//!
//!  - ORCHESTRATOR (`--orchestrate`): the CO-PROCESS entry the CI floor job backgrounds. It reads the
//!    LIVE host memory budget, evaluates the residual-budget width fold (`--width-entry/-function`),
//!    fans the resulting width over the corpus as K leaf child-processes (each an isolated `mktemp`
//!    cache), and JOINS them FAIL-CLOSED: the run is RED if ANY shard reports a violation (incl. the
//!    deepest/last) OR any shard crashes / OOMs / drops (no result file). Width 0 (cannot fit beside
//!    the floor) is itself fail-closed (no fan-out, exit 1). This is TOOTH 1 (join) + TOOTH 2 (budget)
//!    made executable, so both are testable in Rust rather than a fragile shell aggregation.
//!
//! This gate REQUIRES `GUNBC_RESOLVED_GRAPH_CACHE_DIR` set (the orchestrator sets each child's to an
//! isolated empty dir so COLD genuinely misses+computes); it refuses a vacuous audit (DESIGN §5/§6).
//!
//! §5 honest residual: SOUND over the CI corpus, NOT COMPLETE over prod-only realizations absent
//! from CI. Codec depth note: ~79% of the corpus is too deep for serde's 128-level decode, so
//! production `read_cached_file` Misses→recomputes on them (FAIL-SAFE, uncached) — a cache
//! EFFECTIVENESS gap, not a soundness hole (see the audited reach split below).
//!
//! Exit: 0 = clean; 1 = any violation / dropped shard / setup error (fail-closed); 2 = usage error.

#![allow(clippy::disallowed_macros)]

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use v1_compiler::cache_purity_audit::{audit_floor_discovery_corpus, CorpusAuditReport, Shard};
use v1_compiler::cli_run::{make_eval_context, read_host_memory_budget_bytes, resolve_entry_graph};
use v1_compiler::v1_interpreter::{run_in_context_with_args, ExecutionMode, Value};

const DEFAULT_SCAN_DIR: &str = "dsl/test/claim";

struct Args {
    source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    notice_title: String,
    max_entries: Option<usize>,
    shard: Option<Shard>,
    orchestrate: bool,
    width_override: Option<usize>,
    width_entry: Option<String>,
    width_function: Option<String>,
    result_file: Option<String>,
}

fn run() -> Result<ExitCode, ExitCode> {
    let raw: Vec<String> = std::env::args().collect();
    let mut a = Args {
        source_roots: Vec::new(),
        scan_dirs: Vec::new(),
        notice_title: "cache purity audit".to_string(),
        max_entries: None,
        shard: None,
        orchestrate: false,
        width_override: None,
        width_entry: None,
        width_function: None,
        result_file: None,
    };

    let mut i = 1;
    while i < raw.len() {
        match raw[i].as_str() {
            "--source-root" => {
                i += 1;
                a.source_roots.push(arg_value(&raw, i, "--source-root")?);
            }
            "--scan-dir" => {
                i += 1;
                a.scan_dirs.push(arg_value(&raw, i, "--scan-dir")?);
            }
            "--max-entries" => {
                i += 1;
                a.max_entries = Some(parse_usize(
                    &arg_value(&raw, i, "--max-entries")?,
                    "--max-entries",
                )?);
            }
            "--shard" => {
                i += 1;
                a.shard = Some(parse_shard(&arg_value(&raw, i, "--shard")?)?);
            }
            "--orchestrate" => a.orchestrate = true,
            "--width" => {
                i += 1;
                a.width_override = Some(parse_usize(&arg_value(&raw, i, "--width")?, "--width")?);
            }
            "--width-entry" => {
                i += 1;
                a.width_entry = Some(arg_value(&raw, i, "--width-entry")?);
            }
            "--width-function" => {
                i += 1;
                a.width_function = Some(arg_value(&raw, i, "--width-function")?);
            }
            "--result-file" => {
                i += 1;
                a.result_file = Some(arg_value(&raw, i, "--result-file")?);
            }
            "--notice-title" => {
                i += 1;
                a.notice_title = arg_value(&raw, i, "--notice-title")?;
            }
            other => {
                eprintln!("cache_purity_audit: unknown argument '{other}'");
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if a.source_roots.is_empty() {
        eprintln!("cache_purity_audit: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    if a.scan_dirs.is_empty() {
        a.scan_dirs.push(DEFAULT_SCAN_DIR.to_string());
    }

    if a.orchestrate {
        orchestrate(&a)
    } else {
        run_leaf(&a)
    }
}

// ── LEAF: audit one shard (or the whole/bounded corpus when no --shard) ───────────────────────
fn run_leaf(a: &Args) -> Result<ExitCode, ExitCode> {
    // Test hooks (orchestrated leaves only — gated on --shard + --result-file). They short-circuit
    // BEFORE any heavy resolve so the join teeth are fast to prove by execution.
    if let (Some(s), Some(rf)) = (a.shard, a.result_file.as_deref()) {
        if test_env_matches("GUNBC_CPA_TEST_VIOLATE_SHARD", s.index) {
            // Synthetic warm!=cold in THIS shard: write a result with a violation and exit 1.
            println!("::error title={}::synthetic warm!=cold violation injected in shard {}/{} (test hook)", a.notice_title, s.index, s.count);
            write_result_file(
                rf,
                &CorpusAuditReport {
                    entries_discovered: 0,
                    entries_audited: 1,
                    decoded: 0,
                    miss_on_read: 0,
                    rejected: 0,
                    skipped: 0,
                    violations: Vec::new(),
                },
                1,
            );
            return Err(ExitCode::from(1));
        }
        if test_env_matches("GUNBC_CPA_TEST_DROP_SHARD", s.index) {
            // Simulate an OOM-kill / crash: exit WITHOUT writing a result file. The orchestrator must
            // treat the missing result as a fail-closed drop (NOT silently pass).
            eprintln!(
                "cache_purity_audit: shard {}/{} dropping (test hook, no result file)",
                s.index, s.count
            );
            std::process::exit(137);
        }
    }

    println!("::group::{}", a.notice_title);
    let report =
        match audit_floor_discovery_corpus(&a.source_roots, &a.scan_dirs, a.max_entries, a.shard) {
            Ok(r) => r,
            Err(setup_err) => {
                println!("::error title={}::{setup_err}", a.notice_title);
                println!("::endgroup::");
                return Err(ExitCode::from(1));
            }
        };

    for v in &report.violations {
        println!("::error title={}::{v}", a.notice_title);
    }
    let coverage = coverage_line(&report);
    let code = if report.violations.is_empty() {
        println!("cache purity audit: {coverage} — warm==cold (no lossy/stale decode)");
        emit_shard_and_depth_notes(a, &report);
        ExitCode::SUCCESS
    } else {
        println!(
            "cache purity audit FAILED: {}/{} audited entries diverged warm!=cold ({coverage}) — the \
             cache codec is lossy/stale (a warm hit serves a different graph than a cold recompute)",
            report.violations.len(),
            report.entries_audited
        );
        ExitCode::from(1)
    };
    println!("::endgroup::");

    if let Some(rf) = a.result_file.as_deref() {
        write_result_file(rf, &report, report.violations.len());
    }
    Ok(code)
}

// ── ORCHESTRATOR: width → fan-out → fail-closed join ──────────────────────────────────────────
fn orchestrate(a: &Args) -> Result<ExitCode, ExitCode> {
    let width = match a.width_override {
        Some(w) => w,
        None => match eval_audit_width(a) {
            Ok(w) => w,
            Err(msg) => {
                // Fail-closed: an unevaluable width is NOT a green.
                println!(
                    "::error title={}::width derivation failed: {msg}",
                    a.notice_title
                );
                return Err(ExitCode::from(1));
            }
        },
    };
    println!("::group::{} (orchestrator)", a.notice_title);
    println!("cache purity audit orchestrator: co-run width = {width} shard(s)");

    // TOOTH 2: width 0 means the residual budget (0.8·total − floor concurrent peak) cannot fit even
    // ONE audit shard beside the floor → fail closed, NEVER a silent width-1 that busts the shared cap.
    if width == 0 {
        println!(
            "::error title={}::residual budget too small to co-run the cache-purity audit beside the \
             floor (0.8·budget − floor concurrent peak < one audit shard) — fail-closed (DESIGN §5; \
             would otherwise OOM the shared cgroup)",
            a.notice_title
        );
        println!("::endgroup::");
        return Err(ExitCode::from(1));
    }

    let exe = std::env::var_os("GUNBC_CPA_SHARD_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_exe().expect("orchestrator: current_exe unavailable"));

    let mut children = Vec::new();
    let mut tmp_caches = Vec::new();
    for idx in 0..width {
        let cache = orchestrator_tempdir("cache", idx);
        let result = orchestrator_tempdir("result", idx).join("result.txt");
        if let Err(e) = std::fs::create_dir_all(&cache) {
            return fail_spawn(
                a,
                idx,
                &format!("could not create shard cache dir: {e}"),
                &tmp_caches,
            );
        }
        if let Some(parent) = result.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut cmd = Command::new(&exe);
        cmd.arg("--shard").arg(format!("{idx}/{width}"));
        for r in &a.source_roots {
            cmd.arg("--source-root").arg(r);
        }
        for d in &a.scan_dirs {
            cmd.arg("--scan-dir").arg(d);
        }
        cmd.arg("--result-file").arg(&result);
        cmd.arg("--notice-title")
            .arg(format!("{} shard {idx}/{width}", a.notice_title));
        cmd.env("GUNBC_RESOLVED_GRAPH_CACHE_DIR", &cache);
        match cmd.spawn() {
            Ok(child) => children.push((idx, child, result)),
            Err(e) => return fail_spawn(a, idx, &format!("spawn failed: {e}"), &tmp_caches),
        }
        tmp_caches.push(cache);
    }

    // Join EVERY child fail-closed: collect each exit status AND its result file. A dropped/uncollected
    // shard (the co-process fail-open risk) is impossible — we wait every pid and a missing result is RED.
    let mut any_fail = false;
    let mut total_violations = 0usize;
    let mut agg = AggReach::default();
    for (idx, mut child, result) in children {
        let status = child.wait();
        let exit_code = status.as_ref().ok().and_then(|s| s.code());
        let succeeded = status.as_ref().map(|s| s.success()).unwrap_or(false);
        let parsed = std::fs::read_to_string(&result)
            .ok()
            .and_then(|s| parse_result_line(&s));
        match parsed {
            Some(rep) if succeeded && rep.violations == 0 => {
                agg.add(&rep);
            }
            Some(rep) if succeeded => {
                // Exit 0 but the result reports violations — still RED (belt-and-suspenders).
                any_fail = true;
                total_violations += rep.violations;
                agg.add(&rep);
                println!(
                    "::error title={}::shard {idx} RED — violations={} (warm!=cold)",
                    a.notice_title, rep.violations
                );
            }
            Some(rep) => {
                // Non-zero exit with a parsed result: a violation/setup failure in the shard.
                any_fail = true;
                total_violations += rep.violations;
                agg.add(&rep);
                println!(
                    "::error title={}::shard {idx} RED — exit={exit_code:?}, violations={} (warm!=cold or setup failure)",
                    a.notice_title, rep.violations
                );
            }
            None => {
                // Missing/unparseable result = crash / OOM / timeout / drop → fail-closed.
                any_fail = true;
                println!(
                    "::error title={}::shard {idx} DROPPED — no parseable result (exit={exit_code:?}); a \
                     crashed/OOM'd/timed-out audit shard is treated as failed, never silently passed (DESIGN §5)",
                    a.notice_title
                );
            }
        }
    }
    for c in &tmp_caches {
        let _ = std::fs::remove_dir_all(c);
    }

    println!(
        "cache purity audit (co-run, full corpus): {} shards · audited {} · [decoded {} · \
         miss-on-read(deep, fail-safe) {} · rejected {} · skipped {}] · violations {}",
        width,
        agg.audited,
        agg.decoded,
        agg.miss_on_read,
        agg.rejected,
        agg.skipped,
        total_violations
    );
    if agg.miss_on_read > 0 {
        println!(
            "note: {} entr(ies) too DEEP to decode (serde 128-level limit) → production read_cached_file \
             Misses → recomputes (FAIL-SAFE, uncached, NOT lossy); a cache-effectiveness gap, not a §5 \
             soundness hole",
            agg.miss_on_read
        );
    }
    println!(
        "note: sound over the audited corpus; NOT complete over prod-only realizations absent from CI \
         (DESIGN §5 honest edge)"
    );
    println!("::endgroup::");

    if any_fail {
        Err(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Evaluate the residual-budget width fold (single authority, std.realization via the gunbc binding)
/// with the LIVE host budget bound by name — exactly how claim_executor derives the floor width.
fn eval_audit_width(a: &Args) -> Result<usize, String> {
    let entry = a
        .width_entry
        .as_deref()
        .ok_or("orchestrator requires --width-entry (or --width N)")?;
    let function = a
        .width_function
        .as_deref()
        .ok_or("orchestrator requires --width-function (or --width N)")?;
    let (graph, indices) = resolve_entry_graph(&a.source_roots, entry)
        .map_err(|msg| format!("resolve failed for width entry {entry}:\n{msg}"))?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Wet);
    let budget = read_host_memory_budget_bytes().unwrap_or(0);
    match budget {
        0 => eprintln!("cache_purity_audit: live memory budget unavailable — width uses the .dag conservative fallback"),
        b => eprintln!("cache_purity_audit: live memory budget {b} bytes (cgroup memory.max / meminfo)"),
    }
    let budget_arg = i64::try_from(budget).unwrap_or(i64::MAX);
    let value = run_in_context_with_args(
        &ctx,
        function,
        &[(
            Some("memory_budget_bytes".to_string()),
            Value::Int(budget_arg),
        )],
        false,
    )
    .map_err(|e| format!("width eval failed ({entry}::{function}): {e}"))?;
    match value {
        Value::Int(n) if n > 0 => Ok(n as usize),
        // 0 or negative ⟹ over-budget / fail-closed signal; surface as width 0 (orchestrator fails closed).
        Value::Int(_) => Ok(0),
        other => Err(format!("width fn returned a non-Int value: {other:?}")),
    }
}

fn fail_spawn(a: &Args, idx: usize, msg: &str, tmp: &[PathBuf]) -> Result<ExitCode, ExitCode> {
    for c in tmp {
        let _ = std::fs::remove_dir_all(c);
    }
    println!(
        "::error title={}::shard {idx} could not be launched: {msg} (fail-closed)",
        a.notice_title
    );
    println!("::endgroup::");
    Err(ExitCode::from(1))
}

// ── shared helpers ────────────────────────────────────────────────────────────────────────────
#[derive(Default)]
struct AggReach {
    audited: usize,
    decoded: usize,
    miss_on_read: usize,
    rejected: usize,
    skipped: usize,
}
impl AggReach {
    fn add(&mut self, r: &ParsedResult) {
        self.audited += r.audited;
        self.decoded += r.decoded;
        self.miss_on_read += r.miss_on_read;
        self.rejected += r.rejected;
        self.skipped += r.skipped;
    }
}

struct ParsedResult {
    audited: usize,
    decoded: usize,
    miss_on_read: usize,
    rejected: usize,
    skipped: usize,
    violations: usize,
}

fn coverage_line(report: &CorpusAuditReport) -> String {
    format!(
        "audited {} of {} discovered entries [decoded {} · miss-on-read(deep, fail-safe) {} · rejected {} · skipped {}]",
        report.entries_audited, report.entries_discovered, report.decoded, report.miss_on_read, report.rejected, report.skipped
    )
}

fn emit_shard_and_depth_notes(a: &Args, report: &CorpusAuditReport) {
    if let Some(s) = a.shard {
        println!(
            "note: SHARD {}/{} — covers every (idx % {} == {}) entry; the {} shards together audit the \
             FULL corpus exactly once (full coverage, parallelized off the floor critical path)",
            s.index, s.count, s.count, s.index, s.count
        );
    }
    if a.max_entries.is_some() {
        println!(
            "note: BOUNDED — a FAST SMOKE, not the soundness gate (codec fidelity is depth-dependent; \
             run without --max-entries — full corpus, sharded by the orchestrator — for the sound sweep)"
        );
    }
    if report.miss_on_read > 0 {
        println!(
            "note: {} entr(ies) too DEEP to decode (serde 128-level limit) → production read_cached_file \
             Misses → recomputes (FAIL-SAFE, uncached, NOT lossy); a cache-effectiveness gap",
            report.miss_on_read
        );
    }
}

/// One-line parseable result the orchestrator reads back (a missing line = fail-closed drop).
fn write_result_file(path: &str, report: &CorpusAuditReport, violations: usize) {
    let line = format!(
        "CPA_RESULT discovered={} audited={} decoded={} miss_on_read={} rejected={} skipped={} violations={}\n",
        report.entries_discovered, report.entries_audited, report.decoded, report.miss_on_read, report.rejected, report.skipped, violations
    );
    let _ = std::fs::write(path, line);
}

fn parse_result_line(s: &str) -> Option<ParsedResult> {
    let line = s.lines().find(|l| l.starts_with("CPA_RESULT"))?;
    let mut audited = None;
    let mut decoded = None;
    let mut miss_on_read = None;
    let mut rejected = None;
    let mut skipped = None;
    let mut violations = None;
    for tok in line.split_whitespace() {
        if let Some((k, v)) = tok.split_once('=') {
            let n = v.parse::<usize>().ok();
            match k {
                "audited" => audited = n,
                "decoded" => decoded = n,
                "miss_on_read" => miss_on_read = n,
                "rejected" => rejected = n,
                "skipped" => skipped = n,
                "violations" => violations = n,
                _ => {}
            }
        }
    }
    Some(ParsedResult {
        audited: audited?,
        decoded: decoded?,
        miss_on_read: miss_on_read?,
        rejected: rejected?,
        skipped: skipped?,
        violations: violations?,
    })
}

fn orchestrator_tempdir(kind: &str, idx: usize) -> PathBuf {
    std::env::temp_dir().join(format!("cpa-orch-{}-{kind}-{idx}", std::process::id()))
}

fn test_env_matches(var: &str, idx: usize) -> bool {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        == Some(idx)
}

fn parse_usize(raw: &str, flag: &str) -> Result<usize, ExitCode> {
    raw.parse::<usize>().map_err(|_| {
        eprintln!("cache_purity_audit: {flag} expects a non-negative integer, got '{raw}'");
        ExitCode::from(2)
    })
}

fn parse_shard(raw: &str) -> Result<Shard, ExitCode> {
    let usage = || {
        eprintln!("cache_purity_audit: --shard expects I/K (e.g. 0/4), got '{raw}'");
        ExitCode::from(2)
    };
    let (i_str, k_str) = raw.split_once('/').ok_or_else(usage)?;
    let index = i_str.parse::<usize>().map_err(|_| usage())?;
    let count = k_str.parse::<usize>().map_err(|_| usage())?;
    if count == 0 || index >= count {
        eprintln!("cache_purity_audit: --shard {index}/{count} invalid (need 0 <= index < count, count >= 1)");
        return Err(ExitCode::from(2));
    }
    Ok(Shard { index, count })
}

fn arg_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("cache_purity_audit: {flag} requires a value");
            Err(ExitCode::from(2))
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
