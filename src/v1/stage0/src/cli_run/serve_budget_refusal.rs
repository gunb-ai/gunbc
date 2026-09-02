//! THE SEALED CONSEQUENCE OF A BUDGET BREACH, host realization of `std.evaluation_budget`'s
//! `EvaluationBudgetRefusalConsequence`.
//!
//! IT LIVES IN ITS OWN MODULE BECAUSE RUST PRIVACY IS MODULE-SCOPED, and that is the whole
//! difference between a sealed carrier and a claim of one. An earlier revision defined this struct
//! directly inside `cli_run` with private fields and a single `from_exceeded` associated function,
//! and called it sealed. It was not: every one of `cli_run`'s tens of thousands of lines, and every
//! descendant module, could still write the struct literal with four favourable values and no
//! `InterpError` anywhere in sight -- which is exactly the fabricated-refusal state the carrier
//! exists to make unwritable. An `impl` method is not a constructor wall; a module boundary is.
//!
//! THE SURFACE IS FREE FUNCTIONS, NOT METHODS, for a second reason recorded by
//! `emitted_closure_compile_host`: an `impl` method has no `DeclarationRef` spelling, so a seed
//! declaration authored as a method cannot be cited by the namespace authority at all.

/// The opaque consequence. Its fields are private TO THIS MODULE, so `cli_run` can hold and render
/// one but cannot assemble one.
pub(super) struct ServeBudgetRefusal {
    entry: String,
    clock_key: &'static str,
    elapsed_nanos: u128,
    limit_ms: u64,
}

/// THE SOLE PRODUCER, and it takes the typed cause rather than four scalars. There is deliberately
/// no `from_parts(code, entry, clock, elapsed, limit)`: a constructor over loose values is what
/// would let a boundary that merely CHOSE the same text render a refusal. This value exists because
/// an execution produced the exceeded cause.
///
/// `CompletedOverBudget` is NOT routed here and must not be: a claim that reached its verdict and
/// was then reclassified for cost is a different fact with a different remedy, and sharing this
/// refusal through a wildcard or an `is_over_budget` boolean would be the state-space conflation
/// the budget carrier already refuses upstream.
///
/// THE ENTRY IS PROJECTED, NOT SUPPLIED. It comes from the cause, which the interpreter binds to
/// the evaluation whose budget was armed.
pub(super) fn serve_budget_refusal_from_exceeded(
    err: &crate::v1_interpreter::InterpError,
) -> Option<ServeBudgetRefusal> {
    match err {
        crate::v1_interpreter::InterpError::EvaluationBudgetExceeded {
            entry,
            clock,
            elapsed_nanos,
            limit_ms,
        } => Some(ServeBudgetRefusal {
            entry: entry.clone(),
            clock_key: clock.key(),
            elapsed_nanos: *elapsed_nanos,
            limit_ms: *limit_ms,
        }),
        _ => None,
    }
}

/// The machine-readable body. Both quantities plus the clock are reported so a consumer can tell a
/// spin from a stall without parsing prose.
///
/// THE MACHINE CODE IS NOT A FIELD AND NOT A PARAMETER. It is read from
/// `EVALUATION_BUDGET_REFUSAL_CODE`, the generated projection of
/// `std.evaluation_budget evaluation_budget_refusal_code`. A `code` field would make a consequence
/// with an arbitrary or self-contradictory machine identity constructible, and would relocate the
/// invariant into callers.
/// THE ENCODER IS NOT A PARAMETER EITHER, and the earlier revision's `json_string: impl Fn(&str) ->
/// String` was a SECOND bypass of the same wall wearing a different shape. Sealing the constructor
/// stopped a caller assembling a refusal with a chosen code; passing the renderer's encoder let a
/// caller holding a LEGITIMATE refusal supply a closure that ignores its argument and returns any
/// text it likes, so the response could still carry another machine code without the sealed value
/// ever being touched. Construction and rendering are two boundaries and both had to close.
///
/// The child reaches `super::serve_json_string` directly -- a descendant may read ancestor-private
/// items -- so the parent supplies neither the code, nor the entry, nor a function able to replace
/// either.
pub(super) fn serve_budget_refusal_machine_body(refusal: &ServeBudgetRefusal) -> String {
    format!(
        "{{\"code\":{},\"entry\":{},\"clock\":\"{}\",\"elapsed_ns\":{},\"limit_ms\":{}}}\n",
        super::serve_json_string(
            crate::evaluation_budget_consequence_generated::EVALUATION_BUDGET_REFUSAL_CODE
        ),
        super::serve_json_string(&refusal.entry),
        refusal.clock_key,
        refusal.elapsed_nanos,
        refusal.limit_ms
    )
}

pub(super) fn serve_budget_refusal_diagnostic_line(refusal: &ServeBudgetRefusal) -> String {
    format!(
        "serve: refused {} on {} clock: elapsed_ns={} limit_ms={}",
        refusal.entry, refusal.clock_key, refusal.elapsed_nanos, refusal.limit_ms
    )
}
