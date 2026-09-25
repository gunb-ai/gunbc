use im::{vector as vec, Vector as Vec};
use std::rc::Rc;
use v1_compiled::v2_compiler_body_producer as emitted;
use v1_compiled::v2_std_node::{
    node_synthetic as emitted_node_synthetic, Behavior as EBehavior, Connective as EConnective,
    Edge as EEdge, EdgeLabel as EEdgeLabel, Node as ENode, NodeKind as ENodeKind,
};
use v1_compiler::v2_compiler_body_producer as seed;
use v1_compiler::usv_pilot_v2_std_node::{
    node_synthetic as seed_node_synthetic, Behavior as SBehavior, Connective as SConnective,
    Edge as SEdge, EdgeLabel as SEdgeLabel, Node as SNode, NodeKind as SNodeKind,
};

fn outcome_is_accepted_emitted<T>(o: &Rc<emitted::Outcome<T>>) -> bool {
    matches!(&**o, emitted::Outcome::Accepted { .. })
}

fn outcome_is_accepted_seed<T>(o: &Rc<seed::Outcome<T>>) -> bool {
    matches!(&**o, seed::Outcome::Accepted { .. })
}

fn transform_body_emitted() -> Rc<ENode> {
    emitted_node_synthetic(
        Rc::new(ENodeKind::ComputationNode {
            behavior: EBehavior::Transform,
        }),
        Rc::new(vec![Rc::new(EEdge {
            label: Rc::new(EEdgeLabel::Positional),
            target: emitted_node_synthetic(
                Rc::new(ENodeKind::ComputationNode {
                    behavior: EBehavior::Value,
                }),
                Rc::new(vec![]),
            ),
        })]),
    )
}

fn transform_body_seed() -> Rc<SNode> {
    seed_node_synthetic(
        Rc::new(SNodeKind::ComputationNode {
            behavior: SBehavior::Transform,
        }),
        Rc::new(vec![Rc::new(SEdge {
            label: Rc::new(SEdgeLabel::Positional),
            target: seed_node_synthetic(
                Rc::new(SNodeKind::ComputationNode {
                    behavior: SBehavior::Value,
                }),
                Rc::new(vec![]),
            ),
        })]),
    )
}

fn atom_body_emitted() -> Rc<ENode> {
    emitted_node_synthetic(
        Rc::new(ENodeKind::TypeNode {
            connective: Rc::new(EConnective::Atom {
                identity: "residual".to_string(),
            }),
        }),
        Rc::new(vec![]),
    )
}

fn atom_body_seed() -> Rc<SNode> {
    seed_node_synthetic(
        Rc::new(SNodeKind::TypeNode {
            connective: Rc::new(SConnective::Atom {
                identity: "residual".to_string(),
            }),
        }),
        Rc::new(vec![]),
    )
}

fn arrow_signature_emitted() -> Rc<ENode> {
    emitted_node_synthetic(
        Rc::new(ENodeKind::TypeNode {
            connective: Rc::new(EConnective::Arrow),
        }),
        Rc::new(vec![]),
    )
}

fn arrow_signature_seed() -> Rc<SNode> {
    seed_node_synthetic(
        Rc::new(SNodeKind::TypeNode {
            connective: Rc::new(SConnective::Arrow),
        }),
        Rc::new(vec![]),
    )
}

// The Transform fixture carries one positional edge because v2.std.node behavior_edges_conform
// requires `count(children) >= 1` for Transform. It carried none while this row linked a hand
// v2_std_node shim laxer than that authority; against the emitted module the edgeless fixture is
// (correctly) refused, so the shim had been keeping a malformed input green.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let e_dispatch = emitted::body_producer_dispatch_structured_body(transform_body_emitted());
    let s_dispatch = seed::body_producer_dispatch_structured_body(transform_body_seed());
    let dispatch_ok =
        outcome_is_accepted_emitted(&e_dispatch) && outcome_is_accepted_seed(&s_dispatch);
    println!("dispatch_transform eq_accept={dispatch_ok}");
    all_pass &= dispatch_ok;

    let e_reject = emitted::body_producer_dispatch_structured_body(atom_body_emitted());
    let s_reject = seed::body_producer_dispatch_structured_body(atom_body_seed());
    let reject_ok =
        !outcome_is_accepted_emitted(&e_reject) && !outcome_is_accepted_seed(&s_reject);
    println!("dispatch_non_behavior eq_reject={reject_ok}");
    all_pass &= reject_ok;

    // THE FAULT RUN INVERTS EXACTLY ONE PROPOSITION: the planted claim is that emitted
    // produce_arrow ACCEPTS an Atom body. A correct producer refuses it, so the injected run
    // fails (the harness reads that as the fault detected); an accept-everything producer makes
    // the injected run PASS, and cssl_fault_run_detected_the_planted_fault refuses. An earlier
    // revision required `parity && !accepted` there, which no implementation could satisfy.
    let produce_ok = if inject_fault {
        let e_atom = emitted::produce_arrow_with_structured_body(
            arrow_signature_emitted(),
            atom_body_emitted(),
        );
        outcome_is_accepted_emitted(&e_atom)
    } else {
        let e_produce = emitted::produce_arrow_with_structured_body(
            arrow_signature_emitted(),
            transform_body_emitted(),
        );
        let s_produce = seed::produce_arrow_with_structured_body(
            arrow_signature_seed(),
            transform_body_seed(),
        );
        outcome_is_accepted_emitted(&e_produce) && outcome_is_accepted_seed(&s_produce)
    };
    println!("produce_arrow inject_fault={inject_fault} ok={produce_ok}");
    all_pass = if inject_fault { produce_ok } else { all_pass && produce_ok };

    if all_pass {
        println!("SELF_HOST_BODY_PRODUCER_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_BODY_PRODUCER_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
