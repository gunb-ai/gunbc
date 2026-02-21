//! gunbc-sdlc: issue-centric SDLC intake/worker entrypoint.
//!
//! Initial runtime surface:
//! - intake: validate intent contract + deterministic run_key + idempotent ledger update
//! - worker: summarize pending intake ledger state

#![deny(dead_code)]
#![allow(clippy::disallowed_methods)] // CLI-owned local ledgers and git metadata probes are intentional entrypoint concerns.

use gunbc_dag::{
    claim_slot_key, heartbeat_claim, mark_run_completed, mark_run_failed, promote_to_canonical_artifact,
    provisional_marker, reconcile_entries, register_retry_failure, release_claim, retry_ready,
    should_replay_skip, try_acquire_claim, upsert_provisional_artifact, ArtifactLedger,
    ArtifactUpsertOutcome, ClaimAcquireResult, ClaimLedger, ReconcileAction, ReconcileEntry,
    RetryState, RunStateLedger, validate_stage_transition,
};
use gunbc_design_ops::{build_design_prompt, DesignRequest};
use gunbc_ir::transport::github::{
    compare_and_set_stage_label, ensure_sdlc_issue_capabilities, SdlcIssueCapabilities,
};
use gunbc_ir::transport::github::IssueLifecycleStage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

