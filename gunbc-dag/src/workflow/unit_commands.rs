//! Process-unit-to-command mappings for executable workflows (WF6+).
//!
//! Each workflow unit maps to a shell command that implements its behavior.
//! Report and aggregate nodes have no command (executed as no-ops by the executor).

use std::collections::BTreeMap;

use gunbc_ir::NodeId;

use super::executor::UnitCommand;

/// Build command map for CI workflow units.
///
/// Each CI unit maps to the shell command that replaces the old per-node DAG
/// execution. Commands inherit stdio so output is visible in real-time.
pub fn ci_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();

    commands.insert(
        NodeId::from("ci.lint_upsert"),
        UnitCommand::cargo(
            "pragma ensure",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma"],
        ),
    );
    commands.insert(NodeId::from("ci.codegen"), codegen_ensure_command());
    commands.insert(
        NodeId::from("ci.bootstrap"),
        UnitCommand::cargo(
            "bootstrap ensure",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-bootstrap"],
        ),
    );
    commands.insert(
        NodeId::from("ci.pragma"),
        UnitCommand::cargo(
            "pragma",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma"],
        ),
    );
    commands.insert(
        NodeId::from("ci.testgen"),
        UnitCommand::cargo(
            "testgen",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-testgen"],
        ),
    );
    commands.insert(
        NodeId::from("ci.build_compile"),
        UnitCommand::cargo("build", vec!["build", "--workspace"]),
    );
    commands.insert(
        NodeId::from("ci.test_run"),
        UnitCommand::cargo("test", vec!["test", "--workspace"]),
    );
    commands.insert(
        NodeId::from("ci.clippy_run"),
        UnitCommand::cargo(
            "clippy",
            vec!["clippy", "--all-targets", "--", "-D", "warnings"],
        ),
    );
    commands.insert(
        NodeId::from("ci.guardrails"),
        UnitCommand::cargo(
            "guardrails",
            vec!["test", "--workspace", "--lib", "--", "guardrail"],
        ),
    );
    // TODO(Phase 2): ci.verify needs --mode=verify support in the generated runner.
    // The generated CLI runner for codegen-dag does not accept --mode; drift
    // detection is temporarily unavailable.
    commands.insert(
        NodeId::from("ci.verify"),
        UnitCommand::cargo(
            "verify",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen-dag"],
        ),
    );
    // ci.report has no command — handled as no-op by the executor.

    commands
}

/// Build command map for test-all workflow units.
///
/// test-all shares many units with CI via the global planner's cross-workflow
/// dedup. Commands that match CI counterparts produce identical planner keys.
pub fn test_all_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();

    commands.insert(
        NodeId::from("test_all.lint_upsert"),
        UnitCommand::cargo(
            "pragma ensure",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma"],
        ),
    );
    commands.insert(NodeId::from("test_all.codegen"), codegen_ensure_command());
    commands.insert(
        NodeId::from("test_all.testgen"),
        UnitCommand::cargo(
            "testgen",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-testgen"],
        ),
    );
    commands.insert(
        NodeId::from("test_all.build_compile"),
        UnitCommand::cargo("build", vec!["build", "--workspace"]),
    );
    commands.insert(
        NodeId::from("test_all.verify_fix"),
        UnitCommand::cargo(
            "verify-fix",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen-dag"],
        ),
    );
    commands.insert(
        NodeId::from("test_all.cargo_test_xl"),
        UnitCommand::cargo(
            "test-xl",
            vec!["test", "--workspace", "--", "--include-ignored"],
        ),
    );
    // test_all.report has no command — handled as no-op by the executor.

    commands
}

fn compilation_ensure_command() -> UnitCommand {
    UnitCommand::cargo("compilation ensure", vec!["build", "--workspace", "--bins"])
}

fn codegen_ensure_command() -> UnitCommand {
    UnitCommand::cargo(
        "codegen ensure",
        vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen-dag"],
    )
}

