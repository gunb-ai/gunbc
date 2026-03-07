//! Workflow unit command derivation.
//!
//! Commands are derived structurally from the workflow catalog (variant metadata
//! and stage names) rather than hand-authored in a DSL data file.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use gunbc_ir::NodeId;
use gunbc_workflow::UnitCommand;

type WorkflowCommandMap = BTreeMap<String, BTreeMap<NodeId, UnitCommand>>;

static WORKFLOW_COMMANDS: OnceLock<Result<WorkflowCommandMap, String>> = OnceLock::new();

fn commands_by_workflow() -> Result<&'static WorkflowCommandMap, String> {
    WORKFLOW_COMMANDS
        .get_or_init(derive_all_workflow_commands)
        .as_ref()
        .map_err(|error| error.clone())
}

/// Build command map for CI workflow units.
pub fn ci_unit_commands() -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    workflow_unit_commands("ci")
}

/// Build command map for test-all workflow units.
pub fn test_all_unit_commands() -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    workflow_unit_commands("test-all")
}

/// Build command map for a supported workflow name.
pub fn workflow_unit_commands(
    workflow_name: &str,
) -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    let Some(variant) = super::catalog::resolve_workflow_variant(workflow_name)? else {
        return Err(format!(
            "workflow '{}' does not support execution mode; use --plan",
            workflow_name
        ));
    };
    commands_by_workflow()?
        .get(&variant.canonical_name)
        .cloned()
        .ok_or_else(|| {
            format!(
                "workflow '{}' does not support execution mode; use --plan",
                workflow_name
            )
        })
}

/// Derive commands for all workflows from catalog metadata.
fn derive_all_workflow_commands() -> Result<WorkflowCommandMap, String> {
    let variants = super::catalog::all_workflow_variant_defs()?;
    let mut out = BTreeMap::new();

    for variant in variants {
        let spec = super::spec_builders::workflow_spec(&variant.canonical_name)?;
        let mut commands = BTreeMap::new();

        for node in &spec.dag.nodes {
            let stage_name = node
                .id
                .0
                .strip_prefix(&format!("{}.", variant.namespace))
                .unwrap_or(&node.id.0);

            if let Some(cmd) = derive_stage_command(
                stage_name,
                &variant.namespace,
                &variant.canonical_name,
                variant.is_tool,
            ) {
                commands.insert(node.id.clone(), cmd);
            }
        }

        // Aliases get the same commands.
        for alias in &variant.aliases {
            out.insert(alias.clone(), commands.clone());
        }

        out.insert(variant.canonical_name.clone(), commands);
    }

    Ok(out)
}

/// Derive the UnitCommand for a single stage, or None for report/no-op stages.
fn derive_stage_command(
    stage_name: &str,
    namespace: &str,
    workflow_name: &str,
    is_tool: bool,
) -> Option<UnitCommand> {
    // Report stages are no-ops.
    if stage_name == "report" {
        return None;
    }

    // Capability stages (shared across all workflows).
    match stage_name {
        "compilation_ensure" => {
            return Some(UnitCommand::cargo(
                "compilation ensure",
                vec!["build", "--workspace", "--bins"],
            ));
        }
        "codegen_ensure" => {
            return Some(codegen_dag_command("codegen ensure"));
        }
        _ => {}
    }

    // Well-known cargo stages.
    match stage_name {
        "build_compile" | "build" => {
            return Some(UnitCommand::cargo("build", vec!["build", "--workspace"]));
        }
        "test_run" => {
            return Some(UnitCommand::cargo("test", vec!["test", "--workspace"]));
        }
        "clippy_run" => {
            return Some(UnitCommand::cargo(
                "clippy",
                vec!["clippy", "--all-targets", "--", "-D", "warnings"],
            ));
        }
        "guardrails" => {
            return Some(UnitCommand::cargo(
                "guardrails",
                vec!["test", "--workspace", "--lib", "--", "guardrail"],
            ));
        }
        "cargo_test_xl" => {
            return Some(UnitCommand::cargo(
                "test-xl",
                vec!["test", "--workspace", "--", "--include-ignored"],
            ));
        }
        _ => {}
    }

    // CI/test-all codegen/verify stages.
    match stage_name {
        "codegen" | "verify" | "verify_fix" => {
            let label = stage_name.replace('_', " ");
            return Some(codegen_dag_command(&label));
        }
        _ => {}
    }

    // CI/test-all stage → tool binary mapping.
    if let Some(cmd) = ci_stage_to_tool_command(stage_name) {
        return Some(cmd);
    }

    // SDLC-specific stages.
    if namespace == "sdlc" {
        return derive_sdlc_stage_command(stage_name);
    }

    // Gist-specific execution stages.
    if namespace == "gist" && stage_name == "gist_create" {
        return Some(derive_gist_command(workflow_name));
    }

    // Deps-specific stages.
    if namespace == "deps" {
        if let Some(cmd) = derive_deps_stage_command(stage_name) {
            return Some(cmd);
        }
    }

    // Default for tool workflows: run the namespace's binary.
    if is_tool {
        let label = format!("{} ensure", namespace);
        return Some(tool_binary_command(namespace, &label, &[]));
    }

    None
}

