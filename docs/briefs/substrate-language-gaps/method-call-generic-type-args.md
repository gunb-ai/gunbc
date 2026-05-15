# Method-Call Generic Type Arguments

## Gap

The surface grammar does not accept explicit type arguments on method-style calls such as `xs.map<String>(int_label)`.

This is a parser / surface-model gap. Expression calls currently lower through `Call` and `PathCall`, but those carriers do not preserve call-site type arguments. Separately, record fields in `type FreeMonoid<T> { ... }` parse as `SurfaceType`; `SurfaceType::Arrow` has no method-local `type_params` carrier, so `map<U>(...)` cannot be expressed as a field type.

## Current Workable Surface

Use top-level generic std-list functions with named helper functions:

```dag
fn int_label(x: Int) -> String = "one"
let labels: List<String> = map(singleton(1), int_label)
```

The same surface may be referenced as the `std.list` top-level function form in design prose: `list.map<A, B>(xs, named_fn)`.

## When This Matters

This matters for `.dag` authors who want fluent method-style code. It is not blocking current R3 retirement work because driver code can use top-level collection functions.

## Status

Ergonomics gap only. This does not block Path B tokenize/parse driver authoring because the top-level generic function form parses, lowers, infers, and emits.

Estimated effort: substantial parser / surface-model work, likely multi-month.
