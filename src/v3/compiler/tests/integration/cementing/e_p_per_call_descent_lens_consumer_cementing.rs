//! **Layer:** integration (Band-C cementing — TESTING.md §"Band-C")
//!
//! Gate **#76** (`e_p_per_call_descent_evidence_full_coverage`) Phase-3
//! **lens-consumer-path cementing ratchets** for cost.dag + complexity.dag on
//! the producer-side fixture classes that `m2_substrate_inhabitance_test.rs`
//! Phase-1 receipts already pin at the producer surface.
//!
//! **Ratchet status**: all 4 tests are `#[ignore]`'d pending the named
//! follow-on **lens-consumer match-arm walker substrate extension** (canvas
//! TBD — see `<canvas-link>` placeholder below; R3 Substrate Mgr to patch
//! once the canvas lands per the `feedback_post_merge_ledger_receipt_sync`
//! cycle). The tests are authored here so that:
//!   - the §1.8 row #76 evidence-cite has a concrete file to point at;
//!   - the lens-consumer-walker substrate extension PR can drop the
//!     `#[ignore]` markers as part of its same-PR cementing receipt
//!     (matching the hot-fix #2723 / `#[ignore]`'d-scaffold precedent);
//!   - the worker authoring the extension has an executable fixture set
//!     at hand instead of re-deriving them from the m2 producer receipts.
//!
//! ## The gap (consumer-side)
//!
//! Producer surface is correct (m2 receipts):
//!   - `e_p_per_call_descent_evidence_classifies_match_payload_self_call_as_strict_sub_value`
//!     pins `SubValueRelation::StrictSubValue` for `Cons(tail) => f(tail)`;
//!   - `e_p_per_call_descent_evidence_emits_per_arg_relation_for_multi_arg_self_call`
//!     pins per-arg `[StrictSubValue, SubValueUnknown, PreservedValue]`;
//!   - `per_call_pattern_at` projects these to `CallPattern::ChildAccessorCall`
//!     for the head argument (`e_p_per_call_pattern_projects_multi_arg_self_call_from_per_arg_evidence`).
//!
//! Consumer surface (`src/v3/lenses/complexity.dag` + `src/v3/lenses/cost.dag`)
//! has **no match/Disj-arm walker**: `branch_summary` (`complexity.dag:330`)
//! handles `Branch` behavior with `List<BranchPath>` (if-style); ADT match
//! lowers through `Disj`/`Conj` behaviors that the summary walker never
//! enters, so the `acc: List<ComplexityEntry>` table is never populated for
//! match-arm result ports. The `ChildAccessorCall` arm at
//! `complexity.dag:198,261` (and `cost.dag` sibling) is structurally
//! reachable in principle, but is never reached at runtime for these
//! fixtures because `compose_many_inputs` is never invoked on a match-arm
//! result port. Result: `lookup_summary(acc, port)` returns `Miss` on the
//! function's `bind.value` port; the lens `Lookup<ComplexitySummary>`
//! returns `Miss` rather than a `LinearCost`-bearing `Hit`.
//!
//! Net: the 8 existing m2 producer receipts emit per-call descent evidence
//! that the lenses cannot yet consume for ADT-typed recursive functions.
//! Closing gate #76 §Acceptance to PASSING requires extending the lens
//! consumer walker; canonical design references:
//!   - `docs/design-complexity-lens-behavioral-completeness.md` §272, §282
//!     (per-call-class behavioral completeness obligations);
//!   - `docs/briefs/r3-t-e-p-producer-broadening-worker.md` Phase 3
//!     (acceptance: per-lens v2-oracle cementing on each call-site class).
//!
//! ## Why ignored, not removed
//!
//! Per `feedback_only_claim_what_actually_exists`, scope-narrowing the gate
//! to "Int-parameter recursive function class only" (the current sole Hit
//! path) would falsely cash "full_coverage". Per Hot-fix #2723 precedent
//! (Pattern-A scaffold with `#[ignore]`), preserving executable evidence
//! of the gap in-tree under a named blocker is the correct shape: the
//! evidence stays grep-discoverable from the §1.8 row, and dropping the
//! `#[ignore]` is part of the follow-on PR's same-PR cementing receipt
//! rather than that PR having to re-author the fixtures.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, PortId, SymbolicCost};
use v3_compiler::lens_cost::{complexity_of, Certainty, ComplexityLookup, ComplexitySummary};
use v3_compiler::lens_cost_symbolic::{symbolic_cost_lookup, SymbolicCostLookup};

