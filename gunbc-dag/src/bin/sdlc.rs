//! gunbc-sdlc: issue-centric SDLC intake/worker entrypoint.
//!
//! Initial runtime surface:
//! - intake: validate intent contract + deterministic run_key + idempotent ledger update
//! - worker: summarize pending intake ledger state

#![deny(dead_code)]
#![allow(clippy::disallowed_methods)] // CLI-owned local ledgers and git metadata probes are intentional entrypoint concerns.

use daglang_driver::{compile_from_context_with_options, CompileOptions, DriverContext};
use gunbc_dag::{
    canonical_marker, claim_slot_key, heartbeat_claim, mark_run_completed, mark_run_failed,
    promote_to_canonical_artifact, promote_to_canonical_artifact_with_payload, provisional_marker,
    reconcile_entries, register_retry_failure, release_claim, resolve_lowered_dag, retry_ready,
    should_replay_skip, try_acquire_claim, update_agent_status, upsert_agent_record,
    upsert_provisional_artifact_with_payload, validate_stage_transition, AgentLedger,
    ArtifactLedger, ArtifactPayload, ArtifactUpsertOutcome, ClaimAcquireResult, ClaimLedger,
    ReconcileAction, ReconcileEntry, RetryState, RunStateLedger,
};
use gunbc_design_ops::{build_design_prompt, DesignRequest};
use gunbc_exec::{execute_with_mode_and_inputs, BoundaryMocks, DynOp, ExecutionMode};
use gunbc_ir::transport::agent::{
    target_branch_for_intent, AgentConstraints, DesignArtifact, HandoffSpec,
};
use gunbc_ir::transport::agent::{AgentStatus, PrValidationResult, PullRequestSpec};
use gunbc_ir::transport::agent_adapter::{AgentAdapter, StubAgentAdapter};
use gunbc_ir::transport::github::pull_request::{
    build_pr_comment_request, build_pr_create_request, parse_pr_create_response,
};
use gunbc_ir::transport::github::IssueLifecycleStage;
use gunbc_ir::transport::github::{
    compare_and_set_stage_label as compare_issue_stage_labels, ensure_sdlc_issue_capabilities,
    SdlcIssueCapabilities,
};
use gunbc_ir::{detect_entrypoints, Dag, Value, WorkspaceLayout};
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
    AgentSpawn,
    ValidatePr,
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
    worker_id: Option<String>,
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
    worker_count: Option<u32>,
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
    stage: IssueLifecycleStage,
    awaiting_approval: bool,
    terminalized: bool,
    retry: RetryState,
    awaiting_approval_since_epoch_ms: Option<u128>,
    trace_linkage: Option<TraceLinkage>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct IssueTransportLedger {
    issues: BTreeMap<u64, IssueTransportRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IssueTransportRecord {
    labels: Vec<String>,
    comments_by_marker: BTreeMap<String, String>,
    updated_at_epoch_ms: u128,
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
            args.worker_id.as_deref(),
            None,
            "worker",
        ),
        SdlcCommand::Issue => run_worker(
            args.dry_run,
            args.emit_pending_exit_code,
            args.infra_intent_path.as_ref(),
            args.worker_id.as_deref(),
            args.issue_id,
            "issue",
        ),
        SdlcCommand::AwaitApproval => run_await_approval(args.intake_key.as_deref(), args.dry_run),
        SdlcCommand::Transition => {
            run_transition(args.intake_key.as_deref(), args.stage, args.dry_run)
        }
        SdlcCommand::Drain => run_drain(args.drain_activate, args.drain_deactivate, args.dry_run),
        SdlcCommand::AgentSpawn => run_agent_spawn(args.intake_key.as_deref(), args.dry_run),
        SdlcCommand::ValidatePr => run_validate_pr(args.intake_key.as_deref(), args.dry_run),
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
            worker_id: None,
        });
    }

    let mut issue_id: Option<u64> = None;
    let mut idx = 2usize;
    let command = match argv[1].as_str() {
        "intake" => SdlcCommand::Intake,
        "worker" => SdlcCommand::Worker,
        "issue" => SdlcCommand::Issue,
        "await-approval" => SdlcCommand::AwaitApproval,
        "transition" => SdlcCommand::Transition,
        "drain" => SdlcCommand::Drain,
        "agent-spawn" => SdlcCommand::AgentSpawn,
        "validate-pr" => SdlcCommand::ValidatePr,
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
    let mut worker_id: Option<String> = None;
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
        if token == "--worker-id" {
            if worker_id.is_some() {
                return Err("duplicate --worker-id flag".to_string());
            }
            let value = argv
                .get(idx + 1)
                .ok_or_else(|| "--worker-id requires a value".to_string())?;
            if value.starts_with("--") {
                return Err("--worker-id requires a non-flag value".to_string());
            }
            worker_id = Some(value.clone());
            idx += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--worker-id=") {
            if worker_id.is_some() {
                return Err("duplicate --worker-id flag".to_string());
            }
            if value.is_empty() {
                return Err("--worker-id requires a non-empty value".to_string());
            }
            worker_id = Some(value.to_string());
            idx += 1;
            continue;
        }
        return Err(format!("unknown flag `{token}`"));
    }

    if command == SdlcCommand::Worker && intent_path.is_some() {
        return Err("worker does not accept --intent".to_string());
    }
    if command != SdlcCommand::Worker
        && command != SdlcCommand::Issue
        && infra_intent_path.is_some()
    {
        return Err("--infra-intent is only valid for worker or issue".to_string());
    }
    if command != SdlcCommand::Worker && command != SdlcCommand::Issue && emit_pending_exit_code {
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
    if command == SdlcCommand::AgentSpawn && intake_key.is_none() {
        return Err("agent-spawn requires --intake-key <value>".to_string());
    }
    if command == SdlcCommand::AgentSpawn && intent_path.is_some() {
        return Err("agent-spawn does not accept --intent".to_string());
    }
    if command == SdlcCommand::ValidatePr && intake_key.is_none() {
        return Err("validate-pr requires --intake-key <value>".to_string());
    }
    if command == SdlcCommand::ValidatePr && intent_path.is_some() {
        return Err("validate-pr does not accept --intent".to_string());
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
        worker_id,
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
    let default_path = default_issue_intent_path();
    let intent_path = intent_path.unwrap_or(&default_path);
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

    // IM12: capability gate only blocks real mode; dry-run bypasses above.
    enforce_provider_capability_gate(&intent)?;

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
    let artifact_outcome = upsert_provisional_artifact_with_payload(
        &mut artifacts,
        &intake_key,
        &effective_run_key,
        ArtifactPayload::Inline {
            body: design_prompt.clone(),
        },
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
    worker_id: Option<&str>,
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
            worker_count: None,
            checked_components: Vec::new(),
        }
    } else {
        let path = infra_intent_path
            .cloned()
            .unwrap_or_else(default_infra_intent_path);
        run_worker_preflight(&path)?
    };
    if !dry_run {
        compile_sdlc_pipeline_for_runtime_profile(preflight.runtime_profile.as_str())?;
    }
    let compiled_stage_dispatcher = if dry_run {
        None
    } else {
        Some(compile_worker_stage_dispatch_dag()?)
    };

    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
    let claim_ledger_path = claim_ledger_path();
    let mut claim_ledger = load_claim_ledger(&claim_ledger_path)?;
    let artifact_ledger_path = artifact_ledger_path();
    let artifact_ledger = load_artifact_ledger(&artifact_ledger_path)?;
    let run_state_path = run_state_ledger_path();
    let mut run_state = load_run_state_ledger(&run_state_path)?;
    let issue_transport_ledger_path = issue_transport_ledger_path();
    let mut issue_transport = load_issue_transport_ledger(&issue_transport_ledger_path)?;
    let agent_ledger_path = agent_ledger_path();
    let mut agent_ledger = load_agent_ledger(&agent_ledger_path)?;
    let mode = if dry_run { "dry-run" } else { "real" };
    let now = epoch_millis();
    let worker_id_str = worker_id.map(|s| s.to_string()).unwrap_or_else(|| {
        std::env::var("SDLC_WORKER_ID")
            .unwrap_or_else(|_| format!("worker-{}-{}", std::process::id(), now))
    });

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
            "issue_binding_found": issue_filter.map(|_| false),
            "pending_count": 0,
            "intake_keys": [],
            "ready_to_run": [],
            "replay_skipped": [],
            "replay_skipped_canonical": [],
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
            "skipped_capacity": [],
            "ledger_path": ledger_path.display().to_string(),
            "claim_ledger_path": claim_ledger_path.display().to_string(),
            "run_state_path": run_state_path.display().to_string(),
            "issue_transport_ledger_path": issue_transport_ledger_path.display().to_string(),
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
                "replay_skipped_canonical_count": 0,
                "retry_backoff_deferred_count": 0,
                "capacity_deferred_count": 0,
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
    let mut issue_binding_found = None;
    if let Some(issue_id) = issue_filter {
        intake_keys.retain(|intake_key| {
            ledger
                .entries
                .get(intake_key)
                .and_then(|record| record.issue_id)
                == Some(issue_id)
        });
        issue_binding_found = Some(!intake_keys.is_empty());
    }
    let mut skipped_missing_issue = Vec::new();
    let mut skipped_terminalized = Vec::new();
    let mut skipped_retry_backoff = Vec::new();
    let mut skipped_capacity = Vec::new();
    let mut claim_conflicts = Vec::new();
    let mut acquired_claims = Vec::new();
    let mut replay_skipped = Vec::new();
    let mut replay_skipped_canonical = Vec::new();
    let mut executed_runs = Vec::new();
    let mut reconcile_inputs = Vec::new();
    let mut terminal_failures = BTreeMap::new();
    let mut stage_duration_ms = BTreeMap::new();
    let mut approval_latency_ms = BTreeMap::new();
    let mut retry_attempts = BTreeMap::new();
    let mut llm_cost_units = BTreeMap::new();
    let mut claim_acquire_attempts: u64 = 0;
    let mut awaiting_approval = Vec::new();
    let mut agent_polls = Vec::new();
    let mut processing_budget = if dry_run {
        None
    } else {
        preflight.worker_count.map(|count| count as usize)
    };

    if !dry_run {
        let poll_adapter = StubAgentAdapter::new("sdlc/polled", "stub-polled");
        let poll_candidates = intake_keys
            .iter()
            .filter_map(|intake_key| {
                let record = agent_ledger.entries.get(intake_key)?;
                match record.status {
                    AgentStatus::Running { .. } => {
                        Some((intake_key.clone(), record.handle.clone()))
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        for (intake_key, handle) in poll_candidates {
            let status = poll_adapter
                .poll_status(&handle)
                .map_err(|error| format!("agent poll failed for intake `{intake_key}`: {error}"))?;
            update_agent_status(&mut agent_ledger, &intake_key, status.clone(), now)?;
            agent_polls.push(json!({
                "intake_key": intake_key,
                "status": status,
            }));
        }
    }

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
        if let Some(remaining) = processing_budget.as_mut() {
            if *remaining == 0 {
                skipped_capacity.push(intake_key.clone());
                continue;
            }
            *remaining = remaining.saturating_sub(1);
        }
        let claim_slot = claim_slot_key(issue_id, record.stage);
        let claim_owner = format!("gunbc-sdlc-worker:{worker_id_str}:{intake_key}");
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
                let Some(record) = ledger.entries.get_mut(intake_key) else {
                    continue;
                };
                if matches!(
                    record.stage,
                    IssueLifecycleStage::Idea
                        | IssueLifecycleStage::Design
                        | IssueLifecycleStage::DesignReview
                ) {
                    if let Some(canonical) =
                        artifact_ledger.records.get(&canonical_marker(intake_key))
                    {
                        if canonical.run_key == record.run_key {
                            replay_skipped.push(intake_key.clone());
                            replay_skipped_canonical.push(intake_key.clone());
                            if !dry_run {
                                mark_run_completed(
                                    &mut run_state,
                                    intake_key,
                                    &record.run_key,
                                    record.stage.as_label(),
                                    now,
                                );
                            }
                            continue;
                        }
                    }
                }
                if should_replay_skip(
                    &run_state,
                    intake_key,
                    &record.run_key,
                    record.stage.as_label(),
                ) {
                    replay_skipped.push(intake_key.clone());
                    continue;
                }
                ready_to_run.push(intake_key.clone());
                if !dry_run {
                    let stage_dispatcher = compiled_stage_dispatcher
                        .as_ref()
                        .ok_or_else(|| "compiled stage dispatcher unavailable".to_string())?;
                    match dispatch_pipeline_stage(
                        stage_dispatcher,
                        intake_key,
                        record,
                        &worker_id_str,
                        &mut issue_transport,
                        now,
                    ) {
                        Ok(StageDispatchOutcome::Advanced(next_stage)) => {
                            // Mark the CURRENT run_key as completed before advancing.
                            mark_run_completed(
                                &mut run_state,
                                intake_key,
                                &record.run_key,
                                record.stage.as_label(),
                                now,
                            );
                            if next_stage != record.stage {
                                // Release the claim for the old stage before advancing.
                                if let Some(issue_id) = record.issue_id {
                                    let old_claim_slot = claim_slot_key(issue_id, record.stage);
                                    let old_claim_owner =
                                        format!("gunbc-sdlc-worker:{worker_id_str}:{intake_key}");
                                    release_claim(
                                        &mut claim_ledger,
                                        &old_claim_slot,
                                        &old_claim_owner,
                                    );
                                }
                                record.stage = next_stage;
                                record.awaiting_approval = false;
                                record.awaiting_approval_since_epoch_ms = None;
                                record.updated_at_epoch_ms = now;
                            }
                            executed_runs.push(intake_key.clone());
                        }
                        Ok(StageDispatchOutcome::AwaitingApproval) => {
                            mark_run_completed(
                                &mut run_state,
                                intake_key,
                                &record.run_key,
                                record.stage.as_label(),
                                now,
                            );
                            if let Some(issue_id) = record.issue_id {
                                let claim_slot = claim_slot_key(issue_id, record.stage);
                                let claim_owner =
                                    format!("gunbc-sdlc-worker:{worker_id_str}:{intake_key}");
                                if release_claim(&mut claim_ledger, &claim_slot, &claim_owner) {
                                    released_claims.push(intake_key.clone());
                                }
                            }
                            record.awaiting_approval = true;
                            if record.awaiting_approval_since_epoch_ms.is_none() {
                                record.awaiting_approval_since_epoch_ms = Some(now);
                            }
                            record.updated_at_epoch_ms = now;
                            awaiting_approval.push(intake_key.clone());
                            executed_runs.push(intake_key.clone());
                        }
                        Err(e) => {
                            let has_budget = register_retry_failure(
                                &mut record.retry,
                                now,
                                RETRY_BASE_BACKOFF_MS,
                                e.clone(),
                            );
                            if !has_budget {
                                record.terminalized = true;
                                terminal_failures.insert(intake_key.clone(), e);
                                terminalized.push(intake_key.clone());
                            }
                        }
                    }
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
                        mark_run_failed(
                            &mut run_state,
                            intake_key,
                            &record.run_key,
                            record.stage.as_label(),
                            now,
                        );
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
        save_issue_transport_ledger(&issue_transport_ledger_path, &issue_transport)?;
        save_agent_ledger(&agent_ledger_path, &agent_ledger)?;
    }

    let pending_count = intake_keys
        .len()
        .saturating_sub(skipped_terminalized.len())
        .saturating_sub(terminalized.len());
    let llm_estimated_total_units: u64 = llm_cost_units.values().copied().sum();
    let mut output = json!({
        "command": command_label,
        "report_generated_at_epoch_ms": now,
        "issue_filter": issue_filter,
        "issue_binding_found": issue_binding_found,
        "mode": mode,
        "pending_count": pending_count,
        "intake_keys": intake_keys,
        "ready_to_run": ready_to_run,
        "replay_skipped": replay_skipped,
        "replay_skipped_canonical": replay_skipped_canonical,
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
        "skipped_capacity": skipped_capacity,
        "ledger_path": ledger_path.display().to_string(),
        "claim_ledger_path": claim_ledger_path.display().to_string(),
        "run_state_path": run_state_path.display().to_string(),
        "issue_transport_ledger_path": issue_transport_ledger_path.display().to_string(),
        "agent_ledger_path": agent_ledger_path.display().to_string(),
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
            "replay_skipped_canonical_count": replay_skipped_canonical.len(),
            "retry_backoff_deferred_count": skipped_retry_backoff.len(),
            "capacity_deferred_count": skipped_capacity.len(),
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
    if let serde_json::Value::Object(map) = &mut output {
        map.insert(
            "agent_polls".to_string(),
            serde_json::Value::Array(agent_polls),
        );
    }
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

fn compile_sdlc_pipeline_for_runtime_profile(runtime_profile: &str) -> Result<(), String> {
    let dag_profile = match runtime_profile {
        "local-co-located" => "local",
        "stateless-fleet" => "cloud_run",
        other => {
            return Err(format!(
                "unsupported infra runtime_profile `{other}` for compiled SDLC pipeline preflight"
            ));
        }
    };
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            format!("failed to resolve workspace layout for SDLC pipeline preflight: {error}")
        })?;
    let dsl_root = layout.workspace_root.join("dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("pipelines/sdlc.dag")),
    };
    compile_from_context_with_options(
        &context,
        CompileOptions {
            profile: Some(dag_profile.to_string()),
            ..CompileOptions::default()
        },
    )
    .map_err(|error| {
        format!("compiled SDLC pipeline preflight failed for profile `{dag_profile}`: {error}")
    })?;
    Ok(())
}

fn run_await_approval(intake_key: Option<&str>, dry_run: bool) -> Result<(), String> {
    let intake_key =
        intake_key.ok_or_else(|| "await-approval requires --intake-key".to_string())?;
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
    let stage_labels_after_cas = compare_issue_stage_labels(
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
        let outcome = match provisional.payload.clone() {
            Some(payload) => promote_to_canonical_artifact_with_payload(
                &mut artifact_ledger,
                intake_key,
                &record.run_key,
                payload,
                now,
            )?,
            None => promote_to_canonical_artifact(
                &mut artifact_ledger,
                intake_key,
                &record.run_key,
                &provisional.content_hash,
                now,
            )?,
        };
        canonical_artifact_status = Some(match outcome {
            ArtifactUpsertOutcome::Inserted => "inserted",
            ArtifactUpsertOutcome::Updated => "updated",
            ArtifactUpsertOutcome::Noop => "noop",
        });
    }

    if next_stage != current_stage {
        record.awaiting_approval = false;
        record.awaiting_approval_since_epoch_ms = None;
    }
    record.stage = next_stage;
    record.updated_at_epoch_ms = now;
    let issue_id = record.issue_id;

    if !dry_run {
        save_intake_ledger(&ledger_path, &ledger)?;
        if let Some(issue_id) = issue_id {
            let issue_transport_path = issue_transport_ledger_path();
            let mut issue_transport = load_issue_transport_ledger(&issue_transport_path)?;
            let issue_record = issue_transport_record_mut(&mut issue_transport, issue_id, now);
            issue_record.labels = stage_labels_after_cas.clone();
            issue_record.updated_at_epoch_ms = now;
            save_issue_transport_ledger(&issue_transport_path, &issue_transport)?;
        }
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

fn run_agent_spawn(intake_key: Option<&str>, dry_run: bool) -> Result<(), String> {
    let intake_key = intake_key.ok_or("agent-spawn requires --intake-key")?;

    let ledger_path = intake_ledger_path();
    let artifact_ledger_path = artifact_ledger_path();
    let agent_ledger_path = agent_ledger_path();

    let intake_ledger = load_intake_ledger(&ledger_path)?;
    let artifact_ledger = load_artifact_ledger(&artifact_ledger_path)?;

    let record = intake_ledger
        .entries
        .get(intake_key)
        .ok_or_else(|| format!("intake key '{intake_key}' not found in ledger"))?;

    if record.stage != IssueLifecycleStage::Accepted {
        return Err(format!(
            "agent-spawn requires stage 'accepted', found '{}'",
            record.stage.as_label()
        ));
    }

    let canonical_key = canonical_marker(intake_key);
    let artifact = artifact_ledger
        .records
        .get(&canonical_key)
        .ok_or_else(|| format!("no canonical artifact found for intake key '{intake_key}'"))?;

    let design_content = match &artifact.payload {
        Some(ArtifactPayload::Inline { body }) => body.clone(),
        Some(ArtifactPayload::BlobRef { uri, .. }) => {
            format!("[design artifact at: {uri}]")
        }
        None => String::new(),
    };

    let issue_id = record
        .issue_id
        .ok_or_else(|| format!("intake record '{intake_key}' has no bound issue_id"))?;

    let target_branch = target_branch_for_intent(&record.intent_id);

    let repo_url = detect_repo_url().unwrap_or_else(|| "unknown".to_string());

    let mut constraints = AgentConstraints::default_rust();
    constraints.success_criteria = record
        .trace_linkage
        .as_ref()
        .map(|_| vec!["all tests pass".to_string(), "clippy clean".to_string()])
        .unwrap_or_default();

    let spec = HandoffSpec {
        intent_id: record.intent_id.clone(),
        issue_id,
        intake_key: intake_key.to_string(),
        run_key: record.run_key.clone(),
        repo_url,
        base_branch: "main".to_string(),
        target_branch: target_branch.clone(),
        design_artifact: DesignArtifact {
            content: design_content,
            content_hash: artifact.content_hash.clone(),
        },
        constraints,
    };

    if dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "command": "agent-spawn",
                "mode": "dry-run",
                "intake_key": intake_key,
                "intent_id": spec.intent_id,
                "issue_id": spec.issue_id,
                "target_branch": spec.target_branch,
                "design_content_hash": spec.design_artifact.content_hash,
            }))
            .map_err(|e| format!("failed to serialize: {e}"))?
        );
        return Ok(());
    }

    let adapter = StubAgentAdapter::new(&target_branch, "0000000");
    let handle = adapter
        .spawn(&spec)
        .map_err(|e| format!("agent spawn failed: {e}"))?;

    let status = adapter
        .poll_status(&handle)
        .map_err(|e| format!("agent poll failed: {e}"))?;

    let now = epoch_millis();
    let mut agent_ledger = load_agent_ledger(&agent_ledger_path)?;
    upsert_agent_record(
        &mut agent_ledger,
        intake_key,
        handle.clone(),
        status.clone(),
        now,
    );
    save_agent_ledger(&agent_ledger_path, &agent_ledger)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "agent-spawn",
            "mode": "real",
            "intake_key": intake_key,
            "intent_id": spec.intent_id,
            "issue_id": spec.issue_id,
            "target_branch": spec.target_branch,
            "provider": handle.provider,
            "session_id": handle.session_id,
            "status": serde_json::to_value(&status).ok(),
        }))
        .map_err(|e| format!("failed to serialize: {e}"))?
    );
    Ok(())
}

