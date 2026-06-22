#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;

use v1_compiler::cli_run::{
    make_eval_context, resolve_entry_graph, run_claim, run_discovery_corpus_with_options,
    run_value, ClaimOutcome, DiscoveryCorpusOptions,
};
use v1_compiler::v1_interpreter::{run_in_context_with_args, ExecutionMode, InterpContext, Value};

#[derive(Clone)]
enum Runnable {
    SingleClaim {
        entry: String,
        function: String,
    },
    DiscoveryBatch {
        source_roots: Vec<String>,
        scan_dirs: Vec<String>,
        explicit_entries: Vec<(String, String)>,
        skip_unaffected_node_frontier: bool,
    },
}

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("claim_executor: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

fn free_monoid_elems<'a>(value: &'a Value, ctx: &InterpContext) -> Result<Vec<&'a Value>, String> {
    let mut out = Vec::new();
    let mut cur = value;
    loop {
        match cur {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*variant_name, "Cons") => {
                let head = ctx
                    .field(fields, "head")
                    .ok_or_else(|| "Cons without `head` field".to_string())?;
                out.push(head);
                cur = ctx
                    .field(fields, "tail")
                    .ok_or_else(|| "Cons without `tail` field".to_string())?;
            }
            Value::Variant { variant_name, .. } if ctx.sym_eq(*variant_name, "Empty") => {
                return Ok(out);
            }
            Value::List(items) => {
                out.extend(items.iter());
                return Ok(out);
            }
            other => {
                return Err(format!(
                    "expected a List (Cons/Empty), got {}",
                    other.type_label_public()
                ))
            }
        }
    }
}

fn str_list_from_value(value: &Value, ctx: &InterpContext) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for elem in free_monoid_elems(value, ctx)? {
        match elem {
            Value::Str(s) => out.push(s.clone()),
            other => {
                return Err(format!(
                    "expected a List<String> element, got {}",
                    other.type_label_public()
                ))
            }
        }
    }
    Ok(out)
}

fn str_field(
    fields: &std::collections::HashMap<v1_compiler::v1_interpreter::Symbol, Value>,
    name: &str,
    owner: &str,
    ctx: &InterpContext,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(other) => Err(format!(
            "{}.{} is {}, not String",
            owner,
            name,
            ctx.format_value(other)
        )),
        None => Err(format!("{} missing field `{}`", owner, name)),
    }
}

fn runnable_from_value(value: &Value, ctx: &InterpContext) -> Result<Runnable, String> {
    match value {
        Value::Record { type_name, fields } if ctx.sym_eq(*type_name, "ClaimRef") => {
            Ok(Runnable::SingleClaim {
                entry: str_field(fields, "entry", "ClaimRef", ctx)?,
                function: str_field(fields, "function", "ClaimRef", ctx)?,
            })
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableSingleClaim") => Ok(Runnable::SingleClaim {
            entry: str_field(fields, "entry", "RunnableSingleClaim", ctx)?,
            function: str_field(fields, "function", "RunnableSingleClaim", ctx)?,
        }),
        Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "RunnableDiscoveryBatch") => {
            let source_roots = match ctx.field(fields, "source_roots") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => {
                    return Err("RunnableDiscoveryBatch missing field `source_roots`".to_string())
                }
            };
            let scan_dirs = match ctx.field(fields, "scan_dirs") {
                Some(v) => str_list_from_value(v, ctx)?,
                None => return Err("RunnableDiscoveryBatch missing field `scan_dirs`".to_string()),
            };
            let explicit_entries = match ctx.field(fields, "explicit_entries") {
                Some(v) => {
                    let mut out = Vec::new();
                    for elem in free_monoid_elems(v, ctx)? {
                        let efields = match elem {
                            Value::Record { fields, .. } => fields,
                            Value::Variant { fields, .. } => fields,
                            other => {
                                return Err(format!(
                                    "RunnableDiscoveryBatch.explicit_entries element is {}, not a record",
                                    other.type_label_public()
                                ))
                            }
                        };
                        out.push((
                            str_field(efields, "entry", "explicit_entries", ctx)?,
                            str_field(efields, "function", "explicit_entries", ctx)?,
                        ));
                    }
                    out
                }
                None => Vec::new(),
            };
            let skip_unaffected_node_frontier =
                match ctx.field(fields, "skip_unaffected_node_frontier") {
                    Some(v) => match v {
                        Value::Bool(b) => *b,
                        other => {
                            return Err(format!(
                        "RunnableDiscoveryBatch.skip_unaffected_node_frontier must be Bool, got {}",
                        other.type_label_public()
                    ))
                        }
                    },
                    None => false,
                };
            Ok(Runnable::DiscoveryBatch {
                source_roots,
                scan_dirs,
                explicit_entries,
                skip_unaffected_node_frontier,
            })
        }
        other => Err(format!(
            "expected a ClaimRef record or Runnable variant, got {}",
            ctx.format_value(other)
        )),
    }
}

