// The live claim-evaluation kernel: eval-context construction, the unwind boundary, and the
// measured single-claim run. Split from required_floor_runner.rs (pure code motion; no semantic
// change) so the callers that evaluate claims (claim_batch, `gunbc run`, target invocation,
// regen hosts) do not depend on the required-floor host runner.

use super::*;
use crate::v1_compiler_compile;
use crate::v1_interpreter;
use crate::v1_interpreter::Value;
use im::HashMap;
use std::rc::Rc;

/// THE SINGLE authority that maps a caller's flags to an [`ExecutionMode`].
///
/// DESIGN 6b root generalization. `claim_batch` and the `gunbc run` verb both EVALUATE CLAIMS, and
/// each used to decide the mode at its own site: `claim_batch` defaulted to Hermetic and took
/// Wet/Record only by explicit flag, while `gunbc run` derived the mode from `--dry-run` alone and
/// let `--claim-run` fall to Wet. Two authorities for one decision drifted, and that divergence WAS
/// the defect: a `--claim-run` without `--dry-run` ran WET, so a claim whose mocked effect had no
/// bound mock silently dispatched the REAL effect against the host (the mocked journalctl ran live
/// against srv1's journal — a DESIGN §5 fail-open with a live side effect). One total function now
/// owns the mapping; both entrypoints call it.
///
/// The rule, in priority order:
///  1. An EXPLICIT mode (last `--wet` / `--hermetic` / `--record` the caller parsed) always wins —
///     that is the operator's positive authority to perform or record a real effect.
///  2. Otherwise `--dry-run` means Hermetic (its incumbent meaning).
///  3. Otherwise a CLAIM run (`--claim-run`, or `claim_batch`, which is always a claim context)
///     means Hermetic: a claim verifies against declared mocks and is never the operator's authority
///     to run a host effect. Fail-closed — a mocked effect with no bound mock refuses typed rather
///     than dispatching live.
///  4. Otherwise (an ordinary `gunbc run` with no claim/dry flag) it is Wet: an ordinary run
///     legitimately performs real effects (witnesses, the required-floor route).
///
/// Wet claims are NOT lost: they take the declared Wet carrier, `claim_batch --wet` (or an ordinary
/// `gunbc run` with no `--claim-run`), which lands in (1)/(4).
pub fn execution_mode_for(
    explicit: Option<v1_interpreter::ExecutionMode>,
    is_claim: bool,
    dry_run: bool,
) -> v1_interpreter::ExecutionMode {
    match explicit {
        Some(mode) => mode,
        None if dry_run => v1_interpreter::ExecutionMode::Hermetic,
        None if is_claim => v1_interpreter::ExecutionMode::Hermetic,
        None => v1_interpreter::ExecutionMode::Wet,
    }
}

pub fn make_eval_context(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_fixture_store(graph, source_indices, execution_mode, None)
}

pub fn make_eval_context_with_fixture_store(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
) -> v1_interpreter::InterpContext {
    make_eval_context_with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        None,
    )
}

pub fn make_eval_context_with_runtime_options(
    graph: &v1_compiler_compile::ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    execution_mode: v1_interpreter::ExecutionMode,
    fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
) -> v1_interpreter::InterpContext {
    v1_interpreter::InterpContext::with_runtime_options(
        graph,
        source_indices,
        execution_mode,
        fixture_store,
        whole_tree_published_keys,
    )
}

/// The published spelling of a claim's observed `ClaimPreemptionReachability`.
///
/// THE OPERATIONS ARE CARRIED, NOT JUST THE ARM. A reader asking "why could the deadline not
/// fire here" needs the operation name to act, and re-deriving it from the claim body is the
/// positional citation DESIGN §3 forbids. They are joined with `+` because a claim may reach
/// more than one opaque arm and reporting only the first would under-state the surface a repair
/// has to cover.
pub(crate) fn preemption_reachability_label(reach: &v1_interpreter::OpaqueHostCallReach) -> String {
    match reach {
        v1_interpreter::OpaqueHostCallReach::SurfaceUnarmed => "surface_unarmed".to_string(),
        v1_interpreter::OpaqueHostCallReach::CooperativelyPollable => {
            "cooperatively_pollable".to_string()
        }
        v1_interpreter::OpaqueHostCallReach::OpaqueHostCallUnbounded { operations } => {
            format!("opaque_host_call_unbounded:{}", operations.join("+"))
        }
    }
}

