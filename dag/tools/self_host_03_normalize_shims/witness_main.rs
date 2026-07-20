use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

use v1_compiled::v2_compiler_normalize as emitted;
use v1_compiled::v2_std_node::{
    node_synthetic as emitted_node_synthetic, Behavior as EBehavior, Connective as EConnective,
    NodeKind as ENodeKind,
};
use v1_compiler::v2_compiler_normalize as seed;
use v1_compiler::usv_pilot_v2_std_node::{
    node_synthetic as seed_node_synthetic, Behavior as SBehavior, Connective as SConnective,
    NodeKind as SNodeKind,
};

fn outcome_is_accepted_emitted<T>(o: &Rc<emitted::Outcome<T>>) -> bool {
    matches!(&**o, emitted::Outcome::Accepted { .. })
}

fn outcome_is_accepted_seed<T>(o: &Rc<seed::Outcome<T>>) -> bool {
    matches!(&**o, seed::Outcome::Accepted { .. })
}

fn value_node_emitted() -> Rc<v1_compiled::v2_std_node::Node> {
    emitted_node_synthetic(
        Rc::new(ENodeKind::ComputationNode {
            behavior: EBehavior::Value,
        }),
        Rc::new(vec![]),
    )
}

fn value_node_seed() -> Rc<v1_compiler::usv_pilot_v2_std_node::Node> {
    seed_node_synthetic(
        Rc::new(SNodeKind::ComputationNode {
            behavior: SBehavior::Value,
        }),
        Rc::new(vec![]),
    )
}

fn malformed_disj_emitted() -> Rc<v1_compiled::v2_std_node::Node> {
    emitted_node_synthetic(
        Rc::new(ENodeKind::TypeNode {
            connective: Rc::new(EConnective::Disj),
        }),
        Rc::new(vec![]),
    )
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let e_init = emitted::normalize_fold_init(value_node_emitted());
    let s_init = seed::normalize_fold_init(value_node_seed());
    let init_ok =
        outcome_is_accepted_emitted(&e_init) && outcome_is_accepted_seed(&s_init);
    println!("normalize_fold_init eq_accept={init_ok}");
    all_pass &= init_ok;

    let e_parse = if inject_fault {
        malformed_disj_emitted()
    } else {
        value_node_emitted()
    };
    let e_norm = emitted::normalize(e_parse);
    let s_norm = seed::normalize(value_node_seed());
    let norm_ok = outcome_is_accepted_emitted(&e_norm) == outcome_is_accepted_seed(&s_norm)
        && (!inject_fault || !outcome_is_accepted_emitted(&e_norm));
    println!(
        "normalize inject_fault={inject_fault} parity={norm_ok} emitted_accept={} seed_accept={}",
        outcome_is_accepted_emitted(&e_norm),
        outcome_is_accepted_seed(&s_norm)
    );
    all_pass &= norm_ok;

    if all_pass {
        println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
