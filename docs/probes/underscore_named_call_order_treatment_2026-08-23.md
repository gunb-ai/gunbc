# Underscore-idiom named-call order — preregistered treatment

This is one bounded producer decision from the retained `03_ingest` emission board at
`907f19c2cc7`. It is not a partition of that board.

## Current population

The retained log has seven `error[E0308]: arguments to this function are incorrect` blocks in
the emitted `v2.compiler.eval` module whose callees are `eval_value_node`,
`eval_transform_node`, `eval_branch_node`, `eval_loop_node`, `eval_bind_node`, and
`eval_match_node` (two calls reach `eval_loop_node`). Each declaration uses at least one
underscore-prefixed unused parameter and each authored call supplies its caller-visible label.

The other three diagnostics with the same rustc message are controls, not members: two are
positional calls inside fold lambdas, and one supplies `NormalizedTree` values to a function that
requires `Node`. This treatment predicts no change to those three.

## Producer and intended fact

`v1.compiler.emit` `order_typed_call_args` translates source named arguments into target
positional arguments. It previously compared a caller label to the exact declaration spelling.
The accepted label relation already has one authority: `v1.compiler.infer`
`call_arg_label_matches_param`, which defines declaration `_args` as caller label `args` (and also
serves inference's call-shape wall). Exact spelling therefore made emission answer the same fact
differently from inference.

The intended fact is unique: an accepted named call is ordered by declaration parameters using
`call_arg_label_matches_param`. A bare anonymous `_` has no unique caller-visible identity and is
not reordered by name.

## Discriminating treatment and preregistered receipt

The witness compiles two otherwise-identical declarations and calls. The control declares
`direct_order(a, b)`; the treatment declares `underscore_order(_a, b)`. Both calls author `b`
before `a`. Their emitted Rust must contain, respectively, `direct_order(11, 23)` and
`underscore_order(11, 23)`.

Before observing a candidate board, the registered result is:

- all seven in-scope `v2.compiler.eval` E0308 blocks disappear rather than convert: their arity
  and types already agree with the declarations, and only their positional order differs, so no
  E0061 or type-mismatch successor is predicted;
- the three out-of-scope same-message blocks remain byte-identifiable by callee and enclosing
  declaration;
- the control and treatment emit the same positional order;
- any successor diagnostic refutes that prediction and is reported as a conversion, not silently
  counted as removal.