pub fn run_claim_measured(
    ctx: &v1_interpreter::InterpContext,
    closure_subject_digest: &str,
    function: &str,
) -> (ClaimOutcome, v1_interpreter::PerformanceReceipt) {
    let subject_key =
        crate::closure_identity::witness_work_subject_key(closure_subject_digest, function);
    v1_interpreter::eval_profile_reset();
    v1_interpreter::eval_subject_set(subject_key.clone());
    // PER-CLAIM, NOT PER-RUN: reach is a fact about THIS claim's evaluation, so it is cleared
    // here beside the profile and the subject rather than at floor start. The armed surface
    // survives the reset -- disarming per claim would make every claim report `SurfaceUnarmed`.
    v1_interpreter::reset_opaque_host_call_reach();
    if let Some(budget_ms) = ctx.witness_eval_budget() {
        ctx.arm_eval_deadline(budget_ms);
    }
    if let Some(budget_ms) = ctx.witness_wall_budget() {
        // Kill-at-deadline: shell waits poll this and SIGKILL at the ceiling.
        // Completion-side `wall_budget_completion_outcome` stays as the backstop.
        ctx.arm_wall_deadline(budget_ms);
    }
    let started = std::time::Instant::now();
    let cpu_started_nanos = v1_interpreter::thread_cpu_nanos();
    let steps_started = v1_interpreter::evaluator_steps();
    let fill_steps_before = v1_interpreter::shared_artifact_fill_eval_steps();
    let fill_before_nanos = shared_artifact_fill_cpu_nanos();
    let fill_wall_before_nanos = shared_artifact_fill_wall_nanos();
    let outcome = run_claim(ctx, function);
    // CPU consumed by THIS (witness-eval) thread — the budget metric, so the completion-side
    // check matches the cooperative stride-poll and neither fires on cold-I/O or contention
    // wall time.
    let measured_cpu_nanos = v1_interpreter::thread_cpu_nanos().saturating_sub(cpu_started_nanos);
    // Sampled here rather than after the report so the wall clock can be split by the same rule
    // the CPU clock is, and so the reported line and the enforced quantity read one binding.
    let measured_wall_nanos = started.elapsed().as_nanos();
    // SHARED-ARTIFACT FILL IS NOT THIS CLAIM'S MARGINAL COST (operator-line ruling, 2026-08-27).
    // Whatever this claim spent filling a memo is consumed by every later claim naming the same
    // source — one of them measured at literally 0ms in the same run because this one paid — so
    // charging it here makes a merge-blocking ceiling a function of EXECUTION ORDER rather than of
    // the tree. The floor already bills its three `[floor-phase]` warm builds to preparation with
    // `provenance=built-by-preparation` for exactly this reason; this is that rule at memo grain.
    //
    // THE COST IS SPLIT, NEVER DROPPED: `fill_cpu_nanos` is reported per claim and the two halves
    // sum to `measured_cpu_nanos`. A runaway compile cannot hide behind "it was a miss", because
    // the fill is still counted, still attributed and still visible in the receipt.
    let fill_cpu_nanos = shared_artifact_fill_cpu_nanos().saturating_sub(fill_before_nanos);
    let cpu_nanos = measured_cpu_nanos.saturating_sub(fill_cpu_nanos);
    // THE SAME SPLIT ON THE WALL CLOCK. `wall_budget_completion_outcome` below is a
    // merge-blocking ceiling, so charging it the fill made it a function of execution order in
    // exactly the way the ruling above forbids — and unlike the CPU side it had no exemption
    // argument, only an omission. Split, never dropped: both halves are reported and they sum to
    // `measured_wall_nanos`.
    let fill_wall_nanos = shared_artifact_fill_wall_nanos().saturating_sub(fill_wall_before_nanos);
    let wall_nanos = marginal_wall_nanos(measured_wall_nanos, fill_wall_nanos);
    // THE WORK MEASURE, SPLIT BY THE SAME RULE AS THE TWO CLOCKS. Evaluator steps are counted
    // unconditionally by `eval_expr`, so this delta is a property of what the claim evaluated
    // and carries no term for the machine it evaluated on. Netting the stored fills is what
    // keeps that true across execution ORDER as well: without it the claim that happened to
    // fill a shared memo would carry the fill's steps and every later reader would carry none.
    let measured_eval_steps = v1_interpreter::evaluator_steps().wrapping_sub(steps_started);
    let fill_eval_steps =
        v1_interpreter::shared_artifact_fill_eval_steps().wrapping_sub(fill_steps_before);
    let eval_steps = measured_eval_steps.saturating_sub(fill_eval_steps);
    // EITHER clock, not the CPU one. A fill that blocked on I/O can spend wall time while
    // charging almost no CPU, and under a `fill_cpu_nanos > 0` guard that fill would be
    // subtracted from the enforced wall figure and reported nowhere — a cost dropped rather
    // than split, which is the one thing the ruling above forbids.
    if fill_cpu_nanos > 0 || fill_wall_nanos > 0 {
        // REPORTED, NOT ABSORBED. Printed on its own line, per claim, whenever a fill happened,
        // so the quantity the ceiling stops charging is visible at the same grain it was measured
        // — the difference between attributing a cost and losing one. `triggered_by` is this
        // claim, which is precisely what `SharedBuildProvenance::AlreadyWarmOnEntry` records for
        // the larger shared builds: every later claim reading this artifact reads it warm, and
        // this line names who paid.
        eprintln!(
            "[floor-shared-fill] claim={function} marginal_cpu_ms={} fill_cpu_ms={} \
             measured_cpu_ms={} marginal_wall_ms={} fill_wall_ms={} measured_wall_ms={} \
             marginal_eval_steps={} fill_eval_steps={} measured_eval_steps={} \
             provenance=filled-shared-artifact triggered_by={function}",
            cpu_nanos / 1_000_000,
            fill_cpu_nanos / 1_000_000,
            measured_cpu_nanos / 1_000_000,
            wall_nanos / 1_000_000,
            fill_wall_nanos / 1_000_000,
            measured_wall_nanos / 1_000_000,
            eval_steps,
            fill_eval_steps,
            measured_eval_steps,
        );
        // THE DISCRIMINATOR FOR THE PREEMPTION-REACHABILITY COLUMN, PRINTED OVER THE POPULATION
        // THAT MAKES IT DECIDABLE. A shared fill is opaque host work by construction — that is
        // what the fill IS — so any claim reaching this line provably ran a host call, and its
        // reach record is therefore FALSIFIABLE here and nowhere else in the corpus.
        //
        // WHY THE DENOMINATOR IS PRINTED BESIDE THE REACH RATHER THAN THE REACH ALONE. On run
        // 33832137832 the column read `cooperatively_pollable` for 3602 of 3602 claims,
        // INCLUDING a witness authored to enter a listed arm, and an empty reach cannot say
        // which of its two causes produced it: the claim called no listed arm, or the hook was
        // never on the path the call took. `dispatches` separates them. Zero dispatches under a
        // claim that just paid a multi-hundred-millisecond fill is the decoration reading, and
        // it is a statement about the INSTRUMENT; a positive count with an empty reach is a
        // statement about the IDENTITY TEST, and the last name observed says which spelling to
        // compare against. Both readings are actionable and neither is available from the
        // published column.
        let (dispatches, last_name) = v1_interpreter::builtin_dispatches_observed();
        eprintln!(
            "[floor-reach-probe] claim={function} fill_cpu_ms={} reach={} dispatches={}              last_builtin={}",
            fill_cpu_nanos / 1_000_000,
            preemption_reachability_label(&v1_interpreter::opaque_host_call_reach()),
            dispatches,
            last_name.unwrap_or_else(|| "<none>".to_string()),
        );
    }
    ctx.clear_eval_deadline();
    ctx.clear_wall_deadline();
    v1_interpreter::eval_subject_clear();
    let outcome = budget_completion_outcome(ctx.witness_eval_budget(), outcome, cpu_nanos);
    let outcome = wall_budget_completion_outcome(ctx.witness_wall_budget(), outcome, wall_nanos);
    // The receipt records BOTH clocks, and records the same `cpu_nanos` value that
    // `budget_completion_outcome` just enforced against — one binding feeding both, so the
    // enforced and the recorded quantity cannot drift apart. Previously `cpu_nanos` died on
    // the line above and only wall reached the receipt, which is why the cap enforced a
    // quantity no artifact carried.
    let receipt = v1_interpreter::performance_receipt_from_witness(
        subject_key,
        function,
        wall_nanos,
        cpu_nanos,
        eval_steps,
    );
    (outcome, receipt)
}

