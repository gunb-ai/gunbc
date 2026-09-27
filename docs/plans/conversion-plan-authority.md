# Conversion-plan authority (model for ruling)

Status: **model, not built.** For neat-boar-16's ruling. Author: deep-bee-18, node adhoc-032c89dc-138.

## The ruling this serves

The operator ruled on 2026-09-26, recorded in DESIGN §4 by #12332. Coercion is the implicit-safe subset of declared conversions. Any phase that changes interpretation, requires proof, may lose information, needs a policy, or has competing routes makes implicit coercion refuse. The author then proceeds only by naming a declared conversion plan.

neat-boar-16 ruled on 2026-09-27 on the cast question:
- A bare `lit as R` into a refinement refuses.
- Admission goes through ONE declared `refine` plan, whose check is #12375's discharge.
- The 44 census modules are respelled to that plan.

No plan authority exists on main. `bool_indicator` is only DESIGN prose. This document is its model.

## Concepts reused, not re-minted (the DFS)

| Existing concept | What it owns | Relation to the plan authority |
|---|---|---|
| `v2.std.coercion` (`coercion_cast_crossing`, `CoercionResult.quality` = Identity/Exact/Widened) | The IMPLICIT relation: total, exact, proof-free crossings | Unchanged. It is the "no plan named" arm. #12407's refinement-to-declared-carrier widening stays here: subset inclusion is proof-free. A plan is consulted only when the cast names one. |
| `std.literal_elaboration` (`LiteralHomomorphism`, `LiteralUnfolding`) + `gunbc.structural_realization_bindings` `literal_homomorphism_rows` | Literal-introduction homomorphisms: Peano, Boolean, UnicodeScalarSequence | Under ruling B the text crossing is a named conversion, so these rows ARE conversion plans. Their home becomes this authority: `LiteralUnfolding` is reused as a phase kind (below), not redeclared. Moving the rows is a separate replacement migration (see "Cut sequence"). |
| `std.symbol_semantics` (symbol↔text "declared conversion") | The symbol/text crossing | A future row here, not in this cut. |
| `v2.compiler.inferred_tree` `RefinementObligation` + `v2.compiler.refinement_discharge` (#12375) | Deciding a refinement predicate through the one evaluator | The `ProveRefinement` phase's checker. Reused unchanged. |
| `refinement_declaration(index, reference)` (#12407, over `ResolvedTree.symbol_index` from #12432) and `where_predicate_bindings` (quick-crab-850's follow-up to #12381) | Reference → refinement → carrier + bound predicate | How the `refine` phase finds the predicate. |
| `std.decision` | Selection among candidates | NOT consumed. A plan is named by the author, never selected. Two plans matching one crossing is not a selection problem, because the author named one. The coercion refusal on competing routes is the existing "has competing routes" arm. |

## The authority

A new module, `v2.std.conversion_plan`, sits beside `v2.std.coercion`. It is not inside coercion: coercion is the implicit subset, and plans are the explicit superset.

```
type ConversionPhase
  = Reinterpret { from: DeclarationRef, to: DeclarationRef }   // bool_indicator: False->0, True->1
  | Unfold { unfolding: LiteralUnfolding }                     // reuses std.literal_elaboration
  | ProveRefinement                                            // target is a refinement; its predicate
                                                               // becomes a RefinementObligation (#12375)

type ConversionPlan {
  identity: Symbol          // the name an author writes (^refine, ^bool_indicator, ^text_unfold)
  phases: List<ConversionPhase>
}

data conversion_plans: List<ConversionPlan> = [ bool_indicator_plan, refine_plan ]

type ConversionPlanLookup = PlanFound { plan: ConversionPlan } | PlanUnknown | PlanAmbiguous { count: Int }
```

- **One `refine` row, not one per refinement.** It has a single phase, `ProveRefinement`. Its source and destination are read from the cast: the operand's type must be the target refinement's declared carrier, the same carrier `refinement_declaration` projects. A per-refinement plan would be a second authority for what the `where` clause already says.
- **Unknown plan name:** `PlanUnknown` refuses located at the plan edge, reason `conversion_plan_unknown`.
- **Duplicate identities:** `PlanAmbiguous` refuses, reason `conversion_plan_duplicated`. That is an authoring defect in the roster, found the same way `literal_homomorphism_for` finds one.

## The typed node

The cast-with-plan node is today's cast node (#12315) plus one named edge:

```
Transform[coerce, e] + <cast-target> T + <cast-plan> P
```

`P` is an atom carrying the plan identity. `v2.std.type_binder`'s label rule admits `<cast-plan>` only on a cast Transform that also carries `<cast-target>`, just as it admits `<cast-target>` today.

- **Every phase stays represented.** Infer resolves `P` to its `ConversionPlan` and records the plan identity and its phases in the cast's facts.
  - A `ProveRefinement` phase records a `RefinementObligation`, where `application` is `predicate(operand)` inferred by infer.
  - Discharge (#12375) decides the obligation. A failure refuses `refinement_predicate_violated` at the cast site.
- **Target realization** consumes the recorded phases. It may refuse ONLY to realize a phase, with `conversion_phase_unrealizable` per target. It never decides whether the conversion is legal.
- **Runtime operands:** a `ProveRefinement` over an operand that is not a closed literal refuses typed (`refinement_operand_not_closed`) until a checked runtime realization is modeled. That is the brief's "runtime operands refuse typed".

## Surface spellings (a language change; pick one or take it to the operator)

All three read in both directions through the same grammar rows (§4). The emit side is the backward reading of the same production.

1. **`e as T via refine`**
   - Grammar: one new keyword `via`, and the existing cast production gains an optional `via <ident>` suffix.
   - Ingest: builds the cast node plus a `<cast-plan>` atom.
   - Emit: selects the same row backward when the node carries `<cast-plan>`.
   - For: the plan sits visibly on the cast it modifies, the reading is left to right, and `as` keeps meaning "a typed crossing".
   - Against: it adds a keyword, so the lexer's keyword roster grows. Per the memory note, `.dag` keyword collisions are wide.
2. **`e as T by refine`**
   - Same as (1), but reusing a word that is less likely to collide.
   - Against: `by` reads as an agent, not as a route.
3. **`refine(e) as T`**, with no new grammar: the plan is an ordinary call to a declared plan function whose result feeds a cast.
   - For: zero grammar change, and ingest/emit symmetry comes free.
   - Against: it hides the composition inside an application. The ruling says a named composite "may not hide its composition". It also makes `refine` an untyped function over every refinement, which needs a type parameter the cast already supplies. **Recommend against.**

**Recommendation: (1) `via`.**

## Evidence the build PR carries

- `"abc" as NonEmptyStr` (bare) **refuses** at infer with coercion's typed mismatch. This is the old `bcn_cast_into_a_refinement_refuses_at_infer`, now permanent by ruling.
- `"abc" as NonEmptyStr via refine` **admits** through discharge. `"" as NonEmptyStr via refine` **refuses** `refinement_predicate_violated`, located at the cast.
- `x as NonEmptyStr via refine`, with x a parameter, refuses `refinement_operand_not_closed`.
- `1 as NonEmptyStr via refine` refuses: the operand's type is not the declared carrier.
- `"abc" as NonEmptyStr via no_such_plan` refuses `conversion_plan_unknown`, located at the plan atom.
- `true as Int via bool_indicator` admits as `Reinterpret`, and bare `true as Int` refuses. Only if `bool_indicator` is in this cut; see below.
- Ingest-then-emit round trip of the `via` cast is byte-stable (one grammar, both directions).
- The 44 census modules (listed by name in the PR, from PR 12308's list) respelled to `via refine`, each passing infer. This retires the next-rung trigger of `gunbc.recurring_failure_mode` `as_cast_has_no_lowered_form`.

## Cut sequence

1. **Build PR (this item):** `v2.std.conversion_plan` with the `refine` row, plus `bool_indicator` if you want the second row to prove the roster is not refine-shaped. The `via` grammar, the `<cast-plan>` edge, the infer arm recording the phases and obligations, and the census respelling.
   - Depends on #12379 (a Bool-returning application derives, so discharge can evaluate), #12432/#12407 (`refinement_declaration`), and quick-crab-850's `where_predicate_bindings`, which lands with this PR as its first consumer.
2. **Separate replacement migration, later:** `literal_homomorphism_rows` and `std.literal_elaboration`'s roster move into `conversion_plans` as `Unfold` rows. `literal_homomorphism_for` then becomes a `ConversionPlanLookup`. This is XL-0T's crossing, so it is coordinated with that lane, not done here. The phase type already admits it, so no second shape is needed.

## Open questions for the ruling

1. The spelling: (1), (2) or (3).
2. Whether `bool_indicator` ships as a second row in the build PR or waits for its own consumer. Recommend it waits: §3c says a row with no consumer is dangling, and nothing in the corpus casts Bool to Int today.
3. Whether the literal-homomorphism migration (cut 2) is owned by XL-0T or by this lane.
