//! **Layer:** integration
//!
//! R2 Release B5 — Loop **construction-closure** (synthesis Tier 2 §5, PR #809).
//!
//! Audit summary (see PR body for full trace):
//! - **Live lowering:** `lower.rs` — `finalize_mutual_clusters` (`LoopBound::Descent`) and the
//!   single-fn recursive + descent-provable arm (`LoopBound::Cardinality`), both reached from
//!   [`v3_compiler::lower::lower_bodies_phase`].
//! - **Test-only:** [`v3_compiler::dag::Dag::push_loop`] delegates to a single `push_node` site
//!   for synthetic DAGs (`dag/builder.rs`).
//! - **Bootstrap:** `bootstrap_generated*.rs` embed serialized `Behavior::Loop` literals produced
//!   when regen ran the same lowering pipeline — not a parallel lowering algorithm.

use v3_compiler::dag::{Behavior, LoopBound};
use v3_compiler::lens_provenance::{origin_of, Origin};

use crate::common::cached_compile_to_dag;

// `CARGO_MANIFEST_DIR` is the `v3-compiler` crate root (`src/v3/compiler/`).
const LOWER_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lower.rs"));

const BUILDER_RS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/dag/builder.rs"));

/// Construction-closure holds only if production lowering continues to materialize
/// `Behavior::Loop` at exactly these two call sites (single recursive fn + mutual cluster).
#[test]
fn lower_rs_defines_exactly_two_behavior_loop_push_sites() {
    let needle = "dag.push_node(Behavior::Loop(";
    let count = LOWER_RS.matches(needle).count();
    assert_eq!(
        count, 2,
        "construction-closure audit: expected exactly two `{needle}` sites in lower.rs; found {count}. \
         If you added a third Loop origin, update the audit + this gate or document an intentional split."
    );

    let builder_needle = "self.push_node(Behavior::Loop(";
    let builder_count = BUILDER_RS.matches(builder_needle).count();
    assert_eq!(
        builder_count, 1,
        "expected exactly one `{builder_needle}` in dag/builder.rs (test `push_loop` wrapper); found {builder_count}"
    );
}

#[test]
fn every_behavior_loop_after_compile_matches_lowering_bounds_and_accumulated_origin() {
    let src_single = "\
fn count(n: Int) -> Int = if n == 0 then 0 else 1 + count(n - 1)
let _: Int = count(1)
";
    let file_single = "r2_b5_loop_closure_single_recursive.v3";

    let src_mutual = "\
fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)
fn odd(n: Int) -> Bool = if n == 0 then false else even(n - 1)
";
    let file_mutual = "r2_b5_loop_closure_mutual_recursive.v3";

    let mut saw_cardinality_fixture = false;
    let mut saw_descent_fixture = false;

    for (src, file) in [(src_single, file_single), (src_mutual, file_mutual)] {
        let dag = cached_compile_to_dag(src, file);
        assert!(
            dag.diagnostics().is_empty(),
            "{file}: {:?}",
            dag.diagnostics()
        );

        for behavior in dag.nodes() {
            let Behavior::Loop(lp) = behavior else {
                continue;
            };

            match &lp.bound {
                LoopBound::Cardinality { count } => {
                    assert!(
                        dag.port_opt(count).is_some(),
                        "cardinality-bound loops carry a valid explicit count port"
                    );
                    if lp.span.file == file {
                        saw_cardinality_fixture = true;
                    }
                }
                LoopBound::Descent { cluster, measure } => {
                    assert!(
                        (cluster.raw() as usize) < dag.clusters().len(),
                        "descent cluster id must resolve into `dag.clusters()`"
                    );
                    assert_eq!(
                        *measure, lp.source,
                        "descent-bound loops carry the same runtime measure as source"
                    );
                    if lp.span.file == file {
                        saw_descent_fixture = true;
                    }
                }
            }

            let origin = origin_of(&dag, &lp.output);
            assert!(
                matches!(origin, Origin::Accumulated { .. }),
                "Loop output ports must classify as Accumulated in provenance lens; got {origin:?} ({file})"
            );
        }
    }

    assert!(
        saw_cardinality_fixture,
        "expected single-fn recursive fixture to contribute at least one Cardinality Loop in that file"
    );
    assert!(
        saw_descent_fixture,
        "expected mutual-recursion fixture to contribute at least one Descent Loop in that file"
    );
}
