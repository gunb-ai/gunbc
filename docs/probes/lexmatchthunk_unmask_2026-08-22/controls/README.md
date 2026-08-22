# The controlled pair, as a runnable file (2026-08-22)

`algebra_genericity_pair.dag` is the discriminating control for
[`../A_ARM_MASK_MECHANISM.md`](../A_ARM_MASK_MECHANISM.md). **Both arms declare the SAME
non-generic thunk**; they differ only in whether the *algebra carrying the lambda* is generic.

```
gunbc compile --source-root <dir> --entry <dir>/t/arms.dag --output-dir /tmp/out --target rust
```

## Preflight — compiler identity beside source identity

Recorded because a local diagnostic read against a stale binary is indistinguishable from a real
result, and this lane already spent a rebuild proving that the reverted candidate changed nothing:

| axis | value |
|---|---|
| source subject | `967b5bc1b92ee66250e06a7870c132b48a16b80a`, worktree clean (`git status --short` empty) |
| compiler | `target/release/gunbc` **deleted and rebuilt** from that tree (`cargo build --release -p v1-compiler --bin gunbc`, 2m13s) — not the session image's baked `/usr/local/bin/gunbc`, which predates this CLI |
| postdates-stale check | the baked binary rejects `--entry` outright; the rebuilt one accepts it, so the two cannot be confused |
| healthy-pool positive control | **arm A**, which compiles clean in the same run — a refusal in both arms would mean the harness, not the variable |

Result on that binary — **one file, one run, one hard diagnostic**:

| arm | algebra | thunk | result |
|---|---|---|---|
| `arm_a_non_generic_algebra` | `PlainFold { delimited: fn(Thunk, Thunk, Thunk) -> Thunk }` | non-generic | **clean** |
| `arm_b_generic_algebra_concrete_thunk` | `GenericFold<R> { delimited: fn(R, R, R) -> R }` instantiated at `Thunk` | **the same** non-generic thunk | `method 'apply' cannot be resolved: receiver type 'Primitive()'` |

**Why this pairing and not a minimal thunk:** a bare thunk with no algebra around it does **not**
reproduce the refusal, because the genericity that loses the type is in the **algebra**, never in
the thunk. `v2.compiler.tokenize` `LexMatchThunk` is a concrete `fn(String) -> LexMatchResult` and
always was; the receiver that arrives untyped is `open_r`, a parameter of the lambda initializing
`delimited: fn(R, R, R) -> R` on the generic `v2.std.compilers.lexing` `LexPatternFold<R>`. A
reproduction that drops the algebra drops the variable.

## What this pair is the acceptance test FOR (corrected 2026-08-22, before any B-arm measurement)

**`arm_b` is the TERMINAL red for the whole `(c) → (a) → (b)` chain, not a per-step red.** Measured
by `calm-heron-887` on a tree with **(a) applied**: `arm_b` still refuses. So:

- a red `arm_b` after (c), or after (a), is **EXPECTED** and is **not** a falsification of anything —
  it means the chain is incomplete, not that the step failed;
- `arm_b` going green is the **chain's completion signal**, and the trigger for the B arm;
- `arm_a` staying clean throughout remains the harness check: if `arm_a` ever refuses, the harness
  moved rather than the variable.

**Why this correction is legitimate rather than a moved goalpost:** it fixes a *stated expectation*
that had never been tested against the intermediate state — the pair had never been run against a
tree carrying (a) when the instruction was written — and it is recorded **before** the measurement
it governs. Nothing about the registered population, the prediction, or the join rule changes.

**The failure mode it exists to prevent, in both directions:** reading a red `arm_b` after a correct
(c)/(a) landing as the repair being ineffective; or weakening a correct control to make it green,
which is the worse of the two because it destroys the terminal acceptance test for the entire chain.

**Provenance of the MEASUREMENT behind it, corrected by its author (2026-08-22).** The first run
supporting this row was invalid: the mirror half of the (a) patch had been reverted to build a
baseline, the patched copy was saved under `/tmp`, the container wiped `/tmp` between the two steps,
and the restoring `cp` failed silently — so both arms were baseline against baseline. The tell was
in the output and is worth keeping: **two arms reporting IDENTICAL counts (11 and 11, 158 and 158)
is not agreement, it is one instrument run twice.** Re-measured on a genuinely patched binary, with
the symbol verified in the source *and* in the built binary first, the result is unchanged and this
row stands as written. It is recorded because this document's edit was made while the supporting
evidence was invalid, and a claim's basis is part of the claim.

**Provenance of the correction:** `calm-heron-887` first ran this pair and reported `compiled 5
files, 0 diagnostics` — a shell race (a trailing `&` bound to the whole and-chain, so the compile
read a partially written module), which they diagnosed and disclosed themselves rather than filing
it as a defect in the control. Re-run clean, `arm_b` refuses and `arm_a` is clean, as documented
above.
