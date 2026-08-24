# Regenerating the stage0 mirror for a `.dag` compiler-authority edit

Derived from landing one emitter change (gunbc#9101) and having CI reject the first attempt.
Every lane that edits a `src/v1/*.dag` compiler authority pays this cost; this note exists so the
next one does not re-derive it.

## Why the obvious approach fails

`claim_executor --required-regen` compares the committed stage0 mirror against a fresh emit and
refuses on drift. So a `.dag` authority edit refuses until its regenerated mirror is committed
beside it. The tempting move is to regenerate once and commit the result.

**One round is not enough to ESTABLISH the mirror change, and the reason is the bootstrap, not a bug.**
The first candidate is the new `.dag` emitted by the **old** compiler. Committing it means the compiler built *from that mirror*
now carries the new behaviour — so when *it* regenerates, it re-emits every other module the change
reaches. Measured on gunbc#9101, whose change altered how folds with an unused element emit:

```
generation 1   drift: v1_compiler_emit_rust.rs   <- the edited authority itself
generation 2   drift: std_types.rs               <- a module the new emitter now emits differently
generation 3   first_generation_equal=true
```

That is the sequence across the whole exercise, not a transcript of one command: generation 1 was
observed and committed on its own, and CI then refused the branch with `drift: std_types.rs`, which is
what sent me back for the loop. The loop itself reported `ITER=1 RC=1 drift: std_types.rs` and
`ITER=2 RC=0 first_generation_equal=true` — the same two remaining rounds, numbered from where it
started. Run the loop from the beginning and it reports all three.

`std_types.rs` `list_length` is a fold with an unused element: `|acc: i64, _: _|` became
`|acc: i64, _|`. It is not unrelated cleanup riding along with the fix; it is the same change
reaching a module the first generation had not yet re-emitted.

**Do not generalize that second drift into a law — generalize the equality check instead.** A behaviour
change that no other seed module exercises can leave generation two identical to generation one; then a
second regeneration produces no new drift and is still mandatory, because it is what *proves* equality.
The invariant is:

```
one drift-producing regeneration MAY be enough
one regeneration without a subsequent equality check is NEVER enough
```

So the rule is not "expect a second drift". It is: **the generation-one delta is not yet established as
the fixed-point mirror change** — install it, rebuild, regenerate until equality. A branch carrying only
generation one may refuse in CI; gunbc#9101's did.

## The protocol

1. Start from the branch worktree.
2. Run `--required-regen`. On drift: install the **complete** candidate, rebuild the compiler from it,
   repeat.
3. Stop only at `RC=0` / `first_generation_equal=true`.
4. Export the **cumulative** diff against the original branch state: `git diff -- src/v1/stage0/src`.
5. Verify before applying — changed-path list, decoded byte count **and SHA-256** against the
   producer's own, and `git apply --check`.
6. Commit the compiler-authored mirrors beside the `.dag` change.

**The mirror's only sanctioned author is the compiler.** Every byte above comes from the candidate's
own diff; hand-authoring a mirror to satisfy the gate is the laundering the gate exists to prevent,
and it is indistinguishable from regeneration in the diff alone. That is what step 5's verification is
for: it makes "these are the compiler's bytes" checkable rather than asserted.

**A byte count alone does not establish that**, and the difference matters because this is the step the
whole no-hand-authoring claim rests on. A count proves transport length — it catches truncation, and it
proves the receiver did not paste an excerpt — but two different patches can share a count, and
`git apply --check` proves applicability, not equality to the producer's output. Print and check a
digest beside the count:

```
PATCH_BYTES=<count>
PATCH_SHA256=<sha256 of the raw cumulative diff>
```

The evidence chain is then unbroken end to end: remote compiler produces the candidate -> remote
`git diff` -> remote byte count and SHA-256 -> base64 transport -> local count and SHA-256 agree ->
`git apply --check` -> `--required-regen` reaches its fixed point.

## Transport, when the loop cannot run where the commit happens

A remote runner may have no push credentials, and a local build of `v1-compiler` may not be affordable
(shared memory slice, swap disabled). Mirror files are far too large for a log. What works is moving
the **diff** rather than the files: the loop runs remotely, prints the cumulative diff base64-encoded
with a byte-count and SHA-256 header, and the patch is decoded, verified and applied where the commit
happens.
gunbc#9101's two generations were 3748 and 555 bytes.

## Branch tree versus merge ref

CI regenerates on the **merge ref**; the loop above runs on the **branch tree**. These are different
trees whenever the base has moved, so convergence on one does not entail convergence on the other.

**Iterate against the branch tree while authoring, and treat merge-ref convergence as a separate
acceptance fact supplied by CI.** gunbc#9101 is one instance where the transfer held — green on the
merge ref with no further drift — which establishes that it *can*, not that it is automatic. A base
that moves under a branch can require another round, and the honest response is another iteration,
not an assumption that the branch result carries.
