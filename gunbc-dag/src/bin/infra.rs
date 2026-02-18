//! gunbc-infra unified entrypoint for infra workflows.
//!
//! Commands:
//! - `spec`: print infra spec JSON
//! - `graph`: print infra graph DOT
//! - `plan`: run infra planning DAG
//! - `apply`: preview or execute infra apply DAG
//! - `bootstrap`: preview or execute WIF bootstrap DAG

#![deny(dead_code)]

use gunbc_exec::{
    execute, execute_with_mode_and_inputs, print_attention, AttentionLevel, BoundaryMocks,
    ExecutionMode,
};
use gunbc_ir::transport::cloud::CloudRuntimeKind;
use gunbc_ir::{detect_entrypoints, Dag, Value};
use gunbc_lib_cloud_ops::project_spec::{
    RotationHandler, SecretRequirement, SecretStatus, GUNBAI_SECRETS,
};
use gunbc_lib_cloud_ops::{
    build_infra_apply_dag, build_infra_plan_dag, build_wif_bootstrap_dag, render_infra_spec_dot,
    evaluate_health, inspect_login_flow, InfraApplyFilter, InfraSpec, CI_SPEC, DEV_SPEC,
    PROD_SPEC, TEST_SPEC,
};
use serde_json::json;
use std::collections::HashMap;
use std::process;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InfraCommand {
    Bootstrap,
    Plan,
    Apply,
    Spec,
    Graph,
    Login,
    Status,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InfraCliArgs {
    command: InfraCommand,
    environment: String,
    runtime: CloudRuntimeKind,
    target: Vec<String>,
    skip: Vec<String>,
    inputs: HashMap<String, String>,
    execute: bool,
}

impl InfraCliArgs {
    fn for_command(command: InfraCommand) -> Self {
        Self {
            command,
            environment: "dev".to_string(),
            runtime: CloudRuntimeKind::LocalDev,
            target: Vec::new(),
            skip: Vec::new(),
            inputs: HashMap::new(),
            execute: false,
        }
    }

    fn filter(&self) -> InfraApplyFilter {
        InfraApplyFilter {
            target: self.target.clone(),
            skip: self.skip.clone(),
        }
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let args = match parse_cli_args(&argv) {
        Ok(args) => args,
        Err(err) => {
            print_attention(AttentionLevel::Error, "invalid infra CLI arguments", &err);
            print_help();
            process::exit(1);
        }
    };

    if args.command == InfraCommand::Help {
        print_help();
        return;
    }

    if let Err(err) = run_command(args) {
        print_attention(AttentionLevel::Error, "infra command failed", &err);
        process::exit(1);
    }
}

fn run_command(args: InfraCliArgs) -> Result<(), String> {
    let spec = spec_for_env(&args.environment)?;

    match args.command {
        InfraCommand::Spec => {
            let rendered = serde_json::to_string_pretty(&infra_spec_json(spec))
                .map_err(|e| format!("failed to serialize infra spec JSON: {e}"))?;
            println!("{rendered}");
            Ok(())
        }
        InfraCommand::Graph => {
            println!("{}", render_infra_spec_dot(spec));
            Ok(())
        }
        InfraCommand::Plan => run_plan(spec, args.runtime, &args.filter()),
        InfraCommand::Apply => run_apply(spec, &args),
        InfraCommand::Bootstrap => run_bootstrap(spec, &args),
        InfraCommand::Login => run_login(spec),
        InfraCommand::Status => run_status(spec),
        InfraCommand::Help => Ok(()),
    }
}

fn run_plan(spec: &InfraSpec, runtime: CloudRuntimeKind, filter: &InfraApplyFilter) -> Result<(), String> {
    let dag = build_infra_plan_dag(&GUNBAI_SECRETS, spec, runtime, filter)?;
    let log = execute(&dag).map_err(|e| format!("plan execution failed: {e}"))?;
    let plan = log
        .get("plan")
        .ok_or_else(|| "plan log entry missing from execution output".to_string())?;

    let planned_targets = plan
        .outputs
        .get("planned_targets")
        .and_then(Value::as_str_list)
        .unwrap_or_default();
    let target_count = plan
        .outputs
        .get("target_count")
        .and_then(Value::as_int)
        .unwrap_or(planned_targets.len() as i64);

    println!(
        "infra plan (env={}, runtime={}): {} target(s)",
        spec.environment,
        runtime.as_str(),
        target_count
    );
    for target in planned_targets {
        println!(" - {target}");
    }
    Ok(())
}

fn run_apply(spec: &InfraSpec, args: &InfraCliArgs) -> Result<(), String> {
    if !args.execute {
        println!("infra apply preview (no changes). pass --execute to run apply.");
        return run_plan(spec, args.runtime, &args.filter());
    }

    let dag = build_infra_apply_dag(&GUNBAI_SECRETS, spec, args.runtime, &args.filter())?;
    let input_mocks = build_entrypoint_input_mocks(&dag, &args.inputs, false)?;
    let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks))
        .map_err(|e| format!("apply execution failed: {e}"))?;

    let summary = log
        .get("apply_summary")
        .and_then(|entry| entry.outputs.get("report"))
        .and_then(Value::as_str)
        .unwrap_or("infra apply completed");
    println!("{summary}");
    Ok(())
}

