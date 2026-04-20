### No fallbacks that fabricate

Every code path either succeeds fully or fails with a clear error.
No silent degradation: no `.ok()` that swallows errors, no `continue`
that silently drops work, no fallback defaults that produce
valid-looking but wrong output. If a function cannot complete its job,
it must return `Err`.

Fabrication fallbacks are the mechanism by which duplicate
representations and missed enumerations become invisible. They convert
hard failures into silent wrong behavior.

Sample: ownership should not compile to
`Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`. Either the
compiler proves a single semantic consumer and emits the move, or it
surfaces that the proof is missing. The clone branch is a fallback,
even if it preserves correctness.

**Structural prevention:** Typed boundaries that can't represent error
states. `InferredNode = Resolved { node } | CompilerError { ... }`
already does this for inference — you can't accidentally treat an error
as a valid type. The same pattern applies to the emit boundary: emit
receives a type that can't represent error-contaminated nodes. If
inference failed, the node doesn't reach emit — not because a gate
checked for errors, but because the type system makes it
unrepresentable. The escape hatch is `String`/`Node` types that can
carry error sentinels (`"<error:...>"`, `Dynamic`, `LitNull`); the fix
is wrapper types where the error case is structurally distinct.

