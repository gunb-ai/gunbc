# Design note — the constructor arm of the native binder (before code)

Scope as ruled by tidy-ant-251 (2026-11-02): extend v2's existing construct machinery so an **authored**
constructor tag in a nested pattern or a record head resolves through the **declared constructor set of
its expected type**, and the elided `Map { … }` / `PartialFunction { … }` heads bind the same way.
Refusals stay for ambiguity and for a tag that is not in that set. The tag is never resolved by a
name-shaped guess. Reading: tree `d77e2d8a`. No code, no host run yet.

## What the native binder does today (the exact producer)

One function resolves the tag for **both** a construction and a pattern — `resolve_construct_walk` in
`v2.compiler.resolve`:

```
fn resolve_construct_walk(ctx, n, reference, field) -> ResolveNodeWalk {
  resolve_construct_walk_with_tag(n: n, tag: resolve_construct_tag_walk(ctx: ctx, reference: reference), field: field)
}
```

and `resolve_construct_tag_walk` → `resolve_construct_tag_reference` → for a bare single-segment tag:

```
resolve_construct_tag_from_payload_lookup(
  ctx, reference, identity: only,
  lookup: symbol_index_lexical_lookup_for_declared_payload(index: ctx.namespace.symbol_index,
            position: symbol_index_lookup_position(ctx), name: only))
```

`LexicalUnbound` then calls `resolve_atom_unbound` → `resolve_reason_unbound_symbol`. **The tag is looked
up by name, and nothing else.** No parameter of that walk carries a type, so there is no arm that can
read the constructor set of the type the reference stands in. That single fact produces the whole A2 set
(`Value`, `TypeNode`, `Found`, `None`, `Accepted`, `Equal`, `Present`, `ContentHashDifferent`,
`RuntimeAggregate`, `ReceiverElementOrSelf`, and the `Map`/`PartialFunction` heads): in every one of those
96 census modules the reference had no name row, and a constructor tag can never have one that the
expected type does not already imply.

The expectation machinery the fix needs **already exists and is already the shape of the answer**:

* `type ResolveExpectation { expected: Node, position: QualifiedName, at_use_site: Bool }`.
* `resolve_expected_edge` consumes it, and its match on `construct_tag_reading` has exactly the arm I need
  to extend: `ConstructTagElided => resolve_elided_construct_walk(ctx, n, x)` and
  `ConstructTagAuthored { reference: _ } => resolve_child_edge(ctx, e)` — i.e. **for an authored tag the
  expectation is dropped on the floor**; the tag itself is never re-read against `x.expected`.
* `resolve_construct_walk_with_tag` already hands the tag's resolved declaration path down to the field
  rule, and a value field under a record construct is walked with its authored field type as the
  expectation (that is how a headless literal nested under an authored head elaborates today).
* `OccurrenceResolution` already has the identity shape for this verdict: `BoundByExpectedType { identity }`,
  documented as "a headless construct elaborated from an explicitly carried expected type", read by the
  binding census as a row of its own (source `expected_type`).
* The arm reader exists in `v2.std.node_query`: `coproduct_arm_list_from_node`, `coproduct_arm_key_list_from_node`,
  `coproduct_arm_from_edge`, `arm_payload_from_target`, `arm_payload_type_name`,
  `coproduct_nullary_payload_type_name`.

So this is not a new mechanism: it is connecting an existing door (expected type → declared arm set) to
the one call site that never receives it.

## The change

1. `resolve_construct_walk` / `resolve_construct_walk_with_tag` / `resolve_construct_tag_walk` gain an
   **expected-type parameter** (`Optional<Node>`, the resolved declaration node of the expected type, not
   a spelling). Threading points, each of which already has the type in hand:
   * `resolve_expected_edge`'s `ConstructTagAuthored` arm — pass `x.expected` (covers a value under an
     annotated arrow body, a record field with its authored field type, a list element).
   * `resolve_pattern_node_walk`'s nested pattern under a record/pattern field — the field's declared type
     from `resolve_record_field_type_optional(ctx, record, field)`, the same reader
     `resolve_pattern_binder_declared_type` uses. This is what carries `behavior: Value` (field type
     `Behavior`) and `kind: TypeNode { … }`.
   * `resolve_match_arm_walk` / `resolve_match_arm_admitted` — the scrutinee's declared type.
2. In `resolve_construct_tag_reference`, the **single-segment** case gains a second door, and the door
   order is a decision, not an accident (see below):
   * no expected type at the site → today's name door, today's `resolve_atom_unbound` refusal, unchanged;
   * expected type known → read its declared arm list; if exactly one arm's key equals the tag spelling,
     bind the tag to **that arm's declaration** with the census-visible identity
     `BoundByExpectedType { identity }`;
   * expected type known and **no** arm matches → refuse with a new typed cause
     (`resolve_construct_tag_not_in_expected_type`), naming the expected type — **not** `unbound_symbol`,
     and **not** a fall-back to the name door;
   * two arms match, or the expected type itself has no unique resolved declaration → refuse as ambiguous.