const CLAIM_LEASE_TTL_MS: u128 = 30_000;
const RETRY_BASE_BACKOFF_MS: u128 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SdlcCommand {
    Intake,
    Worker,
    Issue,
    AwaitApproval,
    Transition,
    Drain,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    command: SdlcCommand,
    intent_path: Option<PathBuf>,
    infra_intent_path: Option<PathBuf>,
    intake_key: Option<String>,
    issue_id: Option<u64>,
    stage: Option<IssueLifecycleStage>,
    dry_run: bool,
    emit_pending_exit_code: bool,
    drain_activate: bool,
    drain_deactivate: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct IntentSheet {
    intent_id: String,
    title: String,
    objective: String,
    provider: String,
    success_criteria: Vec<String>,
    constraints: Vec<String>,
    owner: String,
    priority: String,
    scope: IntentScope,
    links: IntentLinks,
    idempotency: IntentIdempotency,
    update_strategy: UpdateStrategy,
    tracking: TrackingState,
    acceptance_tests: Vec<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentSheet {
    schema_version: String,
    intent_id: String,
    environment: String,
    runtime_profile: String,
    provider: String,
    policy_version: String,
    components: InfraIntentComponents,
    safety: InfraIntentSafety,
    launch: InfraIntentLaunch,
    drift: InfraIntentDrift,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentComponents {
    claim_store: InfraIntentStore,
    outcome_ledger: InfraIntentStore,
    secrets: InfraIntentSecrets,
    metrics: InfraIntentMetrics,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentStore {
    backend: String,
    dsn: String,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentSecrets {
    credential_policy_profile: String,
    required_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentMetrics {
    sink: String,
    namespace: String,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentSafety {
    fail_closed_on_missing_prereqs: bool,
    require_capability_gate: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentLaunch {
    worker_count: u32,
    lease_ttl_seconds: u32,
    heartbeat_seconds: u32,
    poll_interval_seconds: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct InfraIntentDrift {
    reconcile_mode: String,
    reconcile_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize)]
struct WorkerPreflightSummary {
    status: String,
    intent_path: String,
    intent_id: String,
    environment: String,
    runtime_profile: String,
    checked_components: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct IntentScope {
    #[serde(rename = "in")]
    in_scope: Vec<String>,
    out: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct IntentLinks {
    docs: Vec<String>,
    related_issues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct IntentIdempotency {
    intake_key: String,
    policy_version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdateStrategy {
    comment_mode: String,
    transition_mode: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TrackingState {
    issue_id: Option<u64>,
    run_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct IntakeLedger {
    entries: BTreeMap<String, IntakeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntakeRecord {
    intent_id: String,
    run_key: String,
    issue_id: Option<u64>,
    policy_version: String,
    #[serde(default = "default_stage")]
    stage: IssueLifecycleStage,
    #[serde(default)]
    awaiting_approval: bool,
    #[serde(default)]
    terminalized: bool,
    #[serde(default)]
    retry: RetryState,
    #[serde(default)]
    awaiting_approval_since_epoch_ms: Option<u128>,
    #[serde(default)]
    trace_linkage: Option<TraceLinkage>,
    #[serde(default)]
    created_at_epoch_ms: u128,
    updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraceLinkage {
    repo_root: String,
    branch: String,
    commit: String,
    intent_id: String,
    issue_id: Option<u64>,
    run_key: String,
    linkage_key: String,
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let args = match parse_cli_args(&argv) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("invalid sdlc arguments: {error}");
            print_help();
            std::process::exit(1);
        }
    };

    if args.command == SdlcCommand::Help {
        print_help();
        return;
    }

    let result = match args.command {
        SdlcCommand::Intake => run_intake(args.intent_path.as_ref(), args.dry_run),
        SdlcCommand::Worker => run_worker(
            args.dry_run,
            args.emit_pending_exit_code,
            args.infra_intent_path.as_ref(),
            None,
            "worker",
        ),
        SdlcCommand::Issue => run_worker(
            args.dry_run,
            args.emit_pending_exit_code,
            args.infra_intent_path.as_ref(),
            args.issue_id,
            "issue",
        ),
        SdlcCommand::AwaitApproval => run_await_approval(args.intake_key.as_deref(), args.dry_run),
        SdlcCommand::Transition => {
            run_transition(args.intake_key.as_deref(), args.stage, args.dry_run)
        }
        SdlcCommand::Drain => run_drain(args.drain_activate, args.drain_deactivate, args.dry_run),
        SdlcCommand::Help => Ok(()),
    };
    if let Err(error) = result {
        eprintln!("sdlc command failed: {error}");
        std::process::exit(1);
    }
}

fn parse_cli_args(argv: &[String]) -> Result<CliArgs, String> {
    if argv.len() <= 1 {
        return Ok(CliArgs {
            command: SdlcCommand::Help,
            intent_path: None,
            infra_intent_path: None,
            intake_key: None,
            issue_id: None,
            stage: None,
            dry_run: false,
            emit_pending_exit_code: false,
            drain_activate: false,
            drain_deactivate: false,
        });
    }

    let mut issue_id: Option<u64> = None;
    let mut idx = 2usize;
    let command = match argv[1].as_str() {
        "intake" => SdlcCommand::Intake,
        "worker" => SdlcCommand::Worker,
        "issue" => SdlcCommand::Issue,
        "--issue" => {
            let value = argv
                .get(2)
                .ok_or_else(|| "--issue requires an issue id value".to_string())?;
            if value.starts_with("--") {
                return Err("--issue requires a non-flag issue id value".to_string());
            }
            issue_id = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --issue value `{value}` (expected u64)"))?,
            );
            idx = 3;
            SdlcCommand::Issue
        }
        token if token.starts_with("--issue=") => {
            let value = token.trim_start_matches("--issue=");
            if value.is_empty() {
                return Err("--issue requires a non-empty issue id value".to_string());
            }
            issue_id = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --issue value `{value}` (expected u64)"))?,
            );
            idx = 2;
            SdlcCommand::Issue
        }
        "await-approval" => SdlcCommand::AwaitApproval,
        "transition" => SdlcCommand::Transition,
        "drain" => SdlcCommand::Drain,
        "help" | "--help" | "-h" => SdlcCommand::Help,
        other => return Err(format!("unknown command `{other}`")),
    };

    let mut intent_path: Option<PathBuf> = None;
    let mut infra_intent_path: Option<PathBuf> = None;
    let mut intake_key: Option<String> = None;
    let mut stage: Option<IssueLifecycleStage> = None;
    let mut dry_run = false;
    let mut emit_pending_exit_code = false;
    let mut drain_activate = false;
    let mut drain_deactivate = false;
    while idx < argv.len() {
        let token = &argv[idx];
        if token == "--dry-run" {
            if dry_run {
                return Err("duplicate --dry-run flag".to_string());
            }
            dry_run = true;
            idx += 1;
            continue;
        }
        if token == "--emit-pending-exit-code" {
            if emit_pending_exit_code {
                return Err("duplicate --emit-pending-exit-code flag".to_string());
            }
            emit_pending_exit_code = true;
            idx += 1;
            continue;
        }
        if token == "--issue-id" {
            if issue_id.is_some() {
                return Err("duplicate --issue-id flag".to_string());
            }
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--issue-id requires a value".to_string())?;
            if value.starts_with("--") {
                return Err("--issue-id requires a non-flag value".to_string());
            }
            issue_id = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --issue-id value `{value}` (expected u64)"))?,
            );
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--issue-id=") {
            if issue_id.is_some() {
                return Err("duplicate --issue-id flag".to_string());
            }
            if value.is_empty() {
                return Err("--issue-id requires a non-empty value".to_string());
            }
            issue_id = Some(
                value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --issue-id value `{value}` (expected u64)"))?,
            );
            idx += 1;
            continue;
        }
        if token == "--activate" {
            if drain_activate {
                return Err("duplicate --activate flag".to_string());
            }
            drain_activate = true;
            idx += 1;
            continue;
        }
        if token == "--deactivate" {
            if drain_deactivate {
                return Err("duplicate --deactivate flag".to_string());
            }
            drain_deactivate = true;
            idx += 1;
            continue;
        }
        if token == "--intent" {
            if intent_path.is_some() {
                return Err("duplicate --intent flag".to_string());
            }
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--intent requires a file path".to_string())?;
            if value.starts_with("--") {
                return Err("--intent requires a non-flag file path".to_string());
            }
            intent_path = Some(PathBuf::from(value));
            idx += 2;
            continue;
        }
        if token == "--infra-intent" {
            if infra_intent_path.is_some() {
                return Err("duplicate --infra-intent flag".to_string());
            }
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--infra-intent requires a file path".to_string())?;
            if value.starts_with("--") {
                return Err("--infra-intent requires a non-flag file path".to_string());
            }
            infra_intent_path = Some(PathBuf::from(value));
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--infra-intent=") {
            if infra_intent_path.is_some() {
                return Err("duplicate --infra-intent flag".to_string());
            }
            if value.is_empty() {
                return Err("--infra-intent requires a non-empty file path".to_string());
            }
            infra_intent_path = Some(PathBuf::from(value));
            idx += 1;
            continue;
        }
        if let Some(value) = token.strip_prefix("--intent=") {
            if intent_path.is_some() {
                return Err("duplicate --intent flag".to_string());
            }
            if value.is_empty() {
                return Err("--intent requires a non-empty file path".to_string());
            }
            intent_path = Some(PathBuf::from(value));
            idx += 1;
            continue;
        }
        if token == "--intake-key" {
            if intake_key.is_some() {
                return Err("duplicate --intake-key flag".to_string());
            }
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--intake-key requires a value".to_string())?;
            if value.starts_with("--") {
                return Err("--intake-key requires a non-flag value".to_string());
            }
            intake_key = Some(value.clone());
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--intake-key=") {
            if intake_key.is_some() {
                return Err("duplicate --intake-key flag".to_string());
            }
            if value.is_empty() {
                return Err("--intake-key requires a non-empty value".to_string());
            }
            intake_key = Some(value.to_string());
            idx += 1;
            continue;
        }
        if token == "--stage" {
            if stage.is_some() {
                return Err("duplicate --stage flag".to_string());
            }
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--stage requires a value".to_string())?;
            stage = Some(parse_stage(value)?);
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--stage=") {
            if stage.is_some() {
                return Err("duplicate --stage flag".to_string());
            }
            stage = Some(parse_stage(value)?);
            idx += 1;
            continue;
        }
        return Err(format!("unknown flag `{token}`"));
    }

    if command == SdlcCommand::Intake && intent_path.is_none() {
        return Err("intake requires --intent <path>".to_string());
    }
    if command == SdlcCommand::Worker && intent_path.is_some() {
        return Err("worker does not accept --intent".to_string());
    }
    if command != SdlcCommand::Worker && command != SdlcCommand::Issue && infra_intent_path.is_some()
    {
        return Err("--infra-intent is only valid for worker or issue".to_string());
    }
    if command != SdlcCommand::Worker
        && command != SdlcCommand::Issue
        && emit_pending_exit_code
    {
        return Err("--emit-pending-exit-code is only valid for worker or issue".to_string());
    }
    if command == SdlcCommand::AwaitApproval && intake_key.is_none() {
        return Err("await-approval requires --intake-key <value>".to_string());
    }
    if command == SdlcCommand::AwaitApproval && intent_path.is_some() {
        return Err("await-approval does not accept --intent".to_string());
    }
    if command == SdlcCommand::Transition && intake_key.is_none() {
        return Err("transition requires --intake-key <value>".to_string());
    }
    if command == SdlcCommand::Transition && stage.is_none() {
        return Err("transition requires --stage <idea|design|design-review|accepted|implementation|closed>".to_string());
    }
    if command == SdlcCommand::Transition && intent_path.is_some() {
        return Err("transition does not accept --intent".to_string());
    }
    if command == SdlcCommand::Issue && issue_id.is_none() {
        return Err("issue requires --issue-id <value>".to_string());
    }
    if command == SdlcCommand::Issue && intent_path.is_some() {
        return Err("issue does not accept --intent".to_string());
    }
    if command == SdlcCommand::Issue && intake_key.is_some() {
        return Err("issue does not accept --intake-key".to_string());
    }
    if command == SdlcCommand::Issue && stage.is_some() {
        return Err("issue does not accept --stage".to_string());
    }
    if command != SdlcCommand::Issue && issue_id.is_some() {
        return Err("--issue-id is only valid for issue".to_string());
    }
    if command != SdlcCommand::Drain && (drain_activate || drain_deactivate) {
        return Err("--activate/--deactivate are only valid for drain".to_string());
    }
    if drain_activate && drain_deactivate {
        return Err("drain command cannot use --activate and --deactivate together".to_string());
    }

    Ok(CliArgs {
        command,
        intent_path,
        infra_intent_path,
        intake_key,
        issue_id,
        stage,
        dry_run,
        emit_pending_exit_code,
        drain_activate,
        drain_deactivate,
    })
}

fn parse_stage(value: &str) -> Result<IssueLifecycleStage, String> {
    match value {
        "idea" => Ok(IssueLifecycleStage::Idea),
        "design" => Ok(IssueLifecycleStage::Design),
        "design-review" => Ok(IssueLifecycleStage::DesignReview),
        "accepted" => Ok(IssueLifecycleStage::Accepted),
        "implementation" => Ok(IssueLifecycleStage::Implementation),
        "closed" => Ok(IssueLifecycleStage::Closed),
        _ => Err(format!(
            "invalid stage `{value}`; expected one of: idea, design, design-review, accepted, implementation, closed"
        )),
    }
}

fn run_intake(intent_path: Option<&PathBuf>, dry_run: bool) -> Result<(), String> {
    let intent_path = intent_path.ok_or_else(|| "intake requires --intent <path>".to_string())?;
    let intent = load_intent(intent_path)?;
    validate_intent(&intent)?;

    let computed_run_key = compute_run_key(&intent);
    let design_prompt = build_design_prompt(&DesignRequest {
        title: intent.title.clone(),
        idea: intent.objective.clone(),
        context: intent.notes.clone(),
        acceptance_tests: intent.acceptance_tests.clone(),
    });
    let effective_run_key = intent
        .tracking
        .run_key
        .clone()
        .unwrap_or_else(|| computed_run_key.clone());
    if effective_run_key != computed_run_key {
        return Err(format!(
            "intent tracking.run_key mismatch: expected `{computed_run_key}`, got `{effective_run_key}`"
        ));
    }
    let trace_linkage = if dry_run {
        None
    } else {
        Some(capture_trace_linkage(&intent, &effective_run_key)?)
    };

    if dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "command": "intake",
                "mode": "dry-run",
                "intent_id": intent.intent_id,
                "intake_key": intent.idempotency.intake_key,
                "run_key": effective_run_key,
                "provider": intent.provider,
                "design_prompt": design_prompt,
                "trace_linkage_required": true,
            }))
            .map_err(|error| format!("failed to serialize intake dry-run output: {error}"))?
        );
        return Ok(());
    }

    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
    let artifact_ledger_path = artifact_ledger_path();
    let mut artifacts = load_artifact_ledger(&artifact_ledger_path)?;
    let now = epoch_millis();
    let intake_key = intent.idempotency.intake_key.clone();
    let mut idempotent = false;
    match ledger.entries.get_mut(&intake_key) {
        Some(existing) => {
            if existing.intent_id != intent.intent_id {
                return Err(format!(
                    "intake key conflict: key `{}` already bound to intent `{}`",
                    intake_key, existing.intent_id
                ));
            }
            existing.run_key = effective_run_key.clone();
            existing.issue_id = intent.tracking.issue_id.or(existing.issue_id);
            existing.policy_version = intent.idempotency.policy_version.clone();
            existing.trace_linkage = trace_linkage.clone();
            if existing.created_at_epoch_ms == 0 {
                existing.created_at_epoch_ms = now;
            }
            existing.updated_at_epoch_ms = now;
            idempotent = true;
        }
        None => {
            ledger.entries.insert(
                intake_key.clone(),
                IntakeRecord {
                    intent_id: intent.intent_id.clone(),
                    run_key: effective_run_key.clone(),
                    issue_id: intent.tracking.issue_id,
                    policy_version: intent.idempotency.policy_version.clone(),
                    stage: IssueLifecycleStage::Idea,
                    awaiting_approval: false,
                    terminalized: false,
                    retry: RetryState::default(),
                    awaiting_approval_since_epoch_ms: None,
                    trace_linkage: trace_linkage.clone(),
                    created_at_epoch_ms: now,
                    updated_at_epoch_ms: now,
                },
            );
        }
    }

    save_intake_ledger(&ledger_path, &ledger)?;
    let design_prompt_hash = gunbc_infra::hash::ContentHash::from_bytes(design_prompt.as_bytes())
        .as_str()
        .to_string();
    let artifact_outcome = upsert_provisional_artifact(
        &mut artifacts,
        &intake_key,
        &effective_run_key,
        &design_prompt_hash,
        now,
    )?;
    save_artifact_ledger(&artifact_ledger_path, &artifacts)?;
    let artifact_status = match artifact_outcome {
        ArtifactUpsertOutcome::Inserted => "inserted",
        ArtifactUpsertOutcome::Updated => "updated",
        ArtifactUpsertOutcome::Noop => "noop",
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "intake",
            "mode": "real",
            "intent_id": intent.intent_id,
            "intake_key": intake_key,
            "run_key": effective_run_key,
            "idempotent": idempotent,
            "ledger_path": ledger_path.display().to_string(),
            "design_prompt": design_prompt,
            "artifact_ledger_path": artifact_ledger_path.display().to_string(),
            "artifact_status": artifact_status,
            "trace_linkage": trace_linkage,
        }))
        .map_err(|error| format!("failed to serialize intake output: {error}"))?
    );

    Ok(())
}

fn run_worker(
    dry_run: bool,
    emit_pending_exit_code: bool,
    infra_intent_path: Option<&PathBuf>,
    issue_filter: Option<u64>,
    command_label: &str,
) -> Result<(), String> {
    let preflight = if dry_run {
        WorkerPreflightSummary {
            status: "skipped-dry-run".to_string(),
            intent_path: default_infra_intent_path().display().to_string(),
            intent_id: "n/a".to_string(),
            environment: "n/a".to_string(),
            runtime_profile: "n/a".to_string(),
            checked_components: Vec::new(),
        }
    } else {
        let path = infra_intent_path
            .cloned()
            .unwrap_or_else(default_infra_intent_path);
        run_worker_preflight(&path)?
    };

    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
    let claim_ledger_path = claim_ledger_path();
    let mut claim_ledger = load_claim_ledger(&claim_ledger_path)?;
    let run_state_path = run_state_ledger_path();
    let mut run_state = load_run_state_ledger(&run_state_path)?;
    let mode = if dry_run { "dry-run" } else { "real" };
    let now = epoch_millis();

    let drain_flag = drain_flag_path();
    let drain_active = !dry_run && drain_flag.exists();
    if drain_active {
        let released = release_worker_owned_claims_for_drain(&mut claim_ledger);
        save_claim_ledger(&claim_ledger_path, &claim_ledger)?;
        let output = json!({
            "command": command_label,
            "mode": mode,
            "report_generated_at_epoch_ms": now,
            "issue_filter": issue_filter,
            "pending_count": 0,
            "intake_keys": [],
            "ready_to_run": [],
            "replay_skipped": [],
            "executed_runs": [],
            "acquired_claims": [],
            "released_claims": released,
            "claim_conflicts": [],
            "terminalized": [],
            "terminal_failures": {},
            "awaiting_approval": [],
            "skipped_missing_issue": [],
            "skipped_terminalized": [],
            "skipped_retry_backoff": [],
            "ledger_path": ledger_path.display().to_string(),
            "claim_ledger_path": claim_ledger_path.display().to_string(),
            "run_state_path": run_state_path.display().to_string(),
            "execution_report_path": execution_report_path().display().to_string(),
            "reconcile_actions": [],
            "preflight": preflight,
            "drain": {
                "active": true,
                "flag_path": drain_flag.display().to_string(),
                "released_claim_count": released.len(),
            },
            "summary": {
                "intake_total": 0,
                "ready_to_run_count": 0,
                "executed_count": 0,
                "terminalized_count": 0,
                "awaiting_approval_count": 0,
                "claim_conflict_count": 0,
                "replay_skipped_count": 0,
                "retry_backoff_deferred_count": 0,
            },
            "metrics": {
                "stage_duration_ms": {},
                "approval_latency_ms": {},
                "retry_attempts": {},
                "llm_cost_units": {},
                "cost_units": {
                    "claim_acquire_attempts": 0,
                    "reconcile_actions": 0,
                    "llm_estimated_total_units": 0,
                }
            }
        });
        if !dry_run {
            save_execution_report(&output)?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .map_err(|error| format!("failed to serialize worker drain output: {error}"))?
        );
        if emit_pending_exit_code {
            std::process::exit(0);
        }
        return Ok(());
    }

    let mut intake_keys: Vec<String> = ledger.entries.keys().cloned().collect();
    intake_keys.sort();
    if let Some(issue_id) = issue_filter {
        intake_keys.retain(|intake_key| {
            ledger
                .entries
                .get(intake_key)
                .and_then(|record| record.issue_id)
                == Some(issue_id)
        });
        if intake_keys.is_empty() {
            return Err(format!("no intake entries found for issue_id `{issue_id}`"));
        }
    }
    let mut skipped_missing_issue = Vec::new();
    let mut skipped_terminalized = Vec::new();
    let mut skipped_retry_backoff = Vec::new();
    let mut claim_conflicts = Vec::new();
    let mut acquired_claims = Vec::new();
    let mut replay_skipped = Vec::new();
    let mut executed_runs = Vec::new();
    let mut reconcile_inputs = Vec::new();
    let mut terminal_failures = BTreeMap::new();
    let mut stage_duration_ms = BTreeMap::new();
    let mut approval_latency_ms = BTreeMap::new();
    let mut retry_attempts = BTreeMap::new();
    let mut llm_cost_units = BTreeMap::new();
    let mut claim_acquire_attempts: u64 = 0;
    let mut awaiting_approval = Vec::new();

    for intake_key in &intake_keys {
        let Some(record) = ledger.entries.get_mut(intake_key) else {
            continue;
        };
        stage_duration_ms.insert(
            intake_key.clone(),
            now.saturating_sub(record.updated_at_epoch_ms),
        );
        retry_attempts.insert(intake_key.clone(), record.retry.attempts);
        llm_cost_units.insert(
            intake_key.clone(),
            estimated_llm_cost_units(record.stage, record.awaiting_approval),
        );
        if let Some(since) = record.awaiting_approval_since_epoch_ms {
            approval_latency_ms.insert(intake_key.clone(), now.saturating_sub(since));
        }
        if record.awaiting_approval {
            awaiting_approval.push(intake_key.clone());
        }
        if record.terminalized {
            skipped_terminalized.push(intake_key.clone());
            if let Some(last_error) = &record.retry.last_error {
                terminal_failures.insert(intake_key.clone(), last_error.clone());
            }
            continue;
        }
        if !retry_ready(&record.retry, now) {
            skipped_retry_backoff.push(intake_key.clone());
            continue;
        }
        let Some(issue_id) = record.issue_id else {
            skipped_missing_issue.push(intake_key.clone());
            continue;
        };
        let claim_slot = claim_slot_key(issue_id, record.stage);
        let claim_owner = format!("gunbc-sdlc-worker:{intake_key}");
        claim_acquire_attempts = claim_acquire_attempts.saturating_add(1);
        let claim_result = try_acquire_claim(
            &mut claim_ledger,
            &claim_slot,
            &claim_owner,
            now,
            CLAIM_LEASE_TTL_MS,
        );
        match claim_result {
            ClaimAcquireResult::Conflict { current_owner } => {
                let has_budget = register_retry_failure(
                    &mut record.retry,
                    now,
                    RETRY_BASE_BACKOFF_MS,
                    format!("claim conflict with owner `{current_owner}`"),
                );
                if !has_budget {
                    record.terminalized = true;
                    terminal_failures.insert(
                        intake_key.clone(),
                        record
                            .retry
                            .last_error
                            .clone()
                            .unwrap_or_else(|| "retry_budget_exhausted".to_string()),
                    );
                }
                claim_conflicts.push(intake_key.clone());
                continue;
            }
            ClaimAcquireResult::Acquired | ClaimAcquireResult::StaleReclaimed { .. } => {
                acquired_claims.push(intake_key.clone());
            }
            ClaimAcquireResult::AlreadyOwned => {
                if !heartbeat_claim(
                    &mut claim_ledger,
                    &claim_slot,
                    &claim_owner,
                    now,
                    CLAIM_LEASE_TTL_MS,
                ) {
                    let has_budget = register_retry_failure(
                        &mut record.retry,
                        now,
                        RETRY_BASE_BACKOFF_MS,
                        "claim heartbeat failed for existing owner".to_string(),
                    );
                    if !has_budget {
                        record.terminalized = true;
                        terminal_failures.insert(
                            intake_key.clone(),
                            record
                                .retry
                                .last_error
                                .clone()
                                .unwrap_or_else(|| "retry_budget_exhausted".to_string()),
                        );
                    }
                    claim_conflicts.push(intake_key.clone());
                    continue;
                }
                acquired_claims.push(intake_key.clone());
            }
        }

        let claim_snapshot = claim_ledger.claims.get(&claim_slot).cloned();
        reconcile_inputs.push(ReconcileEntry {
            intake_key: intake_key.clone(),
            claim_slot,
            claim_owner: claim_snapshot.as_ref().map(|claim| claim.owner.clone()),
            claim_expires_at_epoch_ms: claim_snapshot
                .as_ref()
                .map(|claim| claim.lease_expires_at_epoch_ms),
            awaiting_approval: record.awaiting_approval,
            retry: record.retry.clone(),
        });
    }

    let reconcile_plan = reconcile_entries(&reconcile_inputs, now);
    let mut ready_to_run = Vec::new();
    let mut released_claims = Vec::new();
    let mut terminalized = Vec::new();
    for action in &reconcile_plan.actions {
        match action {
            ReconcileAction::ReadyToRun { intake_key } => {
                let Some(record) = ledger.entries.get(intake_key) else {
                    continue;
                };
                if should_replay_skip(&run_state, intake_key, &record.run_key) {
                    replay_skipped.push(intake_key.clone());
                    continue;
                }
                ready_to_run.push(intake_key.clone());
                if !dry_run {
                    mark_run_completed(&mut run_state, intake_key, &record.run_key, now);
                    executed_runs.push(intake_key.clone());
                }
            }
            ReconcileAction::ReleaseClaim {
                intake_key,
                claim_slot,
                owner,
                ..
            } => {
                if release_claim(&mut claim_ledger, claim_slot, owner) {
                    released_claims.push(intake_key.clone());
                }
            }
            ReconcileAction::Terminalize { intake_key, reason } => {
                if let Some(record) = ledger.entries.get_mut(intake_key) {
                    record.terminalized = true;
                    if !dry_run {
                        mark_run_failed(&mut run_state, intake_key, &record.run_key, now);
                    }
                }
                terminal_failures.insert(intake_key.clone(), reason.clone());
                terminalized.push(intake_key.clone());
            }
        }
    }

    if !dry_run {
        save_intake_ledger(&ledger_path, &ledger)?;
        save_claim_ledger(&claim_ledger_path, &claim_ledger)?;
        save_run_state_ledger(&run_state_path, &run_state)?;
    }

    let pending_count = intake_keys
        .len()
        .saturating_sub(skipped_terminalized.len())
        .saturating_sub(terminalized.len());
    let llm_estimated_total_units: u64 = llm_cost_units.values().copied().sum();
    let output = json!({
        "command": command_label,
        "report_generated_at_epoch_ms": now,
        "issue_filter": issue_filter,
        "mode": mode,
        "pending_count": pending_count,
        "intake_keys": intake_keys,
        "ready_to_run": ready_to_run,
        "replay_skipped": replay_skipped,
        "executed_runs": executed_runs,
        "acquired_claims": acquired_claims,
        "released_claims": released_claims,
        "claim_conflicts": claim_conflicts,
        "terminalized": terminalized,
        "terminal_failures": terminal_failures,
        "awaiting_approval": awaiting_approval,
        "skipped_missing_issue": skipped_missing_issue,
        "skipped_terminalized": skipped_terminalized,
        "skipped_retry_backoff": skipped_retry_backoff,
        "ledger_path": ledger_path.display().to_string(),
        "claim_ledger_path": claim_ledger_path.display().to_string(),
        "run_state_path": run_state_path.display().to_string(),
        "execution_report_path": execution_report_path().display().to_string(),
        "reconcile_actions": reconcile_plan.actions,
        "preflight": preflight,
        "drain": {
            "active": false,
            "flag_path": drain_flag.display().to_string(),
        },
        "summary": {
            "intake_total": intake_keys.len(),
            "ready_to_run_count": ready_to_run.len(),
            "executed_count": executed_runs.len(),
            "terminalized_count": terminalized.len(),
            "awaiting_approval_count": awaiting_approval.len(),
            "claim_conflict_count": claim_conflicts.len(),
            "replay_skipped_count": replay_skipped.len(),
            "retry_backoff_deferred_count": skipped_retry_backoff.len(),
        },
        "metrics": {
            "stage_duration_ms": stage_duration_ms,
            "approval_latency_ms": approval_latency_ms,
            "retry_attempts": retry_attempts,
            "llm_cost_units": llm_cost_units,
            "cost_units": {
                "claim_acquire_attempts": claim_acquire_attempts,
                "reconcile_actions": reconcile_plan.actions.len(),
                "llm_estimated_total_units": llm_estimated_total_units,
            }
        }
    });
    if !dry_run {
        save_execution_report(&output)?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("failed to serialize worker output: {error}"))?
    );
    if emit_pending_exit_code && !awaiting_approval.is_empty() {
        std::process::exit(42);
    }
    Ok(())
}

fn run_await_approval(intake_key: Option<&str>, dry_run: bool) -> Result<(), String> {
    let intake_key = intake_key.ok_or_else(|| "await-approval requires --intake-key".to_string())?;
    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
    let Some(record) = ledger.entries.get_mut(intake_key) else {
        return Err(format!("unknown intake key `{intake_key}`"));
    };
    if record.terminalized {
        return Err(format!(
            "cannot await approval for terminalized intake key `{intake_key}`"
        ));
    }
    record.awaiting_approval = true;
    if record.awaiting_approval_since_epoch_ms.is_none() {
        record.awaiting_approval_since_epoch_ms = Some(epoch_millis());
    }
    record.updated_at_epoch_ms = epoch_millis();

    if !dry_run {
        save_intake_ledger(&ledger_path, &ledger)?;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "await-approval",
            "mode": if dry_run { "dry-run" } else { "real" },
            "intake_key": intake_key,
            "awaiting_approval": true,
            "ledger_path": ledger_path.display().to_string(),
        }))
        .map_err(|error| format!("failed to serialize await-approval output: {error}"))?
    );
    Ok(())
}

