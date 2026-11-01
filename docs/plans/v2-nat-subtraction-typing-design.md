# v2: typing `Nat - Nat : Int` and `-Nat : Int` through the one declared embedding (WORK NOTE; read and author only)

Rev read: origin/main 345d6077fc4a (v2 infer, std.coercion), Q1e at origin/q1e-on-pr-q2d (PR 15526). Nothing here was executed.
Shared file: `src/v2/compiler/infer.dag` (silent-raven-360 owns the descent edits); this note edits nothing.

## 0. Two corrections to the premises, found by reading

1. **v2 does not refuse `Nat - Nat`; it leaves it on the counted not-derived frontier.** `infer_transform_derived_optional` routes to
   `infer_transform_binary_infix` / `infer_transform_unary_prefix` only when `infer_transform_is_*_algebra_shape` holds, and both predicates
   require `infer_operator_algebra_field` Present. For `-` (`canonical_operation_op_sub`, `AlgebraInverseCompose`) it is Absent
   (`infer_canonical_operation_field`), so subtraction and negation never enter those arms: they fall to `optional_absent()`, an Accepted program whose
   facts are not derived (`inferred_facts_grounding_derived` false, reason `^inhabitance_undecidable_formal_unresolved`), pinned today by
   `v2.test.claim.body_lowering.infer_transform_binary_infix_witness infer_transform_non_add_remains_frontier_holds` (on `1 - 2`).
   The `infer_transform_operator_unsupported_diagnostic` arm inside `infer_transform_binary_infix` is therefore unreachable for subtraction.
   Completeness gap, counted, not a wrong answer and not a refusal. **That witness must change in the same PR** (`Int - Int` will derive Int).
2. **v2 has no reader of `conversion_plan_rows` today.** `git grep -n 'conversion_plan_rows\|ConversionPlan\|InjectionPhase' origin/main -- src/v2`
   returns one hit, a comment (`src/v2/compiler/tokenize.dag`). PR 15526's sentence "v2 find_witness Widened reads that row" is a statement of the
   WIRING TO COME, not of main: v2.std.coercion's `Widened` arms are `homomorphism_rule_refinement_declared_carrier` and
   `..._refinement_widening` (refinement carriers), a different fact. The v2 consumer is new work and must cite the row, not the Widened arm.
3. **`Nat + Nat` is not accepted in v2 today either (by reading).** `dag_algebra_inhabitance_decls` has rows for Int (ordered ring), Bool, free monoid, host
   text, equivalences, and none for `std.nat Nat`; `infer_algebra_field_result_type` returns `AlgebraFieldNotCarried` for a Nat operand, which refuses as an
   operand-type mismatch. So `Nat + Nat : Nat` is a completeness gap too and a prerequisite of this design (section 2), not a control green both times.
   The before-run decides this; the claim here is from reading.

## 1. One embedding authority across v1 and v2

The embedding is `gunbc.structural_realization_bindings conversion_plan_rows` row `nat_to_int` (`InjectionPhase { source: std.nat Nat, target: std.integer Int,
safety { total, exact, preserves [add, mul, compare], pure, consent_free } }`), consumed by `std.coercion` (`injection_of`, `injection_is_implicit_safe`,
`injection_targets`, `select_arithmetic_carrier`) on the Q1e branch. v2 imports that module as it already imports `std.coercion`
(`v2.compiler.source_authority`, `tokenize`); v2 declares NO embedding, no `Nat -> Int` rule, no list of widenings. The refinement widening in
`v2.std.coercion` stays what it is.

Bridge to settle (the one new identity question): the row speaks `DeclarationRef` (`std.nat.Nat`, `std.integer.Int`); v2 operand types are `Node`s, and v2's `Int`
is the kernel atom `integer_int_type_node()` (what `infer_machine_width_signedness` compares). Read a `Node` to a `DeclarationRef` with
`infer_type_declaration_ref` (as `infer_machine_width_of` does) and map the kernel Int atom to `std.integer.Int` ONCE, in one function, in the direction
result-ref -> result-node (`std.integer.Int` -> `integer_int_type_node()`; any other ref -> its declaration reference node). Q1e records the same risk
("kernel Int may not resolve to a DeclarationRef"); one bridge for both trees, not two.

**The embedding must stay visible in the typed program (DESIGN section 4, every phase represented).** Typing the result Int over Nat operands without
recording `nat_to_int` makes the widening silent. The facts for the Transform must carry, per operand, the route applied (`std.integer.nat_to_int`) or none.
Which carrier holds it (a field on the Transform's `InferredFacts`, or an explicit conversion node the inference inserts) is OPEN and is the E1 implementer's call
with silent-raven-360 because it touches `InferredFacts`; this note does not choose.

## 2. The selection (unique minimal carrier, no name table)

For an arithmetic Transform, replace the "unify operands to one type" step (`infer_unify_transform_operand_types`) by `select_arithmetic_carrier(rows:
conversion_plan_rows, left, right, definers, preserved)`:

- up-closure of each operand's declared carrier under implicit-safe injections that preserve the operator's positive fields (`add`/`sub`: add; `mul`: mul;
  unary `-`: none);
- `definers` = the carriers whose algebra row declares the operator's field(s), **read from the rows v2 already consumes** (`dag_algebra_inhabitance_decls`
  through `algebra_inhabitance_field_reading`), generalised from "the row whose carrier equals the operand" (`infer_algebra_field_result_type`) to "all rows
  declaring the field". For subtraction and unary `-` both fields of `AlgebraInverseCompose` (`ring_field_add`, `group_field_inverse`) must be declared: Int's
  ordered ring declares both; a Nat semiring row declares add and mul and NOT the inverse, so Nat is not a definer of `-`;