3. MEASURED, so the manager's elided-head half is narrower than the census suggests. The `Map { … }` and
   `PartialFunction { … }` refusals are the **type name** in annotation position
   (`exports: Map<ModuleId, List<Node>>` in v2.std.compilers.compilation_unit;
   `keyword_literals: Map<String, LiteralValue>` in std.syntax), where those modules'
   `import std.types { NonEmptyStr, List, Bool, String, Int, brand }` rows omit `Map`. A type annotation
   carries no expected type, so **this arm cannot and must not bind them** — they are the membership
   lane's. Where the type name does have a home the construct head already works today
   (`v2.test.claim.map_carrier_shape_gate` imports `std.types { Int, Map }` and constructs `Map { … }`;
   `v2.compiler.infer` imports `std.algebra { … PartialFunction … }` and writes both
   `PartialFunction { lookup: … }` and `PartialFunction<Node, InferredFacts>` without refusal). I will add
   a control RED asserting exactly that: those two heads stay bound by the type-name door, unchanged.

## Door order — the decision I want reviewed before I write it

Name-first leaves today's accepted verdicts untouched but lets a same-named constructor imported from a
different sum outlive the expected type: `Node` is a type in `v1.std.core` **and** `v2.std.node`, and a
constructor in several sums; `Optional` is a constructor of `v2.std.grammar.GrammarExpr` and a type in
`std.optional`. A name door that wins at a site with a known expected type is the collision the census
shows. My recommendation, and what the note assumes: **at a site whose expected type is resolved, the
arm list is the authority; the name door applies only where no expected type is carried.** Then
`Node`/`Optional` at a site with no expected type keep refusing today's way, and at a site with one they
bind to that type's own arm — or refuse if they are not in it. This is what keeps the rung row's standing
sentence true ("the global-bare Unbound arm refuses rather than falling back to one").

## RED list (one per changed arm) and controls

Every RED is written as a claim module and is RED on main, green after the change — no probe of today's
wrong answer here, because today's answer is a refusal.

| id | site shape | today | after |
|---|---|---|---|
| R1 | authored tag in a **nested pattern field**: `ComputationNode { behavior: Value }` in a module importing `v2.std.node` without naming `Value` | `resolve_reason_unbound_symbol` | binds `Behavior`'s `Value` arm, identity `BoundByExpectedType` |
| R2 | authored tag under a **scrutinee type** in an expression: `Accepted { … }` where the match/annotation says `Outcome` | unbound | binds `Outcome`'s arm |
| R3 | **unchanged-control**: `Map { lookup: … }` with `Map` named, and `PartialFunction { lookup: … }` with `PartialFunction` named | binds through the type-name door | binds the same way, same identity — proves the new arm did not absorb the type-name door |
| R4 | tag **not in** the expected type's set (`behavior: ValueX`) | unbound_symbol | new typed cause naming the expected type |
| R5 | **ambiguous** expected type (two candidate declarations) | — | refuses as ambiguous; no silent pick |
| R6 | **collision control**: `Node` / `Optional` at a site with no expected type | unbound | **still refuses** — this is the control that the new arm did not become a name guess |
| R7 | **elided-literal controls unchanged**: `resolve_anonymous_record_no_expected_type`, `resolve_anonymous_record_expected_type_not_record`, `resolve_anonymous_map_expected_type_not_map`, `resolve_anonymous_map_expected_type_not_closed` | refuse | refuse, same cause |
| R8 | **binder path unchanged**: the fold-cons binder defect (`resolve_pattern_binder_declared_type`) | refuses | refuses — that is a separate rung row and must not be silently fixed here |

Controls at the corpus level, because `v2.compiler.resolve` is load-bearing:
* the whole-corpus native binding census before/after: every diff must be (unbound → bound) **for a tag
  that is an arm of the expected type**, and there must be **no** (bound → bound-with-a-different-identity)
  row at all;
* the existing binding-provenance rows keep their verdicts, and new rows carry source `expected_type`;
* the two scope-not-name pairs my char_at lane already measured green (`base16` 15/0, `yaml_ingest` 32/0)
  stay green;
* host route: fleet instrument-dispatch (BuildBuddy caps at 1 h), one row per leg.

## Open questions I am not deciding alone

1. Door order (above) — recommend expected-type authority where the type is resolved.
2. Measured: `resolve_expected_edge` is called from four places (3820, 3853, 3912, 5399) and **all four are
   value-position walks** — an arrow body, a record field under its authored type, a list element, an
   annotated literal. There is **no pattern-position caller**, so R1's threading through
   `resolve_pattern_node_walk` is genuinely new plumbing and the largest part of the change. It must reuse
   `resolve_record_field_type_optional` (the same reader `resolve_pattern_binder_declared_type` already
   uses for binder types) rather than invent a second field-type reader (DESIGN 6b).
3. Whether an authored tag under an expected type is ever ALSO bound by the name door to a different
   declaration in the corpus today (the `Node`/`Optional` collision shape). R6 is the claim that probes it;
   if it turns out such a site exists and is accepted, the door order in the section above is what decides
   it, and I will bring the site rather than assume.
4. Whether `BoundByExpectedType` census rows for authored tags need a new source label to distinguish
   them from headless-literal elaboration rows (`source expected_type` is currently the only source).
