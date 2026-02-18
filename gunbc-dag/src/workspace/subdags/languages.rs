//! Languages SubDag builder.
//!
//! Wraps the languages DAG as a SubDag node using `DynOp`.

use crate::workspace::convert::convert_node;
use crate::workspace::WorkspaceOp;
use gunbc_exec::{DynOp, ExecError, Executable};
use gunbc_ir::language::{
    build_comment_prefix_subdag, build_config_format_subdag, build_gitignore_subdag,
    build_glob_subdag, build_makefile_subdag, build_naming_conventions_subdag, build_regex_subdag,
    build_rust_subdag, build_turing_complete_subdag, build_type_system_mapping_subdag,
    build_variable_syntax_subdag, LanguageOp,
};
use gunbc_ir::{Dag, Node, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct LanguageExecOp {
    #[allow(dead_code)]
    inner: LanguageOp,
}

impl Executable for LanguageExecOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        Ok(HashMap::new())
    }
}

fn convert_language_op(op: LanguageOp) -> WorkspaceOp {
    DynOp::new(LanguageExecOp { inner: op })
}

/// Build the languages SubDag node.
pub fn build_languages_subdag() -> Node<WorkspaceOp> {
    let mut inner: Dag<WorkspaceOp> = Dag::new();

    // Pattern SubDags (foundations)
    inner.add_node(convert_node(build_regex_subdag(), &convert_language_op));
    inner.add_node(convert_node(build_glob_subdag(), &convert_language_op));
    inner.add_node(convert_node(
        build_variable_syntax_subdag(),
        &convert_language_op,
    ));

    // Trait SubDags (composable characteristics)
    inner.add_node(convert_node(
        build_type_system_mapping_subdag(),
        &convert_language_op,
    ));
    inner.add_node(convert_node(
        build_naming_conventions_subdag(),
        &convert_language_op,
    ));
    inner.add_node(convert_node(
        build_comment_prefix_subdag(),
        &convert_language_op,
    ));

    // Category SubDags
    inner.add_node(convert_node(
        build_turing_complete_subdag(),
        &convert_language_op,
    ));
    inner.add_node(convert_node(
        build_config_format_subdag(),
        &convert_language_op,
    ));

    // Language/Format SubDags
    inner.add_node(convert_node(build_rust_subdag(), &convert_language_op));
    inner.add_node(convert_node(build_gitignore_subdag(), &convert_language_op));
    inner.add_node(convert_node(build_makefile_subdag(), &convert_language_op));

    Node::subdag("languages", inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::NodeBody;

    #[test]
    fn test_languages_subdag_is_subdag() {
        let node = build_languages_subdag();
        assert!(node.is_subdag());
        assert_eq!(node.id.0, "languages");
    }

    #[test]
    fn test_languages_subdag_contains_all_subdags() {
        let node = build_languages_subdag();

        match &node.body {
            NodeBody::SubDag(dag) => {
                let node_ids: Vec<_> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();

                // Pattern SubDags
                assert!(node_ids.contains(&"regex"));
                assert!(node_ids.contains(&"glob"));
                assert!(node_ids.contains(&"variable_syntax"));

                // Trait SubDags
                assert!(node_ids.contains(&"type_system"));
                assert!(node_ids.contains(&"naming"));
                assert!(node_ids.contains(&"comment_prefix"));

                // Category SubDags
                assert!(node_ids.contains(&"turing_complete"));
                assert!(node_ids.contains(&"config_format"));

                // Language SubDags
                assert!(node_ids.contains(&"rust"));
                assert!(node_ids.contains(&"gitignore"));
                assert!(node_ids.contains(&"makefile"));
            }
            _ => panic!("Expected SubDag"),
        }
    }
}
