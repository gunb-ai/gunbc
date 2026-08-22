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

## The collision population, and why the grain is the pool rule

The refusal a re-key would arm fires when two declarations share a spelling **in one folded
population**. That population is not one thing — the v1 emitter builds it three different ways, and
the count differs by an order of magnitude between them. So every row names its pool rule.

- **Rule 1 — import-only union of a root.** Every `.dag` under `src/v1` seeded at once, imports
  followed through a module index over `src/v1` and `dag`: `v1_compiler.cli_run`
  `regen_input_sources`, the population the seed emits itself from. Exact here: no import in that
  population names a module outside the index, so nothing is silently dropped.
- **Rule 2 — single entry, imports plus a reference extension.**
  `load_sources_for_entry_with_index` resolves the import closure and then calls
  `extend_with_reference_closure`, which scans each source for **dotted module paths outside strings
  and annotations** and pulls in each named module's own import closure. An import-edge-only walk
  understates this: `src/v2/compiler/00_compile.dag` is 152 modules / 23 collisions by imports
  alone, **161 / 25** once the extension is applied.
- **Rule 3 — whole-tree union** over the three roots. The upper bound.

**A second rule-2 ambiguity, real in principle and measured at zero here.** Both this walk and the
host scanner it mirrors accept a dotted chain whose prefix resolves in the module index — and
`binder.field` is spelled exactly like `module.member`. (A sibling lane's instrument returned 1,350
sites of which essentially all were field accesses; the true population was 8.) Measured on the live
tree: every module path begins with one of 16 root segments, and the count of sites where a **local
binder** shares one of those roots *and* the two-segment path it forms is a real module is **one** —
`tools.readme`, inside `dag/tools/readme.dag`, a module naming itself. So the rule-2 counts are not
inflated by field accesses today. Not structural: a future parameter named `config` or `std` beside a
matching two-segment module re-opens it silently, which is why the number is carried rather than the
reassurance.

**Rule 2 is not as objective as the other two, and a reader will assume it is.** Its population is a
function of which dotted references the *source* spells, so it inherits any mis-qualification
upstream of it. Receipt, measured by another lane the same day: four dotted references in
`src/v1/05_emit_rust.dag` (two `v2.std.node.Arrow`, two `v2.std.node.Bind`) had been qualified to the
wrong declarer by a repair pass — main spells all four **bare**, and `v1.std.core` declares both
names — and those four references alone pulled `v2.std.node` into the seed pool. Rules 1 and 3
cannot move that way: rule 1 follows declared import edges, rule 3 takes everything. A rule-2 count
measures the tree *and* the correctness of its qualifications, inseparably.

| pool | modules | type decls | colliding names |
|---|---:|---:|---:|
| rule 1 — stage0 regen (the seed emitting itself) | 128 | 974 | **1** |
| rule 2 — `src/v2/compiler/00_compile.dag` | 161 | 1182 | **25** |
| rule 2 — `src/v2/workflow/required_floor.dag` | 45 | 430 | **20** |
| rule 2 — `dag/gunbc/design_document.dag` | 127 | 481 | 3 |
| rule 3 — whole tree (`dag`, `src/v1`, `src/v2`) | — | 9480 | 116 |

The whole-tree row: 9480 declarations, 9352 distinct names, **116 declared more than once**, and
both grains agree — every repeated name is also declared by more than one module. (An earlier
revision of this file reported 97 over `dag` and `src` at file grain; that figure is superseded
rather than kept beside this one.)

**Instrument control.** The compiler's own resolver reports `resolved 6 sources` for this probe's
carrier entry; the walk independently counts 6 modules for it under both rule 1 and rule 2.

**The one collision in the seed's own emit population is live and non-benign.** `Cardinality` is
`Required | Optional` in `std.constructors` and `Required | CardOptional` in `v1.std.core` — two
different unit-only enums under one key. The map's `variant_name_set` for `Cardinality` is whichever
module the fold reached last, and `is_known_variant`, `derive_variant_to_enum` and
`variant_belongs_to_enum` all answer from that one.

**The twenty-plus in every v2 pool are the dual tree, not accidents.** `Int`, `Nat`, `List`, `Bool`,
`Unit`, `TerminationProof`, `RankingDimension`, `Compose`, the ten machine-width integer rows, and
(only via the reference extension) `NonNegativeInt` and `PositiveInt`. DESIGN §3 already names this
pair as the specimen where a bare-name rule realizes the wrong declaration. They are refused as a
body or not at all, which puts the refusal downstream of the two-tree migration.

### A second specimen, and a second failure mode: the miss, with no collision in it

