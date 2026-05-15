# R3 Path B Brief 1 — Generic Method Type Params Investigation

## Finding

The primary gap is not top-level generic function parsing, lowering, or Rust emission. Those already work for the executable list surface in `src/v3/std/list.dag` when the higher-order argument is a named function with an annotated signature:

- `fn map<A, B>(list: List<A>, f: fn(A) -> B) -> List<B>`
- `fn fold<T, U>(list: List<T>, init: U, f: fn(U, T) -> U) -> U`

Two remaining substrate-language gaps surfaced:

1. Record-field / dotted method syntax cannot express method-local type parameters such as `FreeMonoid<T>.map<U>(...)` because `type FreeMonoid<T> { map: ... }` fields parse only as `SurfaceType`, and `SurfaceType::Arrow` has no local `type_params` slot. Expression calls likewise support `Call` and `PathCall`, but not explicit call-site type arguments such as `xs.map<String>(...)`.
2. Inline lambdas against generic callable parameters still fail inference for this shape. `map(singleton(1), |x| "one")` and `fold(singleton(1), "", |acc, x| acc)` parse and lower, then infer reports that the callable argument does not match the expected generic function type. Named functions with explicit signatures do compile and emit.

## Landed Receipt

The target-agnostic algebra-template rows in `dsl/std/algebra.dag` already carry the needed method-local variables through `AlgebraTypeVariable`:

- `MappedElement` for `map` / `flat_map`
- `FoldAccumulator` for `fold`

This PR pins the named-function executable surface with `generic_method_type_params_test`, which compiles and emits:

- non-endomorphic `map`: `List<Int> -> List<String>`
- accumulator-polymorphic `fold`: `List<Int> -> String`

## Follow-Up Boundary

Closing the exact `receiver.map<U>(...)` syntax remains a parser/lowerer surface extension. Full lambda ergonomics also need an infer follow-up so inline lambdas can satisfy generic callable parameters by the expected parameter and contextual return type. The method-syntax follow-up needs a `SurfaceType::Arrow` method-local type parameter carrier and call-site type-argument syntax before the `FreeMonoid<T>` record fields can become the sole syntactic authority for receiver methods.
