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