`Connective` is a **wrong** answer from two declarations collapsing into one entry. This one is a
**missing** answer from a key that was never going to match — no homonym involved. The map's
exposure is therefore not exclusively about collisions.

Path (`emit_typed_record_lit`, this roster's row): `tn_is_known_struct` is
`map_contains_key(emit_info.type_summaries, tn)`, the map is keyed bare, the namespace-cut corpus
spells every construction qualified — so the test misses, `ctor_name` takes the resolve fallback, and
the phantom lookup answers `Absent`, **whose arm emits nothing rather than refusing**. 13 rustc
E0063 `missing field _phantom` at that head: emitted structs declaring the field beside emitted
literals that never set it.

**The two-arm control, which that lane could not build (with imports deleted, a bare cross-module
spelling does not resolve at all) — main is the other arm.** `TargetCapabilityShapeRow` on main: 8
construction sites, **all bare**, zero qualified; its committed mirror
`extdeps_languages_rust_capabilities.rs` carries 8 emitted literals and 9 `_phantom` lines (8 rows +
the enclosing table). Their branch: same type, 8 sites, **all qualified**, 8 E0063. Same type, same
emitter path, opposite spelling, opposite outcome.

**Population, in two steps, because the miss is far wider than the diagnostic.** Every qualified
record literal misses the bare membership test — **1341 sites over 101 types on main today**. Only a
*phantom-bearing* type turns that miss into E0063, and main has 36 such struct types in its mirror,
of which **zero** are ever constructed qualified. Main is population-conditionally correct here on a
*second* unenforced premise — everything is spelled bare — which the cut branch violated for six
types.

**One false positive of my own, recorded because it is this carrier's subject turned on its author.**
My first intersection joined those sets on the **leaf name** and returned one exposed type, `Group`.
It isn't: the phantom-bearing one is `std.algebra.Group`, the qualified construction is
`std.render.Group` — a different declaration sharing a leaf. Joining on a spelling produced a wrong
answer inside a measurement about joining on a spelling. Corrected count: zero.

### The `Connective` specimen: pool named, and it is a third root

`Connective` is declared twice — `v1.std.core` (`src/v1/00_core.dag`) and `v2.std.node`
(`src/v2/std/node.dag`). Wherever both are folded, bare keying puts them in one entry and
`derive_variant_to_enum`'s ambiguity wall cannot fire: **a fail-closed guard downstream of a lossy
index is not fail-closed** — it is structurally unable to observe its own trigger and reads as
coverage on the ledger, which is worse than an absent guard, because an absent guard ranks for
building. `is_known_variant` is the same inversion from the other side.

The pool is `regen_source_roots` on `integration/namespace-cut` (head `48b55fa2d70`, PR #8282),
which returns **three** roots — `src/v1`, `dag` **and** `src/v2` — where main returns two. Verified
by reading that branch: the root list carries `src/v2`, and 36 stage0 mirrors there reference
`crate::v2_std_node`. So rule 3 *is* the regen pool on that branch, and "co-resident in no pool" was
correct for every pool reachable from main and wrong for the branch the artifact lives on.

Measured on that branch's tree, seeding every `src/v1` module:

| walk | modules | type decls | colliding |
|---|---:|---:|---:|
| imports only | 54 | 410 | **0** |
| imports + reference extension | 126 | 990 | **3** — `Cardinality`, `Connective`, `Node` |

The imports are deleted on that branch, so the import walk has nothing left to walk: there, the
reference extension is not a refinement, it is the whole closure. Exactly one v2 module enters the
pool — `v2.std.node` — and it alone brings two of the three collisions.

**`Node` is the third name, and it closes a gap the tracing lane left open.** `Node` is declared in
both `v1.std.core` and `v2.std.node`, so it collapses exactly as `Connective` does; it was filed
there as entering by an unidentified third path. It is the same path. `Edge` and
`NamedEdgeTargetLookup` are declared **only** in `v2.std.node`, do not collide, and remain
unaccounted for — one emitted use-line, at least two causes.

**The generalisation is worth more than the specimen: the folded population is a function of the
root list, not of the module graph.** Adding one root to a `Vec` changed the emitter's identity
answers, and the doc comment beside that root list still describes the two-root closure. A map keyed
on identity would be indifferent to which roots were passed; a map keyed on a spelling is not.

**One account corrected rather than carried.** Main is not silent about `Connective` because
something suppressed the synthesis: main's own `v1_compiler_infer.rs` emits `use
crate::v1_std_core::Connective::{Arrow, Conj, Disj, NoConnective}` — the path runs and answers
correctly, because only one `Connective` is in main's pool. The deficit did not exist until the pool
admitted the homonym.

### `build_shared_types`: the scan row with a production receipt

`shared_types` is folded *from* this map and inherits its bare key, then escapes as a `Set<String>`
tested with `set_contains` at **27 sites on main — only 6 of which pass an already-leaf-reduced
expression** (counted here). On `integration/namespace-cut` (head `45fb8c80b8e`), where those names
are spelled dotted, a dotted spelling missed the bare-keyed set: the type rendered **unwrapped**
there while a caller reaching the same type through a leaf-reduced site rendered it **wrapped** —
the emitted tree disagreeing with itself about one type in both directions.

| `source_indices` | Rc-wrapped | unwrapped |
|---|---:|---:|
| origin/main | 717 | 0 |
| before | 343 | 371 |
| after | 714 | 0 |

(Those measurements are the namespace-cut lane's, cited not re-derived.) The split collapsing to zero
*in the direction main already holds* is the discriminator — a fix that merely moved the
disagreement would not land on main's shape.

**The receipt survived its own confound, which is why it is quoted here.** Those trees were emitted
by a **bootstrap binary of an older vintage** — its mirrors contain no `type_reference_decl_file`,
`decl_identity_file` or `numeric_realization_declaring_modules` at all — and an older emitter is a
*main-like* emitter, whose wrapping is consistent by construction. "The old binary produced the
consistency" was therefore a live alternative explanation for the 714/0. The lane ran the control:
same binary vintage, same tree, same roots, the leaf-reduction the only difference — **with** it
714/0, **without** it 343/371, the original split reproduced exactly. The attribution holds because
of that arm, not because the after-number resembles main's.

The same vintage confound retired that lane's separate `Nat`-versus-`i64` realization diagnosis
(measured with a binary lacking the identity machinery the diagnosis is about). Nothing on this
roster rests on it. And the mirrors at that head are **provisional** — emitted by the
vintage-behind binary — so only the controlled `shared_types` delta is claimed from them. It belongs on this roster because the failure is at the
**consumption of a name the scan returned**, not at any lookup into the map: re-keying the map's
keys cannot reach it.

## The two-part guard test, and why the check being well written is the wrong question

A guard deserves system-level credit only if **both** hold:

- **trigger preservation** — every raw state that *should* trigger it stays distinguishable in the
  guard's **input**;
- **refusal** — it refuses every trigger it can distinguish.

`derive_variant_to_enum` satisfies the second and fails the first, and that is why it survived
review: nothing about the fold's text is wrong. Its input arrived through a map that had already
merged the two declarations by spelling, so the state that should have fired the wall was gone
before the wall could look.

| guard | trigger preserved? | refuses? | return type can express the answer? |
|---|---|---|---|
| `derive_variant_to_enum` | no — merged at `add_emit_item_summary` | yes — empty-string sentinel | yes |
| `is_known_variant` | no — and it quantifies, so it inverts under load | no — plain `Bool` | **no** |
| `variant_belongs_to_enum` | no | no — `false` conflates absent with wrong provider | **no** |
| `reference_derived_variant_induced_parent_spelled` | no — via `variant_name_set` | yes | yes |
| `type_has_fn_fields` | no | no | **no** |
| `is_optional_struct_field` | no | no | **no** |
| `build_qualified_item_registry` | **yes** — qualified `module.leaf` | **yes** — define-and-consume marker | yes |

Read it as two different repairs: a row failing trigger preservation needs a **faithful input**; a
row failing refusal needs a **better check**. One "broken" column would have merged them.

**`is_known_variant` is not fixed by fixing the map.** Its return type is the other half: even with
a faithful index, a `Bool` cannot distinguish *uniquely known* from *ambiguously known*, so the
caller cannot refuse what it cannot be told. The same holds for `variant_belongs_to_enum`,
`type_has_fn_fields` and `is_optional_struct_field`, whose `false` arm already conflates absent with
wrong-provider.

## Population state, because neither "passing" nor "failing" fits

- **PopulationIndependentCorrectness** — correct even when homonyms enter.
- **PopulationConditionalCorrectness** — correct only while an unenforced uniqueness premise holds.
- **TriggeredWrongAnswer** — the population violates the premise and a consumer sees it.

The bare-keyed map is in the **second** state on main and the **third** on `integration/namespace-cut`.
"Main is correct and the branch broke it" understates the defect; "the deficit existed but was
masked on main" overstates it — nothing was masking anything, the mechanism returned the right
answer for the input it had. The structural defect existed; the wrong output did not, until the
collision population arrived. Measured cost of moving between the two states: **one element appended
to a `Vec`**.

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
