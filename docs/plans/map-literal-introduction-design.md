# Map-literal introduction: the model (MQ, PR1)

Status: MODEL. No lowering, resolve, infer or realization change lands here. PR2 adds the map arm
to the ONE elaboration writer that #12711 (anonymous record literals, Form B) builds,
`resolve_construct_walk`. It builds no second site: one writer, both arms.

## The symptom and the slice

`dag/std/types` is file-refused on the native route at `data kernel_type_set: Map<String, Bool> = {
"String": true, .. }`, so every module importing `std.types` is refused with it. #12711 (revised)
lowers a headless brace to `construct_node` with its tag edge targeting
`construct_tag_elided_marker`. Resolve's construct-tag writer then elaborates the brace from a
top-down `ResolveContext.expected`. Infer is a bottom-up fold and cannot synthesize an elided
construct, so elaboration is in resolve, not infer. Under a `Map` head, that writer refuses today
with `map_literal_construction_not_modeled`: there is no modeled introduction to elaborate TO.

## The gap, and why it closes without a new std constructor

`v2.std.list_introduction` heads a list literal with `std.algebra` `FreeMonoid`. That is lawful
because `FreeMonoid` is free and inductive: its elements determine its inhabitant. `Map<K, V>`
aliases `std.algebra` `FinitelySupportedFunction<K, V>`, which is a record of operations and not a
free structure, so a literal has no type constructor to head. That is the gap.

It closes WITHOUT a new `std.algebra` declaration (review condition 1). The carrier already
declares `empty` and `insert`, and the corpus already binds them to reachable, host-bound
realizations: `v2.std.collection` `empty_map` and `map_insert`, which `dag/std/types` itself uses.
A map literal therefore DENOTES the fold of `insert` over `empty`, in authored order. The objection
in the brief was to lowering inventing that fold ad hoc. Here the fold lives in the one authority,
`v2.std.map_introduction`, so it is not a second spelling anywhere. An earlier draft of this PR
proposed `std.algebra` `finitely_supported_function_from_graph`. It is dropped: it established
nothing the fold does not, and it would have been a second constructor beside `empty`/`insert`.

## The model: `v2.std.map_introduction`, one authority

It is modeled on `v2.std.list_introduction`, which carries head path, constructor and reader
together:

- **Head.** `map_introduction_head_path()` = `[^v2, ^std, ^map_introduction,
  ^map_from_entries]`. A map literal is ONE Transform headed by a reference to that declaration.
- **Entries.** One positional `MapIntroductionEntry { key, value }` construct per entry, in authored
  order. `MapIntroductionEntry<K, V>` is declared in this module. `std` has no general product, and
  minting one for a single consumer would be premature. `{}` under a `Map` expected type is the head
  alone.
- **Denotation.**
  `map_from_entries<K, V>(entries: List<MapIntroductionEntry<K, V>>) -> Map<K, V>` is
  `fold(entries, init: empty_map(), f: map_insert)`, and it is defined ONLY here.
- **Constructor and reader.** `lower_map_introduction(entries, source)` and
  `map_introduction_entries_optional(node)` read and write the same head path, so the producer and
  the readers cannot disagree.
- **The duplicate check lives here too** (next section), as
  `map_introduction_duplicate_key(keys, equality) -> Outcome<Unit, MapIntroductionDuplicateKey>`.

## Duplicate keys: checked BEFORE the fold, by the key type's declared equality

`insert` is last-write-wins, so the fold alone would drop a duplicate silently (review
condition 2). The check therefore runs before the fold, at elaboration, and it is a total function
in the authority:

- `MapIntroductionDuplicateKey { first_index, second_index }` is a refusal VALUE naming both
  occurrences. Resolve maps `second_index` to that entry's occurrence and reports
  `resolve_anonymous_map_duplicate_key` there, naming the key and the first occurrence.
- **Equality is the KEY TYPE's declared equality, never assumed structural equality.** Its home is
  the one that already decides equality admissibility: `std.algebra`
  `algebra_profile_equality_extensional`, a consequence of `algebra_profile_support`. Scalars and
  finite-support carriers have decidable extensional equality. Open-support carriers
  (`PartialFunction`, the pointwise power) and arrows do not. This PR adds that declaration to the
  home authorities of the keys / hashing conformance row (`gunbc.design_argument`
  `conformance_domains`, `conformance-identity`) and widens the row's scope sentence to match,
  rather than asserting the extension in prose.
- **Honest coverage at this rung.** That admission is instantiation-blind, by its own declared
  boundary (v1 infer `equality_admission_wall_note`, boundary 1). An opaque brand, or a generic key
  type parameter, ADMITS unjudged. So the undecidable-equality refusal and its control cover
  open-support carriers and arrows only. A map literal whose key type is a type parameter or an
  opaque brand is admitted UNJUDGED at this rung, and the control does not claim otherwise. Next-rung
  trigger, shared with that boundary: instantiation-grain admission, which judges equality where the
  type arguments are known.
- **Decidable at the site, or refuse.** Keys are compared as elaborated literals of `K`. Where `K`'s
  equality is not decidable at elaboration, the literal refuses located at its first key with
  `resolve_anonymous_map_key_equality_undecidable`, and nothing is guessed. That covers an
  open-support `K`, and any key that is not a literal of a scalar carrier. For string-literal keys,
  equality is equality of the unfolded scalar sequences after escape normalization, the declared
  text unfold of DESIGN section 4, and never spelling.

