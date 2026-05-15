# Lambda Inference Against Generic Callable Parameters

## Gap

Inline lambdas do not currently infer cleanly when passed to generic callable parameters such as `map<A, B>(list: List<A>, f: fn(A) -> B) -> List<B>` or `fold<T, U>(list: List<T>, init: U, f: fn(U, T) -> U) -> U`.

Observed during Path B Brief 1 investigation:

```dag
let labels: List<String> = map(singleton(1), |x| "one")
let folded: String = fold(singleton(1), "", |acc, x| acc)
```

Both forms parse and lower, then inference reports that the callable argument does not match the expected generic function type.

## Current Workable Surface

Declare named helper functions with explicit signatures and pass those functions as values:

```dag
fn int_label(x: Int) -> String = "one"
fn keep_label(acc: String, x: Int) -> String = acc

let labels: List<String> = map(singleton(1), int_label)
let folded: String = fold(singleton(1), "", keep_label)
```

## Status

Ergonomics gap only. This does not block Path B tokenize/parse driver authoring as long as driver code uses named helper functions for collection transforms.
