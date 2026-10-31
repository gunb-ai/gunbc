# Q1e: arithmetic over grounded carriers (read-only design)

Scope: v1 binary/unary operator result typing under the side-chat ruling. Revision read: origin/main f5a8efa09b53.
Method: `git grep` and source reading only; nothing executed. Not inspected: v2 infer's binop typing, the Rust twin
`src/v1/stage0/src/std_coercion.rs` beyond its symbol names, E1's text (I only know it as "E1's shortcut").

## (a) Where v1 types operators today

- `v1.compiler.infer` `infer_expr`, arm `ExprBinOp`: infers both operands, then calls `infer_binop_type_node(op, left_type: resolved_type(left_typed), ...)`. **Only the left operand's type is passed.** The right operand is inferred and never consulted for the result.
- `v1.compiler.infer_types` `infer_binop_type_node` -> `infer_arithmetic_binop_type_node`: comparison/logic -> Bool; every arithmetic op -> `left_type` (a Conj left reads its algebra field's type via `binop_algebra_fields`). Net: result type = left operand type.
- `Sub` alone is special: `subtraction_refinement_of` -> `std.coercion` `dag_subtraction_refinement`, which looks the left operand's declaration up in `refinement_cast_rules` (the Int->Nat *proof-cast* table) and returns its `from_type` (Int). Ambiguity (two rows, one declaration) refuses.
- `ExprUnaryOp` `Neg`: result = operand type. So `-n` for `n: Nat` is typed **Nat**; that is the wrong answer under the ruling, and nothing refuses it.
- `Nat + Int` types **Nat** (left); `Int + Nat` types Int. Commutativity of the typing is false today.
- `grounded_binop` (emission of grounded views) calls the same judgment and overrides `result_type` only for the predecessor, where the branch has already proved the subject >= k. That override is a narrowing under a proved guard, not a promotion; it survives.

## (b) Authorities to consume (none new except one row set)

1. **Structure inhabitance** `std.algebra` `algebra_carriers`: `Int -> OrderedRingProfile`, `Nat -> OrderedSemiringProfile`; `ordered_semiring_templates` is the ring surface *without `negate`* (its comment states the omission is the content). `binop_algebra_fields` (op -> `AlgAdd|AlgMul|...`) is the op->structure map. Subtraction maps to `AlgAdd` today, which is why a semiring is wrongly allowed to "have" it; the ring's `negate` is the real requirement of `Sub` and `Neg`.
2. **The declared Nat->Int embedding is the constructor `std.integer` `Int.NonNegative { n: Nat }`.** `Int = NonNegative { n: Nat } | Negative { pred: Nat }` is the canonical carrier, so the injection exists as a declaration; `int_of_group_completion` is the other direction's normalization. It is **distinct from `kernel_grounding_crossing`**: that (std.literal_elaboration, over `gunbc.structural_realization_bindings` `kernel_grounding_rows`) is a *host realization* fact (both Nat and Int realize as the machine integer; `UnboundedNaturals` extent within `UnboundedIntegers`), used for literal admission. The mathematical embedding must not be read from it, and it must not be read from `grounded_primitive_coproduct_identities` (`Int -> Nat` "a Nat IS an Int", a *grounding*, row direction reversed relative to the injection) either.
3. **`std.coercion`**: has `CastRule`, `RefinementCastRule`, `ConversionPlan`/`judge_conversion_plan` (the text-crossing instance of a declared conversion with derived endpoints, "the function IS the plan"). No numeric conversion row exists. The embedding row belongs here as the numeric peer of `literal_homomorphism_rows`.
4. `std.nat` `NatSubtraction` / `nat_sub`: the only declared route *into* Nat from a difference (`NatDifference | NatSubtrahendExceedsMinuend`). It stays the sole non-negativity construction.

New data (the one addition): `numeric_embedding_rows: List<CarrierEmbedding>` in `std.coercion`, with
`CarrierEmbedding { from: DeclarationRef, to: DeclarationRef, injection: DeclarationRef, preserves: List<AlgebraFieldKind> }`
and the first row `{ Nat, Int, std.integer.NonNegative, [AlgAdd, AlgMul, AlgCompare] }`. `preserves` is the implicit-safe obligation (total, exact/injective, op-commuting) stated, not assumed: an embedding is admissible for op `o` only if `field(o)` is in `preserves`.

## (c) Selection as a fold over those authorities

`arithmetic_result(op, L, R) -> Carrier | Refused(cause)`:
1. `f = binop_algebra_fields(op)` refined by structure: `Add->add`, `Mul->mul`, `Sub->{add,negate}` (requires an additive inverse), `Neg->negate`.
2. `up(X) = {X} union {C : embedding X->C in numeric_embedding_rows and f in preserves-or-defined}` (reflexive, one step; transitive closure only when a second row exists).
3. Candidates `K = {C in up(L) intersect up(R) : profile(C) defines f}`. `profile` = `algebra_carriers`.
4. `K` empty -> refuse (no carrier defines the op for these operands). Take the **minimal** elements of `K` under "embeds into". Exactly one -> result type. Two or more incomparable -> **refuse as ambiguous** (typed, located; never row order, same discipline as `SubtractionRefinementAmbiguous`).
5. Unary: `Neg` same fold with `up(X)` only.

Consequences, all derived (no table): Nat+Nat -> Nat (K = {Nat, Int}, minimal Nat); Nat*Nat -> Nat; Nat-Nat -> Int (Nat lacks negate, so K = {Int}); -Nat -> Int; Nat+Int -> Int (up(Nat) intersect up(Int) = {Int}); Int-Int -> Int. Never Int->Nat: no embedding row has Int as `from` toward Nat, and `Int as Nat` stays the refinement-cast refusal.

## (d) Witness cells (v1 `gunbc test` claims, each RED discriminates on the real acceptance path, with accepted control)

| cell | source | verdict |
|---|---|---|
| nat_plus_nat_is_nat | `fn f(a: Nat, b: Nat) -> Nat { a + b }` | accepted (control) |
| nat_minus_nat_is_int | same body, `-> Int` | accepted; same body `-> Nat` **refuses** (RED; implicit Int->Nat) |
| mixed_nat_plus_int_is_int | `fn f(a: Nat, b: Int) -> Int { a + b }` | accepted; `-> Nat` refuses; today the result is Nat so the `-> Int` control is the one that flips (RED before: types Nat, accepted by the grounded identity only by accident, refusal arrives on the Nat-declared version after) |
| mixed_int_plus_nat_commutes | `b + a` | same verdict as above (commutativity) |
| neg_nat_is_int | `fn f(a: Nat) -> Nat { -a }` | **refuses** (RED, today accepted: typed Nat) |
| nat_subtraction_is_the_only_way_back | `nat_sub(a, b)` returns `NatSubtraction`; match arm `NatDifference` yields Nat | accepted (control) |
| ambiguity_refuses | fixture with two incomparable embeddings Nat->A, Nat->B, both defining add, `Nat + Nat`-with-A/B mix | refused, typed ambiguity (needs a fixture-only embedding table passed to the fold, supplied-values style) |
| machine_width_frontier | `UInt8 + UInt8` | typed `ArithmeticWidthUnmodeled` refusal/advisory naming the frontier row; today silent |

Evidence discipline: RED and control run on the real route (inhabitance claim for the real producer, per DESIGN section 3, "witness discriminates at one interface"); the fold itself is witnessed on supplied rows.

## Machine widths: explicit bounded frontier

Population: the 10 rows in `std.integer` (`Int8..Int128`, `UInt8..UInt128`, `Compose<Int|UInt, MachineWidth<N>>`, phantom, transparent to Int at a formal today). Not modeled: overflow behavior, result width, width-mixing. Frontier row (a `gunbc.rung_drop`-style declared row, operator-approved outside the diff): current rung "silently the left type"; temporary: typed unmodeled advisory; restoration trigger names the **capability**: "width-parameterized carriers with a declared overflow outcome per op (checked / widening), so `arithmetic_result` can read a width-aware profile". Until then widths resolve to their base carrier only for the fold's candidate search and are flagged.

## (e) What this lets E1 delete

Read conditionally (E1's text not seen): any hard-coded Nat/Int promotion or "result is the left type" rule in operator typing; `dag_subtraction_refinement` / `subtraction_refinement_of` / `SubtractionRefinement` (and its Rust twin) -- subsumed by the fold, which also removes the misuse of the Int->Nat *proof* cast table as a type-escape oracle; `Neg => operand type`; the `Sub`-only branch in `infer_binop_type_node`. Kept: `grounded_binop`'s guarded narrowing, `NatSubtraction`, `kernel_grounding_rows` (realization only).

## Risks / unknowns

- `Sub -> AlgAdd` in `binop_algebra_fields` is itself a small fork to repair at the root of step 1.
- The Rust twin `std_coercion.rs` needs the same change at seed level (stage0 mirror regen).
- Literals: `a + 1` with `a: Int` and `1` elaborated at Nat via `infer_operand_literal`; the fold must see the literal's elaborated carrier, not Nat, or `Int + 1` would climb to Int by embedding (fine) but `Nat - 1` literal case needs a cell.
- Emission: host operator for a mixed Nat+Int is the machine `+` on both (shared realization); a cell should confirm no cast is emitted.
