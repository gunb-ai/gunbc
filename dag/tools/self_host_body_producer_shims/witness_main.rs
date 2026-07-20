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
        Rc::new(vec![]),
    )
}

fn transform_body_seed() -> Rc<SNode> {
    seed_node_synthetic(
        Rc::new(SNodeKind::ComputationNode {
            behavior: SBehavior::Transform,
        }),
        Rc::new(vec![]),
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

    let e_produce = emitted::produce_arrow_with_structured_body(
        arrow_signature_emitted(),
        if inject_fault {
            atom_body_emitted()
        } else {
            transform_body_emitted()
        },
    );
    let s_produce = seed::produce_arrow_with_structured_body(
        arrow_signature_seed(),
        transform_body_seed(),
    );
    let produce_ok = outcome_is_accepted_emitted(&e_produce)
        == outcome_is_accepted_seed(&s_produce)
        && (!inject_fault || !outcome_is_accepted_emitted(&e_produce));
    println!(
        "produce_arrow inject_fault={inject_fault} parity={produce_ok}"
    );
    all_pass &= produce_ok;

    if all_pass {
        println!("SELF_HOST_BODY_PRODUCER_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_BODY_PRODUCER_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