fn run_transition(
    intake_key: Option<&str>,
    next_stage: Option<IssueLifecycleStage>,
    dry_run: bool,
) -> Result<(), String> {
    let intake_key = intake_key.ok_or_else(|| "transition requires --intake-key".to_string())?;
    let next_stage = next_stage.ok_or_else(|| "transition requires --stage".to_string())?;
    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
    let artifact_ledger_path = artifact_ledger_path();
    let mut artifact_ledger = load_artifact_ledger(&artifact_ledger_path)?;
    let Some(record) = ledger.entries.get_mut(intake_key) else {
        return Err(format!("unknown intake key `{intake_key}`"));
    };
    if record.terminalized {
        return Err(format!(
            "cannot transition terminalized intake key `{intake_key}`"
        ));
    }
    let current_stage = record.stage;
    validate_stage_transition(current_stage, next_stage)?;
    let stage_labels_after_cas = compare_and_set_stage_label(
        &[current_stage.as_label().to_string()],
        current_stage,
        next_stage,
    )?;

    let now = epoch_millis();
    let mut canonical_artifact_status = None;
    if next_stage == IssueLifecycleStage::Accepted {
        let provisional = artifact_ledger
            .records
            .get(&provisional_marker(intake_key))
            .cloned()
            .ok_or_else(|| {
                format!(
                    "cannot promote canonical artifact for `{intake_key}`: missing provisional marker"
                )
            })?;
        if provisional.run_key != record.run_key {
            return Err(format!(
                "cannot promote canonical artifact for `{intake_key}`: provisional run_key `{}` does not match intake run_key `{}`",
                provisional.run_key, record.run_key
            ));
        }
        let outcome = promote_to_canonical_artifact(
            &mut artifact_ledger,
            intake_key,
            &record.run_key,
            &provisional.content_hash,
            now,
        )?;
        canonical_artifact_status = Some(match outcome {
            ArtifactUpsertOutcome::Inserted => "inserted",
            ArtifactUpsertOutcome::Updated => "updated",
            ArtifactUpsertOutcome::Noop => "noop",
        });
    }

    record.stage = next_stage;
    record.updated_at_epoch_ms = now;

    if !dry_run {
        save_intake_ledger(&ledger_path, &ledger)?;
        if canonical_artifact_status.is_some() {
            save_artifact_ledger(&artifact_ledger_path, &artifact_ledger)?;
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "transition",
            "mode": if dry_run { "dry-run" } else { "real" },
            "intake_key": intake_key,
            "from_stage": current_stage.as_label(),
            "to_stage": next_stage.as_label(),
            "stage_labels_after_cas": stage_labels_after_cas,
            "ledger_path": ledger_path.display().to_string(),
            "canonical_artifact_status": canonical_artifact_status,
            "artifact_ledger_path": artifact_ledger_path.display().to_string(),
        }))
        .map_err(|error| format!("failed to serialize transition output: {error}"))?
    );
    Ok(())
}

