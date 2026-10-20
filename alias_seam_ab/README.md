# alias-seam A/B instrument (saved before its probe refs were deleted, 2026-10-20)

Answers: "does substituting in `transparent_alias_rhs` move the diagnostic produced at
`variant_reference_inferred_node`?" (stern-bat-401 msg_11dcdef2(b)).

RUN IT
    cd <repo root> && bash gate_alias_seam_ab_14152.sh <ref-name> <tag>
`<ref-name>` is a git ref name the runner can clone, NOT a SHA (a SHA gave a false VOID once).
It builds once and runs twice, toggling `GUNBC_ALIAS_GUARD_OFF`; it exits VOID if the env gate
is absent from the built tree, so a silent no-op can never be read as a control.

THE ONLY FILE THAT MATTERS IS THE ONE THAT GETS COMPILED
`env_gate.patch` gates the guard in the **compiled mirror** `src/v1/stage0/src/v1_compiler_infer_env.rs`
ONLY — not in `src/v1/04_env.dag`. That is deliberate and it is also the correction I owed
stern-bat-401: the executed binary is built from the mirror, and the gate's VOID check greps the
mirror (`gate_alias_seam_ab_14152.sh`, the `grep -q GUNBC_ALIAS_GUARD_OFF` line), so the setting
that was toggled provably was the setting that executed. A `.dag`-side A/B would need the `.dag`
patched and the mirror re-emitted by the emitter, never hand-edited.
`constant_false_mutant.patch` is the separate stronger mutant (guard term constant-false in BOTH
files) kept for re-derivation; it was not the one used for the verdict.

`alias_seam_ab_test.rs` calls the same host builtin the claim module uses
(`compile_dag_diagnostic_census`), prints every measurement unconditionally, and reserves `-1` for
UNMEASURED (PANIC / NotRunnable) so an instrument refusal is never scored as "0 blocking diagnostics".

RESULT (one binary, two settings, both bands in a single job)
`alias_return_is_a_refusal_fixture` (`fn asks() -> List<Int> { [1,2] |> fold(init: Empty, f: ...) }`)
    CONTROL (guard live): ARM=Observed rows=24 blocking_rows=0 blocking_total=0
    MUTANT  (guard off):  ARM=Observed rows=24 blocking_rows=0 blocking_total=0
Identical by identity across 24 rows; no `MethodNotFound` row and no subject naming
`Coproduct(FreeMonoid)` or `element` in EITHER setting. So the guard is not on the causal path:
the answer is NO.
`alias_field_...` and `alias_initializer_...`: `NotRunnable` ("render selection refused: none of the
5 compiled module(s) is declared in fixture source 'test.dag'") identically in both settings — an
instrument limit of this one-synthetic-module builtin, named as a residual, not evidence either way.

STANDING FINDING FOR CHARTER 4
`dag/test/claim/algebra_receiver_alias_witness.dag` is DECLARED, not executed: `known_red_probe`
has no reader, `transitional_admission_exception_identities` is consumed only by
`gunbc.explicit_witness_admission`, and no workflow enumerates `dag/test/claim`. Had it been run,
`alias_return_is_a_refusal_fixture` would fail today: it asserts >= 1 blocking `MethodNotFound` and
measures 0 through its own builtin.
