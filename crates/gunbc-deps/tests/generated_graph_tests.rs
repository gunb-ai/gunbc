use std::collections::{BTreeMap, BTreeSet};

use gunbc_deps::{build_graph, subdag_for_entry, Mode};
use gunbc_deps::ops::DepOp;
use gunbc_ir::algebra::{Predicate, Value as IrValue};
use gunbc_ir::node::NodeBody;
use gunbc_validate::{validate_acyclic, validate_port_saturation, validate_types};

fn upsert_stage_map() -> BTreeMap<String, BTreeSet<String>> {
    let graph = build_graph(Mode::Check, false);
    let mut stages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for node in &graph.dag.nodes {
        let id = node.id.0.as_str();
        let rest = match id.strip_prefix("dep/") {
            Some(rest) => rest,
            None => continue,
        };
        let (name, stage) = match rest.rsplit_once('/') {
            Some(parts) => parts,
            None => continue,
        };
        stages
            .entry(name.to_string())
            .or_default()
            .insert(stage.to_string());
    }

    stages
}

#[test]
fn generated_upsert_nodes_have_check_install_resolve() {
    let stages = upsert_stage_map();

    for (name, stage_set) in stages {
        assert!(
            stage_set.contains("check"),
            "dep/{name} missing check node"
        );
        assert!(
            stage_set.contains("install"),
            "dep/{name} missing install node"
        );
        assert!(
            stage_set.contains("resolve"),
            "dep/{name} missing resolve node"
        );
    }
}

#[test]
fn install_nodes_guard_needs_create_true() {
    let graph = build_graph(Mode::Check, false);

    for node in &graph.dag.nodes {
        if !node.id.0.ends_with("/install") {
            continue;
        }
        let needs_create = node
            .inputs
            .iter()
            .find(|port| port.name.0 == "needs_create")
            .unwrap_or_else(|| panic!("{} missing needs_create input", node.id.0));
        assert_eq!(
            needs_create.guard,
            Some(Predicate::Eq(IrValue::Bool(true))),
            "{} needs_create guard should be == true",
            node.id.0
        );
    }
}

#[test]
fn install_nodes_have_all_platform_commands() {
    let graph = build_graph(Mode::Upsert, false);

    for node in &graph.dag.nodes {
        let op = match &node.body {
            NodeBody::Opaque(op) => op,
            _ => continue,
        };
        let install = match op {
            DepOp::InstallCommand { cmd, .. } | DepOp::PreviewInstall { cmd, .. } => cmd,
            _ => continue,
        };
        assert!(
            install.linux.is_some(),
            "{} missing linux install command",
            node.id.0
        );
        assert!(
            install.macos.is_some(),
            "{} missing macos install command",
            node.id.0
        );
        assert!(
            install.windows.is_some(),
            "{} missing windows install command",
            node.id.0
        );
    }
}

#[test]
fn entry_subdags_validate() {
    let graph = build_graph(Mode::Check, false);

    for entry in &graph.entries {
        let dag = subdag_for_entry(&graph.dag, entry).unwrap();
        validate_acyclic(&dag).unwrap();
        validate_types(&dag).unwrap();
        validate_port_saturation(&dag).unwrap();
    }
}