fn run_drain(activate: bool, deactivate: bool, dry_run: bool) -> Result<(), String> {
    let flag_path = drain_flag_path();
    let action = if activate {
        "activate"
    } else if deactivate {
        "deactivate"
    } else {
        "status"
    };

    let mut active = flag_path.exists();
    if action == "activate" {
        if !dry_run {
            if let Some(parent) = flag_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create drain flag directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
            std::fs::write(&flag_path, format!("activated_at_ms={}\n", epoch_millis())).map_err(
                |error| {
                    format!(
                        "failed to activate drain flag at {}: {error}",
                        flag_path.display()
                    )
                },
            )?;
            active = true;
        } else {
            active = true;
        }
    } else if action == "deactivate" {
        if !dry_run {
            if flag_path.exists() {
                std::fs::remove_file(&flag_path).map_err(|error| {
                    format!(
                        "failed to deactivate drain flag at {}: {error}",
                        flag_path.display()
                    )
                })?;
            }
            active = false;
        } else {
            active = false;
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "drain",
            "mode": if dry_run { "dry-run" } else { "real" },
            "action": action,
            "active": active,
            "flag_path": flag_path.display().to_string(),
        }))
        .map_err(|error| format!("failed to serialize drain output: {error}"))?
    );
    Ok(())
}