/// Named blocker shared across all `#[ignore]`'d ratchets in this file.
/// Single source of truth so the follow-on lens-consumer-walker PR drops
/// one constant rather than four duplicated strings.
const LENS_CONSUMER_MATCH_ARM_WALKER_BLOCKER: &str =
    "R3 gate #76 blocker: lens-consumer match-arm walker substrate extension required \
     (complexity.dag + cost.dag have no Disj/match-arm summary fold; see module doc \
     + docs/design-complexity-lens-behavioral-completeness.md §272/§282). \
     Canvas link: TBD — R3 Substrate Mgr to patch via feedback_post_merge_ledger_receipt_sync.";

fn find_bind_value(dag: &v3_compiler::dag::Dag, name: &str) -> PortId {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|bind| bind.name == name)
        .unwrap_or_else(|| panic!("bind `{name}` not found"))
        .value
}

fn expect_summary(dag: &v3_compiler::dag::Dag, bind_name: &str) -> ComplexitySummary {
    let port = find_bind_value(dag, bind_name);
    match complexity_of(dag, &port) {
        ComplexityLookup::Hit(summary) => summary,
        ComplexityLookup::Miss => panic!("complexity_of returned Miss for bind `{bind_name}`"),
    }
}

fn expect_symbolic_cost(dag: &v3_compiler::dag::Dag, bind_name: &str) -> SymbolicCost {
    let port = find_bind_value(dag, bind_name);
    match symbolic_cost_of(dag, &port) {
        SymbolicCostLookup::Hit(cost) => cost,
        SymbolicCostLookup::Miss => {
            panic!("symbolic_cost_of returned Miss for bind `{bind_name}`")
        }
    }
}

fn cost_contains_polynomial_or_unknown(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::PolynomialCost { .. } | SymbolicCost::UnknownCost { .. } => true,
        SymbolicCost::SumCost { _0: terms } | SymbolicCost::ProductCost { _0: terms } => terms
            .iter()
            .any(|t| cost_contains_polynomial_or_unknown(t.as_ref())),
        _ => false,
    }
}

fn cost_contains_linear(cost: &SymbolicCost) -> bool {
    match cost {
        SymbolicCost::LinearCost { .. } => true,
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            terms.iter().any(|t| cost_contains_linear(t.as_ref()))
        }
        _ => false,
    }
}

fn linear_size_ports(cost: &SymbolicCost, out: &mut Vec<PortId>) {
    match cost {
        SymbolicCost::LinearCost { _0: var } | SymbolicCost::LogCost { _0: var } => {
            out.push(var.source_port);
        }
        SymbolicCost::PolynomialCost { var, .. } => out.push(var.source_port),
        SymbolicCost::ProductCost { _0: terms } | SymbolicCost::SumCost { _0: terms } => {
            for term in terms.iter() {
                linear_size_ports(term.as_ref(), out);
            }
        }
        SymbolicCost::ConstantCost { .. } | SymbolicCost::UnknownCost { .. } => {}
    }
}

fn assert_proven(certainty: &Certainty, context: &str) {
    assert!(
        matches!(certainty, Certainty::Proven),
        "{context}: expected Proven, got {certainty:?}"
    );
}

fn run_with_lens_stack(name: &'static str, f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(f)
        .expect("spawn lens cementing thread")
        .join()
        .expect("lens cementing thread should not panic");
}