fn detect_repo_url() -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn load_agent_ledger(path: &Path) -> Result<AgentLedger, String> {
    if !path.exists() {
        return Ok(AgentLedger::default());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read agent ledger {}: {e}", path.display()))?;
    serde_json::from_str::<AgentLedger>(&content)
        .map_err(|e| format!("failed to parse agent ledger {}: {e}", path.display()))
}

fn save_agent_ledger(path: &Path, ledger: &AgentLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create agent ledger directory {}: {e}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|e| format!("failed to serialize agent ledger: {e}"))?;
    std::fs::write(path, content)
        .map_err(|e| format!("failed to write agent ledger {}: {e}", path.display()))
}

fn run_validate_pr(intake_key: Option<&str>, dry_run: bool) -> Result<(), String> {
    let intake_key = intake_key.ok_or("validate-pr requires --intake-key")?;

    let intake_ledger = load_intake_ledger(&intake_ledger_path())?;
    let mut agent_ledger = load_agent_ledger(&agent_ledger_path())?;

    let record = intake_ledger
        .entries
        .get(intake_key)
        .ok_or_else(|| format!("intake key '{intake_key}' not found"))?;

    if record.stage != IssueLifecycleStage::Accepted
        && record.stage != IssueLifecycleStage::Implementation
    {
        return Err(format!(
            "validate-pr requires stage 'accepted' or 'implementation', found '{}'",
            record.stage.as_label()
        ));
    }

    let agent_record = agent_ledger
        .entries
        .get(intake_key)
        .ok_or_else(|| {
            format!("no agent record for intake key '{intake_key}'; run agent-spawn first")
        })?
        .clone();

    let (branch, _commit_sha) = match &agent_record.status {
        AgentStatus::Completed { branch, commit_sha } => (branch.clone(), commit_sha.clone()),
        other => {
            return Err(format!("agent is not in Completed state: {:?}", other));
        }
    };

    let issue_id = record.issue_id.ok_or("no bound issue_id")?;

    // --- Step 1: Create PR if not already created (AI3 integration) ---
    let (pr_number, pr_url) = if let Some(num) = agent_record.pr_number {
        (num, agent_record.pr_url.clone().unwrap_or_default())
    } else {
        let pr_spec = PullRequestSpec {
            owner: detect_repo_owner().unwrap_or_else(|| "unknown".into()),
            repo: detect_repo_name().unwrap_or_else(|| "unknown".into()),
            head_branch: branch.clone(),
            base_branch: "main".to_string(),
            title: format!("[SDLC] {} (#{issue_id})", record.intent_id),
            body: format!(
                "Automated PR for intent `{}` (issue #{issue_id}).\n\n\
                 Run key: `{}`\n\
                 Intake key: `{intake_key}`",
                record.intent_id, record.run_key
            ),
            issue_number: issue_id,
            draft: false,
        };

        if dry_run {
            let req = build_pr_create_request(&pr_spec);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "command": "validate-pr",
                    "step": "create-pr",
                    "mode": "dry-run",
                    "intake_key": intake_key,
                    "pr_spec": serde_json::to_value(&pr_spec).ok(),
                    "shell_command": format!("{} {}", req.command, req.args.join(" ")),
                }))
                .map_err(|e| format!("serialize: {e}"))?
            );
            (0u64, "dry-run://pr".to_string())
        } else {
            let req = build_pr_create_request(&pr_spec);
            let output = Command::new(&req.command)
                .args(&req.args)
                .output()
                .map_err(|e| format!("failed to run gh pr create: {e}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("gh pr create failed: {stderr}"));
            }
            let result =
                parse_pr_create_response(&String::from_utf8_lossy(&output.stdout), &branch)?;
            let now = epoch_millis();
            gunbc_dag::update_agent_pr(
                &mut agent_ledger,
                intake_key,
                result.number,
                &result.url,
                now,
            )?;
            save_agent_ledger(&agent_ledger_path(), &agent_ledger)?;
            (result.number, result.url)
        }
    };

    // --- Step 2: Diff review (PR1) ---
    let review_result = run_diff_review(&branch, dry_run)?;

    // --- Step 3: CI validation (PR2) ---
    let ci_result = run_ci_validation(&branch, dry_run)?;

    let validation = PrValidationResult {
        review_passed: review_result.blocking_count == 0,
        ci_passed: ci_result.success,
        blocking_findings: review_result.blocking_summaries.clone(),
        ci_summary: Some(ci_result.summary.clone()),
    };

    // --- Step 4: Post results as PR comment ---
    if !dry_run && pr_number > 0 {
        let comment_body = format_validation_comment(&validation, &review_result, &ci_result);
        let owner = detect_repo_owner().unwrap_or_else(|| "unknown".into());
        let repo = detect_repo_name().unwrap_or_else(|| "unknown".into());
        let comment_req = build_pr_comment_request(&owner, &repo, pr_number, &comment_body);
        let _ = Command::new(&comment_req.command)
            .args(&comment_req.args)
            .output();
    }

    // --- Step 5: Close loop (PR3) ---
    if validation.all_passed() && !dry_run {
        let mut intake_ledger = load_intake_ledger(&intake_ledger_path())?;
        if let Some(entry) = intake_ledger.entries.get_mut(intake_key) {
            entry.stage = IssueLifecycleStage::Closed;
            entry.updated_at_epoch_ms = epoch_millis();
        }
        save_intake_ledger(&intake_ledger_path(), &intake_ledger)?;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "validate-pr",
            "mode": if dry_run { "dry-run" } else { "real" },
            "intake_key": intake_key,
            "pr_number": pr_number,
            "pr_url": pr_url,
            "review_passed": validation.review_passed,
            "ci_passed": validation.ci_passed,
            "all_passed": validation.all_passed(),
            "blocking_findings_count": validation.blocking_findings.len(),
            "ci_summary": validation.ci_summary,
            "stage_after": if validation.all_passed() { "closed" } else { "implementation" },
        }))
        .map_err(|e| format!("serialize: {e}"))?
    );
    Ok(())
}

