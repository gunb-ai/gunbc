use im::vector as vec;
use std::rc::Rc;

use v1_compiled::v2_compiler_normalize as emitted;
use v1_compiled::v2_std_node::{
    node_synthetic, Behavior, Edge, EdgeLabel, Node, NodeKind,
};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare against
// v1_compiler::v2_compiler_normalize, a hand-retained stub whose well_formed returned `true`, so
// "parity with the seed" meant "accepts everything". Its expected verdicts are instead read off
// the authority itself: src/v2/compiler/03_normalize.dag refuses a tree v2.std.node well_formed
// rejects (post_normalize_not_well_formed_diagnostic) and accepts one it admits.

fn accepted<T>(o: &Rc<emitted::Outcome<T>>) -> bool {
    matches!(&**o, emitted::Outcome::Accepted { .. })
}

fn value_node() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![]),
    )
}

// v2.std.node behavior_edges_conform gives Value `count(children) == 0`, so a Value node carrying
// one edge is not well formed.
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

// The planted fault feeds the malformed tree where the well-formed one is expected, so the fault
// run goes red only if emitted normalize really refuses it; a normalize that accepted everything
// would stay green there, and cssl_fault_run_detected_the_planted_fault would refuse the receipt.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let valid_input = if inject_fault {
        value_with_child()
    } else {
        value_node()
    };
    let valid_accepts = accepted(&emitted::normalize(valid_input));
    println!("normalize well_formed inject_fault={inject_fault} accepts={valid_accepts}");
    all_pass &= valid_accepts;

    let malformed_refuses = !accepted(&emitted::normalize(value_with_child()));
    println!("normalize not_well_formed refuses={malformed_refuses}");
    all_pass &= malformed_refuses;

    if all_pass {
        println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