/// **Match-payload structural descent** — `Cons(tail) => ep_count_c(tail)`.
/// Producer receipt pins `SubValueRelation::StrictSubValue` →
/// `CallPattern::ChildAccessorCall { accessor: "_0" }`; this cementing
/// ratchet pins the consumer-path expectation that `complexity_of` returns a
/// `LinearCost`-bearing `Hit` on the bind value port. See module doc for
/// the lens-consumer match-arm walker blocker.
#[test]
#[ignore = "see LENS_CONSUMER_MATCH_ARM_WALKER_BLOCKER constant + module doc — R3 gate #76 blocker"]
fn match_payload_self_call_cements_complexity_lens_consumes_strict_sub_value() {
    run_with_lens_stack("complexity-match-payload-cementing", || {
        let dag = compile_to_dag(
            "\
type EpListC = EpNilC | EpConsC(EpListC)
fn ep_count_c(xs: EpListC) -> Int =
  match xs { EpConsC(tail) => ep_count_c(tail), EpNilC => 0 }
",
            "cement_e_p_match_payload_complexity.v3",
        )
        .expect("match-payload recursion fixture compiles");
        let summary = expect_summary(&dag, "ep_count_c");

        assert!(
            !cost_contains_polynomial_or_unknown(&summary.work),
            "match-payload tail-recursion work must consume `StrictSubValue` as a \
             provable descent (no Polynomial/Unknown carriers); got {:?}",
            summary.work
        );
        assert!(
            cost_contains_linear(&summary.work),
            "match-payload tail-recursion work should carry a `LinearCost` term on \
             the descending parameter (frozen v2 projection), got {:?}",
            summary.work
        );
        assert!(
            cost_contains_linear(&summary.span),
            "match-payload tail-recursion span should carry a `LinearCost` term on \
             the descending parameter (frozen v2 projection), got {:?}",
            summary.span
        );
        assert_proven(&summary.work_certainty, "match-payload work certainty");
        assert_proven(&summary.span_certainty, "match-payload span certainty");
    });
}

/// Cost-lens sibling of the complexity ratchet above. See module doc for
/// the lens-consumer match-arm walker blocker.
#[test]
#[ignore = "see LENS_CONSUMER_MATCH_ARM_WALKER_BLOCKER constant + module doc — R3 gate #76 blocker"]
fn match_payload_self_call_cements_symbolic_cost_consumes_strict_sub_value() {
    run_with_lens_stack("cost-match-payload-cementing", || {
        let dag = compile_to_dag(
            "\
type EpListD = EpNilD | EpConsD(EpListD)
fn ep_count_d(xs: EpListD) -> Int =
  match xs { EpConsD(tail) => ep_count_d(tail), EpNilD => 0 }
",
            "cement_e_p_match_payload_cost.v3",
        )
        .expect("match-payload recursion fixture compiles");
        let bind = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|b| b.name == "ep_count_d")
            .expect("ep_count_d bind");
        let parameter = bind
            .params
            .first()
            .copied()
            .expect("ep_count_d should have one parameter port");
        let cost = expect_symbolic_cost(&dag, "ep_count_d");

        assert!(
            !cost_contains_polynomial_or_unknown(&cost),
            "match-payload tail-recursion symbolic cost must consume `StrictSubValue` \
             provable descent without Polynomial/Unknown carriers, got {cost:?}"
        );
        assert!(
            cost_contains_linear(&cost),
            "match-payload tail-recursion symbolic cost should carry a `LinearCost` \
             term (frozen v2 projection), got {cost:?}"
        );

        let mut ports = Vec::new();
        linear_size_ports(&cost, &mut ports);
        assert!(
            ports.contains(&parameter),
            "match-payload SizeVariable must key off the descending parameter port \
             {parameter:?} (the `xs` port), got cost={cost:?}"
        );
    });
}