fn load_intent(path: &Path) -> Result<IntentSheet, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read intent file {}: {error}", path.display()))?;
    serde_yaml::from_str::<IntentSheet>(&content)
        .map_err(|error| format!("failed to parse intent YAML {}: {error}", path.display()))
}

fn validate_intent(intent: &IntentSheet) -> Result<(), String> {
    require_non_empty("intent_id", &intent.intent_id)?;
    require_non_empty("title", &intent.title)?;
    require_non_empty("objective", &intent.objective)?;
    require_non_empty("provider", &intent.provider)?;
    require_non_empty("owner", &intent.owner)?;
    require_non_empty("priority", &intent.priority)?;
    require_non_empty("idempotency.intake_key", &intent.idempotency.intake_key)?;
    require_non_empty("idempotency.policy_version", &intent.idempotency.policy_version)?;
    require_non_empty("update_strategy.comment_mode", &intent.update_strategy.comment_mode)?;
    require_non_empty(
        "update_strategy.transition_mode",
        &intent.update_strategy.transition_mode,
    )?;

    let capabilities = match intent.provider.as_str() {
        "github" => SdlcIssueCapabilities::github(),
        _ => {
            return Err(format!(
                "unsupported provider `{}`; only `github` is supported",
                intent.provider
            ))
        }
    };
    ensure_sdlc_issue_capabilities(capabilities).map_err(|error| {
        format!(
            "provider capability gate failed for `{}`: {error}",
            intent.provider
        )
    })?;
    if intent.update_strategy.comment_mode != "upsert-by-marker" {
        return Err(format!(
            "unsupported update_strategy.comment_mode `{}`; expected `upsert-by-marker`",
            intent.update_strategy.comment_mode
        ));
    }
    if intent.update_strategy.transition_mode != "compare-and-set" {
        return Err(format!(
            "unsupported update_strategy.transition_mode `{}`; expected `compare-and-set`",
            intent.update_strategy.transition_mode
        ));
    }
    if intent.success_criteria.is_empty() {
        return Err("success_criteria must contain at least one entry".to_string());
    }
    if intent.acceptance_tests.is_empty() {
        return Err("acceptance_tests must contain at least one entry".to_string());
    }
    if intent.scope.in_scope.is_empty() && intent.scope.out.is_empty() {
        return Err("scope.in and scope.out cannot both be empty".to_string());
    }
    if intent.links.docs.is_empty() && intent.links.related_issues.is_empty() {
        return Err("links.docs and links.related_issues cannot both be empty".to_string());
    }
    if intent.constraints.is_empty() {
        return Err("constraints must contain at least one entry".to_string());
    }
    let _ = intent.notes.as_deref();
    Ok(())
}

