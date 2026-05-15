# R3 Path B Brief 1 — Generic Method Type Params Investigation

## Finding

The gap is not top-level generic function parsing, lowering, inference, or Rust emission. Those already work for the executable list surface in `src/v3/std/list.dag`:

- `fn map<A, B>(list: List<A>, f: fn(A) -> B) -> List<B>`
- `fn fold<T, U>(list: List<T>, init: U, f: fn(U, T) -> U) -> U`

The remaining substrate-language gap is narrower: record-field / dotted method syntax cannot express method-local type parameters such as `FreeMonoid<T>.map<U>(...)` because `type FreeMonoid<T> { map: ... }` fields parse only as `SurfaceType`, and `SurfaceType::Arrow` has no local `type_params` slot. Expression calls likewise support `Call` and `PathCall`, but not explicit call-site type arguments such as `xs.map<String>(...)`.

## Landed Receipt

The target-agnostic algebra-template rows in `dsl/std/algebra.dag` already carry the needed method-local variables through `AlgebraTypeVariable`:

- `MappedElement` for `map` / `flat_map`
- `FoldAccumulator` for `fold`

This PR pins that executable surface with `generic_method_type_params_test`, which compiles and emits:

- non-endomorphic `map`: `List<Int> -> List<String>`
- accumulator-polymorphic `fold`: `List<Int> -> String`

## Follow-Up Boundary

Closing the exact `receiver.map<U>(...)` syntax remains a parser/lowerer surface extension, not an inference/emission blocker for the already-supported generic std-list functions. That follow-up needs a `SurfaceType::Arrow` method-local type parameter carrier and call-site type-argument syntax before the `FreeMonoid<T>` record fields can become the sole syntactic authority for receiver methods.
