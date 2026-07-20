use im::vector as vec;
use std::rc::Rc;
use v1_compiled::v2_compiler_use_site_verdict as emitted;
use v1_compiled::v2_std_node::{node_synthetic, Behavior, NodeKind};
use v1_compiler::v2_compiler_use_site_verdict as seed;
use v1_compiler::usv_pilot_v2_std_node::{
    node_synthetic as seed_node_synthetic, Behavior as SeedBehavior, Node as SeedNode,
    NodeKind as SeedNodeKind,
};

fn verdict_eq(e: &emitted::UseSiteVerdict, s: &seed::UseSiteVerdict) -> bool {
    use emitted::UseSiteVerdict as E;
    use seed::UseSiteVerdict as S;
    match (e, s) {
        (E::MoveWhole, S::MoveWhole)
        | (E::Borrow, S::Borrow)
        | (E::CloneShared, S::CloneShared)
        | (E::Unclassified, S::Unclassified) => true,
        (E::MoveField { field: ef, .. }, S::MoveField { field: sf, .. }) => ef == sf,
        _ => false,
    }
}

fn lookup_eq(e: &emitted::UseSiteVerdictLookup, s: &seed::UseSiteVerdictLookup) -> bool {
    use emitted::UseSiteVerdictLookup as E;
    use seed::UseSiteVerdictLookup as S;
    match (e, s) {
        (E::VerdictAbsent, S::VerdictAbsent) | (E::VerdictAmbiguous, S::VerdictAmbiguous) => true,
        (E::VerdictFound { verdict: ev, .. }, S::VerdictFound { verdict: sv, .. }) => {
            verdict_eq(ev, sv)
        }
        _ => false,
    }
}

fn bare_emitted_node() -> Rc<emitted::Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![]),
    )
}

fn bare_seed_node() -> Rc<SeedNode> {
    seed_node_synthetic(
        Rc::new(SeedNodeKind::ComputationNode {
            behavior: SeedBehavior::Value,
        }),
        Rc::new(vec![]),
    )
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;
    let cases = [
        emitted::UseSiteVerdict::MoveWhole,
        emitted::UseSiteVerdict::Borrow,
        emitted::UseSiteVerdict::CloneShared,
        emitted::UseSiteVerdict::Unclassified,
        emitted::UseSiteVerdict::MoveField {
            field: "seen".to_string(),
        },
    ];

    for (i, ev) in cases.iter().enumerate() {
        let sv = match ev {
            emitted::UseSiteVerdict::MoveWhole => seed::UseSiteVerdict::MoveWhole,
            emitted::UseSiteVerdict::Borrow => seed::UseSiteVerdict::Borrow,
            emitted::UseSiteVerdict::CloneShared => seed::UseSiteVerdict::CloneShared,
            emitted::UseSiteVerdict::Unclassified => seed::UseSiteVerdict::Unclassified,
            emitted::UseSiteVerdict::MoveField { field, .. } => seed::UseSiteVerdict::MoveField {
                field: field.clone(),
            },
        };
        let bare_e = bare_emitted_node();
        let bare_s = bare_seed_node();
        let decorated_e = emitted::attach_use_site_verdict(
            bare_e,
            Rc::new(if inject_fault && i == 0 {
                emitted::UseSiteVerdict::Borrow
            } else {
                ev.clone()
            }),
        );
        let decorated_s =
            seed::attach_use_site_verdict(bare_s, Rc::new(sv));
        let er = emitted::use_site_verdict_lookup(decorated_e);
        let sr = seed::use_site_verdict_lookup(decorated_s);
        let ok = lookup_eq(&er, &sr);
        println!(
            "case({i:?}) emitted={er:?} seed={sr:?} eq={ok}"
        );
        all_pass &= ok;
    }

    let absent_e = emitted::use_site_verdict_lookup(bare_emitted_node());
    let absent_s = seed::use_site_verdict_lookup(bare_seed_node());
    let absent_ok = lookup_eq(&absent_e, &absent_s);
    println!("absent emitted={absent_e:?} seed={absent_s:?} eq={absent_ok}");
    all_pass &= absent_ok;

    if all_pass {
        println!("SELF_HOST_USE_SITE_VERDICT_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_USE_SITE_VERDICT_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