fn run_bootstrap(spec: &InfraSpec, args: &InfraCliArgs) -> Result<(), String> {
    let dag = build_wif_bootstrap_dag(spec)?;

    if !args.execute {
        let entrypoints = detect_entrypoints(&dag);
        let mut ports: Vec<String> = entrypoints
            .entrypoint_ports
            .iter()
            .map(|(_, port_name, _)| port_name.0.clone())
            .collect();
        ports.sort();
        ports.dedup();
        println!(
            "infra bootstrap preview (env={}): {} node(s)",
            spec.environment,
            dag.nodes.len()
        );
        if ports.is_empty() {
            println!("required inputs: none");
        } else {
            println!("required inputs: {}", ports.join(", "));
        }
        println!("pass --execute to run bootstrap");
        return Ok(());
    }

    let input_mocks = build_entrypoint_input_mocks(&dag, &args.inputs, true)?;
    let log = execute_with_mode_and_inputs(&dag, ExecutionMode::Real, Some(&input_mocks))
        .map_err(|e| format!("bootstrap execution failed: {e}"))?;

    let report = log
        .get("bootstrap_summary")
        .and_then(|entry| entry.outputs.get("report"))
        .and_then(Value::as_str)
        .unwrap_or("infra bootstrap completed");
    println!("{report}");
    Ok(())
}

fn run_login(spec: &InfraSpec) -> Result<(), String> {
    let diagnostics = inspect_login_flow(spec);

    println!("infra login");
    println!("  environment: {}", diagnostics.environment);
    println!("  adc_path: {}", diagnostics.adc_path);
    println!("  adc_exists: {}", diagnostics.adc_exists);
    println!(
        "  adc_has_refresh_token: {}",
        diagnostics.adc_has_refresh_token
    );
    println!(
        "  impersonation_service_account: {}",
        diagnostics.impersonation_service_account
    );
    println!("  impersonation_ready: {}", diagnostics.impersonation_ready);
    println!();
    if diagnostics.recommendations.is_empty() {
        println!("checks: OK");
    } else {
        println!("checks:");
        for recommendation in &diagnostics.recommendations {
            println!("  - {}", recommendation);
        }
    }
    println!();
    println!("direnv template:");
    println!("{}", diagnostics.direnv_template);
    Ok(())
}

fn run_status(spec: &InfraSpec) -> Result<(), String> {
    let report = evaluate_health(spec);

    println!("infra status");
    println!("  environment: {}", spec.environment);
    println!();
    for item in &report.items {
        let marker = if item.ok { "OK" } else { "FAIL" };
        println!("  [{}] {:<16} {}", marker, item.name, item.detail);
    }
    println!();
    println!("overall: {}", if report.overall_ok { "OK" } else { "FAIL" });

    if report.overall_ok {
        Ok(())
    } else {
        Err("one or more health checks failed".to_string())
    }
}

