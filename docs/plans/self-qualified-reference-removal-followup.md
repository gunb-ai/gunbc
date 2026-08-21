# Follow-up: 9,532 self-qualified references across 766 modules

**Filed 2026-08-21. NOT actioned on `integration/namespace-cut`.** This records a
measured §2 opportunity and, more importantly, retracts the evidence that was
previously believed to stand against it.

## The population

A **self-qualified reference** is a module naming its own declaration with its own
module prefix:

```dag
module std.algebra
type FreeMonoid<T>
...
  keys: fn() -> std.algebra.FreeMonoid<K>    // the prefix names the module we are in
```

Main writes the same field as `keys: fn() -> FreeMonoid<K>`.

Measured on this branch: **9,532 self-qualified references across 766 modules.**
Largest: `v2.std.compilers.target_model` (786), `extdeps.git.object_store` (405),
`extdeps.pijul` (336), `v1.std.core` (309).

This is duplicated work at the meaning layer, 9,532 times — §2, horizontal.

## The evidence that was believed to stand against removal — RETRACTED

An earlier experiment de-qualified 61 self-references in `dag/std/types.dag` and was
reported as **refusing the floor**. That report was used, by me, to argue that
self-qualification is load-bearing at the resolver and that the emitter must
therefore absorb it.

**Both halves are now falsified by measurement:**

1. **The resolver is correct.** A three-arm control on main, with a *shape
   discriminator* so a pass identifies *which* declaration resolved:

   | arm | configuration | result |
   |---|---|---|
   | A | local declaration + bare self-ref, **with** a same-named pool competitor | **PASS — resolved LOCAL** |
   | B | local declaration, no competitor | PASS |
   | C | no local declaration, name only in pool | PASS |

   Local scope shadows the pool. Also: `NonEmptyStr`, the type in the original
   experiment, is declared in **exactly one module corpus-wide** — there was no
   competitor to shadow it, so that mechanism could not have applied to the case that
   motivated it.

2. **The refusal does not reproduce.** Same file, same 38 real self-references
   de-qualified, whole-pool floor frame, head `ad68c81e`: **0 refusals**, preparation
   passed, the fold reached `evaluating 5000 / 9909`, and the run ended on a 720s
   timeout rather than a refusal.

## Why the original attribution was wrong

Not an invented mechanism — an **invented cause**. The de-qualification was genuinely
present in the tree when the floor genuinely refused; only the attribution was wrong.
That window was an active integration in which four separate defect classes were later
found, each of which I was manufacturing at the time: re-exporter qualifications,
param-name mis-qualification, the pool-pull keystone, and a mangling qualifier.

> **An attribution made inside an active integration is provisional by construction.**
> Anything blamed during a merge window must be re-blamed on a quiet tree before it
> becomes a reason for anything.

## Honest scope of the result

The reproduction attempt covered **38 sites in one file**, not 9,532 across 766
modules. It establishes that the specific evidence against removal does not hold. It
does **not** establish that corpus-wide removal is safe.

## Why this is not being actioned here

`integration/namespace-cut` already carries a continuous main integration and a stacked
emitter PR. A 9,532-site corpus edit landing beside them would make every subsequent
attribution on this branch exactly as unreliable as the one this document retracts.

## What would action it

A separate branch on a quiet tree: de-qualify per-module, floor frame per batch,
attribute per batch. The prize is 9,532 redundant qualifications removed and a
correspondingly smaller input population for the qualified-type emitter — which
*shrinks* that problem without solving it, since cross-module qualifications remain
either way.
