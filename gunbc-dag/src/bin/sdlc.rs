//! gunbc-sdlc: issue-centric SDLC intake/worker entrypoint.
//!
//! Initial runtime surface:
//! - intake: validate intent contract + deterministic run_key + idempotent ledger update
//! - worker: summarize pending intake ledger state

#![deny(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SdlcCommand {
    Intake,
    Worker,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    command: SdlcCommand,
    intent_path: Option<PathBuf>,
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
            dry_run: false,
        });
    }

    let command = match argv[1].as_str() {
        "intake" => SdlcCommand::Intake,
        "worker" => SdlcCommand::Worker,
        "help" | "--help" | "-h" => SdlcCommand::Help,
        other => return Err(format!("unknown command `{other}`")),
    };

    let mut intent_path: Option<PathBuf> = None;
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
        return Err(format!("unknown flag `{token}`"));
    }

    if command == SdlcCommand::Intake && intent_path.is_none() {
        return Err("intake requires --intent <path>".to_string());
    }
    if command == SdlcCommand::Worker && intent_path.is_some() {
        return Err("worker does not accept --intent".to_string());
    }

    Ok(CliArgs {
        command,
        intent_path,
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
    let ledger = load_intake_ledger(&ledger_path)?;
    let mode = if dry_run { "dry-run" } else { "real" };
    let mut intake_keys: Vec<String> = ledger.entries.keys().cloned().collect();
    intake_keys.sort();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "command": "worker",
            "mode": mode,
            "pending_count": intake_keys.len(),
            "intake_keys": intake_keys,
            "ledger_path": ledger_path.display().to_string(),
        }))
        .map_err(|error| format!("failed to serialize worker output: {error}"))?
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

    if intent.provider != "github" {
        return Err(format!(
            "unsupported provider `{}`; only `github` is supported",
            intent.provider
        ));
    }
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
    println!("    gunbc-sdlc help");
}
