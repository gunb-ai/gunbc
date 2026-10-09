# Checking mode in v2 infer: the expected type reaches a row before it decides

Owner: smart-newt-725. Status: plan only; nothing here is built. It is sequenced after
eager-newt-412's site-keyed facts store (branch `eager-newt-412/facts-site-key`,
[keying-relation-design.md](keying-relation-design.md) §3e), because both restructure the same fold.

## The defect this answers

`v2.compiler.infer` is one bottom-up fold (`v2.std.node` `fold_node` in `infer_entries_for_tree`).
A row decides from its children alone. A generic record construct is typed from its field values
alone. When no field value fixes a type parameter, the construct stays untyped and counted, even
when the position consuming it declares the instantiation. That happens for a phantom parameter
(`Ph<T> { n: Int }`), or for a parameter that occurs only in a field whose value is itself such a
construct (`Two<T> { a: Ph<T>, b: T }`).

The controls are the two claims in
`v2.test.claim.compiler.infer_expected_type_record_instantiation_witness_test`, enrolled
expected-red (`v2.workflow.floor_expected_red` `floor_expected_red_chunk_infer_checking_mode`),
plus the discriminating refusal below, which this work ENROLLS when it lands. It is done when all
three pass.

The refusal is not enrolled expected-red today because, while it is red, infer does not refuse the
program and runs the whole member, which costs about 75.8k eval steps against the 72.3k floor
budget for a new witness. Once checking mode refuses at the field, the same claim stops early and
fits. Its specimen, to enroll with this work as `etr_a_field_value_disagreeing_with_the_contexts_instance_refuses_holds`:

```
module v2.test.etr_two_bad

import v2.std.logic { Bool }

type Ph<T> {
  n: Int
}

type Two<T> {
  a: Ph<T>
  b: T
}

data t: Two<Bool> = Two { a: Ph { n: 1 }, b: 3 }
```

Infer only the `data` member `t` (handed the module's own indexes) and require a refusal with
`record_field_value_does_not_inhabit` at `b`. The nested `a: Ph<T>` must stay: it is what leaves
`T` fixed only by the context, so the claim discriminates checking mode rather than plain
field-value instantiation.

## The rule that shapes everything: deliver before deciding, never revise

The site-keyed store keeps one fact per site. A row that enters a different second fact at an
occupied site refuses `infer_facts_key_conflict`. So the expected type cannot be applied after a
child has decided, by re-judging it or overwriting its fact. It must reach the child's row before
the row decides.

A consumer-side re-judgment that worked after the fact was built and withdrawn (#12935). It would
have been a second route that this plan deletes.

## The mechanism: the expected type is fold context, not a fact

`v2.std.node` already has the fold this needs: `fold_node_topdown` with
`NodeFoldTopDown { init, child_context, step }`. `child_context(parent, ctx, edge)` computes what a
child is told from its parent and the parent's own context. Checking mode runs infer's gather on
that fold, with the context carrying at most one expected type.

- **It is an input, not a fact.** It is never stored in the facts trie, and no site gets a second
  entry because of it.
- **A child's site is the parent's site plus one step**, which the site-keyed store makes directly
  addressable. That is why this goes second.

## Where an expected type enters (`child_context` arms)

Each arm states the one edge it speaks for. Every other edge passes **no** expectation: context is
never inherited blindly through a node that does not declare a type for that child.

| parent | edge | expected type for the child |
|---|---|---|
| bodied Arrow | `^arrow_body_edge` | the declared return, at its denotation, unless it mentions the Arrow's own type parameters |
| annotated Bind (`let x: T = e`) | the bound value | the annotation |
| record construct | a field edge | the field type the record declares. If the construct itself has an expected `R<A..>` of its own record, the record's binders are first instantiated to `A..`. |
| list introduction | an element | `T`, when the list's own expected type is `FreeMonoid<T>` |
| application | an argument | the formal's declared type, only while the callee's Arrow is known (inline today; named calls after #12506) |

The record-construct arm is the one that matters for the controls, and it is recursive. The
construct's own expectation instantiates its binders, which instantiates the field types, which
become the expectations of nested constructs.

## Which rows consume it (first slice: one)

Only **the record construct row** consumes the expected type in the first slice. When its context
is an Instantiation of its own record with every argument present, it starts the existing
instantiating walk (`infer_judge_formal_args`) from those instances, not from empty. A field value
that disagrees with an instance the context fixed is refused at the value by the same relation and
reason as any field. The construct forms `R<A..>` itself, so the consuming position's existing
judge sees an equal type and decides.

That one row needs two supporting changes, both written once in #12935 and withdrawn with it:

- a starting-instances parameter on `infer_judge_formal_args`;
- the instantiating walk judging a formal that mentions an already-instanced type parameter at that
  instance (`inner: Ph<T>` with `T` fixed reads as `Ph<Bool>`).

Other rows that could consume an expectation are later slices, each with its own red. Two examples:
an empty list literal `[]`, which is on the frontier today, and an unannotated `let`.

## What does not change

- **There is no second judgment.** The consuming position still judges produced against declared
  through `v2.std.inhabitance` `declared_type_inhabitance`. Checking mode only lets the child
  decide with more information.
- **No type is guessed.** A declared type that is a declaration reference (possibly an alias), an
  Instantiation of a different record, the wrong arity, or one mentioning an enclosing function's
  type parameters gives no expectation. The construct stays counted exactly as today.
- **String literals stay out of scope.** They are untyped until work item adhoc-3e4eee5a-d66, so
  the map-literal case stays counted on that item.

## Cost shape

`child_context` runs once per edge. For a construct, the record's declared fields must not be
re-derived per field edge, which would be quadratic in the field count. The context for a
construct's children carries the instantiated field-type map, built once when the construct's own
context is computed. Reading the record declaration is a guarded index lookup
(`v2.std.symbol_index` `symbol_index_lookup`) against the subject's `resolved_declarations`.

## Evidence when built

- The two enrolled controls pass unchanged and leave `floor_expected_red` in the same change, and the
  refusal above is enrolled green, as
  that roster's monotone arm requires.
- Existing record-construct, declared-return, application-argument and optional-at-required
  witnesses stay green.
- A native-route census, base against head: N reach / M changed, at identity grain.
- A cost reading on a construct with many fields, showing the field map is built once.

## Sequencing

1. eager-newt-412's site-keyed facts store lands. It converts the gather rows to per-site entries.
2. Checking mode: the gather moves to `fold_node_topdown`, adds the `child_context` arms above, and
   changes the construct row as the only consumer. One PR, with the controls flipping in it.
3. Later slices add other consumers one at a time, each with its own red.
