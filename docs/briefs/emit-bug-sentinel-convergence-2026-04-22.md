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
- In expression position Python has native fail-closed forms (`raise`, or `python_error_expr_template = "raise RuntimeError({0})"` at `dsl/extdeps/languages/python/emit.dag:111`). In annotation position, given the emitter's deferred-annotation policy, **no Python construct fails at import**.
- This is a genuine gap: Python is the one target where the brief's acceptance criterion ("each target fails at its own compile time if the template fires") is **not** currently met.

### Go — no equivalent in type position
- Go has no macros, no `#error`, no `static_assert`.
- An undefined identifier in type position **does** yield a Go compile error (`undefined: __EMIT_BUG_FOO__`). The current sentinel is mode "undefined identifier" — which is Go's native fail-closed form in this position.
- There is no Go construct that carries a message through a compile error in type position.

### dag — self-referential
- `.dag` emitted with `__EMIT_BUG_FOO__` in a type position is rejected by the dag compiler (undefined identifier → diagnostic). Same class as Go.
- Could plausibly emit a Diagnostic carrier instead, but this is the *emit* spec — a Diagnostic at this layer is equivalent to the upstream-halt argument below.

## The real observation

Three of the four sentinels (Rust, Go, dag) are fail-closed in their target's native sense — they guarantee the downstream compile step fails. The asymmetry is that Rust carries the message through (`compile_error!`); Go/dag rely on "undefined identifier."

**Python is the outlier.** The emitter's `from __future__ import annotations` policy (PEP 563) stores annotations as strings, so the sentinel never evaluates and never fails. The acceptance criterion is **not** currently met for Python — this is a real P3 gap, not just a legibility concern.

For Go and dag, what the sentinel violates is legibility, not fail-closedness: the sentinel is a valid identifier, so (a) it reads like regular code in diffs, and (b) a user could shadow it by declaring it themselves.

## The dissolution that actually works

Per `feedback_fail_closed_discipline.md` (C-8) and `feedback_construction_over_ratchets.md`, the correct model is:

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

1. **No template changes in this PR.** For Go/dag the sentinels are already the native fail-closed form; renaming them is a legibility tweak that doesn't change the receipt. For Python no template change can close the gap on its own — the `from __future__ import annotations` policy short-circuits any annotation-position expression.
2. **Python has a live P3 gap today.** The correct fix lives upstream (see #3) and/or at the emitter-policy layer: either stop emitting error types at all, or revisit the unconditional `from __future__ import annotations` for files where error-type sentinels could appear. Either way it's not a template edit.
3. **Open a C1 lane** (or attach to an existing Diagnostic-propagation lane) to dissolve the error-type emission path entirely: the condition at `05_emit.dag:996, 1046` should produce a Diagnostic and halt. When that lands, all four templates can be deleted — including Rust's `compile_error!` bridge — and the Python gap closes by construction.
4. **Document the receipt**: this file serves as the explicit surfaced finding per STOP-AND-ESCALATE clause #1 ("target-capability limitation worth surfacing explicitly, not papering over"), and flags Python's deferred-annotation gap as a live P3 concern that the upstream-halt dissolution resolves.

## PR framing

This PR delivers the finding artifact only, per the brief's escalation path. The modeling-correct fix is upstream and out of scope.
