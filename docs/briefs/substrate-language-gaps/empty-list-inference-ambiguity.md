# Empty-List Inference Ambiguity

## Gap

The surface accepts empty list literals (`[]`), but inference does not always
derive the intended `List<T>` element type from a generic callable parameter.

Observed during Path B Brief 2 investigation:

```dag
import std.formatting { format }
import std.types { List, String }

let args: List<String> = []
let msg: String = format("{0}", args)
```

and the direct call form:

```dag
let msg: String = format("{0}", [])
```

both hit an inference conflict before emission. The concrete diagnostic observed
in PR #3143 was an implicit template binding conflict while matching the
`format(template: String, args: List<String>) -> String` call against the empty
list literal / typed empty-list binding.

This is an infer-stage gap: the parser and lowerer preserve the literal, and
non-empty `List<String>` arguments work, but the empty literal does not reliably
receive its element type from the expected callable parameter.

## Current Workable Surface

Use a non-empty list fixture when exercising callable behavior:

```dag
let msg: String = format("{1}", ["world"])
```

This preserves the runtime out-of-bounds placeholder check without depending on
empty-list element inference.

For production `.dag` code, avoid passing `[]` through generic callable
boundaries until the empty-list literal has an explicit element-type witness or
inference can bind it from the expected `List<T>`.

## When This Matters

This matters whenever `.dag` authors need an empty collection as an argument to
a generic function or substrate primitive. It is especially visible for
test-fixture authoring, because the natural negative case for
`format("{0}", [])` should exercise runtime placeholder diagnostics but instead
currently stops in inference.

This is class-5-adjacent substrate-language debt: it is small enough to route
around locally, but repeated route-arounds can hide the missing language
capability.

## Status

Scope artifact only. Not a worker-dispatch brief.

Path B Brief 2 works around this in its runtime OOB fixture with
`format("{1}", ["world"])`, while still proving the `format` runtime fails
closed for out-of-bounds indexed placeholders.

Estimated effort: infer-stage work, likely days to a few weeks if the fix is
localized to expected-type propagation for empty list literals; larger if it
requires a more general literal-template / generic-call unification pass.
