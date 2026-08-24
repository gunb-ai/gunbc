# The predicate half of the Set/Map carrier fork, decomposed (2026-08-23)

**Work item:** `node://adhoc-66adece2-31f`. **Session:** `silent-gull-867`.
Measurement this rests on and does not restate:
[`t3_collection_realization_2026-08-22.md`](t3_collection_realization_2026-08-22.md) (the E0308 half)
and [`board_2026-08-23/README.md`](board_2026-08-23/README.md) (the retained log this brief's 27
blocks were partitioned from).

## The 27 blocks, at `.dag` grain

Subject `src/v2/compiler/03_ingest.dag`, ref `98b18cdc81e`, board `CARGO_ERROR_TOTAL=330`. The brief
names 27 blocks in two shapes; both are the same fork read from two positions.

| shape | code | blocks | what rustc saw |
|---|---|---:|---|
| the algebra name leaks as a Rust type | `E0560` | 17 | `struct Rc<PointwisePower<_>> has no field named member` (8) · `struct Rc<PartialFunction<_, _>> has no field named lookup` (9) |
| the carrier's own field read off the realized type | `E0609` | 10 | `no field member on type Rc<OrdSet<…>>` |

Both come from `std.types` naming one spelling for two carriers. A **type** position takes the
`container_template_alias_rows` hop on to the target inhabitant row and lowers `Set<T>` to
`im::OrdSet<T>`; a **record-literal** position takes the `container_template_algebra` hop and stops
at the modeled algebra struct, `Rc<PointwisePower<_>>`. Where one compiled closure holds both, the
two meet as a refusal.

## What this change does

Every corpus site that **constructs by characteristic function** or **reads `.member`** now names
`std.algebra` `PointwisePower` directly, under its own authority, instead of borrowing `Set`. The
populations were read individually, not sampled:

| | count |
|---|---:|
| `Set { member: … }` literals in `src/v2` before | 37 |
| — of them, characteristic functions | 37 |
| — of them, finite enumerations mis-spelled | 0 |
| `Set { … }` literals in `src/v2` after | 0 |
| `.member(` call sites re-pointed | 26 |
| `Set<…>` declarations re-typed to `PointwisePower<…>` | 128 occurrences across 20 files |

Nothing in the finite-`Set` population moved: `std.graph` `visited`, `std.authorization_profile`
`EnumeratedAudience.members`, `std.occurrence_binding_candidates`, `std.syllogism`,
`gunbc.package_delivery`, and the `dag/`-rooted emit templates keep `Set` and keep the `OrdSet`
realization, because every one of them is fed by `empty_set` / `set_insert` and read by
`set_contains` — the finite surface, never `member`.

This closes the 18 blocks of the 27 that are Set-shaped (8 `E0560` + all 10 `E0609`). The 9
remaining are `Map { lookup: … }`, and they are **not** closed here — see below.

## What is deliberately left open, and why

1. **The Map half.** `Map` needs no retarget — `PartialFunction`'s template roster already *is* the
   finite map surface — but its 145 corpus literals do need re-authoring onto the construction route
   `std.primitives` already declares (`empty_map_contract`, `map_insert_contract`; the second is
   bound in `v2.std.collection` `map_insert_host_binding` and then not used, the function writing a
   closure chain instead). 97 of those 145 are `lookup: <named fn>` delegates whose key list has to
   be read out of the delegate body one at a time. That is a lane, not a rider on this one.
2. **The alias itself.** `std.types` still reads `type Set<element> = PointwisePower<element>`, so
   `Set { member: … }` is still *writable* — nothing refuses a new one. Making it unwritable is a
   retarget of that row onto the finite carrier, which is a `std` authority change whose only
   verification is a whole-corpus compile, and which has to move together with the Map row. The
   obligation is recorded on the declaration itself (`dag/std/types.dag`, the annotation above
   `type Set`), with the next-rung trigger named there: the Map-side decomposition landing. Until
   then the class sits at **mitigatable** (DESIGN §4b) and only review sees a regression.

So the honest statement of the rung: this change removes the fork's instances on the Set side and
does not yet make the Set side's invalid state unwritable.

## Controls

- **The re-typed population is exactly the predicate population.** Two independent instruments
  agree: a grep for `Set {` literals (37, each read) and a grep for `.member(` consumers (26, each
  read). No site was found that constructs a `Set` literal and then enumerates it, and no finite-
  surface call (`set_contains` / `set_insert` / `empty_set`) appears in any re-typed declaration's
  module against a re-typed value.
