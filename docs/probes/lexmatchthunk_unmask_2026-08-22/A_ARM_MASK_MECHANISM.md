# What the LexMatchThunk mask actually is — measured at `967b5bc1b92`, B arm not run (2026-08-22)

This document is the A-arm finding that **stopped the B arm before it was built**. The
[pre-registration](PRE_REGISTRATION.md) commits the lane to reporting rather than widening when the
declared one-variable repair turns out not to be one variable, and that is what happened. The
registered population and join rule are untouched and remain valid for whoever lands the repair.

Every claim below was produced by executing the compiler built from this tree, against
hand-written controls in a scratch source root. No claim here is read off the emitted corpus.

## The mask is NOT an emitter decision

The emitter already has the arm the repair was assumed to need. A **non-generic** record with a
function-typed field emits the field-closure call correctly:

```
type Thunk   { apply: fn(String) -> String }
type Algebra { combine: fn(Thunk, Thunk) -> Thunk }
```
→ `(th.apply)(s.clone())` and `(a.apply)((b.apply)(s.clone()))`, **0 diagnostics**.

So "the emitter renders a field-closure call as a method call" is false as a general statement, and
a repair aimed at the emitter would have been aimed at working code.

## The discriminator is GENERICITY, in a one-line controlled pair

| declaration | result |
|---|---|
| `type Algebra { combine: fn(Thunk, Thunk) -> Thunk }` | emits `(a.apply)(…)`, clean |
| `type Algebra<R> { combine: fn(R, R) -> R }` | `method 'apply' cannot be resolved: receiver type 'Primitive()' establishes no method surface` |

Same body, same call, same tree, same binary. `v2.std.compilers.lexing` `LexPatternFold<R>` is the
generic form, so every lambda parameter in `v2.compiler.tokenize`'s algebra (`open_r`, `body_r`,
`close_r`, …) reaches the emitter **with no type at all**. That is exactly the shape
`v1.compiler.infer` `unresolved_method_frontier_note` already records as `Primitive()` — "a lambda
parameter whose type never propagates" — and the same note names "the v2 tokenizer's own
`LexMatchThunk { apply: fn(s) }` idiom" as its motivating residue. The frontier row admits the
call; the emitter then emits a method call; rustc refuses it as `E0599`; and the E0308 sites
downstream of it in the same file are never reached. That is the mask, end to end.

## It is not one seam

The instantiation fails to reach the lambda **even when the expected type is fully explicit**:

```
fn build_annotated() -> Algebra<Thunk> {
  Algebra { unit: Thunk { apply: fn(s) { s } },
            combine: fn(a, b) { Thunk { apply: fn(s) { a.apply(b.apply(s)) } } } }
}
```
→ still refuses. So this is not only the call-argument seam, where the formal is the *callee's* own
type variable (`fold_lex_pattern<R>(algebra: LexPatternFold<R>)`); the annotated-return seam fails
too.

A candidate repair was written and executed rather than argued about: thread the generic
instantiation through the record-literal field loop as a left fold, which is the same substitution
`v1.compiler.infer`'s call seam already performs for arguments (`ArgGenericFoldState`). It was
regenerated into the seed mirror, the compiler was rebuilt from it, and **it fixed neither case**,
so at least one further seam sits underneath. It was reverted rather than carried: an inference
change that buys nothing is not a smaller version of a repair, it is a change with no consumer.

## A fail-open found beside it, worth its own lane

A generic record literal's fields are **not checked against the instantiation at all**:

```
fn generic_bad() -> Algebra<Thunk> { Algebra { unit: "x", unary: fn(a) { a } } }
```
→ `compiled: 6 files emitted, 0 diagnostics`

`unit: R` with `R = Thunk` silently accepts a `String`. Two facts belong in the row rather than in
whoever reads it:

- **It is not a build artefact.** It reproduces on the same binary that produced every other result
  in this document, including the controls that behave correctly.
- **It is a FLOOR class, not a differentiating one.** "Values inhabit declared types" is the
  ordinary compiler floor DESIGN §4b names, and a failure there is a below-baseline safety
  regression — never compensated by higher-order capability. It is below the ladder rather than low
  on it.

It sits adjacent to the mask because both are the same missing propagation: the instantiation never
reaches the field expectation, so nothing checks against it and nothing types the lambda parameters
bound from it. One lane holds both or they are fixed twice. It is independent of the A/B — it would
still be true if the mask were repaired tomorrow.

## Consequence for the board, unchanged by any of this

Repairing the mask will make **~68 E0308 sites appear** in `v2_compiler_tokenize.rs`. That is an
**exposure event, not a regression**: those sites exist at `967b5bc1b92` and are unobservable
because a blocking error aborts the pipeline before the phase that would report them. The
registered population, the prediction `unexplained = 0`, and the join rule in the pre-registration
are the instrument for reading that rise when it happens.

## Amendment (2026-08-22): the B arm needs a repeat-run control

Recorded here, beside the A-arm baseline, rather than in the pre-registration — that file is frozen
by design, and this is a fact learned after it, not a re-specification of it. It does not change
the registered population, the prediction, or the join rule; it adds a control the B arm must carry.

**The emitter is nondeterministic.** `royal-dove-436` observed two consecutive emits of the same
tree, same binary, same environment differing on `v2_lens_enforcement_vocab.rs` and
`v2_std_cross_tree_resolution.rs`. This is a known class rather than a new regression — the raw
corpus emit churns 36–40 files run-to-run, and the seed's `--emit-fresh` twice-zero result is
`rustfmt`-normalized rather than native determinism — but the consequence for this A/B is direct:
**a single run per arm cannot distinguish a real A→B difference from emitter churn.**

The A arm published here is one run. So the B arm, when the repair lands, must take **at least two
emits per arm** and report the within-arm variation beside the across-arm difference; any newly
visible site that also appears when A is re-run against itself is churn, not exposure. Without that
control an `unexplained > 0` result — the outcome the pre-registration calls the interesting one —
could not be told apart from noise, which would waste exactly the arm the experiment exists for.

## Status: PARKED at the B arm

The unmask was re-scoped out of this lane by `smart-ram-730` and dispatched as a v1 inference repair
carrying the fail-open above. This A/B is **parked, not cancelled**: the registered population, the
join rule and this A-arm baseline stand, and the B arm costs one probe run plus one python run once
the repair lands, because the classifier is committed and the raw log is published.
