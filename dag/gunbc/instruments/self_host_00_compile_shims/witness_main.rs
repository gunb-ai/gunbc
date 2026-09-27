use im::{vector as vec, Vector};
use std::rc::Rc;
use v1_compiled::v2_compiler_compile as emitted;
use v1_compiled::v2_compiler_compile::{
    CompileLensGrain, LensGateWitness, RequiredLensCoverage, RequiredLensJoinCause,
};
use v1_compiled::v2_std_node::{node_synthetic, Behavior, NodeKind};
use v1_compiled::v2_std_witness::Witness;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// required_lens_grain_note against a hand copy in v1_compiler::v2_compiler_compile, which said
// nothing about what the compile door does. Its expected verdicts are read off the authority
// instead: src/v2/compiler/00_compile.dag join_required_lens_coverage is the join
// validate_then_compile refuses on -- the required roster observed once per lens at its declared
// grain closes, and a CompileLensRoot lens observed twice refuses with
// RootLensObservedMoreThanOnce (a root-grain lens runs ONCE at the tree root).

fn held(lens: &Rc<v1_compiled::v2_compiler_compile::CompileLens>) -> Rc<LensGateWitness> {
    Rc::new(LensGateWitness {
        lens: lens.identity.clone(),
        grain: lens.grain.clone(),
        witness: Rc::new(Witness::Holds {
            value: node_synthetic(
                Rc::new(NodeKind::ComputationNode {
                    behavior: Behavior::Value,
                }),
                Rc::new(vec![]),
            ),
        }),
    })
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let roster = emitted::required_compile_lens_roster();
    let once: Vector<Rc<LensGateWitness>> = roster.iter().map(held).collect();
    let root_lens = roster
        .iter()
        .find(|l| matches!(l.grain, CompileLensGrain::CompileLensRoot))
        .expect("the required roster registers a root-grain lens")
        .clone();
    let mut twice = once.clone();
    twice.push_back(held(&root_lens));
    let twice_join = emitted::join_required_lens_coverage(roster.clone(), Rc::new(twice));
    let twice_closed = matches!(&*twice_join, RequiredLensCoverage::RequiredLensCoverageClosed { .. });

    // --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): coverage closes
    // although a root-grain lens was observed twice. A correct module reds it; a join that admits
    // repeated root observations greens it, which the harness rejects.
    let all_pass = if inject_fault {
        println!("compile injected: root lens observed twice closes={twice_closed}");
        twice_closed
    } else {
        let once_closed = matches!(
            &*emitted::join_required_lens_coverage(roster.clone(), Rc::new(once)),
            RequiredLensCoverage::RequiredLensCoverageClosed { .. }
        );
        let twice_refused_as_repeat = matches!(&*twice_join,
            RequiredLensCoverage::RequiredLensCoverageRefused { cause }
                if matches!(&**cause, RequiredLensJoinCause::RootLensObservedMoreThanOnce { .. }));
        println!("compile roster observed once closes={once_closed}");
        println!("compile root lens observed twice refused as repeat={twice_refused_as_repeat}");
        once_closed && twice_refused_as_repeat
    };

    if all_pass {
        println!("SELF_HOST_00_COMPILE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_00_COMPILE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
