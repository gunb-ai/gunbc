# Pre-registered verdict criteria for ab_emit_14152.sh (ab1) — written BEFORE reading the output

Run: emit the closure of `dag/std/realization_measurement.dag` with gunbc built from main
(2817c10af02) and from PR-1 head (e5cc5d903e1). Sources are identical; only the compiler differs.

FACTS ALREADY ESTABLISHED (do not re-litigate): `std_realization_measurement.rs` is emitted-only
(untracked on main); the CI message truncates at the first `@` (`... not satisfied @ src/...`), so a
mint id would be invisible in the message; the blamed emitted line 341 col 72 sits on the `.clone()`
inside `witness_measured_rollup_differs_from_predicted_zero<S>`; the callee
`pub fn measure_count<Q, S, M: Clone>(m: Rc<Measure<Q, S, M>>) -> M` (src/v1/stage0/src/std_measure.rs:270)
is the only `Clone` in the emitted file (`S_clone_bound` count 0 for the file, so the requirement is
on the callee side).

V1 — If EMITTED_DIFF_RC != 0 and the diff touches `witness_measured_rollup_differs_from_predicted_zero`
or any line with `S`, the red IS caused by my change, and I must narrow the mint (not widen).
Sub-case V1a: the emitted side carries a mint id (`S@...` or a `K@file:start-end` type argument) →
the mint leaks into a generic type argument, so a call site whose actuals are the *caller's* own type
variables is being re-instantiated when it should not be. Repair: mint only when the callee is generic
AND the actuals supply no binding for `g` AND the call's expected type does not itself contain `g`;
prefer not minting at all over minting wrongly.
Sub-case V1b: the emitted text differs but contains no mint id → my change altered a *type decision*
elsewhere; read the diff line by line before proposing anything.

V2 — If EMITTED_DIFF_RC == 0 (emitted text byte-identical) and main's emitted file ALSO lacks the
bound, then the red is NOT caused by my mint. Then the difference must be in what gets *included* in
the emitted crate (which modules reach the closure), and I look at closure membership, not at types.
I do NOT touch the mint in that case.

V3 — If main's emit fails or `--entry` cannot reach that module, the A/B is VOID; report it and stop.
No conclusion from a run where one side did not emit.

V4 — GUARD lines must read [a] namekeyed=1 span_keyed=0 and [b] namekeyed=1 span_keyed=1. If not,
the run measured the wrong trees and is VOID.