- result = the unique minimal common candidate; `ArithmeticNoCarrier` and `ArithmeticCarrierAmbiguous` refuse with distinct typed reasons (new symbols, not
  `infer_transform_operand_type_mismatch_diagnostic`, which would blur "no operation" with "no unique carrier").

Worked: `Nat - Nat`: up = {Nat, Int}; definers(sub) = {Int}; common = {Int}; minimal Int. `-Nat`: same, preserved none. `Nat + Int`: up_left {Nat, Int},
up_right {Int}; common {Int}; Int. `Nat + Nat`: up {Nat, Int}; definers(add) = {Nat, Int} **only if a Nat row exists**; minimal Nat (Int is reachable from it). Without
the row it would select Int, which is wrong: the Nat semiring row is a PREREQUISITE, added to `dag_algebra_inhabitance_decls` citing the std value that already
is the inhabitance (as `dag_int_algebra_inhabitance` cites `v2.std.integer.int_ordered_ring`), not a new vocabulary. `Int + Int`: Int, unchanged.
A literal operand has no declared carrier of its own: it takes the other operand's (it elaborates there, `infer_operand_literal_elaborates_at`), so `n - 1` with
`n: Nat` is `Nat - Nat : Int`, and `n + 1` is Nat. Comparisons, `||`, `&&`, `==` keep the one-type unify (out of scope: `Nat < Int` is not decided here).

**A section 3 fork to name, not hide.** "Which carriers define a field" is answered in v1/Q1e by `std.algebra algebra_carriers` (profiles) and in v2 by the
language-model rows; the repository already records the two-representations debt (`gunbc.recurring_failure_mode
algebra_structure_has_two_representations_with_no_declared_mapping`). This design reads v2's own rows for `definers` and the shared conversion row for
embeddings; it does not import the std.algebra profile table into v2 (that would be a third path), and it does not claim the two definers answers agree. A
differential check over the same operator/carrier set is the cheap way to see a disagreement; not authored here.

## 3. REDs, controls, fixture shape (authored as a table; unrun)

Each cell is a v2 source `module probe` with `a: Nat`, `b: Nat`, `i: Int` as arrow parameters and the body below, judged through the REAL route
(normalize -> resolve -> `infer_and_discharge`) as `v2.test.claim.body_lowering.infer_transform_binary_infix_witness_helpers` does for add. Oracle: the body's
facts are `inferred_facts_grounding_derived` AND the derived type is the named carrier (RED when the derived type is not that carrier or is not derived).

| cell | body | expected after | today (by reading) | role |
|---|---|---|---|---|
| R1 | `a - b` | derived `Int`, route `nat_to_int` recorded on both operands | accepted, NOT derived (frontier) | RED |
| R2 | `-a` | derived `Int` | accepted, not derived | RED |
| R3 | `a + i` | derived `Int` | refused, `OperandsDisagree` (operand type mismatch) | RED |
| R4 | `a - 1` (literal) | derived `Int` | frontier | RED (literal path) |
| C1 | `a + b` | derived `Nat`, no embedding recorded | refused (`AlgebraFieldNotCarried`, no Nat row) by reading | CONTROL (needs the Nat row; may be RED before) |
| C2 | `i - i`, `i + i` | derived `Int`, none recorded | `+` derived; `-` frontier | CONTROL |
| C3 | `a + 1` | derived `Nat` | by reading refused as C1 | CONTROL |
| C4 | `a + true` | refused, no carrier | refused | CONTROL (a blanket widen goes red) |
| C5 | supplied rows with two incomparable minimal definers | refused, `Ambiguous` | n/a | RED on the fold (supplied rows, not the corpus) |

The existing `infer_transform_non_add_remains_frontier_holds` flips for `1 - 2` (becomes derived `Int`); it is replaced in the same change by the discriminating cells
above, not deleted (DESIGN section 4b rung 4). The add vertical is a LONG-lane witness (the add probe exceeds the 5 s fast-lane budget:
`gunbc.explicit_witness_admission`, `v2.test.claim.witness_admission`); these cells follow it, one file, enrolled the same way.

**One srv1 run, in order:** (a) BEFORE: build a native binary at origin/main, run the file's cells (expect R1,R2,R4 frontier, R3 refused, C1/C3 refused: this
pins the today column, and answers section 0.3); (b) AFTER: the same cells on the implementation head. Same sources, same corpus slice (these nine), so the
before/after is a pair. A fresh target dir, bounded, serial; nothing whole-corpus.

## 4. Out of scope, named
`Div`/`Mod` (reciprocal/`mod`), comparisons across carriers, machine widths (declared stall), pointer-width carriers, where the embedding sits in `InferredFacts`
(open, section 1), and the std.algebra-vs-language-model definers reconciliation (section 2).