fn gist_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("gist.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("gist.codegen_ensure"),
        codegen_ensure_command(),
    );
    commands.insert(
        NodeId::from("gist.gist_create"),
        UnitCommand::cargo(
            "gist snapshot upload",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-gist"],
        ),
    );
    commands
}

fn gist_diff_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("gist.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("gist.codegen_ensure"),
        codegen_ensure_command(),
    );
    commands.insert(
        NodeId::from("gist.gist_create"),
        UnitCommand::cargo(
            "gist diff upload",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-gist-diff"],
        ),
    );
    commands
}

fn gist_recent_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("gist.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("gist.codegen_ensure"),
        codegen_ensure_command(),
    );
    commands.insert(
        NodeId::from("gist.gist_create"),
        UnitCommand::cargo(
            "gist recent upload",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-gist-recent"],
        ),
    );
    commands
}

fn bootstrap_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("bootstrap.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("bootstrap.codegen_ensure"),
        codegen_ensure_command(),
    );
    let bootstrap_cmd = UnitCommand::cargo(
        "bootstrap ensure",
        vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-bootstrap"],
    );
    commands.insert(
        NodeId::from("bootstrap.upsert_makefile"),
        bootstrap_cmd.clone(),
    );
    commands.insert(NodeId::from("bootstrap.upsert_gitignore"), bootstrap_cmd);
    commands
}

fn makegen_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("makegen.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("makegen.codegen_ensure"),
        codegen_ensure_command(),
    );
    commands.insert(
        NodeId::from("makegen.upsert_makefile"),
        UnitCommand::cargo(
            "makegen ensure",
            vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-makegen"],
        ),
    );
    commands
}

fn pragma_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("pragma.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("pragma.codegen_ensure"),
        codegen_ensure_command(),
    );
    let pragma_cmd = UnitCommand::cargo(
        "pragma ensure",
        vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma"],
    );
    commands.insert(NodeId::from("pragma.upsert_clippy"), pragma_cmd.clone());
    commands.insert(NodeId::from("pragma.upsert_allowlist"), pragma_cmd.clone());
    commands.insert(NodeId::from("pragma.upsert_policy"), pragma_cmd);
    commands
}

fn deps_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("deps.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("deps.codegen_ensure"),
        codegen_ensure_command(),
    );
    let deps_cmd = UnitCommand::cargo(
        "deps",
        vec!["run", "-p", "gunbc-deps", "--bin", "gunbc-deps", "--"],
    );
    commands.insert(NodeId::from("deps.execute_installs"), deps_cmd.clone());
    commands.insert(NodeId::from("deps.write_deps_toml"), deps_cmd);
    commands
}

fn build_all_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();
    commands.insert(
        NodeId::from("build_all.compilation_ensure"),
        compilation_ensure_command(),
    );
    commands.insert(
        NodeId::from("build_all.codegen_ensure"),
        codegen_ensure_command(),
    );
    commands.insert(
        NodeId::from("build_all.build"),
        UnitCommand::cargo("build-all", vec!["build", "--workspace"]),
    );
    commands
}