fn batches_from_plan(plan: &Value, ctx: &InterpContext) -> Result<Vec<Vec<Runnable>>, String> {
    let mut batches = Vec::new();
    for batch_val in free_monoid_elems(plan, ctx)? {
        let mut batch = Vec::new();
        for elem in free_monoid_elems(batch_val, ctx)? {
            batch.push(runnable_from_value(elem, ctx)?);
        }
        batches.push(batch);
    }
    Ok(batches)
}

struct ClaimResult {
    function: String,
    ok: bool,
    detail: String,
}

fn run_one_runnable(
    source_roots: Vec<String>,
    runnable: Runnable,
    spawn_width: usize,
) -> ClaimResult {
    match runnable {
        Runnable::SingleClaim { entry, function } => {
            run_single_claim(&source_roots, entry, function)
        }
        Runnable::DiscoveryBatch {
            source_roots: roots,
            scan_dirs,
            explicit_entries,
            skip_unaffected_node_frontier,
        } => run_discovery_batch_node(
            roots,
            scan_dirs,
            explicit_entries,
            skip_unaffected_node_frontier,
            spawn_width,
        ),
    }
}

fn run_single_claim(source_roots: &[String], entry: String, function: String) -> ClaimResult {
    if entry.is_empty() {
        return ClaimResult {
            function,
            ok: false,
            detail: "unrunnable sentinel (unmapped node or non-complete plan) — failing closed"
                .to_string(),
        };
    }
    let (graph, source_indices) = match resolve_entry_graph(source_roots, &entry) {
        Ok(pair) => pair,
        Err(msg) => {
            return ClaimResult {
                function,
                ok: false,
                detail: format!("resolve failed for {}: {}", entry, msg),
            }
        }
    };
    let ctx = make_eval_context(&graph, source_indices, ExecutionMode::Wet);
    match run_claim(&ctx, &function) {
        ClaimOutcome::Pass => ClaimResult {
            function,
            ok: true,
            detail: String::new(),
        },
        ClaimOutcome::Fail => ClaimResult {
            function,
            ok: false,
            detail: "returned Bool(false)".to_string(),
        },
        ClaimOutcome::NotBool { got } => ClaimResult {
            function,
            ok: false,
            detail: format!("returned `{}`, not Bool", got),
        },
        ClaimOutcome::RuntimeError { message } => ClaimResult {
            function,
            ok: false,
            detail: format!("runtime error: {}", message),
        },
    }
}

fn run_discovery_batch_node(
    source_roots: Vec<String>,
    scan_dirs: Vec<String>,
    explicit_entries: Vec<(String, String)>,
    skip_unaffected_node_frontier: bool,
    spawn_width: usize,
) -> ClaimResult {
    let label = format!(
        "discovery-corpus[{} root(s)+{} explicit, width={}]",
        source_roots.len(),
        explicit_entries.len(),
        spawn_width.max(1),
    );
    match run_discovery_corpus_with_options(
        &source_roots,
        &scan_dirs,
        &explicit_entries,
        ExecutionMode::Wet,
        spawn_width,
        DiscoveryCorpusOptions {
            skip_unaffected_node_frontier,
            explicit_roster_only: false,
        },
    ) {
        Ok(summary) if summary.failures.is_empty() => {
            eprintln!(
                "[measurement] discovery corpus: {} witness(es) ({} skipped), resolve {:.3}ms, evalu {:.3}ms, CostAccount.time basis=Measured {}ns",
                summary.total,
                summary.skipped,
                summary.total_resolve_nanos as f64 / 1.0e6,
                summary.total_measured_nanos as f64 / 1.0e6,
                summary.total_measured_nanos,
            );
            ClaimResult {
                function: format!("{label} ({} witnesses)", summary.total),
                ok: true,
                detail: String::new(),
            }
        }
        Ok(summary) => ClaimResult {
            function: label,
            ok: false,
            detail: format!(
                "{} of {} discovery witness(es) failed: {}",
                summary.failures.len(),
                summary.total,
                summary.failures.join("; ")
            ),
        },
        Err(msg) => ClaimResult {
            function: label,
            ok: false,
            detail: format!("discovery corpus failed: {msg}"),
        },
    }
}