- **The rewrite could not touch string data or prose.** The edit was applied outside double-quoted
  spans and outside `//` annotation lines; the two `"Set"` string literals in
  `v2.extdeps.languages.rust` (`target_collection_set_carrier` rows) are unchanged, and the two
  prose blocks that cite the grounding (`v2.std.grammar`'s sync-token note and
  `v2.compiler.02_parse`'s convergence-measure note) were updated by hand, not by the rewrite.
- **Not verified here:** the emitted-Rust block count. Re-measuring the board requires the probe
  dispatch, and this change is landed against the `.dag` acceptance path (`gunbc compile --entry
  src/v2/compiler/03_ingest.dag`) rather than against a re-run of `curated_cargo_probe_one.sh`. The
  claim "18 of 27 close" is derived from the site-to-literal correspondence above, and is a
  prediction about the next board, not a measurement of one.

## Addendum — the Map lane is not the shape the scoping doc predicted (full read, 2026-08-23)

`decomposition_scope.md` classified the 97 `lookup: <named fn>` delegates from a **sample** and
flagged that row as "the one number here that a full read could move." It moves. Read
mechanically over every delegate body in `src/v2` + `dag`:

| classification | count | test |
|---|---:|---|
| finite — branches on the key | **6** | the delegate body contains `<key> ==` or `match <key>` |
| finite one level down | **3** | a callee receiving the key does | 
| **open — no key comparison anywhere in two levels** | **78** | computes a fact from the key unconditionally |

92 delegate names, 87 bodies resolved. The sample that produced the "if-chain over a fixed key list"
reading landed on `v2.program` `program_facts_lookup`, which is one of the 6.

The overwhelming majority are shaped like

```
fn bind_facts_lookup(key: Node) -> Optional<InferredFacts> {
  optional_present(value: claim_inferred_facts_from_nodes(resolved: key, ..))
}
```

— a **total function of the key**, not a table. So most of the Map population is a genuine partial
function rather than a finite table, and 145 re-authorings into finite maps is the wrong lane.

**But it is NOT the symmetric move, and an earlier revision of this addendum said it was.** That
revision read "they should name `PartialFunction` under its own authority, exactly the move this
change made on the Set side." Corrected here rather than left standing, because it is the premise a
reader would plan the Map lane against. `PointwisePower` is **one** template row (`member`), which is
why an open characteristic function inhabits it completely and the Set change is clean.
`PartialFunction` is **thirteen** (`get map_get lookup map_insert map_merge map_has map_contains_key
map_keys map_values with contains length count`) — it bundles a partial function with a finite map,
and an open delegate cannot answer `map_keys`, `map_values` or `count` at all.

The asymmetry is visible in the model's own profile rosters, and it is the Map side that is missing a
carrier, not the Set side that has a spare:

| concept | Set side | Map side |
|---|---|---|
| open / characteristic | `PointwisePower` — 1 template | **missing** |
| finite / enumerable | `BooleanAlgebra` collection — 13 templates | `PartialFunction` — 13 templates |

Inhabitance measured, not argued: **0 of the 145** `Map { … }` literals in `src/v2` + `dag` supply any
field beyond `lookup`. The other twelve rows are promises the entire existing population already
fails to keep. So the Map lane's first move is a **decomposition** of `PartialFunction` — the partial
function keeps its real meaning (`lookup: fn(K) -> V?`, mirroring the single-slot
`v2.std.collection` `TotalMap<K, V> { lookup: fn(K) -> V }` already in tree, so this is the totality
axis on an existing pair rather than a minted name) and the finite map takes its own name — not a
retype. That is a `std` authority change of the same class as the alias row deferred above, so it
moves in its own PR verified by whole-corpus compile, and **both** alias retargets can ride it.

**What this does and does not establish.** The classifier is a text predicate over the body and one
level of callees, so it is a *lower bound on openness*: a delegate could still be finite through a
deeper helper. The 9 finite ones are confirmed by reading them; the 78 are "no key comparison found
within two levels", and the classification converged rather than drifted (going from one level to
two moved 3 of 81), which is evidence it is near-stable but not a proof. The remaining 5 unresolved
names are declared here rather than absorbed into either column.
