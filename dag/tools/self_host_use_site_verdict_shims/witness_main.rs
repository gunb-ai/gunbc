use im_rc::vector as vec;
use std::rc::Rc;
use v1_compiled::v2_compiler_use_site_verdict::*;
use v1_compiled::v2_std_node::*;

fn verdict_eq(a: &UseSiteVerdict, b: &UseSiteVerdict) -> bool {
    use UseSiteVerdict::*;
    match (a, b) {
        (MoveWhole, MoveWhole) | (Borrow, Borrow) | (CloneShared, CloneShared) | (Unclassified, Unclassified) => true,
        (MoveField { field: fa, .. }, MoveField { field: fb, .. }) => fa == fb,
        _ => false,
    }
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;
    let cases = vec![
        Rc::new(UseSiteVerdict::MoveWhole),
        Rc::new(UseSiteVerdict::Borrow),
        Rc::new(UseSiteVerdict::CloneShared),
        Rc::new(UseSiteVerdict::Unclassified),
        Rc::new(UseSiteVerdict::MoveField { field: "seen".to_string() }),
    ];
    for verdict in cases {
        let bare = node_synthetic(
            Rc::new(NodeKind::ComputationNode { behavior: Behavior::Value }),
            Rc::new(vec![]),
        );
        let decorated = attach_use_site_verdict(bare, verdict.clone());
        let ok = match &*use_site_verdict_lookup(decorated) {
            UseSiteVerdictLookup::VerdictFound { verdict: got, .. } => verdict_eq(got, &verdict),
            _ => false,
        };
        all_pass &= ok;
    }
    let absent_ok = matches!(&*use_site_verdict_lookup(node_synthetic(
        Rc::new(NodeKind::ComputationNode { behavior: Behavior::Value }), Rc::new(vec![]),
    )), UseSiteVerdictLookup::VerdictAbsent);
    all_pass &= absent_ok;
    if inject_fault { all_pass = false; }
    if all_pass {
        println!("SELF_HOST_USE_SITE_VERDICT_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_USE_SITE_VERDICT_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
