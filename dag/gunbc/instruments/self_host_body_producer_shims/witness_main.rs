use im::vector as vec;
use std::rc::Rc;
use v1_compiled::v2_compiler_body_producer as emitted;
use v1_compiled::v2_std_node::{node_synthetic, Behavior, Connective, Edge, EdgeLabel, Node, NodeKind};

// NO SEED ORACLE, ON PURPOSE. This driver used to require the same verdicts from a hand copy in
// v1_compiler::v2_compiler_body_producer, which asserted only that two realizations agreed. The
// expected verdicts are read off the authority instead: src/v2/compiler/03_body_producer.dag
// body_producer_dispatch_structured_body accepts a behavior-rooted body (Transform) and refuses a
// type-rooted one (an Atom is not a computation), and produce_arrow_with_structured_body inherits
// that refusal.

fn accepted<T>(o: &Rc<emitted::Outcome<T>>) -> bool {
    matches!(&**o, emitted::Outcome::Accepted { .. })
}

// The Transform fixture carries one positional edge because v2.std.node behavior_edges_conform
// requires `count(children) >= 1` for Transform.
fn transform_body() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Transform,
        }),
        Rc::new(vec![Rc::new(Edge {
            label: Rc::new(EdgeLabel::Positional),
            target: node_synthetic(
                Rc::new(NodeKind::ComputationNode {
                    behavior: Behavior::Value,
                }),
                Rc::new(vec![]),
            ),
        })]),
    )
}

fn atom_body() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Atom {
                identity: "residual".to_string(),
            }),
        }),
        Rc::new(vec![]),
    )
}

fn arrow_signature() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Arrow),
        }),
        Rc::new(vec![]),
    )
}

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): emitted
// produce_arrow ACCEPTS an Atom body. A correct producer refuses it, so the injected run is red;
// an accept-everything producer greens it, which cssl_fault_run_detected_the_planted_fault refuses.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let atom_produced = accepted(&emitted::produce_arrow_with_structured_body(
        arrow_signature(),
        atom_body(),
    ));
    let all_pass = if inject_fault {
        println!("body_producer injected: produce_arrow atom accepted={atom_produced}");
        atom_produced
    } else {
        let dispatch_accepts =
            accepted(&emitted::body_producer_dispatch_structured_body(transform_body()));
        let dispatch_refuses =
            !accepted(&emitted::body_producer_dispatch_structured_body(atom_body()));
        let produce_accepts = accepted(&emitted::produce_arrow_with_structured_body(
            arrow_signature(),
            transform_body(),
        ));
        println!("dispatch transform accepts={dispatch_accepts}");
        println!("dispatch atom refuses={dispatch_refuses}");
        println!("produce_arrow transform accepts={produce_accepts}");
        println!("produce_arrow atom refuses={}", !atom_produced);
        dispatch_accepts && dispatch_refuses && produce_accepts && !atom_produced
    };

    if all_pass {
        println!("SELF_HOST_BODY_PRODUCER_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_BODY_PRODUCER_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
