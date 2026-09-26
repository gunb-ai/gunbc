# Arrow elimination in v2 infer — model for ruling

Status: proposed model, awaiting a v2-foundation ruling before any `04_infer` change.
Trigger: work item "v2: infer derives a Bool-returning application", parent `deep-bee-18`,
consumer `v2.compiler.refinement_discharge`.

## What was measured (main `21f4d0c390`, a locally built `gunbc run` over a probe entry)

| application `Transform[Arrow(Conj{x: Int}, R, body), 1]` | infer | root facts | eval |
|---|---|---|---|
| R = Int, body = int literal | Accepted | GroundingNotDerived | `eval_rejected_grounding_not_derived` |
| R = Bool, body = `true` | Accepted | GroundingNotDerived | `eval_rejected_grounding_not_derived` |
| R = Int, body = `true` | Accepted | GroundingNotDerived | — |
| R = Bool, body = int literal | Accepted | GroundingNotDerived | — |

`infer_compatible_argument_admits` asserts only `Accepted`, so it never witnessed derivation.

## The slice, re-derived (DESIGN §6b)

1. **Return type atom → its grounding.** `v2.compiler.infer` `infer_node_facts` grounds a
   binding atom through `v2.extdeps.languages.dag` `dag_binding_denotation`, which declares
   only `^dag_binding_type_int` and maps it to `dag_int_inhabitant_node` — the *roster record
   describing* Int (inhabitant atom + surface spelling), not the Int type. Bool has no row, so
   a Bool return atom stays on the frontier, and so does its Arrow (the product row needs every
   child).
2. **Arrow introduction.** Nothing compares an Arrow's body type to its declared return. The
   Arrow product row composes children's evidence — including the body's — into the Arrow's
   "type".
3. **Arrow elimination — the earliest unjustified boundary.**
   `infer_transform_derived_optional` derives a type only for binary Int add and casts. No rule
   types `f(a)` as `f`'s codomain, so NO application derives, Int included. Argument
   inhabitance (`infer_application_argument_inhabitance`) is judged, then the node falls to
   the frontier.

## One Int, not two (the unification)

Census of what each form means today:

- `Atom(^dag_binding_type_int)` is the Int **value type** at every consumer: the Int literal
  rule (`infer_branch_int_binding_type_node`), binary add's unified type, match/branch
  unification (`infer_branch_type_is_int`), parameter operand types (the declared domain atom
  via `infer_find_arrow_domain_type_in_tree`), argument inhabitance (declared formal compared
  structurally to the argument's type), and the evaluator's runtime primitive type
  (`v2.extdeps.runtimes.v2_evaluator` `v2_eval_int_type_node`).
- `dag_int_inhabitant_node` (a Conj) is the dag language roster's record about Int. Its only
  use as a *type* is `dag_binding_denotation` → the binding atom's derived type.

So the fork is `dag_binding_denotation` returning the record. The model:

- **`dag_binding_denotation(sym)` is the single binding → value-type authority** and returns
  the value type the binding names: Int → `Atom(^dag_binding_type_int)`, Bool →
  `v2.std.logic` `bool_node()` (the node the Bool literal rule already uses). The Bool row is
  one more row of the same join, not a special case.
- **The literal rules consume it** instead of minting their own nodes: an Int literal's type is
  `dag_binding_denotation(^dag_binding_type_int)`, a Bool literal's is
  `dag_binding_denotation(^dag_binding_type_bool)`. `infer_branch_int_binding_type_node`
  deletes. (The evaluator's `v2_eval_int_type_node` is the runtime's own copy of that node, and
  is left to a later cut; it is equal by structure today.)
- **A type-expression atom's OWN grounding is its kind**, not its denotation:
  `kind_node(TypeDenotationKind)` (`std.kind`), exactly as a roster type member derives today.
  This is forced, not chosen: the Int denotation is structurally the atom itself, so taking it as
  the atom's derived type hits `std.constraints` `canonical_grounding_from_derived_type`'s
  self-evidence wall — and it was always a category error (the type of the expression `Int` is
  a type-kind, not `Int`).

## The two rules

**Arrow introduction (body/return check), at the Arrow product row.** When an Arrow carries a
body edge (`^arrow_body_edge`) and both the body's facts and its declared return atom's
denotation are derived: the body's value type must equal the denotation, structurally with
provenance stripped (`infer_type_equal_ignoring_provenance`, the list rule's authority). A
mismatch refuses with the new reason `^arrow_body_does_not_inhabit_declared_return`, located at
the body. A body or return that is not derived leaves the Arrow on the counted frontier, never
admitted.

**Arrow elimination, in `infer_transform_derived_optional`.** For `Transform[callee, args…]`
whose callee is an Arrow and whose argument inhabitance was admitted: the application's derived
type is `dag_binding_denotation` of the callee's declared return atom. The derivation requires
the callee Arrow itself derived (so the introduction check has run). A non-Arrow callee (an Atom
operator — `FormalUnresolved`), an underived callee, or a return with no denotation stays on the
counted frontier.

Red and controls owed:
- Accepting: `R = Bool, body = true` derives Bool at the application; eval of it equals eval of
  the literal `true` and differs from eval of `false`.
- Accepting: `R = Int, body = 1` derives Int at the application (same rule, not a Bool case).
- Red: `R = Bool, body = 1` and `R = Int, body = true` refuse
  `^arrow_body_does_not_inhabit_declared_return`.
- The existing argument-inhabitance reds stay red.

## Consumers that see the newly derived facts

- `v2.compiler.eval` `inferred_facts_for_eval` — applications now pass its grounding gate.
- `v2.compiler.translate` — the `translate_rejected_grounding_not_derived` gate, same.
- `v2.std.coercion` via `infer_transform_cast_optional` — a cast whose operand is an
  application is now judged by `coercion_cast_crossing` instead of staying on the frontier.
- `v2.compiler.refinement_discharge` (deep-bee-18) — the intended consumer.
- infer's own branch/match/loop unification — an arm that is an application now has a type, so
  a previously frontier arm pair can now unify or refuse `arm type mismatch`. This is the
  population the floor has to census.
- Readers of an Arrow's evidence or a binding atom's resolved type: the return atom's evidence
  changes from the inhabitant record to the type kind. `v2.test.execution.dag_binding_denotation`
  counts derivation only and holds either way.

## Questions for the ruling

1. Is `dag_binding_denotation` the right single authority for binding → value type, or should
   Int's value type live in `std.integer` (as Bool's lives in `std.logic`), with the language row
   pointing at it?
2. The Arrow's composed evidence includes the body's type (and `dag_arrow_with_body_node` lists
   the return atom twice). A function type is domain → codomain. Out of scope here unless you
   rule otherwise; noted so it is not mistaken for part of this model.
