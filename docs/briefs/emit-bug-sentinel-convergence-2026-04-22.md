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

### Python — no direct equivalent in annotation position
- Annotations are expressions. The only failure modes available in expression position are (a) undefined identifier → `NameError` when the annotation is evaluated, or (b) expressions that raise when evaluated (e.g. `(lambda: (_ for _ in ()).throw(RuntimeError("EMIT BUG: {0}")))()`).
- Under `from __future__ import annotations` (default in many modern Python codebases) annotations are stored as strings and **never evaluated at import**, so both (a) and (b) defer failure to runtime type-checkers or typed reflection — weaker than Rust's guarantee.
- The current `__EMIT_BUG_{0}__` sentinel is mode (a). It **is** the native fail-closed form Python has to offer in this position; there is no stronger Python construct.

### Go — no equivalent in type position
- Go has no macros, no `#error`, no `static_assert`.
- An undefined identifier in type position **does** yield a Go compile error (`undefined: __EMIT_BUG_FOO__`). The current sentinel is mode "undefined identifier" — which is Go's native fail-closed form in this position.
- There is no Go construct that carries a message through a compile error in type position.

### dag — self-referential
- `.dag` emitted with `__EMIT_BUG_FOO__` in a type position is rejected by the dag compiler (undefined identifier → diagnostic). Same class as Go.
- Could plausibly emit a Diagnostic carrier instead, but this is the *emit* spec — a Diagnostic at this layer is equivalent to the upstream-halt argument below.

## The real observation

All four sentinels **are** fail-closed in their target's native sense — they all guarantee the downstream compile/import step fails. The asymmetry is only that Rust has a construct that **carries the error message through**. Python/Go/dag have only "undefined identifier" to work with, and its text cannot be arbitrary.

So the brief's acceptance criterion — "each target fails at its own compile time if the template fires" — is **already satisfied** today. What the current sentinel violates is not fail-closed, it's *legibility*: the sentinel is a valid identifier, so (a) it reads like regular code in diffs, and (b) a user could theoretically shadow it by declaring `__EMIT_BUG_UNRESOLVED_TypeVariable__` themselves.

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

1. **No template changes in this PR.** The sentinels are already the native fail-closed form for Python/Go/dag.
2. **Open a C1 lane** (or attach to an existing Diagnostic-propagation lane) to dissolve the error-type emission path entirely: the condition at `05_emit.dag:996, 1046` should produce a Diagnostic and halt. When that lands, all four templates can be deleted — including Rust's `compile_error!` bridge.
3. **Document the receipt**: this file serves as the explicit surfaced finding per STOP-AND-ESCALATE clause #1 ("target-capability limitation worth surfacing explicitly, not papering over").

## PR framing

This PR delivers the finding artifact only, per the brief's escalation path. The modeling-correct fix is upstream and out of scope.
