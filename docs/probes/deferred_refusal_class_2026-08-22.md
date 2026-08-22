# The compiler knew and declined to stop the line — two instances, one class (2026-08-22)

Filed, not chased. Two independently-found defects with different mechanisms and one shape: **a
refusal the compiler was in a position to make, deferred until something far away trips over it.**
Both were found while measuring T3 (`docs/probes/t3_collection_realization_2026-08-22.md`); neither
is a T3 defect. The rows cite each other deliberately — the class is what should rank for repair,
not either instance alone.

## Instance A — a typed refusal written into the artifact while the compile reports success

**Mechanism.** `gunbc compile` emits `compile_error!("…")` into the generated Rust and still prints
`0 blocking error(s), N advisory diagnostic(s)` and **exits 0**.

**Measured.** Renaming `std.algebra` `PointwisePower`'s only field made every `.member(…)` call site
emit `compile_error!("method member is neither a resolved callable receiver field nor a registered
v1_rt bridge function")`. The marker count went **0 → 20** across the emitted tree while the compile
verdict stayed clean; the per-file join against the authored `.member(` census was exact
(1/5/1/1/4/8).

**Why it is below the floor.** The diagnostic is typed and located — the compiler *had* the refusal
— and it leaves as output rather than as a verdict. Anything counting blocking diagnostics sees a
clean compile; `rustc` discovers the problem two stages later.

## Instance B — a builtin that answers "not me" and silently re-enters the `.dag` body

**Mechanism.** The interpreter grounds `v2.std.collection` `map_insert` on the native map shape
only. `eval_builtin`'s `free_call.map_insert` arm matches `[Value::Map, k, v]` and answers
`Ok(None)` for any other receiver; `try_v2_std_collection_map_primitive_grounding` turns that
`Ok(None)` into `None` and falls through to the `.dag` body.

**Measured.** With `map_insert` delegating to its primitive, a record-shaped map (from any
`Map { lookup: … }` literal) recursed to the depth limit: `call depth exceeded 100000 at fn
'map_insert_primitive_delegate'`, 14 floor witnesses. Two-arm probe, one variable: a map from
`empty_map()` grounds and evaluates; a map from a record literal recurses.

**Why it is below the floor.** An unmatched receiver shape yields **neither a value nor a refusal**.
The honest answer available at that point is a typed, located shape mismatch at the call; instead
the failure surfaces 100000 frames away, where it names the delegate rather than the shape fork.
This fall-through is precisely what let one carrier be two shapes with nothing ever saying so.

## Why one class

Neither is a wrong answer; both are a **withheld** answer. In A the refusal is produced and
misrouted into the artifact; in B the refusal is never formed because "cannot handle this shape" is
encoded as `Ok(None)`, which the caller reads as "not applicable". Both are DESIGN §5's line-stop
failure, and both share the tell: **a consumer downstream of the deferral discovers the defect, and
the deficit's frequency on our side is zero by construction** — nothing counts a refusal that left
as output, and nothing counts a fall-through that looks like a miss.

`Ok(None)` meaning both *not applicable* and *cannot handle this input* is the not-applicable /
malformed conflation the recurring-failure list already names, appearing here on a dispatch result
rather than a parse result.

## Status

Both filed at *mitigatable*: the failures are loud when they finally land (a `rustc` error; a
refused recursion) and neither fabricates a plausible value. Neither has a next-rung trigger yet,
because neither has been traced to the decision site that chooses to defer. Not chased here — this
document is the row, and the two instances are its evidence.
