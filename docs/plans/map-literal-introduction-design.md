# Map-literal introduction: the model (MQ, PR1)

Status: MODEL. No lowering, infer or realization change lands here; PR2 fills the map arm of the
one declared-type check site that #12711 (anonymous record literals, Form B) builds. It builds no
second site.

## The symptom and the slice

`dag/std/types` is file-refused on the native route at `data kernel_type_set: Map<String, Bool> = {
"String": true, .. }`, so every module importing `std.types` is refused with it. #12711 routes a
headless brace to the declared-type check site in `v2.compiler.infer`. There a `Map` expected head
takes the map arm, which refuses located `map_literal_construction_not_modeled`. That refusal is
correct: the arm has nothing to elaborate TO. The earliest unjustified boundary is not lowering and
not infer. It is `std.algebra`: **`Map` has no introduction declaration to name.**

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
  `v2.std.list_introduction` does. The check site's map arm is its producer, and infer, eval and
  emit are its readers. There is one spelling of the path.
- **Why a function and not a type.** The relation is not free: a list of entries with a repeated
  key does not denote a function. The introduction is therefore partial, and its partiality is the
  duplicate-key law below. It is not a hidden `insert` fold.

## Typing rule

The expected type at the check site is `Map<K, V>`, resolved through the alias to
`FinitelySupportedFunction<K, V>`. Each entry's key is checked against `K` and each value against
`V`, as check positions (a new `PositionMapEntryKey` / `PositionMapEntryValue` pair in
`v2.std.inhabitance` `DeclaredTypePosition`, beside #12711's `PositionListElement`). So a nested
headless value, such as `Map<String, Rec>` with `{ "a": { f: 1 } }`, elaborates through the same
site recursively. Nothing is synthesised bottom-up and then unified. Refusals, each located at the
offending entry:

- `infer_map_literal_key_type_mismatch`: the key does not inhabit `K`. For `Map<Int, Bool>` with
  `{ "k": true }`, the refusal is located at `"k"`.
- `infer_map_literal_value_type_mismatch`: the value does not inhabit `V`.
- `infer_map_literal_name_key`: a bare-name key under a `Map` expected type. Keys are string
  literals by grammar, and a name key is the record arm's form. #12711 already refuses mixed kinds
  at lowering.

Keys are string literals by grammar today, so `K` must admit a string literal. Under DESIGN §4,
text crossings go through the declared unfold or refuse, so `Map<Int, V>` refuses at the key rather
than coercing.

## Duplicate keys: refuse, located, never last-wins

`infer_map_literal_duplicate_key` refuses at the SECOND occurrence and names the first. Equality is
equality of the elaborated key values in `K`, not of spellings. The check belongs to the
introduction's partiality, so it is decided at the check site where `K` is known. It does not wait
for eval, where `map_insert` would silently overwrite. Because the elaborated program never
contains a graph with a repeated key, eval and emit of the introduction may realize it as a
sequence of inserts without that realization choosing semantics. That is realization, not
authority.

## Consumers (PR2), a declared frontier

| Consumer | Role | Change |
|---|---|---|
| `v2.std.map_introduction` | authority | new: head path, constructor, reader |
| `std.algebra` | declaration | `FiniteGraphEntry`, `finitely_supported_function_from_graph` |
| `v2.compiler.infer` check site, map arm (#12711) | producer | replaces `map_literal_construction_not_modeled` with elaboration + three refusals |
| `v2.std.inhabitance` `DeclaredTypePosition` | vocabulary | map entry key/value positions |
| `v2.compiler.eval`, `v2.std.compilers.target_model`, emit | readers | realize the introduction (graph → finite container) |
| `v2.workflow.compile_door_cause_ownership` | ownership | one row per new refusal; `map_literal_construction_not_modeled` retired |
| `reference_conservation_accepted_drops` `a_map_literal_value_refuses_at_the_literal_holds` | control | flips to a positive control: `std/types`'s literal elaborates to exactly the introduction |
| #12711's `gunbc.rung_drop` (map population) | drop | retired by PR2; its trigger is this capability |

## PR2 controls

1. `dag/std/types` `kernel_type_set` elaborates to a Transform headed by
   `finitely_supported_function_from_graph` with eight `FiniteGraphEntry` children in authored
   order, compared by `content_hash` against the hand-built introduction.
2. A duplicate key refuses `infer_map_literal_duplicate_key` at the second occurrence.
3. `Map<Int, Bool>` with `{ "k": true }` refuses `infer_map_literal_key_type_mismatch` at `"k"`.
4. Mutation: dropping the duplicate check, so the graph reaches eval and `map_insert` last-wins,
   turns control 2 red. The control reads the infer outcome, not the evaluated map.
5. Evidence: a base-vs-head native-route census (recipe on #12550) showing `dag/std/types` and
   every other map-literal file admitted, with no unexplained new refusal.

## Open for review

- Home and name of the introduction. The proposal is `std.algebra`, beside the carrier. The
  alternative is `v2.std.collection`, but that is realization-adjacent and would invert the layer.
- Whether `FiniteGraphEntry` should be a general product (`std` has no `Pair`) rather than a
  map-specific record. Minting a general product only for this use would be premature, so the
  proposal is the specific record until a second consumer appears.