/// Run a witness companion that returns `String` divergence detail (Lane B agreement loudness).
/// Empty string = no divergence detail (clean companion **or** companion not declared).
/// Non-empty refusal sentinel on wrong type / non-missing interpreter error — never silent
/// None when a companion *is* declared (review 41847, §5). A missing companion
/// (`NoSuchFunction` from the `_holds` → `_failure_receipt` naming convention) is
/// "not declared", not a refused receipt — the auto-derived name must not invent a
/// required loudness hook for every Bool(false) witness. This arm used to name a second
/// variant, `NoMainFunction`, because `run_in_context` reported EVERY missing named
/// function that way; that variant is deleted and the one honest arm now covers it.
pub fn run_claim_failure_receipt(ctx: &v1_interpreter::InterpContext, function: &str) -> String {
    match v1_interpreter::run_in_context(ctx, function, false) {
        Ok(Value::Str(ref s)) => s.to_string(),
        Ok(other) => format!(
            "failure_receipt_refused: {function} returned {}, expected String",
            ctx.format_value(&other)
        ),
        Err(v1_interpreter::InterpError::NoSuchFunction { .. }) => String::new(),
        Err(e) => format!("failure_receipt_refused: {function}: {e}"),
    }
}

/// THE UNWIND BOUNDARY. `catch_unwind` sits here, around the evaluation and nothing else, because
/// this is the seam that already turns "what the evaluator did" into "which terminal the claim
/// reached" — every other non-verdict is classified on the lines below, and a panic classified
/// anywhere else would be a second authority for the same question.
///
/// IT DOES NOT LICENSE CONTINUING. Capturing the payload lets the runner name the identity and
/// publish its ledger; the fold above still stops (operator ruling, 2026-08-26). Every other
/// non-verdict here is a state the producer DECIDED to return, so the process's invariants held
/// and the next claim starts from a known state; a panic is an invariant violated at an unknown
/// place, and a later row measured after it would carry an unstated precondition that no row can
/// express. Continuing would manufacture exactly the execution-provenance conflation this floor
/// exists to prevent — a green row after an unwind and a green row before it rendering alike.
///
/// The default hook still prints the panic and its location to stderr before this returns, so the
/// unwind stays as loud as it was; what changes is that it now also becomes a value.
pub(crate) fn run_claim_evaluation(
    ctx: &v1_interpreter::InterpContext,
    function: &str,
) -> Result<v1_interpreter::InterpResult<v1_interpreter::Value>, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        v1_interpreter::run_in_context(ctx, function, false)
    }))
    .map_err(|payload| panic_payload_text(&payload))
}

