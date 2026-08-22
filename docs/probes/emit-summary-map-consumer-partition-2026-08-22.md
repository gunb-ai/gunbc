# The emit summary map: what its consumers hold at the ask, and what they do with what they get (2026-08-22)

**Session:** `crisp-bat-769`. **Work item:** `node://adhoc-0ca3aae1-8b1`.
**Carrier:** `gunbc.emit_summary_map_consumer_partition` — the partition is the deliverable and lives
there; this file is the method, so a reader can re-run the sweep rather than trust the rows.
**Nothing is repaired.** The sweep is a precondition for a refusal, never a follow-up to it.

## The subject

`v1.compiler.infer_emit_info` `build_emit_graph_info` folds every module in the closure into one
`Map<String, TypeSummary>` through `add_emit_item_summary`, keyed on the **bare authored declaration
name**. There is no collision arm: two declarations sharing a spelling resolve last-write-wins in
fold order. `gunbc.bare_name_identity_consumer_census` ranks this first because three of its other
rows are computed *from* this map and inherit its key.

## The question, and why it is two questions

The handover asked whether consumers can supply identity **at the ask**. Scoring only that is
insufficient: a consumer may hold a resolved node when it queries the map and then re-key on a
spelling drawn *out of the summary it just received*. Such a site is repaired by an identity-keyed
map on paper and unrepaired in fact. Every row therefore carries two columns — the ask and the use.

## Method, stated as a selector so its blind spot is visible

1. Every expression in the v1 compiler reading the map: `map_get` / `map_contains_key` against
   `type_summaries`, every call of `lookup_emit_type_summary`, and every `map_values` / `map_keys`
   scan of it. Grouped by enclosing top-level `fn`.
2. For each row: read the body, and follow the value the lookup returns until it is either consumed
   structurally or used as a key again.
3. For each row scored spelling-only: find the fold that severed the identity and record it.

Reproduce the selector with a grep over `src/v1/04_emit_info.dag`, `src/v1/04_infer.dag`,
`src/v1/05_emit_rust.dag` for the three read forms, resolving the enclosing `fn` for each hit.

**Known blind spot, named rather than left silent.** A consumer of a map *derived* from this one —
`variant_to_enum`, `shared_types`, `fielded_variants`, `positional_payload_variants` — inherits the
bare key without ever touching `type_summaries`, and is outside the selector. Those consumers are
visible here only indirectly, through the escape arm of rows that feed them.

## The measurement

23 deciding functions; 32 read expressions.

**The ask column.**

| ask | rows |
|---|---:|
| holds a Node at the lookup (identity derivable there) | 6 |
| composite of two spellings (enum + variant) | 4 |
| spelling only, pulled out of a `TypeSummary` field | 8 |
| no key at all — whole-map scan | 5 |

**The use column, which is where the sizing changes.**

| use | rows |
|---|---:|
| terminal — a Bool or a structural consumption, no spelling leaves | 5 |
| re-keyed into the same map on a value from the summary | 9 |
| a spelling escapes to the caller and is decided on there | 8 |
| re-keyed into a foreign name table with no identity parameter | 1 |

**The cell that decides the sizing: the two columns do not overlap at all.** Of the 6 rows that hold
a node at the ask, **none is terminal** — `enum_variant_shape_sets_for_item`,
`anonymous_record_lit_surface_name`, `collect_anonymous_record_lit_heads`,
`module_data_field_struct_import_names`, `emit_field_value_with_context` and `emit_typed_record_lit`
all query correctly and then decide on a spelling anyway. And of the 5 terminal rows, **none holds a
node**: `variant_belongs_to_enum` (composite), `is_enum_in_summaries`, `type_has_fn_fields` and
`is_optional_struct_field` (spelling only), `is_known_variant` (whole-map scan). Scoring the ask
alone would have filed the first six as repaired by a re-key; scoring the use alone would have filed
the last five the same way.

## Three findings

**1. The severance is not one fold. It is four, and all four hold the node.** The handover named
`build_field_type_map`, whose arm binds `Resolved { node: ft }` and stores
`authored_name_at(...)` — identity in hand at the line it is discarded. Three more do the same:
`collect_type_node_import_surface_names` (fills `field_import_surface_names`, and returns a
`List<String>`, so carrying identity there changes the element type, not just a field),
`build_type_summary` (fills `variant_name_set` from each variant declaration node), and
`add_emit_item_summary` itself (the map key). Good news and bad news together: the repair is still
upstream and shared rather than per-consumer, but it is four folds and three of the summary's own
fields, not one.

**2. Five rows have no key at the ask at all.** `derive_variant_to_enum`, `is_known_variant`,
`close_fn_fields`, `build_shared_types` and `struct_candidates_by_field_names` scan the whole map.
Re-keying its *keys* does not reach them: their input is every value and their **output is a name**
(`build_shared_types` returns a `Set<String>` later tested with `set_contains` at every render
site; `struct_candidates_by_field_names` returns `summary.name` as an emitted struct head).
`is_known_variant` is the sharpest case — it deliberately quantifies over the whole closure, so the
more collisions the map holds, the more often it answers `true`.

**3. One row leaves the map entirely.** `emit_typed_record_lit` takes a field type spelling out of
`field_type_map` and passes it to `rust_zero_value`, a name-keyed table with no identity parameter.
An identity-keyed summary map changes nothing at that hop until the table is keyed too. This is the
`emit_container` shape: keying on identity is not merely absent there, it is impossible without
changing the signature.

## The collision population, measured before any refusal

The refusal a re-key would arm fires when two declarations **in one emitted closure** share a
spelling. Corpus-wide upper bound, measured 2026-08-22 over every module-scope `type NAME`
declaration under `dag/` and `src/`: **9474 declarations, 97 distinct names declared in two or more
distinct files** (116 before same-file duplicates are removed). Specimens:
`String` (`dag/std/string_type.dag`, `src/v2/std/text.dag`), `UriScheme` (`dag/extdeps/uri.dag`,
`src/v2/std/network.dag`), `Token` (`src/v1/00_core.dag`, `src/v2/std/compilers/lexing.dag`).

**This is an upper bound and not the refusal population**, because no closure contains every file;
and it is not a lower bound either, because it counts `type` declarations only and says nothing
about the composite variant keys. The closure-grain number requires executing
`build_emit_graph_info` over a real entry, and that measurement is what must precede landing a
refusal.

## What this means for the repair, stated as a size rather than a plan

Re-keying the map alone repairs **0 of 23 rows** — the intersection of "identity obtainable at the
ask" and "nothing spelling-shaped leaves the site" is empty. Every row needs one of: identity
carried in the summary's *values* (the 9 re-key rows and the 8 escape rows), an identity-bearing
return type for the scans (5 rows, overlapping), an identity parameter on the accessor and the
predicates that take a bare `String` (the 8 spelling-only rows), or a second authority keyed at all
(1 row). That is a larger job
than "re-key the map", and it is a different job from "carry identity forward to the blocked
consumers" — the identity does not need threading to 23 sites, it needs to stop being projected out
at 4 folds and to survive in what the summary carries.

## Rung

The class is unchanged and sits at *mitigatable*: nothing refuses on a collision anywhere on the
read side. The carrier is a hand sweep of a mechanical selector, so a newly authored lookup moves no
number in it and nothing detects the omission. What the carrier holds by construction: an ask cannot
be scored node-holding without naming the expression that holds the node, a spelling-only ask cannot
be filed without naming the fold that severed it (joined against the severance roster by a witness),
and the use column has no arm meaning "not followed".
