use std::collections::HashSet;

use gunbc_ir::algebra::Value as IrValue;
use gunbc_ir::build::{edge, eq_guarded_port, port};
use gunbc_ir::dag::{Dag, DagMetadata, Port};
use gunbc_ir::node::{Node, NodeBody};
use gunbc_ir::types::NodeId;

use crate::ops::{CommandSpec, DepOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Upsert,
}

pub struct DepGraph {
    pub dag: Dag<DepOp>,
    pub entries: Vec<String>,
}

pub fn build_graph(mode: Mode) -> DepGraph {
    let mut builder = GraphBuilder::new();

    // Core deps
    // TODO(deps): add platform-specific package manager nodes (apt/brew/choco)
    // and route curl/zstd installs through them.
    let curl = builder.add_upsert(
        "curl",
        DepOp::CheckCommand {
            name: "curl".to_string(),
            cmd: "curl --version",
        },
        Some(CommandSpec {
            linux: Some("sudo apt-get update && sudo apt-get install -y curl"),
            // TODO(deps): add brew install path for curl.
            macos: Some("brew install curl"),
            // TODO(deps): add choco install path for curl.
            windows: Some("choco install -y curl"),
        }),
        &[],
        mode,
    );

    let zstd = builder.add_upsert(
        "zstd",
        DepOp::CheckCommand {
            name: "zstd".to_string(),
            cmd: "zstd --version",
        },
        Some(CommandSpec {
            linux: Some("sudo apt-get update && sudo apt-get install -y zstd"),
            // TODO(deps): add brew install path for zstd.
            macos: Some("brew install zstd"),
            // TODO(deps): add choco install path for zstd.
            windows: Some("choco install -y zstandard"),
        }),
        &[],
        mode,
    );

    let rustup = builder.add_upsert(
        "rustup",
        DepOp::CheckCommand {
            name: "rustup".to_string(),
            cmd: "rustup --version",
        },
        Some(CommandSpec {
            linux: Some(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
            ),
            macos: Some(
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
            ),
            // TODO(deps): add rustup installer for Windows (powershell / rustup-init.exe).
            windows: None,
        }),
        &[curl.clone()],
        mode,
    );

    let rust = builder.add_upsert(
        "rust",
        DepOp::CheckCommand {
            name: "rust".to_string(),
            cmd: "rustup show active-toolchain",
        },
        Some(CommandSpec {
            linux: Some("rustup toolchain install stable && rustup default stable"),
            macos: Some("rustup toolchain install stable && rustup default stable"),
            // TODO(deps): add windows toolchain install + default selection.
            windows: None,
        }),
        &[rustup.clone()],
        mode,
    );

    let buck2 = builder.add_upsert(
        "buck2",
        DepOp::CheckCommand {
            name: "buck2".to_string(),
            cmd: "buck2 --version",
        },
        Some(CommandSpec {
            linux: Some(
                "ARCH=$(uname -m); if [ \"$ARCH\" != \"x86_64\" ]; then echo 'unsupported arch' >&2; exit 1; fi; \
                URL=\"https://github.com/facebook/buck2/releases/latest/download/buck2-x86_64-unknown-linux-gnu.zst\"; \
                TMP=\"/tmp/buck2.zst\"; OUT=\"/tmp/buck2\"; \
                curl -L \"$URL\" -o \"$TMP\" && zstd -d \"$TMP\" -o \"$OUT\" && chmod +x \"$OUT\" && \
                mkdir -p \"$HOME/.local/bin\" && mv \"$OUT\" \"$HOME/.local/bin/buck2\"",
            ),
            // TODO(deps): add macOS buck2 install (brew or release download).
            macos: None,
            // TODO(deps): add Windows buck2 install (release download + unpack).
            windows: None,
        }),
        &[curl.clone(), zstd.clone()],
        mode,
    );

    let vendor = builder.add_upsert(
        "vendor",
        DepOp::CheckPath {
            name: "vendor".to_string(),
            path: "vendor",
        },
        Some(CommandSpec::all("cargo vendor --locked vendor")),
        &[rust.clone()],
        mode,
    );

    let buck_bootstrap = builder.add_gate(
        "buck_bootstrap",
        &[curl.clone(), zstd.clone(), rust.clone(), buck2.clone(), vendor.clone()],
    );
    let buck_test = builder.add_gate("buck_test", &[rust, buck2, vendor]);

    let mut entries = vec![buck_bootstrap.clone(), buck_test.clone()];
    entries.sort();

    DepGraph {
        dag: builder.finish(),
        entries,
    }
}

