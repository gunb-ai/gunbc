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

## Observed paired receipt

The prediction was tested in one remote dispatch with the annotation-only #9027 correction in
both arms. Because that correction conflicted with the older board tree, both arms took the same
resolved file from #9027; it is therefore common setup rather than a treatment difference. The
before arm was `183e5972999` plus that correction. The after arm was PR #9026 at `e2317ed0838`
(including its regenerated stage0 mirrors) plus the identical correction.

| observation | before | after |
|---|---:|---:|
| coded rustc errors | 324 | 317 |
| E0308 | 128 | 121 |
| scoped `v2.compiler.eval` call-order identities | 7 | 0 |
| three preregistered control identities (location-reference count) | 5 | 5 |
| E0061 | 17 | 17 |

The prediction held exactly: all seven scoped E0308 blocks vanished, none converted, and no
control moved. The paired board total is 324 rather than the authorized retained board's 316
because the common #9027 file resolution changes that composite subject; no count is compared
across those subjects. Only the within-pair delta is claimed here.