#[cfg(test)]
mod execution_mode_for_tests {
    use super::execution_mode_for;
    use crate::v1_interpreter::ExecutionMode;

    // --- the `gunbc run` verb: (explicit=None, is_claim=claim_run, dry_run) ---

    // THE BOUNDARY RED (the defect): a claim run with neither --dry-run nor an explicit mode must
    // be Hermetic. Pre-fix, `run --claim-run` fell to Wet and dispatched a mocked effect live.
    #[test]
    fn claim_run_defaults_hermetic() {
        assert!(matches!(
            execution_mode_for(None, true, false),
            ExecutionMode::Hermetic
        ));
    }

    // DISCRIMINATING CONTROL against an over-broad repair: an ordinary run (no claim, no dry-run)
    // stays Wet, so genuine wet execution (witnesses, the required-floor route) is not refused.
    #[test]
    fn ordinary_run_stays_wet() {
        assert!(matches!(
            execution_mode_for(None, false, false),
            ExecutionMode::Wet
        ));
    }

    // --dry-run keeps its incumbent meaning; --claim-run and --dry-run agree on Hermetic.
    #[test]
    fn dry_run_is_hermetic() {
        assert!(matches!(
            execution_mode_for(None, false, true),
            ExecutionMode::Hermetic
        ));
        assert!(matches!(
            execution_mode_for(None, true, true),
            ExecutionMode::Hermetic
        ));
    }

    // --- claim_batch: always a claim context, mode by explicit flag (last parsed wins upstream) ---

    // The default (no --wet/--hermetic/--record) is Hermetic — claims verify against mocks.
    #[test]
    fn claim_batch_default_is_hermetic() {
        assert!(matches!(
            execution_mode_for(None, true, false),
            ExecutionMode::Hermetic
        ));
    }

    // An explicit operator mode wins over the claim default and over --dry-run: this is what keeps
    // the declared Wet route (claim_batch --wet) and the record route expressible after the fix.
    #[test]
    fn explicit_mode_wins() {
        assert!(matches!(
            execution_mode_for(Some(ExecutionMode::Wet), true, false),
            ExecutionMode::Wet
        ));
        assert!(matches!(
            execution_mode_for(Some(ExecutionMode::Record), true, false),
            ExecutionMode::Record
        ));
        assert!(matches!(
            execution_mode_for(Some(ExecutionMode::Hermetic), true, false),
            ExecutionMode::Hermetic
        ));
        // explicit --wet beats --dry-run (operator's positive authority to run the effect).
        assert!(matches!(
            execution_mode_for(Some(ExecutionMode::Wet), true, true),
            ExecutionMode::Wet
        ));
    }
}
