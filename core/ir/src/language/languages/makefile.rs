//! Makefile SubDag: Makefile format definition.
//!
//! # Composes
//!
//! - ConfigFormat (via category)
//! - VariableSyntax (for $(VAR) expansion)
//!
//! # Configuration
//!
//! - File patterns: `Makefile`, `*.mk`, `GNUmakefile`
//! - Comment prefix: `#`
//! - Indentation: TAB (required by Make)
//! - Variable syntax: $(VAR), ${VAR}, $@, $<, $^

use crate::dag::{Dag, Port};
use crate::language::LanguageOp;
use crate::node::Node;

/// Default Makefile filename - the canonical name for generated Makefiles.
pub const DEFAULT_MAKEFILE_FILENAME: &str = "Makefile";

/// Makefile format static configuration.
pub struct MakefileConfig {
    pub id: &'static str,
    /// The default filename for generated Makefiles.
    pub default_filename: &'static str,
    /// All file patterns that identify Makefiles.
    pub file_patterns: &'static [&'static str],
    pub comment_prefix: &'static str,
    pub indent: &'static str,
}

/// Static Makefile configuration.
pub const MAKEFILE: MakefileConfig = MakefileConfig {
    id: "makefile",
    default_filename: DEFAULT_MAKEFILE_FILENAME,
    file_patterns: &["Makefile", "*.mk", "GNUmakefile"],
    comment_prefix: "#",
    indent: "\t", // Makefiles require tabs!
};

/// Build the Makefile format SubDag node.
///
/// This SubDag composes ConfigFormat category and VariableSyntax,
/// providing Makefile-specific target and variable handling.
///
/// # I/O Contract
///
/// Inputs:
/// - `targets`: List of target definitions
/// - `variables`: Variable definitions (Map)
///
/// Outputs:
/// - `id`: String - Format ID ("makefile")
/// - `content`: String - Rendered Makefile content
pub fn build_makefile_subdag() -> Node<LanguageOp> {
    let mut inner = Dag::new();

    // Makefile configuration node
    inner.add_node(Node::opaque(
        "config",
        vec![],
        vec![
            Port::scalar("id", "String"),
            Port::list("file_patterns", "String"),
            Port::scalar("comment_prefix", "String"),
            Port::scalar("indent", "String"),
        ],
        LanguageOp::MakefileConfig,
    ));

    // Render targets to content
    inner.add_node(Node::opaque(
        "render",
        vec![
            Port::scalar("targets", "Json"), // List of target definitions
            Port::scalar("variables", "Map"),
        ],
        vec![Port::scalar("content", "String")],
        LanguageOp::MakefileRender,
    ));

    // Create the SubDag node with interface
    Node::subdag("makefile", inner)
}

/// A Makefile target definition.
#[derive(Debug, Clone)]
pub struct MakeTarget {
    pub name: String,
    pub dependencies: Vec<String>,
    pub commands: Vec<String>,
    pub phony: bool,
}

impl MakeTarget {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dependencies: Vec::new(),
            commands: Vec::new(),
            phony: false,
        }
    }

    pub fn depends_on(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.commands.push(cmd.into());
        self
    }

    pub fn phony(mut self) -> Self {
        self.phony = true;
        self
    }
}

/// Render a Makefile target.
#[cfg(test)]
pub fn render_target(target: &MakeTarget) -> String {
    let mut output = String::new();

    // Target line
    output.push_str(&target.name);
    output.push(':');

    if !target.dependencies.is_empty() {
        output.push(' ');
        output.push_str(&target.dependencies.join(" "));
    }

    output.push('\n');

    // Commands (must be tab-indented)
    for cmd in &target.commands {
        output.push('\t');
        output.push_str(cmd);
        output.push('\n');
    }

    output
}

/// Render multiple targets with .PHONY declaration.
#[cfg(test)]
pub fn render_targets(targets: &[MakeTarget]) -> String {
    let mut output = String::new();

    // Collect phony targets
    let phony_targets: Vec<_> = targets
        .iter()
        .filter(|t| t.phony)
        .map(|t| t.name.as_str())
        .collect();

    if !phony_targets.is_empty() {
        output.push_str(".PHONY: ");
        output.push_str(&phony_targets.join(" "));
        output.push_str("\n\n");
    }

    // Render each target
    for target in targets {
        output.push_str(&render_target(target));
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeBody;

    #[test]
    fn test_makefile_subdag_is_subdag() {
        let node = build_makefile_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "makefile");
    }

    #[test]
    fn test_makefile_subdag_interface() {
        let node = build_makefile_subdag();

        // Check inputs
        assert!(node.inputs.iter().any(|p| p.name.0 == "targets"));
        assert!(node.inputs.iter().any(|p| p.name.0 == "variables"));

        // Check outputs
        assert!(node.outputs.iter().any(|p| p.name.0 == "id"));
        assert!(node.outputs.iter().any(|p| p.name.0 == "content"));
    }

    #[test]
    fn test_makefile_subdag_structure() {
        let node = build_makefile_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                assert_eq!(dag.nodes.len(), 2);

                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
                assert!(node_ids.contains(&"config"));
                assert!(node_ids.contains(&"render"));
            }
            _ => panic!("Expected SubDag"),
        }
    }

    #[test]
    fn test_render_target() {
        let target = MakeTarget::new("build")
            .depends_on("src/main.rs")
            .command("@cargo build --release");

        let output = render_target(&target);
        assert!(output.contains("build: src/main.rs"));
        assert!(output.contains("\t@cargo build --release"));
    }

    #[test]
    fn test_render_targets_with_phony() {
        let targets = vec![
            MakeTarget::new("clean").phony().command("rm -rf target"),
            MakeTarget::new("test").phony().command("cargo test"),
        ];

        let output = render_targets(&targets);
        assert!(output.contains(".PHONY: clean test"));
        assert!(output.contains("clean:"));
        assert!(output.contains("test:"));
    }

    #[test]
    fn test_makefile_config() {
        assert_eq!(MAKEFILE.id, "makefile");
        assert_eq!(MAKEFILE.comment_prefix, "#");
        assert_eq!(MAKEFILE.indent, "\t");
        assert!(MAKEFILE.file_patterns.contains(&"Makefile"));
    }
}
