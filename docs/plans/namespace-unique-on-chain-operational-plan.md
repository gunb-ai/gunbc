# Namespace unique-on-chain — operational plan for lexical binders

**Status:** proposed implementation plan for review; no implementation authority.

**Authority:** [namespace-resolution-design.md §13](namespace-resolution-design.md#13-resolution-is-unique-on-chain-not-nearest-operator-ruling-ratified-2026-07-21).
That operator-ratified section supersedes the older nearest-wins wording in §§3/6:
resolution considers the full visible ancestor chain, `0` refuses unbound, `1` binds,
and `2+` refuses with the full candidate population. In particular, **shadowing is a
refusal, never a silent rebind**.

**Trigger:** PR #7328 attempted to preserve exact match-pattern binder identity for the
generic-match ownership prerequisite. Review 43794 found that ordinary `let` and lambda
scope construction retained a stale, name-keyed `match_bound_names` marker. The first
proposed repair removed the marker when a nearer binder arrived, thereby making
nearest-wins shadowing explicit. That repair contradicts §13 and is not ratified.

This plan operationalizes the existing namespace authority. It does not create a
match-shadowing subsystem, change `std.occurrence_binding`, or authorize edits to
load-bearing resolver/emitter files.

---

## 1. One question, one authority

There is one semantic question:

> Which declarations are valid candidates for this occurrence at this containment
> position?

The answer is derived from the containment tree and the occurrence's syntactic
reference category. It is not assembled independently by `locals`,
`match_bound_names`, function maps, global-bare maps, or import reachability.

For a reference whose first segment is `s`:

1. Classify the occurrence position using existing syntax:
   `ValueReference`, `TypeReference`, or the already-ruled typed
   pattern/member projection.
2. Walk the occurrence's complete chain of enclosing containment scopes.
3. Collect every declaration exposed by a scope on that chain whose declared
   segment is `s` and whose declaration category is admissible at that occurrence
   position.
4. Preserve exact declaration containment identity and deterministic source order.
5. Collapse duplicate discovery of the *same full containment identity* only.
6. Call the one canonical `occurrence_binding_from_candidates` fold exactly once.
7. Continue only through `OccurrenceBound`.

`OccurrenceUnbound` and `OccurrenceAmbiguous` are terminal, located refusals.
`OccurrenceAmbiguous` retains the complete ordered population. No nearest-level
selection, priority ordering, fallback lookup, name re-lookup, or expected-type
winner is permitted.

“Category” is a property of a declaration/reference position in the one namespace
tree, not a second namespace mechanism. A value occurrence does not admit a type
declaration merely because its text is equal. Within the value category, function
parameters, lambda parameters, `let` declarations, pattern binders, and callable
value declarations are not separate priority tiers.

Projection after the first segment is unchanged: once the first segment is uniquely
bound, remaining segments descend through the one `.`/containment projection.

---

## 2. What counts as visible

Visibility is structural:

- A declaration exposed by an enclosing scope on the occurrence's ancestor chain is
  visible. This includes, for example, a function parameter exposed by the function
  scope and a sibling function exposed by the enclosing module scope.
- A declaration exposed only by a sibling arm or sibling nested scope is not on that
  chain and is not a candidate.
- A declaration below the occurrence is not a candidate.
- A declaration made reachable only by an import list is not a namespace candidate;
  imports are transitional dependency/loading data, not naming authority.
- A builtin is a root declaration and is therefore on every applicable chain.
- An alias is an ordinary declaration node and participates by the same rule.

Structural visibility may exclude off-chain declarations. It may **not** select the
nearest declaration from two same-category declarations exposed by two scopes that
are both on-chain.
That distinction corrects the phrase “structural rung before cardinality” in the P1
charter: structural admissibility defines one population; cardinality is then folded
once over the full admitted population.

### Sibling-arm example — legal and unique

```dag
match input {
  Left  { value: y } => y
  Right { value: y } => y
}
```

Each `y` reference has only its own arm's declaration exposed along its enclosing
scope chain. The equal text does not create ambiguity because the declaration paths
are disjoint.

### Nested `let` example — ambiguous at the reference

```dag
match input {
  Present { value: y } => {
    let y = 1
    y
  }
  Absent => 0
}
```

The final `y` has both the pattern declaration and the `let` declaration exposed by
scopes on its ancestor chain. It must produce `OccurrenceAmbiguous` with both exact
containment paths. Removing either candidate changes the result to
`OccurrenceBound`.

### Nested lambda example — ambiguous at the reference

```dag
match input {
  Present { value: y } =>
    map(items, fn(y) { y })
  Absent => []
}
```

The lambda-body `y` sees both declarations and refuses. A lambda parameter must not
silently clear the pattern candidate.

### Reference-site versus declaration-site refusal

The ratified §13 rule places the refusal at the **reference site**, not at declaration
construction. Therefore an unused nested homonym is not rejected by this plan:

```dag
match input {
  Present { value: y } => {
    let y = 1
    0
  }
  Absent => 0
}
```

Changing this to declaration-time rejection would be a stricter language amendment,
not an implementation detail. It requires an explicit operator amendment to §13 and
is not smuggled into #7328 or P1.

---

## 3. Exact identity flow — PR #7328 boundary

PR #7328's valid responsibility is to stop discarding exact pattern-binder identity:

```text
authored binder token SourceSpan
  → MatchPattern.Bind declaration identity
  → exact inferred binding edge
  → uniquely bound ExprVar projection
```

The span is the current v1 occurrence key needed to connect the authored declaration
to the inferred use. It is not the namespace identity by itself and is never joined
by raw name. The semantic declaration identity remains the declaration node at its
full containment path; `declaration_span` is the v1 bridge that lets downstream
ownership address that node without another name lookup.

#7328 must:

- preserve the authored binder span through parser/core/inference;
- prove equal-text binders in sibling arms remain distinct;
- prove nested patterns preserve each declaration's own span;
- prove a uniquely bound use carries its accepted declaration span;
- use authored `.dag` witnesses and normal regeneration only.

#7328 must not:

- remove an outer candidate when a nearer `let` or lambda parameter appears;
- decide 0/1/many namespace cardinality in a second helper;
- make `match_bound_names` a binding authority;
- add a shadowing-specific diagnostic or resolver;
- hand-edit or add handwritten stage0 Rust witnesses;
- implement generic-match ownership or change #7321.

A shadowing witness that expects ambiguity belongs to the executing namespace
consumer, not to the identity-carrier prerequisite. #7328 may retain a structural
fixture containing equal names only if it observes declaration identity without
claiming a winner for the ambiguous reference.

---

## 4. P1 / #7321 consuming edge

The fresh-main P1 endpoint remains the first production v1 consumer of
`std.occurrence_binding`. Its candidate producer must be amended as follows:

- For each ordinary type, value, and callee occurrence, construct the exact
  `BindingOccurrence<Node>` containment path.
- Collect the complete structurally admissible declaration population before
  folding.
- For lexical/value occurrences, “complete” means the full applicable ancestor
  chain, not the nearest nonempty lexical rung.
- Exclude sibling/off-chain declarations structurally.
- Dedupe only identical full `ContainmentPath<Node>` identities.
- Preserve same terminal nodes at different containment paths as distinct
  candidates.
- Call `occurrence_binding_from_candidates` once.
- Project `TypeBinding`/function signature/value semantics only from the accepted
  declaration terminal in `OccurrenceBound`.
- Emit the existing located `AmbiguousReference` diagnostic from the typed
  `OccurrenceAmbiguous` population. Diagnostic strings are projections, never
  candidate identity.
- Never recover through `match_bound_names`, `locals`, function maps,
  `global_bare`, import reachability, nearest-level selection, or a second lookup.

Builtins remain declarations at the structurally recognized root. Grounded builtin
operations that are not declarations remain their already-ruled separate authority;
they may neither enter the declaration population nor fabricate a declaration.

The current #7321 freeze remains in force. This plan does not bypass its P-derive or
match-ownership emitter prerequisites.

---

## 5. Generic-match ownership boundary

The ownership lane consumes a binding result; it does not resolve names.

After #7328 lands, the match-ownership implementation may use
`MatchBoundBinding.declaration_span` only as the exact declaration edge already
accepted by namespace resolution. It must:

- consume only uniquely bound pattern uses;
- treat absent, ambiguous, or conflicting ownership proof rows as typed refusal;
- never infer a binder from raw text;
- never make nearest-scope selection;
- never re-resolve through `match_bound_names`;
- derive target `Borrow | Move | Refuse` through the ruled ownership proof plus
  target representation evidence.

This preserves the sequencing without creating a cycle: #7328 lands the exact
identity fact; generic-match ownership fixes the emitter's invented clone; P1 then
executes the full namespace candidate fold and makes shadowing refusals observable.

---

## 6. Discriminating witness matrix

Every witness must execute the production candidate producer and canonical fold.
Algebra-only `std.occurrence_binding` claims are necessary but insufficient.

| Case | Candidate population | Required result |
| --- | --- | --- |
| one pattern binder, one use | `[pattern.y]` | Bound to exact pattern path |
| same text in sibling arms | each arm sees only its own `[arm.y]` | Both Bound, distinct paths |
| nested distinct names | `[outer.y]` or `[inner.z]` | Bound |
| pattern `y`, nested `let y`, referenced below | `[pattern.y, let.y]` | Ambiguous, ordered full population |
| pattern `y`, lambda parameter `y`, referenced in lambda | `[pattern.y, lambda.y]` | Ambiguous, ordered full population |
| same-scope duplicate value declarations, if grammar admits them | both exact declarations | Ambiguous at every affected reference |
| exact duplicate discovery path | one identity after producer dedupe | Bound |
| same terminal under different containment paths | two identities | Ambiguous |
| value `x` and type `x` where syntax admits both | category-admissible population only | No cross-category pick/collision |
| local value and callable value with same segment on-chain | both value declarations | Ambiguous |
| unused nested homonym | no occurrence to fold | No reference diagnostic under current §13 |
| unbound name | `[]` | Located Unbound |

For every 0/1/many case, perturbing the population must change the verdict. RED
controls remove the full-chain collection, restore nearest-wins, restore a
name-keyed lookup, or drop one ambiguous candidate; each must fail.

The corpus census before the strict consumer flips must count, by occurrence
category:

- unique full-chain bindings;
- unbound occurrences;
- two-candidate full-chain ambiguities;
- three-or-more full-chain ambiguities;
- candidate pairs split by declaration kind;
- sibling/off-chain equal-text declarations excluded structurally;
- exact duplicate discovery paths collapsed by identity.

The census is a migration worklist, never permission for a fallback. Repairs are
qualification, an ordinary alias, or rename; never import additions or priority
rules.

---

## 7. Sequencing and ownership

1. **This docs-only plan** is reviewed against §13 and the operator's no-parallel-
   subsystem requirement.
2. **#7328 exact identity prerequisite** is rebased/reworked to its narrow boundary:
   exact span flow, `.dag` witnesses, no silent shadowing, no handwritten Rust.
3. **Generic-match ownership prerequisite** consumes the exact uniquely bound edge
   and removes emitter-invented clone behavior.
4. **P-derive authority/consumer prerequisites** land independently.
5. **#7321 P1** rebases current main and implements the single full-chain
   occurrence-binding consumer, including the shadowing ambiguity matrix.
6. **Corpus census and generated repair** eliminate unbound/ambiguous residues by
   qualification/alias/rename, with no import additions.
7. **Reference-derived dependency/loading convergence** consumes accepted binding
   edges.
8. **Dispatch 2** deletes import lines and grammar only after the reference-only
   authority and full refusal matrix are green.

No step may use a default-off escape, nearest-wins compatibility arm, raw-name
identity, hand-generated Rust, or a second resolution carrier.

---

## 8. Done lines

This plan is implemented when all of the following are true:

- Exact pattern declaration identity survives parse → infer without raw-name joins.
- The production v1 resolver folds the full applicable ancestor-chain population
  exactly once through `OccurrenceBindingResult<Node>`.
- A nearer homonym cannot silently change an existing reference's meaning.
- Sibling-arm equal names remain independently and uniquely bound.
- Ambiguity diagnostics carry all exact candidates in deterministic order.
- Type/value/callee consumers continue only from `OccurrenceBound` and never
  re-resolve.
- The generic-match ownership consumer reads the accepted binding edge and has no
  name or clone heuristic fallback.
- The ordinary-compile matrix, v1 build, regen fixed point, diagnostic accounting,
  and full CI are green.

Only after those walls and the remaining reference-derived dependency gates are
green does import deletion become the mechanical Dispatch 2 operation.