/// Map CI/test-all stage names to their corresponding tool binaries.
fn ci_stage_to_tool_command(stage_name: &str) -> Option<UnitCommand> {
    let (tool, label) = match stage_name {
        "lint_upsert" => ("pragma", "pragma ensure"),
        "bootstrap" => ("bootstrap", "bootstrap ensure"),
        "pragma" => ("pragma", "pragma"),
        "testgen" => ("testgen", "testgen"),
        _ => return None,
    };
    Some(tool_binary_command(tool, label, &[]))
}

fn derive_sdlc_stage_command(stage_name: &str) -> Option<UnitCommand> {
    let (label, path) = match stage_name {
        "intake" => ("sdlc intake check", "dsl/workflows/sdlc.dag"),
        "worker" => ("sdlc worker check", "dsl/funcs/sdlc_worker.dag"),
        _ => return None,
    };
    Some(UnitCommand::cargo(
        label,
        vec!["run", "-p", "daglang-cli", "--", "check", path],
    ))
}

fn derive_gist_command(workflow_name: &str) -> UnitCommand {
    match workflow_name {
        "gist-diff" => tool_binary_command("gist", "gist diff upload", &["gist-diff"]),
        "gist-recent" => tool_binary_command("gist", "gist recent upload", &["gist-recent"]),
        _ => tool_binary_command("gist", "gist snapshot upload", &[]),
    }
}

fn derive_deps_stage_command(stage_name: &str) -> Option<UnitCommand> {
    match stage_name {
        "execute_installs" => Some(tool_binary_command("deps", "deps", &["deps"])),
        "write_deps_toml" => Some(tool_binary_command("deps", "deps", &["deps-generate"])),
        _ => None,
    }
}

fn codegen_dag_command(label: &str) -> UnitCommand {
    tool_binary_command("codegen-dag", label, &[])
}

