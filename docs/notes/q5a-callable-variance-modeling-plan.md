# Callable parameter variance: modeling plan (read-only, no code)

Task: keen-heron-809 msg_f0a92bea. Status: PLAN. Nothing here is run; every claim below is a read of origin/main, labelled READ, or a carried measurement, labelled MEASURED. Measurements are in `docs/notes/q5a-callable-parameter-variance-read.md` and the RFM row `gunbc.recurring_failure_mode.callable_parameter_variance_is_direction_blind`.

## 1. What the corpus already models (DESIGN section 6, DFS before vocabulary)

READ, grep over `dag/std` and `src/v2` for Hom, morphism, exponential, functor, contravariant, covariant, Callable:

- No function-space object. There is no `Hom`, no exponential or arrow-object, no functor, and no variance declaration anywhere in `dag/std`, `dag/gunbc` or `src/v2` (the only hits for "covariance" are prose in `v2.test.claim.compiler.infer_free_monoid_element_inhabitance_witness`, which names a covariance route under `List`/`FreeMonoid` as a NOT-YET-DECLARED trigger).
- `std.algebra` `AlgebraTypeTemplate` variant `CallableOf { params, return_type }` describes the shape of a receiver-method template's callable argument (filter, fold, map). It is a signature template, not a type former with a subsumption rule.
- The only declared widening between value types is the grounded-carrier crossing: `std.literal_elaboration` `kernel_grounding_crossing` returns `CrossingTotalWidening` exactly when the source extent lies inside the target extent (so std.nat Nat to std.integer Int widens and the converse refuses with `ExtentNotIncluded`). It is read for `as` casts by `v1.compiler.infer` `grounded_cast_verdict`. It is a relation on VALUES keyed on declaration identity.
- The v2 relation: `v2.std.inhabitance` `declared_type_inhabitance`. Its arrow arm `declared_type_inhabitance_arrow` normalizes both sides through `anonymous_signature_arrow` (domain types positionally, plus codomain) and then calls `declared_type_inhabitance_exact`, EXACT structural equality, with the one-step refinement-to-carrier subsumption `declared_type_inhabitance_declared_carrier`. READ: there is no Nat to Int step in this relation either, at any position.

## 2. Is variance derivable with no new vocabulary?

No. Derivable pieces: (a) the value-level widening relation W(from, to) exists (`kernel_grounding_crossing`); (b) the callable arm already decomposes a callable into domain types and codomain (`inhabitance_arrow_signature_read`). Not derivable: the rule that composes them. "Param contravariant, return covariant" is a theorem about the function space, and nothing declares the function space as a type former over which W lifts. Inferring the rule from the shape of `CallableOf` would be a heuristic on a template that was not written to mean it (DESIGN 4d: a bet typed as deduced).

Smallest declared addition (proposed, one row-shaped carrier, not a patch): a declared LIFTING of the inhabitance relation through the Arrow connective, stating per position its direction:

- domain position i: declared domain_i must inhabit produced domain_i (the relation applied with the two sides swapped);
- codomain: produced codomain must inhabit declared codomain;
- arity equal, as `callable_arrow_mismatch` already requires.

It consumes the existing relation as its base case (so W for Nat/Int and the refinement-to-carrier step come for free), and it is declared once, not per type. Home: beside the relation in `v2.std.inhabitance` (the arrow arm), as a declared homomorphism rule next to `homomorphism_rule_exact_structural_equality` and `homomorphism_rule_refinement_declared_carrier`, e.g. a rule for the arrow lifting. Open question for the operator, not decided here: whether the carrier is a general "variance of a type former" row (needed later for `List` covariance, the trigger named in the free-monoid witness, so that one row family serves both) or an Arrow-only rule. DESIGN section 6 root generalization says the second local implementation of the rule triggers the general form, and the List covariance trigger is that second instance, so the plan recommends the general row, scoped to Arrow in the first PR.

## 3. Where the judgment lives

Not in `callable_component_ground_mismatch`, and not as a new branch in `callable_arrow_mismatch` (a local patch would be the forked-logic trap, DESIGN 6b: the defect is that the comparison is undirected at `node_type_compatible`, which is the earliest unjustified boundary, and a callable's component comparison is only its client).

Two relations exist, and the plan must say which it targets:

- v2 `declared_type_inhabitance_arrow`: ALREADY refuses B and D, by exact equality (invariance), and also refuses the sound A and C. READ, not measured. So in v2 the addition is a COMPLETENESS climb (admit the sound directions through a declared rule), not a soundness fix. That is the right order of obligations under DESIGN 5: refusing too much is loud, admitting too much is silent.
- v1 seed `callable_component_ground_mismatch` (`node_type_compatible`): admits B and D. MEASURED. v1 is semantics-frozen and maintenance-active under the PURPOSE test (`gunbc.v1.v1_maintenance_standing v1_seed_standing`); the seed is a regression oracle, never a specification (DESIGN 7). So the recommended disposition is: v2 carries the declared rule, and the seed's undirected arm is recorded as a seed defect v2 does not carry forward, unless the self-host program needs the seed to refuse B and D. That is an operator decision.

## 4. REDs turned green, controls that stay

Cells are in `dag/test/claim/callable_param_variance_differential_witness.dag` (unparsed, unrun). Expected verdict once the lifting is declared and consumed:

| cell | declared | offered | today (seed, MEASURED) | after |
|---|---|---|---|---|
| A | fn(Nat)->Int | fn(Int)->Int | accepted | accepted (control, sound) |
| B | fn(Int)->Int | fn(Nat)->Int | ACCEPTED (RED) | refused (RED turns green) |
| C | fn(Int)->Int | fn(Int)->Nat | accepted | accepted (control, sound) |
| D | fn(Int)->Nat | fn(Int)->Int | ACCEPTED (RED) | refused (RED turns green) |
| C1 | identical | identical | accepted | accepted (control) |
| String param control | fn(Int)->Int | fn(String)->Int | refused rc=1 | refused (must stay) |
| String return control | fn(Int)->Int | fn(Int)->String | refused rc=1 | refused (must stay) |

In v2 the table differs: A and C are today REFUSED (READ), so their rows are RED in the other direction and turn accepted; B, D and the String controls hold.

Discrimination obligations (DESIGN 4b rung honesty): (i) a mutant lifting that treats the parameter covariantly must make A refuse and B accept, so the pair detects a swapped direction; (ii) a produced type whose W holds in neither direction (Int vs String) refuses in both positions; (iii) the inhabitance claim pairing: the real v2 callable arm is what executes, not a supplied verdict. Pair with the existing positive control that arity mismatch refuses.

## 5. Order and what this plan does not decide

1. Operator ruling: general variance row vs Arrow-only; v1 seed disposition.
2. Measure v2 on the seven cells (srv1, ssh route, RuntimeMaxSec and MemoryMax on the scope) so section 4's v2 column is MEASURED rather than READ. Not run.
3. Modeling PR: the declared lifting plus its consumption in `declared_type_inhabitance_arrow`, with the cells enrolled; the RFM row's trigger_capability is exactly this capability.

Unverified in this plan: that v2 really refuses A and C (READ only); that `kernel_grounding_crossing` is reachable from the relation's Nat to Int step (today it is read only by `as` casts); that the nested-arrow case, which `inhabitance_arrow_signature_read` leaves on the exact comparison, needs recursion (the lifting is compositional, so it does, and the first PR should say so or refuse).