fn eval_plan(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<Vec<Vec<Runnable>>, String> {
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for plan {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Hermetic);
    let plan_value = run_value(&plan_ctx, plan_function).map_err(|msg| {
        format!(
            "plan eval failed ({}::{}): {}",
            plan_entry, plan_function, msg
        )
    })?;
    let batches = batches_from_plan(&plan_value, &plan_ctx)
        .map_err(|msg| format!("malformed plan value: {}", msg))?;
    drop(plan_value);
    drop(plan_graph);
    Ok(batches)
}

fn spawn_width_function_name(plan_function: &str) -> Option<String> {
    plan_function
        .strip_suffix("_batches")
        .map(|prefix| format!("{prefix}_spawn_width"))
}

fn hardware_thread_count_from_value(value: &Value, ctx: &InterpContext) -> Result<usize, String> {
    match value {
        Value::Record { fields, .. } => match ctx.field(fields, "count") {
            Some(Value::Int(n)) => Ok((*n).max(1) as usize),
            Some(other) => Err(format!(
                "HardwareThreadCount.count is {}, not Int",
                ctx.format_value(other)
            )),
            None => Err("HardwareThreadCount missing `count` field".to_string()),
        },
        other => Err(format!(
            "expected HardwareThreadCount record, got {}",
            other.type_label_public()
        )),
    }
}

fn read_host_memory_budget_bytes() -> Option<u64> {
    if let Ok(self_cg) = fs::read_to_string("/proc/self/cgroup") {
        if let Some(rel) = self_cg
            .lines()
            .find_map(|l| l.strip_prefix("0::"))
            .map(|p| p.trim().trim_start_matches('/').to_string())
        {
            let root = Path::new("/sys/fs/cgroup");
            let mut dir = root.join(&rel);
            let mut effective: Option<u64> = None;
            loop {
                if let Ok(s) = fs::read_to_string(dir.join("memory.max")) {
                    let s = s.trim();
                    if s != "max" {
                        if let Ok(v) = s.parse::<u64>() {
                            effective = Some(effective.map_or(v, |cur| cur.min(v)));
                        }
                    }
                }
                if dir == root || !dir.pop() {
                    break;
                }
            }
            if let Some(v) = effective {
                return Some(v);
            }
        }
    }
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for key in ["MemAvailable", "MemTotal"] {
            if let Some(kb) = meminfo
                .lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<u64>().ok())
            {
                return Some(kb.saturating_mul(1024));
            }
        }
    }
    None
}