/// **Multi-argument per-arg vector** — `ep_count_acc_e(tail, acc + 1, limit)`.
/// Producer receipt pins per-arg `[StrictSubValue, SubValueUnknown,
/// PreservedValue]`; this cementing ratchet pins the consumer-path
/// expectation that the lens collapses to the head-argument descent —
/// accumulator and preserved arguments must NOT introduce parallel
/// `SizeVariable`s / `PolynomialCost` / `UnknownCost`. See module doc for
/// the lens-consumer match-arm walker blocker.
#[test]
#[ignore = "see LENS_CONSUMER_MATCH_ARM_WALKER_BLOCKER constant + module doc — R3 gate #76 blocker"]
fn multi_arg_self_call_cements_complexity_lens_collapses_to_head_descent() {
    run_with_lens_stack("complexity-multi-arg-cementing", || {
        let dag = compile_to_dag(
            "\
type EpListE = EpNilE | EpConsE(EpListE)
fn ep_count_acc_e(xs: EpListE, acc: Int, limit: Int) -> Int =
  match xs {
    EpConsE(tail) => ep_count_acc_e(tail, acc + 1, limit),
    EpNilE => acc
  }
",
            "cement_e_p_multi_arg_complexity.v3",
        )
        .expect("multi-arg accumulator fixture compiles");
        let summary = expect_summary(&dag, "ep_count_acc_e");

        assert!(
            !cost_contains_polynomial_or_unknown(&summary.work),
            "multi-arg recursion work must collapse to head-arg descent — \
             accumulator + preserved-arg must not introduce Polynomial/Unknown, \
             got {:?}",
            summary.work
        );
        assert!(
            cost_contains_linear(&summary.work),
            "multi-arg recursion work should carry a `LinearCost` term \
             (per_call_pattern_at projects head-arg `StrictSubValue` to \
             `ChildAccessorCall`), got {:?}",
            summary.work
        );
        assert_proven(&summary.work_certainty, "multi-arg work certainty");
        assert_proven(&summary.span_certainty, "multi-arg span certainty");
    });
}

/// Cost-lens sibling: `symbolic_cost_of` on the multi-arg accumulator fixture
/// must collapse to the descending-parameter `LinearCost`, not multiply across
/// the preserved / accumulator arguments. See module doc for the
/// lens-consumer match-arm walker blocker.
#[test]
#[ignore = "see LENS_CONSUMER_MATCH_ARM_WALKER_BLOCKER constant + module doc — R3 gate #76 blocker"]
fn multi_arg_self_call_cements_symbolic_cost_collapses_to_head_descent() {
    run_with_lens_stack("cost-multi-arg-cementing", || {
        let dag = compile_to_dag(
            "\
type EpListF = EpNilF | EpConsF(EpListF)
fn ep_count_acc_f(xs: EpListF, acc: Int, limit: Int) -> Int =
  match xs {
    EpConsF(tail) => ep_count_acc_f(tail, acc + 1, limit),
    EpNilF => acc
  }
",
            "cement_e_p_multi_arg_cost.v3",
        )
        .expect("multi-arg accumulator fixture compiles");
        let bind = dag
            .nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|b| b.name == "ep_count_acc_f")
            .expect("ep_count_acc_f bind");
        let descending_parameter = bind
            .params
            .first()
            .copied()
            .expect("ep_count_acc_f should have parameters");
        let cost = expect_symbolic_cost(&dag, "ep_count_acc_f");

        assert!(
            !cost_contains_polynomial_or_unknown(&cost),
            "multi-arg recursion symbolic cost must collapse to head-arg descent \
             without Polynomial/Unknown carriers, got {cost:?}"
        );

        let mut ports = Vec::new();
        linear_size_ports(&cost, &mut ports);
        assert!(
            ports.contains(&descending_parameter),
            "multi-arg recursion SizeVariable must include the descending head \
             parameter port {descending_parameter:?}, got cost={cost:?} ports={ports:?}"
        );
        for non_descending in bind.params.iter().skip(1) {
            assert!(
                !ports.contains(non_descending),
                "multi-arg recursion must not introduce a SizeVariable for \
                 non-descending parameter {non_descending:?} (accumulator / \
                 preserved-arg), got cost={cost:?}"
            );
        }
    });
}

/// Single grep-anchor that surfaces the blocker constant in clippy/grep output
/// even on green CI; prevents the `#[ignore]` ratchets from going invisible
/// while the lens-consumer-walker substrate canvas is pending.
#[test]
fn lens_consumer_match_arm_walker_blocker_constant_is_present() {
    assert!(
        LENS_CONSUMER_MATCH_ARM_WALKER_BLOCKER.contains("R3 gate #76 blocker"),
        "blocker constant must remain grep-discoverable for §1.8 row #76 evidence-cite"
    );
}
