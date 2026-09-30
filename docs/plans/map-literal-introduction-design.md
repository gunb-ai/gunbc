# Map-literal introduction: the model (MQ, PR1)

Status: MODEL. No lowering, resolve, infer or realization change lands here; PR2 fills the map arm of the
one declared-type check site that #12711 (anonymous record literals, Form B) builds. It builds no
second site.

## The symptom and the slice

`dag/std/types` is file-refused on the native route at `data kernel_type_set: Map<String, Bool> = {
"String": true, .. }`, so every module importing `std.types` is refused with it. #12711 routes a
headless brace to the one elaboration writer, `resolve_construct_walk` in resolve, which carries a top-down `ResolveContext.expected` (revised in #12711; infer is a bottom-up fold and cannot synthesize an elided construct). There a `Map` expected head
takes the map arm, which refuses located `map_literal_construction_not_modeled`. That refusal is
correct: the arm has nothing to elaborate TO. The earliest unjustified boundary is not lowering and
not the writer. It is `std.algebra`: **`Map` has no introduction declaration to name.**

## Why the list precedent does not transfer as-is (the modeling gap)

`v2.std.list_introduction` heads a list literal with a reference to `std.algebra` `FreeMonoid`. That
is lawful because `FreeMonoid<T>` is a FREE, inductive type (`Empty | Cons { head, tail }`). Its
elements determine its inhabitant, so "the head plus positional elements" DENOTES a value, and
`[]` is the head alone.

`Map<K, V>` aliases `std.algebra` `FinitelySupportedFunction<K, V>`. That is a RECORD OF OPERATIONS
(`lookup`, `empty`, `insert`, `merge`, `map_keys`, ..): an interface a finite map answers, not a
structure it is built from. Heading a Transform with it and hanging `(key, value)` pairs under it
would claim a constructor that the declaration does not have. That is the same error as a
`Map { lookup: .. }` literal naming ten operations and supplying one (see the `std.types` alias
note). Folding over `empty_map`/`map_insert` in lowering is ruled out by the brief. It would also be
realization choosing semantics: `map_insert` is last-wins, so it is exactly the silent overwrite
this model must refuse.

## The model

A finitely supported function is introduced by its **graph**: a finite, functional relation, one
entry per supported key. The introduction declaration is authored in `std.algebra` next to the
carrier it introduces:

```
// the graph of a finitely supported function: entries in authored order, keys pairwise distinct
type FiniteGraphEntry<K, V> { key: K, value: V }
fn finitely_supported_function_from_graph<K, V>(entries: FreeMonoid<FiniteGraphEntry<K, V>>)
  -> FinitelySupportedFunction<K, V>?   // Absent exactly when two entries share a key
```

- **Head.** `std.algebra.finitely_supported_function_from_graph`: the declaration a map literal
  names, as `FreeMonoid` is the declaration a list literal names.
- **Children.** One positional child per entry, in authored order. Each is a
  `FiniteGraphEntry { key, value }` construct, so the entry is an ordinary record construct with no
  new edge kind. `{}` under a `Map` expected type is the head alone, the empty graph.
- **Authority.** `v2.std.map_introduction` carries head path, constructor
  (`lower_map_introduction`) and reader (`map_introduction_entries_optional`) together, exactly as
  `v2.std.list_introduction` does. The map arm of `resolve_construct_walk` is its producer, and infer, eval and
  emit are its readers. There is one spelling of the path.
- **Why a function and not a type.** The relation is not free: a list of entries with a repeated
  key does not denote a function. The introduction is therefore partial, and its partiality is the
  duplicate-key law below. It is not a hidden `insert` fold.

## Typing rule

The expected `Map<K, V>` comes ONLY from an authored annotation (data initializer, fn return, record
field, list element) through `ResolveContext.expected`, resolved through the alias to
`FinitelySupportedFunction<K, V>`. It is never inferred. A headless string-keyed brace with no
authored expected type refuses located (`resolve_map_literal_no_expected_type`). The work splits
along what each stage can decide:

- **Resolve (the map arm of `resolve_construct_walk`)** elaborates the brace to the introduction,
  sets `ResolveContext.expected` to `FiniteGraphEntry<K, V>`'s `value` field type `V` for each
  entry value (so a nested headless value such as `Map<String, Rec>` with `{ "a": { f: 1 } }`
  elaborates recursively through the same writer), and refuses, located at the offending entry:
  - `resolve_map_literal_key_type_mismatch`: `K` does not admit a string-literal key. Keys are
    string literals by grammar, and under DESIGN section 4 a text crossing goes through the declared
    unfold or refuses, so `Map<Int, Bool>` with `{ "k": true }` refuses at `"k"`. This is decided on
    the declaration `K` resolves to, never on its leaf name.
  - `resolve_map_literal_duplicate_key` (below).
  - a mixed name-key/string-key brace already refuses at lowering (#12711).
- **Infer** needs no map-specific arm. The elaborated node applies a declared function to
  `FiniteGraphEntry` constructs, so values that do not inhabit `V` refuse through infer's ordinary
  construct-field check. This is one more reason the head must be a declaration: typing falls out
  of inhabitance, with no second rule.

## Consumers (PR2), a declared frontier

| Consumer | Role | Change |
|---|---|---|
| `v2.std.map_introduction` | authority | new: head path, constructor, reader |
| `std.algebra` | declaration | `FiniteGraphEntry`, `finitely_supported_function_from_graph` |
| `resolve_construct_walk` map arm (#12711, owned by sleek-fox-423) | producer | replaces `map_literal_construction_not_modeled` with elaboration + three located refusals |
| `v2.compiler.infer` | reader | no new arm; values typed through the `FiniteGraphEntry` construct |
| `v2.compiler.eval`, `v2.std.compilers.target_model`, emit | readers | realize the introduction (graph → finite container) |
| `v2.workflow.compile_door_cause_ownership` | ownership | one row per new refusal; `map_literal_construction_not_modeled` retired |
| `reference_conservation_accepted_drops` `a_map_literal_value_refuses_at_the_literal_holds` | control | flips to a positive control: `std/types`'s literal elaborates to exactly the introduction |
| #12711's `gunbc.rung_drop` (map population) | drop | retired by PR2; its trigger is this capability |

## PR2 controls

1. `dag/std/types` `kernel_type_set` elaborates to a Transform headed by
   `finitely_supported_function_from_graph` with eight `FiniteGraphEntry` children in authored
   order, compared by `content_hash` against the hand-built introduction.
2. A duplicate key refuses `resolve_map_literal_duplicate_key` at the second occurrence.
3. `Map<Int, Bool>` with `{ "k": true }` refuses `resolve_map_literal_key_type_mismatch` at `"k"`.
4. Mutation: dropping the duplicate check, so the graph reaches eval and `map_insert` last-wins,
   turns control 2 red. The control reads the resolve outcome, not the evaluated map.
5. Evidence: a base-vs-head native-route census (recipe on #12550) showing `dag/std/types` and
   every other map-literal file admitted, with no unexplained new refusal.

## Open for review

- Home and name of the introduction. The proposal is `std.algebra`, beside the carrier. The
  alternative is `v2.std.collection`, but that is realization-adjacent and would invert the layer.
- Whether `FiniteGraphEntry` should be a general product (`std` has no `Pair`) rather than a
  map-specific record. Minting a general product only for this use would be premature, so the
  proposal is the specific record until a second consumer appears.