fn eval_spawn_width(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<usize, String> {
    let Some(width_fn) = spawn_width_function_name(plan_function) else {
        return Ok(1);
    };
    let (plan_graph, plan_indices) = resolve_entry_graph(source_roots, plan_entry)
        .map_err(|msg| format!("resolve failed for spawn width {}:\n{}", plan_entry, msg))?;
    let plan_ctx = make_eval_context(&plan_graph, plan_indices, ExecutionMode::Wet);
    let budget_bytes = read_host_memory_budget_bytes().unwrap_or(0);
    match budget_bytes {
        0 => eprintln!(
            "claim_executor: live memory budget unavailable — width uses the .dag conservative fallback"
        ),
        b => eprintln!("claim_executor: live memory budget {b} bytes (cgroup memory.max / meminfo)"),
    }
    let budget_arg = i64::try_from(budget_bytes).unwrap_or(i64::MAX);
    let width_value = run_in_context_with_args(
        &plan_ctx,
        &width_fn,
        &[(
            Some("memory_budget_bytes".to_string()),
            Value::Int(budget_arg),
        )],
        false,
    )
    .map_err(|e| {
        format!(
            "spawn width eval failed ({}::{}): {}",
            plan_entry, width_fn, e
        )
    })?;
    hardware_thread_count_from_value(&width_value, &plan_ctx)
}

struct WalkOutcome {
    any_failed: bool,
    batches_run: usize,
}

fn peak_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("VmHWM"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// The cgroup directory whose `memory.max` is the TIGHTEST along the `/proc/self/cgroup`
/// leaf→root walk — the EFFECTIVE budget the OOM-killer enforces (the same ancestor
/// `read_host_memory_budget_bytes` reduces to by `min`, returned here so the peak can be
/// read at the SAME level the budget binds). `None` when `/proc/self/cgroup` is unreadable
/// or no ancestor sets a numeric cap.
fn binding_cap_cgroup_dir() -> Option<std::path::PathBuf> {
    let self_cg = fs::read_to_string("/proc/self/cgroup").ok()?;
    let rel = self_cg
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .map(|p| p.trim().trim_start_matches('/').to_string())?;
    let root = Path::new("/sys/fs/cgroup");
    let mut dir = root.join(&rel);
    let mut best: Option<(u64, std::path::PathBuf)> = None;
    loop {
        if let Ok(s) = fs::read_to_string(dir.join("memory.max")) {
            let s = s.trim();
            if s != "max" {
                if let Ok(v) = s.parse::<u64>() {
                    let take = best.as_ref().map(|(cur, _)| v < *cur).unwrap_or(true);
                    if take {
                        best = Some((v, dir.clone()));
                    }
                }
            }
        }
        if dir == root || !dir.pop() {
            break;
        }
    }
    best.map(|(_, d)| d)
}

/// Whole-tree memory high-water + live pid count at the binding-cap ancestor cgroup. cgroup v2
/// `memory.peak`/`pids.current` are HIERARCHICAL (account for all descendant cgroups), and child
/// rustc/sccache fork-inherit the executor's cgroup, so reading at the budget-binding ancestor
/// captures every PID the budget governs — the SOUND placement divisor input, unlike SELF-RSS
/// `VmHWM` which omits children. Returns `(memory.peak bytes, pids.current, cgroup rel path)`.
/// `None` if no binding ancestor or `memory.peak` is unreadable (kernels < 5.19).
fn cgroup_peak_pids_at_binding_ancestor() -> Option<(u64, u64, String)> {
    let dir = binding_cap_cgroup_dir()?;
    let peak = fs::read_to_string(dir.join("memory.peak"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    let pids = fs::read_to_string(dir.join("pids.current"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let rel = dir
        .strip_prefix("/sys/fs/cgroup")
        .map(|p| format!("/{}", p.to_string_lossy().trim_start_matches('/')))
        .unwrap_or_else(|_| dir.to_string_lossy().into_owned());
    Some((peak, pids, rel))
}

/// Best-effort cgroup-v2 relative path of the sccache SERVER daemon (a `sccache` comm), scanned
/// from `/proc`. Emitted beside the binding-cap ancestor path so the analysis can classify the
/// "accounted exactly once" case: a path UNDER the ancestor → sccache is inside `memory.peak`
/// (don't subtract); a SIBLING path → subtract it as `host_fixed_overhead`. Returns `None` when
/// no sccache server process is found (then it's fixed host overhead by default, fail-closed).
fn sccache_server_cgroup_rel() -> Option<String> {
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let p = entry.path();
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        if fs::read_to_string(p.join("comm")).unwrap_or_default().trim() != "sccache" {
            continue;
        }
        if let Some(rel) = fs::read_to_string(p.join("cgroup"))
            .ok()
            .and_then(|cg| {
                cg.lines()
                    .find_map(|l| l.strip_prefix("0::"))
                    .map(|s| s.trim().to_string())
            })
        {
            return Some(rel);
        }
    }
    None
}

fn run_walk(source_roots: &[String], batches: &[Vec<Runnable>], spawn_width: usize) -> WalkOutcome {
    let width = spawn_width.max(1);
    let mut any_failed = false;
    let mut batches_run = 0usize;
    for (bi, batch) in batches.iter().enumerate() {
        batches_run = bi + 1;
        eprintln!(
            "claim_executor: batch {} — {} node(s), spawn_width={}",
            bi + 1,
            batch.len(),
            width
        );
        let handles: Vec<_> = batch
            .iter()
            .map(|runnable| {
                let roots = source_roots.to_vec();
                let runnable = runnable.clone();
                thread::spawn(move || run_one_runnable(roots, runnable, width))
            })
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(result) => {
                    if result.ok {
                        println!("PASS [batch {}] {}", bi + 1, result.function);
                    } else {
                        println!(
                            "FAIL [batch {}] {} ({})",
                            bi + 1,
                            result.function,
                            result.detail
                        );
                        any_failed = true;
                    }
                }
                Err(_) => {
                    println!("FAIL [batch {}] <claim thread panicked>", bi + 1);
                    any_failed = true;
                }
            }
        }
        if any_failed {
            eprintln!(
                "claim_executor: batch {} had failures — stopping before dependent batches",
                bi + 1
            );
            break;
        }
    }
    WalkOutcome {
        any_failed,
        batches_run,
    }
}

fn remap_entry_for_temp(source_root: &str, temp_src: &Path, entry: &str) -> PathBuf {
    let prefix = format!("{source_root}/");
    if let Some(suffix) = entry.strip_prefix(&prefix) {
        temp_src.join(suffix)
    } else if let Some(suffix) = entry.strip_prefix("src/v2/") {
        temp_src.join(suffix)
    } else {
        PathBuf::from(entry)
    }
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

fn perturb_function_to_false(path: &Path, function: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let needle_fn = format!("fn {function}(");
    let needle_func = format!("func {function}(");
    let start = match (text.find(&needle_func), text.find(&needle_fn)) {
        (Some(f), _) => f,
        (None, Some(f)) => f,
        (None, None) => return Err(format!("{}: missing function {function}", path.display())),
    };
    let brace = start
        + text[start..]
            .find('{')
            .ok_or_else(|| format!("{}: missing body for {function}", path.display()))?;
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

fn run_perturb_check(
    source_roots: &[String],
    plan_entry: &str,
    plan_function: &str,
) -> Result<ExitCode, ExitCode> {
    let batches = match eval_plan(source_roots, plan_entry, plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: --perturb-check: {msg}");
            return Err(ExitCode::from(2));
        }
    };
    if batches.len() < 2 {
        eprintln!(
            "claim_executor: --perturb-check needs a plan with >= 2 batches to witness the \
             walk halt (got {})",
            batches.len()
        );
        return Err(ExitCode::from(2));
    }
    let (gating_entry, gating_function) = match batches[0].first() {
        Some(Runnable::SingleClaim { entry, function }) if !entry.is_empty() => {
            (entry.clone(), function.clone())
        }
        _ => {
            eprintln!(
                "claim_executor: --perturb-check: batch 1 has no plantable SingleClaim gating node"
            );
            return Err(ExitCode::from(2));
        }
    };

    let primary = &source_roots[0];
    let tmp = std::env::temp_dir().join(format!("claim-executor-perturb-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let temp_src = tmp.join("src");
    if let Err(e) = copy_dir_all(Path::new(primary), &temp_src) {
        eprintln!("claim_executor: --perturb-check: {e}");
        return Err(ExitCode::from(2));
    }

    let gating_path = remap_entry_for_temp(primary, &temp_src, &gating_entry);
    if let Err(e) = perturb_function_to_false(&gating_path, &gating_function) {
        let _ = fs::remove_dir_all(&tmp);
        eprintln!("claim_executor: --perturb-check: plant gating->false failed: {e}");
        return Err(ExitCode::from(2));
    }

    let temp_root = temp_src.to_string_lossy().into_owned();
    let remap_root = |root: &str| -> String {
        if root == primary.as_str() {
            temp_root.clone()
        } else {
            root.to_string()
        }
    };
    let remapped: Vec<Vec<Runnable>> = batches
        .iter()
        .map(|batch| {
            batch
                .iter()
                .map(|r| match r {
                    Runnable::SingleClaim { entry, function } => Runnable::SingleClaim {
                        entry: if entry.is_empty() {
                            entry.clone()
                        } else {
                            remap_entry_for_temp(primary, &temp_src, entry)
                                .to_string_lossy()
                                .into_owned()
                        },
                        function: function.clone(),
                    },
                    Runnable::DiscoveryBatch {
                        source_roots: roots,
                        scan_dirs,
                        explicit_entries,
                        skip_unaffected_node_frontier,
                    } => Runnable::DiscoveryBatch {
                        source_roots: roots.iter().map(|r| remap_root(r)).collect(),
                        scan_dirs: scan_dirs.iter().map(|d| remap_root(d)).collect(),
                        explicit_entries: explicit_entries.clone(),
                        skip_unaffected_node_frontier: *skip_unaffected_node_frontier,
                    },
                })
                .collect()
        })
        .collect();

    eprintln!(
        "claim_executor: --perturb-check: planted batch-1 gating witness `{}` -> false; re-walking",
        gating_function
    );
    let outcome = run_walk(&[temp_root], &remapped, 1);
    let _ = fs::remove_dir_all(&tmp);

    if outcome.any_failed && outcome.batches_run == 1 {
        eprintln!(
            "claim_executor: --perturb-check OK: gating batch-1 false -> run failed closed AND \
             walk halted before batch 2 (batches_run=1 of {})",
            batches.len()
        );
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!(
            "claim_executor: --perturb-check FAIL: expected fail-closed + halt-at-batch-1, got \
             any_failed={} batches_run={} (of {})",
            outcome.any_failed,
            outcome.batches_run,
            batches.len()
        );
        Ok(ExitCode::from(1))
    }
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut plan_entry: Option<String> = None;
    let mut plan_function = "bre_claim_batches".to_string();
    let mut notice_title: Option<String> = None;
    let mut perturb_check = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--source-root" => {
                i += 1;
                source_roots.push(require_value(&args, i, "--source-root")?);
            }
            "--plan-entry" => {
                i += 1;
                plan_entry = Some(require_value(&args, i, "--plan-entry")?);
            }
            "--plan-function" => {
                i += 1;
                plan_function = require_value(&args, i, "--plan-function")?;
            }
            "--notice-title" => {
                i += 1;
                notice_title = Some(require_value(&args, i, "--notice-title")?);
            }
            "--perturb-check" => perturb_check = true,
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    if source_roots.is_empty() {
        eprintln!("claim_executor: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let plan_entry = match plan_entry {
        Some(e) => e,
        None => {
            eprintln!("claim_executor: --plan-entry <file.dag> is required");
            return Err(ExitCode::from(2));
        }
    };

    if perturb_check {
        return run_perturb_check(&source_roots, &plan_entry, &plan_function);
    }

    let batches = match eval_plan(&source_roots, &plan_entry, &plan_function) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };

    eprintln!(
        "claim_executor: [{}] executor plan = {} batch(es) from {}::{}",
        notice_title.as_deref().unwrap_or("ci floor"),
        batches.len(),
        plan_entry,
        plan_function
    );

    if batches.is_empty() {
        eprintln!("claim_executor: executor plan produced 0 batches — failing closed");
        return Err(ExitCode::from(1));
    }

    let spawn_width = match eval_spawn_width(&source_roots, &plan_entry, &plan_function) {
        Ok(w) => w,
        Err(msg) => {
            eprintln!("claim_executor: {msg}");
            return Err(ExitCode::from(1));
        }
    };
    eprintln!(
        "claim_executor: spawn_width={} from {}::{}",
        spawn_width,
        plan_entry,
        spawn_width_function_name(&plan_function)
            .as_deref()
            .unwrap_or("<serial>")
    );

    let outcome = run_walk(&source_roots, &batches, spawn_width);
    match peak_rss_bytes() {
        Some(bytes) => eprintln!(
            "[measurement] floor peak RSS: {bytes} bytes (VmHWM) at spawn_width={spawn_width}"
        ),
        None => eprintln!(
            "[measurement] floor peak RSS: unavailable (no /proc/self/status) at spawn_width={spawn_width}"
        ),
    }
    // [measurement] WHOLE-TREE cgroup peak — the SOUND placement divisor input. SELF-RSS above
    // (VmHWM) excludes child rustc/sccache PIDs; cgroup v2 `memory.peak` at the binding-cap
    // ancestor is hierarchical and captures them. Paired with the sccache-server cgroup path so
    // the "accounted exactly once" classification (inside-subtree vs sibling host overhead) is a
    // read of these two paths, not a brittle in-process prefix check. Runtime-harmless read-only.
    match cgroup_peak_pids_at_binding_ancestor() {
        Some((peak, pids, anc_rel)) => {
            let sccache = sccache_server_cgroup_rel()
                .unwrap_or_else(|| "not-found (treat as fixed host overhead)".to_string());
            eprintln!(
                "[measurement] floor cgroup peak: {peak} bytes (memory.peak @ {anc_rel}) pids.current={pids} at spawn_width={spawn_width}; sccache-server cgroup: {sccache}"
            );
        }
        None => eprintln!(
            "[measurement] floor cgroup peak: unavailable (no binding-cap cgroup or memory.peak) at spawn_width={spawn_width}"
        ),
    }
    if outcome.any_failed {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(code) => code,
    }
}
