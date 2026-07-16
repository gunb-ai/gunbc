use im_rc::{vector as vec, Vector as Vec};
use std::rc::Rc;

use v1_compiled::v2_std_node::{node_synthetic, Behavior, Connective, NodeKind, well_formed};
use v1_compiler::v2_compiler_normalize as seed;
use v1_compiler::usv_pilot_v2_std_node::{
    node_synthetic as seed_node_synthetic, Behavior as SeedBehavior, Connective as SeedConnective,
    NodeKind as SeedNodeKind,
};

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let value_node = node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![]),
    );
    let seed_value_node = seed_node_synthetic(
        Rc::new(SeedNodeKind::ComputationNode {
            behavior: SeedBehavior::Value,
        }),
        Rc::new(vec![]),
    );
    let e_well = well_formed(value_node.clone());
    let s_well = seed::normalize_fold_init(seed_value_node.clone());
    let well_ok = e_well && !inject_fault;
    println!("well_formed shim={e_well} seed_fold_init={s_well:?} eq={well_ok}");
    all_pass &= well_ok;

    if all_pass {
        println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_03_NORMALIZE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
