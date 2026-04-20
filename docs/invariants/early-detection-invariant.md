## Early Detection Invariant

Alert others of problems as soon and as loudly as physically possible.

Errors detected at stage N must not survive silently to stage N+1.
If the compiler knows something is wrong — a type mismatch, an
unresolved name, an inference failure — it must report it at the
stage where the information is first available. Deferring errors to
later stages (or worse, to emitted code) is a design failure: it
hides the root cause behind cascading symptoms in a different context.

**The rule:** every stage boundary is a gate. Facts that are wrong
or missing must produce diagnostics at the stage that owns the fact.
If an inference failure reaches the emitter, the emitter should
`compile_error!` — but the real fix is always upstream, in the stage
that failed to resolve the fact.

**No warnings.** Every diagnostic is either an error (compilation
stops or emitted code is structurally wrong) or absent (compilation
succeeds). There is no warning severity. A condition that is wrong
enough to report is wrong enough to fail. Warnings create a class of
"known-bad but tolerated" state that erodes invariants over time — if
the compiler knows something is wrong, it must refuse to proceed, not
annotate and continue. If a condition is truly harmless, it is not a
diagnostic. If it is harmful, it is an error.

**Corollary:** emitted code should never fail to compile due to
errors the compiler could have caught. If `cargo check` on emitted
Rust finds type mismatches, those are emission bugs — the compiler
had the type information and lost it during rendering.

