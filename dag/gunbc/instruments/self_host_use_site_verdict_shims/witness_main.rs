use im::vector as vec;
use std::rc::Rc;
use v1_compiled::v2_compiler_use_site_verdict as emitted;
use v1_compiled::v2_compiler_use_site_verdict::{UseSiteVerdict, UseSiteVerdictLookup};
use v1_compiled::v2_std_node::{node_synthetic, Behavior, Node, NodeKind};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare every lookup against a hand copy in
// v1_compiler::v2_compiler_use_site_verdict, which asserted only that two realizations agreed.
// The expected verdicts are read off the authority instead: src/v2/compiler/use_site_verdict.dag
// use_site_verdict_lookup reads the ONE use_site_verdict edge through v2.std.node
// named_edge_target_lookup -- an attached verdict round-trips, no edge is VerdictAbsent, and two
// edges are VerdictAmbiguous, never a found verdict (neither attachment is the answer).

fn bare_node() -> Rc<Node> {
    node_synthetic(
        Rc::new(NodeKind::ComputationNode {
            behavior: Behavior::Value,
        }),
        Rc::new(vec![]),
    )
}

fn round_trips(v: UseSiteVerdict) -> bool {
    let decorated = emitted::attach_use_site_verdict(bare_node(), Rc::new(v.clone()));
    matches!(&*emitted::use_site_verdict_lookup(decorated),
        UseSiteVerdictLookup::VerdictFound { verdict } if **verdict == v)
}

fn doubly_attached_found() -> bool {
    let once = emitted::attach_use_site_verdict(bare_node(), Rc::new(UseSiteVerdict::Borrow));
    let twice = emitted::attach_use_site_verdict(once, Rc::new(UseSiteVerdict::MoveWhole));
    matches!(
        &*emitted::use_site_verdict_lookup(twice),
        UseSiteVerdictLookup::VerdictFound { .. }
    )
}

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): a node carrying two
// verdict edges yields a found verdict. A correct module reds it; one that answers with either
// attachment greens it, which the harness rejects.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let ambiguous_found = doubly_attached_found();
    let all_pass = if inject_fault {
        println!("use_site_verdict injected: two verdict edges found={ambiguous_found}");
        ambiguous_found
    } else {
        let variants = [
            UseSiteVerdict::MoveWhole,
            UseSiteVerdict::Borrow,
            UseSiteVerdict::CloneShared,
            UseSiteVerdict::Unclassified,
            UseSiteVerdict::MoveField {
                field: "seen".to_string(),
            },
        ];
        let every_variant = variants.iter().all(|v| round_trips(v.clone()));
        let absent = matches!(
            &*emitted::use_site_verdict_lookup(bare_node()),
            UseSiteVerdictLookup::VerdictAbsent
        );
        println!("use_site_verdict every variant round-trips={every_variant}");
        println!("use_site_verdict no edge absent={absent}");
        println!("use_site_verdict two edges refused={}", !ambiguous_found);
        every_variant && absent && !ambiguous_found
    };

    if all_pass {
        println!("SELF_HOST_USE_SITE_VERDICT_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_USE_SITE_VERDICT_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
