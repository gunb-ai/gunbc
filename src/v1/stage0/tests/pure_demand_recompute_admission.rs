//! A pure demand with no declared plural obligation derives no identity: the seed consumes the
//! ladder's AcceptedSingleRecompute judgment BEFORE any key is built (DESIGN section 2;
//! `std.materialization_ladder` rule 5), so an undeclared single demand constructs, looks up and
//! retains no result-memo key merely because its callee is pure.
//!
//! Execute: cargo test -p v1-compiler --test pure_demand_recompute_admission.
//! Required clippy --all-targets compiles this target; no CI step executes it.
//! The declared drop is gunbc.rung_drop.rust_unit_tests_off_the_merge_path.
//!
//! ITS OWN BINARY, DELIBERATELY: the recompute-trace switch is a process-wide latch, and the
//! interpreter's in-crate tests turn it ON and never off. The RED here needs it OFF, so the two
//! arms run sequentially in one test in a process nothing else shares.

use std::rc::Rc;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

const FIXTURE: &str = "module fixture.recompute_admission\n\
                       fn measure(s: String) -> Int { string_length(s) }\n\
                       fn demand() -> Int { measure(s: \"a pure demand, undeclared, single\") }\n";

fn fresh_ctx(result: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult) -> InterpContext {
    let graph = result.graph.as_ref().expect("fixture graph");
    InterpContext::new(
        graph,
        result.source_indices.clone(),
        ExecutionMode::Hermetic,
    )
}

/// Evaluate `demand()` once and return (value, evaluator steps it took).
fn one_demand(ctx: &InterpContext) -> (Value, u64) {
    let before = v1_interpreter::evaluator_steps();
    let v = v1_interpreter::run_in_context(ctx, "fixture.recompute_admission.demand", false)
        .expect("demand evaluates");
    (v, v1_interpreter::evaluator_steps() - before)
}

/// THE RED: on the production route (trace off) two equal pure demands derive ZERO argument
/// identities and both EVALUATE -- the second is not served from the first. Restoring
/// unconditional keying (a key built before admission) turns the first assertion red; restoring
/// a result memo that serves the repeat turns the second red (a served call takes no callee
/// steps). THE POSITIVE CONTROL is the same pair under the recompute-trace ledger, the one
/// keying site that is admitted on this path: it derives exactly one identity per demand (one
/// String argument each) and STILL evaluates both, because the ledger is an instrument and not
/// a provider. Without that control the RED could pass because keying was unreachable for an
/// unrelated reason.
#[test]
fn an_undeclared_single_pure_demand_derives_no_identity_and_is_never_served() {
    let result = compile_to_resolved(Rc::new(im::vector![Rc::new(SourceFile {
        path: "workspace/src/recompute_admission.dag".to_string(),
        content: FIXTURE.to_string(),
    })]));

    // Arm 1: production route.
    std::env::remove_var("GUNBC_RECOMPUTE_TRACE");
    v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
    assert!(!v1_interpreter::eval_recompute_trace_enabled());
    let ctx = fresh_ctx(&result);
    let (first, first_steps) = one_demand(&ctx);
    let (second, second_steps) = one_demand(&ctx);
    assert!(matches!(first, Value::Int(33)), "{first:?}");
    assert_eq!(first, second);
    assert_eq!(
        v1_interpreter::argument_identities_derived(&ctx),
        0,
        "an undeclared single demand must reach no keying site"
    );
    assert!(first_steps > 0);
    assert_eq!(
        first_steps, second_steps,
        "the repeat must evaluate in full: a served repeat takes no callee steps"
    );

    // Arm 2: the recompute-trace ledger, the admitted keying site, keys and still evaluates.
    std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
    v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
    assert!(v1_interpreter::eval_recompute_trace_enabled());
    let traced = fresh_ctx(&result);
    let (_, traced_first_steps) = one_demand(&traced);
    let (_, traced_second_steps) = one_demand(&traced);
    assert_eq!(
        v1_interpreter::argument_identities_derived(&traced),
        2,
        "the ledger derives one identity per demand: one String argument, two demands"
    );
    assert_eq!(traced_first_steps, first_steps);
    assert_eq!(traced_second_steps, first_steps);
    std::env::remove_var("GUNBC_RECOMPUTE_TRACE");
    v1_interpreter::refresh_eval_recompute_trace_enabled_cache_for_tests();
}