/// Build command map for a supported workflow name.
///
/// Name resolution uses the workflow catalog (handles aliases like
/// "gist_snapshot" → "gist" and normalization like "test_all" → "test-all").
pub fn workflow_unit_commands(
    workflow_name: &str,
) -> Result<BTreeMap<NodeId, UnitCommand>, String> {
    let variant = super::catalog::resolve_workflow_variant(workflow_name).ok_or_else(|| {
        format!(
            "workflow '{}' does not support execution mode; use --plan",
            workflow_name
        )
    })?;
    let commands = match variant.canonical_name {
        "ci" => ci_unit_commands(),
        "test-all" => test_all_unit_commands(),
        "gist" => gist_unit_commands(),
        "gist-diff" => gist_diff_unit_commands(),
        "gist-recent" => gist_recent_unit_commands(),
        "bootstrap" => bootstrap_unit_commands(),
        "makegen" => makegen_unit_commands(),
        "pragma" => pragma_unit_commands(),
        "deps" => deps_unit_commands(),
        "build-all" => build_all_unit_commands(),
        other => {
            return Err(format!(
                "workflow '{}' does not support execution mode; use --plan",
                other
            ))
        }
    };
    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_commands_cover_all_non_report_units() {
        let commands = ci_unit_commands();
        // 10 units have commands (report is a no-op).
        assert_eq!(commands.len(), 10);
        assert!(commands.contains_key(&NodeId::from("ci.codegen")));
        assert!(commands.contains_key(&NodeId::from("ci.build_compile")));
        assert!(commands.contains_key(&NodeId::from("ci.test_run")));
        assert!(commands.contains_key(&NodeId::from("ci.clippy_run")));
        // report is intentionally missing (no-op).
        assert!(!commands.contains_key(&NodeId::from("ci.report")));
    }

    #[test]
    fn test_all_commands_cover_all_non_report_units() {
        let commands = test_all_unit_commands();
        // 6 units have commands (report is a no-op).
        assert_eq!(commands.len(), 6);
        assert!(commands.contains_key(&NodeId::from("test_all.codegen")));
        assert!(commands.contains_key(&NodeId::from("test_all.build_compile")));
        assert!(commands.contains_key(&NodeId::from("test_all.cargo_test_xl")));
        assert!(!commands.contains_key(&NodeId::from("test_all.report")));
    }

    #[test]
    fn ci_build_command_targets_workspace() {
        let commands = ci_unit_commands();
        let build = commands
            .get(&NodeId::from("ci.build_compile"))
            .expect("build command");
        assert_eq!(build.program, "cargo");
        assert!(build.args.contains(&"--workspace".to_string()));
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
    fn workflow_unit_commands_supports_pragma_terminal_branches() {
        let commands = workflow_unit_commands("pragma").expect("pragma commands");
        assert!(commands.contains_key(&NodeId::from("pragma.upsert_clippy")));
        assert!(commands.contains_key(&NodeId::from("pragma.upsert_allowlist")));
        assert!(commands.contains_key(&NodeId::from("pragma.upsert_policy")));
    }

    #[test]
    fn workflow_unit_commands_supports_deps_terminal_branches() {
        let commands = workflow_unit_commands("deps").expect("deps commands");
        assert!(commands.contains_key(&NodeId::from("deps.execute_installs")));
        assert!(commands.contains_key(&NodeId::from("deps.write_deps_toml")));
    }

    #[test]
    fn ci_verify_uses_codegen_dag_binary() {
        let ci_commands = ci_unit_commands();
        let verify = ci_commands
            .get(&NodeId::from("ci.verify"))
            .expect("ci.verify command");
        assert_eq!(verify.program, "cargo");
        assert!(
            verify.args.contains(&"gunbc-codegen-dag".to_string()),
            "ci.verify must use codegen DAG wrapper binary for --mode support"
        );

        let ci_codegen = ci_commands
            .get(&NodeId::from("ci.codegen"))
            .expect("ci.codegen command");
        assert!(
            ci_codegen.args.contains(&"gunbc-codegen-dag".to_string()),
            "ci.codegen should use codegen DAG wrapper binary for ensure mode"
        );
        let test_all_commands = test_all_unit_commands();
        let test_all_codegen = test_all_commands
            .get(&NodeId::from("test_all.codegen"))
            .expect("test_all.codegen command");
        assert!(
            test_all_codegen
                .args
                .contains(&"gunbc-codegen-dag".to_string()),
            "test_all.codegen should use codegen DAG wrapper binary"
        );
    }

    #[test]
    fn workflow_unit_commands_supports_gist_aliases() {
        let gist = workflow_unit_commands("gist").expect("gist");
        let snapshot = workflow_unit_commands("gist-snapshot").expect("snapshot");
        assert_eq!(gist.len(), snapshot.len());
        assert!(gist.contains_key(&NodeId::from("gist.gist_create")));
    }
}