fn build_entrypoint_input_mocks<T>(
    dag: &Dag<T>,
    provided_inputs: &HashMap<String, String>,
    allow_access_token_env_fallback: bool,
) -> Result<BoundaryMocks, String> {
    let entrypoints = detect_entrypoints(dag);
    let mut input_mocks = BoundaryMocks::new();
    let mut missing_ports = Vec::new();

    for (node_id, port_name, type_id) in entrypoints.entrypoint_ports {
        let raw = provided_inputs
            .get(&port_name.0)
            .cloned()
            .or_else(|| {
                if allow_access_token_env_fallback && port_name.0 == "access_token" {
                    std::env::var("GCP_ACCESS_TOKEN")
                        .ok()
                        .filter(|value| !value.trim().is_empty())
                } else {
                    None
                }
            });

        if let Some(raw) = raw {
            let parsed = parse_input_value(&type_id.0, &raw)?;
            input_mocks.set_input(node_id.0, port_name.0, parsed);
        } else {
            missing_ports.push(port_name.0);
        }
    }

    if missing_ports.is_empty() {
        return Ok(input_mocks);
    }

    missing_ports.sort();
    missing_ports.dedup();
    Err(format!(
        "missing entrypoint input(s): {} (pass --input NAME=VALUE)",
        missing_ports.join(", ")
    ))
}

fn parse_input_value(type_id: &str, raw: &str) -> Result<Value, String> {
    match type_id {
        "Bool" => match raw.trim().to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(Value::Bool(true)),
            "false" | "0" => Ok(Value::Bool(false)),
            _ => Err(format!("invalid Bool input value '{raw}'")),
        },
        "Int" | "i64" | "I64" => raw
            .trim()
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| format!("invalid Int input value '{raw}'")),
        _ => Ok(Value::Str(raw.to_string())),
    }
}

fn parse_cli_args(argv: &[String]) -> Result<InfraCliArgs, String> {
    let Some(raw_command) = argv.get(1).map(String::as_str) else {
        return Ok(InfraCliArgs::for_command(InfraCommand::Help));
    };
    let command = match raw_command {
        "bootstrap" => InfraCommand::Bootstrap,
        "plan" => InfraCommand::Plan,
        "apply" => InfraCommand::Apply,
        "spec" => InfraCommand::Spec,
        "graph" => InfraCommand::Graph,
        "login" => InfraCommand::Login,
        "status" => InfraCommand::Status,
        "help" | "-h" | "--help" => InfraCommand::Help,
        other => return Err(format!("unknown infra subcommand '{other}'")),
    };
    let mut args = InfraCliArgs::for_command(command);

    let mut i = 2;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if matches!(arg, "-h" | "--help") {
            args.command = InfraCommand::Help;
            return Ok(args);
        }
        if arg == "--execute" {
            args.execute = true;
            i += 1;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--env=") {
            args.environment = value.to_string();
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--runtime=") {
            args.runtime = parse_runtime(value)?;
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--target=") {
            args.target.push(value.to_string());
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--skip=") {
            args.skip.push(value.to_string());
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--input=") {
            let (key, parsed) = parse_input_kv(value)?;
            args.inputs.insert(key.to_string(), parsed.to_string());
            i += 1;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--access-token=") {
            args.inputs
                .insert("access_token".to_string(), value.to_string());
            i += 1;
            continue;
        }

        match arg {
            "--env" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--env requires a value".to_string())?;
                args.environment = value.to_string();
            }
            "--runtime" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--runtime requires a value".to_string())?;
                args.runtime = parse_runtime(value)?;
            }
            "--target" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--target requires a value".to_string())?;
                args.target.push(value.to_string());
            }
            "--skip" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--skip requires a value".to_string())?;
                args.skip.push(value.to_string());
            }
            "--input" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--input requires NAME=VALUE".to_string())?;
                let (key, parsed) = parse_input_kv(value)?;
                args.inputs.insert(key.to_string(), parsed.to_string());
            }
            "--access-token" => {
                i += 1;
                let value = argv
                    .get(i)
                    .ok_or_else(|| "--access-token requires a value".to_string())?;
                args.inputs
                    .insert("access_token".to_string(), value.to_string());
            }
            other if other.starts_with('-') => return Err(format!("unknown flag '{other}'")),
            other => return Err(format!("unexpected argument '{other}'")),
        }

        i += 1;
    }

    Ok(args)
}