fn load_infra_intent(path: &Path) -> Result<InfraIntentSheet, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read infra intent file {}: {error}", path.display()))?;
    serde_yaml::from_str::<InfraIntentSheet>(&content)
        .map_err(|error| format!("failed to parse infra intent YAML {}: {error}", path.display()))
}

fn validate_infra_intent(intent: &InfraIntentSheet) -> Result<(), String> {
    require_non_empty("infra.schema_version", &intent.schema_version)?;
    require_non_empty("infra.intent_id", &intent.intent_id)?;
    require_non_empty("infra.environment", &intent.environment)?;
    require_non_empty("infra.runtime_profile", &intent.runtime_profile)?;
    require_non_empty("infra.provider", &intent.provider)?;
    require_non_empty("infra.policy_version", &intent.policy_version)?;
    require_non_empty(
        "infra.components.claim_store.backend",
        &intent.components.claim_store.backend,
    )?;
    require_non_empty("infra.components.claim_store.dsn", &intent.components.claim_store.dsn)?;
    require_non_empty(
        "infra.components.outcome_ledger.backend",
        &intent.components.outcome_ledger.backend,
    )?;
    require_non_empty(
        "infra.components.outcome_ledger.dsn",
        &intent.components.outcome_ledger.dsn,
    )?;
    require_non_empty(
        "infra.components.secrets.credential_policy_profile",
        &intent.components.secrets.credential_policy_profile,
    )?;
    require_non_empty("infra.components.metrics.sink", &intent.components.metrics.sink)?;
    require_non_empty(
        "infra.components.metrics.namespace",
        &intent.components.metrics.namespace,
    )?;
    require_non_empty("infra.drift.reconcile_mode", &intent.drift.reconcile_mode)?;
    if intent.schema_version != "1" {
        return Err(format!(
            "unsupported infra schema_version `{}`; expected `1`",
            intent.schema_version
        ));
    }

    if intent.components.claim_store.backend != "sqlite" {
        return Err(format!(
            "unsupported infra claim_store backend `{}`; expected `sqlite`",
            intent.components.claim_store.backend
        ));
    }
    if intent.components.outcome_ledger.backend != "sqlite" {
        return Err(format!(
            "unsupported infra outcome_ledger backend `{}`; expected `sqlite`",
            intent.components.outcome_ledger.backend
        ));
    }
    if !intent.safety.fail_closed_on_missing_prereqs {
        return Err("infra safety.fail_closed_on_missing_prereqs must be true".to_string());
    }
    if !intent.safety.require_capability_gate {
        return Err("infra safety.require_capability_gate must be true".to_string());
    }
    if intent.provider != "github" {
        return Err(format!(
            "unsupported infra provider `{}`; expected `github`",
            intent.provider
        ));
    }
    if intent.components.metrics.sink != "stdout" {
        return Err(format!(
            "unsupported infra components.metrics.sink `{}`; expected `stdout`",
            intent.components.metrics.sink
        ));
    }
    if intent.components.claim_store.dsn == intent.components.outcome_ledger.dsn {
        return Err(
            "infra components.claim_store.dsn and components.outcome_ledger.dsn must be distinct"
                .to_string(),
        );
    }
    if intent.components.secrets.required_refs.is_empty() {
        return Err(
            "infra components.secrets.required_refs must contain at least one secret reference"
                .to_string(),
        );
    }
    let mut required_refs = BTreeSet::new();
    for required_ref in &intent.components.secrets.required_refs {
        let trimmed = required_ref.trim();
        if trimmed.is_empty() {
            return Err(
                "infra components.secrets.required_refs cannot contain empty references"
                    .to_string(),
            );
        }
        required_refs.insert(trimmed.to_string());
    }
    if !required_refs.contains("github-token") {
        return Err(
            "infra components.secrets.required_refs must include `github-token` for provider `github`"
                .to_string(),
        );
    }
    if !required_refs.contains("openai-api-key") && !required_refs.contains("anthropic-api-key") {
        return Err(
            "infra components.secrets.required_refs must include at least one LLM credential reference (`openai-api-key` or `anthropic-api-key`)"
                .to_string(),
        );
    }
    if intent.launch.worker_count == 0 {
        return Err("infra launch.worker_count must be >= 1".to_string());
    }
    match intent.runtime_profile.as_str() {
        "stateless-fleet" => {
            if !(5..=10).contains(&intent.launch.worker_count) {
                return Err(
                    "infra runtime_profile `stateless-fleet` requires launch.worker_count between 5 and 10".to_string(),
                );
            }
        }
        "local-co-located" => {
            if intent.launch.worker_count != 1 {
                return Err(
                    "infra runtime_profile `local-co-located` requires launch.worker_count = 1"
                        .to_string(),
                );
            }
        }
        other => {
            return Err(format!(
                "unsupported infra runtime_profile `{other}`; expected `stateless-fleet` or `local-co-located`"
            ));
        }
    }
    if intent.launch.heartbeat_seconds == 0 {
        return Err("infra launch.heartbeat_seconds must be >= 1".to_string());
    }
    if intent.launch.lease_ttl_seconds < intent.launch.heartbeat_seconds {
        return Err(
            "infra launch.lease_ttl_seconds must be >= launch.heartbeat_seconds".to_string(),
        );
    }
    if intent.launch.poll_interval_seconds == 0 {
        return Err("infra launch.poll_interval_seconds must be >= 1".to_string());
    }
    if intent.drift.reconcile_interval_minutes == 0 {
        return Err("infra drift.reconcile_interval_minutes must be >= 1".to_string());
    }
    if intent.drift.reconcile_mode != "plan-then-apply" {
        return Err(format!(
            "unsupported infra drift.reconcile_mode `{}`; expected `plan-then-apply`",
            intent.drift.reconcile_mode
        ));
    }
    let _ = intent.notes.as_deref();
    Ok(())
}