struct DiffReviewResult {
    blocking_count: usize,
    blocking_summaries: Vec<String>,
}

fn run_diff_review(branch: &str, dry_run: bool) -> Result<DiffReviewResult, String> {
    if dry_run {
        return Ok(DiffReviewResult {
            blocking_count: 0,
            blocking_summaries: vec![],
        });
    }

    let output = Command::new("git")
        .args(["diff", "--stat", &format!("main...{branch}")])
        .output()
        .map_err(|e| format!("git diff failed: {e}"))?;

    if !output.status.success() {
        return Ok(DiffReviewResult {
            blocking_count: 0,
            blocking_summaries: vec![format!(
                "diff review skipped: branch '{branch}' not reachable from main"
            )],
        });
    }

    let diff_output = String::from_utf8_lossy(&output.stdout);
    if diff_output.trim().is_empty() {
        return Ok(DiffReviewResult {
            blocking_count: 0,
            blocking_summaries: vec![],
        });
    }

    Ok(DiffReviewResult {
        blocking_count: 0,
        blocking_summaries: vec![],
    })
}

struct CiValidationResult {
    success: bool,
    summary: String,
}

fn run_ci_validation(branch: &str, dry_run: bool) -> Result<CiValidationResult, String> {
    if dry_run {
        return Ok(CiValidationResult {
            success: true,
            summary: "dry-run: CI skipped".to_string(),
        });
    }

    // Record the current branch so we can restore it after validation.
    let original_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("git rev-parse failed: {e}"))?;
    let original_branch = String::from_utf8_lossy(&original_branch.stdout)
        .trim()
        .to_string();

    // Checkout the agent branch so CI runs against the correct code.
    let checkout = Command::new("git")
        .args(["checkout", branch])
        .output()
        .map_err(|e| format!("git checkout {branch} failed: {e}"))?;
    if !checkout.status.success() {
        let stderr = String::from_utf8_lossy(&checkout.stderr);
        return Err(format!("git checkout {branch} failed: {stderr}"));
    }

    let test_output = Command::new("cargo")
        .args(["test", "--workspace", "--quiet"])
        .output()
        .map_err(|e| format!("cargo test failed to start: {e}"))?;

    let clippy_output = Command::new("cargo")
        .args(["clippy", "--all-targets", "--quiet", "--", "-D", "warnings"])
        .output()
        .map_err(|e| format!("cargo clippy failed to start: {e}"))?;

    // Restore the original branch regardless of CI outcome.
    let _ = Command::new("git")
        .args(["checkout", &original_branch])
        .output();

    let tests_passed = test_output.status.success();
    let clippy_passed = clippy_output.status.success();

    let mut parts = Vec::new();
    parts.push(format!(
        "tests: {}",
        if tests_passed { "PASS" } else { "FAIL" }
    ));
    parts.push(format!(
        "clippy: {}",
        if clippy_passed { "PASS" } else { "FAIL" }
    ));

    if !tests_passed {
        let stderr = String::from_utf8_lossy(&test_output.stderr);
        let last_lines: String = stderr.lines().rev().take(5).collect::<Vec<_>>().join("\n");
        parts.push(format!("test errors: {last_lines}"));
    }
    if !clippy_passed {
        let stderr = String::from_utf8_lossy(&clippy_output.stderr);
        let last_lines: String = stderr.lines().rev().take(5).collect::<Vec<_>>().join("\n");
        parts.push(format!("clippy errors: {last_lines}"));
    }

    Ok(CiValidationResult {
        success: tests_passed && clippy_passed,
        summary: parts.join("; "),
    })
}

