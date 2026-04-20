### E-8: Unsupported core behaviors fail closed, never collapse semantically (2026-04-16)

When an emitter reaches a `Behavior` variant (Value, Transform,
Branch, Loop, Bind) for which it has no rendering strategy, it MUST
return a structured `UnsupportedBehavior` error. Silently rendering
a substitute — for example, "render the loop body's result port" in
place of a Loop construct — collapses iteration semantics into a
single-iteration expression and produces emitted code that compiles
but is structurally wrong. The class of silent semantic collapse is
strictly worse than a hard error, because the output passes
downstream checks that assume correctness.

**Why this is distinct from other fail-closed rules.** The
"Fail-closed compilation" invariant (on missing rendering
annotations) and the "Emission is translation" fail-closed paragraph
both cover the case where the emitter lacks a spec entry. E-8
specializes the rule to `Behavior` variants: the emitter has a
`match` on the substrate's closed-set behavior enum, every arm must
either emit correct code or return an error. A fallthrough arm that
emits *something* is a fact-drop at the substrate boundary.

**The canonical incident (2026-04-16):** `emit_go` and `emit_python`
both implemented `Behavior::Loop` as "render the loop body's result
port," which silently collapsed `fold(list, init, f)` into the
first iteration's expression (`f(init, head(list))`) — a Loop over
a list became its first iteration's expression. The collapse was
detected only by CI on a deeply-recursive .dag fixture; most
fixtures happened to have loop bodies that looked plausible when
rendered in isolation. The principle audit caught the class three
review rounds running before the fix landed.

**Structural prevention:** every emitter's behavior-dispatch match
must enumerate all five variants explicitly, and every arm must
either return a correctly-emitted expression OR return an
`UnsupportedBehavior`-class error. Fallthrough arms, default arms,
and arms that emit structural substitutes are forbidden.

**Test at PR review:** grep each emitter for `match * { ... }`
dispatches on `Behavior`. Every arm that does not emit a faithful
construct must return an error. A comment `// TODO: support Loop
properly` over a silent substitute is an E-8 violation — the
comment is prose documentation, not a structural gate.

**Tests that were passing by accident (2026-04-16 remediation):**
when `emit_go` and `emit_python` started fail-closing on Loop,
three tests previously passing through the silent collapse
(`go_lens_unused_parameters_module`,
`emit_python_module_marks_ownership_as_skipped_for_gc_target`,
`emitted_python_lens_matches_emitted_rust_lens_on_reflected_programs`)
were marked `#[ignore]` with the explicit reason *"blocked on
emit_<lang> Behavior::Loop support; previously passed via silent
loop-body collapse."* This is the right shape for E-8 debt: the
ignore ledger names the blocking substrate capability, so the
test's re-enable condition is unambiguous.