Every accepted introduction therefore has pairwise-distinct keys, so the fold's last-wins is
unobservable, and eval and emit may realize the fold however the target prefers.

## Lowering: the edge form of a string-keyed item

A name-keyed item keeps its `Named { name }` field edge (#12711). A string-keyed item lowers to an
edge labelled `MapLiteralEntry` (new, `v2.std.node`). Its target is a node with two positional
children, `(key literal, value)`. Lowering records the key kind and never chooses map or record. A
mixed brace refuses at lowering (#12711).

## Typing: what resolve decides, and what infer still checks

**Expected type, syntactic only** (binds both arms, per the #12711 approval conditions). The
expected `Map<K, V>` is peeled syntactically from an AUTHORED annotation: the data declared type,
the fn return, a record field's declared type, or a `List<T>` element. `Map` reaches its carrier
through resolve's existing alias lookup and nothing more. The writer refuses located, and never
guesses or unifies, whenever reaching a closed `Map<K, V>` would need inference, type-variable
instantiation, or deeper alias unfolding. The expected type is cleared at every other position,
such as a call argument or an unannotated `let`, so it cannot leak.

**The resolve map arm** elaborates to the introduction. For each entry value it sets one more
`ResolveContext.expected` position rule, map entry value gets `V`, so nested headless values
elaborate through the same writer. It refuses located at the offending entry:

- `resolve_anonymous_map_no_expected_type`: a string-keyed brace at a cleared position.
- `resolve_anonymous_map_expected_type_not_closed`: the expected type is not syntactically a closed
  `Map<K, V>`.
- `resolve_anonymous_map_key_kind_mismatch`: `K` does not admit a string-literal key. This is
  decided on the declaration `K` resolves to, never on its leaf name. `Map<Int, Bool>` with
  `{ "k": true }` refuses at `"k"`, because a text crossing goes through the declared unfold or
  refuses.
- `resolve_anonymous_map_key_equality_undecidable` and `resolve_anonymous_map_duplicate_key` (above).

**Infer still checks, because resolve's choice is not proof.** Infer types the elaborated node as an
application of `map_from_entries` over `MapIntroductionEntry` constructs. Its result is checked
against the declared type, and each value is checked against `V` by the ordinary construct-field
check. A value that does not inhabit `V` refuses in infer after a successful elaboration.

## Consumers (PR2), a declared frontier: every new declaration is consumed in PR2

| New declaration | Consumer, and route at execution |
|---|---|
| `map_introduction_head_path` | `lower_map_introduction` and `map_introduction_entries_optional` (one spelling) |
| `lower_map_introduction` | the Map-head arm of `resolve_construct_walk` (owned by sleek-fox-423 under #12711; this lane adds the arm) |
| `map_introduction_entries_optional` | `v2.compiler.infer` (reader arm beside `infer_transform_freemonoid_introduction`), `v2.compiler.eval`, `v2.std.compilers.target_model` / emit |
| `map_from_entries` | the head the introduction names: infer types the node as its application; eval evaluates it; emit realizes it via `empty_map`/`map_insert` |
| `MapIntroductionEntry` | constructed by the resolve arm, typed by infer, read by eval and emit |
| `map_introduction_duplicate_key`, `MapIntroductionDuplicateKey` | called by the resolve arm before elaboration; its refusal value locates `resolve_anonymous_map_duplicate_key` |
| `v2.std.node` `MapLiteralEntry` edge label | written by body lowering, read by the resolve arm |
| `ResolveContext.expected` rule: map entry value gets `V` | the same resolve walk, recursing into nested values |
| the `resolve_anonymous_map_*` refusals | `v2.workflow.compile_door_cause_ownership`, one row each; `map_literal_construction_not_modeled` is retired |
| control `a_map_literal_value_refuses_at_the_literal_holds` | flips to positive control 1 |
| #12711's `gunbc.rung_drop` map population | retired by PR2; its trigger is this capability |

## PR2 controls

1. `dag/std/types` `kernel_type_set` elaborates to a Transform headed by `map_from_entries` with
   eight `MapIntroductionEntry` children in authored order. It is compared by `content_hash` against
   the hand-built introduction.
2. A duplicate key refuses `resolve_anonymous_map_duplicate_key` at the second occurrence, naming
   the key.
3. Distinct keys are accepted. This includes keys that differ only after escape normalization in
   the non-equal direction.
4. A key type without decidable equality (an open-support carrier or an arrow) refuses
   `resolve_anonymous_map_key_equality_undecidable`. Coverage stops there: a type-parameter or
   opaque-brand key admits unjudged at this rung (see Honest coverage).
5. `Map<Int, Bool>` with `{ "k": true }` refuses `resolve_anonymous_map_key_kind_mismatch` at `"k"`.
6. Mutation: the pre-fold check removed, so the fold's last-wins absorbs the duplicate, turns
   control 2 red. The control reads the resolve outcome, not the evaluated map.
7. Infer still checks: `Map<String, Bool>` with `{ "k": 1 }` elaborates and then refuses in infer
   at `1`.
8. No leak: a string-keyed brace as a call argument refuses
   `resolve_anonymous_map_no_expected_type`.
9. Evidence: a base-vs-head native-route census (recipe on #12550) showing `dag/std/types` and every
   other map-literal file admitted, with no unexplained new refusal.