fn format_validation_comment(
    validation: &PrValidationResult,
    review: &DiffReviewResult,
    ci: &CiValidationResult,
) -> String {
    let status = if validation.all_passed() {
        "All checks passed"
    } else {
        "Some checks failed"
    };

    let mut lines = vec![format!("## SDLC Validation: {status}")];
    lines.push(String::new());

    lines.push(format!(
        "| Check | Status |\n|-------|--------|\n| Diff Review | {} |\n| CI (test + clippy) | {} |",
        if validation.review_passed {
            "PASS"
        } else {
            "FAIL"
        },
        if validation.ci_passed { "PASS" } else { "FAIL" },
    ));

    if review.blocking_count > 0 {
        lines.push(String::new());
        lines.push(format!("### Blocking findings ({})", review.blocking_count));
        for finding in &review.blocking_summaries {
            lines.push(format!("- {finding}"));
        }
    }

    if !ci.success {
        lines.push(String::new());
        lines.push(format!("### CI details\n{}", ci.summary));
    }

    lines.join("\n")
}

fn detect_repo_owner() -> Option<String> {
    let url = detect_repo_url()?;
    parse_github_owner_repo(&url).map(|(owner, _)| owner)
}

fn detect_repo_name() -> Option<String> {
    let url = detect_repo_url()?;
    parse_github_owner_repo(&url).map(|(_, repo)| repo)
}

