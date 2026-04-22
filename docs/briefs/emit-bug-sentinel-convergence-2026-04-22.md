# `__EMIT_BUG` sentinel convergence — findings

**Date:** 2026-04-22
**Lane:** F (`S`)
**Status:** STOP-AND-ESCALATE (clause #1 in brief)
**Disposition:** no template changes in this PR; finding surfaces a target-capability asymmetry and points to the correct dissolution, which lives deeper than templates.

## Scope recap

Three emit specs fabricate sentinel identifiers on error-type emission:

| file | line | template |
|---|---|---|
| `dsl/extdeps/languages/dag/emit.dag` | 54 | `"__EMIT_BUG_{0}__"` |
| `dsl/extdeps/languages/go/emit.dag` | 107 | `"__EMIT_BUG_{0}__"` |
| `dsl/extdeps/languages/python/emit.dag` | 115 | `"__EMIT_BUG_{0}__"` |
| `dsl/extdeps/languages/rust/emit.dag` | 141 | `"compile_error!(\"{0}\")"` |

Consumer: `src/v2/05_emit.dag:996, 1046` — `apply_type_template1(template: spec.error_type_template, arg0: "UNRESOLVED_" | "ANONYMOUS_COPRODUCT")`. Fires in **type position** when the graph has an unresolved type variable, a compiler-error node, or an anonymous coproduct.

## Per-target capability review

### Rust — has the escape hatch
`compile_error!("...")` is a macro that fails at macro expansion *regardless of syntactic position* (including type position). It is Rust's native "halt the build with a message" primitive.

### Python — **not fail-closed** in the current emitter
- The Python emitter unconditionally prepends `from __future__ import annotations` (`src/v2/05_emit_python.dag:301`, mirrored in `src/v2/stage0/src/v2_compiler_emit_python.rs:528`, and explicitly locked in by `src/v3/compiler/tests/boundary/m1_4_emit_python_test.rs:553`).
- Under PEP 563 deferred evaluation, annotations are stored as **string literals** and never evaluated at import. `__EMIT_BUG_UNRESOLVED_TypeVariable__` in a type annotation therefore **imports cleanly** — failure is deferred to whatever runtime reflection (typed dispatch, `get_type_hints`, a type checker) later tries to resolve it.
- In expression position Python has native fail-closed forms (`raise`, or `python_error_expr_template = "raise RuntimeError({0})"` at `dsl/extdeps/languages/python/emit.dag:111`). In annotation position, given the emitter's deferred-annotation policy, **no valid-expression template fails at import** — any syntactically-valid expression is stored as a string and never evaluated. (A deliberately malformed template would fail at parse with `SyntaxError`, but that's a file-level parse failure, not an annotation-evaluation fail-closed; the message would not carry the `{0}` label through cleanly, and it couples the fail-closed guarantee to the parser rather than to type resolution.)
- This is a genuine gap: Python is the one target where the brief's acceptance criterion ("each target fails at its own compile time if the template fires") is **not** met by any well-formed annotation-valued template under the emitter's current deferred-annotation policy.