fn run_worker_preflight(intent_path: &Path) -> Result<WorkerPreflightSummary, String> {
    let intent = load_infra_intent(intent_path)?;
    validate_infra_intent(&intent)?;
    ensure_sqlite_store_ready(
        "components.claim_store.dsn",
        &intent.components.claim_store.dsn,
    )?;
    ensure_sqlite_store_ready(
        "components.outcome_ledger.dsn",
        &intent.components.outcome_ledger.dsn,
    )?;

    let checked_components = vec![
        "claim_store".to_string(),
        "outcome_ledger".to_string(),
        "secrets".to_string(),
        "metrics".to_string(),
        "launch".to_string(),
        "drift".to_string(),
        "safety".to_string(),
    ];
    Ok(WorkerPreflightSummary {
        status: "ok".to_string(),
        intent_path: intent_path.display().to_string(),
        intent_id: intent.intent_id,
        environment: intent.environment,
        runtime_profile: intent.runtime_profile,
        checked_components,
    })
}

fn ensure_sqlite_store_ready(field_name: &str, dsn: &str) -> Result<(), String> {
    if dsn.trim().is_empty() {
        return Err(format!("infra {field_name} cannot be empty"));
    }
    let store_path = PathBuf::from(dsn);
    let parent = store_path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "infra preflight failed creating parent directory for {field_name} `{}`: {error}",
            store_path.display()
        )
    })?;
    Ok(())
}

fn require_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field_name} cannot be empty"));
    }
    Ok(())
}

fn compute_run_key(intent: &IntentSheet) -> String {
    format!(
        "sdlc::{provider}::{intent_id}::{intake_key}::v{policy_version}",
        provider = intent.provider.trim(),
        intent_id = intent.intent_id.trim(),
        intake_key = intent.idempotency.intake_key.trim(),
        policy_version = intent.idempotency.policy_version.trim(),
    )
}

