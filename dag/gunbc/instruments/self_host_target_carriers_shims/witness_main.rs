use std::rc::Rc;

use v1_compiled::extdeps_communication_medium::DecodeFidelity;
use v1_compiled::v2_compiler_target_carriers as emitted;
use v1_compiled::v2_std_diagnostic::Outcome;
use v1_compiled::v2_std_node::{node_synthetic, Behavior, Connective, Edge, EdgeLabel, Node, NodeKind};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare emitted lossless_source / source_medium
// against v1_compiler::v2_compiler_target_carriers, a hand copy, over a FreeMonoid<Char> carrier
// the emitter no longer produces (FreeMonoid is emitted as im::Vector, and Medium carries String),
// so it failed to compile against the real crate. Its expected verdicts are read off the authority
// instead: src/v2/compiler/07_target_carriers.dag fidelity_quotient_decode_fidelity folds a target's
// fidelity quotient to Lossless when every disposition is the modeled kind, to Lossy when one
// declares a fail-closed kind, and refuses (target_carriers_fidelity_disposition_malformed) a
// disposition it does not recognize -- it never defaults one to Lossless.

fn atom(identity: &str) -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Atom {
                identity: identity.to_string(),
            }),
        }),
        Rc::new(im::vector![]),
    )
}

fn conj(children: Vec<(Rc<EdgeLabel>, Rc<Node>)>) -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Conj),
        }),
        Rc::new(
            children
                .into_iter()
                .map(|(label, target)| Rc::new(Edge { label, target }))
                .collect(),
        ),
    )
}

fn positional(n: Rc<Node>) -> (Rc<EdgeLabel>, Rc<Node>) {
    (Rc::new(EdgeLabel::Positional), n)
}

fn modeled_only() -> Rc<Node> {
    conj(vec![positional(atom("dag_fidelity_disposition_kind_modeled"))])
}

fn with_fail_closed() -> Rc<Node> {
    let fail_closed = conj(vec![(
        Rc::new(EdgeLabel::Named {
            name: "dag_fidelity_disposition_kind_fail_closed".to_string(),
        }),
        node_synthetic(
            Rc::new(NodeKind::ComputationNode {
                behavior: Behavior::Value,
            }),
            Rc::new(im::vector![]),
        ),
    )]);
    conj(vec![
        positional(atom("dag_fidelity_disposition_kind_modeled")),
        positional(fail_closed),
    ])
}

fn unrecognized() -> Rc<Node> {
    conj(vec![positional(atom("self_host_witness_unknown_disposition"))])
}

fn decode(quotient: Rc<Node>) -> Option<DecodeFidelity> {
    match &*emitted::fidelity_quotient_decode_fidelity(quotient) {
        Outcome::Accepted { value, .. } => Some(value.clone()),
        Outcome::Rejected { .. } => None,
    }
}

// The planted fault feeds the unrecognized quotient where the modeled one is expected, so the
// fault run goes red only if emitted target_carriers really refuses it.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let lossless = decode(if inject_fault { unrecognized() } else { modeled_only() });
    let lossless_ok = matches!(lossless, Some(DecodeFidelity::Lossless));
    println!("quotient modeled inject_fault={inject_fault} fidelity={lossless:?} ok={lossless_ok}");
    let lossy = decode(with_fail_closed());
    let lossy_ok = matches!(lossy, Some(DecodeFidelity::Lossy));
    println!("quotient fail_closed fidelity={lossy:?} ok={lossy_ok}");
    let refused = decode(unrecognized()).is_none();
    println!("quotient unrecognized refused={refused}");
    if lossless_ok && lossy_ok && refused {
        println!("SELF_HOST_TARGET_CARRIERS_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_TARGET_CARRIERS_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