fn tool_binary_command(tool_name: &str, label: &str, extra_args: &[&str]) -> UnitCommand {
    let bin_name = format!("gunbc-{tool_name}");
    let mut args: Vec<String> = vec![
        "run".into(),
        "-p".into(),
        "gunbc-app".into(),
        "--bin".into(),
        bin_name,
    ];
    if !extra_args.is_empty() {
        args.push("--".into());
        args.extend(extra_args.iter().map(|a| a.to_string()));
    }
    UnitCommand::cargo(label, args.iter().map(|s| s.as_str()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_commands_cover_all_non_report_units() {
        let commands = ci_unit_commands().expect("ci commands");
        // 10 units have commands (report is a no-op).
        assert_eq!(commands.len(), 10);
        assert!(commands.contains_key(&NodeId::from("ci.codegen")));
        assert!(commands.contains_key(&NodeId::from("ci.build_compile")));
        assert!(commands.contains_key(&NodeId::from("ci.test_run")));
        assert!(commands.contains_key(&NodeId::from("ci.clippy_run")));
        assert!(!commands.contains_key(&NodeId::from("ci.report")));
    }

    #[test]
    fn test_all_commands_cover_all_non_report_units() {
        let commands = test_all_unit_commands().expect("test-all commands");
        // 6 units have commands (report is a no-op).
        assert_eq!(commands.len(), 6);
        assert!(commands.contains_key(&NodeId::from("test_all.codegen")));
        assert!(commands.contains_key(&NodeId::from("test_all.build_compile")));
        assert!(commands.contains_key(&NodeId::from("test_all.cargo_test_xl")));
        assert!(!commands.contains_key(&NodeId::from("test_all.report")));
    }

    #[test]
    fn workflow_unit_commands_supports_bootstrap() {
        let commands = workflow_unit_commands("bootstrap").expect("bootstrap commands");
        assert!(commands.contains_key(&NodeId::from("bootstrap.compilation_ensure")));
        assert!(commands.contains_key(&NodeId::from("bootstrap.codegen_ensure")));
        assert!(commands.contains_key(&NodeId::from("bootstrap.upsert_makefile")));
        assert!(commands.contains_key(&NodeId::from("bootstrap.upsert_gitignore")));
    }

    #[test]
    fn workflow_unit_commands_supports_gist_aliases() {
        let gist = workflow_unit_commands("gist").expect("gist");
        let snapshot = workflow_unit_commands("gist-snapshot").expect("snapshot");
        assert_eq!(gist.len(), snapshot.len());
        assert!(gist.contains_key(&NodeId::from("gist.gist_create")));
    }

    #[test]
    fn workflow_unit_commands_supports_sdlc() {
        let commands = workflow_unit_commands("sdlc").expect("sdlc commands");
        assert!(commands.contains_key(&NodeId::from("sdlc.compilation_ensure")));
        assert!(commands.contains_key(&NodeId::from("sdlc.codegen_ensure")));
        assert!(commands.contains_key(&NodeId::from("sdlc.intake")));
        assert!(commands.contains_key(&NodeId::from("sdlc.worker")));
        assert!(!commands.contains_key(&NodeId::from("sdlc.report")));
    }

    #[test]
    fn derived_ci_commands_match_expected_programs() {
        let commands = ci_unit_commands().expect("ci commands");
        let codegen = &commands[&NodeId::from("ci.codegen")];
        assert_eq!(codegen.program, "cargo");
        assert!(codegen.args.contains(&"gunbc-codegen-dag".to_string()));

        let build = &commands[&NodeId::from("ci.build_compile")];
        assert_eq!(build.args, vec!["build", "--workspace"]);

        let test = &commands[&NodeId::from("ci.test_run")];
        assert_eq!(test.args, vec!["test", "--workspace"]);

        let clippy = &commands[&NodeId::from("ci.clippy_run")];
        assert!(clippy.args.contains(&"-D".to_string()));
    }

    #[test]
    fn gist_diff_includes_subcommand_arg() {
        let commands = workflow_unit_commands("gist-diff").expect("gist-diff");
        let create = &commands[&NodeId::from("gist.gist_create")];
        assert!(create.args.contains(&"gist-diff".to_string()));
    }

    /// F3: Transitional parity test — derived commands match what was in
    /// workflow_commands.dag. This test validates the derivation before we
    /// deleted the DSL file.
    #[test]
    fn derived_commands_cover_all_workflows() {
        // Every workflow in the catalog should have derived commands.
        let all = super::super::catalog::all_workflow_variant_defs().expect("variants");
        for variant in all {
            let commands = workflow_unit_commands(&variant.canonical_name)
                .unwrap_or_else(|e| panic!("commands for '{}': {e}", variant.canonical_name));
            // Every non-report node in the spec should have a command.
            let spec = super::super::spec_builders::workflow_spec(&variant.canonical_name)
                .unwrap_or_else(|e| panic!("spec for '{}': {e}", variant.canonical_name));
            for node in &spec.dag.nodes {
                let stage = node
                    .id
                    .0
                    .strip_prefix(&format!("{}.", variant.namespace))
                    .unwrap_or(&node.id.0);
                if stage == "report" {
                    assert!(
                        !commands.contains_key(&node.id),
                        "report node '{}' should not have a command",
                        node.id.0
                    );
                } else {
                    assert!(
                        commands.contains_key(&node.id),
                        "non-report node '{}' in workflow '{}' should have a derived command",
                        node.id.0,
                        variant.canonical_name
                    );
                }
            }
        }
    }

    #[test]
    fn deps_stages_include_subcommand_args() {
        let commands = workflow_unit_commands("deps").expect("deps");
        let installs = &commands[&NodeId::from("deps.execute_installs")];
        assert!(installs.args.contains(&"deps".to_string()));
        let write = &commands[&NodeId::from("deps.write_deps_toml")];
        assert!(write.args.contains(&"deps-generate".to_string()));
    }
}