### Go — shadowable, not truly fail-closed
- Go has no macros, no `#error`, no `static_assert`.
- An undefined identifier in type position yields a Go compile error (`undefined: __EMIT_BUG_FOO__`) — **only if the name is actually undefined**. Go identifier syntax (letter-or-`_` start) accepts `__EMIT_BUG_UNRESOLVED_TypeVariable__` as a valid user identifier. If any source file in the same package (user code, generated code, a second error-emitting site in the same compilation, or accidental collision with another tool's sentinels) declares a type of this name, the emission resolves cleanly and the error is silently swallowed.
- This is a C-8 violation, not mere legibility: the sentinel relies on a *negative* property ("no one has declared this name") rather than a *positive* fail-closed mechanism.

### dag — shadowable, not truly fail-closed
- `.dag` identifier rules (`01_tokenize.dag:516-522`) accept `_` as an ident start and `__EMIT_BUG_*__` is a well-formed `.dag` identifier. The dag compiler's unresolved-name diagnostic is what makes the sentinel bite today, but the same shadowing concern as Go applies: any declaration of the sentinel name in scope satisfies the reference.
- The dag parser does not reserve `__*__` forms. `NAME_LIKELY_UNUSED_STOP_SYMBOL`-style conventions are not structurally enforced here.

## The real observation

**Only Rust is genuinely fail-closed.** `compile_error!` is a macro invocation that fails at macro expansion regardless of surrounding declarations — no user code can shadow or intercept it.

**Python, Go, and dag all have live fail-closed gaps**, of two different shapes:

- **Python:** the emitter's `from __future__ import annotations` policy (PEP 563) stores annotations as strings, so the sentinel never evaluates at import. Gap is *deferred evaluation*.
- **Go and dag:** the sentinel is a syntactically-valid user identifier. If any declaration of that name exists in scope (user code, generated code, another emission site in the same compilation, cross-tool collision), the sentinel resolves and the error is silently swallowed. Gap is *shadowable identifier*.

Both gaps are **C-8 violations**: the sentinels rely on negative properties ("annotations aren't evaluated," "no one has declared this name") rather than positive fail-closed mechanisms. The previous framing that classified Go/dag as "legibility, not fail-closedness" was wrong — a name that can be shadowed is not fail-closed, it's a weak bridge that happens to work by convention.

## The dissolution that actually works

Per `feedback_fail_closed_discipline` (C-8) and `feedback_construction_over_ratchets`, the correct model is:

> The template firing means the compiler already knows an error exists and chose to **emit sentinel output** rather than surface a Diagnostic and halt.

The real dissolution is **upstream**: when `05_emit.dag:996, 1046` detects the error condition (`n_is_type_var || n_is_error`, or the anonymous-coproduct branch), it should produce a `Diagnostic` and halt emission — not emit a target-specific error sentinel. Under that model:

- `error_type_template` becomes unreachable on any well-formed compilation.
- The four per-target templates can be deleted entirely.
- `compile_error!` is a clever bridge that preserves target-compile failure *because the upstream halt is missing*; once upstream halts properly, no target needs a bridge.

This is Lane-C1 territory (model the halt upstream; delete the bridge) and explicitly out of this lane's scope ("no change to the emission dispatch logic").

## Why not paper-over renames

Options that were considered and rejected:

1. **Rename to a prefix-with-comment form** (e.g. Go: `/* EMIT BUG: UNRESOLVED_TypeVariable */ __EMIT_BUG_UNRESOLVED_TypeVariable__`). Adds legibility, no fail-closed change. Still a sentinel.
2. **Python: switch to raising-lambda expression**. Fails at annotation evaluation when annotations are evaluated — but is silently skipped under `from __future__ import annotations`. Weaker than the current form under that mode, not stronger.
3. **Go: switch to syntactically invalid identifier** (e.g. `!EMIT_BUG`). Breaks Go parse — worse failure mode than the current "undefined identifier with a helpful name."

None of these move the real needle (upstream halt). All of them add churn without changing the invariant receipt.

## Recommendation

1. **No template changes in this PR.** All three non-Rust sentinels have real C-8 gaps (Python deferred-annotation; Go/dag shadowable identifier). A rename to a still-valid-identifier form does not close the shadowing gap. Only upstream dissolution does.
2. **Three targets have live P3/C-8 gaps today.** Python via `from __future__ import annotations`; Go and dag via shadowable identifier. No template-only edit closes any of them:
   - For Python, any valid annotation expression is stored as a string and never evaluated.
   - For Go/dag, any identifier the template could emit is user-shadowable in the same package/module.
   The correct fix lives upstream (see #3). Emitter-policy workarounds (dropping the future-import; reserving an `__*__` identifier class in `.dag`) are larger changes than the upstream halt and less structurally sound.
3. **Open a C1 lane** (or attach to an existing Diagnostic-propagation lane) to dissolve the error-type emission path entirely: the condition at `05_emit.dag:996, 1046` should produce a Diagnostic and halt. When that lands, all four templates can be deleted — including Rust's `compile_error!` bridge — and all three gaps (Python deferred-annotation; Go/dag shadowable identifier) close by construction.
4. **Document the receipt**: this file serves as the explicit surfaced finding per STOP-AND-ESCALATE clause #1 ("target-capability limitation worth surfacing explicitly, not papering over"), and flags three live C-8 gaps (Python, Go, dag) that the upstream-halt dissolution resolves.

## PR framing

This PR delivers the finding artifact only, per the brief's escalation path. The modeling-correct fix is upstream and out of scope.
