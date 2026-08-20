# A branded carrier can compile clean and refuse to be built

Found while landing `std.durable_compare_and_set` (fabric B). Recorded because
the compiler and the interpreter disagree about the same expression, and the
disagreement is invisible until something executes.

## The observation

`std.durable_compare_and_set` and its witnesses compile with **0 blocking
errors**. Several constructions then refuse at evaluation:

```
cause: TypeError { msg: "cannot cast Int to CasGeneration" }
cause: TypeError { msg: "cannot cast String to ProbeBrandStr" }
cause: TypeError { msg: "cannot cast Int to ProbeBrandInt" }
cause: TypeError { msg: "cannot cast Int to Int" }
```

Two of these deserve attention on their own.

`cannot cast Int to Int` came from `g as Int` where
`CasGeneration = Int where range(min: 1), brand("CasGeneration")`. The
interpreter resolves the brand to its carrier and then refuses the resulting
identity cast — the cast that makes a branded value usable as its own carrier
is the cast that fails.

`cannot cast Int to ProbeBrandInt` was produced by a declaration containing **no
cast at all**: `data probe_data_int_bare: ProbeBrandInt = 7`. So the coercion is
implicit, and deleting the `as` does not avoid it.

## The executed matrix

Every cell below was run. Empty cells were not run and are not guessed.

| construction | branded `NonEmptyStr` | branded `Int` |
|---|---|---|
| `data X: T = lit as T` | **works** (`"cpu" as DomainId`, corpus) | fails |
| `data X: T = lit` (no cast) | — | fails |
| cast inside a `fn` body | fails | fails |
| bare literal at a *field* position | — | **works** |

Two controls make the failures attributable:

- `test fn probe_plain_int() -> Bool { 7 == 7 }` in the same probe file
  returns `true`, so the file, the entry scope, and the interpreter are fine.
- `witness_shared_encodings_sum_the_pool_once` — existing corpus code that
  builds `"cpu" as DomainId` — returns `true`, so branded construction is not
  broken in general.

The bottom-right cell is why this module works: `CasSlotVersion { generation: 7 }`
builds a branded `CasGeneration` at a field position, both inside a `data` and
inside a `fn`, and executes green.

## What is NOT claimed

Three hypotheses were stated and then falsified by the next probe. They are
kept because each was plausible and wrong, and because a confident rule that
transfers to an adjacent question is the failure mode this whole exercise is
about:

1. *"Refined `Int` fails, brands are fine."* Falsified — branded `NonEmptyStr`
   fails inside a `fn` body.
2. *"It is `fn`-body position; `data` position works."* Falsified — branded
   `Int` fails at `data` position too.
3. *"Branded types fail."* Falsified — the corpus `DomainId` witness returns
   `true`.

No single-axis rule survives. The discriminator appears to be a conjunction of
carrier and construction position, and **this document does not assert the
rule** — it records the cells that were executed. Naming a rule here would
repeat the mistake three times over.

## Why it matters beyond this module

A `.dag` carrier can typecheck, pass a whole-tree compile, be structurally
reviewable, and still be impossible to construct where it is needed. That is
specification-without-execution (DESIGN 5) arriving through a channel review
does not cover: the source looks right and the compile is genuinely clean.

It also biases modeling away from the ladder. A branded refined scalar is the
natural carrier for a generation counter and is what DESIGN 4b wants for
climbing — but a carrier constructible only in some positions pushes authors
toward a bare `Int`, which is plausibly how `std.temporal_effect`'s
`LeaseEpoch.generation: Int` got its shape.

## What was done instead of working around it

The explicit casts were removed in favour of field-position literals — the
idiom the corpus already uses (`https_port: 443`), and the position where the
compiler reports the refinement as *enforced* rather than deferred. The brand
on `CasGeneration` is retained.

Dropping `CasGeneration` to a plain `Int` would have made every error vanish
while deleting the distinction the type exists to carry. That is the
author-side absorbing fallback DESIGN 5 names, and it is why this file exists
rather than a one-line respelling.

## Next step

Root-cause the interpreter's coercion rule, or declare the constructible
positions per carrier. Either turns "can I build this carrier here?" into a
decidable question; today it is answerable only by running the program.
