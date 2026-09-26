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

// THE FAULT RUN INVERTS EXACTLY ONE PROPOSITION. Plain: the well-formed tree is accepted AND the
// malformed one is refused. Injected: the malformed tree is ACCEPTED -- the planted fault is the claim
// that normalize admits it. So a correct normalize passes plain and fails injected (the harness
// reads that FAIL as the fault detected); an accept-everything normalize fails plain and PASSES
// injected, so cssl_fault_run_detected_the_planted_fault refuses; a reject-everything normalize
// fails plain. An earlier revision fed the malformed tree into both arms, so the injected run was
// `p && !p` and printed FAIL for every implementation -- a fault arm with no reachable green.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let malformed_accepted = accepted(&emitted::normalize(value_with_child()));
    let all_pass = if inject_fault {
        println!("normalize injected: not_well_formed accepted={malformed_accepted}");
        malformed_accepted
    } else {
        let valid_accepts = accepted(&emitted::normalize(value_node()));
        println!("normalize well_formed accepts={valid_accepts}");
        println!("normalize not_well_formed refuses={}", !malformed_accepted);
        valid_accepts && !malformed_accepted
    };

    if all_pass {
        println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