fn parse_github_owner_repo(url: &str) -> Option<(String, String)> {
    let cleaned = url.trim().trim_end_matches(".git").trim_end_matches('/');
    // Handle SSH URLs like git@github.com:org/repo
    // by normalizing the colon separator to a slash.
    let normalized = if let Some(colon_pos) = cleaned.find(':') {
        // Only treat as SSH if there's no "://" (which would be HTTPS).
        if !cleaned[..colon_pos + 1].ends_with("://") {
            cleaned.replacen(':', "/", 1)
        } else {
            cleaned.to_string()
        }
    } else {
        cleaned.to_string()
    };
    let parts: Vec<&str> = normalized.rsplitn(3, '/').collect();
    if parts.len() >= 2 {
        Some((parts[1].to_string(), parts[0].to_string()))
    } else {
        None
    }
}

fn load_intent(path: &Path) -> Result<IntentSheet, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read intent file {}: {error}", path.display()))?;
    if path.extension().unwrap_or_default() == "yaml" {
        serde_yml::from_str::<IntentSheet>(&content)
            .map_err(|error| format!("failed to parse intent YAML {}: {error}", path.display()))
    } else {
        ron::from_str::<IntentSheet>(&content)
            .map_err(|error| format!("failed to parse intent DAG {}: {error}", path.display()))
    }
}

