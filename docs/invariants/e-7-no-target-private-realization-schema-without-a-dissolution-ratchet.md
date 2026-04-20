### E-7: No target-private realization schema without a dissolution ratchet (2026-04-16)

When a new emission target lands, it MUST consume the shared
realization schema (`v3.std.emit_model.TypeRealization`,
`CallableRealization`, `OperatorRealization`, `PatternRealization`,
etc.) unless the shared schema is missing a fact the target
genuinely needs. In the latter case, the shared schema extends to
carry that fact. Creating a parallel target-private realization
family (`PythonTypeRealization`, `PythonCallableRealization`, etc.)
is permitted only as a time-bounded scaffold with an explicit
dissolution trigger that names the PR or lane converting the target
onto the shared schema.

**Why.** Parallel representation drift is the class of debt that
"No duplicate representations" invariant already forbids in general.
E-7 specializes it to the realization surface, where the temptation
is strongest — each new target emitter starts easy with a private
schema, then calcifies as the consumer count grows. Without a named
dissolution trigger, "this is staged for now" is indistinguishable
from "this is the intended end state."

**The canonical incident (2026-04-16):** `src/v3/spec/python.dag`
declared `PythonTypeRealization`, `PythonOperatorRealization`,
`PythonCallableRealization`, `PythonTypeInstantiationRealization`,
and `PythonPatternRealization` as a second realization family
parallel to the shared `v3.std.emit_model`. Prose dissolution notes
existed on two of the private strategy enums
(`PythonCallableStrategy`, `PythonPatternStrategy`) but not on the
five realization records themselves. The mismatch between "scaffold
documented for two enums" and "scaffold undocumented for five
records" is the smoking gun — without mechanical gating, prose
discipline drifts.

**Structural prevention:** CI grep gate — every target-private
realization data type (match on suffix: `*TypeRealization`,
`*OperatorRealization`, `*CallableRealization`,
`*PatternRealization`, `*TypeInstantiationRealization`) outside of
`v3/std/emit_model.dag` MUST have an adjacent
`🟡 SCAFFOLD ... dissolves when ...` comment with a trigger that
names a specific follow-up. A grep that returns hits without
corresponding dissolution comments fails CI.

**Dissolution trigger shape.** The trigger must name a PR, lane, or
concrete engineering task — not a vague "future consolidation."
Acceptable: "dissolves in Lane 1e (consolidation)," "dissolves when
emit_python migrates to shared CallableRealization (PR TBD)."
Unacceptable: "dissolves when emit_python matures," "eventually
folds into shared schema."

**Test:** renaming a variant or field in the shared schema must
either update the parallel target-private schema in the same PR or
fail closed with a diagnostic. Silent divergence is the hazard this
invariant prevents.

