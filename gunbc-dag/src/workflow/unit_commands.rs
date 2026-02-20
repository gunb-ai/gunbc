//! Process-unit-to-command mappings for CI and test-all workflows (WF6/WF7).
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
        UnitCommand::cargo("pragma ensure", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma", "--", "--mode=ensure"]),
    );
    commands.insert(
        NodeId::from("ci.codegen"),
        UnitCommand::cargo("codegen ensure", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen", "--", "codegen"]),
    );
    commands.insert(
        NodeId::from("ci.bootstrap"),
        UnitCommand::cargo("bootstrap ensure", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-bootstrap", "--", "--mode=ensure"]),
    );
    commands.insert(
        NodeId::from("ci.pragma"),
        UnitCommand::cargo("pragma", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma"]),
    );
    commands.insert(
        NodeId::from("ci.testgen"),
        UnitCommand::cargo("testgen", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-testgen"]),
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
        UnitCommand::cargo("clippy", vec!["clippy", "--all-targets", "--", "-D", "warnings"]),
    );
    commands.insert(
        NodeId::from("ci.guardrails"),
        UnitCommand::cargo("guardrails", vec!["test", "--workspace", "--lib", "--", "guardrail"]),
    );
    commands.insert(
        NodeId::from("ci.verify"),
        UnitCommand::cargo("verify", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen", "--", "codegen", "--mode=verify"]),
    );
    // ci.report has no command — handled as no-op by the executor.

    commands
}

/// Build command map for test-all workflow units.
///
/// test-all shares many units with CI via the global ledger's cross-workflow
/// dedup. Commands that match CI counterparts produce identical ledger keys,
/// so warm-path runs skip them via CachedHit.
pub fn test_all_unit_commands() -> BTreeMap<NodeId, UnitCommand> {
    let mut commands = BTreeMap::new();

    commands.insert(
        NodeId::from("test_all.lint_upsert"),
        UnitCommand::cargo("pragma ensure", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-pragma", "--", "--mode=ensure"]),
    );
    commands.insert(
        NodeId::from("test_all.codegen"),
        UnitCommand::cargo("codegen ensure", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen", "--", "codegen"]),
    );
    commands.insert(
        NodeId::from("test_all.testgen"),
        UnitCommand::cargo("testgen", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-testgen"]),
    );
    commands.insert(
        NodeId::from("test_all.build_compile"),
        UnitCommand::cargo("build", vec!["build", "--workspace"]),
    );
    commands.insert(
        NodeId::from("test_all.verify_fix"),
        UnitCommand::cargo("verify-fix", vec!["run", "-p", "gunbc-dag", "--bin", "gunbc-codegen", "--", "codegen", "--mode=ensure"]),
    );
    commands.insert(
        NodeId::from("test_all.cargo_test_xl"),
        UnitCommand::cargo("test-xl", vec!["test", "--workspace", "--", "--include-ignored"]),
    );
    // test_all.report has no command — handled as no-op by the executor.

    commands
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
}
