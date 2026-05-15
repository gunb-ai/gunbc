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

## Status

Ergonomics gap only. This does not block Path B tokenize/parse driver authoring because the top-level generic function form parses, lowers, infers, and emits.