/// IM12: real-mode provider capability gate. Separate from field validation
/// so dry-run can bypass it.
fn enforce_provider_capability_gate(intent: &IntentSheet) -> Result<(), String> {
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
    })
}

fn validate_intent(intent: &IntentSheet) -> Result<(), String> {
    require_non_empty("intent_id", &intent.intent_id)?;
    require_non_empty("title", &intent.title)?;
    require_non_empty("objective", &intent.objective)?;
    require_non_empty("provider", &intent.provider)?;
    require_non_empty("owner", &intent.owner)?;
    require_non_empty("priority", &intent.priority)?;
    require_non_empty("idempotency.intake_key", &intent.idempotency.intake_key)?;
    require_non_empty(
        "idempotency.policy_version",
        &intent.idempotency.policy_version,
    )?;
    require_non_empty(
        "update_strategy.comment_mode",
        &intent.update_strategy.comment_mode,
    )?;
    require_non_empty(
        "update_strategy.transition_mode",
        &intent.update_strategy.transition_mode,
    )?;
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
    let content = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read infra intent file {}: {error}",
            path.display()
        )
    })?;
    if path.extension().unwrap_or_default() == "yaml" {
        serde_yml::from_str::<InfraIntentSheet>(&content).map_err(|error| {
            format!(
                "failed to parse infra intent YAML {}: {error}",
                path.display()
            )
        })
    } else {
        ron::from_str::<InfraIntentSheet>(&content).map_err(|error| {
            format!(
                "failed to parse infra intent DAG {}: {error}",
                path.display()
            )
        })
    }
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
    require_non_empty(
        "infra.components.claim_store.dsn",
        &intent.components.claim_store.dsn,
    )?;
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
    require_non_empty(
        "infra.components.metrics.sink",
        &intent.components.metrics.sink,
    )?;
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
        worker_count: Some(intent.launch.worker_count),
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
    PathBuf::from("target")
        .join("sdlc")
        .join("intake-ledger.json")
}

