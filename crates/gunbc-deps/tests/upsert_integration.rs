use std::collections::HashMap;

use gunbc_deps::ops::{CommandSpec, DepOp};
use gunbc_exec::{execute, ExecError, ExecutionLog, Value};
use gunbc_ir::algebra::Value as IrValue;
use gunbc_ir::build::{edge, eq_guarded_port, port};
use gunbc_ir::dag::{Dag, DagMetadata};
use gunbc_ir::node::{Node, NodeBody};
use gunbc_ir::types::NodeId;

#[derive(Debug, Clone, Copy)]
enum InstallVariant {
    Install,
    Preview,
    FailIfMissing,
}

fn upsert_dag(
    name: &str,
    check_cmd: &'static str,
    install_cmd: &'static str,
    install_variant: InstallVariant,
) -> Dag<DepOp> {
    let check_id = format!("dep/{name}/check");
    let install_id = format!("dep/{name}/install");
    let resolve_id = format!("dep/{name}/resolve");

    let check = Node {
        id: NodeId(check_id.clone()),
        inputs: vec![],
        outputs: vec![port("present", "Bool"), port("needs_create", "Bool")],
        body: NodeBody::Opaque(DepOp::CheckCommand {
            name: name.to_string(),
            cmd: check_cmd,
        }),
    };

    let install_op = match install_variant {
        InstallVariant::Install => DepOp::InstallCommand {
            name: name.to_string(),
            cmd: CommandSpec::all(install_cmd),
        },
        InstallVariant::Preview => DepOp::PreviewInstall {
            name: name.to_string(),
            cmd: CommandSpec::all(install_cmd),
        },
        InstallVariant::FailIfMissing => DepOp::FailIfMissing {
            name: name.to_string(),
        },
    };

    let install = Node {
        id: NodeId(install_id.clone()),
        inputs: vec![eq_guarded_port("needs_create", "Bool", IrValue::Bool(true))],
        outputs: vec![port("installed", "Bool")],
        body: NodeBody::Opaque(install_op),
    };

    let resolve = Node {
        id: NodeId(resolve_id.clone()),
        inputs: vec![port("present", "Bool"), port("installed", "Bool")],
        outputs: vec![port("ok", "Bool")],
        body: NodeBody::Opaque(DepOp::ResolveUpsert {
            name: name.to_string(),
        }),
    };

    Dag {
        nodes: vec![check, install, resolve],
        edges: vec![
            edge(&check_id, "needs_create", &install_id, "needs_create"),
            edge(&check_id, "present", &resolve_id, "present"),
            edge(&install_id, "installed", &resolve_id, "installed"),
        ],
        metadata: DagMetadata::default(),
    }
}

fn output_for<'a>(log: &'a ExecutionLog, node_id: &str, port: &str) -> Option<&'a Value> {
    log.entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .and_then(|entry| entry.outputs.get(port))
}

#[test]
fn upsert_skips_install_when_present() {
    let dag = upsert_dag("demo", "true", "false", InstallVariant::Install);
    let log = execute(&dag).unwrap();

    let install = output_for(&log, "dep/demo/install", "installed")
        .expect("install output missing");
    assert!(matches!(install, Value::Skipped));

    let ok = output_for(&log, "dep/demo/resolve", "ok").expect("resolve output missing");
    assert!(matches!(ok, Value::Bool(true)));
}

#[test]
fn upsert_runs_install_when_missing() {
    let dag = upsert_dag("demo", "false", "true", InstallVariant::Install);
    let log = execute(&dag).unwrap();

    let install = output_for(&log, "dep/demo/install", "installed")
        .expect("install output missing");
    assert!(matches!(install, Value::Bool(true)));

    let ok = output_for(&log, "dep/demo/resolve", "ok").expect("resolve output missing");
    assert!(matches!(ok, Value::Bool(true)));
}

#[test]
fn check_mode_errors_when_missing() {
    let dag = upsert_dag("demo", "false", "true", InstallVariant::FailIfMissing);
    let err = execute(&dag).unwrap_err();
    let ExecError(msg) = err;
    assert!(msg.contains("installs are disabled"));
}

#[test]
fn upsert_ignores_deps_when_not_required() {
    let dag = upsert_dag("demo", "true", "true", InstallVariant::Preview);
    let log = execute(&dag).unwrap();

    let outputs: HashMap<_, _> = log
        .entries
        .iter()
        .map(|entry| (entry.node_id.as_str(), entry.outputs.clone()))
        .collect();
    assert!(matches!(
        outputs
            .get("dep/demo/resolve")
            .and_then(|out| out.get("ok")),
        Some(Value::Bool(true))
    ));
}