fn parse_runtime(raw: &str) -> Result<CloudRuntimeKind, String> {
    CloudRuntimeKind::parse(raw)
        .ok_or_else(|| format!("unknown runtime '{raw}' (expected local|github|metadata)"))
}

fn parse_input_kv(raw: &str) -> Result<(&str, &str), String> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| format!("invalid --input '{raw}' (expected NAME=VALUE)"))?;
    if key.trim().is_empty() {
        return Err(format!("invalid --input '{raw}' (empty NAME)"));
    }
    Ok((key.trim(), value))
}

fn spec_for_env(environment: &str) -> Result<&'static InfraSpec, String> {
    match environment {
        "dev" => Ok(&DEV_SPEC),
        "ci" => Ok(&CI_SPEC),
        "test" => Ok(&TEST_SPEC),
        "prod" => Ok(&PROD_SPEC),
        other => Err(format!(
            "unknown environment '{other}' (expected dev|ci|test|prod)"
        )),
    }
}

fn infra_spec_json(spec: &InfraSpec) -> serde_json::Value {
    let service_accounts = spec
        .service_accounts
        .iter()
        .map(|sa| {
            json!({
                "name": sa.name,
                "display_name": sa.display_name,
                "description": sa.description,
                "email": sa.email(spec.config.secrets_project),
                "self_roles": sa.self_roles,
                "wif_bindings": sa.wif_bindings,
            })
        })
        .collect::<Vec<_>>();

    let secrets = spec
        .secrets
        .iter()
        .map(|secret| {
            json!({
                "env_name": secret.env_name,
                "secret_id": secret.secret_id,
                "requirement": requirement_label(secret.requirement),
                "status": status_label(secret.status),
                "rotation": rotation_label(secret.rotation),
                "scopes": secret.scopes,
            })
        })
        .collect::<Vec<_>>();

    let wif_mapping = spec
        .wif
        .attribute_mapping
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect::<HashMap<_, _>>();

    json!({
        "environment": spec.environment,
        "config": {
            "project": spec.config.project,
            "project_number": spec.config.project_number,
            "region": spec.config.region,
            "zone": spec.config.zone,
            "domain": spec.config.domain,
            "name_prefix": spec.config.name_prefix,
            "secrets_project": spec.config.secrets_project,
            "secrets_prefix": spec.config.secrets_prefix,
        },
        "wif": {
            "project_number": spec.wif.project_number,
            "pool_id": spec.wif.pool_id,
            "provider_id": spec.wif.provider_id,
            "oidc_issuer_uri": spec.wif.oidc_issuer_uri,
            "attribute_mapping": wif_mapping,
            "attribute_condition": spec.wif.attribute_condition,
            "pool_resource_name": spec.wif.pool_resource_name(),
            "provider_resource_name": spec.wif.provider_resource_name(),
        },
        "service_accounts": service_accounts,
        "secrets": secrets,
    })
}

fn requirement_label(requirement: SecretRequirement) -> &'static str {
    match requirement {
        SecretRequirement::Required => "required",
        SecretRequirement::Optional => "optional",
    }
}

fn status_label(status: SecretStatus) -> &'static str {
    match status {
        SecretStatus::Active => "active",
        SecretStatus::Deleted => "deleted",
    }
}

fn rotation_label(rotation: RotationHandler) -> &'static str {
    match rotation {
        RotationHandler::Manual => "manual",
        RotationHandler::GitHubPat => "github_pat",
        RotationHandler::ServiceAccountKey => "service_account_key",
        RotationHandler::None => "none",
    }
}

