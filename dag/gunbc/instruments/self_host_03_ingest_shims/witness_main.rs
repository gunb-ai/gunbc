use im::{hashmap, vector as vec, HashMap};
use std::rc::Rc;

use v1_compiled::v2_compiler_ingest as emitted;
use v1_compiled::v2_std_diagnostic::Outcome;
use v1_compiled::v2_std_node::{node_synthetic, Connective, Node, NodeKind};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant ingest_module_note
// against v1_compiler::v2_compiler_ingest, a hand copy that no longer exists; string parity with a
// copy of the module under test said nothing about what ingest does. Its expected verdicts are
// read off the authority instead: src/v2/compiler/03_ingest.dag parse_atom_frontier_class
// recovers a parse-tree atom's token class from the grammar relation's binding map, then from its
// token-class set, and refuses (parse_tree_atom_token_class_not_recoverable) an atom named in
// neither -- it never invents a class.

fn atom(identity: &str) -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::TypeNode {
            connective: Rc::new(Connective::Atom {
                identity: identity.to_string(),
            }),
        }),
        Rc::new(vec![]),
    )
}

fn class_of(identity: &str) -> Option<String> {
    let binding_to_class: HashMap<String, String> =
        hashmap! { "fn_name".to_string() => "Ident".to_string() };
    let token_classes: HashMap<String, String> =
        hashmap! { "LParen".to_string() => "LParen".to_string() };
    match &*emitted::parse_atom_frontier_class(
        atom(identity),
        identity.to_string(),
        Rc::new(binding_to_class),
        Rc::new(token_classes),
    ) {
        Outcome::Accepted { value, .. } => Some(value.clone()),
        Outcome::Rejected { .. } => None,
    }
}

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): the injected run is
// PASS exactly when the emitted module accepts what the .dag refuses, so a correct module reds it
// and a module that wrongly accepts greens it -- which the harness then rejects.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let unrecoverable = class_of("unbound_atom");
    let all_pass = if inject_fault {
        println!("frontier class injected: unrecoverable class={unrecoverable:?}");
        unrecoverable.is_some()
    } else {
        let bound = class_of("fn_name");
        let bound_ok = bound.as_deref() == Some("Ident");
        println!("frontier class via binding class={bound:?} ok={bound_ok}");
        let direct = class_of("LParen");
        let direct_ok = direct.as_deref() == Some("LParen");
        println!("frontier class via token set class={direct:?} ok={direct_ok}");
        println!("frontier class unrecoverable refused={}", unrecoverable.is_none());
        bound_ok && direct_ok && unrecoverable.is_none()
    };

    if all_pass {
        println!("SELF_HOST_03_INGEST_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_03_INGEST_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
