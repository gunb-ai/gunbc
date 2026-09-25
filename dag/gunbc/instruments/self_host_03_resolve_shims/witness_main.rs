use im::vector as vec;
use std::rc::Rc;

use v1_compiled::v2_compiler_normalized_tree::admit_unmarked_normalized_roots;
use v1_compiled::v2_compiler_resolve as emitted;
use v1_compiled::v2_extdeps_languages_dag::dag_language_model;
use v1_compiled::v2_std_node::{node_synthetic, Behavior, Edge, EdgeLabel, Node, NodeKind};
use v1_compiled::v2_std_diagnostic::Outcome;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// arrow_domain_param_scoping_note against a hand copy in v1_compiler::v2_compiler_resolve, which
// said nothing about what resolve does (and did not compile: its format string had two holes and
// one argument). Its expected verdicts are read off the authority instead: src/v2/compiler/
// 03_resolve.dag resolve_walk_with_namespace_policy walks a tree v2.std.node well_formed admits
// and refuses one it rejects (malformed_tree_diagnostic).

fn value_node() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![]),
    )
}

// v2.std.node behavior_edges_conform gives Value `count(children) == 0`.
fn value_with_child() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![Rc::new(Edge {
            label: Rc::new(EdgeLabel::Positional),
            target: value_node(),
        })]),
    )
}

// admit_unmarked_normalized_roots does not re-check well_formed, so the malformed root reaches
// resolve's own gate, which is the contract under test.
fn resolve_accepts(root: Rc<Node>) -> bool {
    let trees = match &*admit_unmarked_normalized_roots(Rc::new(vec![root]), None) {
        Outcome::Accepted { value, .. } => value.clone(),
        Outcome::Rejected { .. } => panic!("admit_unmarked_normalized_roots refused a root"),
    };
    let tree = trees.head().expect("one normalized root").clone();
    matches!(
        &*emitted::resolve(tree, dag_language_model()),
        Outcome::Accepted { .. }
    )
}

// The planted fault feeds the malformed tree where the well-formed one is expected, so the fault
// run goes red only if emitted resolve really refuses it.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let valid_input = if inject_fault {
        value_with_child()
    } else {
        value_node()
    };
    let valid_accepts = resolve_accepts(valid_input);
    println!("resolve well_formed inject_fault={inject_fault} accepts={valid_accepts}");
    let malformed_refuses = !resolve_accepts(value_with_child());
    println!("resolve not_well_formed refuses={malformed_refuses}");
    if valid_accepts && malformed_refuses {
        println!("SELF_HOST_03_RESOLVE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_03_RESOLVE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
