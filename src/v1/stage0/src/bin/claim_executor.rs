#![allow(clippy::disallowed_macros)]

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use v1_compiler::cli_run::PhaseProfile;

// HAND-RUST GATE explicit deferral. Lane: required-ci-measurement-host-realization. The authority
// is `v2.workflow.required_ci_measurement`; this seed code realizes its filesystem write, JSON
// transport, process exit and existing required-phase host diagnostics because required CI runs
// the bootstrapped `claim_executor` before a generated replacement owns those effects. This adds
// no competing domain model: the coproduct, blocker fields and build-unreached JSON originate in
// `.dag`. It dissolves at the concrete ROADMAP row `v1-zero-hand-maintained-rust`, whose boundary
// requires every tracked Rust file to be generated or deleted; at that row this realization is
// generated from the measurement model or removed with the v1 seed. Until then this is counted
// hand-maintained bootstrap surface, not a terminal Rust authority.
const REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION: u8 = 1;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "standing", rename_all = "snake_case")]
enum RequiredCiMeasurementReceipt {
    MeasurementCompleted { blockers: Vec<RequiredCiBlocker> },
    MeasurementUnreached { cause: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
struct RequiredCiBlocker {
    phase: String,
    identity: String,
    cause: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VersionedRequiredCiMeasurementReceipt {
    version: u8,
    receipt: RequiredCiMeasurementReceipt,
}

fn write_required_ci_measurement_receipt(
    path: &str,
    receipt: RequiredCiMeasurementReceipt,
) -> Result<(), String> {
    let encoded = serde_json::to_vec_pretty(&VersionedRequiredCiMeasurementReceipt {
        version: REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION,
        receipt,
    })
    .map_err(|e| format!("encode required CI measurement receipt: {e}"))?;
    std::fs::write(path, encoded)
        .map_err(|e| format!("write required CI measurement receipt {path}: {e}"))
}

fn adjudicate_required_ci_measurement_receipt(path: &str) -> Result<ExitCode, ExitCode> {
    let body = std::fs::read(path).map_err(|e| {
        eprintln!("required-ci: adjudication REFUSED receipt unreadable path={path} cause={e}");
        ExitCode::from(1)
    })?;
    let versioned: VersionedRequiredCiMeasurementReceipt =
        serde_json::from_slice(&body).map_err(|e| {
            eprintln!("required-ci: adjudication REFUSED receipt malformed path={path} cause={e}");
            ExitCode::from(1)
        })?;
    if versioned.version != REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION {
        eprintln!(
            "required-ci: adjudication REFUSED receipt version={} expected={}",
            versioned.version, REQUIRED_CI_MEASUREMENT_RECEIPT_VERSION
        );
        return Err(ExitCode::from(1));
    }
    match versioned.receipt {
        RequiredCiMeasurementReceipt::MeasurementUnreached { cause } => {
            eprintln!(
                "required-ci: adjudication REFUSED standing=measurement_unreached cause={cause}"
            );
            Err(ExitCode::from(1))
        }
        RequiredCiMeasurementReceipt::MeasurementCompleted { blockers } if blockers.is_empty() => {
            eprintln!("required-ci: adjudication PASSED standing=measurement_completed blockers=0");
            Ok(ExitCode::SUCCESS)
        }
        RequiredCiMeasurementReceipt::MeasurementCompleted { blockers } => {
            for blocker in &blockers {
                eprintln!(
                    "required-ci: adjudication BLOCKING phase={} identity={} cause={}",
                    blocker.phase, blocker.identity, blocker.cause
                );
            }
            eprintln!(
                "required-ci: adjudication REFUSED standing=measurement_completed blockers={}",
                blockers.len()
            );
            Err(ExitCode::from(1))
        }
    }
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

const FLOOR_WORKER_TERMINAL_ENV: &str = "GUNBC_FLOOR_WORKER_TERMINAL_RECEIPT";
const FLOOR_PHASE_JOURNAL_ENV: &str = "GUNBC_FLOOR_PHASE_JOURNAL";

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

fn run() -> Result<ExitCode, ExitCode> {
    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut verify_artifacts: Vec<String> = Vec::new();
    let mut verify_artifacts_mode = false;
    let mut required_floor_mode = false;
    let mut required_ci_mode = false;
    let mut required_ci_measurement_receipt: Option<String> = None;
    let mut required_ci_adjudicate_receipt: Option<String> = None;
    let mut required_ci_unreached_receipt: Option<String> = None;
    let mut required_ci_unreached_cause: Option<String> = None;
    let mut required_ci_lane: Option<RequiredCiLane> = None;
    let mut required_v2_emission_mode = false;
    let mut required_emit_compile_mode = false;
    let mut required_regen_mode = false;
    let mut emit_partition_crates_mode = false;
    let mut emit_partition_crates_write = false;
    let mut required_regen_fixed_point_mode = false;
    let mut regen_round_cost_mode = false;
    let mut regen_affected_set_mode = false;
    let mut regen_affected_scope = false;
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
            "--measurement-receipt" => {
                i += 1;
                required_ci_measurement_receipt =
                    Some(require_value(&args, i, "--measurement-receipt")?);
            }
            "--adjudicate-measurement-receipt" => {
                i += 1;
                required_ci_adjudicate_receipt =
                    Some(require_value(&args, i, "--adjudicate-measurement-receipt")?);
            }
            "--measurement-unreached-receipt" => {
                i += 1;
                required_ci_unreached_receipt =
                    Some(require_value(&args, i, "--measurement-unreached-receipt")?);
            }
            "--measurement-unreached-cause" => {
                i += 1;
                required_ci_unreached_cause =
                    Some(require_value(&args, i, "--measurement-unreached-cause")?);
            }
            // NAMES A JOB, NOT A PHASE SET. The workflow asks for a lane and this binary decides
            // which phases that lane owns, so widening or re-routing the roster is a change here
            // and never an edit to a YAML step list.
            "--required-lane" => {
                i += 1;
                let value = require_value(&args, i, "--required-lane")?;
                match RequiredCiLane::parse(&value) {
                    Ok(lane) => required_ci_lane = Some(lane),
                    Err(e) => {
                        eprintln!("{e}");
                        return Err(ExitCode::from(2));
                    }
                }
            }
            "--required-v2-emission" => {
                required_v2_emission_mode = true;
            }
            // THE PHASE AS ITS OWN ENTRY POINT, for the reason `--required-v2-emission` is one:
            // running this alone is a real local action, and it runs the SAME producer the
            // required phase runs, so a green here and a green there cannot be two facts.
            "--required-emit-compile" => {
                required_emit_compile_mode = true;
            }
            "--required-regen" => {
                required_regen_mode = true;
            }
            // THE SANCTIONED PRODUCER for the derived partition's boundary files. Named by the
            // phase's own refusal, because a stop whose only remedy does not exist is what
            // produced this class in the first place.
            "--emit-partition-crates" => {
                emit_partition_crates_mode = true;
            }
            "--write" => {
                emit_partition_crates_write = true;
            }
            "--required-regen-fixed-point" => {
                required_regen_fixed_point_mode = true;
            }
            // ONE PRICED REGEN ROUND: seed build, the same `--required-regen` emit, install of
            // what drifted, rebuild from the installed seed, diff — every phase on two clocks,
            // rendered by `gunbc.regen_round_cost`. It installs into src/v1/stage0/src, which
            // is what a round IS; the receipt names the tree and every installed path.
            "--regen-round-cost" => {
                regen_round_cost_mode = true;
            }
            // THE AFFECTED SET OF THE FLOOR'S DIFF RANGE: which committed mirrors can change
            // for the .dag modules this edit touched, as `gunbc.regen_affected_set` bounds it.
            // Reports; it installs nothing. An edited path the tree cannot name refuses.
            "--regen-affected-set" => {
                regen_affected_set_mode = true;
            }
            // CONSUME the bound rather than report it: the round adjudicates, writes and digests
            // only the mirrors `--regen-affected-set` names for this edit, and REFUSES when that
            // bound cannot locate an edited path. Opt-in and narrowing: without it the round is
            // whole-population, which is what the required CI phase runs and what establishes
            // the fixed point a scoped round starts from (v2.workflow.required_regen
            // RegenEmissionScope).
            "--regen-affected-scope" => {
                regen_affected_scope = true;
            }
            "--regen-candidate-dir" => {
                i += 1;
                regen_candidate_dir = require_value(&args, i, "--regen-candidate-dir")?;
            }
            "--regen-receipt" => {
                i += 1;
                regen_receipt_path = require_value(&args, i, "--regen-receipt")?;
            }
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

    if let Some(path) = required_ci_adjudicate_receipt {
        return adjudicate_required_ci_measurement_receipt(&path);
    }
    if let Some(path) = required_ci_unreached_receipt {
        let cause = required_ci_unreached_cause
            .unwrap_or_else(|| "measurement process exited before producing a receipt".to_string());
        return write_required_ci_measurement_receipt(
            &path,
            RequiredCiMeasurementReceipt::MeasurementUnreached { cause },
        )
        .map(|_| ExitCode::SUCCESS)
        .map_err(|e| {
            eprintln!("required-ci: measurement-unreached receipt REFUSED {e}");
            ExitCode::from(1)
        });
    }

    // ORDERED AHEAD OF THE SOURCE-ROOT REQUIREMENT, for the reason `--verify-build-artifacts`
    // is: this mode takes NO roots. It renders from the emitted
    // authority carrier and reads only the files it is adjudicating, so the generic guard below
    // would refuse the only invocation it has -- a mode that could never have run once.
    // MEASURED, NOT REASONED -- the first remote run of this entry point exited 2 with
    // `provide at least one --source-root` before the branch was moved here.
    //
    // THE PRODUCER, AS ITS OWN ENTRY POINT. `--write` installs; without it the run reports and
    // changes nothing, so the read-only form is safe to run anywhere and the `written` list is
    // empty BY CONSTRUCTION rather than by a caller remembering not to ask.
    if emit_partition_crates_mode {
        let outcome =
            v1_compiler::cli_run::run_partition_crate_boundary(emit_partition_crates_write);
        match &outcome {
            v1_compiler::cli_run::PartitionCrateBoundaryOutcome::CarrierRefused { cause } => {
                eprintln!("emit-partition-crates: CarrierRefused cause={cause}");
                return Err(ExitCode::from(1));
            }
            v1_compiler::cli_run::PartitionCrateBoundaryOutcome::Rendered { files, written } => {
                for file in files {
                    eprintln!(
                        "emit-partition-crates: {} {}",
                        file.disposition().name(),
                        file.path
                    );
                }
                eprintln!(
                    "emit-partition-crates: rendered={} written={} mode={}",
                    files.len(),
                    written.len(),
                    if emit_partition_crates_write {
                        "write"
                    } else {
                        "read-only"
                    }
                );
                // A READ-ONLY RUN THAT FOUND DRIFT EXITS NONZERO; the write form is what closes
                // it. Exiting zero here would make the producer's own entry point disagree with
                // the phase that names it as the remedy.
                let unresolved = outcome.divergent().len();
                return if emit_partition_crates_write || unresolved == 0 {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                };
            }
        }
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
    // THE COMPOSED CI RUN — one process per LANE. The roster is five phases: the .dag parse
    // sweep, namespace wave admission, the regen first-generation comparison,
    // generated-artifact drift and the witness floor. `--required-lane` selects which of them
    // this process owns; with no lane, it owns all five, which is what a local run wants.
    //
    // WHAT IT IS AND IS NOT. Sequencing a program's phases is the program's job (DESIGN §3: the
    // workflow is a realization of the intent, not the place the intent lives), so the order
    // lives here rather than in a YAML step list whose preconditions read each other's
    // `outcome`. What is NOT here is a judgement about which checks CI ought to run: the roster
    // is an operator decision, and the 2026-08-21 ruling set it to three, with v2-emission
    // added on 2026-08-23.
    //
    // A LANE IS NOT A NARROWING OF THE ROSTER. Both lanes run in the same required workflow, on
    // every push and pull request, and every phase is owned by exactly one of them
    // (`RequiredCiPhase::lane`, an exhaustive match). What the lane selects is which JOB takes a
    // phase, so that independent phases stop queueing behind each other -- see the roster block
    // beside `RequiredCiLane` for why that boundary is a job boundary of necessity.
    //
    // WHAT WAS DELETED AND WHY IT IS NOT LEFT AS A SKIPPED PHASE (operator ruling, 2026-08-21).
    // Five phases previously ran inside this fold: merge-admission-capture, regen-determinism
    // (the fixed point), receipt-selftest, receipt-vs-changed-authorities, and
    // merge-admission-stamp. They are GONE from this mode — not disabled behind a flag, not
    // reported as SKIPPED — because a phase that always reports the same non-verdict is the
    // absorbing fallback wearing a phase's clothes (DESIGN §5): its deficit frequency is zero by
    // construction and it reads as coverage on the ledger. The capabilities themselves survive
    // where they had their own entry points and consumers: `--required-regen-fixed-point`,
    // The three behavioral producers now run only through their `gunbc test` labels. Their old
    // flags are deleted in the same change as those bindings, so no dual-authority interval exists.
    //
    // WHAT THAT COSTS, named rather than left to be rediscovered. No CI run now measures regen
    // DETERMINISM (that the emitter reproduces its own output), behavioural equivalence of a
    // changed authority against its committed mirror, or the receipt machinery's own
    // discriminating arms; and no run mints a merge-admission receipt. The merge-admission
    // consumers refuse on a missing receipt rather than admitting on one (DESIGN, the CI
    // paragraph's re-add queue), so nothing is admitted by the absence — but the three
    // measurements above are simply not taken, which is a declared rung drop, not a silent one.
    //
    // WHAT THE ORDER IS, AND WHY EACH PHASE RUNS ANYWAY. The five phases are independent —
    // the one real data dependency, the fixed point's need for regen's pass-1 digest, went with
    // the phase that consumed it — so every phase RUNS EVEN AFTER AN EARLIER FAILURE and the run
    // reports the complete ledger instead of letting the first defect hide the rest. The line
    // still stops (a nonzero exit on any failed phase); it stops with every deficit named. This
    // is the stopped-line AUDIT DESIGN §5 sanctions: it reports, it never greens.
    if required_ci_mode {
        let mut phase_failures: Vec<String> = Vec::new();
        let mut measurement_blockers: Vec<RequiredCiBlocker> = Vec::new();
        let mut measurement_unreached: Option<String> = None;
        let mut ran: Vec<&'static str> = Vec::new();
        // THE ONE PARSE, HELD FOR ITS SECOND CONSUMER. The wave-admission phase below reads
        // the index the parse phase built rather than acquiring the corpus again; holding it
        // in an `Option` also keeps `the parse refused` distinguishable from `the parse ran
        // and found nothing`, which is what the wall's own `NotEvaluated` arm exists to keep
        // apart one level down.
        let mut head_index: Option<v1_compiler::cli_run::declaration_index::DeclarationIndex> =
            None;

        // THE ROSTER IS ANNOUNCED BEFORE ANY PHASE RUNS, whole, with each phase's owning lane.
        // A reader of one job's log sees the complete required roster and where the phases this
        // job does not own are being measured, so a lane's coverage is legible from inside it
        // rather than only by holding two logs side by side.
        eprintln!(
            "required-ci: lane={}",
            required_ci_lane
                .map(|l| l.name())
                .unwrap_or("all (no --required-lane given)")
        );
        for phase in REQUIRED_CI_PHASES {
            if !required_ci_phase_selected(phase, required_ci_lane) {
                eprintln!(
                    "required-ci: phase {} ROUTED to lane {} (not this job)",
                    phase.name(),
                    phase.lane().name()
                );
            }
        }

        // PHASE 1 — the .dag parse sweep, over every authored root (src/v1, dag, src/v2).
        // Independent of everything below it. The roster is
        // `cli_run::DAG_PARSE_SWEEP_ROOTS`, shared with the standalone bin so the cheapest
        // local check and this phase cover the same files.
        if required_ci_phase_selected(RequiredCiPhase::Parse, required_ci_lane) {
            eprintln!(
                "required-ci: phase parse (.dag: {})",
                v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS.join(", ")
            );
            match v1_compiler::cli_run::run_dag_parse_sweep(
                &v1_compiler::cli_run::workspace_root(),
                &v1_compiler::cli_run::DAG_PARSE_SWEEP_ROOTS,
            ) {
                Ok(sweep) => {
                    eprintln!(
                        "required-ci: parse OK {} file(s) parse-clean",
                        sweep.parse_clean
                    );
                    head_index = Some(sweep.index.clone());
                    // THE DECLARATION INTEGRITY CHECKS RIDE THE PARSE THAT JUST RAN.
                    //
                    // They are reported inside this phase rather than as a phase of their own
                    // because they are not a second pass over anything: the index was built by
                    // insertion from the sweep above, so there is no walk to order, nothing to
                    // schedule, and no second acquisition of the corpus. DESIGN §6 and §3's
                    // cited-symbol row both name exactly this — one module's facts from one
                    // module's source, at ingestion, instead of a corpus-wide job per question.
                    let population =
                        v1_compiler::cli_run::declaration_index::index_population(&sweep.index);
                    let findings =
                        v1_compiler::cli_run::declaration_index::corpus_findings(&sweep.index);
                    // A GREEN NAMES ITS DENOMINATORS. `checked=0` and `all clean` are different
                    // states with different remedies, and an instrument that renders them
                    // identically is the failure DESIGN §5 names, not a tidy report.
                    eprintln!(
                        "required-ci: declarations modules={} declared={} import_members={} \
                         citations={} debt={} in_fixtures={} outside_index={} kernel_named={} lens_modules={} \
                         cited_authorities_without_import_edges={} retained={:?} \
                         cited_and_called={} called_retained={:?} cited_not_called={} not_called_retained={:?} \
                         primary_dag_entries={} primary_dag_retained={:?} src_v2_pool_only={} pool_only_retained={:?}",
                        population.modules,
                        population.declarations,
                        population.import_members,
                        population.citations,
                        population.citations_pre_existing_debt,
                        population.citations_in_fixtures,
                        population.citations_outside_index,
                        population.import_members_kernel_named,
                        population.lens_modules,
                        population.cited_authorities_without_import_edges.len(),
                        population.cited_authorities_without_import_edges,
                        population.cited_and_called_without_import_edges.len(),
                        population.cited_and_called_without_import_edges,
                        population.cited_not_called_without_import_edges.len(),
                        population.cited_not_called_without_import_edges,
                        population.cited_authorities_under_primary_dag_entry_root.len(),
                        population.cited_authorities_under_primary_dag_entry_root,
                        population.cited_authorities_in_src_v2_dependency_pool_only.len(),
                        population.cited_authorities_in_src_v2_dependency_pool_only,
                    );
                    for finding in &findings {
                        eprintln!(
                            "required-ci: declarations FAIL {}",
                            v1_compiler::cli_run::declaration_index::render_finding(
                                &v1_compiler::cli_run::workspace_root(),
                                finding
                            )
                        );
                    }
                    if !findings.is_empty() {
                        phase_failures
                            .push(format!("declarations ({} finding(s))", findings.len()));
                    }
                    // THE PHASE ROSTER JOIN RIDES THE SAME INGESTION. The substrate authority
                    // for phase identity is `gunbc.required_ci_phase_roster` `RequiredCiPhase`;
                    // this enum is its declared parallel realization, and the two are joined by
                    // variant-set equality in both directions on every required run — the same
                    // shape as the wave wall's vocabulary join, for the same DESIGN §3 reason.
                    // A phase declared and not realized, or realized and not declared, stops
                    // the line here rather than diverging silently.
                    let roster_findings = phase_roster_findings(&sweep.index);
                    for finding in &roster_findings {
                        eprintln!("required-ci: phase-roster FAIL {finding}");
                    }
                    if !roster_findings.is_empty() {
                        phase_failures.push(format!(
                            "phase-roster ({} finding(s))",
                            roster_findings.len()
                        ));
                    }
                }
                Err(errors) => {
                    for e in &errors {
                        eprintln!("required-ci: parse FAIL {e}");
                    }
                    phase_failures.push(format!("parse ({} error(s))", errors.len()));
                }
            }
            ran.push("parse");
        }

        // PHASE — THE NAMESPACE WAVE-ADMISSION WALL.
        //
        // WHAT IT GATES AND WHY IT IS REQUIRED. `gunbc.compiler_frontend_program_interlock`
        // (operator ruling, 2026-08-26) makes the import/namespace plan's disclosed "no CI
        // mechanism" gap a BLOCKER rather than a disclosure: no change that can alter which
        // modules enter a subject, or what an occurrence denotes, may merge before this wall
        // exists, and `milestone_prerequisites` gates `NamespaceFirstSemanticWave` on
        // `NamespaceWaveAdmissionEnrolled` by name.
        //
        // IT REPORTS ITS OWN NON-VERDICTS UNDER THEIR OWN NAMES. `NoSubject` (a push whose
        // baseline is its own head) and `NotEvaluated` (a baseline that does not resolve) are
        // printed as themselves and never as an admission -- and only the first of them
        // passes, because "nothing to compare" and "could not compare" are the two zeros this
        // repository has already been corrected for once.
        if required_ci_phase_selected(RequiredCiPhase::NamespaceWaveAdmission, required_ci_lane) {
            eprintln!("required-ci: phase namespace-wave-admission (closure, subject membership, binding)");
            match &head_index {
                // THE PARSE REFUSED, SO THERE IS NO HEAD TO ADJUDICATE AGAINST. This is not
                // silence: the parse phase has already stopped the line, and adjudicating a
                // corpus half of which failed to parse would report a smaller delta than the
                // one that exists.
                None => {
                    eprintln!(
                        "required-ci: namespace-wave-admission NOT RUN — the parse phase did not \
                         produce an index (it refused, or this lane does not own it)"
                    );
                    phase_failures.push("namespace-wave-admission (no head index)".to_string());
                }
                Some(index) => {
                    // THE VOCABULARY JOIN RUNS FIRST, because every verdict below is stated in
                    // that vocabulary: adjudicating against a superseded disposition set would
                    // produce answers that look like verdicts and are not.
                    let vocabulary =
                        v1_compiler::cli_run::namespace_wave_admission::vocabulary_findings(index);
                    for finding in &vocabulary {
                        eprintln!("required-ci: namespace-wave-admission VOCABULARY {finding}");
                    }
                    if !vocabulary.is_empty() {
                        phase_failures.push(format!(
                            "namespace-wave-admission vocabulary ({} finding(s))",
                            vocabulary.len()
                        ));
                    }
                    match v1_compiler::cli_run::namespace_wave_admission::run_required_wave_admission(
                        index,
                    ) {
                        Ok(outcome) => {
                            if let Some(failure) = report_wave_admission_outcome(&outcome) {
                                phase_failures.push(failure);
                            }
                        }
                        Err(e) => {
                            eprintln!("required-ci: namespace-wave-admission FAIL {e}");
                            phase_failures.push("namespace-wave-admission".to_string());
                        }
                    }
                }
            }
            ran.push("namespace-wave-admission");
        }

        // PHASE 2 — regen first generation: the emitted mirrors against what is committed.
        if required_ci_phase_selected(RequiredCiPhase::Regen, required_ci_lane) {
            eprintln!("required-ci: phase regen (first generation vs committed)");
            match v1_compiler::cli_run::run_required_regen(
                &regen_candidate_dir,
                &regen_receipt_path,
            ) {
                Ok(outcome) => {
                    // Read through accessors, and print `unmeasured` rather than a plausible
                    // default when the pass built the wrong variant (#8650's shape).
                    let fge = outcome
                        .receipt
                        .first_generation_equal()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unmeasured".to_string());
                    let candidate = outcome.receipt.candidate_artifact().unwrap_or("unmeasured");
                    eprintln!(
                        "required-ci: regen first_generation_equal={fge} candidate={candidate}"
                    );
                    for failure in &outcome.failures {
                        eprintln!("required-ci: regen FAIL {failure}");
                    }
                    if !outcome.failures.is_empty() {
                        phase_failures
                            .push(format!("regen ({} failure(s))", outcome.failures.len()));
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
        }

        // PHASE — every committed generated projection against the bytes derived from its .dag
        // authority. This is deliberately not `--required-regen`: that phase's subject is the
        // stage0 mirror population, while this boundary asks the generated-artifact registry for
        // its whole roster, including DESIGN.md, docs/design-failure-modes.md and
        // docs/design-rung-drops.md. The boundary is read-only, so it cannot manufacture
        // its own green by installing generated bytes.
        if required_ci_phase_selected(RequiredCiPhase::GeneratedArtifact, required_ci_lane) {
            eprintln!(
                "required-ci: phase generated-artifact (every committed projection vs its authority)"
            );
            let outcome = v1_compiler::cli_run::run_generated_artifact_boundary(&source_roots);
            let failures_before = phase_failures.len();
            match &outcome {
                v1_compiler::cli_run::GeneratedArtifactBoundaryOutcome::CarrierRefused {
                    cause,
                } => {
                    eprintln!(
                        "required-ci: generated-artifact CarrierRefused cause={cause} \
                         (the authority declined to answer; nothing was compared)"
                    );
                    phase_failures.push(format!("generated-artifact carrier refused: {cause}"));
                }
                v1_compiler::cli_run::GeneratedArtifactBoundaryOutcome::Adjudicated {
                    artifacts,
                    unadjudicated,
                } => {
                    let divergent = v1_compiler::cli_run::boundary_divergent(&outcome);
                    eprintln!(
                        "required-ci: generated-artifact rostered={} adjudicated={} matches={} \
                         drifted={} absent={} unadjudicated={}",
                        artifacts.len() + unadjudicated.len(),
                        artifacts.len(),
                        artifacts.len() - divergent.len(),
                        divergent
                            .iter()
                            .filter(|a| v1_compiler::cli_run::artifact_disposition(a)
                                == v1_compiler::cli_run::ArtifactDisposition::Drifted)
                            .count(),
                        divergent
                            .iter()
                            .filter(|a| v1_compiler::cli_run::artifact_disposition(a)
                                == v1_compiler::cli_run::ArtifactDisposition::Absent)
                            .count(),
                        unadjudicated.len(),
                    );
                    for a in &divergent {
                        eprintln!(
                            "required-ci: generated-artifact {} {}",
                            v1_compiler::cli_run::artifact_disposition_name(
                                v1_compiler::cli_run::artifact_disposition(a)
                            ),
                            a.path
                        );
                    }
                    for u in unadjudicated {
                        eprintln!(
                            "required-ci: generated-artifact UNADJUDICATED {} — {}",
                            u.path, u.cause
                        );
                    }
                    if !divergent.is_empty() {
                        eprintln!(
                            "required-ci: generated-artifact these are GENERATED projections of \
                             their .dag authorities — do not hand-edit; regenerate with: {}",
                            v1_compiler::cli_run::GENERATED_ARTIFACT_PRODUCING_COMMAND
                        );
                        phase_failures.push(format!(
                            "generated-artifact ({} projection(s) not derived)",
                            divergent.len()
                        ));
                    }
                    if !unadjudicated.is_empty() {
                        phase_failures.push(format!(
                            "generated-artifact ({} rostered artifact(s) reached no verdict)",
                            unadjudicated.len()
                        ));
                    }
                }
            }
            if !v1_compiler::cli_run::boundary_is_clean(&outcome)
                && phase_failures.len() == failures_before
            {
                eprintln!(
                    "required-ci: generated-artifact REFUSED — the outcome is not clean and no \
                     branch above named why. Reporting the refusal rather than the silence"
                );
                phase_failures.push("generated-artifact (not clean, unnamed cause)".to_string());
            }
            ran.push("generated-artifact");
        }

        // PHASE 2b — regen SECOND generation: does the emit reproduce itself.
        //
        // WHAT THIS PHASE CAN AND CANNOT SEE, stated here because the honest claim is much
        // narrower than "the fixed point is now checked". When the first generation EQUALS the
        // committed mirrors, this pass is green by construction: the tree it re-emits from is the
        // one that produced the first generation, so only a NONDETERMINISTIC emit can separate
        // them. It therefore catches emit nondeterminism and nothing else. In particular it
        // cannot see a self-consistently wrong seed -- a producer built from a wrong mirror emits
        // that same wrong mirror and every generation agrees -- and enrolling it must not be read
        // as closing that gap. `gunbc.rung_drop` `floor_cut_regen_second_generation_agreement`
        // states the same bound: a repeatability comparison sees a GENERATION DISAGREEMENT, never
        // deterministic wrongness.
        //
        // IT IS ENROLLED ANYWAY BECAUSE IT IS NEARLY FREE AND ITS RED IS REAL. One extra emit, no
        // install and no rebuild -- the expensive variant that installs the generation and
        // rebuilds the producer buys only BUILD nondeterminism on top, at the price of a crate
        // build per required run, and is deliberately not what this enrols.
        if required_ci_phase_selected(RequiredCiPhase::RegenFixedPoint, required_ci_lane) {
            eprintln!("required-ci: phase regen-fixed-point (second generation vs first)");
            match v1_compiler::cli_run::run_required_regen_fixed_point(&regen_receipt_path, None) {
                Ok(outcome) => {
                    // `unmeasured` rather than a plausible default: a None here means the pass
                    // built the wrong receipt variant, which is a defect to report and not a
                    // verdict to substitute.
                    let fpe = outcome
                        .receipt
                        .fixed_point_equal()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unmeasured".to_string());
                    eprintln!("required-ci: regen-fixed-point fixed_point_equal={fpe}");
                    for failure in &outcome.failures {
                        eprintln!("required-ci: regen-fixed-point FAIL {failure}");
                    }
                    if !outcome.failures.is_empty() {
                        phase_failures.push("regen-fixed-point".to_string());
                    } else if outcome.receipt.fixed_point_equal() != Some(true) {
                        eprintln!(
                            "required-ci: regen-fixed-point REFUSED — no fixed_point_equal was                              measured, so this run has no second-generation verdict to report"
                        );
                        phase_failures.push("regen-fixed-point (unmeasured)".to_string());
                    }
                }
                // A PASS THAT CANNOT RUN REFUSES RATHER THAN SKIPS. The reachable causes are an
                // absent or unparseable prior receipt and a prior measured at a different commit,
                // and every one of them means this run holds NO second-generation evidence.
                // Treating "could not check" as "checked" is the absorbing fallback exactly.
                Err(e) => {
                    eprintln!("required-ci: regen-fixed-point REFUSED {e}");
                    phase_failures.push(format!("regen-fixed-point ({e})"));
                }
            }
            ran.push("regen-fixed-point");
        }

        // PHASE 3 — v2 emission. ENROLLED 2026-08-23 on an operator ruling relayed through
        // the requesting session, after the 2026-08-23 break reached main and stayed for
        // hours with every required phase green: the required run parses src/v1 .dag,
        // compares the regen mirrors and folds the floor, and NONE OF THE THREE COMPILES A
        // v2 ENTRY. Measured cost +135s against the floor's ~30-40 minutes.
        //
        // IT NO LONGER SHARES A JOB WITH THE FLOOR AT ALL (operator ruling, 2026-08-25).
        // The phase sits in the `build` lane beside regen; the floor is the other job, and
        // the two run in parallel. The old paragraph here weighed whether to make this an
        // early PREREQUISITE of the floor -- an exit before it -- and that question is now
        // moot rather than answered: a phase in another job cannot precondition this one,
        // and the run reports both lanes' ledgers whatever either does.
        //
        // The SUBJECT widened with the split, which is the same ruling's other half: the
        // roster is `src/v2/compiler/00_compile.dag`, the v2 pipeline root, where it was
        // the smallest entry in the tree. The reason is in `gunbc.ci_layer_roots`
        // `required_v2_emission_entries` -- the cost that argued against it was a cost
        // against a SERIAL run, and this lane's cost is now free up to the floor's
        // duration.
        //
        // The subject is the SAME PRODUCER the cargo board runs
        // (cli_run::compile_entry_emission, which `gunbc compile --entry` also calls), so
        // a green here and an emitting board are one fact rather than two.
        // PHASE 4 — the witness floor. Independent; runs whatever happened above.
        if required_ci_phase_selected(RequiredCiPhase::Floor, required_ci_lane) {
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
                        measurement_blockers.extend(required_floor_measurement_blockers(&outcome));
                    }
                }
                Err(e) => {
                    eprintln!("required-ci: floor refused: {e}");
                    phase_failures.push(format!("floor refused: {e}"));
                    measurement_unreached = Some(e);
                }
            }
            ran.push("floor");
        }

        // COUNTER-KEY CENSUS (dashboard node adhoc-af8a3fe8-13d): this is the
        // phase population. The required-floor aggregate below counts claim outcomes, so the
        // formerly shared `failed` key gave one spelling two meanings in one process's output.
        eprintln!(
            "required-ci: lane={} phases_run={} phases_failed={}",
            required_ci_lane
                .map(|l| l.name())
                .unwrap_or("all (no --required-lane given)"),
            ran.len(),
            phase_failures.len()
        );
        let judged_module_identities =
            v1_compiler::cli_run::required_lane_judged_module_identities_for_ci();
        eprintln!(
            "required-ci: judged-module-identities {:?}",
            judged_module_identities
        );
        eprintln!(
            "required-ci: cross-process-content-judged-module-identities {:?}",
            v1_compiler::cli_run::required_lane_cross_process_content_judged_module_identities_for_ci()
        );
        match v1_compiler::cli_run::source_root_ingest_module_identities_for_ci(&source_roots) {
            Ok(admitted_module_identities) => {
                eprintln!(
                    "required-ci: admitted-module-identities {:?}",
                    admitted_module_identities
                );
                eprintln!(
                    "required-ci: unresolved-module-identities {:?}",
                    v1_compiler::cli_run::declaration_index::modules_unresolved_by_lane(
                        admitted_module_identities,
                        &judged_module_identities,
                    )
                );
            }
            Err(cause) => {
                eprintln!("required-ci: source-root ingest receipt refused: {cause}");
                phase_failures.push(format!("source-root ingest receipt refused: {cause}"));
            }
        }
        for failure in &phase_failures {
            eprintln!("required-ci: FAILED PHASE {failure}");
        }
        if let Some(path) = required_ci_measurement_receipt {
            // Dissolve this compatibility boundary when every required phase returns its own
            // `Vec<RequiredCiBlocker>`: a human diagnostic must not remain the authority for a
            // blocker's phase and identity.
            for failure in &phase_failures {
                if !measurement_blockers
                    .iter()
                    .any(|b| b.phase == failure.as_str())
                    && failure != "floor"
                {
                    measurement_blockers.push(RequiredCiBlocker {
                        phase: failure
                            .split_whitespace()
                            .next()
                            .unwrap_or("unknown")
                            .to_string(),
                        identity: "<phase>".to_string(),
                        cause: failure.clone(),
                    });
                }
            }
            let receipt = match measurement_unreached {
                Some(cause) => RequiredCiMeasurementReceipt::MeasurementUnreached { cause },
                None => RequiredCiMeasurementReceipt::MeasurementCompleted {
                    blockers: measurement_blockers,
                },
            };
            if let Err(e) = write_required_ci_measurement_receipt(&path, receipt) {
                eprintln!("required-ci: measurement REFUSED {e}");
                return Err(ExitCode::from(1));
            }
            eprintln!("required-ci: measurement completed receipt={path}");
            return Ok(ExitCode::SUCCESS);
        }
        return if phase_failures.is_empty() {
            Ok(ExitCode::SUCCESS)
        } else {
            Err(ExitCode::from(1))
        };
    }

    if required_emit_compile_mode {
        let roots = if source_roots.is_empty() {
            v1_compiler::cli_run::witness_layer_roots()
        } else {
            source_roots.clone()
        };
        let probe_root = v1_compiler::cli_run::local_emit_compile_probe_root();
        match v1_compiler::cli_run::run_required_emit_compile(&roots, &probe_root) {
            Ok(outcomes) => {
                let mut not_passed = 0usize;
                for outcome in &outcomes {
                    eprintln!(
                        "required-emit-compile: {}",
                        v1_compiler::cli_run::emit_compile_outcome_summary(outcome)
                    );
                    if !v1_compiler::cli_run::emit_compile_outcome_passed(outcome) {
                        not_passed += 1;
                        if let v1_compiler::cli_run::EmitCompileOutcome::Measured {
                            baseline, ..
                        } = outcome
                        {
                            for line in
                                v1_compiler::cli_run::cargo_verdict_stderr_tail(baseline).lines()
                            {
                                eprintln!("required-emit-compile: cargo| {line}");
                            }
                        }
                    }
                }
                let (report, retention_error) = v1_compiler::cli_run::emit_compile_report(
                    &outcomes,
                    &roots,
                    &probe_root,
                    "required-emit-compile:",
                );
                for line in report {
                    eprintln!("{line}");
                }
                return if not_passed == 0 && retention_error.is_none() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                };
            }
            Err(e) => {
                eprintln!("required-emit-compile: roster refused: {e}");
                return Err(ExitCode::from(1));
            }
        }
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
                        v1_compiler::cli_run::CompileDisposition::Completed { .. } => {}
                        v1_compiler::cli_run::CompileDisposition::Refused { phase, cause } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionRefused {} phase={phase} cause={cause}",
                                run.subject.receipt()
                            );
                        }
                        v1_compiler::cli_run::CompileDisposition::NotExecuted {
                            earlier_phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionNotExecuted {} earlier_phase={earlier_phase} cause={cause}",
                                run.subject.receipt()
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

    // THE CITED-SYMBOL CENSUS IS GONE FROM HERE, AND ITS SUBJECT MOVED RATHER THAN LAPSING.
    //
    // `--required-cited-symbol` ran `v2.lens.cited_symbol_resolution` over the corpus-wide
    // `decl_facts` + `module_declaration_facts` walks, answering each authored reference by a
    // LINEAR SCAN of a flat list of every declaration in the repository. DESIGN §3's rung-drop
    // row names exactly that shape as the thing to stop doing: the wall belongs "checked at
    // ingestion, on the module whose source carries the citation, from that module's own text,
    // rather than reconstructed corpus-wide by a second job".
    //
    // It is now the `parse` phase above, over the per-module declaration index the sweep builds
    // from the parse it was already performing. The mode is deleted rather than left standing as
    // a second route to one question (§3, single authority), and the replacement is STRICTLY
    // WIDER, which is the test §3's replacement doctrine sets: it enrolls every authored
    // `DeclarationRef` in the corpus rather than the five carriers the lens's population named,
    // and it indexes test modules, which `decl_facts` deliberately did not — so the
    // outside-index disposition those exclusions forced is not needed at all.

    // THE HEADS READING'S OWN INSTRUMENT MOVED TO `gunbc test`, AND IT MOVED ATOMICALLY.
    //
    // What stood here was a `--heads-reading-differential` mode: the same producer, reached by a
    // flag on this binary. It is DELETED in the change that makes the instrument addressable as
    // `//gunbc/instruments:heads-reading-differential`, rather than left standing beside it. Two
    // routes to one producer is the fork DESIGN section 3 forbids, and the interval in which both
    // work is exactly when a caller learns the old one and keeps it.
    //
    // The reason it was never enrolled in the required run is unchanged and is now recorded on the
    // target instead: reading the corpus TWICE is precisely the cost the heads reading exists to
    // remove, so paying it on every push would spend more than the repair saves. The instrument is
    // outside `//:required` by construction there -- `gunbc.discovery_census` derives that
    // aggregate from discovered witness sites, and no fold feeds the instrument population into it.

    if regen_affected_set_mode {
        return match v1_compiler::cli_run::run_regen_affected_set(&source_roots) {
            Ok(outcome) => {
                eprint!("{}", outcome.rendered);
                if outcome.arm == "EditedSetUnlocatable" {
                    Err(ExitCode::from(1))
                } else {
                    Ok(ExitCode::SUCCESS)
                }
            }
            Err(e) => {
                eprintln!("regen-affected-set: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    if regen_round_cost_mode {
        return match v1_compiler::cli_run::run_regen_round_cost(
            &regen_candidate_dir,
            &regen_receipt_path,
            &source_roots,
            regen_affected_scope,
        ) {
            Ok(outcome) => {
                eprint!("{}", outcome.rendered);
                eprintln!(
                    "regen-round-cost: receipt={}",
                    outcome.receipt_path.display()
                );
                for failure in &outcome.round_failures {
                    eprintln!("regen-round-cost: FAIL {failure}");
                }
                if outcome.round_failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("regen-round-cost: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    if required_regen_mode {
        let regen = if regen_affected_scope {
            v1_compiler::cli_run::run_required_regen_scoped(
                &regen_candidate_dir,
                &regen_receipt_path,
                &source_roots,
            )
        } else {
            v1_compiler::cli_run::run_required_regen(&regen_candidate_dir, &regen_receipt_path)
        };
        return match regen {
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
// ═══════════════════════════════════════════════════════════════════════════════════════════
// THE REQUIRED RUN'S PHASE ROSTER, AND THE TWO LANES IT PARTITIONS INTO.
// ═══════════════════════════════════════════════════════════════════════════════════════════
//
// WHY A PARTITION EXISTS AT ALL (operator ruling, 2026-08-25). The required run's phases are
// mutually independent -- that property was already established when the roster was composed
// into one process -- but they were also SERIAL, because one process runs them one after
// another. Serial independence is the expensive combination: the witness floor costs ~30-40
// minutes and every other phase waits behind it for no reason a data dependency can name, so
// the run's wall clock is the SUM of things that could have been a MAX.
//
// The split makes it a max. `build` (parse, regen, v2-emission) and `witnesses` (the floor)
// run as two GitHub Actions jobs with no `needs` edge between them, so the required check
// finishes when the slower of the two does, and the build lane's cost is free up to the
// floor's duration. That headroom is what makes the v2-emission phase's subject a cost
// decision rather than a cost objection.
//
// WHAT THIS SUPERSEDES, AND WHAT IT DOES NOT (2026-08-20 consolidation directive: "i would
// much prefer if it was all handled within the witnesses step and within the gunbc binary, not
// at a github actions job level"). That directive has two halves and only one is superseded.
//
//   SURVIVES -- "within the gunbc binary". The phases still live here. Each job makes ONE
//   invocation of this binary and names a LANE; it does not name phases, order them, or wire
//   one phase's precondition to another step's `outcome`. The step-ladder defect the
//   consolidation fixed -- an `if:` naming a sibling step, silently conjoined with GitHub's
//   implicit `success()`, so a regen red disarmed the whole floor -- cannot return, because
//   there is no step whose condition another step's verdict could reach.
//
//   SUPERSEDED -- "not at a github actions job level". PARALLELISM IS NOT EXPRESSIBLE IN THE
//   BINARY, and that is a property of the transport rather than a preference. Two phases
//   running concurrently means two runners, two checkouts and two toolchains; a single process
//   on one runner can thread but cannot acquire a second machine, and the ~9.4 GiB the floor
//   peaks at is a reason not to want them on one anyway. So the lane boundary is a JOB
//   boundary of necessity. What the directive was protecting against -- SEQUENCING and
//   PRECONDITIONS leaking into YAML -- is exactly what does not cross it: the jobs are
//   unordered and unconditioned on each other, which is the whole content of running them in
//   parallel.
//
// THE PARTITION IS TOTAL BY CONSTRUCTION, which is the point of writing it as a match rather
// than as two lists. Two hand-authored rosters could drop a phase from both and each would
// look complete; here every phase has exactly one lane because `RequiredCiPhase::lane` is an
// exhaustive match, and adding a phase without assigning it fails to compile. A phase silently
// belonging to no job is the empty-observation narrow (DESIGN, recurring failure modes) at the
// worst possible grain -- a required check reporting green over a measurement nothing took.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RequiredCiLane {
    Build,
    Witnesses,
}

impl RequiredCiLane {
    fn name(self) -> &'static str {
        match self {
            RequiredCiLane::Build => "build",
            RequiredCiLane::Witnesses => "witnesses",
        }
    }

    /// A REFUSAL, NOT A DEFAULT. An unrecognized lane word names a job that does not exist, and
    /// the failure mode of guessing is that the run silently covers a different set of phases
    /// than the workflow believes it asked for -- green over an unmeasured population.
    fn parse(value: &str) -> Result<RequiredCiLane, String> {
        match value {
            "build" => Ok(RequiredCiLane::Build),
            "witnesses" => Ok(RequiredCiLane::Witnesses),
            other => Err(format!(
                "unknown --required-lane: {other} (expected build | witnesses)"
            )),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RequiredCiPhase {
    Parse,
    NamespaceWaveAdmission,
    Regen,
    GeneratedArtifact,
    RegenFixedPoint,
    Floor,
}

impl RequiredCiPhase {
    fn name(self) -> &'static str {
        match self {
            RequiredCiPhase::Parse => "parse",
            RequiredCiPhase::NamespaceWaveAdmission => "namespace-wave-admission",
            RequiredCiPhase::RegenFixedPoint => "regen-fixed-point",
            RequiredCiPhase::Regen => "regen",
            RequiredCiPhase::GeneratedArtifact => "generated-artifact",
            RequiredCiPhase::Floor => "floor",
        }
    }

    fn lane(self) -> RequiredCiLane {
        match self {
            // PARSE RIDES WITH THE FLOOR, and the reason is its subject rather than its cost.
            // The sweep walks src/v1, dag and src/v2 and answers `does every authored .dag
            // file tokenize and parse` -- the same corpus the floor then prepares strictly. It
            // costs seconds, so it is free wherever it sits; putting it in front of the long
            // lane means the cheapest total refusal over the witness corpus arrives from the
            // job that owns that corpus.
            RequiredCiPhase::Parse => RequiredCiLane::Witnesses,
            // THE WALL CONSUMES THE PARSE THAT PHASE JUST RAN, so it is in the same lane by
            // necessity and not by preference: its head index IS the sweep's index, and a
            // lane boundary between them would mean parsing the corpus twice to answer a
            // question one parse already reached.
            RequiredCiPhase::NamespaceWaveAdmission => RequiredCiLane::Witnesses,
            RequiredCiPhase::Regen => RequiredCiLane::Build,
            RequiredCiPhase::GeneratedArtifact => RequiredCiLane::Build,
            // THE FIXED POINT RIDES WITH REGEN BY NECESSITY, NOT PREFERENCE. It reads the receipt
            // the regen phase wrote at `target/stage0-regen-receipt.json`, and target/ does not
            // survive checkout, so a lane boundary between them would leave it with no prior
            // measurement to reference and nothing to compare.
            RequiredCiPhase::RegenFixedPoint => RequiredCiLane::Build,
            RequiredCiPhase::Floor => RequiredCiLane::Witnesses,
        }
    }
}

// THE REQUIRED GATE IS SIX PHASES. Four are the 2026-08-29 compiler-floor bankruptcy roster;
// generated-artifact returned after its declared exposure produced a real stale projection on
// main. The other three removed phases remain outside required CI and inside the declared drop.
const REQUIRED_CI_PHASES: [RequiredCiPhase; 6] = [
    RequiredCiPhase::Parse,
    RequiredCiPhase::NamespaceWaveAdmission,
    RequiredCiPhase::Regen,
    RequiredCiPhase::GeneratedArtifact,
    RequiredCiPhase::RegenFixedPoint,
    RequiredCiPhase::Floor,
];

/// The `.dag` coproduct this enum realizes, and the declaration whose variants it must equal.
/// `gunbc.required_ci_phase_roster` is the substrate authority for phase identity and lane
/// ownership; this enum, its `lane` match and `REQUIRED_CI_PHASES` are its declared parallel
/// realization, scheduled to delete atomically when the workflow emission consumes the roster
/// whole (`gunbc.required_ci_host_verdict_census` `required_ci_phase_roster_next_rung`).
const PHASE_ROSTER_AUTHORITY_MODULE: &str = "gunbc.required_ci_phase_roster";
const PHASE_ROSTER_AUTHORITY_DECL: &str = "RequiredCiPhase";

/// Every phase this binary realizes, in the authority's own variant spelling.
const PHASE_ROSTER_VARIANT_LABELS: [&str; 6] = [
    "ParsePhase",
    "NamespaceWaveAdmissionPhase",
    "RegenPhase",
    "GeneratedArtifactPhase",
    "RegenFixedPointPhase",
    "FloorPhase",
];

/// Refuse if the host phase enum and the `.dag` phase roster disagree — the same both-directions
/// variant-set join `namespace_wave_admission::vocabulary_findings` executes against its own
/// authority, for the same reason: a second representation diverges on the first amendment, and
/// nothing else joins these two. An absent authority module refuses too — that is the state in
/// which nothing is checking the roster, not permission to proceed.
///
/// WHAT THIS JOIN DELIBERATELY DOES NOT COVER: lane ownership. The host `lane` match is not
/// readable from the declaration index, and a lane divergence cannot un-enrol a phase — both
/// lanes execute in every required run — so the residue is bounded to which job carries a phase,
/// and its terminal is the atomic deletion the roster's census row names.
fn phase_roster_findings(
    index: &v1_compiler::cli_run::declaration_index::DeclarationIndex,
) -> Vec<String> {
    use std::collections::BTreeSet;
    let Some(record) =
        v1_compiler::cli_run::declaration_index::index_get(index, PHASE_ROSTER_AUTHORITY_MODULE)
    else {
        return vec![format!(
            "the phase-roster authority `{PHASE_ROSTER_AUTHORITY_MODULE}` is absent from the \
             index, so nothing joins this host phase enum to the roster it realizes"
        )];
    };
    let Some(authored) = record.decl_fields.get(PHASE_ROSTER_AUTHORITY_DECL) else {
        return vec![format!(
            "`{PHASE_ROSTER_AUTHORITY_MODULE}` declares no `{PHASE_ROSTER_AUTHORITY_DECL}`"
        )];
    };
    let here: BTreeSet<String> = PHASE_ROSTER_VARIANT_LABELS
        .iter()
        .map(|l| l.to_string())
        .collect();
    let mut findings = Vec::new();
    for missing in authored.difference(&here) {
        findings.push(format!(
            "`{PHASE_ROSTER_AUTHORITY_DECL}` declares `{missing}` and this host enum does not \
             realize it — a phase enrolled with no executor"
        ));
    }
    for extra in here.difference(authored) {
        findings.push(format!(
            "this host enum realizes `{extra}` and `{PHASE_ROSTER_AUTHORITY_DECL}` does not \
             declare it — a phase running with no authority"
        ));
    }
    findings
}

/// EVERY PHASE IS ACCOUNTED FOR IN EVERY RUN, whether or not this lane owns it.
///
/// A lane that simply omitted the other lane's phases would produce a ledger in which "this
/// phase passed", "this phase was never reached" and "this phase belongs to the other job" are
/// one silence. So a run selecting a lane prints a ROUTED line for each phase it does not own,
/// naming the lane that does. The reader of one job's log can then reconstruct the whole roster
/// and see exactly which job to look in for the rest, instead of inferring coverage from an
/// absence.
fn required_ci_phase_selected(phase: RequiredCiPhase, lane: Option<RequiredCiLane>) -> bool {
    match lane {
        None => true,
        Some(selected) => phase.lane() == selected,
    }
}

/// Print one wave-admission run, and return the phase failure it carries, if any.
///
/// EVERY GREEN NAMES ITS DENOMINATORS. A run that admitted nothing because it compared
/// nothing and a run that compared a corpus and found no motion render identically unless the
/// population is printed beside the verdict, and rendering them alike is the
/// execution-provenance loss DESIGN names.
fn report_wave_admission_outcome(
    outcome: &v1_compiler::cli_run::namespace_wave_admission::WaveAdmissionOutcome,
) -> Option<String> {
    use v1_compiler::cli_run::namespace_wave_admission as nwa;
    match outcome {
        nwa::WaveAdmissionOutcome::NoSubject { head } => {
            eprintln!(
                "required-ci: namespace-wave-admission NO SUBJECT — the merge base against \
                 origin/main IS {head}, so this run has no diff to adjudicate. Nothing was \
                 compared and nothing is admitted."
            );
            None
        }
        nwa::WaveAdmissionOutcome::NotEvaluated { reason } => {
            eprintln!("required-ci: namespace-wave-admission NotEvaluated — {reason}");
            nwa::wave_admission_refusal(outcome)
        }
        nwa::WaveAdmissionOutcome::Adjudicated {
            base,
            head,
            report,
            roster_touched: _,
        } => {
            let p = &report.population;
            eprintln!(
                "required-ci: namespace-wave-admission base={base} head={head} \
                 modules_compared={} modules_added={} modules_removed={} \
                 membership_edges_head={} binding_rows_compared={} closure_rows_moved={} \
                 deltas={}",
                p.modules_compared,
                p.modules_added,
                p.modules_removed,
                p.membership_edges_head,
                p.binding_rows_compared,
                p.closure_rows_moved,
                report.deltas.len(),
            );
            for delta in &report.deltas {
                eprintln!(
                    "required-ci: namespace-wave-admission {}",
                    nwa::render_delta(delta)
                );
            }
            for stale in &report.stale_admissions {
                eprintln!("required-ci: namespace-wave-admission STALE ADMISSION {stale}");
            }
            for consumed in &report.consumed_admissions {
                eprintln!("required-ci: namespace-wave-admission CONSUMED ADMISSION {consumed}");
            }
            // THE VERDICT IS THE WALL'S, NOT THE PRINTER'S. This function owns the receipts
            // because it owns a stderr; `wave_admission_refusal` owns whether the run refuses,
            // so the arm that decides it can be exercised by a test on the path CI runs rather
            // than only from inside this binary.
            let refusal = nwa::wave_admission_refusal(outcome);
            if refusal.is_none() {
                eprintln!(
                    "required-ci: namespace-wave-admission ADMITTED — every delta is \
                     auto-admitted or named by a transition admission"
                );
            }
            refusal
        }
    }
}

fn report_required_floor_outcome(outcome: &v1_compiler::cli_run::RequiredFloorOutcome) {
    eprintln!(
        "required-floor: subject={} modules_resolved={} modules_excluded={}",
        outcome.subject_digest, outcome.modules_resolved, outcome.modules_excluded
    );
    // THE SUBJECT THE ROSTER WAS PROJECTED FROM, STATED BEFORE THE ROSTER.
    // `planned` is the population that SURVIVED site projection; printing it
    // without `offered` and the declines made the receipt unable to say what it
    // dropped, which is how a roster that narrowed read exactly like one that did
    // not. The categories are printed together so the subtraction is visible rather
    // than inferable.
    // AND THE SENTENCE IS NOW BOUNDED ABOVE, WHICH IT WAS NOT.
    // "every discovered site is exactly one of these" is a totality claim over SITES, and it was
    // exact and silent at the same time: a `*_test.dag` entry declaring no `test fn` contributes
    // no site at all, so it could never appear in any of the four numbers, and the line read as a
    // coverage guarantee over a denominator that had already dropped it. It cannot now — a barren
    // entry stops the line in `run_required_floor` before this point — so the guarantee is stated
    // rather than left for a reader to discover it was never claimed.
    eprintln!(
        "required-floor: declared={} offered={} routed={} declined_long={} declined_fixture={} \
         declined_outside_required_gate={} declined_outside_gate_closure={} \
         declined_discovery_excluded={} — every DECLARED witness identity in the tree is exactly \
         one of these, joined at identity grain (FloorDispositionJoinInexact refuses otherwise), \
         and no `*_test.dag` entry declared zero sites (v2.workflow.floor_discovery_producer \
         refuses a barren or misplaced sidecar upstream of this line, over the full module \
         index)",
        outcome.declared_identities,
        outcome.sites_offered,
        outcome.claims_planned,
        outcome.declined_long_module,
        outcome.declined_fixture_member,
        outcome.declined_outside_required_gate,
        outcome.declined_outside_gate_closure,
        outcome.declined_discovery_excluded
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
    // THE PER-CAUSE COUNTERS ARE DERIVED FROM THE ROWS, never accumulated beside them: a
    // counter that disagrees with the population it summarizes has no constructor here.
    // `interrupted_before_verdict` stays on the line as the population's size, with the causes
    // beside it, because dropping the total would move the question rather than answer it —
    // a reader who wants "how many claims never answered" would have to add the arms and would
    // silently under-count the day a third mechanism gets an arm.
    let interrupted_causes =
        v1_compiler::cli_run::interrupted_cause_census(&outcome.interrupted_before_verdict);
    eprintln!(
        "required-floor: planned={} executed={} not_attempted={} terminal={} passed={} \
         known_red_held={} claims_failed={} stale_quarantine={} \
         interrupted_before_verdict={} interrupted_cpu_deadline={} \
         interrupted_wall_deadline={} completed_over_cost_requirement={} \
         host_tool_unresolved={} route_gap_unenrolled={} route_gap_held={} \
         stale_route_gap={} known_red_now_passing={} known_red_budget_refused={} \
         known_red_passed_over_budget={} known_red_host_tool_unresolved={} \
         known_red_host_effect_refused={} known_red_runtime_errored={} \
         known_red_observation_unreadable={} over_cost_line_diagnostic={} \
         withheld_cost_debt={} stale_cost_debt={}",
        outcome.claims_planned,
        outcome.claims_executed,
        outcome.not_attempted_after_abort,
        outcome.receipt_identities,
        outcome.passed,
        outcome.known_red_held,
        outcome.failures.len(),
        outcome.stale_quarantine.len(),
        outcome.interrupted_before_verdict.len(),
        interrupted_causes.cpu_deadline,
        interrupted_causes.wall_deadline,
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
        outcome.over_cost_line_diagnostic,
        outcome.withheld_cost_debt.len(),
        outcome.stale_cost_debt.len()
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
    // The sibling census memo, reported on its own line for the reason the emit-check line
    // carries both halves: a hit count without its denominator is not a measurement. These are
    // two different memos over two different builtins and must never be summed into one ratio.
    let (census_hits, census_misses) =
        v1_compiler::cli_run::compile_dag_diagnostic_census_memo_counts();
    eprintln!(
        "required-floor: compile_dag_diagnostic_census_memo hits={census_hits} \
         misses={census_misses}"
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
    // THE CAUSE IS ON THE ROW'S OWN LINE, not only in the summary. A reader triaging one
    // identity reads this line and nothing else; leaving the mechanism only in the aggregate
    // would put the evidence on a surface disjoint from the one anyone reads for that identity.
    for refused in &outcome.interrupted_before_verdict {
        eprintln!(
            "required-floor: INTERRUPTED-BEFORE-VERDICT {} raised_by={} \
             cpu_at_least={}ms/{}ms wall_at_least={}ms/{}ms enrolled_expected_red={} {}",
            refused.qualified,
            refused.interrupt.raised_by.label(),
            // BOTH CLOCKS, EACH AGAINST ITS OWN LIMIT. These are LOWER BOUNDS, which is what
            // `at_least` says: the deadline preempted the witness, so the true cost is above
            // them by an unbounded amount. They are printed anyway because the PAIR is what a
            // reader needs — a row blocked on I/O that went away shows a small cpu figure beside
            // a wall figure at its ceiling, and a row that genuinely computed shows cpu at or
            // above the cpu limit. `cost=UNMEASURED` in the sentence that follows stays true of
            // both and is what stops either figure being read as this row's cost.
            refused.interrupt.elapsed_cpu_at_least_ms,
            refused.interrupt.cpu_safety_limit_ms,
            refused.interrupt.elapsed_wall_at_least_ms,
            refused.interrupt.wall_safety_limit_ms,
            refused.enrolled_expected_red,
            refused.detail
        );
    }
    for over_cost in &outcome.completed_over_cost_requirement {
        eprintln!("required-floor: COMPLETED-OVER-COST-REQUIREMENT {over_cost}");
    }
    for unresolved in &outcome.host_tool_unresolved {
        eprintln!("required-floor: HOST-TOOL-UNRESOLVED {unresolved}");
    }
    // STALE WITHHOLDS ARE NAMED INDIVIDUALLY; WITHHELD ROWS ARE NOT. The stale population
    // blocks and every member is a line to delete, so each one is printed. The withheld
    // population is large enough that printing it per-row would bury every other diagnostic in
    // the run — it is counted in the summary above and enumerated in full in
    // `v2.workflow.floor_cost_debt`, which is the authority a reader should be sent to anyway.
    //
    // NO SIZE IS STATED HERE, DELIBERATELY. This comment carried a numeral until review 57254,
    // and it had gone stale against the roster it described — two hand-transcribed counts for one
    // population, disagreeing at the two points a reader is most likely to trust. That is the
    // failure DESIGN's "name the instrument, never transcribe its output" rule exists to prevent,
    // and the fix is not to re-transcribe the current figure: the roster is a function, its size
    // is a property of that function, and any numeral here is a second representation that goes
    // stale the next time a row is enrolled or released. The authority is named instead.
    for stale in &outcome.stale_cost_debt {
        eprintln!("required-floor: STALE-COST-DEBT {stale}");
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
        // WITHHELD ROWS DO NOT BLOCK; A STALE WITHHOLD DOES. `withheld_cost_debt` is the frozen
        // population the 2026-08-27 ceiling restoration declared, and blocking on it would red
        // main for precisely the debt the contract exists to carry down. `stale_cost_debt` is a
        // roster that has stopped describing the tree, which voids the contract's monotone
        // claim, so it blocks exactly as `stale_quarantine` and `stale_route_gap` do.
        && outcome.stale_cost_debt.is_empty()
        // A CHANGED witness identity that did not execute to a passing verdict — declined,
        // absent from the disposition receipt, or without a terminal Passed verdict — reds the
        // required context. The classification authority is
        // `v2.workflow.floor_changed_witness.changed_witness_standing_blocks`; the population
        // is only the identities this change's diff touched, never the standing declined
        // corpus, so this conjunct cannot red a PR for debt it did not author.
        && outcome.changed_witness_blocking.is_empty()
}

fn required_floor_measurement_blockers(
    outcome: &v1_compiler::cli_run::RequiredFloorOutcome,
) -> Vec<RequiredCiBlocker> {
    let mut blockers = Vec::new();
    let mut add = |identity: &str, cause: &str| {
        blockers.push(RequiredCiBlocker {
            phase: "floor".to_string(),
            identity: identity.to_string(),
            cause: cause.to_string(),
        })
    };
    for identity in &outcome.failures {
        add(identity, "claim_failed");
    }
    for identity in &outcome.non_verdict_unenrolled {
        add(identity, "non_verdict_unenrolled");
    }
    for identity in &outcome.stale_non_verdict {
        add(identity, "stale_non_verdict");
    }
    for identity in &outcome.stale_quarantine {
        add(identity, "stale_quarantine");
    }
    for row in &outcome.interrupted_before_verdict {
        add(&row.qualified, "interrupted_before_verdict");
    }
    for identity in &outcome.completed_over_cost_requirement {
        add(identity, "completed_over_cost_requirement");
    }
    for identity in &outcome.host_tool_unresolved {
        add(identity, "host_tool_unresolved");
    }
    for identity in &outcome.route_gap {
        add(identity, "route_gap");
    }
    for identity in &outcome.stale_route_gap {
        add(identity, "stale_route_gap");
    }
    for identity in &outcome.stale_cost_debt {
        add(identity, "stale_cost_debt");
    }
    for identity in &outcome.changed_witness_blocking {
        add(identity, "changed_witness_blocking");
    }
    blockers
}

fn main() -> ExitCode {
    // NO SECOND ARGV READER. What stood here read std::env::args() directly and dispatched the
    // floor coordinator BEFORE run() parsed anything, so deleting --plan-function from run()'s
    // parser left the coordinator fully reachable -- and reachable through a path that removed
    // receipt files and spawned workers before any refusal could fire. Review 55875 caught it,
    // and the state it caught was worse than the one before this branch: --plan-function answered
    // "unknown argument" for every value EXCEPT the one that ran the whole coordinator.
    //
    // The lesson is the cut's, not the coordinator's: a flag is not deleted when one of two
    // parsers stops reading it. run() is now the only thing that reads argv.
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
    fn blocking_measurement_round_trips_with_the_planted_identity_and_refuses() {
        let path =
            std::env::temp_dir().join(format!("gunbc-d0-blocking-{}.json", std::process::id()));
        let identity = "fixture.planted_blocking_diagnostic";
        write_required_ci_measurement_receipt(
            path.to_str().expect("utf8 temp path"),
            RequiredCiMeasurementReceipt::MeasurementCompleted {
                blockers: vec![RequiredCiBlocker {
                    phase: "floor".to_string(),
                    identity: identity.to_string(),
                    cause: "claim_failed".to_string(),
                }],
            },
        )
        .expect("write receipt");
        let body = std::fs::read_to_string(&path).expect("read receipt");
        assert!(
            body.contains(identity),
            "receipt must retain the blocking identity"
        );
        assert!(adjudicate_required_ci_measurement_receipt(path.to_str().unwrap()).is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn measurement_unreached_is_a_present_distinct_refusal() {
        let path =
            std::env::temp_dir().join(format!("gunbc-d0-unreached-{}.json", std::process::id()));
        write_required_ci_measurement_receipt(
            path.to_str().expect("utf8 temp path"),
            RequiredCiMeasurementReceipt::MeasurementUnreached {
                cause: "instrument exited".to_string(),
            },
        )
        .expect("write receipt");
        let body = std::fs::read_to_string(&path).expect("read receipt");
        assert!(body.contains("measurement_unreached"));
        assert!(adjudicate_required_ci_measurement_receipt(path.to_str().unwrap()).is_err());
        let _ = std::fs::remove_file(path);
    }

    fn repo_root_from_manifest() -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root from CARGO_MANIFEST_DIR")
    }

    fn dag_source_from_repo(rel: &str) -> String {
        let path = repo_root_from_manifest().join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
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
    const TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY: &str =
        "dag/gunbc/instruments/floor_effect_gate_witness.dag";
    const TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION: &str = "dag_compile_clean_gate_passes";
    const TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY: &str =
        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag";
    const TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION: &str = "native_selected_logic_production_spec";

    /// Test-only obligation-subject literals must track gunbc.ci_materialization authority rows.
    #[test]
    fn floor_resolve_obligation_seed_constants_match_dag_authority() {
        let ci_materialization = dag_source_from_repo("dag/gunbc/ci/ci_materialization.dag");
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

    // --- floor-finalization DISPOSITION visibility (the fix this PR ships) -----------
    //
    // Three previously-silent cases, each with its own discriminating control so the
    // three cannot collapse back into one indistinguishable "nothing printed" bucket:
    //   A — finalization absent by construction (scoped floor worker)
    //   B — finalization absent incidentally (walk never reached the call)
    //   C — finalization HELD on an otherwise-failed floor (verdict must not vanish)

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

    // ---- walk-attempt identity ----
    //
    // The refusals below are the point of the feature, so each is asserted as a REFUSAL,
    // not merely as "not equal to the good value". The positive controls exist so the
    // negatives are discriminating: a `compose_walk_attempt_id` that refused everything
    // would pass every Err assertion and fail these.

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
