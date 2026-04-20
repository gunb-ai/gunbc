### E-6: No target-spec field without a same-PR consumer (2026-04-16)

A field declared on a target spec (`CallableRealization`,
`TargetExecutionModel`, `PatternRealization`, etc.) MUST be read by
at least one emitter in the same PR that introduces it. Speculative
target-spec fields — declared, parsed, stored, and then never
consulted during emission — become advisory metadata. Once the
"declare now, consume later" pattern is normalized, every subsequent
target spec inherits the same optionality, and the thesis claim
"targets are declarations, emission is mechanical translation" drifts
back toward aspiration.

**Why this is a specialization of "Emission is translation."**
"Emission is translation" already says the emitter reads LanguageSpec
data instead of making decisions. E-6 closes the other direction:
spec data must be authoritative, not advisory. A field the emitter
ignores is indistinguishable from a field the emitter doesn't know
about. Both teach the wrong architecture.

**The canonical counter-example (PR #490 diff):** a regression test
`go_gc_targets_skip_rendering_model_loading` deliberately corrupted
`go_rendering` and asserted emission succeeded. The test codifies
"declared target fact is non-authoritative" as a unit-tested
property — the clearest possible smoking gun.

**Bounded exception:** a target spec field MAY land without an
emitter consumer if paired with an explicit **dissolution ratchet**
naming the PR or lane that will wire consumption. Without the
ratchet, the field is speculative metadata.

**Structural prevention:** at PR review, grep every new field added
to a target-spec record type against the emitter source tree. Every
new field must either (a) have at least one consumer call site added
in the same diff, or (b) sit behind a named scaffold marker with a
dissolution trigger.

**Test:** the PR adding a new target-spec field must include at
least one test that fails if the field is mis-populated. A test that
asserts "spec is ignored" is a reverse signal.