fn capture_trace_linkage(intent: &IntentSheet, run_key: &str) -> Result<TraceLinkage, String> {
    let repo_root = run_git(["rev-parse", "--show-toplevel"])?;
    let branch = run_git(["rev-parse", "--abbrev-ref", "HEAD"])?;
    let commit = run_git(["rev-parse", "HEAD"])?;
    if branch == "HEAD" {
        return Err(
            "trace linkage requires a named branch (detached HEAD is not supported)".to_string(),
        );
    }
    let issue_label = intent
        .tracking
        .issue_id
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unassigned".to_string());
    let linkage_key = format!(
        "intent={};issue={};run_key={};branch={};commit={}",
        intent.intent_id, issue_label, run_key, branch, commit
    );
    Ok(TraceLinkage {
        repo_root,
        branch,
        commit,
        intent_id: intent.intent_id.clone(),
        issue_id: intent.tracking.issue_id,
        run_key: run_key.to_string(),
        linkage_key,
    })
}

fn run_git<const N: usize>(args: [&str; N]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("failed to execute git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "trace linkage requires git metadata; `git {}` failed: {}",
            args.join(" "),
            if stderr.is_empty() {
                "unknown git error".to_string()
            } else {
                stderr
            }
        ));
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        return Err(format!(
            "trace linkage requires git metadata; `git {}` returned empty output",
            args.join(" ")
        ));
    }
    Ok(value)
}

fn intake_ledger_path() -> PathBuf {
    PathBuf::from("target").join("sdlc").join("intake-ledger.json")
}

fn claim_ledger_path() -> PathBuf {
    PathBuf::from("target").join("sdlc").join("claim-ledger.json")
}

fn drain_flag_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("worker-drain.flag")
}

fn artifact_ledger_path() -> PathBuf {
    PathBuf::from("target").join("sdlc").join("artifact-ledger.json")
}

fn run_state_ledger_path() -> PathBuf {
    PathBuf::from("target").join("sdlc").join("run-state-ledger.json")
}

fn execution_report_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("execution-report.json")
}

fn default_infra_intent_path() -> PathBuf {
    PathBuf::from("TODO").join("infra-intent-template.yaml")
}

fn estimated_llm_cost_units(stage: IssueLifecycleStage, awaiting_approval: bool) -> u64 {
    if awaiting_approval {
        return 0;
    }
    match stage {
        IssueLifecycleStage::Idea => 0,
        IssueLifecycleStage::Design => 8,
        IssueLifecycleStage::DesignReview => 13,
        IssueLifecycleStage::Accepted => 2,
        IssueLifecycleStage::Implementation => 5,
        IssueLifecycleStage::Closed => 0,
    }
}

fn load_intake_ledger(path: &Path) -> Result<IntakeLedger, String> {
    if !path.exists() {
        return Ok(IntakeLedger::default());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read intake ledger {}: {error}", path.display()))?;
    serde_json::from_str::<IntakeLedger>(&content)
        .map_err(|error| format!("failed to parse intake ledger {}: {error}", path.display()))
}

fn save_intake_ledger(path: &Path, ledger: &IntakeLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create intake ledger directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|error| format!("failed to serialize intake ledger: {error}"))?;
    std::fs::write(path, content)
        .map_err(|error| format!("failed to write intake ledger {}: {error}", path.display()))
}

fn load_claim_ledger(path: &Path) -> Result<ClaimLedger, String> {
    if !path.exists() {
        return Ok(ClaimLedger::default());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read claim ledger {}: {error}", path.display()))?;
    serde_json::from_str::<ClaimLedger>(&content)
        .map_err(|error| format!("failed to parse claim ledger {}: {error}", path.display()))
}

fn save_claim_ledger(path: &Path, ledger: &ClaimLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create claim ledger directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|error| format!("failed to serialize claim ledger: {error}"))?;
    std::fs::write(path, content)
        .map_err(|error| format!("failed to write claim ledger {}: {error}", path.display()))
}

fn release_worker_owned_claims_for_drain(ledger: &mut ClaimLedger) -> Vec<String> {
    let mut released_intake_keys = Vec::new();
    let slots_to_remove: Vec<String> = ledger
        .claims
        .iter()
        .filter_map(|(slot, record)| {
            record
                .owner
                .strip_prefix("gunbc-sdlc-worker:")
                .map(|intake_key| {
                    released_intake_keys.push(intake_key.to_string());
                    slot.clone()
                })
        })
        .collect();
    for slot in slots_to_remove {
        ledger.claims.remove(&slot);
    }
    released_intake_keys.sort();
    released_intake_keys.dedup();
    released_intake_keys
}

fn load_artifact_ledger(path: &Path) -> Result<ArtifactLedger, String> {
    if !path.exists() {
        return Ok(ArtifactLedger::default());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read artifact ledger {}: {error}", path.display()))?;
    serde_json::from_str::<ArtifactLedger>(&content)
        .map_err(|error| format!("failed to parse artifact ledger {}: {error}", path.display()))
}

fn save_artifact_ledger(path: &Path, ledger: &ArtifactLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create artifact ledger directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|error| format!("failed to serialize artifact ledger: {error}"))?;
    std::fs::write(path, content)
        .map_err(|error| format!("failed to write artifact ledger {}: {error}", path.display()))
}

fn load_run_state_ledger(path: &Path) -> Result<RunStateLedger, String> {
    if !path.exists() {
        return Ok(RunStateLedger::default());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read run state ledger {}: {error}", path.display()))?;
    serde_json::from_str::<RunStateLedger>(&content)
        .map_err(|error| format!("failed to parse run state ledger {}: {error}", path.display()))
}

fn save_run_state_ledger(path: &Path, ledger: &RunStateLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create run state ledger directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|error| format!("failed to serialize run state ledger: {error}"))?;
    std::fs::write(path, content)
        .map_err(|error| format!("failed to write run state ledger {}: {error}", path.display()))
}

fn save_execution_report(report: &serde_json::Value) -> Result<(), String> {
    let path = execution_report_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create execution report directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(report)
        .map_err(|error| format!("failed to serialize execution report: {error}"))?;
    std::fs::write(&path, content)
        .map_err(|error| format!("failed to write execution report {}: {error}", path.display()))
}

const fn default_stage() -> IssueLifecycleStage {
    IssueLifecycleStage::Idea
}

fn epoch_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be >= unix epoch")
        .as_millis()
}

fn print_help() {
    println!("gunbc-sdlc - issue-centric SDLC intake/worker tool");
    println!();
    println!("USAGE:");
    println!("    gunbc-sdlc intake --intent <path> [--dry-run]");
    println!("    gunbc-sdlc worker [--dry-run] [--emit-pending-exit-code] [--infra-intent <path>]");
    println!(
        "    gunbc-sdlc issue --issue-id <value> [--dry-run] [--emit-pending-exit-code] [--infra-intent <path>]"
    );
    println!("    gunbc-sdlc await-approval --intake-key <value> [--dry-run]");
    println!("    gunbc-sdlc transition --intake-key <value> --stage <idea|design|design-review|accepted|implementation|closed> [--dry-run]");
    println!("    gunbc-sdlc drain [--activate|--deactivate] [--dry-run]");
    println!("    gunbc-sdlc help");
}
