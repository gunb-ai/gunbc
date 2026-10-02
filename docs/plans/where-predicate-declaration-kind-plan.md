# Where-predicate declaration kind: subject role and marker disposition (plan)

Status: PLAN, awaiting the operator's ruling on the surface-syntax change (v1 seed parse). No grammar is edited by this document.
Work item: node://adhoc-211e860b-d25. Follows #12506 (frontier row added by swift-ram-269).

## 1. The slice, re-derived (DESIGN §6b)

| link | what it establishes today | what it is entitled to assume |
|---|---|---|
| surface `type Pos = Int where positive, brand("X")` | a list of predicate spellings with optional `(args)` | nothing about parameters |
| `v2.compiler.body_lowering_fold` `body_lower_where_predicate` | one `Transform` per predicate: operator = name atom, positional edges, ONE labelled record (`Conj { min: 1, max: 5 }`) | no subject is written, so none is carried |
| `v2.compiler.resolve` `resolve_where_predicate_call` | binds the operator by ordinary name lookup; refuses an unbound spelling (`resolve_reason_where_predicate_unbound`) | binds only; judges nothing |
| `v2.std.symbol_index` | the predicate's declaration, reachable by path | — |
| `v2.compiler.infer` `RefinementDeclaration` | reads `where_clause` as a DECLARED FRONTIER; nothing judges the call | — |

**Earliest unjustified boundary: the predicate declaration.** The rule "the first parameter is the refined value, never written; further parameters bind by SAME LABEL" is stated only by a `//` annotation in `std.types` (beside `gt_zero`/`range`), which no Accepted program can read (§4c). And `brand(name: String) -> WherePredicateMarker` breaks that rule: it has no refined-value parameter, and only its return type tells it apart.

**Why the fact goes on the declaration, not on the call.** Every call of `range` has the same subject, its first formal. The subject is a property of the predicate. A subject slot added to each lowered call would state that fact again at every use site. It would also still leave `brand` needing a second carrier for "is a marker". That gives one fact two authorities (§3). A std roster row ("these fns are predicates") would be a parallel ledger (§6: let the mark on the carrier be the authority). A declaration mark is the one place that can carry both the subject role and the kind.

## 2. The model

### 2a. Std carrier (lands first: model before implement)

In `std.types` (or a new `std.where_predicate`, if `std.types` must not grow), the declaration kind is a closed coproduct:

```
type WherePredicateKind
  = RefinementPredicate { subject: <formal identity> }   // returns Bool over the subject
  | MarkerPredicate                                     // decides nothing about the value; never evaluated
```

The kind is the value of the declaration's marker. It is not inferred from the return type. A `refinement` fn whose return type is not `Bool` refuses at the declaration. So does a `marker` fn that declares a `subject` formal.

### 2b. Surface (both grammars). Precedent: `test fn`

```
refinement fn range(subject value: Int, min: Int?, max: Int?) -> Bool { ... }
refinement fn gt_zero(subject value: Int) -> Bool { value > 0 }
marker fn brand(name: String) -> WherePredicateMarker { NominalBrand { name: name } }
```

- v2 (`v2.extdeps.languages.dag`): this is a contextual production, exactly like `dag_grammar_test_fn_decl_expr`. It is an ident lexeme followed by `fn_decl`, and the body-lowering arm admits only the lexemes `test`, `refinement` and `marker`, refusing any other with a typed diagnostic. Option to decide at the ruling: fold all three into one `fn_marker_decl` production with a closed marker vocabulary, rather than adding two siblings to `test`.
- The `subject` formal is also a contextual lexeme, in parameter position, admitted only under a `refinement` fn. Exactly one is allowed, and it must be the first formal (fixing position avoids an order fact in two places).
- v1 seed (`v1.compiler.parse`): the same two rows. The seed only has to parse the markers and erase them. It does not need to enforce them, because v1's where tables are already a separate frozen authority (`gunbc.where_refinement_predicate_vocabulary`). This is the operator-ruled change.

### 2c. Lowering, resolve, infer

- Lowering: the declaration's lowered form carries the kind (and the subject formal's identity) as a named edge. The where-call lowering is unchanged: still no subject edge.
- Resolve: unchanged. It binds the operator.
- Infer gets one new arm, which replaces #12506's frontier row. For each predicate call in a where clause, look up the operator's declaration via symbol_index and read its kind:
  - `MarkerPredicate`: accept, by kind.
  - `RefinementPredicate { subject }`: judge the refined carrier against the subject formal's declared type through `v2.std.inhabitance declared_type_inhabitance`, the one existing judgment, so infer has no second notion of the subject. This matters because `PositiveInt = Nat where gt_zero(Int)` and `SecretValue = Secret where string_non_empty(String)` cross carriers. Positional args bind to the non-subject formals in order. Each record label binds to the formal of the same name. An unknown label refuses, and so does a missing non-optional formal.
  - A declaration with no kind (a plain fn named in a where clause) refuses with a new located reason, `infer_reason_where_predicate_kindless`.

## 3. Migration of the population

- `std.types`: `gt_zero`, `range` and `string_non_empty` become `refinement fn`, and `brand` becomes `marker fn`.
- `std.content_hash`: `lower_hex_16/40/64/128` become `refinement fn`.
- `std.unicode` (`unicode_scalar`): becomes `refinement fn`.
- Enumerate the full population with a corpus walk over where-clause operators (the instrument named by `gunbc.where_refinement_predicate_vocabulary`), not by grep. Every spelling must land on a marked declaration in the same PR, or the new kindless refusal reds it. That deletion is the census (§3 replacement migration).
- The `//` annotation in `std.types` shrinks to rationale only: why the subject is unwritten. The rule itself moves to the marker.
- `gunbc.where_refinement_predicate_vocabulary` `WherePredicateGrounding` (`GroundedByDeclaration` / `MarkerByDeclaration`) restates the kind. It should be derived from the declaration's kind, or deleted. It must not stay as a second, hand-authored copy.

## 4. Controls (expecting-red written before the wall; DESIGN §4b)

1. `type P = Int where gt_zero`: accepted (the subject matches).
2. `refinement fn positive(subject s: String) -> Bool` used on `Int`: refuses at the predicate.
3. `where range(min: 1, max: 5)`: binds by label, accepted. `where range(max: 5)` is also accepted (min Absent).
4. `where brand("X")`: accepted by kind.
5. `where range(mn: 1)`: refuses (unknown label).
6. A plain `fn` used as a where predicate: refuses (kindless).
7. Declaration walls: a `refinement fn` returning non-Bool refuses, and a `marker fn` with a `subject` formal refuses.
8. Inhabitance route: `Nat where gt_zero` is accepted through inhabitance, not equality. The ROUTE is asserted, not only the verdict (§3 pairing).

## 5. Sequencing

1. Std carrier, plus this plan.
2. Grammar rows: v2 and the seed (gated on the operator ruling).
3. Migrate the declarations, and make the annotation rationale-only.
4. After #12506 lands: replace its frontier arm with the infer judgment, retire the frontier row, and enroll the controls.

Steps 2–4 can be one PR. The replacement must be atomic at its root (the frontier row and the kindless refusal switch together).