pub fn subdag_for_entry(dag: &Dag<DepOp>, entry: &str) -> Result<Dag<DepOp>, String> {
    let mut keep: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = vec![entry.to_string()];

    while let Some(node) = stack.pop() {
        if !keep.insert(node.clone()) {
            continue;
        }
        for edge in &dag.edges {
            if edge.to_node.0 == node {
                stack.push(edge.from_node.0.clone());
            }
        }
    }

    let nodes: Vec<Node<DepOp>> = dag
        .nodes
        .iter()
        .filter(|n| keep.contains(&n.id.0))
        .cloned()
        .collect();

    if nodes.iter().all(|n| n.id.0 != entry) {
        return Err(format!("unknown entry node '{entry}'"));
    }

    let edges = dag
        .edges
        .iter()
        .filter(|e| keep.contains(&e.from_node.0) && keep.contains(&e.to_node.0))
        .cloned()
        .collect();

    Ok(Dag {
        nodes,
        edges,
        metadata: DagMetadata::default(),
    })
}

struct GraphBuilder {
    nodes: Vec<Node<DepOp>>,
    edges: Vec<gunbc_ir::dag::Edge>,
}

#[derive(Debug, Clone)]
struct DepRef {
    name: &'static str,
    ok_node: String,
}

impl GraphBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn finish(self) -> Dag<DepOp> {
        Dag {
            nodes: self.nodes,
            edges: self.edges,
            metadata: DagMetadata::default(),
        }
    }

    fn add_node(&mut self, id: &str, inputs: Vec<Port>, outputs: Vec<Port>, body: DepOp) {
        self.nodes.push(Node {
            id: NodeId(id.to_string()),
            inputs,
            outputs,
            body: NodeBody::Opaque(body),
        });
    }

    fn add_edge(&mut self, from: &str, from_port: &str, to: &str, to_port: &str) {
        self.edges.push(edge(from, from_port, to, to_port));
    }

    fn add_gate(&mut self, id: &str, deps: &[DepRef]) -> String {
        let inputs: Vec<Port> = deps
            .iter()
            .map(|dep| port(&format!("{}_ok", dep.name), "Bool"))
            .collect();
        let outputs = vec![port("ok", "Bool")];

        self.add_node(
            id,
            inputs,
            outputs,
            DepOp::Gate { name: id.to_string() },
        );

        for dep in deps {
            self.add_edge(&dep.ok_node, "ok", id, &format!("{}_ok", dep.name));
        }

        id.to_string()
    }

    fn add_upsert(
        &mut self,
        name: &'static str,
        check: DepOp,
        install: Option<CommandSpec>,
        deps: &[DepRef],
        mode: Mode,
    ) -> DepRef {
        let check_id = format!("dep/{name}/check");
        let install_id = format!("dep/{name}/install");
        let resolve_id = format!("dep/{name}/resolve");

        self.add_node(
            &check_id,
            vec![],
            vec![port("present", "Bool"), port("needs_create", "Bool")],
            check,
        );

        let mut install_inputs = vec![eq_guarded_port("needs_create", "Bool", IrValue::Bool(true))];
        if !deps.is_empty() {
            install_inputs.push(port("deps_ok", "Bool"));
        }

        let install_cmd = install.unwrap_or_else(|| CommandSpec::all("false"));

        self.add_node(
            &install_id,
            install_inputs,
            vec![port("installed", "Bool")],
            DepOp::InstallCommand {
                name: name.to_string(),
                cmd: install_cmd,
                allow_install: mode == Mode::Upsert,
            },
        );

        self.add_node(
            &resolve_id,
            vec![port("present", "Bool"), port("installed", "Bool")],
            vec![port("ok", "Bool")],
            DepOp::ResolveUpsert {
                name: name.to_string(),
            },
        );

        self.add_edge(&check_id, "needs_create", &install_id, "needs_create");
        self.add_edge(&check_id, "present", &resolve_id, "present");
        self.add_edge(&install_id, "installed", &resolve_id, "installed");

        if !deps.is_empty() {
            let gate_id = format!("dep/{name}/deps");
            let gate = self.add_gate(&gate_id, deps);
            self.add_edge(&gate, "ok", &install_id, "deps_ok");
        }

        DepRef {
            name,
            ok_node: resolve_id,
        }
    }
}