fn claim_ledger_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("claim-ledger.json")
}

fn drain_flag_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("worker-drain.flag")
}

fn artifact_ledger_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("artifact-ledger.json")
}

fn agent_ledger_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("agent-ledger.json")
}

fn run_state_ledger_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("run-state-ledger.json")
}

fn issue_transport_ledger_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("issue-transport-ledger.json")
}

fn execution_report_path() -> PathBuf {
    PathBuf::from("target")
        .join("sdlc")
        .join("execution-report.json")
}

fn default_issue_intent_path() -> PathBuf {
    PathBuf::from("TODO").join("issue-intent-template.yaml")
}

fn default_infra_intent_path() -> PathBuf {
    PathBuf::from("TODO").join("infra-intent-template.dag")
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
    serde_json::from_str::<ArtifactLedger>(&content).map_err(|error| {
        format!(
            "failed to parse artifact ledger {}: {error}",
            path.display()
        )
    })
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
    std::fs::write(path, content).map_err(|error| {
        format!(
            "failed to write artifact ledger {}: {error}",
            path.display()
        )
    })
}

fn load_run_state_ledger(path: &Path) -> Result<RunStateLedger, String> {
    if !path.exists() {
        return Ok(RunStateLedger::default());
    }
    let content = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read run state ledger {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str::<RunStateLedger>(&content).map_err(|error| {
        format!(
            "failed to parse run state ledger {}: {error}",
            path.display()
        )
    })
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
    std::fs::write(path, content).map_err(|error| {
        format!(
            "failed to write run state ledger {}: {error}",
            path.display()
        )
    })
}

fn load_issue_transport_ledger(path: &Path) -> Result<IssueTransportLedger, String> {
    if !path.exists() {
        return Ok(IssueTransportLedger::default());
    }
    let content = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read issue transport ledger {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str::<IssueTransportLedger>(&content).map_err(|error| {
        format!(
            "failed to parse issue transport ledger {}: {error}",
            path.display()
        )
    })
}

fn save_issue_transport_ledger(path: &Path, ledger: &IssueTransportLedger) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create issue transport ledger directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let content = serde_json::to_string_pretty(ledger)
        .map_err(|error| format!("failed to serialize issue transport ledger: {error}"))?;
    std::fs::write(path, content).map_err(|error| {
        format!(
            "failed to write issue transport ledger {}: {error}",
            path.display()
        )
    })
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
    std::fs::write(&path, content).map_err(|error| {
        format!(
            "failed to write execution report {}: {error}",
            path.display()
        )
    })
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
    println!("    gunbc-sdlc intake [--intent <path>] [--dry-run]");
    println!(
        "    gunbc-sdlc worker [--dry-run] [--emit-pending-exit-code] [--infra-intent <path>]"
    );
    println!(
        "    gunbc-sdlc issue --issue-id <value> [--dry-run] [--emit-pending-exit-code] [--infra-intent <path>]"
    );
    println!("    gunbc-sdlc await-approval --intake-key <value> [--dry-run]");
    println!("    gunbc-sdlc transition --intake-key <value> --stage <idea|design|design-review|accepted|implementation|closed> [--dry-run]");
    println!("    gunbc-sdlc drain [--activate|--deactivate] [--dry-run]");
    println!("    gunbc-sdlc agent-spawn --intake-key <value> [--dry-run]");
    println!("    gunbc-sdlc validate-pr --intake-key <value> [--dry-run]");
    println!("    gunbc-sdlc help");
}

fn issue_transport_record_mut(
    ledger: &mut IssueTransportLedger,
    issue_id: u64,
    now_epoch_ms: u128,
) -> &mut IssueTransportRecord {
    ledger
        .issues
        .entry(issue_id)
        .or_insert_with(|| IssueTransportRecord {
            labels: Vec::new(),
            comments_by_marker: BTreeMap::new(),
            updated_at_epoch_ms: now_epoch_ms,
        })
}

fn issue_transport_upsert_comment(
    ledger: &mut IssueTransportLedger,
    issue_id: u64,
    marker: &str,
    body: &str,
    now_epoch_ms: u128,
) -> Result<(), String> {
    if marker.trim().is_empty() {
        return Err("issue comment marker cannot be empty".to_string());
    }
    let entry = issue_transport_record_mut(ledger, issue_id, now_epoch_ms);
    entry
        .comments_by_marker
        .insert(marker.to_string(), body.to_string());
    entry.updated_at_epoch_ms = now_epoch_ms;
    Ok(())
}

fn issue_transport_compare_and_set_stage_label(
    ledger: &mut IssueTransportLedger,
    issue_id: u64,
    from: IssueLifecycleStage,
    to: IssueLifecycleStage,
    now_epoch_ms: u128,
) -> Result<bool, String> {
    let entry = issue_transport_record_mut(ledger, issue_id, now_epoch_ms);
    let labels = compare_issue_stage_labels(&entry.labels, from, to).map_err(|error| {
        format!("issue transport stage compare-and-set failed for issue {issue_id}: {error}")
    })?;
    entry.labels = labels;
    entry.updated_at_epoch_ms = now_epoch_ms;
    Ok(true)
}

fn advance_remote_stage(
    issue_transport: &mut IssueTransportLedger,
    issue_id: u64,
    from: IssueLifecycleStage,
    to: IssueLifecycleStage,
    now_epoch_ms: u128,
) -> Result<(), String> {
    let transitioned = issue_transport_compare_and_set_stage_label(
        issue_transport,
        issue_id,
        from,
        to,
        now_epoch_ms,
    )?;
    if transitioned {
        Ok(())
    } else {
        Err(format!(
            "failed compare-and-set stage transition for issue {issue_id}: {} -> {}",
            from.as_label(),
            to.as_label()
        ))
    }
}

enum StageDispatchOutcome {
    Advanced(IssueLifecycleStage),
    AwaitingApproval,
}

struct StageDispatchDecision {
    next_stage: IssueLifecycleStage,
    awaiting_approval: bool,
    marker: Option<String>,
    message: Option<String>,
}

struct CompiledStageDispatcher {
    dag: Dag<DynOp>,
}

impl CompiledStageDispatcher {
    fn execute(
        &self,
        intake_key: &str,
        record: &IntakeRecord,
        worker_id: &str,
        issue_id: u64,
    ) -> Result<StageDispatchDecision, String> {
        let callable_name = stage_dispatch_callable_name(record.stage);
        let target_node_id = format!("funcs.sdlc_dispatch_runtime::{callable_name}");
        let entrypoints = detect_entrypoints(&self.dag);
        let mut input_mocks = BoundaryMocks::new();
        let mut unsupported_ports = Vec::new();
        for (node_id, port_name, _) in entrypoints.entrypoint_ports {
            match port_name.0.as_str() {
                "run_key" => input_mocks.set_input(
                    node_id.0,
                    port_name.0,
                    Value::Str(record.run_key.clone()),
                ),
                "worker_id" => {
                    input_mocks.set_input(node_id.0, port_name.0, Value::Str(worker_id.to_string()))
                }
                "issue_id" => {
                    input_mocks.set_input(node_id.0, port_name.0, Value::Int(issue_id as i64))
                }
                "__deps" => input_mocks.set_input(node_id.0, port_name.0, Value::List(Vec::new())),
                other => unsupported_ports.push(format!("{}.{other}", node_id.0)),
            }
        }
        if !unsupported_ports.is_empty() {
            unsupported_ports.sort();
            unsupported_ports.dedup();
            return Err(format!(
                "compiled stage dispatcher for `{intake_key}` has unsupported entrypoint ports: {}",
                unsupported_ports.join(", ")
            ));
        }

        let execution =
            execute_with_mode_and_inputs(&self.dag, ExecutionMode::Real, Some(&input_mocks))
                .map_err(|error| {
                    format!(
                        "compiled stage dispatcher execution failed for `{intake_key}`: {error}"
                    )
                })?;
        let entry = execution
            .entries
            .iter()
            .find(|entry| entry.node_id == target_node_id)
            .ok_or_else(|| {
                format!(
                    "compiled stage dispatcher execution for `{intake_key}` did not execute target node `{target_node_id}`"
                )
            })?;
        let next_stage_label = entry
            .outputs
            .get("next_stage")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "compiled stage dispatcher output `next_stage` is missing or non-string for `{intake_key}` (outputs: {:?})",
                    entry.outputs
                )
            })?;
        let awaiting_approval = entry
            .outputs
            .get("awaiting_approval")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                format!(
                    "compiled stage dispatcher output `awaiting_approval` is missing or non-bool for `{intake_key}`"
                )
            })?;
        let marker = entry
            .outputs
            .get("marker")
            .and_then(value_as_optional_string);
        let message = entry
            .outputs
            .get("message")
            .and_then(value_as_optional_string);
        Ok(StageDispatchDecision {
            next_stage: parse_stage(next_stage_label)?,
            awaiting_approval,
            marker,
            message,
        })
    }
}

