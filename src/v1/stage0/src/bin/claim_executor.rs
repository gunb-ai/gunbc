#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use std::rc::Rc;
use std::time::Instant;
#[cfg(test)]
use v1_compiler::cli_run::workspace_root;
use v1_compiler::cli_run::{
    build_floor_discovery_request, make_eval_context, resolve_entry_graph_shared, run_value,
    verify_floor_discovery_terminal_for_coordinator, PhaseProfile,
};
use v1_compiler::memory_governor::{binding_cap_cgroup_dir, leaf_cgroup_dir, mem_total_bytes};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

/// Where a resolved floor-batch clamp came from — seed projection of the `authority: DeclarationRef`
/// field on `std.realization_schedule` `RunnableBatchClamp`.
///
/// Two independently owned populations feed one aligned clamp list: the positional
/// `gunbc.ci_spec` rows for ordinary batches, and the scoped witness batch's own row in
/// `gunbc.ci_layer_roots`. This executor used to reconstruct the origin from the LIST POSITION and
/// spell it in the refusal's format string, which is how a breach came to report `clamp_ms=360000`
/// while citing `gunbc_ci_floor_batch_clamp_params[0]` — a row whose overhead is 240 seconds. The
/// authority now travels WITH the value; the index is carried beside it because an offset into a
/// list is the one part of the citation no symbol can name.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum FloorBatchClampAuthority {
    PositionalCiSpecClamp {
        module_path: String,
        decl_name: String,
        index: usize,
    },
    ScopedBatchOwnedClamp {
        batch_id: String,
        module_path: String,
        decl_name: String,
    },
}

/// A clamp plus the declaration that produced it. Constructing one without an authority is not
/// expressible, which is what keeps the refusal's citation and its number from drifting apart.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ResolvedFloorBatchClamp {
    overhead_ms: u128,
    per_unit_ms: u128,
    authority: FloorBatchClampAuthority,
}

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority:
/// `gunbc.ci_spec.gunbc_ci_floor_batch_stop_policy_claim_executor_seed_note`,
/// type `gunbc.ci_spec.FloorBatchStopPolicy`):
/// seed-side enum mirror + `run_walk` consumer for the event-scoped batch halt;
/// policy mapping and plan roster enrollment are delegated to `.dag` eval
/// (`gunbc_ci_floor_batch_stop_policy_for_github_event`,
/// `gunbc_ci_floor_plan_uses_batch_stop_policy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum FloorBatchStopPolicy {
    StopBeforeDependents,
    FullLedger,
}

/// The residency class a runnable declares (`std.realization_schedule`
/// `RunnableMemoryClass`). A structural marker, not a quantity — the operator ruling
/// that retired predicted-peak byte constants stands, and this carries only the
/// Negligible/Substantial fact co-residence structure keys on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum ParsedMemoryClass {
    Negligible,
    Substantial,
}

/// THE WHOLE PROFILE, retained. `std.realization_schedule.RunnableResourceProfile`
/// declares four facts; the parser used to keep two — `heavy_whole_tree_resolve` (as
/// `use_walk_memo`) and `execution_mode` — and drop `spawns_host_compiler` and the
/// memory class at parse time. That was invisible while only the ordinary batch path
/// consumed profiles, because the ordinary path happens to need exactly the two that
/// survived. It stops being invisible the moment ONE executor serves both populations:
/// a shared `run_stage` cannot enforce a stage's declared resource contract against
/// facts the parse threw away, so the arm-time validator could wall the heavy-resolve
/// case and nothing else (review 2026-07-30, naming this the run_stage prerequisite).
///
/// Retained as a unit rather than as four sibling fields on the runnable so that adding
/// a fifth profile fact is one edit here, not a fifth thing to remember to thread.
/// Whether the plan actually DESCRIBED this runnable's resources, or the parse supplied
/// fail-closed values because no profile was present. Hermetic is genuinely conservative
/// for effects, but `heavy_whole_tree_resolve: false`, `spawns_host_compiler: false` and
/// `memory: Negligible` are OPTIMISTIC assertions about work nobody described — and once
/// a profileless ClaimRef becomes the same SingleClaim variant an explicitly-profiled one
/// does, the stage validator cannot tell "declared Negligible" from "nothing declared"
/// (review 2026-07-31). A pure but whole-tree-heavy ClaimRef would enter a stage looking
/// negligible. Absence is therefore its own state, and stages refuse it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum ParsedProfileProvenance {
    Declared,
    Undeclared,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
struct ParsedRunnableProfile {
    provenance: ParsedProfileProvenance,
    heavy_whole_tree_resolve: bool,
    spawns_host_compiler: bool,
    memory: ParsedMemoryClass,
    execution_mode: ExecutionMode,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ScopedScheduleEntry {
    entry: String,
    function: String,
    witness_kind: String,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum ScopedProcessIsolation {
    SharedWalkProcess,
    SequentialChildProcess,
    FreshJobProcess,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum ScopedWitnessExecutionAuthority {
    InheritedWalkSourceRoots,
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

/// Path-valued arguments resolve against the PROCESS CWD at the CLI boundary, refusing
/// on a nonexistent path — never falling back to the compile-time-baked workspace root
/// (`v1_compiler::cli_run::resolve_cli_path_arg`; DESIGN §5 fail-open closed there).
fn require_path_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    let given = require_value(args, idx, flag)?;
    match v1_compiler::cli_run::resolve_cli_path_arg("claim_executor", flag, &given) {
        Ok(resolved) => Ok(resolved),
        Err(msg) => {
            eprintln!("{msg}");
            Err(ExitCode::from(2))
        }
    }
}

/// Parse the plan-carried finalization VALUE (walk_finalization_note). The executor
/// never selects finalization by a plan function's spelling, and — since the carrier
/// became `WalkPlan<F>` — it does not need to care which instantiation produced the
/// value either: ONE parser reads both. An unrecognized shape is a hard error, and that
/// refusal is load-bearing rather than belt-and-braces: the plan function's declared
/// return type does NOT bound what its body returns (the typechecker does not check
/// return position), so this parser and the enrolled value witnesses are what actually
/// stop a plan from carrying the wrong finalization family.
///
/// `Nat` reaches the interpreter as a native `Int` (the numeric tower is grounded), so
/// a negative value cannot arrive from a well-typed plan; it is still refused rather
/// than carried into a count comparison it could only lose.
/// Transported semantic resolve obligation — population decoded from
/// `WalkPlan.finalization.expected_resolve_obligations`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TransportedObligation {
    identity: String,
    entry: String,
    function: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FloorFinalization {
    expected_obligations: Vec<TransportedObligation>,
}

impl FloorFinalization {
    /// Derived roster size — never a stored count literal (DESIGN §5).
    #[allow(dead_code)]
    fn declared_resolve_count(&self) -> i64 {
        self.expected_obligations.len() as i64
    }
}

#[cfg(test)]
fn classify_witness_expectations(
    outcomes: &[DiscoveryWitnessOutcome],
    expected_red: &[(String, String)],
) -> WitnessExpectationTally {
    classify_witness_expectations_in(outcomes, expected_red, &[])
}

const SCOPED_WITNESS_RECEIPT_PATH: &str = "target/scoped-witness-execution-receipt.tsv";
const SCOPED_WITNESS_RECEIPT_HEADER: &str =
    "head_sha\tbatch_id\tsource_roots_digest\tentry\tfunction\twitness_kind\toutcome\tdetail";

fn initialize_scoped_witness_receipt() -> Result<(), String> {
    let path = Path::new(SCOPED_WITNESS_RECEIPT_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    std::fs::write(path, SCOPED_WITNESS_RECEIPT_HEADER)
        .map_err(|e| format!("write {}: {e}", path.display()))
}

/// One read of the whole-tree job footprint and its budget context, taken in a single cgroup walk so
/// a consumer (placement divisor, compile-jobs derive) can parse usage AND budget from one emitted
/// line without re-walking. `cap_bytes` is the tightest numeric `memory.max` on the leaf→root walk
/// (`None` = uncapped / RAM-bound); `host_ram` is the physical-RAM budget that binds when uncapped;
/// `sccache_under_leaf` classifies the sccache server's cgroup as a descendant (already counted in
/// `leaf_peak`) vs a sibling (must be subtracted as `host_fixed_overhead`) — the "accounted exactly
/// once" decision, read from paths rather than guessed.
struct CgroupMeasurement {
    leaf_peak: u64,
    leaf_rel: String,
    cap_bytes: Option<u64>,
    host_ram: Option<u64>,
    pids_current: u64,
    pids_max: String,
    sccache_rel: Option<String>,
    sccache_under_leaf: bool,
}

fn cgroup_job_measurement() -> Option<CgroupMeasurement> {
    let leaf = leaf_cgroup_dir()?;
    let leaf_peak = fs::read_to_string(leaf.join("memory.peak"))
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    let leaf_rel = leaf
        .strip_prefix("/sys/fs/cgroup")
        .map(|p| format!("/{}", p.to_string_lossy().trim_start_matches('/')))
        .unwrap_or_else(|_| leaf.to_string_lossy().into_owned());
    let cap_bytes = binding_cap_cgroup_dir()
        .and_then(|d| fs::read_to_string(d.join("memory.max")).ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let pids_current = fs::read_to_string(leaf.join("pids.current"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let pids_max = fs::read_to_string(leaf.join("pids.max"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let sccache_rel = sccache_server_cgroup_rel();
    // Descendant iff sccache's cgroup is the leaf or strictly under it — a PATH-COMPONENT prefix,
    // not a bare string prefix, so a sibling like `<leaf>-other.service` is NOT misclassified as a
    // descendant (that would under-count host overhead — the fail-OPEN direction). When the leaf is
    // the cgroup root (`leaf_r` empty — e.g. a single-cgroup container) everything is a descendant.
    // On the real fleet the leaf is the runner-service cgroup and sccache is a sibling service
    // cgroup, so this is `false` (subtract as host_fixed_overhead).
    let leaf_r = leaf_rel.trim_start_matches('/').to_string();
    let sccache_under_leaf = sccache_rel
        .as_deref()
        .map(|r| {
            let r = r.trim_start_matches('/');
            leaf_r.is_empty() || r == leaf_r || r.starts_with(&format!("{leaf_r}/"))
        })
        .unwrap_or(false);
    Some(CgroupMeasurement {
        leaf_peak,
        leaf_rel,
        cap_bytes,
        host_ram: mem_total_bytes(),
        pids_current,
        pids_max,
        sccache_rel,
        sccache_under_leaf,
    })
}

/// Verify each declared release artifact exists, is executable, and is non-empty — failing CLOSED
/// (exit 1) with the GitHub-Actions `::error::` annotation on the first violation. This is the
/// in-binary home of what was previously inline `[ -x ]` / `[ -s ]` shell in the generated ci.yml
/// (DESIGN §5 fail-open guard: an sccache-served truncated/empty cached artifact after a
/// `successful` build). The artifact paths are authored by the .dag spec and passed positionally.
fn verify_build_artifacts(paths: &[String]) -> Result<ExitCode, ExitCode> {
    use std::os::unix::fs::PermissionsExt;
    if paths.is_empty() {
        eprintln!("claim_executor: --verify-build-artifacts requires at least one artifact path");
        return Err(ExitCode::from(2));
    }
    for path in paths {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path.as_str());
        match std::fs::metadata(path) {
            Ok(meta) => {
                let executable = meta.permissions().mode() & 0o111 != 0;
                if !meta.is_file() || !executable {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' absent or not \
                         executable after a 'successful' build (sccache/cache corruption — DESIGN \
                         §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
                if meta.len() == 0 {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' is zero-byte after \
                         a 'successful' build (sccache served a truncated/empty cached artifact — \
                         DESIGN §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
            }
            Err(_) => {
                eprintln!(
                    "::error::build verification: declared artifact '{name}' absent or not \
                     executable after a 'successful' build (sccache/cache corruption — DESIGN §5 \
                     fail-open); failing closed: {path}"
                );
                return Err(ExitCode::from(1));
            }
        }
    }
    eprintln!(
        "claim_executor: build-artifact verification passed ({} declared release binar{} present \
         + non-empty)",
        paths.len(),
        if paths.len() == 1 { "y" } else { "ies" }
    );
    Ok(ExitCode::SUCCESS)
}

/// Single authority for the one-line whole-tree cgroup measurement, shared by the floor run and the
/// standalone `--measure-cgroup-peak` mode so the `ci` and `rust_tests` jobs report an
/// identically-shaped line. `context` distinguishes the call site.
fn emit_cgroup_measurement(context: &str) {
    let emoji = std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true");
    match cgroup_job_measurement() {
        Some(m) => {
            let label = format!("cgroup peak @ {} ({context})", m.leaf_rel);
            eprintln!(
                "{}",
                v1_compiler::cli_run::render_peak_rss_line_mirror(&label, Some(m.leaf_peak), emoji)
            );
            // Diagnostic companions (cap/pids/sccache) stay as Ambient detail beside the
            // Measured peak — not the old raw-byte `[measurement]` dump. Placement still
            // reads the typed `cgroup_job_measurement` fact, not this prose.
            let cap = match m.cap_bytes {
                Some(b) => format!("{b} bytes"),
                None => "uncapped(RAM-bound)".to_string(),
            };
            let host_ram = m
                .host_ram
                .map(|b| format!("{b} bytes"))
                .unwrap_or_else(|| "unknown".to_string());
            let sccache = match (&m.sccache_rel, m.sccache_under_leaf) {
                (Some(r), true) => format!("{r} (descendant: counted in memory.peak)"),
                (Some(r), false) => format!("{r} (sibling: subtract as host_fixed_overhead)"),
                (None, _) => "not-found (treat as fixed host overhead)".to_string(),
            };
            eprintln!(
                "  memory.max={cap} host_ram={host_ram} pids_current={pc} pids_max={pm} sccache-server-cgroup={sccache}",
                pc = m.pids_current,
                pm = m.pids_max
            );
        }
        None => {
            let label = format!("cgroup peak ({context})");
            eprintln!(
                "{}",
                v1_compiler::cli_run::render_peak_rss_line_mirror_with_cause(
                    &label,
                    None,
                    "no leaf cgroup or memory.peak unreadable; kernel < 5.19?",
                    emoji,
                )
            );
        }
    }
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
        if fs::read_to_string(p.join("comm"))
            .unwrap_or_default()
            .trim()
            != "sccache"
        {
            continue;
        }
        if let Some(rel) = fs::read_to_string(p.join("cgroup")).ok().and_then(|cg| {
            cg.lines()
                .find_map(|l| l.strip_prefix("0::"))
                .map(|s| s.trim().to_string())
        }) {
            return Some(rel);
        }
    }
    None
}

/// ONE stage's own receipt, written before the next stage begins. The aggregate receipt
/// below cannot substitute for it: written only after the whole sequence, it does not
/// exist for stage N while stage N+1 is running, and a process death between the two
/// loses every stage that had in fact completed. Per-stage, the disk state answers
/// "which stages ran, and what did each cost" at any instant.
///
/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: `std.types` `path_segment_is_safe`,
/// the single law for "safe as ONE path segment"). This mirrors that predicate clause for
/// clause because the executor is the Rust seed and cannot call the `.dag` surface at the
/// point it observes the environment. It is a REALIZATION of that authority, not a second
/// rule: if the two ever disagree, the `.dag` predicate is right and this is the defect.
/// dissolve-on: the executor's env observation running as a modeled effect, at which point
/// the branding constructor `gunbc.merge_admission.walk_attempt_id` is the only gate and
/// this function deletes.
fn walk_attempt_id_segment_is_safe(raw: &str) -> bool {
    !(raw.is_empty()
        || raw == "."
        || raw == ".."
        || raw.contains('/')
        || raw.contains('\\')
        || raw.contains('\n')
        || raw.contains('\r')
        || raw.contains('\0'))
}

/// Observe the walk-attempt identity, per `gunbc.merge_admission`
/// `merge_admission_attempt_scope_note`: derived from GITHUB_RUN_ID + GITHUB_RUN_ATTEMPT +
/// GITHUB_JOB on GitHub, or supplied explicitly as GUNBC_WALK_ATTEMPT_ID off it.
///
/// REFUSES rather than defaulting. The ruling names the exact failure this prevents: "never
/// a silent constant like a bare local, which would make every local run one attempt and
/// the wrong-attempt refusal unreachable off CI". A default here would not be a convenience,
/// it would disable the identity check everywhere except GitHub — the absorbing fallback in
/// its purest form, since nothing would ever report that identity had been fabricated.
///
/// GUNBC_WALK_ATTEMPT_ID is NOT an escape hatch: it supplies a required input that the
/// environment did not, and a value that fails the segment law still refuses. There is no
/// value of it that makes a refusal not fire.
fn observe_walk_attempt_id() -> Result<String, String> {
    compose_walk_attempt_id(
        &std::env::var("GUNBC_WALK_ATTEMPT_ID").unwrap_or_default(),
        &std::env::var("GITHUB_RUN_ID").unwrap_or_default(),
        &std::env::var("GITHUB_RUN_ATTEMPT").unwrap_or_default(),
        &std::env::var("GITHUB_JOB").unwrap_or_default(),
    )
}

/// The PURE half, split from the observation above for the same reason
/// `gunbc.merge_admission_produce` splits them ("Pure composition of the walk-attempt
/// identity from its parts; the ENV OBSERVATION lives with the wet entry"). It is also what
/// makes the refusals reachable by a test: process env is global, so a test that set it
/// would race every other test in the binary, and a rule that can only be exercised by a
/// racing test is a rule nobody checks.
fn compose_walk_attempt_id(
    explicit: &str,
    run_id: &str,
    run_attempt: &str,
    job: &str,
) -> Result<String, String> {
    if !explicit.trim().is_empty() {
        return if walk_attempt_id_segment_is_safe(explicit) {
            Ok(explicit.to_string())
        } else {
            Err(format!(
                "GUNBC_WALK_ATTEMPT_ID={explicit:?} is not a safe path segment (std.types path_segment_is_safe: non-empty, not `.`/`..`, no `/` `\\` CR LF NUL)"
            ))
        };
    }
    if run_id.is_empty() || run_attempt.is_empty() || job.is_empty() {
        return Err(
            "no walk-attempt identity: GITHUB_RUN_ID/GITHUB_RUN_ATTEMPT/GITHUB_JOB are not all \
             present and GUNBC_WALK_ATTEMPT_ID was not supplied. On-success stages write \
             attempt-scoped receipts, so an unidentified walk refuses here rather than \
             stamping a receipt no consumer could tell apart from another run's"
                .to_string(),
        );
    }
    let composed = format!("{run_id}-{run_attempt}-{job}");
    if walk_attempt_id_segment_is_safe(&composed) {
        Ok(composed)
    } else {
        Err(format!(
            "composed walk-attempt identity {composed:?} is not a safe path segment (std.types path_segment_is_safe)"
        ))
    }
}

const FLOOR_WORKER_TERMINAL_ENV: &str = "GUNBC_FLOOR_WORKER_TERMINAL_RECEIPT";
const FLOOR_PHASE_JOURNAL_ENV: &str = "GUNBC_FLOOR_PHASE_JOURNAL";
const FLOOR_WORKER_OBSERVATION_RECEIPT_PATH: &str = "target/floor-worker-observation-receipt.tsv";
const FLOOR_WET_WITNESS_ROW_OUTCOME_RECEIPT_PATH: &str =
    "target/floor-wet-witness-row-outcome-receipt.tsv";

fn wet_witness_row_outcome_replay_line(
    batch: &str,
    entry: &str,
    function: &str,
    outcome: &str,
    detail: &str,
) -> String {
    format!(
        "[wet-witness-row-outcome] batch={batch} entry={entry} function={function} outcome={outcome} detail={detail}"
    )
}

fn collect_wet_witness_row_outcome_replay_lines(path: &Path) -> Result<Vec<String>, String> {
    let body = fs::read_to_string(path).map_err(|e| {
        format!(
            "read wet witness row-outcome receipt {}: {e}",
            path.display()
        )
    })?;
    let mut lines = Vec::new();
    for line in body.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        let (batch, entry, function, outcome, detail) = match parts.len() {
            4 => (parts[0], parts[1], parts[2], parts[3], ""),
            5 => (parts[0], parts[1], parts[2], parts[3], parts[4]),
            _ => {
                return Err(format!(
                    "malformed wet witness row-outcome line (need 4-5 cols): {line}"
                ));
            }
        };
        lines.push(wet_witness_row_outcome_replay_line(
            batch, entry, function, outcome, detail,
        ));
    }
    Ok(lines)
}

fn replay_floor_wet_witness_row_outcomes_from_receipt(path: &Path) -> Result<usize, String> {
    let lines = collect_wet_witness_row_outcome_replay_lines(path)?;
    for line in &lines {
        eprintln!("{line}");
    }
    Ok(lines.len())
}

fn replay_ordinary_floor_wet_witness_row_outcomes() {
    let path = Path::new(FLOOR_WET_WITNESS_ROW_OUTCOME_RECEIPT_PATH);
    match replay_floor_wet_witness_row_outcomes_from_receipt(path) {
        Ok(count) => {
            eprintln!(
                "[wet-witness-row-outcome] coordinator replayed {count} row(s) from {}",
                path.display()
            );
            append_floor_phase_journal(
                "wet-witness-row-outcome-replay",
                "completed",
                &format!("row_count={count} path={}", path.display()),
            );
            if let Ok(lines) = collect_wet_witness_row_outcome_replay_lines(path) {
                for line in lines {
                    append_floor_phase_journal("wet-witness-row-outcome-replay", "row", &line);
                }
            }
        }
        Err(msg) if path.exists() => {
            eprintln!("claim_executor: wet witness row-outcome coordinator replay refused: {msg}");
            append_floor_phase_journal("wet-witness-row-outcome-replay", "refused", &msg);
        }
        Err(_) => {
            eprintln!(
                "claim_executor: wet witness row-outcome receipt absent at {} — per-row wet batch outcomes unobservable",
                path.display()
            );
            append_floor_phase_journal(
                "wet-witness-row-outcome-replay",
                "absent",
                &format!("path={}", path.display()),
            );
        }
    }
}

fn append_floor_phase_journal(phase: &str, state: &str, detail: &str) {
    let Some(path) = std::env::var_os(FLOOR_PHASE_JOURNAL_ENV) else {
        return;
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "claim_executor: create floor phase journal directory {}: {e}",
                parent.display()
            );
            return;
        }
    }
    let mut file = match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "claim_executor: open floor phase journal {}: {e}",
                path.display()
            );
            return;
        }
    };
    let unix_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let clean_detail = detail.replace(['\t', '\r', '\n'], " ");
    let row = format!(
        "{unix_millis}\t{}\t{phase}\t{state}\t{clean_detail}\n",
        std::process::id()
    );
    use std::io::Write as _;
    if let Err(e) = file
        .write_all(row.as_bytes())
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_data())
    {
        eprintln!(
            "claim_executor: persist floor phase journal {}: {e}",
            path.display()
        );
    }
}

struct ObservedFloorWorker {
    worker: String,
    termination: ProcessTermination,
    terminal_receipt: FloorWorkerTerminalReceipt,
}

/// Seed projection of `std.process_termination` `ProcessTermination` — how an observed process
/// ended. One carrier for both places the executor observes a child: the floor-worker
/// coordinator (which spawns workers directly and reads an `ExitStatus`) and the native
/// bundle transport (which reads the termination the interpreter transport carries).
/// Named for the concept rather than for either subject so the second consumer did not
/// mint a second spelling of it.
#[derive(Debug, PartialEq, Eq, Clone)]
enum ProcessTermination {
    Exited(i32),
    Signaled(i32),
    Unobserved,
}

#[derive(Debug, PartialEq, Eq)]
enum FloorWorkerTerminalReceipt {
    Observed(FloorWorkerTerminalReport),
    Missing,
}

#[derive(Debug, PartialEq, Eq)]
enum FloorWorkerTerminalReport {
    Completed(String),
    Refused(String),
    Failed(String),
    Malformed(String),
}

struct DerivedFloorWorkerOutcome {
    label: &'static str,
    detail: String,
}
fn exit_status_termination(status: &ExitStatus) -> ProcessTermination {
    if let Some(code) = status.code() {
        return ProcessTermination::Exited(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(signal) = status.signal() {
            return ProcessTermination::Signaled(signal);
        }
    }
    ProcessTermination::Unobserved
}

fn floor_worker_termination_label(termination: &ProcessTermination) -> String {
    match termination {
        ProcessTermination::Exited(code) => format!("exited:{code}"),
        ProcessTermination::Signaled(signal) => format!("signaled:{signal}"),
        ProcessTermination::Unobserved => "termination-unobserved".to_string(),
    }
}

fn floor_worker_terminal_report_label(report: &FloorWorkerTerminalReport) -> &'static str {
    match report {
        FloorWorkerTerminalReport::Completed(_) => "completed",
        FloorWorkerTerminalReport::Refused(_) => "refused",
        FloorWorkerTerminalReport::Failed(_) => "failed",
        FloorWorkerTerminalReport::Malformed(_) => "malformed",
    }
}

fn floor_worker_observation_outcome(row: &ObservedFloorWorker) -> DerivedFloorWorkerOutcome {
    let termination = floor_worker_termination_label(&row.termination);
    match &row.terminal_receipt {
        FloorWorkerTerminalReceipt::Missing => DerivedFloorWorkerOutcome {
            label: "died-without-terminal-receipt",
            detail: format!(
                "worker `{}` terminated as {termination}; no terminal receipt was observed",
                row.worker
            ),
        },
        FloorWorkerTerminalReceipt::Observed(report) => match (&row.termination, report) {
            (ProcessTermination::Exited(0), FloorWorkerTerminalReport::Completed(detail)) => {
                DerivedFloorWorkerOutcome {
                    label: "completed",
                    detail: detail.clone(),
                }
            }
            (ProcessTermination::Exited(code), FloorWorkerTerminalReport::Refused(detail))
                if *code != 0 =>
            {
                DerivedFloorWorkerOutcome {
                    label: "refused",
                    detail: detail.clone(),
                }
            }
            (_, FloorWorkerTerminalReport::Failed(detail))
            | (_, FloorWorkerTerminalReport::Malformed(detail)) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: detail.clone(),
            },
            (ProcessTermination::Signaled(signal), _) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: format!(
                    "worker `{}` reported {} but died from signal {signal}",
                    row.worker,
                    floor_worker_terminal_report_label(report)
                ),
            },
            (ProcessTermination::Unobserved, _) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: format!(
                    "worker `{}` reported {} but process termination was unobserved",
                    row.worker,
                    floor_worker_terminal_report_label(report)
                ),
            },
            (ProcessTermination::Exited(code), _) => DerivedFloorWorkerOutcome {
                label: "failed",
                detail: format!(
                    "worker `{}` report {} contradicted exit code {code}",
                    row.worker,
                    floor_worker_terminal_report_label(report)
                ),
            },
        },
    }
}

fn observe_floor_worker(
    worker: &str,
    status: ExitStatus,
    terminal_path: &Path,
) -> ObservedFloorWorker {
    let termination = exit_status_termination(&status);
    let terminal = fs::read_to_string(terminal_path).ok();
    let terminal_receipt = match terminal {
        None => FloorWorkerTerminalReceipt::Missing,
        Some(body) => {
            let line = body.trim_end_matches(['\r', '\n']);
            let (label, terminal_detail) = line.split_once('\t').unwrap_or((line, ""));
            let report = match label {
                "completed" => FloorWorkerTerminalReport::Completed(terminal_detail.to_string()),
                "refused" => FloorWorkerTerminalReport::Refused(terminal_detail.to_string()),
                "failed" => FloorWorkerTerminalReport::Failed(terminal_detail.to_string()),
                _ => FloorWorkerTerminalReport::Malformed(format!(
                    "unknown worker terminal report `{label}`: {terminal_detail}"
                )),
            };
            FloorWorkerTerminalReceipt::Observed(report)
        }
    };
    ObservedFloorWorker {
        worker: worker.to_string(),
        termination,
        terminal_receipt,
    }
}

fn append_floor_worker_observation(row: &ObservedFloorWorker) -> Result<(), String> {
    let path = Path::new(FLOOR_WORKER_OBSERVATION_RECEIPT_PATH);
    let needs_header = !path.exists();
    let mut options = fs::OpenOptions::new();
    options.create(true).append(true);
    let mut file = options
        .open(path)
        .map_err(|e| format!("open floor worker observation {}: {e}", path.display()))?;
    use std::io::Write as _;
    if needs_header {
        writeln!(
            file,
            "worker\ttermination\tterminal_receipt\tterminal_report\toutcome\tdetail"
        )
        .map_err(|e| format!("write floor worker observation header: {e}"))?;
    }
    let outcome = floor_worker_observation_outcome(row);
    let (receipt_label, report_label) = match &row.terminal_receipt {
        FloorWorkerTerminalReceipt::Missing => ("missing", "absent"),
        FloorWorkerTerminalReceipt::Observed(report) => {
            ("observed", floor_worker_terminal_report_label(report))
        }
    };
    let clean_detail = outcome.detail.replace(['\t', '\r', '\n'], " ");
    writeln!(
        file,
        "{}\t{}\t{}\t{}\t{}\t{}",
        row.worker,
        floor_worker_termination_label(&row.termination),
        receipt_label,
        report_label,
        outcome.label,
        clean_detail
    )
    .map_err(|e| format!("write floor worker observation row: {e}"))
}

fn floor_worker_succeeded(row: &ObservedFloorWorker) -> bool {
    floor_worker_observation_outcome(row).label == "completed"
}

fn journal_floor_worker_observation(row: &ObservedFloorWorker) {
    let outcome = floor_worker_observation_outcome(row);
    append_floor_phase_journal(
        "coordinator-observation",
        outcome.label,
        &format!(
            "worker={} termination={} detail={}",
            row.worker,
            floor_worker_termination_label(&row.termination),
            outcome.detail.replace(['\t', '\r', '\n'], " ")
        ),
    );
}

fn source_roots_from_executor_args(args: &[String]) -> Result<Vec<String>, String> {
    let mut roots = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--source-root" {
            i += 1;
            let Some(root) = args.get(i) else {
                return Err("claim_executor: --source-root requires a value".to_string());
            };
            roots.push(root.clone());
        }
        i += 1;
    }
    if roots.is_empty() {
        return Err("claim_executor: --source-root is required".to_string());
    }
    Ok(roots)
}

fn spawn_floor_worker(
    base_args: &[String],
    role: &str,
    batch_id: Option<&str>,
    ordinal: usize,
    walk_attempt_id: &str,
) -> Result<ObservedFloorWorker, String> {
    let worker = match batch_id {
        Some(id) => format!("scoped:{id}"),
        None => "ordinary".to_string(),
    };
    let terminal_path = PathBuf::from(format!(
        "target/floor-worker-terminal-{ordinal}-{}.tsv",
        worker.replace(':', "-")
    ));
    let _ = fs::remove_file(&terminal_path);
    let exe = std::env::current_exe().map_err(|e| format!("locate claim_executor: {e}"))?;
    let mut command = Command::new(exe);
    command.args(base_args).arg("--floor-worker-role").arg(role);
    if let Some(id) = batch_id {
        command.arg("--scoped-batch-id").arg(id);
    }
    command.env(FLOOR_WORKER_TERMINAL_ENV, &terminal_path);
    command.env("GUNBC_FLOOR_WALK_ATTEMPT_ID", walk_attempt_id);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        // When the thin coordinator is killed by the foreign step timeout, workers must not
        // keep running as orphaned claim_executors (signature 2). SIGTERM on parent death
        // gives the worker a chance to flush its terminal receipt before the runner reaps it.
        unsafe {
            command.pre_exec(|| {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    // The worker's post-walk ledger harvest can spend minutes dropping its memo
    // contexts after the final discovery line.  A blocking `Command::status`
    // made that whole interval silent: the worker's own heartbeat shares its
    // allocator and can stop making progress during the drop, while the thin
    // coordinator had no chance to say that the child was still alive.  Keep
    // the execution and receipt ordering unchanged, but let the coordinator
    // observe the wait and emit a pulse from its independent process.
    let mut child = command
        .spawn()
        .map_err(|e| format!("spawn floor worker `{worker}`: {e}"))?;
    let wait_started = Instant::now();
    // Per-worker, beside `wait_started`, NOT a `static`: a process-global counter is shared by
    // every worker's wait loop, so with N workers each one prints roughly every Nth tick and a
    // given worker can skip all of its own heartbeats. Liveness per worker is the entire fact this
    // line carries, so the counter has to have the same scope as the thing it reports on.
    let mut wait_ticks: u64 = 0;
    let status = loop {
        match child
            .try_wait()
            .map_err(|e| format!("observe floor worker `{worker}`: {e}"))?
        {
            Some(status) => {
                append_floor_phase_journal(
                    "coordinator-wait",
                    "completed",
                    &format!(
                        "worker={worker} elapsed_seconds={}",
                        wait_started.elapsed().as_secs()
                    ),
                );
                break status;
            }
            None => {
                append_floor_phase_journal(
                    "coordinator-wait",
                    "running",
                    &format!(
                        "worker={worker} elapsed_seconds={}",
                        wait_started.elapsed().as_secs()
                    ),
                );
                // CADENCE DIVERGENCE, marked here because this diff creates it: the journal append
                // above runs EVERY iteration, this print runs every tenth. They were one cadence
                // before, so anything that reads these two sites as interchangeable is now wrong.
                // Attach no state write to this site — a snapshot written here is up to five
                // minutes stale, and a consumer polling it for liveness (a stuck-worker detector is
                // the live proposal) would inherit that staleness as its detection floor. The
                // per-iteration journal site is where a progress fact belongs.
                //
                // The wait tick is journaled above on every iteration; the console keeps one line
                // per worker per five minutes rather than one per thirty seconds. Not journal-only, and the
                // difference is deliberate: the journal is an artifact a reader reaches AFTER the
                // run, so a floor that hangs would print nothing for ninety minutes and then a
                // timeout, which reads as a dead process rather than a waiting one. Liveness is
                // the one fact a heartbeat exists to carry, so it stays on the console at the
                // coarsest cadence that still carries it.
                {
                    let n = wait_ticks;
                    wait_ticks += 1;
                    if n % 10 == 0 {
                        eprintln!(
                            "[floor-worker-wait] worker={worker} elapsed_seconds={} state=running",
                            wait_started.elapsed().as_secs()
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
    };
    let observed = observe_floor_worker(&worker, status, &terminal_path);
    // Replay before observation persistence: worker stderr may be dropped on Actions after
    // discovery, and a failed worker is the case where the log otherwise says nothing.
    if worker == "ordinary" {
        replay_ordinary_floor_wet_witness_row_outcomes();
    }
    append_floor_worker_observation(&observed)?;
    let outcome = floor_worker_observation_outcome(&observed);
    // The Actions log transport can drop the worker's inherited stderr after a
    // large discovery run. Persist the derived verdict before reporting it on
    // that channel so the existing always() post-step still surfaces the exact
    // failure arm and detail.
    journal_floor_worker_observation(&observed);
    eprintln!(
        "[floor-worker-observation] worker={} termination={} terminal_receipt={:?} outcome={} detail={}",
        observed.worker,
        floor_worker_termination_label(&observed.termination),
        observed.terminal_receipt,
        outcome.label,
        outcome.detail
    );
    Ok(observed)
}

fn floor_plan_function_arg(args: &[String]) -> Option<&str> {
    args.windows(2)
        .find(|pair| pair[0] == "--plan-function")
        .map(|pair| pair[1].as_str())
}

fn coordinator_terminal_refusal(detail: &str) -> ExitCode {
    append_floor_phase_journal("coordinator-terminal", "refused", detail);
    eprintln!("claim_executor: floor coordinator refusal: {detail}");
    ExitCode::from(1)
}

fn coordinator_report_worker_failure(worker: &ObservedFloorWorker, context: &str) -> ExitCode {
    let outcome = floor_worker_observation_outcome(worker);
    let detail = format!(
        "{context}: worker={} termination={} outcome={} detail={}",
        worker.worker,
        floor_worker_termination_label(&worker.termination),
        outcome.label,
        outcome.detail
    );
    coordinator_terminal_refusal(&detail)
}

fn maybe_run_floor_coordinator(args: &[String]) -> Option<ExitCode> {
    if args.iter().any(|arg| arg == "--floor-worker-role")
        || floor_plan_function_arg(args) != Some("gunbc_ci_floor_plan")
    {
        return None;
    }
    if let Some(parent) = Path::new(FLOOR_WORKER_OBSERVATION_RECEIPT_PATH).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator receipt directory refusal: {e}"
            )));
        }
    }
    let _ = fs::remove_file(FLOOR_WORKER_OBSERVATION_RECEIPT_PATH);
    let _ = fs::remove_file(SCOPED_EXECUTION_REQUESTS_PATH);
    if let Some(path) = std::env::var_os(FLOOR_PHASE_JOURNAL_ENV) {
        let _ = fs::remove_file(path);
    }
    if let Err(msg) = initialize_scoped_witness_receipt() {
        return Some(coordinator_terminal_refusal(&format!(
            "floor coordinator receipt arm refusal: {msg}"
        )));
    }
    let walk_attempt_id = match observe_walk_attempt_id() {
        Ok(id) => id,
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator walk-attempt refusal: {msg}"
            )));
        }
    };
    let ordinary = match spawn_floor_worker(args, "ordinary", None, 0, &walk_attempt_id) {
        Ok(observed) => observed,
        Err(msg) => {
            replay_ordinary_floor_wet_witness_row_outcomes();
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator ordinary-worker refusal: {msg}"
            )));
        }
    };
    if !floor_worker_succeeded(&ordinary) {
        return Some(coordinator_report_worker_failure(
            &ordinary,
            "stopping before scoped workers because ordinary worker did not complete",
        ));
    }
    let source_roots = match source_roots_from_executor_args(args) {
        Ok(roots) => roots,
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator source-roots refusal: {msg}"
            )));
        }
    };
    let excludes = v1_compiler::cli_run::witness_exclusion_substrings();
    let pre_plan_request = match build_floor_discovery_request(
        &source_roots,
        &[],
        &excludes,
        &[],
        "Hermetic",
        &source_roots,
    ) {
        Ok(request) => request,
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator pre-plan request refusal: {msg}"
            )));
        }
    };
    if let Err(msg) =
        verify_floor_discovery_terminal_for_coordinator(&walk_attempt_id, &pre_plan_request)
    {
        return Some(coordinator_terminal_refusal(&format!(
            "floor coordinator snapshot terminal refusal: {msg}"
        )));
    }
    let batch_ids = match read_scoped_execution_requests() {
        Ok(requests) => requests
            .into_iter()
            .map(|request| request.batch_id)
            .collect::<Vec<String>>(),
        Err(msg) => {
            return Some(coordinator_terminal_refusal(&format!(
                "floor coordinator scoped request refusal: {msg}"
            )));
        }
    };
    for (index, batch_id) in batch_ids.iter().enumerate() {
        let scoped = match spawn_floor_worker(
            args,
            "scoped",
            Some(batch_id),
            index.saturating_add(1),
            &walk_attempt_id,
        ) {
            Ok(observed) => observed,
            Err(msg) => {
                return Some(coordinator_terminal_refusal(&format!(
                    "floor coordinator scoped-worker `{batch_id}` refusal: {msg}"
                )));
            }
        };
        if !floor_worker_succeeded(&scoped) {
            return Some(coordinator_report_worker_failure(
                &scoped,
                &format!("stopping after scoped worker `{batch_id}` did not complete"),
            ));
        }
    }
    append_floor_phase_journal(
        "coordinator-terminal",
        "completed",
        &format!(
            "scoped_workers={} walk_attempt_id={walk_attempt_id}",
            batch_ids.len()
        ),
    );
    eprintln!(
        "[floor-coordinator] outcome=completed scoped_workers={} walk_attempt_id={walk_attempt_id}",
        batch_ids.len()
    );
    Some(ExitCode::SUCCESS)
}

fn write_floor_worker_terminal(outcome: &str, detail: &str) -> Result<(), String> {
    let Some(path) = std::env::var_os(FLOOR_WORKER_TERMINAL_ENV) else {
        return Ok(());
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create worker terminal directory {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, format!("{outcome}\t{detail}\n"))
        .map_err(|e| format!("write worker terminal {}: {e}", tmp.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("publish worker terminal {}: {e}", path.display()))
}

const SCOPED_EXECUTION_REQUESTS_PATH: &str = "target/floor-attempts/scoped-execution-requests.json";

/// The exact work a scoped child is asked to do, published by the ordinary worker before the
/// child is spawned.
///
/// A scoped child used to receive the ordinary worker's whole CLI and rebuild its way back to one
/// answer: it re-resolved the plan entry, re-evaluated the plan, and scanned the resulting batches
/// for the single `ScopedWitnessBatch` whose id matched its `--scoped-batch-id`. Every other value
/// that evaluation produced was discarded behind a `!Scoped` guard. This carrier hands the child
/// that one answer directly, with the plan-derived budgets it genuinely reads, so the second
/// resolve and evaluation have nothing left to compute.
///
/// The subject fields are not decoration: a child must refuse a request frozen against a different
/// commit, tree, or tool rather than executing it against whatever it finds on disk.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ScopedExecutionRequest {
    tested_commit: String,
    tested_tree: String,
    tool_identity: String,
    batch_id: String,
    source_roots: Vec<String>,
    source_roots_digest: String,
    entries: Vec<ScopedScheduleEntry>,
    scan_dirs: Vec<String>,
    execution_authority: ScopedWitnessExecutionAuthority,
    profile: ParsedRunnableProfile,
    clamp: ResolvedFloorBatchClamp,
    process_isolation: ScopedProcessIsolation,
    /// Plan-derived and read by the child while executing rows. Carried here because the child no
    /// longer evaluates the plan that produced them.
    fast_lane_eval_budget_ms: Option<u64>,
    ordinary_budget_ms: Option<u64>,
    /// Also plan-derived, and the reason the first two CI runs of this change died: the stop
    /// policy is resolved UNCONDITIONALLY for every worker, so a child with no plan context
    /// refused there after the ordinary walk had already succeeded. It is the parent's decision
    /// in exactly the sense the rest of this carrier is — freeze it, hand it over.
    batch_stop_policy: FloorBatchStopPolicy,
}

fn read_scoped_execution_requests() -> Result<Vec<ScopedExecutionRequest>, String> {
    let path = Path::new(SCOPED_EXECUTION_REQUESTS_PATH);
    let body = fs::read_to_string(path)
        .map_err(|e| format!("read scoped requests {}: {e}", path.display()))?;
    let requests: Vec<ScopedExecutionRequest> =
        serde_json::from_str(&body).map_err(|e| format!("parse scoped requests: {e}"))?;
    refuse_duplicate_scoped_batch_ids(&requests, &path.display().to_string())?;
    Ok(requests)
}

/// A batch id addresses exactly one frozen population, so a repeated id is an ambiguity, not an
/// ordering question. It is refused at the READ rather than at either consumer, because the
/// coordinator spawns one child per row and would otherwise turn one contradiction into N parallel
/// workers racing on one batch's outputs. The manifest this carrier replaces carried the same
/// refusal; dropping it on the way through would have been a silent widen (DESIGN §5).
fn refuse_duplicate_scoped_batch_ids(
    requests: &[ScopedExecutionRequest],
    located: &str,
) -> Result<(), String> {
    for (index, request) in requests.iter().enumerate() {
        if requests[..index]
            .iter()
            .any(|earlier| earlier.batch_id == request.batch_id)
        {
            return Err(format!(
                "scoped requests {located} contain duplicate batch id `{}` — refused",
                request.batch_id
            ));
        }
    }
    Ok(())
}

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut measure_cgroup_peak = false;
    let mut verify_artifacts: Vec<String> = Vec::new();
    let mut verify_artifacts_mode = false;
    let mut required_floor_mode = false;
    let mut required_ci_mode = false;
    let mut required_cited_symbol_mode = false;
    let mut required_v2_emission_mode = false;
    let mut required_v2_emission_selftest_mode = false;
    let mut required_regen_mode = false;
    let mut required_regen_fixed_point_mode = false;
    let mut heads_reading_differential_mode = false;
    let mut behavioral_receipt_plan_mode = false;
    let mut behavioral_receipt_selftest_mode = false;
    let mut behavioral_receipt_census_mode = false;
    let mut regen_candidate_dir = "target/stage0-regen-candidate".to_string();
    let mut regen_receipt_path = "target/stage0-regen-receipt.json".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--verify-build-artifacts" => {
                // All remaining positional args are declared release-artifact paths to verify.
                verify_artifacts_mode = true;
                i += 1;
                while i < args.len() {
                    verify_artifacts.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            "--source-root" => {
                i += 1;
                source_roots.push(require_path_value(&args, i, "--source-root")?);
            }
            "--required-floor" => {
                required_floor_mode = true;
            }
            "--required-ci" => {
                required_ci_mode = true;
            }
            "--required-cited-symbol" => {
                required_cited_symbol_mode = true;
            }
            "--required-v2-emission" => {
                required_v2_emission_mode = true;
            }
            "--required-v2-emission-selftest" => {
                required_v2_emission_selftest_mode = true;
            }
            "--required-regen" => {
                required_regen_mode = true;
            }
            "--heads-reading-differential" => {
                heads_reading_differential_mode = true;
            }
            "--required-regen-fixed-point" => {
                required_regen_fixed_point_mode = true;
            }
            "--behavioral-receipt-plan" => {
                behavioral_receipt_plan_mode = true;
            }
            "--behavioral-receipt-selftest" => {
                behavioral_receipt_selftest_mode = true;
            }
            "--behavioral-receipt-census" => {
                behavioral_receipt_census_mode = true;
            }
            "--regen-candidate-dir" => {
                i += 1;
                regen_candidate_dir = require_value(&args, i, "--regen-candidate-dir")?;
            }
            "--regen-receipt" => {
                i += 1;
                regen_receipt_path = require_value(&args, i, "--regen-receipt")?;
            }
            "--measure-cgroup-peak" => measure_cgroup_peak = true,
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    // Build-artifact verification (no plan run): the floor's bootstrap `cargo build` is followed by
    // this check so a `successful` build that nonetheless produced a missing/zero-byte binary
    // (sccache serving a truncated/empty cached artifact — DESIGN §5 fail-open) fails CLOSED before
    // the floor runs. The LOGIC lives here (the floor binary) instead of inline shell in ci.yml; the
    // declared artifact paths are still authored by the .dag spec and passed as positional args.
    // Short-circuits before the plan-arg requirements so it needs no `--plan-entry`.
    if verify_artifacts_mode {
        return verify_build_artifacts(&verify_artifacts);
    }

    // Standalone whole-tree cgroup measurement (no plan run): the `rust_tests` job invokes this
    // after its gate to emit ITS leaf cgroup peak (a separate ephemeral runner cgroup from the `ci`
    // job's), reusing the same single-authority walk/emit. Short-circuits before the plan-arg
    // requirements so it needs no `--source-root`/`--plan-entry`.
    if measure_cgroup_peak {
        let job = std::env::var("GITHUB_JOB").unwrap_or_else(|_| "standalone".to_string());
        emit_cgroup_measurement(&format!("job={job} (--measure-cgroup-peak)"));
        return Ok(ExitCode::SUCCESS);
    }

    // ── V2 EMISSION: TWO STANDALONE ENTRY POINTS BESIDE THE REQUIRED PHASE ──────
    //
    // The v2-emission transaction IS ENROLLED in `--required-ci` as phase 3 (see that
    // mode below), on an operator ruling relayed through the requesting session
    // 2026-08-23, at a measured +135s against the floor's ~30-40 minutes. These two
    // flags are not an opt-in alternative to that phase and must not be read as one:
    // they exist because running the emission alone, or running only its red/green
    // evidence, are real local actions -- the same reason the `src/v1` .dag parse sweep
    // keeps its own bin beside its required phase.
    //
    // This comment said "DELIBERATELY NOT ENROLLED" in the revision that ADDED the
    // enrolment, which is the premise contamination DESIGN warns about rather than a
    // stale comment: a reader grepping here would conclude the phase is opt-in when it
    // is required. It is rewritten rather than annotated, because two accounts of one
    // fact is what produced the contradiction.
    //
    // Both flags run the SAME producer the required phase runs
    // (`cli_run::compile_entry_emission`), so a green here and a green there cannot be
    // different facts.
    if required_v2_emission_selftest_mode {
        let failures = v1_compiler::cli_run::run_required_v2_emission_selftest();
        for failure in &failures {
            eprintln!("required-v2-emission-selftest: FAIL {failure}");
        }
        return if failures.is_empty() {
            eprintln!(
                "required-v2-emission-selftest: OK red fixture refused on the annotation cause, green fixture emitted"
            );
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    if source_roots.is_empty() {
        eprintln!("claim_executor: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }
    let _phase_profile = PhaseProfile::install_from_env();

    // THE REQUIRED WITNESS FLOOR: one repository preparation, one immutable scope per distinct
    // claim scope, one cheap mutable frame per claim, one linear fold over every witness in the
    // tree. It takes no plan, so it short-circuits before the plan-arg requirement — there is no
    // schedule to resolve, no batch to assign, no worker to spawn and no selection to compute,
    // and the absence of those flags is the point rather than an omission. `run_required_floor`
    // refuses when planned, executed and terminal identity counts disagree, so a silently short
    // roster cannot report as a pass.
    if behavioral_receipt_census_mode {
        return run_behavioral_receipt_census(&source_roots);
    }

    if behavioral_receipt_selftest_mode {
        return run_behavioral_receipt_selftest(&source_roots);
    }

    if behavioral_receipt_plan_mode {
        return run_behavioral_receipt_plan(&source_roots);
    }

    // THE COMPOSED CI RUN — one process, three phases: the src/v1 .dag parse sweep, the regen
    // first-generation comparison, and the witness floor.
    //
    // WHAT IT IS AND IS NOT. Sequencing a program's phases is the program's job (DESIGN §3: the
    // workflow is a realization of the intent, not the place the intent lives), so the order
    // lives here rather than in a YAML step list whose preconditions read each other's
    // `outcome`. What is NOT here is a judgement about which checks CI ought to run: the roster
    // is an operator decision, and the 2026-08-21 ruling set it to exactly these three.
    //
    // WHAT WAS DELETED AND WHY IT IS NOT LEFT AS A SKIPPED PHASE (operator ruling, 2026-08-21).
    // Five phases previously ran inside this fold: merge-admission-capture, regen-determinism
    // (the fixed point), receipt-selftest, receipt-vs-changed-authorities, and
    // merge-admission-stamp. They are GONE from this mode — not disabled behind a flag, not
    // reported as SKIPPED — because a phase that always reports the same non-verdict is the
    // absorbing fallback wearing a phase's clothes (DESIGN §5): its deficit frequency is zero by
    // construction and it reads as coverage on the ledger. The capabilities themselves survive
    // where they had their own entry points and consumers: `--required-regen-fixed-point`,
    // `--behavioral-receipt-selftest`, `--behavioral-receipt-plan` and
    // `--behavioral-receipt-census` are unchanged and still run standalone. What ends is their
    // enrolment in the required run, and that is the whole of the change.
    //
    // WHAT THAT COSTS, named rather than left to be rediscovered. No CI run now measures regen
    // DETERMINISM (that the emitter reproduces its own output), behavioural equivalence of a
    // changed authority against its committed mirror, or the receipt machinery's own
    // discriminating arms; and no run mints a merge-admission receipt. The merge-admission
    // consumers refuse on a missing receipt rather than admitting on one (DESIGN, the CI
    // paragraph's re-add queue), so nothing is admitted by the absence — but the three
    // measurements above are simply not taken, which is a declared rung drop, not a silent one.
    //
    // WHAT THE ORDER IS, AND WHY EACH PHASE RUNS ANYWAY. The four phases are independent —
    // the one real data dependency, the fixed point's need for regen's pass-1 digest, went with
    // the phase that consumed it — so every phase RUNS EVEN AFTER AN EARLIER FAILURE and the run
    // reports the complete ledger instead of letting the first defect hide the rest. The line
    // still stops (a nonzero exit on any failed phase); it stops with every deficit named. This
    // is the stopped-line AUDIT DESIGN §5 sanctions: it reports, it never greens.
    if required_ci_mode {
        let mut phase_failures: Vec<String> = Vec::new();
        let mut ran: Vec<&'static str> = Vec::new();

        // PHASE 1 — the .dag parse sweep, over every authored root (src/v1, dag, src/v2).
        // Independent of everything below it. The roster is
        // `cli_run::DAG_PARSE_SWEEP_ROOTS`, shared with the standalone bin so the cheapest
        // local check and this phase cover the same files.
        eprintln!(
            "required-ci: phase parse (.dag: {})",
            v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS.join(", ")
        );
        match v1_compiler::cli_run::run_dag_parse_sweep(
            &v1_compiler::cli_run::workspace_root(),
            &v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS,
        ) {
            Ok(count) => eprintln!("required-ci: parse OK {count} file(s) parse-clean"),
            Err(errors) => {
                for e in &errors {
                    eprintln!("required-ci: parse FAIL {e}");
                }
                phase_failures.push(format!("parse ({} error(s))", errors.len()));
            }
        }
        ran.push("parse");

        // PHASE 2 — regen first generation: the emitted mirrors against what is committed.
        eprintln!("required-ci: phase regen (first generation vs committed)");
        match v1_compiler::cli_run::run_required_regen(&regen_candidate_dir, &regen_receipt_path) {
            Ok(outcome) => {
                // Read through accessors, and print `unmeasured` rather than a plausible
                // default when the pass built the wrong variant (#8650's shape).
                let fge = outcome
                    .receipt
                    .first_generation_equal()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unmeasured".to_string());
                let candidate = outcome.receipt.candidate_artifact().unwrap_or("unmeasured");
                eprintln!("required-ci: regen first_generation_equal={fge} candidate={candidate}");
                for failure in &outcome.failures {
                    eprintln!("required-ci: regen FAIL {failure}");
                }
                if !outcome.failures.is_empty() {
                    phase_failures.push(format!("regen ({} failure(s))", outcome.failures.len()));
                }
            }
            Err(e) => {
                // A REFUSAL IS NOT A MISMATCH: nothing was emitted, so there is no comparison
                // to report, and the refusal is named rather than folded into a drift verdict.
                eprintln!("required-ci: regen refused: {e}");
                phase_failures.push(format!("regen refused: {e}"));
            }
        }
        ran.push("regen");

        // PHASE 3 — v2 emission. ENROLLED 2026-08-23 on an operator ruling relayed through
        // the requesting session, after the 2026-08-23 break reached main and stayed for
        // hours with every required phase green: the required run parses src/v1 .dag,
        // compares the regen mirrors and folds the floor, and NONE OF THE THREE COMPILES A
        // v2 ENTRY. Measured cost +135s against the floor's ~30-40 minutes.
        //
        // ORDERED AHEAD OF THE FLOOR, and the ordering is a report-order fact only: the
        // phases stay independent and every one runs even after an earlier failure, so
        // this does not stop the floor starting on an unemittable tree. Making it an early
        // PREREQUISITE (an exit before the floor) is a scheduling decision that belongs to
        // the operator with the number attached, and it would change this mode's
        // stopped-line audit design, so it is not taken here.
        //
        // The subject is the SAME PRODUCER the cargo board runs
        // (cli_run::compile_entry_emission, which `gunbc compile --entry` also calls), so
        // a green here and an emitting board are one fact rather than two.
        eprintln!("required-ci: phase v2-emission (one entry, the board's producer)");
        match v1_compiler::cli_run::run_required_v2_emission(&source_roots) {
            Ok(runs) => {
                let mut not_completed = 0usize;
                for run in &runs {
                    eprintln!("{}", run.measurement_line("required-ci: v2-emission"));
                    match &run.disposition {
                        v1_compiler::cli_run::EntryEmissionDisposition::Completed { .. } => {}
                        v1_compiler::cli_run::EntryEmissionDisposition::Refused {
                            phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-ci: v2-emission EmissionRefused entry={} phase={phase} cause={cause}",
                                run.entry
                            );
                        }
                        v1_compiler::cli_run::EntryEmissionDisposition::NotExecuted {
                            earlier_phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-ci: v2-emission EmissionNotExecuted entry={} earlier_phase={earlier_phase} cause={cause}",
                                run.entry
                            );
                        }
                    }
                }
                if not_completed > 0 {
                    phase_failures.push(format!("v2-emission ({not_completed} not completed)"));
                }
            }
            Err(e) => {
                eprintln!(
                    "required-ci: v2-emission EmissionNotExecuted earlier_phase=roster cause={e}"
                );
                phase_failures.push(format!("v2-emission roster: {e}"));
            }
        }
        ran.push("v2-emission");

        // PHASE 4 — the witness floor. Independent; runs whatever happened above.
        eprintln!("required-ci: phase floor (one prepared subject, one fold)");
        let commit = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());
        match v1_compiler::cli_run::run_required_floor(
            &source_roots,
            &commit,
            v1_compiler::cli_run::ShardStyle::single_shard(),
        ) {
            Ok(outcome) => {
                report_required_floor_outcome(&outcome);
                if !required_floor_outcome_is_clean(&outcome) {
                    phase_failures.push("floor".to_string());
                }
            }
            Err(e) => {
                eprintln!("required-ci: floor refused: {e}");
                phase_failures.push(format!("floor refused: {e}"));
            }
        }
        ran.push("floor");

        eprintln!(
            "required-ci: phases_run={} failed={}",
            ran.len(),
            phase_failures.len()
        );
        for failure in &phase_failures {
            eprintln!("required-ci: FAILED PHASE {failure}");
        }
        return if phase_failures.is_empty() {
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    if required_v2_emission_mode {
        let roots = if source_roots.is_empty() {
            v1_compiler::cli_run::witness_layer_roots()
        } else {
            source_roots.clone()
        };
        return match v1_compiler::cli_run::run_required_v2_emission(&roots) {
            Ok(runs) => {
                let mut not_completed = 0usize;
                for run in &runs {
                    // ONE SELF-DESCRIBING LINE. The disposition is ON the line, so a run
                    // that never reached the compiler cannot be read as a clean emission
                    // of nothing by anything that reads one line at a time.
                    eprintln!("{}", run.measurement_line("required-v2-emission"));
                    match &run.disposition {
                        v1_compiler::cli_run::EntryEmissionDisposition::Completed { .. } => {}
                        v1_compiler::cli_run::EntryEmissionDisposition::Refused {
                            phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionRefused entry={} phase={phase} cause={cause}",
                                run.entry
                            );
                        }
                        v1_compiler::cli_run::EntryEmissionDisposition::NotExecuted {
                            earlier_phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionNotExecuted entry={} earlier_phase={earlier_phase} cause={cause}",
                                run.entry
                            );
                        }
                    }
                }
                eprintln!(
                    "required-v2-emission: entries={} not_completed={}",
                    runs.len(),
                    not_completed
                );
                if not_completed == 0 {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            // The roster itself being unreadable is reported as itself: no entry ran, so
            // there is no per-entry disposition to render.
            Err(e) => {
                eprintln!(
                    "required-v2-emission: EmissionNotExecuted earlier_phase=roster cause={e}"
                );
                Err(ExitCode::from(1))
            }
        };
    }

    if required_regen_fixed_point_mode {
        return match v1_compiler::cli_run::run_required_regen_fixed_point(&regen_receipt_path, None)
        {
            Ok(outcome) => {
                // The provenance is printed, not just carried. This line previously read
                // `first_generation_equal={}` off the receipt as though the fixed-point pass had
                // measured it; it never does. Labelling it `referenced_` and naming the commit it
                // came from means the log itself distinguishes measured from quoted -- and since
                // the host refuses a cross-tree reference, `referenced_at` equals HEAD on every
                // line that is allowed to print.
                let (referenced_fge, referenced_at) = match outcome.receipt.prior() {
                    Some(prior) => (
                        prior.first_generation_equal.to_string(),
                        prior.commit_sha.clone(),
                    ),
                    None => ("unavailable".to_string(), "unavailable".to_string()),
                };
                eprintln!(
                    "required-regen-fixed-point: fixed_point_equal={} referenced_first_generation_equal={} referenced_at={}",
                    outcome
                        .receipt
                        .fixed_point_equal()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unmeasured".to_string()),
                    referenced_fge,
                    referenced_at
                );
                for failure in &outcome.failures {
                    eprintln!("required-regen-fixed-point: FAIL {failure}");
                }
                if outcome.failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-regen-fixed-point: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    // THE CITED-SYMBOL CENSUS IS ITS OWN REQUIRED CHECK, NOT A PHASE OF `--required-ci`.
    //
    // WHY IT IS NOT A PHASE. The operator narrowed `--required-ci` from eight phases to three on
    // 2026-08-21 (#8791), and that ruling is about what one composed entry point is responsible
    // for. A census with a different subject gets its own named check instead: `--required-ci`
    // stays at exactly three phases, so nothing about the narrowing is weakened, contradicted or
    // quietly reinterpreted. Routing the same phase in under a different name would be the
    // workaround this repository refuses; a distinct concern with its own check is the shape the
    // ruling points at.
    //
    // WHY IT IS NOT A FLOOR CLAIM EITHER, and this one is structural rather than a preference.
    // `run_required_floor` declines any entry that reads the live tree (`DeclinedLiveTree`), and
    // reading the live corpus IS this census's subject -- relocating the witness moves it from
    // `DeclinedLongModule` to `DeclinedLiveTree` and never to `Planned`. No amount of making it
    // cheaper opens that door.
    //
    // WHAT IT REPORTS. Every unresolved reference with the typed arm that refused it, and -- on a
    // green -- the population it checked. An empty refusal list means both "every authored
    // reference resolved" and "there were no references to check"; those are different states and
    // only the first is coverage, so a population it cannot read FAILS rather than greening over
    // an unknown denominator.
    if required_cited_symbol_mode {
        let (ctx, _entry) = match cited_symbol_lens_context(&source_roots) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("cited-symbol: refused: {e}");
                return Err(ExitCode::from(1));
            }
        };
        let rows = match cited_symbol_census(&ctx) {
            Ok(rows) => rows,
            Err(e) => {
                eprintln!("cited-symbol: refused: {e}");
                return Err(ExitCode::from(1));
            }
        };
        if !rows.is_empty() {
            for row in &rows {
                eprintln!("cited-symbol: REFUSED {row}");
            }
            eprintln!(
                "cited-symbol: FAIL {} authored reference(s) do not resolve — a citation outlived \
                 what it names (DESIGN §3)",
                rows.len()
            );
            return Err(ExitCode::from(1));
        }
        return match cited_symbol_population(&ctx) {
            Ok(checked) => {
                eprintln!("cited-symbol: OK every authored reference resolves checked={checked}");
                Ok(ExitCode::SUCCESS)
            }
            Err(e) => {
                eprintln!("cited-symbol: refused: population unreadable: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    // The heads reading's own instrument. It is not enrolled in the required run and this
    // clause does not pretend otherwise: reading 3875 modules TWICE is precisely the cost
    // the heads reading exists to remove, so paying it on every push would spend more than
    // the repair saves. It is a re-runnable receipt — the differential this change was
    // landed on can be re-taken by anyone, on any tree, rather than being a number quoted
    // from a run nobody can repeat.
    if heads_reading_differential_mode {
        let roots = if source_roots.is_empty() {
            vec!["dag".to_string(), "src/v2".to_string()]
        } else {
            source_roots.clone()
        };
        let d = v1_compiler::cli_run::heads_reading_differential(&roots);
        eprintln!(
            "heads-reading-differential: compared={} divergent={} narrowed={} regressed={} both_refused={}",
            d.modules_compared,
            d.divergent.len(),
            d.narrowed.len(),
            d.regressed.len(),
            d.both_refused.len(),
        );
        // The two readings' summed parse wall, from the same process over the same
        // modules. This is the PARSE term only — not the whole `pool_parse` row, which
        // also pays tokenize, newline indexing and per-file setup that neither reading
        // changes.
        eprintln!(
            "heads-reading-differential: full_reading_parse_ms={} heads_reading_parse_ms={}",
            d.full_reading_nanos / 1_000_000,
            d.heads_reading_nanos / 1_000_000,
        );
        for path in d.divergent.iter().take(20) {
            eprintln!("heads-reading-differential: DIVERGENT {path}");
        }
        for path in d.regressed.iter().take(20) {
            eprintln!("heads-reading-differential: REGRESSED {path}");
        }
        for path in d.narrowed.iter().take(20) {
            eprintln!("heads-reading-differential: narrowed (declared scope) {path}");
        }
        return if d.holds() {
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    if required_regen_mode {
        return match v1_compiler::cli_run::run_required_regen(
            &regen_candidate_dir,
            &regen_receipt_path,
        ) {
            Ok(outcome) => {
                // Both values here ARE measured by this pass, so they print unqualified. Read
                // through accessors rather than by matching the variant: the
                // `required_regen_host` module is private to `cli_run`, so the type is usable here
                // but not nameable. The accessors return Option because the sibling variant does
                // not measure these fields; a `None` on this path would mean the first pass built
                // the wrong variant, so it prints `unmeasured` rather than defaulting to a
                // plausible-looking value.
                let fge = outcome
                    .receipt
                    .first_generation_equal()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unmeasured".to_string());
                let candidate = outcome
                    .receipt
                    .candidate_artifact()
                    .unwrap_or("unmeasured")
                    .to_string();
                eprintln!("required-regen: first_generation_equal={fge} candidate={candidate}");
                for failure in &outcome.failures {
                    eprintln!("required-regen: FAIL {failure}");
                }
                if outcome.failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-regen: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    if required_floor_mode {
        let commit = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());
        return match v1_compiler::cli_run::run_required_floor(
            &source_roots,
            &commit,
            v1_compiler::cli_run::ShardStyle::single_shard(),
        ) {
            Ok(outcome) => {
                report_required_floor_outcome(&outcome);
                if required_floor_outcome_is_clean(&outcome) {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-floor: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }
    // THE PLAN/WALK SURFACE IS DELETED (2026-08-25). Every mode above returns before this
    // point, and what stood here -- the --plan-entry walk, the batch executor, the
    // coordinator/worker protocol, the perturb re-walk and their terminal reporting -- was
    // reachable ONLY through --plan-entry. Nothing has supplied it since the 2026-08-15 floor
    // cut deleted the composed floor it drove, and `src/v2/workflow/ci_floor_plan.dag`, which
    // --plan-function named, does not exist.
    //
    // This is a REACHABILITY deletion, not an occupancy one, and the distinction is the one
    // DESIGN names: the arms are not quiet guards that happen to be empty today, their
    // governing MECHANISM was removed, so no input any caller can author reaches them.
    //
    // The refusal is typed and names the live modes rather than exiting silently: an argv this
    // binary no longer understands must stop the line, never fall through to a default.
    eprintln!(
        "claim_executor: CLAIM-EXECUTOR REFUSAL cause=NoRunnableMode — live modes are \
         --required-ci and --verify-build-artifacts. The plan/walk surface (--plan-entry, \
         --plan-function, --floor-worker-role, --scoped-batch-id) was deleted 2026-08-25 with \
         the machinery it drove."
    );
    Err(ExitCode::from(2))
}

fn emit_floor_terminal_outcome(outcome: &str, detail: &str) {
    append_floor_phase_journal("walk-terminal", outcome, detail);
    if outcome == "completed" {
        eprintln!("[floor-terminal] outcome=completed detail={detail}");
    } else {
        eprintln!("[floor-terminal-refusal] outcome=failed detail={detail}");
    }
}

/// Pre-walk worker failures return through `main()` instead of `floor_terminal_fast_exit`.
/// Mirror the walk-terminal journal/stderr emission so those refusals survive Actions log
/// truncation the same way post-walk failures do.
fn emit_worker_terminal_before_return(code: ExitCode) -> ExitCode {
    let Some(path) = std::env::var_os(FLOOR_WORKER_TERMINAL_ENV) else {
        return code;
    };
    if code == ExitCode::SUCCESS {
        return code;
    }
    let path = PathBuf::from(path);
    let (outcome, detail) = if path.exists() {
        match fs::read_to_string(&path) {
            Ok(body) => {
                let line = body.trim_end_matches(['\r', '\n']);
                let (label, detail) = line.split_once('\t').unwrap_or((line, ""));
                let outcome = if label == "completed" {
                    "completed"
                } else {
                    "failed"
                };
                (outcome, detail.to_string())
            }
            Err(e) => ("failed", format!("worker terminal receipt unreadable: {e}")),
        }
    } else {
        let detail = "worker returned before producing a walk terminal receipt".to_string();
        let _ = write_floor_worker_terminal("failed", &detail);
        ("failed", detail)
    };
    emit_floor_terminal_outcome(outcome, &detail);
    code
}

/// Print the floor's complete ledger. ONE implementation, called by `--required-floor` and by
/// the composed `--required-ci` run, so the two modes cannot drift into reporting the same
/// outcome differently (DESIGN §3).
fn report_required_floor_outcome(outcome: &v1_compiler::cli_run::RequiredFloorOutcome) {
    eprintln!(
        "required-floor: subject={} modules_resolved={} modules_excluded={}",
        outcome.subject_digest, outcome.modules_resolved, outcome.modules_excluded
    );
    // THE SUBJECT THE ROSTER WAS PROJECTED FROM, STATED BEFORE THE ROSTER.
    // `planned` is the population that SURVIVED site projection; printing it
    // without `offered` and `declined_long` made the receipt unable to say what it
    // dropped, which is how a roster that narrowed read exactly like one that did
    // not. The three are printed together so the subtraction is visible rather
    // than inferable.
    eprintln!(
        "required-floor: offered={} routed={} declined_long={} declined_fixture={} \
         declined_live={} — every discovered site is exactly one of these",
        outcome.sites_offered,
        outcome.claims_planned,
        outcome.declined_long_module,
        outcome.declined_fixture_member,
        outcome.declined_live_tree
    );
    // WHY route_gap IS NOW SPELLED route_gap_unenrolled, AND WHY route_gap_held JOINS IT HERE.
    // The old field printed `outcome.route_gap.len()` under the bare name `route_gap` — the
    // UNENROLLED gaps alone. Measured on main run 32407436149 that printed `route_gap=0` while
    // 101 enrolled identities were held as route-gapped on a `[floor-route-gap]` line ~300 lines
    // away, and the headline's own categories did not close: executed 9810 − passed 9502 −
    // known_red_held 207 − failed 0 left exactly those 101 unaccounted. A reader did not see an
    // understated count they might interrogate; they saw a ZERO, which closes the question rather
    // than opening it. The correcting evidence existed and was disjoint from the surface anyone
    // reads, so it lands on THIS line — shipping it as yet another separate line would reproduce
    // the defect it repairs.
    //
    // THESE COUNTERS ARE OVERLAPPING AXES, NOT A PARTITION, and the receipt must not be read as
    // one. A single claim that is enrolled expected-red AND route-gapped increments
    // `known_red_host_effect_refused` and `route_gap_held` both (see the ExpectedRedArm::
    // HostEffectRefused arm in cli_run: the two rosters are different axes and the row sits on
    // both). So `passed + known_red_held + … == executed` is NOT an invariant and asserting it
    // would red on a legitimate state. The tree already has the vocabulary for this distinction —
    // `InclusiveCostRow` carries "a counter deliberately INCLUSIVE of another … so the receipt can
    // print it without it ever entering the exclusive sum" — but the floor's ledger has no such
    // typed exclusive/inclusive split, which is why the sum silently failed to close instead of
    // refusing. Modelling that split is a RequiredFloorDisposition question in
    // src/v2/workflow/required_floor.dag, not something to decide inside an eprintln!.
    eprintln!(
        "required-floor: planned={} executed={} terminal={} passed={} \
         known_red_held={} failed={} stale_quarantine={} \
         interrupted_before_verdict={} completed_over_cost_requirement={} \
         host_tool_unresolved={} route_gap_unenrolled={} route_gap_held={} \
         stale_route_gap={} known_red_now_passing={} known_red_budget_refused={} \
         known_red_passed_over_budget={} known_red_host_tool_unresolved={} \
         known_red_host_effect_refused={} known_red_runtime_errored={} \
         known_red_observation_unreadable={} over_cost_line_diagnostic={}",
        outcome.claims_planned,
        outcome.claims_executed,
        outcome.receipt_identities,
        outcome.passed,
        outcome.known_red_held,
        outcome.failures.len(),
        outcome.stale_quarantine.len(),
        outcome.interrupted_before_verdict.len(),
        outcome.completed_over_cost_requirement.len(),
        outcome.host_tool_unresolved.len(),
        outcome.route_gap.len(),
        outcome.route_gap_held,
        outcome.stale_route_gap.len(),
        outcome.known_red_now_passing,
        outcome.known_red_budget_refused,
        outcome.known_red_passed_over_budget,
        outcome.known_red_host_tool_unresolved_held,
        outcome.known_red_host_effect_refused,
        outcome.known_red_runtime_errored.len(),
        outcome.known_red_observation_unreadable.len(),
        outcome.over_cost_line_diagnostic
    );
    // ONE receipt, both numbers (#8642). This replaced a per-miss trace line that had no hit
    // counterpart, so the ratio it is really about was never readable.
    //
    // EXACTLY ONE OF THESE MAY EXIST, and a duplicate is not cosmetic: two lines reporting one
    // pair is the second-representation shape the receipt was introduced to remove, so
    // duplicating it degrades the property it asserts. There WAS a second copy here briefly —
    // this function is re-derived from main's inline block on every merge that touches it, and
    // a note reading "each merge has to graft it back deliberately" instructed the re-add
    // without saying to check whether main's block already carried it. It did. Caught in
    // review 54101. The instruction is deleted with the duplicate: re-derivation copies main's
    // block wholesale, so this line arrives WITH it and needs no grafting.
    let (memo_hits, memo_misses) = v1_compiler::cli_run::compile_dag_rust_emit_check_memo_counts();
    eprintln!(
        "required-floor: compile_dag_rust_emit_check_memo hits={memo_hits} \
         misses={memo_misses}"
    );
    for failure in &outcome.failures {
        eprintln!("required-floor: FAIL {failure}");
    }
    // SEVEN CAUSES, SEVEN COUNTS, ONE STOPPED LINE. All seven refuse the run, and
    // they are reported apart because their remedies differ: a FAIL is a defect to
    // fix, a STALE-QUARANTINE is a fix that already landed and a roster row to
    // delete, an INTERRUPTED-BEFORE-VERDICT is an undecided claim whose real cost
    // is unmeasured (operator ruling 2026-08-19, BUDGET POLICY CUT), a
    // COMPLETED-OVER-COST-REQUIREMENT is a claim that reached a verdict and then
    // was found to cost too much (an exact measurement, not a bound), and a
    // HOST-TOOL-UNRESOLVED is an infra gap to provision (never a witness-cost
    // chase), and a ROUTE-GAP is a claim that never reached its subject because
    // its execution route has no arm for a host effect it reached for — remedied by
    // supplying a route, never by editing the witness, and a STALE-ROUTE-GAP is
    // that same roster's other direction — a route that WAS supplied, whose
    // enrollment must now be deleted. Summing them into `failed` would make an un-quarantine
    // indistinguishable from a regression in the alert signature, which is the
    // conflation `std.witness_admission` rules out. Splitting the former
    // `budget_refused` collection in two makes it visible whether a stopped run
    // is a cost debt on a claim that actually finished, or a claim the safety
    // deadline preempted before it could answer at all — but both still stop the
    // line: an interruption is NotEvaluated, and NotEvaluated is never green.
    for stale in &outcome.stale_quarantine {
        eprintln!("required-floor: STALE-QUARANTINE {stale}");
    }
    for refused in &outcome.interrupted_before_verdict {
        eprintln!("required-floor: INTERRUPTED-BEFORE-VERDICT {refused}");
    }
    for over_cost in &outcome.completed_over_cost_requirement {
        eprintln!("required-floor: COMPLETED-OVER-COST-REQUIREMENT {over_cost}");
    }
    for unresolved in &outcome.host_tool_unresolved {
        eprintln!("required-floor: HOST-TOOL-UNRESOLVED {unresolved}");
    }
    for errored in &outcome.known_red_runtime_errored {
        eprintln!("required-floor: KNOWN-RED-RUNTIME-ERRORED {errored}");
    }
    for unreadable in &outcome.known_red_observation_unreadable {
        eprintln!("required-floor: KNOWN-RED-OBSERVATION-UNREADABLE {unreadable}");
    }
    for unenrolled in &outcome.non_verdict_unenrolled {
        eprintln!("required-floor: NON-VERDICT-UNENROLLED {unenrolled}");
    }
    for stale in &outcome.stale_non_verdict {
        eprintln!("required-floor: STALE-NON-VERDICT {stale}");
    }
    for gap in &outcome.route_gap {
        eprintln!("required-floor: ROUTE-GAP {gap}");
    }
    for stale in &outcome.stale_route_gap {
        eprintln!("required-floor: STALE-ROUTE-GAP {stale}");
    }
    // THE VERDICT LINE, AND WHY A ZERO NEEDED A SENTENCE BESIDE IT.
    //
    // `failed=0` is a sentence a reader can finish alone, and finishes wrongly: it says nothing
    // about whether the population that was supposed to answer actually answered. This exact
    // misreading dispatched a session against a regression that did not exist — a run was
    // compared to a baseline carrying a fix, `planned` matched on both sides, and `failed=0`
    // supplied the confidence that the rest was equivalent. Separating the two questions removes
    // the ability to finish that sentence: `unexpected_failures` is how many claims answered
    // WRONG, `verdict_incomplete` is how many never answered AT ALL, and a run can be admitted
    // while the second is large.
    //
    // ADMITTED IS NOT CLEAN, and the word carries that. `FloorAdmittedWithNonVerdictDebt` gates
    // nothing by itself — the conjunct that gates is growth in the non-verdict population — but
    // it refuses to let a run with 142 unanswered assertions render identically to one with
    // none.
    let verdict_incomplete =
        outcome.known_red_runtime_errored.len() + outcome.known_red_observation_unreadable.len();
    let verdict = if !required_floor_outcome_is_clean(outcome) {
        "FloorRefused"
    } else if verdict_incomplete > 0 {
        "FloorAdmittedWithNonVerdictDebt"
    } else {
        "FloorClean"
    };
    eprintln!(
        "required-floor: verdict={verdict} unexpected_failures={} verdict_incomplete={} \
         non_verdict_unenrolled={} stale_non_verdict={}",
        outcome.failures.len(),
        verdict_incomplete,
        outcome.non_verdict_unenrolled.len(),
        outcome.stale_non_verdict.len()
    );
}

/// Whether the floor outcome permits a green run.
///
/// NINE CAUSES, ONE STOPPED LINE — and the conjunction is written once here rather than at each
/// caller, because a mode that forgot one of them would green a run the other refused. (The
/// count is stated because a reader checks it; it was five before main added `route_gap` and
/// `stale_route_gap`, and the sentence went on saying five through the merge that added them.
/// It briefly said nine while `known_red_runtime_errored` and `known_red_observation_unreadable`
/// were wired in here directly; that was reverted and the count returned to seven.)
///
/// THE EIGHTH IS `non_verdict_unenrolled`, AND IT IS NOT THOSE TWO ARMS MADE GATING. The
/// distinction is the whole design. Those arms are HONEST OBSERVATIONS — they say correctly that
/// an enrolled claim produced no verdict — and gating on them directly would red every lane
/// holding a row of a population nobody has repaired. What was below floor is the COMPOSITION:
/// this function returned CLEAN while an enrolled expected-red assertion had ceased to assert
/// anything, so a true diagnostic sat beside a false conclusion drawn from it. The conjunct
/// therefore gates on GROWTH at identity grain — an identity producing no verdict that
/// `v2.workflow.floor_non_verdict` does not carry — which admits 142 → 0 in any order and
/// refuses 142 → 143, and refuses a swap that leaves the count untouched.
///
/// THE NINTH IS `stale_non_verdict`, AND IT GATES FOR THE REASON THE EIGHTH DOES. A row whose
/// identity has been repaired is a LIVE EXEMPTION until it is deleted: the witness is fixed
/// today and, should it stop producing a verdict again, it is already rostered and the eighth
/// conjunct admits it. Repayment and deletion are therefore one act, which is what
/// `stale_route_gap` and the expected-red staleness join already require. This shipped as
/// report-only for one commit under the argument that refusing "punishes the fix"; it does not
/// — it requires the fix to be complete, and the diagnostic names every row to delete.
fn required_floor_outcome_is_clean(outcome: &v1_compiler::cli_run::RequiredFloorOutcome) -> bool {
    outcome.failures.is_empty()
        && outcome.non_verdict_unenrolled.is_empty()
        && outcome.stale_non_verdict.is_empty()
        && outcome.stale_quarantine.is_empty()
        && outcome.interrupted_before_verdict.is_empty()
        && outcome.completed_over_cost_requirement.is_empty()
        && outcome.host_tool_unresolved.is_empty()
        && outcome.route_gap.is_empty()
        && outcome.stale_route_gap.is_empty()
}

fn main() -> ExitCode {
    let coordinator_args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = maybe_run_floor_coordinator(&coordinator_args) {
        return code;
    }
    // The materialization demand receipt is mandatory on the floor: enable the
    // interpreter's recompute-trace ledger unless the environment already set
    // it. An explicit =0 zeroes the receipt, and the derived ci.yml gate fails
    // closed on keyed_calls=0 — disabling is loud, never silent.
    if std::env::var_os("GUNBC_RECOMPUTE_TRACE").is_none() {
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
    }
    let code = match run() {
        Ok(code) => code,
        Err(code) => code,
    };
    emit_worker_terminal_before_return(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_budget_watchdog_refuses_and_writes_located_receipt() {
        const CHILD_MARKER: &str = "GUNBC_POPULATION_BUDGET_WATCHDOG_CHILD";
        if std::env::var(CHILD_MARKER).as_deref() == Ok("1") {
            let progress = PopulationBudgetProgress::before_first_unit();
            progress.enter(3, "dag/tools/fixture.dag::bounded_stage_claim".to_string());
            let terminal = PathBuf::from("target/population-budget-watchdog-terminal.tsv");
            let _ = fs::remove_file(&terminal);
            std::env::set_var(FLOOR_WORKER_TERMINAL_ENV, &terminal);
            let _armed = arm_population_budget_watchdog(
                "ordinary_floor",
                "fixture::bounded_plan",
                Some(50),
                progress,
            );
            std::thread::sleep(std::time::Duration::from_secs(2));
            panic!("watchdog did not terminate the child");
        }

        let dir = std::env::temp_dir().join(format!(
            "gunbc-population-budget-watchdog-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create watchdog fixture directory");
        let terminal = dir.join("target/population-budget-watchdog-terminal.tsv");
        let output = std::process::Command::new(std::env::current_exe().expect("current test exe"))
            .arg("--exact")
            .arg("tests::population_budget_watchdog_refuses_and_writes_located_receipt")
            .arg("--nocapture")
            .env(CHILD_MARKER, "1")
            .current_dir(&dir)
            .output()
            .expect("run watchdog child");
        assert!(!output.status.success(), "budget child must refuse");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("ORDINARY-FLOOR-OVER-BUDGET")
                && stderr.contains("fixture::bounded_plan")
                && stderr.contains("population_index=3")
                && stderr.contains("dag/tools/fixture.dag::bounded_stage_claim")
                && stderr.contains("elapsed_ms=")
                && stderr.contains("budget_ms=50"),
            "refusal must be typed and located, got: {stderr}"
        );
        let receipt = fs::read_to_string(dir.join("target/floor-population-budget-refusal.txt"))
            .expect("watchdog refusal receipt");
        assert!(
            receipt.contains("population=ordinary_floor")
                && receipt.contains("plan_site=fixture::bounded_plan")
                && receipt.contains("population_index=3")
                && receipt.contains("active_unit=dag/tools/fixture.dag::bounded_stage_claim")
                && receipt.contains("elapsed_ms=")
                && receipt.contains("budget_ms=50")
                && receipt.contains("outcome=refused"),
            "receipt must carry the refused population and its subject: {receipt}"
        );
        let worker_terminal = fs::read_to_string(&terminal).expect("worker terminal receipt");
        assert!(
            worker_terminal.starts_with("refused\t")
                && worker_terminal.contains("ORDINARY-FLOOR-OVER-BUDGET")
                && worker_terminal.contains("fixture::bounded_plan")
                && worker_terminal.contains("population_index=3")
                && worker_terminal.contains("dag/tools/fixture.dag::bounded_stage_claim")
                && worker_terminal.contains("budget_ms=50"),
            "progress announcement and worker terminal receipt must carry the same located refusal: {worker_terminal:?}"
        );
        fs::remove_dir_all(&dir).expect("remove watchdog fixture directory");
    }

    // Process-wide eval-recompute totals are shared across every test in this
    // binary. Tests that drain or assert on the accumulator must serialize so
    // one cannot steal another's totals (DESIGN §5 hermetic discriminating tests).
    static PROCESS_EVAL_RECOMPUTE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_process_eval_recompute_test_lock<F: FnOnce()>(f: F) {
        let _guard = PROCESS_EVAL_RECOMPUTE_TEST_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let prior_trace = std::env::var_os("GUNBC_RECOMPUTE_TRACE");
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
        v1_compiler::v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        match prior_trace {
            Some(value) => std::env::set_var("GUNBC_RECOMPUTE_TRACE", value),
            None => std::env::remove_var("GUNBC_RECOMPUTE_TRACE"),
        }
        v1_compiler::v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
        match run {
            Ok(()) => {}
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    fn with_workspace_root_current_dir<F: FnOnce()>(root: &std::path::Path, f: F) {
        let prior_cwd = std::env::current_dir().ok();
        std::env::set_current_dir(root).expect("chdir to workspace root for receipt paths");
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        if let Some(cwd) = prior_cwd {
            let _ = std::env::set_current_dir(cwd);
        }
        match run {
            Ok(()) => {}
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    fn finalization_record(resolve_counts: &[u64]) -> BatchRecord {
        BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "finalization-fixture".to_string(),
            is_wet: false,
            results: resolve_counts
                .iter()
                .map(|n| ClaimResult {
                    function: "fixture".to_string(),
                    entry: "fixture.dag".to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: *n as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: None,
                })
                .collect(),
        }
    }

    fn test_floor_finalization() -> FloorFinalization {
        FloorFinalization {
            expected_obligations: vec![
                TransportedObligation {
                    identity: "CompileAnchorWholeTree".to_string(),
                    entry: TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY.to_string(),
                    function: TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION.to_string(),
                },
                TransportedObligation {
                    identity: "NativeBundleEscapingEntry".to_string(),
                    entry: TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY.to_string(),
                    function: TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION.to_string(),
                },
            ],
        }
    }

    fn repo_root_from_manifest() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root from CARGO_MANIFEST_DIR")
    }

    fn dag_source_from_repo(rel: &str) -> String {
        let path = repo_root_from_manifest().join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn dag_string_data_literal(source: &str, data_name: &str) -> String {
        let marker = format!("data {data_name}: String = \"");
        let start = source
            .find(&marker)
            .unwrap_or_else(|| panic!("string data row {data_name} not found in authority source"))
            + marker.len();
        let rest = &source[start..];
        let end = rest
            .find('"')
            .expect("unterminated string literal in authority source");
        rest[..end].to_string()
    }

    fn dag_record_string_field(source: &str, data_name: &str, field: &str) -> String {
        let marker = format!("data {data_name}:");
        let start = source
            .find(&marker)
            .unwrap_or_else(|| panic!("data row {data_name} not found in authority source"));
        let slice = &source[start..];
        let field_marker = format!("{field}: \"");
        let fstart = slice
            .find(&field_marker)
            .unwrap_or_else(|| panic!("field {field} not found on data row {data_name}"))
            + field_marker.len();
        let rest = &slice[fstart..];
        let end = rest
            .find('"')
            .expect("unterminated string field in authority source");
        rest[..end].to_string()
    }

    // Test-only mirror of gunbc.ci_materialization obligation subject rows. Production
    // parses transported obligations from WalkPlan.finalization; this module is the
    // checkable drift receipt (review 48261 / 48570).
    const TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY: &str = "dag/tools/floor_effect_gate_witness.dag";
    const TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION: &str = "dag_compile_clean_gate_passes";
    const TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY: &str =
        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag";
    const TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION: &str = "native_selected_logic_production_spec";

    /// Seed-retained walk-memo provider id must track the `.dag` authority row.
    #[test]
    fn floor_entry_walk_memo_provider_id_matches_dag_authority() {
        let floor_materialization = dag_source_from_repo("dag/gunbc/floor_materialization.dag");
        assert_eq!(
            dag_string_data_literal(&floor_materialization, "floor_entry_walk_memo_provider_id"),
            FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID,
        );
    }

    /// Test-only obligation-subject literals must track gunbc.ci_materialization authority rows.
    #[test]
    fn floor_resolve_obligation_seed_constants_match_dag_authority() {
        let ci_materialization = dag_source_from_repo("dag/gunbc/ci_materialization.dag");
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "compile_anchor_obligation_subject",
                "entry"
            ),
            TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "compile_anchor_obligation_subject",
                "function"
            ),
            TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "native_bundle_obligation_subject",
                "entry"
            ),
            TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "native_bundle_obligation_subject",
                "function"
            ),
            TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION,
        );
    }

    /// Resolve realization is acquisition evidence: a semantic witness failure must not
    /// discard the observation recorded when resolve succeeded.
    #[test]
    fn claim_result_for_outcome_preserves_resolve_realization_on_semantic_failure() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/tools/floor_effect_gate_witness.dag")
            .to_string_lossy()
            .into_owned();
        let (graph, indices) =
            resolve_entry_graph(&source_roots, &entry).expect("resolve compile-anchor entry");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let observation =
            Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 42 });
        let result = claim_result_for_outcome(
            &ctx,
            "dag_compile_clean_gate_passes".to_string(),
            entry,
            ClaimOutcome::NotBool {
                got: "unit".to_string(),
            },
            1,
            42,
            observation.clone(),
        );
        assert!(!result.ok);
        assert_eq!(result.resolve_realization, observation);
    }

    #[test]
    fn floor_finalization_preserves_resolve_evidence_when_business_claim_fails() {
        let fin = test_floor_finalization();
        let mut records = obligation_finalization_records(5, 3);
        records[0].results[0].ok = false;
        records[0].results[0].detail = "returned Bool(false)".to_string();
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals.iter().any(|m| {
                m.contains("missing realization observation") || m.contains("count law unevaluable")
            }),
            "semantic witness failure must not fabricate missing resolve evidence: {refusals:?}"
        );
    }

    fn resolve_observation_for_nanos(nanos: u64) -> Option<ResolveRealizationObservation> {
        if nanos > 0 {
            Some(ResolveRealizationObservation::ColdResolvePerformed {
                resolve_nanos: nanos as u128,
            })
        } else {
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool {
                computation_identity: "entry-closure:fixture".to_string(),
                provider_id: FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID.to_string(),
            })
        }
    }

    fn obligation_finalization_records(anchor_nanos: u64, native_nanos: u64) -> Vec<BatchRecord> {
        vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "obligation-fixture".to_string(),
            is_wet: false,
            results: vec![
                ClaimResult {
                    function: "dag_compile_clean_gate_passes".to_string(),
                    entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: anchor_nanos as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: resolve_observation_for_nanos(anchor_nanos),
                },
                ClaimResult {
                    function: "native_selected_logic_production_spec".to_string(),
                    entry:
                        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag"
                            .to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: native_nanos as u128,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: resolve_observation_for_nanos(native_nanos),
                },
            ],
        }]
    }

    /// The `<entry>::<function>` a refusal locates itself at — the same shape the
    /// production caller passes.
    const TEST_PLAN_SITE: &str = "src/v2/workflow/ci_floor_plan.dag::gunbc_ci_floor_plan";

    #[test]
    fn floor_finalization_refuses_unattributed_physical_resolve() {
        let fin = test_floor_finalization();
        let mut records = obligation_finalization_records(3, 0);
        records[0].results.push(ClaimResult {
            function: "extra_witness".to_string(),
            entry: "dag/test/claim/extra.dag".to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 42,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        });
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("unattributed physical resolve")),
            "extra cold resolve must refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_refuses_unattributed_physical_resolve_at_non_rostered_subject() {
        let fin = test_floor_finalization();
        let anchor_entry = TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY;
        let records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "emit-only".to_string(),
            is_wet: false,
            results: vec![ClaimResult {
                function: "emit_host_gate_passes".to_string(),
                entry: anchor_entry.to_string(),
                ok: true,
                detail: String::new(),
                wall_nanos: 0,
                resolve_nanos: 7,
                corpus_resolve_nanos: 0,
                corpus_eval_nanos: 0,
                corpus_witnesses: 0,
                runtime_unit_count: single_claim_runtime_unit_count(),
                witness_row_costs: Vec::new(),
                expectation_refusal: None,
                budget_refusal: None,
                host_dependency_refusal: None,
                resolve_realization: None,
            }],
        }];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals.iter().any(|m| {
                m.contains("unattributed physical resolve")
                    && m.contains(&format!("{anchor_entry}::emit_host_gate_passes"))
            }),
            "non-rostered subject with cold resolve_nanos must be named: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_names_unattributed_subject_when_roster_subject_also_missing() {
        let fin = test_floor_finalization();
        let anchor_entry = TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY;
        let mut records = obligation_finalization_records(5, 0);
        records[0]
            .results
            .retain(|r| r.function == "dag_compile_clean_gate_passes");
        records[0].results[0].resolve_realization = None;
        records[0].results.push(ClaimResult {
            function: "emit_host_gate_passes".to_string(),
            entry: anchor_entry.to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 7,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        });
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals.iter().any(|m| {
                m.contains("unattributed physical resolve")
                    && m.contains(&format!("{anchor_entry}::emit_host_gate_passes"))
            }),
            "unattributed subject must be named even when a roster subject is also missing: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("native_selected_logic_production_spec")),
            "missing roster subject must still be named: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_cheap_gate_cold_on_shared_obligation_entry_is_not_unattributed() {
        let fin = test_floor_finalization();
        let anchor_entry = "dag/tools/floor_effect_gate_witness.dag";
        let records = vec![
            BatchRecord {
                batch_index: 0,
                wall_nanos: 0,
                clamp_ms: None,
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                label: "cheap-gates".to_string(),
                is_wet: false,
                results: vec![ClaimResult {
                    function: "cheap_claim_pool_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 9,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 9 },
                    ),
                }],
            },
            BatchRecord {
                batch_index: 1,
                wall_nanos: 0,
                clamp_ms: None,
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                label: "compile-anchor".to_string(),
                is_wet: false,
                results: vec![ClaimResult {
                    function: "dag_compile_clean_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 0,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::SatisfiedFromSharedPool {
                            computation_identity: format!("entry-closure:{anchor_entry}:Hermetic"),
                            provider_id: FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID.to_string(),
                        },
                    ),
                }],
            },
            BatchRecord {
                batch_index: 2,
                wall_nanos: 0,
                clamp_ms: None,
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                label: "native-bundle".to_string(),
                is_wet: false,
                results: vec![ClaimResult {
                    function: "native_selected_logic_production_spec".to_string(),
                    entry:
                        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag"
                            .to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 3,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: resolve_observation_for_nanos(3),
                }],
            },
        ];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("unattributed physical resolve")),
            "cheap-gate cold on shared obligation entry must not refuse: {refusals:?}"
        );
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("floor resolve obligation missing")),
            "warm compile-clean must satisfy anchor obligation: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_refuses_duplicate_cold_on_obligation_entry() {
        let fin = test_floor_finalization();
        let anchor_entry = "dag/tools/floor_effect_gate_witness.dag";
        let records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            label: "memo-regression".to_string(),
            is_wet: false,
            results: vec![
                ClaimResult {
                    function: "cheap_claim_pool_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 9,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 9 },
                    ),
                },
                ClaimResult {
                    function: "dag_compile_clean_gate_passes".to_string(),
                    entry: anchor_entry.to_string(),
                    ok: true,
                    detail: String::new(),
                    wall_nanos: 0,
                    resolve_nanos: 7,
                    corpus_resolve_nanos: 0,
                    corpus_eval_nanos: 0,
                    corpus_witnesses: 0,
                    runtime_unit_count: single_claim_runtime_unit_count(),
                    witness_row_costs: Vec::new(),
                    expectation_refusal: None,
                    budget_refusal: None,
                    host_dependency_refusal: None,
                    resolve_realization: Some(
                        ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 7 },
                    ),
                },
            ],
        }];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("duplicate cold on obligation entry")),
            "second cold on rostered entry must refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_warm_native_includes_provider_receipt() {
        let fin = test_floor_finalization();
        let records = obligation_finalization_records(3, 0);
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("warm disposition without provider")),
            "warm native with provider must not refuse: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_refuses_warm_without_provider_observation() {
        let fin = test_floor_finalization();
        let mut records = obligation_finalization_records(3, 0);
        records[0].results[1].resolve_realization =
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool {
                computation_identity: "entry-closure:fixture".to_string(),
                provider_id: String::new(),
            });
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("warm disposition without provider id")),
            "fabricated warm without provider must refuse: {refusals:?}"
        );
    }

    // Floor finalization law 1 (in-executor form of the deleted resolve-receipt gate
    // step): transported obligation population must match observed realizations.
    #[test]
    fn floor_finalization_refuses_on_obligation_mismatch_both_directions() {
        let fin = test_floor_finalization();
        let mut partial = obligation_finalization_records(5, 0);
        partial[0]
            .results
            .retain(|r| r.function == "dag_compile_clean_gate_passes");
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &partial, false);
        assert!(
            refusals.iter().any(|m| {
                m.contains("count law unevaluable")
                    && m.contains("never executed on a completed walk")
                    && m.contains("native_selected_logic_production_spec")
            }),
            "missing transported obligation on complete walk must refuse unevaluable: {refusals:?}"
        );
        let under = [finalization_record(&[])];
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &under, false);
        assert!(
            refusals.iter().any(|m| m.contains("count law unevaluable")),
            "missing obligations on complete walk must refuse unevaluable: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_truncated_walk_reports_unevaluable_not_debt_change() {
        let fin = test_floor_finalization();
        let mut partial = obligation_finalization_records(5, 0);
        partial[0]
            .results
            .retain(|r| r.function == "dag_compile_clean_gate_passes");
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &partial, true);
        assert!(
            refusals.iter().any(|m| {
                m.contains("obligations not fully scheduled")
                    && m.contains("native_selected_logic_production_spec")
            }),
            "truncated walk must name never-ran obligations: {refusals:?}"
        );
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("differs from transported")),
            "truncated walk must not diagnose roster debt change: {refusals:?}"
        );
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("floor resolve obligation missing")),
            "truncated walk must not use missing-obligation debt wording: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("walk stopped before dependent batches")),
            "truncated walk must name truncation cause: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_warm_native_satisfies_obligations_with_one_cold() {
        let fin = test_floor_finalization();
        let records = obligation_finalization_records(3, 0);
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("floor resolve obligation")),
            "warm native + cold anchor must satisfy obligations: {refusals:?}"
        );
    }

    #[test]
    fn floor_finalization_matching_obligations_leaves_only_the_materialization_arm() {
        let fin = test_floor_finalization();
        let records = obligation_finalization_records(3, 9);
        let refusals = validate_floor_finalization(&fin, TEST_PLAN_SITE, &records, false);
        assert!(
            !refusals
                .iter()
                .any(|m| m.contains("differs from transported")),
            "obligation law must be satisfied: {refusals:?}"
        );
        assert!(
            refusals
                .iter()
                .any(|m| m.contains("floor materialization receipt missing")),
            "absent receipt must refuse, never pass: {refusals:?}"
        );
    }

    // --- floor-finalization DISPOSITION visibility (the fix this PR ships) -----------
    //
    // Three previously-silent cases, each with its own discriminating control so the
    // three cannot collapse back into one indistinguishable "nothing printed" bucket:
    //   A — finalization absent by construction (scoped floor worker)
    //   B — finalization absent incidentally (walk never reached the call)
    //   C — finalization HELD on an otherwise-failed floor (verdict must not vanish)

    #[test]
    fn floor_finalization_disposition_case_c_held_is_unconditional_on_refusals_alone() {
        // RED CONTROL for case C: `floor_finalization_disposition_from_refusals` takes
        // no `ordinary_failed` (or any other floor-outcome) parameter — so a held
        // verdict cannot be suppressed by floor outcome, because nothing in this call
        // chain can ever see it. If a future edit reintroduces suppression by inlining
        // an outcome check ahead of this call, this function's signature is what would
        // have to change to make it possible, which is the load-bearing fact this test
        // pins.
        let disposition = floor_finalization_disposition_from_refusals(Vec::new());
        assert_eq!(disposition, FloorFinalizationDisposition::Held);
        let lines = floor_finalization_disposition_lines(&disposition);
        assert_eq!(
            lines.len(),
            1,
            "held verdict must emit exactly one line: {lines:?}"
        );
        assert!(
            lines[0].contains("floor contract finalized"),
            "held line missing its verdict text: {lines:?}"
        );
    }

    #[test]
    fn floor_finalization_disposition_case_a_scoped_worker_is_absent_never_refused() {
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction
            )
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert_eq!(
            lines.len(),
            1,
            "absence must emit exactly one counted line: {lines:?}"
        );
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[scoped-worker-by-construction]"),
            "scoped absence must name its reason, not just say nothing: {lines:?}"
        );
        assert!(
            !lines[0].contains("REFUSED"),
            "scoped-by-construction absence must never read as a refusal: {lines:?}"
        );
    }

    #[test]
    fn floor_finalization_disposition_case_b_incidental_absence_is_distinct_from_scoped() {
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::IncidentalAbsence,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(FloorFinalizationAbsenceReason::IncidentalAbsence)
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[incidental-absence]"),
            "incidental absence must carry its own distinct marker: {lines:?}"
        );
        // The discriminating half of the control: A and B must never render identically,
        // or a reader is back to being unable to tell which one happened.
        let scoped_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            ));
        assert_ne!(
            lines, scoped_lines,
            "scoped-by-construction and incidental absence must render as distinguishable lines"
        );
    }

    #[test]
    fn floor_finalization_disposition_plan_declared_absence_is_distinct_from_incidental() {
        // Review 49917 (cursor/composer-2.5): the regen/plan-artifact/falsifier/
        // native-cache-cold plans always declare `NoFinalizationDeclared {}` — that is a
        // PLAN fact, expected on every one of their runs, and must not render as
        // `incidental-absence`, which exists to flag runs that were NOT supposed to skip.
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::PlanDeclaresNoFinalization,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::PlanDeclaresNoFinalization
            )
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[plan-declares-no-finalization]"),
            "plan-declared absence must carry its own distinct marker: {lines:?}"
        );
        let incidental_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::IncidentalAbsence,
            ));
        let scoped_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            ));
        assert_ne!(
            lines, incidental_lines,
            "a plan that always declares no finalization must not dilute the incidental bucket"
        );
        assert_ne!(lines, scoped_lines);
    }

    #[test]
    fn floor_finalization_disposition_undeclared_absence_still_emits_a_counted_line() {
        // Inverse of the defect an earlier revision of this PR reintroduced (review
        // 2026-08-07, smart-badger-549): a caller with no opinion on WHY finalization is
        // absent still cannot construct a silent outcome, because `absence_reason` is a
        // required `FloorFinalizationAbsenceReason`, not an `Option` a caller could pass
        // `None` for. `Undeclared` is that caller's only honest choice, and it renders
        // its own distinct, non-empty line — never nothing, and never mistaken for A or B.
        let disposition = floor_finalization_disposition(
            None,
            FloorFinalizationAbsenceReason::Undeclared,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        assert_eq!(
            disposition,
            FloorFinalizationDisposition::Absent(FloorFinalizationAbsenceReason::Undeclared)
        );
        let lines = floor_finalization_disposition_lines(&disposition);
        assert_eq!(
            lines.len(),
            1,
            "undeclared absence must still emit exactly one counted line: {lines:?}"
        );
        assert!(
            lines[0].contains("FLOOR-FINALIZATION-ABSENT[undeclared]"),
            "undeclared absence must name itself, not read as A or B: {lines:?}"
        );
        let scoped_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::ScopedWorkerByConstruction,
            ));
        let incidental_lines =
            floor_finalization_disposition_lines(&FloorFinalizationDisposition::Absent(
                FloorFinalizationAbsenceReason::IncidentalAbsence,
            ));
        assert_ne!(lines, scoped_lines);
        assert_ne!(lines, incidental_lines);
    }

    #[test]
    fn run_walk_emits_the_finalization_disposition_through_the_injected_sink() {
        // Discriminating control for "nothing proves the verdict is emitted" (review
        // 2026-08-07, smart-badger-549): this calls the REAL run_walk, not a pure
        // disposition function, and inspects the REAL bytes written through the sink
        // production passes as stderr. Deleting or bypassing the
        // `emit_floor_finalization_disposition` call inside `run_walk` reds this test;
        // testing `floor_finalization_disposition_lines` alone cannot.
        let mut sink: Vec<u8> = Vec::new();
        let _outcome = run_walk(
            &[],
            TEST_PLAN_SITE,
            &[],
            &[],
            None,
            FloorFinalizationAbsenceReason::IncidentalAbsence,
            &mut sink,
            None,
            None,
            None,
            &RealizationConcurrency::for_walk(1).expect("test schedule"),
            None,
            FalsifierSelfHostWetBudgets::default(),
            FloorBatchStopPolicy::StopBeforeDependents,
            None,
            None,
            false,
            None,
        );
        let emitted = String::from_utf8(sink).expect("emitted bytes must be valid utf-8");
        assert!(
            emitted.contains("FLOOR-FINALIZATION-ABSENT[incidental-absence]"),
            "run_walk must actually write the disposition line through its sink, not just compute it: {emitted:?}"
        );
    }

    #[test]
    fn floor_finalization_disposition_refused_still_carries_the_law_message() {
        // Sanity: wiring the disposition wrapper through validate_floor_finalization
        // preserves the existing REFUSED law text — this PR changes visibility, never
        // the laws' semantics (constraint from the brief).
        let fin = test_floor_finalization();
        let disposition = floor_finalization_disposition(
            Some(&fin),
            FloorFinalizationAbsenceReason::Undeclared,
            TEST_PLAN_SITE,
            &[],
            false,
        );
        match &disposition {
            FloorFinalizationDisposition::Refused(refusals) => {
                assert!(
                    !refusals.is_empty(),
                    "empty batch records must refuse unexecuted obligations"
                );
            }
            other => panic!("expected Refused with no batch records, got {other:?}"),
        }
        let lines = floor_finalization_disposition_lines(&disposition);
        assert!(
            lines
                .iter()
                .all(|l| l.starts_with("FLOOR-FINALIZATION-REFUSED: ")),
            "every refusal line must carry the located, typed marker: {lines:?}"
        );
    }

    // D2 wiring pin: the fast-exit consumes exactly this mapping, so the terminal
    // process code stays behavior-identical to the ExitCode return it replaced.
    #[test]
    fn walk_exit_code_maps_failure_to_one_success_to_zero() {
        assert_eq!(walk_exit_code(true), 1);
        assert_eq!(walk_exit_code(false), 0);
    }

    #[test]
    fn walk_terminal_detail_carries_located_refusal_not_bare_exit_code() {
        let detail = walk_terminal_detail(
            true,
            &[
                "batch=3 fn=discovery-corpus detail=25 of 8158 discovery witness(es) failed"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("batch=3 fn=discovery-corpus"),
            "the terminal receipt must carry the located batch refusal, got: {detail}"
        );
        assert!(
            detail.contains("mode=WitnessRed"),
            "the terminal receipt must carry the typed mode, got: {detail}"
        );
        assert!(
            !detail.trim().eq("walk terminal exit code 1"),
            "a bare exit-code string is not a located refusal: {detail}"
        );
        assert_eq!(
            walk_terminal_detail(false, &[], &[], false, true),
            "walk terminal exit code 0"
        );
    }

    #[test]
    fn walk_terminal_detail_surfaces_post_walk_compile_clean_refusals() {
        let detail = walk_terminal_detail(true, &[], &[], true, false);
        assert!(detail.contains("compile_clean_over_budget"));
        assert!(detail.contains("compile_clean_cost_drift_receipt_refused"));
    }

    /// Receipted cold-corpus shape: ten typed witness findings plus one eval-budget fact
    /// and a walk soft-deadline interrupt must emit WitnessRed, not BudgetExceeded-only mode.
    #[test]
    fn walk_terminal_mode_witness_findings_outrank_budget_interrupt() {
        let detail = walk_terminal_detail(
            true,
            &[
                "batch=2 fn=discovery-corpus detail=10 of 8158 discovery witness(es) failed: \
                 english_emit_add_prose_holds StaleKnownRed; \
                 realization_vocab_live_corpus_receipt_holds returned Bool(false)"
                    .to_string(),
                "batch=2 fn=discovery-corpus detail=3 of 8158 discovery witness(es) failed: \
                 cannot cast List to List"
                    .to_string(),
                "walk reached its soft deadline before batch 9: deadline_ms=9000000 \
                 elapsed_ms=9000123"
                    .to_string(),
                "batch=7 fn=explicit-corpus detail=wave1_gate1_d eval budget exceeded: \
                 180002ms elapsed > 180000ms substrate long lane budget"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("mode=WitnessRed"),
            "semantic findings must set terminal mode, got: {detail}"
        );
        assert!(
            !detail.contains("mode=BudgetExceeded"),
            "budget interrupt must not collapse terminal mode: {detail}"
        );
        assert!(
            detail
                .contains("ci_failure_class_arm=FloorFailed{class:Structural{reason:WitnessRed}}"),
            "arm must carry WitnessRed reason: {detail}"
        );
        assert!(
            detail.contains("soft deadline"),
            "interrupt evidence must remain in detail: {detail}"
        );
        assert!(
            detail.contains("eval budget exceeded"),
            "budget lower bound must remain in detail: {detail}"
        );
    }

    #[test]
    fn walk_terminal_mode_budget_only_interrupt_stays_budget_exceeded() {
        let detail = walk_terminal_detail(
            true,
            &[
                "walk reached its soft deadline before batch 3: deadline_ms=9000000 \
                 elapsed_ms=9000123"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("mode=BudgetExceeded"),
            "a lone deadline interrupt with no witness findings stays BudgetExceeded: {detail}"
        );
    }

    /// The discriminating control for dropping the `batch_rows > 1` short-circuit. Two
    /// batch rows that are BOTH pure budget interrupts must stay BudgetExceeded: the count
    /// arm classified them WitnessRed, reporting a cost interrupt as a semantic verdict —
    /// the mirror of the defect this classifier exists to fix. The sibling budget-only test
    /// above carries zero batch rows, so it cannot reach this arm; without this case the
    /// removal is indistinguishable from never having had the bug.
    #[test]
    fn walk_terminal_mode_two_budget_only_batch_rows_stay_budget_exceeded() {
        let detail = walk_terminal_detail(
            true,
            &[
                "batch=7 fn=explicit-corpus detail=wave1_gate1_d eval budget exceeded: \
                 180002ms elapsed > 180000ms substrate long lane budget"
                    .to_string(),
                "batch=8 fn=explicit-corpus detail=wave2_gate3_a eval budget exceeded: \
                 180011ms elapsed > 180000ms substrate long lane budget"
                    .to_string(),
                "walk reached its soft deadline before batch 9: deadline_ms=9000000 \
                 elapsed_ms=9000123"
                    .to_string(),
            ],
            &[],
            false,
            true,
        );
        assert!(
            detail.contains("mode=BudgetExceeded"),
            "two budget-only batch rows carry no semantic finding: {detail}"
        );
        assert!(
            !detail.contains("mode=WitnessRed"),
            "row count must not stand in for the semantic property: {detail}"
        );
    }

    #[test]
    fn push_ordinary_receipt_write_refusals_locates_each_failed_writer() {
        let mut details = Vec::new();
        push_ordinary_receipt_write_refusals(&mut details, true, false, true, true, false, true);
        assert_eq!(details.len(), 2);
        assert!(details
            .iter()
            .any(|d| d.contains("batch wall receipt write refused")));
        assert!(details
            .iter()
            .any(|d| d.contains("floor component receipt write refused")));
    }

    #[test]
    fn emit_worker_terminal_before_return_journals_pre_walk_refusal() {
        let dir = std::env::temp_dir().join(format!(
            "claim-executor-worker-terminal-before-return-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&dir);
        let terminal = dir.join("worker-terminal.tsv");
        let journal = dir.join("floor-phase-journal.tsv");
        let _ = fs::remove_file(&terminal);
        let _ = fs::remove_file(&journal);
        fs::write(
            &terminal,
            "refused\tscoped witness scheduling refusal: fixture detail\n",
        )
        .expect("write worker terminal fixture");
        std::env::set_var(FLOOR_WORKER_TERMINAL_ENV, &terminal);
        std::env::set_var(FLOOR_PHASE_JOURNAL_ENV, &journal);
        let code = emit_worker_terminal_before_return(ExitCode::from(1));
        std::env::remove_var(FLOOR_WORKER_TERMINAL_ENV);
        std::env::remove_var(FLOOR_PHASE_JOURNAL_ENV);
        assert_eq!(code, ExitCode::from(1));
        let persisted = fs::read_to_string(&journal).expect("walk-terminal journal row");
        let _ = fs::remove_dir_all(&dir);
        assert!(
            persisted.contains("\twalk-terminal\tfailed\tscoped witness scheduling refusal"),
            "pre-walk worker refusal must reach the durable journal: {persisted:?}"
        );
    }

    fn outcome(entry: &str, function: &str, o: ClaimOutcome) -> DiscoveryWitnessOutcome {
        DiscoveryWitnessOutcome {
            entry: entry.to_string(),
            module_path: "test.m".to_string(),
            function: function.to_string(),
            outcome: o,
            execution_leg: "InterpretedLeg".to_string(),
        }
    }

    fn stale_known_red_result(refusal: Option<ExpectationRefusal>) -> ClaimResult {
        ClaimResult {
            function: "discovery-corpus".to_string(),
            entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
            ok: refusal.is_none(),
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: runtime_unit_count_unavailable("test fixture"),
            witness_row_costs: Vec::new(),
            expectation_refusal: refusal,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        }
    }

    /// THE DISCRIMINATING PAIR for the expectation axis. Either half alone is satisfiable by
    /// a broken mechanism: a classifier that never refuses passes the still-red half, and one
    /// that always refuses passes the unexpected-green half. Both must hold at once.
    #[test]
    fn expected_red_still_red_is_agreement_and_unexpected_green_refuses() {
        let expected_red = vec![(
            "src/v2/test/claim/long/direct_rust_door_production_group_test.dag".to_string(),
            "direct_rust_door_production_group_closing_expectation_holds".to_string(),
        )];

        // HALF ONE — still red is AGREEMENT: not a failure, no refusal, and the component
        // receipt reports no failure mode at all, so the alert has nothing to notify on.
        let still_red = classify_witness_expectations(
            &[outcome(
                "src/v2/test/claim/long/direct_rust_door_production_group_test.dag",
                "direct_rust_door_production_group_closing_expectation_holds",
                ClaimOutcome::Fail,
            )],
            &expected_red,
        );
        assert_eq!(still_red.agreements.len(), 1);
        assert!(still_red.unexpected_failures.is_empty());
        assert_eq!(still_red.refusal(), None);
        let (mode, _) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![stale_known_red_result(
                still_red.refusal(),
            )]));
        assert_eq!(
            mode, "none",
            "a known-red witness holding its quarantine must not reach the alert as a failing component"
        );

        // HALF TWO — unexpected green REFUSES, typed and naming the row to delete. It must
        // not be `WitnessRed`: an un-quarantine and a regression have opposite remedies and
        // must not share a class signature.
        let went_green = classify_witness_expectations(
            &[outcome(
                "src/v2/test/claim/long/direct_rust_door_production_group_test.dag",
                "direct_rust_door_production_group_closing_expectation_holds",
                ClaimOutcome::Pass,
            )],
            &expected_red,
        );
        assert!(went_green.agreements.is_empty());
        let refusal = went_green
            .refusal()
            .expect("an expected-red witness that ran green must refuse, never pass quietly");
        assert!(
            matches!(&refusal, ExpectationRefusal::StaleKnownRed { witnesses } if witnesses.len() == 1),
            "the stale arm, not the coverage arm: this witness WAS observed, running green"
        );
        let (mode, detail) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![stale_known_red_result(
                Some(refusal),
            )]));
        assert_eq!(mode, STALE_KNOWN_RED_MODE);
        assert_ne!(mode, "WitnessRed");
        assert_ne!(mode, "none");
        assert!(
            detail.contains("direct_rust_door_production_group_closing_expectation_holds"),
            "the refusal must name the identity to un-quarantine, got: {detail}"
        );
    }

    /// ONE ROW PER DISPOSITION of the required result table, each discriminating.
    ///
    /// The table exists because the first classifier computed `green = (outcome == Pass)` and
    /// swept every other outcome into the agreement arm. Each row below is an outcome that
    /// arm would have blessed as "the quarantine held" while the assertion produced no verdict
    /// at all.
    #[test]
    fn expected_red_disposition_table_distinguishes_every_outcome() {
        let entry = "src/v2/test/claim/long/direct_rust_door_production_group_test.dag";
        let f = "direct_rust_door_production_group_closing_expectation_holds";
        let expected_red = vec![(entry.to_string(), f.to_string())];

        let row = |o: ClaimOutcome| {
            classify_witness_expectations_in(
                &[outcome(entry, f, o)],
                &expected_red,
                &[(entry.to_string(), f.to_string())],
            )
        };

        // ROW 1 — assertion executed, returned false: the ONLY agreement.
        let t = row(ClaimOutcome::Fail);
        assert_eq!(t.agreements.len(), 1);
        assert_eq!(t.hard_failures(), 0);
        assert!(t.evidence_absent.is_empty());

        // ROW 2 — assertion executed, returned true: stale quarantine, a typed refusal.
        let t = row(ClaimOutcome::Pass);
        assert!(t.agreements.is_empty());
        assert!(t.refusal().is_some());

        // ROW 3 — timed out: a BUDGET failure. An interruption plus a lower bound on cost is
        // never a semantic verdict, so it must not read as a quarantine holding.
        let t = row(ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 5001,
            budget_ms: 5000,
            // The VARIANT now says which case this is, so 5001 is a lower bound by
            // construction: the completed row is a different arm the fixture cannot reach by
            // omitting a field.
            kind: BudgetKind::Cpu,
        });
        assert!(
            t.agreements.is_empty(),
            "a budget kill must never count as agreement — the assertion did not produce a verdict"
        );
        assert_eq!(
            t.refusal(),
            None,
            "a budget kill is not a stale quarantine either"
        );
        assert_eq!(
            t.expected_red_without_verdict,
            vec![(
                entry.to_string(),
                f.to_string(),
                ExpectedRedDisposition::BudgetFailure
            )]
        );
        assert_eq!(t.hard_failures(), 1, "it must red the batch");

        // ROW 4 — interpreter fault, and ROW 4b — non-Bool referent: no verdict produced.
        // Row 5 of the table (refused BEFORE evaluation) shares this arm today because
        // `run_claim` formats the typed error to a String; that is declared on the variant,
        // not papered over by sniffing the message.
        for o in [
            ClaimOutcome::RuntimeError {
                cause: v1_compiler::cli_run::WitnessRuntimeCause::NoSuchVariable,
                message: "undefined variable: X".into(),
            },
            ClaimOutcome::NotBool {
                got: "Record".into(),
            },
        ] {
            let t = row(o);
            assert!(t.agreements.is_empty());
            assert_eq!(
                t.expected_red_without_verdict
                    .iter()
                    .map(|(_, _, d)| d.clone())
                    .collect::<Vec<_>>(),
                vec![ExpectedRedDisposition::InfrastructureOrReferentFailure]
            );
            assert_eq!(t.hard_failures(), 1);
        }

        // ROW 6 — no observation at all: ABSENT EVIDENCE, which is not a verdict in either
        // direction. Not agreement and not a witness failure — but it REFUSES, on its own
        // coverage mode. This row previously asserted `refusal() == None`, which is exactly the
        // hole the coverage arm closes: absence is not a verdict, and it is also not silence.
        let t = classify_witness_expectations_in(
            &[],
            &expected_red,
            &[(entry.to_string(), f.to_string())],
        );
        assert!(t.agreements.is_empty());
        assert_eq!(t.hard_failures(), 0, "the remedy is coverage, not code");
        assert_eq!(t.evidence_absent, vec![(entry.to_string(), f.to_string())]);
        assert!(
            matches!(
                t.refusal(),
                Some(ExpectationRefusal::ExpectedRedEvidenceAbsent { .. })
            ),
            "the coverage arm, distinct from the stale arm rows 2 and 5 exercise"
        );
    }

    /// The regression this table was written for, stated as one assertion: under the `!= Pass`
    /// shortcut EVERY one of these blessed the quarantine. None may.
    #[test]
    fn no_verdict_outcome_is_ever_agreement_for_an_expected_red_witness() {
        let entry = "e.dag";
        let f = "witness_holds";
        let expected_red = vec![(entry.to_string(), f.to_string())];
        for o in [
            ClaimOutcome::BudgetInterrupted {
                elapsed_at_least_ms: 1,
                budget_ms: 1,
                kind: BudgetKind::Wall,
            },
            ClaimOutcome::RuntimeError {
                cause: v1_compiler::cli_run::WitnessRuntimeCause::TypeError,
                message: "boom".into(),
            },
            ClaimOutcome::NotBool { got: "Int".into() },
        ] {
            let t = classify_witness_expectations(&[outcome(entry, f, o.clone())], &expected_red);
            assert!(
                t.agreements.is_empty() && t.hard_failures() == 1,
                "{o:?} must produce no agreement and must red the batch"
            );
        }
        // …while the one that genuinely ran and was false still does agree, so the fix is not
        // simply "nothing agrees any more".
        let t =
            classify_witness_expectations(&[outcome(entry, f, ClaimOutcome::Fail)], &expected_red);
        assert_eq!(t.agreements.len(), 1);
        assert_eq!(t.hard_failures(), 0);
    }

    /// Expectation is a property of the WITNESS, so a quarantined function and a green
    /// sibling in the SAME FILE are classified independently — and a mixed batch stays
    /// ordinary instead of reverting to one polarity for everything in it.
    #[test]
    fn expected_red_is_function_grain_not_file_or_batch_grain() {
        let entry = "src/v2/test/claim/manual/english_emit_add_test.dag";
        let expected_red = vec![(
            entry.to_string(),
            "logic_complement_truth_table".to_string(),
        )];

        let tally = classify_witness_expectations(
            &[
                outcome(entry, "logic_complement_truth_table", ClaimOutcome::Fail),
                outcome(entry, "logic_sibling_that_must_hold", ClaimOutcome::Pass),
                outcome(
                    "other/witness_test.dag",
                    "unrelated_holds",
                    ClaimOutcome::Fail,
                ),
            ],
            &expected_red,
        );
        assert_eq!(tally.agreements.len(), 1, "the quarantined fn is agreement");
        assert_eq!(
            tally.refusal(),
            None,
            "its green sibling is not a stale quarantine"
        );
        assert_eq!(
            tally.unexpected_failures.len(),
            1,
            "an ordinary red in a batch containing a known-red row is still an ordinary red — \
             the batch-wide flag this replaced would have inverted it to a pass"
        );

        // The green SIBLING going red is an ordinary failure, not agreement: file placement
        // never confers polarity.
        let sibling_red = classify_witness_expectations(
            &[outcome(
                entry,
                "logic_sibling_that_must_hold",
                ClaimOutcome::Fail,
            )],
            &expected_red,
        );
        assert!(sibling_red.agreements.is_empty());
        assert_eq!(sibling_red.unexpected_failures.len(), 1);
    }

    /// The one surviving batch-wide consult, kept narrow twice over: a corpus resolve refuse
    /// produces no per-witness outcomes, so agreement requires that every entry declared
    /// `ExpectTypedPreVerdictRefusal` — an ordinary known-red row is NOT enough.
    ///
    /// The discriminating half is the second block. A batch of ordinary known-red rows and a
    /// batch of declared pre-verdict rows are both "all expected-red"; only the second may
    /// invert a refuse. Without that block this test would pass against the defect it exists
    /// to hold closed, because the defect's population is a superset of this one's.
    /// The pre-verdict arm is NON-GREEN, and this OBSERVES the result rather than restating
    /// the constant.
    ///
    /// The first shape of this test compared string constants to each other and to a literal,
    /// while its doc comment claimed "if someone restores the green arm, the first assertion
    /// fails". It did not: nothing executed the arm, so flipping `ok` back to `true` left every
    /// assertion green — a stated regression control that did not exist. That is the same
    /// defect class as a stated identity join that was really a length agreement, and it was
    /// caught the same way, by someone checking the claim against what the code drives. The
    /// arm's construction is now extracted to one function and this asserts its `ok`.
    #[test]
    fn declared_pre_verdict_refusal_is_non_green_and_carries_its_own_mode() {
        let r = pre_verdict_unverified_claim_result("known-red probe", 3, "resolve refused");

        // THE REGRESSION CONTROL, and it now observes the value: restoring the green arm
        // flips this and the test fails.
        assert!(
            !r.ok,
            "a declaration classifies a stop; it does not verify it"
        );

        // THE MODE MUST REACH THE RECEIPT OFF THE VALUE, not off the prose. Carried only in
        // `function`/`detail`, `batch_failure_mode_and_detail` fell through to
        // `falsifier_failure_mode` and reported these batches as ordinary "WitnessRed" while
        // the .dag vocabulary said Refused — one mode in two representations, the second
        // guessed back from a string, which is the defect this PR exists to remove.
        // cursor review 50221 found it; this assertion is what would have caught it.
        let (mode, receipt_detail) = batch_failure_mode_and_detail(&batch_record_for_test(vec![
            pre_verdict_unverified_claim_result("known-red probe", 3, "resolve refused"),
        ]));
        assert_eq!(
            mode, EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
            "the receipt must classify structurally, never as WitnessRed"
        );
        assert_ne!(mode, "WitnessRed");
        assert!(receipt_detail.contains(EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE));
        assert!(matches!(
            &r.expectation_refusal,
            Some(ExpectationRefusal::PreVerdictUnverified { .. })
        ));
        assert!(
            r.detail.contains(EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE),
            "the refusal must be typed at the mode, not left as prose"
        );
        assert!(
            r.detail.contains("resolve refused"),
            "the located cause is carried"
        );
        assert!(
            r.detail.contains('3'),
            "the declared entry count is carried"
        );
        assert!(r
            .function
            .contains(EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE));
        // It is a refusal, not a verdict about any witness: no per-witness populations.
        // (The typed refusal itself IS carried — asserted above; an earlier
        // `expectation_refusal == None` here contradicted that assertion and made the whole
        // test unexecutable, so it never observed anything it claimed to.)
        assert!(r.witness_row_costs.is_empty());

        // Distinct from BOTH siblings on the same axis — different remedies, different modes.
        assert_ne!(
            EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
            EXPECTED_RED_EVIDENCE_ABSENT_MODE
        );
        assert_ne!(
            EXPECTED_RED_PRE_VERDICT_UNVERIFIED_MODE,
            ExpectationRefusal::StaleKnownRed {
                witnesses: Vec::new()
            }
            .mode()
        );
    }

    #[test]
    fn resolve_refuse_agreement_requires_every_entry_expect_a_pre_verdict_refusal() {
        let pre_verdict = (
            "src/v2/test/claim/manual/english_emit_add_test.dag".to_string(),
            "logic_complement_truth_table".to_string(),
        );
        let ordinary_known_red = (
            "dag/test/claim/observation_raw_print_retirement_acceptance_test.dag".to_string(),
            "observation_emit_frontier_is_zero".to_string(),
        );
        let ordinary = (
            "dag/test/claim/design_register_lift_parity_witness_test.dag".to_string(),
            "design_register_lift_parity_holds".to_string(),
        );
        let pre_verdict_roster = vec![pre_verdict.clone()];

        assert!(batch_entries_all_expect_pre_verdict_refusal(
            &[pre_verdict.clone()],
            &pre_verdict_roster
        ));
        assert!(
            !batch_entries_all_expect_pre_verdict_refusal(
                &[pre_verdict.clone(), ordinary.clone()],
                &pre_verdict_roster
            ),
            "one ordinary entry must keep a resolve refuse loud"
        );
        assert!(!batch_entries_all_expect_pre_verdict_refusal(
            &[ordinary.clone()],
            &pre_verdict_roster
        ));
        assert!(!batch_entries_all_expect_pre_verdict_refusal(
            &[],
            &pre_verdict_roster
        ));

        // THE DISCRIMINATION. Both rows below are known-red; only one declared that its
        // agreement is a stop before any verdict. An arbitrary resolve failure in a batch of
        // the other must NOT report every quarantine as holding.
        let expected_red = vec![pre_verdict.clone(), ordinary_known_red.clone()];
        assert!(
            batch_entries_all_in(&[ordinary_known_red.clone()], &expected_red),
            "the ordinary row IS expected-red — this is the population the defect consulted"
        );
        assert!(
            !batch_entries_all_expect_pre_verdict_refusal(
                &[ordinary_known_red],
                &pre_verdict_roster
            ),
            "but it did NOT declare a pre-verdict refusal, so a resolve failure in its batch is \
             an infrastructure fact about the run and proves neither the red nor the quarantine"
        );
    }

    /// THE BUDGET AXIS, separated. A witness obtains its long eval ceiling from its OWN
    /// declared budget, without joining the long-lane BATCH roster — which is what forced an
    /// expensive known-red root to occur twice in the schedule, the second time on a batch
    /// that expects green.
    #[test]
    fn declared_long_budget_arms_ceiling_without_joining_the_long_lane_batch() {
        let known_red_long = "src/v2/test/claim/long/direct_rust_door_production_group_test.dag";
        // The union `witness_long_eval_budget_entries` produces: this path is present because
        // its ADMISSION row declares SubstrateLongLaneEvalBudget, not because it is on the
        // long-lane batch roster.
        let budgets = FalsifierSelfHostWetBudgets {
            substrate_long_lane_entry_paths: vec![known_red_long.to_string()],
            substrate_long_lane_eval_budget_ms: Some(180_000),
            expected_red_witnesses: vec![(
                known_red_long.to_string(),
                "direct_rust_door_production_group_closing_expectation_holds".to_string(),
            )],
            ..Default::default()
        };
        let batch = vec![(
            known_red_long.to_string(),
            "direct_rust_door_production_group_closing_expectation_holds".to_string(),
        )];

        // Both axes hold at once on one witness in one batch: the long ceiling AND
        // expected-red polarity. Under the fused shape this pair was inexpressible.
        assert_eq!(
            select_discovery_batch_budgets(
                ExecutionMode::Hermetic,
                &batch,
                Some(5_000),
                &budgets
            )
            .eval_budget_ms,
            Some(180_000),
            "the declared budget must arm the ceiling, or a known-red row reds at 5s for the wrong reason"
        );
        assert!(batch_entries_all_in(
            &batch,
            &budgets.expected_red_witnesses
        ));
    }

    /// P0-1. ABSENT EVIDENCE STOPS THE LINE. A rostered expected-red witness that produced no
    /// observation is neither agreement nor failure — and the arm that matters is that it is
    /// also not SILENCE. The first shape of this bin only wrote it to stderr, so a batch whose
    /// admitted red control had quietly stopped executing still reported green: the roster
    /// claimed a discriminating red existed, the run produced no evidence either way, and the
    /// green report was read as though it had.
    #[test]
    fn absent_expected_red_evidence_refuses_as_coverage_never_as_a_verdict() {
        let entry = "src/v2/test/claim/long/direct_rust_door_production_group_test.dag";
        let f = "direct_rust_door_production_group_closing_expectation_holds";
        let expected_red = vec![(entry.to_string(), f.to_string())];
        let rostered = expected_red.clone();

        // Nothing came back for the rostered row.
        let tally = classify_witness_expectations_in(&[], &expected_red, &rostered);
        assert_eq!(tally.evidence_absent.len(), 1);
        assert!(
            tally.agreements.is_empty() && tally.unexpected_failures.is_empty(),
            "absence is not a verdict in either direction"
        );
        assert_eq!(
            tally.hard_failures(),
            0,
            "and it must not be counted as a witness failure — the remedy is coverage, not code"
        );

        let refusal = tally
            .refusal()
            .expect("absence must refuse; logging it and passing is coverage by illusion");
        assert!(matches!(
            refusal,
            ExpectationRefusal::ExpectedRedEvidenceAbsent { .. }
        ));
        let (mode, detail) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![stale_known_red_result(
                Some(refusal),
            )]));
        assert_eq!(mode, EXPECTED_RED_EVIDENCE_ABSENT_MODE);
        assert_ne!(mode, STALE_KNOWN_RED_MODE, "nobody observed it passing");
        assert_ne!(mode, "none");
        assert!(detail.contains(f), "the refusal must name what did not run");

        // POSITIVE CONTROL: the same roster, observed. No coverage refusal.
        let observed = classify_witness_expectations_in(
            &[outcome(entry, f, ClaimOutcome::Fail)],
            &expected_red,
            &rostered,
        );
        assert!(observed.evidence_absent.is_empty());
        assert_eq!(observed.refusal(), None);
        assert_eq!(observed.agreements.len(), 1);
    }

    /// P1, second half. The admission authority is an IDENTITY JOIN over witness outcomes, so a
    /// bucket population that agrees in COUNT while disagreeing in identity must still refuse.
    /// A count equality alone cannot see this; that is the whole reason the join replaced it.
    #[test]
    fn non_pass_join_matches_identities_not_merely_counts() {
        let entry = "src/v2/test/claim/manual/english_emit_add_test.dag";
        let expected_red = vec![(
            entry.to_string(),
            "logic_complement_truth_table".to_string(),
        )];
        let mut tally = classify_witness_expectations(
            &[outcome(
                entry,
                "logic_complement_truth_table",
                ClaimOutcome::Fail,
            )],
            &expected_red,
        );
        assert!(tally.non_pass_join_is_complete());
        assert!(still_red_batch_passes(&tally, 1));

        // Same COUNT on both sides — one seen non-pass, one agreement — but the agreement
        // names a different witness. The count equality is satisfied; the join is not.
        tally.agreements = vec![(entry.to_string(), "some_other_witness".to_string())];
        assert_eq!(
            tally.classified_non_pass.len(),
            tally.accounted_non_pass().len()
        );
        assert!(
            !tally.non_pass_join_is_complete(),
            "equal counts over different identities must refuse"
        );
        assert!(
            !still_red_batch_passes(&tally, 1),
            "the join, not the count, is the admission authority"
        );
    }

    /// P1. The still-red pass arm joins on an EXACT count, not `<=`. A failure that arrives
    /// without a per-witness identity must refuse rather than be read as "nothing failed" —
    /// the empty-observation narrow, applied to the batch's own accounting.
    #[test]
    fn still_red_pass_arm_requires_exact_failure_accounting() {
        let entry = "src/v2/test/claim/manual/english_emit_add_test.dag";
        let f = "logic_complement_truth_table";
        let expected_red = vec![(entry.to_string(), f.to_string())];
        let tally =
            classify_witness_expectations(&[outcome(entry, f, ClaimOutcome::Fail)], &expected_red);

        assert_eq!(tally.hard_failures(), 0);
        assert_eq!(tally.classified_non_pass.len(), 1);

        // Lockstep: one classified non-pass, one summary failure. Passes.
        assert!(still_red_batch_passes(&tally, 1));
        // An extra summary failure with no matching per-witness outcome must REFUSE. Under
        // `<=` this returned true, so unjoinable failures were absorbed into a green batch.
        assert!(
            !still_red_batch_passes(&tally, 2),
            "an unjoinable failure must refuse, never be absorbed"
        );
        // And fewer, which is the same accounting break in the other direction.
        assert!(!still_red_batch_passes(&tally, 0));

        // A real unexpected failure still reds regardless of the accounting.
        let with_red = classify_witness_expectations(
            &[outcome(entry, "an_ordinary_sibling", ClaimOutcome::Fail)],
            &expected_red,
        );
        assert_eq!(with_red.hard_failures(), 1);
        assert!(!still_red_batch_passes(&with_red, 1));
    }

    #[test]
    fn self_host_wet_roster_intersection_scopes_wall_budget() {
        let roster = vec![
            "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
            "dag/test/claim/namespace_import_closure_witness_test.dag".into(),
        ];
        assert!(discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
                "self_host_logic_behavioral_receipt_holds".into()
            )],
            &roster
        ));
        // A non-self-host wet witness must not inherit the 600s whole-receipt ceiling.
        assert!(!discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/codex_package_delivery_wet_witness_test.dag".into(),
                "materialize_codex_runtime_bundle_produces_native_executable_holds".into()
            )],
            &roster
        ));
        assert!(!discovery_entries_intersect_roster(&[], &roster));
        assert!(!discovery_entries_intersect_roster(
            &[(
                "dag/test/claim/self_host_logic_behavioral_witness_test.dag".into(),
                "self_host_logic_behavioral_receipt_holds".into()
            )],
            &[]
        ));
    }

    fn batch_record_for_test(results: Vec<ClaimResult>) -> BatchRecord {
        let unit_count = results.len() as u128;
        BatchRecord {
            batch_index: 0,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count,
            runtime_units: FloorRuntimeUnitCount::Observed { units: unit_count },
            results,
            label: "batch-under-test".to_string(),
            is_wet: false,
        }
    }

    /// A budget kill must classify as BudgetExceeded from the VALUE, not from the message.
    ///
    /// RED control: the detail string here deliberately contains none of the substrings
    /// `falsifier_failure_mode` looks for, so under the old string-sniffing path this row
    /// would fall through to the `else` arm and report `WitnessRed` — silently swapping
    /// "re-basis a dated ceiling" for "fix the witness". Passing proves the mode is read
    /// off `budget_refusal`, so message wording can change without moving the class.
    #[test]
    fn budget_kill_classifies_structurally_not_by_message_text() {
        let timed_out = ClaimResult {
            function: "some_witness".to_string(),
            entry: "some_witness_test.dag".to_string(),
            ok: false,
            detail: "wording that mentions no budget phrase at all".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: Some(BudgetRefusal {
                elapsed_ms: 900_001,
                budget_ms: 900_000,
                kind: BudgetKind::Wall,
                completion: v1_compiler::cli_run::BudgetCompletion::Interrupted,
            }),
            host_dependency_refusal: None,
            resolve_realization: None,
        };
        // Sanity: the string classifier alone really would misclassify this detail.
        assert_eq!(
            falsifier_failure_mode(&[timed_out.detail.clone()]),
            "WitnessRed",
            "control is only meaningful if the string path would get this wrong"
        );

        let rec = batch_record_for_test(vec![timed_out]);
        let (mode, _detail) = batch_failure_mode_and_detail(&rec);
        assert_eq!(mode, "BudgetExceeded");

        // And an ordinary witness red must NOT be dragged into BudgetExceeded.
        let plain = ClaimResult {
            function: "other_witness".to_string(),
            entry: "other_witness_test.dag".to_string(),
            ok: false,
            detail: "returned Bool(false)".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        };
        let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![plain]));
        assert_eq!(mode, "WitnessRed");
    }

    /// A host dependency refusal must classify as HostDependencyAbsent from the VALUE,
    /// not WitnessRed — same structural rule as budget_kill_classifies_structurally_not_by_message_text.
    #[test]
    fn host_dependency_absent_classifies_structurally_not_as_witness_red() {
        let npm_absent = ClaimResult {
            function: "materialize_codex_runtime_bundle_produces_native_executable_holds"
                .to_string(),
            entry: "dag/test/claim/codex_package_delivery_wet_witness_test.dag".to_string(),
            ok: false,
            detail: "returned Bool(false) | HostDependencyAbsent{tool=npm,hint=apt install npm}"
                .to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: Some(HostDependencyRefusal {
                tool: "npm".to_string(),
                hint: "apt install npm".to_string(),
            }),
            resolve_realization: None,
        };
        assert_eq!(
            falsifier_failure_mode(&[npm_absent.detail.clone()]),
            "WitnessRed",
            "control: string classifier alone would misclassify"
        );
        let (mode, detail) =
            batch_failure_mode_and_detail(&batch_record_for_test(vec![npm_absent]));
        assert_eq!(mode, HOST_DEPENDENCY_ABSENT_MODE);
        assert!(detail.contains("HostDependencyAbsent{tool=npm"));
    }

    #[test]
    fn host_dependency_refusal_from_detail_parses_wire() {
        let parsed = host_dependency_refusal_from_detail(
            "returned Bool(false) | HostDependencyAbsent{tool=npm,hint=apt install npm}",
        )
        .expect("wire must parse");
        assert_eq!(parsed.tool, "npm");
        assert_eq!(parsed.hint, "apt install npm");
    }

    #[test]
    fn host_dependency_refusal_from_detail_anchors_last_wire_token() {
        let parsed = host_dependency_refusal_from_detail(
            "HostDependencyAbsent{tool=decoy,hint=x} | returned Bool(false) | HostDependencyAbsent{tool=npm,hint=apt install npm}",
        )
        .expect("last wire token must win");
        assert_eq!(parsed.tool, "npm");
        assert_eq!(parsed.hint, "apt install npm");
    }

    #[test]
    fn host_dependency_refusal_from_detail_refuses_brace_in_hint() {
        assert!(host_dependency_refusal_from_detail(
            "HostDependencyAbsent{tool=npm,hint=install {pkg} on runner}"
        )
        .is_none());
    }

    #[test]
    fn falsifier_failure_mode_classifies_three_arms() {
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=1 BudgetExceeded{wall_ms=900000,budget_ms=600000}".into()
            ]),
            "BudgetExceeded"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=3 fn=expensive_wet_witness detail=witness receipt wall budget exceeded: 707687ms elapsed > 600000ms whole-receipt budget".into()
            ]),
            "BudgetExceeded"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=3 fn=expensive_wet_witness detail=wet self-host receipt wall budget exceeded: 707687ms elapsed > 600000ms whole-receipt budget".into()
            ]),
            "BudgetExceeded"
        );
        // The walk's own infra fact is now read STRUCTURALLY, not grepped out of the line
        // the walk itself formatted. Both directions are asserted so the move from text to
        // value is proven rather than assumed: the same text alone no longer classifies as
        // Infra, and the observed fault classifies as Infra regardless of the text.
        let panic_text: Vec<String> = vec![
            "batch=2 infra=thread_panic".into(),
            "batch=1 fn=x detail=stale digest".into(),
        ];
        assert_eq!(
            falsifier_failure_mode(&panic_text),
            "WitnessRed",
            "the rendered line is for humans now — it must not carry the classification"
        );
        assert_eq!(
            falsifier_failure_mode_with_faults(
                &panic_text,
                &[InfraFault::ClaimThreadPanicked { batch_index: 2 }]
            ),
            "Infra"
        );
        assert_eq!(
            falsifier_failure_mode_with_faults(
                &["batch=1 fn=x detail=nothing resembling infra".into()],
                &[InfraFault::ClaimThreadPanicked { batch_index: 1 }]
            ),
            "Infra",
            "an observed fault settles the mode even when no text hints at it"
        );
        // And no fault means no Infra, whatever the prose says.
        assert_eq!(
            falsifier_failure_mode_with_faults(&panic_text, &[]),
            "WitnessRed"
        );
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=1 fn=design_register_lift_parity_holds detail=false".into()
            ]),
            "WitnessRed"
        );
        // BudgetExceeded wins when co-present with witness detail (honest wall kill).
        assert_eq!(
            falsifier_failure_mode(&[
                "batch=2 fn=wet detail=cargo refuse".into(),
                "batch=2 BudgetExceeded{wall_ms=601000,budget_ms=600000}".into()
            ]),
            "BudgetExceeded"
        );
    }

    // The lane-scoping wiring itself, both directions. The receipted defect (run
    // 30176416535) was that EVERY Hermetic batch drew the 5s per-PR fast-lane budget,
    // including the substrate long lane whose rows measure 12–135s. Discriminating: the
    // same execution_mode with different rosters must yield different ceilings.
    #[test]
    fn hermetic_eval_budget_is_scoped_by_lane_not_by_witness_kind() {
        let long_lane_entry =
            "src/v2/test/claim/long/edit_locus_source_provenance_witness_test.dag";
        let budgets = FalsifierSelfHostWetBudgets {
            substrate_long_lane_entry_paths: vec![long_lane_entry.to_string()],
            substrate_long_lane_eval_budget_ms: Some(180_000),
            ..Default::default()
        };
        let on_lane = vec![(
            long_lane_entry.to_string(),
            "lens_edit_locus_source_provenance_affected_set_wire_holds".to_string(),
        )];
        let off_lane = vec![(
            "src/v2/test/claim/self_host/some_fast_witness_test.dag".to_string(),
            "some_fast_holds".to_string(),
        )];

        // On the lane: its own dated ceiling, NOT the 5s per-PR budget.
        let picked = select_discovery_batch_budgets(
            ExecutionMode::Hermetic,
            &on_lane,
            Some(5_000),
            &budgets,
        );
        assert_eq!(picked.eval_budget_ms, Some(180_000));
        assert_eq!(picked.wet_wall_budget_ms, None);

        // Same execution_mode, different roster: the per-PR fast lane is the residual.
        let residual = select_discovery_batch_budgets(
            ExecutionMode::Hermetic,
            &off_lane,
            Some(5_000),
            &budgets,
        );
        assert_eq!(residual.eval_budget_ms, Some(5_000));

        // The whole point: witness kind alone must not decide the ceiling.
        assert_ne!(
            picked.eval_budget_ms, residual.eval_budget_ms,
            "two Hermetic batches on different lanes must not share one ceiling"
        );

        // A rostered lane with no declared ceiling narrows to the fast lane (a loud,
        // named red), never to an unbudgeted eval.
        let unbudgeted = FalsifierSelfHostWetBudgets {
            substrate_long_lane_eval_budget_ms: None,
            ..budgets.clone()
        };
        assert_eq!(
            select_discovery_batch_budgets(
                ExecutionMode::Hermetic,
                &on_lane,
                Some(5_000),
                &unbudgeted
            )
            .eval_budget_ms,
            Some(5_000)
        );

        // Wet lanes are untouched by the long-lane roster: no eval budget, and the
        // self-host wall/interp pair still arms off its own roster.
        let wet = FalsifierSelfHostWetBudgets {
            wall_budget_ms: Some(600_000),
            interp_eval_budget_ms: Some(120_000),
            roster_entry_paths: vec!["dag/test/claim/wet_receipt_test.dag".to_string()],
            ..budgets.clone()
        };
        let wet_pick = select_discovery_batch_budgets(
            ExecutionMode::Wet,
            &[(
                "dag/test/claim/wet_receipt_test.dag".to_string(),
                "receipt_holds".to_string(),
            )],
            Some(5_000),
            &wet,
        );
        assert_eq!(wet_pick.eval_budget_ms, None);
        assert_eq!(wet_pick.wet_wall_budget_ms, Some(600_000));
        assert_eq!(wet_pick.wet_interp_budget_ms, Some(120_000));
    }

    // The mode must survive the projection into the ci_failure_class vocabulary. Before
    // this, both non-Infra modes rendered as the reason-less `FloorFailed{class:Structural}`
    // — the receipted mis-map (run 30176416535: mode=BudgetExceeded, arm=…Structural).
    // Discriminating: a budget kill and a witness red must NOT print the same arm.
    #[test]
    fn ci_failure_class_arm_carries_the_structural_reason() {
        assert_eq!(ci_failure_class_arm("Infra"), "FloorFailed{class:Infra}");
        assert_eq!(
            ci_failure_class_arm("BudgetExceeded"),
            "FloorFailed{class:Structural{reason:BudgetExceeded}}"
        );
        assert_eq!(
            ci_failure_class_arm("WitnessRed"),
            "FloorFailed{class:Structural{reason:WitnessRed}}"
        );
        assert_ne!(
            ci_failure_class_arm("BudgetExceeded"),
            ci_failure_class_arm("WitnessRed"),
            "a budget kill and a witness red must be distinguishable in the emitted arm"
        );
        // End-to-end through the mode classifier: the batch-6 eval-budget detail shape.
        let detail: Vec<String> = vec![
            "batch=6 fn=discovery-corpus detail=7 of 10 discovery witness(es) failed: \
             lens_affected_set_provenance_producer_coverage_receipt_holds runtime error: \
             eval budget exceeded: 5001ms elapsed > 5000ms fast-lane budget"
                .into(),
        ];
        assert_eq!(
            ci_failure_class_arm(falsifier_failure_mode(&detail)),
            "FloorFailed{class:Structural{reason:BudgetExceeded}}"
        );
    }

    // D2 RED control: a receipt that cannot be written REDS the walk (returns false,
    // folded into any_failed → nonzero exit) — it never vanishes behind the fast exit.
    // The base path is a FILE, so create_dir_all refuses.
    #[test]
    fn unwritable_receipt_base_reds_not_vanishes() {
        let base =
            std::env::temp_dir().join(format!("claim-executor-receipt-red-{}", std::process::id()));
        let _ = fs::remove_file(&base);
        fs::write(&base, b"a file where the receipt dir should be").unwrap();
        assert!(!write_resolve_receipt_at(&base, &[], &[], None));
        assert!(!write_batch_wall_receipt_at(&base, &[]));
        let _ = fs::remove_file(&base);
    }

    /// TOTALITY, BOTH DIRECTIONS: a component the stop policy never reached appears in the
    /// receipt under its OWN planned identity, not a placeholder.
    ///
    /// This is the discriminating pair for the identity join. The plan below is the shape
    /// that matters: batch 0 is an ordinary claim that fails, batch 1 is the affected-set
    /// cold control (`predict_only`), and the stop policy means batch 1 never runs — so it
    /// exists only as a padded row. Before this fix that row was written with a literal
    /// "not reached" label and an "off" selection tag, which deleted the one property that
    /// IDENTIFIES the cold control (`gunbc.floor_component_receipt` role note keys on
    /// `predict_only`). `floor_affected_set_control_state` then answered `ControlAbsent`,
    /// conflating "the control was never enrolled" with "the control was enrolled and the
    /// line stopped before it ran" — different states with different remedies.
    ///
    /// The negative half is the point: asserting only that `predict_only` is present would
    /// also pass on a receipt that tagged EVERY padded row predict_only. So the batch-0
    /// side is asserted too — a real `off` component stays `off` — and the vanished
    /// placeholder literal is asserted absent.
    #[test]
    fn unreached_components_carry_planned_identity_not_placeholder() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let base = std::env::temp_dir().join(format!(
            "claim-executor-totality-{}-{}",
            std::process::id(),
            "unreached"
        ));
        let _ = fs::remove_dir_all(&base);

        // The PLAN: two components. Batch 1 is the cold control and is never reached.
        let batches: Vec<Vec<Runnable>> = vec![
            vec![Runnable::SingleClaim {
                entry: "dag/test/claim/some_gate_test.dag".to_string(),
                function: "some_gate_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }],
            vec![Runnable::DiscoveryBatch {
                source_roots: source_roots.clone(),
                scan_dirs: vec!["dag/test/claim".to_string()],
                explicit_entries: Vec::new(),
                native_bundle_entries: Vec::new(),
                exclude_substrings: Vec::new(),
                discovery_scope_dirs: Vec::new(),
                execution_mode: ExecutionMode::Hermetic,
                spawns_host_compiler: false,
            }],
        ];

        // The RUN: only batch 0 produced a record — the line stopped there.
        let batch_records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 1_000_000_000,
            clamp_ms: None,
            unit_count: 1,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 1 },
            results: Vec::new(),
            label: batch_heartbeat_label(&batches[0]),
            is_wet: false,
        }];

        assert!(write_floor_component_receipt_at(
            &base,
            &source_roots,
            &batch_records,
            &batches,
            UnreachedCause::StopPolicy,
        ));
        let body = fs::read_to_string(base.join("floor-component-receipt.json")).unwrap();

        // The unreached cold control is present AS the cold control.
        assert!(
            body.contains("\"selection\": \"predict_only\""),
            "the unreached cold control must keep the predict_only tag that identifies it: {body}"
        );
        assert!(
            body.contains("some_gate_holds"),
            "the reached component keeps its own label: {body}"
        );
        // Its outcome is honestly Skipped with the stop-policy cause — not fabricated Done.
        assert!(
            body.contains("\"outcome\": \"skipped\""),
            "an unreached component concludes Skipped: {body}"
        );
        // The placeholder identity is gone.
        assert!(
            !body.contains("\"label\": \"not reached\""),
            "the placeholder label must not survive: {body}"
        );
        // NEGATIVE HALF: a genuinely `off` component is not relabelled predict_only.
        assert!(
            body.contains("\"selection\": \"off\""),
            "batch 0 is an ordinary off component and must stay off: {body}"
        );

        let _ = fs::remove_dir_all(&base);
    }

    // Hard discovery failure must refuse the batch clamp — never mis-render as
    // FLOOR-BATCH-OVER-BUDGET via the legacy zero→one unit mapping.
    #[test]
    fn hard_discovery_error_refuses_batch_clamp_not_over_budget() {
        let discovery_fail = ClaimResult {
            function: "discovery-corpus".to_string(),
            entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
            ok: false,
            detail: "discovery corpus failed: resolve refused".to_string(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: runtime_unit_count_unavailable("resolve refused"),
            witness_row_costs: Vec::new(),
            budget_refusal: None,
            host_dependency_refusal: None,
            expectation_refusal: None,
            resolve_realization: None,
        };
        let units = aggregate_batch_runtime_units(&[discovery_fail]);
        assert!(
            matches!(units, FloorRuntimeUnitCount::Unavailable { .. }),
            "hard corpus error must mark units unavailable, got {units:?}"
        );
        // Positive control: a single gate row still contributes one observed unit.
        let gate = ClaimResult {
            function: "some_gate_holds".to_string(),
            entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            budget_refusal: None,
            host_dependency_refusal: None,
            expectation_refusal: None,
            resolve_realization: None,
        };
        assert_eq!(
            aggregate_batch_runtime_units(&[gate]),
            FloorRuntimeUnitCount::Observed { units: 1 }
        );
        // RED control: the old zero→one mapping would fabricate OverBudget on a slow wall
        // against overhead-only clamp when the corpus actually failed.
        let overhead_ms = 60_000u128;
        let rate_ms = 1_000u128;
        let wall_ms = 120_000u128;
        if let FloorRuntimeUnitCount::Observed { units: 1 } = units {
            let clamp = overhead_ms + 1 * rate_ms;
            assert!(
                wall_ms > clamp,
                "control: units=1 would have breached overhead-only clamp"
            );
        }
    }

    #[test]
    fn hard_discovery_batch_wall_receipt_is_clamp_refused_not_over_budget() {
        let base = std::env::temp_dir().join(format!(
            "claim-executor-clamp-refused-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        let records = vec![BatchRecord {
            batch_index: 2,
            wall_nanos: 600_000_000_000,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: runtime_unit_count_unavailable("discovery corpus failed: test"),
            results: Vec::new(),
            label: "discovery-corpus".to_string(),
            is_wet: false,
        }];
        assert!(write_batch_wall_receipt_at(&base, &records));
        let body = fs::read_to_string(base.join("floor-batch-wall-receipt.txt")).unwrap();
        assert!(body.contains("batch_3_units=unavailable"));
        assert!(body.contains("batch_3_verdict=ClampRefused"));
        assert!(!body.contains("OverBudget"));
        assert!(body.contains("over_budget_batches=0"));

        let _ = fs::remove_dir_all(&base);
    }

    /// A CHECKPOINT is not a CONCLUSION, and the receipt must say which one it is.
    ///
    /// Both causes collapse to the same `skipped` outcome by design, so asserting the
    /// outcome proves nothing about this distinction — the whole claim lives in the
    /// failure_mode, and it is asserted in BOTH directions on the same plan and the same
    /// records. Without the negative halves this test would pass on a writer that emitted
    /// one constant mode for every unreached row, which is precisely the pre-existing
    /// behaviour being corrected.
    ///
    /// Why it matters: a 170-minute SIGKILL leaves the last checkpoint on disk as the
    /// alert's only evidence. If its tail read `not_reached`, the alert would report a
    /// killed run as a plan that deliberately skipped those components under the stop
    /// policy — a wrong causal story about the exact incident the checkpoint exists to
    /// explain, and one that routes to the wrong owner.
    #[test]
    fn checkpoint_tail_is_run_incomplete_and_final_tail_is_not_reached() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let base = std::env::temp_dir().join(format!(
            "claim-executor-checkpoint-{}-{}",
            std::process::id(),
            "cause"
        ));
        let _ = fs::remove_dir_all(&base);

        let batches: Vec<Vec<Runnable>> = vec![
            vec![Runnable::SingleClaim {
                entry: "dag/test/claim/some_gate_test.dag".to_string(),
                function: "some_gate_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }],
            vec![Runnable::SingleClaim {
                entry: "dag/test/claim/other_gate_test.dag".to_string(),
                function: "other_gate_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }],
        ];
        let batch_records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 1_000_000_000,
            clamp_ms: None,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 1 },
            unit_count: 1,
            results: Vec::new(),
            label: batch_heartbeat_label(&batches[0]),
            is_wet: false,
        }];

        // MID-RUN: batch 1 has not concluded, and its fate is unknown.
        assert!(write_floor_component_receipt_at(
            &base,
            &source_roots,
            &batch_records,
            &batches,
            UnreachedCause::RunIncomplete,
        ));
        let checkpoint = fs::read_to_string(base.join("floor-component-receipt.json")).unwrap();
        assert!(
            checkpoint.contains("\"failure_mode\": \"run_incomplete\""),
            "a checkpoint's unreached tail carries run_incomplete: {checkpoint}"
        );
        assert!(
            checkpoint.contains("\"outcome\": \"pending\""),
            "a checkpoint's unreached tail is pending, not a fabricated skip: {checkpoint}"
        );
        assert!(
            !checkpoint.contains("\"outcome\": \"skipped\""),
            "a checkpoint must not manufacture skipped verdicts for unreached batches: {checkpoint}"
        );
        assert!(
            checkpoint.contains("\"disposition\": \"incomplete\""),
            "a checkpoint names the run-level incomplete fact: {checkpoint}"
        );

        // CONCLUDED: the same records, but the plan is over — the tail was genuinely skipped.
        assert!(write_floor_component_receipt_at(
            &base,
            &source_roots,
            &batch_records,
            &batches,
            UnreachedCause::StopPolicy,
        ));
        let concluded = fs::read_to_string(base.join("floor-component-receipt.json")).unwrap();
        assert!(
            concluded.contains("\"failure_mode\": \"not_reached\""),
            "a concluded walk's unreached tail carries not_reached: {concluded}"
        );
        assert!(
            !concluded.contains("\"failure_mode\": \"run_incomplete\""),
            "a concluded walk must not report itself as still in progress: {concluded}"
        );

        // The rename left no temp file behind for the artifact upload to trip over.
        let strays: Vec<_> = fs::read_dir(&base)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".tmp."))
            .collect();
        assert!(
            strays.is_empty(),
            "temp receipts must not survive: {strays:?}"
        );

        let _ = fs::remove_dir_all(&base);
    }

    // D5 receipt rows, both directions: an over-budget batch records OverBudget and is
    // counted; a within-budget batch records WithinBudget; a budget-less walk records
    // Unbudgeted (falsifier/regen plans).
    #[test]
    fn batch_wall_receipt_verdicts_both_directions() {
        let base =
            std::env::temp_dir().join(format!("claim-executor-batch-wall-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        // Clamped: batch 0 wall 5s > clamp 2s (OverBudget); batch 1 wall 1s < clamp 2s (WithinBudget).
        let clamped_records = vec![
            BatchRecord {
                batch_index: 0,
                wall_nanos: 5_000_000_000, // 5s
                clamp_ms: Some(2_000),     // 2s
                unit_count: 3,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 3 },
                results: Vec::new(),
                label: "batch-0".to_string(),
                is_wet: false,
            },
            BatchRecord {
                batch_index: 1,
                wall_nanos: 1_000_000_000, // 1s
                clamp_ms: Some(2_000),     // 2s
                unit_count: 0,
                runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
                results: Vec::new(),
                label: "batch-1".to_string(),
                is_wet: false,
            },
        ];
        assert!(write_batch_wall_receipt_at(&base, &clamped_records));
        let body = fs::read_to_string(base.join("floor-batch-wall-receipt.txt")).unwrap();
        assert!(body.contains("batch_1_wall_ms=5000"));
        assert!(body.contains("batch_1_units=3"));
        assert!(body.contains("batch_1_clamp_ms=2000"));
        assert!(body.contains("batch_1_verdict=OverBudget"));
        assert!(body.contains("batch_2_verdict=WithinBudget"));
        assert!(body.contains("over_budget_batches=1"));
        // Clamp-less (falsifier/regen plans): records carry clamp_ms None -> Unbudgeted.
        let unbudgeted_records = vec![BatchRecord {
            batch_index: 0,
            wall_nanos: 5_000_000_000,
            clamp_ms: None,
            unit_count: 0,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 0 },
            results: Vec::new(),
            label: "batch-0".to_string(),
            is_wet: false,
        }];
        assert!(write_batch_wall_receipt_at(&base, &unbudgeted_records));
        let body = fs::read_to_string(base.join("floor-batch-wall-receipt.txt")).unwrap();
        assert!(body.contains("batch_1_verdict=Unbudgeted"));
        assert!(body.contains("over_budget_batches=0"));
        let _ = fs::remove_dir_all(&base);
    }

    // One entry, one resolve (lane clause (c)): a negligible-profile SharedClaims
    // group whose (entry, mode) some batch resolves on the memo path is itself
    // routed to the memo lane, so the entry's closure resolves once on the main
    // thread (thread-local `process_shared_index`) instead of cold on a spawned
    // thread — the #7088 cheap-gate batch-0 shape (resolve receipt 4).
    #[test]
    fn same_entry_shared_claims_promote_to_memo_lane_across_batches() {
        let cheap = |f: &str| Runnable::SingleClaim {
            entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
            function: f.to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        };
        let heavy = Runnable::SingleClaim {
            entry: "dag/tools/floor_effect_gate_witness.dag".to_string(),
            function: "dag_compile_clean_gate_passes".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: true,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        };
        let batches = vec![
            vec![
                cheap("extdeps_external_authority_gate_passes"),
                cheap("generated_artifact_drift_gate_passes"),
            ],
            vec![heavy],
        ];
        let keys = memo_path_entry_keys(&batches);
        assert!(keys.contains(&(
            "dag/tools/floor_effect_gate_witness.dag".to_string(),
            ExecutionMode::Wet
        )));
        let empty_memo = std::collections::HashMap::new();
        let cheap_units = group_batch_units(&batches[0]);
        assert_eq!(
            cheap_units.len(),
            1,
            "same-entry claims coalesce into one resolve-group"
        );
        assert_eq!(
            batch_unit_lane(&cheap_units[0], &empty_memo, &keys),
            UnitLane::Memo,
            "batch-0 group must ride the memo lane when a later batch resolves the same entry there"
        );
        // RED control — the promotion is exactly the heavy same-entry declaration:
        // without it the group spawns (the pre-fix behavior).
        let no_heavy = memo_path_entry_keys(&batches[..1]);
        assert!(no_heavy.is_empty());
        assert_eq!(
            batch_unit_lane(&cheap_units[0], &empty_memo, &no_heavy),
            UnitLane::Spawned
        );
    }

    #[test]
    fn on_success_spawned_claims_move_to_warm_main_thread_only() {
        let runnable = Runnable::SingleClaim {
            entry: "dag/tools/merge_admission_walk.dag".to_string(),
            function: "stamp_tested_floor".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        };
        let units = group_batch_units(&[runnable]);
        let memo = std::collections::HashMap::new();
        let memo_entries = std::collections::HashSet::new();
        assert_eq!(
            batch_unit_lane(&units[0], &memo, &memo_entries),
            UnitLane::Spawned,
            "the shared lane authority remains unchanged"
        );
        assert_eq!(
            population_unit_lane(
                StagePopulation::OrdinaryBatch,
                &units[0],
                &memo,
                &memo_entries,
            ),
            UnitLane::Spawned,
            "ordinary claim placement must remain unchanged"
        );
        assert_eq!(
            population_unit_lane(
                StagePopulation::OnSuccessStage,
                &units[0],
                &memo,
                &memo_entries,
            ),
            UnitLane::MainThread,
            "green-only claim must consume the warm main-thread index"
        );
    }

    // Mode is part of the promotion key: a Hermetic group must not share a Wet
    // entry's memo context (the cached InterpContext carries its effect envelope).
    #[test]
    fn memo_lane_promotion_is_mode_keyed() {
        let batches = vec![vec![Runnable::SingleClaim {
            entry: "dag/x.dag".to_string(),
            function: "f".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: true,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Wet,
            },
        }]];
        let keys = memo_path_entry_keys(&batches);
        let hermetic_units = group_batch_units(&[Runnable::SingleClaim {
            entry: "dag/x.dag".to_string(),
            function: "g".to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Hermetic,
            },
        }]);
        let empty_memo = std::collections::HashMap::new();
        assert_eq!(
            batch_unit_lane(&hermetic_units[0], &empty_memo, &keys),
            UnitLane::Spawned,
            "a Wet memo entry must not promote a Hermetic group on the same entry"
        );
    }

    #[test]
    fn floor_arm_time_budget_refusal_plan_functions_match_dag_seed_roster() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("src/v2/workflow/ci_floor_plan.dag")
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph(&roots, &entry).expect("resolve ci_floor_plan");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let value = run_value(
            &ctx,
            "gunbc_floor_arm_time_budget_refusal_plan_function_roster",
        )
        .expect("evaluate materialized plan roster");
        let dag_roster =
            str_list_from_value(&value, &ctx).expect("plan roster must be List<String>");
        assert_eq!(
            dag_roster.as_slice(),
            FLOOR_ARM_TIME_BUDGET_REFUSAL_PLAN_FUNCTIONS,
            "claim_executor seed const must match ci_floor_plan materialized roster \
             (dissolve-on: v2 emit of stage0 host constants)"
        );
    }

    // The seed→.dag render boundary by execution: `render_phase_concluded_line`
    // resolves `gunbc.observation_seed_render` and projects a floor phase mark through
    // the single-authority renderer, so the seed speaks the observation vocabulary
    // rather than a raw `[t+…]` byte string. Discriminating RED: a helper that fell
    // back to the raw marker, dropped the phase, or forked the duration format fails
    // one of the three asserts below (human units, completed glyph, no old marker).
    #[test]
    fn phase_mark_renders_through_the_observation_render_authority() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // Emoji tier (the CI target): 48s concludes as `✅ naming-hygiene walk done in
        // 48 seconds`. overhead_ms is the placement basis only (the 300s clamp overhead).
        let line =
            render_phase_concluded_line(&roots, "naming-hygiene walk", 48_000, 300_000, true)
                .expect("observation_seed_render must resolve and render a phase line");
        assert!(
            line.contains("naming-hygiene walk") && line.contains("done in 48 seconds"),
            "phase line must name the phase in human units: {line:?}"
        );
        assert!(
            line.starts_with('✅'),
            "a Done outcome concludes with the completed glyph at the Emoji tier: {line:?}"
        );
        assert!(
            !line.contains("[t+"),
            "the projection must not carry the deleted raw phase marker: {line:?}"
        );
    }

    fn run_seed_phase_begin_line(
        source_roots: &[String],
        phase: &str,
        overhead_ms: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "phase_begin_line",
            &[
                (Some("phase".to_string()), str_value(phase.to_string())),
                (
                    Some("overhead_ms".to_string()),
                    Value::Int(overhead_ms as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    // Gantt flip byte-oracle: compile-path mirrors of phase_begin_line /
    // phase_concluded_line must stay byte-equal to the .dag seed (justified
    // divergence — interpreter render from inside compile would recurse).
    #[test]
    fn gantt_phase_mirrors_match_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let begin_oracle = run_seed_phase_begin_line(&roots, "compile.frontend", 300_000, true)
            .expect("phase_begin_line must resolve and render");
        let begin_mirror =
            v1_compiler::v1_rt::render_phase_begin_line_mirror("compile.frontend", true);
        assert_eq!(
            begin_oracle, begin_mirror,
            "begin mirror must be byte-equal to seed oracle"
        );
        assert!(
            begin_oracle.starts_with('🔄') && begin_oracle.contains("started compile.frontend"),
            "begin line shape: {begin_oracle:?}"
        );

        let done_oracle =
            render_phase_concluded_line(&roots, "compile.frontend", 12_000, 300_000, true)
                .expect("phase_concluded_line must resolve and render");
        let done_mirror = v1_compiler::v1_rt::render_phase_concluded_line_mirror(
            "compile.frontend",
            12_000,
            true,
        );
        assert_eq!(
            done_oracle, done_mirror,
            "concluded mirror must be byte-equal to seed oracle"
        );
        assert!(
            done_oracle.contains("compile.frontend done in 12 seconds"),
            "concluded line shape: {done_oracle:?}"
        );
    }

    fn run_seed_psi_hold_line(
        source_roots: &[String],
        avg10_bp: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_psi_hold_line",
            &[
                (Some("avg10_bp".to_string()), Value::Int(avg10_bp as i64)),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn run_seed_high_water_hold_line(
        source_roots: &[String],
        current_bytes: u64,
        high_water_bytes: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_high_water_hold_line",
            &[
                (
                    Some("current_bytes".to_string()),
                    Value::Int(current_bytes as i64),
                ),
                (
                    Some("high_water_bytes".to_string()),
                    Value::Int(high_water_bytes as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    // Governor flip byte-oracle: HoldReason → ci_hold_cause_text mirrors must stay
    // byte-equal to seed_psi_hold_line / seed_high_water_hold_line.
    #[test]
    fn governor_hold_mirrors_match_seed_oracle() {
        use v1_compiler::memory_governor::{render_governor_hold_line_mirror, HoldReason};
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // 37.5% → 3750 basis points
        let psi_oracle = run_seed_psi_hold_line(&roots, 3750, true)
            .expect("seed_psi_hold_line must resolve and render");
        let psi_mirror =
            render_governor_hold_line_mirror(&HoldReason::PsiPressure { avg10: 37.5 }, true);
        assert_eq!(
            psi_oracle, psi_mirror,
            "psi hold mirror must be byte-equal to seed oracle"
        );
        assert!(
            psi_oracle.starts_with('⏳') && psi_oracle.contains("blocked on memory reclaim"),
            "psi hold shape: {psi_oracle:?}"
        );

        let hw_oracle = run_seed_high_water_hold_line(&roots, 8_589_934_592, 10_737_418_240, true)
            .expect("seed_high_water_hold_line must resolve and render");
        let hw_mirror = render_governor_hold_line_mirror(
            &HoldReason::CurrentHighWater {
                current: 8_589_934_592,
                high_water: 10_737_418_240,
            },
            true,
        );
        assert_eq!(
            hw_oracle, hw_mirror,
            "high-water hold mirror must be byte-equal to seed oracle"
        );
        assert!(
            hw_oracle.contains("blocked on the memory high-water line"),
            "high-water hold shape: {hw_oracle:?}"
        );
    }

    fn run_seed_typecheck_concluded_line(
        source_roots: &[String],
        module_path: &str,
        elapsed_ms: u64,
        overhead_ms: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "typecheck_concluded_line",
            &[
                (
                    Some("module_path".to_string()),
                    str_value(module_path.to_string()),
                ),
                (
                    Some("elapsed_ms".to_string()),
                    Value::Int(elapsed_ms as i64),
                ),
                (
                    Some("overhead_ms".to_string()),
                    Value::Int(overhead_ms as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    #[test]
    fn typecheck_attribution_mirrors_match_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let oracle = run_seed_typecheck_concluded_line(
            &roots,
            "v2.compiler.normalized_tree",
            606_984,
            300_000,
            true,
        )
        .expect("typecheck_concluded_line must resolve and render");
        let mirror = v1_compiler::cli_run::render_typecheck_concluded_line_mirror(
            "v2.compiler.normalized_tree",
            606_984,
            true,
        );
        assert_eq!(
            oracle, mirror,
            "typecheck concluded mirror must be byte-equal to seed oracle"
        );
        assert!(
            oracle.starts_with('✅')
                && oracle.contains("typecheck v2.compiler.normalized_tree done in 10 minutes"),
            "typecheck concluded shape: {oracle:?}"
        );
        let begin_mirror = v1_compiler::cli_run::render_typecheck_begin_line_mirror(
            "v2.compiler.normalized_tree",
            true,
        );
        assert_eq!(
            begin_mirror,
            "🔄 started typecheck v2.compiler.normalized_tree"
        );
    }

    fn run_seed_witness_claim_result_text(
        source_roots: &[String],
        subject: &str,
        function: &str,
        variant_name: &str,
        cause: Option<&str>,
        wall_nanos: u128,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_ci_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let wall = run_in_context_with_args(
            &ctx,
            "nanosecond",
            &[(Some("count".to_string()), Value::Int(wall_nanos as i64))],
            false,
        )
        .ok()?;
        let out = run_in_context_with_args(
            &ctx,
            "ci_witness_claim_result_text",
            &[
                (Some("subject".to_string()), str_value(subject.to_string())),
                (
                    Some("function".to_string()),
                    str_value(function.to_string()),
                ),
                (
                    Some("verdict".to_string()),
                    Value::Variant {
                        type_name: ctx.sym("CiWitnessVerdict"),
                        variant_name: ctx.sym(variant_name),
                        fields: std::rc::Rc::new(match cause {
                            None => Vec::new(),
                            Some(cause) => vec![(
                                ctx.sym("cause"),
                                Value::Variant {
                                    type_name: ctx.sym("CiWitnessRuntimeCause"),
                                    variant_name: ctx.sym(cause),
                                    fields: std::rc::Rc::new(Vec::new()),
                                },
                            )],
                        }),
                    },
                ),
                (Some("wall".to_string()), wall),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    // ENROLLMENT MAY ONLY HOLD A CLAIM THAT WAS DECIDED. Found on gunbc#8642 by the side
    // thread: the first version of `from_outcome` matched `_ if enrolled => KnownRed` above
    // every failing arm, so an enrolled row that was INTERRUPTED at a budget, or whose host
    // tool could not be resolved, printed KNOWN-RED — owned semantic debt, for two states that
    // carry no semantic verdict at all.
    //
    // RED: restoring the blanket `_ if enrolled` arm turns the two interruption cases into
    // KnownRed and fails this test. The `Fail` case is the positive control — without it, a
    // `from_outcome` that never returned KnownRed at all would also pass.
    #[test]
    fn enrollment_holds_only_semantic_failures_not_undecided_claims() {
        use v1_compiler::cli_run::{BudgetKind, CiWitnessVerdict, ClaimOutcome};

        let interrupted = ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 5_000,
            budget_ms: 5_000,
            kind: BudgetKind::Cpu,
        };
        let missing_tool = ClaimOutcome::HostToolUnresolved {
            name: "git".to_string(),
            probed: vec!["/usr/bin/git".to_string()],
        };

        assert_eq!(
            CiWitnessVerdict::from_outcome(&ClaimOutcome::Fail, true),
            CiWitnessVerdict::KnownRed,
            "positive control: an enrolled SEMANTIC failure is the one thing enrollment holds"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&interrupted, true),
            CiWitnessVerdict::BudgetRefused,
            "an interruption is a lower bound on cost, not a verdict enrollment can hold"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&missing_tool, true),
            CiWitnessVerdict::HostToolUnresolved,
            "an unresolved host tool is an infra gap, not owned semantic debt"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&interrupted, false),
            CiWitnessVerdict::from_outcome(&interrupted, true),
            "enrollment must not change the token for an undecided claim, either direction"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&ClaimOutcome::Pass, true),
            CiWitnessVerdict::Passed,
            "an enrolled row that passes still prints PASSED; the roster staleness is the \
             ledger's to report, not a token the console invents"
        );
    }

    #[test]
    fn render_witness_claim_result_text_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let oracle = run_seed_witness_claim_result_text(
            &roots,
            "test.claim.observation_ci_render_witness_test",
            "w_witness_claim_line_from_module_path_holds",
            "WitnessPassed",
            None,
            230_000_000,
        )
        .expect("ci_witness_claim_result_text must resolve and render");
        let mirror = v1_compiler::cli_run::render_witness_claim_result_text_mirror(
            "test.claim.observation_ci_render_witness_test",
            "w_witness_claim_line_from_module_path_holds",
            230_000_000,
            v1_compiler::cli_run::CiWitnessVerdict::Passed,
        );
        assert_eq!(
            oracle, mirror,
            "witness claim-result mirror must be byte-equal to the .dag oracle"
        );
        assert!(
            mirror.contains("PASSED in 230ms") && !mirror.contains("PASS in "),
            "drift control: legacy PASS token must not return: {mirror:?}"
        );
        assert!(
            mirror.starts_with("//test/claim/observation_ci_render_witness_test:"),
            "drift control: Bazel label prefix required: {mirror:?}"
        );

        let under_boundary = run_seed_witness_claim_result_text(
            &roots,
            "test.claim.foo",
            "w_bar",
            "WitnessPassed",
            None,
            89_000_000_000,
        )
        .expect("89s boundary oracle");
        let at_boundary = run_seed_witness_claim_result_text(
            &roots,
            "test.claim.foo",
            "w_bar",
            "WitnessPassed",
            None,
            90_000_000_000,
        )
        .expect("90s boundary oracle");
        assert_eq!(
            v1_compiler::cli_run::render_witness_claim_result_text_mirror(
                "test.claim.foo",
                "w_bar",
                89_000_000_000,
                v1_compiler::cli_run::CiWitnessVerdict::Passed,
            ),
            under_boundary,
            "89s minute-switch boundary must match oracle"
        );
        assert_eq!(
            v1_compiler::cli_run::render_witness_claim_result_text_mirror(
                "test.claim.foo",
                "w_bar",
                90_000_000_000,
                v1_compiler::cli_run::CiWitnessVerdict::Passed,
            ),
            at_boundary,
            "90s minute-switch boundary must match oracle"
        );
        assert!(
            under_boundary.contains("89 seconds") && at_boundary.contains("1 minutes"),
            "minute-switch drift control: {under_boundary:?} vs {at_boundary:?}"
        );

        // Every arm of the verdict coproduct is rendered by the mirror byte-equal to the
        // `.dag` oracle, and every arm produces a DISTINCT token. The bool this parameter
        // used to be could only witness two of these eight; five typed outcomes were
        // flattened into FAILED at the call site before the renderer was ever reached.
        // RED: collapsing any two arms to one token fails the distinctness assert;
        // drifting one arm's spelling in either representation fails byte-equality.
        let arms: [(&str, v1_compiler::cli_run::CiWitnessVerdict); 8] = [
            (
                "WitnessPassed",
                v1_compiler::cli_run::CiWitnessVerdict::Passed,
            ),
            (
                "WitnessFailed",
                v1_compiler::cli_run::CiWitnessVerdict::Failed,
            ),
            (
                "WitnessNotBool",
                v1_compiler::cli_run::CiWitnessVerdict::NotBool,
            ),
            (
                "WitnessRuntimeError",
                v1_compiler::cli_run::CiWitnessVerdict::RuntimeError(
                    v1_compiler::cli_run::WitnessRuntimeCause::NoSuchVariable,
                ),
            ),
            (
                "WitnessBudgetRefused",
                v1_compiler::cli_run::CiWitnessVerdict::BudgetRefused,
            ),
            (
                "WitnessHostToolUnresolved",
                v1_compiler::cli_run::CiWitnessVerdict::HostToolUnresolved,
            ),
            (
                "WitnessKnownRed",
                v1_compiler::cli_run::CiWitnessVerdict::KnownRed,
            ),
            (
                "WitnessRouteGap",
                v1_compiler::cli_run::CiWitnessVerdict::RouteGap,
            ),
        ];
        let arm_count = arms.len();
        let mut rendered: Vec<String> = Vec::new();
        for (variant, verdict) in arms {
            // The cause rides inside the arm on BOTH sides, so the oracle call carries it too:
            // a fieldless `WitnessRuntimeError` is not constructible in the `.dag` type any more.
            let cause = match verdict {
                v1_compiler::cli_run::CiWitnessVerdict::RuntimeError(_) => {
                    Some("WitnessCauseNoSuchVariable")
                }
                _ => None,
            };
            let oracle_arm = run_seed_witness_claim_result_text(
                &roots,
                "test.claim.foo",
                "w_bar",
                variant,
                cause,
                1_000_000,
            )
            .unwrap_or_else(|| panic!("oracle must render {variant}"));
            let mirror_arm = v1_compiler::cli_run::render_witness_claim_result_text_mirror(
                "test.claim.foo",
                "w_bar",
                1_000_000,
                verdict,
            );
            assert_eq!(
                oracle_arm, mirror_arm,
                "{variant}: mirror must be byte-equal to the .dag oracle"
            );
            rendered.push(mirror_arm);
        }
        let mut distinct = rendered.clone();
        distinct.sort();
        distinct.dedup();
        // WAS THE LITERAL `7` AGAINST AN EIGHT-ARM TABLE, and it did not start failing with this
        // change -- it was already red on `main`. The Rust suite has not run in CI since
        // 2026-07-11 (operator ruling, recorded at `gunbc.commit_workflow`
        // `commit_gate_rust_suite_removed_disposition`), so a stale literal here is invisible
        // until someone runs the bin's tests by hand. Executed receipt: this assertion reports
        // `left: 8 right: 7`, and the eight lines it prints are pairwise distinct with or
        // without the cause field, which the `.dag` side asserts independently in
        // `w_every_verdict_arm_renders_a_distinct_token`.
        //
        // Pinned to `arm_count` rather than to a new literal `8`, so adding a ninth arm cannot
        // reproduce this: the fixture is the table, and a count copied out of it by hand is the
        // second representation that went stale in the first place.
        assert_eq!(
            distinct.len(),
            arm_count,
            "every verdict arm must render a distinct line: {rendered:?}"
        );
        assert!(
            !rendered[6].contains("FAILED"),
            "an enrolled known-red must not be spelled as FAILED: {:?}",
            rendered[6]
        );
    }

    fn run_seed_peak_rss_line(
        source_roots: &[String],
        label: &str,
        rss_bytes: u64,
        rss_available: bool,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_peak_rss_line",
            &[
                (Some("label".to_string()), str_value(label.to_string())),
                (Some("rss_bytes".to_string()), Value::Int(rss_bytes as i64)),
                (
                    Some("rss_available".to_string()),
                    Value::Bool(rss_available),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    #[test]
    fn peak_rss_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // 1 GiB exact → "1.0 GiB"
        let oracle = run_seed_peak_rss_line(&roots, "floor peak RSS", 1_073_741_824, true, true)
            .expect("seed_peak_rss_line must resolve and render");
        let mirror = v1_compiler::cli_run::render_peak_rss_line_mirror(
            "floor peak RSS",
            Some(1_073_741_824),
            true,
        );
        assert_eq!(oracle, mirror, "peak RSS mirror must be byte-equal to seed");
        assert_eq!(oracle, "🕐 floor peak RSS — 1.0 GiB");

        // 1536 MiB → "1.5 GiB"
        let oracle15 = run_seed_peak_rss_line(&roots, "cgroup peak", 1_610_612_736, true, false)
            .expect("seed");
        let mirror15 = v1_compiler::cli_run::render_peak_rss_line_mirror(
            "cgroup peak",
            Some(1_610_612_736),
            false,
        );
        assert_eq!(oracle15, mirror15);
        assert_eq!(oracle15, "◷ cgroup peak — 1.5 GiB");

        let unread = run_seed_peak_rss_line(&roots, "floor peak RSS", 0, false, true)
            .expect("unreadable seed");
        let unread_mirror =
            v1_compiler::cli_run::render_peak_rss_line_mirror("floor peak RSS", None, true);
        assert_eq!(unread, unread_mirror);
        assert!(unread.contains("unreadable (no /proc/self/status)"));
    }

    fn run_seed_shell_effect_failed_line(
        source_roots: &[String],
        intent: &str,
        argv_collapsed: &str,
        exit_code: u64,
        elapsed_ms: u64,
        overhead_ms: u64,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "shell_effect_failed_line",
            &[
                (Some("intent".to_string()), str_value(intent.to_string())),
                (
                    Some("argv_collapsed".to_string()),
                    str_value(argv_collapsed.to_string()),
                ),
                (Some("exit_code".to_string()), Value::Int(exit_code as i64)),
                (
                    Some("elapsed_ms".to_string()),
                    Value::Int(elapsed_ms as i64),
                ),
                (
                    Some("overhead_ms".to_string()),
                    Value::Int(overhead_ms as i64),
                ),
                (Some("emoji".to_string()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    #[test]
    fn shell_effect_failed_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let oracle = run_seed_shell_effect_failed_line(
            &roots,
            "shell.Exec.Run",
            "echo hi",
            1,
            2000,
            300_000,
            true,
        )
        .expect("shell_effect_failed_line must resolve and render");
        let mirror = v1_compiler::v1_interpreter::render_shell_effect_failed_line_mirror(
            "shell.Exec.Run",
            "echo hi",
            1,
            2000,
            true,
        );
        assert_eq!(
            oracle, mirror,
            "shell Failed mirror must be byte-equal to seed oracle"
        );
        assert_eq!(
            oracle,
            "❌ shell.Exec.Run failed: $ echo hi (exit=1) in 2 seconds"
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn run_seed_heartbeat_line(
        source_roots: &[String],
        elapsed_ms: u64,
        batch_label: &str,
        entry_index: u64,
        entry_total: u64,
        rss: Option<u64>,
        swap: Option<u64>,
        pressure: Option<u64>,
        emoji: bool,
    ) -> Option<String> {
        let entry = source_roots
            .iter()
            .map(|r| Path::new(r).join("gunbc/observation_seed_render.dag"))
            .find(|p| p.exists())?
            .to_string_lossy()
            .into_owned();
        let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry).ok()?;
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
        let out = run_in_context_with_args(
            &ctx,
            "seed_heartbeat_line",
            &[
                (Some("elapsed_ms".into()), Value::Int(elapsed_ms as i64)),
                (Some("batch_index".into()), Value::Int(0)),
                (
                    Some("batch_label".into()),
                    str_value(batch_label.to_string()),
                ),
                (Some("entry_index".into()), Value::Int(entry_index as i64)),
                (Some("entry_total".into()), Value::Int(entry_total as i64)),
                (
                    Some("rss_bytes".into()),
                    Value::Int(rss.unwrap_or(0) as i64),
                ),
                (Some("rss_available".into()), Value::Bool(rss.is_some())),
                (
                    Some("swap_bytes".into()),
                    Value::Int(swap.unwrap_or(0) as i64),
                ),
                (Some("swap_available".into()), Value::Bool(swap.is_some())),
                (
                    Some("pressure_bp".into()),
                    Value::Int(pressure.unwrap_or(0) as i64),
                ),
                (
                    Some("pressure_available".into()),
                    Value::Bool(pressure.is_some()),
                ),
                (Some("emoji".into()), Value::Bool(emoji)),
            ],
            false,
        )
        .ok()?;
        match out {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    // The floor-memory heartbeat's seed→.dag boundary, proven by execution:
    // seed_heartbeat_line takes the primitives the heartbeat thread has (elapsed,
    // batch label, entry position, memory vitals) and projects them through the one
    // renderer — identity first, human units, no raw byte dump. These golden strings
    // are the oracle the Rust mirror is proven byte-equal to
    // (`render_heartbeat_line_mirror_matches_seed_oracle`); the subject is batch-grain
    // (parallel entries → no fabricated per-module detail).
    #[test]
    fn seed_heartbeat_line_renders_identity_first_in_human_units() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        // The captured crawl window's memory beat: identity first, human units, the
        // raw byte value absent.
        let line = run_seed_heartbeat_line(
            &roots,
            1_980_000,
            "witness discovery",
            214,
            602,
            Some(16_107_200_512),
            Some(34_359_738_368),
            Some(901),
            true,
        )
        .expect("seed_heartbeat_line must resolve and render");
        assert_eq!(
            line,
            "🕐 33 minutes in — still in witness discovery: entry 214 of 602. memory 15.0 GiB, swap 32.0 GiB, pressure 9.0%"
        );
        assert!(
            !line.contains("16107200512"),
            "raw bytes must not appear: {line}"
        );

        // An unreadable cgroup field names its cause, never a fabricated zero.
        let unreadable = run_seed_heartbeat_line(
            &roots,
            500,
            "self-host fixed-point",
            0,
            2,
            None,
            Some(0),
            None,
            true,
        )
        .expect("resolve");
        assert_eq!(
            unreadable,
            "🕐 500ms in — still in self-host fixed-point: entry 0 of 2. memory unreadable (cgroup field unreadable), swap 0.0 GiB, pressure unreadable (cgroup field unreadable)"
        );
    }

    // Wiring flip 4b: the Rust mirror is proven byte-equal to the 4a seed oracle.
    // The heartbeat thread cannot call the interpreter (duplicate module index under
    // the memory envelope it watches); this pin is what keeps the mirror honest.
    #[test]
    fn render_heartbeat_line_mirror_matches_seed_oracle() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let crawl = run_seed_heartbeat_line(
            &roots,
            1_980_000,
            "witness discovery",
            214,
            602,
            Some(16_107_200_512),
            Some(34_359_738_368),
            Some(901),
            true,
        )
        .expect("seed oracle");
        let mirror = render_heartbeat_line_mirror(
            1_980_000,
            "witness discovery",
            214,
            602,
            Some(16_107_200_512),
            Some(34_359_738_368),
            Some(901),
            true,
        );
        assert_eq!(
            mirror, crawl,
            "mirror must be byte-equal to the seed oracle"
        );
        assert_eq!(
            mirror,
            "🕐 33 minutes in — still in witness discovery: entry 214 of 602. memory 15.0 GiB, swap 32.0 GiB, pressure 9.0%"
        );

        let unreadable_oracle = run_seed_heartbeat_line(
            &roots,
            500,
            "self-host fixed-point",
            0,
            2,
            None,
            Some(0),
            None,
            true,
        )
        .expect("seed oracle");
        let unreadable_mirror = render_heartbeat_line_mirror(
            500,
            "self-host fixed-point",
            0,
            2,
            None,
            Some(0),
            None,
            true,
        );
        assert_eq!(
            unreadable_mirror, unreadable_oracle,
            "unreadable fields must stay byte-equal"
        );
        assert!(
            !mirror.contains("16107200512") && !unreadable_mirror.contains("[floor-memory]"),
            "mirror must not carry the deleted byte-dump shape"
        );
    }

    #[test]
    fn psi_avg10_converts_to_basis_points_at_observation_scale() {
        assert_eq!(psi_avg10_to_basis_points("9.01"), Some(901));
        assert_eq!(psi_avg10_to_basis_points("37.5"), Some(3750));
        assert_eq!(psi_avg10_to_basis_points("0.0"), Some(0));
        assert_eq!(psi_avg10_to_basis_points("garbage"), None);
    }

    // The materialization-receipt chain by execution: a real entry resolves, a
    // claim evaluates on its InterpContext, and the ctx Drop absorbs ledger
    // totals into the process accumulator. Serialized via
    // PROCESS_EVAL_RECOMPUTE_TEST_LOCK so no sibling test can drain or pollute
    // the accumulator mid-assertion.
    #[test]
    fn materialization_receipt_totals_absorb_on_ctx_drop() {
        with_process_eval_recompute_test_lock(|| {
            let root = workspace_root();
            let roots = vec![
                root.join("src/v2").to_string_lossy().into_owned(),
                root.join("dag").to_string_lossy().into_owned(),
            ];
            let entry = root
                .join("dag/test/claim/materialization_ladder_witness_test.dag")
                .to_string_lossy()
                .into_owned();
            let _ = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
            {
                let (graph, indices) =
                    resolve_entry_graph(&roots, &entry).expect("resolve ladder witness entry");
                // Pure render evaluation (ci_render.dag fns over measured data) — no effects, so
                // the hermetic envelope is exact; a service call sneaking in refuses loudly.
                let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
                let outcome = run_claim(&ctx, "single_pure_demand_is_accepted_recompute");
                assert!(
                    matches!(outcome, ClaimOutcome::Pass),
                    "claim must pass for the receipt to be meaningful"
                );
            }
            let totals = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
            assert!(
                totals.keyed_calls > 0,
                "ctx Drop must absorb ledger totals into the process accumulator"
            );
        });
    }

    /// The deadline must FIRE, not merely be derivable.
    ///
    /// `falsifier_soft_deadline_is_derived_and_strictly_inside_the_hard_cap` proves the
    /// arithmetic; a number that no code ever compares against would satisfy it completely.
    /// This runs a real walk with the deadline already elapsed (0ms) and asserts the three
    /// things the mechanism actually owes: admission stops, the run is RED rather than
    /// silently short, and the unadmitted tail is reported as `deadline_reached` — not as
    /// the stop-policy skip it would have been misattributed as before.
    ///
    /// The `None` control is the discriminating half: the SAME two batches with no deadline
    /// run to completion. Without it this test would pass on a walk that was broken in some
    /// unrelated way and stopped early for a different reason.
    #[test]
    fn soft_deadline_stops_admission_and_reports_deadline_reached() {
        let root = workspace_root();
        let roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("src/v2/test/fixture/floor_skip/falsifier_divergence_control_test.dag")
            .to_string_lossy()
            .into_owned();
        let batch = || {
            vec![Runnable::SingleClaim {
                entry: entry.clone(),
                function: "falsifier_green_control_holds".to_string(),
                profile: ParsedRunnableProfile::undeclared(),
            }]
        };
        let batches = vec![batch(), batch()];

        let walk = |deadline: Option<u64>| {
            run_walk(
                &roots,
                "test::soft_deadline_fixture",
                &batches,
                &[],
                None,
                FloorFinalizationAbsenceReason::Undeclared,
                &mut std::io::stderr(),
                None,
                None,
                deadline,
                &RealizationConcurrency::for_walk(1).expect("test schedule"),
                None,
                FalsifierSelfHostWetBudgets::default(),
                FloorBatchStopPolicy::StopBeforeDependents,
                None,
                None,
                false,
                None,
            )
        };

        // CONTROL: no deadline — both components are admitted and the walk is green.
        let unbounded = walk(None);
        assert!(
            !unbounded.any_failed,
            "control walk must be green: {:?}",
            unbounded.failure_details
        );

        // ARMED: a deadline already in the past stops admission at the FIRST batch.
        let bounded = walk(Some(0));
        assert!(
            bounded.any_failed,
            "a walk that could not admit its components must be RED, never silently short"
        );
        assert!(
            bounded
                .failure_details
                .iter()
                .any(|d| d.contains("soft deadline")),
            "the refusal must name the deadline as the cause: {:?}",
            bounded.failure_details
        );
        // NEGATIVE: it must not be reported as the population budget, which is a different
        // ceiling with a different remedy.
        assert!(
            !bounded
                .failure_details
                .iter()
                .any(|d| d.contains("population budget")),
            "a deadline stop must not masquerade as the ordinary population budget: {:?}",
            bounded.failure_details
        );
    }

    fn parse_materialization_receipt_field(body: &str, key: &str) -> Option<u64> {
        body.lines()
            .find_map(|line| line.strip_prefix(key))
            .and_then(|v| v.trim().parse::<u64>().ok())
    }

    fn run_walk_materialization_fixture(
        roots: &[String],
        on_success_stages: &[Vec<Runnable>],
    ) -> WalkOutcome {
        let root = workspace_root();
        let ordinary_entry = root
            .join("src/v2/test/fixture/floor_skip/falsifier_divergence_control_test.dag")
            .to_string_lossy()
            .into_owned();
        let ordinary_batch = vec![vec![Runnable::SingleClaim {
            entry: ordinary_entry,
            function: "falsifier_green_control_holds".to_string(),
            profile: ParsedRunnableProfile::undeclared(),
        }]];
        const TEST_ATTEMPT_ID: &str = "materialization-receipt-test";
        let walk_attempt_id = if on_success_stages.is_empty() {
            None
        } else {
            Some(TEST_ATTEMPT_ID)
        };
        run_walk(
            roots,
            "test::on_success_materialization_fixture",
            &ordinary_batch,
            on_success_stages,
            None,
            FloorFinalizationAbsenceReason::Undeclared,
            &mut std::io::stderr(),
            None,
            None,
            None,
            &RealizationConcurrency::for_walk(1).expect("test schedule"),
            None,
            FalsifierSelfHostWetBudgets::default(),
            FloorBatchStopPolicy::StopBeforeDependents,
            None,
            None,
            true,
            walk_attempt_id,
        )
    }

    // Discriminating control (DESIGN §5): post-floor pure demand must appear in the
    // on-success materialization receipt and NOT in the ordinary-floor receipt, which
    // is harvested before stages run. Population separation is asserted within ONE
    // staged walk (ordinary receipt is written before on_success_stages in the same
    // run_walk — no cross-run keyed_calls equality that parallel siblings could
    // perturb, review 45766). PROCESS_EVAL_RECOMPUTE_TEST_LOCK serializes the
    // trace-enabled receipt tests and restores trace env/cache afterward (review
    // 45737, review 45756).
    #[test]
    fn on_success_materialization_receipt_separates_from_ordinary_floor() {
        with_process_eval_recompute_test_lock(|| {
            let root = workspace_root();
            with_workspace_root_current_dir(&root, || {
                let roots = vec![
                    root.join("src/v2").to_string_lossy().into_owned(),
                    root.join("dag").to_string_lossy().into_owned(),
                ];
                const TEST_ATTEMPT_ID: &str = "materialization-receipt-test";
                let success_receipt = root
                    .join("target")
                    .join(format!("floor-attempt-{TEST_ATTEMPT_ID}"))
                    .join("floor-on-success-materialization-receipt.txt");
                let ordinary_receipt = root.join("target/floor-materialization-receipt.txt");
                let _ = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();

                let run_fixture = |on_success_stages: &[Vec<Runnable>]| {
                    let _ = std::fs::remove_file(&success_receipt);
                    let _ = std::fs::remove_file(&ordinary_receipt);
                    let outcome = run_walk_materialization_fixture(&roots, on_success_stages);
                    assert!(
                        !outcome.any_failed,
                        "walk must pass: {:?}",
                        outcome.failure_details
                    );
                    outcome
                };

                // RED: no on-success stages => no success-stage materialization receipt.
                run_fixture(&[]);
                assert!(
                    !success_receipt.exists(),
                    "empty on_success_stages must not write a success materialization receipt"
                );

                // GREEN: one walk — ordinary receipt harvested before stages; success
                // receipt harvested after stage_memo drops in the same run_walk.
                let stage_entry = root
                    .join("dag/test/claim/materialization_ladder_witness_test.dag")
                    .to_string_lossy()
                    .into_owned();
                let success_stage = vec![vec![Runnable::SingleClaim {
                    entry: stage_entry,
                    function: "single_pure_demand_is_accepted_recompute".to_string(),
                    profile: ParsedRunnableProfile {
                        provenance: ParsedProfileProvenance::Declared,
                        heavy_whole_tree_resolve: false,
                        spawns_host_compiler: false,
                        memory: ParsedMemoryClass::Negligible,
                        execution_mode: ExecutionMode::Hermetic,
                    },
                }]];
                run_fixture(&success_stage);

                let ordinary_body =
                    std::fs::read_to_string(&ordinary_receipt).expect("ordinary receipt");
                let ordinary_keyed =
                    parse_materialization_receipt_field(&ordinary_body, "keyed_calls=")
                        .expect("keyed_calls");
                let success_body = std::fs::read_to_string(&success_receipt)
                    .expect("on-success materialization receipt");
                let success_keyed =
                    parse_materialization_receipt_field(&success_body, "keyed_calls=").unwrap();

                assert!(
                    success_body.starts_with(&format!(
                        "attempt_id={TEST_ATTEMPT_ID}\nplan_site=test::on_success_materialization_fixture\n"
                    )),
                    "success receipt must carry attempt and plan identity in the payload"
                );
                assert!(
                    ordinary_keyed > 0,
                    "ordinary-floor walk must record pure demand in the ordinary receipt"
                );
                assert!(
                    success_keyed > 0,
                    "on-success stage pure demand must be counted in the success receipt"
                );
                assert!(
                    parse_materialization_receipt_field(&success_body, "memo_hits=").is_some()
                        && parse_materialization_receipt_field(&success_body, "memo_misses=")
                            .is_some()
                        && parse_materialization_receipt_field(&success_body, "duplicated_keys=")
                            .is_some(),
                    "success receipt must carry the same field set as the ordinary receipt"
                );
            });
        });
    }

    // The eval-frame memo by execution: the same pure claim evaluated twice on
    // one ctx must (a) produce identical values — the memo-vs-recompute
    // equivalence oracle at the value grain — and (b) record verified hits, so
    // "the cache worked" is a counted fact, never an assumption. Assertions
    // are per-ctx (eval_call_memo_counters), immune to test-process sharing.
    // Serialized on PROCESS_EVAL_RECOMPUTE_TEST_LOCK: ctx Drop absorbs ledger
    // totals into the process accumulator when trace is on, so this test must
    // not overlap materialization-receipt walks (review 45737).
    #[test]
    fn eval_call_memo_serves_verified_hits_with_identical_values() {
        with_process_eval_recompute_test_lock(|| {
            let _ = v1_compiler::v1_interpreter::take_process_eval_recompute_totals();
            let root = workspace_root();
            let roots = vec![
                root.join("src/v2").to_string_lossy().into_owned(),
                root.join("dag").to_string_lossy().into_owned(),
            ];
            let entry = root
                .join("dag/test/claim/materialization_ladder_witness_test.dag")
                .to_string_lossy()
                .into_owned();
            let (graph, indices) =
                resolve_entry_graph(&roots, &entry).expect("resolve ladder witness entry");
            // Pure render evaluation (ci_render.dag fns over measured data) — no effects, so
            // the hermetic envelope is exact; a service call sneaking in refuses loudly.
            let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
            let first = run_value(
                &ctx,
                "cross_frame_duplicate_discharged_by_covering_provider",
            )
            .expect("first evaluation");
            let (_, misses_after_first, _) =
                v1_compiler::v1_interpreter::eval_call_memo_counters(&ctx);
            let second = run_value(
                &ctx,
                "cross_frame_duplicate_discharged_by_covering_provider",
            )
            .expect("second evaluation");
            assert!(
                first == second,
                "memo-served evaluation must equal the recomputed one"
            );
            let (hits, misses, overflow) =
                v1_compiler::v1_interpreter::eval_call_memo_counters(&ctx);
            assert!(
                hits > 0,
                "second identical evaluation must serve verified hits from the eval memo"
            );
            assert!(
                misses >= misses_after_first,
                "miss counter is monotone (counted, never reset)"
            );
            assert_eq!(overflow, 0, "tiny workload must not hit the entry cap");
        });
    }

    fn single(entry: &str, function: &str) -> Runnable {
        Runnable::SingleClaim {
            entry: entry.to_string(),
            function: function.to_string(),
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Hermetic,
            },
        }
    }

    /// A discovery batch that actually discovers something. `scan_dirs` is non-empty
    /// deliberately: `group_batch_units` emits a `BatchUnit::Discovery` only when the batch
    /// has scan dirs or explicit entries, because empty-and-empty is defined to mean zero
    /// witness-corpus nodes (the regen spec's shape, DESIGN "Building & checks"). A fixture
    /// with both empty is therefore not a discovery batch at all, and asserting that it
    /// produces a discovery unit asserts against the spec.
    fn discovery() -> Runnable {
        Runnable::DiscoveryBatch {
            source_roots: vec!["src/v2".to_string()],
            scan_dirs: vec!["dag/test/claim".to_string()],
            explicit_entries: vec![],
            native_bundle_entries: vec![],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: ExecutionMode::Hermetic,
            spawns_host_compiler: false,
        }
    }

    /// Flatten the grouped units back to the (entry, function) claims they will execute, in the
    /// order each unit runs them. `Discovery` contributes no SingleClaim pairs; an empty-entry
    /// sentinel surfaces as `("", function)`. This is the verdict-preservation oracle: grouping is
    /// only allowed to change scheduling, never which claims run.
    fn executed_claims(units: &[BatchUnit]) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for unit in units {
            match unit {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } => {
                    for f in functions {
                        out.push((entry.clone(), f.clone()));
                    }
                }
                BatchUnit::UnrunnableSentinel { function } => {
                    out.push((String::new(), function.clone()));
                }
                BatchUnit::Discovery { .. }
                | BatchUnit::ScopedDiscovery { .. }
                | BatchUnit::NativeBundle { .. } => {}
            }
        }
        out
    }

    #[test]
    fn same_entry_claims_coalesce_into_one_resolve_group() {
        // The floor's batch-2 shape: many gate witnesses share one file. They must collapse to a
        // single resolve-group (one resolve) carrying every function, in input order.
        let batch = vec![
            single("gate.dag", "rust_gate"),
            single("gate.dag", "emit_gate"),
            single("gate.dag", "layering_gate"),
        ];
        let units = group_batch_units(&batch);
        assert_eq!(
            units.len(),
            1,
            "three same-entry claims => one resolve-group"
        );
        match &units[0] {
            BatchUnit::SharedClaims {
                entry, functions, ..
            } => {
                assert_eq!(entry, "gate.dag");
                assert_eq!(functions, &["rust_gate", "emit_gate", "layering_gate"]);
            }
            _ => panic!("expected a SharedClaims unit"),
        }
    }

    #[test]
    fn native_bundle_kind_becomes_only_a_native_unit() {
        let batch = vec![Runnable::DiscoveryBatch {
            source_roots: vec!["src/v2".to_string(), "dag".to_string()],
            scan_dirs: vec![],
            explicit_entries: vec![],
            native_bundle_entries: vec![("bundle.dag".to_string(), "bundle_spec".to_string())],
            exclude_substrings: vec![],
            discovery_scope_dirs: vec![],
            execution_mode: ExecutionMode::Wet,
            spawns_host_compiler: true,
        }];
        let units = group_batch_units(&batch);
        assert_eq!(
            units.len(),
            1,
            "native kind must not create an interpreter discovery unit"
        );
        assert!(matches!(
            &units[0],
            BatchUnit::NativeBundle { entry, selector_function, execution_mode }
                if entry == "bundle.dag"
                    && selector_function == "bundle_spec"
                    && *execution_mode == ExecutionMode::Wet
        ));
    }

    #[test]
    fn native_bundle_refuses_without_fallback() {
        // CI-0 cutover: there is no fallback arm to test — the discriminating
        // evidence is that NOTHING except the full native bar reports ok.
        // Outage: refused (this exact input was the counted-fallback arm before
        // the cutover; it is the permanent regression control for the deleted arm).
        assert!(!native_transition_accepted(false, true, false));
        // Native green but oracle red: refused (equivalence evidence retained).
        assert!(!native_transition_accepted(true, false, true));
        // Native green, oracle green, planted RED not natively reproduced: refused.
        assert!(!native_transition_accepted(true, true, false));
        // Full native bar: accepted.
        assert!(native_transition_accepted(true, true, true));
        // Population counts: interpreted and fallback are structurally zero,
        // and a divergence (ran, wrong output) is its own column — never
        // laundered into "unavailable" (review 50560-class finding).
        assert_eq!(
            native_transition_population_counts(3, true, false),
            (3, 0, 0, 0, 0)
        );
        assert_eq!(
            native_transition_population_counts(3, false, false),
            (0, 0, 3, 0, 0)
        );
        assert_eq!(
            native_transition_population_counts(3, false, true),
            (0, 0, 0, 3, 0)
        );
    }

    /// A signalled process must not borrow an exit code it does not have. The seed
    /// rendered `.code().unwrap_or(-1)` for every leg before this, which made a runner
    /// OOM-kill read exactly like a process that chose to exit -1.
    #[test]
    fn native_leg_signal_death_is_not_rendered_as_an_exit_code() {
        assert_eq!(
            ProcessTermination::Signaled(9).located(),
            "killed by signal 9"
        );
        assert_eq!(ProcessTermination::Exited(101).located(), "exited 101");
        assert_eq!(
            ProcessTermination::Unobserved.located(),
            "termination unobserved"
        );
        // The discriminating half: no termination renders as the old fabricated -1,
        // and the signal arm shares no rendering with any exit arm.
        for termination in [
            ProcessTermination::Signaled(9),
            ProcessTermination::Signaled(11),
            ProcessTermination::Unobserved,
        ] {
            assert!(
                !termination.located().contains("exited"),
                "{termination:?} rendered as an exit"
            );
        }
    }

    /// Malformed transport wire must not borrow the legitimate `Unobserved` arm. "The
    /// process never produced a status" and "the transport violated its own modeled
    /// wire" have different owners and different fixes; collapsing them is the same
    /// state-space conflation this change removes at the `ExitStatus` boundary, one
    /// layer down. ONLY the explicit modeled `ProcessTerminationUnobserved` variant
    /// decodes to `Unobserved` — every other shape carries a located refusal.
    #[test]
    fn malformed_termination_wire_refuses_instead_of_reading_as_unobserved() {
        let graph = v1_compiler::v1_compiler_infer_items::ResolvedGraph {
            modules: Rc::new(Default::default()),
            item_registry: Rc::new(Default::default()),
            diagnostics: Rc::new(Default::default()),
            emit_graph_info: v1_compiler::v1_compiler_infer_emit_info::empty_emit_graph_info(),
        };
        let ctx = InterpContext::new(&graph, Rc::new(Default::default()), ExecutionMode::Hermetic);
        let variant = |name: &str, fields: Vec<(&str, Value)>| Value::Variant {
            type_name: ctx.sym("ProcessTermination"),
            variant_name: ctx.sym(name),
            fields: Rc::new(v1_compiler::v1_interpreter::sorted_fields(
                fields.into_iter().map(|(k, v)| (ctx.sym(k), v)).collect(),
            )),
        };
        // The modeled arms decode — including the ONE legitimate route to Unobserved.
        assert_eq!(
            transport_termination(Some(&variant("ProcessTerminationUnobserved", vec![])), &ctx)
                .unwrap(),
            ProcessTermination::Unobserved
        );
        assert_eq!(
            transport_termination(
                Some(&variant("ProcessExited", vec![("code", Value::Int(101))])),
                &ctx
            )
            .unwrap(),
            ProcessTermination::Exited(101)
        );
        assert_eq!(
            transport_termination(
                Some(&variant("ProcessSignaled", vec![("signal", Value::Int(9))])),
                &ctx
            )
            .unwrap(),
            ProcessTermination::Signaled(9)
        );
        // Every malformed shape refuses with a located cause, never Unobserved.
        let malformed: Vec<(&str, Option<Value>)> = vec![
            ("absent field", None),
            ("non-variant value", Some(Value::Int(0))),
            ("unknown variant", Some(variant("ProcessVanished", vec![]))),
            ("missing code", Some(variant("ProcessExited", vec![]))),
            (
                "non-integer signal",
                Some(variant(
                    "ProcessSignaled",
                    vec![("signal", str_value("9".to_string()))],
                )),
            ),
        ];
        for (label, value) in malformed {
            let refusal = transport_termination(value.as_ref(), &ctx)
                .err()
                .unwrap_or_else(|| panic!("{label}: malformed wire decoded as legitimate"));
            assert!(
                !refusal.located.is_empty(),
                "{label}: refusal carries no location"
            );
        }
    }

    /// The excerpt is the TAIL: cargo writes its error last and a panic writes its
    /// message last, so a head-anchored excerpt would reliably carry the least useful
    /// bytes. It is bounded so one refusal cannot flood the floor's result stream.
    #[test]
    fn stream_excerpt_keeps_the_tail_bounded_and_marks_truncation() {
        assert_eq!(stream_excerpt(b""), "<empty>");
        // An absent stream and a stream that said nothing are different observations
        // and neither may read as a message.
        assert_eq!(stream_excerpt(b"\n \n"), "<whitespace only>");
        // A stream almost always ends in a newline; the trailing marker must not
        // survive as a line the process never wrote.
        assert_eq!(
            stream_excerpt(b"error: no such command\n"),
            "error: no such command"
        );
        assert_eq!(
            stream_excerpt(b"first line\nsecond line\n"),
            "first line ⏎ second line"
        );
        let long: Vec<u8> = std::iter::repeat(b'x')
            .take(4000)
            .chain(b"THE ACTUAL ERROR".iter().copied())
            .collect();
        let excerpt = stream_excerpt(&long);
        assert!(excerpt.contains("THE ACTUAL ERROR"), "tail was dropped");
        assert!(excerpt.starts_with('…'), "truncation was not marked");
        assert!(
            excerpt.len() < 1400,
            "excerpt is unbounded: {}",
            excerpt.len()
        );
    }

    #[test]
    fn grouping_preserves_every_claim_exactly_once() {
        // Verdict preservation: no claim dropped, duplicated, or invented — grouping only reorders
        // by coalescing same-entry claims (keeping their relative order). Mixed entries interleave.
        let batch = vec![
            single("a.dag", "a1"),
            single("b.dag", "b1"),
            single("a.dag", "a2"),
            discovery(),
            single("b.dag", "b2"),
            single("", "__unmapped__"),
        ];
        let units = group_batch_units(&batch);

        // a.dag coalesces (a1 before a2), b.dag coalesces (b1 before b2), discovery + sentinel stay.
        let mut got = executed_claims(&units);
        let mut want = vec![
            ("a.dag".to_string(), "a1".to_string()),
            ("a.dag".to_string(), "a2".to_string()),
            ("b.dag".to_string(), "b1".to_string()),
            ("b.dag".to_string(), "b2".to_string()),
            (String::new(), "__unmapped__".to_string()),
        ];
        got.sort();
        want.sort();
        assert_eq!(got, want, "exact same claim set runs after grouping");

        // Same-entry relative order is preserved within each coalesced group.
        let a = units
            .iter()
            .find_map(|u| match u {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } if entry == "a.dag" => Some(functions),
                _ => None,
            })
            .expect("a.dag group present");
        assert_eq!(a, &["a1", "a2"]);
    }

    #[test]
    fn empty_entry_sentinel_is_its_own_failing_unit() {
        // The unmapped-node sentinel (empty entry) must never coalesce into a real resolve-group;
        // it is a stand-alone fail-closed unit so a non-complete plan stays red (DESIGN §5).
        let batch = vec![single("", "__gunbc_ci_floor_unmapped__")];
        let units = group_batch_units(&batch);
        assert_eq!(units.len(), 1);
        match &units[0] {
            BatchUnit::UnrunnableSentinel { function } => {
                assert_eq!(function, "__gunbc_ci_floor_unmapped__");
            }
            _ => panic!("empty-entry claim must be an UnrunnableSentinel, not a resolve-group"),
        }

        // And it fails closed when run.
        let unit = units.into_iter().next().unwrap();
        let results = run_batch_unit(
            vec!["src/v2".to_string()],
            unit,
            RealizationConcurrency::for_walk(1).expect("test schedule"),
            None,
            FalsifierSelfHostWetBudgets::default(),
            None,
        );
        assert_eq!(results.len(), 1);
        assert!(!results[0].ok, "unmapped sentinel must fail closed");
    }

    #[test]
    fn discovery_corpus_kind_label_follows_profile_not_roster_size() {
        assert_eq!(
            discovery_corpus_kind_label(&[], ExecutionMode::Wet, true),
            "bin-witness-corpus"
        );
        assert_eq!(
            discovery_corpus_kind_label(&[], ExecutionMode::Wet, false),
            "execution-corpus"
        );
        assert_eq!(
            discovery_corpus_kind_label(
                &["dag/test/claim".to_string()],
                ExecutionMode::Hermetic,
                false,
            ),
            "discovery-corpus"
        );
        assert_eq!(
            discovery_corpus_kind_label(&[], ExecutionMode::Hermetic, false),
            "explicit-corpus"
        );
    }

    #[test]
    fn discovery_batch_stays_an_isolated_unit() {
        // A discovery corpus node never merges with SingleClaims — it keeps its own shard-parallel
        // resolve path.
        let batch = vec![
            single("gate.dag", "g1"),
            discovery(),
            single("gate.dag", "g2"),
        ];
        let units = group_batch_units(&batch);
        assert_eq!(units.len(), 2, "gate group + discovery");
        assert!(units
            .iter()
            .any(|u| matches!(u, BatchUnit::Discovery { .. })));
        let gate = units
            .iter()
            .find_map(|u| match u {
                BatchUnit::SharedClaims {
                    entry, functions, ..
                } if entry == "gate.dag" => Some(functions),
                _ => None,
            })
            .expect("gate group present");
        assert_eq!(gate, &["g1", "g2"], "both gate claims kept in one group");
    }

    // --- build-artifact verification teeth (DESIGN §5 fail-open guard) ---

    fn write_exec(dir: &std::path::Path, name: &str, bytes: &[u8]) -> String {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        std::fs::write(&p, bytes).expect("write artifact");
        let mut perms = std::fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&p, perms).expect("chmod");
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn verify_build_artifacts_accepts_nonempty_executables() {
        let dir = std::env::temp_dir().join(format!("cev-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"\x7fELF-not-really-but-nonempty");
        let b = write_exec(&dir, "gunbc", b"binary");
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Ok(c) if c == ExitCode::SUCCESS),
            "real bins pass"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_zero_byte() {
        let dir = std::env::temp_dir().join(format!("cev-zero-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"ok");
        let b = write_exec(&dir, "gunbc", b""); // sccache served a truncated/empty cached artifact
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "zero-byte artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_missing() {
        let missing = std::env::temp_dir()
            .join(format!("cev-missing-{}/gunbc", std::process::id()))
            .to_string_lossy()
            .into_owned();
        let r = verify_build_artifacts(&[missing]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "absent artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_empty_arglist() {
        let r = verify_build_artifacts(&[]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(2)),
            "no declared artifacts is a usage error, fail closed"
        );
    }

    /// Warm==cold purity oracle for the cross-batch resolve memo (DESIGN §5).
    ///
    /// Proves that `run_memo_shared_claims` (warm path: 2nd+ calls share the cached
    /// `InterpContext`) produces byte-identical `(function, ok, detail)` results vs
    /// `run_shared_entry_claims` (cold path: fresh resolve each time). Goes RED if
    /// the shared `InterpContext` has interior mutability that contaminates results
    /// across claims — the failure mode the declaration-validity lenses cannot catch.
    #[test]
    fn memo_warm_cold_results_are_identical() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let functions: &[String] = &[
            "witness_negligible_profile_is_not_heavy".to_string(),
            "witness_substantial_memory_forbids_corpus_co_residence".to_string(),
        ];

        // Cold path: two independent resolves, one function each.
        let cold: Vec<(String, bool, String)> = functions
            .iter()
            .flat_map(|f| {
                run_shared_entry_claims(
                    &source_roots,
                    &entry,
                    std::slice::from_ref(f),
                    ExecutionMode::Hermetic,
                    None,
                )
                .into_iter()
                .map(|r| (r.function, r.ok, r.detail))
            })
            .collect();

        // Warm path: single memo, first call resolves fresh, second hits the cache.
        let mut memo = std::collections::HashMap::new();
        let warm: Vec<(String, bool, String)> = functions
            .iter()
            .flat_map(|f| {
                run_memo_shared_claims(
                    &source_roots,
                    &entry,
                    std::slice::from_ref(f),
                    ExecutionMode::Hermetic,
                    &mut memo,
                    None,
                )
                .into_iter()
                .map(|r| (r.function, r.ok, r.detail))
            })
            .collect();

        assert_eq!(
            warm, cold,
            "memo warm path must be byte-identical to cold path — shared InterpContext is pure"
        );
    }

    /// Resolve-count oracle: proves the memo fires resolve_entry_graph exactly once
    /// per distinct entry per walk (DESIGN §2 — no redundant resolve).
    ///
    /// Goes RED if the memo is bypassed: both calls would have resolve_nanos > 0,
    /// meaning resolve_entry_graph fired twice for the same entry instead of once.
    #[test]
    fn memo_deduplicates_resolve_count() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();

        let mut memo = std::collections::HashMap::new();

        // First call for this entry: must resolve (resolve_nanos > 0).
        let first = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_negligible_profile_is_not_heavy".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
            None,
        );
        assert!(
            first[0].resolve_nanos > 0,
            "first call must pay the resolve cost (resolve_entry_graph fires); ok={} detail={}",
            first[0].ok,
            first[0].detail
        );

        // Second call for the same entry AND mode: must cache-hit (resolve_nanos == 0).
        let second = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_substantial_memory_forbids_corpus_co_residence".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
            None,
        );
        assert_eq!(
            second[0].resolve_nanos, 0,
            "second call must cache-hit — resolve_entry_graph must NOT fire again"
        );
    }

    /// Warm memo reuse must attach SatisfiedFromSharedPool to the first claim in the
    /// group — finalization reads resolve_realization from find_claim_result, not from
    /// resolve_nanos alone. Goes RED if `first` is gated on `fresh_resolve`.
    #[test]
    fn memo_warm_attaches_shared_pool_observation_on_first_claim() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let mut memo = std::collections::HashMap::new();
        let _cold = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_negligible_profile_is_not_heavy".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
            None,
        );
        let warm = run_memo_shared_claims(
            &source_roots,
            &entry,
            &["witness_substantial_memory_forbids_corpus_co_residence".to_string()],
            ExecutionMode::Hermetic,
            &mut memo,
            None,
        );
        assert_eq!(warm[0].resolve_nanos, 0);
        match warm[0].resolve_realization.as_ref() {
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool { provider_id, .. }) => {
                assert_eq!(provider_id, FLOOR_ENTRY_WALK_MEMO_PROVIDER_ID);
            }
            other => panic!(
                "warm memo first claim must carry SatisfiedFromSharedPool observation, got {other:?}"
            ),
        }
    }

    #[test]
    fn take_group_observation_attaches_only_to_rostered_subject() {
        let subjects: ObligationSubjectSet =
            [("fixture.dag".to_string(), "rostered_fn".to_string())]
                .into_iter()
                .collect();
        let observation =
            Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 42 });
        let mut attached = false;
        assert!(take_group_observation_for_claim(
            Some(&subjects),
            "fixture.dag",
            "other_fn",
            &observation,
            &mut attached,
        )
        .is_none());
        assert!(!attached);
        assert!(matches!(
            take_group_observation_for_claim(
                Some(&subjects),
                "fixture.dag",
                "rostered_fn",
                &observation,
                &mut attached,
            ),
            Some(ResolveRealizationObservation::ColdResolvePerformed { resolve_nanos: 42 })
        ));
    }

    #[test]
    fn obligation_observation_skips_non_rostered_co_residents() {
        let root = workspace_root();
        let source_roots = vec![
            root.join("src/v2").to_string_lossy().into_owned(),
            root.join("dag").to_string_lossy().into_owned(),
        ];
        let entry = root
            .join("dag/test/claim/runnable_resource_profile_witness_test.dag")
            .to_string_lossy()
            .into_owned();
        let rostered_fn = "witness_substantial_memory_forbids_corpus_co_residence".to_string();
        let non_rostered_fn = "witness_negligible_profile_is_not_heavy".to_string();
        let subjects: ObligationSubjectSet =
            [(entry.clone(), rostered_fn.clone())].into_iter().collect();
        let mode = ExecutionMode::Hermetic;

        let cheap_only = run_shared_entry_claims(
            &source_roots,
            &entry,
            &[non_rostered_fn.clone()],
            mode,
            Some(&subjects),
        );
        assert!(cheap_only[0].resolve_realization.is_none());

        let mut memo = std::collections::HashMap::new();
        let rostered_only = run_memo_shared_claims(
            &source_roots,
            &entry,
            &[rostered_fn.clone()],
            mode,
            &mut memo,
            Some(&subjects),
        );
        assert!(matches!(
            rostered_only[0].resolve_realization,
            Some(ResolveRealizationObservation::ColdResolvePerformed { .. })
        ));

        let co_resident = run_memo_shared_claims(
            &source_roots,
            &entry,
            &[non_rostered_fn, rostered_fn],
            mode,
            &mut memo,
            Some(&subjects),
        );
        assert!(co_resident[0].resolve_realization.is_none());
        assert!(matches!(
            co_resident[1].resolve_realization,
            Some(ResolveRealizationObservation::SatisfiedFromSharedPool { .. })
        ));
    }

    /// The committed basis file must actually load. A basis that parses to zero rows
    /// pins `drift_exceeded` at 0 by construction and reports it as "no drift" — the
    /// state this file spent its whole life in (`basis_absent=5344` on run 30550328673),
    /// where the loud missing-file diagnostic never fired because the file *existed* and
    /// was merely empty of data. So this asserts the artifact is non-vacuous, not that
    /// the parser works in the abstract.
    #[test]
    fn committed_witness_row_cost_basis_loads_and_excludes_the_censored_row() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("dag/gunbc/witness_row_cost_basis.tsv");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("basis file unreadable at {}: {e}", path.display()));

        let mut loaded = 0usize;
        for line in text.lines().skip(1) {
            match parse_witness_row_cost_basis_line(line) {
                Ok(Some(_)) => loaded += 1,
                Ok(None) => {}
                // A refused row is silently skipped by the loader, so a malformed commit
                // would degrade to BasisAbsent instead of failing. Catch it here instead.
                Err(msg) => panic!("committed basis row refused by the loader: {msg}"),
            }
        }
        assert!(
            loaded >= 1000,
            "committed basis loaded only {loaded} row(s) — a near-empty basis reports \
             drift_exceeded=0 by construction, never by measurement"
        );

        // The corpus's most expensive row was KILLED at its 900000ms deadline, so its
        // recorded 900794ms is a censored measurement, not a completed one. Seeding it
        // would pin it WithinBasis below ~1.8M ms under the 2x comparator and enshrine a
        // deadline ceiling as normal cost. It must stay BasisAbsent — the honest third
        // state — until ClaimOutcome distinguishes killed from completed, which it now does.
        for line in text.lines() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            assert!(
                !line.contains("resolution_divergence_silent_pick_gate_keystone_holds"),
                "censored (deadline-killed) row must not be seeded as a basis: {line}"
            );
        }
    }

    fn drift_authority_source_roots() -> Vec<String> {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        vec![
            workspace.join("dag").to_string_lossy().into_owned(),
            workspace.join("src/v2").to_string_lossy().into_owned(),
        ]
    }

    fn drift_basis_fixture(clock_constructor: &'static str) -> WitnessRowCostBasisRow {
        WitnessRowCostBasisRow {
            eval_ms_basis: 10,
            run_ref: "synthetic-run".to_string(),
            clock_constructor,
        }
    }

    /// The END-TO-END proof that the drift seam carries the clock, which neither the `.dag`
    /// witnesses nor the parser tests can give: the witnesses never cross this Rust boundary,
    /// and the parser only shows the cell is read.
    ///
    /// Two claims. First, every verdict string in the receipt is a name the AUTHORITY
    /// produced — the seed no longer decides that exceeding a basis means `DriftExceeded`, so
    /// an arm added to `WitnessRowCostVerdict` reaches the cadence receipt with no Rust edit.
    /// Second, a cross-clock pair REFUSES, asserted on BOTH sides of the 2× ratio: 21ms and
    /// 20ms against a 10ms basis land on opposite sides of the threshold, so a single case
    /// would be satisfiable by a comparator refusing for the wrong reason. The pair proves
    /// the refusal is decided by the CLOCK, not the magnitude.
    #[test]
    fn drift_verdicts_come_from_the_authority_and_a_cross_clock_basis_refuses() {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("workspace root");
        let roots = drift_authority_source_roots();
        let entry = workspace.join("dag/gunbc/witness_row_cost.dag");
        let (graph, indices) =
            resolve_entry_graph(&roots, &entry.to_string_lossy()).expect("resolve");
        let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);

        let wall = drift_basis_fixture("clock_basis_wall");
        let cpu = drift_basis_fixture("clock_basis_cpu");
        let verdict = |observed, basis| {
            witness_row_cost_verdict_via_authority(&ctx, observed, basis).expect("verdict")
        };

        // Same clock on both sides: the ratio decides, in both directions.
        assert_eq!(verdict(21, Some(&wall)), "DriftExceeded");
        assert_eq!(verdict(20, Some(&wall)), "WithinBasis");

        // No dated basis: the honest third state, and it too comes from the authority.
        assert_eq!(verdict(999, None), "BasisAbsent");

        // Different clocks: refused on BOTH sides of the ratio. Before the clock crossed
        // this seam these two answered `DriftExceeded` and `WithinBasis` — confident
        // verdicts about two different quantities.
        assert_eq!(verdict(21, Some(&cpu)), "BasisClockMismatch");
        assert_eq!(verdict(20, Some(&cpu)), "BasisClockMismatch");
    }

    #[test]
    fn parse_witness_row_cost_basis_line_requires_srv_fleet_arm64() {
        // RED control for review 43284: wrong/missing host_class must refuse, never load.
        let ok = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64\twall")
            .expect("parse")
            .expect("row");
        assert_eq!(ok.0, ("e.dag".to_string(), "f".to_string()));
        assert_eq!(ok.1.eval_ms_basis, 10);
        assert_eq!(ok.1.run_ref, "run-1");
        assert_eq!(ok.1.clock_constructor, "clock_basis_wall");

        // The clock is READ, not assumed: a cpu-clocked basis row loads as cpu, which is
        // what lets the comparator refuse it against a wall observation instead of
        // answering. If this cell were ignored the whole column would be decoration.
        let cpu = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64\tcpu")
            .expect("parse")
            .expect("row");
        assert_eq!(cpu.1.clock_constructor, "clock_basis_cpu");

        assert!(parse_witness_row_cost_basis_line("# comment")
            .unwrap()
            .is_none());
        assert!(parse_witness_row_cost_basis_line("").unwrap().is_none());

        let wrong = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tlocal_x86\twall")
            .expect_err("wrong host_class must refuse");
        assert!(
            wrong.contains("host_class") && wrong.contains("srv_fleet_arm64"),
            "expected host_class refusal, got: {wrong}"
        );

        // A row with no clock cell REFUSES rather than defaulting to wall. Defaulting
        // would be right for every row in the file today and silently wrong the first
        // time one is seeded from a CPU receipt — the exact failure the column exists to
        // prevent, so the pre-clock 5-column shape must not still parse.
        let short = parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64")
            .expect_err("a row without a clock column must refuse");
        assert!(short.contains("need 6 cols"), "got: {short}");

        // An unmodelled clock is not a clock. It cannot map to a ClockBasis constructor,
        // so it cannot enter a comparison at all.
        let unknown =
            parse_witness_row_cost_basis_line("e.dag\tf\t10\trun-1\tsrv_fleet_arm64\tmonotonic")
                .expect_err("unknown clock must refuse");
        assert!(
            unknown.contains("clock") && unknown.contains("monotonic"),
            "got: {unknown}"
        );

        let zero = parse_witness_row_cost_basis_line("e.dag\tf\t0\trun-1\tsrv_fleet_arm64\twall")
            .expect_err("zero eval must refuse");
        assert!(zero.contains("zero eval_ms_basis"), "got: {zero}");
    }

    #[test]
    /// Discriminating replay control: planted passed, failed, and selection-skipped rows
    /// must surface as three distinct coordinator replay outcomes (not collapsed).
    /// Evidence: `claim_executor` unit tests — the binary that gates CI floor merge.
    fn wet_witness_row_outcome_replay_discriminates_passed_failed_and_selection_skipped() {
        let base =
            std::env::temp_dir().join(format!("claim-executor-wet-outcome-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let records = vec![BatchRecord {
            batch_index: 3,
            wall_nanos: 0,
            clamp_ms: None,
            unit_count: 3,
            runtime_units: FloorRuntimeUnitCount::Observed { units: 3 },
            label: "bin-witness-corpus".to_string(),
            is_wet: true,
            results: vec![ClaimResult {
                function: "bin-witness-corpus (3 witnesses)".to_string(),
                entry: DISCOVERY_AGGREGATE_ENTRY.to_string(),
                ok: false,
                detail: "1 of 3 discovery witness(es) failed".to_string(),
                wall_nanos: 0,
                resolve_nanos: 0,
                corpus_resolve_nanos: 0,
                corpus_eval_nanos: 0,
                corpus_witnesses: 3,
                runtime_unit_count: discovery_runtime_unit_count_from_summary(3),
                witness_row_costs: vec![
                    WitnessRowCost {
                        entry: "dag/test/claim/stage0_rust_host_observation_live_witness_test.dag"
                            .to_string(),
                        function: "planted_pass_wet_row".to_string(),
                        eval_wall_nanos: 1_000_000,
                        eval_cpu_nanos: Some(800_000),
                        resolve_nanos: 0,
                        warm_nanos: 0,
                        outcome: "Done".to_string(),
                        detail: String::new(),
                    },
                    WitnessRowCost {
                        entry: "dag/test/claim/planted_fail_wet_row_test.dag".to_string(),
                        function: "planted_fail_wet_row".to_string(),
                        eval_wall_nanos: 500_000,
                        eval_cpu_nanos: Some(400_000),
                        resolve_nanos: 0,
                        warm_nanos: 0,
                        outcome: "Failed".to_string(),
                        detail: "returned Bool(false)".to_string(),
                    },
                    WitnessRowCost {
                        entry: "dag/other.dag".to_string(),
                        function: "planted_selection_skip_wet_row".to_string(),
                        eval_wall_nanos: 0,
                        eval_cpu_nanos: None,
                        resolve_nanos: 0,
                        warm_nanos: 0,
                        outcome: "selection-skipped".to_string(),
                        detail: "affected-set".to_string(),
                    },
                ],
                expectation_refusal: None,
                budget_refusal: None,
                host_dependency_refusal: None,
                resolve_realization: None,
            }],
        }];
        assert!(write_floor_wet_witness_row_outcome_receipt_at(
            &base, &records
        ));
        let path = base.join("floor-wet-witness-row-outcome-receipt.tsv");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("planted_pass_wet_row"));
        assert!(body.contains("planted_pass_wet_row\tpassed"));
        assert!(body.contains("planted_fail_wet_row"));
        assert!(body.contains("planted_fail_wet_row\tfailed\t"));
        assert!(body.contains("planted_selection_skip_wet_row"));
        assert!(body.contains("selection-skipped"));
        let lines = collect_wet_witness_row_outcome_replay_lines(&path).expect("replay lines");
        assert_eq!(lines.len(), 3);
        let passed_line = lines
            .iter()
            .find(|line| line.contains("planted_pass_wet_row"))
            .expect("passed replay line");
        let failed_line = lines
            .iter()
            .find(|line| line.contains("planted_fail_wet_row"))
            .expect("failed replay line");
        let skipped_line = lines
            .iter()
            .find(|line| line.contains("planted_selection_skip_wet_row"))
            .expect("selection-skipped replay line");
        assert!(passed_line.contains("outcome=passed"));
        assert!(failed_line.contains("outcome=failed"));
        assert!(skipped_line.contains("outcome=selection-skipped"));
        assert_ne!(passed_line, failed_line);
        assert_ne!(passed_line, skipped_line);
        assert_ne!(failed_line, skipped_line);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn wet_witness_row_outcome_label_maps_three_distinct_values() {
        assert_eq!(wet_witness_row_outcome_label("Done"), "passed");
        assert_eq!(
            wet_witness_row_outcome_label("selection-skipped"),
            "selection-skipped"
        );
        assert_eq!(wet_witness_row_outcome_label("Failed"), "failed");
        assert_eq!(wet_witness_row_outcome_label("Refused"), "failed");
    }

    /// Discovery budget kills must classify structurally like any other batch.
    #[test]
    fn discovery_budget_kill_classifies_structurally_on_the_falsifier_path() {
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let killed_detail =
            "1 of 1 discovery witness(es) failed: fn=expensive_witness killed at its wall budget: \
             900001ms elapsed > 900000ms budget";
        assert_eq!(
            falsifier_failure_mode(&[killed_detail.to_string()]),
            "WitnessRed",
            "control is only meaningful if the string path would misclassify this detail"
        );

        let summary_with = |outcome: ClaimOutcome| DiscoverySummary {
            total: 1,
            passed: 0,
            skipped: 0,
            deferred_rows: Vec::new(),
            divergences: Vec::new(),
            failures: vec![killed_detail.into()],
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "dag/test/claim/expensive_wet_witness_test.dag".into(),
                module_path: "test.claim.expensive_wet_witness".into(),
                function: "expensive_witness_keystone_holds".into(),
                outcome,
                execution_leg: "InterpretedLeg".into(),
            }],
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
            total_entry_groups: 0,
            selected_entry_groups: 0,
        };
        let killed = ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 900_001,
            budget_ms: 900_000,
            kind: BudgetKind::Wall,
        };

        // Both arms: the receipt projection may succeed or refuse, and a receipt refusal must
        // ADD a cause rather than erase the budget kill already established.
        for projected in [
            Ok(Vec::new()),
            Err("[witness-row-cost] REFUSED: missing measured resolve parent".to_string()),
        ] {
            let result = discovery_claim_result(
                "discovery-corpus".into(),
                false,
                killed_detail.to_string(),
                &summary_with(killed.clone()),
                projected,
                None,
            );
            assert!(
                result.budget_refusal.is_some(),
                "discovery batch must lift the budget kill out of witness_outcomes"
            );
            let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![result]));
            assert_eq!(mode, "BudgetExceeded");
        }

        // And a discovery batch with an ordinary red must NOT be dragged into BudgetExceeded.
        let result = discovery_claim_result(
            "discovery-corpus".into(),
            false,
            "red".into(),
            &summary_with(ClaimOutcome::Fail),
            Ok(Vec::new()),
            None,
        );
        assert!(result.budget_refusal.is_none());
    }

    /// RED control for review 51796: discovery batch must lift host-dependency refusals out of
    /// `summary.failures`, parallel to `discovery_budget_kill_classifies_structurally_on_the_falsifier_path`.
    #[test]
    fn discovery_host_dependency_absent_classifies_structurally_on_the_falsifier_path() {
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let wire = "HostDependencyAbsent{tool=npm,hint=apt install npm}";
        let failure = format!(
            "materialize_codex_runtime_bundle_produces_native_executable_holds \
             (dag/test/claim/codex_package_delivery_wet_witness_test.dag) returned Bool(false) | \
             {wire}"
        );
        let aggregate_detail = format!("1 of 1 discovery witness(es) failed: {failure}");
        assert_eq!(
            falsifier_failure_mode(&[aggregate_detail.clone()]),
            "WitnessRed",
            "control: string classifier alone would misclassify"
        );

        let summary_with = |outcome: ClaimOutcome, failures: Vec<String>| DiscoverySummary {
            total: 1,
            passed: 0,
            skipped: 0,
            deferred_rows: Vec::new(),
            divergences: Vec::new(),
            failures,
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "dag/test/claim/codex_package_delivery_wet_witness_test.dag".into(),
                module_path: "test.claim.codex_package_delivery_wet_witness_test".into(),
                function: "materialize_codex_runtime_bundle_produces_native_executable_holds"
                    .into(),
                outcome,
                execution_leg: "InterpretedLeg".into(),
            }],
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
            total_entry_groups: 0,
            selected_entry_groups: 0,
        };

        for projected in [
            Ok(Vec::new()),
            Err("[witness-row-cost] REFUSED: missing measured resolve parent".to_string()),
        ] {
            let result = discovery_claim_result(
                "discovery-corpus".into(),
                false,
                aggregate_detail.clone(),
                &summary_with(ClaimOutcome::Fail, vec![failure.clone()]),
                projected,
                None,
            );
            assert!(
                result.host_dependency_refusal.is_some(),
                "discovery batch must lift the host dependency refusal out of summary.failures"
            );
            let (mode, _) = batch_failure_mode_and_detail(&batch_record_for_test(vec![result]));
            assert_eq!(mode, HOST_DEPENDENCY_ABSENT_MODE);
        }

        let result = discovery_claim_result(
            "discovery-corpus".into(),
            false,
            "red".into(),
            &summary_with(ClaimOutcome::Fail, vec!["returned Bool(false)".into()]),
            Ok(Vec::new()),
            None,
        );
        assert!(result.host_dependency_refusal.is_none());
    }

    #[test]
    fn discovery_claim_result_preserves_caller_detail_on_receipt_refuse() {
        // RED control for review 43284: receipt refusal must append, never overwrite
        // the discovery failure diagnostic.
        use v1_compiler::cli_run::{
            ClaimOutcome, DiscoverySummary, DiscoveryWitnessOutcome, EntryResolveReceipt,
            ResolveStageNanos,
        };
        let summary = DiscoverySummary {
            total: 1,
            passed: 0,
            skipped: 0,
            deferred_rows: Vec::new(),
            divergences: Vec::new(),
            failures: vec!["e.dag::f failed".into()],
            witness_outcomes: vec![DiscoveryWitnessOutcome {
                entry: "e.dag".into(),
                module_path: "test.e".into(),
                function: "f".into(),
                outcome: ClaimOutcome::Fail,
                execution_leg: "InterpretedLeg".into(),
            }],
            // Empty entry_resolve_receipts → the authored receipt projection refuses.
            entry_resolve_receipts: Vec::<EntryResolveReceipt>::new(),
            total_resolve_nanos: 0,
            total_stage_nanos: ResolveStageNanos::default(),
            performance_receipts: Vec::new(),
            total_measured_nanos: 0,
            roster_closure_nodes: 0,
            total_entry_groups: 0,
            selected_entry_groups: 0,
        };
        let prior = "1 of 1 discovery witness(es) failed: e.dag::f failed";
        let result = discovery_claim_result(
            "probe".into(),
            false,
            prior.to_string(),
            &summary,
            Err("[witness-row-cost] REFUSED: missing measured resolve parent for e.dag".into()),
            None,
        );
        assert!(!result.ok);
        assert!(
            result.detail.contains(prior),
            "caller discovery failure must be preserved, got: {}",
            result.detail
        );
        assert!(
            result.detail.contains("witness row-cost receipt refused"),
            "receipt refusal must also be present, got: {}",
            result.detail
        );
    }

    fn latch_result(function: &str) -> ClaimResult {
        ClaimResult {
            function: function.to_string(),
            entry: "latch_fixture.dag".to_string(),
            ok: true,
            detail: String::new(),
            wall_nanos: 0,
            resolve_nanos: 0,
            corpus_resolve_nanos: 0,
            corpus_eval_nanos: 0,
            corpus_witnesses: 0,
            runtime_unit_count: single_claim_runtime_unit_count(),
            witness_row_costs: Vec::new(),
            expectation_refusal: None,
            budget_refusal: None,
            host_dependency_refusal: None,
            resolve_realization: None,
        }
    }

    /// LATCH, not timing. Each member increments a shared counter, then waits on a
    /// condvar until the counter reaches 2 — which can only happen if the OTHER member
    /// is already running. Under a serial executor the first member waits forever for a
    /// peer that has not been started, so the bounded wait expires and `observed_peer`
    /// stays false. The bound is a deadlock detector, never the assertion: the assertion
    /// is that each member SAW the other, which no amount of slowness can fake and no
    /// amount of speed can satisfy serially.
    #[test]
    fn stage_members_actually_overlap() {
        use std::sync::{Arc, Condvar, Mutex};

        let state = Arc::new((Mutex::new(0usize), Condvar::new()));
        let observed_peer = Arc::new(Mutex::new(vec![false, false]));

        let work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = (0..2)
            .map(|i| {
                let state = state.clone();
                let observed = observed_peer.clone();
                let boxed: Box<dyn FnOnce() -> Vec<ClaimResult> + Send> = Box::new(move || {
                    let (lock, cvar) = &*state;
                    let mut count = lock.lock().unwrap();
                    *count += 1;
                    cvar.notify_all();
                    let mut saw_peer = *count == 2;
                    while !saw_peer {
                        let (guard, timeout) = cvar
                            .wait_timeout(count, std::time::Duration::from_secs(10))
                            .unwrap();
                        count = guard;
                        saw_peer = *count == 2;
                        if timeout.timed_out() {
                            break;
                        }
                    }
                    observed.lock().unwrap()[i] = saw_peer;
                    vec![latch_result(&format!("member-{i}"))]
                });
                boxed
            })
            .collect();

        let (results, panicked) = join_units(spawn_units(work));
        assert!(!panicked, "no member should panic");
        assert_eq!(results.len(), 2, "both members must report");
        let observed = observed_peer.lock().unwrap();
        assert!(
            observed[0] && observed[1],
            "each member must observe the other running concurrently; \
             serial execution leaves one or both false: {observed:?}"
        );
    }

    /// The other half of the barrier: the join returns only after EVERY member finished.
    /// The slow member sets a flag last; if `join_units` returned early, the flag would
    /// still be false when the assertion runs, and the result count would be short.
    /// Together with the stage loop's sequential `&mut stage_memo` borrow, this is what
    /// makes "stage N+1 cannot begin before every stage-N member completed" true.
    #[test]
    fn join_waits_for_every_member_before_returning() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let finished = Arc::new(AtomicUsize::new(0));
        let members = 4usize;
        let work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = (0..members)
            .map(|i| {
                let finished = finished.clone();
                let boxed: Box<dyn FnOnce() -> Vec<ClaimResult> + Send> = Box::new(move || {
                    // Staggered so at least one member is still running well after the
                    // others have returned — the case an early join would expose.
                    std::thread::sleep(std::time::Duration::from_millis(20 * (i as u64 + 1)));
                    finished.fetch_add(1, Ordering::SeqCst);
                    vec![latch_result(&format!("member-{i}"))]
                });
                boxed
            })
            .collect();

        let (results, panicked) = join_units(spawn_units(work));
        assert!(!panicked);
        assert_eq!(
            finished.load(Ordering::SeqCst),
            members,
            "join must not return before every member completed"
        );
        assert_eq!(
            results.len(),
            members,
            "every member's results must be collected"
        );
    }

    /// A panicking member is collected as an INFRA fault, not silently dropped and not
    /// propagated: the caller still needs to close its host-effect group and report the
    /// fault distinctly from a claim verdict. The surviving members' results still come
    /// back, so one bad unit does not erase the stage's evidence.
    #[test]
    fn join_reports_a_panicking_member_without_losing_the_others() {
        let work: Vec<Box<dyn FnOnce() -> Vec<ClaimResult> + Send>> = vec![
            Box::new(|| vec![latch_result("ok-member")]),
            Box::new(|| panic!("member exploded")),
        ];
        let (results, panicked) = join_units(spawn_units(work));
        assert!(panicked, "a panicking member must be reported");
        assert_eq!(
            results.len(),
            1,
            "the surviving member's results must still be collected"
        );
    }

    // ---- walk-attempt identity ----
    //
    // The refusals below are the point of the feature, so each is asserted as a REFUSAL,
    // not merely as "not equal to the good value". The positive controls exist so the
    // negatives are discriminating: a `compose_walk_attempt_id` that refused everything
    // would pass every Err assertion and fail these.

    #[test]
    fn attempt_identity_composes_from_the_github_triple() {
        assert_eq!(
            compose_walk_attempt_id("", "30654655022", "1", "ci").unwrap(),
            "30654655022-1-ci"
        );
    }

    #[test]
    fn attempt_identity_prefers_the_explicit_value() {
        assert_eq!(
            compose_walk_attempt_id("local-probe-7", "30654655022", "1", "ci").unwrap(),
            "local-probe-7",
            "an explicitly supplied identity must win over the ambient GitHub triple"
        );
    }

    #[test]
    fn attempt_identity_refuses_rather_than_defaulting_when_absent() {
        // THE load-bearing control. The ruling forbids "a silent constant like a bare
        // local, which would make every local run one attempt and the wrong-attempt
        // refusal unreachable off CI". If someone ever adds a default, this reds.
        let refusal = compose_walk_attempt_id("", "", "", "")
            .expect_err("an unidentified walk must refuse, never default");
        assert!(
            refusal.contains("GUNBC_WALK_ATTEMPT_ID"),
            "the refusal must name the input that would satisfy it, got: {refusal}"
        );
    }

    #[test]
    fn attempt_identity_refuses_a_partial_github_triple() {
        // A partial triple is the shape a non-`ci` job or a changed workflow produces.
        // Composing from it would silently collide two jobs of one run onto one identity.
        for (run_id, attempt, job) in [
            ("30654655022", "1", ""),
            ("30654655022", "", "ci"),
            ("", "1", "ci"),
        ] {
            assert!(
                compose_walk_attempt_id("", run_id, attempt, job).is_err(),
                "partial triple ({run_id:?},{attempt:?},{job:?}) must refuse"
            );
        }
    }

    #[test]
    fn attempt_identity_refuses_every_unsafe_path_segment() {
        // Mirrors `std.types.path_segment_is_safe` clause for clause. `..` and `/` are the
        // escapes that would let a receipt be written outside its attempt directory; CR and
        // LF are the ones that would split the line-oriented receipt written under that
        // name, so two lines could be forged from one field.
        for bad in ["", ".", "..", "a/b", "a\\b", "a\nb", "a\rb", "a\0b"] {
            assert!(
                compose_walk_attempt_id(bad, "run", "1", "ci").is_err() || bad.trim().is_empty(),
                "unsafe explicit segment {bad:?} must refuse"
            );
            assert!(
                !walk_attempt_id_segment_is_safe(bad),
                "segment {bad:?} must not be considered safe"
            );
        }
        assert!(
            walk_attempt_id_segment_is_safe("30654655022-1-ci"),
            "a real composed identity must be accepted — else the negatives above prove nothing"
        );
    }

    #[test]
    fn floor_worker_missing_terminal_receipt_is_an_observed_death() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-missing-terminal-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&terminal);
        let status = Command::new("sh").arg("-c").arg("exit 0").status().unwrap();
        let observed = observe_floor_worker("ordinary", status, &terminal);
        let outcome = floor_worker_observation_outcome(&observed);
        assert_eq!(
            observed.terminal_receipt,
            FloorWorkerTerminalReceipt::Missing
        );
        assert_eq!(outcome.label, "died-without-terminal-receipt");
        assert!(outcome.detail.contains("ordinary"));
        assert!(outcome.detail.contains("no terminal receipt"));
        assert!(
            !floor_worker_succeeded(&observed),
            "an OS-success exit cannot turn receipt absence into floor success"
        );
    }

    #[test]
    fn floor_phase_journal_persists_a_completed_path() {
        let journal = std::env::temp_dir().join(format!(
            "claim-executor-floor-phase-positive-control-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&journal);
        std::env::set_var(FLOOR_PHASE_JOURNAL_ENV, &journal);
        append_floor_phase_journal("positive-control", "completed", "known-green-path");
        std::env::remove_var(FLOOR_PHASE_JOURNAL_ENV);

        let persisted = fs::read_to_string(&journal)
            .expect("the synced out-of-band journal must be readable after a completed path");
        let _ = fs::remove_file(&journal);
        assert!(
            persisted.contains("\tpositive-control\tcompleted\tknown-green-path\n"),
            "the persisted row must retain the phase, state, and detail: {persisted:?}"
        );
    }

    #[test]
    fn floor_worker_verdict_reaches_the_durable_journal() {
        let journal = std::env::temp_dir().join(format!(
            "claim-executor-floor-worker-verdict-journal-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&journal);
        std::env::set_var(FLOOR_PHASE_JOURNAL_ENV, &journal);
        journal_floor_worker_observation(&ObservedFloorWorker {
            worker: "scoped:batch-a".to_string(),
            termination: ProcessTermination::Exited(1),
            terminal_receipt: FloorWorkerTerminalReceipt::Observed(
                FloorWorkerTerminalReport::Failed("witness row was red".to_string()),
            ),
        });
        std::env::remove_var(FLOOR_PHASE_JOURNAL_ENV);

        let persisted = fs::read_to_string(&journal)
            .expect("the synced worker verdict must survive on the out-of-band journal");
        let _ = fs::remove_file(&journal);
        assert!(
            persisted.contains(
                "\tcoordinator-observation\tfailed\tworker=scoped:batch-a termination=exited:1 detail=witness row was red\n"
            ),
            "the journal must retain the derived verdict rather than only process completion: {persisted:?}"
        );
    }

    #[test]
    fn scoped_execution_authority_inherits_walk_roots_without_widening_subject_roots() {
        let walk_roots = vec!["dag".to_string(), "src/v2".to_string()];
        let subject_roots = vec!["dag".to_string(), "src/v1".to_string()];
        let authority_roots = scoped_execution_authority_source_roots(
            ScopedWitnessExecutionAuthority::InheritedWalkSourceRoots,
            &walk_roots,
        );

        assert_eq!(authority_roots, walk_roots);
        assert_eq!(subject_roots, ["dag", "src/v1"]);
        assert_ne!(
            authority_roots, subject_roots,
            "the selector authority must not be fused into the scoped witness subject envelope"
        );
    }

    #[test]
    fn floor_worker_terminal_crosses_receipt_with_exit_status() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-terminal-cross-{}",
            std::process::id()
        ));
        fs::write(&terminal, "completed\twalk said complete\n").unwrap();
        let status = Command::new("sh").arg("-c").arg("exit 7").status().unwrap();
        let observed = observe_floor_worker("scoped:batch-a", status, &terminal);
        let _ = fs::remove_file(&terminal);
        assert_eq!(observed.termination, ProcessTermination::Exited(7));
        assert!(matches!(
            observed.terminal_receipt,
            FloorWorkerTerminalReceipt::Observed(FloorWorkerTerminalReport::Completed(_))
        ));
        assert_eq!(floor_worker_observation_outcome(&observed).label, "failed");
        assert!(
            !floor_worker_succeeded(&observed),
            "a completed receipt cannot normalize a failing process exit"
        );
    }

    #[test]
    fn floor_worker_refusal_remains_distinct_from_failure() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-refused-{}",
            std::process::id()
        ));
        fs::write(&terminal, "refused\tFreshJobProcess has no realization\n").unwrap();
        let status = Command::new("sh").arg("-c").arg("exit 1").status().unwrap();
        let observed = observe_floor_worker("scoped:batch-a", status, &terminal);
        let _ = fs::remove_file(&terminal);
        let outcome = floor_worker_observation_outcome(&observed);
        assert_eq!(outcome.label, "refused");
        assert!(outcome.detail.contains("FreshJobProcess"));
        assert!(!floor_worker_succeeded(&observed));
    }

    #[cfg(unix)]
    #[test]
    fn floor_worker_signal_death_is_not_flattened_to_an_exit_code() {
        let terminal = std::env::temp_dir().join(format!(
            "claim-executor-worker-signaled-{}",
            std::process::id()
        ));
        let _ = fs::remove_file(&terminal);
        let status = Command::new("sh")
            .arg("-c")
            .arg("kill -TERM $$")
            .status()
            .unwrap();
        let observed = observe_floor_worker("scoped:batch-a", status, &terminal);
        assert!(matches!(
            observed.termination,
            ProcessTermination::Signaled(_)
        ));
        assert_eq!(
            floor_worker_observation_outcome(&observed).label,
            "died-without-terminal-receipt"
        );
        assert!(!floor_worker_succeeded(&observed));
    }

    #[test]
    fn floor_worker_signal_overrules_a_completed_terminal_report() {
        let observed = ObservedFloorWorker {
            worker: "scoped:batch-a".to_string(),
            termination: ProcessTermination::Signaled(9),
            terminal_receipt: FloorWorkerTerminalReceipt::Observed(
                FloorWorkerTerminalReport::Completed("walk said complete".to_string()),
            ),
        };
        let outcome = floor_worker_observation_outcome(&observed);
        assert_eq!(outcome.label, "failed");
        assert!(outcome.detail.contains("signal 9"));
        assert!(!floor_worker_succeeded(&observed));
    }
}

#[cfg(test)]
mod witness_walk_flags_tests {
    use super::witness_walk_flags;

    /// The scoped-child boundary deletion narrowed the roster walk. This pins that it narrowed
    /// ONLY the walk: a scoped child still executes rows, so it must still be handed the
    /// per-witness eval budget. RED if the two questions are ever collapsed back into one flag —
    /// the collapse is silent at the type level and would weaken a budget wall while reading as a
    /// pure scope narrowing.
    #[test]
    fn witness_walk_flags_split_the_two_questions() {
        let ordinary = witness_walk_flags(true, false);
        assert!(ordinary.executes_witness_rows);
        assert!(
            ordinary.schedules_discovery,
            "an ordinary worker carrying witness rows must still derive the roster"
        );

        let scoped = witness_walk_flags(true, true);
        assert!(
            scoped.executes_witness_rows,
            "a scoped child executes its frozen rows, so it must keep the eval budget"
        );
        assert!(
            !scoped.schedules_discovery,
            "a scoped child must never re-derive a roster its parent already froze"
        );

        // No rows at all: neither question is yes, for either role.
        for is_scoped in [false, true] {
            let empty = witness_walk_flags(false, is_scoped);
            assert!(!empty.executes_witness_rows);
            assert!(!empty.schedules_discovery);
        }
    }
}

#[cfg(test)]
mod scoped_execution_request_tests {
    use super::*;

    fn entry(name: &str) -> ScopedScheduleEntry {
        ScopedScheduleEntry {
            entry: format!("dag/test/claim/{name}_test.dag"),
            function: format!("{name}_holds"),
            witness_kind: "CorpusWitnessKind".to_string(),
        }
    }

    fn request(batch_id: &str, entries: Vec<ScopedScheduleEntry>) -> ScopedExecutionRequest {
        ScopedExecutionRequest {
            tested_commit: "a".repeat(40),
            tested_tree: "b".repeat(40),
            tool_identity: "tool-identity".to_string(),
            batch_id: batch_id.to_string(),
            source_roots: vec!["dag".to_string(), "src/v1".to_string()],
            source_roots_digest: "digest".to_string(),
            entries,
            scan_dirs: Vec::new(),
            execution_authority: ScopedWitnessExecutionAuthority::InheritedWalkSourceRoots,
            profile: ParsedRunnableProfile {
                provenance: ParsedProfileProvenance::Declared,
                heavy_whole_tree_resolve: false,
                spawns_host_compiler: false,
                memory: ParsedMemoryClass::Negligible,
                execution_mode: ExecutionMode::Hermetic,
            },
            clamp: ResolvedFloorBatchClamp {
                overhead_ms: 1,
                per_unit_ms: 2,
                authority: FloorBatchClampAuthority::ScopedBatchOwnedClamp {
                    batch_id: batch_id.to_string(),
                    module_path: "gunbc.ci_layer_roots".to_string(),
                    decl_name: "scoped_witness_batches".to_string(),
                },
            },
            process_isolation: ScopedProcessIsolation::SequentialChildProcess,
            fast_lane_eval_budget_ms: Some(5_000),
            ordinary_budget_ms: Some(60_000),
            batch_stop_policy: FloorBatchStopPolicy::StopBeforeDependents,
        }
    }

    /// EXACT IDENTITY TRANSPORT. The child executes what the parent froze — not a re-derivation of
    /// it — so the round trip must preserve the identity set exactly. RED if a field is dropped
    /// from the carrier or reordered into a different batch.
    #[test]
    fn round_trip_preserves_the_frozen_identity_set() {
        let original = request("v1_claim_scoped", vec![entry("alpha"), entry("beta")]);
        let wire = serde_json::to_string(&[original.clone()]).expect("serialize");
        let decoded: Vec<ScopedExecutionRequest> = serde_json::from_str(&wire).expect("decode");
        assert_eq!(decoded.len(), 1);

        let before: Vec<(String, String)> = original
            .entries
            .iter()
            .map(|e| (e.entry.clone(), e.function.clone()))
            .collect();
        let after: Vec<(String, String)> = decoded[0]
            .entries
            .iter()
            .map(|e| (e.entry.clone(), e.function.clone()))
            .collect();
        assert_eq!(
            before, after,
            "the frozen identity set must survive transport exactly"
        );
        assert_eq!(decoded[0].fast_lane_eval_budget_ms, Some(5_000));
        assert_eq!(decoded[0].ordinary_budget_ms, Some(60_000));

        // The runnable the child executes carries the same identities, in order.
        match decoded[0].to_runnable() {
            Runnable::ScopedWitnessBatch {
                entries, batch_id, ..
            } => {
                assert_eq!(batch_id, "v1_claim_scoped");
                let rebuilt: Vec<(String, String)> = entries
                    .iter()
                    .map(|e| (e.entry.clone(), e.function.clone()))
                    .collect();
                assert_eq!(rebuilt, before);
            }
            _ => panic!("expected a scoped witness batch"),
        }
    }

    /// PLANTED OMITTED IDENTITY. A request that lost one of its frozen rows must not silently
    /// execute the smaller set: the identity sets differ, and this control is what notices.
    #[test]
    fn planted_omitted_identity_is_visible_in_the_request() {
        let full = request("v1_claim_scoped", vec![entry("alpha"), entry("beta")]);
        let truncated = request("v1_claim_scoped", vec![entry("alpha")]);
        let full_ids: Vec<String> = full.entries.iter().map(|e| e.function.clone()).collect();
        let truncated_ids: Vec<String> = truncated
            .entries
            .iter()
            .map(|e| e.function.clone())
            .collect();
        assert_ne!(
            full_ids, truncated_ids,
            "an omitted frozen identity must be a visible difference, never an equal set"
        );
        assert_eq!(truncated_ids.len(), full_ids.len() - 1);
    }

    /// PLANTED WRONG SUBJECT. Selection alone is not verification: a request frozen against another
    /// commit, tree, or tool must refuse rather than execute against whatever is on disk. This
    /// drives the PRODUCTION refusal on each axis — an earlier revision asserted field inequality
    /// on hand-authored data, which restates the comparison instead of exercising it and would
    /// have stayed green if the loader stopped consulting one axis (review 51445). Observing the
    /// live subject is git work and stays with the floor; deciding on it is what refuses here.
    #[test]
    fn planted_wrong_subject_differs_on_every_axis() {
        let frozen = request("v1_claim_scoped", vec![entry("alpha")]);
        for (commit, tree, tool) in [
            (
                "c".repeat(40),
                frozen.tested_tree.clone(),
                frozen.tool_identity.clone(),
            ),
            (
                frozen.tested_commit.clone(),
                "d".repeat(40),
                frozen.tool_identity.clone(),
            ),
            (
                frozen.tested_commit.clone(),
                frozen.tested_tree.clone(),
                "other-tool".to_string(),
            ),
        ] {
            let refusal = refuse_subject_mismatch(&frozen, &commit, &tree, &tool)
                .expect_err("a subject perturbed on any axis must refuse");
            assert!(
                refusal.contains("frozen against a different subject")
                    && refusal.contains(&frozen.batch_id),
                "the refusal must say what it refused and for which batch: {refusal}"
            );
        }

        // The unperturbed subject must be ACCEPTED — otherwise the refusals above would pass for a
        // reason unrelated to what they claim (a check that refuses everything is not a check).
        refuse_subject_mismatch(
            &frozen,
            &frozen.tested_commit,
            &frozen.tested_tree,
            &frozen.tool_identity,
        )
        .expect("the subject it was frozen against must be accepted");
    }

    /// THE DEFECT TWO FULL CI FLOORS PAID FOR. `resolve_floor_batch_stop_policy` is read
    /// unconditionally by every worker, so a scoped child — which has no plan context by
    /// construction after this change — refused there AFTER the ordinary walk had already
    /// succeeded, and the coordinator reported it as "worker returned before producing a walk
    /// terminal receipt" because the refusal path exited without writing one.
    ///
    /// The wall is that every plan-derived value a child reads must travel ON the request. This
    /// pins the population: adding a plan read to the child's path without adding its field here
    /// fails, rather than being discovered by a 70-minute floor.
    #[test]
    fn every_plan_derived_value_the_child_reads_travels_on_the_request() {
        let frozen = request("v1_claim_scoped", vec![entry("alpha")]);

        // Each of these is read by a scoped child at execution time and is derived from the plan
        // the child no longer evaluates. Absence is not "default it" — it is unrepresentable.
        assert_eq!(frozen.fast_lane_eval_budget_ms, Some(5_000));
        assert_eq!(frozen.ordinary_budget_ms, Some(60_000));
        assert_eq!(
            frozen.batch_stop_policy,
            FloorBatchStopPolicy::StopBeforeDependents
        );

        // And they must survive the transport — a field the parent freezes but the JSON drops
        // puts the child right back where it was, reading a plan it does not have.
        let encoded = serde_json::to_string(&vec![frozen.clone()]).expect("serializes");
        let decoded: Vec<ScopedExecutionRequest> =
            serde_json::from_str(&encoded).expect("round-trips");
        assert_eq!(
            decoded[0].fast_lane_eval_budget_ms,
            frozen.fast_lane_eval_budget_ms
        );
        assert_eq!(decoded[0].ordinary_budget_ms, frozen.ordinary_budget_ms);
        assert_eq!(decoded[0].batch_stop_policy, frozen.batch_stop_policy);
    }

    /// cursor review 51430 found this: the manifest this carrier replaces refused a duplicate
    /// batch id on read, and the replacement only JSON-parsed. The coordinator spawns one child
    /// per published row, so a repeated id spawned parallel workers for one batch instead of
    /// stopping the line. This drives the production refusal, not a restatement of it.
    #[test]
    fn a_repeated_batch_id_refuses_before_anything_spawns() {
        let unique = vec![
            request("v1_claim_scoped", vec![entry("alpha")]),
            request("v1_claim_scoped_two", vec![entry("beta")]),
        ];
        assert!(
            refuse_duplicate_scoped_batch_ids(&unique, "fixture").is_ok(),
            "distinct batch ids must pass — otherwise the refusal below proves nothing"
        );

        // Same id, DIFFERENT populations: the contradiction a first-wins read would resolve by
        // silently picking one.
        let duplicated = vec![
            request("v1_claim_scoped", vec![entry("alpha")]),
            request("v1_claim_scoped", vec![entry("beta")]),
        ];
        let refusal = refuse_duplicate_scoped_batch_ids(&duplicated, "fixture")
            .expect_err("a repeated batch id must refuse");
        assert!(
            refusal.contains("duplicate batch id") && refusal.contains("v1_claim_scoped"),
            "the refusal must name what it refused and where: {refusal}"
        );
    }
}

/// Run git in the workspace and return trimmed stdout, or a refusal naming the command.
///
/// Kept when the mirror-drift gate was removed: the behavioral receipt resolves its own baseline
/// through it, so this is a shared helper rather than that gate's private one.
fn git_stdout(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .env("GIT_PAGER", "cat")
        .output()
        .map_err(|e| format!("spawn git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// BEHAVIORAL RECEIPT — selection and corpus-domain derivation
// ---------------------------------------------------------------------------
//
// WHAT THIS ANSWERS THAT NOTHING ELSE DOES. `--required-regen` proves the committed mirrors
// equal what the authority emits, and `--required-regen-fixed-point` proves the emit repeats.
// Neither ever COMPILES the emitted candidate, let alone runs it: the regen host spawns exactly
// rustfmt, rustfmt and git. So the whole promotion story rests on bytes — and DESIGN §7 says in
// as many words that a byte-identical fixed point is NOT the goal, because matching bytes forces
// the emitter to reproduce the seed's warts. The evidence it asks for instead is execution:
// the emitted module compiles and is behaviorally equivalent to the seed on a discriminating
// corpus.
//
// WHY THE NAIVE FORM OF THAT IS VACUOUS TODAY, and this is the whole reason the mode is shaped
// the way it is. After the mirror convergence every emitted candidate is byte-identical to its
// committed mirror (measured: 0 drifted of 129). Emitting both sides and diffing therefore
// compares a program against itself — a receipt that cannot fail is not a weak receipt, it is
// not a receipt. The question with content is the one every future authority edit raises: THE
// AUTHORITY CHANGED AND THE EMISSION CHANGED WITH IT — is the new module still behaviorally the
// same? That is exactly what byte-equality cannot answer, and it has content precisely when
// candidate and seed differ.
//
// SELECTION IS DEMAND-DIRECTED AND DERIVED. Two compiler builds per module across 129 modules is
// not a per-PR gate, it is a budget denominated in the corpus rather than in the change — the
// cost shape DESIGN §5 names, where the bill grows with the repository until it breaks. So the
// subject is the modules whose `.dag` AUTHORITY moved in this diff, and the mapping from an
// authority to its emitted mirror is read off the mirror's own authority header rather
// than from any authored roster. Nothing here can be forged by editing a list.
//
// AND IF THE SELECTION CANNOT BE COMPUTED IT REFUSES. It does not widen to the whole population.
// "I could not determine what changed" and "everything changed" are different states, and
// rendering the first as the second is the absorbing fallback — nothing is missed, so it looks
// safe, while the deficit's frequency drops to zero by construction and the cost lands on every
// future run.

/// Why a changed authority produced no receipt. Every arm is COUNTED and NAMED in the report:
/// a module that silently produces nothing is indistinguishable from a module that passed, and
/// that conflation is what makes an unexecuted receipt read as a green one.
///
/// EVERY ARM HAS A PRODUCER. An earlier revision declared three, adding `NoEmissionChange`
/// (authority moved but emission did not) and `CorpusNotDerivable`. Neither was ever constructed:
/// the first needs the emission this fragment does not yet compute, and the second was a second
/// name for `ModuleCorpusPlan::refused`, which already carries every refusal with the function
/// and type that caused it. A declared-but-unconstructed variant is vocabulary claiming a
/// distinction nothing draws, so both were deleted rather than carried until a producer appeared.
/// `NoEmissionChange` returns when the two-build differential lands and can actually observe it.
///
/// `NoFunctionHasACorpus` is the one that came back, WITH a producer and at a different grain --
/// see its own comment.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptExclusion {
    /// No emitted mirror in the generated population names this authority, under EITHER header
    /// convention the corpus uses (`MirrorIndex`). This is a fact rather than a lookup failure:
    /// the index refuses outright if any self-declared generated file is unindexable, so it
    /// cannot reach this arm while blind. Legitimate whenever an authority emits something that
    /// is not a Rust module mirror -- a workflow YAML, a fixture -- and named rather than
    /// skipped, because a changed authority that maps to nothing must be visible.
    NoEmittedMirror { module_path: String },

    /// The authority declares NO functions at all, so it has no behaviour that could diverge.
    ///
    /// SEPARATE FROM `NoFunctionHasACorpus` BECAUSE THEY ARE DIFFERENT STATES WITH DIFFERENT
    /// REMEDIES, and the arm that used to carry both said something false about this one. A
    /// module with twenty functions none of whose corpora this fragment can derive is a DEFICIT
    /// in the derivation -- it ranks, it names the types responsible, and closing them makes the
    /// module checkable. A module with zero functions is not a deficit at all: there is nothing
    /// to derive and never will be, so reporting it as `none of its 0 declared functions yields
    /// a call` sends a reader to fix a derivation that has no subject, and inflates the
    /// non-derivability population with rows no work can remove.
    NoFunctionDeclared { module_path: String },

    /// NOT ONE FUNCTION in this authority yields a call, so there is no corpus to compare -- and
    /// the arm carries EVERY declared function with the cause that stopped it, because the
    /// deficit lives at the function and a module-level "nothing derived" cannot be acted on.
    ///
    /// WHY THIS IS AN EXCLUSION AND NOT A REFUSAL, which is the whole of the change that
    /// introduced it. The verdict used to be decided by a FILE-LEVEL count crossing zero: a
    /// module where 1 of 20 functions derived ran and reported EQUIVALENT, and a module where 0
    /// of 20 derived hard-failed required CI. One deficit -- functions whose corpus this fragment
    /// cannot derive -- and two opposite verdicts, separated by nothing but where the count
    /// happened to land. Worse, the red had no closing move: an author cannot make
    /// `List<T>` enumerable to get their diff through, so the only way past the gate was to stop
    /// touching the authority. A gate whose sole closing move does not exist does not enforce,
    /// it launders (DESIGN.md, the fixed-point repair).
    ///
    /// So non-derivability is reported at the grain it occurs at -- per function, typed, located,
    /// counted -- and a module with no covered function is EXCLUDED from a differential it can
    /// never take, exactly as `NoEmittedMirror` is. It is not silence: the count of uncovered
    /// functions is printed on every run beside the module count, so the deficit stays rankable
    /// rather than having its frequency zeroed.
    NoFunctionHasACorpus {
        module_path: String,
        /// `(function, why it yields no call)`, one row per declared function.
        uncovered: Vec<(String, RefusalCause)>,
    },
}

/// The domain a corpus actually enumerated, reported as a DERIVED FACT rather than a label.
///
/// Both arms are COVERAGE CLAIMS. There is deliberately no "bounded sample" arm: an earlier
/// revision enumerated `Int` over [-2,2] and `List` to length 3 and reported the result beside
/// genuine exhaustive coverage. That arm is DELETED rather than widened, because a window is not
/// a weaker proof of the same thing — it is a receipt that USUALLY cannot fail, and a
/// usually-passing receipt reports as done while a refusal is counted and ranks for work. The
/// window was also measurably absurd in place: it enumerated a lower-hex-digit predicate over
/// five values containing no hex digit at all.
#[derive(Debug, Clone, PartialEq)]
enum EnumeratedDomain {
    /// Finite closed domain, fully covered: closed nullary enums, Bool, and records over them.
    /// Enumeration IS the domain.
    Exhaustive { cardinality: usize },
    /// Infinite domain, fully covered ANYWAY, because the function cannot distinguish the values
    /// inside a class.
    ///
    /// The argument, in full, because the claim is only as good as it: if a parameter's every
    /// occurrence in the body is an operand of a comparison against an integer literal, then the
    /// literals cut the integers into finitely many classes -- the points themselves and the open
    /// gaps between them -- and every comparison in the function yields the same answer for any
    /// two values drawn from one class. The function's behaviour is therefore constant within a
    /// class, so one representative per class covers the type. This is exhaustive in the same
    /// sense as a closed enum, NOT an approximation of it.
    ///
    /// The premise is what makes it sound, and it is checked rather than assumed: the moment the
    /// parameter is returned, embedded in a record, passed to another function, or arithmetically
    /// combined, its VALUE reaches the output and two members of one class stop agreeing --
    /// `fn shard_count_positive(n: Int) -> Int { if n <= 0 { 1 } else { n } }` returns 5 for 5 and
    /// 7 for 7, both in the class `n > 0`. So any occurrence that is not a literal comparison
    /// REFUSES the whole parameter. The check is conservative in the safe direction: an
    /// occurrence it cannot classify refuses rather than being assumed harmless.
    ExhaustiveOverDerivedPartition {
        cardinality: usize,
        partition: String,
    },
}

/// One parameter's domain as the ACTUAL VALUES, rendered as Rust expressions against the emitted
/// mirror, plus how that domain was established.
///
/// The count is `values.len()`. It is not carried separately, because a cardinality computed
/// beside an enumeration is a second producer of one fact: the two can disagree, and the one that
/// gets reported is the one that never ran. An earlier revision derived only the count, which is
/// why the corpus could be described in the report but never executed.
#[derive(Debug, Clone)]
struct ParameterDomain {
    values: Vec<String>,
    partition: Option<String>,
}

impl EnumeratedDomain {
    fn report(&self) -> String {
        match self {
            EnumeratedDomain::Exhaustive { cardinality } => {
                format!("exhaustive(|domain|={cardinality})")
            }
            EnumeratedDomain::ExhaustiveOverDerivedPartition {
                cardinality,
                partition,
            } => format!("exhaustive-over-partition(|reps|={cardinality}, {partition})"),
        }
    }
}

/// A type declared by the authority, in the only two shapes the fragment can enumerate.
#[derive(Debug, Clone)]
enum DagTypeDecl {
    /// `type AxisGoal = HigherIsBetter | LowerIsBetter` — a closed coproduct of NULLARY variants.
    /// A variant carrying a payload is deliberately NOT this: it would need its payload's domain
    /// enumerated too, and admitting it here without doing that would silently under-cover.
    ClosedNullaryEnum { variants: Vec<String> },
    /// `type DominanceTally { saw_better: Bool, saw_worse: Bool }` — a record over named fields.
    Record { fields: Vec<(String, String)> },
    /// A closed coproduct whose variants carry payloads. NOT enumerable here — each payload needs
    /// its own domain — but recorded as its own kind rather than left unparsed, because the two
    /// produce the same refusal COUNT and completely different refusal MEANINGS. Reporting "not a
    /// closed type declared by this authority" about a type that is declared and IS closed sends
    /// the reader to close something already closed.
    PayloadCoproduct { variant_count: usize },
    /// DECLARED, but carrying no constructor set this fragment can enumerate: an opaque type, an
    /// alias, a form the reader does not model, or a record whose field types it could not read.
    ///
    /// This arm exists so those declarations are REGISTERED rather than dropped. Dropped, they
    /// were reported as "no module in the corpus declares this type" -- a positive claim about the
    /// corpus produced by the reader's own silence, which is the empty-observation narrow with a
    /// wildcard for a cause. Registered, they refuse honestly and rank at zero work.
    DeclaredNotEnumerable { form: String },
}

/// Parse one `.dag` source through the GRAMMAR-OWNED parser and return its module node.
///
/// This replaces a hand-rolled line reader, and the replacement is the point rather than a
/// tidying. The line reader recognised only declarations that fit on one line: measured across
/// the corpus that was 989 of 8796, so it was blind to 88% of the type declarations in the
/// repository, and every one of them was then refused as "not a closed type declared by this
/// authority" -- false, since they are declared and many are closed. Nothing unsound was claimed,
/// because the refusal did stop the line; but a refusal's job beyond stopping is to RANK, and
/// that one sent the reader to close types that were already closed.
///
/// The deeper defect is that a hand-rolled reader is a SECOND PARSER for `.dag` standing beside
/// the real one, which is the single-authority violation in its plainest form: it will be wrong
/// again whenever the grammar moves, and wrong SILENTLY, because a line reader cannot distinguish
/// "did not match" from "is not there". `parse_with_table` can: it returns an error arm, so a
/// source it cannot read REFUSES instead of yielding zero declarations.
fn parse_dag_module_node(
    file: &str,
    source: &str,
) -> Result<Rc<v1_compiler::v1_std_core::Node>, String> {
    use v1_compiler::v1_compiler_parse::parse_with_table;
    use v1_compiler::v1_compiler_tokenize::tokenize;
    use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table, NewlineIndex};

    // The map is built with the runtime's own constructors rather than a `std::HashMap`: the
    // parser's source-index map is an `im::HashMap`, and reaching for the concrete type here
    // would be this file asserting a representation the parser owns.
    let index = build_newline_index(file.to_string(), source.to_string());
    let indices = v1_compiler::v1_rt::rc_map_insert(
        v1_compiler::v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
        file.to_string(),
        index,
    );
    let parsed = parse_with_table(
        tokenize(source.to_string(), file.to_string()),
        indices,
        empty_intern_table(),
    );
    if let Some(err) = parsed.result.error.clone() {
        return Err(format!(
            "{file}: the grammar refused this source: {:?}",
            err.diagnostic
        ));
    }
    parsed
        .result
        .module
        .clone()
        .ok_or_else(|| format!("{file}: the parse produced neither a module nor an error"))
}

/// Render a type-annotation node back to the name the corpus writes, generics included.
fn type_text(node: &v1_compiler::v1_std_core::Node) -> String {
    // Generic ARGUMENTS are children of the type node. Reading `params` instead rendered
    // `List<AxisComparison>` as bare `List`, which then failed the `List<` prefix test and fell
    // through to "not a closed type declared by this authority" — a refusal naming the wrong
    // cause for an entire population, and the reason the first corpus histogram had no List row
    // at all despite lists being the second-largest blocker.
    let args: Vec<String> = node.children.iter().map(|c| type_text(c)).collect();
    if args.is_empty() {
        return node.name.clone();
    }
    format!("{}<{}>", node.name, args.join(", "))
}

/// Read every type declaration off a parsed module node.
///
/// A declaration's SHAPE is read from the substrate's own connective rather than from
/// punctuation: `Disj` is a coproduct and `Conj` is a record. That is why this reader does not
/// care whether the author wrote the body on one line or ten -- the distinction the line reader
/// tripped on does not exist at this layer, which is the strongest evidence that the layer is the
/// right one.
fn type_decls_from_module(
    module: &v1_compiler::v1_std_core::Node,
) -> std::collections::HashMap<String, DagTypeDecl> {
    use v1_compiler::v1_std_core::Connective;
    let mut out = std::collections::HashMap::new();
    for decl in module.children.iter() {
        // WHICH CHILDREN ARE TYPE DECLARATIONS AT ALL -- measured, not named from memory. The
        // grouped shape census over one module's children is unambiguous:
        //
        //   is_type=true   conn=Conj/Disj    body=false  params=0  inferred=false
        //   is_type=false  conn=NoConnective body=true   (compare_int, tally_verdict, no_names)
        //
        // A function or a data row carries a BODY; a type declaration does not. Without this the
        // arm has no notion of its own subject, which is the single cause of BOTH failures here:
        // filtering implicitly on two connectives was too narrow (70% of declarations dropped),
        // and filtering on nothing was too wide (every function registered as a type, 6509 read
        // against 957 authored). Neither was a bug in the filter; both were its absence.
        if decl.body.is_some() {
            continue;
        }
        match decl.connective {
            Connective::Disj => {
                let variants: Vec<String> = decl.children.iter().map(|v| v.name.clone()).collect();
                if variants.is_empty() {
                    continue;
                }
                // A variant with no fields of its own is nullary. A variant carrying a payload
                // needs that payload's domain enumerated too, so the whole coproduct is recorded
                // as payload-carrying rather than being enumerated one-value-per-variant, which
                // would under-cover silently.
                let all_nullary = decl.children.iter().all(|v| v.children.is_empty());
                if all_nullary {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::ClosedNullaryEnum { variants },
                    );
                } else {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::PayloadCoproduct {
                            variant_count: variants.len(),
                        },
                    );
                }
            }
            Connective::Conj => {
                // A field's TYPE is its child, exactly as a parameter's is. Reading
                // `type_annotation` here yielded no fields for every record, so each one was
                // dropped from the type environment and later refused as "not a closed type
                // declared by this authority" — false, and pointing at the wrong repair.
                // A RECORD FIELD'S TYPE IS IN `inferred`, NOT IN `children`. Measured, not
                // assumed -- the fourth node-shape fact this reader needed and the third one an
                // assumption got wrong. A PARAMETER's type is its child, which is true and stays
                // true; reading a field the same way returned nothing for every field of every
                // record, so `filter_map` emptied the list and the guard below dropped the whole
                // declaration. std.pareto read 6 of 13 types and ZERO of its 7 records.
                let mut fields: Vec<(String, String)> = Vec::new();
                let mut unreadable: Vec<String> = Vec::new();
                for f in decl.children.iter() {
                    match f.inferred.as_ref().map(|i| i.as_ref()) {
                        Some(v1_compiler::v1_std_core::InferredNode::Resolved { node }) => {
                            fields.push((f.name.clone(), type_text(node)))
                        }
                        _ => unreadable.push(f.name.clone()),
                    }
                }
                // A PARTIALLY READ RECORD IS NOT A RECORD. Enumerating the fields that happened
                // to resolve would produce a Cartesian product over a SUBSET of the record's
                // fields -- a constructor expression missing fields, which does not compile, and
                // a domain claim that is simply false. So the whole declaration refuses, naming
                // the fields responsible, rather than silently narrowing itself.
                if unreadable.is_empty() && !fields.is_empty() {
                    out.insert(decl.name.clone(), DagTypeDecl::Record { fields });
                } else {
                    out.insert(
                        decl.name.clone(),
                        DagTypeDecl::DeclaredNotEnumerable {
                            form: format!(
                                "record with {} field(s) whose declared type the reader could not \
                                 read: {}",
                                unreadable.len(),
                                unreadable.join(", ")
                            ),
                        },
                    );
                }
            }
            // NO WILDCARD. The declaration vocabulary is CLOSED, and a catch-all here discards
            // exactly the guarantee that closure exists to provide -- silently, in the seed, where
            // nothing enforces the exhaustiveness the substrate would. Every remaining form is
            // registered as declared-but-not-enumerable so it refuses by name instead of becoming
            // a false claim that the corpus does not declare it. Opaque types and aliases are the
            // bulk of this arm and are genuinely not enumerable; that answer is now SAID rather
            // than inferred from an absence.
            other => {
                out.insert(
                    decl.name.clone(),
                    DagTypeDecl::DeclaredNotEnumerable {
                        form: format!(
                            "declaration form {other:?} carries no enumerable constructor set"
                        ),
                    },
                );
            }
        }
    }
    out
}

/// The classes an `Int` parameter's own comparisons cut the integers into, with one
/// representative per class.
#[derive(Debug, Clone, PartialEq)]
struct IntPartition {
    /// The integer literals this parameter is compared against, sorted and deduplicated.
    literals: Vec<i64>,
    /// One value per equivalence class. For literals L the classes are each point `l` and each
    /// open gap between consecutive points, plus the two unbounded ends; the representative set
    /// below hits every non-empty one of them.
    representatives: Vec<i64>,
}

impl IntPartition {
    /// Representatives are `{l-1, l, l+1}` over every literal `l`.
    ///
    /// That this covers every class is a three-case check, not an approximation: the point class
    /// `{l}` is hit by `l`; the gap `(l_i, l_{i+1})` is hit by `l_i + 1` unless the gap is empty
    /// (`l_{i+1} == l_i + 1`); and the two unbounded ends are hit by `min-1` and `max+1`. The
    /// `±1` values are also exactly where an off-by-one divergence lives, which is why they are
    /// taken rather than an arbitrary interior point.
    fn from_literals(mut literals: Vec<i64>) -> IntPartition {
        literals.sort_unstable();
        literals.dedup();
        let mut reps: Vec<i64> = Vec::new();
        for l in &literals {
            for r in [l.saturating_sub(1), *l, l.saturating_add(1)] {
                if !reps.contains(&r) {
                    reps.push(r);
                }
            }
        }
        // No literals means the parameter is never compared against one. Reached only when it is
        // never mentioned at all -- any other use refuses upstream -- so the function ignores it
        // and a single arbitrary value covers the whole type.
        if reps.is_empty() {
            reps.push(0);
        }
        reps.sort_unstable();
        IntPartition {
            literals,
            representatives: reps,
        }
    }

    fn describe(&self) -> String {
        let lits = if self.literals.is_empty() {
            "unused by the body".to_string()
        } else {
            format!(
                "literals {{{}}}",
                self.literals
                    .iter()
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        format!(
            "{lits}, reps {{{}}}",
            self.representatives
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

/// Derive the partition for one `Int` parameter from the BODY NODE that uses it, or REFUSE
/// naming the occurrence that defeated it.
///
/// This is the whole soundness argument in code. Every occurrence of the parameter must be an
/// operand of a comparison whose other operand is an integer literal. Everything else -- being
/// returned, passed as an argument, added, having a field read off it -- lets the parameter's
/// VALUE reach the output, at which point two members of one class stop agreeing and the
/// partition claim is false. `fn shard_count_positive(n: Int) -> Int { if n <= 0 { 1 } else { n } }`
/// compares `n` against a literal AND returns it, so it returns 5 for 5 and 7 for 7 while both
/// sit in the class `n > 0`. That function refuses here.
///
/// IT WALKS THE PARSED TREE, NOT TEXT. An earlier revision lexed the body itself, and carried the
/// apparatus that implies: a hand tokenizer, a whitelist of what may abut a literal operand so
/// that `a + 1 < n` refused instead of reading `1`, comment stripping so an annotation naming the
/// parameter was not read as a use of it, and a lambda-parameter scan because it could not model
/// scope. Every one of those was a workaround for not having the parse, and every one is deleted:
/// `ExprBinOp` already carries its operator, `ExprLiteral` already holds a typed `LitInt`, and a
/// rebound name is simply a different node. The tell that the old version was in the wrong layer
/// is that its bug -- misreading `a - 1 < n` -- cannot be expressed in this one.
/// WHY a parameter's domain could not be derived -- as a typed cause, not a sentence.
///
/// The census ranks refusals to decide what to ground next, and ranking needs the SUBJECT: the
/// type, or the class of obstacle. The first revision carried only a formatted message and ranked
/// on it, which put `x (used outside a literal comparison...)` at the top with 1500 -- every
/// parameter in the corpus that happens to be named `x`, collapsed into one row that names no
/// type and no work. A parameter name is not a unit of work; the string was doing double duty as
/// an identity and as prose, and it was wrong at the identity job.
///
/// `describe()` is derived from this, so the sentence and the ranking key cannot disagree.
#[derive(Clone, PartialEq, Eq, Hash)]
enum RefusalCause {
    UnboundedString {
        ty: String,
    },
    UnboundedSequence {
        ty: String,
    },
    PayloadCoproduct {
        ty: String,
        variants: usize,
    },
    DeclaredNotEnumerable {
        ty: String,
        form: String,
    },
    /// The type is declared SOMEWHERE in the corpus but is not visible from the module under
    /// plan -- an import-closure gap in the reader, not a property of the type.
    TypeNotVisibleHere {
        ty: String,
    },
    /// No module in the corpus declares this type. Genuinely outside what the authority carries.
    TypeNotDeclaredAnywhere {
        ty: String,
    },
    ProductTooLarge {
        ty: String,
    },
    NestedTooDeep {
        ty: String,
    },
    /// The Int class. Keyed WITHOUT the parameter name -- the name is in the message for locating
    /// it, never in the identity, or one class fragments into as many rows as there are spellings.
    IntValueEscapesComparison {
        param: String,
    },
    IntComparedToNonLiteral {
        param: String,
    },
    IntThroughContainer,
    IntNoBody {
        param: String,
    },
    TupleBudgetExceeded {
        param: String,
    },
    /// Every parameter's domain derived, and the product of them contains NO tuple -- so the
    /// function yields no call while looking, in a count of derivable functions, exactly like one
    /// that yields a thousand.
    ///
    /// COUNTED SEPARATELY FROM THE NEVER-DERIVABLE ONES ON PURPOSE: the remedies differ. A
    /// function refused for `List<T>` needs a length partition; a function with an empty product
    /// needs the enumerator that produced an empty value set fixed. Summing them would name one
    /// piece of work where there are two.
    ///
    /// REACHABILITY, STATED RATHER THAN IMPLIED: no enumerator in this fragment returns an empty
    /// value set today (a zero-variant coproduct is dropped before it becomes a `DagTypeDecl`, and
    /// a zero-field record enumerates to the one empty literal), so there is no live specimen in
    /// the corpus. The producer is the classification in `function_grain_coverage`, which is
    /// executed against this state by `empty_derived_domain_is_uncovered_not_covered`. It exists
    /// because a zero that reads as success is the exact failure this whole arm closes: a
    /// function counted as covered while contributing nothing to the transcript.
    EmptyDerivedDomain,
    /// A refusal reached through a record field, carrying the field path for locating and the
    /// UNDERLYING cause for ranking -- grounding the inner type unlocks the outer record too.
    ViaField {
        ty: String,
        field: String,
        inner: Box<RefusalCause>,
    },
}

impl RefusalCause {
    /// The subject the work would be done to. Ranking key: a `ViaField` ranks as its inner cause,
    /// because the fix is the inner type and counting the wrapper separately would split one
    /// piece of work across as many rows as there are records that embed it.
    fn subject(&self) -> String {
        match self {
            RefusalCause::UnboundedString { ty }
            | RefusalCause::UnboundedSequence { ty }
            | RefusalCause::PayloadCoproduct { ty, .. }
            | RefusalCause::DeclaredNotEnumerable { ty, .. }
            | RefusalCause::ProductTooLarge { ty }
            | RefusalCause::NestedTooDeep { ty } => ty.clone(),
            // THE KIND IS PART OF THE KEY for these two, because the two name different work and
            // the first revision of the split keyed on the bare type name -- so the ranked list
            // printed exactly what it printed before the split, and the whole correction was
            // invisible in its own output. `declared_anywhere` is corpus-global, so a given type
            // falls entirely into one bucket; the tag is therefore stable per type, not a source
            // of fragmentation.
            RefusalCause::TypeNotVisibleHere { ty } => {
                format!("{ty} [declared in corpus, NOT VISIBLE to the reader]")
            }
            RefusalCause::TypeNotDeclaredAnywhere { ty } => {
                format!("{ty} [undeclared anywhere in corpus]")
            }
            RefusalCause::IntValueEscapesComparison { .. } => {
                "Int (value escapes literal comparison)".to_string()
            }
            RefusalCause::IntComparedToNonLiteral { .. } => {
                "Int (compared against a non-literal)".to_string()
            }
            RefusalCause::IntThroughContainer => "Int (reached through a container)".to_string(),
            RefusalCause::IntNoBody { .. } => "Int (no attached body node)".to_string(),
            RefusalCause::TupleBudgetExceeded { .. } => {
                format!("(combination exceeds {MAX_TUPLES_PER_FUNCTION} tuples)")
            }
            RefusalCause::EmptyDerivedDomain => "(derived domain contains no values)".to_string(),
            RefusalCause::ViaField { inner, .. } => inner.subject(),
        }
    }

    fn describe(&self) -> String {
        match self {
            RefusalCause::UnboundedString { ty } => format!("{ty} (unbounded String domain)"),
            RefusalCause::UnboundedSequence { ty } => {
                format!("{ty} (unbounded sequence length; a length partition is not derived)")
            }
            RefusalCause::PayloadCoproduct { ty, variants } => format!(
                "{ty} (closed coproduct, but {variants} variants carry payloads whose domains are \
                 not enumerated)"
            ),
            // THESE TWO WERE ONE ARM, AND THE ONE ARM MISDIRECTED. It read "not a closed type
            // declared by this authority", which parses as "declared, but not closed" -- while
            // the branch is reached ONLY when the type is not in the type environment at all.
            // Node topped the corpus ranking at 798 under that label and was read, by me, as the
            // one big groundable item. It is nothing of the sort: it is a 20-field record with an
            // unbounded String, a recursive List<Node>, and self-reference. Splitting the arm is
            // what makes the difference between "my reader cannot see it" and "the corpus does
            // not have it" reportable, and only the first is work anyone can do.
            RefusalCause::DeclaredNotEnumerable { ty, form } => {
                format!("{ty} (declared, but not enumerable: {form})")
            }
            RefusalCause::TypeNotVisibleHere { ty } => format!(
                "{ty} (declared elsewhere in the corpus but not visible from this module -- an \
                 import-closure gap in the reader, not a property of the type)"
            ),
            RefusalCause::TypeNotDeclaredAnywhere { ty } => {
                format!("{ty} (no module in the corpus declares this type)")
            }
            RefusalCause::ProductTooLarge { ty } => format!(
                "{ty} (record product exceeds {MAX_TUPLES_PER_FUNCTION} values; refusing rather \
                 than sampling)"
            ),
            RefusalCause::NestedTooDeep { ty } => {
                format!("{ty} (nesting deeper than the fragment enumerates)")
            }
            RefusalCause::IntValueEscapesComparison { param } => format!(
                "{param} (used outside a literal comparison, so its value reaches the result and \
                 members of one class need not agree)"
            ),
            RefusalCause::IntComparedToNonLiteral { param } => format!(
                "{param} (compared against a non-literal; the partition is derived from literals, \
                 and a comparison against another parameter or a call would need a joint partition \
                 this fragment does not derive)"
            ),
            RefusalCause::IntThroughContainer => {
                "Int (reached through a container; a partition is \
                 derived from a parameter's own comparisons and does not follow into a field or \
                 element)"
                    .to_string()
            }
            RefusalCause::IntNoBody { param } => {
                format!("{param} (Int parameter on a function with no attached body node)")
            }
            RefusalCause::TupleBudgetExceeded { param } => format!(
                "{param} (corpus exceeds {MAX_TUPLES_PER_FUNCTION} tuples; refusing rather than \
                 sampling)"
            ),
            RefusalCause::EmptyDerivedDomain => {
                "every parameter derived, but their product contains no tuple, so this function \
                 yields no call at all"
                    .to_string()
            }
            RefusalCause::ViaField { ty, field, inner } => {
                format!("{ty}.{field}: {}", inner.describe())
            }
        }
    }
}

/// `Debug` IS `describe()`. A derived `Debug` would print the variant name, which is the one
/// rendering of a refusal that cannot be acted on -- and a test asserting on it would pin the
/// spelling of a Rust identifier rather than the fact.
impl std::fmt::Debug for RefusalCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.describe())
    }
}

fn derive_int_partition(
    param: &str,
    body: &v1_compiler::v1_std_core::Node,
) -> Result<IntPartition, RefusalCause> {
    let mut literals = Vec::new();
    visit_int_param_occurrences(param, body, None, &mut literals)?;
    Ok(IntPartition::from_literals(literals))
}

/// Recurse the body, carrying the enclosing comparison so an occurrence can be judged in context.
///
/// `enclosing` is `Some((op_is_comparison, sibling))` when this node is a direct operand of a
/// binary operation. An occurrence of the parameter is admitted ONLY when that context is a
/// comparison and the sibling is an integer literal.
fn visit_int_param_occurrences(
    param: &str,
    node: &v1_compiler::v1_std_core::Node,
    enclosing: Option<(bool, &v1_compiler::v1_std_core::Node)>,
    literals: &mut Vec<i64>,
) -> Result<(), RefusalCause> {
    use v1_compiler::std_syntax::BinOp;
    use v1_compiler::v1_std_core::ExprData;

    if matches!(node.expr_data.as_ref(), ExprData::ExprVar { .. }) && node.name == param {
        return match enclosing {
            Some((true, sibling)) => match int_literal_of(sibling) {
                Some(v) => {
                    literals.push(v);
                    Ok(())
                }
                None => Err(RefusalCause::IntComparedToNonLiteral {
                    param: param.to_string(),
                }),
            },
            _ => Err(RefusalCause::IntValueEscapesComparison {
                param: param.to_string(),
            }),
        };
    }

    let comparison = match node.expr_data.as_ref() {
        ExprData::ExprBinOp { op, .. } => Some(matches!(
            op,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge
        )),
        _ => None,
    };
    // A binary operation's two operands are each other's sibling; every other node's children
    // are visited with NO enclosing comparison, which is what makes the default a refusal.
    if let (Some(is_cmp), 2) = (comparison, node.children.len()) {
        let lhs = &node.children[0];
        let rhs = &node.children[1];
        visit_int_param_occurrences(param, lhs, Some((is_cmp, rhs)), literals)?;
        visit_int_param_occurrences(param, rhs, Some((is_cmp, lhs)), literals)?;
        return Ok(());
    }
    for child in node.children.iter() {
        visit_int_param_occurrences(param, child, None, literals)?;
    }
    for p in node.params.iter() {
        visit_int_param_occurrences(param, p, None, literals)?;
    }
    if let Some(b) = node.body.as_ref() {
        visit_int_param_occurrences(param, b, None, literals)?;
    }
    Ok(())
}

fn int_literal_of(node: &v1_compiler::v1_std_core::Node) -> Option<i64> {
    use v1_compiler::std_syntax::LiteralValue;
    use v1_compiler::v1_std_core::ExprData;
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitInt { value } => Some(*value),
            _ => None,
        },
        _ => None,
    }
}

/// The cap on one function's corpus. Exceeding it REFUSES rather than sampling: a receipt that
/// ran a subset while reporting the whole is the fabricated-plausible-output failure, and the
/// cheapest way to get one is a Cartesian product nobody bounded.
const MAX_TUPLES_PER_FUNCTION: usize = 4096;

/// Enumerate a parameter's domain as Rust expressions, or REFUSE naming the type that defeated it.
///
/// `module_alias` is the emitted mirror's module, so every constructor is written against the
/// artifact under test rather than against a guess about where a type lives.
fn enumerate_parameter_values(
    ty: &str,
    types: &std::collections::HashMap<String, DagTypeDecl>,
    declared_anywhere: &std::collections::HashSet<String>,
    depth: usize,
    int_partition: Option<&IntPartition>,
    module_alias: &str,
) -> Result<ParameterDomain, RefusalCause> {
    if depth > 4 {
        return Err(RefusalCause::NestedTooDeep { ty: ty.to_string() });
    }
    let ty = ty.trim();
    if ty == "Bool" {
        return Ok(ParameterDomain {
            values: vec!["false".to_string(), "true".to_string()],
            partition: None,
        });
    }
    if ty == "Int" {
        return match int_partition {
            Some(p) => Ok(ParameterDomain {
                values: p
                    .representatives
                    .iter()
                    .map(|r| format!("{r}i64"))
                    .collect(),
                partition: Some(p.describe()),
            }),
            None => Err(RefusalCause::IntThroughContainer),
        };
    }
    if ty.starts_with("List<") {
        return Err(RefusalCause::UnboundedSequence { ty: ty.to_string() });
    }
    match types.get(ty) {
        Some(DagTypeDecl::ClosedNullaryEnum { variants }) => Ok(ParameterDomain {
            values: variants
                .iter()
                .map(|v| format!("{module_alias}::{ty}::{v}"))
                .collect(),
            partition: None,
        }),
        Some(DagTypeDecl::PayloadCoproduct { variant_count }) => {
            Err(RefusalCause::PayloadCoproduct {
                ty: ty.to_string(),
                variants: *variant_count,
            })
        }
        Some(DagTypeDecl::DeclaredNotEnumerable { form }) => {
            Err(RefusalCause::DeclaredNotEnumerable {
                ty: ty.to_string(),
                form: form.clone(),
            })
        }
        Some(DagTypeDecl::Record { fields }) => {
            // The record's own domain is the Cartesian product of its fields'. Built as literal
            // constructor expressions so the driver names every field, which is also what makes a
            // field added upstream a COMPILE error in the driver rather than a silent default.
            let mut acc: Vec<Vec<(String, String)>> = vec![Vec::new()];
            let mut partitioned = Vec::new();
            for (fname, fty) in fields {
                let d = enumerate_parameter_values(
                    fty,
                    types,
                    declared_anywhere,
                    depth + 1,
                    None,
                    module_alias,
                )
                .map_err(|e| RefusalCause::ViaField {
                    ty: ty.to_string(),
                    field: fname.clone(),
                    inner: Box::new(e),
                })?;
                if let Some(pt) = d.partition.clone() {
                    partitioned.push(format!("{fname}: {pt}"));
                }
                let mut next = Vec::new();
                for prefix in &acc {
                    for v in &d.values {
                        if next.len() > MAX_TUPLES_PER_FUNCTION {
                            return Err(RefusalCause::ProductTooLarge { ty: ty.to_string() });
                        }
                        let mut row = prefix.clone();
                        row.push((fname.clone(), v.clone()));
                        next.push(row);
                    }
                }
                acc = next;
            }
            Ok(ParameterDomain {
                values: acc
                    .into_iter()
                    .map(|row| {
                        let inner: Vec<String> =
                            row.into_iter().map(|(f, v)| format!("{f}: {v}")).collect();
                        format!("{module_alias}::{ty} {{ {} }}", inner.join(", "))
                    })
                    .collect(),
                partition: if partitioned.is_empty() {
                    None
                } else {
                    Some(partitioned.join("; "))
                },
            })
        }
        None => Err(if ty == "String" || ty == "NonEmptyStr" {
            RefusalCause::UnboundedString { ty: ty.to_string() }
        } else if declared_anywhere.contains(ty) {
            RefusalCause::TypeNotVisibleHere { ty: ty.to_string() }
        } else {
            RefusalCause::TypeNotDeclaredAnywhere { ty: ty.to_string() }
        }),
    }
}

/// One function as the authority declares it: its parameters and the body node that uses them.
///
/// The body is carried because an `Int` parameter's domain is a fact about HOW THIS FUNCTION USES
/// IT, not about the type: `Int` has no finite domain, but the comparisons a body performs cut it
/// into finitely many classes. A signature-only planner cannot ask that question at all.
#[derive(Debug, Clone)]
struct DagFnSignature {
    name: String,
    params: Vec<(String, String)>,
    body: Option<Rc<v1_compiler::v1_std_core::Node>>,
}

/// Read every function off a parsed module node.
///
/// The third and last hand-rolled reader to go. Its predecessor matched `fn ` at the start of a
/// line and then took the parameter list up to the first `)` on that same line, so a signature
/// spanning lines produced no entry at all -- 14 of `v1.compiler.emit_rust`'s 631 went missing
/// that way, and only a declared-versus-parsed counter made the gap visible rather than reading
/// as a module with a smaller surface. The parser has already separated a declaration's
/// parameters from its body, so none of that arises here.
///
/// That same counter then caught THIS function selecting on the wrong discriminator, which is why
/// it is kept rather than retired once the parser owned the read: two independent readers of one
/// fact disagreeing is the only cheap signal that one of them is wrong.
fn fn_signatures_from_module(module: &v1_compiler::v1_std_core::Node) -> Vec<DagFnSignature> {
    let mut out = Vec::new();
    for decl in module.children.iter() {
        // A declaration is a FUNCTION exactly when it carries a body. Measured against the live
        // tree rather than assumed: `std.pareto`'s items report `compare_int` and friends as
        // `conn=NoConnective children=0 params=N body=true`, while its types report `Disj` with
        // variants or `Conj` with fields and no body. `Connective::Arrow` — which an earlier
        // revision selected on — marks a `Callable` TYPE EXPRESSION, not a declaration, so that
        // filter matched nothing and every module reported `parsed=0`.
        // A FUNCTION carries a body AND a resolved return type in `inferred`. A `data` row also
        // carries a body, which is why selecting on the body alone over-counted: `std.pareto`
        // reported 36 parsed against 33 authored `fn` lines, its three `data` rows swept in.
        // Measured discriminator — `data no_names: List<NonEmptyStr> = []` reports
        // `ta=Some("List") inf=none`, while every function reports `ta=None inf=Resolved{..}`:
        // the declared type of a constant lives in `type_annotation`, a function's return type in
        // `inferred`.
        let (Some(_), Some(_)) = (decl.body.as_ref(), decl.inferred.as_ref()) else {
            continue;
        };
        // A parameter's TYPE is its single CHILD, not a `type_annotation`. Measured: every
        // parameter in `std.pareto` reports `ta=None children=1`. An earlier revision read
        // `type_annotation`, got `None` for every parameter, and derived the empty string as the
        // type name — so all 514 corpus refusals named the SAME empty type and the blocker
        // histogram collapsed to one meaningless row. A refusal that names nothing ranks nothing,
        // which is the defect this whole fragment exists to remove, reproduced in its purest form.
        let params: Vec<(String, String)> = decl
            .params
            .iter()
            .map(|p| {
                let ty = p
                    .children
                    .iter()
                    .next()
                    .map(|t| type_text(t))
                    .unwrap_or_else(|| "<parameter with no type node>".to_string());
                (p.name.clone(), ty)
            })
            .collect();
        out.push(DagFnSignature {
            name: decl.name.clone(),
            params,
            body: decl.body.clone(),
        });
    }
    out
}

/// What the fragment can say about one module's surface.
struct ModuleCorpusPlan {
    module_path: String,
    /// Functions whose every parameter domain derived, with the combined domain per function AND
    /// the argument tuples that domain actually consists of. The tuples are what the driver runs;
    /// carrying only the domain description is how an earlier revision could report a corpus it
    /// had never executed.
    derivable: Vec<(String, EnumeratedDomain, Vec<Vec<String>>)>,
    /// Functions that defeated derivation, each naming the type responsible.
    refused: Vec<(String, RefusalCause)>,
    /// `fn` lines the authority declares vs signatures actually parsed. These must agree; a gap
    /// means the parser missed a form, and reporting the pair is what stops a silent miss from
    /// reading as a module with a small surface.
    declared_fn_lines: usize,
    parsed_signatures: usize,
}

/// One module's surface partitioned AT FUNCTION GRAIN: which declared functions actually yield a
/// call, and which yield none and why.
///
/// This is the fact the receipt is denominated in. `ModuleCorpusPlan::derivable` is close to it
/// but not equal to it: a function whose parameter domains all derived while their product is
/// empty sits in `derivable` and contributes nothing, so counting that vector counts a coverage
/// claim rather than coverage.
#[derive(Debug)]
struct FunctionGrainCoverage {
    /// Functions that yield at least one call, with how many.
    covered: Vec<(String, usize)>,
    /// Functions that yield none, each with the cause. `RefusalCause::EmptyDerivedDomain`
    /// distinguishes "derived, but to nothing" from "did not derive".
    uncovered: Vec<(String, RefusalCause)>,
}

impl FunctionGrainCoverage {
    fn calls(&self) -> usize {
        self.covered.iter().map(|(_, n)| n).sum()
    }
}

/// The partition itself. Pure over the plan, which is what lets both arms of it be executed
/// against hand-built states rather than only against whatever the live corpus happens to hold.
fn function_grain_coverage(plan: &ModuleCorpusPlan) -> FunctionGrainCoverage {
    let mut covered = Vec::new();
    let mut uncovered = Vec::new();
    for (name, _domain, tuples) in &plan.derivable {
        if tuples.is_empty() {
            uncovered.push((name.clone(), RefusalCause::EmptyDerivedDomain));
        } else {
            covered.push((name.clone(), tuples.len()));
        }
    }
    for (name, cause) in &plan.refused {
        uncovered.push((name.clone(), cause.clone()));
    }
    FunctionGrainCoverage { covered, uncovered }
}

/// THE PLAN-GRAIN HALF OF THE ONE SELECTION CRITERION: is there a subject to compare?
///
/// `Ok(None)` selects. `Ok(Some(_))` excludes, typed and counted. `Err(_)` refuses, because the
/// third state is the reader's own blindness and it is not a property of the authority.
///
/// The three answers used to be two, and the missing distinction is the whole point. `zero
/// functions covered` was one arm covering three different states -- an authority with no
/// functions at all, an authority whose functions all defeat derivation, and a parse this
/// fragment failed to read -- with one message that was false about two of them. That is the
/// state-space conflation DESIGN.md names: one value standing for several states whose remedies
/// differ. Splitting it costs nothing and makes each population rankable on its own terms.
fn plan_grain_selection(
    plan: &ModuleCorpusPlan,
    coverage: FunctionGrainCoverage,
) -> Result<Option<ReceiptExclusion>, String> {
    // THE READER'S OWN BLINDNESS IS NOT A PROPERTY OF THE AUTHORITY. The `fn ` line count and the
    // parsed signature count are two readers of one fact, and the pair is carried precisely so a
    // disagreement is visible. When the parser sees none of the functions the source declares,
    // `zero covered` is IGNORANCE rather than the fact that nothing is derivable, and excluding
    // on it would publish a reader deficit as a fact about the module.
    if plan.parsed_signatures == 0 && plan.declared_fn_lines > 0 {
        return Err(format!(
            "behavioral-receipt: {} declares {} `fn ` line(s) and the parser produced no \
             signature at all. The two readers disagree, so this selection cannot say whether \
             the module has no derivable function or whether this fragment simply failed to read \
             it -- and those have opposite remedies. Refusing rather than excluding: an exclusion \
             here would publish the reader's blindness as a fact about the authority",
            plan.module_path, plan.declared_fn_lines
        ));
    }
    if plan.parsed_signatures == 0 {
        return Ok(Some(ReceiptExclusion::NoFunctionDeclared {
            module_path: plan.module_path.clone(),
        }));
    }
    if coverage.covered.is_empty() {
        return Ok(Some(ReceiptExclusion::NoFunctionHasACorpus {
            module_path: plan.module_path.clone(),
            uncovered: coverage.uncovered,
        }));
    }
    Ok(None)
}

/// Does a plan-grain exclusion SURVIVE what the generated-artifact population says about the
/// mirror? `None` means it does not -- the module is selected after all.
///
/// PURE, AND SEPARATED FROM THE ASK FOR ONE REASON: the divergence it decides has an EMPTY
/// POPULATION today, so nothing in the live corpus executes the branch that matters. An empty
/// population is not a safe place to leave a behaviour undecided -- the first real member would
/// settle it by accident, and whoever met it would be debugging a difference nobody had written
/// down. Pulling the decision out of the loop lets a planted control state the intent and execute
/// it permanently (DESIGN.md 4b(4): the evidence stays enrolled, it does not retire when the
/// production population catches up).
fn exclusion_survives_generated_artifact_population(
    exclusion: ReceiptExclusion,
    body: &GeneratedArtifactPathBody,
) -> Option<ReceiptExclusion> {
    match body {
        // A POSITIVE ANSWER: this path is a module mirror, not a generated artifact. Only here is
        // "no function yields a call" the whole story, so only here is the exclusion a fact.
        GeneratedArtifactPathBody::NotGenerated => Some(exclusion),
        // Produced OR Refused: either way the path belongs to the generated-artifact population,
        // whose subject is BYTES rather than behaviour -- and for that subject a declared function
        // is not required, because the identity check calls nothing. The differential loop is
        // where that population's answer is reported (identity, drift, or a generator refusal);
        // deciding it here would be a second adjudicator of one question.
        GeneratedArtifactPathBody::Produced(_) | GeneratedArtifactPathBody::Refused(_) => None,
    }
}

/// A plan CARRYING AT LEAST ONE CALL. There is no other way to build one.
///
/// The differential used to check this itself -- `if total == 0 { Refused }` -- which is
/// validation of a state the caller was free to construct (DESIGN.md section 5: prefer making the
/// bad state unwritable to flagging it afterwards). Selection now decides admission at function
/// grain, and hands the differential a value that cannot represent an empty corpus, so the
/// comparison-over-nothing the old guard existed to catch has no spelling. The guard is deleted
/// rather than kept beside the new arm: two answers to one question is the dual authority
/// DESIGN.md section 3 forbids, and the surviving one would be the one that never runs.
struct AdmittedPlan<'a> {
    plan: &'a ModuleCorpusPlan,
    coverage: FunctionGrainCoverage,
}

impl<'a> AdmittedPlan<'a> {
    /// `Err` carries the function-grain partition of a module no function of which yields a call
    /// -- the rows the exclusion is built from, so the refusal and the report read the same fact.
    fn of(plan: &'a ModuleCorpusPlan) -> Result<AdmittedPlan<'a>, FunctionGrainCoverage> {
        let coverage = function_grain_coverage(plan);
        if coverage.covered.is_empty() {
            Err(coverage)
        } else {
            Ok(AdmittedPlan { plan, coverage })
        }
    }
}

/// Every `.dag` module reachable from the source roots, keyed by its declared module path.
fn collect_dag_module_sources(
    source_roots: &[String],
) -> Result<std::collections::HashMap<String, String>, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    let mut out = std::collections::HashMap::new();
    let mut stack: Vec<PathBuf> = source_roots.iter().map(|r| workspace.join(r)).collect();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(mp) = v1_compiler::cli_run::extract_module_path_public(&content) {
                        out.insert(mp, content);
                    }
                }
            }
        }
    }
    Ok(out)
}

/// The types VISIBLE to a module: its own declarations plus those of its transitive imports.
///
/// THIS RESOLVES, IT DOES NOT WIDEN, and the distinction is the whole risk of this function.
/// Reaching further to find a declaration feels like covering more, and the refusal count can
/// quietly drop for the wrong reason. So what is found is put through `derive_parameter_domain`
/// UNCHANGED: a record with a `String` field still refuses, a `String where` refinement still
/// refuses, a coproduct with payload variants still refuses. The only thing that changes is
/// WHERE a declaration may be found — `Ordering` is `Less | Equal | Greater` whether it is
/// declared in this module or in `std.algebra`, and refusing it for its address rather than its
/// shape was an artifact of the implementation, not a property of the fragment.
///
/// The control for that claim is external and specific: `std.content_hash` refuses 26 of its 27
/// functions before that change, and must still refuse 26 of 27 after it. If that number moves,
/// this resolved nothing and widened something.
///
/// BOTH READS GO THROUGH THE GRAMMAR. An earlier revision found declarations with a line-based
/// reader and found imports with `line.strip_prefix("import ")`. Each was a second implementation
/// of something the parser already owns, and the first was measurably wrong -- blind to 88% of
/// the corpus's type declarations. They are gone rather than kept working: a hand reader that
/// currently agrees with the grammar has exactly the standing the line reader had until it was
/// measured. Imports now come from the module node's `params` and declarations from its
/// `children`, which is where the parser puts them.
///
/// A module that will not parse REFUSES the whole plan rather than contributing nothing. Silently
/// skipping it would make an unreadable import indistinguishable from an import that declares no
/// types, and the second is a fine reason to derive nothing while the first is not.
fn visible_type_decls(
    module_path: &str,
    source: &str,
    modules: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashMap<String, DagTypeDecl>, String> {
    let mut merged = std::collections::HashMap::new();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![(module_path.to_string(), source.to_string())];
    let mut depth_guard = 0usize;
    while let Some((mp, src)) = queue.pop() {
        depth_guard += 1;
        if depth_guard > 4096 {
            return Err(format!(
                "{module_path}: import closure exceeded 4096 modules; refusing rather than \
                 reporting a partial type environment"
            ));
        }
        if !seen.insert(mp.clone()) {
            continue;
        }
        let node = parse_dag_module_node(&format!("{mp}.dag"), &src)?;
        // A module's OWN declarations win: a local name shadows an imported one, and taking the
        // import would answer with a different type than the module compiles against.
        for (name, decl) in type_decls_from_module(&node) {
            merged.entry(name).or_insert(decl);
        }
        for imp in node.params.iter() {
            if let Some(next_src) = modules.get(&imp.name) {
                queue.push((imp.name.clone(), next_src.clone()));
            }
        }
    }
    Ok(merged)
}

/// The one entry point. An earlier revision also had a module-local planner that consulted only
/// the types declared in the module under plan; it is DELETED rather than kept beside this one.
/// Two resolvers over one question would answer differently for any imported type — which is the
/// exact defect this change fixes — and keeping the narrower one available is how a caller
/// silently gets the old answer back.
/// Every type name any module in the corpus declares.
///
/// Exists to separate two states a single refusal arm used to conflate: a type the reader could
/// not SEE from this module, and a type the corpus does not HAVE. Only the first is work.
fn declared_type_names(
    modules: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashSet<String>, String> {
    let mut out = std::collections::HashSet::new();
    for (mp, src) in modules {
        let node = parse_dag_module_node(&format!("{mp}.dag"), src)?;
        for (name, _decl) in type_decls_from_module(&node) {
            out.insert(name);
        }
    }
    Ok(out)
}

fn plan_module_corpus(
    module_path: &str,
    source: &str,
    module: &v1_compiler::v1_std_core::Node,
    types: &std::collections::HashMap<String, DagTypeDecl>,
    declared_anywhere: &std::collections::HashSet<String>,
    module_alias: &str,
) -> ModuleCorpusPlan {
    let sigs = fn_signatures_from_module(module);
    // Kept as a CROSS-CHECK, not as the source of truth. The authored `fn ` line count and the
    // parsed function count come from two different readers, so a disagreement means one of them
    // is wrong -- which is exactly how the line reader's 14 missing signatures were found. Now
    // that the parser owns the read they should agree, and the pair is reported so that a future
    // divergence is visible rather than silently halving a module's surface.
    let declared_fn_lines = source
        .lines()
        .filter(|l| l.trim_start().starts_with("fn "))
        .count();
    let mut derivable = Vec::new();
    let mut refused = Vec::new();
    for sig in &sigs {
        let mut partitioned = false;
        let mut partitions = Vec::new();
        let mut failure: Option<RefusalCause> = None;
        // Tuples are accumulated as a Cartesian product across parameters. A zero-parameter
        // function has exactly one tuple -- the empty one -- which is a real call, not an absence.
        let mut tuples: Vec<Vec<String>> = vec![Vec::new()];
        for (pname, pty) in &sig.params {
            // An `Int` parameter is asked about its own occurrences first. The partition IS its
            // domain, so a refusal here refuses the function -- there is no fallback window to
            // drop to, by design.
            let int_partition = if pty.trim() == "Int" {
                // No body means no occurrences to justify a partition. That REFUSES rather than
                // defaulting to "unused, so one value covers it": a declaration whose body the
                // parser did not attach is an unknown, and treating an unknown as an empty set of
                // uses is the narrow that turns "I could not see" into "there was nothing there".
                let Some(body) = sig.body.as_ref() else {
                    failure = Some(RefusalCause::IntNoBody {
                        param: pname.clone(),
                    });
                    break;
                };
                match derive_int_partition(pname, body) {
                    Ok(p) => Some(p),
                    Err(e) => {
                        failure = Some(e);
                        break;
                    }
                }
            } else {
                None
            };
            match enumerate_parameter_values(
                pty,
                types,
                declared_anywhere,
                0,
                int_partition.as_ref(),
                module_alias,
            ) {
                Ok(d) => {
                    if let Some(pt) = d.partition.clone() {
                        partitioned = true;
                        partitions.push(format!("{pname}: {pt}"));
                    }
                    let mut next: Vec<Vec<String>> = Vec::new();
                    for prefix in &tuples {
                        for v in &d.values {
                            if next.len() >= MAX_TUPLES_PER_FUNCTION {
                                break;
                            }
                            let mut row = prefix.clone();
                            row.push(v.clone());
                            next.push(row);
                        }
                    }
                    if next.len() >= MAX_TUPLES_PER_FUNCTION {
                        failure = Some(RefusalCause::TupleBudgetExceeded {
                            param: pname.clone(),
                        });
                        break;
                    }
                    tuples = next;
                }
                Err(offending) => {
                    failure = Some(offending);
                    break;
                }
            }
        }
        match failure {
            Some(offending) => refused.push((sig.name.clone(), offending)),
            None => {
                // The reported cardinality IS the number of tuples that will run. Deriving it
                // separately would let the report and the execution disagree.
                let cardinality = tuples.len();
                let domain = if partitioned {
                    EnumeratedDomain::ExhaustiveOverDerivedPartition {
                        cardinality,
                        partition: partitions.join("; "),
                    }
                } else {
                    EnumeratedDomain::Exhaustive { cardinality }
                };
                derivable.push((sig.name.clone(), domain, tuples));
            }
        }
    }
    ModuleCorpusPlan {
        module_path: module_path.to_string(),
        derivable,
        refused,
        declared_fn_lines,
        parsed_signatures: sigs.len(),
    }
}

/// The verdict for one candidate module. Three arms, and the third is not a soft pass.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptVerdict {
    /// Both builds ran the derived corpus and every call THAT COULD BE COMPARED agreed.
    ///
    /// `nondeterministic_calls` is carried on this arm rather than printed beside it because a
    /// green with an excluded population is a DIFFERENT claim from a green over everything, and
    /// separating the two would let the weaker one be read as the stronger. Zero is the ordinary
    /// case and reads as the full claim.
    Equivalent {
        calls: usize,
        nondeterministic_calls: usize,
        nondeterministic_functions: Vec<String>,
    },
    /// Both builds ran and at least one call disagreed. The first difference is carried because
    /// a count alone cannot be acted on.
    Divergent {
        calls: usize,
        first_difference: String,
    },
    /// The comparison could NOT be taken. Never reported as equivalence: an emit that failed, or
    /// a driver that would not compile, is ignorance, and rendering ignorance as the clean verdict
    /// is the empty-observation narrow. A corpus with nothing in it is NOT among these any more --
    /// it never reaches the differential, because `AdmittedPlan` cannot carry it.
    Refused { reason: String },
    /// EVERY derived call in this module renders unstably, so no comparison exists to take.
    ///
    /// Not `Equivalent` (nothing was compared) and not `Divergent` (nothing disagreed about the
    /// program). Not `Refused` either: a refusal in this fragment means the measurement could not
    /// be attempted, whereas this one WAS attempted and produced a well-defined result -- the
    /// subject is unaskable, and the fix lives in emission rather than in this gate or in the
    /// diff under test.
    NondeterministicRendering {
        unstable_calls: usize,
        functions: Vec<String>,
    },
}

/// Generate the driver: one `println!` per call in the derived corpus.
///
/// The transcript line carries the function name and the argument expressions as authored, so a
/// divergence names the exact call rather than an index into a product nobody can reconstruct.
/// Output goes through `{:?}`, which is why the fragment admits only types the emitted mirror
/// derives `Debug` on.
fn generate_receipt_driver(module_alias: &str, plan: &ModuleCorpusPlan) -> String {
    let mut out = String::new();
    out.push_str("// GENERATED by claim_executor --behavioral-receipt. Do not edit.\n");
    // FULLY QUALIFIED, with no `use ... as m` alias. An earlier revision aliased the module for
    // calls while the enumerated constructor values were rendered against the bare module name,
    // so the driver referred to the same module two ways and only one of them resolved. One
    // spelling, produced in one place, cannot drift from itself.
    out.push_str("fn main() {\n");
    for (name, _domain, tuples) in &plan.derivable {
        for args in tuples {
            let call = format!("{module_alias}::{name}({})", args.join(", "));
            let shown = args.join(", ").replace('"', "\\\"");
            out.push_str(&format!(
                "    println!(\"{name}({shown}) = {{:?}}\", {call});\n"
            ));
        }
    }
    out.push_str("}\n");
    out
}

/// One driver's output, plus WHICH of its lines are not a function of the program alone.
///
/// A line is `unstable` when two executions of the SAME binary printed different text for it.
/// That is proof, not inference: the code, the inputs and the build are identical across the two
/// runs, so anything that differs came from somewhere other than the program's meaning. In this
/// corpus the somewhere is `HashMap`/`HashSet` iteration order reaching `{:?}`, whose seed is
/// randomized per process -- measured at 20 distinct transcripts over 20 executions of one
/// unchanged binary for `std.algebra::kernel_algebra_profile_value`.
///
/// WHY THIS IS MEASURED RATHER THAN DECIDED FROM THE TYPE. Order-dependent rendering is a
/// property of the value's TRANSITIVE shape: a record CONTAINING a map renders nondeterministically
/// while its own return type says `Record`. Any check keying on the outermost constructor
/// under-refuses by construction, and it under-refuses SILENTLY -- the missed call lands in
/// `Divergent`, indistinguishable from a real divergence. Running the binary twice keys on the
/// property itself, so there is no type walk to keep in sync with the corpus.
///
/// THE RESIDUE, STATED: two randomized renderings can coincide, so a subject that happened to
/// agree twice is not caught. That makes every count derived from this a FLOOR, and the floor is
/// printed as such rather than left in this comment. It is a residue that SHRINKS with more
/// executions, unlike a structural blind spot, but this is not an argument for adding runs
/// speculatively -- one extra run is what the evidence to date justifies.
#[derive(Debug, Clone, PartialEq)]
struct DriverTranscript {
    lines: Vec<String>,
    /// Indices into `lines` that differed between the two executions.
    unstable: std::collections::BTreeSet<usize>,
}

impl DriverTranscript {
    fn of(first: Vec<String>, second: Vec<String>) -> Self {
        // A length difference between two runs of one binary is itself instability, and it is
        // not attributable to any single index -- so every line of the longer run is marked
        // rather than none. Silently comparing the common prefix would hide it.
        let unstable = if first.len() != second.len() {
            (0..first.len().max(second.len())).collect()
        } else {
            first
                .iter()
                .zip(second.iter())
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .map(|(i, _)| i)
                .collect()
        };
        Self {
            lines: first,
            unstable,
        }
    }
}

/// Build the crate as it currently stands, compile the driver against it, and return the
/// transcript. Both halves of the differential go through THIS function, so the two transcripts
/// cannot differ because of how they were produced.
/// WHICH crate the driver is built against and linked to.
///
/// These three were literals inside `run_receipt_driver`. A hardwired `-p v1-compiler` is policy
/// standing inside a mechanism (DESIGN §3: an argv carrying a literal it should receive as a
/// parameter), and it is exactly what made the mode impossible to exercise against anything but
/// the production mirror -- so its own arms could only ever be run by a human with a script.
/// Parameterised, the control and the production path are ONE code path with one argument
/// different, rather than two machineries that could drift apart while both looked green.
struct ReceiptCrate {
    package: String,
    extern_name: String,
    rlib: String,
}

impl ReceiptCrate {
    fn v1_compiler() -> Self {
        Self {
            package: "v1-compiler".to_string(),
            extern_name: "v1_compiler".to_string(),
            rlib: "libv1_compiler.rlib".to_string(),
        }
    }

    fn receipt_fixture() -> Self {
        Self {
            package: "v1-receipt-fixture".to_string(),
            extern_name: "v1_receipt_fixture".to_string(),
            rlib: "libv1_receipt_fixture.rlib".to_string(),
        }
    }
}

fn run_receipt_driver(
    workspace: &std::path::Path,
    krate: &ReceiptCrate,
    driver_src: &str,
    label: &str,
) -> Result<DriverTranscript, String> {
    let drv_dir = workspace.join("target/behavioral-receipt");
    fs::create_dir_all(&drv_dir).map_err(|e| format!("create {}: {e}", drv_dir.display()))?;
    let src_path = drv_dir.join(format!("driver_{label}.rs"));
    fs::write(&src_path, driver_src).map_err(|e| format!("write driver: {e}"))?;

    let lib = Command::new("cargo")
        .args(["build", "--release", "-p", &krate.package, "--lib"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn cargo ({label}): {e}"))?;
    if !lib.status.success() {
        return Err(format!(
            "{label}: the crate did not build; the candidate is not admissible without a compile. \
             {}",
            String::from_utf8_lossy(&lib.stderr)
                .lines()
                .filter(|l| l.starts_with("error"))
                .take(4)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    let bin_path = drv_dir.join(format!("driver_{label}"));
    let rustc = Command::new("rustc")
        .args(["--edition", "2021", "-O"])
        .arg(&src_path)
        .arg("--extern")
        .arg(format!(
            "{}={}",
            krate.extern_name,
            workspace.join("target/release").join(&krate.rlib).display()
        ))
        .arg("-L")
        .arg(workspace.join("target/release/deps"))
        .arg("-o")
        .arg(&bin_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn rustc ({label}): {e}"))?;
    if !rustc.status.success() {
        return Err(format!(
            "{label}: the driver did not compile against the mirror: {}",
            String::from_utf8_lossy(&rustc.stderr)
                .lines()
                .filter(|l| l.starts_with("error"))
                .take(4)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    // TWICE, AND THE SECOND RUN IS THE POINT. The two are executions of the SAME BINARY -- the
    // cargo build and the rustc compile above are already paid, so this costs milliseconds and
    // not a rebuild. Two executions of one unchanged binary that disagree PROVE the disagreeing
    // call renders nondeterministically, which is the only way to learn that fact: it is a
    // property of the value's rendering, not of its type, so nothing before the run can know it.
    let first = run_driver_binary(workspace, &bin_path, label)?;
    let second = run_driver_binary(workspace, &bin_path, label)?;
    Ok(DriverTranscript::of(first, second))
}

fn run_driver_binary(
    workspace: &std::path::Path,
    bin_path: &std::path::Path,
    label: &str,
) -> Result<Vec<String>, String> {
    let run = Command::new(bin_path)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn driver ({label}): {e}"))?;
    if !run.status.success() {
        return Err(format!(
            "{label}: the driver ran and exited {}; a corpus call panicking is a behavioural fact, \
             but it is not one this comparison can attribute, so it refuses",
            run.status
        ));
    }
    Ok(String::from_utf8_lossy(&run.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// The two-build differential for ONE candidate module.
///
/// Seed transcript first, from the tree as committed. Then the emitted candidate is written over
/// its mirror, the crate is rebuilt, and the SAME driver runs again. The mirror is restored on
/// every path, including failure -- leaving a candidate in the tree would make the next reader's
/// measurement a lie.
///
/// This is what the whole fragment is for. CI proves the committed mirrors equal what the
/// authority emits and that the emit repeats; it never compiles the candidate, let alone runs it.
/// A byte comparison cannot distinguish a rename from a semantic change, and DESIGN §7 says a
/// byte-identical fixed point is explicitly NOT the goal -- behavioural equivalence on a
/// discriminating corpus is.
fn behavioral_differential(
    workspace: &std::path::Path,
    krate: &ReceiptCrate,
    mirror_path: &std::path::Path,
    candidate_source: &str,
    admitted: &AdmittedPlan<'_>,
    module_alias: &str,
) -> ReceiptVerdict {
    // No emptiness check: `AdmittedPlan` cannot be built from a plan with no call in it, so the
    // comparison-over-nothing this function used to guard against is unrepresentable here rather
    // than rejected here.
    let plan = admitted.plan;
    let driver = generate_receipt_driver(module_alias, plan);
    let shown = mirror_path.display().to_string();

    // THE SEED MUST ACTUALLY BE THE SEED (review 54094). This function installs a candidate over
    // the committed mirror and restores it on every path -- but a process killed between the two
    // leaves the candidate in the tree, and the NEXT run would then read it as the committed
    // bytes and compare the candidate against itself. That comparison answers EQUIVALENT, which
    // is the worst available wrong answer: a green that means nothing, produced by a mechanism
    // whose entire job is to be trusted.
    //
    // The residue cannot be prevented -- no arrangement of writes survives SIGKILL -- so it is
    // made LOUD instead. A dirty path is a refusal, not a warning, and it names the recovery.
    match git_stdout(
        workspace,
        &[
            "status",
            "--porcelain",
            "--",
            &mirror_path.to_string_lossy(),
        ],
    ) {
        Ok(out) if !out.trim().is_empty() => {
            return ReceiptVerdict::Refused {
                reason: format!(
                    "{shown} is modified in the working tree, so the bytes this would read as the \
                     SEED are not the committed bytes. Most likely a previous run was killed \
                     between installing a candidate and restoring it -- in which case this run \
                     would compare that candidate against itself and answer EQUIVALENT. Restore \
                     it (`git checkout -- {shown}`) before measuring: {}",
                    out.trim()
                ),
            }
        }
        Ok(_) => {}
        Err(e) => {
            return ReceiptVerdict::Refused {
                reason: format!(
                    "cannot determine whether {shown} is clean ({e}), so cannot establish that the \
                     seed transcript comes from committed bytes"
                ),
            }
        }
    }
    let committed = match fs::read_to_string(mirror_path) {
        Ok(c) => c,
        Err(e) => {
            return ReceiptVerdict::Refused {
                reason: format!("read {shown}: {e}"),
            }
        }
    };

    let seed = match run_receipt_driver(workspace, krate, &driver, "seed") {
        Ok(t) => t,
        Err(e) => return ReceiptVerdict::Refused { reason: e },
    };

    if let Err(e) = fs::write(mirror_path, candidate_source) {
        return ReceiptVerdict::Refused {
            reason: format!("install candidate {shown}: {e}"),
        };
    }
    let candidate = run_receipt_driver(workspace, krate, &driver, "candidate");
    // Restore BEFORE interpreting the result, so no early return can leave the candidate in place.
    if let Err(e) = fs::write(mirror_path, &committed) {
        return ReceiptVerdict::Refused {
            reason: format!(
                "restore {shown} after the candidate build: {e}. The tree may still hold \
                 the candidate; do not trust a later measurement without checking"
            ),
        };
    }
    let candidate = match candidate {
        Ok(t) => t,
        Err(e) => return ReceiptVerdict::Refused { reason: e },
    };

    if seed.lines.len() != candidate.lines.len() {
        return ReceiptVerdict::Divergent {
            calls: seed.lines.len(),
            first_difference: format!(
                "transcript lengths differ: seed {} lines, candidate {} lines",
                seed.lines.len(),
                candidate.lines.len()
            ),
        };
    }
    // THE UNION, NOT THE SEED'S SET ALONE. Each side is its own binary and each was measured
    // independently, so a call can render unstably in one and (by coincidence, on that pair of
    // runs) stably in the other. Comparing a line either side proved unstable would score a
    // coin flip as a behavioural difference, which is the fabricated-difference failure this
    // whole change exists to remove.
    let unstable: std::collections::BTreeSet<usize> =
        seed.unstable.union(&candidate.unstable).copied().collect();
    let excluded = nondeterministic_call_functions(admitted, &unstable);

    let compared: Vec<(usize, (&String, &String))> = seed
        .lines
        .iter()
        .zip(candidate.lines.iter())
        .enumerate()
        .filter(|(i, _)| !unstable.contains(i))
        .collect();

    // Nothing left to compare is NOT equivalence, and it is not a divergence either: it is a
    // module whose every derived call renders unstably, so this fragment cannot ask it anything
    // honestly. Reported as its own verdict rather than folded into either, because the action
    // it calls for -- make emission deterministic -- is neither "fix the diff" nor "nothing to do".
    if compared.is_empty() {
        return ReceiptVerdict::NondeterministicRendering {
            unstable_calls: unstable.len(),
            functions: excluded,
        };
    }
    for (_, (a, b)) in &compared {
        if a != b {
            return ReceiptVerdict::Divergent {
                calls: compared.len(),
                first_difference: format!("seed: {a}  |  candidate: {b}"),
            };
        }
    }
    ReceiptVerdict::Equivalent {
        calls: compared.len(),
        nondeterministic_calls: unstable.len(),
        nondeterministic_functions: excluded,
    }
}

/// Which declared functions own the calls at `unstable`.
///
/// The driver prints one line per call in exactly the order `AdmittedPlan` enumerates them, so
/// the mapping is positional and derived from the same iteration that produced the transcript --
/// not a second traversal that could disagree with it. Names, because a COUNT of excluded calls
/// cannot be acted on and a name can: it is the function whose return value to make deterministic.
fn nondeterministic_call_functions(
    admitted: &AdmittedPlan<'_>,
    unstable: &std::collections::BTreeSet<usize>,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut index = 0usize;
    for (name, _domain, tuples) in &admitted.plan.derivable {
        for _ in tuples {
            if unstable.contains(&index) && !names.iter().any(|n| n == name) {
                names.push(name.clone());
            }
            index += 1;
        }
    }
    names
}

/// What the generated-artifact population says about one repo-relative path.
///
/// Three states because the honest answers are three. `NotGenerated` is a POSITIVE answer -- this
/// path is not in the generated-artifact population at all -- and is what routes a caller to the
/// mirror-emit population. Folding it into `Refused` would tell a caller that generation FAILED
/// for an ordinary mirror, which is false and differently actionable.
enum GeneratedArtifactPathBody {
    Produced(String),
    Refused(String),
    NotGenerated,
}

/// Ask the already-resolved generated-artifact authority for the body it generates at a path.
///
/// THIS IS NOT A SECOND PRODUCER. The `.dag` side is a projection over the same three authorities
/// `main_wet` uses -- the committed-artifact roster, `artifact_path`, and the single
/// `artifact_generate` dispatch -- asked by path instead of by artifact. Reaching past it to a
/// per-artifact emitter would have been the forked dispatch DESIGN §3 forbids.
///
/// COST SHAPE, and it is why this takes a CONTEXT rather than `source_roots`. The first draft
/// resolved `generated_artifact_emit`'s whole closure inside the per-module loop, making the unit
/// of computation the corpus while the unit of fact was one path -- DESIGN §6's cost-shape defect,
/// where the rule is that a proven one is fixed regardless of the realized n. The caller resolves
/// once; each path is then one interpreter call against that context.
/// The generated-artifact authority's evaluation context, resolved AT MOST ONCE per run and
/// shared by every caller that needs it.
///
/// One cell rather than one resolve per asking site: selection asks it for a module that yields
/// no call, and the differential loop asks it for every selected module. Two resolves of one
/// closure would be two producers of the same context and would pay the corpus-sized cost twice.
fn generated_artifact_ctx<'a>(
    source_roots: &[String],
    cell: &'a mut Option<v1_compiler::v1_interpreter::InterpContext>,
) -> Result<&'a v1_compiler::v1_interpreter::InterpContext, String> {
    if cell.is_none() {
        let entry = "dag/gunbc/generated_artifact_emit.dag";
        let (graph, indices) =
            v1_compiler::cli_run::resolve_entry_graph_shared(source_roots, entry)
                .map_err(|e| format!("resolve {entry}: {e}"))?;
        *cell = Some(v1_compiler::cli_run::make_eval_context(
            &graph,
            indices,
            // HERMETIC, not Wet. The projection is pure -- it folds a roster and returns a String
            // -- so a host effect reached during it would mean a generator is doing something
            // this gate must not perform on its behalf. Hermetic refuses there instead of
            // carrying it out.
            v1_compiler::v1_interpreter::ExecutionMode::Hermetic,
        ));
    }
    Ok(cell.as_ref().expect("the context was just installed"))
}

fn generated_artifact_body_for_path(
    ctx: &v1_compiler::v1_interpreter::InterpContext,
    repo_rel_path: &str,
) -> Result<GeneratedArtifactPathBody, String> {
    use v1_compiler::v1_interpreter::Value;
    let out = v1_compiler::v1_interpreter::run_in_context_with_args(
        ctx,
        "generated_artifact_body_for_path",
        &[(
            Some("path".to_string()),
            Value::Str(repo_rel_path.to_string().into()),
        )],
        false,
    )
    .map_err(|e| format!("generated_artifact_body_for_path({repo_rel_path}): {e:?}"))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &out
    else {
        // No default arm: a shape this code does not understand is ignorance, and guessing
        // NotGenerated here would silently route a real generated artifact to the mirror emit
        // and refuse it there for the wrong reason.
        return Err(format!(
            "generated_artifact_body_for_path({repo_rel_path}) returned a non-variant value"
        ));
    };
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathNotGenerated") {
        return Ok(GeneratedArtifactPathBody::NotGenerated);
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyProduced") {
        return match ctx.field(fields, "content") {
            Some(Value::Str(c)) => Ok(GeneratedArtifactPathBody::Produced(c.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyProduced for {repo_rel_path} carried no String content"
            )),
        };
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyRefused") {
        return match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Ok(GeneratedArtifactPathBody::Refused(r.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyRefused for {repo_rel_path} carried no String reason"
            )),
        };
    }
    Err(format!(
        "generated_artifact_body_for_path({repo_rel_path}) returned an unknown variant"
    ))
}

/// Map an authority module path to the emitted mirror that declares it as its authority.
///
/// DERIVED, never authored: each generated file names its authority in its own header, written
/// by the emitter, so the mapping is a property of the artifact rather than of a roster someone
/// maintains, and it cannot be forged by editing a list.
///
/// TWO HEADER KEYS, BECAUSE THE CORPUS HAS TWO EMITTERS. The v1 compiler writes
/// `// Source module: <mod>`; `gunbc`'s own artifact emitters write `// Authority: <mod> ...`.
/// Measured over `src/v1/stage0/src`: 130 files declare themselves generated -- 126 by the first
/// key, 2 by the second (`bootstrap_stage0_crate_layout_generated.rs`,
/// `v1_interpreter_dispatch_generated.rs`), and `lib.rs`/`main.rs`, which are crate roots rather
/// than module mirrors. An earlier revision of this function read ONLY the first key, so those
/// two real mirrors of two real authorities were invisible to it: a change to either would have
/// been reported as "no emitted mirror declares it" -- FALSE, and false in the direction that
/// silently skips the check.
///
/// THAT WAS THE SAME TWO-ZEROS CONFLATION THIS MODE FIXES ONE LEVEL UP: "nothing mirrors this
/// authority" and "I could not find what mirrors it under the key I searched" are different
/// states with different remedies, and they printed the same exclusion line. So the index does
/// not merely learn the second key -- learning keys one incident at a time is how the blind spot
/// recurs. It ASSERTS ITS OWN KEY-SPACE COMPLETENESS: any file that declares itself generated
/// and yet carries neither key is a THIRD convention, which means this index has just gone blind
/// again, and the whole selection REFUSES rather than excluding authorities it cannot see.
///
/// The two crate roots are exempt by KIND, not by roster: `lib.rs` and `main.rs` are cargo's
/// crate-root names (an external, versioned authority), they aggregate the module SET rather
/// than mirroring any one authority, and no authority module path can ever select them.
struct MirrorIndex {
    by_module: std::collections::HashMap<String, String>,
}

/// The header keys, as a closed set read in one place. A key added here is a decision recorded
/// once; a key MISSING here is caught by the completeness refusal below rather than by silence.
const MIRROR_AUTHORITY_HEADER_KEYS: [&str; 2] = ["// Source module: ", "// Authority: "];

fn mirror_authority_of_header(content: &str) -> Option<String> {
    for line in content.lines().take(6) {
        let line = line.trim();
        for key in MIRROR_AUTHORITY_HEADER_KEYS {
            if let Some(rest) = line.strip_prefix(key) {
                // The `Authority:` form carries a symbol and a regen recipe after the module
                // path, separated by whitespace or `;`. Take the module path and nothing else --
                // comparing the whole remainder against a module path matches nothing, which is
                // the same silent miss one layer down.
                let module = rest
                    .trim()
                    .split([' ', ';', '\t'])
                    .next()
                    .unwrap_or_default()
                    .trim();
                if !module.is_empty() {
                    return Some(module.to_string());
                }
            }
        }
    }
    None
}

fn build_mirror_index(stage0_src: &std::path::Path) -> Result<MirrorIndex, String> {
    let entries =
        fs::read_dir(stage0_src).map_err(|e| format!("read_dir {}: {e}", stage0_src.display()))?;
    let mut by_module: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut unindexable: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let base = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        match mirror_authority_of_header(&content) {
            Some(module) => {
                by_module.insert(module, base);
            }
            None => {
                let declares_generated = content
                    .lines()
                    .next()
                    .map(|l| l.trim_start().starts_with("// Generated by "))
                    .unwrap_or(false);
                let is_crate_root = matches!(base.as_str(), "lib.rs" | "main.rs");
                if declares_generated && !is_crate_root {
                    unindexable.push(base);
                }
            }
        }
    }
    if !unindexable.is_empty() {
        unindexable.sort();
        return Err(format!(
            "the mirror index cannot see {} generated file(s) that declare themselves generated \
             but carry neither known authority header ({}): {}. A third header convention means \
             every \"no emitted mirror\" answer below is IGNORANCE rather than a fact, so the \
             selection refuses instead of excluding authorities it cannot see. Either the new \
             convention joins MIRROR_AUTHORITY_HEADER_KEYS, or the emitter writes an existing one",
            unindexable.len(),
            MIRROR_AUTHORITY_HEADER_KEYS.join("| "),
            unindexable.join(", ")
        ));
    }
    Ok(MirrorIndex { by_module })
}

/// THE POPULATION, AND WHAT DEFEATS IT -- a census, not a gate.
///
/// The differential answers ONE candidate. This answers the prior question: across every module
/// the seed actually carries, how much of each one's surface can be covered at all, and what
/// stands in the way of the rest. It runs no build and installs no candidate; it is the ranking
/// input for how far to extend a mechanism that is now proven on one module, and it deliberately
/// exits SUCCESS on any population -- a census that refused would be a gate, and nothing here
/// establishes what the right coverage is.
///
/// The population is DERIVED, never authored: every emitted mirror names its authority in its
/// own header, so the roster is a property of the artifacts. A module whose authority source is
/// missing is REPORTED, not skipped -- a census that silently drops what it cannot read reports a
/// smaller corpus as a cleaner one.
fn behavioral_receipt_census(source_roots: &[String]) -> Result<bool, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");
    let modules = collect_dag_module_sources(source_roots)?;
    let declared_anywhere = declared_type_names(&modules)?;

    let mut roster: Vec<(String, String)> = Vec::new();
    let entries =
        fs::read_dir(&stage0_src).map_err(|e| format!("read_dir {}: {e}", stage0_src.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("dir entry: {e}"))?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        // Same reader as the plan mode's index, so the census and the gate cannot disagree
        // about which files mirror an authority -- including the two that name it with the
        // second header convention.
        if let Some(declared) = mirror_authority_of_header(&content) {
            roster.push((
                declared,
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string(),
            ));
        }
    }
    roster.sort();

    let mut fns_total = 0usize;
    let mut fns_derivable = 0usize;
    let mut fns_refused = 0usize;
    let mut calls_total = 0usize;
    let mut modules_planned = 0usize;
    // NOT "no authority": the census cannot distinguish an authority that does not exist from one
    // that exists outside the roots it was given, so it reports what it actually knows and names
    // the roots. The first revision of this line said NO AUTHORITY SOURCE, and 55 of 127 modules
    // hit it -- every one of them a v1.compiler module whose .dag lives under src/v1, which is
    // simply not a scanned root. A refusal that names the wrong cause ranks the wrong work.
    let mut out_of_scope: Vec<String> = Vec::new();
    // Keyed on the TYPE that defeated derivation, because that is the unit of work: grounding one
    // type unlocks every function whose only obstacle was that type. Counting refusals instead
    // would rank the same fix once per site.
    let mut by_blocker: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut types_declared_total = 0usize;
    let mut types_read_total = 0usize;
    let mut type_reader_gaps: Vec<(String, usize, usize, Vec<String>)> = Vec::new();

    for (module_path, mirror) in &roster {
        let Some(source) = modules.get(module_path) else {
            out_of_scope.push(module_path.clone());
            continue;
        };
        let alias = format!("v1_compiler::{}", mirror.trim_end_matches(".rs"));
        let node = parse_dag_module_node(&format!("{module_path}.dag"), source)?;
        let types = visible_type_decls(module_path, source, &modules)?;
        let plan = plan_module_corpus(
            module_path,
            source,
            &node,
            &types,
            &declared_anywhere,
            &alias,
        );
        modules_planned += 1;
        fns_total += plan.parsed_signatures;
        fns_derivable += plan.derivable.len();
        fns_refused += plan.refused.len();
        let calls: usize = plan.derivable.iter().map(|(_, _, t)| t.len()).sum();
        calls_total += calls;
        for (_f, why) in &plan.refused {
            *by_blocker.entry(why.subject()).or_insert(0) += 1;
        }
        // THE SAME CROSS-CHECK THE FUNCTION READER CARRIES, for types. Two readers of one fact --
        // an authored `type ` line count and the count the parser actually produced -- so a
        // disagreement means the type reader missed a form. That is not hypothetical: Node ranked
        // 798 as "undeclared anywhere in corpus" while src/v1/00_core.dag declares it plainly, and
        // nothing in the output said the reader had skipped it.
        // The authored names, not just a count: the count says a form was missed, the NAMES say
        // which declarations, and only the second is actionable without guessing.
        let authored: Vec<String> = source
            .lines()
            .filter_map(|l| l.trim_start().strip_prefix("type "))
            .filter_map(|rest| {
                // CUT AT `<` TOO. Without it the authored name of a generic declaration came out
                // as `Magma<T>` or, worse, `Map<key,` -- while the reader registers the bare name
                // -- so the comparison reported 11 modules as having gaps whose type_lines and
                // types_read were EQUAL. The falsifier was manufacturing its own false positives,
                // which is the one failure mode a cross-check cannot be allowed to have: it spends
                // exactly the attention it exists to direct.
                rest.split(|c: char| c.is_whitespace() || c == '{' || c == '=' || c == '<')
                    .find(|t| !t.is_empty())
                    .map(str::to_string)
            })
            .collect();
        let read = type_decls_from_module(&node);
        let type_lines = authored.len();
        let types_read = read.len();
        types_declared_total += type_lines;
        types_read_total += types_read;
        let missed: Vec<String> = authored
            .iter()
            .filter(|n| !read.contains_key(*n))
            .cloned()
            .collect();
        if !missed.is_empty() {
            type_reader_gaps.push((module_path.clone(), type_lines, types_read, missed));
        }
        eprintln!(
            "receipt-census: {module_path} parsed={} derivable={} calls={} refused={} type_lines={type_lines} types_read={types_read}",
            plan.parsed_signatures,
            plan.derivable.len(),
            calls,
            plan.refused.len()
        );
    }

    let mut ranked: Vec<(String, usize)> = by_blocker.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    eprintln!(
        "receipt-census: modules_with_mirror={} planned={} authority_outside_scanned_roots={}",
        roster.len(),
        modules_planned,
        out_of_scope.len()
    );
    // A COUNT IS NOT A SAFE REPORT HERE. Read as a small number, out-of-scope looks like a rounding
    // error; but the same shape at a wrong root set is a hole centred on whatever nobody scanned,
    // and this arm has already been that hole once -- it swallowed 55 of 127 as "no authority"
    // because src/v1 was not a root. So the fraction is stated against the population, and any
    // out-of-scope module at all makes the run say what its coverage actually is rather than
    // leading with the modules it managed to plan.
    if !out_of_scope.is_empty() {
        eprintln!(
            "receipt-census: COVERAGE {}/{} modules planned; {} of the corpus was NOT measured \
             because no scanned root holds its authority. Roots given: {}. A module below is not \
             evidence that its authority is missing -- it is evidence that this run could not see \
             it, and if this fraction is large the root set is the finding, not the corpus",
            modules_planned,
            roster.len(),
            out_of_scope.len(),
            source_roots.join(", ")
        );
    }
    for m in &out_of_scope {
        eprintln!("receipt-census: OUT OF SCOPE {m} — counted, not skipped");
    }
    eprintln!(
        "receipt-census: functions parsed={fns_total} derivable={fns_derivable} refused={fns_refused} calls={calls_total}"
    );
    eprintln!(
        "receipt-census: TYPE READER type_lines={types_declared_total} types_read={types_read_total} modules_with_a_gap={}",
        type_reader_gaps.len()
    );
    if !type_reader_gaps.is_empty() {
        eprintln!(
            "receipt-census: a gap means the reader did not produce a declaration the source \
             authors, so every refusal naming those types is measuring THIS READER, not the corpus"
        );
        let mut worst = type_reader_gaps.clone();
        worst.sort_by(|a, b| b.3.len().cmp(&a.3.len()));
        for (m, lines, read, missed) in worst.iter() {
            eprintln!(
                "receipt-census:   GAP {m} type_lines={lines} types_read={read} missed={}",
                missed.join(", ")
            );
        }
        // WHAT SHAPE the missed declarations actually have, for one module, printed rather than
        // assumed. Three times now a node-shape assumption has been wrong and each time the cost
        // was a whole measurement built on it; the shape is cheap to report and there is no reason
        // for the next reader of this output to have to re-derive it.
        // WHAT DISTINGUISHES A TYPE DECLARATION FROM A FUNCTION OR A DATA ROW, printed for one
        // module rather than assumed. The reader must filter on a property of the node; every
        // previous attempt to name one from memory has been wrong.
        if let Some(src) = modules.get("std.pareto") {
            if let Ok(node) = parse_dag_module_node("std.pareto.dag", src) {
                let authored: std::collections::HashSet<String> = src
                    .lines()
                    .filter_map(|l| l.trim_start().strip_prefix("type "))
                    .filter_map(|r| {
                        r.split(|c: char| c.is_whitespace() || c == '{' || c == '=' || c == '<')
                            .find(|t| !t.is_empty())
                            .map(str::to_string)
                    })
                    .collect();
                let mut shapes: std::collections::BTreeMap<String, (usize, Vec<String>)> =
                    std::collections::BTreeMap::new();
                for c in node.children.iter() {
                    let key = format!(
                        "is_type={} conn={:?} body={} params={} inferred={} children={}",
                        authored.contains(&c.name),
                        c.connective,
                        c.body.is_some(),
                        c.params.len(),
                        c.inferred.is_some(),
                        c.children.len()
                    );
                    let e = shapes.entry(key).or_insert((0, Vec::new()));
                    e.0 += 1;
                    if e.1.len() < 3 {
                        e.1.push(c.name.clone());
                    }
                }
                for (k, (n, ex)) in &shapes {
                    eprintln!(
                        "receipt-census:   CHILDSHAPE {n:4}  {k}  e.g. {}",
                        ex.join(", ")
                    );
                }
            }
        }
        if let Some((m, _, _, missed)) = worst.iter().find(|(m, _, _, _)| m == "std.pareto") {
            if let Some(src) = modules.get(m) {
                if let Ok(node) = parse_dag_module_node(&format!("{m}.dag"), src) {
                    for name in missed {
                        match node.children.iter().find(|c| &c.name == name) {
                            Some(c) => {
                                let f = c.children.iter().next();
                                eprintln!(
                                    "receipt-census:   SHAPE {m}::{name} connective={:?} \
                                     children={} | field name={:?} children={} conn={:?} \
                                     type_annotation={:?} inferred={}",
                                    c.connective,
                                    c.children.len(),
                                    f.map(|f| f.name.clone()),
                                    f.map(|f| f.children.len()).unwrap_or(0),
                                    f.map(|f| f.connective.clone()),
                                    f.and_then(|f| f.type_annotation.as_ref())
                                        .map(|t| t.name.clone()),
                                    f.map(|f| f.inferred.is_some()).unwrap_or(false)
                                )
                            }
                            None => eprintln!(
                                "receipt-census:   SHAPE {m}::{name} — NO module child carries \
                                 this name; the declaration is not where the reader looks"
                            ),
                        }
                    }
                }
            }
        }
    }
    eprintln!("receipt-census: refusals ranked by the type responsible");
    for (why, n) in ranked.iter().take(40) {
        eprintln!("receipt-census:   {n:5}  {why}");
    }
    if ranked.len() > 40 {
        let tail: usize = ranked.iter().skip(40).map(|(_, n)| n).sum();
        eprintln!(
            "receipt-census:   {tail:5}  [{} further distinct causes, not shown]",
            ranked.len() - 40
        );
    }
    Ok(true)
}

fn run_behavioral_receipt_census(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_census(source_roots) {
        Ok(_) => Ok(ExitCode::SUCCESS),
        Err(e) => {
            eprintln!("receipt-census: REFUSED — {e}");
            Err(ExitCode::from(1))
        }
    }
}

/// THE RECEIPT'S OWN ARMS, ENROLLED.
///
/// A behavioral receipt is only evidence if it can still tell equivalence from divergence. That
/// property is not established by the mode existing, and it is not established by a transcript in
/// a pull request: a red control that no longer discriminates looks exactly like a red control
/// that does, until the day it is needed. So both arms live here and execute -- DESIGN §4b: the
/// discriminating RED and the positive control REMAIN ENROLLED as the executing evidence that the
/// rung stays real.
///
/// Run against a CONTROLLED FIXTURE (`fixtures/receipt_fixture`), never against the live corpus.
/// The fixture independently authors its own input and its own expected outcome, which is what
/// DESIGN §5 requires of an oracle -- a measurement copied from the current tree is not one. It
/// also means this control cannot be satisfied by a tree in which nothing happens to have changed.
///
/// WHAT THIS CONTROL COVERS: the grammar-backed read of a declared surface, the derivation of the
/// corpus from it, the enumeration of argument tuples, driver generation, both builds, the
/// transcript comparison, and the verdict. WHAT IT DOES NOT COVER, stated rather than implied:
/// the emit, and the emit-path-to-mirror lookup. Those have their own gates; this one would
/// report a false green about them, so it does not speak about them at all.
/// THE CITED-SYMBOL CENSUS, RUN AS ITS OWN REQUIRED CHECK BECAUSE IT CANNOT BE RUN AS A CLAIM.
///
/// `v2.lens.cited_symbol_resolution` resolves every structural `DeclarationRef` the repository
/// authors -- hand-authored doc binds, the generated design document's references, and the roster
/// registry -- against live declaration facts, and refuses a reference whose module, declaration or
/// field is absent or ambiguous. Its live-corpus claim was carried only by a witness under
/// `dag/test/claim/long/`, whose entire file is classified `OfflineLocalRecipe`, so nothing executed
/// it: on unmodified main the lens was RED with 27 refusing production references while every
/// required check was green.
///
/// WHY NOT THE FLOOR. `run_required_floor` declines an identity whose entry reads the live tree
/// (`DeclinedLiveTree`), and reading the live corpus IS this census's subject. Relocating the
/// witness out of the long home moves it from one decline arm to the other and never to `Planned`.
/// The route is therefore a MODE, `--required-cited-symbol`, invoked by its own job -- not a claim,
/// so it sits outside both the per-claim safety deadline and the live-tree arm, and not a phase of
/// `--required-ci`, which the operator narrowed to three on 2026-08-21 and which this leaves
/// byte-unchanged.
///
/// Returns the located refusals -- one line per unresolved reference, carrying the typed arm that
/// refused it -- so the failure names what to fix rather than reporting a count.
/// The size of the population the census just checked, so a green names its denominator.
///
/// An empty refusal list is returned both when every reference resolves and when there are no
/// references at all; those are different states and only the first is coverage.
fn cited_symbol_population(ctx: &InterpContext) -> Result<i64, String> {
    match run_value(ctx, "cited_symbol_production_reference_count") {
        Ok(Value::Int(n)) => Ok(n),
        Ok(other) => Err(format!(
            "cited_symbol_production_reference_count must be an Int, got {other:?}"
        )),
        Err(msg) => Err(msg),
    }
}

/// The lens's evaluation context. The caller builds it ONCE and lends it to both readers -- the
/// refusal report and the population count -- which is why they take a `&InterpContext` rather
/// than source roots: two constructions would be two reads of a tree that can change between them,
/// and a denominator is only meaningful for the census it accompanies. An earlier revision said
/// this while both helpers built their own; the comment was true of the intent and false of the
/// code, which is the stale-claim class this very census exists to catch (found in review 54581).
fn cited_symbol_lens_context(source_roots: &[String]) -> Result<(InterpContext, String), String> {
    const LENS_REL: &str = "lens/cited_symbol_resolution.dag";
    let entry = source_roots
        .iter()
        .map(|r| Path::new(r).join(LENS_REL))
        .find(|p| p.exists())
        .ok_or_else(|| {
            format!(
                "cited-symbol: no source root carries {LENS_REL} (roots: {}) -- the census cannot \
                 be run, which is ignorance and not a green",
                source_roots.join(", ")
            )
        })?
        .to_string_lossy()
        .into_owned();
    let (graph, indices) = resolve_entry_graph_shared(source_roots, &entry)
        .map_err(|e| format!("cited-symbol: resolve {entry} failed: {e}"))?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    Ok((ctx, entry))
}

fn cited_symbol_census(ctx: &InterpContext) -> Result<Vec<String>, String> {
    match run_value(ctx, "cited_symbol_unresolved_reference_report") {
        Ok(Value::List(rows)) => {
            let mut out = Vec::new();
            for row in rows.iter() {
                match row {
                    Value::Str(s) => out.push(s.to_string()),
                    other => {
                        return Err(format!(
                            "cited-symbol: report rows must be String, got {other:?} (fail-closed)"
                        ))
                    }
                }
            }
            Ok(out)
        }
        Ok(other) => Err(format!(
            "cited-symbol: cited_symbol_unresolved_reference_report must return a List, got \
             {other:?} (fail-closed)"
        )),
        Err(msg) => Err(format!(
            "cited-symbol: census unavailable (fail-closed): {msg}"
        )),
    }
}

fn behavioral_receipt_selftest(source_roots: &[String]) -> Result<bool, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    // NOT under src/v1: regen seeds every .dag there into the stage0 compile closure, and this
    // authority must never be emitted. Measured, not assumed -- placing it there made the emit
    // produce receipt_fixture.rs with no committed mirror, and required-regen refused the whole
    // surface as a population mismatch.
    let fixture = workspace.join("fixtures/receipt_fixture");
    let module_path = "receipt.fixture";
    let alias = "v1_receipt_fixture";

    let authority = fixture.join("authority.dag");
    let source =
        fs::read_to_string(&authority).map_err(|e| format!("read {}: {e}", authority.display()))?;
    let modules = collect_dag_module_sources(source_roots)?;
    let node = parse_dag_module_node(&format!("{module_path}.dag"), &source)?;
    let types = visible_type_decls(module_path, &source, &modules)?;
    let declared_anywhere = declared_type_names(&modules)?;
    let plan = plan_module_corpus(
        module_path,
        &source,
        &node,
        &types,
        &declared_anywhere,
        alias,
    );

    eprintln!(
        "receipt-selftest: fixture parsed={} derivable={} refused={}",
        plan.parsed_signatures,
        plan.derivable.len(),
        plan.refused.len()
    );
    for (f, d, tuples) in &plan.derivable {
        eprintln!(
            "receipt-selftest:   derivable {f} {} calls={}",
            d.report(),
            tuples.len()
        );
    }
    for (f, why) in &plan.refused {
        eprintln!("receipt-selftest:   REFUSED {f} — {}", why.describe());
    }

    // THE PRECONDITION THE ARMS DEPEND ON, checked before the arms rather than assumed by them.
    //
    // Arm 2 changes `band_of` at exactly one input in all of i64: level = 100. If the boundary
    // enumeration ever regresses to sampling, or to a window that excludes 100, arm 2 goes GREEN
    // and this whole control dies without a sound. So the tuple set is required to CONTAIN that
    // input, and the requirement is stated over the enumerated corpus -- the thing that will
    // actually run -- not over the domain description of it.
    let band_of = plan
        .derivable
        .iter()
        .find(|(f, _, _)| f == "band_of")
        .ok_or_else(|| {
            format!(
                "the fixture's band_of did not derive, so arm 2 could not discriminate even if it \
                 ran. Refusing rather than reporting a control that cannot fail. Refusals: {}",
                plan.refused
                    .iter()
                    .map(|(f, w)| format!("{f}: {}", w.describe()))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })?;
    if !matches!(
        band_of.1,
        EnumeratedDomain::ExhaustiveOverDerivedPartition { .. }
    ) {
        return Err(format!(
            "band_of derived as {}, not over a derived partition. The Int partition is the thing \
             arm 2 exercises; covering it some other way would leave that arm untested",
            band_of.1.report()
        ));
    }
    if !band_of.2.iter().any(|t| t == &vec!["100i64".to_string()]) {
        return Err(format!(
            "the enumerated corpus for band_of does not contain the boundary input 100i64, so arm \
             2's single behavioural difference is outside what the receipt would run. Enumerated: \
             {:?}",
            band_of.2
        ));
    }

    let seed_path = fixture.join("src/lib.rs");
    let mut ok = true;
    let krate = ReceiptCrate::receipt_fixture();
    // The fixture must ADMIT, and it says so here rather than deep inside an arm: a fixture that
    // stopped yielding calls would otherwise turn both arms into an exclusion and this control
    // into a control that cannot fail.
    let admitted = AdmittedPlan::of(&plan).map_err(|c| {
        format!(
            "the fixture yields no call at all, so neither arm could discriminate. Uncovered: {}",
            c.uncovered
                .iter()
                .map(|(f, w)| format!("{f}: {}", w.describe()))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;

    for (arm, candidate_file, expect_equivalent) in [
        ("preserving", "behaviour_preserving.rs", true),
        ("changing", "behaviour_changing.rs", false),
    ] {
        let cand_path = fixture.join("candidates").join(candidate_file);
        let candidate = fs::read_to_string(&cand_path)
            .map_err(|e| format!("read {}: {e}", cand_path.display()))?;
        // An arm whose candidate bytes equal the seed's proves nothing in EITHER direction: the
        // preserving arm would report equivalence for the trivial reason, and the changing arm
        // would report equivalence and be read as a regression. Checked, not trusted.
        let seed = fs::read_to_string(&seed_path)
            .map_err(|e| format!("read {}: {e}", seed_path.display()))?;
        if seed == candidate {
            return Err(format!(
                "arm {arm}: {candidate_file} is byte-identical to the fixture seed, so this arm \
                 compares a file with itself"
            ));
        }

        let verdict =
            behavioral_differential(&workspace, &krate, &seed_path, &candidate, &admitted, alias);
        match (&verdict, expect_equivalent) {
            (
                ReceiptVerdict::Equivalent {
                    calls,
                    nondeterministic_calls,
                    ..
                },
                true,
            ) => {
                // FALSE-POSITIVE CONTROL for the two-run instability probe, and it is the reason
                // this arm now reads a second field. The fixture's corpus is deterministic, so
                // the probe must find NOTHING unstable in it. Without this, a probe that marked
                // every line unstable would still print EQUIVALENT here -- over an empty compared
                // set it would not even reach this arm, but over a partially-marked one it would,
                // and the arm would pass while the gate had quietly stopped comparing anything.
                if *nondeterministic_calls != 0 {
                    eprintln!(
                        "receipt-selftest: arm {arm} EQUIVALENT but the instability probe marked {nondeterministic_calls} call(s) unstable in a DETERMINISTIC fixture — the probe is producing false positives, so its exclusions cannot be trusted"
                    );
                    ok = false;
                } else {
                    eprintln!("receipt-selftest: arm {arm} EQUIVALENT over {calls} derived calls — as required");
                }
            }
            (
                ReceiptVerdict::Divergent {
                    calls,
                    first_difference,
                },
                false,
            ) => {
                // Not merely THAT it diverged: divergence at the wrong call would mean the arm is
                // catching something other than the difference it was authored to catch, and a
                // control that passes for the wrong reason is not a control.
                if !(first_difference.contains("band_of") && first_difference.contains("100i64")) {
                    eprintln!(
                        "receipt-selftest: arm {arm} DIVERGENT over {calls} calls but at the WRONG \
                         call — expected band_of(100i64): {first_difference}"
                    );
                    ok = false;
                } else {
                    eprintln!(
                        "receipt-selftest: arm {arm} DIVERGENT over {calls} derived calls at the \
                         authored difference — {first_difference}"
                    );
                }
            }
            (v, _) => {
                eprintln!(
                    "receipt-selftest: arm {arm} expected {} but got {v:?}",
                    if expect_equivalent {
                        "EQUIVALENT"
                    } else {
                        "DIVERGENT"
                    }
                );
                ok = false;
            }
        }
    }
    Ok(ok)
}

fn run_behavioral_receipt_selftest(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_selftest(source_roots) {
        Ok(true) => Ok(ExitCode::SUCCESS),
        Ok(false) => {
            eprintln!(
                "receipt-selftest: REFUSED — the behavioral receipt's own arms no longer \
                 discriminate. Until this is green, no verdict the mode reports is evidence"
            );
            Err(ExitCode::from(1))
        }
        Err(e) => {
            eprintln!("receipt-selftest: REFUSED — {e}");
            Err(ExitCode::from(1))
        }
    }
}

/// STANDALONE INVOCATION, where an absent subject is a MISUSE rather than a state to report.
///
/// Naming this mode on the command line asserts there is a pull request to check. If the merge
/// base is the head there is not, so the invocation is answered rather than silently succeeding.
/// This mode has no other caller: the 2026-08-21 operator ruling cut the receipt out of
/// `--required-ci`, so nothing invokes it automatically and its only route is someone typing it.
///
/// A DIFF-SUBJECT GATE CANNOT BE EXERCISED BY THE BRANCH IT PROTECTS, and this paragraph is
/// carried here because the property is a fact about THIS gate's subject rather than about the
/// phase that used to run it. It was learned expensively and it survives its enrolment.
///
/// The subject is a diff against the merge base. On main the merge base IS the head, so there is
/// no pull request to check and never was -- which means NO main run, green or otherwise, is
/// evidence about this gate: comparing a PR's red against main's green compares a run against a
/// SKIP, not against a pass. Coverage is entirely PR-side by construction. The consequence that
/// makes it worth writing down: a defect that reds every PR touching one class of authority can
/// sit indefinitely while main stays green, because the only runs that could have seen it are
/// the ones a person reads as "my branch is broken". That is not hypothetical -- it is how the
/// wet-actuator selection defect (gunbc#8704, excluded at selection since) survived.
///
/// SO IT BINDS ANY FUTURE PROPOSAL, not just this one. If a diff-subject gate is ever enrolled
/// in CI again -- this receipt or another -- read this first: it needs a real subject on main
/// (the PUSH RANGE is one, and is a DIFFERENT subject rather than a stand-in for a PR diff), or
/// it is accepting PR-only coverage knowingly. What must NOT move is the absent-subject arm:
/// making it return an answer gives it a deficit frequency of zero by construction, which is the
/// absorbing fallback wearing the fix's clothes (DESIGN section 5).
fn run_behavioral_receipt_plan(source_roots: &[String]) -> Result<ExitCode, ExitCode> {
    match behavioral_receipt_plan(source_roots) {
        Ok(ReceiptPlanOutcome::Ran { agreed: true }) => Ok(ExitCode::SUCCESS),
        Ok(ReceiptPlanOutcome::Ran { agreed: false }) => Err(ExitCode::from(1)),
        Ok(ReceiptPlanOutcome::NoSubject { head }) => {
            eprintln!(
                "behavioral-receipt: NO SUBJECT — the merge base resolves to HEAD ({head}), so \
                 the diff compares this commit against itself and cannot observe what changed. \
                 This is not an empty selection and is not reported as a pass: `nothing changed` \
                 and `I could not see what changed` are different states. This mode's subject is \
                 a pull request against main; invoke it there"
            );
            Err(ExitCode::from(1))
        }
        Err(refusal) => {
            eprintln!("behavioral-receipt: refused: {refusal}");
            Err(ExitCode::from(1))
        }
    }
}

/// What the per-PR receipt run found, as a state rather than a bool.
///
/// `NoSubject` exists because "this run has nothing to check" and "this run checked and agreed"
/// are the two zeros this mode was corrected for once already, one level down. Collapsing them
/// into `Ok(true)` is precisely how the vacuous pass on `push: main` was written.
#[derive(Debug, Clone, PartialEq)]
enum ReceiptPlanOutcome {
    /// The merge base resolves to HEAD, so no diff exists to read. Not a pass, not a failure --
    /// an absent subject.
    NoSubject { head: String },
    /// A selection was computed and every selected module reached a verdict.
    Ran { agreed: bool },
}

fn behavioral_receipt_plan(source_roots: &[String]) -> Result<ReceiptPlanOutcome, String> {
    let workspace = v1_compiler::cli_run::workspace_root();
    let stage0_src = workspace.join("src/v1/stage0/src");

    // BASELINE FIRST, asserted and printed, exactly as the mirror-drift gate does and for the
    // same reason: a selection computed against an unresolvable baseline is ignorance, and the
    // tempting fallback -- treat everything as changed -- is the absorbing arm that turns a
    // per-change gate into a per-corpus one.
    let head = git_stdout(&workspace, &["rev-parse", "HEAD"])?;
    let base = git_stdout(&workspace, &["merge-base", "origin/main", "HEAD"]).map_err(|e| {
        format!(
            "cannot resolve the merge base against origin/main ({e}). The selection is NOT \
             widened to the whole population in this case: `I could not determine what changed` \
             and `everything changed` are different states, and two compiler builds per module \
             across the corpus is a budget breach denominated in the repository rather than in \
             the change. Fetch the base first: \
             `git fetch --depth=200 origin main:refs/remotes/origin/main`"
        )
    })?;
    eprintln!("behavioral-receipt: merge_base={base} head={head}");

    // A BASELINE THAT IS THE HEAD IS NOT AN EMPTY SELECTION -- IT IS NO OBSERVATION AT ALL.
    //
    // On a push to main, `git merge-base origin/main HEAD` resolves to HEAD itself, so the diff
    // compares the commit against itself and yields zero changed authorities. The empty-selection
    // arm below would then report a real pass over a corpus that was never looked at. That is the
    // empty-observation narrow DESIGN names by its live specimen -- a push whose baseline ref IS
    // the pushed ref -- and it is the mirror of the absorbing fallback: a widen is merely
    // expensive, a narrow is silently uncovered.
    //
    // `nothing changed` and `I could not see what changed` are different states with different
    // remedies, so they get different answers. This one refuses, and it names the invocation that
    // makes sense instead of guessing at a substitute baseline: the subject of this gate is a
    // pull request, and a push to a branch that IS the baseline has no such subject.
    if base == head {
        return Ok(ReceiptPlanOutcome::NoSubject { head });
    }

    let changed = git_stdout(
        &workspace,
        &["diff", "--name-only", &base, &head, "--", "*.dag"],
    )?;
    let changed: Vec<String> = changed
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    // Loaded once for the whole run: the type a module compiles against may be declared in any
    // module it transitively imports.
    let modules = collect_dag_module_sources(source_roots)?;
    let declared_anywhere = declared_type_names(&modules)?;

    // Built ONCE, before any authority is classified, and it refuses if it cannot see the whole
    // generated population -- so an exclusion below is a fact about the corpus rather than a fact
    // about what this reader happened to recognise.
    let mirror_index = build_mirror_index(&stage0_src)?;

    let mut exclusions: Vec<ReceiptExclusion> = Vec::new();
    let mut plans: Vec<(String, ModuleCorpusPlan, String)> = Vec::new();
    // BUILT ON FIRST DEMAND, not before the loop. Only a module that yields no call needs to ask
    // the generated-artifact population during selection, which is a small population and often
    // an empty one; resolving its closure unconditionally would put a corpus-sized resolve on
    // every run of a per-change gate -- the cost shape DESIGN §6 names, where the unit of
    // computation is the corpus and the unit of fact is one path.
    let mut generated_ctx_cell: Option<v1_compiler::v1_interpreter::InterpContext> = None;
    // BOTH GRAINS, ACCUMULATED ACROSS EVERY CHANGED AUTHORITY -- including the excluded ones,
    // which is the point: a function that yields no call is uncovered whether or not its module
    // had a sibling that saved it from exclusion.
    let mut declared_functions = 0usize;
    let mut covered_functions = 0usize;

    for rel in &changed {
        let abs = workspace.join(rel);
        let Ok(source) = fs::read_to_string(&abs) else {
            continue;
        };
        let Some(module_path) = v1_compiler::cli_run::extract_module_path_public(&source) else {
            continue;
        };
        match mirror_index.by_module.get(&module_path).cloned() {
            None => exclusions.push(ReceiptExclusion::NoEmittedMirror { module_path }),
            Some(mirror) => {
                // The Rust module path is the mirror's basename without its extension — derived
                // from the artifact, like the mapping that found it, never spelled out here.
                // The full crate path to the emitted mirror, derived from the artifact's own
                // basename. Both the enumerated constructor values and the generated calls are
                // written against THIS string, so there is one spelling of the module under test.
                let alias = format!("v1_compiler::{}", mirror.trim_end_matches(".rs"));
                let node = parse_dag_module_node(&format!("{module_path}.dag"), &source)?;
                let types = visible_type_decls(&module_path, &source, &modules)?;
                let plan = plan_module_corpus(
                    &module_path,
                    &source,
                    &node,
                    &types,
                    &declared_anywhere,
                    &alias,
                );
                // ADMISSION IS DECIDED HERE, AT FUNCTION GRAIN, and it is decided from a fact the
                // source already carries -- no build, no emit, no differential is spent on a
                // module that cannot produce a call. The decision itself is a PURE function, so
                // both of its exclusion arms and its refusal can be executed against hand-built
                // states rather than only against whatever the live corpus happens to hold.
                let coverage = function_grain_coverage(&plan);
                declared_functions += coverage.covered.len() + coverage.uncovered.len();
                covered_functions += coverage.covered.len();
                match plan_grain_selection(&plan, coverage)? {
                    None => plans.push((mirror, plan, alias)),
                    Some(exclusion) => {
                        // A ZERO-CALL MODULE IS NOT AUTOMATICALLY A SUBJECTLESS ONE, and getting
                        // this wrong is how an exclusion becomes the thing it was meant to
                        // prevent. #8753 established the fact by measurement: for a mirror in the
                        // GENERATED-ARTIFACT population the subject is the artifact's BYTES, not
                        // its behaviour, and that population's only drift observer anywhere is the
                        // identity check inside the differential loop. Excluding here on `no
                        // function yields a call` would delete that observer for exactly the
                        // artifacts that have no functions to call -- silently, and while printing
                        // a line that says the module has nothing to compare.
                        //
                        // So the population is ASKED, and it is asked with the same projection
                        // the loop asks (`generated_artifact_body_for_path`), not with a second
                        // roster. `NotGenerated` is a positive answer -- this path is a module
                        // mirror -- and only then is the exclusion a fact.
                        let repo_rel = format!("src/v1/stage0/src/{mirror}");
                        let ctx = generated_artifact_ctx(source_roots, &mut generated_ctx_cell)?;
                        let body = generated_artifact_body_for_path(ctx, &repo_rel)?;
                        match exclusion_survives_generated_artifact_population(exclusion, &body) {
                            Some(exclusion) => exclusions.push(exclusion),
                            None => plans.push((mirror, plan, alias)),
                        }
                    }
                }
            }
        }
    }

    // BOTH GRAINS, SIDE BY SIDE, ON EVERY RUN -- even when they agree (operator directive,
    // 2026-08-21). A module count alone reads as coverage ("3 modules, one pass"); a function
    // count alone reads as what happened but hides how much of the corpus was in scope at all.
    // The gap between the two sentences is the thing worth seeing, and a reader given one number
    // infers the other one wrongly.
    //
    // EXCLUSIONS ARE COUNTED PER ARM, not as one total. A total says how many changed authorities
    // reached no verdict; only the split says WHY, and the arms rank differently: a
    // non-derivability row is work someone can do, an outside-the-population row is a fact no
    // work removes. Collapsing them is how a population that cannot shrink gets read as debt that
    // simply has not been paid.
    let mut excluded_no_mirror = 0usize;
    let mut excluded_no_function_declared = 0usize;
    let mut excluded_no_corpus = 0usize;
    for e in &exclusions {
        match e {
            ReceiptExclusion::NoEmittedMirror { .. } => excluded_no_mirror += 1,
            ReceiptExclusion::NoFunctionDeclared { .. } => excluded_no_function_declared += 1,
            ReceiptExclusion::NoFunctionHasACorpus { .. } => excluded_no_corpus += 1,
        }
    }
    eprintln!(
        "behavioral-receipt: GRAIN module: changed_authorities={} selected={} excluded={} \
         (no-emitted-mirror={excluded_no_mirror} \
         no-function-declared={excluded_no_function_declared} \
         no-function-has-a-corpus={excluded_no_corpus}) | \
         function: declared={declared_functions} covered={covered_functions} uncovered={}",
        changed.len(),
        plans.len(),
        exclusions.len(),
        declared_functions - covered_functions
    );
    for e in &exclusions {
        match e {
            ReceiptExclusion::NoEmittedMirror { module_path } => eprintln!(
                "behavioral-receipt: excluded {module_path} — no emitted mirror in the \
                 generated population names it as its authority, under either header \
                 convention (the index refuses outright if any generated file is \
                 unindexable, so this is a fact about the corpus and not a lookup miss)"
            ),
            ReceiptExclusion::NoFunctionDeclared { module_path } => eprintln!(
                "behavioral-receipt: excluded {module_path} — the authority declares no \
                 functions, so it carries no behaviour that could diverge and there is nothing \
                 for a differential to compare. NOT a derivation deficit: it contributes no \
                 uncovered function to the FUNCTION-GRAIN line, because there is no function to \
                 cover"
            ),
            ReceiptExclusion::NoFunctionHasACorpus {
                module_path,
                uncovered,
            } => {
                eprintln!(
                    "behavioral-receipt: excluded {module_path} — none of its {} declared \
                     functions yields a call, so there is no corpus to compare. NOT a failure of \
                     this diff: the same non-derivability in a module with one derivable sibling \
                     runs and passes, so refusing here would be a verdict decided by where a \
                     file-level count landed. Every uncovered function is named below and counted \
                     in the FUNCTION-GRAIN line",
                    uncovered.len()
                );
                for (f, why) in uncovered {
                    eprintln!(
                        "behavioral-receipt:   uncovered {module_path}::{f} — {}",
                        why.describe()
                    );
                }
            }
        }
    }
    for (_mirror, p, _alias) in &plans {
        // Both counts are coverage claims, so they are reported as the two ways coverage was
        // ESTABLISHED -- closed type versus derived partition -- and not as strong-versus-weak.
        // There is no third number here any more; the bounded column it replaced counted
        // functions that had been sampled, not covered.
        let closed = p
            .derivable
            .iter()
            .filter(|(_, d, _)| matches!(d, EnumeratedDomain::Exhaustive { .. }))
            .count();
        eprintln!(
            "behavioral-receipt: {} fn_lines={} parsed={} derivable={} (closed-type={} derived-partition={}) refused={}",
            p.module_path,
            p.declared_fn_lines,
            p.parsed_signatures,
            p.derivable.len(),
            closed,
            p.derivable.len() - closed,
            p.refused.len()
        );
        for (f, d, _tuples) in &p.derivable {
            eprintln!(
                "behavioral-receipt:   derivable {}::{f} {}",
                p.module_path,
                d.report()
            );
        }
        for (f, why) in &p.refused {
            eprintln!(
                "behavioral-receipt:   REFUSED {}::{f} — corpus not derivable: {}",
                p.module_path,
                why.describe()
            );
        }
    }

    // THE DIFFERENTIAL. Everything above decides WHAT to run; this runs it.
    //
    // The emit happens once for the whole selection rather than once per module: it is the
    // expensive step, and asking for it per candidate would make a two-module change cost twice
    // what a one-module change costs for no additional information.
    if plans.is_empty() {
        eprintln!(
            "behavioral-receipt: no changed authority module reached the differential — every one \
             was excluded above, each under one of the three typed arms counted on the \
             module-grain line. Nothing to compare, so this run costs nothing. That \
             is a real pass over an EMPTY selection, stated rather than printed as a bare PASS, \
             and the FUNCTION-GRAIN counts above say how much surface that silence covers"
        );
        return Ok(ReceiptPlanOutcome::Ran { agreed: true });
    }

    // A DECLARED CAP, REFUSED ABOVE RATHER THAN SAMPLED (operator ruling, 2026-08-20).
    //
    // The differential costs one crate build per selected module plus one for the seed. The
    // tempting arm when a PR touches many authorities is to check the first few and report a pass
    // -- the absorbing fallback exactly: the deficit's frequency goes to zero by construction and
    // nobody learns the gate stopped covering things. So an over-cap selection REFUSES, typed and
    // counted, and an over-cap PR is visible rather than quietly under-checked.
    //
    // WHAT KIND OF NUMBER THIS IS, because a bare literal in a merge-blocking check is exactly
    // what DESIGN §5 tells reviewers to distrust (review 54096 asked, correctly). It is a POLICY
    // BUDGET -- one of the four sanctioned grounds -- and the resource it caps is CI wall clock:
    // each selected module costs a full v1-compiler release build, so four modules is roughly
    // forty minutes on a job that already runs thirty. It is NOT a measurement copied from the
    // tree, and automating its update would not collapse this check to `measure() == measure()`.
    //
    // DISSOLUTION, so it is a policy rather than a bounded scaffold wearing one's clothes: this
    // cap exists only because the differential rebuilds the whole crate per candidate. It is
    // RAISED, not removed, when that stops being true -- the seed transcript is captured once for
    // a whole selection today, but each candidate install still forces a full rebuild, and a
    // per-module compilation unit would make the cost linear in the changed surface rather than
    // in the crate. Until then a four-authority PR is genuinely more than this gate can check,
    // and saying so is the honest answer rather than checking three of four and printing a pass.
    // A PR that legitimately needs more splits, or the operator raises the number here.
    const MAX_SELECTED_MODULES_PER_RUN: usize = 3;
    if plans.len() > MAX_SELECTED_MODULES_PER_RUN {
        eprintln!(
            "behavioral-receipt: REFUSED — {} authority modules selected, above the declared cap \
             of {MAX_SELECTED_MODULES_PER_RUN}. Not sampling the first {MAX_SELECTED_MODULES_PER_RUN}: \
             a partial check reported as a pass is how a gate stops covering things without anyone \
             finding out. Selected: {}",
            plans.len(),
            plans
                .iter()
                .map(|(_, p, _)| p.module_path.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return Ok(ReceiptPlanOutcome::Ran { agreed: false });
    }

    let emitted = v1_compiler::cli_run::emitted_generated_sources()?;
    let mut all_equivalent = true;
    // Accumulated as each module is admitted and printed after every verdict, so the denominator
    // is the SAME partition the admission was decided from. Recomputing it from the plan
    // afterwards would be a second producer of one fact, and the one that gets reported is the
    // one that never gated anything.
    // ONE RESOLVE for the whole run -- see `generated_artifact_body_for_path`'s cost-shape note.
    // The cell may already hold it: selection asks the same authority when a module yields no
    // call, and asking it twice would resolve one closure twice.
    let generated_ctx = generated_artifact_ctx(source_roots, &mut generated_ctx_cell)?;
    let mut denominators: Vec<String> = Vec::new();
    let mut nondeterministic_calls_total = 0usize;
    let mut nondeterministic_modules = 0usize;
    for (mirror, plan, alias) in &plans {
        // WHICH POPULATION OWNS THIS MIRROR, decided before anything is fetched.
        //
        // Two generators write into `src/v1/stage0/src`: the v1 compiler emits module mirrors,
        // and gunbc's artifact emitters write a handful of generated files whose headers say so.
        // This fragment used to ask only the first and refuse when it came back empty, which made
        // the SECOND population permanently unaskable -- any change to the interpreter dispatch
        // roster or the stage0 crate layout redded this run with "a missing candidate is
        // ignorance", correct as written and with no reachable green.
        //
        // The population is asked FIRST and answers positively. This is deliberately not a
        // fallback from the mirror-emit miss: a fallback would make absence in one producer mean
        // presence in the other, so a genuinely unknown path would silently be regenerated from
        // nothing instead of refused. Here each population answers for what it owns, and a path
        // in neither still refuses.
        let repo_rel = format!("src/v1/stage0/src/{mirror}");
        let generated = match generated_artifact_body_for_path(&generated_ctx, &repo_rel) {
            Ok(g) => g,
            Err(e) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — could not ask the generated-artifact \
                     population about {repo_rel}: {e}. Not equivalence: an unanswered question \
                     is ignorance",
                    plan.module_path
                );
                all_equivalent = false;
                continue;
            }
        };
        let owned_candidate: Option<String> = match generated {
            GeneratedArtifactPathBody::Produced(content) => Some(content),
            GeneratedArtifactPathBody::Refused(reason) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — {repo_rel} is a generated artifact and its \
                     generator refused: {reason}",
                    plan.module_path
                );
                all_equivalent = false;
                continue;
            }
            GeneratedArtifactPathBody::NotGenerated => None,
        };
        // A GENERATED ARTIFACT IS CHECKED BY IDENTITY, NOT BY CALLING IT -- and this is the
        // correct check for it, not a weaker stand-in for the differential.
        //
        // MEASURED, after the first draft of this change got it wrong. Producing the candidate
        // was necessary and not sufficient: with the candidate in hand the differential went one
        // step further and refused again, because it compiles a driver that CALLS the authority's
        // declared functions against the mirror --
        //
        //   the driver did not compile against the mirror: error[E0425]: cannot find function
        //   `v1_interpreter_arm_shape_derivability` in module v1_compiler::v1_interpreter_dispatch_generated
        //
        // -- and `v1_interpreter_dispatch_generated.rs` exposes enums and `lookup_*` fns. It is a
        // file DERIVED FROM the authority's data, not a Rust projection of the authority's
        // functions, so those functions are not there and never will be. The differential's
        // precondition (the mirror answers the same calls the authority declares) simply does not
        // hold for this population.
        //
        // For a generated artifact the whole content IS the product, so byte identity between a
        // freshly generated candidate and the committed file is the complete statement of
        // correctness -- which is exactly the drift check that has had no owner since the
        // generated-artifact drift gates were dropped in the floor cut.
        let candidate_source: &String = match owned_candidate.as_ref() {
            Some(candidate) => {
                let committed_path = workspace.join(&repo_rel);
                match fs::read_to_string(&committed_path) {
                    Err(e) => {
                        eprintln!(
                            "behavioral-receipt: {} REFUSED — {repo_rel} is a generated artifact \
                             but its committed bytes could not be read ({e}), so identity cannot \
                             be established",
                            plan.module_path
                        );
                        all_equivalent = false;
                    }
                    Ok(committed_bytes) if committed_bytes == *candidate => {
                        eprintln!(
                            "behavioral-receipt: {} ARTIFACT-IDENTICAL — {repo_rel} regenerates \
                             byte-for-byte from its authority. This is identity, not behavioural \
                             equivalence: the artifact exposes no function this fragment could \
                             call, so its bytes are the whole claim",
                            plan.module_path
                        );
                    }
                    Ok(_) => {
                        eprintln!(
                            "behavioral-receipt: {} ARTIFACT-DRIFT — {repo_rel} does not match \
                             what its authority generates. Regenerate it (main_wet on \
                             dag/tools/generated_artifact_gate.dag) and commit the result",
                            plan.module_path
                        );
                        all_equivalent = false;
                    }
                }
                continue;
            }
            None => match emitted.get(mirror) {
                Some(c) => c,
                None => {
                    eprintln!(
                        "behavioral-receipt: {} REFUSED — {mirror} is in neither population: the \
                         v1 emit produced no mirror for it and it is not a committed generated \
                         artifact. Not equivalence: a missing candidate is ignorance",
                        plan.module_path
                    );
                    all_equivalent = false;
                    continue;
                }
            },
        };
        // Infallible in fact -- selection only pushed plans that admitted -- but derived here
        // rather than asserted, so the differential's precondition is carried by the value it
        // receives instead of by a comment about an earlier loop.
        let admitted = match AdmittedPlan::of(plan) {
            Ok(a) => a,
            Err(coverage) => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — selected but yields no call ({} uncovered \
                     functions). Selection and admission disagree, which is a defect in this \
                     fragment, not in the authority",
                    plan.module_path,
                    coverage.uncovered.len()
                );
                all_equivalent = false;
                continue;
            }
        };
        denominators.push(format!(
            "behavioral-receipt: DENOMINATOR {} — {} derived calls over {} of {} declared \
             functions; the other {} yield no call and are NOT covered by this verdict",
            plan.module_path,
            admitted.coverage.calls(),
            admitted.coverage.covered.len(),
            plan.parsed_signatures,
            admitted.coverage.uncovered.len()
        ));
        match behavioral_differential(
            &workspace,
            &ReceiptCrate::v1_compiler(),
            &workspace.join("src/v1/stage0/src").join(mirror),
            candidate_source,
            &admitted,
            alias,
        ) {
            ReceiptVerdict::Equivalent {
                calls,
                nondeterministic_calls,
                nondeterministic_functions,
            } => {
                if nondeterministic_calls == 0 {
                    eprintln!(
                        "behavioral-receipt: {} EQUIVALENT over {calls} derived calls",
                        plan.module_path
                    );
                } else {
                    eprintln!(
                        "behavioral-receipt: {} EQUIVALENT over {calls} derived calls, with \
                         {nondeterministic_calls} EXCLUDED as nondeterministically rendered — \
                         {}. Those calls were NOT compared, so this green does not cover them",
                        plan.module_path,
                        nondeterministic_functions.join(", ")
                    );
                    nondeterministic_calls_total += nondeterministic_calls;
                    nondeterministic_modules += 1;
                }
            }
            ReceiptVerdict::NondeterministicRendering {
                unstable_calls,
                functions,
            } => {
                eprintln!(
                    "behavioral-receipt: {} NONDETERMINISTIC-RENDERING — all {unstable_calls} \
                     derived call(s) render unstably, so nothing could be compared: {}. This is \
                     a property of what the mirror RETURNS, not a defect in this diff, and it is \
                     not scored as a divergence",
                    plan.module_path,
                    functions.join(", ")
                );
                nondeterministic_calls_total += unstable_calls;
                nondeterministic_modules += 1;
            }
            ReceiptVerdict::Divergent {
                calls,
                first_difference,
            } => {
                eprintln!(
                    "behavioral-receipt: {} DIVERGENT over {calls} derived calls — {first_difference}",
                    plan.module_path
                );
                all_equivalent = false;
            }
            ReceiptVerdict::Refused { reason } => {
                eprintln!(
                    "behavioral-receipt: {} REFUSED — {reason}",
                    plan.module_path
                );
                all_equivalent = false;
            }
        }
    }
    // THE DENOMINATOR, EVERY RUN (operator ruling, 2026-08-20). A green here means the DERIVED
    // CALLS in the selected modules agreed -- never that a module is behaviourally equivalent.
    // Printing a bare PASS is how, inside a week, someone reads this as promotion evidence.
    for line in &denominators {
        eprintln!("{line}");
    }
    // EVERY RUN, INCLUDING ZERO -- a counter that appears only when nonzero teaches a reader that
    // its absence means "not measured", and the two then look alike in a log tail.
    //
    // THE FLOOR IS IN THE LINE, NOT IN A NOTE BESIDE IT. The line outlives the note: someone will
    // trend this number, watch it sit at N, and conclude the class is nearly closed. It is a floor
    // because the probe proves instability by DISAGREEMENT between two runs, and two randomized
    // renderings can coincide -- an uncaught call is scored as an ordinary comparison and, if it
    // then differs across the seed/candidate pair, inflates the DIVERGENT count instead.
    //
    // DISSOLUTION: this goes to zero when emission is deterministic (a `BTreeMap` container
    // template rather than `HashMap`), NOT when the probe gets better at spotting the residue.
    // A shrinking count from a sharper probe would be the metric improving while the defect stays.
    eprintln!(
        "behavioral-receipt: nondeterministic_rendering={nondeterministic_calls_total} call(s) \
         across {nondeterministic_modules} module(s) — FLOOR, not a total: instability is proved \
         by two runs disagreeing, so a call whose randomized rendering happened to agree twice is \
         not counted here and is compared as if it were deterministic. Not failures; each is a \
         subject this fragment cannot ask about until emission is deterministic"
    );
    Ok(ReceiptPlanOutcome::Ran {
        agreed: all_equivalent,
    })
}

#[cfg(test)]
mod driver_transcript_tests {
    use super::*;

    fn lines(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    /// The property the exclusion rests on: two runs of ONE binary that disagree at an index
    /// prove that index is not a function of the program alone.
    #[test]
    fn a_line_that_differs_between_two_runs_is_unstable() {
        let t = DriverTranscript::of(
            lines(&["stable() = 1", "m() = {a, b}", "also_stable() = 2"]),
            lines(&["stable() = 1", "m() = {b, a}", "also_stable() = 2"]),
        );
        assert_eq!(t.unstable, [1].into_iter().collect());
        // The transcript itself is the FIRST run, unchanged -- the probe observes, it does not
        // rewrite what gets compared.
        assert_eq!(
            t.lines,
            lines(&["stable() = 1", "m() = {a, b}", "also_stable() = 2"])
        );
    }

    /// THE FALSE-POSITIVE CONTROL, at unit grain. A deterministic corpus must yield NOTHING
    /// unstable, or every module would silently stop being compared while still printing a green.
    #[test]
    fn two_identical_runs_mark_nothing_unstable() {
        let t = DriverTranscript::of(
            lines(&["a() = 1", "b() = 2"]),
            lines(&["a() = 1", "b() = 2"]),
        );
        assert!(t.unstable.is_empty());
    }

    /// A length difference is instability that belongs to no single index. Marking every line is
    /// the fail-closed reading; comparing the common prefix would silently compare a shifted pair.
    #[test]
    fn a_length_difference_marks_every_line() {
        let t = DriverTranscript::of(lines(&["a() = 1"]), lines(&["a() = 1", "b() = 2"]));
        assert_eq!(t.unstable, [0, 1].into_iter().collect());
    }

    /// THE RESIDUE, ASSERTED SO IT IS NOT MISTAKEN FOR A CLOSED CLASS. Two randomized renderings
    /// can coincide; when they do, the probe cannot see it and the call is compared as if it were
    /// deterministic. This test PINS that limitation rather than hiding it -- if someone later
    /// makes the probe complete, this test fails and forces the FLOOR wording to be revisited.
    #[test]
    fn a_nondeterministic_call_that_agreed_twice_is_not_caught() {
        let t = DriverTranscript::of(lines(&["m() = {a, b}"]), lines(&["m() = {a, b}"]));
        assert!(t.unstable.is_empty());
    }
}

#[cfg(test)]
mod function_grain_admission_tests {
    use super::*;

    fn plan(derivable: Vec<(&str, usize)>, refused: Vec<(&str, RefusalCause)>) -> ModuleCorpusPlan {
        let declared = derivable.len() + refused.len();
        ModuleCorpusPlan {
            module_path: "test.module".to_string(),
            derivable: derivable
                .into_iter()
                .map(|(name, calls)| {
                    (
                        name.to_string(),
                        EnumeratedDomain::Exhaustive { cardinality: calls },
                        (0..calls).map(|i| vec![format!("{i}i64")]).collect(),
                    )
                })
                .collect(),
            refused: refused
                .into_iter()
                .map(|(name, cause)| (name.to_string(), cause))
                .collect(),
            declared_fn_lines: declared,
            parsed_signatures: declared,
        }
    }

    fn unbounded() -> RefusalCause {
        RefusalCause::UnboundedString {
            ty: "String".to_string(),
        }
    }

    /// BOTH ARMS OF THE ONE DEFECT, IN ONE TEST, and it goes green only when BOTH are closed.
    ///
    /// The specimen is gunbc#8704, which tripped both in a single run:
    /// `v2.compiler.self_host.stage0_crate_layout` (a wet-actuator mirror the emit never writes)
    /// and `gunbc.stage0_crate_layout_generated` (zero declared functions). A fix closing one
    /// half leaves a live red reachable by the very change that motivated it, so the assertions
    /// are stated together rather than in two tests that could be satisfied separately.
    #[test]
    fn a_zero_function_authority_is_excluded_as_such_and_not_as_a_derivation_deficit() {
        let mut p = plan(vec![], vec![]);
        p.declared_fn_lines = 0;
        p.parsed_signatures = 0;
        let coverage = function_grain_coverage(&p);
        match plan_grain_selection(&p, coverage).expect("a module with no functions is a fact") {
            Some(ReceiptExclusion::NoFunctionDeclared { module_path }) => {
                assert_eq!(module_path, "test.module")
            }
            other => panic!(
                "an authority declaring no function has no behaviour that could diverge, so it \
                 must be excluded AS SUCH -- reporting it as `none of its 0 declared functions \
                 yields a call` sends a reader to close a derivation with no subject: {other:?}"
            ),
        }
    }

    /// THE PLANTED CONTROL FOR AN EMPTY POPULATION, and it is planted precisely BECAUSE the
    /// population is empty. No authority in the corpus today declares zero functions AND owns a
    /// generated artifact, so nothing live exercises this branch -- which means main and this
    /// branch disagree about it with no measurement in either direction, and the first real member
    /// would decide the verdict by accident. This test is the statement of intent, executing:
    ///
    ///   such a module is SELECTED, not excluded, because its subject is the artifact's BYTES.
    ///
    /// The identity check in the differential loop calls no function, so "declares no function"
    /// says nothing about whether it can be checked -- and that check is the only drift observer
    /// those artifacts have had since the floor cut removed the generated-artifact drift gates
    /// (DESIGN.md names them first in what the cut left unguarded). Excluding here would delete
    /// it. Per DESIGN.md 4b(4) this control stays enrolled once the population is non-empty; it is
    /// the evidence that the higher rung is real, not scaffolding for its absence.
    #[test]
    fn a_zero_function_authority_owning_a_generated_artifact_is_selected_not_excluded() {
        let excluded = ReceiptExclusion::NoFunctionDeclared {
            module_path: "test.module".to_string(),
        };
        let survives = exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::Produced("generated bytes".to_string()),
        );
        assert!(
            survives.is_none(),
            "an artifact-owning authority has a subject -- its bytes -- and excluding it would \
             delete the only drift observer those artifacts have"
        );
    }

    /// The same for a generator that REFUSED: the path still belongs to the artifact population,
    /// and the loop is where that refusal is reported. Swallowing it here would turn a generator
    /// failure into a quiet exclusion -- a refusal downgraded to a skip.
    #[test]
    fn a_generator_refusal_does_not_become_an_exclusion() {
        let excluded = ReceiptExclusion::NoFunctionHasACorpus {
            module_path: "test.module".to_string(),
            uncovered: vec![("alpha".to_string(), unbounded())],
        };
        assert!(exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::Refused("generator said no".to_string()),
        )
        .is_none());
    }

    /// THE OTHER SIDE OF THE SAME CONTROL, without which the two above could be satisfied by never
    /// excluding anything: an ordinary module mirror answers `NotGenerated`, and there the
    /// exclusion IS the fact and must survive.
    #[test]
    fn an_ordinary_mirror_keeps_its_exclusion() {
        let excluded = ReceiptExclusion::NoFunctionDeclared {
            module_path: "test.module".to_string(),
        };
        match exclusion_survives_generated_artifact_population(
            excluded,
            &GeneratedArtifactPathBody::NotGenerated,
        ) {
            Some(ReceiptExclusion::NoFunctionDeclared { module_path }) => {
                assert_eq!(module_path, "test.module")
            }
            other => panic!("a module mirror with no function has no subject at all: {other:?}"),
        }
    }

    /// THE FALSE-POSITIVE CONTROL for the arm above, read beside it: a module that DOES declare
    /// functions, none of them derivable, must still land on the derivation-deficit arm. Without
    /// this, a fix could satisfy the test above by sending every uncovered module to the new arm
    /// and the deficit population would silently go to zero.
    #[test]
    fn a_module_whose_functions_all_refuse_stays_a_derivation_deficit() {
        let p = plan(vec![], vec![("alpha", unbounded())]);
        let coverage = function_grain_coverage(&p);
        match plan_grain_selection(&p, coverage).expect("non-derivability is a fact, not ignorance")
        {
            Some(ReceiptExclusion::NoFunctionHasACorpus { uncovered, .. }) => {
                assert_eq!(uncovered.len(), 1)
            }
            other => panic!("a declared-but-underivable function is a rankable deficit: {other:?}"),
        }
    }

    /// A module with one derivable function is SELECTED -- the arm that must not be swallowed by
    /// either exclusion above.
    #[test]
    fn a_module_with_a_derivable_function_is_selected() {
        let p = plan(vec![("alpha", 2)], vec![("beta", unbounded())]);
        let coverage = function_grain_coverage(&p);
        assert!(plan_grain_selection(&p, coverage)
            .expect("a derivable function is a subject")
            .is_none());
    }

    /// THE READER-BLINDNESS ARM. `fn ` lines declared and no signature parsed is a disagreement
    /// between two readers of one fact; answering it with an exclusion would publish this
    /// fragment's own failure as a property of the authority, so it refuses.
    #[test]
    fn a_parse_that_sees_none_of_the_declared_functions_refuses_rather_than_excluding() {
        let mut p = plan(vec![], vec![]);
        p.declared_fn_lines = 7;
        p.parsed_signatures = 0;
        let coverage = function_grain_coverage(&p);
        let refusal = plan_grain_selection(&p, coverage)
            .expect_err("two readers disagreeing is ignorance, not a fact about the module");
        assert!(
            refusal.contains("readers disagree") && refusal.contains("test.module"),
            "the refusal must name what it refused and why: {refusal}"
        );
    }

    /// THE DISCRIMINATING RED for the whole change. A module whose every declared function refuses
    /// derivation used to reach the differential and come back `Refused`, which made the diff that
    /// touched it hard-fail required CI. It must now fail admission instead -- and name every
    /// function, because a module-level "nothing derived" is not something an author can act on.
    #[test]
    fn a_module_with_no_derivable_function_is_excluded_not_refused() {
        let p = plan(vec![], vec![("alpha", unbounded()), ("beta", unbounded())]);
        let coverage = AdmittedPlan::of(&p).err().expect(
            "a module yielding no call must not be admissible to a differential that would \
             compare a program against itself over an empty transcript",
        );
        let named: Vec<&str> = coverage.uncovered.iter().map(|(f, _)| f.as_str()).collect();
        assert_eq!(named, vec!["alpha", "beta"]);
        assert_eq!(coverage.calls(), 0);
    }

    /// THE POSITIVE CONTROL, and the one that makes the red above load-bearing rather than a
    /// mechanism that refuses everything. One derivable function is enough to admit, and the
    /// refused sibling stays counted as uncovered rather than disappearing into the pass.
    #[test]
    fn one_function_with_a_call_admits_and_the_rest_stay_counted() {
        let p = plan(vec![("alpha", 3)], vec![("beta", unbounded())]);
        let admitted = AdmittedPlan::of(&p).expect("one function with calls must admit");
        assert_eq!(admitted.coverage.covered, vec![("alpha".to_string(), 3)]);
        assert_eq!(admitted.coverage.calls(), 3);
        assert_eq!(
            admitted
                .coverage
                .uncovered
                .iter()
                .map(|(f, _)| f.as_str())
                .collect::<Vec<_>>(),
            vec!["beta"],
            "a function nothing ran must remain visible in the denominator of a passing module"
        );
    }

    /// THE SUBTLER HALF: a function that derived a domain containing nothing sits in `derivable`
    /// and contributes no call. Counting it as covered is a zero that reads as success. It is
    /// uncovered, with its OWN cause -- separate from the never-derivable ones, whose remedy is a
    /// different piece of work.
    #[test]
    fn empty_derived_domain_is_uncovered_not_covered() {
        let p = plan(vec![("alpha", 0)], vec![("beta", unbounded())]);
        assert_eq!(
            p.derivable.len(),
            1,
            "precondition: the plan does record this function as derivable, which is exactly why \
             counting that vector would over-report coverage"
        );
        let coverage = AdmittedPlan::of(&p)
            .err()
            .expect("a derivable function with an empty domain yields no call, so nothing admits");
        assert_eq!(
            coverage.uncovered,
            vec![
                ("alpha".to_string(), RefusalCause::EmptyDerivedDomain),
                ("beta".to_string(), unbounded()),
            ],
            "the two causes must stay distinguishable: one needs an enumerator fixed, the other \
             needs a type grounded"
        );
    }

    /// A module carrying an empty-domain function BESIDE a real one still admits, and the empty
    /// one is not silently promoted into the covered count by the module having passed.
    #[test]
    fn an_empty_domain_function_does_not_ride_a_covered_sibling() {
        let p = plan(vec![("alpha", 2), ("empty", 0)], vec![]);
        let admitted = AdmittedPlan::of(&p).expect("alpha yields calls, so the module admits");
        assert_eq!(admitted.coverage.covered, vec![("alpha".to_string(), 2)]);
        assert_eq!(
            admitted.coverage.uncovered,
            vec![("empty".to_string(), RefusalCause::EmptyDerivedDomain)]
        );
    }
}
