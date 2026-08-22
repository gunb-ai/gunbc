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

Same body, same call, same tree, same binary. **The generic carrier is the ALGEBRA, never the
thunk** — a distinction worth stating flatly, because the lane briefly lost a cycle to reading it
the other way: `v2.compiler.tokenize` `LexMatchThunk` is a concrete `fn(String) -> LexMatchResult`
and always was, and a minimal thunk with no algebra around it does **not** reproduce the refusal.
The refusing receiver is `open_r`, a parameter of the lambda initializing `delimited: fn(R, R, R)
-> R` on the generic `v2.std.compilers.lexing` `LexPatternFold<R>`. The runnable pair that varies
*only* algebra genericity, with the same non-generic thunk in both arms, is in
[`controls/`](controls/). `LexPatternFold<R>` is the generic form, so every lambda parameter in `v2.compiler.tokenize`'s algebra (`open_r`, `body_r`,
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

**How strong that negative result is, stated exactly (2026-08-22).** The install *was* verified at
the file level — `git diff --stat` on the mirror showed 228 insertions / 170 deletions, and cargo
recompiled `v1-compiler` from it — so this is **not** the silently-failed-restore class, where the
patched file never changed at all. What was **not** done is a symbol-level check that the rebuilt
binary carries the change. So the honest strength is *mirror confirmed changed and crate confirmed
rebuilt, binary not symbol-verified*. It is also no longer cheaply re-checkable: re-installing that
candidate mirror on current main fails to compile (3 errors, `E0063` — the seed's `Node` has moved
since `967b5bc1b92`), so re-verification means regenerating the candidate at its own ref. The
conclusion is left standing at that strength rather than upgraded by assertion, and the repair lane
should treat "at least one further seam" as *measured but not binary-verified*.

## A fail-open found beside it, worth its own lane

A generic record literal's fields are **not checked against the instantiation at all**:

```
fn generic_bad() -> Algebra<Thunk> { Algebra { unit: "x", unary: fn(a) { a } } }
```
→ `compiled: 6 files emitted, 0 diagnostics`

**Still true on current main**, re-run 2026-08-22 at `abf7194e2b` with a compiler built from that
tree: same fixture, same `0 diagnostics`. So it is not an artifact of the `967b5bc1b92` snapshot,
and the repair lane holding it is holding a live defect rather than a historical one.

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
`v2_std_cross_tree_resolution.rs`, then characterised the class on request:

- **Shape: pure line reordering.** Across three emits of one unchanged tree, all three pairwise
  different, the entire difference is one `pub use` line moving one position; line counts identical
  (336) and the sorted-line test identical. So it is import-emission order — a set or map iteration
  — not a value nondeterminism.
- **The churn SET itself varies.** The second run churned only one of the two files. So a single
  run cannot even establish the churn population, which means **an exclusion list derived from one
  run is not sound** — a stronger objection than the one this amendment first recorded.
- **`rustfmt` normalizes it away**, verified by execution with a discriminating control (two files
  differing only in the order of two `pub use` lines converge byte-identically after formatting).

**What the B arm does with that, and why it is now cheaper than the first version of this
amendment:** compare **`rustfmt`-normalized** output, which makes this class *unrepresentable in the
oracle* rather than something measured and subtracted — strictly better than reporting a variation
number someone then has to interpret. The **≥ 2 emits per arm** control stays, run over normalized
output: it is now a *detector for any class that survives normalization*, not a subtraction. If it
returns zero variation the variation report is unnecessary; if it does not, it has found a class
that matters more than this one and earns its place.

**The caveat, unclosed and stated as such:** this shows the *reordering* class does not survive
normalization, not that *no* nondeterminism does. The known raw-corpus churn is 36–40 files on the
full corpus against two files on this closure, and nobody has shown that population is all
reordering.

**One property of this lane's instrument makes the risk smaller than it looks:** the A/B compares
**diagnostic boards**, not emitted bytes, and the join rule registered for it already excludes
generated line numbers — which is the only thing a pure `pub use` reordering can move. The control
is still required, because that argument covers the characterised class and not the uncharacterised
remainder.

## The baseline arm must be re-taken at the repair's parent (2026-08-22)

Registered here as soon as it became true, and before any repair exists, because it is the kind of
thing that is cheap now and unrecoverable later. **The A arm published above is at
`967b5bc1b92`, and main has since moved.** It remains the *mechanism* baseline — what the mask is,
and why — but it is **not a valid comparison arm** for the repair when that lands.

The A/B needs its two arms **one variable apart**. So the baseline is re-taken at the **repair
commit's own parent**, and the B arm at the repair commit. Comparing a post-repair board against
`967b5bc1b92` would put every unrelated landing in between inside the delta, and the join would
attribute other lanes' work to the repair — a difference that is real, arrives with a plausible
story, and is entirely an artifact of the two arms being several refs apart.

Nothing else about the registration changes: the population, the prediction and the join rule are
fixed, and the arms simply move together to the repair's own ref pair.

## Status: PARKED at the B arm

The unmask was re-scoped out of this lane by `smart-ram-730` and dispatched as a v1 inference repair
carrying the fail-open above. This A/B is **parked, not cancelled**: the registered population, the
join rule and this A-arm baseline stand, and the B arm costs one probe run plus one python run once
the repair lands, because the classifier is committed and the raw log is published.