fn print_help() {
    println!("gunbc-infra - Unified infra CLI");
    println!();
    println!("Usage:");
    println!("  gunbc-infra <subcommand> [OPTIONS]");
    println!();
    println!("Subcommands:");
    println!("  bootstrap   Build or execute WIF bootstrap DAG");
    println!("  plan        Execute infra planning DAG");
    println!("  apply       Preview or execute infra apply DAG");
    println!("  spec        Print InfraSpec JSON");
    println!("  graph       Print InfraSpec graph (DOT)");
    println!("  login       Verify ADC + impersonation and print direnv template");
    println!("  status      Health checks for auth, projects, service accounts, secrets");
    println!("  help        Show this help");
    println!();
    println!("Common options:");
    println!("  --env <dev|ci|test|prod>        Environment (default: dev)");
    println!("  --runtime <local|github|metadata>  Runtime kind (default: local)");
    println!("  --target <id>                   Include target (repeatable)");
    println!("  --skip <id>                     Exclude target (repeatable)");
    println!("  --input NAME=VALUE              Entrypoint input (repeatable)");
    println!("  --execute                       Execute mutating subcommands");
    println!("  --access-token TOKEN            Convenience alias for --input access_token=TOKEN");
    println!("  -h, --help                      Show this help");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_plan_defaults_env_and_runtime() {
        let parsed = parse_cli_args(&argv(&["gunbc-infra", "plan"])).expect("should parse");
        assert_eq!(parsed.command, InfraCommand::Plan);
        assert_eq!(parsed.environment, "dev");
        assert_eq!(parsed.runtime, CloudRuntimeKind::LocalDev);
        assert!(!parsed.execute);
    }

    #[test]
    fn parse_apply_collects_targets_skips_and_inputs() {
        let parsed = parse_cli_args(&argv(&[
            "gunbc-infra",
            "apply",
            "--env",
            "ci",
            "--runtime=github",
            "--target",
            "secret:github-token",
            "--skip=secret:aws-role",
            "--input",
            "secret_value=abc",
            "--execute",
        ]))
        .expect("should parse");

        assert_eq!(parsed.command, InfraCommand::Apply);
        assert_eq!(parsed.environment, "ci");
        assert_eq!(parsed.runtime, CloudRuntimeKind::GitHubActions);
        assert_eq!(parsed.target, vec!["secret:github-token".to_string()]);
        assert_eq!(parsed.skip, vec!["secret:aws-role".to_string()]);
        assert_eq!(
            parsed.inputs.get("secret_value"),
            Some(&"abc".to_string())
        );
        assert!(parsed.execute);
    }

    #[test]
    fn parse_access_token_alias_sets_input() {
        let parsed = parse_cli_args(&argv(&[
            "gunbc-infra",
            "bootstrap",
            "--access-token",
            "tok",
        ]))
        .expect("should parse");
        assert_eq!(
            parsed.inputs.get("access_token"),
            Some(&"tok".to_string())
        );
    }

    #[test]
    fn parse_runtime_rejects_unknown_value() {
        let err = parse_runtime("bogus").expect_err("unknown runtime should fail");
        assert!(err.contains("unknown runtime"));
    }

    #[test]
    fn parse_login_and_status_commands() {
        let login = parse_cli_args(&argv(&["gunbc-infra", "login"])).expect("login should parse");
        assert_eq!(login.command, InfraCommand::Login);

        let status = parse_cli_args(&argv(&["gunbc-infra", "status"])).expect("status should parse");
        assert_eq!(status.command, InfraCommand::Status);
    }

    #[test]
    fn parse_input_value_handles_bool_and_int() {
        assert_eq!(parse_input_value("Bool", "true").unwrap(), Value::Bool(true));
        assert_eq!(parse_input_value("Int", "42").unwrap(), Value::Int(42));
    }

    #[test]
    fn spec_for_env_rejects_unknown() {
        let err = spec_for_env("staging").expect_err("unknown env should fail");
        assert!(err.contains("unknown environment"));
    }
}
