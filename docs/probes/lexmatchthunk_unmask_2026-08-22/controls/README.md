# The controlled pair, as a runnable file (2026-08-22)

`algebra_genericity_pair.dag` is the discriminating control for
[`../A_ARM_MASK_MECHANISM.md`](../A_ARM_MASK_MECHANISM.md). **Both arms declare the SAME
non-generic thunk**; they differ only in whether the *algebra carrying the lambda* is generic.

```
gunbc compile --source-root <dir> --entry <dir>/t/arms.dag --output-dir /tmp/out --target rust
```

Result on a pristine `967b5bc1b92` binary — **one file, one run, one hard diagnostic**:

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
