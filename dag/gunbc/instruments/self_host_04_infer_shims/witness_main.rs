use im::vector as vec;
use std::rc::Rc;

use v1_compiled::v2_compiler_infer as emitted;
use v1_compiled::v2_std_diagnostic::Outcome;
use v1_compiled::v2_std_node::{node_synthetic, Behavior, Connective, Node, NodeKind};
use v1_compiled::v2_std_witness::Witness;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// infer_arrow_domain_binding_heuristic_note against a hand copy in v1_compiler; the .dag no longer
// declares that constant, so the driver named a symbol the emitted module does not have. Its
// expected verdicts are read off the authority instead: src/v2/compiler/04_infer.dag infer
// records facts for the nodes of the tree it is given, and inferred_facts_witness_for_node
// answers Holds for such a node and refuses (infer_facts_lookup_miss) a node the tree does not
// contain.

fn value_node() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![]),
    )
}

// Structurally distinct from every node of the inferred tree (facts are keyed by node value).
fn foreign_node() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Conj),
        }),
        Rc::new(vec![]),
    )
}

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): the injected run is
// PASS exactly when the emitted module accepts what the .dag refuses, so a correct module reds it
// and a module that wrongly accepts greens it -- which the harness then rejects.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let tree = match &*emitted::infer(value_node()) {
        Outcome::Accepted { value, .. } => value.clone(),
        Outcome::Rejected { diagnostics } => {
            let shown: String = format!("{:?}", diagnostics).chars().take(400).collect();
            println!("infer refused a well-formed Value tree: {shown}");
            println!("SELF_HOST_04_INFER_BEHAVIORAL_RECEIPT: FAIL");
            std::process::exit(1);
        }
    };
    let foreign_held = matches!(
        &*emitted::inferred_facts_witness_for_node(tree.clone(), foreign_node()),
        Witness::Holds { .. }
    );
    let all_pass = if inject_fault {
        println!("infer injected: foreign node held={foreign_held}");
        foreign_held
    } else {
        let covered_holds = matches!(
            &*emitted::inferred_facts_witness_for_node(tree, value_node()),
            Witness::Holds { .. }
        );
        println!("infer covers root holds={covered_holds}");
        println!("infer foreign node refused={}", !foreign_held);
        covered_holds && !foreign_held
    };

    if all_pass {
        println!("SELF_HOST_04_INFER_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_04_INFER_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
