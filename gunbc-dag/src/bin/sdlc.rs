//! gunbc-sdlc: issue-centric SDLC intake/worker entrypoint.
//!
//! Initial runtime surface:
//! - intake: validate intent contract + deterministic run_key + idempotent ledger update
//! - worker: summarize pending intake ledger state

#![deny(dead_code)]

use gunbc_dag::{
    claim_slot_key, reconcile_entries, register_retry_failure, release_claim, try_acquire_claim,
    ClaimAcquireResult, ClaimLedger, ReconcileAction, ReconcileEntry, RetryState,
};
use gunbc_ir::transport::github::{ensure_sdlc_issue_capabilities, SdlcIssueCapabilities};
use gunbc_ir::transport::github::IssueLifecycleStage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CLAIM_LEASE_TTL_MS: u128 = 30_000;
const RETRY_BASE_BACKOFF_MS: u128 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SdlcCommand {
    Intake,
    Worker,
    AwaitApproval,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    command: SdlcCommand,
    intent_path: Option<PathBuf>,
    intake_key: Option<String>,
    dry_run: bool,
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
    created_at_epoch_ms: u128,
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
        SdlcCommand::Worker => run_worker(args.dry_run),
        SdlcCommand::AwaitApproval => run_await_approval(args.intake_key.as_deref(), args.dry_run),
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
            intake_key: None,
            dry_run: false,
        });
    }

    let command = match argv[1].as_str() {
        "intake" => SdlcCommand::Intake,
        "worker" => SdlcCommand::Worker,
        "await-approval" => SdlcCommand::AwaitApproval,
        "help" | "--help" | "-h" => SdlcCommand::Help,
        other => return Err(format!("unknown command `{other}`")),
    };

    let mut intent_path: Option<PathBuf> = None;
    let mut intake_key: Option<String> = None;
    let mut dry_run = false;
    let mut idx = 2usize;
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
        return Err(format!("unknown flag `{token}`"));
    }

    if command == SdlcCommand::Intake && intent_path.is_none() {
        return Err("intake requires --intent <path>".to_string());
    }
    if command == SdlcCommand::Worker && intent_path.is_some() {
        return Err("worker does not accept --intent".to_string());
    }
    if command == SdlcCommand::AwaitApproval && intake_key.is_none() {
        return Err("await-approval requires --intake-key <value>".to_string());
    }
    if command == SdlcCommand::AwaitApproval && intent_path.is_some() {
        return Err("await-approval does not accept --intent".to_string());
    }

    Ok(CliArgs {
        command,
        intent_path,
        intake_key,
        dry_run,
    })
}

fn run_intake(intent_path: Option<&PathBuf>, dry_run: bool) -> Result<(), String> {
    let intent_path = intent_path.ok_or_else(|| "intake requires --intent <path>".to_string())?;
    let intent = load_intent(intent_path)?;
    validate_intent(&intent)?;

    let computed_run_key = compute_run_key(&intent);
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
            }))
            .map_err(|error| format!("failed to serialize intake dry-run output: {error}"))?
        );
        return Ok(());
    }

    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
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
                    created_at_epoch_ms: now,
                    updated_at_epoch_ms: now,
                },
            );
        }
    }

    save_intake_ledger(&ledger_path, &ledger)?;
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
        }))
        .map_err(|error| format!("failed to serialize intake output: {error}"))?
    );

    Ok(())
}

fn run_worker(dry_run: bool) -> Result<(), String> {
    let ledger_path = intake_ledger_path();
    let mut ledger = load_intake_ledger(&ledger_path)?;
    let claim_ledger_path = claim_ledger_path();
    let mut claim_ledger = load_claim_ledger(&claim_ledger_path)?;
    let mode = if dry_run { "dry-run" } else { "real" };
    let now = epoch_millis();

    let mut intake_keys: Vec<String> = ledger.entries.keys().cloned().collect();
    intake_keys.sort();
    let mut skipped_missing_issue = Vec::new();
    let mut skipped_terminalized = Vec::new();
    let mut claim_conflicts = Vec::new();
    let mut acquired_claims = Vec::new();
    let mut reconcile_inputs = Vec::new();
    let mut stage_duration_ms = BTreeMap::new();
    let mut approval_latency_ms = BTreeMap::new();
    let mut retry_attempts = BTreeMap::new();
    let mut claim_acquire_attempts: u64 = 0;

    for intake_key in &intake_keys {
        let Some(record) = ledger.entries.get_mut(intake_key) else {
            continue;
        };
        stage_duration_ms.insert(
            intake_key.clone(),
            now.saturating_sub(record.updated_at_epoch_ms),
        );
        retry_attempts.insert(intake_key.clone(), record.retry.attempts);
        if let Some(since) = record.awaiting_approval_since_epoch_ms {
            approval_latency_ms.insert(intake_key.clone(), now.saturating_sub(since));
        }
        if record.terminalized {
            skipped_terminalized.push(intake_key.clone());
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
                }
                claim_conflicts.push(intake_key.clone());
                continue;
            }
            ClaimAcquireResult::Acquired
            | ClaimAcquireResult::AlreadyOwned
            | ClaimAcquireResult::StaleReclaimed { .. } => {
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
            ReconcileAction::ReadyToRun { intake_key } => ready_to_run.push(intake_key.clone()),
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
            ReconcileAction::Terminalize { intake_key, .. } => {
                if let Some(record) = ledger.entries.get_mut(intake_key) {
                    record.terminalized = true;
                }
                terminalized.push(intake_key.clone());
            }
        }
    }

    if !dry_run {
        save_intake_ledger(&ledger_path, &ledger)?;
        save_claim_ledger(&claim_ledger_path, &claim_ledger)?;
    }

    let pending_count = intake_keys
        .len()
        .saturating_sub(skipped_terminalized.len())
        .saturating_sub(terminalized.len());
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "worker",
            "mode": mode,
            "pending_count": pending_count,
            "intake_keys": intake_keys,
            "ready_to_run": ready_to_run,
            "acquired_claims": acquired_claims,
            "released_claims": released_claims,
            "claim_conflicts": claim_conflicts,
            "terminalized": terminalized,
            "skipped_missing_issue": skipped_missing_issue,
            "skipped_terminalized": skipped_terminalized,
            "ledger_path": ledger_path.display().to_string(),
            "claim_ledger_path": claim_ledger_path.display().to_string(),
            "reconcile_actions": reconcile_plan.actions,
            "metrics": {
                "stage_duration_ms": stage_duration_ms,
                "approval_latency_ms": approval_latency_ms,
                "retry_attempts": retry_attempts,
                "cost_units": {
                    "claim_acquire_attempts": claim_acquire_attempts,
                    "reconcile_actions": reconcile_plan.actions.len(),
                }
            }
        }))
        .map_err(|error| format!("failed to serialize worker output: {error}"))?
    );
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

fn intake_ledger_path() -> PathBuf {
    PathBuf::from("target").join("sdlc").join("intake-ledger.json")
}

fn claim_ledger_path() -> PathBuf {
    PathBuf::from("target").join("sdlc").join("claim-ledger.json")
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
    println!("    gunbc-sdlc worker [--dry-run]");
    println!("    gunbc-sdlc await-approval --intake-key <value> [--dry-run]");
    println!("    gunbc-sdlc help");
}