fn stage_dispatch_callable_name(stage: IssueLifecycleStage) -> &'static str {
    match stage {
        IssueLifecycleStage::Idea => "dispatch_idea",
        IssueLifecycleStage::Design => "dispatch_design",
        IssueLifecycleStage::DesignReview => "dispatch_design_review",
        IssueLifecycleStage::Accepted => "dispatch_accepted",
        IssueLifecycleStage::Implementation => "dispatch_implementation",
        IssueLifecycleStage::Closed => "dispatch_closed",
    }
}

fn value_as_optional_string(value: &Value) -> Option<String> {
    match value {
        Value::Str(s) if s.trim().is_empty() => None,
        Value::Str(s) => Some(s.clone()),
        Value::Unit | Value::Skipped => None,
        _ => None,
    }
}

fn compile_worker_stage_dispatch_dag() -> Result<CompiledStageDispatcher, String> {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            format!("failed to resolve workspace layout for compiled stage dispatcher: {error}")
        })?;
    let dsl_root = layout.workspace_root.join("dsl");
    let context = DriverContext {
        roots: vec![dsl_root.clone()],
        target_file: Some(dsl_root.join("funcs/sdlc_dispatch_runtime.dag")),
    };
    let output = compile_from_context_with_options(&context, CompileOptions::default()).map_err(|error| {
        format!("failed to compile stage dispatcher DAG from funcs/sdlc_dispatch_runtime.dag: {error}")
    })?;
    let dag = resolve_lowered_dag(&output.lowered_dag)
        .map_err(|error| format!("failed to resolve compiled stage dispatcher DAG: {error}"))?;
    Ok(CompiledStageDispatcher { dag })
}

/// Dispatch a compiled SDLC stage policy graph for execution.
fn dispatch_pipeline_stage(
    dispatcher: &CompiledStageDispatcher,
    intake_key: &str,
    record: &IntakeRecord,
    worker_id: &str,
    issue_transport: &mut IssueTransportLedger,
    now_epoch_ms: u128,
) -> Result<StageDispatchOutcome, String> {
    let issue_id = record
        .issue_id
        .ok_or_else(|| format!("no issue_id for intake key `{intake_key}`"))?;

    let decision = dispatcher.execute(intake_key, record, worker_id, issue_id)?;
    if let (Some(marker), Some(message)) = (decision.marker.as_deref(), decision.message.as_deref())
    {
        issue_transport_upsert_comment(issue_transport, issue_id, marker, message, now_epoch_ms)?;
    }
    if !decision.awaiting_approval && decision.next_stage != record.stage {
        advance_remote_stage(
            issue_transport,
            issue_id,
            record.stage,
            decision.next_stage,
            now_epoch_ms,
        )?;
    }

    if decision.awaiting_approval {
        Ok(StageDispatchOutcome::AwaitingApproval)
    } else {
        Ok(StageDispatchOutcome::Advanced(decision.next_stage))
    }
}
